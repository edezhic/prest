
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

async fn homepage() -> impl IntoResponse {
    maud_to_response(maud::html!(
        html {
            head {
                title {"With TLS"}
                link rel="stylesheet" href="https://cdn.jsdelivr.net/npm/water.css@2/out/dark.css" {}
            }
            body {h1{"Check out the connection / protocol!"}}
        }
    ))
}  
