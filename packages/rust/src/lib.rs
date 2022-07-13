mod utils;


use wasm_bindgen::prelude::*;

// When the `wee_alloc` feature is enabled, use `wee_alloc` as the global
// allocator.
#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

#[wasm_bindgen]
extern {
    fn alert(s: String);

    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);

    // The `console.log` is quite polymorphic, so we can bind it with multiple
    // signatures. Note that we need to use `js_name` to ensure we always call
    // `log` in JS.
    #[wasm_bindgen(js_namespace = console, js_name = log)]
    fn log_u32(a: u32);

    // Multiple arguments too!
    #[wasm_bindgen(js_namespace = console, js_name = log)]
    fn log_many(a: &str, b: &str);
}

#[wasm_bindgen(js_name="start")]
pub fn start() {
    utils::set_panic_hook()
}

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(typescript_type = "number[]")]
    pub type ArrayOfNumbers;
}

#[wasm_bindgen(js_name="addArray")]
pub fn add_array(arr: ArrayOfNumbers) -> u32 {
    let rust_arr: Vec<u32> = arr.into_serde().unwrap();
    let mut sum: u32 = 0;
    for element in rust_arr {
        sum = element + sum
    }
    return sum
}

#[wasm_bindgen(js_name="helloWorld")]
pub fn hello_world(name: Option<String>) {
    match name {
        Some(name) => alert(format!("Hello {}", name)),
        None => alert(String::from("Hello world!"))
    }
}
