mod entities;
mod migrator;

use entities::{prelude::*, *};
use prest::*;
use sea_orm::{ActiveModelTrait, ActiveValue, Database, DatabaseConnection, EntityTrait};
use sea_orm_migration::migrator::MigratorTrait;
use uuid::Uuid;

const DB_URL: &str = "postgres://postgres:password@localhost/prest";

static DB: Lazy<DatabaseConnection> = Lazy::new(|| {
    block_on(async {
        let db = Database::connect(DB_URL)
            .await
            .unwrap();
        migrator::Migrator::refresh(&db).await.unwrap();
        db
    })
});

#[derive(serde::Deserialize)]
struct NewTodo {
    task: String,
}

#[derive(serde::Deserialize)]
struct ToggleTodo {
    uuid: Uuid,
    done: bool,
}

#[derive(serde::Deserialize)]
struct DeleteTodo {
    uuid: Uuid,
}

fn main() {
    Router::new()
        .route(
            "/",
            get(|| async { html!(@for todo in Todos::find().all(&*DB).await.unwrap() {(todo)}) })
                .put(|Form(NewTodo { task }): Form<NewTodo>| async move {
                    todos::ActiveModel {
                        uuid: ActiveValue::Set(Uuid::new_v4()),
                        task: ActiveValue::Set(task),
                        done: ActiveValue::Set(false),
                    }
                    .insert(&*DB)
                    .await
                    .unwrap();
                    Redirect::to("/")
                })
                .patch(
                    |Form(ToggleTodo { uuid, done }): Form<ToggleTodo>| async move {
                        todos::ActiveModel {
                            uuid: ActiveValue::Set(uuid),
                            done: ActiveValue::Set(!done),
                            ..Default::default()
                        }
                        .update(&*DB)
                        .await
                        .unwrap();
                        Redirect::to("/")
                    },
                )
                .delete(|Form(DeleteTodo { uuid }): Form<DeleteTodo>| async move {
                    todos::ActiveModel {
                        uuid: ActiveValue::Set(uuid),
                        ..Default::default()
                    }
                    .delete(&*DB)
                    .await
                    .unwrap();
                    Redirect::to("/")
                }),
        )
        .wrap_non_htmx(page)
        .serve(ServeOptions::default())
}

impl Render for todos::Model {
    fn render(&self) -> Markup {
        let id = format!("uuid-{}", &self.uuid);
        let cb = format!("on change from .{id} trigger submit on #{id}");
        html!(
            div style="height: 64px; display: flex; justify-content: space-between; align-items: center;" {
                form #(id) hx-patch="/"  style="margin-bottom: 0px;" {
                    input type="hidden" name="uuid" value={(self.uuid)} {}
                    input type="hidden" name="done" value={(self.done)} {}
                    label {
                        @match self.done {
                            false => { input .(id) type="checkbox" _={(cb)} {} },
                            true  => { input .(id) type="checkbox" _={(cb)} checked {} },
                        }
                        {(self.task)}
                    }
                }
                form hx-delete="/" style="margin-bottom: 0px;" {
                    input type="hidden" name="uuid" value={(self.uuid)} {}
                    input."secondary outline" type="submit" value="Delete" style="margin-bottom: 0px;" {}
                }
            }
        )
    }
}

async fn page(content: Markup) -> Markup {
    html! { html data-theme="dark" {
        (Head::example("With SeaORM Postgres"))
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
