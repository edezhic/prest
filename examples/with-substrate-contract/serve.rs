use prest::*;
use std::{sync::Arc, process::Command};
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

async fn home() -> Markup {
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
    html!(
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
        
    )
}

async fn test() -> Markup {
    let output = match Command::new("cargo").arg("test").current_dir(contract_path()).output() {
        Ok(output) => String::from_utf8(output.stdout).unwrap(),
        Err(e) => e.to_string()
    };
    html!(code {(PreEscaped(output))})
}

async fn build() -> Markup {
    let output = match Command::new("cargo").arg("contract").arg("build").current_dir(contract_path()).output() {
        Ok(output) => String::from_utf8(output.stdout).unwrap(),
        Err(e) => e.to_string()
    };
    html!(code {(PreEscaped(output))})
}

async fn deploy() -> Markup {
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

    html!(
        button hx-get="/read" {"Get the value from the contract"}
        button hx-get="/flip" {"Flip the value in the contract"}
        code #"output" {(PreEscaped(output))}
    )
}

async fn read() -> Markup {
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
    html!(code {(PreEscaped(output))})
}

async fn flip() -> Markup {
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
    html!(code {(PreEscaped(output))})
}

fn full_html(content: Markup) -> Markup {
    html!(
        html data-theme="dark" {
            (Head::default())
            body { main."container" hx-target="main" hx-swap="beforeend" {(content)} }
        }
    )
}

fn contract_path() -> std::path::PathBuf {
    let base = env!("CARGO_MANIFEST_DIR");
    let path = format!("{base}/contract/");
    std::fs::canonicalize(path).unwrap()
}