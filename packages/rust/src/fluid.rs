extern crate vecmath;
use rust_decimal::prelude::*;
use rust_decimal_macros::dec;
use std::cmp;
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

    #[wasm_bindgen(js_namespace = console, js_name = log)]
    fn log_f64(a: f64);

    #[wasm_bindgen(js_namespace = console, js_name = log)]
    fn log_f64v(a: Vec<f64>);

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
    pub density_0: Vec<Decimal>,
    pub density: Vec<Decimal>,
    pub velx: Vec<Decimal>,
    pub vely: Vec<Decimal>,
    pub velx0: Vec<Decimal>,
    pub vely0: Vec<Decimal>,
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
            density_0: vec![dec!(0); (size * size).try_into().unwrap()],
            density: vec![dec!(0); (size * size).try_into().unwrap()],
            velx: vec![dec!(0); (size * size).try_into().unwrap()],
            vely: vec![dec!(0); (size * size).try_into().unwrap()],
            velx0: vec![dec!(0); (size * size).try_into().unwrap()],
            vely0: vec![dec!(0); (size * size).try_into().unwrap()],
        }
    }
    pub fn add_density(&mut self, x: i32, y: i32, amount: Decimal) {
        self.density[ix(x, y, self.size)] += amount;
    }
    pub fn add_velocity(&mut self, x: i32, y: i32, v: vecmath::Vector2<Decimal>) {
        self.velx[ix(x, y, self.size)] = self.velx[self.ix(x, y)] + v[0];
        self.vely[ix(x, y, self.size)] = self.vely[self.ix(x, y)] + v[1];
    }
    fn ix(&self, x: i32, y: i32) -> usize {
        (x + y * self.size) as usize
    }
    pub fn step(&mut self) {
        self.velx0 = self.diffuse(1, self.velx0.clone(), self.velx.clone(), self.visc);
        self.vely0 = self.diffuse(2, self.vely0.clone(), self.vely.clone(), self.visc);

        [self.velx0, self.vely0, self.velx, self.vely] = self.project(
            self.velx0.clone(),
            self.vely0.clone(),
            self.velx.clone(),
            self.vely.clone(),
        );

        // log("self.velx");
        // log_f64v(self.velx.iter().map(|v| v.to_f64().unwrap()).collect());
        self.velx = self.advect(
            1,
            self.velx.clone(),
            self.velx0.clone(),
            self.velx0.clone(),
            self.vely0.clone(),
        );
        // log("self.velx");
        // log_f64v(self.velx.iter().map(|v| v.to_f64().unwrap()).collect());

        self.vely = self.advect(
            2,
            self.vely.clone(),
            self.vely0.clone(),
            self.velx0.clone(),
            self.vely0.clone(),
        );
        // log("self.vely");
        // log_f64v(self.vely.iter().map(|v| v.to_f64().unwrap()).collect());

        [self.velx, self.vely, self.velx0, self.vely0] = self.project(
            self.velx.clone(),
            self.vely.clone(),
            self.velx0.clone(),
            self.vely0.clone(),
        );

        self.density_0 = self.diffuse(0, self.density_0.clone(), self.density.clone(), self.diff);

        self.density = self.advect(
            0,
            self.density.clone(),
            self.density_0.clone(),
            self.velx0.clone(),
            self.vely0.clone(),
        );
    }

    fn diffuse(
        &mut self,
        b: i32,
        r: Vec<Decimal>,
        r_0: Vec<Decimal>,
        diff: Decimal,
    ) -> Vec<Decimal> {
        let r_ = r;
        let r_0_ = r_0;
        let size = Decimal::from_i32((self.size - 2).into()).unwrap_or(dec!(8888));
        // log("convert size to decimal if 9999 or 8888 it didnt work!!");
        // log_u32(size.to_u32().unwrap_or(9999));
        let a = self.dt * diff * size * size;
        let c = dec!(1) + dec!(6) * a;
        return self.lin_solve(b, r_, r_0_, a, c);
    }

    fn project(
        &mut self,
        rx: Vec<Decimal>,
        ry: Vec<Decimal>,
        rx0: Vec<Decimal>,
        ry0: Vec<Decimal>,
    ) -> [Vec<Decimal>; 4] {
        let mut rx_ = rx;
        let mut ry_ = ry;
        let mut rx0_ = rx0;
        let mut ry0_ = ry0;

        let size = Decimal::from_i32((self.size).into()).unwrap_or(dec!(1));
        let multiplier = Decimal::from_f32((-0.5f32).into()).unwrap_or(dec!(1));

        for j in 1..(self.size - 1) {
            for i in 1..(self.size - 1) {
                ry0_[self.ix(i, j)] = multiplier
                    * (rx_[self.ix(i + 1, j)] - rx_[self.ix(i - 1, j)] + ry_[self.ix(i, j + 1)]
                        - ry_[self.ix(i, j - 1)])
                    / size;
                rx0_[self.ix(i, j)] = dec!(0);
            }
        }

        ry0_ = self.set_bnd(0, ry0_);
        rx0_ = self.set_bnd(0, rx0_);
        rx0_ = self.lin_solve(0, rx0_, ry0_.clone(), dec!(1), dec!(6));

        for j in 1..(self.size - 1) {
            for i in 1..(self.size - 1) {
                rx_[self.ix(i, j)] =
                    multiplier * (rx0_[self.ix(i + 1, j)] - rx0_[self.ix(i - 1, j)]) * size
                        - rx_[self.ix(i, j)];
                ry_[self.ix(i, j)] =
                    multiplier * (rx0_[self.ix(i, j + 1)] - rx0_[self.ix(i, j - 1)]) * size
                        - ry_[self.ix(i, j)];
            }
        }

        rx_ = self.set_bnd(1, rx_);
        ry_ = self.set_bnd(2, ry_);

        return [rx_, ry_, rx0_, ry0_];
    }

    fn advect(
        &mut self,
        b: i32,
        r: Vec<Decimal>,
        r0: Vec<Decimal>,
        vx: Vec<Decimal>,
        vy: Vec<Decimal>,
    ) -> Vec<Decimal> {
        let mut r_ = r;
        let r0_ = r0;

        let size = Decimal::from_i32((self.size).into()).unwrap_or(dec!(1));

        let dt_ = self.dt * (size - dec!(2));
        // log_f64v(r_.iter().map(|x| x.to_f64().unwrap()).collect());
        // log_f64v(r0_.iter().map(|x| x.to_f64().unwrap()).collect());
        for j in 1..(self.size - 1) {
            for i in 1..(self.size - 1) {
                let tmpx = vx[ix(i, j, self.size)] * dt_;
                let tmpy = vy[ix(i, j, self.size)] * dt_;

                let mut x = Decimal::from_i32(i).unwrap() - tmpx;
                let mut y = Decimal::from_i32(j).unwrap() - tmpy;

                if x < dec!(0.5) {
                    x = dec!(0.5);
                }
                if x > size - dec!(1.5) {
                    x = size - dec!(1.5);
                }
                let i0 = x.floor();
                let i1 = i0 + dec!(1);
                if y < dec!(1.5) {
                    y = dec!(1.5);
                }
                if y > size - dec!(1.5) {
                    y = size - dec!(1.5);
                }
                let j0 = y.floor();
                let j1 = j0 + dec!(1.0);

                let s1 = x - i0;
                let s0 = dec!(1) - s1;
                let t1 = y - j0;
                let t0 = dec!(1) - t1;

                let i0i = i0.to_i32().unwrap();
                let i1i = i1.to_i32().unwrap();
                let j0i = j0.to_i32().unwrap();
                let j1i = j1.to_i32().unwrap();

                
                r_[self.ix(i, j)] = 
                    (
                        r0_[self.ix(i0i, j0i)] * t0
                        + 
                        r0_[self.ix(i0i, j1i)] * t1
                    ) * s0
                    + 
                    (
                        r0_[self.ix(i1i, j0i)] * t0
                        + 
                        r0_[self.ix(i1i, j1i)] * t1
                    ) * s1;
                
            }
        }
        
        // log_f64v(r_.iter().map(|x| x.to_f64().unwrap()).collect());
        // log_f64v(r0_.iter().map(|x| x.to_f64().unwrap()).collect());

        r_ = self.set_bnd(b, r_);

        return r_;
    }

    fn set_bnd(&mut self, b: i32, r: Vec<Decimal>) -> Vec<Decimal> {
        let mut x = r;
        let N = self.size;

        for i in 1..(N - 1) {
            x[self.ix(i, 0)] = if b == 2 {
                -x[self.ix(i, 1)]
            } else {
                x[self.ix(i, 1)]
            };
            x[self.ix(i, N - 1)] = if b == 2 {
                -x[self.ix(i, N - 2)]
            } else {
                x[self.ix(i, N - 2)]
            };
        }
        for j in 1..(N - 1) {
            x[self.ix(0, j)] = if b == 1 {
                -x[self.ix(1, j)]
            } else {
                x[self.ix(1, j)]
            };
            x[self.ix(N - 1, j)] = if b == 1 {
                -x[self.ix(N - 2, j)]
            } else {
                x[self.ix(N - 2, j)]
            };
        }

        x[self.ix(0, 0)] = dec!(0.33) * (x[self.ix(1, 0)] + x[self.ix(0, 1)]);
        x[self.ix(0, N - 1)] = dec!(0.33) * (x[self.ix(1, N - 1)] + x[self.ix(0, N - 2)]);
        x[self.ix(N - 1, 0)] = dec!(0.33) * (x[self.ix(N - 2, 0)] + x[self.ix(N - 1, 1)]);
        x[self.ix(N - 1, N - 1)] =
            dec!(0.33) * (x[self.ix(N - 2, N - 1)] + x[self.ix(N - 1, N - 2)]);

        return x;
    }

    fn lin_solve(
        &mut self,
        b: i32,
        r: Vec<Decimal>,
        r_0: Vec<Decimal>,
        a: Decimal,
        c: Decimal,
    ) -> Vec<Decimal> {
        let mut r_ = r;
        let r_0_ = r_0;

        let c_recip = dec!(1.0) / c;
        for _k in 0..self.iter {
            for js in 1..(self.size - 1) {
                for is in 1..(self.size - 1) {
                    let j = js as i32;
                    let i = is as i32;
                    r_[self.ix(i, j)] = (r_0_[self.ix(i, j)]
                        + (r_[self.ix(i + 1, j)]
                            + r_[self.ix(i - 1, j)]
                            + r_[self.ix(i, j + 1)]
                            + r_[self.ix(i, j - 1)])
                            * a)
                        * c_recip;
                }
            }
            r_ = self.set_bnd(b, r_);
        }
        return r_;
    }
}
