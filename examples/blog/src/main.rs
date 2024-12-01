use prest::*;

embed_build_output_as!(BuiltAssets);

#[derive(Debug, Table, Default, Serialize, Deserialize)]
struct Todo {
    #[serde(default = "Uuid::now_v7")]
    pub id: Uuid,
    pub task: String,
    #[serde(default)]
    pub done: bool,
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
        task: "example".into(),
        done: false,
    }
    .save()
    .expect("should save");

    blog::routes().embed(BuiltAssets).run()
}
