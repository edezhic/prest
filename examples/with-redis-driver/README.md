Minimalistic todo app powered by the [redis client](https://github.com/redis-rs/redis-rs). Created just to showcase how to connect to a redis instance from rust and use that connection in handlers. But overall redis is not designed for this type of apps at all. 

To get it started locally you can use the official redis docker image: `docker run -p 6379:6379 -d redis:latest`

{Cargo.toml}

{serve.rs}