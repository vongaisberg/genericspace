use num_traits::{Inv, One, Pow, Zero};
use std::ops::{Add, Mul, Sub};
//use std::num::{Zero, One};

pub trait Field:
    Add<Output = Self>
    + Sub<Output = Self>
    + Mul<Output = Self>
    + One
    + Zero
    + Inv<Output = Self>
    + std::fmt::Debug
    + Clone
    + Copy
{
}

pub trait MathSpace<K: Field> {
    fn distance(&self, first: &[K; 2], second: &[K; 2]) -> K;

    //fn scalar_product(first: [K;2], first: [K;2]) -> K;

    //fn dimension() -> u8;

    fn add(&self, first: &[K; 2], second: &[K; 2]) -> [K; 2];

    fn sub(&self, first: &[K; 2], second: &[K; 2]) -> [K; 2];

    fn mul(&self, scalar: &K, vector: &[K; 2]) -> [K; 2];
}

pub struct EuclideanSpace<K: Field + Pow<f32, Output = K>> {
    pub field: std::marker::PhantomData<K>,
}

impl<K: Field + Pow<f32, Output = K>> MathSpace<K> for EuclideanSpace<K> {
    fn distance(&self, first: &[K; 2], second: &[K; 2]) -> K {
        let diff = self.sub(second, first);
        self.scalar_product(diff, diff).pow(0.5f32)
    }

    fn add(&self, first: &[K; 2], second: &[K; 2]) -> [K; 2] {
        [first[0] + second[0], first[1] + second[1]]
    }

    fn sub(&self, first: &[K; 2], second: &[K; 2]) -> [K; 2] {
        [first[0] - second[0], first[1] - second[1]]
    }

    fn mul(&self, scalar: &K, vector: &[K; 2]) -> [K; 2] {
        [*scalar * vector[0], *scalar * vector[1]]
    }
}

impl<K: Field + Pow<f32, Output = K>> EuclideanSpace<K> {
    fn scalar_product(&self, first: [K; 2], second: [K; 2]) -> K {
        first[0] * second[0] + first[1] * second[1]
    }
}
