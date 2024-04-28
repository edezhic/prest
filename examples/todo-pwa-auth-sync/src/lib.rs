use prest::*;

pub fn shared_routes() -> Router {
    route("/", get(home))
}

async fn home() -> Markup {
    into_page(html!(
        span."loading loading-spinner loading-lg" hx-get="/todos" hx-trigger="load" hx-swap="outerHTML" hx-push-url="true"
            hx-on--after-request="if (!event.detail.successful) { document.getElementById('alert').style.display = 'flex'; this.remove() }" {}
        div #"alert" role="alert" class="alert alert-error justify-center" style="display: none;" {"Couldn't fetch the todos :("}
    ))
    .await
}

pub async fn into_page(content: Markup) -> Markup {
    html! { html data-theme="dark" {
        (Head::with_title("Todo app"))
        body."max-w-screen-sm mx-auto mt-12 flex flex-col items-center" {
            (content)
            (Scripts::default())
        }
    }}
}

#[cfg(sw)]
#[wasm_bindgen(start)]
pub fn main() {
    shared_routes().handle_fetch_events()
}
