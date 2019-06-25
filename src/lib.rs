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
        let mut elems = vec![PhysicsObject::<f64>::new(
            [200f64, 400f64],
            [0f64, 0f64],
            150f64,
        )];
        elems.pop();

        // let mut elems: Vec<PhysicsObject::<f64>>::new();
        // for i in 0..60 {
        //     for j in 0..60 {
        //         //seed = (seed * 0x5DEECE66Du64 + 0xBu64) & ((1u64 << 48) - 1);

        //         elems.push(PhysicsObject::<f64>::new(
        //             [(40.0 * i as f64) + 400.0, (40.0 * j as f64) + 400.0],
        //             [0f64, 0f64],
        //             150f64,
        //         ));
        //     }
        // }
        // elems.push(PhysicsObject::<f64>::new(
        //     [2800f64, 2800f64],
        //     [-2f64, -0f64],
        //     30000f64,
        // ));

        for i in 0..100 {
            elems.push(PhysicsObject::<f64>::new(
                [rng.gen_range(200.0, 1000.0), rng.gen_range(200.0, 900.0)],
                [rng.gen_range(-4.0, 4.0), rng.gen_range(-4.0, 4.0)],
                rng.gen_range(100.0, 200.0),
            ))
        }

        // let mut elems = vec![
        //     PhysicsObject::<f64>::new([200f64, 400f64], [0f64, 0f64], 150f64),
        //     PhysicsObject::<f64>::new([200f64, 200f64], [0f64, 0f64], 150f64),
        //     PhysicsObject::<f64>::new([200f64, 109f64], [0f64, 0f64], 150f64),
        //     PhysicsObject::<f64>::new([250f64, 100f64], [0f64, 0f64], 150f64),
        //     PhysicsObject::<f64>::new([300f64, 240f64], [0f64, 0f64], 150f64),
        //     PhysicsObject::<f64>::new([270f64, 140f64], [0f64, 0f64], 150f64),
        //     PhysicsObject::<f64>::new([400f64, 40f64], [0f64, 0f64], 150f64),
        //     PhysicsObject::<f64>::new([350f64, 120f64], [0f64, 0f64], 150f64),
        //     PhysicsObject::<f64>::new([400f64, 400f64], [0f64, 0f64], 150f64),
        // ];

        // for i in 1..20 {
        //     elems.push(PhysicsObject::<f64>::new(
        //         [rng.gen_range(100.0, 300.0), rng.gen_range(100.0, 300.0)],
        //         // [rng.gen_range(-0.002, 0.002), rng.gen_range(-0.02, 0.02)],
        //         [0., 0.],
        //         //rng.gen_range(50.0, 60.0),
        //         1.0,
        //     ))
        // }

        Universe {
            phys: PhysicsSpace::new(
                elems,
                0.3f64,
                EuclideanSpace::<f64> {
                    field: std::marker::PhantomData::<f64>,
                },
                30000f64,
                15f64,
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
