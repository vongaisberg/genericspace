extern crate wasm_bindgen;

mod barnes_hut;
mod physics;
mod types;
mod utils;

use physics::{PhysicsObject, PhysicsSpace};
use types::EuclideanSpace;
use types::Field;
use wasm_bindgen::prelude::*;
use js_sys::Float32Array;

use rand::os::OsRng;
use rand::Rng;
// When the `wee_alloc` feature is enabled, use `wee_alloc` as the global
// allocator.
#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

impl Field for f32 {}

#[wasm_bindgen]
extern "C" {
    fn alert(s: &str);
}

#[wasm_bindgen]
pub fn greet() {
    alert("Hello, asdasdaswasm-generic-space!");
}

#[wasm_bindgen]
pub struct Universe {
    phys: PhysicsSpace<f32, EuclideanSpace<f32>>,
    // Reusable buffer for positions - avoids allocation every frame
    position_buffer: Vec<f32>,
}
#[wasm_bindgen]
impl Universe {
    pub fn new() -> Universe {
        //let mut rng = rand::thread_rng();

        let mut rng = OsRng::new().unwrap();;


        let mut elems = Vec::new();

    let speed_range = 10.0;

        //for i in 0..2000 {
        //    elems.push(PhysicsObject::<f64>::new(
        //        [rng.gen_range(250.0, 1200.0), rng.gen_range(250.0, 1200.0)],
        //        [rng.gen_range(-speed_range, speed_range), rng.gen_range(-speed_range, speed_range)],
        //        //rng.gen_range(1.0, 2.0),
        //        0.1
        //    ))
        //}


        use std::f32::consts::PI;

        // Uniformly rotating disk parameters
        let center = [800.0f32, 800.0f32];
        let min_radius = 50.0f32;
        let max_radius = 2000.0f32;
        let omega = 0.003f32; // angular velocity (rad/tick, adjust as needed)

        for _i in 0..10000 {
            // Uniform sampling in the disk
            let r: f32 = num_traits::Float::sqrt(rng.gen_range(0.0f32, 1.0f32) * (max_radius * max_radius - min_radius * min_radius) + min_radius * min_radius);
            let theta: f32 = rng.gen_range(0.0f32, 2.0f32 * PI);

            let x = center[0] + r * theta.cos();
            let y = center[1] + r * theta.sin();

            // Tangential velocity for uniform rotation: v = omega * r (perpendicular to position vector)
            let vx = -omega * r * theta.sin();
            let vy = omega * r * theta.cos();

            elems.push(PhysicsObject::<f32>::new(
                [x, y],
                [vx, vy],
                0.01
            ));
        }

        elems.push(PhysicsObject::<f32>::new(
            [800.0, 800.0],
            [0.0, 0.0],
            100.0,
        ));



        let num_particles = elems.len();
        Universe {
            phys: PhysicsSpace::new(
                elems,
                100f32,
                EuclideanSpace::<f32> {
                    field: std::marker::PhantomData::<f32>,
                },
                10000f32,
                10f32,
                0.7f32, // Barnes-Hut theta parameter (0.5-1.0, lower = more accurate)
            ),
            position_buffer: vec![0.0f32; num_particles * 2],
        }
    }

    /// Run one simulation tick
    pub fn tick(&mut self) {
        self.phys.tick();
    }

    /// Get the number of particles
    pub fn particle_count(&self) -> usize {
        self.phys.elements.len()
    }

    /// Get positions as a Float32Array view into WASM memory
    /// Format: [x0, y0, x1, y1, x2, y2, ...]
    /// This avoids creating objects and crossing the WASM boundary per-particle
    pub fn get_positions(&mut self) -> Float32Array {
        let count = self.phys.elements.len();
        
        // Resize buffer if needed (particles may have been removed)
        if self.position_buffer.len() != count * 2 {
            self.position_buffer.resize(count * 2, 0.0);
        }
        
        // Copy positions into flat buffer
        for (i, elem) in self.phys.elements.iter().enumerate() {
            self.position_buffer[i * 2] = elem.position_vector[0];
            self.position_buffer[i * 2 + 1] = elem.position_vector[1];
        }
        
        // Return a view into WASM memory - no copy!
        unsafe {
            Float32Array::view(&self.position_buffer)
        }
    }
}
