pub mod home;
pub mod external_link {
    pub fn render(text: &str, url: &str) -> maud::Markup {
        let href = format!("https://{}", url);
        maud::html!(a."external-link" href={(href)} target="_blank" {(text)})
    }
}
pub mod full_html {
    pub fn render(content: maud::Markup) -> maud::Markup {
        maud::html!(
            html {
                head {
                    title {"PWRS app"}
                    link rel="icon" href="/favicon.png" {}
                    link rel="manifest" href="/.webmanifest" {}
                    link rel="stylesheet" href="https://fonts.googleapis.com/css2?family=Roboto:wght@300;400;500;700&display=swap" {}
                    link rel="stylesheet" href="/ui.css" {}
                    script src="/ui.js" {}
                    script src="https://unpkg.com/htmx.org@1.9.0" integrity="sha384-aOxz9UdWG0yBiyrTwPeMibmaoq07/d3a96GCbb9x60f3mOt5zwkjdbcHFnKH8qls" crossorigin="anonymous" {}
                    meta
                        name="viewport"
                        content="width=device-width, initial-scale=1.0";
                }
                body {
                    main."slide-transition" {(content)}
                }
            }
        )
    }    
}
