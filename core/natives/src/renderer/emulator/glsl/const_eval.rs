use std::collections::HashMap;
use std::ops::{BitAnd, BitOr, BitXor, Neg, Not};

use glsl::syntax::{ArraySpecifier, BinaryOp, Expr, Identifier, StructSpecifier, TypeSpecifier, TypeSpecifierNonArray, UnaryOp};
use nalgebra::{Const, DMatrix, Matrix2, Matrix2x3, Matrix2x4, Matrix3, Matrix3x2, Matrix3x4, Matrix4, Matrix4x2, Matrix4x3, Scalar, Vector2, Vector3, Vector4};

use crate::prelude::*;
use crate::renderer::emulator::glsl::const_eval::function::{ParameterBaseType, ParameterSize, ParameterType};


pub trait ConstLookup {
    fn lookup_const(&self, ident: &Identifier) -> Option<&ConstVal>;

    fn is_const(&self, ident: &Identifier) -> bool {
        self.lookup_const(ident).is_some()
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub enum BaseTypeSize {
    Scalar,
    Vec2,
    Vec3,
    Vec4,
    Mat2,
    Mat23,
    Mat24,
    Mat32,
    Mat3,
    Mat34,
    Mat42,
    Mat43,
    Mat4
}

/// Constant scalar or vector type
#[derive(Clone, PartialEq, Hash, Debug)]
pub enum ConstSVVal<T: Scalar> {
    Scalar(T),
    Vec2(Vector2<T>),
    Vec3(Vector3<T>),
    Vec4(Vector4<T>),
}

impl<T: Scalar> ConstSVVal<T> {
    fn get_size(&self) -> BaseTypeSize {
        match self {
            ConstSVVal::Scalar(_) => BaseTypeSize::Scalar,
            ConstSVVal::Vec2(_) => BaseTypeSize::Vec2,
            ConstSVVal::Vec3(_) => BaseTypeSize::Vec3,
            ConstSVVal::Vec4(_) => BaseTypeSize::Vec4,
        }
    }

    fn map<R: Scalar, F: FnMut(T) -> R>(&self, mut f: F) -> ConstSVVal<R> {
        match self {
            ConstSVVal::Scalar(v) => ConstSVVal::Scalar(f(v.clone())),
            ConstSVVal::Vec2(v) => ConstSVVal::Vec2(v.map(f)),
            ConstSVVal::Vec3(v) => ConstSVVal::Vec3(v.map(f)),
            ConstSVVal::Vec4(v) => ConstSVVal::Vec4(v.map(f)),
        }
    }

    fn zip_map<T2: Scalar, R: Scalar, F: FnMut(T, T2) -> R>(&self, other: &ConstSVVal<T2>, mut f: F) -> Option<ConstSVVal<R>> {
        match (self, other) {
            (ConstSVVal::Scalar(a), ConstSVVal::Scalar(b)) => Some(ConstSVVal::Scalar(f(a.clone(), b.clone()))),
            (ConstSVVal::Vec2(a), ConstSVVal::Vec2(b)) => Some(ConstSVVal::Vec2(a.zip_map(b, f))),
            (ConstSVVal::Vec3(a), ConstSVVal::Vec3(b)) => Some(ConstSVVal::Vec3(a.zip_map(b, f))),
            (ConstSVVal::Vec4(a), ConstSVVal::Vec4(b)) => Some(ConstSVVal::Vec4(a.zip_map(b, f))),
            _ => None
        }
    }
}

/// Constant matrix type
#[derive(Clone, PartialEq, Hash, Debug)]
pub enum ConstMVal<T: Scalar> {
    Mat2(Matrix2<T>),
    Mat23(Matrix2x3<T>),
    Mat24(Matrix2x4<T>),
    Mat32(Matrix3x2<T>),
    Mat3(Matrix3<T>),
    Mat34(Matrix3x4<T>),
    Mat42(Matrix4x2<T>),
    Mat43(Matrix4x3<T>),
    Mat4(Matrix4<T>),
}

impl<T: Scalar> ConstMVal<T> {
    fn get_size(&self) -> BaseTypeSize {
        match self {
            ConstMVal::Mat2(_) => BaseTypeSize::Mat2,
            ConstMVal::Mat23(_) => BaseTypeSize::Mat23,
            ConstMVal::Mat24(_) => BaseTypeSize::Mat24,
            ConstMVal::Mat32(_) => BaseTypeSize::Mat32,
            ConstMVal::Mat3(_) => BaseTypeSize::Mat3,
            ConstMVal::Mat34(_) => BaseTypeSize::Mat34,
            ConstMVal::Mat42(_) => BaseTypeSize::Mat42,
            ConstMVal::Mat43(_) => BaseTypeSize::Mat43,
            ConstMVal::Mat4(_) => BaseTypeSize::Mat4,
        }
    }

    fn map<R: Scalar, F: FnMut(T) -> R>(&self, mut f: F) -> ConstMVal<R> {
        match self {
            ConstMVal::Mat2(v) => ConstMVal::Mat2(v.map(f)),
            ConstMVal::Mat23(v) => ConstMVal::Mat23(v.map(f)),
            ConstMVal::Mat24(v) => ConstMVal::Mat24(v.map(f)),
            ConstMVal::Mat32(v) => ConstMVal::Mat32(v.map(f)),
            ConstMVal::Mat3(v) => ConstMVal::Mat3(v.map(f)),
            ConstMVal::Mat34(v) => ConstMVal::Mat34(v.map(f)),
            ConstMVal::Mat42(v) => ConstMVal::Mat42(v.map(f)),
            ConstMVal::Mat43(v) => ConstMVal::Mat43(v.map(f)),
            ConstMVal::Mat4(v) => ConstMVal::Mat4(v.map(f)),
        }
    }

    fn zip_map<T2: Scalar, R: Scalar, F: FnMut(T, T2) -> R>(&self, other: &ConstMVal<T2>, mut f: F) -> Option<ConstMVal<R>> {
        match (self, other) {
            (ConstMVal::Mat2(a), ConstMVal::Mat2(b)) => Some(ConstMVal::Mat2(a.zip_map(b, f))),
            (ConstMVal::Mat23(a), ConstMVal::Mat23(b)) => Some(ConstMVal::Mat23(a.zip_map(b, f))),
            (ConstMVal::Mat24(a), ConstMVal::Mat24(b)) => Some(ConstMVal::Mat24(a.zip_map(b, f))),
            (ConstMVal::Mat32(a), ConstMVal::Mat32(b)) => Some(ConstMVal::Mat32(a.zip_map(b, f))),
            (ConstMVal::Mat3(a), ConstMVal::Mat3(b)) => Some(ConstMVal::Mat3(a.zip_map(b, f))),
            (ConstMVal::Mat34(a), ConstMVal::Mat34(b)) => Some(ConstMVal::Mat34(a.zip_map(b, f))),
            (ConstMVal::Mat42(a), ConstMVal::Mat42(b)) => Some(ConstMVal::Mat42(a.zip_map(b, f))),
            (ConstMVal::Mat43(a), ConstMVal::Mat43(b)) => Some(ConstMVal::Mat43(a.zip_map(b, f))),
            (ConstMVal::Mat4(a), ConstMVal::Mat4(b)) => Some(ConstMVal::Mat4(a.zip_map(b, f))),
            _ => None
        }
    }
}

/// Constant scalar, vector or matrix type
#[derive(Clone, PartialEq, Hash, Debug)]
pub enum ConstSVMVal<T: Scalar> {
    SV(ConstSVVal<T>),
    M(ConstMVal<T>),
}

impl<T: Scalar> ConstSVMVal<T> {
    fn get_size(&self) -> BaseTypeSize {
        match self {
            ConstSVMVal::SV(v) => v.get_size(),
            ConstSVMVal::M(v) => v.get_size(),
        }
    }

    fn map<R: Scalar, F: FnMut(T) -> R>(self, mut f: F) -> ConstSVMVal<R> {
        match self {
            ConstSVMVal::SV(v) => v.map(f).into(),
            ConstSVMVal::M(v) => v.map(f).into(),
        }
    }

    fn zip_map<T2: Scalar, R: Scalar, F: FnMut(T, T2) -> R>(&self, other: &ConstSVMVal<T2>, mut f: F) -> Option<ConstSVMVal<R>> {
        match (self, other) {
            (ConstSVMVal::SV(a), ConstSVMVal::SV(b)) => a.zip_map(b, f).map(ConstSVMVal::from),
            (ConstSVMVal::M(a), ConstSVMVal::M(b)) => a.zip_map(b, f).map(ConstSVMVal::from),
            _ => None,
        }
    }
}

impl<T: Scalar> From<ConstSVVal<T>> for ConstSVMVal<T> {
    fn from(sv: ConstSVVal<T>) -> Self {
        Self::SV(sv)
    }
}

impl<T: Scalar> From<ConstMVal<T>> for ConstSVMVal<T> {
    fn from(m: ConstMVal<T>) -> Self {
        Self::M(m)
    }
}

#[derive(Clone, PartialEq, Debug)]
pub enum ConstBaseVal {
    Bool(ConstSVVal<bool>),
    Int(ConstSVVal<i32>),
    UInt(ConstSVVal<u32>),
    Float(ConstSVMVal<f32>),
    Double(ConstSVMVal<f64>),
}

impl ConstBaseVal {
    pub fn get_size(&self) -> BaseTypeSize {
        match self {
            Self::Bool(v) => v.get_size(),
            Self::Int(v) => v.get_size(),
            Self::UInt(v) => v.get_size(),
            Self::Float(v) => v.get_size(),
            Self::Double(v) => v.get_size(),
        }
    }

    pub fn type_specifier(&self) -> TypeSpecifier {
        TypeSpecifier::new(self.type_specifier_non_array())
    }

    pub fn type_specifier_non_array(&self) -> TypeSpecifierNonArray {
        match self {
            Self::Bool(ConstSVVal::Scalar(_)) => TypeSpecifierNonArray::Bool,
            Self::Bool(ConstSVVal::Vec2(_)) => TypeSpecifierNonArray::BVec2,
            Self::Bool(ConstSVVal::Vec3(_)) => TypeSpecifierNonArray::BVec3,
            Self::Bool(ConstSVVal::Vec4(_)) => TypeSpecifierNonArray::BVec4,
            Self::Int(ConstSVVal::Scalar(_)) => TypeSpecifierNonArray::Int,
            Self::Int(ConstSVVal::Vec2(_)) => TypeSpecifierNonArray::IVec2,
            Self::Int(ConstSVVal::Vec3(_)) => TypeSpecifierNonArray::IVec3,
            Self::Int(ConstSVVal::Vec4(_)) => TypeSpecifierNonArray::IVec4,
            Self::UInt(ConstSVVal::Scalar(_)) => TypeSpecifierNonArray::UInt,
            Self::UInt(ConstSVVal::Vec2(_)) => TypeSpecifierNonArray::UVec2,
            Self::UInt(ConstSVVal::Vec3(_)) => TypeSpecifierNonArray::UVec3,
            Self::UInt(ConstSVVal::Vec4(_)) => TypeSpecifierNonArray::UVec4,
            Self::Float(ConstSVMVal::SV(ConstSVVal::Scalar(_))) => TypeSpecifierNonArray::Float,
            Self::Float(ConstSVMVal::SV(ConstSVVal::Vec2(_))) => TypeSpecifierNonArray::Vec2,
            Self::Float(ConstSVMVal::SV(ConstSVVal::Vec3(_))) => TypeSpecifierNonArray::Vec3,
            Self::Float(ConstSVMVal::SV(ConstSVVal::Vec4(_))) => TypeSpecifierNonArray::Vec4,
            Self::Float(ConstSVMVal::M(ConstMVal::Mat2(_))) => TypeSpecifierNonArray::Mat2,
            Self::Float(ConstSVMVal::M(ConstMVal::Mat23(_))) => TypeSpecifierNonArray::Mat23,
            Self::Float(ConstSVMVal::M(ConstMVal::Mat24(_))) => TypeSpecifierNonArray::Mat24,
            Self::Float(ConstSVMVal::M(ConstMVal::Mat32(_))) => TypeSpecifierNonArray::Mat32,
            Self::Float(ConstSVMVal::M(ConstMVal::Mat3(_))) => TypeSpecifierNonArray::Mat3,
            Self::Float(ConstSVMVal::M(ConstMVal::Mat34(_))) => TypeSpecifierNonArray::Mat34,
            Self::Float(ConstSVMVal::M(ConstMVal::Mat42(_))) => TypeSpecifierNonArray::Mat42,
            Self::Float(ConstSVMVal::M(ConstMVal::Mat43(_))) => TypeSpecifierNonArray::Mat43,
            Self::Float(ConstSVMVal::M(ConstMVal::Mat4(_))) => TypeSpecifierNonArray::Mat4,
            Self::Double(ConstSVMVal::SV(ConstSVVal::Scalar(_))) => TypeSpecifierNonArray::Double,
            Self::Double(ConstSVMVal::SV(ConstSVVal::Vec2(_))) => TypeSpecifierNonArray::DVec2,
            Self::Double(ConstSVMVal::SV(ConstSVVal::Vec3(_))) => TypeSpecifierNonArray::DVec3,
            Self::Double(ConstSVMVal::SV(ConstSVVal::Vec4(_))) => TypeSpecifierNonArray::DVec4,
            Self::Double(ConstSVMVal::M(ConstMVal::Mat2(_))) => TypeSpecifierNonArray::DMat2,
            Self::Double(ConstSVMVal::M(ConstMVal::Mat23(_))) => TypeSpecifierNonArray::DMat23,
            Self::Double(ConstSVMVal::M(ConstMVal::Mat24(_))) => TypeSpecifierNonArray::DMat24,
            Self::Double(ConstSVMVal::M(ConstMVal::Mat32(_))) => TypeSpecifierNonArray::DMat32,
            Self::Double(ConstSVMVal::M(ConstMVal::Mat3(_))) => TypeSpecifierNonArray::DMat3,
            Self::Double(ConstSVMVal::M(ConstMVal::Mat34(_))) => TypeSpecifierNonArray::DMat34,
            Self::Double(ConstSVMVal::M(ConstMVal::Mat42(_))) => TypeSpecifierNonArray::DMat42,
            Self::Double(ConstSVMVal::M(ConstMVal::Mat43(_))) => TypeSpecifierNonArray::DMat43,
            Self::Double(ConstSVMVal::M(ConstMVal::Mat4(_))) => TypeSpecifierNonArray::DMat4,
        }
    }
}

#[derive(Clone, PartialEq, Debug)]
pub struct ConstStruct {
    type_specifier: StructSpecifier,
    entries: HashMap<String, ConstVal>,
}

impl ConstStruct {
}

impl ConstLookup for ConstStruct {
    fn lookup_const(&self, ident: &Identifier) -> Option<&ConstVal> {
        self.entries.get(&ident.0)
    }
}

#[derive(Clone, PartialEq, Debug)]
pub enum ConstVal {
    Base(ConstBaseVal),
    Array(TypeSpecifierNonArray, Box<[ConstVal]>),
    Struct(ConstStruct),
}

impl ConstVal {
    pub fn type_specifier(&self) -> TypeSpecifier {
        match self {
            ConstVal::Base(b) => b.type_specifier(),
            ConstVal::Array(_, _) => todo!(),
            ConstVal::Struct(_) => todo!(),
        }
    }
}

#[derive(Clone, PartialEq, Debug)]
pub enum ConstEvalError {
    UnknownIdentifier(String),
    IllegalUnaryOp(UnaryOp),
    IllegalUnaryOperand(UnaryOp, TypeSpecifier),
    IllegalBinaryOp(BinaryOp),
    IllegalBinaryOperand(BinaryOp, TypeSpecifier, TypeSpecifier),
}

mod function {
    use std::any::TypeId;
    use std::cmp::Ordering;
    use std::ops::Mul;
    use lazy_static::lazy_static;
    use nalgebra::{Matrix2, Matrix2x3, Matrix2x4, Matrix3, Matrix3x2, Matrix3x4, Matrix4, Matrix4x2, Matrix4x3, Scalar, Vector2, Vector3, Vector4};
    use crate::renderer::emulator::glsl::const_eval::{BaseTypeSize, ConstBaseVal, ConstMVal, ConstSVMVal, ConstSVVal};

    #[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Debug)]
    pub enum ParameterBaseType {
        Bool,
        Int,
        UInt,
        Float,
        Double,
    }

    impl ParameterBaseType {
        pub fn from_const_val(val: &ConstBaseVal) -> Self {
            match val {
                ConstBaseVal::Bool(_) => Self::Bool,
                ConstBaseVal::Int(_) => Self::Int,
                ConstBaseVal::UInt(_) => Self::UInt,
                ConstBaseVal::Float(_) => Self::Float,
                ConstBaseVal::Double(_) => Self::Double,
            }
        }

        /// Ordered by glsl implicit casting rules. If a < b then a can be implicitly cast to b.
        pub fn cast_cmp(&self, other: &Self) -> Option<Ordering> {
            if self == other {
                Some(Ordering::Equal)
            } else {
                match (self, other) {
                    (Self::Int, Self::UInt) |
                    (Self::Int, Self::Float) |
                    (Self::Int, Self::Double) |
                    (Self::UInt, Self::Float) |
                    (Self::UInt, Self::Double) |
                    (Self::Float, Self::Double) => Some(Ordering::Less),
                    (Self::UInt, Self::Int) |
                    (Self::Float, Self::Int) |
                    (Self::Double, Self::Int) |
                    (Self::Float, Self::UInt) |
                    (Self::Double, Self::UInt) |
                    (Self::Double, Self::Float) => Some(Ordering::Greater),
                    _ => None,
                }
            }
        }

        pub fn can_cast_into(&self, other: &Self) -> bool {
            match self.cast_cmp(other) {
                Some(Ordering::Less) |
                Some(Ordering::Equal) => true,
                _ => false
            }
        }
    }

    #[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Debug)]
    pub enum ParameterSize {
        Scalar,
        Vec2,
        Vec3,
        Vec4,
        GenSVec,
        Mat2,
        Mat23,
        Mat24,
        Mat32,
        Mat3,
        Mat34,
        Mat42,
        Mat43,
        Mat4,
        GenMat,
    }

    impl ParameterSize {
        pub fn matches(&self, val: BaseTypeSize) -> bool {
            match (val, self) {
                (BaseTypeSize::Scalar, Self::Scalar) |
                (BaseTypeSize::Vec2, Self::Vec2) |
                (BaseTypeSize::Vec3, Self::Vec3) |
                (BaseTypeSize::Vec4, Self::Vec4) |
                (BaseTypeSize::Mat2, Self::Mat2) |
                (BaseTypeSize::Mat23, Self::Mat23) |
                (BaseTypeSize::Mat24, Self::Mat24) |
                (BaseTypeSize::Mat32, Self::Mat32) |
                (BaseTypeSize::Mat3, Self::Mat3) |
                (BaseTypeSize::Mat34, Self::Mat34) |
                (BaseTypeSize::Mat42, Self::Mat42) |
                (BaseTypeSize::Mat43, Self::Mat43) |
                (BaseTypeSize::Mat4, Self::Mat4) |
                (BaseTypeSize::Scalar, Self::GenSVec) |
                (BaseTypeSize::Vec2, Self::GenSVec) |
                (BaseTypeSize::Vec3, Self::GenSVec) |
                (BaseTypeSize::Vec4, Self::GenSVec) |
                (BaseTypeSize::Mat2, Self::GenMat) |
                (BaseTypeSize::Mat23, Self::GenMat) |
                (BaseTypeSize::Mat24, Self::GenMat) |
                (BaseTypeSize::Mat32, Self::GenMat) |
                (BaseTypeSize::Mat3, Self::GenMat) |
                (BaseTypeSize::Mat34, Self::GenMat) |
                (BaseTypeSize::Mat42, Self::GenMat) |
                (BaseTypeSize::Mat43, Self::GenMat) |
                (BaseTypeSize::Mat4, Self::GenMat) => true,
                _ => false,
            }
        }
    }

    #[derive(Copy, Clone, Eq, PartialEq, Debug)]
    pub struct ParameterType {
        base_type: ParameterBaseType,
        size: ParameterSize,
    }

    impl ParameterType {
        pub fn new(base_type: ParameterBaseType, size: ParameterSize) -> Self {
            Self {
                base_type,
                size
            }
        }

        /// Compare function of the glsl implicit casting rules. For any a and b where a can be
        /// implicitly cast to b, a < b.
        ///
        /// Note that a < b does not imply that a can be cast to b. Assume we have a = (Int, Vec2),
        /// b = (UInt, Vec2), c = (bool, Vec2). Then all of the following orderings are valid:
        /// \[c, a, b], \[a, c, b] and \[a, b, c], since c cannot be cast to a or b and vice versa.
        /// It is only guaranteed that the order of c is consistent at runtime.
        pub fn cast_cmp(&self, other: &Self) -> Ordering {
            let size_ord = self.size.cmp(&other.size);
            if size_ord == Ordering::Equal {
                self.base_type.partial_cmp(&other.base_type).unwrap_or_else(||
                    self.base_type.cmp(&other.base_type)
                )
            } else {
                size_ord
            }
        }
    }

    struct Overload {
        prototype: Box<[ParameterType]>,
        function: Box<dyn Fn(&[&ConstBaseVal]) -> Option<ConstBaseVal> + Send + Sync>,
    }

    impl Overload {
        fn from_fn_0<R, F>(f: F) -> Self where R: ConstParameter, F: Fn() -> R + Send + Sync + 'static {
            let prototype = Box::new([]);
            let function = Box::new(move |params: &[&ConstBaseVal]| {
                if params.len() != 0 {
                    panic!("Parameter list length mismatch. Expected 0 but got {:?}", params.len());
                } else {
                    Some(f().to_val())
                }
            });

            Self {
                prototype,
                function
            }
        }

        fn from_fn_1<R, T0, F>(f: F) -> Self where R: ConstParameter, T0: ConstParameter + 'static, F: Fn(T0) -> Option<R> + Send + Sync + 'static {
            let prototype = Box::new([T0::get_type()]);
            let function = Box::new(move |params: &[&ConstBaseVal]| {
                if params.len() != 1 {
                    panic!("Parameter list length mismatch. Expected 1 but got {:?}", params.len());
                } else {
                    let t0 = T0::from_val(params[0]).unwrap_or_else(|| panic!("Implicit cast failed: {:?} to {:?}", params[0].type_specifier(), TypeId::of::<T0>()));
                    f(t0).map(R::to_val)
                }
            });

            Self {
                prototype,
                function
            }
        }

        fn from_fn_2<R, T0, T1, F>(f: F) -> Self where R: ConstParameter, T0: ConstParameter + 'static, T1: ConstParameter + 'static, F: Fn(T0, T1) -> Option<R> + Send + Sync + 'static {
            let prototype = Box::new([T0::get_type(), T1::get_type()]);
            let function = Box::new(move |params: &[&ConstBaseVal]| {
                if params.len() != 2 {
                    panic!("Parameter list length mismatch. Expected 2 but got {:?}", params.len());
                } else {
                    let t0 = T0::from_val(params[0]).unwrap_or_else(|| panic!("Implicit cast failed: {:?} to {:?}", params[0].type_specifier(), TypeId::of::<T0>()));
                    let t1 = T1::from_val(params[1]).unwrap_or_else(|| panic!("Implicit cast failed: {:?} to {:?}", params[1].type_specifier(), TypeId::of::<T0>()));
                    f(t0, t1).map(R::to_val)
                }
            });

            Self {
                prototype,
                function
            }
        }

        fn compatible_with(&self, params: &[(BaseTypeSize, ParameterBaseType)]) -> bool {
            if params.len() != self.prototype.len() {
                return false;
            }

            for ((size, base_type), proto) in params.iter().zip(self.prototype.iter()) {
                if !proto.size.matches(*size) {
                    return false;
                }
                if !base_type.can_cast_into(&proto.base_type) {
                    return false;
                }
            }
            true
        }

        /// Evaluates this overload on the provided parameters performing implicit casting if
        /// necessary.
        ///
        /// # Panics
        /// If the provided parameters cannot be implicitly cast to the required type. Check
        /// compatibility with [Overload::compatible_with] first if needed.
        fn eval(&self, params: &[&ConstBaseVal]) -> Option<ConstBaseVal> {
            (self.function)(params)
        }

        fn cast_cmp(&self, other: &Self) -> Ordering {
            let len_cmp = self.prototype.len().cmp(&other.prototype.len());
            if len_cmp == Ordering::Equal {
                self.prototype.iter().zip(other.prototype.iter()).fold(Ordering::Equal, |i, (a, b)| {
                    if i == Ordering::Equal {
                        a.cast_cmp(b)
                    } else {
                        i
                    }
                })
            } else {
                len_cmp
            }
        }
    }

    pub struct ConstEvalFunction {
        overloads: Vec<Overload>,
    }

    impl ConstEvalFunction {
        pub fn new() -> Self {
            Self {
                overloads: Vec::new(),
            }
        }

        fn add_overload(&mut self, overload: Overload) {
            self.overloads.push(overload);
            self.overloads.sort_by(Overload::cast_cmp)
        }

        /// Adds an overload to this function taking no parameters.
        pub fn add_overload_0<R, F>(&mut self, f: F) where R: ConstParameter, F: Fn() -> R + Send + Sync + 'static {
            self.add_overload(Overload::from_fn_0(f))
        }

        /// Adds an overload to this function taking 1 parameter.
        ///
        /// If the provided function returns [`None`] when evaluated it is not interpreted as an
        /// error but indicates that the parameters do not match the function prototype (for example
        /// when using generic sized vectors/matrices). The [ConstEvalFunction::eval] method will
        /// not immediately return but continue searching for a matching overload if a function
        /// returns [`None`].
        pub fn add_overload_1<R, T0, F>(&mut self, f: F) where R: ConstParameter, T0: ConstParameter + 'static, F: Fn(T0) -> Option<R> + Send + Sync + 'static {
            self.add_overload(Overload::from_fn_1(f))
        }

        /// Adds an overload to this function taking 2 parameter.
        ///
        /// If the provided function returns [`None`] when evaluated it is not interpreted as an
        /// error but indicates that the parameters do not match the function prototype (for example
        /// when using generic sized vectors/matrices). The [ConstEvalFunction::eval] method will
        /// not immediately return but continue searching for a matching overload if a function
        /// returns [`None`].
        pub fn add_overload_2<R, T0, T1, F>(&mut self, f: F) where R: ConstParameter, T0: ConstParameter + 'static, T1: ConstParameter + 'static, F: Fn(T0, T1) -> Option<R> + Send + Sync + 'static {
            self.add_overload(Overload::from_fn_2(f))
        }

        /// Evaluates the function on the provided parameters. Returns [`None`] if no matching
        /// overload could be found.
        pub fn eval(&self, params: &[&ConstBaseVal]) -> Option<ConstBaseVal> {
            let mut types = Vec::with_capacity(params.len());
            for param in params {
                types.push((param.get_size(), ParameterBaseType::from_const_val(param)));
            }

            for func in &self.overloads {
                if func.compatible_with(&types) {
                    if let Some(result) = func.eval(params) {
                        return Some(result);
                    }
                }
            };
            None
        }
    }

    pub trait ConstParameter: Sized {
        fn get_type() -> ParameterType;

        fn from_val(val: &ConstBaseVal) -> Option<Self>;

        fn to_val(self) -> ConstBaseVal;
    }

    impl ConstParameter for bool {
        fn get_type() -> ParameterType {
            ParameterType::new(ParameterBaseType::Bool, ParameterSize::Scalar)
        }

        fn from_val(val: &ConstBaseVal) -> Option<Self> {
            match val {
                ConstBaseVal::Bool(ConstSVVal::Scalar(v)) => Some(*v),
                _ => None,
            }
        }

        fn to_val(self) -> ConstBaseVal {
            ConstBaseVal::Bool(ConstSVVal::Scalar(self))
        }
    }

    impl ConstParameter for Vector2<bool> {
        fn get_type() -> ParameterType {
            ParameterType::new(ParameterBaseType::Bool, ParameterSize::Vec2)
        }

        fn from_val(val: &ConstBaseVal) -> Option<Self> {
            match val {
                ConstBaseVal::Bool(ConstSVVal::Vec2(v)) => Some(*v),
                _ => None,
            }
        }

        fn to_val(self) -> ConstBaseVal {
            ConstBaseVal::Bool(ConstSVVal::Vec2(self))
        }
    }

    impl ConstParameter for Vector3<bool> {
        fn get_type() -> ParameterType {
            ParameterType::new(ParameterBaseType::Bool, ParameterSize::Vec3)
        }

        fn from_val(val: &ConstBaseVal) -> Option<Self> {
            match val {
                ConstBaseVal::Bool(ConstSVVal::Vec3(v)) => Some(*v),
                _ => None,
            }
        }

        fn to_val(self) -> ConstBaseVal {
            ConstBaseVal::Bool(ConstSVVal::Vec3(self))
        }
    }

    impl ConstParameter for Vector4<bool> {
        fn get_type() -> ParameterType {
            ParameterType::new(ParameterBaseType::Bool, ParameterSize::Vec4)
        }

        fn from_val(val: &ConstBaseVal) -> Option<Self> {
            match val {
                ConstBaseVal::Bool(ConstSVVal::Vec4(v)) => Some(*v),
                _ => None,
            }
        }

        fn to_val(self) -> ConstBaseVal {
            ConstBaseVal::Bool(ConstSVVal::Vec4(self))
        }
    }

    impl ConstParameter for ConstSVVal<bool> {
        fn get_type() -> ParameterType {
            ParameterType::new(ParameterBaseType::Bool, ParameterSize::GenSVec)
        }

        fn from_val(val: &ConstBaseVal) -> Option<Self> {
            match val {
                ConstBaseVal::Bool(v) => Some(v.clone()),
                _ => None,
            }
        }

        fn to_val(self) -> ConstBaseVal {
            ConstBaseVal::Bool(self)
        }
    }

    impl ConstParameter for i32 {
        fn get_type() -> ParameterType {
            ParameterType::new(ParameterBaseType::Int, ParameterSize::Scalar)
        }

        fn from_val(val: &ConstBaseVal) -> Option<Self> {
            match val {
                ConstBaseVal::Int(ConstSVVal::Scalar(v)) => Some(*v),
                _ => None,
            }
        }

        fn to_val(self) -> ConstBaseVal {
            ConstBaseVal::Int(ConstSVVal::Scalar(self))
        }
    }

    impl ConstParameter for Vector2<i32> {
        fn get_type() -> ParameterType {
            ParameterType::new(ParameterBaseType::Int, ParameterSize::Vec2)
        }

        fn from_val(val: &ConstBaseVal) -> Option<Self> {
            match val {
                ConstBaseVal::Int(ConstSVVal::Vec2(v)) => Some(*v),
                _ => None,
            }
        }

        fn to_val(self) -> ConstBaseVal {
            ConstBaseVal::Int(ConstSVVal::Vec2(self))
        }
    }

    impl ConstParameter for Vector3<i32> {
        fn get_type() -> ParameterType {
            ParameterType::new(ParameterBaseType::Int, ParameterSize::Vec3)
        }

        fn from_val(val: &ConstBaseVal) -> Option<Self> {
            match val {
                ConstBaseVal::Int(ConstSVVal::Vec3(v)) => Some(*v),
                _ => None,
            }
        }

        fn to_val(self) -> ConstBaseVal {
            ConstBaseVal::Int(ConstSVVal::Vec3(self))
        }
    }

    impl ConstParameter for Vector4<i32> {
        fn get_type() -> ParameterType {
            ParameterType::new(ParameterBaseType::Int, ParameterSize::Vec4)
        }

        fn from_val(val: &ConstBaseVal) -> Option<Self> {
            match val {
                ConstBaseVal::Int(ConstSVVal::Vec4(v)) => Some(*v),
                _ => None,
            }
        }

        fn to_val(self) -> ConstBaseVal {
            ConstBaseVal::Int(ConstSVVal::Vec4(self))
        }
    }

    impl ConstParameter for ConstSVVal<i32> {
        fn get_type() -> ParameterType {
            ParameterType::new(ParameterBaseType::Int, ParameterSize::GenSVec)
        }

        fn from_val(val: &ConstBaseVal) -> Option<Self> {
            match val {
                ConstBaseVal::Int(v) => Some(v.clone()),
                _ => None,
            }
        }

        fn to_val(self) -> ConstBaseVal {
            ConstBaseVal::Int(self)
        }
    }

    impl ConstParameter for u32 {
        fn get_type() -> ParameterType {
            ParameterType::new(ParameterBaseType::UInt, ParameterSize::Scalar)
        }

        fn from_val(val: &ConstBaseVal) -> Option<Self> {
            match val {
                ConstBaseVal::Int(ConstSVVal::Scalar(v)) => Some(*v as u32),
                ConstBaseVal::UInt(ConstSVVal::Scalar(v)) => Some(*v),
                _ => None,
            }
        }

        fn to_val(self) -> ConstBaseVal {
            ConstBaseVal::UInt(ConstSVVal::Scalar(self))
        }
    }

    impl ConstParameter for Vector2<u32> {
        fn get_type() -> ParameterType {
            ParameterType::new(ParameterBaseType::UInt, ParameterSize::Vec2)
        }

        fn from_val(val: &ConstBaseVal) -> Option<Self> {
            match val {
                ConstBaseVal::Int(ConstSVVal::Vec2(v)) => Some(v.map(|v| v as u32)),
                ConstBaseVal::UInt(ConstSVVal::Vec2(v)) => Some(*v),
                _ => None,
            }
        }

        fn to_val(self) -> ConstBaseVal {
            ConstBaseVal::UInt(ConstSVVal::Vec2(self))
        }
    }

    impl ConstParameter for Vector3<u32> {
        fn get_type() -> ParameterType {
            ParameterType::new(ParameterBaseType::UInt, ParameterSize::Vec3)
        }

        fn from_val(val: &ConstBaseVal) -> Option<Self> {
            match val {
                ConstBaseVal::Int(ConstSVVal::Vec3(v)) => Some(v.map(|v| v as u32)),
                ConstBaseVal::UInt(ConstSVVal::Vec3(v)) => Some(*v),
                _ => None,
            }
        }

        fn to_val(self) -> ConstBaseVal {
            ConstBaseVal::UInt(ConstSVVal::Vec3(self))
        }
    }

    impl ConstParameter for Vector4<u32> {
        fn get_type() -> ParameterType {
            ParameterType::new(ParameterBaseType::UInt, ParameterSize::Vec4)
        }

        fn from_val(val: &ConstBaseVal) -> Option<Self> {
            match val {
                ConstBaseVal::Int(ConstSVVal::Vec4(v)) => Some(v.map(|v| v as u32)),
                ConstBaseVal::UInt(ConstSVVal::Vec4(v)) => Some(*v),
                _ => None,
            }
        }

        fn to_val(self) -> ConstBaseVal {
            ConstBaseVal::UInt(ConstSVVal::Vec4(self))
        }
    }

    impl ConstParameter for ConstSVVal<u32> {
        fn get_type() -> ParameterType {
            ParameterType::new(ParameterBaseType::UInt, ParameterSize::GenSVec)
        }

        fn from_val(val: &ConstBaseVal) -> Option<Self> {
            match val {
                ConstBaseVal::Int(v) => Some(v.map(|v| v as u32)),
                ConstBaseVal::UInt(v) => Some(v.clone()),
                _ => None,
            }
        }

        fn to_val(self) -> ConstBaseVal {
            ConstBaseVal::UInt(self)
        }
    }

    impl ConstParameter for f32 {
        fn get_type() -> ParameterType {
            ParameterType::new(ParameterBaseType::Float, ParameterSize::Scalar)
        }

        fn from_val(val: &ConstBaseVal) -> Option<Self> {
            match val {
                ConstBaseVal::Int(ConstSVVal::Scalar(v)) => Some(*v as f32),
                ConstBaseVal::UInt(ConstSVVal::Scalar(v)) => Some(*v as f32),
                ConstBaseVal::Float(ConstSVMVal::SV(ConstSVVal::Scalar(v))) => Some(*v),
                _ => None,
            }
        }

        fn to_val(self) -> ConstBaseVal {
            ConstBaseVal::Float(ConstSVMVal::SV(ConstSVVal::Scalar(self)))
        }
    }

    impl ConstParameter for Vector2<f32> {
        fn get_type() -> ParameterType {
            ParameterType::new(ParameterBaseType::Float, ParameterSize::Vec2)
        }

        fn from_val(val: &ConstBaseVal) -> Option<Self> {
            match val {
                ConstBaseVal::Int(ConstSVVal::Vec2(v)) => Some(v.map(|v| v as f32)),
                ConstBaseVal::UInt(ConstSVVal::Vec2(v)) => Some(v.map(|v| v as f32)),
                ConstBaseVal::Float(ConstSVMVal::SV(ConstSVVal::Vec2(v))) => Some(*v),
                _ => None,
            }
        }

        fn to_val(self) -> ConstBaseVal {
            ConstBaseVal::Float(ConstSVMVal::SV(ConstSVVal::Vec2(self)))
        }
    }

    impl ConstParameter for Vector3<f32> {
        fn get_type() -> ParameterType {
            ParameterType::new(ParameterBaseType::Float, ParameterSize::Vec3)
        }

        fn from_val(val: &ConstBaseVal) -> Option<Self> {
            match val {
                ConstBaseVal::Int(ConstSVVal::Vec3(v)) => Some(v.map(|v| v as f32)),
                ConstBaseVal::UInt(ConstSVVal::Vec3(v)) => Some(v.map(|v| v as f32)),
                ConstBaseVal::Float(ConstSVMVal::SV(ConstSVVal::Vec3(v))) => Some(*v),
                _ => None,
            }
        }

        fn to_val(self) -> ConstBaseVal {
            ConstBaseVal::Float(ConstSVMVal::SV(ConstSVVal::Vec3(self)))
        }
    }

    impl ConstParameter for Vector4<f32> {
        fn get_type() -> ParameterType {
            ParameterType::new(ParameterBaseType::Float, ParameterSize::Vec4)
        }

        fn from_val(val: &ConstBaseVal) -> Option<Self> {
            match val {
                ConstBaseVal::Int(ConstSVVal::Vec4(v)) => Some(v.map(|v| v as f32)),
                ConstBaseVal::UInt(ConstSVVal::Vec4(v)) => Some(v.map(|v| v as f32)),
                ConstBaseVal::Float(ConstSVMVal::SV(ConstSVVal::Vec4(v))) => Some(*v),
                _ => None,
            }
        }

        fn to_val(self) -> ConstBaseVal {
            ConstBaseVal::Float(ConstSVMVal::SV(ConstSVVal::Vec4(self)))
        }
    }

    impl ConstParameter for ConstSVVal<f32> {
        fn get_type() -> ParameterType {
            ParameterType::new(ParameterBaseType::Float, ParameterSize::GenSVec)
        }

        fn from_val(val: &ConstBaseVal) -> Option<Self> {
            match val {
                ConstBaseVal::Int(v) => Some(v.map(|v| v as f32)),
                ConstBaseVal::UInt(v) => Some(v.map(|v| v as f32)),
                ConstBaseVal::Float(ConstSVMVal::SV(v)) => Some(v.clone()),
                _ => None,
            }
        }

        fn to_val(self) -> ConstBaseVal {
            ConstBaseVal::Float(ConstSVMVal::SV(self))
        }
    }

    impl ConstParameter for Matrix2<f32> {
        fn get_type() -> ParameterType {
            ParameterType::new(ParameterBaseType::Float, ParameterSize::Mat2)
        }

        fn from_val(val: &ConstBaseVal) -> Option<Self> {
            match val {
                ConstBaseVal::Float(ConstSVMVal::M(ConstMVal::Mat2(v))) => Some(*v),
                _ => None,
            }
        }

        fn to_val(self) -> ConstBaseVal {
            ConstBaseVal::Float(ConstSVMVal::M(ConstMVal::Mat2(self)))
        }
    }

    impl ConstParameter for Matrix2x3<f32> {
        fn get_type() -> ParameterType {
            ParameterType::new(ParameterBaseType::Float, ParameterSize::Mat23)
        }

        fn from_val(val: &ConstBaseVal) -> Option<Self> {
            match val {
                ConstBaseVal::Float(ConstSVMVal::M(ConstMVal::Mat23(v))) => Some(*v),
                _ => None,
            }
        }

        fn to_val(self) -> ConstBaseVal {
            ConstBaseVal::Float(ConstSVMVal::M(ConstMVal::Mat23(self)))
        }
    }

    impl ConstParameter for Matrix2x4<f32> {
        fn get_type() -> ParameterType {
            ParameterType::new(ParameterBaseType::Float, ParameterSize::Mat24)
        }

        fn from_val(val: &ConstBaseVal) -> Option<Self> {
            match val {
                ConstBaseVal::Float(ConstSVMVal::M(ConstMVal::Mat24(v))) => Some(*v),
                _ => None,
            }
        }

        fn to_val(self) -> ConstBaseVal {
            ConstBaseVal::Float(ConstSVMVal::M(ConstMVal::Mat24(self)))
        }
    }

    impl ConstParameter for Matrix3x2<f32> {
        fn get_type() -> ParameterType {
            ParameterType::new(ParameterBaseType::Float, ParameterSize::Mat32)
        }

        fn from_val(val: &ConstBaseVal) -> Option<Self> {
            match val {
                ConstBaseVal::Float(ConstSVMVal::M(ConstMVal::Mat32(v))) => Some(*v),
                _ => None,
            }
        }

        fn to_val(self) -> ConstBaseVal {
            ConstBaseVal::Float(ConstSVMVal::M(ConstMVal::Mat32(self)))
        }
    }

    impl ConstParameter for Matrix3<f32> {
        fn get_type() -> ParameterType {
            ParameterType::new(ParameterBaseType::Float, ParameterSize::Mat3)
        }

        fn from_val(val: &ConstBaseVal) -> Option<Self> {
            match val {
                ConstBaseVal::Float(ConstSVMVal::M(ConstMVal::Mat3(v))) => Some(*v),
                _ => None,
            }
        }

        fn to_val(self) -> ConstBaseVal {
            ConstBaseVal::Float(ConstSVMVal::M(ConstMVal::Mat3(self)))
        }
    }

    impl ConstParameter for Matrix3x4<f32> {
        fn get_type() -> ParameterType {
            ParameterType::new(ParameterBaseType::Float, ParameterSize::Mat34)
        }

        fn from_val(val: &ConstBaseVal) -> Option<Self> {
            match val {
                ConstBaseVal::Float(ConstSVMVal::M(ConstMVal::Mat34(v))) => Some(*v),
                _ => None,
            }
        }

        fn to_val(self) -> ConstBaseVal {
            ConstBaseVal::Float(ConstSVMVal::M(ConstMVal::Mat34(self)))
        }
    }

    impl ConstParameter for Matrix4x2<f32> {
        fn get_type() -> ParameterType {
            ParameterType::new(ParameterBaseType::Float, ParameterSize::Mat42)
        }

        fn from_val(val: &ConstBaseVal) -> Option<Self> {
            match val {
                ConstBaseVal::Float(ConstSVMVal::M(ConstMVal::Mat42(v))) => Some(*v),
                _ => None,
            }
        }

        fn to_val(self) -> ConstBaseVal {
            ConstBaseVal::Float(ConstSVMVal::M(ConstMVal::Mat42(self)))
        }
    }

    impl ConstParameter for Matrix4x3<f32> {
        fn get_type() -> ParameterType {
            ParameterType::new(ParameterBaseType::Float, ParameterSize::Mat43)
        }

        fn from_val(val: &ConstBaseVal) -> Option<Self> {
            match val {
                ConstBaseVal::Float(ConstSVMVal::M(ConstMVal::Mat43(v))) => Some(*v),
                _ => None,
            }
        }

        fn to_val(self) -> ConstBaseVal {
            ConstBaseVal::Float(ConstSVMVal::M(ConstMVal::Mat43(self)))
        }
    }

    impl ConstParameter for Matrix4<f32> {
        fn get_type() -> ParameterType {
            ParameterType::new(ParameterBaseType::Float, ParameterSize::Mat4)
        }

        fn from_val(val: &ConstBaseVal) -> Option<Self> {
            match val {
                ConstBaseVal::Float(ConstSVMVal::M(ConstMVal::Mat4(v))) => Some(*v),
                _ => None,
            }
        }

        fn to_val(self) -> ConstBaseVal {
            ConstBaseVal::Float(ConstSVMVal::M(ConstMVal::Mat4(self)))
        }
    }

    impl ConstParameter for ConstMVal<f32> {
        fn get_type() -> ParameterType {
            ParameterType::new(ParameterBaseType::Float, ParameterSize::GenMat)
        }

        fn from_val(val: &ConstBaseVal) -> Option<Self> {
            match val {
                ConstBaseVal::Float(ConstSVMVal::M(v)) => Some(v.clone()),
                _ => None,
            }
        }

        fn to_val(self) -> ConstBaseVal {
            ConstBaseVal::Float(ConstSVMVal::M(self))
        }
    }

    impl ConstParameter for f64 {
        fn get_type() -> ParameterType {
            ParameterType::new(ParameterBaseType::Double, ParameterSize::Scalar)
        }

        fn from_val(val: &ConstBaseVal) -> Option<Self> {
            match val {
                ConstBaseVal::Int(ConstSVVal::Scalar(v)) => Some(*v as f64),
                ConstBaseVal::UInt(ConstSVVal::Scalar(v)) => Some(*v as f64),
                ConstBaseVal::Float(ConstSVMVal::SV(ConstSVVal::Scalar(v))) => Some(*v as f64),
                ConstBaseVal::Double(ConstSVMVal::SV(ConstSVVal::Scalar(v))) => Some(*v),
                _ => None,
            }
        }

        fn to_val(self) -> ConstBaseVal {
            ConstBaseVal::Double(ConstSVMVal::SV(ConstSVVal::Scalar(self)))
        }
    }

    impl ConstParameter for Vector2<f64> {
        fn get_type() -> ParameterType {
            ParameterType::new(ParameterBaseType::Double, ParameterSize::Vec2)
        }

        fn from_val(val: &ConstBaseVal) -> Option<Self> {
            match val {
                ConstBaseVal::Int(ConstSVVal::Vec2(v)) => Some(v.map(|v| v as f64)),
                ConstBaseVal::UInt(ConstSVVal::Vec2(v)) => Some(v.map(|v| v as f64)),
                ConstBaseVal::Float(ConstSVMVal::SV(ConstSVVal::Vec2(v))) => Some(v.map(|v| v as f64)),
                ConstBaseVal::Double(ConstSVMVal::SV(ConstSVVal::Vec2(v))) => Some(*v),
                _ => None,
            }
        }

        fn to_val(self) -> ConstBaseVal {
            ConstBaseVal::Double(ConstSVMVal::SV(ConstSVVal::Vec2(self)))
        }
    }

    impl ConstParameter for Vector3<f64> {
        fn get_type() -> ParameterType {
            ParameterType::new(ParameterBaseType::Double, ParameterSize::Vec3)
        }

        fn from_val(val: &ConstBaseVal) -> Option<Self> {
            match val {
                ConstBaseVal::Int(ConstSVVal::Vec3(v)) => Some(v.map(|v| v as f64)),
                ConstBaseVal::UInt(ConstSVVal::Vec3(v)) => Some(v.map(|v| v as f64)),
                ConstBaseVal::Float(ConstSVMVal::SV(ConstSVVal::Vec3(v))) => Some(v.map(|v| v as f64)),
                ConstBaseVal::Double(ConstSVMVal::SV(ConstSVVal::Vec3(v))) => Some(*v),
                _ => None,
            }
        }

        fn to_val(self) -> ConstBaseVal {
            ConstBaseVal::Double(ConstSVMVal::SV(ConstSVVal::Vec3(self)))
        }
    }

    impl ConstParameter for Vector4<f64> {
        fn get_type() -> ParameterType {
            ParameterType::new(ParameterBaseType::Double, ParameterSize::Vec4)
        }

        fn from_val(val: &ConstBaseVal) -> Option<Self> {
            match val {
                ConstBaseVal::Int(ConstSVVal::Vec4(v)) => Some(v.map(|v| v as f64)),
                ConstBaseVal::UInt(ConstSVVal::Vec4(v)) => Some(v.map(|v| v as f64)),
                ConstBaseVal::Float(ConstSVMVal::SV(ConstSVVal::Vec4(v))) => Some(v.map(|v| v as f64)),
                ConstBaseVal::Double(ConstSVMVal::SV(ConstSVVal::Vec4(v))) => Some(*v),
                _ => None,
            }
        }

        fn to_val(self) -> ConstBaseVal {
            ConstBaseVal::Double(ConstSVMVal::SV(ConstSVVal::Vec4(self)))
        }
    }

    impl ConstParameter for ConstSVVal<f64> {
        fn get_type() -> ParameterType {
            ParameterType::new(ParameterBaseType::Double, ParameterSize::GenSVec)
        }

        fn from_val(val: &ConstBaseVal) -> Option<Self> {
            match val {
                ConstBaseVal::Int(v) => Some(v.map(|v| v as f64)),
                ConstBaseVal::UInt(v) => Some(v.map(|v| v as f64)),
                ConstBaseVal::Float(ConstSVMVal::SV(v)) => Some(v.map(|v| v as f64)),
                ConstBaseVal::Double(ConstSVMVal::SV(v)) => Some(v.clone()),
                _ => None,
            }
        }

        fn to_val(self) -> ConstBaseVal {
            ConstBaseVal::Double(ConstSVMVal::SV(self))
        }
    }

    impl ConstParameter for Matrix2<f64> {
        fn get_type() -> ParameterType {
            ParameterType::new(ParameterBaseType::Double, ParameterSize::Mat2)
        }

        fn from_val(val: &ConstBaseVal) -> Option<Self> {
            match val {
                ConstBaseVal::Float(ConstSVMVal::M(ConstMVal::Mat2(v))) => Some(v.map(|v| v as f64)),
                ConstBaseVal::Double(ConstSVMVal::M(ConstMVal::Mat2(v))) => Some(*v),
                _ => None,
            }
        }

        fn to_val(self) -> ConstBaseVal {
            ConstBaseVal::Double(ConstSVMVal::M(ConstMVal::Mat2(self)))
        }
    }

    impl ConstParameter for Matrix2x3<f64> {
        fn get_type() -> ParameterType {
            ParameterType::new(ParameterBaseType::Double, ParameterSize::Mat23)
        }

        fn from_val(val: &ConstBaseVal) -> Option<Self> {
            match val {
                ConstBaseVal::Float(ConstSVMVal::M(ConstMVal::Mat23(v))) => Some(v.map(|v| v as f64)),
                ConstBaseVal::Double(ConstSVMVal::M(ConstMVal::Mat23(v))) => Some(*v),
                _ => None,
            }
        }

        fn to_val(self) -> ConstBaseVal {
            ConstBaseVal::Double(ConstSVMVal::M(ConstMVal::Mat23(self)))
        }
    }

    impl ConstParameter for Matrix2x4<f64> {
        fn get_type() -> ParameterType {
            ParameterType::new(ParameterBaseType::Double, ParameterSize::Mat24)
        }

        fn from_val(val: &ConstBaseVal) -> Option<Self> {
            match val {
                ConstBaseVal::Float(ConstSVMVal::M(ConstMVal::Mat24(v))) => Some(v.map(|v| v as f64)),
                ConstBaseVal::Double(ConstSVMVal::M(ConstMVal::Mat24(v))) => Some(*v),
                _ => None,
            }
        }

        fn to_val(self) -> ConstBaseVal {
            ConstBaseVal::Double(ConstSVMVal::M(ConstMVal::Mat24(self)))
        }
    }

    impl ConstParameter for Matrix3x2<f64> {
        fn get_type() -> ParameterType {
            ParameterType::new(ParameterBaseType::Double, ParameterSize::Mat32)
        }

        fn from_val(val: &ConstBaseVal) -> Option<Self> {
            match val {
                ConstBaseVal::Float(ConstSVMVal::M(ConstMVal::Mat32(v))) => Some(v.map(|v| v as f64)),
                ConstBaseVal::Double(ConstSVMVal::M(ConstMVal::Mat32(v))) => Some(*v),
                _ => None,
            }
        }

        fn to_val(self) -> ConstBaseVal {
            ConstBaseVal::Double(ConstSVMVal::M(ConstMVal::Mat32(self)))
        }
    }

    impl ConstParameter for Matrix3<f64> {
        fn get_type() -> ParameterType {
            ParameterType::new(ParameterBaseType::Double, ParameterSize::Mat3)
        }

        fn from_val(val: &ConstBaseVal) -> Option<Self> {
            match val {
                ConstBaseVal::Float(ConstSVMVal::M(ConstMVal::Mat3(v))) => Some(v.map(|v| v as f64)),
                ConstBaseVal::Double(ConstSVMVal::M(ConstMVal::Mat3(v))) => Some(*v),
                _ => None,
            }
        }

        fn to_val(self) -> ConstBaseVal {
            ConstBaseVal::Double(ConstSVMVal::M(ConstMVal::Mat3(self)))
        }
    }

    impl ConstParameter for Matrix3x4<f64> {
        fn get_type() -> ParameterType {
            ParameterType::new(ParameterBaseType::Double, ParameterSize::Mat34)
        }

        fn from_val(val: &ConstBaseVal) -> Option<Self> {
            match val {
                ConstBaseVal::Float(ConstSVMVal::M(ConstMVal::Mat34(v))) => Some(v.map(|v| v as f64)),
                ConstBaseVal::Double(ConstSVMVal::M(ConstMVal::Mat34(v))) => Some(*v),
                _ => None,
            }
        }

        fn to_val(self) -> ConstBaseVal {
            ConstBaseVal::Double(ConstSVMVal::M(ConstMVal::Mat34(self)))
        }
    }

    impl ConstParameter for Matrix4x2<f64> {
        fn get_type() -> ParameterType {
            ParameterType::new(ParameterBaseType::Double, ParameterSize::Mat42)
        }

        fn from_val(val: &ConstBaseVal) -> Option<Self> {
            match val {
                ConstBaseVal::Float(ConstSVMVal::M(ConstMVal::Mat42(v))) => Some(v.map(|v| v as f64)),
                ConstBaseVal::Double(ConstSVMVal::M(ConstMVal::Mat42(v))) => Some(*v),
                _ => None,
            }
        }

        fn to_val(self) -> ConstBaseVal {
            ConstBaseVal::Double(ConstSVMVal::M(ConstMVal::Mat42(self)))
        }
    }

    impl ConstParameter for Matrix4x3<f64> {
        fn get_type() -> ParameterType {
            ParameterType::new(ParameterBaseType::Double, ParameterSize::Mat43)
        }

        fn from_val(val: &ConstBaseVal) -> Option<Self> {
            match val {
                ConstBaseVal::Float(ConstSVMVal::M(ConstMVal::Mat43(v))) => Some(v.map(|v| v as f64)),
                ConstBaseVal::Double(ConstSVMVal::M(ConstMVal::Mat43(v))) => Some(*v),
                _ => None,
            }
        }

        fn to_val(self) -> ConstBaseVal {
            ConstBaseVal::Double(ConstSVMVal::M(ConstMVal::Mat43(self)))
        }
    }

    impl ConstParameter for Matrix4<f64> {
        fn get_type() -> ParameterType {
            ParameterType::new(ParameterBaseType::Double, ParameterSize::Mat4)
        }

        fn from_val(val: &ConstBaseVal) -> Option<Self> {
            match val {
                ConstBaseVal::Float(ConstSVMVal::M(ConstMVal::Mat4(v))) => Some(v.map(|v| v as f64)),
                ConstBaseVal::Double(ConstSVMVal::M(ConstMVal::Mat4(v))) => Some(*v),
                _ => None,
            }
        }

        fn to_val(self) -> ConstBaseVal {
            ConstBaseVal::Double(ConstSVMVal::M(ConstMVal::Mat4(self)))
        }
    }

    impl ConstParameter for ConstMVal<f64> {
        fn get_type() -> ParameterType {
            ParameterType::new(ParameterBaseType::Double, ParameterSize::GenMat)
        }

        fn from_val(val: &ConstBaseVal) -> Option<Self> {
            match val {
                ConstBaseVal::Float(ConstSVMVal::M(v)) => Some(v.map(|v| v as f64)),
                ConstBaseVal::Double(ConstSVMVal::M(v)) => Some(v.clone()),
                _ => None,
            }
        }

        fn to_val(self) -> ConstBaseVal {
            ConstBaseVal::Double(ConstSVMVal::M(self))
        }
    }

    fn add_sv_binop_components<T, F>(func: &mut ConstEvalFunction, f: F) where F: Fn(T, T) -> T + Clone + Send + Sync + 'static, T: ConstParameter + Scalar, ConstSVVal<T>: ConstParameter {
        let fc = f.clone();
        func.add_overload_2(move |a: ConstSVVal<T>, b: T| Some(a.map(|v| fc(v, b.clone()))));
        let fc = f.clone();
        func.add_overload_2(move |a: T, b: ConstSVVal<T>| Some(b.map(|v| fc(a.clone(), v))));
        let fc = f.clone();
        func.add_overload_2(move |a: ConstSVVal<T>, b: ConstSVVal<T>| a.zip_map(&b, &fc));
    }

    fn add_i32_binop_components<F>(func: &mut ConstEvalFunction, f: F) where F: Fn(i32, i32) -> i32 + Clone + Send + Sync + 'static {
        add_sv_binop_components(func, f);
    }

    fn add_u32_binop_components<F>(func: &mut ConstEvalFunction, f: F) where F: Fn(u32, u32) -> u32 + Clone + Send + Sync + 'static {
        add_sv_binop_components(func, f);
    }

    lazy_static! {
        static ref OP_UNARY_ADD: ConstEvalFunction = {
            let mut f = ConstEvalFunction::new();
            f.add_overload_1(|v: ConstSVVal<i32>| Some(v));
            f.add_overload_1(|v: ConstSVVal<u32>| Some(v));
            f
        };
        static ref OP_BINARY_ADD: ConstEvalFunction = {
            let mut f = ConstEvalFunction::new();
            add_i32_binop_components(&mut f, |a, b| a + b);
            add_u32_binop_components(&mut f, |a, b| a + b);
            f
        };
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        const BASE_TYPE_VALUES: &[ParameterBaseType] = &[ParameterBaseType::Bool, ParameterBaseType::Int, ParameterBaseType::UInt, ParameterBaseType::Float, ParameterBaseType::Double];
        const SIZE_VALUES: &[ParameterSize] = &[ParameterSize::Scalar, ParameterSize::Vec2, ParameterSize::Vec3, ParameterSize::Vec4, ParameterSize::GenSVec, ParameterSize::Mat2, ParameterSize::Mat23, ParameterSize::Mat24, ParameterSize::Mat32, ParameterSize::Mat3, ParameterSize::Mat34, ParameterSize::Mat42, ParameterSize::Mat43, ParameterSize::Mat4, ParameterSize::GenMat];

        #[test]
        fn test_add() {
            let a = ConstBaseVal::Bool(ConstSVVal::Scalar(true));
            let b = ConstBaseVal::Int(ConstSVVal::Vec2(Vector2::new(4, 9)));
            let c = ConstBaseVal::UInt(ConstSVVal::Vec2(Vector2::new(2, 5)));
            let d = ConstBaseVal::UInt(ConstSVVal::Vec2(Vector2::new(6, 14)));
            assert_eq!(OP_BINARY_ADD.eval(&[&a]), None);
            assert_eq!(OP_BINARY_ADD.eval(&[&b, &c]), Some(d.clone()));
            assert_eq!(OP_BINARY_ADD.eval(&[&c, &b]), Some(d));
        }

        #[test]
        fn base_type_order_samples() {
            assert_eq!(ParameterBaseType::Bool.cast_cmp(&ParameterBaseType::Bool), Some(Ordering::Equal));
            assert_eq!(ParameterBaseType::Bool.cast_cmp(&ParameterBaseType::Float), None);
            assert_eq!(ParameterBaseType::Int.cast_cmp(&ParameterBaseType::UInt), Some(Ordering::Less));
            assert_eq!(ParameterBaseType::Double.cast_cmp(&ParameterBaseType::UInt), Some(Ordering::Greater));
        }

        #[test]
        fn base_type_order_consistency() {
            for a in BASE_TYPE_VALUES {
                for b in BASE_TYPE_VALUES {
                    let expected_bca = a.cast_cmp(b).map(Ordering::reverse);
                    assert_eq!(b.cast_cmp(a), expected_bca);
                }
            }
        }

        #[test]
        fn parameter_type_order_consistency() {
            let mut types = Vec::with_capacity(BASE_TYPE_VALUES.len() * SIZE_VALUES.len());
            for base_type in BASE_TYPE_VALUES {
                for size in SIZE_VALUES {
                    types.push(ParameterType::new(*base_type, *size));
                }
            }
            for a in &types {
                for b in &types {
                    let expected_bca = a.cast_cmp(b).reverse();
                    assert_eq!(b.cast_cmp(a), expected_bca);
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
}