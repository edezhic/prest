use prest::*;

mod llm;

state!(LLM: Mutex<llm::Mistral> = { Mutex::new(llm::init()?) });

#[derive(serde::Deserialize)]
struct Prompt {
    pub content: String,
}

fn main() {
    route("/", get(page))
        .route(
            "/prompt",
            post(|Form(prompt): Form<Prompt>| async move {
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
                std::thread::spawn(move || {
                    *llm = llm::init().unwrap();
                });
                Redirect::to("/")
            }),
        )
        .run()
}

async fn page() -> Markup {
    let ready = if let Some(llm) = Lazy::get(&LLM) {
        llm.try_lock().is_ok()
    } else {
        std::thread::spawn(|| {
            Lazy::force(&LLM);
        });
        false
    };
    html!( html data-theme="dark" {
        (Head::with_title("With Mistral LLM"))
        body."max-w-screen-sm mx-auto mt-8" {
            @if ready {
                div {(history(false).await)}
            } @else {
                div hx-get="/" hx-target="body" hx-trigger="load delay:1s"{}
                span."loading loading-dots loading-lg"{}
            }
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
            ins hx-get="/more" hx-target="div" hx-trigger="load"{}
            span."loading loading-dots loading-sm"{}
            br{}
            button."btn btn-ghost" hx-get="/" hx-target="body" {"Pause"}
        }
        @else {
            form hx-post="/prompt" hx-target="div"  {
                input."input" type="text" name="content" placeholder="Prompt" required {}
                button."btn btn-outline btn-primary" type="submit" {(btn)}
            }
        }
        button."btn btn-ghost" hx-get="/reset" hx-target="body" {"Reset"}
    )
}
