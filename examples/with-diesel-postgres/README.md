is a todo app powered by [Diesel](https://github.com/launchbadge/sqlx) ORM with [PostgreSQL](https://www.postgresql.org/) DB

1. `cargo install diesel_cli --no-default-features --features postgres`
2. `docker run -p 5432:5432 -e POSTGRES_PASSWORD=password -d postgres`
3. `diesel setup --migration-dir="./examples/with-diesel-postgres/migrations/"`


great quote from https://diesel.rs/guides/getting-started:
`Unfortunately, the results won’t be terribly interesting ... still, we’ve written a decent amount of code, so let’s commit.`