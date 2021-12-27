use poem::{Endpoint, IntoResponse, Request, Response, Result};
use tracing::{info, warn};

pub async fn log<E: Endpoint>(next: E, req: Request) -> Result<Response> {
    // Dec 24 23:37:47.729  INFO blog::middleware: 200 OK - socket://127.0.0.1:53372 0ms GET /signin?a=1
    let remote = (&req).remote_addr().to_string();
    let method = (&req).method().to_string();
    let uri = (&req).original_uri().to_string();
    let start = std::time::Instant::now();

    let res = next.call(req).await;

    let cost = start.elapsed().as_millis(); // ms

    let s = format!("{} {}ms {} {}", remote, cost, method, uri);

    match res {
        Ok(resp) => {
            let resp = resp.into_response();
            info!("{} - {}", resp.status(), s);
            Ok(resp)
        }
        Err(err) => {
            warn!("{} - {} => {}", err.status(), s, err);
            Err(err)
        }
    }
}


pub async fn _auth<E: Endpoint>(_next: E, _req: Request) -> Result<Response> {
    unimplemented!()
}