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

        // advect_velocity(1, Vx, Vx0, Vx0, Vy0, dt, N);
        // advect_velocity(2, Vy, Vy0, Vx0, Vy0, dt, N);

        self.project_inverse();

        self.diffuse(false);
        // advect_density(0, density, s, Vx, Vy, dt, N);
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
        for i in 1..(self.size - 1) {
            self.vel_0[ix(i, 0, self.size)] = vecmath::vec2_neg(self.vel_0[ix(i, 1, self.size)]);
            self.vel_0[ix(i, (self.size - 1), self.size)] =
                vecmath::vec2_neg(self.vel_0[ix(i, (self.size - 2), self.size)]);
            self.vel_0[ix(0, i, self.size)] = vecmath::vec2_neg(self.vel_0[ix(1, i, self.size)]);
            self.vel_0[ix((self.size - 1), i, self.size)] =
                vecmath::vec2_neg(self.vel_0[ix((self.size - 2), i, self.size)]);
        }
        let len1 = (self.vel_0[ix(0, 0, self.size)][0].powu(2)
            + self.vel_0[ix(0, 0, self.size)][1].powu(2))
        .sqrt()
        .unwrap_or(dec!(1));
        self.vel_0[ix(0, 0, self.size)] = vecmath::vec2_scale([dec!(1), dec!(1)], len1);

        let len2 = (self.vel_0[ix(0, self.size - 1, self.size)][0].powu(2)
            + self.vel_0[ix(0, self.size - 1, self.size)][1].powu(2))
        .sqrt()
        .unwrap_or(dec!(1));
        self.vel_0[ix(0, self.size - 1, self.size)] =
            vecmath::vec2_scale([dec!(1), dec!(-1)], len2);

        let len3 = (self.vel_0[ix(self.size - 1, 0, self.size)][0].powu(2)
            + self.vel_0[ix(self.size - 1, 0, self.size)][1].powu(2))
        .sqrt()
        .unwrap_or(dec!(1));
        self.vel_0[ix(self.size - 1, 0, self.size)] =
            vecmath::vec2_scale([dec!(-1), dec!(1)], len3);

        let len4 = (self.vel_0[ix(self.size - 1, self.size - 1, self.size)][0].powu(2)
            + self.vel_0[ix(self.size - 1, self.size - 1, self.size)][1].powu(2))
        .sqrt()
        .unwrap_or(dec!(1));
        self.vel_0[ix(self.size - 1, self.size - 1, self.size)] =
            vecmath::vec2_scale([dec!(-1), dec!(-1)], len4);
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
    // fn advect_density
    fn advect_velocity(
        &mut self,
        // int b, float[] d, float[] d0,  float[] velocX, float[] velocY float dt, int N
    ) {
        // float i0, i1, j0, j1;
        let size = Decimal::from_i32((self.size).into()).unwrap_or(dec!(1));

        let dt_ = self.dt * (size - dec!(2));
        // float dty = dt * (N - 2);

        // float s0, s1, t0, t1;
        // float tmp1, tmp2, x, y;

        // float Nfloat = N;
        // float ifloat, jfloat;
        // int i, j;

        for j in 1..(self.size - 1) {
            for i in 1..(self.size - 1) {
                let tmp = vecmath::vec2_scale(self.vel_0[ix(i, j, self.size)], dt_);
                // tmp1 = dtx * velocX[IX(i, j)];
                //             tmp2 = dty * velocY[IX(i, j)];
                let mut x = Decimal::from_i32(i).unwrap() - tmp[0];
                let mut y = Decimal::from_i32(j).unwrap() - tmp[1];

                if x < dec!(0.5) {
                    x = dec!(0.5);
                }
                if x > size + dec!(0.5) {
                    x = size + dec!(0.5);
                }
                let i0 = x;
                let i1 = i0 + dec!(1);
                if y < dec!(0.5) {
                    y = dec!(0.5);
                }
                if y > size + dec!(0.5) {
                    y = size + dec!(0.5);
                }
                let j0 = y;
                let j1 = j0 + dec!(1.0);

                let s1 = x - i0;
                let s0 = dec!(1) - s1;
                let t1 = y - j0;
                let t0 = dec!(1) - t1;

                let i0i = i0.to_i32();
                let i1i = i1.to_i32();
                let j0i = j0.to_i32();
                let j1i = j1.to_i32();
                //

                self.vel[ix(i, j, self.size)] = vec2_add(
                    vec2_scale(
                        vec2_add(
                            vec2_scale(self.vel_0[ix(i0i, j0i, self.size)], t0),
                            vec2_scale(self.vel_0[ix(i0i, j1i, self.size)], t1),
                        ),
                        s0,
                    ),
                    vec2_scale(
                        vec2_add(
                            vec2_scale(self.vel_0[ix(i1i, j0i, self.size)], t0),
                            vec2_scale(self.vel_0[ix(i1i, j1i, self.size)], t1),
                        ),
                        s1,
                    ),
                );

                // self.vel[ix(i, j, self.size)] = s0
                // ( self.vel_0[ix(i0i, j0i, self.size)]*t0
                //         +  self.vel_0[ix(i0i, j1i, self.size)]*t1 ) * s0
                //     +   (  self.vel_0[ix(i1i, j0i, self.size)]*t0
                //         +   self.vel_0[ix(i1i, j1i, self.size)]*t1) * s1;
            }
        }
        // set_bnd(b, d, N);
    }
}
