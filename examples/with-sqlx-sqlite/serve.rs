use async_lazy::Lazy;
use prest::*;
use sqlx::{Pool, Sqlite};

/*
static DB: Lazy<Pool<Sqlite>> = Lazy::new(|| {
    Box::pin(async {
        sqlx::sqlite::SqlitePoolOptions::new()
            .connect("sqlite::memory:")
            .await
            .unwrap()
    })
});
 */

#[tokio::main]
async fn main() {
    start_printing_traces();

    let DB = sqlx::sqlite::SqlitePoolOptions::new()
    .connect("sqlite::memory:")
    .await
    .unwrap();

    // force connection pool initialization
    //tokio::spawn(async { *DB.force().await }).await.unwrap();

    if let Err(e) = sqlx::migrate!().run(&DB).await {
        panic!("migrations failed due to {e:?}");
    }

    let service = Router::new().route("/", template!(b{"All clear"}));
    serve(service, Default::default()).await.unwrap();
}
