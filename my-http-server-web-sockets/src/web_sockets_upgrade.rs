use hyper::body::Bytes;

use http_body_util::Full;
use hyper::header::{HeaderValue, UPGRADE};
use hyper::upgrade::Upgraded;
use hyper::{Request, Response, Result, StatusCode};

pub async fn upgrade(
    req: Request<hyper::body::Incoming>,
) -> Result<(Option<Upgraded>, Response<Full<Bytes>>)> {
    let full_body = http_body_util::Full::new(hyper::body::Bytes::from(vec![]));

    let mut res = Response::new(full_body);

    if !req.headers().contains_key(UPGRADE) {
        *res.status_mut() = StatusCode::BAD_REQUEST;
        return Ok((None, res));
    }

    let upgraded = hyper::upgrade::on(req).await?;

    *res.status_mut() = StatusCode::SWITCHING_PROTOCOLS;
    res.headers_mut()
        .insert(UPGRADE, HeaderValue::from_static("foobar"));
    Ok((Some(upgraded), res))
}
