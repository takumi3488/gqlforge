use std::sync::Arc;

use http::Method;
use http_body_util::{BodyExt, Either, Full};
use hyper::body::Incoming;
use hyper::service::service_fn;
use hyper_util::rt::{TokioExecutor, TokioIo};
use hyper_util::server::conn::auto::Builder;
use tokio::net::TcpListener;
use tokio::sync::oneshot;

use super::server_config::ServerConfig;
use crate::core::Errata;
use crate::core::async_graphql_hyper::{GraphQLBatchRequest, GraphQLRequest};
use crate::core::http::handle_request;
use crate::core::http::sse::{SseBody, handle_sse_request};

pub async fn start_http_1(
    sc: Arc<ServerConfig>,
    server_up_sender: Option<oneshot::Sender<()>>,
) -> anyhow::Result<()> {
    let addr = sc.addr();
    let listener = TcpListener::bind(&addr).await.map_err(Errata::from)?;

    let enable_batch = sc.blueprint.server.enable_batch_requests;

    super::log_launch(sc.as_ref());

    if let Some(sender) = server_up_sender {
        sender
            .send(())
            .or(Err(anyhow::anyhow!("Failed to send message")))?;
    }

    let graphql_stream_path = format!("{}/stream", sc.blueprint.server.routes.graphql());

    loop {
        let (stream, _addr) = listener.accept().await?;
        let io = TokioIo::new(stream);
        let sc = sc.clone();
        let graphql_stream_path = graphql_stream_path.clone();

        tokio::spawn(async move {
            let svc = service_fn(move |req: http::Request<Incoming>| {
                let sc = sc.clone();
                let graphql_stream_path = graphql_stream_path.clone();
                async move {
                    let is_sse =
                        req.method() == Method::POST && req.uri().path() == graphql_stream_path;

                    let (parts, body) = req.into_parts();
                    let bytes = body.collect().await?.to_bytes();
                    let req = http::Request::from_parts(parts, Full::new(bytes));

                    if is_sse {
                        match handle_sse_request(req, sc.app_ctx.clone()).await {
                            Ok(resp) => Ok(resp.map(Either::Right)),
                            Err(e) => {
                                tracing::error!("SSE handler error: {}", e);
                                let body = Full::new(bytes::Bytes::from(format!(
                                    r#"{{"error": "{}"}}"#,
                                    e
                                )));
                                Ok(http::Response::builder()
                                    .status(500)
                                    .body(Either::<Full<bytes::Bytes>, SseBody>::Left(body))
                                    .unwrap())
                            }
                        }
                    } else {
                        let result = if enable_batch {
                            handle_request::<GraphQLBatchRequest>(req, sc.app_ctx.clone()).await
                        } else {
                            handle_request::<GraphQLRequest>(req, sc.app_ctx.clone()).await
                        };
                        result.map(|resp| resp.map(Either::Left))
                    }
                }
            });

            let mut builder = Builder::new(TokioExecutor::new());
            builder.http1();

            if let Err(e) = builder.serve_connection(io, svc).await {
                tracing::error!("Error serving connection: {}", e);
            }
        });
    }
}
