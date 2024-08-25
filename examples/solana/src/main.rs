#![allow(dead_code)]

use prest::*;

use anchor_client::{
    solana_sdk::{
        pubkey::Pubkey,
        signature::{Keypair, Signer},
        system_program,
    }, Client, ClientError, Cluster::Localnet, Program
};
use std::process::Command;
use todo_solana::{accounts, instruction, TodoList, ID as PROGRAM_ID};

state!(KEYPAIR: Arc<Keypair> = { Arc::new(Keypair::new()) });
state!(PROGRAM: Program<Arc<Keypair>> = {
    Client::new(Localnet, KEYPAIR.clone()).program(PROGRAM_ID)?
});
state!(TODO_LIST_PDA: Pubkey = { Pubkey::find_program_address(&[KEYPAIR.pubkey().as_ref()], &PROGRAM.id()).0 });

fn main() {
    init!();
    setup_local_solana_environment();
    route("/", get(list).post(add))
        .route("/toggle/:index", get(toggle))
        .route("/delete/:index", get(delete))
        .wrap_non_htmx(into_page)
        .run()
}

fn setup_local_solana_environment() {
    if let Err(e) = Command::new("solana").arg("-v").output() {
        const INSTALL_SOLANA_UNIX: &str =
            r#"sh -c "$(curl -sSfL https://release.solana.com/v1.18.18/install)""#;
        const DOWNLOAD_SOLANA_WINDOWS: &str = r#"cmd /c "curl https://release.solana.com/v1.18.18/solana-install-init-x86_64-pc-windows-msvc.exe --output C:\solana-install-tmp\solana-install-init.exe --create-dirs""#;
        const INSTALL_SOLANA_WINDOWS: &str =
            r#"C:\solana-install-tmp\solana-install-init.exe v1.18.18"#;

        let err =
            format!("{e}\nLooks like Solana CLI is not installed, you can install it with:\n");
        #[cfg(target_os = "windows")]
        error!("{err}{DOWNLOAD_SOLANA_WINDOWS}\nand\n{INSTALL_SOLANA_WINDOWS}");
        #[cfg(not(target_os = "windows"))]
        error!("{err}{INSTALL_SOLANA_UNIX}");
        return;
    }

    std::thread::spawn(|| {
        Command::new("solana-test-validator")
            .args(&["--reset", "--ledger", "./target/ledger"])
            .stdout(std::process::Stdio::null())
            .spawn()
            .expect("Failed to start solana-test-validator")
    });
    info!("Awaiting local cluster start...");
    std::thread::sleep(std::time::Duration::from_secs(5));

    info!("Building the program...");
    Command::new("cargo")
        .args(&["build-sbf", "--", "--lib"])
        .output()
        .expect("Program should build successfully");

    info!("Deploying the program...");
    Command::new("solana")
        .args(&["program", "deploy", "./target/deploy/todo_solana.so"])
        .output()
        .expect("Program deploy should be successful");

    info!("Airdropping SOL to the test keypair...");
    Command::new("solana")
        .args(&["airdrop", "10", &KEYPAIR.pubkey().to_string()])
        .args(&["--commitment", "finalized"])
        .output()
        .expect("Successful SOL airdrop");
}

async fn list() -> Markup {
    let my_todos = match PROGRAM.account::<TodoList>(*TODO_LIST_PDA).await {
        Ok(list) => list.items,
        Err(ClientError::AccountNotFound) => vec![],
        Err(e) => {
            return html!{"Can't get todo list:" br; code{(e)}};
        }
    };
    
    html!(@for (index, todo) in my_todos.iter().enumerate() {
        $"flex justify-between items-center" hx-target="#list" hx-swap="outerHTML" {
            input type="checkbox" hx-get=(format!("/toggle/{index}")) checked[todo.done] {}
            label $"ml-4 text-lg" {(todo.task)}
            button $"ml-auto" hx-get=(format!("/delete/{index}")) {"Delete"}
        }
    })
}

async fn into_page(content: Markup) -> Markup {
    html! {(DOCTYPE) html {(Head::with_title("With Solana storage"))
        body $"max-w-screen-sm px-8 mx-auto mt-12 flex flex-col items-center" {
            form method="POST" hx-target="#list" hx-on--after-request="this.reset()" {
                input $"border rounded-md" type="text" name="task" {}
                button $"ml-4" type="submit" {"Add"}
            }
            #"list" $"w-full" {(content)}
            (Scripts::default())
        }
    }}
}

#[derive(Serialize, Deserialize)]
struct NewTodo {
    task: String,
}

async fn add(Form(todo): Form<NewTodo>) -> Markup {
    if let Err(e) = PROGRAM
        .request()
        .accounts(accounts::AddTodo {
            list: *TODO_LIST_PDA,
            owner: PROGRAM.payer(),
            system_program: system_program::ID,
        })
        .args(instruction::AddTodo { task: todo.task })
        .send()
        .await
    {
        error!("couldn't create todo: {e}");
    };
    list().await
}

async fn toggle(Path(index): Path<u32>) -> Markup {
    if let Err(e) = PROGRAM
        .request()
        .accounts(accounts::ToggleTodo {
            list: *TODO_LIST_PDA,
            owner: PROGRAM.payer(),
        })
        .args(instruction::ToggleTodo { index })
        .send()
        .await
    {
        error!("couldn't toggle todo: {e}");
    };
    list().await
}

async fn delete(Path(index): Path<u32>) -> Markup {
    if let Err(e) = PROGRAM
        .request()
        .accounts(accounts::DeleteTodo {
            list: *TODO_LIST_PDA,
            owner: PROGRAM.payer(),
        })
        .args(instruction::DeleteTodo { index })
        .send()
        .await
    {
        error!("couldn't delete todo: {e}");
    };
    list().await
}


