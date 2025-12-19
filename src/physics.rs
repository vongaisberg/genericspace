use crate::barnes_hut::{build_tree, QuadTree};
use crate::types::Field;
use crate::types::MathSpace;
use num_traits::Pow;
use wasm_bindgen::prelude::*;

#[derive(Debug, Clone)]
pub struct PhysicsObject<K: Field> {
    pub position_vector: [K; 2],
    pub direction_vector: [K; 2],
    pub acceleration_vector: [K; 2],
    pub mass: K,
    status: ObjectStatus,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ObjectStatus {
    Default,
    Deleted,
}

//WASM logging
#[wasm_bindgen]
extern "C" {
    // Use `js_namespace` here to bind `console.log(..)` instead of just
    // `log(..)`
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
macro_rules! console_log {
    // Note that this is using the `log` function imported above during
    // `bare_bones`
    ($($t:tt)*) => (log(&format_args!($($t)*).to_string()))
}


impl<K: Field> PhysicsObject<K> {
    pub fn new(position_vector: [K; 2], direction_vector: [K; 2], mass: K) -> Self {
        PhysicsObject {
            position_vector,
            direction_vector,
            mass,
            acceleration_vector: [K::zero(), K::zero()],
            status: ObjectStatus::Default,
        }
    }
}

pub struct PhysicsSpace<K: Field + PartialOrd + Pow<f32, Output = K>, S: MathSpace<K>> {
    pub elements: Vec<PhysicsObject<K>>,
    gravitational_constant: K,
    math_space: S,
    radius: K,              //Elements that are further than K away from [0,0] get deleted
    softening_squared: K,   //Softening² parameter to prevent force singularities at close distances
    theta: f32,             //Barnes-Hut opening angle (0.5-1.0 typical, lower = more accurate)
}

impl<K: Field + PartialOrd + Pow<f32, Output = K>, S: MathSpace<K>> PhysicsSpace<K, S> {
    pub fn new(
        elements: Vec<PhysicsObject<K>>,
        gravitational_constant: K,
        math_space: S,
        radius: K,
        softening: K,
        theta: f32,
    ) -> Self {
        Self {
            elements,
            gravitational_constant,
            math_space,
            radius,
            softening_squared: softening * softening,
            theta,
        }
    }

    #[allow(dead_code)]
    fn acceleration_direct(&self, e1_pos: &[K; 2], skip_index: usize) -> [K; 2] {
        let g = self.gravitational_constant;
        let soft_sq = self.softening_squared;
        
        let mut acc = [K::zero(), K::zero()];
        
        for (i, e2) in self.elements.iter().enumerate() {
            // Skip self
            if i == skip_index {
                continue;
            }
            
            // Vector from e1 to e2
            let dx = e2.position_vector[0] - e1_pos[0];
            let dy = e2.position_vector[1] - e1_pos[1];
            
            // Softened distance: r_soft = sqrt(r² + ε²)
            let dist_sq_soft = dx * dx + dy * dy + soft_sq;
            let dist_soft = dist_sq_soft.pow(0.5f32);
            
            // Plummer softening: a = G * m * (dx, dy) / r_soft³
            let factor = e2.mass * g * dist_sq_soft.inv() * dist_soft.inv();
            acc[0] = acc[0] + dx * factor;
            acc[1] = acc[1] + dy * factor;
        }
        
        acc
    }

    #[allow(dead_code)]
    pub fn print(&self) {
        self.elements.iter().for_each(|e| {
            println!(
                "pos: {:?}, dir: {:?}, mass: {:?}",
                e.position_vector, e.direction_vector, e.mass
            );
        })
    }
}

/// Specialized implementation for f32 with Barnes-Hut
impl<S: MathSpace<f32>> PhysicsSpace<f32, S> {
    pub fn tick(&mut self) {
        let m = &self.math_space;
        let radius = self.radius;

        // Remove elements that are too far away
        self.elements.retain(|e| {
            m.distance(&[0.0, 0.0], &e.position_vector) <= radius
        });

        // Build Barnes-Hut tree once per frame
        let tree = build_tree(&self.elements);

        // Apply leapfrog integration using Barnes-Hut for acceleration
        let g = self.gravitational_constant;
        let soft_sq = self.softening_squared;
        let theta = self.theta;
        
        let updated: Vec<_> = self.elements
            .iter()
            .enumerate()
            .map(|(i, obj)| {
                self.leapfrog_with_tree(obj, i, &tree, g, soft_sq, theta)
            })
            .collect();
        self.elements = updated;
    }

    fn leapfrog_with_tree(
        &self,
        obj: &PhysicsObject<f32>,
        index: usize,
        tree: &QuadTree,
        g: f32,
        soft_sq: f32,
        theta: f32,
    ) -> PhysicsObject<f32> {
        let half = 0.5f32;

        // x(i+1) = x(i) + v(i) + 0.5 * a(i)
        let next_pos = [
            obj.position_vector[0] + obj.direction_vector[0] + half * obj.acceleration_vector[0],
            obj.position_vector[1] + obj.direction_vector[1] + half * obj.acceleration_vector[1],
        ];
        
        // a(i+1) using Barnes-Hut
        let next_acc = tree.calculate_force(next_pos, theta, g, soft_sq, index);

        // v(i+1) = v(i) + 0.5 * (a(i+1) + a(i))
        let next_dir = [
            obj.direction_vector[0] + half * (next_acc[0] + obj.acceleration_vector[0]),
            obj.direction_vector[1] + half * (next_acc[1] + obj.acceleration_vector[1]),
        ];

        PhysicsObject {
            position_vector: next_pos,
            direction_vector: next_dir,
            acceleration_vector: next_acc,
            mass: obj.mass,
            status: obj.status,
        }
    }
}
