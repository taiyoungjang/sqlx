use crate::decode::Decode;
//use crate::encode::{Encode, IsNull};
use crate::error::BoxDynError;
use crate::mssql::protocol::type_info::{DataType, TypeInfo};
use crate::mssql::{Mssql, MssqlTypeInfo, MssqlValueRef};
use crate::types::Type;

impl Type<Mssql> for Vec<u8> {
    fn type_info() -> MssqlTypeInfo {
        MssqlTypeInfo(TypeInfo::new(DataType::Guid, 16))
    }

    fn compatible(ty: &MssqlTypeInfo) -> bool {
        matches!( ty.0.ty, DataType::Guid  ) && ty.0.size == 16
    }
}
/*
impl Encode<'_, Mssql> for &'_ Vec<u8> {

     fn encode_by_ref(&self, b: &mut Vec<u8>) -> IsNull {

         let header:[u8;1] = [/*VarLenType::Guid 0x24, 16,*/ 16];
         b.extend(&header);
         let buf = self.as_slice().to_vec();
         //b.extend(&buf);
         b.extend(&[
              buf[3], buf[2], buf[1], buf[0], buf[5], buf[4], buf[7], buf[6], buf[8], buf[9], buf[10],
              buf[11], buf[12], buf[13], buf[14], buf[15],
         ]);
         println!("encode_by_ref: buf:{:?}", /*self, header, header.len(),*/ buf);
         println!("encode_by_ref: b:{:?} len:{}", /*self, header, header.len(),*/  b, b.len());
         IsNull::No
     }

}
*/
impl Decode<'_, Mssql> for Vec<u8> {
    fn decode(value: MssqlValueRef<'_>) -> Result<Self, BoxDynError> {
        let b = value.as_bytes()?;
        match b.len() {
            16 => {
                Ok( [
                    b[3], b[2], b[1], b[0], b[5], b[4], b[7], b[6], b[8], b[9], b[10],
                    b[11], b[12], b[13], b[14], b[15],
                ].to_vec())
            }
            _ => Ok(vec![]),
        }
    }
}

// impl Encode<'_, Mssql> for Cow<'_, str> {
//     fn produces(&self) -> Option<MssqlTypeInfo> {
//         match self {
//             Cow::Borrowed(str) => <&str as Encode<Mssql>>::produces(str),
//             Cow::Owned(str) => <&str as Encode<Mssql>>::produces(&(str.as_ref())),
//         }
//     }
//
//     fn encode_by_ref(&self, buf: &mut Vec<u8>) -> IsNull {
//         match self {
//             Cow::Borrowed(str) => <&str as Encode<Mssql>>::encode_by_ref(str, buf),
//             Cow::Owned(str) => <&str as Encode<Mssql>>::encode_by_ref(&(str.as_ref()), buf),
//         }
//     }
// }

