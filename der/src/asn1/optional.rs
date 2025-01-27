//! ASN.1 `OPTIONAL` as mapped to Rust's `Option` type

use crate::{Choice, Decode, Decoder, DerOrd, Encode, Length, Reader, Result, Tag, Writer};
use core::cmp::Ordering;

impl<'a, T> Decode<'a> for Option<T>
where
    T: Choice<'a>, // NOTE: all `Decode + Tagged` types receive a blanket `Choice` impl
{
    fn decode(decoder: &mut Decoder<'a>) -> Result<Option<T>> {
        if let Some(byte) = decoder.peek_byte() {
            if T::can_decode(Tag::try_from(byte)?) {
                return T::decode(decoder).map(Some);
            }
        }

        Ok(None)
    }
}

impl<T> DerOrd for Option<T>
where
    T: DerOrd,
{
    fn der_cmp(&self, other: &Self) -> Result<Ordering> {
        match self {
            Some(a) => match other {
                Some(b) => a.der_cmp(b),
                None => Ok(Ordering::Greater),
            },
            None => Ok(Ordering::Less),
        }
    }
}

impl<T> Encode for Option<T>
where
    T: Encode,
{
    fn encoded_len(&self) -> Result<Length> {
        (&self).encoded_len()
    }

    fn encode(&self, writer: &mut dyn Writer) -> Result<()> {
        (&self).encode(writer)
    }
}

impl<T> Encode for &Option<T>
where
    T: Encode,
{
    fn encoded_len(&self) -> Result<Length> {
        match self {
            Some(encodable) => encodable.encoded_len(),
            None => Ok(0u8.into()),
        }
    }

    fn encode(&self, encoder: &mut dyn Writer) -> Result<()> {
        match self {
            Some(encodable) => encodable.encode(encoder),
            None => Ok(()),
        }
    }
}
