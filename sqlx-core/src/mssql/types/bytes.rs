use crate::decode::Decode;
use crate::encode::{Encode, IsNull};
use crate::error::BoxDynError;
use crate::mssql::protocol::type_info::{DataType, TypeInfo};
use crate::mssql::{Mssql, MssqlTypeInfo, MssqlValueRef};
use crate::types::Type;

impl Type<Mssql> for Vec<u8> {
    fn type_info() -> MssqlTypeInfo {
        MssqlTypeInfo(TypeInfo::new(DataType::BigVarBinary, 0))
    }

    fn compatible(ty: &MssqlTypeInfo) -> bool {
        matches!( ty.0.ty, DataType::BigBinary | DataType::BigVarBinary  )
    }
}

impl Encode<'_, Mssql> for &'_ Vec<u8> {
    fn produces(&self) -> Option<MssqlTypeInfo> {
        // an empty string needs to be encoded as `nvarchar(2)`
        Some(MssqlTypeInfo(TypeInfo {
            ty: DataType::BigVarBinary,
            size: (self.len() as u32),
            scale: 0,
            precision: 0,
            collation: None,
        }))
    }

     fn encode_by_ref(&self, b: &mut Vec<u8>) -> IsNull {
         let buf = self.as_slice();
         b.extend(buf);
         IsNull::No
     }

}

impl Decode<'_, Mssql> for Vec<u8> {
    fn decode(value: MssqlValueRef<'_>) -> Result<Self, BoxDynError> {
        Ok( value.as_bytes().unwrap_or( &[]).to_vec() )
    }
}

