use crate::decode::Decode;
use crate::error::BoxDynError;
use crate::mssql::protocol::type_info::{DataType, TypeInfo};
use crate::mssql::{Mssql, MssqlTypeInfo, MssqlValueRef};
use crate::types::Type;
use std::borrow::Cow;
use std::ops::Sub;
use binary_reader::Endian;
use std::time::SystemTime;
use chrono;

impl Type<Mssql> for SystemTime {
    fn type_info() -> MssqlTypeInfo {
        MssqlTypeInfo(TypeInfo::new(DataType::DateTimeOffsetN, 0))
    }

    fn compatible(ty: &MssqlTypeInfo) -> bool {
        matches!(
            ty.0.ty,
            DataType::DateTimeOffsetN
        )
    }
}

// impl Encode<'_, Mssql> for &'_ SystemTime {
//     fn encode_by_ref(&self, _buf: &mut Vec<u8>) -> IsNull {
//         //let utc_dt = self.to_offset(UtcOffset::UTC);
//         //TODO
//         IsNull::No
//     }
// }

impl Type<Mssql> for Cow<'_, SystemTime> {
    fn type_info() -> MssqlTypeInfo {
        <&SystemTime as Type<Mssql>>::type_info()
    }

    fn compatible(ty: &MssqlTypeInfo) -> bool {
        <&SystemTime as Type<Mssql>>::compatible(ty)
    }
}

// impl Encode<'_, Mssql> for Cow<'_, SystemTime> {
//     fn encode_by_ref(&self, buf: &mut Vec<u8>) -> IsNull {
//         match self {
//             Cow::Borrowed(o) => <&SystemTime as Encode<Mssql>>::encode_by_ref(o, buf),
//             Cow::Owned(o) => <&SystemTime as Encode<Mssql>>::encode_by_ref(&o, buf),
//         }
//     }
// }

impl<'r> Decode<'r, Mssql> for SystemTime {
    fn decode(value: MssqlValueRef<'r>) -> Result<Self, BoxDynError> {
        let buf = value.as_bytes()?;
        let rlen = buf.len();
        let s = match rlen {
            0 => SystemTime::UNIX_EPOCH,
            _ => {
                let mut src = binary_reader::BinaryReader::from_u8(&buf);
                src.set_endian(Endian::Little);
                let datetime2 = DateTime2::decode(& mut src, 7, rlen - 5 ).unwrap();
                let offset = src.read_i16().unwrap();
                let date = from_days(datetime2.date.days() as i64, 1);
                let ns = datetime2.time.increments as i64 * 10i64.pow(9 - datetime2.time.scale as u32);
                let time = chrono::NaiveTime::from_hms_opt(0,0,0).unwrap() + chrono::Duration::nanoseconds(ns);
                let offset = chrono::Duration::minutes(offset as i64);
                let naive = chrono::NaiveDateTime::new(date, time).sub(offset);
                //let datetime = chrono::DateTime::<Utc>::from_utc(naive, Utc);
                let timestamp = i64::clamp(naive.timestamp(), 0, i64::MAX as i64);
                SystemTime::UNIX_EPOCH + std::time::Duration::new(timestamp as u64, 0 as u32)
            }
        };
        Ok(s)
    }
}

pub struct DateTime2 {
    date: Date,
    time: Time,
}
impl DateTime2 {
    /// Construct a new `DateTime2` from the date and time components.
    pub fn new(date: Date, time: Time) -> Self {
        Self { date, time }
    }

    /// The date component.
    #[allow(dead_code)]
    pub fn date(self) -> Date {
        self.date
    }

    /// The time component.
    #[allow(dead_code)]
    pub fn time(self) -> Time {
        self.time
    }

    pub(crate) fn decode(src: &mut binary_reader::BinaryReader, n: usize, rlen: usize) -> crate::error::Result<Self>
    {
        let time = Time::decode(src, n, rlen as usize).unwrap();

        let mut bytes = [0u8; 4];
        let reads= src.read(3).unwrap();
        bytes[..3].clone_from_slice(reads);
        let date = Date::new(u32::from_le_bytes(bytes));

        Ok(Self::new(date, time))
    }
}
pub struct Time {
    increments: u64,
    scale: u8,
}
impl Time {
    /// Construct a new `Time`
    #[allow(dead_code)]
    pub fn new(increments: u64, scale: u8) -> Self {
        Self { increments, scale }
    }

    #[inline]
    #[allow(dead_code)]
    /// Number of 10^-n second increments since midnight, where `n` is defined
    /// in [`scale`].
    ///
    /// [`scale`]: #method.scale
    pub fn increments(self) -> u64 {
        self.increments
    }

    #[inline]
    #[allow(dead_code)]
    /// The accuracy of the increments.
    pub fn scale(self) -> u8 {
        self.scale
    }

    #[inline]
    #[allow(dead_code)]
    /// Length of the field in number of bytes.
    pub(crate) fn len(self) -> crate::error::Result<u8> {
        Ok(match self.scale {
            0..=2 => 3,
            3..=4 => 4,
            5..=7 => 5,
            _ => {
                return Err(err_protocol!(format!("timen: invalid scale {}", self.scale)));
            }
        })
    }

    pub(crate) fn decode(src: &mut binary_reader::BinaryReader, n: usize, rlen: usize) -> crate::error::Result<Time>
    {
        let val =  match (n, rlen) {
            (0..=2, 3) => {
                let hi = src.read_u16().unwrap() as u64;
                let lo = src.read_u8().unwrap() as u64;
                hi | lo << 16
            }
            (3..=4, 4) => {
                src.read_u32().unwrap() as u64
            },
            (5..=7, 5) => {
                let hi = src.read_u32().unwrap() as u64;
                let lo = src.read_u8().unwrap() as u64;
                hi | lo << 32
           }
            _ => {
                return Err(err_protocol!("timen: invalid length {} {}", n, rlen))
            }
        };

        Ok(Time {
            increments: val,
            scale: n as u8,
        })
    }
}

pub struct Date(u32);
impl Date {
    #[inline]
    /// Construct a new `Date`
    ///
    /// # Panics
    /// max value of 3 bytes (`u32::max_value() > 8`)
    pub fn new(days: u32) -> Date {
        assert_eq!(days >> 24, 0);
        Date(days)
    }

    #[inline]
    /// The number of days from 1st of January, year 1.
    pub fn days(self) -> u32 {
        self.0
    }
    #[allow(dead_code)]
    pub(crate) async fn decode(src: &mut binary_reader::BinaryReader) -> crate::error::Result<Self>
    {
        Ok(Self::new(src.read_u32().unwrap()))
    }
}
fn from_days(days: i64, start_year: i32) -> chrono::NaiveDate {
    chrono::NaiveDate::from_ymd_opt(start_year, 1, 1).unwrap() + chrono::Duration::days(days as i64)
}
