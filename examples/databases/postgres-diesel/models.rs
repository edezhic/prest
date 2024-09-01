use diesel::{pg::Pg, prelude::*};
use prest::{Deserialize, Serialize, Uuid};

#[derive(Queryable, Selectable, Insertable, Serialize, Deserialize)]
#[diesel(table_name = crate::schema::todos)]
#[diesel(check_for_backend(Pg))]
pub struct Todo {
    #[serde(default = "Uuid::now_v7")]
    pub uuid: Uuid,
    #[serde(default)]
    pub task: String,
    #[serde(default)]
    pub done: bool,
}
