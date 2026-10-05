# Routes — path parameters, model routes, OPTIONS

## Route Path Parameters

Routes can include path parameters using `{param_name}` syntax:
```rust
#[http_route(
    method: "GET",
    route: "/api/users/{userId}/posts/{postId}",
    // ...
)]
```

The corresponding input model must have matching `#[http_path]` fields:
```rust
#[derive(MyHttpInput)]
pub struct GetPostInputModel {
    #[http_path(name = "userId", description = "User ID")]
    pub user_id: String,
    
    #[http_path(name = "postId", description = "Post ID")]
    pub post_id: String,
}
```

## Model Routes

Input models can define alternative route patterns through the `get_model_routes()` function (automatically generated). This allows the same action to handle multiple route patterns that map to the same input model structure.

## OPTIONS Method

The framework supports OPTIONS method for CORS preflight requests. Register OPTIONS actions the same way as other HTTP methods:
```rust
#[http_route(
    method: "OPTIONS",
    route: "/api/cors-endpoint",
    // ...
)]
```
