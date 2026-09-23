use std::rc::Rc;

use rustc_hash::{FxHashMap, FxHashSet};

use super::validate_locals_not_reassigned_after_render::each_invoked_async_operand;
use crate::react_compiler_hir::environment::Environment;
use crate::react_compiler_hir::{
    ArrayElement, ArrayPatternElement, BlockId, HirFunction, IdentifierId, Instruction,
    InstructionId, InstructionKind, InstructionValue, ObjectPropertyKey, ObjectPropertyOrSpread,
    Pattern, PrimitiveValue,
};

type Captures = FxHashMap<IdentifierId, FxHashSet<IdentifierId>>;
type Properties = FxHashMap<Option<String>, Values>;

pub(super) struct AsyncInvocation {
    pub captures: FxHashSet<IdentifierId>,
    pub context_values: Rc<Captures>,
}

pub(super) struct AsyncCallables {
    pub identifiers: FxHashSet<IdentifierId>,
    pub invocations: FxHashMap<InstructionId, Vec<AsyncInvocation>>,
}

/// Pair each possible async function with the live bindings on the paths where
/// that function is selected. Merging another callable must not mix its bindings
/// into this function's captures.
#[derive(Clone, Default, PartialEq, Eq)]
struct Values {
    identities: FxHashSet<IdentifierId>,
    contexts: FxHashMap<IdentifierId, Rc<Captures>>,
}

impl Values {
    fn single(identifier: IdentifierId) -> Self {
        Self { identities: FxHashSet::from_iter([identifier]), contexts: FxHashMap::default() }
    }

    fn extend(&mut self, other: &Self) {
        self.identities.extend(&other.identities);
        for (&function, contexts) in &other.contexts {
            let into = self.contexts.entry(function).or_default();
            if Rc::ptr_eq(into, contexts) {
                continue;
            }
            let into = Rc::make_mut(into);
            for (&binding, values) in contexts.iter() {
                into.entry(binding).or_default().extend(values);
            }
        }
    }

    fn update_context(&mut self, binding: IdentifierId, values: &FxHashSet<IdentifierId>) {
        for contexts in self.contexts.values_mut() {
            Rc::make_mut(contexts).insert(binding, values.clone());
        }
    }
}

#[derive(Clone, Default, PartialEq, Eq)]
struct CallableState {
    values: FxHashMap<IdentifierId, Values>,
    context_values: Captures,
    properties: FxHashMap<IdentifierId, Properties>,
    keys: FxHashMap<IdentifierId, String>,
}

impl CallableState {
    fn values(&self, identifier: IdentifierId) -> Values {
        self.values.get(&identifier).cloned().unwrap_or_default()
    }

    fn copy(&mut self, from: IdentifierId, into: IdentifierId) {
        self.values.insert(into, self.values(from));
        if let Some(key) = self.keys.get(&from).cloned() {
            self.keys.insert(into, key);
        } else {
            self.keys.remove(&into);
        }
    }

    fn allocate(&mut self, identifier: IdentifierId) {
        self.values.insert(identifier, Values::single(identifier));
    }

    fn update_context(&mut self, binding: IdentifierId) {
        let values = self.values(binding).identities;
        self.context_values.insert(binding, values.clone());
        for value in self.values.values_mut() {
            value.update_context(binding, &values);
        }
        for properties in self.properties.values_mut() {
            for value in properties.values_mut() {
                value.update_context(binding, &values);
            }
        }
    }

    fn merge(&mut self, other: &Self) {
        for (&binding, values) in &other.context_values {
            self.context_values.entry(binding).or_default().extend(values);
        }
        for (&id, values) in &other.values {
            self.values.entry(id).or_default().extend(values);
        }
        for (&object, properties) in &other.properties {
            let into = self.properties.entry(object).or_default();
            for (key, values) in properties {
                into.entry(key.clone()).or_default().extend(values);
            }
        }
        self.keys.retain(|id, key| other.keys.get(id) == Some(key));
    }

    fn key(&self, property: &ObjectPropertyKey<'_>) -> Option<String> {
        match property {
            ObjectPropertyKey::Identifier { name, .. } | ObjectPropertyKey::String { name, .. } => {
                Some(name.to_string())
            }
            ObjectPropertyKey::Computed { name, .. } => self.keys.get(&name.identifier).cloned(),
        }
    }

    fn properties(&self, identifier: IdentifierId) -> Properties {
        let mut result = Properties::default();
        for object in self.values.get(&identifier).into_iter().flat_map(|values| &values.identities)
        {
            if let Some(properties) = self.properties.get(object) {
                for (key, values) in properties {
                    result.entry(key.clone()).or_default().extend(values);
                }
            }
        }
        result
    }

    fn load(&mut self, object: IdentifierId, key: Option<String>, into: IdentifierId) {
        let values = self
            .properties(object)
            .into_iter()
            .filter(|(property, _)| key.is_none() || property.is_none() || key == *property)
            .fold(Values::default(), |mut result, (_, values)| {
                result.extend(&values);
                result
            });
        self.values.insert(into, values);
        self.keys.remove(&into);
    }

    fn store(&mut self, object: IdentifierId, key: Option<String>, values: Values) {
        let objects = self.values(object);
        let definite = objects.identities.len() == 1 && key.is_some();
        for object in objects.identities {
            let properties = self.properties.entry(object).or_default();
            if definite {
                properties.insert(key.clone(), values.clone());
            } else {
                properties.entry(key.clone()).or_default().extend(&values);
            }
        }
    }

    fn destructure(&mut self, pattern: &Pattern<'_>, source: IdentifierId) {
        match pattern {
            Pattern::Object(pattern) => {
                let mut excluded = FxHashSet::default();
                for property in &pattern.properties {
                    match property {
                        ObjectPropertyOrSpread::Property(property) => {
                            let key = self.key(&property.key);
                            if let Some(key) = &key {
                                excluded.insert(key.clone());
                            }
                            self.load(source, key, property.place.identifier);
                        }
                        ObjectPropertyOrSpread::Spread(spread) => {
                            let mut properties = self.properties(source);
                            properties.retain(|key, _| {
                                key.as_ref().is_none_or(|key| !excluded.contains(key))
                            });
                            self.allocate(spread.place.identifier);
                            self.properties.insert(spread.place.identifier, properties);
                        }
                    }
                }
            }
            Pattern::Array(pattern) => {
                for (index, element) in pattern.items.iter().enumerate() {
                    match element {
                        ArrayPatternElement::Place(place) => {
                            self.load(source, Some(index.to_string()), place.identifier)
                        }
                        ArrayPatternElement::Spread(spread) => {
                            let properties = self
                                .properties(source)
                                .into_iter()
                                .filter_map(|(key, values)| {
                                    let key = match key {
                                        Some(key) => Some(
                                            key.parse::<usize>()
                                                .ok()?
                                                .checked_sub(index)?
                                                .to_string(),
                                        ),
                                        None => None,
                                    };
                                    Some((key, values))
                                })
                                .collect();
                            self.allocate(spread.place.identifier);
                            self.properties.insert(spread.place.identifier, properties);
                        }
                        ArrayPatternElement::Hole => {}
                    }
                }
            }
        }
    }

    fn apply(
        &mut self,
        instruction: &Instruction<'_>,
        hoisted: &FxHashMap<IdentifierId, IdentifierId>,
        async_functions: &FxHashSet<IdentifierId>,
        captured_contexts: &Captures,
    ) {
        let id = instruction.lvalue.identifier;
        match &instruction.value {
            InstructionValue::FunctionExpression { .. } | InstructionValue::ObjectMethod { .. } => {
                self.allocate(id);
                if async_functions.contains(&id) {
                    let mut contexts = self.context_values.clone();
                    for &capture in captured_contexts.get(&id).into_iter().flatten() {
                        contexts.entry(capture).or_insert_with(|| FxHashSet::from_iter([capture]));
                    }
                    self.values.get_mut(&id).unwrap().contexts.insert(id, Rc::new(contexts));
                }
            }
            InstructionValue::Primitive { value, .. } => {
                let key = match value {
                    PrimitiveValue::String(value) => value.to_string(),
                    PrimitiveValue::Number(value) => value.to_string(),
                    PrimitiveValue::Boolean(value) => value.to_string(),
                    PrimitiveValue::Null => "null".to_string(),
                    PrimitiveValue::Undefined => "undefined".to_string(),
                };
                self.keys.insert(id, key);
                self.values.insert(id, Values::default());
            }
            InstructionValue::LoadLocal { place, .. }
            | InstructionValue::LoadContext { place, .. } => self.copy(place.identifier, id),
            InstructionValue::TypeCastExpression { value, .. } => self.copy(value.identifier, id),
            InstructionValue::StoreLocal { lvalue, value, .. }
            | InstructionValue::StoreContext { lvalue, value, .. } => {
                if matches!(instruction.value, InstructionValue::StoreContext { .. })
                    && lvalue.kind == InstructionKind::Function
                    && hoisted.contains_key(&lvalue.place.identifier)
                {
                    return;
                }
                self.copy(value.identifier, lvalue.place.identifier);
                self.copy(value.identifier, id);
                if matches!(instruction.value, InstructionValue::StoreContext { .. }) {
                    self.update_context(lvalue.place.identifier);
                }
            }
            InstructionValue::DeclareContext { lvalue, .. } => {
                if lvalue.kind == InstructionKind::HoistedFunction
                    && let Some(value) = hoisted.get(&lvalue.place.identifier)
                {
                    let mut values = Values::single(*value);
                    if async_functions.contains(value) {
                        values.contexts.insert(*value, Rc::new(self.context_values.clone()));
                    }
                    self.values.insert(lvalue.place.identifier, values);
                } else {
                    self.values.insert(lvalue.place.identifier, Values::default());
                }
                self.keys.remove(&lvalue.place.identifier);
                self.update_context(lvalue.place.identifier);
            }
            InstructionValue::ObjectExpression { properties, .. } => {
                self.allocate(id);
                self.properties.insert(id, Properties::default());
                for property in properties {
                    match property {
                        ObjectPropertyOrSpread::Property(property) => {
                            self.store(
                                id,
                                self.key(&property.key),
                                self.values(property.place.identifier),
                            );
                        }
                        ObjectPropertyOrSpread::Spread(spread) => {
                            for (key, values) in self.properties(spread.place.identifier) {
                                self.store(id, key, values);
                            }
                        }
                    }
                }
            }
            InstructionValue::ArrayExpression { elements, .. } => {
                self.allocate(id);
                self.properties.insert(id, Properties::default());
                let mut index = Some(0usize);
                for element in elements {
                    match element {
                        ArrayElement::Place(place) => self.store(
                            id,
                            index.map(|index| index.to_string()),
                            self.values(place.identifier),
                        ),
                        ArrayElement::Spread(spread) => {
                            // Unknown spread lengths make subsequent indices uncertain.
                            let values = self
                                .properties(spread.place.identifier)
                                .into_values()
                                .fold(Values::default(), |mut result, values| {
                                    result.extend(&values);
                                    result
                                });
                            self.store(id, None, values);
                            index = None;
                            continue;
                        }
                        ArrayElement::Hole => {}
                    }
                    index = index.map(|index| index + 1);
                }
            }
            InstructionValue::PropertyLoad { object, property, .. } => {
                self.load(object.identifier, Some(property.to_string()), id)
            }
            InstructionValue::ComputedLoad { object, property, .. } => {
                self.load(object.identifier, self.keys.get(&property.identifier).cloned(), id)
            }
            InstructionValue::PropertyStore { object, property, value, .. } => self.store(
                object.identifier,
                Some(property.to_string()),
                self.values(value.identifier),
            ),
            InstructionValue::ComputedStore { object, property, value, .. } => self.store(
                object.identifier,
                self.keys.get(&property.identifier).cloned(),
                self.values(value.identifier),
            ),
            InstructionValue::PropertyDelete { object, property, .. } => {
                self.store(object.identifier, Some(property.to_string()), Values::default())
            }
            InstructionValue::ComputedDelete { object, property, .. } => self.store(
                object.identifier,
                self.keys.get(&property.identifier).cloned(),
                Values::default(),
            ),
            InstructionValue::Destructure { lvalue, value, .. } => {
                self.destructure(&lvalue.pattern, value.identifier);
                self.copy(value.identifier, id);
            }
            InstructionValue::PrefixUpdateContext { lvalue, .. }
            | InstructionValue::PostfixUpdateContext { lvalue, .. } => {
                self.values.insert(lvalue.identifier, Values::default());
                self.keys.remove(&lvalue.identifier);
                self.update_context(lvalue.identifier);
            }
            _ => {}
        }
    }
}

/// Resolve async identity at each value-producing instruction. Heap properties
/// use reaching values across the CFG: a load snapshots the value before later
/// writes, and a definite store replaces the old member. Containment never makes
/// a container itself callable.
pub(super) fn infer_async_callable_contexts(
    func: &HirFunction,
    async_functions: &FxHashSet<IdentifierId>,
    captured_contexts: &Captures,
    hoisted: &FxHashMap<IdentifierId, IdentifierId>,
    env: &Environment<'_>,
) -> AsyncCallables {
    let mut result =
        AsyncCallables { identifiers: FxHashSet::default(), invocations: FxHashMap::default() };
    if async_functions.is_empty() {
        return result;
    }
    let mut states: FxHashMap<BlockId, CallableState> = FxHashMap::default();
    loop {
        let mut changed = false;
        for (&block_id, block) in &func.body.blocks {
            let mut predecessors =
                block.preds.iter().filter_map(|predecessor| states.get(predecessor));
            let mut state = predecessors.next().cloned().unwrap_or_default();
            for predecessor in predecessors {
                state.merge(predecessor);
            }
            for phi in &block.phis {
                let mut values = Values::default();
                let mut keys = Vec::new();
                for (predecessor, operand) in &phi.operands {
                    if let Some(predecessor) = states.get(predecessor) {
                        values.extend(&predecessor.values(operand.identifier));
                        keys.push(predecessor.keys.get(&operand.identifier));
                    }
                }
                state.values.insert(phi.place.identifier, values);
                if let Some(Some(key)) = keys.first()
                    && keys.iter().all(|other| *other == Some(*key))
                {
                    state.keys.insert(phi.place.identifier, (*key).clone());
                } else {
                    state.keys.remove(&phi.place.identifier);
                }
            }
            for &instruction_id in &block.instructions {
                let instruction = &func.instructions[instruction_id.index()];
                let mut invocations = Vec::new();
                for operand in each_invoked_async_operand(&instruction.value, env) {
                    for (function, context_values) in state.values(operand.identifier).contexts {
                        invocations.push(AsyncInvocation {
                            captures: captured_contexts.get(&function).cloned().unwrap_or_default(),
                            context_values,
                        });
                    }
                }
                result.invocations.insert(instruction_id, invocations);
                state.apply(instruction, hoisted, async_functions, captured_contexts);
            }
            if states.get(&block_id) != Some(&state) {
                states.insert(block_id, state);
                changed = true;
            }
        }
        if !changed {
            break;
        }
    }
    for state in states.values() {
        for (&identifier, values) in &state.values {
            if !values.contexts.is_empty() {
                result.identifiers.insert(identifier);
            }
        }
    }
    result
}
