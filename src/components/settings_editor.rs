use dioxus::prelude::*;

use crate::Settings;

#[css_module("/assets/styles/settings_editor.css")]
struct Styles;

#[component]
pub fn SettingsEditor() -> Element {
    let settings = use_context::<Settings>();
    let is_open = *settings.is_editor_open.read();
    rsx! {
        // note that blitz doesn't support dialog
        dialog {
            class: format!("{} {}",
                Styles::settings_overlay,
                if is_open { Styles::settings_open.to_string() } else { "".to_string() }),
            div {
                div {
                    class: Styles::header,
                    div {
                        class: Styles::header_text,
                        "Settings"
                    },
                    button {
                        class: Styles::close_settings,
                        onclick: move |_| consume_context::<Settings>().close(),
                        "Close"
                    }
                }
            }
        }
    }
}
