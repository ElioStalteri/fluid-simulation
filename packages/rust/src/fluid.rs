extern crate vecmath;
use rust_decimal::prelude::*;
use rust_decimal_macros::dec;
use std::convert::TryInto;
use wasm_bindgen::prelude::*;
use std::cmp;

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
    pub density_0: Vec<Decimal>,
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
            density_0: vec![dec!(0); (size * size).try_into().unwrap()],
            density: vec![dec!(0); (size * size).try_into().unwrap()],
            vel: vec![[dec!(0); 2]; (size * size).try_into().unwrap()],
            vel_0: vec![[dec!(0); 2]; (size * size).try_into().unwrap()],
        }
    }
    pub fn add_density(&mut self, x: i32, y: i32, amount: Decimal) {
        self.density[ix(x, y, self.size)] += amount;
    }
    pub fn add_velocity(&mut self, x: i32, y: i32, v: vecmath::Vector2<Decimal>) {
        self.vel[ix(x, y, self.size)] = vecmath::vec2_add(self.vel[ix(x, y, self.size)], v);
    }
    pub fn step(&mut self) {
        self.vel_0 = self.diffuse_vector(self.vel_0.clone(), self.vel.clone());
        // diffuse(1, Vx0, Vx, visc, dt, 4, N);
        // diffuse(2, Vy0, Vy, visc, dt, 4, N);
        // diffuse(3, Vz0, Vz, visc, dt, 4, N);

        // let res = self.project(self.vel_0.clone(), self.vel.clone());
        // self.vel_0 = res[0].clone();
        // self.vel = res[1].clone();
        // project(Vx0, Vy0, Vz0, Vx, Vy, 4, N);

        self.vel = self.advect_vector(self.vel.clone(),self.vel_0.clone(),self.vel_0.clone());
        // advect(1, Vx, Vx0, Vx0, Vy0, Vz0, dt, N);
        // advect(2, Vy, Vy0, Vx0, Vy0, Vz0, dt, N);
        // advect(3, Vz, Vz0, Vx0, Vy0, Vz0, dt, N);

        // let res1 = self.project(self.vel.clone(), self.vel_0.clone());
        // self.vel = res1[0].clone();
        // self.vel_0 = res1[1].clone();
        // project(Vx, Vy, Vz, Vx0, Vy0, 4, N);

        self.density_0 = self.diffuse_value(self.density_0.clone(), self.density.clone());
        // diffuse(0, s, density, diff, dt, 4, N);

        self.density = self.advect_value(self.density.clone(),self.density_0.clone(),self.vel.clone());
        // advect(0, density, s, Vx, Vy, Vz, dt, N);
    }
    
    fn diffuse_vector(
        &mut self,
        r: Vec<vecmath::Vector2<Decimal>>,
        r_0: Vec<vecmath::Vector2<Decimal>>,
    ) -> Vec<vecmath::Vector2<Decimal>> {
        let r_ = r;
        let r_0_ = r_0;
        let size = Decimal::from_i32((self.size - 2).into()).unwrap_or(dec!(8888));
        // log("convert size to decimal if 9999 or 8888 it didnt work!!");
        // log_u32(size.to_u32().unwrap_or(9999));
        let a = self.dt * self.diff * size * size;
        let c = dec!(1) + dec!(6) * a;
        return self.lin_solve_vector(r_, r_0_, a, c);
    }
    fn diffuse_value(&mut self, r: Vec<Decimal>, r_0: Vec<Decimal>) -> Vec<Decimal> {
        let r_ = r;
        let r_0_ = r_0;
        let size = Decimal::from_i32((self.size - 2).into()).unwrap_or(dec!(8888));
        // log("convert size to decimal if 9999 or 8888 it didnt work!!");
        // log_u32(size.to_u32().unwrap_or(9999));
        let a = self.dt * self.diff * size * size;
        let c = dec!(1) + dec!(6) * a;
        return self.lin_solve_value(r_, r_0_, a, c);
    }
    
    fn project(
        &mut self,
        r: Vec<vecmath::Vector2<Decimal>>,
        r_0: Vec<vecmath::Vector2<Decimal>>,
    ) -> [Vec<vecmath::Vector2<Decimal>>; 2] {
        let mut r_ = r;
        let mut r_0_ = r_0;
        let size = Decimal::from_i32((self.size).into()).unwrap_or(dec!(1));
        let multiplier = Decimal::from_f32((-0.5f32).into()).unwrap_or(dec!(1));

        for j in 1..(self.size - 1) {
            for i in 1..(self.size - 1) {
                r_0_[ix(i, j, self.size)][1] = multiplier
                    * (r_[ix(i + 1, j, self.size)][0] - r_[ix(i - 1, j, self.size)][0]
                        + r_[ix(i, j + 1, self.size)][1]
                        - r_[ix(i, j - 1, self.size)][1])
                    / size;
                r_0_[ix(i, j, self.size)][0] = dec!(0);
            }
        }
        r_0_ = self.set_bnd_vector(r_0_);
        let mut p = r_0_.iter().map(|v| v[0]).collect::<Vec<Decimal>>();
        let div = r_0_.iter().map(|v| v[1]).collect::<Vec<Decimal>>();
        let div2 = div.clone();
        p = self.lin_solve_value(p, div, dec!(1), dec!(6));
        let mut tmp: Vec<vecmath::Vector2<Decimal>> = vec!(); 
        for i in 0..p.len() {
            tmp.push([p[i],div2[i]])
        }
        r_0_ = tmp;

        for j in 1..(self.size - 1) {
            for i in 1..(self.size - 1) {
                r_[ix(i, j, self.size)][0] =
                    multiplier * (p[ix(i + 1, j, self.size)] - p[ix(i - 1, j, self.size)]) * size
                        - r_[ix(i, j, self.size)][0];
                r_[ix(i, j, self.size)][1] =
                    multiplier * (p[ix(i, j + 1, self.size)] - p[ix(i, j - 1, self.size)]) * size
                        - r_[ix(i, j, self.size)][1];
            }
        }
        r_ = self.set_bnd_vector(r_);
        return [r_, r_0_];
    }
    
    fn advect_vector(
        &mut self,
        r: Vec<vecmath::Vector2<Decimal>>,
        r_0: Vec<vecmath::Vector2<Decimal>>,
        vel: Vec<vecmath::Vector2<Decimal>>,
    ) -> Vec<vecmath::Vector2<Decimal>> {
        let mut r_ = r;
        let r_0_ = r_0;

        let size = Decimal::from_i32((self.size).into()).unwrap_or(dec!(1));

        let dt_ = self.dt * (size - dec!(2));

        for j in 1..(self.size - 1) {
            for i in 1..(self.size - 1) {
                let tmp = vecmath::vec2_scale(vel[ix(i, j, self.size)], dt_);

                let mut x = Decimal::from_i32(i).unwrap() - tmp[0];
                let mut y = Decimal::from_i32(j).unwrap() - tmp[1];

                if x < dec!(0.5) {
                    x = dec!(0.5);
                }
                if x > size + dec!(0.5) {
                    x = size + dec!(0.5);
                }
                let i0 = x.floor();
                let i1 = i0 + dec!(1);
                if y < dec!(0.5) {
                    y = dec!(0.5);
                }
                if y > size + dec!(0.5) {
                    y = size + dec!(0.5);
                }
                let j0 = y.floor();
                let j1 = j0 + dec!(1.0);

                let s1 = x - i0;
                let s0 = dec!(1) - s1;
                let t1 = y - j0;
                let t0 = dec!(1) - t1;

                let i0i = i0.floor().to_i32().unwrap();
                let i1i = i1.floor().to_i32().unwrap();
                let j0i = j0.floor().to_i32().unwrap();
                let j1i = j1.floor().to_i32().unwrap();

                r_[ix(i, j, self.size)] = vecmath::vec2_add(
                    vecmath::vec2_scale(
                        vecmath::vec2_add(
                            vecmath::vec2_scale(r_0_[cmp::min(ix(i0i, j0i, self.size),(self.size - 1) as usize)], t0),
                            vecmath::vec2_scale(r_0_[cmp::min(ix(i0i, j1i, self.size),(self.size - 1) as usize)], t1),
                        ),
                        s0,
                    ),
                    vecmath::vec2_scale(
                        vecmath::vec2_add(
                            vecmath::vec2_scale(r_0_[cmp::min(ix(i1i, j0i, self.size),(self.size - 1) as usize)], t0),
                            vecmath::vec2_scale(r_0_[cmp::min(ix(i1i, j1i, self.size),(self.size - 1) as usize)], t1),
                        ),
                        s1,
                    ),
                );
            }
        }
        r_ = self.set_bnd_vector(r_);
        return r_;
    }
    fn advect_value(
        &mut self,
        r: Vec<Decimal>,
        r_0: Vec<Decimal>,
        vel: Vec<vecmath::Vector2<Decimal>>,
    ) -> Vec<Decimal> {
        let mut r_ = r;
        let r_0_ = r_0;

        let size = Decimal::from_i32((self.size).into()).unwrap_or(dec!(1));

        let dt_ = self.dt * (size - dec!(2));

        for j in 1..(self.size - 1) {
            for i in 1..(self.size - 1) {
                let tmp = vecmath::vec2_scale(vel[ix(i, j, self.size)], dt_);

                let mut x = Decimal::from_i32(i).unwrap() - tmp[0];
                let mut y = Decimal::from_i32(j).unwrap() - tmp[1];

                if x < dec!(0.5) {
                    x = dec!(0.5);
                }
                if x > size + dec!(0.5) {
                    x = size + dec!(0.5);
                }
                let i0 = x.floor();
                let i1 = i0 + dec!(1);
                if y < dec!(0.5) {
                    y = dec!(0.5);
                }
                if y > size + dec!(0.5) {
                    y = size + dec!(0.5);
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

                r_[ix(i, j, self.size)] =
                    (r_0_[ix(i0i, j0i, self.size)] * t0 + r_0_[ix(i0i, j1i, self.size)] * t1) * s0
                        + (r_0_[ix(i1i, j0i, self.size)] * t0 + r_0_[ix(i1i, j1i, self.size)] * t1)
                            * s1;
            }
        }
        return r_;
    }

    fn set_bnd_vector(
        &mut self,
        r: Vec<vecmath::Vector2<Decimal>>,
    ) -> Vec<vecmath::Vector2<Decimal>> {
        let mut r_ = r;

        let n = self.size - 1;
        // loop all borders and negate the array
        for i in 1..n {
            r_[ix(i, 0, self.size)] = vecmath::vec2_neg(r_[ix(i, 1, self.size)]);
            r_[ix(i, n, self.size)] = vecmath::vec2_neg(r_[ix(i, self.size - 2, self.size)]);
            r_[ix(0, i, self.size)] = vecmath::vec2_neg(r_[ix(1, i, self.size)]);
            r_[ix(n, i, self.size)] = vecmath::vec2_neg(r_[ix(self.size - 2, i, self.size)]);
        }
        let len1 = (r_[ix(0, 0, self.size)][0].powu(2) + r_[ix(0, 0, self.size)][1].powu(2))
            .sqrt()
            .unwrap_or(dec!(1));
        r_[ix(0, 0, self.size)] = vecmath::vec2_scale([dec!(1), dec!(1)], len1);

        let len2 = (r_[ix(0, n, self.size)][0].powu(2)
            + r_[ix(0, n, self.size)][1].powu(2))
        .sqrt()
        .unwrap_or(dec!(1));
        r_[ix(0, n, self.size)] = vecmath::vec2_scale([dec!(1), dec!(-1)], len2);

        let len3 = (r_[ix(n, 0, self.size)][0].powu(2)
            + r_[ix(n, 0, self.size)][1].powu(2))
        .sqrt()
        .unwrap_or(dec!(1));
        r_[ix(n, 0, self.size)] = vecmath::vec2_scale([dec!(-1), dec!(1)], len3);

        let len4 = (r_[ix(n, n, self.size)][0].powu(2)
            + r_[ix(n, n, self.size)][1].powu(2))
        .sqrt()
        .unwrap_or(dec!(1));
        r_[ix(n, n, self.size)] =
            vecmath::vec2_scale([dec!(-1), dec!(-1)], len4);
        return r_;
    }

    fn lin_solve_vector(
        &mut self,
        r: Vec<vecmath::Vector2<Decimal>>,
        r_0: Vec<vecmath::Vector2<Decimal>>,
        a: Decimal,
        c: Decimal,
    ) -> Vec<vecmath::Vector2<Decimal>> {
        let mut r_ = r;
        let r_0_ = r_0;

        let c_recip = dec!(1.0) / c;
        for _k in 0..self.iter {
            for js in 1..(self.size - 1) {
                for is in 1..(self.size - 1) {
                    let j = js as i32;
                    let i = is as i32;
                    r_[ix(i, j, self.size)] = vecmath::vec2_scale(
                        vecmath::vec2_add(
                            r_0_[ix(i, j, self.size)],
                            vecmath::vec2_scale(
                                vecmath::vec2_add(
                                    vecmath::vec2_add(
                                        r_[ix(i + 1, j, self.size)],
                                        r_[ix(i - 1, j, self.size)],
                                    ),
                                    vecmath::vec2_add(
                                        r_[ix(i, j + 1, self.size)],
                                        r_[ix(i, j - 1, self.size)],
                                    ),
                                ),
                                a,
                            ),
                        ),
                        c_recip,
                    );
                }
            }
            r_ = self.set_bnd_vector(r_);
        }
        return r_;
    }
    
    fn lin_solve_value(
        &mut self,
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
                    r_[ix(i, j, self.size)] = (
                            r_0_[ix(i, j, self.size)]
                            + (
                                r_[ix(i + 1, j, self.size)]
                                + r_[ix(i - 1, j, self.size)]
                                + r_[ix(i, j + 1, self.size)]
                                + r_[ix(i, j - 1, self.size)]
                            ) * a
                        ) * c_recip;
                }
            }
        }
        return r_;
    }
}
