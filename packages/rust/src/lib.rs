mod fluid;
mod utils;
use rust_decimal::prelude::*;
use rust_decimal_macros::dec;
// use std::convert::TryInto;

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
    static ref FLUID_INSTANCE: Mutex<fluid::Fluid> =
        Mutex::new(fluid::Fluid::create(1, dec!(0), dec!(0), dec!(0), 4,));
}

#[wasm_bindgen(js_name = "create_fluid")]
pub fn create_fluid(size: Option<i32>) {
    let mut tmp = FLUID_INSTANCE.lock().unwrap();

    // 41 mins to render
    *tmp = fluid::Fluid::create(size.unwrap_or(0), dec!(0.01), dec!(0), dec!(0.1), 4);
    // tmp.step();
    // log("initial creation log");
    // log_u32(tmp.size as u32);
    // let tmp1 = Decimal::from_i32((tmp.size - 2).into()).unwrap_or(dec!(8888));
    // log_u32(tmp1.to_u32().unwrap_or(9999));
    // log("finish initial creation log");
}

#[wasm_bindgen(js_name = "fluid_step")]
pub fn fluid_step() {
    let mut tmp = FLUID_INSTANCE.lock().unwrap();
    tmp.step();
}

#[wasm_bindgen(js_name = "fluid_add_density")]
pub fn fluid_add_density(x: Option<i32>, y: Option<i32>, amount: Option<i32>) {
    let mut tmp = FLUID_INSTANCE.lock().unwrap();
    tmp.add_density(
        x.unwrap(),
        y.unwrap(),
        Decimal::from_i32(amount.unwrap()).unwrap(),
    );
}

#[wasm_bindgen(js_name = "fluid_add_velocity")]
pub fn fluid_add_velocity(x: Option<i32>, y: Option<i32>, vx: Option<f64>, vy: Option<f64>) {
    let mut tmp = FLUID_INSTANCE.lock().unwrap();
    tmp.add_velocity(
        x.unwrap(),
        y.unwrap(),
        [
            Decimal::from_f64(vx.unwrap()).unwrap(),
            Decimal::from_f64(vy.unwrap()).unwrap(),
        ],
    );
}

#[wasm_bindgen(js_name = "fluid_get_density")]
pub fn fluid_get_density() -> Vec<f64> {
    let tmp = FLUID_INSTANCE.lock().unwrap();
    return tmp
        .density
        .iter()
        .map(|v| {
            // if v > &dec!(0.001) {
                v.to_f64().unwrap()
            // } else {
                // 0f64
            // }
        })
        .collect();
    // get_density(x.unwrap(),y.unwrap()).to_f64().unwrap()
}

#[wasm_bindgen(js_name = "fluid_get_velocity")]
pub fn fluid_get_velocity() -> Vec<f64> {
    let tmp = FLUID_INSTANCE.lock().unwrap();
    return tmp
        .velx
        .iter()
        .map(|v| {
            // if (v[0] + v[1]) > dec!(0.001) {
            //     (v[0] + v[1]).to_f64().unwrap()
            // } else {
            //     0f64
            // }
            // (v[0] + v[1]).to_f64().unwrap()
            v.to_f64().unwrap()
        })
        .collect();
    // get_density(x.unwrap(),y.unwrap()).to_f64().unwrap()
}
