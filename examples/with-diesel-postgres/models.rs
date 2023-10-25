use diesel::{prelude::*, pg::Pg};
use uuid::Uuid;

#[derive(Queryable, Selectable, Insertable, serde::Deserialize)]
#[diesel(table_name = crate::schema::todos)]
#[diesel(check_for_backend(Pg))]
pub struct Todo {
    #[serde(default = "new_uuid")]
    pub uuid: Uuid,
    #[serde(default)]
    pub task: String,
    #[serde(default)]
    pub done: bool,
}

fn new_uuid() -> Uuid {
    uuid::Uuid::new_v4()
}