use std::collections::HashMap;
use std::sync::Arc;

use async_trait::async_trait;
use bytes::Bytes;
use pingora_core::upstreams::peer::HttpPeer;
use pingora_core::Result as PingoraResult;
use pingora_error::{Error, ErrorType};
use pingora_proxy::{ProxyHttp, Session};
use tracing::{debug, warn};
use url::Url;

use crate::config::ReverseProxyConfig;

#[derive(Clone)]
pub struct ReverseProxy {
    routes: Arc<HashMap<String, HttpPeer>>,
    default: Option<HttpPeer>,
}

impl ReverseProxy {
    pub fn from_config(config: &ReverseProxyConfig) -> anyhow::Result<Self> {
        let mut peers = HashMap::new();
        for (hostname, target) in &config.routes {
            let peer = build_peer(target)?;
            peers.insert(hostname.to_lowercase(), peer);
        }

        let default = match &config.default_upstream {
            Some(url) => Some(build_peer(url)?),
            None => None,
        };

        Ok(Self {
            routes: Arc::new(peers),
            default,
        })
    }

    fn resolve_route(&self, hostname: &str) -> Option<HttpPeer> {
        let normalized = hostname.to_lowercase();
        self.routes
            .get(&normalized)
            .cloned()
            .or_else(|| self.default.clone())
    }

    fn extract_hostname(session: &Session) -> Option<String> {
        session
            .req_header()
            .headers
            .get("host")
            .and_then(|v| v.to_str().ok())
            .map(|s| s.to_string())
    }
}

#[derive(Default, Clone)]
pub struct RequestContext {
    hostname: Option<String>,
}

#[async_trait]
impl ProxyHttp for ReverseProxy {
    type CTX = RequestContext;

    fn new_ctx(&self) -> Self::CTX {
        RequestContext::default()
    }

    async fn request_filter(
        &self,
        session: &mut Session,
        ctx: &mut Self::CTX,
    ) -> PingoraResult<bool> {
        ctx.hostname = Self::extract_hostname(session);
        if ctx.hostname.is_none() {
            warn!("request missing SNI/Host information");
            let _ = session
                .respond_error_with_body(400, Bytes::from_static(b"missing host information"))
                .await;
            return Ok(true);
        }
        Ok(false)
    }

    async fn upstream_peer(
        &self,
        _session: &mut Session,
        ctx: &mut Self::CTX,
    ) -> PingoraResult<Box<HttpPeer>> {
        if let Some(host) = ctx.hostname.clone() {
            if let Some(peer) = self.resolve_route(&host) {
                debug!("routing {host} to {}", peer._address);
                return Ok(Box::new(peer));
            }
        }

        Err(Error::e_explain(
            ErrorType::HTTPStatus(502),
            "no upstream configured for hostname",
        )?)
    }
}

fn build_peer(target: &str) -> anyhow::Result<HttpPeer> {
    let url = Url::parse(target)?;
    let host = url
        .host_str()
        .ok_or_else(|| anyhow::anyhow!("missing host in upstream url"))?;
    let port = url
        .port_or_known_default()
        .ok_or_else(|| anyhow::anyhow!("missing port for upstream"))?;
    let tls = matches!(url.scheme(), "https" | "wss");

    Ok(HttpPeer::new((host, port), tls, host.to_string()))
}
