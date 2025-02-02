use prest::*;
use todo_pwa_auth::{into_page, shared_routes};

embed_build_output_as!(BuiltAssets);

#[derive(Storage, Default, Serialize, Deserialize)]
#[serde(default)]
struct Todo {
    #[serde(default = "Uuid::now_v7")]
    pub id: Uuid,
    #[serde(default)]
    pub owner: Uuid,
    pub task: String,
    pub done: bool,
}

impl Render for Todo {
    fn render(&self) -> Markup {
        html! {
            $"flex justify-between items-center" swap-this vals=(json!(self)) {
                input type="checkbox" patch="/todos" checked[self.done] {}
                label $"ml-4 text-lg" {(self.task)}
                button $"ml-auto" delete="/todos" {"Delete"}
            }
        }
    }
}

#[init]
async fn main() -> Result {
    shared_routes()
        .route(
            "/todos",
            get(|auth: Auth| async move {
                ok(html!(
                    @if let Some(user) = auth.user {
                        form put="/todos" into-end-of="#list" after-request="this.reset()" {
                            input $"border rounded-md" type="text" name="task" {}
                            button $"ml-4" type="submit" {"Add"}
                        }
                        div #list $"w-full" {(Todo::select_by_owner(&user.id).await?)}
                    } @else {
                        @if *WITH_GOOGLE_AUTH {
                            a $"p-4 border rounded-md" href=(GOOGLE_LOGIN_ROUTE) {"Login with Google"}
                            div {"OR"}
                        }
                        form $"flex flex-col gap-4 items-center" method="POST" action=(LOGIN_ROUTE) {
                            input $"border rounded-md mx-4" type="text" name="username" placeholder="username" {}
                            input $"border rounded-md mx-4" type="password" name="password" placeholder="password" {}
                            input type="hidden" name="signup" value="true" {}
                            button $"ml-4" type="submit" {"Sign in / Sign up"}
                        }
                    }
                ))
            })
                .put(|user: User, Vals(mut todo): Vals<Todo>| async move {
                    todo.owner = user.id;
                    ok(todo.save().await?.render())
                })
                .patch(|user: User, Vals(mut todo): Vals<Todo>| async move {
                    if !todo.check_owner(user.id).await? {
                        return Err(Error::Unauthorized);
                    }
                    Ok(todo.update_done(!todo.done).await?.render())
                })
                .delete(|user: User, Vals(todo): Vals<Todo>| async move {
                    if !todo.check_owner(user.id).await? {
                        return Err(Error::Unauthorized);
                    }
                    Ok(todo.remove().await?)
                }),
        )
        .wrap_non_htmx(into_page)
        .embed(BuiltAssets)
        .run()
        .await
}
