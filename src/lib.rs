use std::error::Error;
use std::mem;
use bytemuck::Pod;
use bytemuck::Zeroable;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast; 
use web_sys::console;
use wgpu::util::DeviceExt;
use wgpu::vertex_attr_array;
use winit::{
    event::*,
    event_loop::{ControlFlow, EventLoop},
    window::{WindowBuilder, Window}, dpi::PhysicalSize,
    platform::web::WindowExtWebSys
};


#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
struct Vertex {
    position: [f32; 3],
    color: [f32; 3], 
}

impl Vertex {
    const LAYOUT: [wgpu::VertexAttribute; 2] =
        wgpu::vertex_attr_array![0 => Float32x3, 1 => Float32x3]; 
    
    fn desc<'a>() -> wgpu::VertexBufferLayout<'a> {
        let array_stride = mem::size_of::<Vertex>() as wgpu::BufferAddress;
        
        // wgpu::VertexBufferLayout {
        //     array_stride,
        //     step_mode: wgpu::VertexStepMode::Vertex,
        //     attributes: &[
        //         wgpu::VertexAttribute {
        //             shader_location: 0, 
        //             format: wgpu::VertexFormat::Float32x3, 
        //             offset: 0,
        //         },
        //         wgpu::VertexAttribute {
        //             shader_location: 1, 
        //             format: wgpu::VertexFormat::Float32x3, 
        //             offset: mem::size_of::<[f32; 3]>() as wgpu::BufferAddress,
        //         }
        //     ]
        // }

        wgpu::VertexBufferLayout {
            array_stride,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &Self::LAYOUT
        }
    } 
}


const VERTS: &[Vertex] = &[
    Vertex { position: [ 0.0,  0.5, 0.0], color: [1., 0.0, 0.0] },
    Vertex { position: [-0.5, -0.5, 0.0], color: [0.0, 1., 0.0] },
    Vertex { position: [ 0.5, -0.5, 0.0], color: [0.0, 0.0, 1.] },
]; 

struct State {
    surface: wgpu::Surface,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    size: PhysicalSize<u32>,
    render_pipeline: wgpu::RenderPipeline,
    vertex_buffer: wgpu::Buffer,
    vertex_count: u32, 
}

impl State {
    async fn new(window: &Window) -> Self {
        let size = window.inner_size();

        let instance = wgpu::Instance::new(wgpu::Backends::all());
        let surface = unsafe { instance.create_surface(window) };

        // Adapter is a handle to graphics card driver
        let adapter = instance.request_adapter(
            &wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                compatible_surface: Some(&surface),
                // If `true`, force wgpu to pick an adapter that will work on all hardware.
                // This usually means that the rendering backend will use a "software" system,
                // instead of hardware such as a GPU.
                force_fallback_adapter: false
            }, 
        ).await.expect("Unable to create surface");


        // Can be used for API call tracing, if that feature is
        // enabled in wgpu-core
        let trace = None; 

        let (device, queue) = adapter.request_device(
            &wgpu::DeviceDescriptor {
                features: wgpu::Features::empty(),
                // max_compute_workgroups_per_dimension: 0 was problematic
                limits: wgpu::Limits::downlevel_webgl2_defaults(),
                label: Some("Root device"),
            },
            trace
        ).await.expect("Failed to query device");

        // Log all available features
        console::log_1(&JsValue::from(format!("Features \n    {:?}", adapter.features()))); 
        console::log_1(&JsValue::from(format!("{:#?}", adapter.limits())));

        // This will define how the surface creates its underlying SurfaceTextures.
        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface.get_preferred_format(&adapter).expect("Unable to get preferred format"),
            width: size.width,
            height: size.height,
            present_mode: wgpu::PresentMode::Fifo
        };

        surface.configure(&device, &config);

        let module = device.create_shader_module(&wgpu::ShaderModuleDescriptor {
            label: Some("Main shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("shader.wgsl").into())
        });

        // or
        // let shader = device.create_shader_module(&include_wgsl!("shader.wgsl"));

        let render_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Render pipeline layout"),
            bind_group_layouts: &[],
            push_constant_ranges: &[]
        });

        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Vertex Buffer"),
            usage: wgpu::BufferUsages::VERTEX,
            // Bytemuck provides utilities for safe-ishly bitfiddling structs 
            contents: bytemuck::cast_slice(VERTS), 
        }); 

        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Render pipeline"),
            layout: Some(&render_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &module,
                entry_point: "vs_main",
                buffers: &[Vertex::desc()]
            },
            fragment: Some(wgpu::FragmentState {
                entry_point: "fs_main",
                module: &module,
                targets: &[
                    wgpu::ColorTargetState {
                        format: config.format,
                        blend: Some(wgpu::BlendState::REPLACE),
                        write_mask: wgpu::ColorWrites::ALL
                    }
                ]
            }),
            primitive: Default::default(),
            // wgpu::PrimitiveState {
            //     // topology: wgpu::PrimitiveTopology::TriangleList,
            //     // strip_index_format: None,
            //     // front_face: wgpu::FrontFace::Ccw,
            //     // cull_mode: Some(wgpu::Face::Back),
            //     // unclipped_depth: false,
            //     // polygon_mode: wgpu::PolygonMode::Fill,
            //     // conservative: false,
            //     ..Default::default()
            //     // 
            // },
            multisample: wgpu::MultisampleState { count: 1, mask: !0, alpha_to_coverage_enabled: false },
            depth_stencil: None,
            multiview: None
        });

        let vertex_count = VERTS.len() as u32; 

        Self { surface, device, queue, config, size, render_pipeline, vertex_buffer, vertex_count }
    }

    fn resize(&mut self, new_size: PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.size = new_size;
            self.config.width = new_size.width;
            self.config.height = new_size.height;
            self.surface.configure(&self.device, &self.config); 
        }
    }

    fn input(&mut self, _event: &WindowEvent) -> bool {
        false
    }
    
    fn update(&mut self) {
        todo!()
    }
    
    fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
        // The get_current_texture function will wait for the surface to provide
        // a new SurfaceTexture that we will render to.
        let output = self.surface.get_current_texture()?;
        let mut desc = wgpu::TextureViewDescriptor::default();

        desc.label = Some("Main output view"); 

        // We need to do this because we want to control how the render code interacts with the texture.
        let view = output.texture.create_view(&desc);

        // The encoder builds a command buffer that we can then send to the gpu
        let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Render Encoder"),
        });

        // Clear color attachment
        let color_clear = wgpu::Color { r: 1., g: 0., b: 0., a: 1. };
        let color_attachment = wgpu::RenderPassColorAttachment {
            view: &view, // Texture to save to 
            resolve_target: None,
            ops: wgpu::Operations {
                load: wgpu::LoadOp::Clear(color_clear),
                store: true
            }
        };

        // Clear the screen
        let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Render pass"),
            depth_stencil_attachment: None, 
            color_attachments: &[color_attachment]
        });

        render_pass.set_pipeline(&self.render_pipeline);
        render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..)); 
        render_pass.draw(0..self.vertex_count, 0..1); 

        // By storing render_pass, we perform a mutable borrow of the encoder. In order to call 
        // encoder.finish() (also mutable), we need to drop the reference
        drop(render_pass);

        self.queue.submit(std::iter::once(encoder.finish())); 

        output.present();

        Ok(())
    }
}


fn init_logger() {
    std::panic::set_hook(Box::new(console_error_panic_hook::hook));
    console_log::init_with_level(log::Level::Warn).expect("Could't initialize logger");
}
fn init_window(event_loop: &EventLoop<()>) -> Result<Window, Box<dyn Error>> {
    let window = WindowBuilder::new().build(&event_loop)?;

    window.set_inner_size(PhysicalSize::new(450, 400));
     
    let web_window = web_sys::window().ok_or("No window found")?;
    let web_document = web_window.document().ok_or("No document found")?;
    let web_body = web_document.body().ok_or("No body found")?;
    let web_canvas = web_sys::Element::from(window.canvas());

    web_body.append_child(&web_canvas).map_err(|_| "Failed to append canvas to document body")?; 
    
    return Ok(window); 
}


fn kill_window() -> Result<(), Box<dyn Error>> {
    console::log_1(&"Kill window!".into());

    let web_window = web_sys::window().ok_or("No window found")?;
    let web_document = web_window.document().ok_or("No document found")?;
    let web_body = web_document.body().ok_or("No body found")?;
    let child = web_body.first_child().ok_or("No child found")?;
    
    web_body.remove_child(&child).map_err(|_| "Failed to remove child")?; 

    Ok(())
}

fn request_animation_frame(f: &Closure<dyn FnMut()>) {
    web_sys::window().unwrap()
        .request_animation_frame(f.as_ref().unchecked_ref())
        .expect("should register `requestAnimationFrame` OK");
}

fn log_event(log_list: &web_sys::Element, event: &Event<()>) {
        log::debug!("{:?}", event);

        // Getting access to browser logs requires a lot of setup on mobile devices.
        // So we implement this basic logging system into the page to give developers an easy alternative.
        // As a bonus its also kind of handy on desktop.
        if let Event::WindowEvent { event, .. } = &event {
            let window = web_sys::window().unwrap();
            let document = window.document().unwrap();
            let log = document.create_element("li").unwrap();
            log.set_text_content(Some(&format!("{:?}", event)));
            log_list
                .insert_before(&log, log_list.first_child().as_ref())
                .unwrap();
        }
}

#[wasm_bindgen(start)]
pub async fn run() {
    console::log_1(&JsValue::from("Run")); 

    init_logger();

    console::log_1(&"Creating event loop!".into());

    let event_loop = EventLoop::new(); 
    let window = init_window(&event_loop).expect("Unable to create window");

    console::log_1(&"Creating state!".into());

    let mut state = State::new(&window).await; 

    console::log_1(&"Initializing event loop!".into());

    // let f = Rc::new(RefCell::new(None));
    // let g = f.clone(); 
    
    // *g.borrow_mut() = Some(Closure::wrap(
    //     Box::new(move || {
    //         console::log_1(&"Initializing event loop!".into());
            
    //         request_animation_frame(f.borrow().as_ref().unwrap()); 
    //     }) as Box<dyn FnMut()>
    // )); 
        
    // request_animation_frame(g.borrow().as_ref().unwrap()); 
    
    event_loop.run(move |event, _, control_flow| {
        // *control_flow = ControlFlow::Poll;
        // console::log_1(&"Fire ev".into()); 
        // *control_flow = ControlFlow::Exit; 
        log::debug!("{:?}", event);

        // kill_window().expect("End it");

        if let Event::RedrawRequested(window_id) = event {
            if window_id != window.id() { return; }

            // console::log_1(&"Redraw requested!".into());

            // state.update();
            match state.render() {
                Ok(_) => {},
                Err(e) => eprintln!("{:?}", e)
            }
        }

        if let Event::MainEventsCleared = event {
            // RedrawRequested will only trigger once, unless we manually
            // request it.
            window.request_redraw(); 
        }
        
        if let Event::WindowEvent { ref event, window_id } = event {
            if window_id != window.id() { return; }
            
            match event {
                WindowEvent::Resized(size) => {
                    log::debug!("Resizing window");
                    state.resize(*size); 
                }
                WindowEvent::CloseRequested |
                WindowEvent::KeyboardInput {
                    input: KeyboardInput {
                        state: ElementState::Pressed,
                        virtual_keycode: Some(VirtualKeyCode::Escape),
                        ..
                    }, ..
                } => {
                    console::log_1(&"EXIT!".into()); 
                    // *control_flow = ControlFlow::Exit
                },
                _ => {}    
            }
        }
    });

    // window.set_cursor_visible(true); 

}
