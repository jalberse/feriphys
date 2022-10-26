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

/// The set difference between trio and duo (i.e. the element from the trio
/// missing from the duo).
/// Assumes that duo is a strict subset of trio, else panic!
pub fn tuple_difference<T>(trio: (T, T, T), duo: (T, T)) -> T
where
    T: std::cmp::PartialEq,
{
    if trio.0 != duo.0 && trio.0 != duo.1 {
        assert!((duo.0 == trio.1 && duo.1 == trio.2) || (duo.0 == trio.2 && duo.1 == trio.1));
        trio.0
    } else if trio.1 != duo.0 && trio.1 != duo.1 {
        assert!((duo.0 == trio.0 && duo.1 == trio.2) || (duo.0 == trio.2 && duo.1 == trio.0));
        trio.1
    } else {
        assert!((duo.0 == trio.0 && duo.1 == trio.1) || (duo.0 == trio.1 && duo.1 == trio.0));
        trio.2
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn tuple_difference() {
        assert_eq!(7, super::tuple_difference((7, 8, 9), (8, 9)));
    }

    #[test]
    #[should_panic]
    fn tuple_difference_panic() {
        super::tuple_difference((0, 1, 2), (0, 3));
    }
}
