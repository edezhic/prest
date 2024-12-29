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

#[init]
async fn main() -> Result {
    // example table with data to showcase in the admin panel
    Todo {
        id: Uuid::now_v7(),
        custom: Inner {
            a: "v5 release".to_owned(),
            b: Default::default(),
        },
        done: false,
    }
    .save()
    .await?;
    
    blog::routes().embed(BuiltAssets).run().await
}
