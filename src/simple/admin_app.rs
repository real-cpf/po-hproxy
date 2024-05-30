
use std::time::Duration;
use async_trait::async_trait;
use bytes::Bytes;
use http::{Response, StatusCode};
use log::info;
use moka::sync::Cache;
use pingora_core::apps::http_app::ServeHttp;
use pingora_core::protocols::http::server::Session;
use pingora_core::protocols::http::ServerSession;
use pingora_timeout::timeout;
use serde::{Deserialize, Serialize};
use crate::simple::mut_route_proxy::PeerAddr;

pub struct AdminApp {
    pub map:Cache<String,PeerAddr>,
}
#[derive(Serialize, Deserialize)]
struct NewRoute {
    route: String,
    addr: String,
    port: u16,
}

#[async_trait]
impl ServeHttp for AdminApp {
    async fn response(&self, http_session: &mut ServerSession) -> Response<Vec<u8>> {
        let read_timeout = 2000;
        let body = match timeout(
            Duration::from_millis(read_timeout),
            http_session.read_request_body(),
        )
            .await
        {
            Ok(res) => res.unwrap().unwrap_or_else(|| Bytes::from("no body!")),
            Err(_) => {
                panic!("Timed out after {:?}ms", read_timeout);
            }
        };

        let v:NewRoute= serde_json::from_slice(body.to_vec().as_slice()).unwrap();
        let route = v.route;
        let addr = v.addr;
        let port = v.port;
        info!("new route ==={route} > {addr}:{port}");
        self.map.insert(route,PeerAddr(addr,port));

        let ret = Bytes::from("ok");
        Response::builder()
            .status(StatusCode::OK)
            .header(http::header::CONTENT_TYPE, "text/html")
            .header(http::header::CONTENT_LENGTH, ret.len())
            .body(ret.to_vec())
            .unwrap()
    }
}