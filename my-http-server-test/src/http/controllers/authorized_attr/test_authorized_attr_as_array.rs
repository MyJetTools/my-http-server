use my_http_server::macros::*;
use my_http_server::*;

#[http_route(
    method: "GET",
    route: "/api/authorized",
    summary: "Test with Authorized attribute",
    description: "Test with Authorized attribute",
    controller: "TestVecOfEnumAsI32",
    authorized: ["Admin"],
    result:[
        {status_code: 200, description: "Ok response"},
    ]
)]

pub struct TestAuthorizedAction {}

impl TestAuthorizedAction {
    pub fn new() -> Self {
        Self {}
    }
}

async fn handle_request(
    _action: &TestAuthorizedAction,
    _ctx: &HttpContext,
) -> Result<HttpOkResult, HttpFailResult> {
    return HttpOutput::Empty.into_ok_result(true).into();
}

#[cfg(test)]
mod tests {
    use super::TestAuthorizedAction;
    use my_http_server::controllers::actions::GetDescription;

    #[test]
    fn test_claims() {
        let description = TestAuthorizedAction::get_description().unwrap();

        match description.should_be_authorized {
            my_http_server::controllers::documentation::ShouldBeAuthorized::Yes => {
                panic!("ShouldBeAuthorized::Yes is wrong")
            }
            my_http_server::controllers::documentation::ShouldBeAuthorized::YesWithClaims(
                value,
            ) => {
                let claims = value.required_claims.as_slice();

                assert_eq!(1, claims.len());
                assert_eq!("Admin", claims[0])
            }
            my_http_server::controllers::documentation::ShouldBeAuthorized::No => {
                panic!("ShouldBeAuthorized::No is wrong")
            }
            my_http_server::controllers::documentation::ShouldBeAuthorized::UseGlobal => {
                panic!("ShouldBeAuthorized::UseGlobal is wrong")
            }
        }
    }
}
