// Verifies the macro split: the model is marked up ONCE and gets its schema + client request
// builder from my-http-utils (`MyHttpInput`, `MyHttpObjectStructure`) while the server-only parse
// comes from `MyHttpInputServer` here, and `#[http_route]` (server) glues them together.

use my_http_server::controllers::documentation::DataTypeProvider;
use my_http_server::macros::*;
use my_http_server::*;
use serde::*;

// One markup, two derives: my-http-utils gives schema + client builder, MyHttpInputServer gives parse.
#[derive(MyHttpInput, MyHttpInputServer)]
pub struct UpdateUserRequest {
    #[http_path(name = "id", description = "User id")]
    pub id: String,

    #[http_query(name = "notify", description = "Notify flag")]
    pub notify: bool,

    #[http_header(name = "X-Token", description = "Auth token")]
    pub token: String,

    #[http_body(name = "email", description = "New email")]
    pub email: String,
}

#[derive(Serialize, Deserialize, MyHttpObjectStructure)]
pub struct UpdateUserResponse {
    pub id: String,
}

#[http_route(
    method: "POST",
    route: "/api/users/{id}",
    controller: "Users",
    summary: "Update user",
    description: "Updates a user's email",
    input_data: "UpdateUserRequest",
    result: [
        { status_code: 200, description: "Ok", model: "UpdateUserResponse" },
    ]
)]
pub struct UpdateUserAction;

async fn handle_request(
    _action: &UpdateUserAction,
    _input: UpdateUserRequest,
    _ctx: &mut HttpContext,
) -> Result<HttpOkResult, HttpFailResult> {
    HttpOutput::as_json(UpdateUserResponse {
        id: "1".to_string(),
    })
    .into_ok_result(true)
    .into()
}

#[test]
fn macro_split_round_trip_compiles() {
    // Schema half — from my-http-utils MyHttpInput.
    let params = UpdateUserRequest::get_input_params();
    assert_eq!(params.len(), 4);

    // Output schema — from my-http-utils MyHttpObjectStructure.
    let structure = UpdateUserResponse::get_http_data_structure();
    assert_eq!(structure.main.fields.len(), 1);

    // Server action description — from #[http_route], built off the my-http-utils schema.
    let description = UpdateUserAction::get_description();
    assert!(description.is_some());
}
