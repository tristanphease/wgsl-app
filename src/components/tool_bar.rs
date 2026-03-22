use dioxus::prelude::*;

#[css_module("/assets/style/tool_bar.css")]
struct Styles;

#[component]
pub fn ToolBar(on_compile: EventHandler<()>) -> Element {
    rsx! {
        div { class: Styles::tool_bar,
            button { onclick: move |_| on_compile.call(()), "compile" }
        }
    }
}
