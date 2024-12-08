use host::admin::{DELETE_SVG, DONE_SVG, EDIT_SVG};

use crate::*;

pub(crate) async fn db_page() -> Markup {
    let tables = html! {
        @for table in DB_SCHEMA.tables() {
            $"font-bold text-lg" {(table.name())}
            a get=(table.full_path()) trigger="load" swap-this {}
        }
    };
    html!((tables))
}

pub(crate) fn db_routes() -> Router {
    let mut router = route("/", get(db_page));
    for table in DB_SCHEMA.tables() {
        let get_by_id_path = format!("{}/:id", table.relative_path());
        router = router
            .route(
                table.relative_path(),
                get(|| async {
                    let rows = table
                        .get_all()
                        .await
                        .into_iter()
                        .map(|row| view_row(table, row));
                    html!(
                        table $"w-full font-mono text-[0.5rem] lg:text-sm" {
                            @let columns = table.schema().iter().map(|c| (c.name, c.rust_type));
                            @for (name, rust_type) in columns {th {(name)" ("(rust_type)")"}}
                            @for row in rows {(row)}
                        }
                    )
                }),
            )
            .route(
                &get_by_id_path,
                get(|Path(id): Path<String>| async {
                    let row = table.get_row_by_id(id).await?;
                    ok(edit_row(table, row))
                }),
            )
            .route(
                table.relative_path(),
                post(|req: Request| async {
                    let id = table.save(req).await?;
                    let row = table.get_row_by_id(id).await?;
                    ok(view_row(table, row))
                }),
            )
            .route(
                table.relative_path(),
                delete(|req: Request| async { table.remove(req).await }),
            );
    }
    router
}

fn view_row(table: &dyn TableSchemaTrait, values: Vec<String>) -> Markup {
    let schema = table.schema();
    let key_selector = format!("a{}", values[0].clone());

    let cells: Vec<_> = std::iter::zip(schema, &values)
        .map(|(schema, value)| view_cell(schema, value))
        .collect();

    let id = std::iter::zip(schema, &values)
        .find(|(col, _)| col.key)
        .map(|(_, v)| v)
        .expect("Some column must be primary key");
    let edit_url = format!("{}/{id}", table.full_path());

    html!(tr #(key_selector) ."relative" {
        @for cell in cells {(cell)}
        td $"flex justify-around items-center" {
            button $"w-6 hover:text-gray-50" get=(edit_url) into=(format!("#{key_selector}")) {(EDIT_SVG)}
        }
    })
}

fn view_cell(_schema: &ColumnSchema, value: &String) -> Markup {
    html! {td $"text-center" {(value)}}
}

fn edit_cell(schema: &ColumnSchema, value: &String, key_selector: &String) -> Markup {
    let input_type = column_input_type(schema);

    let checked = match value.as_str() {
        "true" => true,
        _ => false,
    };

    let onchange = match input_type {
        "checkbox" => Some("this.value = this.checked ? 'true' : 'false'"),
        _ => None,
    };

    html! {
        td $"text-center" {
            input
                $"bg-stone-900 accent-stone-600 px-2 py-1"
                .(key_selector)
                onchange=[(onchange)]
                type=(input_type)
                name=(schema.name)
                value=(value)
                readonly[schema.key]
                checked[checked] {}
        }
    }
}

fn edit_row(table: &dyn TableSchemaTrait, values: Vec<String>) -> Markup {
    let schema = table.schema();
    let key_selector = format!("key{}", values[0].clone());
    let inputs_classname = format!(".{key_selector}");

    let cells: Vec<_> = std::iter::zip(schema, &values)
        .map(|(schema, value)| edit_cell(schema, value, &key_selector))
        .collect();

    html!(tr #(key_selector) ."relative" {
        @for cell in cells {(cell)}
        td $"flex justify-around items-center" into=(format!("#{key_selector}")) include=(inputs_classname) {
            button $"w-6 hover:text-gray-50" post=(table.full_path()) {(DONE_SVG)}
            button $"w-6 hover:text-gray-50" delete=(table.full_path()) {(DELETE_SVG)}
        }
    })
}

fn column_input_type(column: &ColumnSchema) -> &str {
    match column.glue_type {
        "BOOLEAN" => "checkbox",
        t if t.starts_with("UINT") || t.starts_with("INT") || t.starts_with("F") => "number",
        "U64" | "U8" | "F64" => "number",
        "TEXT" | _ => "text",
    }
}
