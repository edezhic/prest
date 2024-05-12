use crate::*;

use pingora::{
    proxy::{http_proxy_service, ProxyHttp, Session as PingoraSession},
    server::Server,
    upstreams::peer::HttpPeer,
    Error as PingoraError, Result as PingoraResult,
};

type Domain = String;
type Port = u16;
use std::collections::HashMap;
use std::sync::RwLock;

state!(APPS: RwLock<HashMap<Domain, Port>> = { RwLock::default() });

pub fn start(port: Port) {
    let proxy_addr = format!("0.0.0.0:{port}");
    let mut my_server = Server::new(None).unwrap();
    my_server.bootstrap();
    let mut proxy = http_proxy_service(&my_server.configuration, Proxy);
    proxy.add_tcp(&proxy_addr);
    my_server.add_service(proxy);
    std::thread::spawn(move || my_server.run_forever());
}

pub async fn assign(port: Port, domain: Domain, name: String) -> Result<()> {
    let password = std::env::var("SSH_PASSWORD").unwrap_or("".to_owned());
        
    let resp = reqwest::Client::new()
        .get("http://127.0.0.1/prest-proxy-add-app")
        .header("password", password)
        .header("port", port)
        .header("domain", domain)
        .header("name", name)
        //.header("tls", "true")
        .send()
        .await
        .expect("Proxy should be running and accepting apps");

    if resp.status() == 202 {
        Ok(())
    } else {
        Err(anyhow!("Couldn't assign app in the proxy").into())
    }
}


struct Proxy;
#[async_trait]
impl ProxyHttp for Proxy {
    type CTX = ();
    fn new_ctx(&self) -> () {
        ()
    }

    fn suppress_error_log(
        &self,
        _session: &PingoraSession,
        _ctx: &Self::CTX,
        _error: &PingoraError,
    ) -> bool {
        true
    }

    async fn request_filter(
        &self,
        session: &mut PingoraSession,
        _ctx: &mut Self::CTX,
    ) -> PingoraResult<bool>
    where
        Self::CTX: Send + Sync,
    {
        if session.req_header().uri.path() == "/prest-proxy-add-app" {
            //let failure
            if let Some(password) = &session.req_header().headers.get("password") {
                let expected_password = std::env::var("SSH_PASSWORD").unwrap_or("".to_owned());
                if password.to_str().unwrap() == expected_password.as_str() {
                    let headers = &session.req_header().headers;
                    let Some(port) = headers
                        .get("port")
                        .map(|v| v.to_str().ok())
                        .flatten()
                        .map(|v| v.parse::<u16>().ok())
                        .flatten()
                    else {
                        return Err(PingoraError::new_str("should include port"));
                    };

                    let Some(domain) = headers.get("domain").map(|v| v.to_str().ok()).flatten()
                    else {
                        return Err(PingoraError::new_str("should include domain"));
                    };

                    let Some(name) = headers.get("name").map(|v| v.to_str().ok()).flatten() else {
                        return Err(PingoraError::new_str("should include name"));
                    };

                    let tls = headers.get("tls").is_some();

                    APPS.write().unwrap().insert(domain.to_owned(), port);

                    let addr = format!("{}://{domain}", if tls { "https" } else { "http" });

                    info!("Accepting connections to {name} at {addr}");

                    session.respond_error(202).await;

                    return Ok(true);
                }

                return Err(PingoraError::new_str("no host header"));
            };
        }
        Ok(false)
    }

    async fn upstream_peer(
        &self,
        session: &mut PingoraSession,
        _ctx: &mut (),
    ) -> PingoraResult<Box<HttpPeer>> {
        let Some(host_header) = &session.req_header().headers.get("host") else {
            session.respond_error(400).await;
            return Err(PingoraError::new_str("no host header"));
        };

        let Ok(host) = host_header.to_str() else {
            session.respond_error(400).await;
            return Err(PingoraError::new_str("invalid host header"));
        };

        let Some(app_port) = APPS.read().unwrap().get(host).copied() else {
            session.respond_error(404).await;
            return Err(PingoraError::new_str("unknown app requested"));
        };

        let upstream = format!("127.0.0.1:{app_port}");

        let peer = Box::new(HttpPeer::new(upstream.clone(), false, upstream));
        Ok(peer)
    }
}
