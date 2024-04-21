use std::{collections::HashSet, marker::PhantomData, path::PathBuf, sync::Arc};

use oxc_span::CompactStr;

use crate::module_record::ModuleRecord;

pub struct ModuleGraphVisitorBuilder<'a, T> {
    max_depth: u32,
    filter: Option<Box<dyn Fn((&CompactStr, &Arc<ModuleRecord>), &ModuleRecord) -> bool + 'a>>,
    event: Option<
        Box<
            dyn FnMut(ModuleGraphVisitorEvent, (&CompactStr, &Arc<ModuleRecord>), &ModuleRecord)
                + 'a,
        >,
    >,
    enter: Option<Box<dyn FnMut((&CompactStr, &Arc<ModuleRecord>), &ModuleRecord) + 'a>>,
    leave: Option<Box<dyn FnMut((&CompactStr, &Arc<ModuleRecord>), &ModuleRecord) + 'a>>,
    _marker: PhantomData<T>,
}

pub enum VisitFoldWhile<T> {
    Stop(T),
    Next(T),
}

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
    pub fn max_depth(mut self, max_depth: u32) -> Self {
        self.max_depth = max_depth;
        self
    }

    pub fn filter<F: (Fn((&CompactStr, &Arc<ModuleRecord>), &ModuleRecord) -> bool) + 'a>(
        mut self,
        filter: F,
    ) -> Self {
        self.filter = Some(Box::new(filter));
        self
    }

    pub fn event<
        F: FnMut(ModuleGraphVisitorEvent, (&CompactStr, &Arc<ModuleRecord>), &ModuleRecord) + 'a,
    >(
        mut self,
        event: F,
    ) -> Self {
        self.event = Some(Box::new(event));
        self
    }

    pub fn enter<F: FnMut((&CompactStr, &Arc<ModuleRecord>), &ModuleRecord) + 'a>(
        mut self,
        enter: F,
    ) -> Self {
        self.enter = Some(Box::new(enter));
        self
    }

    pub fn leave<F: FnMut((&CompactStr, &Arc<ModuleRecord>), &ModuleRecord) + 'a>(
        mut self,
        leave: F,
    ) -> Self {
        self.leave = Some(Box::new(leave));
        self
    }

    pub fn visit_fold<
        V: Fn(T, (&CompactStr, &Arc<ModuleRecord>), &ModuleRecord) -> VisitFoldWhile<T>,
    >(
        self,
        initial_value: T,
        module: &ModuleRecord,
        visit: V,
    ) -> ModuleGraphVisitResult<T> {
        let mut visitor =
            ModuleGraphVisitor { traversed: HashSet::new(), depth: 0, max_depth: self.max_depth };
        let filter = self.filter.unwrap_or_else(|| Box::new(|_, _| true));
        let event = self.event.unwrap_or_else(|| Box::new(|_, _, _| {}));
        let enter = self.enter.unwrap_or_else(|| Box::new(|_, _| {}));
        let leave = self.leave.unwrap_or_else(|| Box::new(|_, _| {}));
        let result =
            visitor.filter_fold_while(initial_value, module, filter, visit, event, enter, leave);

        ModuleGraphVisitResult::with_result(result, visitor)
    }
}

impl<'a, T> Default for ModuleGraphVisitorBuilder<'a, T> {
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
    pub traversed: HashSet<PathBuf>,
    pub max_depth: u32,
}

impl<T> ModuleGraphVisitResult<T> {
    fn with_result(result: T, visitor: ModuleGraphVisitor) -> Self {
        Self { result, traversed: visitor.traversed, max_depth: visitor.max_depth }
    }
}

#[derive(Debug)]
struct ModuleGraphVisitor {
    traversed: HashSet<PathBuf>,
    depth: u32,
    max_depth: u32,
}

impl ModuleGraphVisitor {
    pub(self) fn filter_fold_while<
        T,
        Filter: Fn((&CompactStr, &Arc<ModuleRecord>), &ModuleRecord) -> bool,
        Fold: FnMut(T, (&CompactStr, &Arc<ModuleRecord>), &ModuleRecord) -> VisitFoldWhile<T>,
        EventMod: FnMut(ModuleGraphVisitorEvent, (&CompactStr, &Arc<ModuleRecord>), &ModuleRecord),
        EnterMod: FnMut((&CompactStr, &Arc<ModuleRecord>), &ModuleRecord),
        LeaveMod: FnMut((&CompactStr, &Arc<ModuleRecord>), &ModuleRecord),
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
        let x = self
            .filter_fold_recursive(
                VisitFoldWhile::Next(initial_value),
                module_record,
                &filter,
                &mut fold,
                &mut event,
                &mut enter,
                &mut leave,
            )
            .into_inner();
        x
    }

    #[allow(clippy::too_many_arguments)]
    fn filter_fold_recursive<
        T,
        Filter: Fn((&CompactStr, &Arc<ModuleRecord>), &ModuleRecord) -> bool,
        Fold: FnMut(T, (&CompactStr, &Arc<ModuleRecord>), &ModuleRecord) -> VisitFoldWhile<T>,
        EventMod: FnMut(ModuleGraphVisitorEvent, (&CompactStr, &Arc<ModuleRecord>), &ModuleRecord),
        EnterMod: FnMut((&CompactStr, &Arc<ModuleRecord>), &ModuleRecord),
        LeaveMod: FnMut((&CompactStr, &Arc<ModuleRecord>), &ModuleRecord),
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
        macro_rules! accumulate {
            ($acc:expr) => {
                accumulator = $acc;

                if accumulator.is_done() {
                    break;
                }
            };
        }
        for module_record_ref in &module_record.loaded_modules {
            if self.depth > self.max_depth {
                return VisitFoldWhile::Stop(accumulator.into_inner());
            }

            let path = &module_record_ref.resolved_absolute_path;
            if !self.traversed.insert(path.clone()) {
                continue;
            }

            if !filter(module_record_ref.pair(), module_record) {
                continue;
            }

            self.depth += 1;

            event(ModuleGraphVisitorEvent::Enter, module_record_ref.pair(), module_record);
            enter(module_record_ref.pair(), module_record);

            accumulate!(fold(accumulator.into_inner(), module_record_ref.pair(), module_record));
            accumulate!(self.filter_fold_recursive(
                accumulator,
                module_record_ref.value(),
                filter,
                fold,
                event,
                enter,
                leave
            ));

            event(ModuleGraphVisitorEvent::Leave, module_record_ref.pair(), module_record);
            leave(module_record_ref.pair(), module_record);

            self.depth -= 1;
        }

        accumulator
    }
}
