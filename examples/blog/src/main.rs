use prest::*;

embed_build_output_as!(BuiltAssets);

#[derive(Debug, Table, Default, Serialize, Deserialize)]
struct Todo {
    pub id: Uuid,
    pub custom: Inner,
    pub done: bool,
}

#[derive(Debug, Default, Serialize, Deserialize, PartialEq)]
struct Inner {
    a: String,
    b: NaiveDateTime,
}

fn main() {
    init!(tables Todo);

    // prepare table to showcase in the admin panel
    Todo {
        id: Uuid::now_v7(),
        custom: Inner {
            a: "' delete * from users;".to_owned(),
            b: Default::default(),
        },
        done: false,
    }
    .save()
    .expect("should save");

    blog::routes().embed(BuiltAssets).run()
}
