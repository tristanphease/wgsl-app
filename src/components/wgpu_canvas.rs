use dioxus::logger::tracing::*;
use dioxus::prelude::*;

const VERTEX_SHADER: &str = include_str!("../../assets/shaders/vertex.wgsl");
const FRAGMENT_SHADER: &str = include_str!("../../assets/shaders/fragment.wgsl");
const CANVAS_ID: &'static str = "wgpu-canvas";

#[css_module("/assets/styles/wgpu_canvas.css")]
struct Styles;

#[derive(Props, PartialEq, Clone)]
pub struct WgpuCanvasProps {
    compile_status: ReadSignal<CanvasCompileStatus>,
    fragment_shader_text: ReadSignal<String>,
    set_compile_status: ReadSignal<EventHandler<CanvasCompileStatus>>,
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
    {
        let WgpuCanvasProps {
            compile_status,
            fragment_shader_text,
            set_compile_status,
        } = props;
        // this is really weird but essentially we need the component to only run once since otherwise
        // it creates multiple paint sources and the channels get mixed up
        // there might be a better way to handle this but i'm not sure how
        let compile_status = use_memo(move || compile_status());
        let fragment_shader_text = use_memo(move || fragment_shader_text());
        let set_compile_status = use_memo(move || set_compile_status());
        rsx! {
            NativeWgpuCanvas{
                compile_status,
                fragment_shader_text,
                set_compile_status
            }
        }
    }
}

#[component]
#[cfg(feature = "native")]
pub fn NativeWgpuCanvas(
    compile_status: Memo<CanvasCompileStatus>,
    fragment_shader_text: Memo<String>,
    set_compile_status: Memo<EventHandler<CanvasCompileStatus>>,
) -> Element {
    use crate::wgpu_render::native_wgpu_render::CanvasPaintSource;
    use dioxus_native::use_wgpu;

    let shader = format!("{}{}", VERTEX_SHADER, FRAGMENT_SHADER);
    let paint_source = CanvasPaintSource::new(shader);
    let sender = paint_source.sender();
    let paint_source_id = use_wgpu(move || paint_source);

    use_effect(move || {
        if compile_status() == CanvasCompileStatus::NeedsCompile {
            use crate::wgpu_render::native_wgpu_render::CanvasMessage;

            let shader = format!("{}{}", VERTEX_SHADER, &fragment_shader_text());
            info!("rendering new shader");
            let send_result = sender.send(CanvasMessage::SetShader(shader));
            if let Err(error) = send_result {
                error!("got error sending message = {error:?}");
            }
            set_compile_status().call(CanvasCompileStatus::FinishedCompile);
        }
    });

    rsx! {
        div {
            class: Styles::canvas_container,
            canvas {
                id: CANVAS_ID,
                "src": paint_source_id
            }
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
                        .read()
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
        div {
            class: Styles::canvas_container,
            canvas {
                onmounted: move |event| {
                    let event = event.as_web_event();
                    canvas_element
                        .set(Some(event.dyn_into::<web_sys::HtmlCanvasElement>().unwrap()));
                },
                id: CANVAS_ID,
            }
        }
    }
}
