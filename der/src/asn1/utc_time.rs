//! ASN.1 `UTCTime` support.

use crate::{
    asn1::Any,
    datetime::{self, DateTime},
    ord::OrdIsValueOrd,
    ByteSlice, DecodeValue, Decoder, EncodeValue, Error, ErrorKind, FixedTag, Header, Length,
    Result, Tag, Writer,
};
use core::time::Duration;

#[cfg(feature = "std")]
use std::time::SystemTime;

/// Maximum year that can be represented as a `UTCTime`.
pub const MAX_YEAR: u16 = 2049;

/// ASN.1 `UTCTime` type.
///
/// This type implements the validity requirements specified in
/// [RFC 5280 Section 4.1.2.5.1][1], namely:
///
/// > For the purposes of this profile, UTCTime values MUST be expressed in
/// > Greenwich Mean Time (Zulu) and MUST include seconds (i.e., times are
/// > `YYMMDDHHMMSSZ`), even where the number of seconds is zero.  Conforming
/// > systems MUST interpret the year field (`YY`) as follows:
/// >
/// > - Where `YY` is greater than or equal to 50, the year SHALL be
/// >   interpreted as `19YY`; and
/// > - Where `YY` is less than 50, the year SHALL be interpreted as `20YY`.
///
/// [1]: https://tools.ietf.org/html/rfc5280#section-4.1.2.5.1
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
pub struct UtcTime(DateTime);

impl UtcTime {
    /// Length of an RFC 5280-flavored ASN.1 DER-encoded [`UtcTime`].
    pub const LENGTH: Length = Length::new(13);

    /// Create a [`UtcTime`] from a [`DateTime`].
    pub fn from_date_time(datetime: DateTime) -> Result<Self> {
        if datetime.year() <= MAX_YEAR {
            Ok(Self(datetime))
        } else {
            Err(Self::TAG.value_error())
        }
    }

    /// Convert this [`UtcTime`] into a [`DateTime`].
    pub fn to_date_time(&self) -> DateTime {
        self.0
    }

    /// Create a new [`UtcTime`] given a [`Duration`] since `UNIX_EPOCH`
    /// (a.k.a. "Unix time")
    pub fn from_unix_duration(unix_duration: Duration) -> Result<Self> {
        DateTime::from_unix_duration(unix_duration)?.try_into()
    }

    /// Get the duration of this timestamp since `UNIX_EPOCH`.
    pub fn to_unix_duration(&self) -> Duration {
        self.0.unix_duration()
    }

    /// Instantiate from [`SystemTime`].
    #[cfg(feature = "std")]
    #[cfg_attr(docsrs, doc(cfg(feature = "std")))]
    pub fn from_system_time(time: SystemTime) -> Result<Self> {
        DateTime::try_from(time)
            .map_err(|_| Self::TAG.value_error())?
            .try_into()
    }

    /// Convert to [`SystemTime`].
    #[cfg(feature = "std")]
    #[cfg_attr(docsrs, doc(cfg(feature = "std")))]
    pub fn to_system_time(&self) -> SystemTime {
        self.0.to_system_time()
    }
}

impl DecodeValue<'_> for UtcTime {
    fn decode_value(decoder: &mut Decoder<'_>, header: Header) -> Result<Self> {
        match *ByteSlice::decode_value(decoder, header)?.as_slice() {
            // RFC 5280 requires mandatory seconds and Z-normalized time zone
            [year1, year2, mon1, mon2, day1, day2, hour1, hour2, min1, min2, sec1, sec2, b'Z'] => {
                let year = u16::from(datetime::decode_decimal(Self::TAG, year1, year2)?);
                let month = datetime::decode_decimal(Self::TAG, mon1, mon2)?;
                let day = datetime::decode_decimal(Self::TAG, day1, day2)?;
                let hour = datetime::decode_decimal(Self::TAG, hour1, hour2)?;
                let minute = datetime::decode_decimal(Self::TAG, min1, min2)?;
                let second = datetime::decode_decimal(Self::TAG, sec1, sec2)?;

                // RFC 5280 rules for interpreting the year
                let year = if year >= 50 {
                    year.checked_add(1900)
                } else {
                    year.checked_add(2000)
                }
                .ok_or(ErrorKind::DateTime)?;

                DateTime::new(year, month, day, hour, minute, second)
                    .map_err(|_| Self::TAG.value_error())
                    .and_then(|dt| Self::from_unix_duration(dt.unix_duration()))
            }
            _ => Err(Self::TAG.value_error()),
        }
    }
}

impl EncodeValue for UtcTime {
    fn value_len(&self) -> Result<Length> {
        Ok(Self::LENGTH)
    }

    fn encode_value(&self, writer: &mut dyn Writer) -> Result<()> {
        let year = match self.0.year() {
            y @ 1950..=1999 => y.checked_sub(1900),
            y @ 2000..=2049 => y.checked_sub(2000),
            _ => return Err(Self::TAG.value_error()),
        }
        .and_then(|y| u8::try_from(y).ok())
        .ok_or(ErrorKind::DateTime)?;

        datetime::encode_decimal(writer, Self::TAG, year)?;
        datetime::encode_decimal(writer, Self::TAG, self.0.month())?;
        datetime::encode_decimal(writer, Self::TAG, self.0.day())?;
        datetime::encode_decimal(writer, Self::TAG, self.0.hour())?;
        datetime::encode_decimal(writer, Self::TAG, self.0.minutes())?;
        datetime::encode_decimal(writer, Self::TAG, self.0.seconds())?;
        writer.write_byte(b'Z')
    }
}

impl FixedTag for UtcTime {
    const TAG: Tag = Tag::UtcTime;
}

impl OrdIsValueOrd for UtcTime {}

impl From<&UtcTime> for UtcTime {
    fn from(value: &UtcTime) -> UtcTime {
        *value
    }
}

impl From<UtcTime> for DateTime {
    fn from(utc_time: UtcTime) -> DateTime {
        utc_time.0
    }
}

impl From<&UtcTime> for DateTime {
    fn from(utc_time: &UtcTime) -> DateTime {
        utc_time.0
    }
}

impl TryFrom<DateTime> for UtcTime {
    type Error = Error;

    fn try_from(datetime: DateTime) -> Result<Self> {
        Self::from_date_time(datetime)
    }
}

impl TryFrom<&DateTime> for UtcTime {
    type Error = Error;

    fn try_from(datetime: &DateTime) -> Result<Self> {
        Self::from_date_time(*datetime)
    }
}

#[cfg(feature = "std")]
#[cfg_attr(docsrs, doc(cfg(feature = "std")))]
impl From<UtcTime> for SystemTime {
    fn from(utc_time: UtcTime) -> SystemTime {
        utc_time.to_system_time()
    }
}

impl TryFrom<Any<'_>> for UtcTime {
    type Error = Error;

    fn try_from(any: Any<'_>) -> Result<UtcTime> {
        any.decode_into()
    }
}

#[cfg(test)]
mod tests {
    use super::UtcTime;
    use crate::{Decode, Encode, Encoder};
    use hex_literal::hex;

    #[test]
    fn round_trip_vector() {
        let example_bytes = hex!("17 0d 39 31 30 35 30 36 32 33 34 35 34 30 5a");
        let utc_time = UtcTime::from_der(&example_bytes).unwrap();
        assert_eq!(utc_time.to_unix_duration().as_secs(), 673573540);

        let mut buf = [0u8; 128];
        let mut encoder = Encoder::new(&mut buf);
        utc_time.encode(&mut encoder).unwrap();
        assert_eq!(example_bytes, encoder.finish().unwrap());
    }
}
