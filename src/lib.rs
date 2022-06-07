mod texture_resource;
mod camera; 

use std::error::Error;
use std::mem;
use bytemuck::Pod;
use bytemuck::Zeroable;
use bytemuck::cast_slice;
use camera::Camera;
use camera::CameraUniform;
use texture_resource::TextureResource;
use wasm_bindgen::prelude::*;
use web_sys::console;
use wgpu::TextureUsages;
use wgpu::util::BufferInitDescriptor;
use wgpu::util::DeviceExt;
use wgpu::*; 
use winit::{
  event::*,
  event_loop::{EventLoop},
  window::{WindowBuilder, Window}, dpi::PhysicalSize,
  platform::web::WindowExtWebSys
};



#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
struct Vertex {
  position: [f32; 3],
  color: [f32; 3],
  texture_coords: [f32; 2]
}

impl Vertex {
  // Workaround for rust bug? 
  const LAYOUT: [VertexAttribute; 3] =
    vertex_attr_array![0 => Float32x3, 1 => Float32x3, 2 => Float32x2]; 
  
  fn desc<'a>() -> VertexBufferLayout<'a> {
    let array_stride = mem::size_of::<Vertex>() as BufferAddress;
    
    // VertexBufferLayout {
    //     array_stride,
    //     step_mode: VertexStepMode::Vertex,
    //     attributes: &[
    //         VertexAttribute {
    //             shader_location: 0, 
    //             format: VertexFormat::Float32x3, 
    //             offset: 0,
    //         },
    //         VertexAttribute {
    //             shader_location: 1, 
    //             format: VertexFormat::Float32x3, 
    //             offset: mem::size_of::<[f32; 3]>() as BufferAddress,
    //         }
    //     ]
    // }

    VertexBufferLayout {
      array_stride,
      step_mode: VertexStepMode::Vertex,
      attributes: &Self::LAYOUT
    }
  } 
}


const VERTS: &[Vertex] = &[
  Vertex { position: [-0.5,  0.5, 0.0], color: [1., 0.0, 0.0], texture_coords: [0., 1.] },
  Vertex { position: [-0.5, -0.5, 0.0], color: [0.0, 1., 0.0], texture_coords: [0., 0.]  },
  Vertex { position: [ 0.5, -0.5, 0.0], color: [0.0, 0.0, 1.], texture_coords: [1., 0.]  },

  Vertex { position: [-0.5,  0.5, 0.0], color: [1., 0.0, 0.0], texture_coords: [0., 1.]  },
  Vertex { position: [ 0.5, -0.5, 0.0], color: [0.0, 0.0, 1.], texture_coords: [1., 0.]  },
  Vertex { position: [ 0.5,  0.5, 0.0], color: [0.0, 0.0, 1.], texture_coords: [1., 1.]  },
]; 

struct State {
  surface: Surface,
  device: Device,
  queue: Queue,
  config: SurfaceConfiguration,
  size: PhysicalSize<u32>,
  render_pipeline: RenderPipeline,
  vertex_buffer: Buffer,
  vertex_count: u32,

  diffuse_bind_group: BindGroup,

  camera: Camera,
  camera_uniform: CameraUniform,
  camera_buf: Buffer,
  camera_bind_group: BindGroup
}

impl State {
  async fn new(window: &Window) -> Self {
    let size = window.inner_size();

    let instance = Instance::new(Backends::all());
    let surface = unsafe { instance.create_surface(window) };

    // Adapter is a handle to graphics card driver
    let adapter = instance.request_adapter(
      &RequestAdapterOptions {
        power_preference: PowerPreference::HighPerformance,
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
      &DeviceDescriptor {
        features: Features::empty(),
        // max_compute_workgroups_per_dimension: 0 was problematic
        limits: Limits::downlevel_webgl2_defaults(),
        label: Some("Root device"),
      },
      trace
    ).await.expect("Failed to query device");

    // Log all available features
    console::log_1(&JsValue::from(format!("Features \n    {:?}", adapter.features()))); 
    console::log_1(&JsValue::from(format!("{:#?}", adapter.limits())));

    // This will define how the surface creates its underlying SurfaceTextures.
    let config = SurfaceConfiguration {
      usage: TextureUsages::RENDER_ATTACHMENT,
      format: surface.get_preferred_format(&adapter).expect("Unable to get preferred format"),
      width: size.width,
      height: size.height,
      present_mode: PresentMode::Fifo
    };

    surface.configure(&device, &config);

    let diffuse_bytes = include_bytes!("happy.png");
    let diffuse_resource = TextureResource::from_bytes(&device, &queue, diffuse_bytes, "diffuse-texture");
    // let diffuse_resource = TextureResource::from_url(&device, &queue, "./happy.png", "diffuse-texture")
    //     .await.expect("Get diffuse resource"); 

    let texture_bind_group_layout = device.create_bind_group_layout(&BindGroupLayoutDescriptor {
      label: Some("Diffuse texture bind group"), 
      entries: &[
        BindGroupLayoutEntry {
          binding: 0,
          visibility: ShaderStages::FRAGMENT,
          ty: BindingType::Texture {
            sample_type: TextureSampleType::Float { filterable: true },
            view_dimension: TextureViewDimension::D2,
            multisampled: false
          },
          count: None,
        },
        BindGroupLayoutEntry {
          binding: 1,
          visibility: ShaderStages::FRAGMENT,
          ty: BindingType::Sampler(SamplerBindingType::Filtering),
          count: None
        }
      ] 
    });

    let diffuse_bind_group = device.create_bind_group({
      &BindGroupDescriptor {
        label: Some("Diffuse bind group"),
        layout: &texture_bind_group_layout, 
        entries: &[
          BindGroupEntry {
            binding: 0,
            resource: BindingResource::TextureView(&diffuse_resource.view)
          },
          BindGroupEntry {
            binding: 1,
            resource: BindingResource::Sampler(&diffuse_resource.sampler)
          }
        ]
      }
    }); 
    

    let module = device.create_shader_module(&ShaderModuleDescriptor {
      label: Some("Main shader"),
      source: ShaderSource::Wgsl(include_str!("shader.wgsl").into())
    });

    let camera = Camera::new(config.width as f32 / config.height as f32);
    let mut camera_uniform = CameraUniform::new();

    camera_uniform.update(&camera);

    let camera_buf = device.create_buffer_init(&BufferInitDescriptor {
      label: Some("Camera buf"),
      contents: cast_slice(&[camera_uniform]),
      usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST
    });

    let camera_layout = device.create_bind_group_layout(&BindGroupLayoutDescriptor {
      label: Some("Camera bind group layout"),
      entries: &[
        BindGroupLayoutEntry {
          count: None,
          binding: 0,
          visibility: ShaderStages::VERTEX,
          ty: BindingType::Buffer {
            ty: BufferBindingType::Uniform,
            has_dynamic_offset: false,
            min_binding_size: None
          }
        }
      ]
    });

    let camera_bind_group = device.create_bind_group(&BindGroupDescriptor {
      label: Some("Camera bind group"), 
      layout: &camera_layout,
      entries: &[
        BindGroupEntry {
          binding: 0,
          resource: camera_buf.as_entire_binding()
        }
      ]
    });

    let render_pipeline_layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
      label: Some("Render pipeline layout"),
      bind_group_layouts: &[
        &texture_bind_group_layout,
        &camera_layout,
      ],
      push_constant_ranges: &[]
    });

    let vertex_buffer = device.create_buffer_init(&util::BufferInitDescriptor {
      label: Some("Vertex Buffer"),
      usage: BufferUsages::VERTEX,
      // Bytemuck provides utilities for safe-ishly bitfiddling structs 
      contents: bytemuck::cast_slice(VERTS), 
    }); 

    let render_pipeline = device.create_render_pipeline(&RenderPipelineDescriptor {
      label: Some("Render pipeline"),
      layout: Some(&render_pipeline_layout),
      vertex: VertexState {
        module: &module,
        entry_point: "vs_main",
        buffers: &[Vertex::desc()]
      },
      fragment: Some(FragmentState {
        entry_point: "fs_main",
        module: &module,
        targets: &[
          ColorTargetState {
            format: config.format,
            blend: Some(BlendState::REPLACE),
            write_mask: ColorWrites::ALL
          }
        ]
      }),
      primitive: PrimitiveState {
        // topology: PrimitiveTopology::TriangleList,
        // strip_index_format: None,
        front_face: FrontFace::Ccw,
        // cull_mode: Some(Face::Back),
        // unclipped_depth: false,
        // polygon_mode: PolygonMode::Fill,
        // conservative: false,
        ..Default::default()
      },
      multisample: MultisampleState { count: 1, mask: !0, alpha_to_coverage_enabled: false },
      depth_stencil: None,
      multiview: None
    });

    let vertex_count = VERTS.len() as u32;


    Self { surface, device, queue, config, size, render_pipeline, vertex_buffer, vertex_count, diffuse_bind_group, camera, camera_bind_group, camera_buf, camera_uniform }
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
  
  fn render(&mut self) -> Result<(), SurfaceError> {
    // The get_current_texture function will wait for the surface to provide
    // a new SurfaceTexture that we will render to.
    let output = self.surface.get_current_texture()?;
    let mut desc = TextureViewDescriptor::default();

    desc.label = Some("Main output view"); 

    // We need to do this because we want to control how the render code interacts with the texture.
    let view = output.texture.create_view(&desc);

    // The encoder builds a command buffer that we can then send to the gpu
    let mut encoder = self.device.create_command_encoder(&CommandEncoderDescriptor {
      label: Some("Render Encoder"),
    });

    // Clear color attachment
    let color_clear = Color { r: 1., g: 0., b: 0., a: 1. };
    let color_attachment = RenderPassColorAttachment {
      view: &view, // Texture to save to 
      resolve_target: None,
      ops: Operations {
        load: LoadOp::Clear(color_clear),
        store: true
      }
    };

    // Clear the screen
    let mut render_pass = encoder.begin_render_pass(&RenderPassDescriptor {
      label: Some("Render pass"),
      depth_stencil_attachment: None, 
      color_attachments: &[color_attachment]
    });

    render_pass.set_pipeline(&self.render_pipeline);
    render_pass.set_bind_group(0, &self.diffuse_bind_group, &[]);
    render_pass.set_bind_group(1, &self.camera_bind_group, &[]);
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


// fn kill_window() -> Result<(), Box<dyn Error>> {
//   console::log_1(&"Kill window!".into());

//   let web_window = web_sys::window().ok_or("No window found")?;
//   let web_document = web_window.document().ok_or("No document found")?;
//   let web_body = web_document.body().ok_or("No body found")?;
//   let child = web_body.first_child().ok_or("No child found")?;
  
//   web_body.remove_child(&child).map_err(|_| "Failed to remove child")?; 

//   Ok(())
// }

// fn request_animation_frame(f: &Closure<dyn FnMut()>) {
//   web_sys::window().unwrap()
//     .request_animation_frame(f.as_ref().unchecked_ref())
//     .expect("should register `requestAnimationFrame` OK");
// }

// fn log_event(log_list: &web_sys::Element, event: &Event<()>) {
//   log::debug!("{:?}", event);

//   // Getting access to browser logs requires a lot of setup on mobile devices.
//   // So we implement this basic logging system into the page to give developers an easy alternative.
//   // As a bonus its also kind of handy on desktop.
//   if let Event::WindowEvent { event, .. } = &event {
//     let window = web_sys::window().unwrap();
//     let document = window.document().unwrap();
//     let log = document.create_element("li").unwrap();
//     log.set_text_content(Some(&format!("{:?}", event)));
//     log_list
//       .insert_before(&log, log_list.first_child().as_ref())
//       .unwrap();
//   }
// }

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
  
  event_loop.run(move |event, _, _control_flow| {
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
