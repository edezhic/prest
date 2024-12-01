use crate::*;

pub(crate) async fn db_page() -> Markup {
    let tables = html! {
        @for table in DB_SCHEMA.tables() {
            $"font-bold text-lg" {(table.name())}
            div ."loader" get=(table.full_path()) trigger="load" swap-full into="this" {}
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
                        table $"w-full font-mono" {
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
    let mut cells = vec![];
    let key_selector = format!("a{}", values[0].clone());
    // let inputs_classname = format!(".{key_selector}");

    for (_, value) in std::iter::zip(schema, &values) {
        let cell = html! {td $"text-center" {(value)}};
        cells.push(cell);
    }

    let id = std::iter::zip(schema, &values)
        .find(|(col, _)| col.key)
        .map(|(_, v)| v)
        .expect("Some column must be primary key");
    let edit_url = format!("{}/{id}", table.full_path());

    html!(tr #(key_selector) ."relative" {
        @for cell in cells {(cell)}
        td $"flex justify-around items-center" {
            button $"w-6 hover:text-gray-50" get=(edit_url) swap-full into=(format!("#{key_selector}")) {(edit_svg())}
        }
    })
}

fn edit_row(table: &dyn TableSchemaTrait, values: Vec<String>) -> Markup {
    let schema = table.schema();
    let mut cells = vec![];
    let key_selector = format!("a{}", values[0].clone());
    let inputs_classname = format!(".{key_selector}");

    for (column, value) in std::iter::zip(schema, values) {
        let input_type = column_input_type(column);

        let checked = match value.as_str() {
            "true" => true,
            _ => false,
        };

        let onchange = match input_type {
            "checkbox" => Some("this.value = this.checked ? 'true' : 'false'"),
            _ => None,
        };

        let cell = html! {
            td $"text-center" {
                input
                    $"bg-stone-900 accent-stone-600 px-2 py-1"
                    .(key_selector)
                    onchange=[(onchange)]
                    type=(input_type)
                    name=(column.name)
                    value=(value)
                    checked[checked] {}
            }
        };
        cells.push(cell);
    }

    html!(tr #(key_selector) ."relative" {
        @for cell in cells {(cell)}
        td $"flex justify-around items-center" swap-full into=(format!("#{key_selector}")) include=(inputs_classname) {
            button $"w-6 hover:text-gray-50" post=(table.full_path()) {(done_svg())}
            button $"w-6 hover:text-gray-50" delete=(table.full_path()) {(delete_svg())}
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

fn edit_svg() -> Markup {
    html!(
        svg viewBox="0 0 24 24" fill="none" {
            path fill-rule="evenodd" clip-rule="evenodd" d="M20.8477 1.87868C19.6761 0.707109 17.7766 0.707105 16.605 1.87868L2.44744 16.0363C2.02864 16.4551 1.74317 16.9885 1.62702 17.5692L1.03995 20.5046C0.760062 21.904 1.9939 23.1379 3.39334 22.858L6.32868 22.2709C6.90945 22.1548 7.44285 21.8693 7.86165 21.4505L22.0192 7.29289C23.1908 6.12132 23.1908 4.22183 22.0192 3.05025L20.8477 1.87868ZM18.0192 3.29289C18.4098 2.90237 19.0429 2.90237 19.4335 3.29289L20.605 4.46447C20.9956 4.85499 20.9956 5.48815 20.605 5.87868L17.9334 8.55027L15.3477 5.96448L18.0192 3.29289ZM13.9334 7.3787L3.86165 17.4505C3.72205 17.5901 3.6269 17.7679 3.58818 17.9615L3.00111 20.8968L5.93645 20.3097C6.13004 20.271 6.30784 20.1759 6.44744 20.0363L16.5192 9.96448L13.9334 7.3787Z" fill="currentColor" {}
        }
    )
}

fn done_svg() -> Markup {
    html!(
        svg viewBox="0 -1.5 11 11" {
            g stroke="none" stroke-width="1" fill="none" fill-rule="evenodd" {
                g transform="translate(-304.000000, -366.000000)" fill="currentColor" {
                    g transform="translate(56.000000, 160.000000)" {
                        polygon points="259 207.6 252.2317 214 252.2306 213.999 252.2306 214 248 210 249.6918 208.4 252.2306 210.8 257.3082 206" {}
                    }
                }
            }
        }
    )
}

fn delete_svg() -> Markup {
    html!(
        svg viewBox="0 0 24 24" fill="none" {
            path d="M10 11V17" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" {}
            path d="M14 11V17" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" {}
            path d="M4 7H20" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" {}
            path d="M6 7H12H18V18C18 19.6569 16.6569 21 15 21H9C7.34315 21 6 19.6569 6 18V7Z" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" {}
            path d="M9 5C9 3.89543 9.89543 3 11 3H13C14.1046 3 15 3.89543 15 5V7H9V5Z" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" {}
        }
    )
}
