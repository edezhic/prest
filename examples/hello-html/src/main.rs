use prest::*;

fn main() {
    route("/", get(home).post(submit))
        .wrap_non_htmx(page)
        .run()
}

async fn page(content: Markup) -> Markup {
    html!(
        (DOCTYPE)
        (Head::example("Hello HTML"))
        body ."container" hx-target="main" hx-swap="innerHtml transition:true" hx-boost="true" {
            nav{ ul{li{"Common nav"}} ul{li{"Item"}li{"Item"}} }
            main ."view-transition" {(content)}
            (Scripts::default())
        }
    )
}

async fn home() -> Markup {
    html!(
        h3{"Say hello to ..."}
        form hx-post="/" {
            input name="message" type="text";
            button type="submit" {"Salute!"}
        }
        p style="text-align: center" {"Or..."}
        button _="
                on pointerdown 
                    toggle .party then
                    if my.innerText is not 'Stop it' 
                        then set my.innerText to 'Stop it' 
                        else set my.innerText to 'Lets try again'" 
            {"maybe lets get the party started?"}
        style {"
            @keyframes party { 
                0%   { background: #1095c1; } 
                33%  { background: #bf10c1; } 
                66%  { background: #2ac110; }
            }
            .party { animation: party 3s infinite linear; }
        "}
    )
}

#[derive(serde::Deserialize)]
struct FormInputs {
    message: String,
}

async fn submit(Form(FormInputs { message }): Form<FormInputs>) -> Markup {
    html!(
        @if message == "" {h3{"Nothing to salute :("}}
        @else {h1{"Hello "(message)"!"}}
        a href="/" {"Go back and try again"}
    )
}
