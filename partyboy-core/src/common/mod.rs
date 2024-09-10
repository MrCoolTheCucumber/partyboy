use std::{
    ops::{Deref, DerefMut, Index, IndexMut},
    slice::SliceIndex,
};

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

mod boxed_slice;

pub use boxed_slice::BoxedSlice;

/// A type to wrap 2d arrays so we can serialize and deserialize them more easily
/// by converting it into/from a 1d vector.
#[derive(Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(
    feature = "serde",
    serde(from = "SerializableD2Array", into = "SerializableD2Array")
)]
pub(crate) struct D2Array<const N: usize, const M: usize> {
    array: Box<[[u8; N]; M]>,
}

impl<const N: usize, const M: usize> D2Array<N, M> {
    pub fn new_zeroed() -> Self {
        Self {
            array: boxarray::boxarray(0),
        }
    }
}

#[cfg(feature = "serde")]
#[derive(Serialize, Deserialize)]
struct SerializableD2Array {
    // Ideally we could use const generic expressions and define an array [T; N * M]
    // and then just use mem::transmute to convert to/from the D2Array type
    vec: Vec<u8>,
}

#[cfg(feature = "serde")]
impl<const N: usize, const M: usize> From<SerializableD2Array> for D2Array<N, M> {
    fn from(val: SerializableD2Array) -> Self {
        let mut d2 = Self::new_zeroed();

        let mut vec_iter = val.vec.chunks_exact(N);
        for inner_arr in &mut *d2.array {
            let chunk = vec_iter.next().expect("Unable to get chunk");
            inner_arr.copy_from_slice(chunk);
        }

        d2
    }
}

#[cfg(feature = "serde")]
impl<const N: usize, const M: usize> From<D2Array<N, M>> for SerializableD2Array {
    fn from(val: D2Array<N, M>) -> Self {
        let mut vec = Vec::new();
        for m in 0..M {
            for n in 0..N {
                vec.push(val.array[m][n]);
            }
        }

        SerializableD2Array { vec }
    }
}

impl<const N: usize, const M: usize> AsRef<[[u8; N]; M]> for D2Array<N, M> {
    fn as_ref(&self) -> &[[u8; N]; M] {
        &self.array
    }
}

impl<const N: usize, const M: usize> Deref for D2Array<N, M> {
    type Target = [[u8; N]; M];

    fn deref(&self) -> &Self::Target {
        &self.array
    }
}

impl<const N: usize, const M: usize> DerefMut for D2Array<N, M> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.array
    }
}

impl<const N: usize, const M: usize, Idx> Index<Idx> for D2Array<N, M>
where
    Idx: SliceIndex<[[u8; N]], Output = [u8; N]>,
{
    type Output = [u8; N];

    fn index(&self, index: Idx) -> &Self::Output {
        &self.array[index]
    }
}

impl<const N: usize, const M: usize, Idx> IndexMut<Idx> for D2Array<N, M>
where
    Idx: SliceIndex<[[u8; N]], Output = [u8; N]>,
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
        let arr: D2Array<0x100, 2> = D2Array::new_zeroed();
        let _ = arr[0];
    }

    #[test]
    fn d2array_is_mutable_via_index() {
        let mut arr: D2Array<0x100, 2> = D2Array::new_zeroed();
        arr[0][1] = 10;
    }

    #[test]
    fn d2array_should_serde_correctly() {
        let mut arr: D2Array<250, 2> = D2Array::new_zeroed();

        for (i, val) in arr[0].iter_mut().enumerate() {
            *val = i as u8;
        }
        for i in (0u8..250).rev() {
            arr[1][i as usize] = i.abs_diff(250);
        }

        let serialized_arr = serde_json::to_string(&arr).expect("Unable to serialize arr");

        let deserialized_arr: D2Array<250, 2> =
            serde_json::from_str(&serialized_arr).expect("Unable to deserialize arr");

        // verify values
        deserialized_arr.array[0]
            .iter()
            .enumerate()
            .for_each(|(i, val): (usize, &u8)| assert_eq!(i, *val as usize));

        deserialized_arr.array[1]
            .iter()
            .enumerate()
            .rev()
            .for_each(|(i, val): (usize, &u8)| assert_eq!(i, val.abs_diff(250) as usize));
    }
}
