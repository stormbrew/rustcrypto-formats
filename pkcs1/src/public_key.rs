//! PKCS#1 RSA Public Keys.

use crate::{Error, Result};
use der::{asn1::UIntBytes, Decode, Decoder, Encode, Sequence};

#[cfg(feature = "alloc")]
use der::Document;

#[cfg(feature = "pem")]
use der::pem::PemLabel;

/// PKCS#1 RSA Public Keys as defined in [RFC 8017 Appendix 1.1].
///
/// ASN.1 structure containing a serialized RSA public key:
///
/// ```text
/// RSAPublicKey ::= SEQUENCE {
///     modulus           INTEGER,  -- n
///     publicExponent    INTEGER   -- e
/// }
/// ```
///
/// [RFC 8017 Appendix 1.1]: https://datatracker.ietf.org/doc/html/rfc8017#appendix-A.1.1
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct RsaPublicKey<'a> {
    /// `n`: RSA modulus
    pub modulus: UIntBytes<'a>,

    /// `e`: RSA public exponent
    pub public_exponent: UIntBytes<'a>,
}

impl<'a> Decode<'a> for RsaPublicKey<'a> {
    fn decode(decoder: &mut Decoder<'a>) -> der::Result<Self> {
        decoder.sequence(|decoder| {
            Ok(Self {
                modulus: decoder.decode()?,
                public_exponent: decoder.decode()?,
            })
        })
    }
}

impl<'a> Sequence<'a> for RsaPublicKey<'a> {
    fn fields<F, T>(&self, f: F) -> der::Result<T>
    where
        F: FnOnce(&[&dyn Encode]) -> der::Result<T>,
    {
        f(&[&self.modulus, &self.public_exponent])
    }
}

impl<'a> TryFrom<&'a [u8]> for RsaPublicKey<'a> {
    type Error = Error;

    fn try_from(bytes: &'a [u8]) -> Result<Self> {
        Ok(Self::from_der(bytes)?)
    }
}

#[cfg(feature = "alloc")]
#[cfg_attr(docsrs, doc(cfg(feature = "alloc")))]
impl TryFrom<RsaPublicKey<'_>> for Document {
    type Error = Error;

    fn try_from(spki: RsaPublicKey<'_>) -> Result<Document> {
        Self::try_from(&spki)
    }
}

#[cfg(feature = "alloc")]
#[cfg_attr(docsrs, doc(cfg(feature = "alloc")))]
impl TryFrom<&RsaPublicKey<'_>> for Document {
    type Error = Error;

    fn try_from(spki: &RsaPublicKey<'_>) -> Result<Document> {
        Ok(Self::encode_msg(spki)?)
    }
}

#[cfg(feature = "pem")]
#[cfg_attr(docsrs, doc(cfg(feature = "pem")))]
impl PemLabel for RsaPublicKey<'_> {
    const PEM_LABEL: &'static str = "RSA PUBLIC KEY";
}
