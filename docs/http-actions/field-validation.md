# Field Validation, Custom Input Fields, Debugging Input Models

## Field Validation

You can add custom validators to input fields. Validators can access the HTTP context for more complex validation:

```rust
#[derive(MyHttpInput)]
pub struct CreateUserInputModel {
    #[http_body(
        name = "email", 
        description = "Email address",
        validator = "validate_email"
    )]
    pub email: String,
}

// Simple validator (value only)
fn validate_email(value: &str) -> Result<(), String> {
    if value.contains('@') {
        Ok(())
    } else {
        Err("Invalid email format".to_string())
    }
}

// Validator with HTTP context access
fn validate_email_with_context(ctx: &HttpContext, value: &str) -> Result<(), HttpFailResult> {
    // Can access request headers, path, etc. from ctx
    if value.contains('@') {
        Ok(())
    } else {
        Err(HttpFailResult::as_validation_error(
            "Invalid email format".to_string()
        ))
    }
}
```

**Validator Signatures:**
- Simple: `fn validator_name(value: &str) -> Result<(), String>`
- With context: `fn validator_name(ctx: &HttpContext, value: &str) -> Result<(), HttpFailResult>`

## Debugging Input Models

Add `#[debug]` attribute to a field or use `print_request_to_console` flag to debug request parsing:
```rust
#[derive(MyHttpInput)]
pub struct DebugInputModel {
    #[http_query(
        name = "test",
        description = "Test parameter",
        print_request_to_console
    )]
    pub test: String,
}
```

## Custom HttpInputFields

You can create custom input field types based on `String` with additional validation and processing. This is useful for fields like passwords, emails, or other types that need special handling:

```rust
#[http_input_field(open_api_type: "Password")]
pub struct PasswordField(String);

fn process_value(src: &str) -> Result<rust_extensions::StrOrString, HttpFailResult> {
    // Password validation
    if src.len() < 8 {
        return Err(HttpFailResult::as_validation_error(
            "Password must be at least 8 characters long".to_string(),
        ));
    }
    
    let src = src.trim();
    let src = src.to_lowercase();
    Ok(rust_extensions::StrOrString::create_as_string(src))
}
```

**Usage in Input Models:**
```rust
#[derive(MyHttpInput)]
pub struct AuthenticateInputModel {
    #[http_form_data(description = "Email of user")]
    pub email: String,
    
    #[http_form_data(description = "Password of user")]
    pub password: PasswordField,  // Custom field type
}
```

**OpenAPI Types:**
- `String` (default)
- `Password` - Renders as password input in Swagger UI
