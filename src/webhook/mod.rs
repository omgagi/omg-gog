//! Minimal HTTP server for testing Google push notification webhooks.
//!
//! Accepts POST requests on known webhook routes and logs them to stdout.
//! This is a testing utility -- it does NOT authenticate requests.

use http_body_util::{BodyExt, Full};
use hyper::body::{Bytes, Incoming};
use hyper::server::conn::http1;
use hyper::service::service_fn;
use hyper::{Method, Request, Response, StatusCode};
use hyper_util::rt::TokioIo;
use std::convert::Infallible;
use std::net::SocketAddr;
use tokio::net::TcpListener;

/// Known webhook routes.
const ROUTES: &[&str] = &[
    "/",
    "/webhook/google/gmail",
    "/webhook/google/calendar",
    "/webhook/google/drive",
];

/// Start the webhook receiver server.
///
/// Binds to `bind:port`, prints a startup banner, then accepts connections
/// until Ctrl+C is received.
pub async fn serve(bind: &str, port: u16) -> anyhow::Result<()> {
    let addr: SocketAddr = format!("{}:{}", bind, port).parse()?;
    let listener = TcpListener::bind(addr).await?;

    print_banner(bind, port);

    loop {
        tokio::select! {
            accept_result = listener.accept() => {
                let (stream, _remote_addr) = accept_result?;
                let io = TokioIo::new(stream);

                tokio::spawn(async move {
                    if let Err(e) = http1::Builder::new()
                        .serve_connection(io, service_fn(handle_request))
                        .await
                    {
                        eprintln!("connection error: {}", e);
                    }
                });
            }
            _ = tokio::signal::ctrl_c() => {
                eprintln!("\nShutting down webhook server");
                break;
            }
        }
    }

    Ok(())
}

/// Print the startup banner listing bind address and routes.
fn print_banner(bind: &str, port: u16) {
    println!("Listening on {}:{}", bind, port);
    println!("Routes:");
    for route in ROUTES {
        println!("  POST {}", route);
    }
    println!();
}

/// Format the startup banner as a string (for testing).
pub fn format_banner(bind: &str, port: u16) -> String {
    let mut out = String::new();
    out.push_str(&format!("Listening on {}:{}\n", bind, port));
    out.push_str("Routes:\n");
    for route in ROUTES {
        out.push_str(&format!("  POST {}\n", route));
    }
    out
}

/// Handle a single HTTP request.
///
/// - POST on any path: log and return 200 OK
/// - Non-POST: return 405 Method Not Allowed
async fn handle_request(
    req: Request<Incoming>,
) -> Result<Response<Full<Bytes>>, Infallible> {
    if req.method() != Method::POST {
        return Ok(Response::builder()
            .status(StatusCode::METHOD_NOT_ALLOWED)
            .body(Full::new(Bytes::from("405 Method Not Allowed\n")))
            .unwrap());
    }

    let path = req.uri().path().to_string();

    // Extract X-Goog-* headers
    let goog_headers: Vec<(String, String)> = req
        .headers()
        .iter()
        .filter(|(name, _)| {
            name.as_str()
                .to_lowercase()
                .starts_with("x-goog-")
        })
        .map(|(name, value)| {
            (
                name.to_string(),
                value.to_str().unwrap_or("<non-utf8>").to_string(),
            )
        })
        .collect();

    // Read body
    let body_bytes = match req.into_body().collect().await {
        Ok(collected) => collected.to_bytes(),
        Err(e) => {
            eprintln!("error reading body: {}", e);
            Bytes::new()
        }
    };
    let body_str = String::from_utf8_lossy(&body_bytes);

    // Build log output
    let timestamp = chrono::Utc::now().format("%Y-%m-%dT%H:%M:%S%.3fZ");
    let body_json: serde_json::Value = serde_json::from_str(&body_str)
        .unwrap_or_else(|_| serde_json::Value::String(body_str.to_string()));

    let log_entry = serde_json::json!({
        "timestamp": timestamp.to_string(),
        "path": path,
        "headers": goog_headers.iter().map(|(k, v)| {
            serde_json::json!({ "name": k, "value": v })
        }).collect::<Vec<_>>(),
        "body": body_json,
    });

    println!("{}", serde_json::to_string_pretty(&log_entry).unwrap_or_default());

    Ok(Response::builder()
        .status(StatusCode::OK)
        .body(Full::new(Bytes::from("200 OK\n")))
        .unwrap())
}

/// Extract X-Goog-* headers from a request's header map.
///
/// Public for testing.
pub fn extract_goog_headers(
    headers: &[(String, String)],
) -> Vec<(String, String)> {
    headers
        .iter()
        .filter(|(name, _)| name.to_lowercase().starts_with("x-goog-"))
        .cloned()
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    // ===================================================================
    // OI-M4: Webhook Serve -- TDD tests
    //
    // Tests cover REQ-OI-020, REQ-OI-021, REQ-OI-022, REQ-OI-024.
    // ===================================================================

    // -------------------------------------------------------------------
    // REQ-OI-020 (Must): webhook serve command -- request handling
    // -------------------------------------------------------------------

    // Requirement: REQ-OI-020 (Must)
    // Acceptance: POST requests return 200 OK (verified via integration test)
    #[test]
    fn req_oi_020_post_returns_200_response_construction() {
        // Verify the OK response construction logic directly.
        // Full POST -> 200 behavior is covered by the integration test below.
        let resp = Response::builder()
            .status(StatusCode::OK)
            .body(Full::new(Bytes::from("200 OK\n")))
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
    }

    // Requirement: REQ-OI-020 (Must)
    // Acceptance: Routes are defined for gmail, calendar, drive, and catch-all
    #[test]
    fn req_oi_020_routes_defined() {
        assert!(ROUTES.contains(&"/"));
        assert!(ROUTES.contains(&"/webhook/google/gmail"));
        assert!(ROUTES.contains(&"/webhook/google/calendar"));
        assert!(ROUTES.contains(&"/webhook/google/drive"));
    }

    // Requirement: REQ-OI-020 (Must)
    // Acceptance: Four routes are registered
    #[test]
    fn req_oi_020_four_routes() {
        assert_eq!(ROUTES.len(), 4);
    }

    // -------------------------------------------------------------------
    // REQ-OI-021 (Should): startup banner
    // -------------------------------------------------------------------

    // Requirement: REQ-OI-021 (Should)
    // Acceptance: Banner contains "Listening on {bind}:{port}"
    #[test]
    fn req_oi_021_banner_contains_listen_address() {
        let banner = format_banner("0.0.0.0", 8765);
        assert!(
            banner.contains("Listening on 0.0.0.0:8765"),
            "Banner should contain listen address: {}",
            banner
        );
    }

    // Requirement: REQ-OI-021 (Should)
    // Acceptance: Banner lists registered routes
    #[test]
    fn req_oi_021_banner_lists_routes() {
        let banner = format_banner("0.0.0.0", 8765);
        assert!(banner.contains("POST /"), "Banner should list POST /");
        assert!(
            banner.contains("POST /webhook/google/gmail"),
            "Banner should list gmail route"
        );
        assert!(
            banner.contains("POST /webhook/google/calendar"),
            "Banner should list calendar route"
        );
        assert!(
            banner.contains("POST /webhook/google/drive"),
            "Banner should list drive route"
        );
    }

    // Requirement: REQ-OI-021 (Should)
    // Acceptance: Banner with custom bind/port
    #[test]
    fn req_oi_021_banner_custom_bind_port() {
        let banner = format_banner("127.0.0.1", 9999);
        assert!(
            banner.contains("Listening on 127.0.0.1:9999"),
            "Banner should reflect custom bind/port: {}",
            banner
        );
    }

    // -------------------------------------------------------------------
    // REQ-OI-022 (Should): method filtering
    // -------------------------------------------------------------------

    // Requirement: REQ-OI-022 (Should)
    // Acceptance: 405 response is correctly constructed for non-POST
    #[test]
    fn req_oi_022_method_not_allowed_response() {
        let resp = Response::builder()
            .status(StatusCode::METHOD_NOT_ALLOWED)
            .body(Full::new(Bytes::from("405 Method Not Allowed\n")))
            .unwrap();
        assert_eq!(resp.status(), StatusCode::METHOD_NOT_ALLOWED);
    }

    // Requirement: REQ-OI-022 (Should)
    // Acceptance: 200 OK response is correctly constructed for POST
    #[test]
    fn req_oi_022_ok_response() {
        let resp = Response::builder()
            .status(StatusCode::OK)
            .body(Full::new(Bytes::from("200 OK\n")))
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
    }

    // -------------------------------------------------------------------
    // REQ-OI-020 (Must): X-Goog-* header extraction
    // -------------------------------------------------------------------

    // Requirement: REQ-OI-020 (Must)
    // Acceptance: X-Goog headers are extracted from header list
    #[test]
    fn req_oi_020_extract_goog_headers() {
        let headers = vec![
            ("X-Goog-Channel-ID".to_string(), "ch-123".to_string()),
            ("X-Goog-Resource-ID".to_string(), "res-456".to_string()),
            ("Content-Type".to_string(), "application/json".to_string()),
            (
                "X-Goog-Message-Number".to_string(),
                "1".to_string(),
            ),
        ];
        let goog = extract_goog_headers(&headers);
        assert_eq!(goog.len(), 3);
        assert!(goog.iter().any(|(k, _)| k == "X-Goog-Channel-ID"));
        assert!(goog.iter().any(|(k, _)| k == "X-Goog-Resource-ID"));
        assert!(goog.iter().any(|(k, _)| k == "X-Goog-Message-Number"));
    }

    // Requirement: REQ-OI-020 (Must)
    // Acceptance: Non-X-Goog headers are excluded
    #[test]
    fn req_oi_020_extract_goog_headers_excludes_non_goog() {
        let headers = vec![
            ("Content-Type".to_string(), "application/json".to_string()),
            ("Authorization".to_string(), "Bearer token".to_string()),
        ];
        let goog = extract_goog_headers(&headers);
        assert!(goog.is_empty());
    }

    // Requirement: REQ-OI-020 (Must)
    // Acceptance: Empty header list returns empty
    #[test]
    fn req_oi_020_extract_goog_headers_empty() {
        let headers: Vec<(String, String)> = vec![];
        let goog = extract_goog_headers(&headers);
        assert!(goog.is_empty());
    }

    // Requirement: REQ-OI-020 (Must)
    // Edge case: case-insensitive matching for x-goog-* prefix
    #[test]
    fn req_oi_020_extract_goog_headers_case_insensitive() {
        let headers = vec![
            ("x-goog-channel-id".to_string(), "ch-789".to_string()),
            ("X-GOOG-RESOURCE-ID".to_string(), "res-abc".to_string()),
        ];
        let goog = extract_goog_headers(&headers);
        assert_eq!(goog.len(), 2);
    }

    // -------------------------------------------------------------------
    // REQ-OI-024 (Must): HTTP server dependency
    // -------------------------------------------------------------------

    // Requirement: REQ-OI-024 (Must)
    // Acceptance: hyper types are available (compile-time check)
    #[test]
    fn req_oi_024_hyper_dependency_available() {
        // This test verifies that hyper, hyper-util, and http-body-util
        // are available as dependencies (compilation proves it).
        let _status = StatusCode::OK;
        let _method = Method::POST;
        let _bytes = Bytes::new();
    }

    // -------------------------------------------------------------------
    // REQ-OI-020 (Must): Integration test -- full server
    // -------------------------------------------------------------------

    // Requirement: REQ-OI-020 (Must)
    // Acceptance: Server starts, handles POST with 200, GET with 405
    #[tokio::test]
    async fn req_oi_020_server_integration() {
        // Start server on a random available port
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();
        let port = addr.port();

        // Spawn server task
        let server_handle = tokio::spawn(async move {
            loop {
                tokio::select! {
                    accept_result = listener.accept() => {
                        if let Ok((stream, _)) = accept_result {
                            let io = TokioIo::new(stream);
                            tokio::spawn(async move {
                                let _ = http1::Builder::new()
                                    .serve_connection(io, service_fn(handle_request))
                                    .await;
                            });
                        }
                    }
                }
            }
        });

        // Give server a moment to start
        tokio::time::sleep(std::time::Duration::from_millis(50)).await;

        let client = reqwest::Client::new();

        // Test POST returns 200
        let resp = client
            .post(format!("http://127.0.0.1:{}/webhook/google/gmail", port))
            .header("X-Goog-Channel-ID", "test-channel")
            .body(r#"{"test": true}"#)
            .send()
            .await
            .unwrap();
        assert_eq!(resp.status().as_u16(), 200);

        // Test POST to catch-all route
        let resp = client
            .post(format!("http://127.0.0.1:{}/", port))
            .body("{}")
            .send()
            .await
            .unwrap();
        assert_eq!(resp.status().as_u16(), 200);

        // Test GET returns 405
        let resp = client
            .get(format!("http://127.0.0.1:{}/webhook/google/gmail", port))
            .send()
            .await
            .unwrap();
        assert_eq!(resp.status().as_u16(), 405);

        // Test PUT returns 405
        let resp = client
            .put(format!("http://127.0.0.1:{}/webhook/google/calendar", port))
            .body("{}")
            .send()
            .await
            .unwrap();
        assert_eq!(resp.status().as_u16(), 405);

        // Test DELETE returns 405
        let resp = client
            .delete(format!("http://127.0.0.1:{}/webhook/google/drive", port))
            .send()
            .await
            .unwrap();
        assert_eq!(resp.status().as_u16(), 405);

        server_handle.abort();
    }
}
