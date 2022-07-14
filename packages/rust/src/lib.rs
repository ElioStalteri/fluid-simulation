mod fluid;
mod utils;
use rust_decimal_macros::dec;
use std::convert::TryInto;

use wasm_bindgen::prelude::*;

// When the `wee_alloc` feature is enabled, use `wee_alloc` as the global
// allocator.
#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

#[wasm_bindgen]
extern "C" {
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

#[wasm_bindgen(js_name = "start")]
pub fn start() {
    utils::set_panic_hook()
}

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(typescript_type = "number[]")]
    pub type ArrayOfNumbers;
}

#[wasm_bindgen(js_name = "addArray")]
pub fn add_array(arr: ArrayOfNumbers) -> u32 {
    let rust_arr: Vec<u32> = arr.into_serde().unwrap();
    let mut sum: u32 = 0;
    for element in rust_arr {
        sum = element + sum
    }
    return sum;
}

#[wasm_bindgen(js_name = "helloWorld")]
pub fn hello_world(name: Option<String>) {
    match name {
        Some(name) => alert(format!("Hello {}", name)),
        None => alert(String::from("Hello world!")),
    }
}

use lazy_static::lazy_static; // 1.4.0
use std::sync::Mutex;

lazy_static! {
    static ref FLUID_INSTANCE: Mutex<Vec<fluid::Fluid>> = Mutex::new(vec![]);
}

#[wasm_bindgen(js_name = "create_fluid")]
pub fn create_fluid(size: Option<i32>) {
    let mut tmp = FLUID_INSTANCE.lock().unwrap();
    if tmp.len() > 0 {
        tmp.remove(0);
    }
    tmp.push(fluid::Fluid::create(
        size.unwrap_or(0),
        dec!(0),
        dec!(0),
        dec!(0),
        4,
    ));
    log_u32(tmp.get(0).unwrap().size as u32);
    log_u32(tmp.len() as u32);
}
