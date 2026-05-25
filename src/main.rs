use std::time::Duration;

use dioxus::prelude::*;
use dioxus_sdk_time::use_debounce;

use crate::components::{
    header::HeaderComponent,
    notification::Notification,
    settings_editor::SettingsEditor,
    text_editor::TextEditor,
    tool_bar::ToolBar,
    wgpu_canvas::{CanvasCompileInfo, CanvasCompileStatus, NewCompileStatus, WgpuCanvas},
};

mod components;
pub mod wgpu_render;

static GLOBAL_STYLES: Asset = asset!("/assets/styles/global.css");
const DEFAULT_FRAGMENT_SHADER: &str = include_str!("../assets/shaders/fragment.wgsl");

#[derive(Clone)]
struct ErrorNotification {
    title: String,
    body: String,
}

fn main() {
    #[cfg(feature = "tracing")]
    tracing_subscriber::fmt::init();

    #[cfg(feature = "native")]
    dioxus_native::launch(App);

    #[cfg(feature = "web")]
    dioxus::launch(App);
}

#[css_module("/assets/styles/main.css")]
struct MainStyles;

/// Settings for the application
#[derive(Clone)]
struct Settings {
    /// Whether the settings editor is open
    is_editor_open: Signal<bool>,
    /// Whether the shader should automatically compile on save
    compile_setting: Signal<CompileSetting, SyncStorage>,
    /// If compile settings is auto then this is the time between editing and compiling
    auto_compile_time: Signal<u32>,
}

/// The settings for compiling the shader
#[derive(Clone, PartialEq, Eq)]
enum CompileSetting {
    /// Have to manually trigger a recompile
    Manual,
    /// Automatically trigger a recompile after the number of milliseconds
    Auto,
}

impl std::ops::Not for CompileSetting {
    type Output = Self;

    fn not(self) -> Self::Output {
        match self {
            Self::Manual => Self::Auto,
            Self::Auto => Self::Manual,
        }
    }
}

impl Settings {
    fn open(&mut self) {
        *self.is_editor_open.write() = true;
    }

    fn close(&mut self) {
        *self.is_editor_open.write() = false;
    }
}

#[component]
fn App() -> Element {
    let mut current_frag_text = use_signal(|| DEFAULT_FRAGMENT_SHADER.to_string());
    let mut compile_info = use_signal_sync(|| CanvasCompileInfo::new());

    let mut compile_id = use_signal(|| 0u32);
    let is_settings_open = use_signal(|| false);
    let compile_setting = use_signal_sync(|| CompileSetting::Manual);
    let compile_wait = use_signal(|| 1000u32);
    use_context_provider(|| Settings {
        is_editor_open: is_settings_open,
        compile_setting: compile_setting,
        auto_compile_time: compile_wait,
    });

    let mut compile_timeout = use_debounce(
        // this duration doesn't update reactively
        Duration::from_millis(u64::from(*compile_wait.read())),
        move |()| {
            let mut val = compile_id.write();
            *val += 1;
            let new_id = *val;

            compile_info
                .write()
                .set_compile_status(CanvasCompileStatus::NeedsCompile, new_id);
        },
    );

    use_effect(move || {
        // read signal to subscribe and trigger effect
        current_frag_text.read();
        if let CompileSetting::Auto = *compile_setting.peek() {
            compile_timeout.action(());
        }
    });

    let error_notification: Signal<Option<ErrorNotification>> = use_signal(|| None);

    rsx! {
        document::Link { rel: "stylesheet", href: GLOBAL_STYLES }
        document::Link { rel: "icon", href: asset!("/assets/favicon.ico") }
        document::Title { "wgsl app" }
        div {
            class: MainStyles::app_root,
            HeaderComponent { },
            SettingsEditor { },
            main {
                class: MainStyles::main_wrapper,
                TextEditor {
                    text: current_frag_text(),
                    modify_text: move |new_text| current_frag_text.set(new_text),
                }
                WgpuCanvas {
                    compile_info: compile_info,
                    fragment_shader_text: current_frag_text,
                    set_compile_status: move |input| {
                        let NewCompileStatus(id, status) = input;
                        compile_info.write().set_compile_status(status, id);
                    },
                }
                ToolBar {
                    on_compile: move |_| {
                        let mut val = compile_id.write();
                        *val += 1;
                        let new_id = *val;
                        compile_info.write().set_compile_status(CanvasCompileStatus::NeedsCompile, new_id);
                    },
                }
            }
        }
        // this is at the end because of https://github.com/DioxusLabs/blitz/issues/387
        if let Some(notif) = error_notification() {
            Notification {
                title: notif.title,
                body: notif.body,
            }
        },
    }
}
