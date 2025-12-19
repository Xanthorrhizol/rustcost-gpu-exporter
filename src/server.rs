use std::sync::{Arc, Mutex};
use tiny_http::{Header, Response, Server};

pub fn run_server(metrics: Arc<Mutex<String>>) {
    let server = Server::http("0.0.0.0:8000").unwrap();
    println!("Serving metrics on http://localhost:8000/metrics");

    for req in server.incoming_requests() {
        if req.url() == "/metrics" {
            let body = metrics
                .lock()
                .map(|s| s.clone())
                .unwrap_or_else(|_| "{\"error\":\"unavailable\"}".into());

            let response = Response::from_string(body)
                .with_header(Header::from_bytes(b"Content-Type", b"application/json").unwrap());
            let _ = req.respond(response);
        } else {
            let _ = req
                .respond(Response::from_string("{\"error\":\"not_found\"}").with_status_code(404));
        }
    }
}
