use prest::*;
use reqwest::get as fetch;
use scraper::{Html, Selector};

#[derive(Storage, Serialize, Deserialize)]
struct Story {
    pub title: String,
    pub content: String,
}

#[init(log_filters=[("html5ever", "info"), ("selectors", "info")])]
async fn main() -> Result {
    spawn(scrape(
        "https://apnews.com",
        Selector::parse(".Page-content .PageList-items-item a").unwrap(),
        Selector::parse("h1.Page-headline").unwrap(),
        Selector::parse(".RichTextStoryBody > p").unwrap(),
    ));

    route(
        "/",
        get(|| async {
            ok(html!(html {(Head::with_title("With scraping"))
                body { @for story in Story::get_all().await? {
                    div $"my-2" {
                        h3 {(story.title)}
                        div $"text-sm" {(format!("{:.150}...", story.content))}
                    }
                }}
            }))
        }),
    )
    .run()
    .await
}

async fn scrape(
    url: &str,
    links_selector: Selector,
    title_selector: Selector,
    content_selector: Selector,
) -> Somehow {
    let text = fetch(url).await?.text().await?;

    let links = get_links(text, &links_selector);

    // restricting amount of parsed pages
    let links = &links[0..5];

    let responses = join_all(links.into_iter().map(|link| fetch(link)))
        .await
        .into_iter()
        .filter_map(|resp| resp.ok());

    let stories: Vec<Story> = join_all(responses.map(|resp| resp.text()))
        .await
        .into_iter()
        .filter_map(|text| text.ok())
        .map(|text| get_content(text, &title_selector, &content_selector))
        .collect();

    for story in stories {
        story.save().await?;
    }

    Ok(())
}

fn get_content(text: String, title_selector: &Selector, content_selector: &Selector) -> Story {
    let document = Html::parse_document(&text);

    let title = document
        .select(title_selector)
        .map(|t| t.inner_html())
        .next()
        .unwrap();

    let content = document
        .select(content_selector)
        .fold(String::new(), |full, p| {
            p.text().fold(full, |full, text| full + text) + "\n"
        });

    Story { title, content }
}

fn get_links(text: String, selector: &Selector) -> Vec<String> {
    let document = Html::parse_document(&text);

    let mut links = document
        .select(&selector)
        .filter_map(|x| x.value().attr("href"))
        .map(ToOwned::to_owned)
        .collect::<Vec<String>>();

    links.sort_unstable();
    links.dedup();
    links
}
