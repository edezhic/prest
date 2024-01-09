Minimalistic todo app with storage based on [PostgreSQL](https://www.postgresql.org/) DB with [Diesel](https://github.com/launchbadge/sqlx) ORM - probably the first mature rust orm, currently used in [crates.io](https://crates.io/) and many other projects.

It has a number of advantages - stability, feature-completeness, plenty of configs and utility crates, and easy to use once you've set it up. High-performance and low-risk choice for lots of projects. However, the initial setup might be tricky because diesel crates link to host-provided db client libraries. 

This example is using [diesel-async](https://github.com/weiznich/diesel_async) because the rest of the server is async, and it's intended to showcase basic apis with a UI similar to other DB examples to easily compare their usage. To get started with this one you'll need:

1. `cargo install diesel_cli --no-default-features --features postgres` - install diesel CLI that you'll need for common diesel-related ops
2. `docker run -p 5432:5432 -e POSTGRES_PASSWORD=password -d postgres` - start a postgres instance in a docker container
3. `cd examples/databases/postgres-diesel && diesel setup --migration-dir="./migrations/"` - setup database & migrations
4. `cargo run -p postgres-diesel` - to start the example

It's powered by a few additional dependencies in the manifest:

{Cargo.toml}

A separate manifest for the diesel configuration:

{diesel.toml}

A model that defines the auto-generated schema:

{models.rs}

And a prest app that uses all of the above to manage todos:

{serve.rs}
