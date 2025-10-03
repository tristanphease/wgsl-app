use dioxus::prelude::*;

#[component]
pub fn ToolBar(on_compile: EventHandler<()>,) -> Element {
    rsx! {
        div { class: "tool-bar",
            button { onclick: move |_| on_compile.call(()), "compile" }
        }
    }
}
