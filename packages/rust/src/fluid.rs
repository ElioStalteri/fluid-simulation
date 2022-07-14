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
    pub s_: Vec<Decimal>,
    pub density: Vec<Decimal>,
    pub vel: Vec<vecmath::Vector2<Decimal>>,
    pub vel_0: Vec<vecmath::Vector2<Decimal>>,
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
            s_: vec![dec!(0); (size * size).try_into().unwrap()],
            density: vec![dec!(0); (size * size).try_into().unwrap()],
            vel: vec![[dec!(0); 2]; (size * size).try_into().unwrap()],
            vel_0: vec![[dec!(0); 2]; (size * size).try_into().unwrap()],
        }
    }
    pub fn add_density(&mut self, x: i32, y: i32, amount: Decimal) {
        self.density[ix(x, y, self.size)] += amount;
    }
    pub fn add_valocity(&mut self, x: i32, y: i32, v: vecmath::Vector2<Decimal>) {
        self.vel[ix(x, y, self.size)] = vecmath::vec2_add(self.vel[ix(x, y, self.size)], v);
    }
    pub fn step(&mut self) {
        self.diffuse(true);

        self.project();

        // advect(1, Vx, Vx0, Vx0, Vy0, Vz0, dt, N);
        // advect(2, Vy, Vy0, Vx0, Vy0, Vz0, dt, N);
        // advect(3, Vz, Vz0, Vx0, Vy0, Vz0, dt, N);

        self.project_inverse();

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
                    self.vel_0[ix(i, j, self.size)] = vecmath::vec2_scale(
                        vecmath::vec2_add(
                            self.vel[ix(i, j, self.size)],
                            vecmath::vec2_scale(
                                vecmath::vec2_add(
                                    vecmath::vec2_add(
                                        self.vel_0[ix(i + 1, j, self.size)],
                                        self.vel_0[ix(i - 1, j, self.size)],
                                    ),
                                    vecmath::vec2_add(
                                        self.vel_0[ix(i, j + 1, self.size)],
                                        self.vel_0[ix(i, j - 1, self.size)],
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
            self.vel_0[ix(i, 0, self.size)] = vecmath::vec2_neg(self.vel_0[ix(i, 1, self.size)]);
            self.vel_0[ix(i, (self.size - 1), self.size)] =
                vecmath::vec2_neg(self.vel_0[ix(i, (self.size - 2), self.size)]);
            self.vel_0[ix(0, i, self.size)] = vecmath::vec2_neg(self.vel_0[ix(1, i, self.size)]);
            self.vel_0[ix((self.size - 1), i, self.size)] =
                vecmath::vec2_neg(self.vel_0[ix((self.size - 2), i, self.size)]);
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
    fn project(&mut self) {
        let size = Decimal::from_i32((self.size - 2).into()).unwrap_or(dec!(1));
        let multiplier = Decimal::from_f32((-0.5f32).into()).unwrap_or(dec!(1));

        for j in 1..(self.size - 1) {
            for i in 1..(self.size - 1) {
                self.vel_0[ix(i, j, self.size)][1] = multiplier
                    * (self.vel[ix(i + 1, j, self.size)][0] - self.vel[ix(i - 1, j, self.size)][0]
                        + self.vel[ix(i, j + 1, self.size)][1]
                        - self.vel[ix(i, j - 1, self.size)][1])
                    / size;
                self.vel_0[ix(i, j, self.size)][0] = dec!(0);
            }
        }
        self.lin_solve(true, dec!(1), dec!(6));

        for j in 1..(self.size - 1) {
            for i in 1..(self.size - 1) {
                self.vel[ix(i, j, self.size)][0] = multiplier
                    * (self.vel_0[ix(i + 1, j, self.size)][0]
                        - self.vel_0[ix(i - 1, j, self.size)][0])
                    * size
                    - self.vel[ix(i, j, self.size)][0];
                self.vel[ix(i, j, self.size)][1] = multiplier
                    * (self.vel_0[ix(i, j + 1, self.size)][0]
                        - self.vel_0[ix(i, j - 1, self.size)][0])
                    * size
                    - self.vel[ix(i, j, self.size)][1];
            }
        }
        self.set_bnd();
    }
    fn project_inverse(&mut self) {
        let size = Decimal::from_i32((self.size - 2).into()).unwrap_or(dec!(1));
        let multiplier = Decimal::from_f32((-0.5f32).into()).unwrap_or(dec!(1));

        for j in 1..(self.size - 1) {
            for i in 1..(self.size - 1) {
                self.vel[ix(i, j, self.size)][1] = multiplier
                    * (self.vel_0[ix(i + 1, j, self.size)][0]
                        - self.vel_0[ix(i - 1, j, self.size)][0]
                        + self.vel_0[ix(i, j + 1, self.size)][1]
                        - self.vel_0[ix(i, j - 1, self.size)][1])
                    / size;
                self.vel[ix(i, j, self.size)][0] = dec!(0);
            }
        }
        self.lin_solve(true, dec!(1), dec!(6));

        for j in 1..(self.size - 1) {
            for i in 1..(self.size - 1) {
                self.vel_0[ix(i, j, self.size)][0] = multiplier
                    * (self.vel[ix(i + 1, j, self.size)][0] - self.vel[ix(i - 1, j, self.size)][0])
                    * size
                    - self.vel_0[ix(i, j, self.size)][0];
                self.vel_0[ix(i, j, self.size)][1] = multiplier
                    * (self.vel[ix(i, j + 1, self.size)][0] - self.vel[ix(i, j - 1, self.size)][0])
                    * size
                    - self.vel_0[ix(i, j, self.size)][1];
            }
        }
        self.set_bnd();
    }
    fn advect(
        &mut self,
        // int b, float[] d, float[] d0,  float[] velocX, float[] velocY float dt, int N
    ) {
        // float i0, i1, j0, j1;

        // float dtx = dt * (N - 2);
        // float dty = dt * (N - 2);

        // float s0, s1, t0, t1;
        // float tmp1, tmp2, x, y;

        // float Nfloat = N;
        // float ifloat, jfloat;
        // int i, j;

        //     for(j = 1, jfloat = 1; j < N - 1; j++, jfloat++) {
        //         for(i = 1, ifloat = 1; i < N - 1; i++, ifloat++) {
        //             tmp1 = dtx * velocX[IX(i, j)];
        //             tmp2 = dty * velocY[IX(i, j)];
        //             x    = ifloat - tmp1;
        //             y    = jfloat - tmp2;

        //             if(x < 0.5f) x = 0.5f;
        //             if(x > Nfloat + 0.5f) x = Nfloat + 0.5f;
        //             i0 = floorf(x);
        //             i1 = i0 + 1.0f;
        //             if(y < 0.5f) y = 0.5f;
        //             if(y > Nfloat + 0.5f) y = Nfloat + 0.5f;
        //             j0 = floorf(y);
        //             j1 = j0 + 1.0f;
        //

        //             s1 = x - i0;
        //             s0 = 1.0f - s1;
        //             t1 = y - j0;
        //             t0 = 1.0f - t1;
        //

        //             int i0i = i0;
        //             int i1i = i1;
        //             int j0i = j0;
        //             int j1i = j1;
        //

        //             d[IX(i, j)] =
        //                  s0 * (t0 * d0[IX(i0i,j0i)])
        //                      +(t1 * d0[IX(i0i,j1i)])
        //                  +s1 * (t0 * d0[IX(i1i,j0i)])
        //                      +(t1 * d0[IX(i1i,j1i)])

        //         }
        //     }
        // set_bnd(b, d, N);
    }
}
