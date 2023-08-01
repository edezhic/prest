#[tokio::main]
async fn main() {
    pwrs_host::init_logging();
    pwrs_host::serve(lib::service(), 80).await.unwrap();
}
