use dioxus::prelude::*;

#[component]
pub fn HeaderStyle() -> Element {
    rsx! {
        style { {include_str!("header.css")} }
    }
}

#[component]
pub fn HeaderComponent() -> Element {
    rsx! {
        HeaderStyle {}
        h1 { class: "header", "wgsl editor" }
    }
}
