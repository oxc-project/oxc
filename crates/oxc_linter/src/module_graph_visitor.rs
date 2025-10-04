use std::{
    marker::PhantomData,
    path::PathBuf,
    sync::{Arc, Weak},
};

use rustc_hash::FxHashSet;

use oxc_span::CompactStr;

use crate::ModuleRecord;

type ModulePair<'a> = (&'a CompactStr, &'a Arc<ModuleRecord>);

type FilterFn<'a> = dyn Fn(ModulePair, &ModuleRecord) -> bool + 'a;
type EventFn<'a> = dyn FnMut(ModuleGraphVisitorEvent, ModulePair, &ModuleRecord) + 'a;
type EnterFn<'a> = dyn FnMut(ModulePair, &ModuleRecord) + 'a;
type LeaveFn<'a> = dyn FnMut(ModulePair, &ModuleRecord) + 'a;

/// A builder for creating visitors that walk through the module graph
pub struct ModuleGraphVisitorBuilder<'a, T> {
    max_depth: u32,
    filter: Option<Box<FilterFn<'a>>>,
    event: Option<Box<EventFn<'a>>>,
    enter: Option<Box<EnterFn<'a>>>,
    leave: Option<Box<LeaveFn<'a>>>,
    _marker: PhantomData<T>,
}

/// Returning value of `visit_fold` closures, It will stop on returning a `Stop` variant,
/// Otherwise it would continue with the iteration.
pub enum VisitFoldWhile<T> {
    Stop(T),
    Next(T),
}

/// Indicates the type of event for `EventFn` closure.
pub enum ModuleGraphVisitorEvent {
    Enter,
    Leave,
}

impl<T> VisitFoldWhile<T> {
    pub fn is_done(&self) -> bool {
        matches!(self, Self::Stop(_))
    }

    pub fn into_inner(self) -> T {
        match self {
            Self::Stop(inner) | Self::Next(inner) => inner,
        }
    }
}

impl<'a, T> ModuleGraphVisitorBuilder<'a, T> {
    /// Sets the `self.max_depth`.
    #[must_use]
    pub fn max_depth(mut self, max_depth: u32) -> Self {
        self.max_depth = max_depth;
        self
    }

    /// Sets the filter closure.
    #[must_use]
    pub fn filter<F: (Fn(ModulePair, &ModuleRecord) -> bool) + 'a>(mut self, filter: F) -> Self {
        self.filter = Some(Box::new(filter));
        self
    }

    /// Sets the generic event closure.
    #[must_use]
    pub fn event<F: FnMut(ModuleGraphVisitorEvent, ModulePair, &ModuleRecord) + 'a>(
        mut self,
        event: F,
    ) -> Self {
        self.event = Some(Box::new(event));
        self
    }

    // /// Sets the enter module event closure.
    // #[must_use]
    // pub fn enter<F: FnMut(ModulePair, &ModuleRecord) + 'a>(mut self, enter: F) -> Self {
    // self.enter = Some(Box::new(enter));
    // self
    // }

    // /// Sets the leave module event closure.
    // #[must_use]
    // pub fn leave<F: FnMut(ModulePair, &ModuleRecord) + 'a>(mut self, leave: F) -> Self {
    // self.leave = Some(Box::new(leave));
    // self
    // }

    /// Behaves similar to a flat fold_while iteration.
    pub fn visit_fold<V: Fn(T, ModulePair, &ModuleRecord) -> VisitFoldWhile<T>>(
        self,
        initial_value: T,
        module: &ModuleRecord,
        visit: V,
    ) -> ModuleGraphVisitResult<T> {
        let mut visitor = ModuleGraphVisitor {
            traversed: FxHashSet::default(),
            depth: 0,
            max_depth: self.max_depth,
        };
        let filter = self.filter.unwrap_or_else(|| Box::new(|_, _| true));
        let event = self.event.unwrap_or_else(|| Box::new(|_, _, _| {}));
        let enter = self.enter.unwrap_or_else(|| Box::new(|_, _| {}));
        let leave = self.leave.unwrap_or_else(|| Box::new(|_, _| {}));
        let result =
            visitor.filter_fold_while(initial_value, module, filter, visit, event, enter, leave);

        ModuleGraphVisitResult::with_result(result, visitor)
    }
}

impl<T> Default for ModuleGraphVisitorBuilder<'_, T> {
    fn default() -> Self {
        Self {
            max_depth: u32::MAX,
            filter: None,
            event: None,
            enter: None,
            leave: None,
            _marker: std::marker::PhantomData {},
        }
    }
}

pub struct ModuleGraphVisitResult<T> {
    pub result: T,
    pub _traversed: FxHashSet<PathBuf>,
    pub _max_depth: u32,
}

impl<T> ModuleGraphVisitResult<T> {
    fn with_result(result: T, visitor: ModuleGraphVisitor) -> Self {
        Self { result, _traversed: visitor.traversed, _max_depth: visitor.max_depth }
    }
}

#[derive(Debug)]
struct ModuleGraphVisitor {
    traversed: FxHashSet<PathBuf>,
    depth: u32,
    max_depth: u32,
}

impl ModuleGraphVisitor {
    fn filter_fold_while<
        T,
        Filter: Fn(ModulePair, &ModuleRecord) -> bool,
        Fold: FnMut(T, ModulePair, &ModuleRecord) -> VisitFoldWhile<T>,
        EventMod: FnMut(ModuleGraphVisitorEvent, ModulePair, &ModuleRecord),
        EnterMod: FnMut(ModulePair, &ModuleRecord),
        LeaveMod: FnMut(ModulePair, &ModuleRecord),
    >(
        &mut self,
        initial_value: T,
        module_record: &ModuleRecord,
        filter: Filter,
        mut fold: Fold,
        mut event: EventMod,
        mut enter: EnterMod,
        mut leave: LeaveMod,
    ) -> T {
        self.filter_fold_recursive(
            VisitFoldWhile::Next(initial_value),
            module_record,
            &filter,
            &mut fold,
            &mut event,
            &mut enter,
            &mut leave,
        )
        .into_inner()
    }

    fn filter_fold_recursive<
        T,
        Filter: Fn(ModulePair, &ModuleRecord) -> bool,
        Fold: FnMut(T, ModulePair, &ModuleRecord) -> VisitFoldWhile<T>,
        EventMod: FnMut(ModuleGraphVisitorEvent, ModulePair, &ModuleRecord),
        EnterMod: FnMut(ModulePair, &ModuleRecord),
        LeaveMod: FnMut(ModulePair, &ModuleRecord),
    >(
        &mut self,
        mut accumulator: VisitFoldWhile<T>,
        module_record: &ModuleRecord,
        filter: &Filter,
        fold: &mut Fold,
        event: &mut EventMod,
        enter: &mut EnterMod,
        leave: &mut LeaveMod,
    ) -> VisitFoldWhile<T> {
        // Sort entries to ensure deterministic iteration order.
        // The module graph is populated via parallel insertion (par_drain in runtime.rs),
        // which causes non-deterministic insertion order into FxHashMap.
        // Different iteration orders can cause cycle detection to find or miss cycles
        // depending on which path reaches a node first (due to the `traversed` set).
        let mut entries: Vec<_> = module_record
            .loaded_modules()
            .iter()
            .map(|(k, v)| (k.clone(), Weak::clone(v)))
            .collect();
        entries.sort_unstable_by(|a, b| a.0.cmp(&b.0));

        for (key, weak_module_record) in entries {
            if self.depth > self.max_depth {
                return VisitFoldWhile::Stop(accumulator.into_inner());
            }

            let loaded_module_record = weak_module_record.upgrade().unwrap();

            let path = &loaded_module_record.resolved_absolute_path;

            if !self.traversed.insert(path.clone()) {
                continue;
            }

            let pair = (&key, &loaded_module_record);

            if !filter(pair, module_record) {
                continue;
            }

            self.depth += 1;

            event(ModuleGraphVisitorEvent::Enter, pair, module_record);
            enter(pair, module_record);

            accumulator = fold(accumulator.into_inner(), pair, module_record);
            if accumulator.is_done() {
                break;
            }

            accumulator =
                self.filter_fold_recursive(accumulator, pair.1, filter, fold, event, enter, leave);
            if accumulator.is_done() {
                break;
            }

            event(ModuleGraphVisitorEvent::Leave, pair, module_record);
            leave(pair, module_record);

            self.depth -= 1;
        }

        accumulator
    }
}
