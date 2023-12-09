Minimalistic todo app powered by the [official rust mongo driver](https://github.com/mongodb/mongo-rust-driver). To get it started you can simply run the official mongo docker container: `docker run -p 27017:27017 -d mongo:latest`.

In general working with mongo in rust is pretty easy thanks to serde's auto (de)serialization tools and driver's utility macros:

{Cargo.toml}

{serve.rs}