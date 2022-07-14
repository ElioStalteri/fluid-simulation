extern crate vecmath;
use rust_decimal::prelude::*;
use rust_decimal_macros::dec;
use std::convert::TryInto;

fn IX(x: i32, y: i32, size: i32) -> usize {
    (x + y * size) as usize
}

fn lin_solve(
    mut v_0: Vec<vecmath::Vector2<Decimal>>,
    v: Vec<vecmath::Vector2<Decimal>>,
    a: Decimal,
    c: Decimal,
    iter: i32,
    n: i32,
    size: i32,
) {
    let c_recip = dec!(1.0) / c;
    for _k in 0..iter {
        for js in 1..(n - 1) {
            for is in 1..(n - 1) {
                let j = js as i32;
                let i = is as i32;
                v_0[IX(i, j, size)] = vecmath::vec2_scale(
                    vecmath::vec2_add(
                        v[IX(i, j, size)],
                        vecmath::vec2_scale(
                            vecmath::vec2_add(
                                vecmath::vec2_add(v_0[IX(i + 1, j, size)], v_0[IX(i - 1, j, size)]),
                                vecmath::vec2_add(v_0[IX(i, j + 1, size)], v_0[IX(i, j - 1, size)]),
                            ),
                            a,
                        ),
                    ),
                    c_recip,
                );
            }
        }
    }
}

pub struct Fluid {
    pub size: i32,
    pub dt: Decimal,
    pub diff: Decimal,
    pub visc: Decimal,
    pub s: Vec<Decimal>,
    pub density: Vec<Decimal>,
    pub v: Vec<vecmath::Vector2<Decimal>>,
    pub v_0: Vec<vecmath::Vector2<Decimal>>,
}

impl Fluid {
    pub fn create(size: i32, diffusion: Decimal, viscosity: Decimal, dt: Decimal) -> Fluid {
        Self {
            size,
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
}
