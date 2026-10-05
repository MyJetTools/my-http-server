# Enums as Input Types

The framework supports string and integer enums for input models using `MyHttpStringEnum` and `MyHttpIntegerEnum`:

**String Enum:**
```rust
#[derive(Clone, Copy, MyHttpStringEnum)]
pub enum DataSynchronizationPeriod {
    #[http_enum_case(id = "0", value = "i", description = "Immediately Persist")]
    Immediately,
    
    #[http_enum_case(id = "1", value = "1", description = "Persist during 1 sec")]
    Sec1,
    
    #[http_enum_case(id = "5", value = "5", description = "Persist during 5 sec", default)]
    Sec5,
    
    #[http_enum_case(id = "15", value = "15", description = "Persist during 15 sec")]
    Sec15,
}

#[derive(MyHttpInput)]
pub struct SyncInputModel {
    #[http_query(name = "syncPeriod", description = "Synchronization period", default = "Sec5")]
    pub sync_period: DataSynchronizationPeriod,
}
```

**Integer Enum:**
```rust
#[derive(Clone, Copy, MyHttpIntegerEnum)]
pub enum StatusCode {
    #[http_enum_case(id = "200", description = "OK")]
    Ok,
    
    #[http_enum_case(id = "404", description = "Not Found")]
    NotFound,
}
```

**Enum Case Attributes:**
- `id` - Numeric identifier for the enum case (required)
- `value` - String value used in HTTP requests (optional, defaults to variant name)
- `description` - Description for Swagger documentation (required)
- `default` - Marks this case as the default value (optional)
