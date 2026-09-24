use std::rc::Rc;

use rustc_hash::{FxHashMap, FxHashSet};

use super::validate_locals_not_reassigned_after_render::each_invoked_async_operand;
use crate::react_compiler_hir::environment::Environment;
use crate::react_compiler_hir::visitors::{each_instruction_value_operand, each_terminal_operand};
use crate::react_compiler_hir::{
    ArrayElement, ArrayPatternElement, BlockId, HirFunction, IdentifierId, Instruction,
    InstructionId, InstructionKind, InstructionValue, ObjectPropertyKey, ObjectPropertyOrSpread,
    ParamPattern, Pattern, PlaceOrSpread, PrimitiveValue, Terminal,
};

type Captures = FxHashMap<IdentifierId, FxHashSet<IdentifierId>>;
type Properties = FxHashMap<Option<String>, Values>;

/// Refine captures used only through named members. A direct escape, unknown
/// property, or nested closure keeps the whole capture. This lets a returned
/// closure select individual rest arguments without taint from unused siblings.
fn infer_capture_members(
    func: &HirFunction,
    env: &Environment<'_>,
) -> FxHashMap<IdentifierId, FxHashSet<String>> {
    let mut members: FxHashMap<_, FxHashSet<String>> =
        func.context.iter().map(|place| (place.identifier, FxHashSet::default())).collect();
    if members.is_empty() {
        return members;
    }
    let mut origins: Captures =
        members.keys().map(|&id| (id, FxHashSet::from_iter([id]))).collect();
    let mut keys = FxHashMap::default();
    let mut receivers = FxHashMap::default();
    for instruction in &func.instructions {
        match &instruction.value {
            InstructionValue::Primitive { value: PrimitiveValue::String(value), .. } => {
                keys.insert(instruction.lvalue.identifier, value.to_string());
            }
            InstructionValue::Primitive { value: PrimitiveValue::Number(value), .. } => {
                keys.insert(instruction.lvalue.identifier, value.to_string());
            }
            InstructionValue::PropertyLoad { object, .. }
            | InstructionValue::ComputedLoad { object, .. } => {
                receivers.insert(instruction.lvalue.identifier, object.identifier);
            }
            _ => {}
        }
    }
    loop {
        let mut changed = false;
        let mut copy = |from, into| {
            let values = origins.get(&from).cloned().unwrap_or_default();
            let target = origins.entry(into).or_default();
            let old_len = target.len();
            target.extend(values);
            changed |= old_len != target.len();
        };
        for (_, block) in &func.body.blocks {
            for phi in &block.phis {
                for operand in phi.operands.values() {
                    copy(operand.identifier, phi.place.identifier);
                }
            }
            for &id in &block.instructions {
                let instruction = &func.instructions[id.index()];
                match &instruction.value {
                    InstructionValue::LoadLocal { place, .. }
                    | InstructionValue::LoadContext { place, .. } => {
                        copy(place.identifier, instruction.lvalue.identifier)
                    }
                    InstructionValue::TypeCastExpression { value, .. } => {
                        copy(value.identifier, instruction.lvalue.identifier)
                    }
                    InstructionValue::StoreLocal { lvalue, value, .. }
                    | InstructionValue::StoreContext { lvalue, value, .. } => {
                        copy(value.identifier, lvalue.place.identifier);
                        copy(value.identifier, instruction.lvalue.identifier);
                    }
                    _ => {}
                }
            }
        }
        if !changed {
            break;
        }
    }
    let remove = |id, members: &mut FxHashMap<IdentifierId, FxHashSet<String>>| {
        for origin in origins.get(&id).into_iter().flatten() {
            members.remove(origin);
        }
    };
    for instruction in &func.instructions {
        match &instruction.value {
            InstructionValue::PropertyLoad { object, property, .. } => {
                for origin in origins.get(&object.identifier).into_iter().flatten() {
                    if let Some(members) = members.get_mut(origin) {
                        members.insert(property.to_string());
                    }
                }
            }
            InstructionValue::ComputedLoad { object, property, .. } => {
                if let Some(key) = keys.get(&property.identifier) {
                    for origin in origins.get(&object.identifier).into_iter().flatten() {
                        if let Some(members) = members.get_mut(origin) {
                            members.insert(key.clone());
                        }
                    }
                } else {
                    remove(object.identifier, &mut members);
                }
            }
            InstructionValue::LoadLocal { .. }
            | InstructionValue::LoadContext { .. }
            | InstructionValue::StoreLocal { .. }
            | InstructionValue::StoreContext { .. }
            | InstructionValue::TypeCastExpression { .. } => {}
            InstructionValue::FunctionExpression { lowered_func, .. }
            | InstructionValue::ObjectMethod { lowered_func, .. } => {
                for context in &env.functions[lowered_func.func].context {
                    remove(context.identifier, &mut members);
                }
            }
            value => {
                for operand in each_instruction_value_operand(value, env) {
                    if let InstructionValue::MethodCall { receiver, property, .. } = value
                        && operand.identifier == receiver.identifier
                        && receivers.get(&property.identifier) == Some(&receiver.identifier)
                    {
                        continue;
                    }
                    remove(operand.identifier, &mut members);
                }
            }
        }
    }
    for (_, block) in &func.body.blocks {
        for operand in each_terminal_operand(&block.terminal) {
            remove(operand.identifier, &mut members);
        }
    }
    members
}

// Named members include every reaching write to that member. The unknown-key
// bucket is only the fallback for names that have not been written explicitly.
fn merge_properties(into: &mut Properties, other: &Properties) {
    let fallback = into.get(&None).cloned().unwrap_or_default();
    for (key, values) in into.iter_mut() {
        if let Some(other) = other.get(key).or_else(|| other.get(&None)) {
            values.extend(other);
        }
    }
    for (key, values) in other {
        if !into.contains_key(key) {
            let mut values = values.clone();
            values.extend(&fallback);
            into.insert(key.clone(), values);
        }
    }
}

pub(super) struct AsyncInvocation {
    pub captures: FxHashSet<IdentifierId>,
    pub context_values: Rc<Captures>,
}

pub(super) struct AsyncCallables {
    pub identifiers: FxHashSet<IdentifierId>,
    pub invocations: FxHashMap<InstructionId, Vec<AsyncInvocation>>,
    pub returned: ReturnedCallables,
}

#[derive(Clone, Default, PartialEq, Eq)]
pub(super) struct ReturnedCallables {
    values: Values,
    external_contexts: FxHashSet<IdentifierId>,
    params: Vec<(IdentifierId, bool)>,
    capture_members: FxHashMap<IdentifierId, FxHashSet<String>>,
}

#[derive(Clone, PartialEq, Eq)]
struct Callable {
    is_async: bool,
    captures: FxHashSet<IdentifierId>,
    context_values: Rc<Captures>,
    returned: Option<Rc<ReturnedCallables>>,
    capture_members: FxHashMap<IdentifierId, FxHashSet<String>>,
}

/// Pair each possible function with the live bindings on the paths where
/// that function is selected. Merging another callable must not mix its bindings
/// into this function's captures.
#[derive(Clone, Default, PartialEq, Eq)]
struct Values {
    identities: FxHashSet<IdentifierId>,
    contexts: FxHashMap<IdentifierId, Callable>,
}

impl Values {
    fn single(identifier: IdentifierId) -> Self {
        Self { identities: FxHashSet::from_iter([identifier]), contexts: FxHashMap::default() }
    }

    fn extend(&mut self, other: &Self) {
        self.identities.extend(&other.identities);
        for (&function, callable) in &other.contexts {
            let Some(into) = self.contexts.get_mut(&function) else {
                self.contexts.insert(function, callable.clone());
                continue;
            };
            if Rc::ptr_eq(&into.context_values, &callable.context_values) {
                continue;
            }
            let into = Rc::make_mut(&mut into.context_values);
            for (&binding, values) in callable.context_values.iter() {
                into.entry(binding).or_default().extend(values);
            }
        }
    }

    fn update_context(&mut self, binding: IdentifierId, values: &FxHashSet<IdentifierId>) {
        for callable in self.contexts.values_mut() {
            Rc::make_mut(&mut callable.context_values).insert(binding, values.clone());
        }
    }
}

#[derive(Clone, Default, PartialEq, Eq)]
struct CallableState {
    values: FxHashMap<IdentifierId, Values>,
    context_values: Captures,
    properties: FxHashMap<IdentifierId, Properties>,
    keys: FxHashMap<IdentifierId, String>,
    builtin_function_methods: FxHashMap<IdentifierId, String>,
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
        if let Some(method) = self.builtin_function_methods.get(&from).cloned() {
            self.builtin_function_methods.insert(into, method);
        } else {
            self.builtin_function_methods.remove(&into);
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
            merge_properties(self.properties.entry(object).or_default(), properties);
        }
        self.keys.retain(|id, key| other.keys.get(id) == Some(key));
        self.builtin_function_methods
            .retain(|id, method| other.builtin_function_methods.get(id) == Some(method));
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
                merge_properties(&mut result, properties);
            }
        }
        result
    }

    fn load(&mut self, object: IdentifierId, key: Option<String>, into: IdentifierId) {
        let properties = self.properties(object);
        if let Some(key) = &key
            && matches!(key.as_str(), "call" | "apply" | "bind")
            && !self.values(object).contexts.is_empty()
            && !properties.contains_key(&Some(key.clone()))
            && !properties.contains_key(&None)
        {
            self.builtin_function_methods.insert(into, key.clone());
        } else {
            self.builtin_function_methods.remove(&into);
        }
        let values = if key.is_some() {
            properties.get(&key).or_else(|| properties.get(&None)).cloned().unwrap_or_default()
        } else {
            properties.into_values().fold(Values::default(), |mut result, values| {
                result.extend(&values);
                result
            })
        };
        self.values.insert(into, values);
        self.keys.remove(&into);
    }

    fn invoked(&self, instruction: &InstructionValue<'_>) -> Values {
        match instruction {
            InstructionValue::CallExpression { callee, .. }
            | InstructionValue::NewExpression { callee, .. } => self.values(callee.identifier),
            InstructionValue::MethodCall { receiver, property, .. } => {
                if self
                    .builtin_function_methods
                    .get(&property.identifier)
                    .is_some_and(|method| method != "bind")
                {
                    self.values(receiver.identifier)
                } else {
                    self.values(property.identifier)
                }
            }
            InstructionValue::TaggedTemplateExpression { tag, .. } => self.values(tag.identifier),
            _ => Values::default(),
        }
    }

    fn call_result(&mut self, instruction: &Instruction<'_>) {
        if let InstructionValue::MethodCall { receiver, property, .. } = &instruction.value
            && self
                .builtin_function_methods
                .get(&property.identifier)
                .is_some_and(|method| method == "bind")
        {
            // Binding preserves the target's identity and live captures without
            // invoking it. The continuation starts only when the result is called.
            self.copy(receiver.identifier, instruction.lvalue.identifier);
            return;
        }
        let applied_arguments = match &instruction.value {
            InstructionValue::MethodCall { property, args, .. }
                if self
                    .builtin_function_methods
                    .get(&property.identifier)
                    .is_some_and(|name| name == "apply") =>
            {
                args.get(1).and_then(|argument| match argument {
                    PlaceOrSpread::Place(place) => Some(self.properties(place.identifier)),
                    PlaceOrSpread::Spread(_) => None,
                })
            }
            _ => None,
        };
        let arguments = match &instruction.value {
            InstructionValue::CallExpression { args, .. }
            | InstructionValue::NewExpression { args, .. } => Some(args.as_slice()),
            InstructionValue::MethodCall { property, args, .. } => {
                match self.builtin_function_methods.get(&property.identifier).map(String::as_str) {
                    Some("call") => Some(args.get(1..).unwrap_or_default()),
                    Some("apply") => None,
                    _ => Some(args.as_slice()),
                }
            }
            _ => None,
        };
        let mut result = Values::default();
        for callable in self.invoked(&instruction.value).contexts.into_values() {
            // Async calls produce promises, not the eventual returned callable.
            if callable.is_async {
                continue;
            }
            let Some(returned) = &callable.returned else { continue };
            let mut values = returned.values.clone();
            let mut substitutions = Captures::default();
            let mut rest_arguments = FxHashMap::default();
            for &binding in &returned.external_contexts {
                if let Some(values) = callable.context_values.get(&binding) {
                    substitutions.insert(binding, values.clone());
                }
            }
            if let Some(arguments) = arguments {
                for (index, &(parameter, is_rest)) in returned.params.iter().enumerate() {
                    if is_rest {
                        let mut elements = Properties::default();
                        let mut uncertain_index = false;
                        for (index, argument) in
                            arguments[index.min(arguments.len())..].iter().enumerate()
                        {
                            match argument {
                                PlaceOrSpread::Place(argument) => {
                                    elements
                                        .entry((!uncertain_index).then(|| index.to_string()))
                                        .or_default()
                                        .extend(&self.values(argument.identifier));
                                }
                                PlaceOrSpread::Spread(spread) => {
                                    uncertain_index = true;
                                    for values in
                                        self.properties(spread.place.identifier).into_values()
                                    {
                                        elements.entry(None).or_default().extend(&values);
                                    }
                                }
                            }
                        }
                        substitutions.insert(
                            parameter,
                            elements
                                .values()
                                .flat_map(|value| value.identities.iter().copied())
                                .collect(),
                        );
                        rest_arguments.insert(parameter, elements);
                    } else {
                        let Some(PlaceOrSpread::Place(argument)) = arguments.get(index) else {
                            break;
                        };
                        substitutions
                            .insert(parameter, self.values(argument.identifier).identities);
                    }
                }
            } else if let Some(arguments) = &applied_arguments {
                for (index, &(parameter, is_rest)) in returned.params.iter().enumerate() {
                    if is_rest {
                        let elements: Properties = arguments
                            .iter()
                            .filter_map(|(key, value)| {
                                let key = match key {
                                    Some(key) => Some(
                                        key.parse::<usize>().ok()?.checked_sub(index)?.to_string(),
                                    ),
                                    None => None,
                                };
                                Some((key, value.clone()))
                            })
                            .collect();
                        substitutions.insert(
                            parameter,
                            elements
                                .values()
                                .flat_map(|value| value.identities.iter().copied())
                                .collect(),
                        );
                        rest_arguments.insert(parameter, elements);
                        continue;
                    }
                    if let Some(values) =
                        arguments.get(&Some(index.to_string())).or_else(|| arguments.get(&None))
                    {
                        substitutions.insert(parameter, values.identities.clone());
                    }
                }
            }
            // Returning a parameter or outer binding snapshots its selected
            // callable, whereas a newly returned closure retains live captures.
            for identity in &returned.values.identities {
                if let Some(selected) = substitutions.get(identity) {
                    values.identities.remove(identity);
                    for &selected in selected {
                        values.extend(&self.values(selected));
                    }
                }
            }
            for closure in values.contexts.values_mut() {
                for (&binding, values) in Rc::make_mut(&mut closure.context_values).iter_mut() {
                    if returned.external_contexts.contains(&binding)
                        && let Some(current) = substitutions.get(&binding)
                    {
                        *values = current.clone();
                    } else {
                        *values = values
                            .iter()
                            .flat_map(|value| {
                                if let Some(elements) = rest_arguments.get(value)
                                    && let Some(members) = closure.capture_members.get(&binding)
                                    && members.iter().all(|member| member.parse::<usize>().is_ok())
                                {
                                    return members
                                        .iter()
                                        .flat_map(|member| {
                                            elements
                                                .get(&Some(member.clone()))
                                                .into_iter()
                                                .chain(elements.get(&None))
                                        })
                                        .flat_map(|value| value.identities.iter().copied())
                                        .collect();
                                }
                                substitutions
                                    .get(value)
                                    .cloned()
                                    .unwrap_or_else(|| FxHashSet::from_iter([*value]))
                            })
                            .collect();
                    }
                }
            }
            result.extend(&values);
        }
        self.values.insert(instruction.lvalue.identifier, result);
    }

    fn store(&mut self, object: IdentifierId, key: Option<String>, values: Values) {
        let objects = self.values(object);
        let definite = objects.identities.len() == 1 && key.is_some();
        for object in objects.identities {
            let properties = self.properties.entry(object).or_default();
            if key.is_none() {
                for property in properties.values_mut() {
                    property.extend(&values);
                }
                properties.entry(None).or_default().extend(&values);
            } else if definite {
                properties.insert(key.clone(), values.clone());
            } else {
                let fallback = properties.get(&None).cloned().unwrap_or_default();
                properties.entry(key.clone()).or_insert(fallback).extend(&values);
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
        function_returns: &FxHashMap<IdentifierId, ReturnedCallables>,
        captured_bindings: &FxHashSet<IdentifierId>,
    ) {
        let id = instruction.lvalue.identifier;
        match &instruction.value {
            InstructionValue::FunctionExpression { .. } | InstructionValue::ObjectMethod { .. } => {
                self.allocate(id);
                let mut contexts = self.context_values.clone();
                for &capture in captured_contexts.get(&id).into_iter().flatten() {
                    contexts.entry(capture).or_insert_with(|| FxHashSet::from_iter([capture]));
                }
                self.values.get_mut(&id).unwrap().contexts.insert(
                    id,
                    Callable {
                        is_async: async_functions.contains(&id),
                        captures: captured_contexts.get(&id).cloned().unwrap_or_default(),
                        context_values: Rc::new(contexts),
                        returned: function_returns.get(&id).cloned().map(Rc::new),
                        capture_members: function_returns
                            .get(&id)
                            .map(|summary| summary.capture_members.clone())
                            .unwrap_or_default(),
                    },
                );
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
                if matches!(instruction.value, InstructionValue::StoreContext { .. })
                    || captured_bindings.contains(&lvalue.place.identifier)
                {
                    self.update_context(lvalue.place.identifier);
                }
            }
            InstructionValue::DeclareContext { lvalue, .. } => {
                if lvalue.kind == InstructionKind::HoistedFunction
                    && let Some(value) = hoisted.get(&lvalue.place.identifier)
                {
                    let mut values = Values::single(*value);
                    values.contexts.insert(
                        *value,
                        Callable {
                            is_async: async_functions.contains(value),
                            captures: captured_contexts.get(value).cloned().unwrap_or_default(),
                            context_values: Rc::new(self.context_values.clone()),
                            returned: function_returns.get(value).cloned().map(Rc::new),
                            capture_members: function_returns
                                .get(value)
                                .map(|summary| summary.capture_members.clone())
                                .unwrap_or_default(),
                        },
                    );
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
                            let mut properties = self.properties(spread.place.identifier);
                            if let Some(values) = properties.remove(&None) {
                                self.store(id, None, values);
                            }
                            for (key, values) in properties {
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
            InstructionValue::CallExpression { .. }
            | InstructionValue::NewExpression { .. }
            | InstructionValue::MethodCall { .. }
            | InstructionValue::TaggedTemplateExpression { .. } => self.call_result(instruction),
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
    function_returns: &FxHashMap<IdentifierId, ReturnedCallables>,
    env: &Environment<'_>,
) -> AsyncCallables {
    let mut result = AsyncCallables {
        identifiers: FxHashSet::default(),
        invocations: FxHashMap::default(),
        returned: ReturnedCallables::default(),
    };
    let captured_bindings = captured_contexts.values().flatten().copied().collect();
    let mut states: FxHashMap<BlockId, CallableState> = FxHashMap::default();
    loop {
        let mut changed = false;
        for (&block_id, block) in &func.body.blocks {
            let mut predecessors =
                block.preds.iter().filter_map(|predecessor| states.get(predecessor));
            let mut state = predecessors.next().cloned().unwrap_or_else(|| {
                let mut state = CallableState::default();
                for place in
                    func.context.iter().chain(func.params.iter().map(|param| match param {
                        ParamPattern::Place(place) => place,
                        ParamPattern::Spread(spread) => &spread.place,
                    }))
                {
                    state.allocate(place.identifier);
                    state
                        .context_values
                        .insert(place.identifier, FxHashSet::from_iter([place.identifier]));
                }
                state
            });
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
                let mut invoked = state.invoked(&instruction.value);
                for operand in each_invoked_async_operand(&instruction.value, env) {
                    invoked.extend(&state.values(operand.identifier));
                }
                for callable in invoked.contexts.into_values() {
                    if callable.is_async {
                        invocations.push(AsyncInvocation {
                            captures: callable.captures,
                            context_values: callable.context_values,
                        });
                    }
                }
                result.invocations.insert(instruction_id, invocations);
                state.apply(
                    instruction,
                    hoisted,
                    async_functions,
                    captured_contexts,
                    function_returns,
                    &captured_bindings,
                );
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
            if values.contexts.values().any(|callable| callable.is_async) {
                result.identifiers.insert(identifier);
            }
        }
    }
    for (&block_id, block) in &func.body.blocks {
        if let Terminal::Return { value, .. } = &block.terminal {
            result.returned.values.extend(&states[&block_id].values(value.identifier));
        }
    }
    result.returned.external_contexts.extend(func.context.iter().map(|place| place.identifier));
    result.returned.params.extend(func.params.iter().map(|param| match param {
        ParamPattern::Place(place) => (place.identifier, false),
        ParamPattern::Spread(spread) => (spread.place.identifier, true),
    }));
    result.returned.capture_members = infer_capture_members(func, env);
    result
}
