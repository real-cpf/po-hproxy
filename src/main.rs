use std::collections::HashMap;
use pingora_core::prelude::Opt;
use pingora_core::server::Server;
use structopt::StructOpt;

mod simple;

use simple::gateway::SimpleGateway;
use crate::simple::proxy::{PeerAddr, SimpleProxy};

// 使用httpbin 验证转发 以及header修改
// podman run -p 8080:80 docker.io/kong/httpbin:latest

// RUST_LOG=INFO cargo run
// curl localhost:18081/get -v
// 该请求会转发到8080 并体系header修改
fn main() {
    env_logger::init();

    let opt = Opt::from_args();
    let mut my_server = Server::new(Some(opt)).unwrap();

    my_server.bootstrap();

    let mut simple_service = pingora_proxy::http_proxy_service(&my_server.configuration,
                                                               SimpleGateway::new("simple".to_string()));
    simple_service.add_tcp("0.0.0.0:18081");

    let mut where_go_to: HashMap<String, PeerAddr> = HashMap::new();
    where_go_to.insert("a".to_string(),PeerAddr("127.0.0.1".to_string(),8080));
    where_go_to.insert("b".to_string(),PeerAddr("127.0.0.1".to_string(),8081));
    where_go_to.insert("c".to_string(),PeerAddr("127.0.0.1".to_string(),8082));
    let mut simple_proxy = SimpleProxy::new(where_go_to);

    let mut simple_proxy_service = pingora_proxy::http_proxy_service(&my_server.configuration,
                                                                     simple_proxy);
    simple_proxy_service.add_tcp("0.0.0.0:18082");


    my_server.add_service(simple_service);

    my_server.add_service(simple_proxy_service);

    my_server.run_forever();
}
