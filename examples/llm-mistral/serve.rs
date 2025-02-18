use prest::*;

mod llm;

state!(LLM: Mutex<llm::Mistral> = { Mutex::new(llm::init()?) });

#[derive(Deserialize)]
struct Prompt {
    pub content: String,
}

#[init]
async fn main() -> Result {
    info!("Initializing LLM...");
    let _ = *LLM;

    route("/", get(page))
        .route(
            "/prompt",
            post(|Vals(prompt): Vals<Prompt>| async move {
                {
                    let mut llm = LLM.lock().await;
                    if llm.history.len() == 0 {
                        llm.prompt(&prompt.content).unwrap()
                    } else {
                        let prompt = "\n".to_owned() + &prompt.content;
                        llm.prompt(&prompt).unwrap()
                    }
                }
                history(true).await
            }),
        )
        .route(
            "/more",
            get(|| async {
                let in_progress = LLM.lock().await.more();
                history(in_progress).await
            }),
        )
        .route(
            "/reset",
            get(|| async {
                let mut llm = LLM.lock().await;
                *llm = llm::init().unwrap();
                Redirect::to("/")
            }),
        )
        .run()
        .await
}

async fn page() -> Markup {
    html!( html { (Head::with_title("With Mistral LLM"))
        body $"max-w-screen-sm mx-auto mt-8" {
            div {(history(false).await)}
            (Scripts::default())
        }
    })
}

async fn history(in_progress: bool) -> Markup {
    let content = LLM.lock().await.history.clone();
    let btn = if content.len() == 0 {
        "Start generating"
    } else {
        "Append and continue"
    };
    html!(
        (PreEscaped(content))
        @if in_progress {
            ins get="/more" target="div" trigger="load"{}
            span {"loading..."}
            br{}
            button get="/" target="body" {"Pause"}
        }
        @else {
            form post="/prompt" target="div"  {
                input type="text" name="content" placeholder="Prompt" required {}
                button type="submit" {(btn)}
            }
        }
        button get="/reset" target="body" {"Reset"}
    )
}
