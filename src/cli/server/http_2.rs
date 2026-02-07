#![allow(clippy::too_many_arguments)]
use std::sync::Arc;

use http_body_util::{BodyExt, Full};
use hyper::body::Incoming;
use hyper::service::service_fn;
use hyper_util::rt::{TokioExecutor, TokioIo};
use hyper_util::server::conn::auto::Builder;
use rustls_pki_types::CertificateDer;
use tokio::net::TcpListener;
use tokio::sync::oneshot;
use tokio_rustls::TlsAcceptor;

use super::server_config::ServerConfig;
use crate::core::async_graphql_hyper::{GraphQLBatchRequest, GraphQLRequest};
use crate::core::config::PrivateKey;
use crate::core::http::handle_request;
use crate::core::Errata;

pub async fn start_http_2(
    sc: Arc<ServerConfig>,
    cert: Vec<CertificateDer<'static>>,
    key: PrivateKey,
    server_up_sender: Option<oneshot::Sender<()>>,
) -> anyhow::Result<()> {
    let addr = sc.addr();

    let mut server_config = rustls::ServerConfig::builder_with_provider(Arc::new(
        rustls::crypto::aws_lc_rs::default_provider(),
    ))
    .with_safe_default_protocol_versions()?
    .with_no_client_auth()
    .with_single_cert(cert, key.into_inner())?;
    server_config.alpn_protocols = vec![b"h2".to_vec(), b"http/1.1".to_vec()];
    let tls_acceptor = TlsAcceptor::from(Arc::new(server_config));

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
        let tls_acceptor = tls_acceptor.clone();
        let sc = sc.clone();

        tokio::spawn(async move {
            let tls_stream = match tls_acceptor.accept(stream).await {
                Ok(s) => s,
                Err(e) => {
                    tracing::error!("TLS handshake error: {}", e);
                    return;
                }
            };
            let io = TokioIo::new(tls_stream);

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
            builder.http2();

            if let Err(e) = builder.serve_connection(io, svc).await {
                tracing::error!("Error serving connection: {}", e);
            }
        });
    }
}
