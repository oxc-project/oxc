use std::rc::Rc;

use rustc_hash::{FxHashMap, FxHashSet};

use super::validate_locals_not_reassigned_after_render::no_alias_callback_parameters;
use crate::react_compiler_hir::environment::Environment;
use crate::react_compiler_hir::visitors::{each_instruction_value_operand, each_terminal_operand};
use crate::react_compiler_hir::{
    ArrayElement, ArrayPatternElement, BlockId, HirFunction, IdentifierId, Instruction,
    InstructionId, InstructionKind, InstructionValue, ObjectPropertyKey, ObjectPropertyOrSpread,
    ParamPattern, Pattern, PlaceOrSpread, PrimitiveValue, Terminal,
};

type Captures = FxHashMap<IdentifierId, FxHashSet<IdentifierId>>;
type Properties = FxHashMap<Option<String>, Values>;

#[derive(Clone, PartialEq, Eq, Hash)]
struct Projection {
    object: IdentifierId,
    key: Option<String>,
}

// Parameter member reads retain their source until a caller supplies the object.
// Instruction identities make this graph finite even for cyclic property walks.
type Projections = FxHashMap<IdentifierId, FxHashSet<Projection>>;

#[derive(Clone, PartialEq, Eq)]
struct Arguments {
    elements: Properties,
    length: Option<usize>,
}

impl Default for Arguments {
    fn default() -> Self {
        Self { elements: Properties::default(), length: Some(0) }
    }
}

impl Arguments {
    fn append(&mut self, other: &Self) {
        for (key, values) in &other.elements {
            let key = self.length.and_then(|offset| {
                key.as_ref()?.parse::<usize>().ok()?.checked_add(offset).map(|i| i.to_string())
            });
            self.elements.entry(key).or_default().extend(values);
        }
        self.length = self.length.zip(other.length).and_then(|(a, b)| a.checked_add(b));
    }

    fn merge(&mut self, other: &Self) {
        merge_properties(&mut self.elements, &other.elements);
        if self.length != other.length {
            self.length = None;
        }
    }

    fn skip(mut self, count: usize) -> Self {
        self.elements = self
            .elements
            .into_iter()
            .filter_map(|(key, value)| {
                let key = match key {
                    Some(key) => Some(key.parse::<usize>().ok()?.checked_sub(count)?.to_string()),
                    None => None,
                };
                Some((key, value))
            })
            .collect();
        self.length = self.length.map(|length| length.saturating_sub(count));
        self
    }
}

/// Track both sides of suspension: an eager-only read cannot observe a later
/// assignment, while a deferred read must be checked when the caller yields.
fn infer_capture_phases(
    func: &HirFunction,
    invocations: &FxHashMap<InstructionId, Vec<AsyncInvocation>>,
) -> (FxHashSet<IdentifierId>, FxHashSet<IdentifierId>) {
    const EAGER: u8 = 1;
    const DEFERRED: u8 = 2;
    let contexts: FxHashSet<_> = func.context.iter().map(|place| place.identifier).collect();
    let mut eager = FxHashSet::default();
    let mut deferred = FxHashSet::default();
    if contexts.is_empty() {
        return (eager, deferred);
    }
    let mut phases = FxHashMap::default();
    loop {
        let mut changed = false;
        for (&id, block) in &func.body.blocks {
            let mut phase = if id == func.body.entry { EAGER } else { 0 };
            for predecessor in &block.preds {
                phase |= phases.get(predecessor).copied().unwrap_or_default();
            }
            if phase == 0 {
                continue;
            }
            for &id in &block.instructions {
                for invocation in invocations.get(&id).into_iter().flatten() {
                    if phase & EAGER != 0 {
                        eager.extend(invocation.eager_captures.intersection(&contexts).copied());
                    }
                    if phase & DEFERRED != 0 {
                        deferred.extend(invocation.eager_captures.intersection(&contexts).copied());
                    }
                    deferred.extend(invocation.deferred_captures.intersection(&contexts).copied());
                }
                match &func.instructions[id.index()].value {
                    InstructionValue::Await { .. } => phase = DEFERRED,
                    InstructionValue::LoadContext { place, .. }
                    | InstructionValue::LoadLocal { place, .. }
                        if contexts.contains(&place.identifier) =>
                    {
                        if phase & EAGER != 0 {
                            eager.insert(place.identifier);
                        }
                        if phase & DEFERRED != 0 {
                            deferred.insert(place.identifier);
                        }
                    }
                    _ => {}
                }
            }
            if phases.get(&id) != Some(&phase) {
                phases.insert(id, phase);
                changed = true;
            }
        }
        if !changed {
            break;
        }
    }
    (eager, deferred)
}

/// Refine captures used only through named members. A direct escape, unknown
/// property, or nested closure keeps the whole capture. This lets a returned
/// closure select individual rest arguments without taint from unused siblings.
fn infer_capture_members(
    func: &HirFunction,
    env: &Environment<'_>,
    rest_arguments: bool,
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
            InstructionValue::PropertyLoad { object, property, .. } => {
                if !rest_arguments || property.to_string().parse::<usize>().is_ok() {
                    receivers.insert(instruction.lvalue.identifier, object.identifier);
                }
            }
            InstructionValue::ComputedLoad { object, property, .. }
                if !rest_arguments
                    || keys
                        .get(&property.identifier)
                        .is_some_and(|key| key.parse::<usize>().is_ok()) =>
            {
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
    pub deferred_captures: FxHashSet<IdentifierId>,
    pub eager_captures: FxHashSet<IdentifierId>,
    pub context_values: Rc<Captures>,
}

pub(super) struct AsyncCallables {
    pub identifiers: FxHashSet<IdentifierId>,
    pub invocations: FxHashMap<InstructionId, Vec<AsyncInvocation>>,
    pub returned: ReturnedCallables,
    pub pending_before: FxHashMap<InstructionId, AsyncInvocation>,
    pub pending_at_exit: FxHashMap<BlockId, AsyncInvocation>,
}

#[derive(Clone, Default, PartialEq, Eq)]
pub(super) struct ReturnedCallables {
    values: Values,
    external_contexts: FxHashSet<IdentifierId>,
    params: Vec<(IdentifierId, bool)>,
    capture_members: FxHashMap<IdentifierId, FxHashSet<String>>,
    capture_reads: FxHashMap<IdentifierId, FxHashSet<String>>,
    eager_captures: FxHashSet<IdentifierId>,
    deferred_captures: FxHashSet<IdentifierId>,
    projections: Projections,
}

#[derive(Clone, PartialEq, Eq)]
struct Callable {
    is_async: bool,
    deferred_captures: FxHashSet<IdentifierId>,
    context_values: Rc<Captures>,
    returned: Option<Rc<ReturnedCallables>>,
    capture_members: FxHashMap<IdentifierId, FxHashSet<String>>,
    capture_reads: FxHashMap<IdentifierId, FxHashSet<String>>,
    eager_captures: FxHashSet<IdentifierId>,
    bound_arguments: Arguments,
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
            into.bound_arguments.merge(&callable.bound_arguments);
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
struct PendingCapture {
    roots: FxHashSet<IdentifierId>,
    members: Option<FxHashSet<String>>,
    observed: FxHashSet<IdentifierId>,
    dependencies: FxHashMap<IdentifierId, Option<FxHashSet<String>>>,
}

fn merge_members(into: &mut Option<FxHashSet<String>>, other: &Option<FxHashSet<String>>) {
    if let Some(into_members) = into {
        if let Some(other) = other {
            into_members.extend(other.iter().cloned());
        } else {
            *into = None;
        }
    }
}

impl PendingCapture {
    fn merge(&mut self, other: &Self) {
        self.roots.extend(&other.roots);
        merge_members(&mut self.members, &other.members);
        self.observed.extend(&other.observed);
        for (&id, members) in &other.dependencies {
            if let Some(into) = self.dependencies.get_mut(&id) {
                merge_members(into, members);
            } else {
                self.dependencies.insert(id, members.clone());
            }
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
    array_lengths: FxHashMap<IdentifierId, usize>,
    arrays: FxHashSet<IdentifierId>,
    projections: Projections,
    pending: FxHashMap<IdentifierId, PendingCapture>,
}

impl CallableState {
    fn refresh_capture(&self, capture: &mut PendingCapture) {
        capture.observed.clear();
        capture.dependencies.clear();
        let mut pending: Vec<_> =
            capture.roots.iter().map(|&id| (id, capture.members.clone())).collect();
        while let Some((id, mut members)) = pending.pop() {
            if self.arrays.contains(&id)
                && members.as_ref().is_some_and(|members| {
                    members.iter().any(|member| {
                        matches!(
                            member.as_str(),
                            "forEach"
                                | "map"
                                | "flatMap"
                                | "filter"
                                | "find"
                                | "findIndex"
                                | "findLast"
                                | "findLastIndex"
                                | "some"
                                | "every"
                                | "reduce"
                                | "reduceRight"
                                | "sort"
                                | "toSorted"
                        ) && self.properties.get(&id).is_some_and(|properties| {
                            !properties.contains_key(&Some(member.clone()))
                        })
                    })
                })
            {
                // Inherited array callback consumers can pass any element to
                // their callback. An own method still selects only that field.
                members = None;
            }
            if let Some(existing) = capture.dependencies.get_mut(&id) {
                let old = existing.clone();
                merge_members(existing, &members);
                if *existing == old {
                    continue;
                }
            } else {
                capture.dependencies.insert(id, members.clone());
            }
            let callable = self.values.get(&id).filter(|values| !values.contexts.is_empty());
            if members.is_none()
                && let Some(callable) = callable
            {
                capture.observed.insert(id);
                capture.observed.extend(callable.contexts.keys().copied());
                continue;
            }
            let Some(properties) = self.properties.get(&id) else {
                capture.observed.insert(id);
                continue;
            };
            let mut add = |values: &Values| {
                pending.extend(values.identities.iter().map(|&id| (id, None)));
            };
            if let Some(members) = &members {
                for member in members {
                    if let Some(values) =
                        properties.get(&Some(member.clone())).or_else(|| properties.get(&None))
                    {
                        add(values);
                    }
                }
            } else {
                for values in properties.values() {
                    add(values);
                }
            }
        }
    }

    fn capture(&self, binding: IdentifierId, callable: &Callable) -> PendingCapture {
        let mut capture = PendingCapture {
            roots: callable
                .context_values
                .get(&binding)
                .cloned()
                .unwrap_or_else(|| FxHashSet::from_iter([binding])),
            members: callable.capture_reads.get(&binding).cloned(),
            ..PendingCapture::default()
        };
        self.refresh_capture(&mut capture);
        capture
    }

    fn invocation(&mut self, callable: Callable) -> AsyncInvocation {
        let mut contexts = (*callable.context_values).clone();
        for &binding in &callable.eager_captures {
            contexts.insert(binding, self.capture(binding, &callable).observed);
        }
        for &binding in &callable.deferred_captures {
            let capture = self.capture(binding, &callable);
            if let Some(current) = self.pending.get_mut(&binding) {
                current.merge(&capture);
            } else {
                self.pending.insert(binding, capture);
            }
        }
        AsyncInvocation {
            eager_captures: callable.eager_captures,
            deferred_captures: callable.deferred_captures,
            context_values: Rc::new(contexts),
        }
    }

    fn pending_observation(&self) -> AsyncInvocation {
        let mut contexts = self.context_values.clone();
        for (&binding, capture) in &self.pending {
            contexts.insert(binding, capture.observed.clone());
        }
        AsyncInvocation {
            eager_captures: FxHashSet::default(),
            deferred_captures: self.pending.keys().copied().collect(),
            context_values: Rc::new(contexts),
        }
    }

    fn invoked_callbacks(
        &self,
        instruction: &InstructionValue<'_>,
        env: &Environment<'_>,
    ) -> Values {
        let args = match instruction {
            InstructionValue::CallExpression { args, .. }
            | InstructionValue::MethodCall { args, .. } => args,
            _ => return Values::default(),
        };
        let arguments = self.arguments(args);
        let count = arguments.length.unwrap_or_else(|| {
            arguments
                .elements
                .keys()
                .filter_map(|key| key.as_ref()?.parse::<usize>().ok())
                .max()
                .unwrap_or(0)
                + 1
        });
        let mut result = Values::default();
        for index in no_alias_callback_parameters(instruction, env, false, count) {
            if let Some(values) = arguments.elements.get(&Some(index.to_string())) {
                result.extend(values);
            }
            if let Some(values) = arguments.elements.get(&None) {
                result.extend(values);
            }
        }
        result
    }

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
        if let Some(mut capture) = self.pending.remove(&binding) {
            capture.roots.clone_from(&values);
            self.refresh_capture(&mut capture);
            self.pending.insert(binding, capture);
        }
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
        for (&binding, capture) in &other.pending {
            if let Some(into) = self.pending.get_mut(&binding) {
                into.merge(capture);
            } else {
                self.pending.insert(binding, capture.clone());
            }
        }
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
        self.array_lengths.retain(|id, length| other.array_lengths.get(id) == Some(length));
        self.arrays.extend(&other.arrays);
        for (&id, projections) in &other.projections {
            self.projections.entry(id).or_default().extend(projections.iter().cloned());
        }
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
        let mut values = if key.is_some() {
            properties.get(&key).or_else(|| properties.get(&None)).cloned().unwrap_or_default()
        } else {
            properties.into_values().fold(Values::default(), |mut result, values| {
                result.extend(&values);
                result
            })
        };
        for object in self.values(object).identities {
            if self.projections.contains_key(&object) {
                self.projections
                    .entry(into)
                    .or_default()
                    .insert(Projection { object, key: key.clone() });
                values.identities.insert(into);
            }
        }
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

    fn array_length(&self, values: &Values) -> Option<usize> {
        let mut lengths = values.identities.iter().map(|id| self.array_lengths.get(id).copied());
        let length = lengths.next()??;
        lengths.all(|other| other == Some(length)).then_some(length)
    }

    fn arguments(&self, args: &[PlaceOrSpread]) -> Arguments {
        let mut result = Arguments::default();
        for argument in args {
            let next = match argument {
                PlaceOrSpread::Place(place) => Arguments {
                    elements: FxHashMap::from_iter([(
                        Some("0".to_string()),
                        self.values(place.identifier),
                    )]),
                    length: Some(1),
                },
                PlaceOrSpread::Spread(spread) => Arguments {
                    elements: self.properties(spread.place.identifier),
                    length: self.array_length(&self.values(spread.place.identifier)),
                },
            };
            result.append(&next);
        }
        result
    }

    fn project(&self, objects: &Values, key: &Option<String>) -> Values {
        let mut result = Values::default();
        for object in &objects.identities {
            let Some(properties) = self.properties.get(object) else { continue };
            if key.is_some() {
                if let Some(values) = properties.get(key).or_else(|| properties.get(&None)) {
                    result.extend(values);
                }
            } else {
                for values in properties.values() {
                    result.extend(values);
                }
            }
        }
        result
    }

    fn substitute(
        &self,
        id: IdentifierId,
        substitutions: &FxHashMap<IdentifierId, Values>,
        projections: &Projections,
        rest_arguments: &FxHashMap<IdentifierId, Properties>,
        visiting: &mut FxHashSet<IdentifierId>,
    ) -> Option<Values> {
        if let Some(values) = substitutions.get(&id) {
            return Some(values.clone());
        }
        let sources = projections.get(&id)?;
        if sources.is_empty() || !visiting.insert(id) {
            return None;
        }
        let mut result = Values::default();
        for source in sources {
            if let Some(elements) = rest_arguments.get(&source.object) {
                if source.key.is_some() {
                    if let Some(values) = elements.get(&source.key).or_else(|| elements.get(&None))
                    {
                        result.extend(values);
                    }
                } else {
                    for values in elements.values() {
                        result.extend(values);
                    }
                }
                continue;
            }
            if let Some(objects) =
                self.substitute(source.object, substitutions, projections, rest_arguments, visiting)
            {
                result.extend(&self.project(&objects, &source.key));
            } else {
                result.identities.insert(id);
            }
        }
        visiting.remove(&id);
        Some(result)
    }

    fn call_result(&mut self, instruction: &Instruction<'_>) {
        let mut binding = None;
        let arguments = match &instruction.value {
            InstructionValue::CallExpression { args, .. }
            | InstructionValue::NewExpression { args, .. } => self.arguments(args),
            InstructionValue::MethodCall { receiver, property, args, .. } => {
                match self.builtin_function_methods.get(&property.identifier).map(String::as_str) {
                    Some("bind") => {
                        binding = Some(receiver.identifier);
                        self.arguments(args).skip(1)
                    }
                    Some("call") => self.arguments(args).skip(1),
                    Some("apply") => args.get(1).map_or_else(Arguments::default, |argument| {
                        let place = match argument {
                            PlaceOrSpread::Place(place) => place,
                            PlaceOrSpread::Spread(spread) => &spread.place,
                        };
                        Arguments {
                            elements: self.properties(place.identifier),
                            length: self.array_length(&self.values(place.identifier)),
                        }
                    }),
                    _ => self.arguments(args),
                }
            }
            _ => Arguments::default(),
        };
        if let Some(receiver) = binding {
            let mut values = self.values(receiver);
            // Bound arguments snapshot their values. Keep identities here so
            // binding a function to itself cannot build recursive summaries.
            let mut arguments = arguments;
            for values in arguments.elements.values_mut() {
                values.contexts.clear();
            }
            for callable in values.contexts.values_mut() {
                callable.bound_arguments.append(&arguments);
            }
            values.identities = FxHashSet::from_iter([instruction.lvalue.identifier]);
            self.values.insert(instruction.lvalue.identifier, values);
            return;
        }
        let mut result = Values::default();
        for callable in self.invoked(&instruction.value).contexts.into_values() {
            // Async calls produce promises, not the eventual returned callable.
            if callable.is_async {
                continue;
            }
            let Some(returned) = &callable.returned else { continue };
            let mut arguments = {
                let mut bound = callable.bound_arguments.clone();
                bound.append(&arguments);
                bound
            };
            for values in arguments.elements.values_mut() {
                for identity in values.identities.clone() {
                    values.extend(&self.values(identity));
                }
            }
            let mut values = returned.values.clone();
            let mut substitutions = FxHashMap::default();
            let mut rest_arguments = FxHashMap::default();
            for &binding in &returned.external_contexts {
                if let Some(identities) = callable.context_values.get(&binding) {
                    let mut values = Values::default();
                    values.identities.clone_from(identities);
                    for &id in identities {
                        values.extend(&self.values(id));
                    }
                    substitutions.insert(binding, values);
                }
            }
            for (index, &(parameter, is_rest)) in returned.params.iter().enumerate() {
                if is_rest {
                    let elements: Properties = arguments
                        .elements
                        .iter()
                        .filter_map(|(key, value)| {
                            let key = match key {
                                Some(key) => {
                                    Some(key.parse::<usize>().ok()?.checked_sub(index)?.to_string())
                                }
                                None => None,
                            };
                            Some((key, value.clone()))
                        })
                        .collect();
                    let mut values = Values::default();
                    for element in elements.values() {
                        values.extend(element);
                    }
                    substitutions.insert(parameter, values);
                    rest_arguments.insert(parameter, elements);
                } else {
                    let mut values = arguments
                        .elements
                        .get(&Some(index.to_string()))
                        .cloned()
                        .unwrap_or_default();
                    if let Some(unknown) = arguments.elements.get(&None) {
                        values.extend(unknown);
                    }
                    substitutions.insert(parameter, values);
                }
            }
            // Returning a parameter snapshots the selected callable; a newly
            // returned closure retains the caller's live binding information.
            for identity in &returned.values.identities {
                if let Some(selected) = self.substitute(
                    *identity,
                    &substitutions,
                    &returned.projections,
                    &rest_arguments,
                    &mut FxHashSet::default(),
                ) {
                    values.identities.remove(identity);
                    values.extend(&selected);
                }
            }
            for closure in values.contexts.values_mut() {
                for (&binding, values) in Rc::make_mut(&mut closure.context_values).iter_mut() {
                    if returned.external_contexts.contains(&binding)
                        && let Some(current) = substitutions.get(&binding)
                    {
                        *values = current.identities.clone();
                    } else {
                        *values = values
                            .iter()
                            .flat_map(|value| {
                                if let Some(elements) = rest_arguments.get(value)
                                    && let Some(members) = closure.capture_members.get(&binding)
                                {
                                    return members
                                        .iter()
                                        .filter(|member| member.parse::<usize>().is_ok())
                                        .flat_map(|member| {
                                            elements
                                                .get(&Some(member.clone()))
                                                .into_iter()
                                                .chain(elements.get(&None))
                                        })
                                        .flat_map(|value| value.identities.iter().copied())
                                        .collect();
                                }
                                self.substitute(
                                    *value,
                                    &substitutions,
                                    &returned.projections,
                                    &rest_arguments,
                                    &mut FxHashSet::default(),
                                )
                                .map_or_else(
                                    || FxHashSet::from_iter([*value]),
                                    |values| values.identities,
                                )
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
        for &object in &objects.identities {
            self.array_lengths.remove(&object);
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
        // Update only pending reads touched by this write. Recomputing every
        // capture after a CFG join would mix in stores from non-invoking paths.
        let affected: Vec<_> = self
            .pending
            .iter()
            .filter_map(|(&binding, capture)| {
                objects
                    .identities
                    .iter()
                    .any(|object| {
                        capture.dependencies.get(object).is_some_and(|members| {
                            key.as_ref().is_none_or(|key| {
                                members.as_ref().is_none_or(|members| members.contains(key))
                            })
                        })
                    })
                    .then_some(binding)
            })
            .collect();
        for binding in affected {
            let mut capture = self.pending.remove(&binding).unwrap();
            self.refresh_capture(&mut capture);
            self.pending.insert(binding, capture);
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
                        deferred_captures: function_returns
                            .get(&id)
                            .map(|summary| summary.deferred_captures.clone())
                            .unwrap_or_default(),
                        context_values: Rc::new(contexts),
                        returned: function_returns.get(&id).cloned().map(Rc::new),
                        bound_arguments: Arguments::default(),
                        eager_captures: function_returns
                            .get(&id)
                            .map(|summary| summary.eager_captures.clone())
                            .unwrap_or_default(),
                        capture_reads: function_returns
                            .get(&id)
                            .map(|summary| summary.capture_reads.clone())
                            .unwrap_or_default(),
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
                            deferred_captures: function_returns
                                .get(value)
                                .map(|summary| summary.deferred_captures.clone())
                                .unwrap_or_default(),
                            context_values: Rc::new(self.context_values.clone()),
                            returned: function_returns.get(value).cloned().map(Rc::new),
                            bound_arguments: Arguments::default(),
                            eager_captures: function_returns
                                .get(value)
                                .map(|summary| summary.eager_captures.clone())
                                .unwrap_or_default(),
                            capture_reads: function_returns
                                .get(value)
                                .map(|summary| summary.capture_reads.clone())
                                .unwrap_or_default(),
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
                self.arrays.insert(id);
                let mut contents = Arguments::default();
                for element in elements {
                    let next = match element {
                        ArrayElement::Place(place) => Arguments {
                            elements: FxHashMap::from_iter([(
                                Some("0".to_string()),
                                self.values(place.identifier),
                            )]),
                            length: Some(1),
                        },
                        ArrayElement::Spread(spread) => Arguments {
                            elements: self.properties(spread.place.identifier),
                            length: self.array_length(&self.values(spread.place.identifier)),
                        },
                        ArrayElement::Hole => {
                            Arguments { elements: Properties::default(), length: Some(1) }
                        }
                    };
                    contents.append(&next);
                }
                self.properties.insert(id, contents.elements);
                if let Some(length) = contents.length {
                    self.array_lengths.insert(id, length);
                } else {
                    self.array_lengths.remove(&id);
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
        pending_before: FxHashMap::default(),
        pending_at_exit: FxHashMap::default(),
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
                    state.projections.entry(place.identifier).or_default();
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
                invoked.extend(&state.invoked_callbacks(&instruction.value, env));
                for callable in invoked.contexts.into_values() {
                    if callable.is_async {
                        invocations.push(state.invocation(callable));
                    }
                }
                result.invocations.insert(instruction_id, invocations);
                if matches!(instruction.value, InstructionValue::Await { .. }) {
                    result.pending_before.insert(instruction_id, state.pending_observation());
                }
                state.apply(
                    instruction,
                    hoisted,
                    async_functions,
                    captured_contexts,
                    function_returns,
                    &captured_bindings,
                );
            }
            if matches!(block.terminal, Terminal::Return { .. } | Terminal::Throw { .. }) {
                result.pending_at_exit.insert(block_id, state.pending_observation());
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
    result.returned.capture_members = infer_capture_members(func, env, true);
    result.returned.capture_reads = infer_capture_members(func, env, false);
    (result.returned.eager_captures, result.returned.deferred_captures) =
        infer_capture_phases(func, &result.invocations);
    for projections in states
        .values()
        .map(|state| &state.projections)
        .chain(function_returns.values().map(|summary| &summary.projections))
    {
        for (&id, sources) in projections {
            result.returned.projections.entry(id).or_default().extend(sources.iter().cloned());
        }
    }
    result
}
