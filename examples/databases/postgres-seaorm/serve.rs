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

#[init]
async fn main() -> Result {
    route(
        "/",
        get(|| async { Todos::find().all(&*DB).await.unwrap().render() })
            .put(|Vals(NewTodo { task }): Vals<NewTodo>| async move {
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
                |Vals(ToggleTodo { uuid, done }): Vals<ToggleTodo>| async move {
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
            .delete(|Vals(DeleteTodo { uuid }): Vals<DeleteTodo>| async move {
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
    .await
}

impl Render for todos::Model {
    fn render(&self) -> Markup {
        html!(
            $"flex items-center" vals=(json!(self)) {
                input type="checkbox" patch="/" checked[self.done] {}
                label $"ml-4 text-lg" {(self.task)}
                button $"ml-auto" detele="/" {"Delete"}
            }
        )
    }
}

async fn page(content: Markup) -> Markup {
    html! { html data-theme="dark" {
        (Head::with_title("With SeaORM Postgres"))
        body $"max-w-screen-sm mx-auto mt-12" target="div" {
            form $"flex gap-4 justify-center" put="/" into-end-of="#list" after-request="this.reset()" {
                input $"border rounded-md" type="text" name="task" {}
                button type="submit" {"Add"}
            }
            div #list $"w-full" {(content)}
            (Scripts::default())
        }
    }}
}
