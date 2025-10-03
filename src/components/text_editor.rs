use dioxus::prelude::*;

#[component]
pub fn TextEditor(text: String, modify_text: EventHandler<String>) -> Element {
    rsx! {
        style { {include_str!("text_editor.css")} }
        textarea {
            class: "text-editor",
            oninput: move |input| { modify_text.call(input.value()) },
            value: "{text}",
        }
    }
}
