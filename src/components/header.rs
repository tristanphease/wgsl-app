use dioxus::prelude::*;

use crate::Settings;

#[css_module("/assets/styles/header.css")]
struct Style;

#[component]
pub fn HeaderComponent() -> Element {
    rsx! {
        div {
            class: Style::header,
            h1 { class: Style::header_text, "wgsl editor" }
            button {
                class: Style::settings,
                onclick: move |_| consume_context::<Settings>().open(),
                "Settings" // todo: change this to icon
            }
        }
    }
}
