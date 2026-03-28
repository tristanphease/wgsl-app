use dioxus::prelude::*;

use crate::components::{
    header::HeaderComponent,
    settings_editor::SettingsEditor,
    text_editor::TextEditor,
    tool_bar::ToolBar,
    wgpu_canvas::{CanvasCompileStatus, WgpuCanvas},
};

mod components;
pub mod wgpu_render;

static GLOBAL_STYLES: Asset = asset!("/assets/styles/global.css");
const DEFAULT_FRAGMENT_SHADER: &str = include_str!("../assets/shaders/fragment.wgsl");

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
    compile_setting: Signal<CompileSetting>,
}

/// The settings for compiling the shader
#[derive(Clone)]
enum CompileSetting {
    /// Have to manually trigger a recompile
    Manual,
    /// Automatically trigger a recompile after the number of milliseconds
    Auto(u32),
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
    let mut needs_compile = use_signal(|| CanvasCompileStatus::FinishedCompile);

    let is_settings_open = use_signal(|| false);
    let compile_setting = use_signal(|| CompileSetting::Manual);
    use_context_provider(|| Settings {
        is_editor_open: is_settings_open,
        compile_setting: compile_setting,
    });

    rsx! {
        document::Link { rel: "stylesheet", href: GLOBAL_STYLES }
        document::Link { rel: "icon", href: asset!("/assets/favicon.ico") }
        document::Title { "wgsl app" }
        div {
            class: MainStyles::app_root,
            HeaderComponent {},
            SettingsEditor { },
            main {
                class: MainStyles::main_wrapper,
                TextEditor {
                    text: current_frag_text(),
                    modify_text: move |new_text| current_frag_text.set(new_text),
                }
                WgpuCanvas {
                    compile_status: needs_compile,
                    fragment_shader_text: current_frag_text,
                    set_compile_status: move |status| {
                        match status {
                            CanvasCompileStatus::Compiling => {
                                if needs_compile() == CanvasCompileStatus::NeedsCompile {
                                    *needs_compile.write() = status;
                                }
                            }
                            CanvasCompileStatus::FinishedCompile => {
                                if needs_compile() == CanvasCompileStatus::Compiling
                                    || needs_compile() == CanvasCompileStatus::NeedsCompile
                                {
                                    *needs_compile.write() = status;
                                }
                            }
                            _ => {}
                        }
                    },
                }
                ToolBar {
                    on_compile: move |_| {
                        needs_compile.set(CanvasCompileStatus::NeedsCompile);
                    },
                }
            }
        }
    }
}
