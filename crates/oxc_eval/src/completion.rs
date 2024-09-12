//! Contains an implementation of [6.2.4 Completion Records](https://262.ecma-international.org/15.0/index.html#sec-completion-record-specification-type).
use std::borrow::Cow;

use oxc_diagnostics::OxcDiagnostic;

use crate::Value;

// use crate::Value;

/// 6.2.4 Completion Records
///
/// NOTE(@DonIsaac): it would be better to use an enum instead of a tagged
/// struct. However, I'm not certain about how `UpdateEmpty` is used, so I'm
/// just following the spec for now. Better to be safe than sorry and refactor later. See
/// commented-out code below for a draft implementation
#[derive(Debug)]
pub struct CompletionRecord<'a> {
    /// `[[Type]]`
    ///
    /// The type of completion that occurred..
    ty: CompletionType,
    /// `[[Value]]`
    ///
    /// The value that was produced
    ///
    /// Value: Any value except a [`CompletionRecord`].
    ///
    /// - NOTE(@DonIsaac): [`None`] is `EMPTY`.
    /// - NOTE(@DonIsaac): use a generic? Will this ever contain a non-`Value`?
    value: Option<Value<'a>>,
    /// `[[Target]]`
    ///
    /// The target label for directed control transfers.
    target: Option<Cow<'a, str>>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CompletionType {
    /// A normal completion (e.g. a value was produced from evaluation)
    ///
    /// > NOTE(@DonIsaac): Normal completions should always have values and shouldn't have labels
    Normal,
    /// A break completion.
    ///
    /// NOTE(@DonIsaac): break completions shouldn't have values, but may have labels
    /// > NOTE(@DonIsaac): Do yields produce value-full break completions?
    Break,
    /// A continue completion, e.g. `continue` statement within a loop
    ///
    /// > NOTE(@DonIsaac): continue completions shouldn't have values, but may have labels.
    Continue,
    /// A return completion, e.g. `return` statement within a function
    Return,
    /// A throw completion, e.g. `throw` statement`
    ///
    /// > NOTE(@DonIsaac): throw completions should always have values. I'm not sure about labels.
    Throw,
}

impl<'a> CompletionRecord<'a> {
    /// 6.2.4.1 NormalCompletion(`value`)
    ///
    /// > The abstract operation NormalCompletion takes argument `value` (any > [`value`](`Value`)
    /// > except a [Completion Record](`CompletionRecord`)) and returns a [normal
    /// > completion](`CompletionType::Normal`).
    #[inline]
    #[must_use]
    pub fn normal(value: Value<'a>) -> Self {
        // 1. Return Completion Record { [[Type]]: normal, [[Value]]: value, [[Target]]: empty }.
        Self::new(CompletionType::Normal, Some(value), None)
    }

    /// 6.2.4.2 ThrowCompletion(`value`)
    ///
    /// > The abstract operation ThrowCompletion takes argument `value` ([an
    /// > ECMAScript language value](`Value`) and returns a [throw
    /// > completion](`CompletionType::Throw`).
    #[inline]
    #[must_use]
    pub fn throw(value: Value<'a>) -> Self {
        // 1. Return Completion Record { [[Type]]: throw, [[Value]]: value, [[Target]]: empty }.
        Self::new(CompletionType::Throw, Some(value), None)
    }

    #[inline]
    #[must_use]
    pub(crate) fn new(
        ty: CompletionType,
        value: Option<Value<'a>>,
        target: Option<Cow<'a, str>>,
    ) -> Self {
        Self { ty, value, target }
    }

    /// 6.2.4.3 UpdateEmpty (`completionRecord`, `value`)
    ///
    /// > The abstract operation UpdateEmpty takes arguments `completionRecord` (a
    /// > [Completion Record](`CompletionRecord`) and value (any value except a Completion Record) and
    /// > returns a [Completion Record](`CompletionRecord`).
    #[must_use]
    pub fn update_empty(self, value: Value<'a>) -> Self {
        // 1. Assert: If completionRecord is either a return completion or a
        //    throw completion, then completionRecord.[[Value]] is not empty.
        #[cfg(debug_assertions)]
        if matches!(self.ty, CompletionType::Return | CompletionType::Throw) {
            assert!(self.value.is_some());
        }
        // 2. If completionRecord.[[Value]] is not empty, return ? completionRecord.
        // 3. Return Completion Record { [[Type]]: completionRecord.[[Type]],
        //    [[Value]]: value, [[Target]]: completionRecord.[[Target]] }.
        match self.value {
            Some(_) => self,
            None => Self { value: Some(value), ..self },
        }
    }

    /// The type of completion that occurred.
    #[inline]
    pub fn r#type(&self) -> CompletionType {
        self.ty
    }

    /// The value that was produced.
    #[inline]
    pub fn value(&self) -> Option<&Value<'a>> {
        self.value.as_ref()
    }

    /// The target label for directed control transfers.
    #[inline]
    pub fn target(&self) -> Option<&Cow<'a, str>> {
        self.target.as_ref()
    }
}

// /// Completion Record
// ///
// /// The _Completion Record_ specification type is used to explain the runtime
// /// propagation of values and control flow such as the behaviour of statements
// /// (`break`, `continue`, `return` and `throw`) that perform nonlocal transfers of
// /// control.
// ///
// /// ### References
// /// - [ECMA-262 - 6.2.4 The Completion Record Specification Type](https://262.ecma-international.org/15.0/index.html#sec-completion-record-specification-type)
// pub enum CompletionRecord<'a> {
//     // TODO: [[Target]]
//     Normal(Value<'a>),
//     Break,
//     Continue,
//     Return(Value<'a>),
//     Throw(Value<'a>),
// }

// impl<'a> CompletionRecord<'a> {
//     /// > _abrupt completion_ refers to any Completion Record with a [[Type]] value other than NORMAL.
//     #[inline]
//     pub fn is_abrupt(&self) -> bool {
//         !matches!(self, Self::Normal(_))
//     }

//     // /// ### 6.2.4.3 UpdateEmpty(`completionRecord`, `value`)
//     // /// >The abstract operation UpdateEmpty takes arguments `completionRecord` (a
//     // /// Completion Record) and `value` (any value except a Completion Record) and
//     // /// returns a [`CompletionRecord`].
//     // /// ### References
//     // /// - [ECMA-262 6.2.4.3 UpdateEmpty](https://262.ecma-international.org/15.0/index.html#sec-updateempty)
//     // pub fn update_empty(mut self, value: Value<'a>) -> Self {
//     //     // 1. Assert: if completionRecord is either a return completion or a
//     //     //    throw completion, then completionRecord.[[Value]] is not empty.
//     // }
// }

// pub struct TypeError(OxcDiagnostic);

// impl TypeError {
//     pub fn new<S>(message: S) -> Self
//     where
//         S: Into<Cow<'static, str>>,
//     {
//         Self(OxcDiagnostic::error(message))
//     }
// }

// impl Deref for TypeError {
//     type Target = OxcDiagnostic;

//     fn deref(&self) -> &Self::Target {
//         &self.0
//     }
// }

// impl From<&'static str> for TypeError {
//     fn from(value: &'static str) -> Self {
//         Self(OxcDiagnostic::error(value))
//     }
// }

// impl<T> From<TypeError> for Result<T, TypeError> {
//     #[inline]
//     fn from(val: TypeError) -> Self {
//         Err(val)
//     }
// }

// impl From<TypeError> for OxcDiagnostic {
//     #[inline]
//     fn from(val: TypeError) -> Self {
//         val.0
//     }
// }

pub type TypeError = OxcDiagnostic;
