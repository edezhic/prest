use prest::*;
use todo_pwa_auth_sync::{into_page, shared_routes};

embed_build_output_as!(BuiltAssets);

state!(TODO_UPDATES: SseBroadcast<Option<Todo>> = { SseBroadcast::default() });

#[derive(Table, Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default)]
struct Todo {
    #[serde(default = "Uuid::now_v7")]
    pub id: Uuid,
    #[serde(default)]
    pub owner: Uuid,
    pub task: String,
    pub done: bool,
}

impl Todo {
    fn render_for(&self, maybe_user: &Option<User>) -> Markup {
        let owned = maybe_user
            .as_ref()
            .map(|u| u.id == self.owner)
            .unwrap_or(false);

        html! {
            $"flex items-center" sse-swap=(self.id) hx-swap="outerHTML" hx-vals=(json!(self)) {
                input type="checkbox" hx-patch="/todos" disabled[!owned] checked[self.done] {}
                label $"ml-4 text-lg" {(self.task)}
                button $"ml-auto" hx-delete="/todos" disabled[!owned] {"Delete"}
            }
        }
    }
}

fn main() {
    init!(tables Todo);
    shared_routes()
        .route(
            "/todos",
            get(|auth: Auth| async move {
                html!(
                    @if auth.user.is_some() {
                        form hx-put="/todos" hx-swap="none" hx-on--after-request="this.reset()" {
                            input $"border rounded-md" type="text" name="task" {}
                            button $"ml-4" type="submit" {"Add"}
                        }
                    } @else {
                        form $"flex flex-col gap-4 items-center" method="POST" action=(LOGIN_ROUTE) {
                            input $"border rounded-md mx-4" type="text" name="username" placeholder="username" {}
                            input $"border rounded-md mx-4" type="password" name="password" placeholder="password" {}
                            input type="hidden" name="signup" value="true" {}
                            button $"ml-4" type="submit" {"Sign in / Sign up"}
                        }
                    }
                    #"todos" $"w-full" hx-ext="sse" sse-connect="/todos/subscribe" sse-swap="add" hx-swap="beforeend" {
                        @for item in Todo::find_all() {(item.render_for(&auth.user))}
                    }
                )
            })
                .put(|user: User, Form(mut todo): Form<Todo>| async move {
                    todo.owner = user.id;
                    todo.save()?;
                    TODO_UPDATES.send("add", Some(todo)).await?;
                    OK
                })
                .patch(|user: User, Form(mut todo): Form<Todo>| async move {
                    if !todo.check_owner(user.id)? {
                        return Err(Error::Unauthorized);
                    }
                    todo.update_done(!todo.done)?;
                    TODO_UPDATES.send(todo.id.to_string(), Some(todo)).await?;
                    OK
                })
                .delete(|user: User, Query(todo): Query<Todo>| async move {
                    if !todo.check_owner(user.id)? {
                        return Err(Error::Unauthorized);
                    }
                    todo.remove()?;
                    TODO_UPDATES.send(todo.id.to_string(), None).await?;
                    OK
                }),
        )
        .wrap_non_htmx(into_page)
        .route(
            "/todos/subscribe",
            get(|auth: Auth| async {
                TODO_UPDATES.stream_and_render(move |_event, todo| {
                    todo.map(|t| t.render_for(&auth.user)).unwrap_or_default()
                })
            }),
        )
        .embed(BuiltAssets)
        .run();
}
