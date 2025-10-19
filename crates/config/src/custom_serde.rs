use data_encoding::HEXLOWER;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

pub struct HexArray<const N: usize>(pub [u8; N]);

impl<const N: usize> Serialize for HexArray<N> {
    fn serialize<S>(&self, ser: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let str = HEXLOWER.encode(&self.0);
        ser.serialize_str(&str)
    }
}

impl<'de, const N: usize> Deserialize<'de> for HexArray<N> {
    fn deserialize<D>(de: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let str = String::deserialize(de)?;

        let vec = HEXLOWER
            .decode(str.as_bytes())
            .map_err(serde::de::Error::custom)?;

        let arr = vec.try_into().map_err(|_| {
            serde::de::Error::custom(format!(
                "invalid len, expected {} bytes ({} characters)",
                N,
                N * 2
            ))
        })?;

        Ok(Self(arr))
    }
}
