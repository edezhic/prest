Minimalistic todo app powered by [SeaORM](https://www.sea-ql.org/SeaORM/)-based connection to Postgres. Seaorm is async-first, dynamic and includes powerful tools for testing. Also it supports [Seaography](https://www.sea-ql.org/SeaORM/docs/seaography/seaography-intro/) - library that can automatically build graphql endpoints from seaorm entities. Overall [SeaQL](https://www.sea-ql.org/) provides pretty much everything necessary to work with postgres, mysql and sqlite, and currently it is the main competitor of [diesel](https://prest.blog/with-diesel-postgres).

To work with it you'll need the [sea-orm-cli](https://www.sea-ql.org/SeaORM/docs/generate-entity/sea-orm-cli/), running postgres instance and a connection string defined in `.env` or environment variables. Usually entities are generated using the cli from the database schema - you write migrations, run them, invoke cli and get the entities:

{Cargo.toml}

{migrator.rs}

{serve.rs}