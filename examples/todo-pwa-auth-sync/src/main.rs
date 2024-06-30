use async_broadcast::{broadcast, Receiver, Sender};
use prest::*;
use todo_pwa_auth_sync::{into_page, shared_routes};

embed_build_output_as!(BuiltAssets);

state!(BROADCAST: (Sender<BroadcastMsg>, Receiver<BroadcastMsg>) = { broadcast(1000) });

#[derive(Clone)]
struct BroadcastMsg {
    pub event: String,
    pub data: Option<Todo>,
}

#[derive(Table, Clone, Default, Serialize, Deserialize)]
#[serde(default)]
struct Todo {
    #[serde(default = "Uuid::new_v4")]
    pub id: Uuid,
    #[serde(default)]
    pub owner: Uuid,
    pub task: String,
    pub done: bool,
}

fn main() {
    init!(tables Todo);
    shared_routes()
        .route("/todos", get(todos).put(add).patch(toggle).delete(delete))
        .wrap_non_htmx(into_page)
        .route("/todos/subscribe", get(todos_subscribe))
        .embed(BuiltAssets)
        .run();
}

impl Todo {
    fn render(&self, user: &Option<User>) -> Markup {
        let owned = match user {
            Some(user) => user.id == self.owner,
            None => false,
        };
        html! {
            ."flex items-center" sse-swap=(self.id) hx-swap="outerHTML" hx-vals=(json!(self)) {
                input."toggle toggle-primary" type="checkbox" hx-patch="/todos" disabled[!owned] checked[self.done] {}
                label."ml-4 text-lg" {(self.task)}
                button."btn btn-ghost ml-auto"  hx-delete="/todos" disabled[!owned] {"Delete"}
            }
        }
    }
}

async fn todos(auth: Auth) -> Markup {
    html!(
        @if auth.user.is_some() {
            form hx-put="/todos" hx-swap="none" hx-on--after-request="this.reset()" {
                input."input input-bordered input-primary" type="text" name="task" {}
                button."btn btn-outline btn-primary ml-4" type="submit" {"Add"}
            }
        } @else {
            @if *WITH_GOOGLE_AUTH {
                a."btn btn-ghost" href=(GOOGLE_LOGIN_ROUTE) {"Login with Google"}
                ."divider" {"OR"}
            }
            form."flex flex-col gap-4 items-center" method="POST" action=(LOGIN_ROUTE) {
                input."input input-bordered input-primary" type="text" name="username" placeholder="username" {}
                input."input input-bordered input-primary" type="password" name="password" placeholder="password" {}
                input type="hidden" name="signup" value="true" {}
                button."btn btn-outline btn-primary ml-4" type="submit" {"Sign in / Sign up"}
            }
        }
        #"todos" ."w-full" hx-ext="sse" sse-connect="/todos/subscribe" sse-swap="add" hx-swap="beforeend" {
            @for item in Todo::find_all() {(item.render(&auth.user))}
        }
    )
}

async fn todos_subscribe(auth: Auth) -> Sse<impl Stream<Item = SseItem>> {
    let stream = BROADCAST.1.new_receiver().map(move |msg| {
        let data = match msg.data {
            Some(todo) => todo.render(&auth.user).0,
            None => "".to_owned(),
        };
        SseEvent::default().event(msg.event.as_str()).data(data)
    });
    Sse::new(stream.map(Ok)).keep_alive(SseKeepAlive::default())
}

async fn add(user: User, Form(mut todo): Form<Todo>) -> Result {
    todo.owner = user.id;
    todo.save()?;
    BROADCAST
        .0
        .broadcast_direct(BroadcastMsg {
            event: "add".to_owned(),
            data: Some(todo),
        })
        .await
        .map_err(|e| anyhow!("{e}"))?;
    Ok(())
}

async fn toggle(user: User, Form(mut todo): Form<Todo>) -> Result {
    if !todo.check_owner(user.id)? {
        return Err(Error::Unauthorized);
    }
    todo.update_done(!todo.done)?;
    BROADCAST
        .0
        .broadcast_direct(BroadcastMsg {
            event: todo.id.to_string(),
            data: Some(todo),
        })
        .await
        .map_err(|e| anyhow!("{e}"))?;
    Ok(())
}
async fn delete(user: User, Form(todo): Form<Todo>) -> Result {
    if !todo.check_owner(user.id)? {
        return Err(Error::Unauthorized);
    }
    todo.remove()?;
    BROADCAST
        .0
        .broadcast_direct(BroadcastMsg {
            event: todo.id.to_string(),
            data: None,
        })
        .await
        .map_err(|e| anyhow!("{e}"))?;
    Ok(())
}
