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

## Replayed Requests: the Server Refuses Them

Encryption proves that the sender holds a key. It does not stop someone from capturing a body on the way and sending it again: the copy decrypts exactly like the original. So once a body has opened, before the action runs (and before a compressed body is decompressed), the server checks it:

1. **The clock of the sender.** A body whose `ts` is more than **10 seconds** off the server's clock, **in the past or in the future**, is refused.
2. **The salt.** A body whose salt was already accepted is refused. Check and remember are one step under one lock, so of two copies that arrive at once, one passes.

Both refusals are `403`, and the text says which one and, for the clock, by how much:

```text
Refused the encrypted request body (Content-Encoding: aes128gcm): the sender's clock (its timestamp) is 14.2s off the server's clock, and at most 10s either way is accepted
Refused the encrypted request body (Content-Encoding: aes128gcm): a body with this salt was already accepted. A body is accepted once: a retry encrypts it again
```

**How long a salt is kept.** A salt is kept with its `ts` until `ts + 10 s` is behind the server's clock. From then on a copy fails the clock check by itself, so there is no moment when a copy passes. Every encrypted request first forgets the pairs that have run out, so what is kept is the bodies accepted in the last 20 seconds at most. A plain request does not touch any of this.

**The window** is set on the keys:

```rust
let keys = Arc::new(
    BodyEncryptionKeys::new(vec![key]).with_max_clock_skew(Duration::from_secs(30)),
);
```

A wider window tolerates clients with worse clocks, and keeps every salt that much longer. Both clocks are wall clocks: keep the clients and the server synchronised (NTP).

What is left to the action:

- **Taking encrypted bodies only.** `get_encryption_info()` is `None` for a plain body and for an empty one, which carries no salt and no clock to check. An action that changes something answers those `415`:

```rust
let Some(info) = ctx.request.get_encryption_info() else {
    return Err(HttpFailResult::from((415u16, "The body has to come encrypted".to_string())));
};

// ... apply the change: the body is fresh and seen once ...

HttpOutput::as_json(result).encode(&info.key).into_ok_result(false)
```

- **Retries.** A retry that should go through encrypts the body again: a fresh `ts` and a fresh salt. Resending the same bytes is a replay, and it is refused. fl-url resends the bytes it built (and replays only idempotent methods), so a write that has to be retried is a new call.
- **Limits.** The salts are kept by the `BodyEncryptionKeys`: every server of the process it is handed to shares them (a TCP port and a unix socket of the same service, say). Behind several instances, a copy sent to another instance passes, so these calls go to one instance. A restart forgets the salts: a body accepted just before the restart can be sent again for the rest of its window.
- **Streamed bodies** (`#[http_body_as_stream]`) are handed on still encrypted, so the server checks nothing: an action that decrypts one itself checks it itself.
