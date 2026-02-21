/// Flow type JSON representation types.
///
/// Port of `Flood/FlowTypes.ts` from the React Compiler.
///
/// TypeScript definitions for Flow type JSON representations, based on the
/// output of Flow's internal type converter. These represent the serialized
/// form of Flow's type system that the compiler can consume.
/// Polarity of a type parameter.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Polarity {
    Positive,
    Negative,
    Neutral,
}

/// A Flow type node — the serialized representation of a Flow type.
#[derive(Debug, Clone)]
pub enum FlowType {
    Open,
    Def(Box<DefType>),
    Eval { ty: Box<FlowType>, destructor: Destructor },
    Generic { name: String, bound: Box<FlowType>, no_infer: bool },
    ThisInstance { name: String, is_this: bool },
    ThisTypeApp { t1: Box<FlowType>, t2: Box<FlowType>, t_list: Option<Vec<FlowType>> },
    TypeApp { ty: Box<FlowType>, targs: Vec<FlowType>, from_value: bool, use_desc: bool },
    FunProto,
    ObjProto,
    NullProto,
    Module { name: String },
    InternalObj,
    FunImplicitReturn,
    Any(AnySource),
    AnyValueType,
    AnnotateT { name: String, targ: Box<FlowType> },
    MergedT(Vec<FlowType>),
    UnionT(Vec<FlowType>),
    IntersectionT(Vec<FlowType>),
    OpaqueT { underlying: Option<Box<FlowType>>, super_ty: Option<Box<FlowType>>, name: String },
    Enum { name: String },
    ClassT { name: String },
}

/// Source annotation for `any` types.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AnySource {
    Annotated,
    Error,
    Unresolved,
    CatchBinding,
}

/// A definition type.
#[derive(Debug, Clone)]
pub enum DefType {
    Num,
    Str,
    Bool,
    BigInt,
    Symbol,
    NumericStrKey,
    Null,
    Void,
    Mixed,
    Empty,
    SingletonStr(String),
    SingletonNum(f64),
    SingletonBool(bool),
    SingletonBigInt(String),
    Fun(FunType),
    Obj(ObjType),
    Arr { element: Box<FlowType>, readonly: bool },
    Tuple { elements: Vec<TupleElement> },
    Indexed { dict: DictType },
    StrLitSet(Vec<String>),
    Interface { extends: Vec<FlowType>, body: ObjType },
    ClassBody { elements: Vec<ClassElement> },
    Instance(InstanceType),
    TypeOf(Box<FlowType>),
    React(ReactType),
}

/// A function type definition.
#[derive(Debug, Clone)]
pub struct FunType {
    pub params: Vec<FunParam>,
    pub rest_param: Option<Box<FlowType>>,
    pub return_type: Box<FlowType>,
    pub type_params: Vec<TypeParam>,
    pub is_predicate: bool,
}

/// A function parameter.
#[derive(Debug, Clone)]
pub struct FunParam {
    pub name: Option<String>,
    pub ty: FlowType,
    pub polarity: Polarity,
}

/// A type parameter.
#[derive(Debug, Clone)]
pub struct TypeParam {
    pub name: String,
    pub bound: FlowType,
    pub polarity: Polarity,
    pub default: Option<FlowType>,
}

/// An object type definition.
#[derive(Debug, Clone)]
pub struct ObjType {
    pub props: Vec<ObjProp>,
    pub dict: Option<DictType>,
    pub exact: bool,
}

/// An object property.
#[derive(Debug, Clone)]
pub struct ObjProp {
    pub name: String,
    pub ty: FlowType,
    pub polarity: Polarity,
    pub optional: bool,
}

/// A dictionary/indexer type.
#[derive(Debug, Clone)]
pub struct DictType {
    pub key: Box<FlowType>,
    pub value: Box<FlowType>,
    pub polarity: Polarity,
}

/// A tuple element.
#[derive(Debug, Clone)]
pub struct TupleElement {
    pub name: Option<String>,
    pub ty: FlowType,
    pub polarity: Polarity,
    pub optional: bool,
}

/// A class element.
#[derive(Debug, Clone)]
pub struct ClassElement {
    pub name: String,
    pub ty: FlowType,
    pub polarity: Polarity,
    pub static_member: bool,
}

/// An instance type.
#[derive(Debug, Clone)]
pub struct InstanceType {
    pub name: String,
    pub type_args: Vec<FlowType>,
}

/// React-specific type definitions.
#[derive(Debug, Clone)]
pub enum ReactType {
    CreateElement,
    ElementRef(Box<FlowType>),
    ElementConfig(Box<FlowType>),
    ElementProps(Box<FlowType>),
    Ref(Box<FlowType>),
}

/// A destructor (used in Eval types).
#[derive(Debug, Clone)]
pub enum Destructor {
    NonMaybeType,
    PropertyType { name: String },
    ElementType { index: u32 },
    ReadOnly,
    Partial,
    Required,
    SpreadType,
    RestType,
    ValuesType,
    CallType { args: Vec<FlowType> },
    ConditionalType { check: Box<FlowType>, extends: Box<FlowType>, true_type: Box<FlowType>, false_type: Box<FlowType> },
    MappedType,
    TypeMap { ty: Box<FlowType> },
    ReactCheckComponentConfig,
    ReactCheckComponentRef,
}
