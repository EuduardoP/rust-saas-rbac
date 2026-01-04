use api::EchoInput;
use dioxus::{fullstack::Json, prelude::*};

const ECHO_CSS: Asset = asset!("/assets/styling/echo.css");

#[component]
pub fn Echo() -> Element {
    let mut response = use_signal(|| None::<EchoInput>);

    rsx! {
        document::Link { rel: "stylesheet", href: ECHO_CSS }
        div {
            id: "echo",
            h4 { "ServerFn Echo" }

            input {
                placeholder: "Type here to echo...",
                oninput: move |event| async move {
                    if let Ok(data) = api::echo(Json( EchoInput {
                        message: event.value().clone(),
                    })).await {
                        response.set(Some(data));
                    }
                },
            }

            if let Some(resp) = response.read().as_ref() {
                p {
                    "Server echoed: "
                    i { "{resp.message}" }
                }
            }
        }
    }
}
