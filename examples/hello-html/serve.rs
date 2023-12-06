use prest::*;

fn main() {
    Router::new()
        .route("/", get(home))
        .route("/page", post(submit))
        .wrap_non_htmx(wrapper)
        .serve(ServeOptions::default())
}

async fn home() -> Markup {
    html!(h1{"Homepage content"})
}

async fn submit(Form(x): Form<String>) {
    todo!("{x}")
}

async fn wrapper(content: Markup) -> Markup {
    html!(
        (Head::example("Hello HTML"))
        body {
            main ."view-transition" {(content)}
            footer {"Powered by prest"}
            (Scripts::default())
        }
    )
}
