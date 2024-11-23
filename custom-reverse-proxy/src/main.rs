use async_trait::async_trait;
use pingora::{
    prelude::HttpPeer,
    proxy::{http_proxy_service, ProxyHttp, Session},
    server::Server,
    Result,
};

pub struct ReverseProxy;

impl ReverseProxy {
    pub fn new() -> Self {
        ReverseProxy {}
    }
}

#[async_trait]
impl ProxyHttp for ReverseProxy {
    type CTX = ();

    fn new_ctx(&self) -> Self::CTX {
        ()
    }

    async fn upstream_peer(&self, _session: &mut Session, _ctx: &mut ()) -> Result<Box<HttpPeer>> {
        Ok(Box::new(HttpPeer::new(
            "one.one.one.one:443".to_string(),
            true,
            "one.one.one.one".to_string(),
        )))
    }
}

fn main() {
    let mut server = Server::new(None).expect("[main] failed to start pingora server");
    server.bootstrap();

    let mut service = http_proxy_service(&server.configuration, ReverseProxy::new());

    service
        .add_tls("0.0.0.0:4430", "./keys/one_cert.crt", "./keys/one_key.pem") // TODO change this to config
        .expect("[main] failed to add tls config");

    server.add_service(service);
    server.run_forever()
}
