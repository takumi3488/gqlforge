use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::sync::Arc;

use crate::cli::runtime::init;
use crate::core::app_context::AppContext;
use crate::core::blueprint::{Blueprint, Http};
use crate::core::rest::{EndpointSet, Unchecked};

pub struct ServerConfig {
    pub blueprint: Blueprint,
    pub app_ctx: Arc<AppContext>,
}

impl ServerConfig {
    pub async fn new(
        blueprint: Blueprint,
        endpoints: EndpointSet<Unchecked>,
    ) -> anyhow::Result<Self> {
        let rt = init(&blueprint)?;

        let endpoints = endpoints.into_checked(&blueprint, rt.clone()).await?;
        let app_context = Arc::new(AppContext::new(blueprint.clone(), rt, endpoints));

        Ok(Self { app_ctx: app_context, blueprint })
    }

    pub fn addr(&self) -> SocketAddr {
        (self.blueprint.server.hostname, self.blueprint.server.port).into()
    }

    pub fn http_version(&self) -> String {
        match self.blueprint.server.http {
            Http::HTTP2 { cert: _, key: _ } => "HTTP/2".to_string(),
            _ => "HTTP/1.1".to_string(),
        }
    }

    pub fn graphiql_url(&self) -> String {
        let protocol = match self.http_version().as_str() {
            "HTTP/2" => "https",
            _ => "http",
        };
        let mut addr = self.addr();

        if addr.ip().is_unspecified() {
            addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), addr.port());
        }

        format!("{}://{}", protocol, addr)
    }
}
