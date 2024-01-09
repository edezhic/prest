use prest::*;
use todo_pwa_auth::{into_page, shared_routes};

embed_build_output_as!(BuiltAssets);

#[derive(Table, Default, serde::Serialize, serde::Deserialize)]
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
    Todo::migrate();
    shared_routes()
        .route("/todos", get(todos).put(add).patch(toggle).delete(delete))
        .wrap_non_htmx(into_page)
        .embed(BuiltAssets)
        .run();
}

async fn todos(auth: Auth) -> Markup {
    if let Some(user) = auth.user {
        html!(
            form hx-put="/todos" hx-target="div" hx-swap="beforeend" hx-on--after-request="this.reset()" {
                input."input input-bordered input-primary" type="text" name="task" {}
                button."btn btn-outline btn-primary ml-4" type="submit" {"Add"}
            }
            ."w-full" {@for todo in Todo::find_by_owner(&user.id) {(todo)}}
        )
    } else {
        html!(
            @if *WITH_GOOGLE_AUTH {
                a."btn btn-ghost" href="/auth/google" {"Login with Google"}
                ."divider" {"OR"}
            }
            ."flex" {
                ."bg-base-100 border-base-300 rounded-box p-6" {
                    form."flex flex-col gap-4 items-center" method="POST" action="/auth/username_password/signin" {
                        input."input input-bordered input-primary" type="text" name="username" placeholder="username" {}
                        input."input input-bordered input-primary" type="password" name="password" placeholder="password" {}
                        button."btn btn-outline btn-primary ml-4" type="submit" {"Sign in"}
                    }
                }
                ."divider divider-horizontal" {}
                ."bg-base-100 border-base-300 rounded-box p-6" {
                    form."flex flex-col gap-4 items-center" method="POST" action="/auth/username_password/signup" {
                        input."input input-bordered input-primary" type="text" name="username" placeholder="username" {}
                        input."input input-bordered input-primary" type="password" name="password" placeholder="password" {}
                        button."btn btn-outline btn-primary ml-4" type="submit" {"Sign up"}
                    }
                }
            }
        )
    }
}

async fn add(user: User, Form(mut todo): Form<Todo>) -> Markup {
    todo.owner = user.id;
    todo.save().unwrap().render()
}

async fn toggle(user: User, Form(mut todo): Form<Todo>) -> impl IntoResponse {
    if todo.owner == user.id {
        todo.update_done(!todo.done)
            .unwrap()
            .render()
            .into_response()
    } else {
        StatusCode::UNAUTHORIZED.into_response()
    }
}
async fn delete(user: User, Form(todo): Form<Todo>) -> impl IntoResponse {
    if todo.owner == user.id {
        todo.remove().unwrap();
        StatusCode::OK
    } else {
        StatusCode::UNAUTHORIZED
    }
}

impl Render for Todo {
    fn render(&self) -> Markup {
        html! {
            ."flex items-center" hx-target="this" hx-swap="outerHTML" hx-vals=(json!(self)) {
                input."toggle toggle-primary" type="checkbox" hx-patch="/todos" checked[self.done] {}
                label."ml-4 text-lg" {(self.task)}
                button."btn btn-ghost ml-auto" hx-delete="/todos" {"Delete"}
            }
        }
    }
}
