use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use pingora_core::prelude::Opt;
use pingora_core::server::Server;
use structopt::StructOpt;
use moka::sync::Cache;
use pingora_core::services::listening::Service;

mod simple;

use simple::gateway::SimpleGateway;
use crate::simple::admin_app::AdminApp;
use crate::simple::mut_route_proxy::{MutRouteProxy, PeerAddr};

// 使用httpbin 验证转发 以及header修改
// podman run -p 8080:80 docker.io/kong/httpbin:latest


// RUST_LOG=INFO cargo run
// curl localhost:18081/get -v
// 该请求会转发到8080 并体系header修改

// 动态路由转发

// 1.check route exits ?
// curl --request POST --url 'http://127.0.0.1:18081/a/post?a=3' --header 'Accept: *' --header 'Content-Type: *'
// 2. add route
// curl --request GET \
//   --url http://127.0.0.1:8989/ \
//   --header 'Content-Type: application/json' \
//   --header 'User-Agent: insomnia/2023.5.8' \
//   --data '{
// 	"route":"a",
// 	"addr":"127.0.0.1",
// 	"port":8080
// }'
// 3. check again
// curl --request POST --url 'http://127.0.0.1:18081/a/post?a=3' --header 'Accept: *' --header 'Content-Type: *'
fn main() {
    env_logger::init();

    let opt = Opt::from_args();
    let mut my_server = Server::new(Some(opt)).unwrap();

    my_server.bootstrap();

    let cache:Cache<String,PeerAddr> = Cache::new(120);


    let mut_route_proxy = MutRouteProxy::new(cache.clone());

    // 静态路由 转发

    // let mut simple_service = pingora_proxy::http_proxy_service(&my_server.configuration,
    //                                                            SimpleGateway::new("simple".to_string()));
    // simple_service.add_tcp("0.0.0.0:18081");
    //
    // let mut where_go_to: HashMap<String, PeerAddr> = HashMap::new();
    // where_go_to.insert("a".to_string(), PeerAddr("127.0.0.1".to_string(), 8080));
    // where_go_to.insert("b".to_string(), PeerAddr("127.0.0.1".to_string(), 8081));
    // where_go_to.insert("c".to_string(), PeerAddr("127.0.0.1".to_string(), 8082));
    // let mut simple_proxy = SimpleProxy::new(where_go_to);
    //
    // let mut simple_proxy_service = pingora_proxy::http_proxy_service(&my_server.configuration,
    //                                                                  simple_proxy);
    // simple_proxy_service.add_tcp("0.0.0.0:18082");
    //
    //
    // my_server.add_service(simple_service);
    //
    // my_server.add_service(simple_proxy_service);

    // 动态路由 转发
    let mut mut_route_proxy_service = pingora_proxy::http_proxy_service(&my_server.configuration,mut_route_proxy);

    mut_route_proxy_service.add_tcp("0.0.0.0:18081");

    my_server.add_service(mut_route_proxy_service);

    let admin_app = AdminApp{
        map:cache.clone(),
    };

    let mut admin_service = Service::new("adminApp".to_string(),Arc::new(admin_app));
    admin_service.add_tcp("0.0.0.0:8989");

    my_server.add_service(admin_service);

    my_server.run_forever();
}
