# Troubleshooting

## Common Compilation Errors

### Error: `no function or associated item named 'get_http_data_structure' found for struct`

**Cause:** You're using `model: YourModel` in the `result:` field of `http_route` macro, but `YourModel` doesn't derive `MyHttpObjectStructure`.

**Solution:** Add `MyHttpObjectStructure` to your response model's derive macro:

```rust
// Before (incorrect)
#[derive(Serialize, Deserialize)]
struct MyResponse {
    field: String,
}

// After (correct)
#[derive(Serialize, Deserialize, MyHttpObjectStructure)]
struct MyResponse {
    field: String,
}
```

**Alternative Solution:** If you don't need Swagger documentation for the response model, remove `model:` from the result field:

```rust
// In http_route macro
result: [
    {status_code: 200, description: "Ok response"},  // No model: field
]
```

### Error: Missing comma in `http_route` macro

**Cause:** Missing comma between fields in the `http_route` macro attributes.

**Solution:** Ensure all fields are separated by commas:

```rust
// Incorrect
controller: "ControllerName"
summary: "Summary",

// Correct
controller: "ControllerName",
summary: "Summary",
```

### Error: `MyHttpObjectStructure` not found

**Cause:** The `MyHttpObjectStructure` trait is not in scope.

**Solution:** Ensure you have `use my_http_server::macros::*;` at the top of your file, which brings `MyHttpObjectStructure` into scope.
