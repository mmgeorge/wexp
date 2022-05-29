use std::error::Error;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast; 
use web_sys::console;
use winit::{
    event::*,
    event_loop::{ControlFlow, EventLoop},
    window::{WindowBuilder, Window}, dpi::PhysicalSize,
    platform::web::WindowExtWebSys
};

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
pub fn run() {
    init_logger(); 

    let event_loop = EventLoop::new(); 
    let window = init_window(&event_loop).expect("Unable to create window"); 

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
        
        if let Event::WindowEvent { ref event, window_id } = event {
            if window_id != window.id() { return; }
            
            match event {
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
