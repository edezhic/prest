use prest::*;

fn main() {
    Router::new()
        .route("/", get(home))
        .route("/submit", post(submit))
        .wrap_non_htmx(wrapper)
        .serve(ServeOptions::default())
}

async fn home() -> Markup {
    html!(
        h3{"Say hello to ..."}
        form hx-post="/submit" { 
            input name="message" type="text"; 
            button type="submit" {"Salute!"}
        }
        p style="text-align: center" {"Or..."}
        button _="on pointerdown 
                    if my.innerText is not 'Stop it' 
                        then set my.innerText to 'Stop it' 
                        else set my.innerText to 'Lets try again' 
                    repeat until event pointerup
                    set rand to Math.random() * 360
                    transition
                        *background-color
                        to `hsl($rand 100% 90%)`
                        over 250ms
                    end" 
            {"maybe lets get the party started?"}
    )
}

#[derive(serde::Deserialize)]
struct FormInputs {
    message: String
}

async fn submit(Form(FormInputs { message }): Form<FormInputs>) -> Markup {
    let message = if message == "" {
        "Nothing to salute :(".to_owned()
    } else {
        format!("Hello {message}!")
    };
    html!(
        h3{(message)}
        a href="/" {"Go back and try again"}
    )
}

async fn wrapper(content: Markup) -> Markup {
    html!(
        (Head::example("Hello HTML"))
        body ."container" hx-target="main" hx-swap="innerHtml transition:true" hx-boost="true" {
            nav{ul{li{"Common nav"}}ul{li{"Item"}li{"Item"}}}
            main ."view-transition" {(content)}
            (Scripts::default())
        }
    )
}
