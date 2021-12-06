use crate::array::{ArrayBuilder, ArrayBuilderImpl, PrimitiveArrayBuilder};
use crate::error::Result;
use crate::error::RwError;
use crate::types::DataSize;
use crate::types::DataType;
use crate::types::DataTypeKind;
use crate::types::DataTypeRef;
use crate::types::PrimitiveDataType;
use risingwave_pb::data::data_type::TypeName;
use risingwave_pb::data::DataType as DataTypeProto;
use risingwave_pb::ToProto;
use std::any::Any;
use std::convert::TryFrom;
use std::default::Default;
use std::mem::size_of;
use std::sync::Arc;

macro_rules! make_numeric_type {
    ($name:ident, $native_ty:ty, $data_ty:expr, $proto_ty:expr) => {
        #[derive(Debug, Eq, PartialEq)]
        pub struct $name {
            nullable: bool,
        }

        impl $name {
            pub fn new(nullable: bool) -> Self {
                Self { nullable }
            }

            pub fn create(nullable: bool) -> DataTypeRef {
                Arc::new(Self::new(nullable))
            }
        }

        impl Default for $name {
            fn default() -> Self {
                Self { nullable: false }
            }
        }

        impl DataType for $name {
            fn data_type_kind(&self) -> DataTypeKind {
                $data_ty
            }

            fn is_nullable(&self) -> bool {
                self.nullable
            }

            fn create_array_builder(&self, capacity: usize) -> Result<ArrayBuilderImpl> {
                Ok(PrimitiveArrayBuilder::<$native_ty>::new(capacity)?.into())
            }

            fn to_protobuf(&self) -> Result<risingwave_proto::data::DataType> {
                self.to_prost()
                    .map(|x| x.to_proto::<risingwave_proto::data::DataType>())
            }

            fn to_prost(&self) -> Result<DataTypeProto> {
                let proto = DataTypeProto {
                    type_name: $proto_ty as i32,
                    is_nullable: self.nullable,
                    ..Default::default()
                };
                Ok(proto)
            }

            fn as_any(&self) -> &dyn Any {
                self
            }

            fn data_size(&self) -> DataSize {
                DataSize::Fixed(size_of::<$native_ty>())
            }
        }

        impl PrimitiveDataType for $name {
            const DATA_TYPE_KIND: DataTypeKind = $data_ty;
            type N = $native_ty;
        }

        impl<'a> TryFrom<&'a DataTypeProto> for $name {
            type Error = RwError;

            fn try_from(proto: &'a DataTypeProto) -> Result<Self> {
                ensure!(proto.get_type_name() == $proto_ty);
                Ok(Self {
                    nullable: proto.is_nullable,
                })
            }
        }
    };
}

make_numeric_type!(Int16Type, i16, DataTypeKind::Int16, TypeName::Int16);
make_numeric_type!(Int32Type, i32, DataTypeKind::Int32, TypeName::Int32);
make_numeric_type!(Int64Type, i64, DataTypeKind::Int64, TypeName::Int64);
make_numeric_type!(Float32Type, f32, DataTypeKind::Float32, TypeName::Float);
make_numeric_type!(Float64Type, f64, DataTypeKind::Float64, TypeName::Double);
make_numeric_type!(DateType, i32, DataTypeKind::Date, TypeName::Date);
