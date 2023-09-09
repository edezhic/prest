use futures::future::join_all;
use pwrs::Result;
use reqwest::get;
use scraper::{Html, Selector};
struct Target {
    pub url: String,
    pub links: Selector,
    pub title: Selector,
    pub content: Selector,
}

#[tokio::main]
async fn main() {
    // starting scraping in a separate OS thread because it involves some !Send values
    std::thread::spawn(|| {
        scrape(Target {
            url: "https://apnews.com".to_owned(),
            links: Selector::parse(".Page-content .PageList-items-item a").unwrap(),
            title: Selector::parse("h1.Page-headline").unwrap(),
            content: Selector::parse(".RichTextStoryBody > p").unwrap(),
        })
    });

    let service = pwrs::Router::new().route("/", pwrs::get(homepage));
    pwrs::host::serve(service, 80).await.unwrap();
}

async fn homepage() -> impl pwrs::IntoResponse {
    pwrs::maud_to_response(maud::html!(
        html {
            head {
                title {"With scraping"}
                link rel="stylesheet" href="https://cdn.jsdelivr.net/npm/water.css@2/out/dark.css" {}
            }
            body {h1{"Check out terminal for scraping results!"}}
        }
    ))
}  

struct Story {
    pub title: String,
    pub content: String,
}

#[tokio::main]
async fn scrape(target: Target) -> Result<()> {
    let mut stories = vec![];
    let response = get(&target.url).await?.text().await?;
    let document = Html::parse_document(&response);

    // select links from the target
    let mut links = document
        .select(&target.links)
        .map(|x| x.value().attr("href").unwrap())
        .collect::<Vec<&str>>();
    // remove duplicates
    links.sort_unstable();
    links.dedup();

    // await requests to each link
    let results = join_all(links.into_iter().map(|link| get(link))).await;
    // filter out unsuccessful results
    let responses = results.into_iter().filter_map(|resp| resp.ok());
    // await bodies of successful responses
    let texts = join_all(responses.map(|resp| resp.text())).await;
    // filter out malformed bodies and parse as html
    let bodies: Vec<Html> = texts
        .into_iter()
        .filter_map(|text| text.ok())
        .map(|text| Html::parse_document(&text))
        .collect();

    for body in bodies {
        // select title's inner html and take the first match
        let title = body
            .select(&target.title)
            .map(|t| t.inner_html())
            .next()
            .unwrap();
        // select content's text nodes and fold them together
        let content = body.select(&target.content).fold(String::new(), |full, p| {
            p.text().fold(full, |full, text| full + text) + "\n"
        });

        stories.push(Story { title, content });
    }
    for story in stories {
        println!("---{}\n{:.150}...\n\n", story.title, story.content);
    }
    Ok(())
}
