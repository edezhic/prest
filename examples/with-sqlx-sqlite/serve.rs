use prest::*;
use sqlx::{migrate, Pool, Sqlite};

static DB: Lazy<Pool<Sqlite>> = Lazy::new(|| {
    sqlx::sqlite::SqlitePoolOptions::new()
        .connect_lazy("sqlite::memory:")
        .expect("successful DB connection")
});

#[tokio::main]
async fn main() {
    start_printing_traces();

    migrate!().run(&*DB).await.unwrap();

    let service = Router::new().route("/", template!(b{"All clear"}));
    serve(service, Default::default()).await.unwrap();
}
