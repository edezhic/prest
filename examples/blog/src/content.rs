use prest::*;

state!(README: String = { 
    let src = include_str!("../../../README.md");
    let homepage = src.trim_start_matches("# prest").trim_start();
    md_to_html(homepage) 
});
state!(INTERNALS: String = {
    let md = include_str!("../../../UNDER_THE_HOOD.md").to_owned();
    let processed = preprocess_md(md, "../../..", Some(include_str!("../../../Cargo.toml")));
    md_to_html(&processed)
});
state!(RUST: String = { md_to_html(include_str!("../../../RUST.md")) });
state!(PREST_VERSION: String = {
    let manifest = include_str!("../../../Cargo.toml").parse::<toml::Table>().unwrap();
    manifest["package"]["version"].as_str().unwrap().to_owned()
});

embed_as!(ExamplesDocs from "../" only "*.md");
embed_as!(ExamplesCode from "../" except "*.md");

#[allow(dead_code)]
pub struct Example {
    pub path: String,
    pub url: String,
    pub label: String,
    pub content: String,
    pub category: ExampleCategory,
}

#[derive(PartialEq, Serialize)]
pub enum ExampleCategory {
    Blog,
    Database,
    Todo,
    Other,
}
pub use ExampleCategory::*;

state!(EXAMPLES: Vec<Example> = {
    let mut examples = vec![];
    for path in ExamplesDocs::iter() {
        let path = path.to_string();
        let url = if path.starts_with("blog") {
            "/about".to_owned()
        } else {
            format!("/{}", path.trim_start_matches("databases/").trim_end_matches("/README.md"))
        };
        let label = url.replace('/', "").replace('-', " ");
        let category = match path.split('/').next().unwrap() {
            "databases" => Database,
            s if s.contains("todo") => Todo,
            s if s == "blog" => Blog,
            _ => Other
        };
        let doc = ExamplesDocs::get_content(&path).unwrap();
        let dir = path.trim_end_matches("/README.md");
        let processed = preprocess_md(doc, &dir, None);
        let content = md_to_html(&processed);
        examples.push(Example { path, url, label, content, category });
    }
    examples
});

pub fn preprocess_md(doc: String, doc_dir: &str, code: Option<&str>) -> String {
    let mut processed = String::new();
    for line in doc.lines() {
        // replace references like {src/main.rs 50:71} with lines from the referenced files
        if line.starts_with("{") && line.ends_with("}") {
            let reference = line.replace(['{', '}'], "");
            let Some(relative_path) = reference.split(':').next() else {
                panic!("Reference {reference} from {doc_dir} has invalid syntax")
            };
            let lang = source_language(&relative_path);
            let full_path = format!("{doc_dir}/{relative_path}");
            let code = match code {
                Some(code) => code.to_owned(),
                None => match ExamplesCode::get_content(&full_path) {
                    Some(code) => code,
                    None => panic!("Not found {full_path} mentioned in {doc_dir}"),
                },
            };
            let code = match reference.split(':').skip(1).next() {
                // select lines like :25 or :50-71
                Some(lines_refs) => {
                    let mut refs = lines_refs.split('-');
                    let start = refs.next().unwrap().parse::<usize>().unwrap();
                    let end = if let Some(end) = refs.next() {
                        end.parse::<usize>().unwrap()
                    } else {
                        start
                    };
                    let mut snippet = if start > 1 {
                        "...\n".to_owned()
                    } else {
                        String::new()
                    };
                    snippet += &code.lines().skip(start - 1).take(end - start + 1).fold(
                        String::new(),
                        |mut code, line| {
                            code += line;
                            code + "\n"
                        },
                    );
                    if code.lines().count() > end {
                        snippet += "...\n";
                    }
                    snippet
                }
                None => code,
            };
            processed += &format!("`{reference}`\n");
            processed += &format!("\n```{lang}\n{}\n```\n", code.trim_end());
        } else {
            processed += &format!("{line}\n");
        }
    }
    processed
}

use markdown::{to_html_with_options, Options};
pub fn md_to_html(md: &str) -> String {
    #[cfg(debug_assertions)]
    let md = md.replace("https://prest.blog", "http://localhost");
    to_html_with_options(&md, &Options::gfm()).unwrap()
}

fn source_language(filename: &str) -> &str {
    match filename {
        f if f.ends_with(".rs") => "rust",
        f if f.ends_with(".toml") => "toml",
        f if f.ends_with(".css") => "css",
        f if f.ends_with(".scss") => "scss",
        f if f.ends_with(".html") => "html",
        f if f.ends_with(".sql") => "sql",
        f if f.ends_with(".ts") => "typescript",
        _ => "",
    }
}
