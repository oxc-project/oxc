/// Type system types for the React Compiler HIR.
///
/// Port of `HIR/Types.ts` from the React Compiler.
///
/// Represents the type information inferred during compilation, including
/// primitive types, function types, object types, type variables, and more.
use std::sync::atomic::{AtomicU32, Ordering};

/// Newtype for type IDs to prevent accidental misuse with plain numbers.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TypeId(pub u32);

/// A built-in type (primitive, function, or object).
#[derive(Debug, Clone, PartialEq)]
pub enum BuiltInType {
    Primitive,
    Function(FunctionType),
    Object(ObjectType),
}

/// The type of a value in the HIR.
#[derive(Debug, Clone, PartialEq)]
pub enum Type {
    /// A primitive type (string, number, boolean, null, undefined).
    Primitive,
    /// A function type (callable object with call signature).
    Function(FunctionType),
    /// An object type.
    Object(ObjectType),
    /// A phi type (union of types from different control-flow paths).
    Phi(PhiType),
    /// A type variable (unresolved type).
    Var(TypeId),
    /// A polymorphic type.
    Poly,
    /// A property type (type of a specific property of an object).
    Property(Box<PropType>),
    /// An object method type.
    ObjectMethod,
}

/// A function type, representing any callable object.
///
/// Note: `shape_id` links to an ObjectShape in the ShapeRegistry.
/// `return_type` is the return type of the function. `is_constructor` indicates
/// if the function can be called with `new`.
#[derive(Debug, Clone, PartialEq)]
pub struct FunctionType {
    pub shape_id: Option<String>,
    pub return_type: Box<Type>,
    pub is_constructor: bool,
}

/// An object type, potentially associated with a known shape.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ObjectType {
    pub shape_id: Option<String>,
}

/// A phi type — the union of types from different control-flow paths.
#[derive(Debug, Clone, PartialEq)]
pub struct PhiType {
    pub operands: Vec<Type>,
}

/// The name of a property, either a literal or computed.
#[derive(Debug, Clone, PartialEq)]
pub enum PropertyName {
    Literal { value: PropertyLiteral },
    Computed { value: Box<Type> },
}

/// A property type, representing the type of `object.property`.
#[derive(Debug, Clone, PartialEq)]
pub struct PropType {
    pub object_type: Type,
    pub object_name: String,
    pub property_name: PropertyName,
}

/// A property literal value (string or numeric key).
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum PropertyLiteral {
    String(String),
    Number(i64),
}

impl std::fmt::Display for PropertyLiteral {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PropertyLiteral::String(s) => write!(f, "{s}"),
            PropertyLiteral::Number(n) => write!(f, "{n}"),
        }
    }
}

/// Global counter for generating unique type IDs.
static TYPE_COUNTER: AtomicU32 = AtomicU32::new(0);

/// Create a fresh type variable with a unique ID.
pub fn make_type() -> Type {
    let id = TYPE_COUNTER.fetch_add(1, Ordering::Relaxed);
    Type::Var(TypeId(id))
}

/// Reset the type counter (for testing purposes).
pub fn reset_type_counter() {
    TYPE_COUNTER.store(0, Ordering::Relaxed);
}

/// Duplicates the given type, copying types that are exact while creating fresh
/// type identifiers for any abstract types.
pub fn duplicate_type(ty: &Type) -> Type {
    match ty {
        Type::Function(func) => Type::Function(FunctionType {
            return_type: Box::new(duplicate_type(&func.return_type)),
            shape_id: func.shape_id.clone(),
            is_constructor: func.is_constructor,
        }),
        Type::Object(obj) => Type::Object(ObjectType { shape_id: obj.shape_id.clone() }),
        Type::ObjectMethod => Type::ObjectMethod,
        Type::Phi(phi) => {
            Type::Phi(PhiType { operands: phi.operands.iter().map(duplicate_type).collect() })
        }
        Type::Poly => Type::Poly,
        Type::Primitive => Type::Primitive,
        Type::Property(prop) => Type::Property(Box::new(PropType {
            object_type: duplicate_type(&prop.object_type),
            object_name: prop.object_name.clone(),
            property_name: prop.property_name.clone(),
        })),
        Type::Var(_) => make_type(),
    }
}

/// Check if two types are structurally equal.
pub fn type_equals(a: &Type, b: &Type) -> bool {
    match (a, b) {
        (Type::Primitive, Type::Primitive)
        | (Type::Poly, Type::Poly)
        | (Type::ObjectMethod, Type::ObjectMethod) => true,
        (Type::Var(id_a), Type::Var(id_b)) => id_a == id_b,
        (Type::Function(fa), Type::Function(fb)) => {
            fa.shape_id == fb.shape_id && type_equals(&fa.return_type, &fb.return_type)
        }
        (Type::Object(oa), Type::Object(ob)) => oa.shape_id == ob.shape_id,
        (Type::Phi(pa), Type::Phi(pb)) => {
            if pa.operands.len() != pb.operands.len() {
                return false;
            }
            // Note: The original TS implementation has a bug here where it always returns false.
            // We port it faithfully.
            false
        }
        (Type::Property(pa), Type::Property(pb)) => {
            type_equals(&pa.object_type, &pb.object_type)
                && pa.object_name == pb.object_name
                && pa.property_name == pb.property_name
        }
        _ => false,
    }
}

impl Type {
    /// Returns the kind name of this type (useful for debugging/display).
    pub fn kind(&self) -> &'static str {
        match self {
            Type::Primitive => "Primitive",
            Type::Function(_) => "Function",
            Type::Object(_) => "Object",
            Type::Phi(_) => "Phi",
            Type::Var(_) => "Type",
            Type::Poly => "Poly",
            Type::Property(_) => "Property",
            Type::ObjectMethod => "ObjectMethod",
        }
    }

    /// Returns the shape_id if this type has one (Function or Object).
    pub fn shape_id(&self) -> Option<&str> {
        match self {
            Type::Function(f) => f.shape_id.as_deref(),
            Type::Object(o) => o.shape_id.as_deref(),
            _ => None,
        }
    }
}
