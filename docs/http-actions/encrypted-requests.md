# Encrypted Requests

The server decrypts a request body sent with `Content-Encoding: aes128gcm` (RFC 8188) before the action runs, using the keys given to `set_body_encryption_keys` (see [registration-and-startup](registration-and-startup.md)).

## The Request on the Wire

The body is my-http-utils' `EncryptedBody` (`my_http_server::EncryptedBody`), the standard fl-url and the services behind my-reverse-proxy share. What is encrypted is the sender's clock, then the body:

```text
Content-Encoding: aes128gcm

body on the wire = aes128gcm( ts || the body )
ts               = the sender's clock: unix microseconds, an i64 in 8 bytes, little-endian
```

The server takes `ts` off and hands the action the rest, so an input model parses the body as if it had come in plain, whatever its type: JSON, YAML, protobuf, raw.

A client does not put the format together by hand:

```rust
// fl-url (the `encryption` feature): the body is encrypted, Content-Encoding is set
FlUrl::new(url).encrypted(&key).post_json(&request).await?;

// my-http-utils: the same bytes, for any transport
let body = HttpRequestBody::as_json(&request).encrypt(&key)?;      // Content-Encoding: aes128gcm
let body = HttpRequestBody::as_json(&request).compress().encrypt(&key)?; // gzip, aes128gcm

// or the body type itself
let bytes = EncryptedBody {
    salt: EncryptedBody::random_salt(),
    timestamp: DateTimeAsMicroseconds::now(),
    body: json.into_bytes(),
}
.encrypt(&key);
```

`key` is an `HttpBodyEncryptionKey`; a `BodyEncryptionKey` of the server's list derefs to one.

- **Compressed, then encrypted.** `Content-Encoding: gzip, aes128gcm` (any one of `gzip`, `deflate`, `br`, `zstd` before `aes128gcm`): the server decrypts, then decompresses, under the same size limit as any compressed body. Two compressions, or `aes128gcm` anywhere but last, are answered `400`.
- **An empty body** is empty bytes to the standard, encrypted or not: there is nothing in it to encrypt, so a client sends it without `Content-Encoding`. One that comes with the header anyway reaches the action as an empty body with no encryption info. It proves nothing about who sent it.
- **`400`** when no key opens the body, when it is damaged or cut short, or when what opened is shorter than the 8 bytes of `ts`. The text says which: `Can not read the encrypted request body (Content-Encoding: aes128gcm): the body opens but is shorter than the 8-byte timestamp it starts with`.
- **`415`** with `Accept-Encoding` while the server has no keys.

Only requests carry `ts`. An answer encrypted with `encode` is plain aes128gcm and holds exactly what the action wrote; the client opens it with `key.decrypt(&body)` (see [responses](responses.md)).

## What the Action Reads

```rust
let info = ctx.request.get_encryption_info(); // None: the body came in plain
```

| `EncryptionInfo` field | What it is |
|---|---|
| `key` | The key from the server's list that opened the body. `.encode(&info.key)` answers with it. |
| `timestamp` | `ts`, the sender's clock, as a `DateTimeAsMicroseconds`. |
| `salt` | 16 random bytes, fresh in every body. Not secret: they are the first bytes of the body. |
| `record_size`, `header_key_id` | The rest of the body's header. |

The info is filled in when the body is read. An action whose input model reads the body gets it already read.

## Best Practice: Refuse Replayed Requests

Encryption proves that the sender holds a key. It does not stop someone from capturing a body on the way and sending it again: the copy decrypts exactly like the original. A replayed read does no harm, because its answer is encrypted with the key. A replayed write applies the change again. So an action that changes something does three things:

1. **Takes encrypted bodies only.** `get_encryption_info()` is `None` for a plain body: answer `415`.
2. **Checks the clock of the sender.** Refuse a body whose `timestamp` is more than **10 seconds** off the server's clock, **in the past or in the future**.
3. **Remembers the salt.** Keep the salt of every accepted body for **20 seconds**, and refuse a body whose salt is already kept. Check and remember in one step under one lock, so that two copies arriving at once do not both pass.

Why these numbers: a body passes the clock check only while its `timestamp` is within 10 seconds of the server's clock, which is 20 seconds at most. Its salt is kept for 20 seconds from the moment the body was first accepted. While its `timestamp` would still let a copy through, the salt refuses it; after that, the `timestamp` does. So the salt must be kept for twice the time window, and the window must be bounded on both sides: a `timestamp` from the future would keep a body passing after its salt is forgotten.

```rust
use std::collections::{hash_map::Entry, HashMap};
use std::sync::Mutex;
use std::time::{Duration, Instant};

pub const TS_WINDOW: Duration = Duration::from_secs(10);
const SALT_MEMORY: Duration = Duration::from_secs(20); // twice TS_WINDOW

/// The salts of the bodies accepted in the last 20 seconds. One per process, kept in AppContext.
#[derive(Default)]
pub struct AcceptedSalts {
    salts: Mutex<HashMap<[u8; 16], Instant>>,
}

impl AcceptedSalts {
    /// `true` the first time a salt comes within `SALT_MEMORY`, `false` for every copy after it.
    pub fn accept(&self, salt: [u8; 16]) -> bool {
        let now = Instant::now();
        let mut salts = self.salts.lock().unwrap();

        // Fine for the traffic of an admin action; a busy one sweeps on a timer instead.
        salts.retain(|_, accepted| now.duration_since(*accepted) < SALT_MEMORY);

        match salts.entry(salt) {
            Entry::Occupied(_) => false,
            Entry::Vacant(entry) => {
                entry.insert(now);
                true
            }
        }
    }
}
```

In the action:

```rust
async fn handle_request(
    action: &SetSettingsAction,
    input_data: SetSettingsHttpInput,
    ctx: &mut HttpContext,
) -> Result<HttpOkResult, HttpFailResult> {
    let Some(info) = ctx.request.get_encryption_info() else {
        return Err(HttpFailResult::from((415u16, "The body has to come encrypted".to_string())));
    };

    // `abs_diff`: a timestamp of i64::MIN must be refused, not overflow.
    let off = Duration::from_micros(
        DateTimeAsMicroseconds::now()
            .unix_microseconds
            .abs_diff(info.timestamp.unix_microseconds),
    );

    if off > TS_WINDOW {
        return Err(HttpFailResult::as_forbidden(Some(format!(
            "The clock of the request is {:?} off the server's clock: at most {:?} is accepted",
            off, TS_WINDOW
        ))));
    }

    if !action.app.accepted_salts.accept(info.salt) {
        return Err(HttpFailResult::as_forbidden(Some("This body was already accepted")));
    }

    // ... apply the change ...

    HttpOutput::as_json(result).encode(&info.key).into_ok_result(false)
}
```

- **Order.** Check the clock first, then the salt. A body refused for its clock is never remembered.
- **Status.** `403` with the reason. The clock error says how far off the clocks are: a client whose clock drifts more than 10 seconds is refused on every call, and the text tells it why. Both sides need synchronised clocks (NTP).
- **Retries.** A retry that should go through encrypts the body again: a fresh `ts` and a fresh salt. Resending the same bytes is a replay, and it is refused. fl-url resends the bytes it built (and replays only idempotent methods), so a write that has to be retried is a new call.
- **Limits.** The salts live in the process. Behind several instances, a copy sent to another instance passes, so these calls go to one instance, or the salts go to a shared store. A restart forgets the salts: a body accepted just before the restart can be sent again for the rest of its time window.
