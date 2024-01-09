Minimalistic todo app powered by [SQLx](https://github.com/launchbadge/sqlx)-based connection to the [SQLite](https://www.sqlite.org/index.html) DB. The core feature of sqlx is that it's macros can run queries during the build time to test their overall correctness. Also, it's a pretty good choice if you prefer good old sql.

{Cargo.toml}

{build.rs}

{migrations/20220718111257_todos.sql}

{serve.rs}