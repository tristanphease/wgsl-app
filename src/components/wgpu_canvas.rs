use dioxus::logger::tracing::*;
use dioxus::prelude::*;

const VERTEX_SHADER: &str = include_str!("../../assets/shaders/vertex.wgsl");
const FRAGMENT_SHADER: &str = include_str!("../../assets/shaders/fragment.wgsl");
const CANVAS_ID: &'static str = "wgpu-canvas";

#[css_module("/assets/styles/wgpu_canvas.css")]
struct Styles;

pub struct NewCompileStatus(pub u32, pub CanvasCompileStatus);

#[derive(Props, PartialEq, Clone)]
pub struct WgpuCanvasProps {
    compile_info: ReadSignal<CanvasCompileInfo, SyncStorage>,
    fragment_shader_text: ReadSignal<String>,
    set_compile_status: ReadSignal<EventHandler<NewCompileStatus>>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CanvasCompileStatus {
    // needs a compile
    NeedsCompile,
    // is compiling
    // Compiling,
    // finished compiling
    FinishedCompile,
    // error occurred while compiling
    ErrorCompile,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CanvasCompileInfo {
    id: u32,
    status: CanvasCompileStatus,
}

impl CanvasCompileInfo {
    pub fn new() -> Self {
        Self {
            id: 0,
            status: CanvasCompileStatus::FinishedCompile,
        }
    }

    pub fn needs_compile(&self) -> bool {
        self.status == CanvasCompileStatus::NeedsCompile
    }

    pub fn set_compile_status(&mut self, new_status: CanvasCompileStatus, id: u32) {
        // only update if id is higher or the same as existing one
        if id >= self.id {
            self.id = id;
            self.status = new_status;
        }
    }

    pub fn get_current_id(&self) -> u32 {
        self.id
    }
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
            compile_info,
            fragment_shader_text,
            set_compile_status,
        } = props;
        // this is really weird but essentially we need the component to only run once since otherwise
        // it creates multiple paint sources and the channels get mixed up
        // there might be a better way to handle this but i'm not sure how
        let compile_info = use_memo(move || compile_info());
        let fragment_shader_text = use_memo(move || fragment_shader_text());
        let set_compile_status = use_memo(move || set_compile_status());
        rsx! {
            NativeWgpuCanvas{
                compile_info,
                fragment_shader_text,
                set_compile_status
            }
        }
    }
}

#[component]
#[cfg(feature = "native")]
pub fn NativeWgpuCanvas(
    compile_info: Memo<CanvasCompileInfo>,
    fragment_shader_text: Memo<String>,
    set_compile_status: Memo<EventHandler<NewCompileStatus>>,
) -> Element {
    use crate::wgpu_render::native_wgpu_render::{
        CanvasCompileMessage, CanvasCompileResponse, CanvasPaintSource,
    };
    use dioxus_native::use_wgpu;
    use futures::StreamExt;

    let coroutine = use_coroutine(
        move |mut rx: UnboundedReceiver<CanvasCompileResponse>| async move {
            while let Some(compile_response) = rx.next().await {
                let result = match compile_response.message {
                    CanvasCompileMessage::CompileSuccess => CanvasCompileStatus::FinishedCompile,
                    CanvasCompileMessage::CompileError(_error_message) => {
                        CanvasCompileStatus::ErrorCompile
                    }
                };
                set_compile_status
                    .write()
                    .call(NewCompileStatus(compile_response.id, result));
            }
        },
    );

    let shader = format!("{}{}", VERTEX_SHADER, FRAGMENT_SHADER);
    let paint_source = CanvasPaintSource::new(shader, Some(coroutine.tx()));
    let sender = paint_source.sender();
    let paint_source_id = use_wgpu(move || paint_source);

    use_effect(move || {
        let compile_info = compile_info.read();
        if compile_info.needs_compile() {
            use crate::wgpu_render::native_wgpu_render::CanvasMessage;

            let shader = format!("{}{}", VERTEX_SHADER, &fragment_shader_text());
            info!("rendering new shader");
            let send_result = sender.send(CanvasMessage::SetShader(
                compile_info.get_current_id(),
                shader,
            ));
            if let Err(error) = send_result {
                error!("got error sending message = {error:?}");
            }
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

    use_resource(move || async move {
        let compile_info = *props.compile_info.read();
        let set_compile = props.set_compile_status.read();
        if compile_info.needs_compile() {
            if let Some(ref mut renderer) = *canvas_renderer.write() {
                let current_id = compile_info.get_current_id();

                // set_compile.call(NewCompileStatus(current_id, CanvasCompileStatus::Compiling));

                let shader = format!("{}{}", VERTEX_SHADER, &props.fragment_shader_text);
                info!("rendering new shader");
                let shader_compile = renderer.set_shader(&shader).await;
                let result = match shader_compile {
                    Ok(_) => {
                        renderer.render();
                        CanvasCompileStatus::FinishedCompile
                    }
                    Err(_) => {
                        warn!("error compiling shader");
                        CanvasCompileStatus::ErrorCompile
                    }
                };
                set_compile.call(NewCompileStatus(current_id, result));
            } else {
                warn!("can't get renderer");
            }
        }
    });

    use_effect(move || {
        if let Some(canvas) = &*canvas_element.read() {
            let canvas = canvas.clone();
            spawn(async move {
                let shader = format!("{}{}", VERTEX_SHADER, FRAGMENT_SHADER);
                let renderer = WebCanvasRenderer::new(canvas, &shader).await;
                if let Ok(renderer) = renderer {
                    canvas_renderer.set(Some(renderer));
                    props
                        .set_compile_status
                        .read()
                        .call(NewCompileStatus(0, CanvasCompileStatus::NeedsCompile));
                }
            });
        } else {
            warn!("canvas element not found");
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
