use std::{
    fmt::Debug,
    ops::{Deref, DerefMut, Index, IndexMut},
    slice::SliceIndex,
};

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

#[derive(Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "serde", serde(from = "Vec<T>", into = "Vec<T>"))]
pub struct BoxedSlice<T, const N: usize>
where
    T: Clone + Copy + Default + Debug,
{
    inner: Box<[T; N]>,
}

impl<T, const N: usize> BoxedSlice<T, N>
where
    T: Clone + Copy + Default + Debug,
{
    pub fn new_with(initial: T) -> Self {
        Self {
            inner: vec![initial; N].into_boxed_slice().try_into().unwrap(),
        }
    }

    pub fn new_with_slice(val: [T; N]) -> Self {
        Self {
            inner: Box::new(val),
        }
    }
}

impl<T, const N: usize> Default for BoxedSlice<T, N>
where
    T: Clone + Copy + Default + Debug,
{
    fn default() -> Self {
        Self::new_with(T::default())
    }
}

#[cfg(feature = "serde")]
impl<T, const N: usize> From<Vec<T>> for BoxedSlice<T, N>
where
    T: Clone + Copy + Default + Debug,
{
    fn from(value: Vec<T>) -> Self {
        Self {
            inner: value.into_boxed_slice().try_into().unwrap(),
        }
    }
}

#[cfg(feature = "serde")]
impl<T, const N: usize> From<BoxedSlice<T, N>> for Vec<T>
where
    T: Clone + Copy + Default + Debug,
{
    fn from(value: BoxedSlice<T, N>) -> Self {
        value.inner.to_vec()
    }
}

impl<T, const N: usize> AsRef<[T; N]> for BoxedSlice<T, N>
where
    T: Clone + Copy + Default + Debug,
{
    fn as_ref(&self) -> &[T; N] {
        &self.inner
    }
}

impl<T, const N: usize> Deref for BoxedSlice<T, N>
where
    T: Clone + Copy + Default + Debug,
{
    type Target = [T; N];

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl<T, const N: usize> DerefMut for BoxedSlice<T, N>
where
    T: Clone + Copy + Default + Debug,
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}

impl<T, const N: usize, Idx> Index<Idx> for BoxedSlice<T, N>
where
    T: Clone + Copy + Default + Debug,
    Idx: SliceIndex<[T], Output = T>,
{
    type Output = T;

    fn index(&self, index: Idx) -> &Self::Output {
        &self.inner[index]
    }
}

impl<T, const N: usize, Idx> IndexMut<Idx> for BoxedSlice<T, N>
where
    T: Clone + Copy + Default + Debug,
    Idx: SliceIndex<[T], Output = T>,
{
    fn index_mut(&mut self, index: Idx) -> &mut Self::Output {
        &mut self.inner[index]
    }
}

#[cfg(test)]
mod tests {
    use super::BoxedSlice;

    #[test]
    fn test_serde() {
        let mut boxed_slice = BoxedSlice::<u8, 0xFF>::default();
        boxed_slice
            .iter_mut()
            .enumerate()
            .for_each(|(i, el)| *el = i as u8);

        let serialized_arr = serde_json::to_string(&boxed_slice).expect("Unable to serialize arr");

        let deserialized_arr: BoxedSlice<u8, 0xFF> =
            serde_json::from_str(&serialized_arr).expect("Unable to deserialize arr");

        deserialized_arr
            .iter()
            .enumerate()
            .for_each(|(i, el)| assert_eq!(i, *el as usize));
    }
}
