use prest::*;
use pingora::{
    proxy::{http_proxy_service, ProxyHttp, Session as PingoraSession},
    server::Server,
    upstreams::peer::HttpPeer,
    Result as PingoraResult,
    Error as PingoraError,
};
struct Proxy;
#[async_trait]
impl ProxyHttp for Proxy {
    type CTX = ();
    fn new_ctx(&self) -> () {
        ()
    }
    
    fn suppress_error_log(&self, _session: &PingoraSession, _ctx: &Self::CTX, _error: &PingoraError) -> bool {
        true
    }

    async fn upstream_peer(
        &self,
        session: &mut PingoraSession,
        _ctx: &mut (),
    ) -> PingoraResult<Box<HttpPeer>> {
        let Some(host_header) = &session.req_header().headers.get("host") else {
            return Err(PingoraError::new_str("no host header"));
        };

        let Ok(host) = host_header.to_str() else {
            return Err(PingoraError::new_str("invalid host header"));
        };

        let upstream = match host {
            "localhost" | _ => "127.0.0.1:47351",
            //"TODO" => {
            //    warn!("unknown host requested: {host}");
            //    return Err(PingoraError::new_str("unknown host requested"))
            //}
        };

        // Set SNI?
        let peer = Box::new(HttpPeer::new(
            upstream,
            false,
            "SNI".to_owned(),
        ));
        Ok(peer)
    }
}


embed_build_output_as!(BuiltAssets);

fn main() {
    init!();
    std::thread::spawn(|| {
        let proxy_addr = "0.0.0.0:80";
        let mut my_server = Server::new(None).unwrap();
        my_server.bootstrap();
        let mut proxy = http_proxy_service(&my_server.configuration, Proxy);
        proxy.add_tcp(&proxy_addr);
        my_server.add_service(proxy);
        info!("Starting proxy at {proxy_addr}");
        my_server.run_forever();
    });

    blog::routes().embed(BuiltAssets).run()
}
