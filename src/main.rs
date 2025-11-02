use dioxus::prelude::*;

use crate::components::{
    header::HeaderComponent,
    text_editor::TextEditor,
    tool_bar::ToolBar,
    wgpu_canvas::{CanvasCompileStatus, WgpuCanvas},
};

mod components;
pub mod wgpu_render;

static STYLES: Asset = asset!("/assets/main.css");
const DEFAULT_FRAGMENT_SHADER: &str = include_str!("../assets/shader/fragment.wgsl");

fn main() {
    #[cfg(feature = "tracing")]
    tracing_subscriber::fmt::init();

    #[cfg(feature = "native")]
    dioxus_native::launch(App);

    #[cfg(feature = "web")]
    dioxus::launch(App);
}

#[component]
fn App() -> Element {
    let mut current_frag_text = use_signal(|| DEFAULT_FRAGMENT_SHADER.to_string());
    let mut needs_compile = use_signal(|| CanvasCompileStatus::FinishedCompile);

    rsx! {
        document::Link { rel: "stylesheet", href: STYLES }
        document::Link { rel: "icon", href: asset!("/assets/favicon.ico") }
        document::Title { "wgsl app" }
        HeaderComponent {}
        div { id: "main-wrapper",
            TextEditor {
                text: current_frag_text(),
                modify_text: move |new_text| current_frag_text.set(new_text),
            }
            WgpuCanvas {
                compile_status: needs_compile(),
                fragment_shader_text: current_frag_text,
                set_compile_status: move |status| {
                    match status {
                        CanvasCompileStatus::Compiling => {
                            if *needs_compile.read() == CanvasCompileStatus::NeedsCompile {
                                *needs_compile.write() = status;
                            }
                        }
                        CanvasCompileStatus::FinishedCompile => {
                            if *needs_compile.read() == CanvasCompileStatus::Compiling
                                || *needs_compile.read() == CanvasCompileStatus::NeedsCompile
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
