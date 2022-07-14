extern crate vecmath;
use rust_decimal::prelude::*;
use rust_decimal_macros::dec;
use std::convert::TryInto;

fn IX(x: i32, y: i32, size: i32) -> usize {
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
        self.density[IX(x, y, self.size)] += amount;
    }
    pub fn add_valocity(&mut self, x: i32, y: i32, v: vecmath::Vector2<Decimal>) {
        self.v[IX(x, y, self.size)] = vecmath::vec2_add(self.v[IX(x, y, self.size)], v);
    }
    fn lin_solve(&mut self, a: Decimal, c: Decimal) {
        let c_recip = dec!(1.0) / c;
        for _k in 0..self.iter {
            for js in 1..(self.size - 1) {
                for is in 1..(self.size - 1) {
                    let j = js as i32;
                    let i = is as i32;
                    self.v_0[IX(i, j, self.size)] = vecmath::vec2_scale(
                        vecmath::vec2_add(
                            self.v[IX(i, j, self.size)],
                            vecmath::vec2_scale(
                                vecmath::vec2_add(
                                    vecmath::vec2_add(
                                        self.v_0[IX(i + 1, j, self.size)],
                                        self.v_0[IX(i - 1, j, self.size)],
                                    ),
                                    vecmath::vec2_add(
                                        self.v_0[IX(i, j + 1, self.size)],
                                        self.v_0[IX(i, j - 1, self.size)],
                                    ),
                                ),
                                a,
                            ),
                        ),
                        c_recip,
                    );
                }
            }
            self.set_bnd();
        }
    }
    fn set_bnd(&mut self) {
        // loop all borders and negate the array
        for i in 0..(self.size - 1) {
            self.v_0[IX(i, 0, self.size)] = vecmath::vec2_neg(self.v_0[IX(i, 1, self.size)]);
            self.v_0[IX(i, (self.size - 1), self.size)] =
                vecmath::vec2_neg(self.v_0[IX(i, (self.size - 2), self.size)]);
            self.v_0[IX(0, i, self.size)] = vecmath::vec2_neg(self.v_0[IX(1, i, self.size)]);
            self.v_0[IX((self.size - 1), i, self.size)] =
                vecmath::vec2_neg(self.v_0[IX((self.size - 2), i, self.size)]);
        }
    }
    fn diffuse(&mut self) {
        let size = Decimal::try_new((self.size - 2).into(), u32::MAX).unwrap_or(dec!(0));
        let a = self.dt * self.diff * size * size;
        self.lin_solve(a, dec!(1) + dec!(6) * a);
    }
}
