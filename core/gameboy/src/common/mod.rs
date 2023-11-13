use std::{
    ops::{Deref, DerefMut, Index, IndexMut},
    slice::SliceIndex,
};

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// A type to wrap 2d arrays so we can serialize and deserialize them more easily
/// by converting it into/from a 1d vector.
#[derive(Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(
    feature = "serde",
    serde(from = "SerializableD2Array<T>", into = "SerializableD2Array<T>")
)]
pub(crate) struct D2Array<T, const N: usize, const M: usize>
where
    T: Default + Clone + Copy,
{
    array: Box<[[T; N]; M]>,
}

#[cfg(feature = "serde")]
#[derive(Serialize, Deserialize)]
struct SerializableD2Array<T> {
    // Ideally we could use const generic expressions and define an array [T; N * M]
    // and then just use mem::transmute to convert to/from the D2Array type
    vec: Vec<T>,
}

#[cfg(feature = "serde")]
impl<T, const N: usize, const M: usize> From<SerializableD2Array<T>> for D2Array<T, N, M>
where
    T: Default + Clone + Copy,
{
    fn from(val: SerializableD2Array<T>) -> Self {
        let mut array = [[T::default(); N]; M];

        let mut vec_iter = val.vec.chunks_exact(N);
        for inner_arr in &mut array {
            let chunk = vec_iter.next().expect("Unable to get chunk");
            inner_arr.copy_from_slice(chunk);
        }

        D2Array {
            array: Box::new(array),
        }
    }
}

#[cfg(feature = "serde")]
impl<T, const N: usize, const M: usize> From<D2Array<T, N, M>> for SerializableD2Array<T>
where
    T: Default + Clone + Copy,
{
    fn from(val: D2Array<T, N, M>) -> Self {
        let mut vec = Vec::new();
        for m in 0..M {
            for n in 0..N {
                vec.push(val.array[m][n]);
            }
        }

        SerializableD2Array { vec }
    }
}

impl<T, const N: usize, const M: usize> From<[[T; N]; M]> for D2Array<T, N, M>
where
    T: Default + Clone + Copy,
{
    fn from(array: [[T; N]; M]) -> Self {
        Self {
            array: Box::new(array),
        }
    }
}

impl<T, const N: usize, const M: usize> AsRef<[[T; N]; M]> for D2Array<T, N, M>
where
    T: Default + Clone + Copy,
{
    fn as_ref(&self) -> &[[T; N]; M] {
        &self.array
    }
}

impl<T, const N: usize, const M: usize> Deref for D2Array<T, N, M>
where
    T: Default + Clone + Copy,
{
    type Target = [[T; N]; M];

    fn deref(&self) -> &Self::Target {
        &self.array
    }
}

impl<T, const N: usize, const M: usize> DerefMut for D2Array<T, N, M>
where
    T: Default + Clone + Copy,
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.array
    }
}

impl<T, const N: usize, const M: usize, Idx> Index<Idx> for D2Array<T, N, M>
where
    T: Default + Clone + Copy,
    Idx: SliceIndex<[[T; N]], Output = [T; N]>,
{
    type Output = [T; N];

    fn index(&self, index: Idx) -> &Self::Output {
        &self.array[index]
    }
}

impl<T, const N: usize, const M: usize, Idx> IndexMut<Idx> for D2Array<T, N, M>
where
    T: Default + Clone + Copy,
    Idx: SliceIndex<[[T; N]], Output = [T; N]>,
{
    fn index_mut(&mut self, index: Idx) -> &mut Self::Output {
        &mut self.array[index]
    }
}

#[cfg(test)]
mod tests {
    use super::D2Array;

    #[test]
    fn d2array_is_indexable() {
        let arr: D2Array<u8, 0x100, 2> = [[0; 0x100]; 2].into();
        let _ = arr[0];
    }

    #[test]
    fn d2array_is_mutable_via_index() {
        let mut arr: D2Array<u8, 0x100, 2> = [[0; 0x100]; 2].into();
        arr[0][1] = 10;
    }

    #[test]
    fn d2array_should_serde_correctly() {
        let mut arr: D2Array<usize, 0x100, 2> = [[0; 0x100]; 2].into();

        for (i, val) in arr[0].iter_mut().enumerate() {
            *val = i;
        }
        for i in (0..0x100).rev() {
            arr[1][i] = i.abs_diff(0x100);
        }

        let serialized_arr = serde_json::to_string(&arr).expect("Unable to serialize arr");

        let deserialized_arr: D2Array<usize, 0x100, 2> =
            serde_json::from_str(&serialized_arr).expect("Unable to deserialize arr");

        // verify values
        deserialized_arr.array[0]
            .iter()
            .enumerate()
            .for_each(|(i, val): (usize, &usize)| assert_eq!(i, *val));

        deserialized_arr.array[1]
            .iter()
            .enumerate()
            .rev()
            .for_each(|(i, val): (usize, &usize)| assert_eq!(i, val.abs_diff(0x100)));
    }
}
