use std::sync::Arc;

use http_body_util::{BodyExt, Full};
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

    loop {
        let (stream, _addr) = listener.accept().await?;
        let io = TokioIo::new(stream);
        let sc = sc.clone();

        tokio::spawn(async move {
            let svc = service_fn(move |req: http::Request<Incoming>| {
                let sc = sc.clone();
                async move {
                    let (parts, body) = req.into_parts();
                    let bytes = body.collect().await?.to_bytes();
                    let req = http::Request::from_parts(parts, Full::new(bytes));
                    if enable_batch {
                        handle_request::<GraphQLBatchRequest>(req, sc.app_ctx.clone()).await
                    } else {
                        handle_request::<GraphQLRequest>(req, sc.app_ctx.clone()).await
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
