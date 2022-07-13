extern crate vecmath;
use rust_decimal::prelude::*;
use rust_decimal_macros::dec;
use std::convert::TryInto;

fn IX(x: i32, y: i32, size: i32) -> usize {
    (x + y * size) as usize
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
}
