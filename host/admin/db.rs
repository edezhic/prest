use crate::*;

#[derive(Serialize)]
struct TableDescription {
    name: String,
    fields: Vec<FieldSchema>,
}

#[derive(Deserialize)]
struct TableQueryParams {
    offset: Option<usize>,
    limit: Option<usize>,
}

pub(crate) async fn schema() -> impl IntoResponse {
    let descriptions = DB
        .custom_schemas()
        .iter()
        .map(|s| TableDescription {
            name: s.name().to_owned(),
            fields: s.fields().to_vec(),
        })
        .collect::<Vec<_>>();
    Json(descriptions)
}

#[derive(Serialize)]
struct TableData {
    name: String,
    fields: Vec<FieldSchema>,
    rows: Vec<Vec<String>>,
    has_more: bool,
    total_pages: Option<usize>,
}

// table_data_json function removed - now handled within table_routes()

pub(crate) async fn db_page() -> Markup {
    html! {
        a _="on load call loadSchema() then remove me" {}
        div #db-container {
            // React component will be rendered here
        }
    }
}

pub(crate) fn table_routes() -> Router {
    let mut router = Router::new();
    for table in DB.custom_schemas() {
        let table_name = table.name().to_owned();
        router = router.route(
            table.relative_path(),
            get({
                let table_name = table_name.clone();
                move |Vals(params): Vals<TableQueryParams>| async move {
                    let table = DB
                        .custom_schemas()
                        .into_iter()
                        .find(|t| t.name() == table_name)
                        .ok_or_else(|| e!("Table not found: {}", table_name))?;

                    let offset = params.offset.unwrap_or(0);
                    let limit = params.limit.unwrap_or(20);

                    let (rows, has_more) = table.get_as_strings_paginated(offset, limit).await?;

                    ok(Json(TableData {
                        name: table.name().to_owned(),
                        fields: table.fields().to_vec(),
                        rows,
                        has_more,
                        total_pages: None, // We don't calculate total pages for performance
                    }))
                }
            })
            .put({
                let table_name = table_name.clone();
                move |req: Request| async move {
                    let table = DB
                        .custom_schemas()
                        .into_iter()
                        .find(|t| t.name() == table_name)
                        .ok_or_else(|| e!("Table not found: {}", table_name))?;
                    let id = table.save(req).await?;
                    ok(Json(serde_json::json!({ "success": true, "id": id })))
                }
            })
            .patch({
                let table_name = table_name.clone();
                move |req: Request| async move {
                    let table = DB
                        .custom_schemas()
                        .into_iter()
                        .find(|t| t.name() == table_name)
                        .ok_or_else(|| e!("Table not found: {}", table_name))?;
                    let id = table.save(req).await?;
                    ok(Json(serde_json::json!({ "success": true, "id": id })))
                }
            })
            .delete({
                move |req: Request| async move {
                    let table = DB
                        .custom_schemas()
                        .into_iter()
                        .find(|t| t.name() == table_name)
                        .ok_or_else(|| e!("Table not found: {}", table_name))?;
                    table.remove(req).await?;
                    ok(Json(serde_json::json!({ "success": true })))
                }
            }),
        );
    }
    router
}
