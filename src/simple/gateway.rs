
use std::fmt;
use std::os::fd::RawFd;
use std::time::Duration;
use async_trait::async_trait;
use bytes::Bytes;
use log::info;
use pingora_core::prelude::HttpPeer;
use pingora_core::protocols::Digest;
use pingora_error::{Error, ErrorSource, ErrorType, ImmutStr, RetryType};
use pingora_http::{RequestHeader, ResponseHeader};
use pingora_proxy::{ProxyHttp, Session};



fn log_summary(session: &mut Session,func_name:&str) {
    let summary = session.request_summary();
    info!("==>{func_name:?} {summary:?}")
}

#[derive(Debug)]
struct MyPeerError {
    case:String,
}

impl MyPeerError {
    fn new(case:String) -> MyPeerError{
        MyPeerError{
            case
        }
    }
}
impl fmt::Display for MyPeerError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "the my error is here!")
    }
}

impl std::error::Error for MyPeerError {}

pub struct SimpleGateway {
    name:String,
}

impl SimpleGateway {
    pub fn new(name:String) -> SimpleGateway {
        SimpleGateway{
            name
        }
    }
}
#[async_trait]
impl ProxyHttp for SimpleGateway{
    type CTX = ();
    fn new_ctx(&self) -> Self::CTX {}

    ///
    /// 转发
    async fn upstream_peer(&self, session: &mut Session, ctx: &mut Self::CTX) -> pingora_error::Result<Box<HttpPeer>> {
        log_summary(session,"upstream_peer");

        // let s: ImmutStr = "test".into();
        // let e = Error {
        //     etype:ErrorType::Custom("my error"),
        //     esource:ErrorSource::Upstream,
        //     retry:RetryType::Decided(false),
        //     cause:Some(Box::new(MyPeerError::new("my case".to_string()))),
        //     context:Some(s)
        // };
        // Err(Box::new(e))

        let peer = Box::new(HttpPeer::new(("127.0.0.1", 8080), false, "1.1.1.1".to_string()));
        Ok(peer)
    }

    ///
    /// 首先处理请求
    /// 解析、验证、速率限制
    /// 注意 返回true意味着请求退出 false 继续
    async fn request_filter(&self, _session: &mut Session, _ctx: &mut Self::CTX) -> pingora_error::Result<bool> where Self::CTX: Send + Sync {
        log_summary(_session,"request_filter");
        Ok(false)
    }
    
    ///
    /// 决定是否继续请求
    async fn proxy_upstream_filter(&self, _session: &mut Session, _ctx: &mut Self::CTX) -> pingora_error::Result<bool> where Self::CTX: Send + Sync {
        log_summary(_session,"proxy_upstream_filter");
        Ok(true)
    }

    ///
    /// 请求发到上游前修改请求
    async fn upstream_request_filter(&self, _session: &mut Session, _upstream_request: &mut RequestHeader, _ctx: &mut Self::CTX) -> pingora_error::Result<()> where Self::CTX: Send + Sync {
        log_summary(_session,"upstream_request_filter");
        _upstream_request.insert_header("service-name",&self.name)
    }

    ///
    /// 在返回下游前修改响应
    async fn response_filter(&self, _session: &mut Session, _upstream_response: &mut ResponseHeader, _ctx: &mut Self::CTX) -> pingora_error::Result<()> where Self::CTX: Send + Sync {
        log_summary(_session,"response_filter");
        _upstream_response.insert_header("server-resp",&self.name)
    }

    ///
    /// 针对响应正文
    fn response_body_filter(&self, _session: &mut Session, _body: &mut Option<Bytes>, _end_of_stream: bool, _ctx: &mut Self::CTX) -> pingora_error::Result<Option<Duration>> where Self::CTX: Send + Sync {
        log_summary(_session,"response_body_filter");
        Ok(Some(Duration::from_secs(0)))
    }


    ///
    /// session日志
    async fn logging(&self, _session: &mut Session, _e: Option<&Error>, _ctx: &mut Self::CTX) where Self::CTX: Send + Sync {
        let res_code = _session.response_written()
            .map_or(0,|resp|resp.status.as_u16());
        info!("{} response code: {res_code}",self.request_summary(_session,_ctx));
    }

    ///
    /// 刚刚连接
    async fn connected_to_upstream(&self, _session: &mut Session, _reused: bool, _peer: &HttpPeer, _fd: RawFd, _digest: Option<&Digest>, _ctx: &mut Self::CTX) -> pingora_error::Result<()> where Self::CTX: Send + Sync {
        log_summary(_session,"connected_to_upstream");
        Ok(())
    }
}
