use diesel::pg::PgConnection;
use diesel::prelude::*;
use prest::*;

#[tokio::main]
async fn main() {
    set_dot_env_variables();
    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    PgConnection::establish(&database_url)
        .unwrap_or_else(|_| panic!("Error connecting to {}", database_url));

    let service = Router::new().route("/", template!(h1{"Hello world!"}));
    serve(service, Default::default()).await.unwrap();
}
