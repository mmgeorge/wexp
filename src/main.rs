use wexp::run;
use wasm_bindgen::prelude::wasm_bindgen;
use console_error_panic_hook::set_once as set_panic_hook;

// #[cfg_attr(target_arch="wasm32", wasm_bindgen(start))]
#[wasm_bindgen(inline_js = "export function snippetTest() { console.log('Hello from JS FFI!'); }")]
extern "C" {
    fn snippetTest();
}

pub fn main() {
    set_panic_hook();
    
    snippetTest(); 

    run(); 
}
