use crate::types::Field;
use crate::types::MathSpace;
use std::ops::{Add, Mul, Sub};
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
    MergedInto(usize),
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
    fn clone_change_position(&self, pos_vec: [K; 2]) -> PhysicsObject<K> {
        PhysicsObject {
            position_vector: pos_vec,
            direction_vector: self.direction_vector.clone(),
            mass: self.mass.clone(),
            acceleration_vector: self.acceleration_vector.clone(),
            status: self.status,
        }
    }

    fn clone_change_direction(&self, dir_vec: [K; 2]) -> PhysicsObject<K> {
        PhysicsObject {
            direction_vector: dir_vec,
            position_vector: self.position_vector.clone(),
            mass: self.mass.clone(),
            acceleration_vector: self.acceleration_vector.clone(),
            status: self.status,
        }
    }

    fn clone_change_position_direction(
        &self,
        pos_vec: [K; 2],
        dir_vec: [K; 2],
    ) -> PhysicsObject<K> {
        PhysicsObject {
            direction_vector: dir_vec,
            position_vector: pos_vec,
            mass: self.mass.clone(),
            acceleration_vector: self.acceleration_vector.clone(),
            status: self.status,
        }
    }
    fn clone_change_status(&self, status: ObjectStatus) -> PhysicsObject<K> {
        PhysicsObject {
            direction_vector: self.direction_vector.clone(),
            position_vector: self.position_vector.clone(),
            mass: self.mass.clone(),
            acceleration_vector: self.acceleration_vector.clone(),
            status: status,
        }
    }

    pub fn new(position_vector: [K; 2], direction_vector: [K; 2], mass: K) -> Self {
        PhysicsObject {
            position_vector: position_vector,
            direction_vector: direction_vector,
            mass: mass,
            acceleration_vector: [K::zero(), K::zero()],
            status: ObjectStatus::Default,
        }
    }
}

pub struct PhysicsSpace<K: Field + PartialOrd, S: MathSpace<K>> {
    pub elements: Vec<PhysicsObject<K>>,
    gravitational_constant: K,
    math_space: S,
    radius: K,  //Elements that are further than K away from [0,0] get deleted
    epsilon: K, //Small number to fix some numerical errors
    merge_counter: f64,
}

impl<K: Field + PartialOrd, S: MathSpace<K>> PhysicsSpace<K, S> {
    pub fn new(
        elements: Vec<PhysicsObject<K>>,
        gravitational_constant: K,
        math_space: S,
        radius: K,
        epsilon: K,
    ) -> Self {
        Self {
            elements: elements,
            gravitational_constant: gravitational_constant,
            math_space: math_space,
            radius: radius,
            epsilon: epsilon,
            merge_counter: 0f64,
        }
    }

    fn leapfrog_integration(&self, obj: &PhysicsObject<K>) -> PhysicsObject<K> {
       // console_log!("Particle {:?}", obj);
        
        let m = &self.math_space;
       // console_log!("Distance from 00: {:?}", m.distance(&[K::zero(), K::zero()], &obj.position_vector));
        let zeropointfive = (K::one() + K::one()).inv();

        //x(i+1) = x(i) +v(i) + 0.5 a(i)
        let next_pos = m.add(
            &m.add(&obj.position_vector, &obj.direction_vector),
            &m.mul(&zeropointfive, &obj.acceleration_vector),
        );
        //a(i+1)
        let next_acc = self.acceleration(
            &obj.clone_change_position(next_pos.clone()),
            &obj.position_vector,
        );

        //v(i+1) = v(i) + 0.5( a(i+1) + a(i) )
        let next_dir = m.add(
            &obj.direction_vector,
            &m.mul(&zeropointfive, &m.add(&next_acc, &obj.acceleration_vector)),
        );

        PhysicsObject {
            position_vector: next_pos,
            direction_vector: next_dir,
            acceleration_vector: next_acc,
            mass: obj.mass.clone(),
            status: obj.status,
        }
    }

    fn euler_integration(&self, obj: &PhysicsObject<K>) -> PhysicsObject<K> {
        let m = &self.math_space;
        let next_obj =
            obj.clone_change_position(m.add(&obj.position_vector, &obj.direction_vector));
        println!(
            "Acceleration {:?}",
            &self.acceleration(&next_obj, &obj.position_vector)
        );
        next_obj.clone_change_direction(m.add(
            &next_obj.direction_vector,
            &self.acceleration(&next_obj, &obj.position_vector),
        ))
    }

    fn acceleration(&self, e1: &PhysicsObject<K>, old_pos: &[K; 2]) -> [K; 2] {
        let m = &self.math_space;
        self.elements
            .iter()
            .map(|e2| {
                //Calculate the gravity effect on e1 while being attracted by e2
                let distance = m.distance(&e2.position_vector, &e1.position_vector);
                let old_distance = m.distance(&e2.position_vector, &old_pos);
                //           println!("Distance {:?}",distance);
                if !(distance.is_zero() || old_distance.is_zero()) {
                    let distance_vector = m.sub(&e2.position_vector, &e1.position_vector);
                    //             println!("Distance vector {:?}", distance_vector);

                    let distance_unit_vector = m.mul(&distance.clone().inv(), &distance_vector);
                    //           println!("Distance unit vector {:?}", distance_unit_vector);
                    let acceleration = e2.mass.clone()
                        * self.gravitational_constant.clone()
                        * ((distance.clone() * distance.clone()).inv());
                    //         println!("Acceleration {:?}", acceleration);
                    m.mul(&acceleration, &distance_unit_vector)
                } else {
                    [K::zero(), K::zero()]
                }
            })
            .fold([K::zero(), K::zero()], |a, acc| m.add(&a, &acc))
    }

    fn merge(&self, f: &PhysicsObject<K>, s: &PhysicsObject<K>) -> PhysicsObject<K> {
    //    console_log!("#########################Merging {:?} with {:?}", f, s);

        let m = &self.math_space;
       let p = PhysicsObject {
            position_vector: m.mul(
                &(f.mass.clone() + s.mass.clone()).inv(),
                &m.add(
                    &m.mul(&f.mass, &f.position_vector),
                    &m.mul(&s.mass, &s.position_vector),
                ),
            ), // Weighted average of position vectors
            direction_vector: m.mul(
                &(f.mass.clone() + s.mass.clone()).inv(),
                &m.add(
                    &m.mul(&f.mass, &f.direction_vector),
                    &m.mul(&s.mass, &s.direction_vector),
                ),
            ), //Weighted average of direction vectors
            acceleration_vector: m.mul(
                &(f.mass.clone() + s.mass.clone()).inv(),
                &m.add(
                    &m.mul(&f.mass, &f.acceleration_vector),
                    &m.mul(&s.mass, &s.acceleration_vector),
                ),
            ),
        //    acceleration_vector: [K::zero(), K::zero()],
            status: ObjectStatus::Default,
            mass: f.mass.clone() + s.mass.clone(), //Sum of masses
        };
      //  console_log!("Resulting {:?}", p);
        p
    }

    pub fn print(&self) {
        self.elements.iter().for_each(|e| {
            println!(
                "pos: {:?}, dir: {:?}, mass: {:?}",
                e.position_vector, e.direction_vector, e.mass
            );
        })
    }
}

impl<K: Field + PartialOrd, S: MathSpace<K>> PhysicsSpace<K, S> {
    pub fn tick(&mut self) {
       // console_log!("Tick ");
        let m = &self.math_space;
        let mut elements = self.elements.clone();

        for i in 0..elements.len() {
            //Remove elements that are too far away
            match elements[i].status {
                ObjectStatus::Default => {
                    //Only remove elements that have not been removed or merged
                    if m.distance(&[K::zero(), K::zero()], &elements[i].position_vector)
                        > self.radius
                    {
                        //  println!("Deleting {:?}", elements[i]);
                        elements[i].status = ObjectStatus::Deleted

                    } else {
                        // If status is still default, check merges
                        checkMerge(self, &mut elements, i);
                    }
                }
                // If particle A was merged into B, check if other particles would have merged into A. If yes, also merge them into B
                ObjectStatus::MergedInto(k) => checkMerge(self, &mut elements, i),
                _ => {}
            }
            // {}
        }

        // elements = elements
        //     .iter()
        //     .map(|e| match e.status {
        //         ObjectStatus::MergedInto(f) => e.clone_change_status(ObjectStatus::Default),
        //         _ => e.clone(),
        //     })
        //     .collect();
        elements.retain(|e| e.status == ObjectStatus::Default);

        fn checkMerge<L: Field + PartialOrd, M: MathSpace<L>>(
            phys: &PhysicsSpace<L, M>,
            elements: &mut Vec<PhysicsObject<L>>,
            i: usize,
        ) {
            let m = &phys.math_space;
            for j in i + 1..phys.elements.len() {
                // Merge elements that are too close together
                // Always merge j into i. Update the values of i and mark j as Merged(into)

                if m.distance(&elements[i].position_vector, &elements[j].position_vector)
                    < phys.epsilon
                {
                    match elements[i].status {
                        ObjectStatus::Default => {
                            //If i was not merger into anything, merge j into i
                            elements[i] = phys.merge(&elements[i], &elements[j]);
                            elements[j].status = ObjectStatus::MergedInto(i);
                        }
                        ObjectStatus::MergedInto(k) => {
                            //If i was merged into k, merge j into k

                            elements[k] = phys.merge(&elements[k], &elements[j]);
                            elements[j].status = ObjectStatus::MergedInto(k);
                        }
                        _ => {}
                    }
                }
            }
        }

self.elements = elements;
        self.elements = self.elements
            .iter()
            .map(|e1| self.leapfrog_integration(e1))
            .collect();
    }
}
