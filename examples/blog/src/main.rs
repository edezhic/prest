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
    // pre-init lazy statics
    let _ = *blog::EXAMPLES;
    let _ = *blog::INTERNALS;
    let _ = *blog::README;
    let _ = *blog::RUST;
    let _ = *blog::PREST_VERSION;

    // prepare table to showcase in the admin panel
    Todo {
        id: Uuid::now_v7(),
        custom: Default::default(),
        done: false,
    }
    .save()
    .expect("should save");

    blog::routes().embed(BuiltAssets).run()
}
