use std::collections::HashMap;
use std::str::FromStr;
use async_trait::async_trait;
use bytes::Bytes;
use http::Uri;
use http::uri::{Authority, PathAndQuery, Scheme};
use log::info;
use pingora_core::prelude::HttpPeer;
use pingora_error::Error;
use pingora_http::{RequestHeader, ResponseHeader};
use pingora_proxy::{ProxyHttp, Session};
use moka::sync::Cache;




#[derive(Clone,Debug, PartialEq, PartialOrd, Eq, Ord)]
pub struct PeerAddr(pub String, pub u16);

pub struct MutRouteProxy {
    where_go_to:Cache<String,PeerAddr>,
}

impl MutRouteProxy {
    pub fn new(where_go_to:Cache<String,PeerAddr>) -> MutRouteProxy {
        MutRouteProxy{
            where_go_to
        }
    }
}

pub struct CurrentRoute{
    inner_route:String,
    goto_route:String,
}

#[async_trait]
impl ProxyHttp for MutRouteProxy {
    type CTX = CurrentRoute;

    fn new_ctx(&self) -> Self::CTX {
        CurrentRoute {
            inner_route:"".to_string(),
            goto_route:"".to_string(),
        }
    }

    async fn upstream_peer(&self, session: &mut Session, ctx: &mut Self::CTX) -> pingora_error::Result<Box<HttpPeer>> {
        log_summary(session,"upstream_peer");

        let peer = self.where_go_to.get(&ctx.inner_route).unwrap();
        let peer_addr = &peer.0;
        let peer_port = peer.1;
        let bp = Box::new(HttpPeer::new((String::from(peer_addr),peer_port),false,"".to_string()));
        Ok(bp)
    }
    async fn request_filter(&self, _session: &mut Session, _ctx: &mut Self::CTX) -> pingora_error::Result<bool> where Self::CTX: Send + Sync {
        log_summary(_session,"request_filter");
        let path = _session.req_header().uri.path_and_query().unwrap().to_string();
        let path_arr = path.strip_prefix("/").unwrap().split_once("/");
        if path_arr.is_none() {
            return Ok(true)
        }
        let path_arr = path_arr.unwrap();
        if !self.where_go_to.contains_key(path_arr.0) {

            return Ok(true)
        }
        _ctx.inner_route = path_arr.0.to_string();
        _ctx.goto_route = path_arr.1.to_string();
        Ok(false)
    }

    async fn upstream_request_filter(&self, _session: &mut Session, _upstream_request: &mut RequestHeader, _ctx: &mut Self::CTX) -> pingora_error::Result<()> where Self::CTX: Send + Sync {

        let path = &_ctx.goto_route;
        let v = PathAndQuery::from_str(&path).unwrap();
        _upstream_request.set_uri(Uri::from(v));
        Ok(())

    }

    async fn response_filter(&self, _session: &mut Session, _upstream_response: &mut ResponseHeader, _ctx: &mut Self::CTX) -> pingora_error::Result<()> where Self::CTX: Send + Sync {
        // _upstream_response.remove_header("Content-Length");
        // _upstream_response
        //     .insert_header("Transfer-Encoding", "Chunked")
        //     .unwrap();
        Ok(())
    }

    async fn logging(&self, _session: &mut Session, _e: Option<&Error>, _ctx: &mut Self::CTX) where Self::CTX: Send + Sync {
        log_summary(_session,"logging");
        if _ctx.goto_route.is_empty() {
            let mut req_h = ResponseHeader::build(404,Some(6)).unwrap();

            let b = Bytes::from("can not find proxy path");
            req_h.append_header("Content-Length",b.len()).unwrap();
            _session.write_response_header(Box::new(req_h)).await.unwrap();

            _session.write_response_body(b).await.unwrap();
            _session.set_keepalive(None);

            info!("==>is write{:?}",_session.response_written());

            _session.finish_body().await.unwrap();
        }
    }

}


fn log_summary(session: &mut Session,func_name:&str) {
    let summary = session.request_summary();
    info!("==>{func_name:?} {summary:?}")
}