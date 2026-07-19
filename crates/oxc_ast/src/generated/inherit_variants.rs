// Auto-generated code, DO NOT EDIT DIRECTLY!
// To edit this generated file you have to edit `tasks/ast_tools/src/generators/inherit_variants.rs`.

// PROTOTYPE: This file has been rewritten by hand for the tagged-pointer `Expression` /
// `MemberExpression` design. Enums which inherit `Expression`'s variants now have a single
// `Expression(Expression<'a>)` variant, and enums which inherit `MemberExpression`'s variants
// have a single `MemberExpression(MemberExpression<'a>)` variant.
// Do NOT re-run `just ast` - it would overwrite this file.

// Some `TryFrom` impls have a single non-shared variant left for the catch-all arm
#![expect(clippy::match_wildcard_for_single_variants)]

use std::ptr::NonNull;

use crate::ast::*;

impl<'a> ArrayExpressionElement<'a> {
    /// Return if a [`ArrayExpressionElement`] is a [`Expression`].
    #[inline]
    pub fn is_expression(&self) -> bool {
        matches!(self, Self::Expression(_))
    }

    /// Convert a [`ArrayExpressionElement`] into a [`Expression`].
    ///
    /// # Panics
    /// Panics if not convertible.
    #[inline]
    pub fn into_expression(self) -> Expression<'a> {
        match self {
            Self::Expression(it) => it,
            _ => panic!("Cannot convert to Expression"),
        }
    }

    /// Convert a [`&ArrayExpressionElement`] to a [`&Expression`].
    ///
    /// [`&ArrayExpressionElement`]: ArrayExpressionElement
    /// [`&Expression`]: Expression
    #[inline]
    pub fn as_expression(&self) -> Option<&Expression<'a>> {
        if let Self::Expression(it) = self { Some(it) } else { None }
    }

    /// Convert a [`&mut ArrayExpressionElement`] to a [`&mut Expression`].
    ///
    /// [`&mut ArrayExpressionElement`]: ArrayExpressionElement
    /// [`&mut Expression`]: Expression
    #[inline]
    pub fn as_expression_mut(&mut self) -> Option<&mut Expression<'a>> {
        if let Self::Expression(it) = self { Some(it) } else { None }
    }

    /// Convert a [`&ArrayExpressionElement`] to a [`&Expression`].
    ///
    /// # Panics
    /// Panics if not convertible.
    ///
    /// [`&ArrayExpressionElement`]: ArrayExpressionElement
    /// [`&Expression`]: Expression
    #[inline]
    pub fn to_expression(&self) -> &Expression<'a> {
        self.as_expression().unwrap()
    }

    /// Convert a [`&mut ArrayExpressionElement`] to a [`&mut Expression`].
    ///
    /// # Panics
    /// Panics if not convertible.
    ///
    /// [`&mut ArrayExpressionElement`]: ArrayExpressionElement
    /// [`&mut Expression`]: Expression
    #[inline]
    pub fn to_expression_mut(&mut self) -> &mut Expression<'a> {
        self.as_expression_mut().unwrap()
    }
}

impl<'a> TryFrom<ArrayExpressionElement<'a>> for Expression<'a> {
    type Error = ();

    /// Convert a [`ArrayExpressionElement`] to a [`Expression`].
    ///
    /// # Errors
    /// Returns `Err` if not convertible.
    #[inline]
    fn try_from(value: ArrayExpressionElement<'a>) -> Result<Self, Self::Error> {
        match value {
            ArrayExpressionElement::Expression(it) => Ok(it),
            _ => Err(()),
        }
    }
}

impl<'a> From<Expression<'a>> for ArrayExpressionElement<'a> {
    /// Convert a [`Expression`] to a [`ArrayExpressionElement`].
    #[inline]
    fn from(value: Expression<'a>) -> Self {
        ArrayExpressionElement::Expression(value)
    }
}

impl<'a> ArrayExpressionElement<'a> {
    /// Return if a [`ArrayExpressionElement`] is a [`MemberExpression`].
    #[inline]
    pub fn is_member_expression(&self) -> bool {
        self.as_expression().is_some_and(Expression::is_member_expression)
    }

    /// Convert a [`ArrayExpressionElement`] into a [`MemberExpression`].
    ///
    /// # Panics
    /// Panics if not convertible.
    #[inline]
    pub fn into_member_expression(self) -> MemberExpression<'a> {
        self.into_expression().into_member_expression()
    }

    /// Convert a [`&ArrayExpressionElement`] to a [`&MemberExpression`].
    ///
    /// [`&ArrayExpressionElement`]: ArrayExpressionElement
    /// [`&MemberExpression`]: MemberExpression
    #[inline]
    pub fn as_member_expression(&self) -> Option<&MemberExpression<'a>> {
        self.as_expression().and_then(Expression::as_member_expression)
    }

    /// Convert a [`&mut ArrayExpressionElement`] to a [`&mut MemberExpression`].
    ///
    /// [`&mut ArrayExpressionElement`]: ArrayExpressionElement
    /// [`&mut MemberExpression`]: MemberExpression
    #[inline]
    pub fn as_member_expression_mut(&mut self) -> Option<&mut MemberExpression<'a>> {
        self.as_expression_mut().and_then(Expression::as_member_expression_mut)
    }

    /// Convert a [`&ArrayExpressionElement`] to a [`&MemberExpression`].
    ///
    /// # Panics
    /// Panics if not convertible.
    ///
    /// [`&ArrayExpressionElement`]: ArrayExpressionElement
    /// [`&MemberExpression`]: MemberExpression
    #[inline]
    pub fn to_member_expression(&self) -> &MemberExpression<'a> {
        self.as_member_expression().unwrap()
    }

    /// Convert a [`&mut ArrayExpressionElement`] to a [`&mut MemberExpression`].
    ///
    /// # Panics
    /// Panics if not convertible.
    ///
    /// [`&mut ArrayExpressionElement`]: ArrayExpressionElement
    /// [`&mut MemberExpression`]: MemberExpression
    #[inline]
    pub fn to_member_expression_mut(&mut self) -> &mut MemberExpression<'a> {
        self.as_member_expression_mut().unwrap()
    }
}

impl<'a> TryFrom<ArrayExpressionElement<'a>> for MemberExpression<'a> {
    type Error = ();

    /// Convert a [`ArrayExpressionElement`] to a [`MemberExpression`].
    ///
    /// # Errors
    /// Returns `Err` if not convertible.
    #[inline]
    fn try_from(value: ArrayExpressionElement<'a>) -> Result<Self, Self::Error> {
        match value {
            ArrayExpressionElement::Expression(it) => MemberExpression::try_from(it),
            _ => Err(()),
        }
    }
}

impl<'a> From<MemberExpression<'a>> for ArrayExpressionElement<'a> {
    /// Convert a [`MemberExpression`] to a [`ArrayExpressionElement`].
    #[inline]
    fn from(value: MemberExpression<'a>) -> Self {
        ArrayExpressionElement::Expression(Expression::from(value))
    }
}

impl<'a> PropertyKey<'a> {
    /// Return if a [`PropertyKey`] is a [`Expression`].
    #[inline]
    pub fn is_expression(&self) -> bool {
        matches!(self, Self::Expression(_))
    }

    /// Convert a [`PropertyKey`] into a [`Expression`].
    ///
    /// # Panics
    /// Panics if not convertible.
    #[inline]
    pub fn into_expression(self) -> Expression<'a> {
        match self {
            Self::Expression(it) => it,
            _ => panic!("Cannot convert to Expression"),
        }
    }

    /// Convert a [`&PropertyKey`] to a [`&Expression`].
    ///
    /// [`&PropertyKey`]: PropertyKey
    /// [`&Expression`]: Expression
    #[inline]
    pub fn as_expression(&self) -> Option<&Expression<'a>> {
        if let Self::Expression(it) = self { Some(it) } else { None }
    }

    /// Convert a [`&mut PropertyKey`] to a [`&mut Expression`].
    ///
    /// [`&mut PropertyKey`]: PropertyKey
    /// [`&mut Expression`]: Expression
    #[inline]
    pub fn as_expression_mut(&mut self) -> Option<&mut Expression<'a>> {
        if let Self::Expression(it) = self { Some(it) } else { None }
    }

    /// Convert a [`&PropertyKey`] to a [`&Expression`].
    ///
    /// # Panics
    /// Panics if not convertible.
    ///
    /// [`&PropertyKey`]: PropertyKey
    /// [`&Expression`]: Expression
    #[inline]
    pub fn to_expression(&self) -> &Expression<'a> {
        self.as_expression().unwrap()
    }

    /// Convert a [`&mut PropertyKey`] to a [`&mut Expression`].
    ///
    /// # Panics
    /// Panics if not convertible.
    ///
    /// [`&mut PropertyKey`]: PropertyKey
    /// [`&mut Expression`]: Expression
    #[inline]
    pub fn to_expression_mut(&mut self) -> &mut Expression<'a> {
        self.as_expression_mut().unwrap()
    }
}

impl<'a> TryFrom<PropertyKey<'a>> for Expression<'a> {
    type Error = ();

    /// Convert a [`PropertyKey`] to a [`Expression`].
    ///
    /// # Errors
    /// Returns `Err` if not convertible.
    #[inline]
    fn try_from(value: PropertyKey<'a>) -> Result<Self, Self::Error> {
        match value {
            PropertyKey::Expression(it) => Ok(it),
            _ => Err(()),
        }
    }
}

impl<'a> From<Expression<'a>> for PropertyKey<'a> {
    /// Convert a [`Expression`] to a [`PropertyKey`].
    #[inline]
    fn from(value: Expression<'a>) -> Self {
        PropertyKey::Expression(value)
    }
}

impl<'a> PropertyKey<'a> {
    /// Return if a [`PropertyKey`] is a [`MemberExpression`].
    #[inline]
    pub fn is_member_expression(&self) -> bool {
        self.as_expression().is_some_and(Expression::is_member_expression)
    }

    /// Convert a [`PropertyKey`] into a [`MemberExpression`].
    ///
    /// # Panics
    /// Panics if not convertible.
    #[inline]
    pub fn into_member_expression(self) -> MemberExpression<'a> {
        self.into_expression().into_member_expression()
    }

    /// Convert a [`&PropertyKey`] to a [`&MemberExpression`].
    ///
    /// [`&PropertyKey`]: PropertyKey
    /// [`&MemberExpression`]: MemberExpression
    #[inline]
    pub fn as_member_expression(&self) -> Option<&MemberExpression<'a>> {
        self.as_expression().and_then(Expression::as_member_expression)
    }

    /// Convert a [`&mut PropertyKey`] to a [`&mut MemberExpression`].
    ///
    /// [`&mut PropertyKey`]: PropertyKey
    /// [`&mut MemberExpression`]: MemberExpression
    #[inline]
    pub fn as_member_expression_mut(&mut self) -> Option<&mut MemberExpression<'a>> {
        self.as_expression_mut().and_then(Expression::as_member_expression_mut)
    }

    /// Convert a [`&PropertyKey`] to a [`&MemberExpression`].
    ///
    /// # Panics
    /// Panics if not convertible.
    ///
    /// [`&PropertyKey`]: PropertyKey
    /// [`&MemberExpression`]: MemberExpression
    #[inline]
    pub fn to_member_expression(&self) -> &MemberExpression<'a> {
        self.as_member_expression().unwrap()
    }

    /// Convert a [`&mut PropertyKey`] to a [`&mut MemberExpression`].
    ///
    /// # Panics
    /// Panics if not convertible.
    ///
    /// [`&mut PropertyKey`]: PropertyKey
    /// [`&mut MemberExpression`]: MemberExpression
    #[inline]
    pub fn to_member_expression_mut(&mut self) -> &mut MemberExpression<'a> {
        self.as_member_expression_mut().unwrap()
    }
}

impl<'a> TryFrom<PropertyKey<'a>> for MemberExpression<'a> {
    type Error = ();

    /// Convert a [`PropertyKey`] to a [`MemberExpression`].
    ///
    /// # Errors
    /// Returns `Err` if not convertible.
    #[inline]
    fn try_from(value: PropertyKey<'a>) -> Result<Self, Self::Error> {
        match value {
            PropertyKey::Expression(it) => MemberExpression::try_from(it),
            _ => Err(()),
        }
    }
}

impl<'a> From<MemberExpression<'a>> for PropertyKey<'a> {
    /// Convert a [`MemberExpression`] to a [`PropertyKey`].
    #[inline]
    fn from(value: MemberExpression<'a>) -> Self {
        PropertyKey::Expression(Expression::from(value))
    }
}

impl<'a> Argument<'a> {
    /// Return if a [`Argument`] is a [`Expression`].
    #[inline]
    pub fn is_expression(&self) -> bool {
        matches!(self, Self::Expression(_))
    }

    /// Convert a [`Argument`] into a [`Expression`].
    ///
    /// # Panics
    /// Panics if not convertible.
    #[inline]
    pub fn into_expression(self) -> Expression<'a> {
        match self {
            Self::Expression(it) => it,
            _ => panic!("Cannot convert to Expression"),
        }
    }

    /// Convert a [`&Argument`] to a [`&Expression`].
    ///
    /// [`&Argument`]: Argument
    /// [`&Expression`]: Expression
    #[inline]
    pub fn as_expression(&self) -> Option<&Expression<'a>> {
        if let Self::Expression(it) = self { Some(it) } else { None }
    }

    /// Convert a [`&mut Argument`] to a [`&mut Expression`].
    ///
    /// [`&mut Argument`]: Argument
    /// [`&mut Expression`]: Expression
    #[inline]
    pub fn as_expression_mut(&mut self) -> Option<&mut Expression<'a>> {
        if let Self::Expression(it) = self { Some(it) } else { None }
    }

    /// Convert a [`&Argument`] to a [`&Expression`].
    ///
    /// # Panics
    /// Panics if not convertible.
    ///
    /// [`&Argument`]: Argument
    /// [`&Expression`]: Expression
    #[inline]
    pub fn to_expression(&self) -> &Expression<'a> {
        self.as_expression().unwrap()
    }

    /// Convert a [`&mut Argument`] to a [`&mut Expression`].
    ///
    /// # Panics
    /// Panics if not convertible.
    ///
    /// [`&mut Argument`]: Argument
    /// [`&mut Expression`]: Expression
    #[inline]
    pub fn to_expression_mut(&mut self) -> &mut Expression<'a> {
        self.as_expression_mut().unwrap()
    }
}

impl<'a> TryFrom<Argument<'a>> for Expression<'a> {
    type Error = ();

    /// Convert a [`Argument`] to a [`Expression`].
    ///
    /// # Errors
    /// Returns `Err` if not convertible.
    #[inline]
    fn try_from(value: Argument<'a>) -> Result<Self, Self::Error> {
        match value {
            Argument::Expression(it) => Ok(it),
            _ => Err(()),
        }
    }
}

impl<'a> From<Expression<'a>> for Argument<'a> {
    /// Convert a [`Expression`] to a [`Argument`].
    #[inline]
    fn from(value: Expression<'a>) -> Self {
        Argument::Expression(value)
    }
}

impl<'a> Argument<'a> {
    /// Return if a [`Argument`] is a [`MemberExpression`].
    #[inline]
    pub fn is_member_expression(&self) -> bool {
        self.as_expression().is_some_and(Expression::is_member_expression)
    }

    /// Convert a [`Argument`] into a [`MemberExpression`].
    ///
    /// # Panics
    /// Panics if not convertible.
    #[inline]
    pub fn into_member_expression(self) -> MemberExpression<'a> {
        self.into_expression().into_member_expression()
    }

    /// Convert a [`&Argument`] to a [`&MemberExpression`].
    ///
    /// [`&Argument`]: Argument
    /// [`&MemberExpression`]: MemberExpression
    #[inline]
    pub fn as_member_expression(&self) -> Option<&MemberExpression<'a>> {
        self.as_expression().and_then(Expression::as_member_expression)
    }

    /// Convert a [`&mut Argument`] to a [`&mut MemberExpression`].
    ///
    /// [`&mut Argument`]: Argument
    /// [`&mut MemberExpression`]: MemberExpression
    #[inline]
    pub fn as_member_expression_mut(&mut self) -> Option<&mut MemberExpression<'a>> {
        self.as_expression_mut().and_then(Expression::as_member_expression_mut)
    }

    /// Convert a [`&Argument`] to a [`&MemberExpression`].
    ///
    /// # Panics
    /// Panics if not convertible.
    ///
    /// [`&Argument`]: Argument
    /// [`&MemberExpression`]: MemberExpression
    #[inline]
    pub fn to_member_expression(&self) -> &MemberExpression<'a> {
        self.as_member_expression().unwrap()
    }

    /// Convert a [`&mut Argument`] to a [`&mut MemberExpression`].
    ///
    /// # Panics
    /// Panics if not convertible.
    ///
    /// [`&mut Argument`]: Argument
    /// [`&mut MemberExpression`]: MemberExpression
    #[inline]
    pub fn to_member_expression_mut(&mut self) -> &mut MemberExpression<'a> {
        self.as_member_expression_mut().unwrap()
    }
}

impl<'a> TryFrom<Argument<'a>> for MemberExpression<'a> {
    type Error = ();

    /// Convert a [`Argument`] to a [`MemberExpression`].
    ///
    /// # Errors
    /// Returns `Err` if not convertible.
    #[inline]
    fn try_from(value: Argument<'a>) -> Result<Self, Self::Error> {
        match value {
            Argument::Expression(it) => MemberExpression::try_from(it),
            _ => Err(()),
        }
    }
}

impl<'a> From<MemberExpression<'a>> for Argument<'a> {
    /// Convert a [`MemberExpression`] to a [`Argument`].
    #[inline]
    fn from(value: MemberExpression<'a>) -> Self {
        Argument::Expression(Expression::from(value))
    }
}

impl<'a> ForStatementInit<'a> {
    /// Return if a [`ForStatementInit`] is a [`Expression`].
    #[inline]
    pub fn is_expression(&self) -> bool {
        matches!(self, Self::Expression(_))
    }

    /// Convert a [`ForStatementInit`] into a [`Expression`].
    ///
    /// # Panics
    /// Panics if not convertible.
    #[inline]
    pub fn into_expression(self) -> Expression<'a> {
        match self {
            Self::Expression(it) => it,
            _ => panic!("Cannot convert to Expression"),
        }
    }

    /// Convert a [`&ForStatementInit`] to a [`&Expression`].
    ///
    /// [`&ForStatementInit`]: ForStatementInit
    /// [`&Expression`]: Expression
    #[inline]
    pub fn as_expression(&self) -> Option<&Expression<'a>> {
        if let Self::Expression(it) = self { Some(it) } else { None }
    }

    /// Convert a [`&mut ForStatementInit`] to a [`&mut Expression`].
    ///
    /// [`&mut ForStatementInit`]: ForStatementInit
    /// [`&mut Expression`]: Expression
    #[inline]
    pub fn as_expression_mut(&mut self) -> Option<&mut Expression<'a>> {
        if let Self::Expression(it) = self { Some(it) } else { None }
    }

    /// Convert a [`&ForStatementInit`] to a [`&Expression`].
    ///
    /// # Panics
    /// Panics if not convertible.
    ///
    /// [`&ForStatementInit`]: ForStatementInit
    /// [`&Expression`]: Expression
    #[inline]
    pub fn to_expression(&self) -> &Expression<'a> {
        self.as_expression().unwrap()
    }

    /// Convert a [`&mut ForStatementInit`] to a [`&mut Expression`].
    ///
    /// # Panics
    /// Panics if not convertible.
    ///
    /// [`&mut ForStatementInit`]: ForStatementInit
    /// [`&mut Expression`]: Expression
    #[inline]
    pub fn to_expression_mut(&mut self) -> &mut Expression<'a> {
        self.as_expression_mut().unwrap()
    }
}

impl<'a> TryFrom<ForStatementInit<'a>> for Expression<'a> {
    type Error = ();

    /// Convert a [`ForStatementInit`] to a [`Expression`].
    ///
    /// # Errors
    /// Returns `Err` if not convertible.
    #[inline]
    fn try_from(value: ForStatementInit<'a>) -> Result<Self, Self::Error> {
        match value {
            ForStatementInit::Expression(it) => Ok(it),
            _ => Err(()),
        }
    }
}

impl<'a> From<Expression<'a>> for ForStatementInit<'a> {
    /// Convert a [`Expression`] to a [`ForStatementInit`].
    #[inline]
    fn from(value: Expression<'a>) -> Self {
        ForStatementInit::Expression(value)
    }
}

impl<'a> ForStatementInit<'a> {
    /// Return if a [`ForStatementInit`] is a [`MemberExpression`].
    #[inline]
    pub fn is_member_expression(&self) -> bool {
        self.as_expression().is_some_and(Expression::is_member_expression)
    }

    /// Convert a [`ForStatementInit`] into a [`MemberExpression`].
    ///
    /// # Panics
    /// Panics if not convertible.
    #[inline]
    pub fn into_member_expression(self) -> MemberExpression<'a> {
        self.into_expression().into_member_expression()
    }

    /// Convert a [`&ForStatementInit`] to a [`&MemberExpression`].
    ///
    /// [`&ForStatementInit`]: ForStatementInit
    /// [`&MemberExpression`]: MemberExpression
    #[inline]
    pub fn as_member_expression(&self) -> Option<&MemberExpression<'a>> {
        self.as_expression().and_then(Expression::as_member_expression)
    }

    /// Convert a [`&mut ForStatementInit`] to a [`&mut MemberExpression`].
    ///
    /// [`&mut ForStatementInit`]: ForStatementInit
    /// [`&mut MemberExpression`]: MemberExpression
    #[inline]
    pub fn as_member_expression_mut(&mut self) -> Option<&mut MemberExpression<'a>> {
        self.as_expression_mut().and_then(Expression::as_member_expression_mut)
    }

    /// Convert a [`&ForStatementInit`] to a [`&MemberExpression`].
    ///
    /// # Panics
    /// Panics if not convertible.
    ///
    /// [`&ForStatementInit`]: ForStatementInit
    /// [`&MemberExpression`]: MemberExpression
    #[inline]
    pub fn to_member_expression(&self) -> &MemberExpression<'a> {
        self.as_member_expression().unwrap()
    }

    /// Convert a [`&mut ForStatementInit`] to a [`&mut MemberExpression`].
    ///
    /// # Panics
    /// Panics if not convertible.
    ///
    /// [`&mut ForStatementInit`]: ForStatementInit
    /// [`&mut MemberExpression`]: MemberExpression
    #[inline]
    pub fn to_member_expression_mut(&mut self) -> &mut MemberExpression<'a> {
        self.as_member_expression_mut().unwrap()
    }
}

impl<'a> TryFrom<ForStatementInit<'a>> for MemberExpression<'a> {
    type Error = ();

    /// Convert a [`ForStatementInit`] to a [`MemberExpression`].
    ///
    /// # Errors
    /// Returns `Err` if not convertible.
    #[inline]
    fn try_from(value: ForStatementInit<'a>) -> Result<Self, Self::Error> {
        match value {
            ForStatementInit::Expression(it) => MemberExpression::try_from(it),
            _ => Err(()),
        }
    }
}

impl<'a> From<MemberExpression<'a>> for ForStatementInit<'a> {
    /// Convert a [`MemberExpression`] to a [`ForStatementInit`].
    #[inline]
    fn from(value: MemberExpression<'a>) -> Self {
        ForStatementInit::Expression(Expression::from(value))
    }
}

impl<'a> ExportDefaultDeclarationKind<'a> {
    /// Return if a [`ExportDefaultDeclarationKind`] is a [`Expression`].
    #[inline]
    pub fn is_expression(&self) -> bool {
        matches!(self, Self::Expression(_))
    }

    /// Convert a [`ExportDefaultDeclarationKind`] into a [`Expression`].
    ///
    /// # Panics
    /// Panics if not convertible.
    #[inline]
    pub fn into_expression(self) -> Expression<'a> {
        match self {
            Self::Expression(it) => it,
            _ => panic!("Cannot convert to Expression"),
        }
    }

    /// Convert a [`&ExportDefaultDeclarationKind`] to a [`&Expression`].
    ///
    /// [`&ExportDefaultDeclarationKind`]: ExportDefaultDeclarationKind
    /// [`&Expression`]: Expression
    #[inline]
    pub fn as_expression(&self) -> Option<&Expression<'a>> {
        if let Self::Expression(it) = self { Some(it) } else { None }
    }

    /// Convert a [`&mut ExportDefaultDeclarationKind`] to a [`&mut Expression`].
    ///
    /// [`&mut ExportDefaultDeclarationKind`]: ExportDefaultDeclarationKind
    /// [`&mut Expression`]: Expression
    #[inline]
    pub fn as_expression_mut(&mut self) -> Option<&mut Expression<'a>> {
        if let Self::Expression(it) = self { Some(it) } else { None }
    }

    /// Convert a [`&ExportDefaultDeclarationKind`] to a [`&Expression`].
    ///
    /// # Panics
    /// Panics if not convertible.
    ///
    /// [`&ExportDefaultDeclarationKind`]: ExportDefaultDeclarationKind
    /// [`&Expression`]: Expression
    #[inline]
    pub fn to_expression(&self) -> &Expression<'a> {
        self.as_expression().unwrap()
    }

    /// Convert a [`&mut ExportDefaultDeclarationKind`] to a [`&mut Expression`].
    ///
    /// # Panics
    /// Panics if not convertible.
    ///
    /// [`&mut ExportDefaultDeclarationKind`]: ExportDefaultDeclarationKind
    /// [`&mut Expression`]: Expression
    #[inline]
    pub fn to_expression_mut(&mut self) -> &mut Expression<'a> {
        self.as_expression_mut().unwrap()
    }
}

impl<'a> TryFrom<ExportDefaultDeclarationKind<'a>> for Expression<'a> {
    type Error = ();

    /// Convert a [`ExportDefaultDeclarationKind`] to a [`Expression`].
    ///
    /// # Errors
    /// Returns `Err` if not convertible.
    #[inline]
    fn try_from(value: ExportDefaultDeclarationKind<'a>) -> Result<Self, Self::Error> {
        match value {
            ExportDefaultDeclarationKind::Expression(it) => Ok(it),
            _ => Err(()),
        }
    }
}

impl<'a> From<Expression<'a>> for ExportDefaultDeclarationKind<'a> {
    /// Convert a [`Expression`] to a [`ExportDefaultDeclarationKind`].
    #[inline]
    fn from(value: Expression<'a>) -> Self {
        ExportDefaultDeclarationKind::Expression(value)
    }
}

impl<'a> ExportDefaultDeclarationKind<'a> {
    /// Return if a [`ExportDefaultDeclarationKind`] is a [`MemberExpression`].
    #[inline]
    pub fn is_member_expression(&self) -> bool {
        self.as_expression().is_some_and(Expression::is_member_expression)
    }

    /// Convert a [`ExportDefaultDeclarationKind`] into a [`MemberExpression`].
    ///
    /// # Panics
    /// Panics if not convertible.
    #[inline]
    pub fn into_member_expression(self) -> MemberExpression<'a> {
        self.into_expression().into_member_expression()
    }

    /// Convert a [`&ExportDefaultDeclarationKind`] to a [`&MemberExpression`].
    ///
    /// [`&ExportDefaultDeclarationKind`]: ExportDefaultDeclarationKind
    /// [`&MemberExpression`]: MemberExpression
    #[inline]
    pub fn as_member_expression(&self) -> Option<&MemberExpression<'a>> {
        self.as_expression().and_then(Expression::as_member_expression)
    }

    /// Convert a [`&mut ExportDefaultDeclarationKind`] to a [`&mut MemberExpression`].
    ///
    /// [`&mut ExportDefaultDeclarationKind`]: ExportDefaultDeclarationKind
    /// [`&mut MemberExpression`]: MemberExpression
    #[inline]
    pub fn as_member_expression_mut(&mut self) -> Option<&mut MemberExpression<'a>> {
        self.as_expression_mut().and_then(Expression::as_member_expression_mut)
    }

    /// Convert a [`&ExportDefaultDeclarationKind`] to a [`&MemberExpression`].
    ///
    /// # Panics
    /// Panics if not convertible.
    ///
    /// [`&ExportDefaultDeclarationKind`]: ExportDefaultDeclarationKind
    /// [`&MemberExpression`]: MemberExpression
    #[inline]
    pub fn to_member_expression(&self) -> &MemberExpression<'a> {
        self.as_member_expression().unwrap()
    }

    /// Convert a [`&mut ExportDefaultDeclarationKind`] to a [`&mut MemberExpression`].
    ///
    /// # Panics
    /// Panics if not convertible.
    ///
    /// [`&mut ExportDefaultDeclarationKind`]: ExportDefaultDeclarationKind
    /// [`&mut MemberExpression`]: MemberExpression
    #[inline]
    pub fn to_member_expression_mut(&mut self) -> &mut MemberExpression<'a> {
        self.as_member_expression_mut().unwrap()
    }
}

impl<'a> TryFrom<ExportDefaultDeclarationKind<'a>> for MemberExpression<'a> {
    type Error = ();

    /// Convert a [`ExportDefaultDeclarationKind`] to a [`MemberExpression`].
    ///
    /// # Errors
    /// Returns `Err` if not convertible.
    #[inline]
    fn try_from(value: ExportDefaultDeclarationKind<'a>) -> Result<Self, Self::Error> {
        match value {
            ExportDefaultDeclarationKind::Expression(it) => MemberExpression::try_from(it),
            _ => Err(()),
        }
    }
}

impl<'a> From<MemberExpression<'a>> for ExportDefaultDeclarationKind<'a> {
    /// Convert a [`MemberExpression`] to a [`ExportDefaultDeclarationKind`].
    #[inline]
    fn from(value: MemberExpression<'a>) -> Self {
        ExportDefaultDeclarationKind::Expression(Expression::from(value))
    }
}

impl<'a> JSXExpression<'a> {
    /// Return if a [`JSXExpression`] is a [`Expression`].
    #[inline]
    pub fn is_expression(&self) -> bool {
        matches!(self, Self::Expression(_))
    }

    /// Convert a [`JSXExpression`] into a [`Expression`].
    ///
    /// # Panics
    /// Panics if not convertible.
    #[inline]
    pub fn into_expression(self) -> Expression<'a> {
        match self {
            Self::Expression(it) => it,
            _ => panic!("Cannot convert to Expression"),
        }
    }

    /// Convert a [`&JSXExpression`] to a [`&Expression`].
    ///
    /// [`&JSXExpression`]: JSXExpression
    /// [`&Expression`]: Expression
    #[inline]
    pub fn as_expression(&self) -> Option<&Expression<'a>> {
        if let Self::Expression(it) = self { Some(it) } else { None }
    }

    /// Convert a [`&mut JSXExpression`] to a [`&mut Expression`].
    ///
    /// [`&mut JSXExpression`]: JSXExpression
    /// [`&mut Expression`]: Expression
    #[inline]
    pub fn as_expression_mut(&mut self) -> Option<&mut Expression<'a>> {
        if let Self::Expression(it) = self { Some(it) } else { None }
    }

    /// Convert a [`&JSXExpression`] to a [`&Expression`].
    ///
    /// # Panics
    /// Panics if not convertible.
    ///
    /// [`&JSXExpression`]: JSXExpression
    /// [`&Expression`]: Expression
    #[inline]
    pub fn to_expression(&self) -> &Expression<'a> {
        self.as_expression().unwrap()
    }

    /// Convert a [`&mut JSXExpression`] to a [`&mut Expression`].
    ///
    /// # Panics
    /// Panics if not convertible.
    ///
    /// [`&mut JSXExpression`]: JSXExpression
    /// [`&mut Expression`]: Expression
    #[inline]
    pub fn to_expression_mut(&mut self) -> &mut Expression<'a> {
        self.as_expression_mut().unwrap()
    }
}

impl<'a> TryFrom<JSXExpression<'a>> for Expression<'a> {
    type Error = ();

    /// Convert a [`JSXExpression`] to a [`Expression`].
    ///
    /// # Errors
    /// Returns `Err` if not convertible.
    #[inline]
    fn try_from(value: JSXExpression<'a>) -> Result<Self, Self::Error> {
        match value {
            JSXExpression::Expression(it) => Ok(it),
            _ => Err(()),
        }
    }
}

impl<'a> From<Expression<'a>> for JSXExpression<'a> {
    /// Convert a [`Expression`] to a [`JSXExpression`].
    #[inline]
    fn from(value: Expression<'a>) -> Self {
        JSXExpression::Expression(value)
    }
}

impl<'a> JSXExpression<'a> {
    /// Return if a [`JSXExpression`] is a [`MemberExpression`].
    #[inline]
    pub fn is_member_expression(&self) -> bool {
        self.as_expression().is_some_and(Expression::is_member_expression)
    }

    /// Convert a [`JSXExpression`] into a [`MemberExpression`].
    ///
    /// # Panics
    /// Panics if not convertible.
    #[inline]
    pub fn into_member_expression(self) -> MemberExpression<'a> {
        self.into_expression().into_member_expression()
    }

    /// Convert a [`&JSXExpression`] to a [`&MemberExpression`].
    ///
    /// [`&JSXExpression`]: JSXExpression
    /// [`&MemberExpression`]: MemberExpression
    #[inline]
    pub fn as_member_expression(&self) -> Option<&MemberExpression<'a>> {
        self.as_expression().and_then(Expression::as_member_expression)
    }

    /// Convert a [`&mut JSXExpression`] to a [`&mut MemberExpression`].
    ///
    /// [`&mut JSXExpression`]: JSXExpression
    /// [`&mut MemberExpression`]: MemberExpression
    #[inline]
    pub fn as_member_expression_mut(&mut self) -> Option<&mut MemberExpression<'a>> {
        self.as_expression_mut().and_then(Expression::as_member_expression_mut)
    }

    /// Convert a [`&JSXExpression`] to a [`&MemberExpression`].
    ///
    /// # Panics
    /// Panics if not convertible.
    ///
    /// [`&JSXExpression`]: JSXExpression
    /// [`&MemberExpression`]: MemberExpression
    #[inline]
    pub fn to_member_expression(&self) -> &MemberExpression<'a> {
        self.as_member_expression().unwrap()
    }

    /// Convert a [`&mut JSXExpression`] to a [`&mut MemberExpression`].
    ///
    /// # Panics
    /// Panics if not convertible.
    ///
    /// [`&mut JSXExpression`]: JSXExpression
    /// [`&mut MemberExpression`]: MemberExpression
    #[inline]
    pub fn to_member_expression_mut(&mut self) -> &mut MemberExpression<'a> {
        self.as_member_expression_mut().unwrap()
    }
}

impl<'a> TryFrom<JSXExpression<'a>> for MemberExpression<'a> {
    type Error = ();

    /// Convert a [`JSXExpression`] to a [`MemberExpression`].
    ///
    /// # Errors
    /// Returns `Err` if not convertible.
    #[inline]
    fn try_from(value: JSXExpression<'a>) -> Result<Self, Self::Error> {
        match value {
            JSXExpression::Expression(it) => MemberExpression::try_from(it),
            _ => Err(()),
        }
    }
}

impl<'a> From<MemberExpression<'a>> for JSXExpression<'a> {
    /// Convert a [`MemberExpression`] to a [`JSXExpression`].
    #[inline]
    fn from(value: MemberExpression<'a>) -> Self {
        JSXExpression::Expression(Expression::from(value))
    }
}

impl<'a> SimpleAssignmentTarget<'a> {
    /// Return if a [`SimpleAssignmentTarget`] is a [`MemberExpression`].
    #[inline]
    pub fn is_member_expression(&self) -> bool {
        matches!(self, Self::MemberExpression(_))
    }

    /// Convert a [`SimpleAssignmentTarget`] into a [`MemberExpression`].
    ///
    /// # Panics
    /// Panics if not convertible.
    #[inline]
    pub fn into_member_expression(self) -> MemberExpression<'a> {
        match self {
            Self::MemberExpression(it) => it,
            _ => panic!("Cannot convert to MemberExpression"),
        }
    }

    /// Convert a [`&SimpleAssignmentTarget`] to a [`&MemberExpression`].
    ///
    /// [`&SimpleAssignmentTarget`]: SimpleAssignmentTarget
    /// [`&MemberExpression`]: MemberExpression
    #[inline]
    pub fn as_member_expression(&self) -> Option<&MemberExpression<'a>> {
        if let Self::MemberExpression(it) = self { Some(it) } else { None }
    }

    /// Convert a [`&mut SimpleAssignmentTarget`] to a [`&mut MemberExpression`].
    ///
    /// [`&mut SimpleAssignmentTarget`]: SimpleAssignmentTarget
    /// [`&mut MemberExpression`]: MemberExpression
    #[inline]
    pub fn as_member_expression_mut(&mut self) -> Option<&mut MemberExpression<'a>> {
        if let Self::MemberExpression(it) = self { Some(it) } else { None }
    }

    /// Convert a [`&SimpleAssignmentTarget`] to a [`&MemberExpression`].
    ///
    /// # Panics
    /// Panics if not convertible.
    ///
    /// [`&SimpleAssignmentTarget`]: SimpleAssignmentTarget
    /// [`&MemberExpression`]: MemberExpression
    #[inline]
    pub fn to_member_expression(&self) -> &MemberExpression<'a> {
        self.as_member_expression().unwrap()
    }

    /// Convert a [`&mut SimpleAssignmentTarget`] to a [`&mut MemberExpression`].
    ///
    /// # Panics
    /// Panics if not convertible.
    ///
    /// [`&mut SimpleAssignmentTarget`]: SimpleAssignmentTarget
    /// [`&mut MemberExpression`]: MemberExpression
    #[inline]
    pub fn to_member_expression_mut(&mut self) -> &mut MemberExpression<'a> {
        self.as_member_expression_mut().unwrap()
    }
}

impl<'a> TryFrom<SimpleAssignmentTarget<'a>> for MemberExpression<'a> {
    type Error = ();

    /// Convert a [`SimpleAssignmentTarget`] to a [`MemberExpression`].
    ///
    /// # Errors
    /// Returns `Err` if not convertible.
    #[inline]
    fn try_from(value: SimpleAssignmentTarget<'a>) -> Result<Self, Self::Error> {
        match value {
            SimpleAssignmentTarget::MemberExpression(it) => Ok(it),
            _ => Err(()),
        }
    }
}

impl<'a> From<MemberExpression<'a>> for SimpleAssignmentTarget<'a> {
    /// Convert a [`MemberExpression`] to a [`SimpleAssignmentTarget`].
    #[inline]
    fn from(value: MemberExpression<'a>) -> Self {
        SimpleAssignmentTarget::MemberExpression(value)
    }
}

impl<'a> ChainElement<'a> {
    /// Return if a [`ChainElement`] is a [`MemberExpression`].
    #[inline]
    pub fn is_member_expression(&self) -> bool {
        matches!(self, Self::MemberExpression(_))
    }

    /// Convert a [`ChainElement`] into a [`MemberExpression`].
    ///
    /// # Panics
    /// Panics if not convertible.
    #[inline]
    pub fn into_member_expression(self) -> MemberExpression<'a> {
        match self {
            Self::MemberExpression(it) => it,
            _ => panic!("Cannot convert to MemberExpression"),
        }
    }

    /// Convert a [`&ChainElement`] to a [`&MemberExpression`].
    ///
    /// [`&ChainElement`]: ChainElement
    /// [`&MemberExpression`]: MemberExpression
    #[inline]
    pub fn as_member_expression(&self) -> Option<&MemberExpression<'a>> {
        if let Self::MemberExpression(it) = self { Some(it) } else { None }
    }

    /// Convert a [`&mut ChainElement`] to a [`&mut MemberExpression`].
    ///
    /// [`&mut ChainElement`]: ChainElement
    /// [`&mut MemberExpression`]: MemberExpression
    #[inline]
    pub fn as_member_expression_mut(&mut self) -> Option<&mut MemberExpression<'a>> {
        if let Self::MemberExpression(it) = self { Some(it) } else { None }
    }

    /// Convert a [`&ChainElement`] to a [`&MemberExpression`].
    ///
    /// # Panics
    /// Panics if not convertible.
    ///
    /// [`&ChainElement`]: ChainElement
    /// [`&MemberExpression`]: MemberExpression
    #[inline]
    pub fn to_member_expression(&self) -> &MemberExpression<'a> {
        self.as_member_expression().unwrap()
    }

    /// Convert a [`&mut ChainElement`] to a [`&mut MemberExpression`].
    ///
    /// # Panics
    /// Panics if not convertible.
    ///
    /// [`&mut ChainElement`]: ChainElement
    /// [`&mut MemberExpression`]: MemberExpression
    #[inline]
    pub fn to_member_expression_mut(&mut self) -> &mut MemberExpression<'a> {
        self.as_member_expression_mut().unwrap()
    }
}

impl<'a> TryFrom<ChainElement<'a>> for MemberExpression<'a> {
    type Error = ();

    /// Convert a [`ChainElement`] to a [`MemberExpression`].
    ///
    /// # Errors
    /// Returns `Err` if not convertible.
    #[inline]
    fn try_from(value: ChainElement<'a>) -> Result<Self, Self::Error> {
        match value {
            ChainElement::MemberExpression(it) => Ok(it),
            _ => Err(()),
        }
    }
}

impl<'a> From<MemberExpression<'a>> for ChainElement<'a> {
    /// Convert a [`MemberExpression`] to a [`ChainElement`].
    #[inline]
    fn from(value: MemberExpression<'a>) -> Self {
        ChainElement::MemberExpression(value)
    }
}

impl<'a> AssignmentTarget<'a> {
    /// Return if a [`AssignmentTarget`] is a [`SimpleAssignmentTarget`].
    #[inline]
    pub fn is_simple_assignment_target(&self) -> bool {
        matches!(
            self,
            Self::AssignmentTargetIdentifier(_)
                | Self::TSAsExpression(_)
                | Self::TSSatisfiesExpression(_)
                | Self::TSNonNullExpression(_)
                | Self::TSTypeAssertion(_)
                | Self::MemberExpression(_)
        )
    }

    /// Convert a [`AssignmentTarget`] to a [`SimpleAssignmentTarget`].
    ///
    /// # Panics
    /// Panics if not convertible.
    #[inline]
    pub fn into_simple_assignment_target(self) -> SimpleAssignmentTarget<'a> {
        SimpleAssignmentTarget::try_from(self).unwrap()
    }

    /// Convert an [`&AssignmentTarget`] to an [`&SimpleAssignmentTarget`].
    ///
    /// [`&AssignmentTarget`]: AssignmentTarget
    /// [`&SimpleAssignmentTarget`]: SimpleAssignmentTarget
    #[inline]
    pub fn as_simple_assignment_target(&self) -> Option<&SimpleAssignmentTarget<'a>> {
        if self.is_simple_assignment_target() {
            // SAFETY: Transmute is safe because discriminants + types are identical between
            // `parent` and `child` for the shared variants
            Some(unsafe { NonNull::from_ref(self).cast::<SimpleAssignmentTarget>().as_ref() })
        } else {
            None
        }
    }

    /// Convert an [`&mut AssignmentTarget`] to an [`&mut SimpleAssignmentTarget`].
    ///
    /// [`&mut AssignmentTarget`]: AssignmentTarget
    /// [`&mut SimpleAssignmentTarget`]: SimpleAssignmentTarget
    #[inline]
    pub fn as_simple_assignment_target_mut(&mut self) -> Option<&mut SimpleAssignmentTarget<'a>> {
        if self.is_simple_assignment_target() {
            // SAFETY: Transmute is safe because discriminants + types are identical between
            // `parent` and `child` for the shared variants
            Some(unsafe { NonNull::from_mut(self).cast::<SimpleAssignmentTarget>().as_mut() })
        } else {
            None
        }
    }

    /// Convert an [`&AssignmentTarget`] to an [`&SimpleAssignmentTarget`].
    ///
    /// # Panics
    /// Panics if not convertible.
    ///
    /// [`&AssignmentTarget`]: AssignmentTarget
    /// [`&SimpleAssignmentTarget`]: SimpleAssignmentTarget
    #[inline]
    pub fn to_simple_assignment_target(&self) -> &SimpleAssignmentTarget<'a> {
        self.as_simple_assignment_target().unwrap()
    }

    /// Convert an [`&mut AssignmentTarget`] to an [`&mut SimpleAssignmentTarget`].
    ///
    /// # Panics
    /// Panics if not convertible.
    ///
    /// [`&mut AssignmentTarget`]: AssignmentTarget
    /// [`&mut SimpleAssignmentTarget`]: SimpleAssignmentTarget
    #[inline]
    pub fn to_simple_assignment_target_mut(&mut self) -> &mut SimpleAssignmentTarget<'a> {
        self.as_simple_assignment_target_mut().unwrap()
    }
}

impl<'a> TryFrom<AssignmentTarget<'a>> for SimpleAssignmentTarget<'a> {
    type Error = ();

    /// Convert a [`AssignmentTarget`] to a [`SimpleAssignmentTarget`].
    ///
    /// # Errors
    /// Returns `Err` if not convertible.
    #[inline]
    fn try_from(value: AssignmentTarget<'a>) -> Result<Self, Self::Error> {
        // Compiler should implement this as a check of discriminant and then zero-cost transmute,
        // as discriminants for `parent` and `child` are aligned
        match value {
            AssignmentTarget::AssignmentTargetIdentifier(o) => {
                Ok(SimpleAssignmentTarget::AssignmentTargetIdentifier(o))
            }
            AssignmentTarget::TSAsExpression(o) => Ok(SimpleAssignmentTarget::TSAsExpression(o)),
            AssignmentTarget::TSSatisfiesExpression(o) => {
                Ok(SimpleAssignmentTarget::TSSatisfiesExpression(o))
            }
            AssignmentTarget::TSNonNullExpression(o) => {
                Ok(SimpleAssignmentTarget::TSNonNullExpression(o))
            }
            AssignmentTarget::TSTypeAssertion(o) => Ok(SimpleAssignmentTarget::TSTypeAssertion(o)),
            AssignmentTarget::MemberExpression(o) => {
                Ok(SimpleAssignmentTarget::MemberExpression(o))
            }
            _ => Err(()),
        }
    }
}

impl<'a> From<SimpleAssignmentTarget<'a>> for AssignmentTarget<'a> {
    /// Convert a [`SimpleAssignmentTarget`] to a [`AssignmentTarget`].
    #[inline]
    fn from(value: SimpleAssignmentTarget<'a>) -> Self {
        // Compiler should implement this as zero-cost transmute as discriminants
        // for `child` and `parent` are aligned
        match value {
            SimpleAssignmentTarget::AssignmentTargetIdentifier(o) => {
                AssignmentTarget::AssignmentTargetIdentifier(o)
            }
            SimpleAssignmentTarget::TSAsExpression(o) => AssignmentTarget::TSAsExpression(o),
            SimpleAssignmentTarget::TSSatisfiesExpression(o) => {
                AssignmentTarget::TSSatisfiesExpression(o)
            }
            SimpleAssignmentTarget::TSNonNullExpression(o) => {
                AssignmentTarget::TSNonNullExpression(o)
            }
            SimpleAssignmentTarget::TSTypeAssertion(o) => AssignmentTarget::TSTypeAssertion(o),
            SimpleAssignmentTarget::MemberExpression(o) => AssignmentTarget::MemberExpression(o),
        }
    }
}

impl<'a> SimpleAssignmentTarget<'a> {
    /// Convert a [`&SimpleAssignmentTarget`] to a [`&AssignmentTarget`].
    ///
    /// [`&SimpleAssignmentTarget`]: SimpleAssignmentTarget
    /// [`&AssignmentTarget`]: AssignmentTarget
    #[inline]
    pub fn as_assignment_target(&self) -> &AssignmentTarget<'a> {
        // SAFETY: Transmute is safe because discriminants + types are identical between
        // `parent` and `child` for the shared variants
        unsafe { NonNull::from_ref(self).cast::<AssignmentTarget>().as_ref() }
    }
}

impl<'a> AssignmentTarget<'a> {
    /// Return if a [`AssignmentTarget`] is a [`MemberExpression`].
    #[inline]
    pub fn is_member_expression(&self) -> bool {
        matches!(self, Self::MemberExpression(_))
    }

    /// Convert a [`AssignmentTarget`] into a [`MemberExpression`].
    ///
    /// # Panics
    /// Panics if not convertible.
    #[inline]
    pub fn into_member_expression(self) -> MemberExpression<'a> {
        match self {
            Self::MemberExpression(it) => it,
            _ => panic!("Cannot convert to MemberExpression"),
        }
    }

    /// Convert a [`&AssignmentTarget`] to a [`&MemberExpression`].
    ///
    /// [`&AssignmentTarget`]: AssignmentTarget
    /// [`&MemberExpression`]: MemberExpression
    #[inline]
    pub fn as_member_expression(&self) -> Option<&MemberExpression<'a>> {
        if let Self::MemberExpression(it) = self { Some(it) } else { None }
    }

    /// Convert a [`&mut AssignmentTarget`] to a [`&mut MemberExpression`].
    ///
    /// [`&mut AssignmentTarget`]: AssignmentTarget
    /// [`&mut MemberExpression`]: MemberExpression
    #[inline]
    pub fn as_member_expression_mut(&mut self) -> Option<&mut MemberExpression<'a>> {
        if let Self::MemberExpression(it) = self { Some(it) } else { None }
    }

    /// Convert a [`&AssignmentTarget`] to a [`&MemberExpression`].
    ///
    /// # Panics
    /// Panics if not convertible.
    ///
    /// [`&AssignmentTarget`]: AssignmentTarget
    /// [`&MemberExpression`]: MemberExpression
    #[inline]
    pub fn to_member_expression(&self) -> &MemberExpression<'a> {
        self.as_member_expression().unwrap()
    }

    /// Convert a [`&mut AssignmentTarget`] to a [`&mut MemberExpression`].
    ///
    /// # Panics
    /// Panics if not convertible.
    ///
    /// [`&mut AssignmentTarget`]: AssignmentTarget
    /// [`&mut MemberExpression`]: MemberExpression
    #[inline]
    pub fn to_member_expression_mut(&mut self) -> &mut MemberExpression<'a> {
        self.as_member_expression_mut().unwrap()
    }
}

impl<'a> TryFrom<AssignmentTarget<'a>> for MemberExpression<'a> {
    type Error = ();

    /// Convert a [`AssignmentTarget`] to a [`MemberExpression`].
    ///
    /// # Errors
    /// Returns `Err` if not convertible.
    #[inline]
    fn try_from(value: AssignmentTarget<'a>) -> Result<Self, Self::Error> {
        match value {
            AssignmentTarget::MemberExpression(it) => Ok(it),
            _ => Err(()),
        }
    }
}

impl<'a> From<MemberExpression<'a>> for AssignmentTarget<'a> {
    /// Convert a [`MemberExpression`] to a [`AssignmentTarget`].
    #[inline]
    fn from(value: MemberExpression<'a>) -> Self {
        AssignmentTarget::MemberExpression(value)
    }
}

impl<'a> AssignmentTarget<'a> {
    /// Return if a [`AssignmentTarget`] is a [`AssignmentTargetPattern`].
    #[inline]
    pub fn is_assignment_target_pattern(&self) -> bool {
        matches!(self, Self::ArrayAssignmentTarget(_) | Self::ObjectAssignmentTarget(_))
    }

    /// Convert a [`AssignmentTarget`] to a [`AssignmentTargetPattern`].
    ///
    /// # Panics
    /// Panics if not convertible.
    #[inline]
    pub fn into_assignment_target_pattern(self) -> AssignmentTargetPattern<'a> {
        AssignmentTargetPattern::try_from(self).unwrap()
    }

    /// Convert an [`&AssignmentTarget`] to an [`&AssignmentTargetPattern`].
    ///
    /// [`&AssignmentTarget`]: AssignmentTarget
    /// [`&AssignmentTargetPattern`]: AssignmentTargetPattern
    #[inline]
    pub fn as_assignment_target_pattern(&self) -> Option<&AssignmentTargetPattern<'a>> {
        if self.is_assignment_target_pattern() {
            // SAFETY: Transmute is safe because discriminants + types are identical between
            // `parent` and `child` for the shared variants
            Some(unsafe { NonNull::from_ref(self).cast::<AssignmentTargetPattern>().as_ref() })
        } else {
            None
        }
    }

    /// Convert an [`&mut AssignmentTarget`] to an [`&mut AssignmentTargetPattern`].
    ///
    /// [`&mut AssignmentTarget`]: AssignmentTarget
    /// [`&mut AssignmentTargetPattern`]: AssignmentTargetPattern
    #[inline]
    pub fn as_assignment_target_pattern_mut(&mut self) -> Option<&mut AssignmentTargetPattern<'a>> {
        if self.is_assignment_target_pattern() {
            // SAFETY: Transmute is safe because discriminants + types are identical between
            // `parent` and `child` for the shared variants
            Some(unsafe { NonNull::from_mut(self).cast::<AssignmentTargetPattern>().as_mut() })
        } else {
            None
        }
    }

    /// Convert an [`&AssignmentTarget`] to an [`&AssignmentTargetPattern`].
    ///
    /// # Panics
    /// Panics if not convertible.
    ///
    /// [`&AssignmentTarget`]: AssignmentTarget
    /// [`&AssignmentTargetPattern`]: AssignmentTargetPattern
    #[inline]
    pub fn to_assignment_target_pattern(&self) -> &AssignmentTargetPattern<'a> {
        self.as_assignment_target_pattern().unwrap()
    }

    /// Convert an [`&mut AssignmentTarget`] to an [`&mut AssignmentTargetPattern`].
    ///
    /// # Panics
    /// Panics if not convertible.
    ///
    /// [`&mut AssignmentTarget`]: AssignmentTarget
    /// [`&mut AssignmentTargetPattern`]: AssignmentTargetPattern
    #[inline]
    pub fn to_assignment_target_pattern_mut(&mut self) -> &mut AssignmentTargetPattern<'a> {
        self.as_assignment_target_pattern_mut().unwrap()
    }
}

impl<'a> TryFrom<AssignmentTarget<'a>> for AssignmentTargetPattern<'a> {
    type Error = ();

    /// Convert a [`AssignmentTarget`] to a [`AssignmentTargetPattern`].
    ///
    /// # Errors
    /// Returns `Err` if not convertible.
    #[inline]
    fn try_from(value: AssignmentTarget<'a>) -> Result<Self, Self::Error> {
        // Compiler should implement this as a check of discriminant and then zero-cost transmute,
        // as discriminants for `parent` and `child` are aligned
        match value {
            AssignmentTarget::ArrayAssignmentTarget(o) => {
                Ok(AssignmentTargetPattern::ArrayAssignmentTarget(o))
            }
            AssignmentTarget::ObjectAssignmentTarget(o) => {
                Ok(AssignmentTargetPattern::ObjectAssignmentTarget(o))
            }
            _ => Err(()),
        }
    }
}

impl<'a> From<AssignmentTargetPattern<'a>> for AssignmentTarget<'a> {
    /// Convert a [`AssignmentTargetPattern`] to a [`AssignmentTarget`].
    #[inline]
    fn from(value: AssignmentTargetPattern<'a>) -> Self {
        // Compiler should implement this as zero-cost transmute as discriminants
        // for `child` and `parent` are aligned
        match value {
            AssignmentTargetPattern::ArrayAssignmentTarget(o) => {
                AssignmentTarget::ArrayAssignmentTarget(o)
            }
            AssignmentTargetPattern::ObjectAssignmentTarget(o) => {
                AssignmentTarget::ObjectAssignmentTarget(o)
            }
        }
    }
}

impl<'a> AssignmentTargetPattern<'a> {
    /// Convert a [`&AssignmentTargetPattern`] to a [`&AssignmentTarget`].
    ///
    /// [`&AssignmentTargetPattern`]: AssignmentTargetPattern
    /// [`&AssignmentTarget`]: AssignmentTarget
    #[inline]
    pub fn as_assignment_target(&self) -> &AssignmentTarget<'a> {
        // SAFETY: Transmute is safe because discriminants + types are identical between
        // `parent` and `child` for the shared variants
        unsafe { NonNull::from_ref(self).cast::<AssignmentTarget>().as_ref() }
    }
}

impl<'a> AssignmentTargetMaybeDefault<'a> {
    /// Return if a [`AssignmentTargetMaybeDefault`] is a [`AssignmentTarget`].
    #[inline]
    pub fn is_assignment_target(&self) -> bool {
        matches!(
            self,
            Self::AssignmentTargetIdentifier(_)
                | Self::TSAsExpression(_)
                | Self::TSSatisfiesExpression(_)
                | Self::TSNonNullExpression(_)
                | Self::TSTypeAssertion(_)
                | Self::MemberExpression(_)
                | Self::ArrayAssignmentTarget(_)
                | Self::ObjectAssignmentTarget(_)
        )
    }

    /// Convert a [`AssignmentTargetMaybeDefault`] to a [`AssignmentTarget`].
    ///
    /// # Panics
    /// Panics if not convertible.
    #[inline]
    pub fn into_assignment_target(self) -> AssignmentTarget<'a> {
        AssignmentTarget::try_from(self).unwrap()
    }

    /// Convert an [`&AssignmentTargetMaybeDefault`] to an [`&AssignmentTarget`].
    ///
    /// [`&AssignmentTargetMaybeDefault`]: AssignmentTargetMaybeDefault
    /// [`&AssignmentTarget`]: AssignmentTarget
    #[inline]
    pub fn as_assignment_target(&self) -> Option<&AssignmentTarget<'a>> {
        if self.is_assignment_target() {
            // SAFETY: Transmute is safe because discriminants + types are identical between
            // `parent` and `child` for the shared variants
            Some(unsafe { NonNull::from_ref(self).cast::<AssignmentTarget>().as_ref() })
        } else {
            None
        }
    }

    /// Convert an [`&mut AssignmentTargetMaybeDefault`] to an [`&mut AssignmentTarget`].
    ///
    /// [`&mut AssignmentTargetMaybeDefault`]: AssignmentTargetMaybeDefault
    /// [`&mut AssignmentTarget`]: AssignmentTarget
    #[inline]
    pub fn as_assignment_target_mut(&mut self) -> Option<&mut AssignmentTarget<'a>> {
        if self.is_assignment_target() {
            // SAFETY: Transmute is safe because discriminants + types are identical between
            // `parent` and `child` for the shared variants
            Some(unsafe { NonNull::from_mut(self).cast::<AssignmentTarget>().as_mut() })
        } else {
            None
        }
    }

    /// Convert an [`&AssignmentTargetMaybeDefault`] to an [`&AssignmentTarget`].
    ///
    /// # Panics
    /// Panics if not convertible.
    ///
    /// [`&AssignmentTargetMaybeDefault`]: AssignmentTargetMaybeDefault
    /// [`&AssignmentTarget`]: AssignmentTarget
    #[inline]
    pub fn to_assignment_target(&self) -> &AssignmentTarget<'a> {
        self.as_assignment_target().unwrap()
    }

    /// Convert an [`&mut AssignmentTargetMaybeDefault`] to an [`&mut AssignmentTarget`].
    ///
    /// # Panics
    /// Panics if not convertible.
    ///
    /// [`&mut AssignmentTargetMaybeDefault`]: AssignmentTargetMaybeDefault
    /// [`&mut AssignmentTarget`]: AssignmentTarget
    #[inline]
    pub fn to_assignment_target_mut(&mut self) -> &mut AssignmentTarget<'a> {
        self.as_assignment_target_mut().unwrap()
    }
}

impl<'a> TryFrom<AssignmentTargetMaybeDefault<'a>> for AssignmentTarget<'a> {
    type Error = ();

    /// Convert a [`AssignmentTargetMaybeDefault`] to a [`AssignmentTarget`].
    ///
    /// # Errors
    /// Returns `Err` if not convertible.
    #[inline]
    fn try_from(value: AssignmentTargetMaybeDefault<'a>) -> Result<Self, Self::Error> {
        // Compiler should implement this as a check of discriminant and then zero-cost transmute,
        // as discriminants for `parent` and `child` are aligned
        match value {
            AssignmentTargetMaybeDefault::AssignmentTargetIdentifier(o) => {
                Ok(AssignmentTarget::AssignmentTargetIdentifier(o))
            }
            AssignmentTargetMaybeDefault::TSAsExpression(o) => {
                Ok(AssignmentTarget::TSAsExpression(o))
            }
            AssignmentTargetMaybeDefault::TSSatisfiesExpression(o) => {
                Ok(AssignmentTarget::TSSatisfiesExpression(o))
            }
            AssignmentTargetMaybeDefault::TSNonNullExpression(o) => {
                Ok(AssignmentTarget::TSNonNullExpression(o))
            }
            AssignmentTargetMaybeDefault::TSTypeAssertion(o) => {
                Ok(AssignmentTarget::TSTypeAssertion(o))
            }
            AssignmentTargetMaybeDefault::MemberExpression(o) => {
                Ok(AssignmentTarget::MemberExpression(o))
            }
            AssignmentTargetMaybeDefault::ArrayAssignmentTarget(o) => {
                Ok(AssignmentTarget::ArrayAssignmentTarget(o))
            }
            AssignmentTargetMaybeDefault::ObjectAssignmentTarget(o) => {
                Ok(AssignmentTarget::ObjectAssignmentTarget(o))
            }
            _ => Err(()),
        }
    }
}

impl<'a> From<AssignmentTarget<'a>> for AssignmentTargetMaybeDefault<'a> {
    /// Convert a [`AssignmentTarget`] to a [`AssignmentTargetMaybeDefault`].
    #[inline]
    fn from(value: AssignmentTarget<'a>) -> Self {
        // Compiler should implement this as zero-cost transmute as discriminants
        // for `child` and `parent` are aligned
        match value {
            AssignmentTarget::AssignmentTargetIdentifier(o) => {
                AssignmentTargetMaybeDefault::AssignmentTargetIdentifier(o)
            }
            AssignmentTarget::TSAsExpression(o) => AssignmentTargetMaybeDefault::TSAsExpression(o),
            AssignmentTarget::TSSatisfiesExpression(o) => {
                AssignmentTargetMaybeDefault::TSSatisfiesExpression(o)
            }
            AssignmentTarget::TSNonNullExpression(o) => {
                AssignmentTargetMaybeDefault::TSNonNullExpression(o)
            }
            AssignmentTarget::TSTypeAssertion(o) => {
                AssignmentTargetMaybeDefault::TSTypeAssertion(o)
            }
            AssignmentTarget::MemberExpression(o) => {
                AssignmentTargetMaybeDefault::MemberExpression(o)
            }
            AssignmentTarget::ArrayAssignmentTarget(o) => {
                AssignmentTargetMaybeDefault::ArrayAssignmentTarget(o)
            }
            AssignmentTarget::ObjectAssignmentTarget(o) => {
                AssignmentTargetMaybeDefault::ObjectAssignmentTarget(o)
            }
        }
    }
}

impl<'a> AssignmentTarget<'a> {
    /// Convert a [`&AssignmentTarget`] to a [`&AssignmentTargetMaybeDefault`].
    ///
    /// [`&AssignmentTarget`]: AssignmentTarget
    /// [`&AssignmentTargetMaybeDefault`]: AssignmentTargetMaybeDefault
    #[inline]
    pub fn as_assignment_target_maybe_default(&self) -> &AssignmentTargetMaybeDefault<'a> {
        // SAFETY: Transmute is safe because discriminants + types are identical between
        // `parent` and `child` for the shared variants
        unsafe { NonNull::from_ref(self).cast::<AssignmentTargetMaybeDefault>().as_ref() }
    }
}

impl<'a> AssignmentTargetMaybeDefault<'a> {
    /// Return if a [`AssignmentTargetMaybeDefault`] is a [`SimpleAssignmentTarget`].
    #[inline]
    pub fn is_simple_assignment_target(&self) -> bool {
        matches!(
            self,
            Self::AssignmentTargetIdentifier(_)
                | Self::TSAsExpression(_)
                | Self::TSSatisfiesExpression(_)
                | Self::TSNonNullExpression(_)
                | Self::TSTypeAssertion(_)
                | Self::MemberExpression(_)
        )
    }

    /// Convert a [`AssignmentTargetMaybeDefault`] to a [`SimpleAssignmentTarget`].
    ///
    /// # Panics
    /// Panics if not convertible.
    #[inline]
    pub fn into_simple_assignment_target(self) -> SimpleAssignmentTarget<'a> {
        SimpleAssignmentTarget::try_from(self).unwrap()
    }

    /// Convert an [`&AssignmentTargetMaybeDefault`] to an [`&SimpleAssignmentTarget`].
    ///
    /// [`&AssignmentTargetMaybeDefault`]: AssignmentTargetMaybeDefault
    /// [`&SimpleAssignmentTarget`]: SimpleAssignmentTarget
    #[inline]
    pub fn as_simple_assignment_target(&self) -> Option<&SimpleAssignmentTarget<'a>> {
        if self.is_simple_assignment_target() {
            // SAFETY: Transmute is safe because discriminants + types are identical between
            // `parent` and `child` for the shared variants
            Some(unsafe { NonNull::from_ref(self).cast::<SimpleAssignmentTarget>().as_ref() })
        } else {
            None
        }
    }

    /// Convert an [`&mut AssignmentTargetMaybeDefault`] to an [`&mut SimpleAssignmentTarget`].
    ///
    /// [`&mut AssignmentTargetMaybeDefault`]: AssignmentTargetMaybeDefault
    /// [`&mut SimpleAssignmentTarget`]: SimpleAssignmentTarget
    #[inline]
    pub fn as_simple_assignment_target_mut(&mut self) -> Option<&mut SimpleAssignmentTarget<'a>> {
        if self.is_simple_assignment_target() {
            // SAFETY: Transmute is safe because discriminants + types are identical between
            // `parent` and `child` for the shared variants
            Some(unsafe { NonNull::from_mut(self).cast::<SimpleAssignmentTarget>().as_mut() })
        } else {
            None
        }
    }

    /// Convert an [`&AssignmentTargetMaybeDefault`] to an [`&SimpleAssignmentTarget`].
    ///
    /// # Panics
    /// Panics if not convertible.
    ///
    /// [`&AssignmentTargetMaybeDefault`]: AssignmentTargetMaybeDefault
    /// [`&SimpleAssignmentTarget`]: SimpleAssignmentTarget
    #[inline]
    pub fn to_simple_assignment_target(&self) -> &SimpleAssignmentTarget<'a> {
        self.as_simple_assignment_target().unwrap()
    }

    /// Convert an [`&mut AssignmentTargetMaybeDefault`] to an [`&mut SimpleAssignmentTarget`].
    ///
    /// # Panics
    /// Panics if not convertible.
    ///
    /// [`&mut AssignmentTargetMaybeDefault`]: AssignmentTargetMaybeDefault
    /// [`&mut SimpleAssignmentTarget`]: SimpleAssignmentTarget
    #[inline]
    pub fn to_simple_assignment_target_mut(&mut self) -> &mut SimpleAssignmentTarget<'a> {
        self.as_simple_assignment_target_mut().unwrap()
    }
}

impl<'a> TryFrom<AssignmentTargetMaybeDefault<'a>> for SimpleAssignmentTarget<'a> {
    type Error = ();

    /// Convert a [`AssignmentTargetMaybeDefault`] to a [`SimpleAssignmentTarget`].
    ///
    /// # Errors
    /// Returns `Err` if not convertible.
    #[inline]
    fn try_from(value: AssignmentTargetMaybeDefault<'a>) -> Result<Self, Self::Error> {
        // Compiler should implement this as a check of discriminant and then zero-cost transmute,
        // as discriminants for `parent` and `child` are aligned
        match value {
            AssignmentTargetMaybeDefault::AssignmentTargetIdentifier(o) => {
                Ok(SimpleAssignmentTarget::AssignmentTargetIdentifier(o))
            }
            AssignmentTargetMaybeDefault::TSAsExpression(o) => {
                Ok(SimpleAssignmentTarget::TSAsExpression(o))
            }
            AssignmentTargetMaybeDefault::TSSatisfiesExpression(o) => {
                Ok(SimpleAssignmentTarget::TSSatisfiesExpression(o))
            }
            AssignmentTargetMaybeDefault::TSNonNullExpression(o) => {
                Ok(SimpleAssignmentTarget::TSNonNullExpression(o))
            }
            AssignmentTargetMaybeDefault::TSTypeAssertion(o) => {
                Ok(SimpleAssignmentTarget::TSTypeAssertion(o))
            }
            AssignmentTargetMaybeDefault::MemberExpression(o) => {
                Ok(SimpleAssignmentTarget::MemberExpression(o))
            }
            _ => Err(()),
        }
    }
}

impl<'a> From<SimpleAssignmentTarget<'a>> for AssignmentTargetMaybeDefault<'a> {
    /// Convert a [`SimpleAssignmentTarget`] to a [`AssignmentTargetMaybeDefault`].
    #[inline]
    fn from(value: SimpleAssignmentTarget<'a>) -> Self {
        // Compiler should implement this as zero-cost transmute as discriminants
        // for `child` and `parent` are aligned
        match value {
            SimpleAssignmentTarget::AssignmentTargetIdentifier(o) => {
                AssignmentTargetMaybeDefault::AssignmentTargetIdentifier(o)
            }
            SimpleAssignmentTarget::TSAsExpression(o) => {
                AssignmentTargetMaybeDefault::TSAsExpression(o)
            }
            SimpleAssignmentTarget::TSSatisfiesExpression(o) => {
                AssignmentTargetMaybeDefault::TSSatisfiesExpression(o)
            }
            SimpleAssignmentTarget::TSNonNullExpression(o) => {
                AssignmentTargetMaybeDefault::TSNonNullExpression(o)
            }
            SimpleAssignmentTarget::TSTypeAssertion(o) => {
                AssignmentTargetMaybeDefault::TSTypeAssertion(o)
            }
            SimpleAssignmentTarget::MemberExpression(o) => {
                AssignmentTargetMaybeDefault::MemberExpression(o)
            }
        }
    }
}

impl<'a> SimpleAssignmentTarget<'a> {
    /// Convert a [`&SimpleAssignmentTarget`] to a [`&AssignmentTargetMaybeDefault`].
    ///
    /// [`&SimpleAssignmentTarget`]: SimpleAssignmentTarget
    /// [`&AssignmentTargetMaybeDefault`]: AssignmentTargetMaybeDefault
    #[inline]
    pub fn as_assignment_target_maybe_default(&self) -> &AssignmentTargetMaybeDefault<'a> {
        // SAFETY: Transmute is safe because discriminants + types are identical between
        // `parent` and `child` for the shared variants
        unsafe { NonNull::from_ref(self).cast::<AssignmentTargetMaybeDefault>().as_ref() }
    }
}

impl<'a> AssignmentTargetMaybeDefault<'a> {
    /// Return if a [`AssignmentTargetMaybeDefault`] is a [`MemberExpression`].
    #[inline]
    pub fn is_member_expression(&self) -> bool {
        matches!(self, Self::MemberExpression(_))
    }

    /// Convert a [`AssignmentTargetMaybeDefault`] into a [`MemberExpression`].
    ///
    /// # Panics
    /// Panics if not convertible.
    #[inline]
    pub fn into_member_expression(self) -> MemberExpression<'a> {
        match self {
            Self::MemberExpression(it) => it,
            _ => panic!("Cannot convert to MemberExpression"),
        }
    }

    /// Convert a [`&AssignmentTargetMaybeDefault`] to a [`&MemberExpression`].
    ///
    /// [`&AssignmentTargetMaybeDefault`]: AssignmentTargetMaybeDefault
    /// [`&MemberExpression`]: MemberExpression
    #[inline]
    pub fn as_member_expression(&self) -> Option<&MemberExpression<'a>> {
        if let Self::MemberExpression(it) = self { Some(it) } else { None }
    }

    /// Convert a [`&mut AssignmentTargetMaybeDefault`] to a [`&mut MemberExpression`].
    ///
    /// [`&mut AssignmentTargetMaybeDefault`]: AssignmentTargetMaybeDefault
    /// [`&mut MemberExpression`]: MemberExpression
    #[inline]
    pub fn as_member_expression_mut(&mut self) -> Option<&mut MemberExpression<'a>> {
        if let Self::MemberExpression(it) = self { Some(it) } else { None }
    }

    /// Convert a [`&AssignmentTargetMaybeDefault`] to a [`&MemberExpression`].
    ///
    /// # Panics
    /// Panics if not convertible.
    ///
    /// [`&AssignmentTargetMaybeDefault`]: AssignmentTargetMaybeDefault
    /// [`&MemberExpression`]: MemberExpression
    #[inline]
    pub fn to_member_expression(&self) -> &MemberExpression<'a> {
        self.as_member_expression().unwrap()
    }

    /// Convert a [`&mut AssignmentTargetMaybeDefault`] to a [`&mut MemberExpression`].
    ///
    /// # Panics
    /// Panics if not convertible.
    ///
    /// [`&mut AssignmentTargetMaybeDefault`]: AssignmentTargetMaybeDefault
    /// [`&mut MemberExpression`]: MemberExpression
    #[inline]
    pub fn to_member_expression_mut(&mut self) -> &mut MemberExpression<'a> {
        self.as_member_expression_mut().unwrap()
    }
}

impl<'a> TryFrom<AssignmentTargetMaybeDefault<'a>> for MemberExpression<'a> {
    type Error = ();

    /// Convert a [`AssignmentTargetMaybeDefault`] to a [`MemberExpression`].
    ///
    /// # Errors
    /// Returns `Err` if not convertible.
    #[inline]
    fn try_from(value: AssignmentTargetMaybeDefault<'a>) -> Result<Self, Self::Error> {
        match value {
            AssignmentTargetMaybeDefault::MemberExpression(it) => Ok(it),
            _ => Err(()),
        }
    }
}

impl<'a> From<MemberExpression<'a>> for AssignmentTargetMaybeDefault<'a> {
    /// Convert a [`MemberExpression`] to a [`AssignmentTargetMaybeDefault`].
    #[inline]
    fn from(value: MemberExpression<'a>) -> Self {
        AssignmentTargetMaybeDefault::MemberExpression(value)
    }
}

impl<'a> AssignmentTargetMaybeDefault<'a> {
    /// Return if a [`AssignmentTargetMaybeDefault`] is a [`AssignmentTargetPattern`].
    #[inline]
    pub fn is_assignment_target_pattern(&self) -> bool {
        matches!(self, Self::ArrayAssignmentTarget(_) | Self::ObjectAssignmentTarget(_))
    }

    /// Convert a [`AssignmentTargetMaybeDefault`] to a [`AssignmentTargetPattern`].
    ///
    /// # Panics
    /// Panics if not convertible.
    #[inline]
    pub fn into_assignment_target_pattern(self) -> AssignmentTargetPattern<'a> {
        AssignmentTargetPattern::try_from(self).unwrap()
    }

    /// Convert an [`&AssignmentTargetMaybeDefault`] to an [`&AssignmentTargetPattern`].
    ///
    /// [`&AssignmentTargetMaybeDefault`]: AssignmentTargetMaybeDefault
    /// [`&AssignmentTargetPattern`]: AssignmentTargetPattern
    #[inline]
    pub fn as_assignment_target_pattern(&self) -> Option<&AssignmentTargetPattern<'a>> {
        if self.is_assignment_target_pattern() {
            // SAFETY: Transmute is safe because discriminants + types are identical between
            // `parent` and `child` for the shared variants
            Some(unsafe { NonNull::from_ref(self).cast::<AssignmentTargetPattern>().as_ref() })
        } else {
            None
        }
    }

    /// Convert an [`&mut AssignmentTargetMaybeDefault`] to an [`&mut AssignmentTargetPattern`].
    ///
    /// [`&mut AssignmentTargetMaybeDefault`]: AssignmentTargetMaybeDefault
    /// [`&mut AssignmentTargetPattern`]: AssignmentTargetPattern
    #[inline]
    pub fn as_assignment_target_pattern_mut(&mut self) -> Option<&mut AssignmentTargetPattern<'a>> {
        if self.is_assignment_target_pattern() {
            // SAFETY: Transmute is safe because discriminants + types are identical between
            // `parent` and `child` for the shared variants
            Some(unsafe { NonNull::from_mut(self).cast::<AssignmentTargetPattern>().as_mut() })
        } else {
            None
        }
    }

    /// Convert an [`&AssignmentTargetMaybeDefault`] to an [`&AssignmentTargetPattern`].
    ///
    /// # Panics
    /// Panics if not convertible.
    ///
    /// [`&AssignmentTargetMaybeDefault`]: AssignmentTargetMaybeDefault
    /// [`&AssignmentTargetPattern`]: AssignmentTargetPattern
    #[inline]
    pub fn to_assignment_target_pattern(&self) -> &AssignmentTargetPattern<'a> {
        self.as_assignment_target_pattern().unwrap()
    }

    /// Convert an [`&mut AssignmentTargetMaybeDefault`] to an [`&mut AssignmentTargetPattern`].
    ///
    /// # Panics
    /// Panics if not convertible.
    ///
    /// [`&mut AssignmentTargetMaybeDefault`]: AssignmentTargetMaybeDefault
    /// [`&mut AssignmentTargetPattern`]: AssignmentTargetPattern
    #[inline]
    pub fn to_assignment_target_pattern_mut(&mut self) -> &mut AssignmentTargetPattern<'a> {
        self.as_assignment_target_pattern_mut().unwrap()
    }
}

impl<'a> TryFrom<AssignmentTargetMaybeDefault<'a>> for AssignmentTargetPattern<'a> {
    type Error = ();

    /// Convert a [`AssignmentTargetMaybeDefault`] to a [`AssignmentTargetPattern`].
    ///
    /// # Errors
    /// Returns `Err` if not convertible.
    #[inline]
    fn try_from(value: AssignmentTargetMaybeDefault<'a>) -> Result<Self, Self::Error> {
        // Compiler should implement this as a check of discriminant and then zero-cost transmute,
        // as discriminants for `parent` and `child` are aligned
        match value {
            AssignmentTargetMaybeDefault::ArrayAssignmentTarget(o) => {
                Ok(AssignmentTargetPattern::ArrayAssignmentTarget(o))
            }
            AssignmentTargetMaybeDefault::ObjectAssignmentTarget(o) => {
                Ok(AssignmentTargetPattern::ObjectAssignmentTarget(o))
            }
            _ => Err(()),
        }
    }
}

impl<'a> From<AssignmentTargetPattern<'a>> for AssignmentTargetMaybeDefault<'a> {
    /// Convert a [`AssignmentTargetPattern`] to a [`AssignmentTargetMaybeDefault`].
    #[inline]
    fn from(value: AssignmentTargetPattern<'a>) -> Self {
        // Compiler should implement this as zero-cost transmute as discriminants
        // for `child` and `parent` are aligned
        match value {
            AssignmentTargetPattern::ArrayAssignmentTarget(o) => {
                AssignmentTargetMaybeDefault::ArrayAssignmentTarget(o)
            }
            AssignmentTargetPattern::ObjectAssignmentTarget(o) => {
                AssignmentTargetMaybeDefault::ObjectAssignmentTarget(o)
            }
        }
    }
}

impl<'a> AssignmentTargetPattern<'a> {
    /// Convert a [`&AssignmentTargetPattern`] to a [`&AssignmentTargetMaybeDefault`].
    ///
    /// [`&AssignmentTargetPattern`]: AssignmentTargetPattern
    /// [`&AssignmentTargetMaybeDefault`]: AssignmentTargetMaybeDefault
    #[inline]
    pub fn as_assignment_target_maybe_default(&self) -> &AssignmentTargetMaybeDefault<'a> {
        // SAFETY: Transmute is safe because discriminants + types are identical between
        // `parent` and `child` for the shared variants
        unsafe { NonNull::from_ref(self).cast::<AssignmentTargetMaybeDefault>().as_ref() }
    }
}

impl<'a> Statement<'a> {
    /// Return if a [`Statement`] is a [`Declaration`].
    #[inline]
    pub fn is_declaration(&self) -> bool {
        matches!(
            self,
            Self::VariableDeclaration(_)
                | Self::FunctionDeclaration(_)
                | Self::ClassDeclaration(_)
                | Self::TSTypeAliasDeclaration(_)
                | Self::TSInterfaceDeclaration(_)
                | Self::TSEnumDeclaration(_)
                | Self::TSModuleDeclaration(_)
                | Self::TSGlobalDeclaration(_)
                | Self::TSImportEqualsDeclaration(_)
        )
    }

    /// Convert a [`Statement`] to a [`Declaration`].
    ///
    /// # Panics
    /// Panics if not convertible.
    #[inline]
    pub fn into_declaration(self) -> Declaration<'a> {
        Declaration::try_from(self).unwrap()
    }

    /// Convert a [`&Statement`] to a [`&Declaration`].
    ///
    /// [`&Statement`]: Statement
    /// [`&Declaration`]: Declaration
    #[inline]
    pub fn as_declaration(&self) -> Option<&Declaration<'a>> {
        if self.is_declaration() {
            // SAFETY: Transmute is safe because discriminants + types are identical between
            // `parent` and `child` for the shared variants
            Some(unsafe { NonNull::from_ref(self).cast::<Declaration>().as_ref() })
        } else {
            None
        }
    }

    /// Convert a [`&mut Statement`] to a [`&mut Declaration`].
    ///
    /// [`&mut Statement`]: Statement
    /// [`&mut Declaration`]: Declaration
    #[inline]
    pub fn as_declaration_mut(&mut self) -> Option<&mut Declaration<'a>> {
        if self.is_declaration() {
            // SAFETY: Transmute is safe because discriminants + types are identical between
            // `parent` and `child` for the shared variants
            Some(unsafe { NonNull::from_mut(self).cast::<Declaration>().as_mut() })
        } else {
            None
        }
    }

    /// Convert a [`&Statement`] to a [`&Declaration`].
    ///
    /// # Panics
    /// Panics if not convertible.
    ///
    /// [`&Statement`]: Statement
    /// [`&Declaration`]: Declaration
    #[inline]
    pub fn to_declaration(&self) -> &Declaration<'a> {
        self.as_declaration().unwrap()
    }

    /// Convert a [`&mut Statement`] to a [`&mut Declaration`].
    ///
    /// # Panics
    /// Panics if not convertible.
    ///
    /// [`&mut Statement`]: Statement
    /// [`&mut Declaration`]: Declaration
    #[inline]
    pub fn to_declaration_mut(&mut self) -> &mut Declaration<'a> {
        self.as_declaration_mut().unwrap()
    }
}

impl<'a> Declaration<'a> {
    /// Convert a [`&Declaration`] to a [`&Statement`].
    ///
    /// [`&Declaration`]: Declaration
    /// [`&Statement`]: Statement
    #[inline]
    pub fn as_statement(&self) -> &Statement<'a> {
        // SAFETY: Transmute is safe because discriminants + types are identical between
        // `parent` and `child` for the shared variants
        unsafe { NonNull::from_ref(self).cast::<Statement>().as_ref() }
    }
}

impl<'a> TryFrom<Statement<'a>> for Declaration<'a> {
    type Error = ();

    /// Convert a [`Statement`] to a [`Declaration`].
    ///
    /// # Errors
    /// Returns `Err` if not convertible.
    #[inline]
    fn try_from(value: Statement<'a>) -> Result<Self, Self::Error> {
        // Compiler should implement this as a check of discriminant and then zero-cost transmute,
        // as discriminants for `parent` and `child` are aligned
        match value {
            Statement::VariableDeclaration(o) => Ok(Declaration::VariableDeclaration(o)),
            Statement::FunctionDeclaration(o) => Ok(Declaration::FunctionDeclaration(o)),
            Statement::ClassDeclaration(o) => Ok(Declaration::ClassDeclaration(o)),
            Statement::TSTypeAliasDeclaration(o) => Ok(Declaration::TSTypeAliasDeclaration(o)),
            Statement::TSInterfaceDeclaration(o) => Ok(Declaration::TSInterfaceDeclaration(o)),
            Statement::TSEnumDeclaration(o) => Ok(Declaration::TSEnumDeclaration(o)),
            Statement::TSModuleDeclaration(o) => Ok(Declaration::TSModuleDeclaration(o)),
            Statement::TSGlobalDeclaration(o) => Ok(Declaration::TSGlobalDeclaration(o)),
            Statement::TSImportEqualsDeclaration(o) => {
                Ok(Declaration::TSImportEqualsDeclaration(o))
            }
            _ => Err(()),
        }
    }
}

impl<'a> From<Declaration<'a>> for Statement<'a> {
    /// Convert a [`Declaration`] to a [`Statement`].
    #[inline]
    fn from(value: Declaration<'a>) -> Self {
        // Compiler should implement this as zero-cost transmute as discriminants
        // for `child` and `parent` are aligned
        match value {
            Declaration::VariableDeclaration(o) => Statement::VariableDeclaration(o),
            Declaration::FunctionDeclaration(o) => Statement::FunctionDeclaration(o),
            Declaration::ClassDeclaration(o) => Statement::ClassDeclaration(o),
            Declaration::TSTypeAliasDeclaration(o) => Statement::TSTypeAliasDeclaration(o),
            Declaration::TSInterfaceDeclaration(o) => Statement::TSInterfaceDeclaration(o),
            Declaration::TSEnumDeclaration(o) => Statement::TSEnumDeclaration(o),
            Declaration::TSModuleDeclaration(o) => Statement::TSModuleDeclaration(o),
            Declaration::TSGlobalDeclaration(o) => Statement::TSGlobalDeclaration(o),
            Declaration::TSImportEqualsDeclaration(o) => Statement::TSImportEqualsDeclaration(o),
        }
    }
}

impl<'a> Statement<'a> {
    /// Return if a [`Statement`] is a [`ModuleDeclaration`].
    #[inline]
    pub fn is_module_declaration(&self) -> bool {
        matches!(
            self,
            Self::ImportDeclaration(_)
                | Self::ExportAllDeclaration(_)
                | Self::ExportDefaultDeclaration(_)
                | Self::ExportNamedDeclaration(_)
                | Self::TSExportAssignment(_)
                | Self::TSNamespaceExportDeclaration(_)
        )
    }

    /// Convert a [`Statement`] to a [`ModuleDeclaration`].
    ///
    /// # Panics
    /// Panics if not convertible.
    #[inline]
    pub fn into_module_declaration(self) -> ModuleDeclaration<'a> {
        ModuleDeclaration::try_from(self).unwrap()
    }

    /// Convert a [`&Statement`] to a [`&ModuleDeclaration`].
    ///
    /// [`&Statement`]: Statement
    /// [`&ModuleDeclaration`]: ModuleDeclaration
    #[inline]
    pub fn as_module_declaration(&self) -> Option<&ModuleDeclaration<'a>> {
        if self.is_module_declaration() {
            // SAFETY: Transmute is safe because discriminants + types are identical between
            // `parent` and `child` for the shared variants
            Some(unsafe { NonNull::from_ref(self).cast::<ModuleDeclaration>().as_ref() })
        } else {
            None
        }
    }

    /// Convert a [`&mut Statement`] to a [`&mut ModuleDeclaration`].
    ///
    /// [`&mut Statement`]: Statement
    /// [`&mut ModuleDeclaration`]: ModuleDeclaration
    #[inline]
    pub fn as_module_declaration_mut(&mut self) -> Option<&mut ModuleDeclaration<'a>> {
        if self.is_module_declaration() {
            // SAFETY: Transmute is safe because discriminants + types are identical between
            // `parent` and `child` for the shared variants
            Some(unsafe { NonNull::from_mut(self).cast::<ModuleDeclaration>().as_mut() })
        } else {
            None
        }
    }

    /// Convert a [`&Statement`] to a [`&ModuleDeclaration`].
    ///
    /// # Panics
    /// Panics if not convertible.
    ///
    /// [`&Statement`]: Statement
    /// [`&ModuleDeclaration`]: ModuleDeclaration
    #[inline]
    pub fn to_module_declaration(&self) -> &ModuleDeclaration<'a> {
        self.as_module_declaration().unwrap()
    }

    /// Convert a [`&mut Statement`] to a [`&mut ModuleDeclaration`].
    ///
    /// # Panics
    /// Panics if not convertible.
    ///
    /// [`&mut Statement`]: Statement
    /// [`&mut ModuleDeclaration`]: ModuleDeclaration
    #[inline]
    pub fn to_module_declaration_mut(&mut self) -> &mut ModuleDeclaration<'a> {
        self.as_module_declaration_mut().unwrap()
    }
}

impl<'a> ModuleDeclaration<'a> {
    /// Convert a [`&ModuleDeclaration`] to a [`&Statement`].
    ///
    /// [`&ModuleDeclaration`]: ModuleDeclaration
    /// [`&Statement`]: Statement
    #[inline]
    pub fn as_statement(&self) -> &Statement<'a> {
        // SAFETY: Transmute is safe because discriminants + types are identical between
        // `parent` and `child` for the shared variants
        unsafe { NonNull::from_ref(self).cast::<Statement>().as_ref() }
    }
}

impl<'a> TryFrom<Statement<'a>> for ModuleDeclaration<'a> {
    type Error = ();

    /// Convert a [`Statement`] to a [`ModuleDeclaration`].
    ///
    /// # Errors
    /// Returns `Err` if not convertible.
    #[inline]
    fn try_from(value: Statement<'a>) -> Result<Self, Self::Error> {
        // Compiler should implement this as a check of discriminant and then zero-cost transmute,
        // as discriminants for `parent` and `child` are aligned
        match value {
            Statement::ImportDeclaration(o) => Ok(ModuleDeclaration::ImportDeclaration(o)),
            Statement::ExportAllDeclaration(o) => Ok(ModuleDeclaration::ExportAllDeclaration(o)),
            Statement::ExportDefaultDeclaration(o) => {
                Ok(ModuleDeclaration::ExportDefaultDeclaration(o))
            }
            Statement::ExportNamedDeclaration(o) => {
                Ok(ModuleDeclaration::ExportNamedDeclaration(o))
            }
            Statement::TSExportAssignment(o) => Ok(ModuleDeclaration::TSExportAssignment(o)),
            Statement::TSNamespaceExportDeclaration(o) => {
                Ok(ModuleDeclaration::TSNamespaceExportDeclaration(o))
            }
            _ => Err(()),
        }
    }
}

impl<'a> From<ModuleDeclaration<'a>> for Statement<'a> {
    /// Convert a [`ModuleDeclaration`] to a [`Statement`].
    #[inline]
    fn from(value: ModuleDeclaration<'a>) -> Self {
        // Compiler should implement this as zero-cost transmute as discriminants
        // for `child` and `parent` are aligned
        match value {
            ModuleDeclaration::ImportDeclaration(o) => Statement::ImportDeclaration(o),
            ModuleDeclaration::ExportAllDeclaration(o) => Statement::ExportAllDeclaration(o),
            ModuleDeclaration::ExportDefaultDeclaration(o) => {
                Statement::ExportDefaultDeclaration(o)
            }
            ModuleDeclaration::ExportNamedDeclaration(o) => Statement::ExportNamedDeclaration(o),
            ModuleDeclaration::TSExportAssignment(o) => Statement::TSExportAssignment(o),
            ModuleDeclaration::TSNamespaceExportDeclaration(o) => {
                Statement::TSNamespaceExportDeclaration(o)
            }
        }
    }
}

impl<'a> ForStatementLeft<'a> {
    /// Return if a [`ForStatementLeft`] is a [`AssignmentTarget`].
    #[inline]
    pub fn is_assignment_target(&self) -> bool {
        matches!(
            self,
            Self::AssignmentTargetIdentifier(_)
                | Self::TSAsExpression(_)
                | Self::TSSatisfiesExpression(_)
                | Self::TSNonNullExpression(_)
                | Self::TSTypeAssertion(_)
                | Self::MemberExpression(_)
                | Self::ArrayAssignmentTarget(_)
                | Self::ObjectAssignmentTarget(_)
        )
    }

    /// Convert a [`ForStatementLeft`] to a [`AssignmentTarget`].
    ///
    /// # Panics
    /// Panics if not convertible.
    #[inline]
    pub fn into_assignment_target(self) -> AssignmentTarget<'a> {
        AssignmentTarget::try_from(self).unwrap()
    }

    /// Convert an [`&ForStatementLeft`] to an [`&AssignmentTarget`].
    ///
    /// [`&ForStatementLeft`]: ForStatementLeft
    /// [`&AssignmentTarget`]: AssignmentTarget
    #[inline]
    pub fn as_assignment_target(&self) -> Option<&AssignmentTarget<'a>> {
        if self.is_assignment_target() {
            // SAFETY: Transmute is safe because discriminants + types are identical between
            // `parent` and `child` for the shared variants
            Some(unsafe { NonNull::from_ref(self).cast::<AssignmentTarget>().as_ref() })
        } else {
            None
        }
    }

    /// Convert an [`&mut ForStatementLeft`] to an [`&mut AssignmentTarget`].
    ///
    /// [`&mut ForStatementLeft`]: ForStatementLeft
    /// [`&mut AssignmentTarget`]: AssignmentTarget
    #[inline]
    pub fn as_assignment_target_mut(&mut self) -> Option<&mut AssignmentTarget<'a>> {
        if self.is_assignment_target() {
            // SAFETY: Transmute is safe because discriminants + types are identical between
            // `parent` and `child` for the shared variants
            Some(unsafe { NonNull::from_mut(self).cast::<AssignmentTarget>().as_mut() })
        } else {
            None
        }
    }

    /// Convert an [`&ForStatementLeft`] to an [`&AssignmentTarget`].
    ///
    /// # Panics
    /// Panics if not convertible.
    ///
    /// [`&ForStatementLeft`]: ForStatementLeft
    /// [`&AssignmentTarget`]: AssignmentTarget
    #[inline]
    pub fn to_assignment_target(&self) -> &AssignmentTarget<'a> {
        self.as_assignment_target().unwrap()
    }

    /// Convert an [`&mut ForStatementLeft`] to an [`&mut AssignmentTarget`].
    ///
    /// # Panics
    /// Panics if not convertible.
    ///
    /// [`&mut ForStatementLeft`]: ForStatementLeft
    /// [`&mut AssignmentTarget`]: AssignmentTarget
    #[inline]
    pub fn to_assignment_target_mut(&mut self) -> &mut AssignmentTarget<'a> {
        self.as_assignment_target_mut().unwrap()
    }
}

impl<'a> TryFrom<ForStatementLeft<'a>> for AssignmentTarget<'a> {
    type Error = ();

    /// Convert a [`ForStatementLeft`] to a [`AssignmentTarget`].
    ///
    /// # Errors
    /// Returns `Err` if not convertible.
    #[inline]
    fn try_from(value: ForStatementLeft<'a>) -> Result<Self, Self::Error> {
        // Compiler should implement this as a check of discriminant and then zero-cost transmute,
        // as discriminants for `parent` and `child` are aligned
        match value {
            ForStatementLeft::AssignmentTargetIdentifier(o) => {
                Ok(AssignmentTarget::AssignmentTargetIdentifier(o))
            }
            ForStatementLeft::TSAsExpression(o) => Ok(AssignmentTarget::TSAsExpression(o)),
            ForStatementLeft::TSSatisfiesExpression(o) => {
                Ok(AssignmentTarget::TSSatisfiesExpression(o))
            }
            ForStatementLeft::TSNonNullExpression(o) => {
                Ok(AssignmentTarget::TSNonNullExpression(o))
            }
            ForStatementLeft::TSTypeAssertion(o) => Ok(AssignmentTarget::TSTypeAssertion(o)),
            ForStatementLeft::MemberExpression(o) => Ok(AssignmentTarget::MemberExpression(o)),
            ForStatementLeft::ArrayAssignmentTarget(o) => {
                Ok(AssignmentTarget::ArrayAssignmentTarget(o))
            }
            ForStatementLeft::ObjectAssignmentTarget(o) => {
                Ok(AssignmentTarget::ObjectAssignmentTarget(o))
            }
            _ => Err(()),
        }
    }
}

impl<'a> From<AssignmentTarget<'a>> for ForStatementLeft<'a> {
    /// Convert a [`AssignmentTarget`] to a [`ForStatementLeft`].
    #[inline]
    fn from(value: AssignmentTarget<'a>) -> Self {
        // Compiler should implement this as zero-cost transmute as discriminants
        // for `child` and `parent` are aligned
        match value {
            AssignmentTarget::AssignmentTargetIdentifier(o) => {
                ForStatementLeft::AssignmentTargetIdentifier(o)
            }
            AssignmentTarget::TSAsExpression(o) => ForStatementLeft::TSAsExpression(o),
            AssignmentTarget::TSSatisfiesExpression(o) => {
                ForStatementLeft::TSSatisfiesExpression(o)
            }
            AssignmentTarget::TSNonNullExpression(o) => ForStatementLeft::TSNonNullExpression(o),
            AssignmentTarget::TSTypeAssertion(o) => ForStatementLeft::TSTypeAssertion(o),
            AssignmentTarget::MemberExpression(o) => ForStatementLeft::MemberExpression(o),
            AssignmentTarget::ArrayAssignmentTarget(o) => {
                ForStatementLeft::ArrayAssignmentTarget(o)
            }
            AssignmentTarget::ObjectAssignmentTarget(o) => {
                ForStatementLeft::ObjectAssignmentTarget(o)
            }
        }
    }
}

impl<'a> AssignmentTarget<'a> {
    /// Convert a [`&AssignmentTarget`] to a [`&ForStatementLeft`].
    ///
    /// [`&AssignmentTarget`]: AssignmentTarget
    /// [`&ForStatementLeft`]: ForStatementLeft
    #[inline]
    pub fn as_for_statement_left(&self) -> &ForStatementLeft<'a> {
        // SAFETY: Transmute is safe because discriminants + types are identical between
        // `parent` and `child` for the shared variants
        unsafe { NonNull::from_ref(self).cast::<ForStatementLeft>().as_ref() }
    }
}

impl<'a> ForStatementLeft<'a> {
    /// Return if a [`ForStatementLeft`] is a [`SimpleAssignmentTarget`].
    #[inline]
    pub fn is_simple_assignment_target(&self) -> bool {
        matches!(
            self,
            Self::AssignmentTargetIdentifier(_)
                | Self::TSAsExpression(_)
                | Self::TSSatisfiesExpression(_)
                | Self::TSNonNullExpression(_)
                | Self::TSTypeAssertion(_)
                | Self::MemberExpression(_)
        )
    }

    /// Convert a [`ForStatementLeft`] to a [`SimpleAssignmentTarget`].
    ///
    /// # Panics
    /// Panics if not convertible.
    #[inline]
    pub fn into_simple_assignment_target(self) -> SimpleAssignmentTarget<'a> {
        SimpleAssignmentTarget::try_from(self).unwrap()
    }

    /// Convert an [`&ForStatementLeft`] to an [`&SimpleAssignmentTarget`].
    ///
    /// [`&ForStatementLeft`]: ForStatementLeft
    /// [`&SimpleAssignmentTarget`]: SimpleAssignmentTarget
    #[inline]
    pub fn as_simple_assignment_target(&self) -> Option<&SimpleAssignmentTarget<'a>> {
        if self.is_simple_assignment_target() {
            // SAFETY: Transmute is safe because discriminants + types are identical between
            // `parent` and `child` for the shared variants
            Some(unsafe { NonNull::from_ref(self).cast::<SimpleAssignmentTarget>().as_ref() })
        } else {
            None
        }
    }

    /// Convert an [`&mut ForStatementLeft`] to an [`&mut SimpleAssignmentTarget`].
    ///
    /// [`&mut ForStatementLeft`]: ForStatementLeft
    /// [`&mut SimpleAssignmentTarget`]: SimpleAssignmentTarget
    #[inline]
    pub fn as_simple_assignment_target_mut(&mut self) -> Option<&mut SimpleAssignmentTarget<'a>> {
        if self.is_simple_assignment_target() {
            // SAFETY: Transmute is safe because discriminants + types are identical between
            // `parent` and `child` for the shared variants
            Some(unsafe { NonNull::from_mut(self).cast::<SimpleAssignmentTarget>().as_mut() })
        } else {
            None
        }
    }

    /// Convert an [`&ForStatementLeft`] to an [`&SimpleAssignmentTarget`].
    ///
    /// # Panics
    /// Panics if not convertible.
    ///
    /// [`&ForStatementLeft`]: ForStatementLeft
    /// [`&SimpleAssignmentTarget`]: SimpleAssignmentTarget
    #[inline]
    pub fn to_simple_assignment_target(&self) -> &SimpleAssignmentTarget<'a> {
        self.as_simple_assignment_target().unwrap()
    }

    /// Convert an [`&mut ForStatementLeft`] to an [`&mut SimpleAssignmentTarget`].
    ///
    /// # Panics
    /// Panics if not convertible.
    ///
    /// [`&mut ForStatementLeft`]: ForStatementLeft
    /// [`&mut SimpleAssignmentTarget`]: SimpleAssignmentTarget
    #[inline]
    pub fn to_simple_assignment_target_mut(&mut self) -> &mut SimpleAssignmentTarget<'a> {
        self.as_simple_assignment_target_mut().unwrap()
    }
}

impl<'a> TryFrom<ForStatementLeft<'a>> for SimpleAssignmentTarget<'a> {
    type Error = ();

    /// Convert a [`ForStatementLeft`] to a [`SimpleAssignmentTarget`].
    ///
    /// # Errors
    /// Returns `Err` if not convertible.
    #[inline]
    fn try_from(value: ForStatementLeft<'a>) -> Result<Self, Self::Error> {
        // Compiler should implement this as a check of discriminant and then zero-cost transmute,
        // as discriminants for `parent` and `child` are aligned
        match value {
            ForStatementLeft::AssignmentTargetIdentifier(o) => {
                Ok(SimpleAssignmentTarget::AssignmentTargetIdentifier(o))
            }
            ForStatementLeft::TSAsExpression(o) => Ok(SimpleAssignmentTarget::TSAsExpression(o)),
            ForStatementLeft::TSSatisfiesExpression(o) => {
                Ok(SimpleAssignmentTarget::TSSatisfiesExpression(o))
            }
            ForStatementLeft::TSNonNullExpression(o) => {
                Ok(SimpleAssignmentTarget::TSNonNullExpression(o))
            }
            ForStatementLeft::TSTypeAssertion(o) => Ok(SimpleAssignmentTarget::TSTypeAssertion(o)),
            ForStatementLeft::MemberExpression(o) => {
                Ok(SimpleAssignmentTarget::MemberExpression(o))
            }
            _ => Err(()),
        }
    }
}

impl<'a> From<SimpleAssignmentTarget<'a>> for ForStatementLeft<'a> {
    /// Convert a [`SimpleAssignmentTarget`] to a [`ForStatementLeft`].
    #[inline]
    fn from(value: SimpleAssignmentTarget<'a>) -> Self {
        // Compiler should implement this as zero-cost transmute as discriminants
        // for `child` and `parent` are aligned
        match value {
            SimpleAssignmentTarget::AssignmentTargetIdentifier(o) => {
                ForStatementLeft::AssignmentTargetIdentifier(o)
            }
            SimpleAssignmentTarget::TSAsExpression(o) => ForStatementLeft::TSAsExpression(o),
            SimpleAssignmentTarget::TSSatisfiesExpression(o) => {
                ForStatementLeft::TSSatisfiesExpression(o)
            }
            SimpleAssignmentTarget::TSNonNullExpression(o) => {
                ForStatementLeft::TSNonNullExpression(o)
            }
            SimpleAssignmentTarget::TSTypeAssertion(o) => ForStatementLeft::TSTypeAssertion(o),
            SimpleAssignmentTarget::MemberExpression(o) => ForStatementLeft::MemberExpression(o),
        }
    }
}

impl<'a> SimpleAssignmentTarget<'a> {
    /// Convert a [`&SimpleAssignmentTarget`] to a [`&ForStatementLeft`].
    ///
    /// [`&SimpleAssignmentTarget`]: SimpleAssignmentTarget
    /// [`&ForStatementLeft`]: ForStatementLeft
    #[inline]
    pub fn as_for_statement_left(&self) -> &ForStatementLeft<'a> {
        // SAFETY: Transmute is safe because discriminants + types are identical between
        // `parent` and `child` for the shared variants
        unsafe { NonNull::from_ref(self).cast::<ForStatementLeft>().as_ref() }
    }
}

impl<'a> ForStatementLeft<'a> {
    /// Return if a [`ForStatementLeft`] is a [`MemberExpression`].
    #[inline]
    pub fn is_member_expression(&self) -> bool {
        matches!(self, Self::MemberExpression(_))
    }

    /// Convert a [`ForStatementLeft`] into a [`MemberExpression`].
    ///
    /// # Panics
    /// Panics if not convertible.
    #[inline]
    pub fn into_member_expression(self) -> MemberExpression<'a> {
        match self {
            Self::MemberExpression(it) => it,
            _ => panic!("Cannot convert to MemberExpression"),
        }
    }

    /// Convert a [`&ForStatementLeft`] to a [`&MemberExpression`].
    ///
    /// [`&ForStatementLeft`]: ForStatementLeft
    /// [`&MemberExpression`]: MemberExpression
    #[inline]
    pub fn as_member_expression(&self) -> Option<&MemberExpression<'a>> {
        if let Self::MemberExpression(it) = self { Some(it) } else { None }
    }

    /// Convert a [`&mut ForStatementLeft`] to a [`&mut MemberExpression`].
    ///
    /// [`&mut ForStatementLeft`]: ForStatementLeft
    /// [`&mut MemberExpression`]: MemberExpression
    #[inline]
    pub fn as_member_expression_mut(&mut self) -> Option<&mut MemberExpression<'a>> {
        if let Self::MemberExpression(it) = self { Some(it) } else { None }
    }

    /// Convert a [`&ForStatementLeft`] to a [`&MemberExpression`].
    ///
    /// # Panics
    /// Panics if not convertible.
    ///
    /// [`&ForStatementLeft`]: ForStatementLeft
    /// [`&MemberExpression`]: MemberExpression
    #[inline]
    pub fn to_member_expression(&self) -> &MemberExpression<'a> {
        self.as_member_expression().unwrap()
    }

    /// Convert a [`&mut ForStatementLeft`] to a [`&mut MemberExpression`].
    ///
    /// # Panics
    /// Panics if not convertible.
    ///
    /// [`&mut ForStatementLeft`]: ForStatementLeft
    /// [`&mut MemberExpression`]: MemberExpression
    #[inline]
    pub fn to_member_expression_mut(&mut self) -> &mut MemberExpression<'a> {
        self.as_member_expression_mut().unwrap()
    }
}

impl<'a> TryFrom<ForStatementLeft<'a>> for MemberExpression<'a> {
    type Error = ();

    /// Convert a [`ForStatementLeft`] to a [`MemberExpression`].
    ///
    /// # Errors
    /// Returns `Err` if not convertible.
    #[inline]
    fn try_from(value: ForStatementLeft<'a>) -> Result<Self, Self::Error> {
        match value {
            ForStatementLeft::MemberExpression(it) => Ok(it),
            _ => Err(()),
        }
    }
}

impl<'a> From<MemberExpression<'a>> for ForStatementLeft<'a> {
    /// Convert a [`MemberExpression`] to a [`ForStatementLeft`].
    #[inline]
    fn from(value: MemberExpression<'a>) -> Self {
        ForStatementLeft::MemberExpression(value)
    }
}

impl<'a> ForStatementLeft<'a> {
    /// Return if a [`ForStatementLeft`] is a [`AssignmentTargetPattern`].
    #[inline]
    pub fn is_assignment_target_pattern(&self) -> bool {
        matches!(self, Self::ArrayAssignmentTarget(_) | Self::ObjectAssignmentTarget(_))
    }

    /// Convert a [`ForStatementLeft`] to a [`AssignmentTargetPattern`].
    ///
    /// # Panics
    /// Panics if not convertible.
    #[inline]
    pub fn into_assignment_target_pattern(self) -> AssignmentTargetPattern<'a> {
        AssignmentTargetPattern::try_from(self).unwrap()
    }

    /// Convert an [`&ForStatementLeft`] to an [`&AssignmentTargetPattern`].
    ///
    /// [`&ForStatementLeft`]: ForStatementLeft
    /// [`&AssignmentTargetPattern`]: AssignmentTargetPattern
    #[inline]
    pub fn as_assignment_target_pattern(&self) -> Option<&AssignmentTargetPattern<'a>> {
        if self.is_assignment_target_pattern() {
            // SAFETY: Transmute is safe because discriminants + types are identical between
            // `parent` and `child` for the shared variants
            Some(unsafe { NonNull::from_ref(self).cast::<AssignmentTargetPattern>().as_ref() })
        } else {
            None
        }
    }

    /// Convert an [`&mut ForStatementLeft`] to an [`&mut AssignmentTargetPattern`].
    ///
    /// [`&mut ForStatementLeft`]: ForStatementLeft
    /// [`&mut AssignmentTargetPattern`]: AssignmentTargetPattern
    #[inline]
    pub fn as_assignment_target_pattern_mut(&mut self) -> Option<&mut AssignmentTargetPattern<'a>> {
        if self.is_assignment_target_pattern() {
            // SAFETY: Transmute is safe because discriminants + types are identical between
            // `parent` and `child` for the shared variants
            Some(unsafe { NonNull::from_mut(self).cast::<AssignmentTargetPattern>().as_mut() })
        } else {
            None
        }
    }

    /// Convert an [`&ForStatementLeft`] to an [`&AssignmentTargetPattern`].
    ///
    /// # Panics
    /// Panics if not convertible.
    ///
    /// [`&ForStatementLeft`]: ForStatementLeft
    /// [`&AssignmentTargetPattern`]: AssignmentTargetPattern
    #[inline]
    pub fn to_assignment_target_pattern(&self) -> &AssignmentTargetPattern<'a> {
        self.as_assignment_target_pattern().unwrap()
    }

    /// Convert an [`&mut ForStatementLeft`] to an [`&mut AssignmentTargetPattern`].
    ///
    /// # Panics
    /// Panics if not convertible.
    ///
    /// [`&mut ForStatementLeft`]: ForStatementLeft
    /// [`&mut AssignmentTargetPattern`]: AssignmentTargetPattern
    #[inline]
    pub fn to_assignment_target_pattern_mut(&mut self) -> &mut AssignmentTargetPattern<'a> {
        self.as_assignment_target_pattern_mut().unwrap()
    }
}

impl<'a> TryFrom<ForStatementLeft<'a>> for AssignmentTargetPattern<'a> {
    type Error = ();

    /// Convert a [`ForStatementLeft`] to a [`AssignmentTargetPattern`].
    ///
    /// # Errors
    /// Returns `Err` if not convertible.
    #[inline]
    fn try_from(value: ForStatementLeft<'a>) -> Result<Self, Self::Error> {
        // Compiler should implement this as a check of discriminant and then zero-cost transmute,
        // as discriminants for `parent` and `child` are aligned
        match value {
            ForStatementLeft::ArrayAssignmentTarget(o) => {
                Ok(AssignmentTargetPattern::ArrayAssignmentTarget(o))
            }
            ForStatementLeft::ObjectAssignmentTarget(o) => {
                Ok(AssignmentTargetPattern::ObjectAssignmentTarget(o))
            }
            _ => Err(()),
        }
    }
}

impl<'a> From<AssignmentTargetPattern<'a>> for ForStatementLeft<'a> {
    /// Convert a [`AssignmentTargetPattern`] to a [`ForStatementLeft`].
    #[inline]
    fn from(value: AssignmentTargetPattern<'a>) -> Self {
        // Compiler should implement this as zero-cost transmute as discriminants
        // for `child` and `parent` are aligned
        match value {
            AssignmentTargetPattern::ArrayAssignmentTarget(o) => {
                ForStatementLeft::ArrayAssignmentTarget(o)
            }
            AssignmentTargetPattern::ObjectAssignmentTarget(o) => {
                ForStatementLeft::ObjectAssignmentTarget(o)
            }
        }
    }
}

impl<'a> AssignmentTargetPattern<'a> {
    /// Convert a [`&AssignmentTargetPattern`] to a [`&ForStatementLeft`].
    ///
    /// [`&AssignmentTargetPattern`]: AssignmentTargetPattern
    /// [`&ForStatementLeft`]: ForStatementLeft
    #[inline]
    pub fn as_for_statement_left(&self) -> &ForStatementLeft<'a> {
        // SAFETY: Transmute is safe because discriminants + types are identical between
        // `parent` and `child` for the shared variants
        unsafe { NonNull::from_ref(self).cast::<ForStatementLeft>().as_ref() }
    }
}

impl<'a> TSTupleElement<'a> {
    /// Return if a [`TSTupleElement`] is a [`TSType`].
    #[inline]
    pub fn is_ts_type(&self) -> bool {
        matches!(
            self,
            Self::TSAnyKeyword(_)
                | Self::TSBigIntKeyword(_)
                | Self::TSBooleanKeyword(_)
                | Self::TSIntrinsicKeyword(_)
                | Self::TSNeverKeyword(_)
                | Self::TSNullKeyword(_)
                | Self::TSNumberKeyword(_)
                | Self::TSObjectKeyword(_)
                | Self::TSStringKeyword(_)
                | Self::TSSymbolKeyword(_)
                | Self::TSUndefinedKeyword(_)
                | Self::TSUnknownKeyword(_)
                | Self::TSVoidKeyword(_)
                | Self::TSArrayType(_)
                | Self::TSConditionalType(_)
                | Self::TSConstructorType(_)
                | Self::TSFunctionType(_)
                | Self::TSImportType(_)
                | Self::TSIndexedAccessType(_)
                | Self::TSInferType(_)
                | Self::TSIntersectionType(_)
                | Self::TSLiteralType(_)
                | Self::TSMappedType(_)
                | Self::TSNamedTupleMember(_)
                | Self::TSTemplateLiteralType(_)
                | Self::TSThisType(_)
                | Self::TSTupleType(_)
                | Self::TSTypeLiteral(_)
                | Self::TSTypeOperatorType(_)
                | Self::TSTypePredicate(_)
                | Self::TSTypeQuery(_)
                | Self::TSTypeReference(_)
                | Self::TSUnionType(_)
                | Self::TSParenthesizedType(_)
                | Self::JSDocNullableType(_)
                | Self::JSDocNonNullableType(_)
                | Self::JSDocUnknownType(_)
        )
    }

    /// Convert a [`TSTupleElement`] to a [`TSType`].
    ///
    /// # Panics
    /// Panics if not convertible.
    #[inline]
    pub fn into_ts_type(self) -> TSType<'a> {
        TSType::try_from(self).unwrap()
    }

    /// Convert a [`&TSTupleElement`] to a [`&TSType`].
    ///
    /// [`&TSTupleElement`]: TSTupleElement
    /// [`&TSType`]: TSType
    #[inline]
    pub fn as_ts_type(&self) -> Option<&TSType<'a>> {
        if self.is_ts_type() {
            // SAFETY: Transmute is safe because discriminants + types are identical between
            // `parent` and `child` for the shared variants
            Some(unsafe { NonNull::from_ref(self).cast::<TSType>().as_ref() })
        } else {
            None
        }
    }

    /// Convert a [`&mut TSTupleElement`] to a [`&mut TSType`].
    ///
    /// [`&mut TSTupleElement`]: TSTupleElement
    /// [`&mut TSType`]: TSType
    #[inline]
    pub fn as_ts_type_mut(&mut self) -> Option<&mut TSType<'a>> {
        if self.is_ts_type() {
            // SAFETY: Transmute is safe because discriminants + types are identical between
            // `parent` and `child` for the shared variants
            Some(unsafe { NonNull::from_mut(self).cast::<TSType>().as_mut() })
        } else {
            None
        }
    }

    /// Convert a [`&TSTupleElement`] to a [`&TSType`].
    ///
    /// # Panics
    /// Panics if not convertible.
    ///
    /// [`&TSTupleElement`]: TSTupleElement
    /// [`&TSType`]: TSType
    #[inline]
    pub fn to_ts_type(&self) -> &TSType<'a> {
        self.as_ts_type().unwrap()
    }

    /// Convert a [`&mut TSTupleElement`] to a [`&mut TSType`].
    ///
    /// # Panics
    /// Panics if not convertible.
    ///
    /// [`&mut TSTupleElement`]: TSTupleElement
    /// [`&mut TSType`]: TSType
    #[inline]
    pub fn to_ts_type_mut(&mut self) -> &mut TSType<'a> {
        self.as_ts_type_mut().unwrap()
    }
}

impl<'a> TSType<'a> {
    /// Convert a [`&TSType`] to a [`&TSTupleElement`].
    ///
    /// [`&TSType`]: TSType
    /// [`&TSTupleElement`]: TSTupleElement
    #[inline]
    pub fn as_ts_tuple_element(&self) -> &TSTupleElement<'a> {
        // SAFETY: Transmute is safe because discriminants + types are identical between
        // `parent` and `child` for the shared variants
        unsafe { NonNull::from_ref(self).cast::<TSTupleElement>().as_ref() }
    }
}

impl<'a> TryFrom<TSTupleElement<'a>> for TSType<'a> {
    type Error = ();

    /// Convert a [`TSTupleElement`] to a [`TSType`].
    ///
    /// # Errors
    /// Returns `Err` if not convertible.
    #[inline]
    fn try_from(value: TSTupleElement<'a>) -> Result<Self, Self::Error> {
        // Compiler should implement this as a check of discriminant and then zero-cost transmute,
        // as discriminants for `parent` and `child` are aligned
        match value {
            TSTupleElement::TSAnyKeyword(o) => Ok(TSType::TSAnyKeyword(o)),
            TSTupleElement::TSBigIntKeyword(o) => Ok(TSType::TSBigIntKeyword(o)),
            TSTupleElement::TSBooleanKeyword(o) => Ok(TSType::TSBooleanKeyword(o)),
            TSTupleElement::TSIntrinsicKeyword(o) => Ok(TSType::TSIntrinsicKeyword(o)),
            TSTupleElement::TSNeverKeyword(o) => Ok(TSType::TSNeverKeyword(o)),
            TSTupleElement::TSNullKeyword(o) => Ok(TSType::TSNullKeyword(o)),
            TSTupleElement::TSNumberKeyword(o) => Ok(TSType::TSNumberKeyword(o)),
            TSTupleElement::TSObjectKeyword(o) => Ok(TSType::TSObjectKeyword(o)),
            TSTupleElement::TSStringKeyword(o) => Ok(TSType::TSStringKeyword(o)),
            TSTupleElement::TSSymbolKeyword(o) => Ok(TSType::TSSymbolKeyword(o)),
            TSTupleElement::TSUndefinedKeyword(o) => Ok(TSType::TSUndefinedKeyword(o)),
            TSTupleElement::TSUnknownKeyword(o) => Ok(TSType::TSUnknownKeyword(o)),
            TSTupleElement::TSVoidKeyword(o) => Ok(TSType::TSVoidKeyword(o)),
            TSTupleElement::TSArrayType(o) => Ok(TSType::TSArrayType(o)),
            TSTupleElement::TSConditionalType(o) => Ok(TSType::TSConditionalType(o)),
            TSTupleElement::TSConstructorType(o) => Ok(TSType::TSConstructorType(o)),
            TSTupleElement::TSFunctionType(o) => Ok(TSType::TSFunctionType(o)),
            TSTupleElement::TSImportType(o) => Ok(TSType::TSImportType(o)),
            TSTupleElement::TSIndexedAccessType(o) => Ok(TSType::TSIndexedAccessType(o)),
            TSTupleElement::TSInferType(o) => Ok(TSType::TSInferType(o)),
            TSTupleElement::TSIntersectionType(o) => Ok(TSType::TSIntersectionType(o)),
            TSTupleElement::TSLiteralType(o) => Ok(TSType::TSLiteralType(o)),
            TSTupleElement::TSMappedType(o) => Ok(TSType::TSMappedType(o)),
            TSTupleElement::TSNamedTupleMember(o) => Ok(TSType::TSNamedTupleMember(o)),
            TSTupleElement::TSTemplateLiteralType(o) => Ok(TSType::TSTemplateLiteralType(o)),
            TSTupleElement::TSThisType(o) => Ok(TSType::TSThisType(o)),
            TSTupleElement::TSTupleType(o) => Ok(TSType::TSTupleType(o)),
            TSTupleElement::TSTypeLiteral(o) => Ok(TSType::TSTypeLiteral(o)),
            TSTupleElement::TSTypeOperatorType(o) => Ok(TSType::TSTypeOperatorType(o)),
            TSTupleElement::TSTypePredicate(o) => Ok(TSType::TSTypePredicate(o)),
            TSTupleElement::TSTypeQuery(o) => Ok(TSType::TSTypeQuery(o)),
            TSTupleElement::TSTypeReference(o) => Ok(TSType::TSTypeReference(o)),
            TSTupleElement::TSUnionType(o) => Ok(TSType::TSUnionType(o)),
            TSTupleElement::TSParenthesizedType(o) => Ok(TSType::TSParenthesizedType(o)),
            TSTupleElement::JSDocNullableType(o) => Ok(TSType::JSDocNullableType(o)),
            TSTupleElement::JSDocNonNullableType(o) => Ok(TSType::JSDocNonNullableType(o)),
            TSTupleElement::JSDocUnknownType(o) => Ok(TSType::JSDocUnknownType(o)),
            _ => Err(()),
        }
    }
}

impl<'a> From<TSType<'a>> for TSTupleElement<'a> {
    /// Convert a [`TSType`] to a [`TSTupleElement`].
    #[inline]
    fn from(value: TSType<'a>) -> Self {
        // Compiler should implement this as zero-cost transmute as discriminants
        // for `child` and `parent` are aligned
        match value {
            TSType::TSAnyKeyword(o) => TSTupleElement::TSAnyKeyword(o),
            TSType::TSBigIntKeyword(o) => TSTupleElement::TSBigIntKeyword(o),
            TSType::TSBooleanKeyword(o) => TSTupleElement::TSBooleanKeyword(o),
            TSType::TSIntrinsicKeyword(o) => TSTupleElement::TSIntrinsicKeyword(o),
            TSType::TSNeverKeyword(o) => TSTupleElement::TSNeverKeyword(o),
            TSType::TSNullKeyword(o) => TSTupleElement::TSNullKeyword(o),
            TSType::TSNumberKeyword(o) => TSTupleElement::TSNumberKeyword(o),
            TSType::TSObjectKeyword(o) => TSTupleElement::TSObjectKeyword(o),
            TSType::TSStringKeyword(o) => TSTupleElement::TSStringKeyword(o),
            TSType::TSSymbolKeyword(o) => TSTupleElement::TSSymbolKeyword(o),
            TSType::TSUndefinedKeyword(o) => TSTupleElement::TSUndefinedKeyword(o),
            TSType::TSUnknownKeyword(o) => TSTupleElement::TSUnknownKeyword(o),
            TSType::TSVoidKeyword(o) => TSTupleElement::TSVoidKeyword(o),
            TSType::TSArrayType(o) => TSTupleElement::TSArrayType(o),
            TSType::TSConditionalType(o) => TSTupleElement::TSConditionalType(o),
            TSType::TSConstructorType(o) => TSTupleElement::TSConstructorType(o),
            TSType::TSFunctionType(o) => TSTupleElement::TSFunctionType(o),
            TSType::TSImportType(o) => TSTupleElement::TSImportType(o),
            TSType::TSIndexedAccessType(o) => TSTupleElement::TSIndexedAccessType(o),
            TSType::TSInferType(o) => TSTupleElement::TSInferType(o),
            TSType::TSIntersectionType(o) => TSTupleElement::TSIntersectionType(o),
            TSType::TSLiteralType(o) => TSTupleElement::TSLiteralType(o),
            TSType::TSMappedType(o) => TSTupleElement::TSMappedType(o),
            TSType::TSNamedTupleMember(o) => TSTupleElement::TSNamedTupleMember(o),
            TSType::TSTemplateLiteralType(o) => TSTupleElement::TSTemplateLiteralType(o),
            TSType::TSThisType(o) => TSTupleElement::TSThisType(o),
            TSType::TSTupleType(o) => TSTupleElement::TSTupleType(o),
            TSType::TSTypeLiteral(o) => TSTupleElement::TSTypeLiteral(o),
            TSType::TSTypeOperatorType(o) => TSTupleElement::TSTypeOperatorType(o),
            TSType::TSTypePredicate(o) => TSTupleElement::TSTypePredicate(o),
            TSType::TSTypeQuery(o) => TSTupleElement::TSTypeQuery(o),
            TSType::TSTypeReference(o) => TSTupleElement::TSTypeReference(o),
            TSType::TSUnionType(o) => TSTupleElement::TSUnionType(o),
            TSType::TSParenthesizedType(o) => TSTupleElement::TSParenthesizedType(o),
            TSType::JSDocNullableType(o) => TSTupleElement::JSDocNullableType(o),
            TSType::JSDocNonNullableType(o) => TSTupleElement::JSDocNonNullableType(o),
            TSType::JSDocUnknownType(o) => TSTupleElement::JSDocUnknownType(o),
        }
    }
}

impl<'a> TSTypeQueryExprName<'a> {
    /// Return if a [`TSTypeQueryExprName`] is a [`TSTypeName`].
    #[inline]
    pub fn is_ts_type_name(&self) -> bool {
        matches!(
            self,
            Self::IdentifierReference(_) | Self::QualifiedName(_) | Self::ThisExpression(_)
        )
    }

    /// Convert a [`TSTypeQueryExprName`] to a [`TSTypeName`].
    ///
    /// # Panics
    /// Panics if not convertible.
    #[inline]
    pub fn into_ts_type_name(self) -> TSTypeName<'a> {
        TSTypeName::try_from(self).unwrap()
    }

    /// Convert a [`&TSTypeQueryExprName`] to a [`&TSTypeName`].
    ///
    /// [`&TSTypeQueryExprName`]: TSTypeQueryExprName
    /// [`&TSTypeName`]: TSTypeName
    #[inline]
    pub fn as_ts_type_name(&self) -> Option<&TSTypeName<'a>> {
        if self.is_ts_type_name() {
            // SAFETY: Transmute is safe because discriminants + types are identical between
            // `parent` and `child` for the shared variants
            Some(unsafe { NonNull::from_ref(self).cast::<TSTypeName>().as_ref() })
        } else {
            None
        }
    }

    /// Convert a [`&mut TSTypeQueryExprName`] to a [`&mut TSTypeName`].
    ///
    /// [`&mut TSTypeQueryExprName`]: TSTypeQueryExprName
    /// [`&mut TSTypeName`]: TSTypeName
    #[inline]
    pub fn as_ts_type_name_mut(&mut self) -> Option<&mut TSTypeName<'a>> {
        if self.is_ts_type_name() {
            // SAFETY: Transmute is safe because discriminants + types are identical between
            // `parent` and `child` for the shared variants
            Some(unsafe { NonNull::from_mut(self).cast::<TSTypeName>().as_mut() })
        } else {
            None
        }
    }

    /// Convert a [`&TSTypeQueryExprName`] to a [`&TSTypeName`].
    ///
    /// # Panics
    /// Panics if not convertible.
    ///
    /// [`&TSTypeQueryExprName`]: TSTypeQueryExprName
    /// [`&TSTypeName`]: TSTypeName
    #[inline]
    pub fn to_ts_type_name(&self) -> &TSTypeName<'a> {
        self.as_ts_type_name().unwrap()
    }

    /// Convert a [`&mut TSTypeQueryExprName`] to a [`&mut TSTypeName`].
    ///
    /// # Panics
    /// Panics if not convertible.
    ///
    /// [`&mut TSTypeQueryExprName`]: TSTypeQueryExprName
    /// [`&mut TSTypeName`]: TSTypeName
    #[inline]
    pub fn to_ts_type_name_mut(&mut self) -> &mut TSTypeName<'a> {
        self.as_ts_type_name_mut().unwrap()
    }
}

impl<'a> TSTypeName<'a> {
    /// Convert a [`&TSTypeName`] to a [`&TSTypeQueryExprName`].
    ///
    /// [`&TSTypeName`]: TSTypeName
    /// [`&TSTypeQueryExprName`]: TSTypeQueryExprName
    #[inline]
    pub fn as_ts_type_query_expr_name(&self) -> &TSTypeQueryExprName<'a> {
        // SAFETY: Transmute is safe because discriminants + types are identical between
        // `parent` and `child` for the shared variants
        unsafe { NonNull::from_ref(self).cast::<TSTypeQueryExprName>().as_ref() }
    }
}

impl<'a> TryFrom<TSTypeQueryExprName<'a>> for TSTypeName<'a> {
    type Error = ();

    /// Convert a [`TSTypeQueryExprName`] to a [`TSTypeName`].
    ///
    /// # Errors
    /// Returns `Err` if not convertible.
    #[inline]
    fn try_from(value: TSTypeQueryExprName<'a>) -> Result<Self, Self::Error> {
        // Compiler should implement this as a check of discriminant and then zero-cost transmute,
        // as discriminants for `parent` and `child` are aligned
        match value {
            TSTypeQueryExprName::IdentifierReference(o) => Ok(TSTypeName::IdentifierReference(o)),
            TSTypeQueryExprName::QualifiedName(o) => Ok(TSTypeName::QualifiedName(o)),
            TSTypeQueryExprName::ThisExpression(o) => Ok(TSTypeName::ThisExpression(o)),
            _ => Err(()),
        }
    }
}

impl<'a> From<TSTypeName<'a>> for TSTypeQueryExprName<'a> {
    /// Convert a [`TSTypeName`] to a [`TSTypeQueryExprName`].
    #[inline]
    fn from(value: TSTypeName<'a>) -> Self {
        // Compiler should implement this as zero-cost transmute as discriminants
        // for `child` and `parent` are aligned
        match value {
            TSTypeName::IdentifierReference(o) => TSTypeQueryExprName::IdentifierReference(o),
            TSTypeName::QualifiedName(o) => TSTypeQueryExprName::QualifiedName(o),
            TSTypeName::ThisExpression(o) => TSTypeQueryExprName::ThisExpression(o),
        }
    }
}

/// Macro for matching [`AssignmentTarget`]'s variants.
///
/// Includes variants inherited from [`SimpleAssignmentTarget`], [`MemberExpression`], [`AssignmentTargetPattern`].
#[macro_export]
macro_rules! match_assignment_target {
    ($ty:ident) => {
        $ty::AssignmentTargetIdentifier(_)
            | $ty::TSAsExpression(_)
            | $ty::TSSatisfiesExpression(_)
            | $ty::TSNonNullExpression(_)
            | $ty::TSTypeAssertion(_)
            | $ty::MemberExpression(_)
            | $ty::ArrayAssignmentTarget(_)
            | $ty::ObjectAssignmentTarget(_)
    };
}
pub use match_assignment_target;

/// Macro for matching [`SimpleAssignmentTarget`]'s variants.
///
/// Includes variants inherited from [`MemberExpression`].
#[macro_export]
macro_rules! match_simple_assignment_target {
    ($ty:ident) => {
        $ty::AssignmentTargetIdentifier(_)
            | $ty::TSAsExpression(_)
            | $ty::TSSatisfiesExpression(_)
            | $ty::TSNonNullExpression(_)
            | $ty::TSTypeAssertion(_)
            | $ty::MemberExpression(_)
    };
}
pub use match_simple_assignment_target;

/// Macro for matching [`AssignmentTargetPattern`]'s variants.
#[macro_export]
macro_rules! match_assignment_target_pattern {
    ($ty:ident) => {
        $ty::ArrayAssignmentTarget(_) | $ty::ObjectAssignmentTarget(_)
    };
}
pub use match_assignment_target_pattern;

/// Macro for matching [`Declaration`]'s variants.
#[macro_export]
macro_rules! match_declaration {
    ($ty:ident) => {
        $ty::VariableDeclaration(_)
            | $ty::FunctionDeclaration(_)
            | $ty::ClassDeclaration(_)
            | $ty::TSTypeAliasDeclaration(_)
            | $ty::TSInterfaceDeclaration(_)
            | $ty::TSEnumDeclaration(_)
            | $ty::TSModuleDeclaration(_)
            | $ty::TSGlobalDeclaration(_)
            | $ty::TSImportEqualsDeclaration(_)
    };
}
pub use match_declaration;

/// Macro for matching [`ModuleDeclaration`]'s variants.
#[macro_export]
macro_rules! match_module_declaration {
    ($ty:ident) => {
        $ty::ImportDeclaration(_)
            | $ty::ExportAllDeclaration(_)
            | $ty::ExportDefaultDeclaration(_)
            | $ty::ExportNamedDeclaration(_)
            | $ty::TSExportAssignment(_)
            | $ty::TSNamespaceExportDeclaration(_)
    };
}
pub use match_module_declaration;

/// Macro for matching [`TSType`]'s variants.
#[macro_export]
macro_rules! match_ts_type {
    ($ty:ident) => {
        $ty::TSAnyKeyword(_)
            | $ty::TSBigIntKeyword(_)
            | $ty::TSBooleanKeyword(_)
            | $ty::TSIntrinsicKeyword(_)
            | $ty::TSNeverKeyword(_)
            | $ty::TSNullKeyword(_)
            | $ty::TSNumberKeyword(_)
            | $ty::TSObjectKeyword(_)
            | $ty::TSStringKeyword(_)
            | $ty::TSSymbolKeyword(_)
            | $ty::TSUndefinedKeyword(_)
            | $ty::TSUnknownKeyword(_)
            | $ty::TSVoidKeyword(_)
            | $ty::TSArrayType(_)
            | $ty::TSConditionalType(_)
            | $ty::TSConstructorType(_)
            | $ty::TSFunctionType(_)
            | $ty::TSImportType(_)
            | $ty::TSIndexedAccessType(_)
            | $ty::TSInferType(_)
            | $ty::TSIntersectionType(_)
            | $ty::TSLiteralType(_)
            | $ty::TSMappedType(_)
            | $ty::TSNamedTupleMember(_)
            | $ty::TSTemplateLiteralType(_)
            | $ty::TSThisType(_)
            | $ty::TSTupleType(_)
            | $ty::TSTypeLiteral(_)
            | $ty::TSTypeOperatorType(_)
            | $ty::TSTypePredicate(_)
            | $ty::TSTypeQuery(_)
            | $ty::TSTypeReference(_)
            | $ty::TSUnionType(_)
            | $ty::TSParenthesizedType(_)
            | $ty::JSDocNullableType(_)
            | $ty::JSDocNonNullableType(_)
            | $ty::JSDocUnknownType(_)
    };
}
pub use match_ts_type;

/// Macro for matching [`TSTypeName`]'s variants.
#[macro_export]
macro_rules! match_ts_type_name {
    ($ty:ident) => {
        $ty::IdentifierReference(_) | $ty::QualifiedName(_) | $ty::ThisExpression(_)
    };
}
pub use match_ts_type_name;
