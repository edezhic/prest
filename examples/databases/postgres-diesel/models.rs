use diesel::{pg::Pg, prelude::*};
use prest::{Uuid, Serialize, Deserialize};

#[derive(Queryable, Selectable, Insertable, Serialize, Deserialize)]
#[diesel(table_name = crate::schema::todos)]
#[diesel(check_for_backend(Pg))]
pub struct Todo {
    #[serde(default = "Uuid::new_v4")]
    pub uuid: Uuid,
    #[serde(default)]
    pub task: String,
    #[serde(default)]
    pub done: bool,
}
