#[derive(rust_embed::RustEmbed, Clone)]
#[folder = "./pub"]
struct Assets;

#[tokio::main]
async fn main() {
    let service = pwrs::Router::new()
        .merge(shared::service())
        .layer(pwrs::host::embed(Assets));

    // init http -> https redirection service
    tokio::spawn(pwrs::host::redirect_to_origin("https://localhost"));

    let tls_config = pwrs::host::RustlsConfig::from_pem_file("./cert.pem", "./key.pem")
        .await
        .unwrap();

    pwrs::host::serve_tls(service, tls_config).await.unwrap();
}
