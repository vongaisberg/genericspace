extern crate wasm_bindgen;

mod physics;
mod types;
mod utils;

use physics::{PhysicsObject, PhysicsSpace};
use types::EuclideanSpace;
use types::Field;
use wasm_bindgen::prelude::*;

use rand::os::OsRng;
use rand::Rng;
// When the `wee_alloc` feature is enabled, use `wee_alloc` as the global
// allocator.
#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

impl Field for f64 {}

#[wasm_bindgen]
extern "C" {
    fn alert(s: &str);
}

#[wasm_bindgen]
pub fn greet() {
    alert("Hello, asdasdaswasm-generic-space!");
}

#[wasm_bindgen]
pub struct VisibleUniverse {
    elems: Vec<[f64; 2]>,
}
#[wasm_bindgen]
pub struct Vector {
    value: [f64; 2],
}
#[wasm_bindgen]
impl Vector {
    pub fn getX(&self) -> f64 {
        self.value[0]
    }

    pub fn getY(&self) -> f64 {
        self.value[1]
    }
}

#[wasm_bindgen]
impl VisibleUniverse {
    pub fn length(&self) -> u32 {
        self.elems.len() as u32
    }

    pub fn get(&self, i: usize) -> Vector {
        Vector {
            value: *self.elems.get(i).unwrap(),
        }
    }
}
#[wasm_bindgen]
pub struct Universe {
    phys: PhysicsSpace<f64, EuclideanSpace<f64>>,
}
#[wasm_bindgen]
impl Universe {
    pub fn new() -> Universe {
        //let mut rng = rand::thread_rng();

        let mut rng = OsRng::new().unwrap();;


        let mut elems = Vec::new();

    let speed_range = 1.0;

        for i in 0..100 {
            elems.push(PhysicsObject::<f64>::new(
                [rng.gen_range(50.0, 800.0), rng.gen_range(50.0, 800.0)],
                [rng.gen_range(-speed_range, speed_range), rng.gen_range(-speed_range, speed_range)],
                rng.gen_range(1.0, 2.0),
            ))
        }

        Universe {
            phys: PhysicsSpace::new(
                elems,
                10f64,
                EuclideanSpace::<f64> {
                    field: std::marker::PhantomData::<f64>,
                },
                3000f64,
                5f64,
            ),
        }
    }

    pub fn tick(&mut self) -> VisibleUniverse {
        self.phys.tick();
        VisibleUniverse {
            elems: self
                .phys
                .elements
                .iter()
                .map(|e| e.position_vector)
                .collect(),
        }
    }
}
