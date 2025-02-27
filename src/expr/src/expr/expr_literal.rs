// Copyright 2023 RisingWave Labs
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use std::convert::TryFrom;

use risingwave_common::array::DataChunk;
use risingwave_common::row::OwnedRow;
use risingwave_common::types::{literal_type_match, DataType, Datum};
use risingwave_common::util::value_encoding::deserialize_datum;
use risingwave_pb::expr::expr_node::{RexNode, Type};
use risingwave_pb::expr::ExprNode;

use super::ValueImpl;
use crate::expr::Expression;
use crate::{bail, ensure, ExprError, Result};

/// A literal expression.
#[derive(Debug)]
pub struct LiteralExpression {
    return_type: DataType,
    literal: Datum,
}

#[async_trait::async_trait]
impl Expression for LiteralExpression {
    fn return_type(&self) -> DataType {
        self.return_type.clone()
    }

    async fn eval_v2(&self, input: &DataChunk) -> Result<ValueImpl> {
        Ok(ValueImpl::Scalar {
            value: self.literal.clone(),
            capacity: input.capacity(),
        })
    }

    async fn eval_row(&self, _input: &OwnedRow) -> Result<Datum> {
        Ok(self.literal.as_ref().cloned())
    }

    fn eval_const(&self) -> Result<Datum> {
        Ok(self.literal.clone())
    }
}

impl LiteralExpression {
    pub fn new(return_type: DataType, literal: Datum) -> Self {
        assert!(literal_type_match(&return_type, literal.as_ref()));
        LiteralExpression {
            return_type,
            literal,
        }
    }

    pub fn literal(&self) -> Datum {
        self.literal.clone()
    }
}

impl<'a> TryFrom<&'a ExprNode> for LiteralExpression {
    type Error = ExprError;

    fn try_from(prost: &'a ExprNode) -> Result<Self> {
        ensure!(prost.get_expr_type().unwrap() == Type::ConstantValue);
        let ret_type = DataType::from(prost.get_return_type().unwrap());
        if prost.rex_node.is_none() {
            return Ok(Self {
                return_type: ret_type,
                literal: None,
            });
        }

        if let RexNode::Constant(prost_value) = prost.get_rex_node().unwrap() {
            // TODO: We need to unify these
            let value = deserialize_datum(
                prost_value.get_body().as_slice(),
                &DataType::from(prost.get_return_type().unwrap()),
            )
            .map_err(|e| ExprError::Internal(e.into()))?;
            Ok(Self {
                return_type: ret_type,
                literal: value,
            })
        } else {
            bail!("Cannot parse the RexNode");
        }
    }
}

#[cfg(test)]
mod tests {
    use risingwave_common::array::{I32Array, StructValue};
    use risingwave_common::types::test_utils::IntervalTestExt;
    use risingwave_common::types::{Decimal, Interval, IntoOrdered, Scalar, ScalarImpl};
    use risingwave_common::util::value_encoding::serialize_datum;
    use risingwave_pb::data::data_type::{IntervalType, TypeName};
    use risingwave_pb::data::{PbDataType, PbDatum};
    use risingwave_pb::expr::expr_node::RexNode::Constant;
    use risingwave_pb::expr::expr_node::Type;
    use risingwave_pb::expr::ExprNode;

    use super::*;

    #[test]
    fn test_struct_expr_literal_from() {
        let value = StructValue::new(vec![
            Some(ScalarImpl::Utf8("12222".into())),
            Some(2.into()),
            None,
        ]);
        let body = serialize_datum(Some(value.clone().to_scalar_value()).as_ref());
        let expr = ExprNode {
            expr_type: Type::ConstantValue as i32,
            return_type: Some(PbDataType {
                type_name: TypeName::Struct as i32,
                field_type: vec![
                    PbDataType {
                        type_name: TypeName::Varchar as i32,
                        ..Default::default()
                    },
                    PbDataType {
                        type_name: TypeName::Int32 as i32,
                        ..Default::default()
                    },
                    PbDataType {
                        type_name: TypeName::Int32 as i32,
                        ..Default::default()
                    },
                ],
                ..Default::default()
            }),
            rex_node: Some(Constant(PbDatum { body })),
        };
        let expr = LiteralExpression::try_from(&expr).unwrap();
        assert_eq!(value.to_scalar_value(), expr.literal().unwrap());
    }

    #[test]
    fn test_expr_literal_from() {
        let v = true;
        let t = TypeName::Boolean;
        let bytes = serialize_datum(Some(v.to_scalar_value()).as_ref());

        let expr = LiteralExpression::try_from(&make_expression(Some(bytes), t)).unwrap();
        assert_eq!(v.to_scalar_value(), expr.literal().unwrap());

        let v = 1i16;
        let t = TypeName::Int16;
        let bytes = serialize_datum(Some(v.to_scalar_value()).as_ref());

        let expr = LiteralExpression::try_from(&make_expression(Some(bytes), t)).unwrap();
        assert_eq!(v.to_scalar_value(), expr.literal().unwrap());

        let v = 1i32;
        let t = TypeName::Int32;
        let bytes = serialize_datum(Some(v.to_scalar_value()).as_ref());
        let expr = LiteralExpression::try_from(&make_expression(Some(bytes), t)).unwrap();
        assert_eq!(v.to_scalar_value(), expr.literal().unwrap());

        let v = 1i64;
        let t = TypeName::Int64;
        let bytes = serialize_datum(Some(v.to_scalar_value()).as_ref());
        let expr = LiteralExpression::try_from(&make_expression(Some(bytes), t)).unwrap();
        assert_eq!(v.to_scalar_value(), expr.literal().unwrap());

        let v = 1f32.into_ordered();
        let t = TypeName::Float;
        let bytes = serialize_datum(Some(v.to_scalar_value()).as_ref());
        let expr = LiteralExpression::try_from(&make_expression(Some(bytes), t)).unwrap();
        assert_eq!(v.to_scalar_value(), expr.literal().unwrap());

        let v = 1f64.into_ordered();
        let t = TypeName::Double;
        let bytes = serialize_datum(Some(v.to_scalar_value()).as_ref());
        let expr = LiteralExpression::try_from(&make_expression(Some(bytes), t)).unwrap();
        assert_eq!(v.to_scalar_value(), expr.literal().unwrap());

        let v = None;
        let t = TypeName::Float;
        let expr = LiteralExpression::try_from(&make_expression(None, t)).unwrap();
        assert_eq!(v, expr.literal());

        let v: Box<str> = "varchar".into();
        let t = TypeName::Varchar;
        let bytes = serialize_datum(Some(v.clone().to_scalar_value()).as_ref());
        let expr = LiteralExpression::try_from(&make_expression(Some(bytes), t)).unwrap();
        assert_eq!(v.to_scalar_value(), expr.literal().unwrap());

        let v = Decimal::new(3141, 3);
        let t = TypeName::Decimal;
        let bytes = serialize_datum(Some(v.to_scalar_value()).as_ref());
        let expr = LiteralExpression::try_from(&make_expression(Some(bytes), t)).unwrap();
        assert_eq!(v.to_scalar_value(), expr.literal().unwrap());

        let v = 32i32;
        let t = TypeName::Interval;
        let bytes = serialize_datum(Some(Interval::from_month(v).to_scalar_value()).as_ref());
        let expr = LiteralExpression::try_from(&make_expression(Some(bytes), t)).unwrap();
        assert_eq!(
            Interval::from_month(v).to_scalar_value(),
            expr.literal().unwrap()
        );
    }

    fn make_expression(bytes: Option<Vec<u8>>, data_type: TypeName) -> ExprNode {
        ExprNode {
            expr_type: Type::ConstantValue as i32,
            return_type: Some(PbDataType {
                type_name: data_type as i32,
                interval_type: IntervalType::Month as i32,
                ..Default::default()
            }),
            rex_node: bytes.map(|bs| RexNode::Constant(PbDatum { body: bs })),
        }
    }

    #[tokio::test]
    async fn test_literal_eval_dummy_chunk() {
        let literal = LiteralExpression::new(DataType::Int32, Some(1.into()));
        let result = literal.eval(&DataChunk::new_dummy(1)).await.unwrap();
        assert_eq!(*result, I32Array::from_iter([1]).into());
    }

    #[tokio::test]
    async fn test_literal_eval_row_dummy_chunk() {
        let literal = LiteralExpression::new(DataType::Int32, Some(1.into()));
        let result = literal.eval_row(&OwnedRow::new(vec![])).await.unwrap();
        assert_eq!(result, Some(1.into()))
    }
}
