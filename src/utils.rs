use std::ops::Mul;

use itertools::Itertools;

pub fn vec_add<T>(v1: &[T], v2: &[T]) -> Vec<T>
where
    T: std::ops::Add<Output = T> + Copy,
{
    if v1.len() != v2.len() {
        panic!("Cannot multiply vectors of different lengths!")
    }

    v1.iter().zip(v2).map(|(&i1, &i2)| i1 + i2).collect()
}

pub fn scale<T>(vec: &[T], scalar: f32) -> Vec<T>
where
    T: Mul<f32, Output = T> + Copy,
{
    vec.into_iter().map(|x| *x * scalar).collect_vec()
}
