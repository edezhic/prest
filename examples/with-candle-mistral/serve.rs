mod llm;
use prest::*;
use std::sync::Arc;
use tokio::sync::Mutex;

static LLM: Lazy<Arc<Mutex<llm::Mistral>>> = Lazy::new(|| { 
    println!("Initiating a model...");
    Arc::new(Mutex::new(llm::init(Default::default()).unwrap()))
});

#[derive(serde::Deserialize)]
struct Prompt {
    pub content: String
}

#[tokio::main(flavor = "current_thread")]
async fn main() {
    let service = Router::new()
        .route("/", get(page))
        .route("/prompt", post(|Form(prompt): Form<Prompt>| async move {
            LLM.lock().await.prompt(&prompt.content).unwrap();
            history(true).await
        }))
        .route("/more", get(|| async { 
            let in_progress = LLM.lock().await.more();
            history(in_progress).await
        }))
        .route("/reset", get(|| async {
            let mut llm = LLM.lock().await;
            std::thread::spawn(move || {
                *llm = llm::init(Default::default()).unwrap();
            });
            Redirect::to("/")
        }));
    
    serve(service, Default::default()).await.unwrap();
}

async fn page() -> Markup {
    let ready = if let Some(llm) = Lazy::get(&LLM) {
        llm.try_lock().is_ok()
    } else {
        std::thread::spawn(|| { Lazy::force(&LLM); });
        false
    };
    html!( html data-theme="dark" {
        head {
            title {"LLM Chat"}
            link rel="stylesheet" href="https://cdn.jsdelivr.net/npm/@picocss/pico@1/css/pico.min.css"{}
            script src="https://unpkg.com/htmx.org@1.9.0" integrity="sha384-aOxz9UdWG0yBiyrTwPeMibmaoq07/d3a96GCbb9x60f3mOt5zwkjdbcHFnKH8qls" crossorigin="anonymous"{}
            script src="https://unpkg.com/hyperscript.org@0.9.11"{}
        }
        body."container" { 
            @if ready {
                article {(history(false).await)}
            } @else {
                article hx-get="/" hx-target="body" hx-trigger="load delay:1s" aria-busy="true"{}
            }
        }
    })
}

async fn history(in_progress: bool) -> Markup {
    let content = LLM.lock().await.history.clone();
    let fresh = content.len() == 0;
    html!(
        (PreEscaped(content))
        @if in_progress { 
            ins hx-get="/more" hx-target="article" hx-trigger="load" aria-busy="true" style="margin-left: 4px"{}
            button."secondary" hx-get="/" hx-target="body" style="margin-top: 2rem" {"Pause"}
        }
        @else { 
            form hx-post="/prompt" hx-target="article" style="margin-top: 1rem" {
                input type="text" id="content" name="content" placeholder="Prompt" required {}
                button type="submit" {(if fresh {"Start generating"} else {"Append and continue"})}
            }
        }
        button."secondary outline" hx-get="/reset" hx-target="body" style="margin-top: 1rem" {"Reset"}
    )
}
