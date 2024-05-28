## po-hproxy

> `P`ing`O`ra
> 
> `H`ttp
> 
> `Proxy`


### simple
```bash
# 使用httpbin 验证转发 以及header修改
podman run -p 8080:80 docker.io/kong/httpbin:latest

RUST_LOG=INFO cargo run

# 该请求会转发到8080 并体系header修改
curl localhost:18081/get -v
```