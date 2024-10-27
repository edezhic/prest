mod entities;
mod migrator;

use prest::*;

use entities::{prelude::*, *};
use sea_orm::{ActiveModelTrait, ActiveValue, Database, DatabaseConnection, EntityTrait};
use sea_orm_migration::migrator::MigratorTrait;

state!(DB: DatabaseConnection = async {
    let db = Database::connect("postgres://postgres:password@localhost/prest").await?;
    migrator::Migrator::refresh(&db).await?;
    db
});

#[derive(Deserialize)]
struct NewTodo {
    task: String,
}

#[derive(Deserialize)]
struct ToggleTodo {
    uuid: Uuid,
    done: bool,
}

#[derive(Deserialize)]
struct DeleteTodo {
    uuid: Uuid,
}

fn main() {
    route(
        "/",
        get(|| async { html!(@for todo in Todos::find().all(&*DB).await.unwrap() {(todo)}) })
            .put(|Form(NewTodo { task }): Form<NewTodo>| async move {
                todos::ActiveModel {
                    uuid: ActiveValue::Set(Uuid::now_v7()),
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
    .run()
}

impl Render for todos::Model {
    fn render(&self) -> Markup {
        html!(
            $"flex items-center" hx-vals=(json!(self)) {
                input type="checkbox" hx-patch="/" checked[self.done] {}
                label $"ml-4 text-lg" {(self.task)}
                button $"ml-auto" hx-delete="/" {"Delete"}
            }
        )
    }
}

async fn page(content: Markup) -> Markup {
    html! { html data-theme="dark" {
        (Head::with_title("With SeaORM Postgres"))
        body."max-w-screen-sm mx-auto mt-12" hx-target="div" {
            form $"flex gap-4 justify-center" hx-put="/" hx-target="#list" hx-swap="beforeend" hx-on--after-request="this.reset()" {
                input $"border rounded-md" type="text" name="task" {}
                button type="submit" {"Add"}
            }
            div id="list" $"w-full" {(content)}
            (Scripts::default())
        }
    }}
}
