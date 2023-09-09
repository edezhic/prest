
#[tokio::main]
async fn main() {
    let service = prest::Router::new().route("/", prest::get(homepage));

    // init http -> https redirection service
    tokio::spawn(prest::host::redirect_to_origin("https://localhost"));

    let tls_config = prest::host::RustlsConfig::from_pem_file("./cert.pem", "./key.pem")
        .await
        .unwrap();

    prest::host::serve_tls(service, tls_config).await.unwrap();
}

async fn homepage() -> impl prest::IntoResponse {
    prest::maud_to_response(maud::html!(
        html {
            head {
                title {"With TLS"}
                link rel="stylesheet" href="https://cdn.jsdelivr.net/npm/water.css@2/out/dark.css" {}
            }
            body {h1{"Check out the connection / protocol!"}}
        }
    ))
}  
