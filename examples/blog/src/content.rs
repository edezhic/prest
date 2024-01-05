use prest::*;

pub struct Readme {
    pub path: String,
    pub url: String,
    pub label: String,
    pub content: String,
    pub step: bool,
    pub with: bool,
}

state!(READMES: Vec<Readme> = {
    let mut examples = vec![];
    for path in ExamplesDocs::iter() {
        let path = path.to_string();

        let url = if path.starts_with("blog") {
            "/about".to_owned()
        } else {
            format!("/{}", path.trim_end_matches("/README.md"))
        };

        let label = match &path {
            path if path.starts_with("step-1") => "todo app".to_owned(),
            path if path.starts_with("step-2") => "pwa".to_owned(),
            path if path.starts_with("step-3") => "auth".to_owned(),
            _ => url.replace("/", "").replace("-", " ").replace("with ", ""),
        };

        let step = path.starts_with("step");
        let with = path.starts_with("with");

        let raw_doc = ExamplesDocs::get_content(&path).unwrap();
        let processed = preprocess_md(raw_doc, &path);
        let content = md_to_html(&processed);
        
        examples.push(Readme { path, url, label, content, step, with });
    }
    examples
});

embed_as!(ExamplesDocs from "../" only "*.md");
embed_as!(ExamplesCode from "../" except "*.md");

pub fn preprocess_md(raw_doc: String, doc_path: &str) -> String {
    let mut processed = String::new();
    for line in raw_doc.lines() {
        // lines like {path} are converted into the contents of file
        if line.starts_with("{") && line.ends_with("}") {
            let inline_file = line.replace(['{', '}'], "");
            let inline_path = format!("{}/{inline_file}", doc_path.trim_end_matches("/README.md"));
            let code = match ExamplesCode::get_content(&inline_path) {
                Some(code) => code,
                None => panic!("Not found {inline_path} mentioned in {doc_path}"),
            };
            let code_type = match &inline_file {
                f if f.ends_with(".rs") => "rust",
                f if f.ends_with(".toml") => "toml",
                f if f.ends_with(".css") => "css",
                f if f.ends_with(".scss") => "scss",
                f if f.ends_with(".html") => "html",
                f if f.ends_with(".sql") => "sql",
                f if f.ends_with(".ts") => "typescript",
                _ => "",
            };
            processed += &format!("`/{inline_file}`\n");
            processed += &format!("\n```{code_type}\n{}\n```\n", code.trim_end());
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