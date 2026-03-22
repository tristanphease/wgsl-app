use dioxus::prelude::*;

#[css_module("/assets/styles/header.css")]
struct Style;

#[component]
pub fn HeaderComponent() -> Element {
    rsx! {
        h1 { class: Style::header, "wgsl editor" }
    }
}
