use dioxus::prelude::*;

#[css_module("/assets/styles/text_editor.css")]
struct Styles;

#[component]
pub fn TextEditor(text: String, modify_text: EventHandler<String>) -> Element {
    rsx! {
        textarea {
            class: Styles::text_editor,
            oninput: move |input| { modify_text.call(input.value()) },
            value: "{text}",
        }
    }
}
