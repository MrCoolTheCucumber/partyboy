#[cfg(feature = "web")]
use wasm_bindgen::prelude::wasm_bindgen;

#[cfg_attr(feature = "web", wasm_bindgen)]
pub struct BitPackedState {
    bytes: usize,
    data: Vec<u64>,
}

#[cfg_attr(feature = "web", wasm_bindgen)]
impl BitPackedState {
    const CHUNK_SIZE: usize = 8;

    pub fn pack(state: Vec<u8>) -> Self {
        let bytes = state.len();

        let chunks = state.chunks_exact(Self::CHUNK_SIZE);
        let remainder = chunks.remainder();

        let mut data = chunks
            .map(|chunk| {
                let as_array_ref: &[u8; Self::CHUNK_SIZE] = chunk.try_into().unwrap();
                u64::from_ne_bytes(*as_array_ref)
            })
            .collect::<Vec<u64>>();

        if !remainder.is_empty() {
            let mut pack: u64 = 0;
            remainder
                .iter()
                .enumerate()
                .for_each(|(i, byte)| pack |= (*byte as u64) << (i * 8));
            data.push(pack);
        }

        Self { bytes, data }
    }

    pub fn unpack(&self) -> Vec<u8> {
        let remainder = self.bytes % Self::CHUNK_SIZE;
        let remainding = remainder != 0;
        let take = self.data.len() - usize::from(remainding);

        let mut unpacked = self
            .data
            .iter()
            .take(take)
            .flat_map(|packed| packed.to_ne_bytes())
            .collect::<Vec<u8>>();

        if remainding {
            let mut remainding = Vec::new();
            let packed = self.data.last().unwrap();
            for i in 0..remainder {
                remainding.push((packed >> (i * 8)) as u8);
            }

            unpacked.append(&mut remainding);
        }

        unpacked
    }
}

#[cfg(test)]
mod test {
    use super::BitPackedState;

    #[test]
    fn pack_and_unpack_with_remainder() {
        let arr = vec![0u8, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 0, 0, 0, 12, 13];
        let packed = BitPackedState::pack(arr.clone());
        let unpacked = packed.unpack();
        assert_eq!(arr, unpacked);
    }

    #[test]
    fn pack_and_unpack_no_remainder() {
        let arr = vec![0u8, 1, 2, 3, 4, 5, 6, 7];
        let packed = BitPackedState::pack(arr.clone());
        let unpacked = packed.unpack();
        assert_eq!(arr, unpacked);
    }
}
