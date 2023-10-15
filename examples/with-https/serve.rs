
use prest::*;

#[tokio::main]
async fn main() {
    let service = Router::new().route("/", get(homepage));

    // init http -> https redirection service
    tokio::spawn(redirect_to_origin("https://localhost"));

    let tls_config = RustlsConfig::from_pem_file("./cert.pem", "./key.pem")
        .await
        .unwrap();

    serve_tls(service, tls_config).await.unwrap();
}

async fn homepage() -> Markup {
    html!((DOCTYPE) html {
        (Head::default().title("With TLS"))
        body {h1{"Check out the connection / protocol!"}}
    })
}  
