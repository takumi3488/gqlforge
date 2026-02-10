use std::path::Path;

use anyhow::Result;
use bytes::Bytes;
use http::{Request, Response, StatusCode};
use http_body_util::{BodyExt, Full};
use tower::ServiceExt;
use tower_http::services::{ServeDir, ServeFile};

fn is_file_request(path: &str) -> bool {
    path.rsplit('/')
        .next()
        .is_some_and(|segment| segment.contains('.'))
}

fn build_serve_request(req: &Request<Full<Bytes>>) -> Result<Request<Full<Bytes>>> {
    Ok(Request::builder()
        .method(req.method())
        .uri(req.uri())
        .body(Full::<Bytes>::default())?)
}

pub async fn try_serve_spa(
    req: &Request<Full<Bytes>>,
    spa_dir: &Path,
) -> Result<Response<Full<Bytes>>> {
    let path = req.uri().path();

    if is_file_request(path) {
        let service = ServeDir::new(spa_dir);
        let response = service.oneshot(build_serve_request(req)?).await?;
        let (parts, body) = response.into_parts();
        let bytes = body.collect().await?.to_bytes();
        let response = Response::from_parts(parts, Full::new(bytes));
        if response.status() == StatusCode::NOT_FOUND {
            return Ok(Response::builder()
                .status(StatusCode::NOT_FOUND)
                .body(Full::default())?);
        }
        Ok(response)
    } else {
        let service = ServeDir::new(spa_dir).fallback(ServeFile::new(spa_dir.join("index.html")));
        let response = service.oneshot(build_serve_request(req)?).await?;
        let (parts, body) = response.into_parts();
        let bytes = body.collect().await?.to_bytes();
        Ok(Response::from_parts(parts, Full::new(bytes)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_file_request() {
        assert!(is_file_request("/assets/app.js"));
        assert!(is_file_request("/style.css"));
        assert!(is_file_request("/index.html"));
        assert!(is_file_request("/fonts/roboto.woff2"));

        assert!(!is_file_request("/dashboard"));
        assert!(!is_file_request("/users/123"));
        assert!(!is_file_request("/"));
        assert!(!is_file_request("/about"));
    }
}
