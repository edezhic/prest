Minimalistic todo app powered by the [official rust mongo driver](https://github.com/mongodb/mongo-rust-driver). To get it running you can use the official mongo docker container: `docker run -p 27017:27017 -d mongo:latest`. In general working with mongo in rust is fairly straightforward thanks to the integration with serde's auto (de)serialization tools and driver's utility macros:

{Cargo.toml}

{src/main.rs}