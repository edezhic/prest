use prest::*;
use todo_pwa_auth::{into_page, shared_routes};

embed_build_output_as!(BuiltAssets);

#[derive(Table, Default, Serialize, Deserialize)]
#[serde(default)]
struct Todo {
    #[serde(default = "Uuid::now_v7")]
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
        .embed(BuiltAssets)
        .run();
}

async fn todos(auth: Auth) -> Markup {
    html!(
        @if let Some(user) = auth.user {
            form hx-put="/todos" hx-target="#list" hx-swap="beforeend" hx-on--after-request="this.reset()" {
                input $"border rounded-md" type="text" name="task" {}
                button $"ml-4" type="submit" {"Add"}
            }
            #"list" $"w-full" {@for todo in Todo::find_by_owner(&user.id) {(todo)}}
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
    )
}

async fn add(user: User, Form(mut todo): Form<Todo>) -> Result<Markup> {
    todo.owner = user.id;
    Ok(todo.save()?.render())
}

async fn toggle(user: User, Form(mut todo): Form<Todo>) -> Result<Markup> {
    if !todo.check_owner(user.id)? {
        return Err(Error::Unauthorized);
    }
    Ok(todo.update_done(!todo.done)?.render())
}

async fn delete(user: User, Form(todo): Form<Todo>) -> Result {
    if !todo.check_owner(user.id)? {
        return Err(Error::Unauthorized);
    }
    Ok(todo.remove()?)
}

impl Render for Todo {
    fn render(&self) -> Markup {
        html! {
            $"flex items-center" hx-target="this" hx-swap="outerHTML" hx-vals=(json!(self)) {
                input type="checkbox" hx-patch="/todos" checked[self.done] {}
                label $"ml-4 text-lg" {(self.task)}
                button $"ml-auto" hx-delete="/todos" {"Delete"}
            }
        }
    }
}
