use dioxus::logger::tracing::*;
use dioxus::prelude::*;

const VERTEX_SHADER: &str = include_str!("../../assets/shader/vertex.wgsl");
const FRAGMENT_SHADER: &str = include_str!("../../assets/shader/fragment.wgsl");

#[derive(Props, PartialEq, Clone)]
pub struct WgpuCanvasProps {
    compile_status: CanvasCompileStatus,
    fragment_shader_text: String,
    set_compile_status: EventHandler<CanvasCompileStatus>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum CanvasCompileStatus {
    NeedsCompile,
    Compiling,
    FinishedCompile,
}

#[component]
pub fn WgpuCanvas(props: WgpuCanvasProps) -> Element {
    #[cfg(feature = "web")]
    rsx! {
        WebWgpuCanvas { ..props }
    }
    #[cfg(feature = "native")]
    rsx! {
        NativeWgpuCanvas { ..props }
    }
}

#[component]
#[cfg(feature = "native")]
pub fn NativeWgpuCanvas(props: WgpuCanvasProps) -> Element {
    use crate::wgpu_render::native_wgpu_render::CanvasPaintSource;
    use dioxus_native::use_wgpu;

    let shader = format!("{}{}", VERTEX_SHADER, FRAGMENT_SHADER);
    let paint_source = CanvasPaintSource::new(shader);
    let sender = paint_source.sender();
    let paint_source_id = use_wgpu(move || paint_source);

    use_effect(use_reactive(
        (&props.compile_status,),
        move |compile_status| {
            if compile_status.0 == CanvasCompileStatus::NeedsCompile {
                use crate::wgpu_render::native_wgpu_render::CanvasMessage;

                let shader = format!("{}{}", VERTEX_SHADER, &props.fragment_shader_text);
                info!("rendering new shader");
                let _ = sender.send(CanvasMessage::SetShader(shader));
            }
        },
    ));

    rsx! {
        div { id: "canvas-container",
            canvas { id: "wgpu-canvas", "src": paint_source_id }
        }
    }
}

#[component]
#[cfg(feature = "web")]
pub fn WebWgpuCanvas(props: WgpuCanvasProps) -> Element {
    use crate::wgpu_render::web_wgpu_render::WebCanvasRenderer;
    use dioxus::web::WebEventExt;
    use web_sys::wasm_bindgen::JsCast;

    let mut canvas_element: Signal<Option<web_sys::HtmlCanvasElement>> = use_signal(|| None);
    let mut canvas_renderer: Signal<Option<WebCanvasRenderer>> = use_signal(|| None);

    use_effect(use_reactive(
        (&props.compile_status,),
        move |compile_status| {
            if compile_status.0 == CanvasCompileStatus::NeedsCompile {
                if let Some(ref mut renderer) = *canvas_renderer.write() {
                    use dioxus::logger::tracing::info;

                    let shader = format!("{}{}", VERTEX_SHADER, &props.fragment_shader_text);
                    info!("rendering new shader");
                    renderer.set_shader(&shader);
                    props
                        .set_compile_status
                        .call(CanvasCompileStatus::FinishedCompile);
                } else {
                    warn!("can't get renderer");
                }
            }
        },
    ));

    use_effect(move || {
        if let Some(canvas) = &*canvas_element.read() {
            let canvas = canvas.clone();
            spawn(async move {
                let shader = format!("{}{}", VERTEX_SHADER, FRAGMENT_SHADER);
                let renderer = WebCanvasRenderer::new(canvas, &shader).await;
                if let Ok(renderer) = renderer {
                    canvas_renderer.set(Some(renderer));
                }
            });
        } else {
            warn!("canvas element not found");
        }
    });

    use_future(move || async move {
        loop {
            gloo_timers::future::sleep(std::time::Duration::from_millis(50)).await;
            if let Some(ref mut renderer) = *canvas_renderer.write() {
                renderer.render();
            }
        }
    });

    rsx! {
        div { id: "canvas-container",
            canvas {
                onmounted: move |event| {
                    let event = event.as_web_event();
                    canvas_element
                        .set(Some(event.dyn_into::<web_sys::HtmlCanvasElement>().unwrap()));
                },
                id: "wgpu-canvas",
            }
        }
    }
}
