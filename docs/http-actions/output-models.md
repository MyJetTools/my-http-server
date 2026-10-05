# Output Models

**⚠️ CRITICAL: When Using `model:` in Result Field**

When you specify `model: YourResponseModel` in the `result:` field of the `http_route` macro, the response model **MUST** derive `MyHttpObjectStructure` in addition to `Serialize` and `Deserialize`.

**Required Pattern:**
```rust
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, MyHttpObjectStructure)]
pub struct YourResponseModel {
    pub field1: String,
    pub field2: i32,
}
```

**In the http_route macro:**
```rust
#[http_route(
    // ... other fields ...
    result: [
        {status_code: 200, description: "Ok response", model: YourResponseModel},
    ]
)]
```

**Why This Is Required:**

The `model:` field in the result tells the framework to generate Swagger/OpenAPI documentation for your response model. The framework needs `MyHttpObjectStructure` to introspect the model structure and generate the proper schema documentation.

**What Happens If You Forget:**

If you use `model:` without deriving `MyHttpObjectStructure`, you will get a compilation error:

```
error[E0599]: no function or associated item named `get_http_data_structure` found for struct `YourResponseModel` in the current scope
```

**Solution:** Add `MyHttpObjectStructure` to your response model's derive macro.

**When You DON'T Need `MyHttpObjectStructure`:**

If you don't use `model:` in the result field, you can omit `MyHttpObjectStructure`:

```rust
// Simple response without model specification
#[derive(Serialize)]
struct SimpleResponse {
    message: String,
}

// In http_route macro - no model: field
result: [
    {status_code: 200, description: "Ok response"},
]

// In handler - still works fine
HttpOutput::as_json(response).into_ok_result(false)
```

**Complete Example:**

```rust
use serde::{Deserialize, Serialize};

// Response model with MyHttpObjectStructure (required when using model:)
#[derive(Serialize, Deserialize, MyHttpObjectStructure)]
pub struct CertificateInfoHttpModel {
    pub cn: String,
    pub expires: String,
}

#[http_route(
    method: "GET",
    route: "/api/certificates/info",
    input_data: GetCertInfoInput,
    controller: "Certificates",
    summary: "Get certificate information",
    description: "Returns certificate details",
    result: [
        {status_code: 200, description: "Certificate info", model: CertificateInfoHttpModel},
    ]
)]
pub struct GetCertInfoAction {
    // ...
}
```

**Summary:**

| Scenario | Required Derives |
|----------|-----------------|
| Using `model:` in result | `Serialize`, `Deserialize`, `MyHttpObjectStructure` |
| Not using `model:` in result | `Serialize` (or `Serialize, Deserialize` if needed) |
