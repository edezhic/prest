use host::admin::{DELETE_SVG, DONE_SVG, EDIT_SVG};

use crate::*;

pub(crate) async fn db_page() -> Markup {
    let tables = html! {
        @for table in DB.custom_tables() {
            $"font-bold text-lg" {(table.name())}
            a get=(table.full_path()) trigger="load" swap-this {}
        }
    };
    html!((tables))
}

pub(crate) async fn db_routes() -> Router {
    let mut router = route("/", get(db_page));
    for table in DB.custom_tables() {
        let get_by_id_path = format!("{}/:id", table.relative_path());
        router = router
            .route(
                table.relative_path(),
                get(|| async {
                    let rows = table
                        .get_all()
                        .await?
                        .into_iter()
                        .map(|row| view_row(table, row));
                    ok(html!(
                        table #(table.name()) .table-editor $"w-full font-mono text-[0.5rem] lg:text-sm" {
                            @let columns = table.columns().iter().map(|c| (c.name, c.rust_type));
                            @for (name, rust_type) in columns {th {(name)" ("(rust_type)")"}}
                            th #actions $"w-[70px]" {}
                            (create_form(table))
                            @for row in rows {(row)}
                        }
                    ))
                }),
            )
            .route(
                &get_by_id_path,
                get(|Path(id): Path<String>| async {
                    let row = table.get_row_by_id(id).await?;
                    ok(edit_row(table, row))
                }),
            )
            .route(table.relative_path(), put(|req: Request| async {
                let id = table.save(req).await?;
                let row = table.get_row_by_id(id).await?;
                ok(view_row(table, row))
            }))
            .route(
                table.relative_path(),
                patch(|req: Request| async {
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

fn create_form(table: TableSchema) -> Markup {
    let columns = table.columns();
    let key_selector = key_selector(table, None);

    let cells = columns.iter().map(|schema| {
        html!(
            td .create { (column_input(schema, None, &key_selector)) }
        )
    });

    html!(tr #(key_selector) {
        @for cell in cells {(cell)}
        td .actions put-after={"#"(key_selector)} include={"."(key_selector)} { div {
            button put=(table.full_path()) after-request={"reset('."(key_selector)"')"} {(DONE_SVG)}
        }}
    })
}

fn view_row(table: TableSchema, values: Vec<String>) -> Markup {
    let columns = table.columns();
    let key_selector = key_selector(table, Some(&values));

    let cells = values.iter().map(|value| html! {td ."view" {(value)}});

    let id = std::iter::zip(columns, &values)
        .find(|(col, _)| col.pkey)
        .map(|(_, v)| v)
        .expect("Some column must be primary key");
    let edit_url = format!("{}/{id}", table.full_path());

    html!(tr #(key_selector) {
        @for cell in cells {(cell)}
        td .actions { div {
            button get=(edit_url) target={"#"(key_selector)} {(EDIT_SVG)}
        }}
    })
}

fn edit_row(table: TableSchema, values: Vec<String>) -> Markup {
    let columns = table.columns();
    let key_selector = key_selector(table, Some(&values));

    let cells = std::iter::zip(columns, &values).map(|(schema, value)| {
        html!(
            td .edit {
                (column_input(schema, Some(value), &key_selector))
                @if schema.pkey {
                    (value)
                }
            }
        )
    });

    html!(tr #(key_selector) {
        @for cell in cells {(cell)}
        td .actions target={"#"(key_selector)} include={"."(key_selector)} { div {
            button patch=(table.full_path()) {(DONE_SVG)}
            button hx-confirm="are you sure you want to delete?" delete=(table.full_path()) {(DELETE_SVG)}
        }}
    })
}

fn column_input(schema: &ColumnSchema, value: Option<&str>, key_selector: &String) -> Markup {
    let input_type = if value.is_some() && schema.pkey {
        "hidden"
    } else {
        column_input_type(schema)
    };

    let checked = value.filter(|v| *v == "true").is_some();

    html! {
        input
            .(key_selector)
            type=(input_type)
            name=(schema.name)
            value=[value]
            checked[checked] {}
    }
}

fn key_selector(table: TableSchema, values: Option<&Vec<String>>) -> String {
    if let Some(values) = values {
        let pkey_index = table
            .columns()
            .iter()
            .position(|c| c.pkey)
            .expect("Some column must be the primary key");
        format!("key{}", values[pkey_index].clone())
    } else {
        format!("new_{}", table.name())
    }
}

fn column_input_type(column: &ColumnSchema) -> &str {
    let singular = !column.list && !column.optional;
    match column.sql_type {
        "BOOLEAN" if singular => "checkbox",
        _ if column.numeric && singular => "number",
        "TEXT" | _ => "text",
    }
}
