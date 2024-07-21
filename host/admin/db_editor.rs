use crate::*;

pub(crate) fn db_routes() -> Router {
    let mut router = Router::new();
    for table in DB_SCHEMA.tables() {
        router = router.route(table.path(), get(|| async {
            let schema = table.schema();
            let values = table.get_all();
            let mut rows = vec![];
            for row_values in values {
                let mut cells = vec![];
                let key_selector = format!("a{}", row_values[0].clone());
                let inputs_classname = format!(".{key_selector}");

                for (schema, value) in std::iter::zip(schema, row_values) {
                    let input_type = match schema.glue_type {
                            "BOOLEAN" => "checkbox",
                            t if t.starts_with("UINT") || t.starts_with("INT") || t.starts_with("F") => "number",
                            "U64" | "U8" | "F64" => "number",
                            "TEXT" | _ => "text",
                    };

                    let cell_class = match schema.key {
                        true => "hidden",
                        false => "text-center",
                    };

                    let input_class = match input_type {
                        "text" | "number" => "input input-bordered w-full",
                        "checkbox" => "checkbox",
                        _ => "",
                    };

                    let checked = match value.as_str() {
                        "true" => true,
                        _ => false,
                    };

                    let onchange = match input_type {
                        "checkbox" => Some("this.value = this.checked ? 'true' : 'false'"),
                        _ => None
                    };

                    let cell = html! {
                        td.(cell_class) {input.(input_class).(key_selector) onchange=[(onchange)] type=(input_type) name=(schema.name) value=(value) checked[checked] {}}
                    };
                    cells.push(cell);
                }
                rows.push(html!(tr #(key_selector) ."relative" { 
                    @for cell in cells {(cell)}
                    td $"flex justify-around items-center" {
                        button hx-post=(table.path()) hx-swap="none" hx-include=(inputs_classname) type="submit" {"Save"}
                        button hx-delete=(table.path()) hx-swap="outerHtml" hx-target=(format!("#{key_selector}")) hx-include=(inputs_classname) type="submit" {"Delete"}   
                    }
                }))
            }
            html!(
                table $"w-full" {
                    @let headers = table.schema().iter().filter(|c| !c.key).map(|c| c.name);
                    @for header in headers {th {(header)}} th{"Actions"}
                    @for row in rows {(row)}
                }
            )
        })).route(table.path(), post(|req: Request| async {
            table.save(req).await
        })).route(table.path(), delete(|req: Request| async {
            table.remove(req).await
        }));
    }
    router
}
