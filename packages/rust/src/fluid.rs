extern crate vecmath;
use rust_decimal::prelude::*;
use rust_decimal_macros::dec;
use std::convert::TryInto;
use wasm_bindgen::prelude::*;

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

fn ix(x: i32, y: i32, size: i32) -> usize {
    (x + y * size) as usize
}

pub struct Fluid {
    pub size: i32,
    pub iter: i32,
    pub dt: Decimal,
    pub diff: Decimal,
    pub visc: Decimal,
    pub s: Vec<Decimal>,
    pub density: Vec<Decimal>,
    pub v: Vec<vecmath::Vector2<Decimal>>,
    pub v_0: Vec<vecmath::Vector2<Decimal>>,
}

impl Fluid {
    pub fn create(
        size: i32,
        diffusion: Decimal,
        viscosity: Decimal,
        dt: Decimal,
        iter: i32,
    ) -> Fluid {
        Self {
            size,
            iter,
            diff: diffusion,
            visc: viscosity,
            dt,
            s: vec![dec!(0); (size * size).try_into().unwrap()],
            density: vec![dec!(0); (size * size).try_into().unwrap()],
            v: vec![[dec!(0); 2]; (size * size).try_into().unwrap()],
            v_0: vec![[dec!(0); 2]; (size * size).try_into().unwrap()],
        }
    }
    pub fn add_density(&mut self, x: i32, y: i32, amount: Decimal) {
        self.density[ix(x, y, self.size)] += amount;
    }
    pub fn add_valocity(&mut self, x: i32, y: i32, v: vecmath::Vector2<Decimal>) {
        self.v[ix(x, y, self.size)] = vecmath::vec2_add(self.v[ix(x, y, self.size)], v);
    }
    pub fn Step(&mut self) {
        self.diffuse(true);

        // project(Vx0, Vy0, Vz0, Vx, Vy, 4, N);

        // advect(1, Vx, Vx0, Vx0, Vy0, Vz0, dt, N);
        // advect(2, Vy, Vy0, Vx0, Vy0, Vz0, dt, N);
        // advect(3, Vz, Vz0, Vx0, Vy0, Vz0, dt, N);

        // project(Vx, Vy, Vz, Vx0, Vy0, 4, N);

        self.diffuse(false);
        // advect(0, density, s, Vx, Vy, Vz, dt, N);
    }
    fn lin_solve(&mut self, check_bnd: bool, a: Decimal, c: Decimal) {
        let c_recip = dec!(1.0) / c;
        for _k in 0..self.iter {
            for js in 1..(self.size - 1) {
                for is in 1..(self.size - 1) {
                    let j = js as i32;
                    let i = is as i32;
                    self.v_0[ix(i, j, self.size)] = vecmath::vec2_scale(
                        vecmath::vec2_add(
                            self.v[ix(i, j, self.size)],
                            vecmath::vec2_scale(
                                vecmath::vec2_add(
                                    vecmath::vec2_add(
                                        self.v_0[ix(i + 1, j, self.size)],
                                        self.v_0[ix(i - 1, j, self.size)],
                                    ),
                                    vecmath::vec2_add(
                                        self.v_0[ix(i, j + 1, self.size)],
                                        self.v_0[ix(i, j - 1, self.size)],
                                    ),
                                ),
                                a,
                            ),
                        ),
                        c_recip,
                    );
                }
            }
            if check_bnd {
                self.set_bnd();
            }
        }
    }
    fn set_bnd(&mut self) {
        // loop all borders and negate the array
        for i in 0..(self.size - 1) {
            self.v_0[ix(i, 0, self.size)] = vecmath::vec2_neg(self.v_0[ix(i, 1, self.size)]);
            self.v_0[ix(i, (self.size - 1), self.size)] =
                vecmath::vec2_neg(self.v_0[ix(i, (self.size - 2), self.size)]);
            self.v_0[ix(0, i, self.size)] = vecmath::vec2_neg(self.v_0[ix(1, i, self.size)]);
            self.v_0[ix((self.size - 1), i, self.size)] =
                vecmath::vec2_neg(self.v_0[ix((self.size - 2), i, self.size)]);
        }
    }
    fn diffuse(&mut self, check_bnd: bool) {
        let size = Decimal::from_i32((self.size - 2).into()).unwrap_or(dec!(8888));
        log("convert size to decimal if 9999 or 8888 it didnt work!!");
        log_u32(size.to_u32().unwrap_or(9999));
        let a = self.dt * self.diff * size * size;
        let c = dec!(1) + dec!(6) * a;
        self.lin_solve(check_bnd, a, c);
    }
}
