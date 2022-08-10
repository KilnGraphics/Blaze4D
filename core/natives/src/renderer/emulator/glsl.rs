use std::collections::HashMap;

use glsl::syntax::{Expr, Identifier, StructSpecifier, TypeSpecifier, TypeSpecifierNonArray, UnaryOp};
use glsl::syntax::TypeSpecifierNonArray::TypeName;

use crate::prelude::*;

pub fn const_eval<L: ConstLookup>(expr: &Expr, lookup: &L) -> Result<Const, ConstEvalError> {
    match expr {
        Expr::Variable(ident) => lookup.lookup_const(ident).cloned().ok_or(ConstEvalError::UnknownIdentifier(ident.0.clone()))?,
        Expr::IntConst(v) => Const { type_spec: TypeSpecifier::new(TypeSpecifierNonArray::Int), val: ConstVal::Int(*v) },
        Expr::UIntConst(v) => Const { type_spec: TypeSpecifier::new(TypeSpecifierNonArray::UInt), val: ConstVal::UInt(*v) },
        Expr::BoolConst(v) => Const { type_spec: TypeSpecifier::new(TypeSpecifierNonArray::Bool), val: ConstVal::Bool(*v) },
        Expr::FloatConst(v) => Const { type_spec: TypeSpecifier::new(TypeSpecifierNonArray::Float), val: ConstVal::Float(*v) },
        Expr::DoubleConst(v) => Const { type_spec: TypeSpecifier::new(TypeSpecifierNonArray::Double), val: ConstVal::Double(*v) },
        _ => todo!(),
    };

    todo!()
}

pub fn const_eval_unary<T: ConstLookup>(op: UnaryOp, a: &Expr, lookup: &T) -> Result<Const, ConstEvalError> {
    if op == UnaryOp::Inc || op == UnaryOp::Dec {
        return Err(ConstEvalError::IllegalUnaryOp(op))
    }

    todo!()
}

pub trait ConstLookup {
    fn lookup_const(&self, ident: &Identifier) -> Option<&Const>;

    fn is_const(&self, ident: &Identifier) -> bool {
        self.lookup_const(ident).is_some()
    }
}

#[derive(Clone)]
pub struct Const {
    pub type_spec: TypeSpecifier,
    pub val: ConstVal,
}

impl Const {
    /// Attempts to implicitly convert this const value to the target type. Only conversions legal
    /// for glsl implicit conversions are supported. If the target type is equal to the const type
    /// a copy of this const is returned.
    ///
    /// If the conversion is not legal `None` is returned.
    pub fn implicit_convert_to(&self, target: &TypeSpecifierNonArray) -> Option<Const> {
        if self.type_spec.array_specifier.is_some() {
            return None;
        }
        if &self.type_spec.ty == target {
            return Some(self.clone());
        }

        let val = match target {
            TypeSpecifierNonArray::UInt => {
                ConstVal::UInt(match &self.val {
                    ConstVal::Int(v) => *v as u32,
                    _ => return None,
                })
            },
            TypeSpecifierNonArray::Float => {
                ConstVal::Float(match &self.val {
                    ConstVal::Int(v) => *v as f32,
                    ConstVal::UInt(v) => *v as f32,
                    _ => return None,
                })
            },
            TypeSpecifierNonArray::Double => {
                ConstVal::Double(match &self.val {
                    ConstVal::Int(v) => *v as f64,
                    ConstVal::UInt(v) => *v as f64,
                    ConstVal::Float(v) => *v as f64,
                    _ => return None,
                })
            },
            TypeSpecifierNonArray::UVec2 => {
                ConstVal::UVec2(match &self.val {
                    ConstVal::IVec2(v) => v.map(|v| v as u32),
                    _ => return None,
                })
            },
            TypeSpecifierNonArray::UVec3 => {
                ConstVal::UVec3(match &self.val {
                    ConstVal::IVec3(v) => v.map(|v| v as u32),
                    _ => return None,
                })
            },
            TypeSpecifierNonArray::UVec4 => {
                ConstVal::UVec4(match &self.val {
                    ConstVal::IVec4(v) => v.map(|v| v as u32),
                    _ => return None,
                })
            },
            TypeSpecifierNonArray::Vec2 => {
                ConstVal::Vec2(match &self.val {
                    ConstVal::IVec2(v) => v.map(|v| v as f32),
                    ConstVal::UVec2(v) => v.map(|v| v as f32),
                    _ => return None,
                })
            },
            TypeSpecifierNonArray::Vec3 => {
                ConstVal::Vec3(match &self.val {
                    ConstVal::IVec3(v) => v.map(|v| v as f32),
                    ConstVal::UVec3(v) => v.map(|v| v as f32),
                    _ => return None,
                })
            },
            TypeSpecifierNonArray::Vec4 => {
                ConstVal::Vec4(match &self.val {
                    ConstVal::IVec4(v) => v.map(|v| v as f32),
                    ConstVal::UVec4(v) => v.map(|v| v as f32),
                    _ => return None,
                })
            },
            TypeSpecifierNonArray::DVec2 => {
                ConstVal::DVec2(match &self.val {
                    ConstVal::IVec2(v) => v.map(|v| v as f64),
                    ConstVal::UVec2(v) => v.map(|v| v as f64),
                    ConstVal::Vec2(v) => v.map(|v| v as f64),
                    _ => return None,
                })
            },
            TypeSpecifierNonArray::DVec3 => {
                ConstVal::DVec3(match &self.val {
                    ConstVal::IVec3(v) => v.map(|v| v as f64),
                    ConstVal::UVec3(v) => v.map(|v| v as f64),
                    ConstVal::Vec3(v) => v.map(|v| v as f64),
                    _ => return None,
                })
            },
            TypeSpecifierNonArray::DVec4 => {
                ConstVal::DVec4(match &self.val {
                    ConstVal::IVec4(v) => v.map(|v| v as f64),
                    ConstVal::UVec4(v) => v.map(|v| v as f64),
                    ConstVal::Vec4(v) => v.map(|v| v as f64),
                    _ => return None,
                })
            },
            TypeSpecifierNonArray::DMat2 => {
                ConstVal::DMat2(match &self.val {
                    ConstVal::Mat2(v) => v.map(|v| v as f64),
                    _ => return None,
                })
            },
            TypeSpecifierNonArray::DMat3 => {
                ConstVal::DMat3(match &self.val {
                    ConstVal::Mat3(v) => v.map(|v| v as f64),
                    _ => return None,
                })
            },
            TypeSpecifierNonArray::DMat4 => {
                ConstVal::DMat4(match &self.val {
                    ConstVal::Mat4(v) => v.map(|v| v as f64),
                    _ => return None,
                })
            },
            TypeSpecifierNonArray::DMat23 => {
                ConstVal::DMat23(match &self.val {
                    ConstVal::Mat23(v) => v.map(|v| v as f64),
                    _ => return None,
                })
            },
            TypeSpecifierNonArray::DMat24 => {
                ConstVal::DMat24(match &self.val {
                    ConstVal::Mat24(v) => v.map(|v| v as f64),
                    _ => return None,
                })
            },
            TypeSpecifierNonArray::DMat32 => {
                ConstVal::DMat32(match &self.val {
                    ConstVal::Mat32(v) => v.map(|v| v as f64),
                    _ => return None,
                })
            },
            TypeSpecifierNonArray::DMat34 => {
                ConstVal::DMat34(match &self.val {
                    ConstVal::Mat34(v) => v.map(|v| v as f64),
                    _ => return None,
                })
            },
            TypeSpecifierNonArray::DMat42 => {
                ConstVal::DMat42(match &self.val {
                    ConstVal::Mat42(v) => v.map(|v| v as f64),
                    _ => return None,
                })
            },
            TypeSpecifierNonArray::DMat43 => {
                ConstVal::DMat43(match &self.val {
                    ConstVal::Mat43(v) => v.map(|v| v as f64),
                    _ => return None,
                })
            }
            _ => return None,
        };

        Some(Const {
            type_spec: TypeSpecifier::new(target.clone()),
            val
        })
    }


}

#[derive(Clone)]
pub enum ConstVal {
    Bool(bool),
    Int(i32),
    UInt(u32),
    Float(f32),
    Double(f64),
    Vec2(Vec2f32),
    Vec3(Vec3f32),
    Vec4(Vec4f32),
    DVec2(Vec2f64),
    DVec3(Vec3f64),
    DVec4(Vec4f64),
    BVec2(Vec2Bool),
    BVec3(Vec3Bool),
    BVec4(Vec4Bool),
    IVec2(Vec2i32),
    IVec3(Vec3i32),
    IVec4(Vec4i32),
    UVec2(Vec2u32),
    UVec3(Vec3u32),
    UVec4(Vec4u32),
    Mat2(Mat2f32),
    Mat3(Mat3f32),
    Mat4(Mat4f32),
    Mat23(Mat2x3f32),
    Mat24(Mat2x4f32),
    Mat32(Mat3x2f32),
    Mat34(Mat3x4f32),
    Mat42(Mat4x2f32),
    Mat43(Mat4x3f32),
    DMat2(Mat2f64),
    DMat3(Mat3f64),
    DMat4(Mat4f64),
    DMat23(Mat2x3f64),
    DMat24(Mat2x4f64),
    DMat32(Mat3x2f64),
    DMat34(Mat3x4f64),
    DMat42(Mat4x2f64),
    DMat43(Mat4x3f64),
    Struct(ConstStruct),
    Array(Box<[ConstVal]>),
}

#[derive(Clone)]
pub struct ConstStruct {
    entries: HashMap<String, Const>,
}

impl ConstStruct {
}

impl ConstLookup for ConstStruct {
    fn lookup_const(&self, ident: &Identifier) -> Option<&Const> {
        self.entries.get(&ident.0)
    }
}

pub enum ConstEvalError {
    UnknownIdentifier(String),
    IllegalUnaryOp(UnaryOp),
    IllegalUnaryOperand(UnaryOp, ConstVal),
}