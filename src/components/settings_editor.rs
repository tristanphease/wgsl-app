use dioxus::prelude::*;

use crate::{CompileSetting, Settings};

#[css_module("/assets/styles/settings_editor.css")]
struct Styles;

#[component]
pub fn SettingsEditor() -> Element {
    let mut settings = use_context::<Settings>();
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
                div {
                    class: Styles::settings_wrapper,
                    label {
                        "Should auto-compile?"
                        input {
                            type: "checkbox",
                            onclick: move |_| settings.compile_setting.toggle()
                        }
                    },
                    // consider replacing with input range when blitz supports that
                    div {
                        class: Styles::button_collection,
                        NumberButton {
                            num: 200,
                            onclick: move |val| {
                                *settings.auto_compile_time.write() = val
                            },
                            current_number: settings.auto_compile_time,
                            current_setting: settings.compile_setting,
                        },
                        NumberButton {
                            num: 1000,
                            onclick: move |val| {
                                *settings.auto_compile_time.write() = val
                            },
                            current_number: settings.auto_compile_time,
                            current_setting: settings.compile_setting,
                        }
                        NumberButton {
                            num: 2000,
                            onclick: move |val| {
                                *settings.auto_compile_time.write() = val
                            },
                            current_number: settings.auto_compile_time,
                            current_setting: settings.compile_setting,
                        }
                    }
                }
            }
        }
    }
}

#[component]
fn NumberButton(
    num: u32,
    onclick: EventHandler<u32>,
    current_number: ReadSignal<u32>,
    current_setting: ReadSignal<CompileSetting, SyncStorage>,
) -> Element {
    let disabled =
        use_memo(move || current_number() == num || current_setting != CompileSetting::Auto);
    rsx! {
        button {
            onclick: move |_| onclick.call(num),
            disabled: disabled(),
            {num.to_string()}
        }
    }
}
