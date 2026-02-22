/// Flood type system — Flow-based type definitions.
///
/// Port of `Flood/Types.ts` and `Flood/FlowTypes.ts` from the React Compiler.
///
/// This module defines the type system used by the experimental "Forest" feature
/// (enabled via `enableForest` config flag). It provides a richer type system
/// than the basic Type enum, supporting Flow-style nominal types, generics,
/// nullable types, and component types.
use rustc_hash::FxHashMap;

/// A type variable ID in the Flood type system.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct VariableId(pub u32);

/// A nominal type ID.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct NominalId(pub u32);

/// A linear (structural) type ID.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct LinearId(pub u32);

/// A type parameter ID.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TypeParameterId(pub u32);

/// Platform marker for type resolution.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Platform {
    Server,
    Client,
    Universal,
}

/// A type in the Flood type system.
#[derive(Debug, Clone)]
pub enum FloodType {
    /// A concrete (fully-known) type.
    Concrete { ty: ConcreteType, platform: Platform },
    /// A type variable (to be resolved by unification).
    Variable { id: VariableId },
}

/// A resolved (concrete) type.
#[derive(Debug, Clone)]
pub enum ConcreteType {
    Enum,
    Mixed,
    Number,
    String,
    Boolean,
    Void,
    Nullable(Box<FloodType>),
    Array {
        element: Box<FloodType>,
    },
    Set {
        element: Box<FloodType>,
    },
    Map {
        key: Box<FloodType>,
        value: Box<FloodType>,
    },
    Function {
        type_parameters: Option<Vec<TypeParameter>>,
        params: Vec<FloodType>,
        return_type: Box<FloodType>,
    },
    Component {
        props: FxHashMap<String, FloodType>,
        children: Option<Box<FloodType>>,
    },
    Generic {
        id: TypeParameterId,
        bound: Box<FloodType>,
    },
    Object {
        id: NominalId,
        members: FxHashMap<String, FloodType>,
    },
    Tuple {
        id: NominalId,
        members: Vec<FloodType>,
    },
    Structural {
        id: LinearId,
    },
    Union(Vec<FloodType>),
    Intersection(Vec<FloodType>),
}

/// A type parameter definition.
#[derive(Debug, Clone)]
pub struct TypeParameter {
    pub id: TypeParameterId,
    pub bound: FloodType,
}

/// The Flood type environment.
///
/// Manages type bindings, unification state, and error reporting for the
/// Flow-based type inference system.
#[derive(Debug, Clone)]
pub struct FloodTypeEnv {
    /// Map from variable ID to its resolved type.
    bindings: FxHashMap<VariableId, FloodType>,
    /// Next variable ID for fresh type variables.
    next_var_id: u32,
    /// Next nominal ID.
    next_nominal_id: u32,
}

impl Default for FloodTypeEnv {
    fn default() -> Self {
        Self::new()
    }
}

impl FloodTypeEnv {
    /// Create a new empty type environment.
    pub fn new() -> Self {
        Self { bindings: FxHashMap::default(), next_var_id: 0, next_nominal_id: 0 }
    }

    /// Create a fresh type variable.
    pub fn fresh_var(&mut self) -> FloodType {
        let id = VariableId(self.next_var_id);
        self.next_var_id += 1;
        FloodType::Variable { id }
    }

    /// Create a fresh nominal ID.
    pub fn fresh_nominal(&mut self) -> NominalId {
        let id = NominalId(self.next_nominal_id);
        self.next_nominal_id += 1;
        id
    }

    /// Bind a type variable to a type.
    pub fn bind(&mut self, var: VariableId, ty: FloodType) {
        self.bindings.insert(var, ty);
    }

    /// Resolve a type by following variable bindings.
    pub fn resolve(&self, ty: &FloodType) -> FloodType {
        match ty {
            FloodType::Variable { id } => match self.bindings.get(id) {
                Some(bound) => self.resolve(bound),
                None => ty.clone(),
            },
            FloodType::Concrete { .. } => ty.clone(),
        }
    }

    /// Unify two types.
    ///
    /// # Errors
    /// Returns an error string if the types are incompatible.
    pub fn unify(&mut self, a: &FloodType, b: &FloodType) -> Result<(), String> {
        let a = self.resolve(a);
        let b = self.resolve(b);

        match (&a, &b) {
            (FloodType::Variable { id: id_a }, FloodType::Variable { id: id_b })
                if id_a == id_b =>
            {
                Ok(())
            }
            (FloodType::Variable { id }, _) => {
                self.bind(*id, b);
                Ok(())
            }
            (_, FloodType::Variable { id }) => {
                self.bind(*id, a);
                Ok(())
            }
            (FloodType::Concrete { ty: ty_a, .. }, FloodType::Concrete { ty: ty_b, .. }) => {
                self.unify_concrete(ty_a, ty_b)
            }
        }
    }

    fn unify_concrete(&mut self, a: &ConcreteType, b: &ConcreteType) -> Result<(), String> {
        match (a, b) {
            (ConcreteType::Number, ConcreteType::Number)
            | (ConcreteType::String, ConcreteType::String)
            | (ConcreteType::Boolean, ConcreteType::Boolean)
            | (ConcreteType::Void, ConcreteType::Void)
            | (ConcreteType::Mixed, ConcreteType::Mixed)
            | (ConcreteType::Enum, ConcreteType::Enum) => Ok(()),
            (ConcreteType::Nullable(inner_a), ConcreteType::Nullable(inner_b)) => {
                self.unify(inner_a, inner_b)
            }
            (ConcreteType::Array { element: a }, ConcreteType::Array { element: b }) => {
                self.unify(a, b)
            }
            _ => Err(format!("Cannot unify {a:?} with {b:?}")),
        }
    }
}
