use prest::*;
use std::{sync::Arc, process::Command};
use once_cell::sync::Lazy;
use tokio::sync::Mutex;

static NODE: Lazy<Arc<Mutex<std::process::Child>>> = Lazy::new(|| { 
    println!("Starting the node...");
    let node = Arc::new(Mutex::new(
        Command::new("substrate-contracts-node").spawn().unwrap()
    ));
    std::thread::sleep(std::time::Duration::from_secs(1)); // wait for node to init
    node
});

static CONTRACT_ADDR: Lazy<Arc<Mutex<Option<String>>>> = Lazy::new(|| { Arc::new(Mutex::new(None)) });


#[tokio::main]
async fn main() {
    let service = Router::new()
        .route("/", get(home))
        .route("/build", get(build))
        .route("/test", get(test))
        .route("/deploy", get(deploy))
        .route("/read", get(read))
        .route("/flip", get(flip))
        .layer(Htmxify::wrap(full_html));
    serve(service, Default::default()).await.unwrap();
}

async fn home() -> Html<String> {
    let cargo_contract_error = if let Err(e) = Command::new("cargo").arg("contract").arg("help").output() {
        Some(e.to_string())
    } else {
        None
    };
    let substrate_node_error = if let Err(e) = Command::new("substrate-contracts-node").arg("chain-info").output() {
        Some(e.to_string())
    } else {
        None
    };
    let tooling_ready = cargo_contract_error.is_none() && substrate_node_error.is_none();
    let contract_ready = CONTRACT_ADDR.lock().await.is_some();
    Html(maud::html!(
        @if tooling_ready {
            ."grid" {
                button hx-get="/build" {"Build"}
                button hx-get="/test" {"Run tests"}
            }
            @if contract_ready {
                button hx-get="/read" {"Get the value from the contract"}
                button hx-get="/flip" {"Flip the value in the contract"}
            } @else {
                button #"deploy" hx-get="/deploy" hx-swap="innerHTML" {"Deploy to the local chain"}
            }
        } else {
            @if let Some(e) = cargo_contract_error {
                p {"Looks like cargo contract CLI is not installed, try " code{"cargo install cargo-contract"} " and refresh this page"}
                p {"The encountered error:"}
                code {(PreEscaped(e))}
            }
            @if let Some(e) = substrate_node_error {
                p {"Looks like substrate contracts node CLI is not installed, try " code{"cargo install contracts-node"} " and refresh this page"}
                p {"The encountered error:"}
                code {(PreEscaped(e))}
            }
        }
        
    ).0)
}

async fn test() -> Html<String> {
    let output = match Command::new("cargo").arg("test").current_dir(contract_path()).output() {
        Ok(output) => String::from_utf8(output.stdout).unwrap(),
        Err(e) => e.to_string()
    };
    Html(maud::html!(code {(PreEscaped(output))}).0)
}

async fn build() -> Html<String> {
    let output = match Command::new("cargo").arg("contract").arg("build").current_dir(contract_path()).output() {
        Ok(output) => String::from_utf8(output.stdout).unwrap(),
        Err(e) => e.to_string()
    };
    Html(maud::html!(code {(PreEscaped(output))}).0)
}

async fn deploy() -> Html<String> {
    Lazy::force(&NODE);
    let output = match Command::new("cargo")
        .arg("contract")
        .arg("instantiate")
        .args(["--constructor", "new"])
        .args(["--args", "false"])
        .args(["--suri", "//Alice"])
        .arg("-x")
        .arg("--skip-confirm")
        .current_dir(contract_path())
        .output() {
            Ok(output) => String::from_utf8(output.stdout).unwrap(),
            Err(e) => e.to_string()
    };
    let addr = output.split(" ").last().unwrap().replace("\n", "");
    let mut guard = CONTRACT_ADDR.lock().await;
    *guard = Some(addr);

    Html(maud::html!(
        button hx-get="/read" {"Get the value from the contract"}
        button hx-get="/flip" {"Flip the value in the contract"}
        code #"output" {(PreEscaped(output))}
    ).0)
}

async fn read() -> Html<String> {
    let addr = CONTRACT_ADDR.lock().await.clone().unwrap();
    let output = match Command::new("cargo")
        .arg("contract")
        .arg("call")
        .args(["--contract", &addr])
        .args(["--message", "get"])
        .args(["--suri", r#"//Alice"#])
        .current_dir(contract_path())
        .output() {
            Ok(output) => String::from_utf8(output.stdout).unwrap(),
            Err(e) => e.to_string()
    };
    Html(maud::html!(code {(PreEscaped(output))}).0)
}

async fn flip() -> Html<String> {
    let addr = CONTRACT_ADDR.lock().await.clone().unwrap();
    let output = match Command::new("cargo")
        .arg("contract")
        .arg("call")
        .args(["--contract", &addr])
        .args(["--message", "flip"])
        .args(["--suri", "//Alice"])
        .arg("-x")
        .arg("--skip-confirm")
        .current_dir(contract_path())
        .output() {
            Ok(output) => String::from_utf8(output.stdout).unwrap(),
            Err(e) => e.to_string()
    };
    Html(maud::html!(code {(PreEscaped(output))}).0)
}

fn full_html(content: String) -> String {
    maud::html!(
        html data-theme="dark" {
            (prest::maud_pwa_head("Prest Blog", Some(maud::html!(
                link rel="stylesheet" href="https://cdn.jsdelivr.net/npm/@picocss/pico@1/css/pico.min.css"{}
                script src="https://unpkg.com/htmx.org@1.9.0" integrity="sha384-aOxz9UdWG0yBiyrTwPeMibmaoq07/d3a96GCbb9x60f3mOt5zwkjdbcHFnKH8qls" crossorigin="anonymous"{}
            ))))
            body { main."container" hx-target="main" hx-swap="beforeend" {(PreEscaped(content))} }
        }
    ).0
}

fn contract_path() -> std::path::PathBuf {
    let base = env!("CARGO_MANIFEST_DIR");
    let path = format!("{base}/contract/");
    std::fs::canonicalize(path).unwrap()
}