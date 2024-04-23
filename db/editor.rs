use crate::*;

pub fn add_db_editor(mut router: Router) -> Router {
    for schema in DB_SCHEMA.tables().iter() {
        router = router.merge(schema.router());
    }
    router.route("/db", get(db_page))
}

async fn db_page() -> impl IntoResponse {
    html!{(DOCTYPE) (Head::with_title("DB Admin"))
            body."max-w-screen-sm mx-auto mt-12 flex flex-col items-center" {
                @for table in DB_SCHEMA.tables() {
                    h2 {(table.name())}
                    table {
                        @let headers = table.schema().iter().filter(|c| !c.key).map(|c| c.name);
                        @for header in headers {th {(header)}}
                        tr."loader" hx-get=(table.get_all_route()) hx-trigger="load" hx-swap="outerHTML" hx-target="this" {}
                    }
                    
                }
                (Scripts::default())
            }
        }
}