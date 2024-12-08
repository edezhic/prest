use prest::*;

pub fn shared_routes() -> Router {
    route("/", get(home))
}

async fn home() -> Markup {
    into_page(html!(
        a get="/todos" trigger="load" hx-push-url="true"
            after-request="if (!event.detail.successful) { document.getElementById('error').style.display = 'flex'; this.remove() }" {}
        div #"error" style="display: none;" {"Couldn't connect to the server :("}
    ))
    .await
}

pub async fn into_page(content: Markup) -> Markup {
    html! { html { (Head::with_title("Todo PWA app with auth and sync"))
        body $"max-w-screen-sm mx-auto px-8 mt-12 flex flex-col items-center" {
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
