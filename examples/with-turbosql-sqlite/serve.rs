use prest::*;
use turbosql::{Turbosql, select, execute};

#[derive(Default, Turbosql, serde::Deserialize)]
pub struct Todo {
    pub rowid: Option<i64>,
    pub task: Option<String>,
    pub done: Option<bool>,
}

#[tokio::main]
async fn main() {
    let service = Router::new()
        .route(
            "/",
            get(html!(@for todo in select!(Vec<Todo>).unwrap() {(todo)}))
                .put(|Form(mut todo): Form<Todo>| async move {
                    todo.done = Some(false);
                    todo.insert().unwrap();
                    Redirect::to("/")
                })
                .patch(|Form(todo): Form<Todo>| async move {
                    todo.update().unwrap();
                    Redirect::to("/")
                })
                .delete(|Form(Todo { rowid, .. }): Form<Todo>| async move {
                    execute!("DELETE FROM todo WHERE rowid = " rowid.unwrap()).unwrap();
                    Redirect::to("/")
                }),
        )
        .layer(HTMXify::wrap(page));

    serve(service, Default::default()).await
}

impl Render for Todo {
    fn render(&self) -> Markup {
        let rowid = self.rowid.clone().unwrap();
        let task = self.task.clone().unwrap();
        let done = self.done.clone().unwrap();
        let id = format!("rowid-{rowid}");
        let cb = format!("on change from .{id} trigger submit on #{id}");
        html!(
            div style="height: 64px; display: flex; justify-content: space-between; align-items: center;" {
                form #(id) hx-patch="/"  style="margin-bottom: 0px;" {
                    input type="hidden" name="rowid" value={(rowid)} {}
                    input type="hidden" name="done" value={(done)} {}
                    label {
                        @match done {
                            false => { input .(id) type="checkbox" _={(cb)} {} },
                            true  => { input .(id) type="checkbox" _={(cb)} checked {} },
                        }
                        {(task)}
                    }
                }
                form hx-delete="/" style="margin-bottom: 0px;" {
                    input type="hidden" name="rowid" value={(rowid)} {}
                    input."secondary outline" type="submit" value="Delete" style="margin-bottom: 0px;" {}
                }
            }
        )
    }
}

pub fn page(content: Markup) -> Markup {
    html! { html data-theme="dark" {
        (Head::default().title("Todo"))
        body."container" hx-target="article" style="margin-top: 16px;" {
            form hx-put="/" _="on htmx:afterRequest reset() me" {
                label for="task" {"Task description:"}
                input type="text" name="task" {}
                button type="submit" {"Add"}
            }
            article {(content)}
            (Scripts::default())
        }
    }}
}
