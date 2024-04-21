use std::{collections::HashSet, marker::PhantomData, path::PathBuf, sync::Arc};

use oxc_span::CompactStr;

use crate::module_record::ModuleRecord;

pub struct ModuleGraphVisitorBuilder<T> {
    max_depth: u32,
    filter: Option<Box<dyn Fn((&CompactStr, &Arc<ModuleRecord>), &ModuleRecord) -> bool>>,
    enter: Option<Box<dyn Fn((&CompactStr, &Arc<ModuleRecord>), &ModuleRecord)>>,
    leave: Option<Box<dyn Fn((&CompactStr, &Arc<ModuleRecord>), &ModuleRecord)>>,
    _marker: PhantomData<T>,
}

pub enum VisitFoldWhile<T> {
    Stop(T),
    Next(T),
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

impl<T> ModuleGraphVisitorBuilder<T> {
    pub fn max_depth(mut self, max_depth: u32) -> Self {
        self.max_depth = max_depth;
        self
    }

    pub fn filter<'a, F: Fn((&CompactStr, &Arc<ModuleRecord>), &ModuleRecord) -> bool + 'static>(
        mut self,
        filter: F,
    ) -> Self {
        self.filter = Some(Box::new(filter));
        self
    }

    pub fn enter<'a, F: Fn((&CompactStr, &Arc<ModuleRecord>), &ModuleRecord) + 'static>(
        mut self,
        enter: F,
    ) -> Self {
        self.enter = Some(Box::new(enter));
        self
    }

    pub fn leave<'a, F: Fn((&CompactStr, &Arc<ModuleRecord>), &ModuleRecord) + 'static>(
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
        let mut visitor = ModuleGraphVisitor {
            traversed: HashSet::new(),
            stack: Vec::new(),
            max_depth: self.max_depth,
        };
        let filter = self.filter.unwrap_or_else(|| Box::new(|_, _| true));
        let enter = self.enter.unwrap_or_else(|| Box::new(|_, _| {}));
        let leave = self.leave.unwrap_or_else(|| Box::new(|_, _| {}));
        let result = visitor.filter_fold_while(initial_value, module, filter, visit, enter, leave);

        ModuleGraphVisitResult::with_result(result, visitor)
    }
}

impl<T> Default for ModuleGraphVisitorBuilder<T> {
    fn default() -> Self {
        Self {
            max_depth: u32::MAX,
            filter: None,
            enter: None,
            leave: None,
            _marker: std::marker::PhantomData {},
        }
    }
}

pub struct ModuleGraphVisitResult<T> {
    pub result: T,
    pub traversed: HashSet<PathBuf>,
    pub stack: Vec<(CompactStr, PathBuf)>,
    pub max_depth: u32,
}

impl<T> ModuleGraphVisitResult<T> {
    fn with_result(result: T, visitor: ModuleGraphVisitor) -> Self {
        Self {
            result,
            traversed: visitor.traversed,
            stack: visitor.stack,
            max_depth: visitor.max_depth,
        }
    }
}

#[derive(Debug)]
struct ModuleGraphVisitor {
    traversed: HashSet<PathBuf>,
    stack: Vec<(CompactStr, PathBuf)>,
    max_depth: u32,
}

impl ModuleGraphVisitor {
    pub(self) fn filter_fold_while<
        T,
        Filter: Fn((&CompactStr, &Arc<ModuleRecord>), &ModuleRecord) -> bool,
        Fold: FnMut(T, (&CompactStr, &Arc<ModuleRecord>), &ModuleRecord) -> VisitFoldWhile<T>,
        EnterMod: FnMut((&CompactStr, &Arc<ModuleRecord>), &ModuleRecord),
        LeaveMod: FnMut((&CompactStr, &Arc<ModuleRecord>), &ModuleRecord),
    >(
        &mut self,
        initial_value: T,
        module_record: &ModuleRecord,
        filter: Filter,
        mut fold: Fold,
        mut enter: EnterMod,
        mut leave: LeaveMod,
    ) -> T {
        let x = self
            .filter_fold_recursive(
                VisitFoldWhile::Next(initial_value),
                module_record,
                &filter,
                &mut fold,
                &mut enter,
                &mut leave,
            )
            .into_inner();
        x
    }

    pub(self) fn filter_fold_recursive<
        T,
        Filter: Fn((&CompactStr, &Arc<ModuleRecord>), &ModuleRecord) -> bool,
        Fold: FnMut(T, (&CompactStr, &Arc<ModuleRecord>), &ModuleRecord) -> VisitFoldWhile<T>,
        EnterMod: FnMut((&CompactStr, &Arc<ModuleRecord>), &ModuleRecord),
        LeaveMod: FnMut((&CompactStr, &Arc<ModuleRecord>), &ModuleRecord),
    >(
        &mut self,
        mut accumulator: VisitFoldWhile<T>,
        module_record: &ModuleRecord,
        filter: &Filter,
        fold: &mut Fold,
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
            if self.stack.len() as u32 > self.max_depth {
                return VisitFoldWhile::Stop(accumulator.into_inner());
            }
            let path = &module_record_ref.resolved_absolute_path;
            if !self.traversed.insert(path.clone()) {
                continue;
            }

            if !filter(module_record_ref.pair(), module_record) {
                continue;
            }

            self.stack.push((module_record_ref.key().clone(), path.clone()));
            enter(module_record_ref.pair(), module_record);

            accumulate!(fold(accumulator.into_inner(), module_record_ref.pair(), module_record));
            accumulate!(self.filter_fold_recursive(
                accumulator,
                module_record_ref.value(),
                filter,
                fold,
                enter,
                leave
            ));

            leave(module_record_ref.pair(), module_record);

            self.stack.pop();
        }

        accumulator
    }
}
