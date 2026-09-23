use rustc_hash::{FxHashMap, FxHashSet};

use crate::react_compiler_hir::{
    ArrayElement, ArrayPatternElement, BlockId, HirFunction, IdentifierId, Instruction,
    InstructionKind, InstructionValue, ObjectPropertyKey, ObjectPropertyOrSpread, Pattern,
    PrimitiveValue,
};

type Values = FxHashSet<IdentifierId>;
type Properties = FxHashMap<Option<String>, Values>;
type Captures = FxHashMap<IdentifierId, Values>;

#[derive(Clone, Default, PartialEq, Eq)]
struct CallableState {
    values: Captures,
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
        self.values.insert(identifier, FxHashSet::from_iter([identifier]));
    }

    fn merge(&mut self, other: &Self) {
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
        for object in self.values.get(&identifier).into_iter().flatten() {
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
            .flat_map(|(_, values)| values)
            .collect();
        self.values.insert(into, values);
        self.keys.remove(&into);
    }

    fn store(&mut self, object: IdentifierId, key: Option<String>, values: Values) {
        let objects = self.values(object);
        let definite = objects.len() == 1 && key.is_some();
        for object in objects {
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
    ) {
        let id = instruction.lvalue.identifier;
        match &instruction.value {
            InstructionValue::FunctionExpression { .. } | InstructionValue::ObjectMethod { .. } => {
                self.allocate(id)
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
            }
            InstructionValue::DeclareContext { lvalue, .. } => {
                if lvalue.kind == InstructionKind::HoistedFunction
                    && let Some(value) = hoisted.get(&lvalue.place.identifier)
                {
                    self.values.insert(lvalue.place.identifier, FxHashSet::from_iter([*value]));
                } else {
                    self.values.insert(lvalue.place.identifier, Values::default());
                }
                self.keys.remove(&lvalue.place.identifier);
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
                                .flatten()
                                .collect();
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
) -> Captures {
    if async_functions.is_empty() {
        return Captures::default();
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
                        values.extend(predecessor.values(operand.identifier));
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
            for instruction in &block.instructions {
                state.apply(&func.instructions[instruction.index()], hoisted);
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
    let mut result = Captures::default();
    for state in states.values() {
        for (&identifier, values) in &state.values {
            for value in values {
                if async_functions.contains(value) {
                    result
                        .entry(identifier)
                        .or_default()
                        .extend(captured_contexts.get(value).into_iter().flatten().copied());
                }
            }
        }
    }
    result
}
