use dioxus::prelude::*;

#[css_module("/assets/styles/notification.css")]
struct Styles;

#[component]
pub fn Notification(title: ReadSignal<String>, body: ReadSignal<String>) -> Element {
    rsx! {
        div {
            class: Styles::notification,
            div {
                "{title}"
            },
            div {
                "{body}"
            }
        }
    }
}
