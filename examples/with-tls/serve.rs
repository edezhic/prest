
#[tokio::main]
async fn main() {
    let service = pwrs::Router::new().route("/", pwrs::get(homepage));

    // init http -> https redirection service
    tokio::spawn(pwrs::host::redirect_to_origin("https://localhost"));

    let tls_config = pwrs::host::RustlsConfig::from_pem_file("./cert.pem", "./key.pem")
        .await
        .unwrap();

    pwrs::host::serve_tls(service, tls_config).await.unwrap();
}

async fn homepage() -> impl pwrs::IntoResponse {
    pwrs::maud_to_response(maud::html!(
        html {
            head {
                title {"With TLS"}
                link rel="stylesheet" href="https://cdn.jsdelivr.net/npm/water.css@2/out/dark.css" {}
            }
            body {h1{"Check out the connection / protocol!"}}
        }
    ))
}  
