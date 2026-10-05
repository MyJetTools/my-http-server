# Example: Complete Action

Two complete action files. Each holds the route, the action struct, its models and the handler. Registration is one line in `build_controllers.rs` and the module export lives in `{group}/mod.rs` — both in topic [`registration-and-startup`](registration-and-startup.md).

**GET with a query parameter and a JSON response** — `src/http_server/controllers/certificates/get_cert_info_action.rs`:

```rust
use std::sync::Arc;

use my_http_server::macros::*;
use my_http_server::*;
use serde::{Deserialize, Serialize};

use crate::app::AppContext;

#[http_route(
    method: "GET",
    route: "/api/certificates/v1/info",
    controller: "Certificates",
    summary: "Get certificate information",
    description: "Returns the certificate issued for a domain",
    input_data: "GetCertInfoInputModel",
    result: [
        {status_code: 200, description: "Certificate info", model: "CertificateInfoHttpModel"},
        {status_code: 404, description: "No certificate for this domain"},
    ]
)]
pub struct GetCertInfoAction {
    app: Arc<AppContext>,
}

impl GetCertInfoAction {
    pub fn new(app: Arc<AppContext>) -> Self {
        Self { app }
    }
}

#[derive(MyHttpInput)]
pub struct GetCertInfoInputModel {
    #[http_query(name = "domain", description = "Domain name")]
    pub domain: String,
}

#[derive(Serialize, Deserialize, MyHttpObjectStructure)]
pub struct CertificateInfoHttpModel {
    pub cn: String,
    pub expires: String,
}

async fn handle_request(
    action: &GetCertInfoAction,
    input_data: GetCertInfoInputModel,
    _ctx: &HttpContext,
) -> Result<HttpOkResult, HttpFailResult> {
    // Business logic lives in scripts/ — the action only maps it to HTTP.
    let cert = crate::scripts::get_cert_info(&action.app, &input_data.domain).await;

    let Some(cert) = cert else {
        return HttpFailResult::as_not_found(
            format!("No certificate for domain '{}'", input_data.domain),
            false,
        )
        .into_err();
    };

    let response = CertificateInfoHttpModel {
        cn: cert.cn,
        expires: cert.expires,
    };

    HttpOutput::as_json(response).into_ok_result(true).into()
}
```

**POST with a JSON body and an empty response** — `src/http_server/controllers/certbot/add_domain_action.rs`:

```rust
use std::sync::Arc;

use my_http_server::macros::*;
use my_http_server::*;

use crate::app::AppContext;

#[http_route(
    method: "POST",
    route: "/api/certbot/v1/add-domain",
    controller: "Certbot",
    summary: "Issue a certificate for a domain",
    description: "Requests a certificate for the domain and stores it",
    input_data: "AddDomainInputModel",
    result: [
        {status_code: 204, description: "Certificate issued"},
        {status_code: 500, description: "Certificate could not be issued"},
    ]
)]
pub struct AddDomainAction {
    app: Arc<AppContext>,
}

impl AddDomainAction {
    pub fn new(app: Arc<AppContext>) -> Self {
        Self { app }
    }
}

#[derive(MyHttpInput)]
pub struct AddDomainInputModel {
    #[http_body(name = "domain", description = "Domain name to add certificate for")]
    pub domain: String,

    #[http_body(name = "email", description = "Email address for certificate registration")]
    pub email: String,
}

async fn handle_request(
    action: &AddDomainAction,
    input_data: AddDomainInputModel,
    _ctx: &HttpContext,
) -> Result<HttpOkResult, HttpFailResult> {
    let result =
        crate::scripts::add_domain(&action.app, &input_data.domain, &input_data.email).await;

    if let Err(err) = result {
        return HttpFailResult::as_fatal_error(err).into_err();
    }

    // No response body — 204 No Content.
    HttpOutput::Empty.into_ok_result(true).into()
}
```
