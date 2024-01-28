#![cfg(feature = "serde")]

use std::marker::PhantomData;

use serde::{de::Visitor, ser::SerializeSeq, Deserializer, Serializer};

pub fn rom_bank_serialize<S>(x: &Vec<[u8; 0x4000]>, s: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    let mut seq = s.serialize_seq(Some(x.len() * 0x4000))?;
    for arr in x {
        for e in arr {
            seq.serialize_element(e)?;
        }
    }

    seq.end()
}

pub fn ram_bank_serialize<S>(x: &Vec<[u8; 0x2000]>, s: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    let mut seq = s.serialize_seq(Some(x.len() * 0x2000))?;
    for arr in x {
        for e in arr {
            seq.serialize_element(e)?;
        }
    }

    seq.end()
}

struct MyVisitor<const N: usize> {
    marker: PhantomData<fn() -> MyVisitor<N>>,
}

impl<const N: usize> MyVisitor<N> {
    fn new() -> Self {
        Self {
            marker: PhantomData,
        }
    }
}

impl<'de, const N: usize> Visitor<'de> for MyVisitor<N> {
    type Value = Vec<[u8; N]>;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str(format!("A vec of [u8; {:#06X}]", N).as_str())
    }

    fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
    where
        A: serde::de::SeqAccess<'de>,
    {
        let total_len = seq.size_hint().unwrap_or(0);
        let mut vec: Vec<u8> = Vec::with_capacity(total_len);

        while let Ok(Some(e)) = seq.next_element::<u8>() {
            vec.push(e);
        }

        if total_len == 0 {
            return Ok(Vec::new());
        }

        Ok(vec
            .chunks_exact(N)
            .map(|chunk| chunk.try_into().unwrap())
            .collect())
    }
}

pub fn rom_bank_deserialize<'de, D>(deserializer: D) -> Result<Vec<[u8; 0x4000]>, D::Error>
where
    D: Deserializer<'de>,
{
    deserializer.deserialize_seq(MyVisitor::<0x4000>::new())
}

pub fn ram_bank_deserialize<'de, D>(deserializer: D) -> Result<Vec<[u8; 0x2000]>, D::Error>
where
    D: Deserializer<'de>,
{
    deserializer.deserialize_seq(MyVisitor::<0x2000>::new())
}

#[cfg(test)]
mod tests {
    #[test]
    fn rom_bank_ser() {
        // let mut rom = vec![[0u8; 0x4000], [0; 0x4000], [0; 0x4000]];
        // rom[0][0x2001] = 1;
        // rom[1][0x1004] = 1;
        // rom[2][0x3955] = 0xFF;

        // let ser = serde_json::to_string(&rom).unwrap();
        // let de = serde_json::from_str(&ser).unwrap();
    }
}
