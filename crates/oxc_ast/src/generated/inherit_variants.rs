// Auto-generated code, DO NOT EDIT DIRECTLY!
// To edit this generated file you have to edit `tasks/ast_tools/src/generators/inherit_variants.rs`.

// Some `TryFrom` impls have a single non-shared variant left for the catch-all arm
#![expect(clippy::match_wildcard_for_single_variants)]

use crate::ast::*;

impl<'a> Expression<'a> {
    /// Return if an [`Expression`] is a [`MemberExpression`].
    #[inline]
    pub fn is_member_expression(&self) -> bool {
        matches!(
            self,
            Self::ComputedMemberExpression(_)
                | Self::StaticMemberExpression(_)
                | Self::PrivateFieldExpression(_)
        )
    }

    /// Convert an [`Expression`] to a [`MemberExpression`].
    ///
    /// # Panics
    /// Panics if not convertible.
    #[inline]
    pub fn into_member_expression(self) -> MemberExpression<'a> {
        MemberExpression::try_from(self).unwrap()
    }

    /// Convert an [`&Expression`] to a [`&MemberExpression`].
    ///
    /// [`&Expression`]: Expression
    /// [`&MemberExpression`]: MemberExpression
    #[inline]
    pub fn as_member_expression(&self) -> Option<&MemberExpression<'a>> {
        if self.is_member_expression() {
            // SAFETY: Transmute is safe because discriminants + types are identical between
            // `parent` and `child` for the shared variants
            Some(unsafe { &*std::ptr::from_ref(self).cast::<MemberExpression>() })
        } else {
            None
        }
    }

    /// Convert an [`&mut Expression`] to a [`&mut MemberExpression`].
    ///
    /// [`&mut Expression`]: Expression
    /// [`&mut MemberExpression`]: MemberExpression
    #[inline]
    pub fn as_member_expression_mut(&mut self) -> Option<&mut MemberExpression<'a>> {
        if self.is_member_expression() {
            // SAFETY: Transmute is safe because discriminants + types are identical between
            // `parent` and `child` for the shared variants
            Some(unsafe { &mut *std::ptr::from_mut(self).cast::<MemberExpression>() })
        } else {
            None
        }
    }

    /// Convert an [`&Expression`] to a [`&MemberExpression`].
    ///
    /// # Panics
    /// Panics if not convertible.
    ///
    /// [`&Expression`]: Expression
    /// [`&MemberExpression`]: MemberExpression
    #[inline]
    pub fn to_member_expression(&self) -> &MemberExpression<'a> {
        self.as_member_expression().unwrap()
    }

    /// Convert an [`&mut Expression`] to a [`&mut MemberExpression`].
    ///
    /// # Panics
    /// Panics if not convertible.
    ///
    /// [`&mut Expression`]: Expression
    /// [`&mut MemberExpression`]: MemberExpression
    #[inline]
    pub fn to_member_expression_mut(&mut self) -> &mut MemberExpression<'a> {
        self.as_member_expression_mut().unwrap()
    }
}

impl<'a> MemberExpression<'a> {
    /// Convert a [`&MemberExpression`] to an [`&Expression`].
    ///
    /// [`&MemberExpression`]: MemberExpression
    /// [`&Expression`]: Expression
    #[inline]
    pub fn as_expression(&self) -> &Expression<'a> {
        // SAFETY: Transmute is safe because discriminants + types are identical between
        // `parent` and `child` for the shared variants
        unsafe { &*std::ptr::from_ref(self).cast::<Expression>() }
    }
}

impl<'a> TryFrom<Expression<'a>> for MemberExpression<'a> {
    type Error = ();

    /// Convert an [`Expression`] to a [`MemberExpression`].
    ///
    /// # Errors
    /// Returns `Err` if not convertible.
    #[inline]
    fn try_from(value: Expression<'a>) -> Result<Self, Self::Error> {
        // Compiler should implement this as a check of discriminant and then zero-cost transmute,
        // as discriminants for `parent` and `child` are aligned
        match value {
            Expression::ComputedMemberExpression(o) => {
                Ok(MemberExpression::ComputedMemberExpression(o))
            }
            Expression::StaticMemberExpression(o) => {
                Ok(MemberExpression::StaticMemberExpression(o))
            }
            Expression::PrivateFieldExpression(o) => {
                Ok(MemberExpression::PrivateFieldExpression(o))
            }
            _ => Err(()),
        }
    }
}

impl<'a> From<MemberExpression<'a>> for Expression<'a> {
    /// Convert a [`MemberExpression`] to an [`Expression`].
    #[inline]
    fn from(value: MemberExpression<'a>) -> Self {
        // Compiler should implement this as zero-cost transmute as discriminants
        // for `child` and `parent` are aligned
        match value {
            MemberExpression::ComputedMemberExpression(o) => {
                Expression::ComputedMemberExpression(o)
            }
            MemberExpression::StaticMemberExpression(o) => Expression::StaticMemberExpression(o),
            MemberExpression::PrivateFieldExpression(o) => Expression::PrivateFieldExpression(o),
        }
    }
}

impl<'a> ArrayExpressionElement<'a> {
    /// Return if an [`ArrayExpressionElement`] is an [`Expression`].
    #[inline]
    pub fn is_expression(&self) -> bool {
        matches!(
            self,
            Self::BooleanLiteral(_)
                | Self::NullLiteral(_)
                | Self::NumericLiteral(_)
                | Self::BigIntLiteral(_)
                | Self::RegExpLiteral(_)
                | Self::StringLiteral(_)
                | Self::TemplateLiteral(_)
                | Self::Identifier(_)
                | Self::MetaProperty(_)
                | Self::Super(_)
                | Self::ArrayExpression(_)
                | Self::ArrowFunctionExpression(_)
                | Self::AssignmentExpression(_)
                | Self::AwaitExpression(_)
                | Self::BinaryExpression(_)
                | Self::CallExpression(_)
                | Self::ChainExpression(_)
                | Self::ClassExpression(_)
                | Self::ConditionalExpression(_)
                | Self::FunctionExpression(_)
                | Self::ImportExpression(_)
                | Self::LogicalExpression(_)
                | Self::NewExpression(_)
                | Self::ObjectExpression(_)
                | Self::ParenthesizedExpression(_)
                | Self::SequenceExpression(_)
                | Self::TaggedTemplateExpression(_)
                | Self::ThisExpression(_)
                | Self::UnaryExpression(_)
                | Self::UpdateExpression(_)
                | Self::YieldExpression(_)
                | Self::PrivateInExpression(_)
                | Self::JSXElement(_)
                | Self::JSXFragment(_)
                | Self::TSAsExpression(_)
                | Self::TSSatisfiesExpression(_)
                | Self::TSTypeAssertion(_)
                | Self::TSNonNullExpression(_)
                | Self::TSInstantiationExpression(_)
                | Self::V8IntrinsicExpression(_)
                | Self::ComputedMemberExpression(_)
                | Self::StaticMemberExpression(_)
                | Self::PrivateFieldExpression(_)
        )
    }

    /// Convert an [`ArrayExpressionElement`] to an [`Expression`].
    ///
    /// # Panics
    /// Panics if not convertible.
    #[inline]
    pub fn into_expression(self) -> Expression<'a> {
        Expression::try_from(self).unwrap()
    }

    /// Convert an [`&ArrayExpressionElement`] to an [`&Expression`].
    ///
    /// [`&ArrayExpressionElement`]: ArrayExpressionElement
    /// [`&Expression`]: Expression
    #[inline]
    pub fn as_expression(&self) -> Option<&Expression<'a>> {
        if self.is_expression() {
            // SAFETY: Transmute is safe because discriminants + types are identical between
            // `parent` and `child` for the shared variants
            Some(unsafe { &*std::ptr::from_ref(self).cast::<Expression>() })
        } else {
            None
        }
    }

    /// Convert an [`&mut ArrayExpressionElement`] to an [`&mut Expression`].
    ///
    /// [`&mut ArrayExpressionElement`]: ArrayExpressionElement
    /// [`&mut Expression`]: Expression
    #[inline]
    pub fn as_expression_mut(&mut self) -> Option<&mut Expression<'a>> {
        if self.is_expression() {
            // SAFETY: Transmute is safe because discriminants + types are identical between
            // `parent` and `child` for the shared variants
            Some(unsafe { &mut *std::ptr::from_mut(self).cast::<Expression>() })
        } else {
            None
        }
    }

    /// Convert an [`&ArrayExpressionElement`] to an [`&Expression`].
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

    /// Convert an [`&mut ArrayExpressionElement`] to an [`&mut Expression`].
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

impl<'a> Expression<'a> {
    /// Convert an [`&Expression`] to an [`&ArrayExpressionElement`].
    ///
    /// [`&Expression`]: Expression
    /// [`&ArrayExpressionElement`]: ArrayExpressionElement
    #[inline]
    pub fn as_array_expression_element(&self) -> &ArrayExpressionElement<'a> {
        // SAFETY: Transmute is safe because discriminants + types are identical between
        // `parent` and `child` for the shared variants
        unsafe { &*std::ptr::from_ref(self).cast::<ArrayExpressionElement>() }
    }
}

impl<'a> TryFrom<ArrayExpressionElement<'a>> for Expression<'a> {
    type Error = ();

    /// Convert an [`ArrayExpressionElement`] to an [`Expression`].
    ///
    /// # Errors
    /// Returns `Err` if not convertible.
    #[inline]
    fn try_from(value: ArrayExpressionElement<'a>) -> Result<Self, Self::Error> {
        // Compiler should implement this as a check of discriminant and then zero-cost transmute,
        // as discriminants for `parent` and `child` are aligned
        match value {
            ArrayExpressionElement::BooleanLiteral(o) => Ok(Expression::BooleanLiteral(o)),
            ArrayExpressionElement::NullLiteral(o) => Ok(Expression::NullLiteral(o)),
            ArrayExpressionElement::NumericLiteral(o) => Ok(Expression::NumericLiteral(o)),
            ArrayExpressionElement::BigIntLiteral(o) => Ok(Expression::BigIntLiteral(o)),
            ArrayExpressionElement::RegExpLiteral(o) => Ok(Expression::RegExpLiteral(o)),
            ArrayExpressionElement::StringLiteral(o) => Ok(Expression::StringLiteral(o)),
            ArrayExpressionElement::TemplateLiteral(o) => Ok(Expression::TemplateLiteral(o)),
            ArrayExpressionElement::Identifier(o) => Ok(Expression::Identifier(o)),
            ArrayExpressionElement::MetaProperty(o) => Ok(Expression::MetaProperty(o)),
            ArrayExpressionElement::Super(o) => Ok(Expression::Super(o)),
            ArrayExpressionElement::ArrayExpression(o) => Ok(Expression::ArrayExpression(o)),
            ArrayExpressionElement::ArrowFunctionExpression(o) => {
                Ok(Expression::ArrowFunctionExpression(o))
            }
            ArrayExpressionElement::AssignmentExpression(o) => {
                Ok(Expression::AssignmentExpression(o))
            }
            ArrayExpressionElement::AwaitExpression(o) => Ok(Expression::AwaitExpression(o)),
            ArrayExpressionElement::BinaryExpression(o) => Ok(Expression::BinaryExpression(o)),
            ArrayExpressionElement::CallExpression(o) => Ok(Expression::CallExpression(o)),
            ArrayExpressionElement::ChainExpression(o) => Ok(Expression::ChainExpression(o)),
            ArrayExpressionElement::ClassExpression(o) => Ok(Expression::ClassExpression(o)),
            ArrayExpressionElement::ConditionalExpression(o) => {
                Ok(Expression::ConditionalExpression(o))
            }
            ArrayExpressionElement::FunctionExpression(o) => Ok(Expression::FunctionExpression(o)),
            ArrayExpressionElement::ImportExpression(o) => Ok(Expression::ImportExpression(o)),
            ArrayExpressionElement::LogicalExpression(o) => Ok(Expression::LogicalExpression(o)),
            ArrayExpressionElement::NewExpression(o) => Ok(Expression::NewExpression(o)),
            ArrayExpressionElement::ObjectExpression(o) => Ok(Expression::ObjectExpression(o)),
            ArrayExpressionElement::ParenthesizedExpression(o) => {
                Ok(Expression::ParenthesizedExpression(o))
            }
            ArrayExpressionElement::SequenceExpression(o) => Ok(Expression::SequenceExpression(o)),
            ArrayExpressionElement::TaggedTemplateExpression(o) => {
                Ok(Expression::TaggedTemplateExpression(o))
            }
            ArrayExpressionElement::ThisExpression(o) => Ok(Expression::ThisExpression(o)),
            ArrayExpressionElement::UnaryExpression(o) => Ok(Expression::UnaryExpression(o)),
            ArrayExpressionElement::UpdateExpression(o) => Ok(Expression::UpdateExpression(o)),
            ArrayExpressionElement::YieldExpression(o) => Ok(Expression::YieldExpression(o)),
            ArrayExpressionElement::PrivateInExpression(o) => {
                Ok(Expression::PrivateInExpression(o))
            }
            ArrayExpressionElement::JSXElement(o) => Ok(Expression::JSXElement(o)),
            ArrayExpressionElement::JSXFragment(o) => Ok(Expression::JSXFragment(o)),
            ArrayExpressionElement::TSAsExpression(o) => Ok(Expression::TSAsExpression(o)),
            ArrayExpressionElement::TSSatisfiesExpression(o) => {
                Ok(Expression::TSSatisfiesExpression(o))
            }
            ArrayExpressionElement::TSTypeAssertion(o) => Ok(Expression::TSTypeAssertion(o)),
            ArrayExpressionElement::TSNonNullExpression(o) => {
                Ok(Expression::TSNonNullExpression(o))
            }
            ArrayExpressionElement::TSInstantiationExpression(o) => {
                Ok(Expression::TSInstantiationExpression(o))
            }
            ArrayExpressionElement::V8IntrinsicExpression(o) => {
                Ok(Expression::V8IntrinsicExpression(o))
            }
            ArrayExpressionElement::ComputedMemberExpression(o) => {
                Ok(Expression::ComputedMemberExpression(o))
            }
            ArrayExpressionElement::StaticMemberExpression(o) => {
                Ok(Expression::StaticMemberExpression(o))
            }
            ArrayExpressionElement::PrivateFieldExpression(o) => {
                Ok(Expression::PrivateFieldExpression(o))
            }
            _ => Err(()),
        }
    }
}

impl<'a> From<Expression<'a>> for ArrayExpressionElement<'a> {
    /// Convert an [`Expression`] to an [`ArrayExpressionElement`].
    #[inline]
    fn from(value: Expression<'a>) -> Self {
        // Compiler should implement this as zero-cost transmute as discriminants
        // for `child` and `parent` are aligned
        match value {
            Expression::BooleanLiteral(o) => ArrayExpressionElement::BooleanLiteral(o),
            Expression::NullLiteral(o) => ArrayExpressionElement::NullLiteral(o),
            Expression::NumericLiteral(o) => ArrayExpressionElement::NumericLiteral(o),
            Expression::BigIntLiteral(o) => ArrayExpressionElement::BigIntLiteral(o),
            Expression::RegExpLiteral(o) => ArrayExpressionElement::RegExpLiteral(o),
            Expression::StringLiteral(o) => ArrayExpressionElement::StringLiteral(o),
            Expression::TemplateLiteral(o) => ArrayExpressionElement::TemplateLiteral(o),
            Expression::Identifier(o) => ArrayExpressionElement::Identifier(o),
            Expression::MetaProperty(o) => ArrayExpressionElement::MetaProperty(o),
            Expression::Super(o) => ArrayExpressionElement::Super(o),
            Expression::ArrayExpression(o) => ArrayExpressionElement::ArrayExpression(o),
            Expression::ArrowFunctionExpression(o) => {
                ArrayExpressionElement::ArrowFunctionExpression(o)
            }
            Expression::AssignmentExpression(o) => ArrayExpressionElement::AssignmentExpression(o),
            Expression::AwaitExpression(o) => ArrayExpressionElement::AwaitExpression(o),
            Expression::BinaryExpression(o) => ArrayExpressionElement::BinaryExpression(o),
            Expression::CallExpression(o) => ArrayExpressionElement::CallExpression(o),
            Expression::ChainExpression(o) => ArrayExpressionElement::ChainExpression(o),
            Expression::ClassExpression(o) => ArrayExpressionElement::ClassExpression(o),
            Expression::ConditionalExpression(o) => {
                ArrayExpressionElement::ConditionalExpression(o)
            }
            Expression::FunctionExpression(o) => ArrayExpressionElement::FunctionExpression(o),
            Expression::ImportExpression(o) => ArrayExpressionElement::ImportExpression(o),
            Expression::LogicalExpression(o) => ArrayExpressionElement::LogicalExpression(o),
            Expression::NewExpression(o) => ArrayExpressionElement::NewExpression(o),
            Expression::ObjectExpression(o) => ArrayExpressionElement::ObjectExpression(o),
            Expression::ParenthesizedExpression(o) => {
                ArrayExpressionElement::ParenthesizedExpression(o)
            }
            Expression::SequenceExpression(o) => ArrayExpressionElement::SequenceExpression(o),
            Expression::TaggedTemplateExpression(o) => {
                ArrayExpressionElement::TaggedTemplateExpression(o)
            }
            Expression::ThisExpression(o) => ArrayExpressionElement::ThisExpression(o),
            Expression::UnaryExpression(o) => ArrayExpressionElement::UnaryExpression(o),
            Expression::UpdateExpression(o) => ArrayExpressionElement::UpdateExpression(o),
            Expression::YieldExpression(o) => ArrayExpressionElement::YieldExpression(o),
            Expression::PrivateInExpression(o) => ArrayExpressionElement::PrivateInExpression(o),
            Expression::JSXElement(o) => ArrayExpressionElement::JSXElement(o),
            Expression::JSXFragment(o) => ArrayExpressionElement::JSXFragment(o),
            Expression::TSAsExpression(o) => ArrayExpressionElement::TSAsExpression(o),
            Expression::TSSatisfiesExpression(o) => {
                ArrayExpressionElement::TSSatisfiesExpression(o)
            }
            Expression::TSTypeAssertion(o) => ArrayExpressionElement::TSTypeAssertion(o),
            Expression::TSNonNullExpression(o) => ArrayExpressionElement::TSNonNullExpression(o),
            Expression::TSInstantiationExpression(o) => {
                ArrayExpressionElement::TSInstantiationExpression(o)
            }
            Expression::V8IntrinsicExpression(o) => {
                ArrayExpressionElement::V8IntrinsicExpression(o)
            }
            Expression::ComputedMemberExpression(o) => {
                ArrayExpressionElement::ComputedMemberExpression(o)
            }
            Expression::StaticMemberExpression(o) => {
                ArrayExpressionElement::StaticMemberExpression(o)
            }
            Expression::PrivateFieldExpression(o) => {
                ArrayExpressionElement::PrivateFieldExpression(o)
            }
        }
    }
}

impl<'a> ArrayExpressionElement<'a> {
    /// Return if an [`ArrayExpressionElement`] is a [`MemberExpression`].
    #[inline]
    pub fn is_member_expression(&self) -> bool {
        matches!(
            self,
            Self::ComputedMemberExpression(_)
                | Self::StaticMemberExpression(_)
                | Self::PrivateFieldExpression(_)
        )
    }

    /// Convert an [`ArrayExpressionElement`] to a [`MemberExpression`].
    ///
    /// # Panics
    /// Panics if not convertible.
    #[inline]
    pub fn into_member_expression(self) -> MemberExpression<'a> {
        MemberExpression::try_from(self).unwrap()
    }

    /// Convert an [`&ArrayExpressionElement`] to a [`&MemberExpression`].
    ///
    /// [`&ArrayExpressionElement`]: ArrayExpressionElement
    /// [`&MemberExpression`]: MemberExpression
    #[inline]
    pub fn as_member_expression(&self) -> Option<&MemberExpression<'a>> {
        if self.is_member_expression() {
            // SAFETY: Transmute is safe because discriminants + types are identical between
            // `parent` and `child` for the shared variants
            Some(unsafe { &*std::ptr::from_ref(self).cast::<MemberExpression>() })
        } else {
            None
        }
    }

    /// Convert an [`&mut ArrayExpressionElement`] to a [`&mut MemberExpression`].
    ///
    /// [`&mut ArrayExpressionElement`]: ArrayExpressionElement
    /// [`&mut MemberExpression`]: MemberExpression
    #[inline]
    pub fn as_member_expression_mut(&mut self) -> Option<&mut MemberExpression<'a>> {
        if self.is_member_expression() {
            // SAFETY: Transmute is safe because discriminants + types are identical between
            // `parent` and `child` for the shared variants
            Some(unsafe { &mut *std::ptr::from_mut(self).cast::<MemberExpression>() })
        } else {
            None
        }
    }

    /// Convert an [`&ArrayExpressionElement`] to a [`&MemberExpression`].
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

    /// Convert an [`&mut ArrayExpressionElement`] to a [`&mut MemberExpression`].
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

impl<'a> MemberExpression<'a> {
    /// Convert a [`&MemberExpression`] to an [`&ArrayExpressionElement`].
    ///
    /// [`&MemberExpression`]: MemberExpression
    /// [`&ArrayExpressionElement`]: ArrayExpressionElement
    #[inline]
    pub fn as_array_expression_element(&self) -> &ArrayExpressionElement<'a> {
        // SAFETY: Transmute is safe because discriminants + types are identical between
        // `parent` and `child` for the shared variants
        unsafe { &*std::ptr::from_ref(self).cast::<ArrayExpressionElement>() }
    }
}

impl<'a> TryFrom<ArrayExpressionElement<'a>> for MemberExpression<'a> {
    type Error = ();

    /// Convert an [`ArrayExpressionElement`] to a [`MemberExpression`].
    ///
    /// # Errors
    /// Returns `Err` if not convertible.
    #[inline]
    fn try_from(value: ArrayExpressionElement<'a>) -> Result<Self, Self::Error> {
        // Compiler should implement this as a check of discriminant and then zero-cost transmute,
        // as discriminants for `parent` and `child` are aligned
        match value {
            ArrayExpressionElement::ComputedMemberExpression(o) => {
                Ok(MemberExpression::ComputedMemberExpression(o))
            }
            ArrayExpressionElement::StaticMemberExpression(o) => {
                Ok(MemberExpression::StaticMemberExpression(o))
            }
            ArrayExpressionElement::PrivateFieldExpression(o) => {
                Ok(MemberExpression::PrivateFieldExpression(o))
            }
            _ => Err(()),
        }
    }
}

impl<'a> From<MemberExpression<'a>> for ArrayExpressionElement<'a> {
    /// Convert a [`MemberExpression`] to an [`ArrayExpressionElement`].
    #[inline]
    fn from(value: MemberExpression<'a>) -> Self {
        // Compiler should implement this as zero-cost transmute as discriminants
        // for `child` and `parent` are aligned
        match value {
            MemberExpression::ComputedMemberExpression(o) => {
                ArrayExpressionElement::ComputedMemberExpression(o)
            }
            MemberExpression::StaticMemberExpression(o) => {
                ArrayExpressionElement::StaticMemberExpression(o)
            }
            MemberExpression::PrivateFieldExpression(o) => {
                ArrayExpressionElement::PrivateFieldExpression(o)
            }
        }
    }
}

impl<'a> PropertyKey<'a> {
    /// Return if a [`PropertyKey`] is an [`Expression`].
    #[inline]
    pub fn is_expression(&self) -> bool {
        matches!(
            self,
            Self::BooleanLiteral(_)
                | Self::NullLiteral(_)
                | Self::NumericLiteral(_)
                | Self::BigIntLiteral(_)
                | Self::RegExpLiteral(_)
                | Self::StringLiteral(_)
                | Self::TemplateLiteral(_)
                | Self::Identifier(_)
                | Self::MetaProperty(_)
                | Self::Super(_)
                | Self::ArrayExpression(_)
                | Self::ArrowFunctionExpression(_)
                | Self::AssignmentExpression(_)
                | Self::AwaitExpression(_)
                | Self::BinaryExpression(_)
                | Self::CallExpression(_)
                | Self::ChainExpression(_)
                | Self::ClassExpression(_)
                | Self::ConditionalExpression(_)
                | Self::FunctionExpression(_)
                | Self::ImportExpression(_)
                | Self::LogicalExpression(_)
                | Self::NewExpression(_)
                | Self::ObjectExpression(_)
                | Self::ParenthesizedExpression(_)
                | Self::SequenceExpression(_)
                | Self::TaggedTemplateExpression(_)
                | Self::ThisExpression(_)
                | Self::UnaryExpression(_)
                | Self::UpdateExpression(_)
                | Self::YieldExpression(_)
                | Self::PrivateInExpression(_)
                | Self::JSXElement(_)
                | Self::JSXFragment(_)
                | Self::TSAsExpression(_)
                | Self::TSSatisfiesExpression(_)
                | Self::TSTypeAssertion(_)
                | Self::TSNonNullExpression(_)
                | Self::TSInstantiationExpression(_)
                | Self::V8IntrinsicExpression(_)
                | Self::ComputedMemberExpression(_)
                | Self::StaticMemberExpression(_)
                | Self::PrivateFieldExpression(_)
        )
    }

    /// Convert a [`PropertyKey`] to an [`Expression`].
    ///
    /// # Panics
    /// Panics if not convertible.
    #[inline]
    pub fn into_expression(self) -> Expression<'a> {
        Expression::try_from(self).unwrap()
    }

    /// Convert a [`&PropertyKey`] to an [`&Expression`].
    ///
    /// [`&PropertyKey`]: PropertyKey
    /// [`&Expression`]: Expression
    #[inline]
    pub fn as_expression(&self) -> Option<&Expression<'a>> {
        if self.is_expression() {
            // SAFETY: Transmute is safe because discriminants + types are identical between
            // `parent` and `child` for the shared variants
            Some(unsafe { &*std::ptr::from_ref(self).cast::<Expression>() })
        } else {
            None
        }
    }

    /// Convert a [`&mut PropertyKey`] to an [`&mut Expression`].
    ///
    /// [`&mut PropertyKey`]: PropertyKey
    /// [`&mut Expression`]: Expression
    #[inline]
    pub fn as_expression_mut(&mut self) -> Option<&mut Expression<'a>> {
        if self.is_expression() {
            // SAFETY: Transmute is safe because discriminants + types are identical between
            // `parent` and `child` for the shared variants
            Some(unsafe { &mut *std::ptr::from_mut(self).cast::<Expression>() })
        } else {
            None
        }
    }

    /// Convert a [`&PropertyKey`] to an [`&Expression`].
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

    /// Convert a [`&mut PropertyKey`] to an [`&mut Expression`].
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

impl<'a> Expression<'a> {
    /// Convert an [`&Expression`] to a [`&PropertyKey`].
    ///
    /// [`&Expression`]: Expression
    /// [`&PropertyKey`]: PropertyKey
    #[inline]
    pub fn as_property_key(&self) -> &PropertyKey<'a> {
        // SAFETY: Transmute is safe because discriminants + types are identical between
        // `parent` and `child` for the shared variants
        unsafe { &*std::ptr::from_ref(self).cast::<PropertyKey>() }
    }
}

impl<'a> TryFrom<PropertyKey<'a>> for Expression<'a> {
    type Error = ();

    /// Convert a [`PropertyKey`] to an [`Expression`].
    ///
    /// # Errors
    /// Returns `Err` if not convertible.
    #[inline]
    fn try_from(value: PropertyKey<'a>) -> Result<Self, Self::Error> {
        // Compiler should implement this as a check of discriminant and then zero-cost transmute,
        // as discriminants for `parent` and `child` are aligned
        match value {
            PropertyKey::BooleanLiteral(o) => Ok(Expression::BooleanLiteral(o)),
            PropertyKey::NullLiteral(o) => Ok(Expression::NullLiteral(o)),
            PropertyKey::NumericLiteral(o) => Ok(Expression::NumericLiteral(o)),
            PropertyKey::BigIntLiteral(o) => Ok(Expression::BigIntLiteral(o)),
            PropertyKey::RegExpLiteral(o) => Ok(Expression::RegExpLiteral(o)),
            PropertyKey::StringLiteral(o) => Ok(Expression::StringLiteral(o)),
            PropertyKey::TemplateLiteral(o) => Ok(Expression::TemplateLiteral(o)),
            PropertyKey::Identifier(o) => Ok(Expression::Identifier(o)),
            PropertyKey::MetaProperty(o) => Ok(Expression::MetaProperty(o)),
            PropertyKey::Super(o) => Ok(Expression::Super(o)),
            PropertyKey::ArrayExpression(o) => Ok(Expression::ArrayExpression(o)),
            PropertyKey::ArrowFunctionExpression(o) => Ok(Expression::ArrowFunctionExpression(o)),
            PropertyKey::AssignmentExpression(o) => Ok(Expression::AssignmentExpression(o)),
            PropertyKey::AwaitExpression(o) => Ok(Expression::AwaitExpression(o)),
            PropertyKey::BinaryExpression(o) => Ok(Expression::BinaryExpression(o)),
            PropertyKey::CallExpression(o) => Ok(Expression::CallExpression(o)),
            PropertyKey::ChainExpression(o) => Ok(Expression::ChainExpression(o)),
            PropertyKey::ClassExpression(o) => Ok(Expression::ClassExpression(o)),
            PropertyKey::ConditionalExpression(o) => Ok(Expression::ConditionalExpression(o)),
            PropertyKey::FunctionExpression(o) => Ok(Expression::FunctionExpression(o)),
            PropertyKey::ImportExpression(o) => Ok(Expression::ImportExpression(o)),
            PropertyKey::LogicalExpression(o) => Ok(Expression::LogicalExpression(o)),
            PropertyKey::NewExpression(o) => Ok(Expression::NewExpression(o)),
            PropertyKey::ObjectExpression(o) => Ok(Expression::ObjectExpression(o)),
            PropertyKey::ParenthesizedExpression(o) => Ok(Expression::ParenthesizedExpression(o)),
            PropertyKey::SequenceExpression(o) => Ok(Expression::SequenceExpression(o)),
            PropertyKey::TaggedTemplateExpression(o) => Ok(Expression::TaggedTemplateExpression(o)),
            PropertyKey::ThisExpression(o) => Ok(Expression::ThisExpression(o)),
            PropertyKey::UnaryExpression(o) => Ok(Expression::UnaryExpression(o)),
            PropertyKey::UpdateExpression(o) => Ok(Expression::UpdateExpression(o)),
            PropertyKey::YieldExpression(o) => Ok(Expression::YieldExpression(o)),
            PropertyKey::PrivateInExpression(o) => Ok(Expression::PrivateInExpression(o)),
            PropertyKey::JSXElement(o) => Ok(Expression::JSXElement(o)),
            PropertyKey::JSXFragment(o) => Ok(Expression::JSXFragment(o)),
            PropertyKey::TSAsExpression(o) => Ok(Expression::TSAsExpression(o)),
            PropertyKey::TSSatisfiesExpression(o) => Ok(Expression::TSSatisfiesExpression(o)),
            PropertyKey::TSTypeAssertion(o) => Ok(Expression::TSTypeAssertion(o)),
            PropertyKey::TSNonNullExpression(o) => Ok(Expression::TSNonNullExpression(o)),
            PropertyKey::TSInstantiationExpression(o) => {
                Ok(Expression::TSInstantiationExpression(o))
            }
            PropertyKey::V8IntrinsicExpression(o) => Ok(Expression::V8IntrinsicExpression(o)),
            PropertyKey::ComputedMemberExpression(o) => Ok(Expression::ComputedMemberExpression(o)),
            PropertyKey::StaticMemberExpression(o) => Ok(Expression::StaticMemberExpression(o)),
            PropertyKey::PrivateFieldExpression(o) => Ok(Expression::PrivateFieldExpression(o)),
            _ => Err(()),
        }
    }
}

impl<'a> From<Expression<'a>> for PropertyKey<'a> {
    /// Convert an [`Expression`] to a [`PropertyKey`].
    #[inline]
    fn from(value: Expression<'a>) -> Self {
        // Compiler should implement this as zero-cost transmute as discriminants
        // for `child` and `parent` are aligned
        match value {
            Expression::BooleanLiteral(o) => PropertyKey::BooleanLiteral(o),
            Expression::NullLiteral(o) => PropertyKey::NullLiteral(o),
            Expression::NumericLiteral(o) => PropertyKey::NumericLiteral(o),
            Expression::BigIntLiteral(o) => PropertyKey::BigIntLiteral(o),
            Expression::RegExpLiteral(o) => PropertyKey::RegExpLiteral(o),
            Expression::StringLiteral(o) => PropertyKey::StringLiteral(o),
            Expression::TemplateLiteral(o) => PropertyKey::TemplateLiteral(o),
            Expression::Identifier(o) => PropertyKey::Identifier(o),
            Expression::MetaProperty(o) => PropertyKey::MetaProperty(o),
            Expression::Super(o) => PropertyKey::Super(o),
            Expression::ArrayExpression(o) => PropertyKey::ArrayExpression(o),
            Expression::ArrowFunctionExpression(o) => PropertyKey::ArrowFunctionExpression(o),
            Expression::AssignmentExpression(o) => PropertyKey::AssignmentExpression(o),
            Expression::AwaitExpression(o) => PropertyKey::AwaitExpression(o),
            Expression::BinaryExpression(o) => PropertyKey::BinaryExpression(o),
            Expression::CallExpression(o) => PropertyKey::CallExpression(o),
            Expression::ChainExpression(o) => PropertyKey::ChainExpression(o),
            Expression::ClassExpression(o) => PropertyKey::ClassExpression(o),
            Expression::ConditionalExpression(o) => PropertyKey::ConditionalExpression(o),
            Expression::FunctionExpression(o) => PropertyKey::FunctionExpression(o),
            Expression::ImportExpression(o) => PropertyKey::ImportExpression(o),
            Expression::LogicalExpression(o) => PropertyKey::LogicalExpression(o),
            Expression::NewExpression(o) => PropertyKey::NewExpression(o),
            Expression::ObjectExpression(o) => PropertyKey::ObjectExpression(o),
            Expression::ParenthesizedExpression(o) => PropertyKey::ParenthesizedExpression(o),
            Expression::SequenceExpression(o) => PropertyKey::SequenceExpression(o),
            Expression::TaggedTemplateExpression(o) => PropertyKey::TaggedTemplateExpression(o),
            Expression::ThisExpression(o) => PropertyKey::ThisExpression(o),
            Expression::UnaryExpression(o) => PropertyKey::UnaryExpression(o),
            Expression::UpdateExpression(o) => PropertyKey::UpdateExpression(o),
            Expression::YieldExpression(o) => PropertyKey::YieldExpression(o),
            Expression::PrivateInExpression(o) => PropertyKey::PrivateInExpression(o),
            Expression::JSXElement(o) => PropertyKey::JSXElement(o),
            Expression::JSXFragment(o) => PropertyKey::JSXFragment(o),
            Expression::TSAsExpression(o) => PropertyKey::TSAsExpression(o),
            Expression::TSSatisfiesExpression(o) => PropertyKey::TSSatisfiesExpression(o),
            Expression::TSTypeAssertion(o) => PropertyKey::TSTypeAssertion(o),
            Expression::TSNonNullExpression(o) => PropertyKey::TSNonNullExpression(o),
            Expression::TSInstantiationExpression(o) => PropertyKey::TSInstantiationExpression(o),
            Expression::V8IntrinsicExpression(o) => PropertyKey::V8IntrinsicExpression(o),
            Expression::ComputedMemberExpression(o) => PropertyKey::ComputedMemberExpression(o),
            Expression::StaticMemberExpression(o) => PropertyKey::StaticMemberExpression(o),
            Expression::PrivateFieldExpression(o) => PropertyKey::PrivateFieldExpression(o),
        }
    }
}

impl<'a> PropertyKey<'a> {
    /// Return if a [`PropertyKey`] is a [`MemberExpression`].
    #[inline]
    pub fn is_member_expression(&self) -> bool {
        matches!(
            self,
            Self::ComputedMemberExpression(_)
                | Self::StaticMemberExpression(_)
                | Self::PrivateFieldExpression(_)
        )
    }

    /// Convert a [`PropertyKey`] to a [`MemberExpression`].
    ///
    /// # Panics
    /// Panics if not convertible.
    #[inline]
    pub fn into_member_expression(self) -> MemberExpression<'a> {
        MemberExpression::try_from(self).unwrap()
    }

    /// Convert a [`&PropertyKey`] to a [`&MemberExpression`].
    ///
    /// [`&PropertyKey`]: PropertyKey
    /// [`&MemberExpression`]: MemberExpression
    #[inline]
    pub fn as_member_expression(&self) -> Option<&MemberExpression<'a>> {
        if self.is_member_expression() {
            // SAFETY: Transmute is safe because discriminants + types are identical between
            // `parent` and `child` for the shared variants
            Some(unsafe { &*std::ptr::from_ref(self).cast::<MemberExpression>() })
        } else {
            None
        }
    }

    /// Convert a [`&mut PropertyKey`] to a [`&mut MemberExpression`].
    ///
    /// [`&mut PropertyKey`]: PropertyKey
    /// [`&mut MemberExpression`]: MemberExpression
    #[inline]
    pub fn as_member_expression_mut(&mut self) -> Option<&mut MemberExpression<'a>> {
        if self.is_member_expression() {
            // SAFETY: Transmute is safe because discriminants + types are identical between
            // `parent` and `child` for the shared variants
            Some(unsafe { &mut *std::ptr::from_mut(self).cast::<MemberExpression>() })
        } else {
            None
        }
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

impl<'a> MemberExpression<'a> {
    /// Convert a [`&MemberExpression`] to a [`&PropertyKey`].
    ///
    /// [`&MemberExpression`]: MemberExpression
    /// [`&PropertyKey`]: PropertyKey
    #[inline]
    pub fn as_property_key(&self) -> &PropertyKey<'a> {
        // SAFETY: Transmute is safe because discriminants + types are identical between
        // `parent` and `child` for the shared variants
        unsafe { &*std::ptr::from_ref(self).cast::<PropertyKey>() }
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
        // Compiler should implement this as a check of discriminant and then zero-cost transmute,
        // as discriminants for `parent` and `child` are aligned
        match value {
            PropertyKey::ComputedMemberExpression(o) => {
                Ok(MemberExpression::ComputedMemberExpression(o))
            }
            PropertyKey::StaticMemberExpression(o) => {
                Ok(MemberExpression::StaticMemberExpression(o))
            }
            PropertyKey::PrivateFieldExpression(o) => {
                Ok(MemberExpression::PrivateFieldExpression(o))
            }
            _ => Err(()),
        }
    }
}

impl<'a> From<MemberExpression<'a>> for PropertyKey<'a> {
    /// Convert a [`MemberExpression`] to a [`PropertyKey`].
    #[inline]
    fn from(value: MemberExpression<'a>) -> Self {
        // Compiler should implement this as zero-cost transmute as discriminants
        // for `child` and `parent` are aligned
        match value {
            MemberExpression::ComputedMemberExpression(o) => {
                PropertyKey::ComputedMemberExpression(o)
            }
            MemberExpression::StaticMemberExpression(o) => PropertyKey::StaticMemberExpression(o),
            MemberExpression::PrivateFieldExpression(o) => PropertyKey::PrivateFieldExpression(o),
        }
    }
}

impl<'a> Argument<'a> {
    /// Return if an [`Argument`] is an [`Expression`].
    #[inline]
    pub fn is_expression(&self) -> bool {
        matches!(
            self,
            Self::BooleanLiteral(_)
                | Self::NullLiteral(_)
                | Self::NumericLiteral(_)
                | Self::BigIntLiteral(_)
                | Self::RegExpLiteral(_)
                | Self::StringLiteral(_)
                | Self::TemplateLiteral(_)
                | Self::Identifier(_)
                | Self::MetaProperty(_)
                | Self::Super(_)
                | Self::ArrayExpression(_)
                | Self::ArrowFunctionExpression(_)
                | Self::AssignmentExpression(_)
                | Self::AwaitExpression(_)
                | Self::BinaryExpression(_)
                | Self::CallExpression(_)
                | Self::ChainExpression(_)
                | Self::ClassExpression(_)
                | Self::ConditionalExpression(_)
                | Self::FunctionExpression(_)
                | Self::ImportExpression(_)
                | Self::LogicalExpression(_)
                | Self::NewExpression(_)
                | Self::ObjectExpression(_)
                | Self::ParenthesizedExpression(_)
                | Self::SequenceExpression(_)
                | Self::TaggedTemplateExpression(_)
                | Self::ThisExpression(_)
                | Self::UnaryExpression(_)
                | Self::UpdateExpression(_)
                | Self::YieldExpression(_)
                | Self::PrivateInExpression(_)
                | Self::JSXElement(_)
                | Self::JSXFragment(_)
                | Self::TSAsExpression(_)
                | Self::TSSatisfiesExpression(_)
                | Self::TSTypeAssertion(_)
                | Self::TSNonNullExpression(_)
                | Self::TSInstantiationExpression(_)
                | Self::V8IntrinsicExpression(_)
                | Self::ComputedMemberExpression(_)
                | Self::StaticMemberExpression(_)
                | Self::PrivateFieldExpression(_)
        )
    }

    /// Convert an [`Argument`] to an [`Expression`].
    ///
    /// # Panics
    /// Panics if not convertible.
    #[inline]
    pub fn into_expression(self) -> Expression<'a> {
        Expression::try_from(self).unwrap()
    }

    /// Convert an [`&Argument`] to an [`&Expression`].
    ///
    /// [`&Argument`]: Argument
    /// [`&Expression`]: Expression
    #[inline]
    pub fn as_expression(&self) -> Option<&Expression<'a>> {
        if self.is_expression() {
            // SAFETY: Transmute is safe because discriminants + types are identical between
            // `parent` and `child` for the shared variants
            Some(unsafe { &*std::ptr::from_ref(self).cast::<Expression>() })
        } else {
            None
        }
    }

    /// Convert an [`&mut Argument`] to an [`&mut Expression`].
    ///
    /// [`&mut Argument`]: Argument
    /// [`&mut Expression`]: Expression
    #[inline]
    pub fn as_expression_mut(&mut self) -> Option<&mut Expression<'a>> {
        if self.is_expression() {
            // SAFETY: Transmute is safe because discriminants + types are identical between
            // `parent` and `child` for the shared variants
            Some(unsafe { &mut *std::ptr::from_mut(self).cast::<Expression>() })
        } else {
            None
        }
    }

    /// Convert an [`&Argument`] to an [`&Expression`].
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

    /// Convert an [`&mut Argument`] to an [`&mut Expression`].
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

impl<'a> Expression<'a> {
    /// Convert an [`&Expression`] to an [`&Argument`].
    ///
    /// [`&Expression`]: Expression
    /// [`&Argument`]: Argument
    #[inline]
    pub fn as_argument(&self) -> &Argument<'a> {
        // SAFETY: Transmute is safe because discriminants + types are identical between
        // `parent` and `child` for the shared variants
        unsafe { &*std::ptr::from_ref(self).cast::<Argument>() }
    }
}

impl<'a> TryFrom<Argument<'a>> for Expression<'a> {
    type Error = ();

    /// Convert an [`Argument`] to an [`Expression`].
    ///
    /// # Errors
    /// Returns `Err` if not convertible.
    #[inline]
    fn try_from(value: Argument<'a>) -> Result<Self, Self::Error> {
        // Compiler should implement this as a check of discriminant and then zero-cost transmute,
        // as discriminants for `parent` and `child` are aligned
        match value {
            Argument::BooleanLiteral(o) => Ok(Expression::BooleanLiteral(o)),
            Argument::NullLiteral(o) => Ok(Expression::NullLiteral(o)),
            Argument::NumericLiteral(o) => Ok(Expression::NumericLiteral(o)),
            Argument::BigIntLiteral(o) => Ok(Expression::BigIntLiteral(o)),
            Argument::RegExpLiteral(o) => Ok(Expression::RegExpLiteral(o)),
            Argument::StringLiteral(o) => Ok(Expression::StringLiteral(o)),
            Argument::TemplateLiteral(o) => Ok(Expression::TemplateLiteral(o)),
            Argument::Identifier(o) => Ok(Expression::Identifier(o)),
            Argument::MetaProperty(o) => Ok(Expression::MetaProperty(o)),
            Argument::Super(o) => Ok(Expression::Super(o)),
            Argument::ArrayExpression(o) => Ok(Expression::ArrayExpression(o)),
            Argument::ArrowFunctionExpression(o) => Ok(Expression::ArrowFunctionExpression(o)),
            Argument::AssignmentExpression(o) => Ok(Expression::AssignmentExpression(o)),
            Argument::AwaitExpression(o) => Ok(Expression::AwaitExpression(o)),
            Argument::BinaryExpression(o) => Ok(Expression::BinaryExpression(o)),
            Argument::CallExpression(o) => Ok(Expression::CallExpression(o)),
            Argument::ChainExpression(o) => Ok(Expression::ChainExpression(o)),
            Argument::ClassExpression(o) => Ok(Expression::ClassExpression(o)),
            Argument::ConditionalExpression(o) => Ok(Expression::ConditionalExpression(o)),
            Argument::FunctionExpression(o) => Ok(Expression::FunctionExpression(o)),
            Argument::ImportExpression(o) => Ok(Expression::ImportExpression(o)),
            Argument::LogicalExpression(o) => Ok(Expression::LogicalExpression(o)),
            Argument::NewExpression(o) => Ok(Expression::NewExpression(o)),
            Argument::ObjectExpression(o) => Ok(Expression::ObjectExpression(o)),
            Argument::ParenthesizedExpression(o) => Ok(Expression::ParenthesizedExpression(o)),
            Argument::SequenceExpression(o) => Ok(Expression::SequenceExpression(o)),
            Argument::TaggedTemplateExpression(o) => Ok(Expression::TaggedTemplateExpression(o)),
            Argument::ThisExpression(o) => Ok(Expression::ThisExpression(o)),
            Argument::UnaryExpression(o) => Ok(Expression::UnaryExpression(o)),
            Argument::UpdateExpression(o) => Ok(Expression::UpdateExpression(o)),
            Argument::YieldExpression(o) => Ok(Expression::YieldExpression(o)),
            Argument::PrivateInExpression(o) => Ok(Expression::PrivateInExpression(o)),
            Argument::JSXElement(o) => Ok(Expression::JSXElement(o)),
            Argument::JSXFragment(o) => Ok(Expression::JSXFragment(o)),
            Argument::TSAsExpression(o) => Ok(Expression::TSAsExpression(o)),
            Argument::TSSatisfiesExpression(o) => Ok(Expression::TSSatisfiesExpression(o)),
            Argument::TSTypeAssertion(o) => Ok(Expression::TSTypeAssertion(o)),
            Argument::TSNonNullExpression(o) => Ok(Expression::TSNonNullExpression(o)),
            Argument::TSInstantiationExpression(o) => Ok(Expression::TSInstantiationExpression(o)),
            Argument::V8IntrinsicExpression(o) => Ok(Expression::V8IntrinsicExpression(o)),
            Argument::ComputedMemberExpression(o) => Ok(Expression::ComputedMemberExpression(o)),
            Argument::StaticMemberExpression(o) => Ok(Expression::StaticMemberExpression(o)),
            Argument::PrivateFieldExpression(o) => Ok(Expression::PrivateFieldExpression(o)),
            _ => Err(()),
        }
    }
}

impl<'a> From<Expression<'a>> for Argument<'a> {
    /// Convert an [`Expression`] to an [`Argument`].
    #[inline]
    fn from(value: Expression<'a>) -> Self {
        // Compiler should implement this as zero-cost transmute as discriminants
        // for `child` and `parent` are aligned
        match value {
            Expression::BooleanLiteral(o) => Argument::BooleanLiteral(o),
            Expression::NullLiteral(o) => Argument::NullLiteral(o),
            Expression::NumericLiteral(o) => Argument::NumericLiteral(o),
            Expression::BigIntLiteral(o) => Argument::BigIntLiteral(o),
            Expression::RegExpLiteral(o) => Argument::RegExpLiteral(o),
            Expression::StringLiteral(o) => Argument::StringLiteral(o),
            Expression::TemplateLiteral(o) => Argument::TemplateLiteral(o),
            Expression::Identifier(o) => Argument::Identifier(o),
            Expression::MetaProperty(o) => Argument::MetaProperty(o),
            Expression::Super(o) => Argument::Super(o),
            Expression::ArrayExpression(o) => Argument::ArrayExpression(o),
            Expression::ArrowFunctionExpression(o) => Argument::ArrowFunctionExpression(o),
            Expression::AssignmentExpression(o) => Argument::AssignmentExpression(o),
            Expression::AwaitExpression(o) => Argument::AwaitExpression(o),
            Expression::BinaryExpression(o) => Argument::BinaryExpression(o),
            Expression::CallExpression(o) => Argument::CallExpression(o),
            Expression::ChainExpression(o) => Argument::ChainExpression(o),
            Expression::ClassExpression(o) => Argument::ClassExpression(o),
            Expression::ConditionalExpression(o) => Argument::ConditionalExpression(o),
            Expression::FunctionExpression(o) => Argument::FunctionExpression(o),
            Expression::ImportExpression(o) => Argument::ImportExpression(o),
            Expression::LogicalExpression(o) => Argument::LogicalExpression(o),
            Expression::NewExpression(o) => Argument::NewExpression(o),
            Expression::ObjectExpression(o) => Argument::ObjectExpression(o),
            Expression::ParenthesizedExpression(o) => Argument::ParenthesizedExpression(o),
            Expression::SequenceExpression(o) => Argument::SequenceExpression(o),
            Expression::TaggedTemplateExpression(o) => Argument::TaggedTemplateExpression(o),
            Expression::ThisExpression(o) => Argument::ThisExpression(o),
            Expression::UnaryExpression(o) => Argument::UnaryExpression(o),
            Expression::UpdateExpression(o) => Argument::UpdateExpression(o),
            Expression::YieldExpression(o) => Argument::YieldExpression(o),
            Expression::PrivateInExpression(o) => Argument::PrivateInExpression(o),
            Expression::JSXElement(o) => Argument::JSXElement(o),
            Expression::JSXFragment(o) => Argument::JSXFragment(o),
            Expression::TSAsExpression(o) => Argument::TSAsExpression(o),
            Expression::TSSatisfiesExpression(o) => Argument::TSSatisfiesExpression(o),
            Expression::TSTypeAssertion(o) => Argument::TSTypeAssertion(o),
            Expression::TSNonNullExpression(o) => Argument::TSNonNullExpression(o),
            Expression::TSInstantiationExpression(o) => Argument::TSInstantiationExpression(o),
            Expression::V8IntrinsicExpression(o) => Argument::V8IntrinsicExpression(o),
            Expression::ComputedMemberExpression(o) => Argument::ComputedMemberExpression(o),
            Expression::StaticMemberExpression(o) => Argument::StaticMemberExpression(o),
            Expression::PrivateFieldExpression(o) => Argument::PrivateFieldExpression(o),
        }
    }
}

impl<'a> Argument<'a> {
    /// Return if an [`Argument`] is a [`MemberExpression`].
    #[inline]
    pub fn is_member_expression(&self) -> bool {
        matches!(
            self,
            Self::ComputedMemberExpression(_)
                | Self::StaticMemberExpression(_)
                | Self::PrivateFieldExpression(_)
        )
    }

    /// Convert an [`Argument`] to a [`MemberExpression`].
    ///
    /// # Panics
    /// Panics if not convertible.
    #[inline]
    pub fn into_member_expression(self) -> MemberExpression<'a> {
        MemberExpression::try_from(self).unwrap()
    }

    /// Convert an [`&Argument`] to a [`&MemberExpression`].
    ///
    /// [`&Argument`]: Argument
    /// [`&MemberExpression`]: MemberExpression
    #[inline]
    pub fn as_member_expression(&self) -> Option<&MemberExpression<'a>> {
        if self.is_member_expression() {
            // SAFETY: Transmute is safe because discriminants + types are identical between
            // `parent` and `child` for the shared variants
            Some(unsafe { &*std::ptr::from_ref(self).cast::<MemberExpression>() })
        } else {
            None
        }
    }

    /// Convert an [`&mut Argument`] to a [`&mut MemberExpression`].
    ///
    /// [`&mut Argument`]: Argument
    /// [`&mut MemberExpression`]: MemberExpression
    #[inline]
    pub fn as_member_expression_mut(&mut self) -> Option<&mut MemberExpression<'a>> {
        if self.is_member_expression() {
            // SAFETY: Transmute is safe because discriminants + types are identical between
            // `parent` and `child` for the shared variants
            Some(unsafe { &mut *std::ptr::from_mut(self).cast::<MemberExpression>() })
        } else {
            None
        }
    }

    /// Convert an [`&Argument`] to a [`&MemberExpression`].
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

    /// Convert an [`&mut Argument`] to a [`&mut MemberExpression`].
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

impl<'a> MemberExpression<'a> {
    /// Convert a [`&MemberExpression`] to an [`&Argument`].
    ///
    /// [`&MemberExpression`]: MemberExpression
    /// [`&Argument`]: Argument
    #[inline]
    pub fn as_argument(&self) -> &Argument<'a> {
        // SAFETY: Transmute is safe because discriminants + types are identical between
        // `parent` and `child` for the shared variants
        unsafe { &*std::ptr::from_ref(self).cast::<Argument>() }
    }
}

impl<'a> TryFrom<Argument<'a>> for MemberExpression<'a> {
    type Error = ();

    /// Convert an [`Argument`] to a [`MemberExpression`].
    ///
    /// # Errors
    /// Returns `Err` if not convertible.
    #[inline]
    fn try_from(value: Argument<'a>) -> Result<Self, Self::Error> {
        // Compiler should implement this as a check of discriminant and then zero-cost transmute,
        // as discriminants for `parent` and `child` are aligned
        match value {
            Argument::ComputedMemberExpression(o) => {
                Ok(MemberExpression::ComputedMemberExpression(o))
            }
            Argument::StaticMemberExpression(o) => Ok(MemberExpression::StaticMemberExpression(o)),
            Argument::PrivateFieldExpression(o) => Ok(MemberExpression::PrivateFieldExpression(o)),
            _ => Err(()),
        }
    }
}

impl<'a> From<MemberExpression<'a>> for Argument<'a> {
    /// Convert a [`MemberExpression`] to an [`Argument`].
    #[inline]
    fn from(value: MemberExpression<'a>) -> Self {
        // Compiler should implement this as zero-cost transmute as discriminants
        // for `child` and `parent` are aligned
        match value {
            MemberExpression::ComputedMemberExpression(o) => Argument::ComputedMemberExpression(o),
            MemberExpression::StaticMemberExpression(o) => Argument::StaticMemberExpression(o),
            MemberExpression::PrivateFieldExpression(o) => Argument::PrivateFieldExpression(o),
        }
    }
}

impl<'a> AssignmentTarget<'a> {
    /// Return if an [`AssignmentTarget`] is a [`SimpleAssignmentTarget`].
    #[inline]
    pub fn is_simple_assignment_target(&self) -> bool {
        matches!(
            self,
            Self::AssignmentTargetIdentifier(_)
                | Self::TSAsExpression(_)
                | Self::TSSatisfiesExpression(_)
                | Self::TSNonNullExpression(_)
                | Self::TSTypeAssertion(_)
                | Self::ComputedMemberExpression(_)
                | Self::StaticMemberExpression(_)
                | Self::PrivateFieldExpression(_)
        )
    }

    /// Convert an [`AssignmentTarget`] to a [`SimpleAssignmentTarget`].
    ///
    /// # Panics
    /// Panics if not convertible.
    #[inline]
    pub fn into_simple_assignment_target(self) -> SimpleAssignmentTarget<'a> {
        SimpleAssignmentTarget::try_from(self).unwrap()
    }

    /// Convert an [`&AssignmentTarget`] to a [`&SimpleAssignmentTarget`].
    ///
    /// [`&AssignmentTarget`]: AssignmentTarget
    /// [`&SimpleAssignmentTarget`]: SimpleAssignmentTarget
    #[inline]
    pub fn as_simple_assignment_target(&self) -> Option<&SimpleAssignmentTarget<'a>> {
        if self.is_simple_assignment_target() {
            // SAFETY: Transmute is safe because discriminants + types are identical between
            // `parent` and `child` for the shared variants
            Some(unsafe { &*std::ptr::from_ref(self).cast::<SimpleAssignmentTarget>() })
        } else {
            None
        }
    }

    /// Convert an [`&mut AssignmentTarget`] to a [`&mut SimpleAssignmentTarget`].
    ///
    /// [`&mut AssignmentTarget`]: AssignmentTarget
    /// [`&mut SimpleAssignmentTarget`]: SimpleAssignmentTarget
    #[inline]
    pub fn as_simple_assignment_target_mut(&mut self) -> Option<&mut SimpleAssignmentTarget<'a>> {
        if self.is_simple_assignment_target() {
            // SAFETY: Transmute is safe because discriminants + types are identical between
            // `parent` and `child` for the shared variants
            Some(unsafe { &mut *std::ptr::from_mut(self).cast::<SimpleAssignmentTarget>() })
        } else {
            None
        }
    }

    /// Convert an [`&AssignmentTarget`] to a [`&SimpleAssignmentTarget`].
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

    /// Convert an [`&mut AssignmentTarget`] to a [`&mut SimpleAssignmentTarget`].
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

impl<'a> SimpleAssignmentTarget<'a> {
    /// Convert a [`&SimpleAssignmentTarget`] to an [`&AssignmentTarget`].
    ///
    /// [`&SimpleAssignmentTarget`]: SimpleAssignmentTarget
    /// [`&AssignmentTarget`]: AssignmentTarget
    #[inline]
    pub fn as_assignment_target(&self) -> &AssignmentTarget<'a> {
        // SAFETY: Transmute is safe because discriminants + types are identical between
        // `parent` and `child` for the shared variants
        unsafe { &*std::ptr::from_ref(self).cast::<AssignmentTarget>() }
    }
}

impl<'a> TryFrom<AssignmentTarget<'a>> for SimpleAssignmentTarget<'a> {
    type Error = ();

    /// Convert an [`AssignmentTarget`] to a [`SimpleAssignmentTarget`].
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
            AssignmentTarget::ComputedMemberExpression(o) => {
                Ok(SimpleAssignmentTarget::ComputedMemberExpression(o))
            }
            AssignmentTarget::StaticMemberExpression(o) => {
                Ok(SimpleAssignmentTarget::StaticMemberExpression(o))
            }
            AssignmentTarget::PrivateFieldExpression(o) => {
                Ok(SimpleAssignmentTarget::PrivateFieldExpression(o))
            }
            _ => Err(()),
        }
    }
}

impl<'a> From<SimpleAssignmentTarget<'a>> for AssignmentTarget<'a> {
    /// Convert a [`SimpleAssignmentTarget`] to an [`AssignmentTarget`].
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
            SimpleAssignmentTarget::ComputedMemberExpression(o) => {
                AssignmentTarget::ComputedMemberExpression(o)
            }
            SimpleAssignmentTarget::StaticMemberExpression(o) => {
                AssignmentTarget::StaticMemberExpression(o)
            }
            SimpleAssignmentTarget::PrivateFieldExpression(o) => {
                AssignmentTarget::PrivateFieldExpression(o)
            }
        }
    }
}

impl<'a> AssignmentTarget<'a> {
    /// Return if an [`AssignmentTarget`] is a [`MemberExpression`].
    #[inline]
    pub fn is_member_expression(&self) -> bool {
        matches!(
            self,
            Self::ComputedMemberExpression(_)
                | Self::StaticMemberExpression(_)
                | Self::PrivateFieldExpression(_)
        )
    }

    /// Convert an [`AssignmentTarget`] to a [`MemberExpression`].
    ///
    /// # Panics
    /// Panics if not convertible.
    #[inline]
    pub fn into_member_expression(self) -> MemberExpression<'a> {
        MemberExpression::try_from(self).unwrap()
    }

    /// Convert an [`&AssignmentTarget`] to a [`&MemberExpression`].
    ///
    /// [`&AssignmentTarget`]: AssignmentTarget
    /// [`&MemberExpression`]: MemberExpression
    #[inline]
    pub fn as_member_expression(&self) -> Option<&MemberExpression<'a>> {
        if self.is_member_expression() {
            // SAFETY: Transmute is safe because discriminants + types are identical between
            // `parent` and `child` for the shared variants
            Some(unsafe { &*std::ptr::from_ref(self).cast::<MemberExpression>() })
        } else {
            None
        }
    }

    /// Convert an [`&mut AssignmentTarget`] to a [`&mut MemberExpression`].
    ///
    /// [`&mut AssignmentTarget`]: AssignmentTarget
    /// [`&mut MemberExpression`]: MemberExpression
    #[inline]
    pub fn as_member_expression_mut(&mut self) -> Option<&mut MemberExpression<'a>> {
        if self.is_member_expression() {
            // SAFETY: Transmute is safe because discriminants + types are identical between
            // `parent` and `child` for the shared variants
            Some(unsafe { &mut *std::ptr::from_mut(self).cast::<MemberExpression>() })
        } else {
            None
        }
    }

    /// Convert an [`&AssignmentTarget`] to a [`&MemberExpression`].
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

    /// Convert an [`&mut AssignmentTarget`] to a [`&mut MemberExpression`].
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

impl<'a> MemberExpression<'a> {
    /// Convert a [`&MemberExpression`] to an [`&AssignmentTarget`].
    ///
    /// [`&MemberExpression`]: MemberExpression
    /// [`&AssignmentTarget`]: AssignmentTarget
    #[inline]
    pub fn as_assignment_target(&self) -> &AssignmentTarget<'a> {
        // SAFETY: Transmute is safe because discriminants + types are identical between
        // `parent` and `child` for the shared variants
        unsafe { &*std::ptr::from_ref(self).cast::<AssignmentTarget>() }
    }
}

impl<'a> TryFrom<AssignmentTarget<'a>> for MemberExpression<'a> {
    type Error = ();

    /// Convert an [`AssignmentTarget`] to a [`MemberExpression`].
    ///
    /// # Errors
    /// Returns `Err` if not convertible.
    #[inline]
    fn try_from(value: AssignmentTarget<'a>) -> Result<Self, Self::Error> {
        // Compiler should implement this as a check of discriminant and then zero-cost transmute,
        // as discriminants for `parent` and `child` are aligned
        match value {
            AssignmentTarget::ComputedMemberExpression(o) => {
                Ok(MemberExpression::ComputedMemberExpression(o))
            }
            AssignmentTarget::StaticMemberExpression(o) => {
                Ok(MemberExpression::StaticMemberExpression(o))
            }
            AssignmentTarget::PrivateFieldExpression(o) => {
                Ok(MemberExpression::PrivateFieldExpression(o))
            }
            _ => Err(()),
        }
    }
}

impl<'a> From<MemberExpression<'a>> for AssignmentTarget<'a> {
    /// Convert a [`MemberExpression`] to an [`AssignmentTarget`].
    #[inline]
    fn from(value: MemberExpression<'a>) -> Self {
        // Compiler should implement this as zero-cost transmute as discriminants
        // for `child` and `parent` are aligned
        match value {
            MemberExpression::ComputedMemberExpression(o) => {
                AssignmentTarget::ComputedMemberExpression(o)
            }
            MemberExpression::StaticMemberExpression(o) => {
                AssignmentTarget::StaticMemberExpression(o)
            }
            MemberExpression::PrivateFieldExpression(o) => {
                AssignmentTarget::PrivateFieldExpression(o)
            }
        }
    }
}

impl<'a> AssignmentTarget<'a> {
    /// Return if an [`AssignmentTarget`] is an [`AssignmentTargetPattern`].
    #[inline]
    pub fn is_assignment_target_pattern(&self) -> bool {
        matches!(self, Self::ArrayAssignmentTarget(_) | Self::ObjectAssignmentTarget(_))
    }

    /// Convert an [`AssignmentTarget`] to an [`AssignmentTargetPattern`].
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
            Some(unsafe { &*std::ptr::from_ref(self).cast::<AssignmentTargetPattern>() })
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
            Some(unsafe { &mut *std::ptr::from_mut(self).cast::<AssignmentTargetPattern>() })
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

impl<'a> AssignmentTargetPattern<'a> {
    /// Convert an [`&AssignmentTargetPattern`] to an [`&AssignmentTarget`].
    ///
    /// [`&AssignmentTargetPattern`]: AssignmentTargetPattern
    /// [`&AssignmentTarget`]: AssignmentTarget
    #[inline]
    pub fn as_assignment_target(&self) -> &AssignmentTarget<'a> {
        // SAFETY: Transmute is safe because discriminants + types are identical between
        // `parent` and `child` for the shared variants
        unsafe { &*std::ptr::from_ref(self).cast::<AssignmentTarget>() }
    }
}

impl<'a> TryFrom<AssignmentTarget<'a>> for AssignmentTargetPattern<'a> {
    type Error = ();

    /// Convert an [`AssignmentTarget`] to an [`AssignmentTargetPattern`].
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
    /// Convert an [`AssignmentTargetPattern`] to an [`AssignmentTarget`].
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

impl<'a> SimpleAssignmentTarget<'a> {
    /// Return if a [`SimpleAssignmentTarget`] is a [`MemberExpression`].
    #[inline]
    pub fn is_member_expression(&self) -> bool {
        matches!(
            self,
            Self::ComputedMemberExpression(_)
                | Self::StaticMemberExpression(_)
                | Self::PrivateFieldExpression(_)
        )
    }

    /// Convert a [`SimpleAssignmentTarget`] to a [`MemberExpression`].
    ///
    /// # Panics
    /// Panics if not convertible.
    #[inline]
    pub fn into_member_expression(self) -> MemberExpression<'a> {
        MemberExpression::try_from(self).unwrap()
    }

    /// Convert a [`&SimpleAssignmentTarget`] to a [`&MemberExpression`].
    ///
    /// [`&SimpleAssignmentTarget`]: SimpleAssignmentTarget
    /// [`&MemberExpression`]: MemberExpression
    #[inline]
    pub fn as_member_expression(&self) -> Option<&MemberExpression<'a>> {
        if self.is_member_expression() {
            // SAFETY: Transmute is safe because discriminants + types are identical between
            // `parent` and `child` for the shared variants
            Some(unsafe { &*std::ptr::from_ref(self).cast::<MemberExpression>() })
        } else {
            None
        }
    }

    /// Convert a [`&mut SimpleAssignmentTarget`] to a [`&mut MemberExpression`].
    ///
    /// [`&mut SimpleAssignmentTarget`]: SimpleAssignmentTarget
    /// [`&mut MemberExpression`]: MemberExpression
    #[inline]
    pub fn as_member_expression_mut(&mut self) -> Option<&mut MemberExpression<'a>> {
        if self.is_member_expression() {
            // SAFETY: Transmute is safe because discriminants + types are identical between
            // `parent` and `child` for the shared variants
            Some(unsafe { &mut *std::ptr::from_mut(self).cast::<MemberExpression>() })
        } else {
            None
        }
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

impl<'a> MemberExpression<'a> {
    /// Convert a [`&MemberExpression`] to a [`&SimpleAssignmentTarget`].
    ///
    /// [`&MemberExpression`]: MemberExpression
    /// [`&SimpleAssignmentTarget`]: SimpleAssignmentTarget
    #[inline]
    pub fn as_simple_assignment_target(&self) -> &SimpleAssignmentTarget<'a> {
        // SAFETY: Transmute is safe because discriminants + types are identical between
        // `parent` and `child` for the shared variants
        unsafe { &*std::ptr::from_ref(self).cast::<SimpleAssignmentTarget>() }
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
        // Compiler should implement this as a check of discriminant and then zero-cost transmute,
        // as discriminants for `parent` and `child` are aligned
        match value {
            SimpleAssignmentTarget::ComputedMemberExpression(o) => {
                Ok(MemberExpression::ComputedMemberExpression(o))
            }
            SimpleAssignmentTarget::StaticMemberExpression(o) => {
                Ok(MemberExpression::StaticMemberExpression(o))
            }
            SimpleAssignmentTarget::PrivateFieldExpression(o) => {
                Ok(MemberExpression::PrivateFieldExpression(o))
            }
            _ => Err(()),
        }
    }
}

impl<'a> From<MemberExpression<'a>> for SimpleAssignmentTarget<'a> {
    /// Convert a [`MemberExpression`] to a [`SimpleAssignmentTarget`].
    #[inline]
    fn from(value: MemberExpression<'a>) -> Self {
        // Compiler should implement this as zero-cost transmute as discriminants
        // for `child` and `parent` are aligned
        match value {
            MemberExpression::ComputedMemberExpression(o) => {
                SimpleAssignmentTarget::ComputedMemberExpression(o)
            }
            MemberExpression::StaticMemberExpression(o) => {
                SimpleAssignmentTarget::StaticMemberExpression(o)
            }
            MemberExpression::PrivateFieldExpression(o) => {
                SimpleAssignmentTarget::PrivateFieldExpression(o)
            }
        }
    }
}

impl<'a> AssignmentTargetMaybeDefault<'a> {
    /// Return if an [`AssignmentTargetMaybeDefault`] is an [`AssignmentTarget`].
    #[inline]
    pub fn is_assignment_target(&self) -> bool {
        matches!(
            self,
            Self::AssignmentTargetIdentifier(_)
                | Self::TSAsExpression(_)
                | Self::TSSatisfiesExpression(_)
                | Self::TSNonNullExpression(_)
                | Self::TSTypeAssertion(_)
                | Self::ComputedMemberExpression(_)
                | Self::StaticMemberExpression(_)
                | Self::PrivateFieldExpression(_)
                | Self::ArrayAssignmentTarget(_)
                | Self::ObjectAssignmentTarget(_)
        )
    }

    /// Convert an [`AssignmentTargetMaybeDefault`] to an [`AssignmentTarget`].
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
            Some(unsafe { &*std::ptr::from_ref(self).cast::<AssignmentTarget>() })
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
            Some(unsafe { &mut *std::ptr::from_mut(self).cast::<AssignmentTarget>() })
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

impl<'a> AssignmentTarget<'a> {
    /// Convert an [`&AssignmentTarget`] to an [`&AssignmentTargetMaybeDefault`].
    ///
    /// [`&AssignmentTarget`]: AssignmentTarget
    /// [`&AssignmentTargetMaybeDefault`]: AssignmentTargetMaybeDefault
    #[inline]
    pub fn as_assignment_target_maybe_default(&self) -> &AssignmentTargetMaybeDefault<'a> {
        // SAFETY: Transmute is safe because discriminants + types are identical between
        // `parent` and `child` for the shared variants
        unsafe { &*std::ptr::from_ref(self).cast::<AssignmentTargetMaybeDefault>() }
    }
}

impl<'a> TryFrom<AssignmentTargetMaybeDefault<'a>> for AssignmentTarget<'a> {
    type Error = ();

    /// Convert an [`AssignmentTargetMaybeDefault`] to an [`AssignmentTarget`].
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
            AssignmentTargetMaybeDefault::ComputedMemberExpression(o) => {
                Ok(AssignmentTarget::ComputedMemberExpression(o))
            }
            AssignmentTargetMaybeDefault::StaticMemberExpression(o) => {
                Ok(AssignmentTarget::StaticMemberExpression(o))
            }
            AssignmentTargetMaybeDefault::PrivateFieldExpression(o) => {
                Ok(AssignmentTarget::PrivateFieldExpression(o))
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
    /// Convert an [`AssignmentTarget`] to an [`AssignmentTargetMaybeDefault`].
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
            AssignmentTarget::ComputedMemberExpression(o) => {
                AssignmentTargetMaybeDefault::ComputedMemberExpression(o)
            }
            AssignmentTarget::StaticMemberExpression(o) => {
                AssignmentTargetMaybeDefault::StaticMemberExpression(o)
            }
            AssignmentTarget::PrivateFieldExpression(o) => {
                AssignmentTargetMaybeDefault::PrivateFieldExpression(o)
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

impl<'a> AssignmentTargetMaybeDefault<'a> {
    /// Return if an [`AssignmentTargetMaybeDefault`] is a [`SimpleAssignmentTarget`].
    #[inline]
    pub fn is_simple_assignment_target(&self) -> bool {
        matches!(
            self,
            Self::AssignmentTargetIdentifier(_)
                | Self::TSAsExpression(_)
                | Self::TSSatisfiesExpression(_)
                | Self::TSNonNullExpression(_)
                | Self::TSTypeAssertion(_)
                | Self::ComputedMemberExpression(_)
                | Self::StaticMemberExpression(_)
                | Self::PrivateFieldExpression(_)
        )
    }

    /// Convert an [`AssignmentTargetMaybeDefault`] to a [`SimpleAssignmentTarget`].
    ///
    /// # Panics
    /// Panics if not convertible.
    #[inline]
    pub fn into_simple_assignment_target(self) -> SimpleAssignmentTarget<'a> {
        SimpleAssignmentTarget::try_from(self).unwrap()
    }

    /// Convert an [`&AssignmentTargetMaybeDefault`] to a [`&SimpleAssignmentTarget`].
    ///
    /// [`&AssignmentTargetMaybeDefault`]: AssignmentTargetMaybeDefault
    /// [`&SimpleAssignmentTarget`]: SimpleAssignmentTarget
    #[inline]
    pub fn as_simple_assignment_target(&self) -> Option<&SimpleAssignmentTarget<'a>> {
        if self.is_simple_assignment_target() {
            // SAFETY: Transmute is safe because discriminants + types are identical between
            // `parent` and `child` for the shared variants
            Some(unsafe { &*std::ptr::from_ref(self).cast::<SimpleAssignmentTarget>() })
        } else {
            None
        }
    }

    /// Convert an [`&mut AssignmentTargetMaybeDefault`] to a [`&mut SimpleAssignmentTarget`].
    ///
    /// [`&mut AssignmentTargetMaybeDefault`]: AssignmentTargetMaybeDefault
    /// [`&mut SimpleAssignmentTarget`]: SimpleAssignmentTarget
    #[inline]
    pub fn as_simple_assignment_target_mut(&mut self) -> Option<&mut SimpleAssignmentTarget<'a>> {
        if self.is_simple_assignment_target() {
            // SAFETY: Transmute is safe because discriminants + types are identical between
            // `parent` and `child` for the shared variants
            Some(unsafe { &mut *std::ptr::from_mut(self).cast::<SimpleAssignmentTarget>() })
        } else {
            None
        }
    }

    /// Convert an [`&AssignmentTargetMaybeDefault`] to a [`&SimpleAssignmentTarget`].
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

    /// Convert an [`&mut AssignmentTargetMaybeDefault`] to a [`&mut SimpleAssignmentTarget`].
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

impl<'a> SimpleAssignmentTarget<'a> {
    /// Convert a [`&SimpleAssignmentTarget`] to an [`&AssignmentTargetMaybeDefault`].
    ///
    /// [`&SimpleAssignmentTarget`]: SimpleAssignmentTarget
    /// [`&AssignmentTargetMaybeDefault`]: AssignmentTargetMaybeDefault
    #[inline]
    pub fn as_assignment_target_maybe_default(&self) -> &AssignmentTargetMaybeDefault<'a> {
        // SAFETY: Transmute is safe because discriminants + types are identical between
        // `parent` and `child` for the shared variants
        unsafe { &*std::ptr::from_ref(self).cast::<AssignmentTargetMaybeDefault>() }
    }
}

impl<'a> TryFrom<AssignmentTargetMaybeDefault<'a>> for SimpleAssignmentTarget<'a> {
    type Error = ();

    /// Convert an [`AssignmentTargetMaybeDefault`] to a [`SimpleAssignmentTarget`].
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
            AssignmentTargetMaybeDefault::ComputedMemberExpression(o) => {
                Ok(SimpleAssignmentTarget::ComputedMemberExpression(o))
            }
            AssignmentTargetMaybeDefault::StaticMemberExpression(o) => {
                Ok(SimpleAssignmentTarget::StaticMemberExpression(o))
            }
            AssignmentTargetMaybeDefault::PrivateFieldExpression(o) => {
                Ok(SimpleAssignmentTarget::PrivateFieldExpression(o))
            }
            _ => Err(()),
        }
    }
}

impl<'a> From<SimpleAssignmentTarget<'a>> for AssignmentTargetMaybeDefault<'a> {
    /// Convert a [`SimpleAssignmentTarget`] to an [`AssignmentTargetMaybeDefault`].
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
            SimpleAssignmentTarget::ComputedMemberExpression(o) => {
                AssignmentTargetMaybeDefault::ComputedMemberExpression(o)
            }
            SimpleAssignmentTarget::StaticMemberExpression(o) => {
                AssignmentTargetMaybeDefault::StaticMemberExpression(o)
            }
            SimpleAssignmentTarget::PrivateFieldExpression(o) => {
                AssignmentTargetMaybeDefault::PrivateFieldExpression(o)
            }
        }
    }
}

impl<'a> AssignmentTargetMaybeDefault<'a> {
    /// Return if an [`AssignmentTargetMaybeDefault`] is a [`MemberExpression`].
    #[inline]
    pub fn is_member_expression(&self) -> bool {
        matches!(
            self,
            Self::ComputedMemberExpression(_)
                | Self::StaticMemberExpression(_)
                | Self::PrivateFieldExpression(_)
        )
    }

    /// Convert an [`AssignmentTargetMaybeDefault`] to a [`MemberExpression`].
    ///
    /// # Panics
    /// Panics if not convertible.
    #[inline]
    pub fn into_member_expression(self) -> MemberExpression<'a> {
        MemberExpression::try_from(self).unwrap()
    }

    /// Convert an [`&AssignmentTargetMaybeDefault`] to a [`&MemberExpression`].
    ///
    /// [`&AssignmentTargetMaybeDefault`]: AssignmentTargetMaybeDefault
    /// [`&MemberExpression`]: MemberExpression
    #[inline]
    pub fn as_member_expression(&self) -> Option<&MemberExpression<'a>> {
        if self.is_member_expression() {
            // SAFETY: Transmute is safe because discriminants + types are identical between
            // `parent` and `child` for the shared variants
            Some(unsafe { &*std::ptr::from_ref(self).cast::<MemberExpression>() })
        } else {
            None
        }
    }

    /// Convert an [`&mut AssignmentTargetMaybeDefault`] to a [`&mut MemberExpression`].
    ///
    /// [`&mut AssignmentTargetMaybeDefault`]: AssignmentTargetMaybeDefault
    /// [`&mut MemberExpression`]: MemberExpression
    #[inline]
    pub fn as_member_expression_mut(&mut self) -> Option<&mut MemberExpression<'a>> {
        if self.is_member_expression() {
            // SAFETY: Transmute is safe because discriminants + types are identical between
            // `parent` and `child` for the shared variants
            Some(unsafe { &mut *std::ptr::from_mut(self).cast::<MemberExpression>() })
        } else {
            None
        }
    }

    /// Convert an [`&AssignmentTargetMaybeDefault`] to a [`&MemberExpression`].
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

    /// Convert an [`&mut AssignmentTargetMaybeDefault`] to a [`&mut MemberExpression`].
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

impl<'a> MemberExpression<'a> {
    /// Convert a [`&MemberExpression`] to an [`&AssignmentTargetMaybeDefault`].
    ///
    /// [`&MemberExpression`]: MemberExpression
    /// [`&AssignmentTargetMaybeDefault`]: AssignmentTargetMaybeDefault
    #[inline]
    pub fn as_assignment_target_maybe_default(&self) -> &AssignmentTargetMaybeDefault<'a> {
        // SAFETY: Transmute is safe because discriminants + types are identical between
        // `parent` and `child` for the shared variants
        unsafe { &*std::ptr::from_ref(self).cast::<AssignmentTargetMaybeDefault>() }
    }
}

impl<'a> TryFrom<AssignmentTargetMaybeDefault<'a>> for MemberExpression<'a> {
    type Error = ();

    /// Convert an [`AssignmentTargetMaybeDefault`] to a [`MemberExpression`].
    ///
    /// # Errors
    /// Returns `Err` if not convertible.
    #[inline]
    fn try_from(value: AssignmentTargetMaybeDefault<'a>) -> Result<Self, Self::Error> {
        // Compiler should implement this as a check of discriminant and then zero-cost transmute,
        // as discriminants for `parent` and `child` are aligned
        match value {
            AssignmentTargetMaybeDefault::ComputedMemberExpression(o) => {
                Ok(MemberExpression::ComputedMemberExpression(o))
            }
            AssignmentTargetMaybeDefault::StaticMemberExpression(o) => {
                Ok(MemberExpression::StaticMemberExpression(o))
            }
            AssignmentTargetMaybeDefault::PrivateFieldExpression(o) => {
                Ok(MemberExpression::PrivateFieldExpression(o))
            }
            _ => Err(()),
        }
    }
}

impl<'a> From<MemberExpression<'a>> for AssignmentTargetMaybeDefault<'a> {
    /// Convert a [`MemberExpression`] to an [`AssignmentTargetMaybeDefault`].
    #[inline]
    fn from(value: MemberExpression<'a>) -> Self {
        // Compiler should implement this as zero-cost transmute as discriminants
        // for `child` and `parent` are aligned
        match value {
            MemberExpression::ComputedMemberExpression(o) => {
                AssignmentTargetMaybeDefault::ComputedMemberExpression(o)
            }
            MemberExpression::StaticMemberExpression(o) => {
                AssignmentTargetMaybeDefault::StaticMemberExpression(o)
            }
            MemberExpression::PrivateFieldExpression(o) => {
                AssignmentTargetMaybeDefault::PrivateFieldExpression(o)
            }
        }
    }
}

impl<'a> AssignmentTargetMaybeDefault<'a> {
    /// Return if an [`AssignmentTargetMaybeDefault`] is an [`AssignmentTargetPattern`].
    #[inline]
    pub fn is_assignment_target_pattern(&self) -> bool {
        matches!(self, Self::ArrayAssignmentTarget(_) | Self::ObjectAssignmentTarget(_))
    }

    /// Convert an [`AssignmentTargetMaybeDefault`] to an [`AssignmentTargetPattern`].
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
            Some(unsafe { &*std::ptr::from_ref(self).cast::<AssignmentTargetPattern>() })
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
            Some(unsafe { &mut *std::ptr::from_mut(self).cast::<AssignmentTargetPattern>() })
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

impl<'a> AssignmentTargetPattern<'a> {
    /// Convert an [`&AssignmentTargetPattern`] to an [`&AssignmentTargetMaybeDefault`].
    ///
    /// [`&AssignmentTargetPattern`]: AssignmentTargetPattern
    /// [`&AssignmentTargetMaybeDefault`]: AssignmentTargetMaybeDefault
    #[inline]
    pub fn as_assignment_target_maybe_default(&self) -> &AssignmentTargetMaybeDefault<'a> {
        // SAFETY: Transmute is safe because discriminants + types are identical between
        // `parent` and `child` for the shared variants
        unsafe { &*std::ptr::from_ref(self).cast::<AssignmentTargetMaybeDefault>() }
    }
}

impl<'a> TryFrom<AssignmentTargetMaybeDefault<'a>> for AssignmentTargetPattern<'a> {
    type Error = ();

    /// Convert an [`AssignmentTargetMaybeDefault`] to an [`AssignmentTargetPattern`].
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
    /// Convert an [`AssignmentTargetPattern`] to an [`AssignmentTargetMaybeDefault`].
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

impl<'a> ChainElement<'a> {
    /// Return if a [`ChainElement`] is a [`MemberExpression`].
    #[inline]
    pub fn is_member_expression(&self) -> bool {
        matches!(
            self,
            Self::ComputedMemberExpression(_)
                | Self::StaticMemberExpression(_)
                | Self::PrivateFieldExpression(_)
        )
    }

    /// Convert a [`ChainElement`] to a [`MemberExpression`].
    ///
    /// # Panics
    /// Panics if not convertible.
    #[inline]
    pub fn into_member_expression(self) -> MemberExpression<'a> {
        MemberExpression::try_from(self).unwrap()
    }

    /// Convert a [`&ChainElement`] to a [`&MemberExpression`].
    ///
    /// [`&ChainElement`]: ChainElement
    /// [`&MemberExpression`]: MemberExpression
    #[inline]
    pub fn as_member_expression(&self) -> Option<&MemberExpression<'a>> {
        if self.is_member_expression() {
            // SAFETY: Transmute is safe because discriminants + types are identical between
            // `parent` and `child` for the shared variants
            Some(unsafe { &*std::ptr::from_ref(self).cast::<MemberExpression>() })
        } else {
            None
        }
    }

    /// Convert a [`&mut ChainElement`] to a [`&mut MemberExpression`].
    ///
    /// [`&mut ChainElement`]: ChainElement
    /// [`&mut MemberExpression`]: MemberExpression
    #[inline]
    pub fn as_member_expression_mut(&mut self) -> Option<&mut MemberExpression<'a>> {
        if self.is_member_expression() {
            // SAFETY: Transmute is safe because discriminants + types are identical between
            // `parent` and `child` for the shared variants
            Some(unsafe { &mut *std::ptr::from_mut(self).cast::<MemberExpression>() })
        } else {
            None
        }
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

impl<'a> MemberExpression<'a> {
    /// Convert a [`&MemberExpression`] to a [`&ChainElement`].
    ///
    /// [`&MemberExpression`]: MemberExpression
    /// [`&ChainElement`]: ChainElement
    #[inline]
    pub fn as_chain_element(&self) -> &ChainElement<'a> {
        // SAFETY: Transmute is safe because discriminants + types are identical between
        // `parent` and `child` for the shared variants
        unsafe { &*std::ptr::from_ref(self).cast::<ChainElement>() }
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
        // Compiler should implement this as a check of discriminant and then zero-cost transmute,
        // as discriminants for `parent` and `child` are aligned
        match value {
            ChainElement::ComputedMemberExpression(o) => {
                Ok(MemberExpression::ComputedMemberExpression(o))
            }
            ChainElement::StaticMemberExpression(o) => {
                Ok(MemberExpression::StaticMemberExpression(o))
            }
            ChainElement::PrivateFieldExpression(o) => {
                Ok(MemberExpression::PrivateFieldExpression(o))
            }
            _ => Err(()),
        }
    }
}

impl<'a> From<MemberExpression<'a>> for ChainElement<'a> {
    /// Convert a [`MemberExpression`] to a [`ChainElement`].
    #[inline]
    fn from(value: MemberExpression<'a>) -> Self {
        // Compiler should implement this as zero-cost transmute as discriminants
        // for `child` and `parent` are aligned
        match value {
            MemberExpression::ComputedMemberExpression(o) => {
                ChainElement::ComputedMemberExpression(o)
            }
            MemberExpression::StaticMemberExpression(o) => ChainElement::StaticMemberExpression(o),
            MemberExpression::PrivateFieldExpression(o) => ChainElement::PrivateFieldExpression(o),
        }
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
            Some(unsafe { &*std::ptr::from_ref(self).cast::<Declaration>() })
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
            Some(unsafe { &mut *std::ptr::from_mut(self).cast::<Declaration>() })
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
        unsafe { &*std::ptr::from_ref(self).cast::<Statement>() }
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
            Some(unsafe { &*std::ptr::from_ref(self).cast::<ModuleDeclaration>() })
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
            Some(unsafe { &mut *std::ptr::from_mut(self).cast::<ModuleDeclaration>() })
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
        unsafe { &*std::ptr::from_ref(self).cast::<Statement>() }
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

impl<'a> ForStatementInit<'a> {
    /// Return if a [`ForStatementInit`] is an [`Expression`].
    #[inline]
    pub fn is_expression(&self) -> bool {
        matches!(
            self,
            Self::BooleanLiteral(_)
                | Self::NullLiteral(_)
                | Self::NumericLiteral(_)
                | Self::BigIntLiteral(_)
                | Self::RegExpLiteral(_)
                | Self::StringLiteral(_)
                | Self::TemplateLiteral(_)
                | Self::Identifier(_)
                | Self::MetaProperty(_)
                | Self::Super(_)
                | Self::ArrayExpression(_)
                | Self::ArrowFunctionExpression(_)
                | Self::AssignmentExpression(_)
                | Self::AwaitExpression(_)
                | Self::BinaryExpression(_)
                | Self::CallExpression(_)
                | Self::ChainExpression(_)
                | Self::ClassExpression(_)
                | Self::ConditionalExpression(_)
                | Self::FunctionExpression(_)
                | Self::ImportExpression(_)
                | Self::LogicalExpression(_)
                | Self::NewExpression(_)
                | Self::ObjectExpression(_)
                | Self::ParenthesizedExpression(_)
                | Self::SequenceExpression(_)
                | Self::TaggedTemplateExpression(_)
                | Self::ThisExpression(_)
                | Self::UnaryExpression(_)
                | Self::UpdateExpression(_)
                | Self::YieldExpression(_)
                | Self::PrivateInExpression(_)
                | Self::JSXElement(_)
                | Self::JSXFragment(_)
                | Self::TSAsExpression(_)
                | Self::TSSatisfiesExpression(_)
                | Self::TSTypeAssertion(_)
                | Self::TSNonNullExpression(_)
                | Self::TSInstantiationExpression(_)
                | Self::V8IntrinsicExpression(_)
                | Self::ComputedMemberExpression(_)
                | Self::StaticMemberExpression(_)
                | Self::PrivateFieldExpression(_)
        )
    }

    /// Convert a [`ForStatementInit`] to an [`Expression`].
    ///
    /// # Panics
    /// Panics if not convertible.
    #[inline]
    pub fn into_expression(self) -> Expression<'a> {
        Expression::try_from(self).unwrap()
    }

    /// Convert a [`&ForStatementInit`] to an [`&Expression`].
    ///
    /// [`&ForStatementInit`]: ForStatementInit
    /// [`&Expression`]: Expression
    #[inline]
    pub fn as_expression(&self) -> Option<&Expression<'a>> {
        if self.is_expression() {
            // SAFETY: Transmute is safe because discriminants + types are identical between
            // `parent` and `child` for the shared variants
            Some(unsafe { &*std::ptr::from_ref(self).cast::<Expression>() })
        } else {
            None
        }
    }

    /// Convert a [`&mut ForStatementInit`] to an [`&mut Expression`].
    ///
    /// [`&mut ForStatementInit`]: ForStatementInit
    /// [`&mut Expression`]: Expression
    #[inline]
    pub fn as_expression_mut(&mut self) -> Option<&mut Expression<'a>> {
        if self.is_expression() {
            // SAFETY: Transmute is safe because discriminants + types are identical between
            // `parent` and `child` for the shared variants
            Some(unsafe { &mut *std::ptr::from_mut(self).cast::<Expression>() })
        } else {
            None
        }
    }

    /// Convert a [`&ForStatementInit`] to an [`&Expression`].
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

    /// Convert a [`&mut ForStatementInit`] to an [`&mut Expression`].
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

impl<'a> Expression<'a> {
    /// Convert an [`&Expression`] to a [`&ForStatementInit`].
    ///
    /// [`&Expression`]: Expression
    /// [`&ForStatementInit`]: ForStatementInit
    #[inline]
    pub fn as_for_statement_init(&self) -> &ForStatementInit<'a> {
        // SAFETY: Transmute is safe because discriminants + types are identical between
        // `parent` and `child` for the shared variants
        unsafe { &*std::ptr::from_ref(self).cast::<ForStatementInit>() }
    }
}

impl<'a> TryFrom<ForStatementInit<'a>> for Expression<'a> {
    type Error = ();

    /// Convert a [`ForStatementInit`] to an [`Expression`].
    ///
    /// # Errors
    /// Returns `Err` if not convertible.
    #[inline]
    fn try_from(value: ForStatementInit<'a>) -> Result<Self, Self::Error> {
        // Compiler should implement this as a check of discriminant and then zero-cost transmute,
        // as discriminants for `parent` and `child` are aligned
        match value {
            ForStatementInit::BooleanLiteral(o) => Ok(Expression::BooleanLiteral(o)),
            ForStatementInit::NullLiteral(o) => Ok(Expression::NullLiteral(o)),
            ForStatementInit::NumericLiteral(o) => Ok(Expression::NumericLiteral(o)),
            ForStatementInit::BigIntLiteral(o) => Ok(Expression::BigIntLiteral(o)),
            ForStatementInit::RegExpLiteral(o) => Ok(Expression::RegExpLiteral(o)),
            ForStatementInit::StringLiteral(o) => Ok(Expression::StringLiteral(o)),
            ForStatementInit::TemplateLiteral(o) => Ok(Expression::TemplateLiteral(o)),
            ForStatementInit::Identifier(o) => Ok(Expression::Identifier(o)),
            ForStatementInit::MetaProperty(o) => Ok(Expression::MetaProperty(o)),
            ForStatementInit::Super(o) => Ok(Expression::Super(o)),
            ForStatementInit::ArrayExpression(o) => Ok(Expression::ArrayExpression(o)),
            ForStatementInit::ArrowFunctionExpression(o) => {
                Ok(Expression::ArrowFunctionExpression(o))
            }
            ForStatementInit::AssignmentExpression(o) => Ok(Expression::AssignmentExpression(o)),
            ForStatementInit::AwaitExpression(o) => Ok(Expression::AwaitExpression(o)),
            ForStatementInit::BinaryExpression(o) => Ok(Expression::BinaryExpression(o)),
            ForStatementInit::CallExpression(o) => Ok(Expression::CallExpression(o)),
            ForStatementInit::ChainExpression(o) => Ok(Expression::ChainExpression(o)),
            ForStatementInit::ClassExpression(o) => Ok(Expression::ClassExpression(o)),
            ForStatementInit::ConditionalExpression(o) => Ok(Expression::ConditionalExpression(o)),
            ForStatementInit::FunctionExpression(o) => Ok(Expression::FunctionExpression(o)),
            ForStatementInit::ImportExpression(o) => Ok(Expression::ImportExpression(o)),
            ForStatementInit::LogicalExpression(o) => Ok(Expression::LogicalExpression(o)),
            ForStatementInit::NewExpression(o) => Ok(Expression::NewExpression(o)),
            ForStatementInit::ObjectExpression(o) => Ok(Expression::ObjectExpression(o)),
            ForStatementInit::ParenthesizedExpression(o) => {
                Ok(Expression::ParenthesizedExpression(o))
            }
            ForStatementInit::SequenceExpression(o) => Ok(Expression::SequenceExpression(o)),
            ForStatementInit::TaggedTemplateExpression(o) => {
                Ok(Expression::TaggedTemplateExpression(o))
            }
            ForStatementInit::ThisExpression(o) => Ok(Expression::ThisExpression(o)),
            ForStatementInit::UnaryExpression(o) => Ok(Expression::UnaryExpression(o)),
            ForStatementInit::UpdateExpression(o) => Ok(Expression::UpdateExpression(o)),
            ForStatementInit::YieldExpression(o) => Ok(Expression::YieldExpression(o)),
            ForStatementInit::PrivateInExpression(o) => Ok(Expression::PrivateInExpression(o)),
            ForStatementInit::JSXElement(o) => Ok(Expression::JSXElement(o)),
            ForStatementInit::JSXFragment(o) => Ok(Expression::JSXFragment(o)),
            ForStatementInit::TSAsExpression(o) => Ok(Expression::TSAsExpression(o)),
            ForStatementInit::TSSatisfiesExpression(o) => Ok(Expression::TSSatisfiesExpression(o)),
            ForStatementInit::TSTypeAssertion(o) => Ok(Expression::TSTypeAssertion(o)),
            ForStatementInit::TSNonNullExpression(o) => Ok(Expression::TSNonNullExpression(o)),
            ForStatementInit::TSInstantiationExpression(o) => {
                Ok(Expression::TSInstantiationExpression(o))
            }
            ForStatementInit::V8IntrinsicExpression(o) => Ok(Expression::V8IntrinsicExpression(o)),
            ForStatementInit::ComputedMemberExpression(o) => {
                Ok(Expression::ComputedMemberExpression(o))
            }
            ForStatementInit::StaticMemberExpression(o) => {
                Ok(Expression::StaticMemberExpression(o))
            }
            ForStatementInit::PrivateFieldExpression(o) => {
                Ok(Expression::PrivateFieldExpression(o))
            }
            _ => Err(()),
        }
    }
}

impl<'a> From<Expression<'a>> for ForStatementInit<'a> {
    /// Convert an [`Expression`] to a [`ForStatementInit`].
    #[inline]
    fn from(value: Expression<'a>) -> Self {
        // Compiler should implement this as zero-cost transmute as discriminants
        // for `child` and `parent` are aligned
        match value {
            Expression::BooleanLiteral(o) => ForStatementInit::BooleanLiteral(o),
            Expression::NullLiteral(o) => ForStatementInit::NullLiteral(o),
            Expression::NumericLiteral(o) => ForStatementInit::NumericLiteral(o),
            Expression::BigIntLiteral(o) => ForStatementInit::BigIntLiteral(o),
            Expression::RegExpLiteral(o) => ForStatementInit::RegExpLiteral(o),
            Expression::StringLiteral(o) => ForStatementInit::StringLiteral(o),
            Expression::TemplateLiteral(o) => ForStatementInit::TemplateLiteral(o),
            Expression::Identifier(o) => ForStatementInit::Identifier(o),
            Expression::MetaProperty(o) => ForStatementInit::MetaProperty(o),
            Expression::Super(o) => ForStatementInit::Super(o),
            Expression::ArrayExpression(o) => ForStatementInit::ArrayExpression(o),
            Expression::ArrowFunctionExpression(o) => ForStatementInit::ArrowFunctionExpression(o),
            Expression::AssignmentExpression(o) => ForStatementInit::AssignmentExpression(o),
            Expression::AwaitExpression(o) => ForStatementInit::AwaitExpression(o),
            Expression::BinaryExpression(o) => ForStatementInit::BinaryExpression(o),
            Expression::CallExpression(o) => ForStatementInit::CallExpression(o),
            Expression::ChainExpression(o) => ForStatementInit::ChainExpression(o),
            Expression::ClassExpression(o) => ForStatementInit::ClassExpression(o),
            Expression::ConditionalExpression(o) => ForStatementInit::ConditionalExpression(o),
            Expression::FunctionExpression(o) => ForStatementInit::FunctionExpression(o),
            Expression::ImportExpression(o) => ForStatementInit::ImportExpression(o),
            Expression::LogicalExpression(o) => ForStatementInit::LogicalExpression(o),
            Expression::NewExpression(o) => ForStatementInit::NewExpression(o),
            Expression::ObjectExpression(o) => ForStatementInit::ObjectExpression(o),
            Expression::ParenthesizedExpression(o) => ForStatementInit::ParenthesizedExpression(o),
            Expression::SequenceExpression(o) => ForStatementInit::SequenceExpression(o),
            Expression::TaggedTemplateExpression(o) => {
                ForStatementInit::TaggedTemplateExpression(o)
            }
            Expression::ThisExpression(o) => ForStatementInit::ThisExpression(o),
            Expression::UnaryExpression(o) => ForStatementInit::UnaryExpression(o),
            Expression::UpdateExpression(o) => ForStatementInit::UpdateExpression(o),
            Expression::YieldExpression(o) => ForStatementInit::YieldExpression(o),
            Expression::PrivateInExpression(o) => ForStatementInit::PrivateInExpression(o),
            Expression::JSXElement(o) => ForStatementInit::JSXElement(o),
            Expression::JSXFragment(o) => ForStatementInit::JSXFragment(o),
            Expression::TSAsExpression(o) => ForStatementInit::TSAsExpression(o),
            Expression::TSSatisfiesExpression(o) => ForStatementInit::TSSatisfiesExpression(o),
            Expression::TSTypeAssertion(o) => ForStatementInit::TSTypeAssertion(o),
            Expression::TSNonNullExpression(o) => ForStatementInit::TSNonNullExpression(o),
            Expression::TSInstantiationExpression(o) => {
                ForStatementInit::TSInstantiationExpression(o)
            }
            Expression::V8IntrinsicExpression(o) => ForStatementInit::V8IntrinsicExpression(o),
            Expression::ComputedMemberExpression(o) => {
                ForStatementInit::ComputedMemberExpression(o)
            }
            Expression::StaticMemberExpression(o) => ForStatementInit::StaticMemberExpression(o),
            Expression::PrivateFieldExpression(o) => ForStatementInit::PrivateFieldExpression(o),
        }
    }
}

impl<'a> ForStatementInit<'a> {
    /// Return if a [`ForStatementInit`] is a [`MemberExpression`].
    #[inline]
    pub fn is_member_expression(&self) -> bool {
        matches!(
            self,
            Self::ComputedMemberExpression(_)
                | Self::StaticMemberExpression(_)
                | Self::PrivateFieldExpression(_)
        )
    }

    /// Convert a [`ForStatementInit`] to a [`MemberExpression`].
    ///
    /// # Panics
    /// Panics if not convertible.
    #[inline]
    pub fn into_member_expression(self) -> MemberExpression<'a> {
        MemberExpression::try_from(self).unwrap()
    }

    /// Convert a [`&ForStatementInit`] to a [`&MemberExpression`].
    ///
    /// [`&ForStatementInit`]: ForStatementInit
    /// [`&MemberExpression`]: MemberExpression
    #[inline]
    pub fn as_member_expression(&self) -> Option<&MemberExpression<'a>> {
        if self.is_member_expression() {
            // SAFETY: Transmute is safe because discriminants + types are identical between
            // `parent` and `child` for the shared variants
            Some(unsafe { &*std::ptr::from_ref(self).cast::<MemberExpression>() })
        } else {
            None
        }
    }

    /// Convert a [`&mut ForStatementInit`] to a [`&mut MemberExpression`].
    ///
    /// [`&mut ForStatementInit`]: ForStatementInit
    /// [`&mut MemberExpression`]: MemberExpression
    #[inline]
    pub fn as_member_expression_mut(&mut self) -> Option<&mut MemberExpression<'a>> {
        if self.is_member_expression() {
            // SAFETY: Transmute is safe because discriminants + types are identical between
            // `parent` and `child` for the shared variants
            Some(unsafe { &mut *std::ptr::from_mut(self).cast::<MemberExpression>() })
        } else {
            None
        }
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

impl<'a> MemberExpression<'a> {
    /// Convert a [`&MemberExpression`] to a [`&ForStatementInit`].
    ///
    /// [`&MemberExpression`]: MemberExpression
    /// [`&ForStatementInit`]: ForStatementInit
    #[inline]
    pub fn as_for_statement_init(&self) -> &ForStatementInit<'a> {
        // SAFETY: Transmute is safe because discriminants + types are identical between
        // `parent` and `child` for the shared variants
        unsafe { &*std::ptr::from_ref(self).cast::<ForStatementInit>() }
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
        // Compiler should implement this as a check of discriminant and then zero-cost transmute,
        // as discriminants for `parent` and `child` are aligned
        match value {
            ForStatementInit::ComputedMemberExpression(o) => {
                Ok(MemberExpression::ComputedMemberExpression(o))
            }
            ForStatementInit::StaticMemberExpression(o) => {
                Ok(MemberExpression::StaticMemberExpression(o))
            }
            ForStatementInit::PrivateFieldExpression(o) => {
                Ok(MemberExpression::PrivateFieldExpression(o))
            }
            _ => Err(()),
        }
    }
}

impl<'a> From<MemberExpression<'a>> for ForStatementInit<'a> {
    /// Convert a [`MemberExpression`] to a [`ForStatementInit`].
    #[inline]
    fn from(value: MemberExpression<'a>) -> Self {
        // Compiler should implement this as zero-cost transmute as discriminants
        // for `child` and `parent` are aligned
        match value {
            MemberExpression::ComputedMemberExpression(o) => {
                ForStatementInit::ComputedMemberExpression(o)
            }
            MemberExpression::StaticMemberExpression(o) => {
                ForStatementInit::StaticMemberExpression(o)
            }
            MemberExpression::PrivateFieldExpression(o) => {
                ForStatementInit::PrivateFieldExpression(o)
            }
        }
    }
}

impl<'a> ForStatementLeft<'a> {
    /// Return if a [`ForStatementLeft`] is an [`AssignmentTarget`].
    #[inline]
    pub fn is_assignment_target(&self) -> bool {
        matches!(
            self,
            Self::AssignmentTargetIdentifier(_)
                | Self::TSAsExpression(_)
                | Self::TSSatisfiesExpression(_)
                | Self::TSNonNullExpression(_)
                | Self::TSTypeAssertion(_)
                | Self::ComputedMemberExpression(_)
                | Self::StaticMemberExpression(_)
                | Self::PrivateFieldExpression(_)
                | Self::ArrayAssignmentTarget(_)
                | Self::ObjectAssignmentTarget(_)
        )
    }

    /// Convert a [`ForStatementLeft`] to an [`AssignmentTarget`].
    ///
    /// # Panics
    /// Panics if not convertible.
    #[inline]
    pub fn into_assignment_target(self) -> AssignmentTarget<'a> {
        AssignmentTarget::try_from(self).unwrap()
    }

    /// Convert a [`&ForStatementLeft`] to an [`&AssignmentTarget`].
    ///
    /// [`&ForStatementLeft`]: ForStatementLeft
    /// [`&AssignmentTarget`]: AssignmentTarget
    #[inline]
    pub fn as_assignment_target(&self) -> Option<&AssignmentTarget<'a>> {
        if self.is_assignment_target() {
            // SAFETY: Transmute is safe because discriminants + types are identical between
            // `parent` and `child` for the shared variants
            Some(unsafe { &*std::ptr::from_ref(self).cast::<AssignmentTarget>() })
        } else {
            None
        }
    }

    /// Convert a [`&mut ForStatementLeft`] to an [`&mut AssignmentTarget`].
    ///
    /// [`&mut ForStatementLeft`]: ForStatementLeft
    /// [`&mut AssignmentTarget`]: AssignmentTarget
    #[inline]
    pub fn as_assignment_target_mut(&mut self) -> Option<&mut AssignmentTarget<'a>> {
        if self.is_assignment_target() {
            // SAFETY: Transmute is safe because discriminants + types are identical between
            // `parent` and `child` for the shared variants
            Some(unsafe { &mut *std::ptr::from_mut(self).cast::<AssignmentTarget>() })
        } else {
            None
        }
    }

    /// Convert a [`&ForStatementLeft`] to an [`&AssignmentTarget`].
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

    /// Convert a [`&mut ForStatementLeft`] to an [`&mut AssignmentTarget`].
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

impl<'a> AssignmentTarget<'a> {
    /// Convert an [`&AssignmentTarget`] to a [`&ForStatementLeft`].
    ///
    /// [`&AssignmentTarget`]: AssignmentTarget
    /// [`&ForStatementLeft`]: ForStatementLeft
    #[inline]
    pub fn as_for_statement_left(&self) -> &ForStatementLeft<'a> {
        // SAFETY: Transmute is safe because discriminants + types are identical between
        // `parent` and `child` for the shared variants
        unsafe { &*std::ptr::from_ref(self).cast::<ForStatementLeft>() }
    }
}

impl<'a> TryFrom<ForStatementLeft<'a>> for AssignmentTarget<'a> {
    type Error = ();

    /// Convert a [`ForStatementLeft`] to an [`AssignmentTarget`].
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
            ForStatementLeft::ComputedMemberExpression(o) => {
                Ok(AssignmentTarget::ComputedMemberExpression(o))
            }
            ForStatementLeft::StaticMemberExpression(o) => {
                Ok(AssignmentTarget::StaticMemberExpression(o))
            }
            ForStatementLeft::PrivateFieldExpression(o) => {
                Ok(AssignmentTarget::PrivateFieldExpression(o))
            }
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
    /// Convert an [`AssignmentTarget`] to a [`ForStatementLeft`].
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
            AssignmentTarget::ComputedMemberExpression(o) => {
                ForStatementLeft::ComputedMemberExpression(o)
            }
            AssignmentTarget::StaticMemberExpression(o) => {
                ForStatementLeft::StaticMemberExpression(o)
            }
            AssignmentTarget::PrivateFieldExpression(o) => {
                ForStatementLeft::PrivateFieldExpression(o)
            }
            AssignmentTarget::ArrayAssignmentTarget(o) => {
                ForStatementLeft::ArrayAssignmentTarget(o)
            }
            AssignmentTarget::ObjectAssignmentTarget(o) => {
                ForStatementLeft::ObjectAssignmentTarget(o)
            }
        }
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
                | Self::ComputedMemberExpression(_)
                | Self::StaticMemberExpression(_)
                | Self::PrivateFieldExpression(_)
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

    /// Convert a [`&ForStatementLeft`] to a [`&SimpleAssignmentTarget`].
    ///
    /// [`&ForStatementLeft`]: ForStatementLeft
    /// [`&SimpleAssignmentTarget`]: SimpleAssignmentTarget
    #[inline]
    pub fn as_simple_assignment_target(&self) -> Option<&SimpleAssignmentTarget<'a>> {
        if self.is_simple_assignment_target() {
            // SAFETY: Transmute is safe because discriminants + types are identical between
            // `parent` and `child` for the shared variants
            Some(unsafe { &*std::ptr::from_ref(self).cast::<SimpleAssignmentTarget>() })
        } else {
            None
        }
    }

    /// Convert a [`&mut ForStatementLeft`] to a [`&mut SimpleAssignmentTarget`].
    ///
    /// [`&mut ForStatementLeft`]: ForStatementLeft
    /// [`&mut SimpleAssignmentTarget`]: SimpleAssignmentTarget
    #[inline]
    pub fn as_simple_assignment_target_mut(&mut self) -> Option<&mut SimpleAssignmentTarget<'a>> {
        if self.is_simple_assignment_target() {
            // SAFETY: Transmute is safe because discriminants + types are identical between
            // `parent` and `child` for the shared variants
            Some(unsafe { &mut *std::ptr::from_mut(self).cast::<SimpleAssignmentTarget>() })
        } else {
            None
        }
    }

    /// Convert a [`&ForStatementLeft`] to a [`&SimpleAssignmentTarget`].
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

    /// Convert a [`&mut ForStatementLeft`] to a [`&mut SimpleAssignmentTarget`].
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

impl<'a> SimpleAssignmentTarget<'a> {
    /// Convert a [`&SimpleAssignmentTarget`] to a [`&ForStatementLeft`].
    ///
    /// [`&SimpleAssignmentTarget`]: SimpleAssignmentTarget
    /// [`&ForStatementLeft`]: ForStatementLeft
    #[inline]
    pub fn as_for_statement_left(&self) -> &ForStatementLeft<'a> {
        // SAFETY: Transmute is safe because discriminants + types are identical between
        // `parent` and `child` for the shared variants
        unsafe { &*std::ptr::from_ref(self).cast::<ForStatementLeft>() }
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
            ForStatementLeft::ComputedMemberExpression(o) => {
                Ok(SimpleAssignmentTarget::ComputedMemberExpression(o))
            }
            ForStatementLeft::StaticMemberExpression(o) => {
                Ok(SimpleAssignmentTarget::StaticMemberExpression(o))
            }
            ForStatementLeft::PrivateFieldExpression(o) => {
                Ok(SimpleAssignmentTarget::PrivateFieldExpression(o))
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
            SimpleAssignmentTarget::ComputedMemberExpression(o) => {
                ForStatementLeft::ComputedMemberExpression(o)
            }
            SimpleAssignmentTarget::StaticMemberExpression(o) => {
                ForStatementLeft::StaticMemberExpression(o)
            }
            SimpleAssignmentTarget::PrivateFieldExpression(o) => {
                ForStatementLeft::PrivateFieldExpression(o)
            }
        }
    }
}

impl<'a> ForStatementLeft<'a> {
    /// Return if a [`ForStatementLeft`] is a [`MemberExpression`].
    #[inline]
    pub fn is_member_expression(&self) -> bool {
        matches!(
            self,
            Self::ComputedMemberExpression(_)
                | Self::StaticMemberExpression(_)
                | Self::PrivateFieldExpression(_)
        )
    }

    /// Convert a [`ForStatementLeft`] to a [`MemberExpression`].
    ///
    /// # Panics
    /// Panics if not convertible.
    #[inline]
    pub fn into_member_expression(self) -> MemberExpression<'a> {
        MemberExpression::try_from(self).unwrap()
    }

    /// Convert a [`&ForStatementLeft`] to a [`&MemberExpression`].
    ///
    /// [`&ForStatementLeft`]: ForStatementLeft
    /// [`&MemberExpression`]: MemberExpression
    #[inline]
    pub fn as_member_expression(&self) -> Option<&MemberExpression<'a>> {
        if self.is_member_expression() {
            // SAFETY: Transmute is safe because discriminants + types are identical between
            // `parent` and `child` for the shared variants
            Some(unsafe { &*std::ptr::from_ref(self).cast::<MemberExpression>() })
        } else {
            None
        }
    }

    /// Convert a [`&mut ForStatementLeft`] to a [`&mut MemberExpression`].
    ///
    /// [`&mut ForStatementLeft`]: ForStatementLeft
    /// [`&mut MemberExpression`]: MemberExpression
    #[inline]
    pub fn as_member_expression_mut(&mut self) -> Option<&mut MemberExpression<'a>> {
        if self.is_member_expression() {
            // SAFETY: Transmute is safe because discriminants + types are identical between
            // `parent` and `child` for the shared variants
            Some(unsafe { &mut *std::ptr::from_mut(self).cast::<MemberExpression>() })
        } else {
            None
        }
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

impl<'a> MemberExpression<'a> {
    /// Convert a [`&MemberExpression`] to a [`&ForStatementLeft`].
    ///
    /// [`&MemberExpression`]: MemberExpression
    /// [`&ForStatementLeft`]: ForStatementLeft
    #[inline]
    pub fn as_for_statement_left(&self) -> &ForStatementLeft<'a> {
        // SAFETY: Transmute is safe because discriminants + types are identical between
        // `parent` and `child` for the shared variants
        unsafe { &*std::ptr::from_ref(self).cast::<ForStatementLeft>() }
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
        // Compiler should implement this as a check of discriminant and then zero-cost transmute,
        // as discriminants for `parent` and `child` are aligned
        match value {
            ForStatementLeft::ComputedMemberExpression(o) => {
                Ok(MemberExpression::ComputedMemberExpression(o))
            }
            ForStatementLeft::StaticMemberExpression(o) => {
                Ok(MemberExpression::StaticMemberExpression(o))
            }
            ForStatementLeft::PrivateFieldExpression(o) => {
                Ok(MemberExpression::PrivateFieldExpression(o))
            }
            _ => Err(()),
        }
    }
}

impl<'a> From<MemberExpression<'a>> for ForStatementLeft<'a> {
    /// Convert a [`MemberExpression`] to a [`ForStatementLeft`].
    #[inline]
    fn from(value: MemberExpression<'a>) -> Self {
        // Compiler should implement this as zero-cost transmute as discriminants
        // for `child` and `parent` are aligned
        match value {
            MemberExpression::ComputedMemberExpression(o) => {
                ForStatementLeft::ComputedMemberExpression(o)
            }
            MemberExpression::StaticMemberExpression(o) => {
                ForStatementLeft::StaticMemberExpression(o)
            }
            MemberExpression::PrivateFieldExpression(o) => {
                ForStatementLeft::PrivateFieldExpression(o)
            }
        }
    }
}

impl<'a> ForStatementLeft<'a> {
    /// Return if a [`ForStatementLeft`] is an [`AssignmentTargetPattern`].
    #[inline]
    pub fn is_assignment_target_pattern(&self) -> bool {
        matches!(self, Self::ArrayAssignmentTarget(_) | Self::ObjectAssignmentTarget(_))
    }

    /// Convert a [`ForStatementLeft`] to an [`AssignmentTargetPattern`].
    ///
    /// # Panics
    /// Panics if not convertible.
    #[inline]
    pub fn into_assignment_target_pattern(self) -> AssignmentTargetPattern<'a> {
        AssignmentTargetPattern::try_from(self).unwrap()
    }

    /// Convert a [`&ForStatementLeft`] to an [`&AssignmentTargetPattern`].
    ///
    /// [`&ForStatementLeft`]: ForStatementLeft
    /// [`&AssignmentTargetPattern`]: AssignmentTargetPattern
    #[inline]
    pub fn as_assignment_target_pattern(&self) -> Option<&AssignmentTargetPattern<'a>> {
        if self.is_assignment_target_pattern() {
            // SAFETY: Transmute is safe because discriminants + types are identical between
            // `parent` and `child` for the shared variants
            Some(unsafe { &*std::ptr::from_ref(self).cast::<AssignmentTargetPattern>() })
        } else {
            None
        }
    }

    /// Convert a [`&mut ForStatementLeft`] to an [`&mut AssignmentTargetPattern`].
    ///
    /// [`&mut ForStatementLeft`]: ForStatementLeft
    /// [`&mut AssignmentTargetPattern`]: AssignmentTargetPattern
    #[inline]
    pub fn as_assignment_target_pattern_mut(&mut self) -> Option<&mut AssignmentTargetPattern<'a>> {
        if self.is_assignment_target_pattern() {
            // SAFETY: Transmute is safe because discriminants + types are identical between
            // `parent` and `child` for the shared variants
            Some(unsafe { &mut *std::ptr::from_mut(self).cast::<AssignmentTargetPattern>() })
        } else {
            None
        }
    }

    /// Convert a [`&ForStatementLeft`] to an [`&AssignmentTargetPattern`].
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

    /// Convert a [`&mut ForStatementLeft`] to an [`&mut AssignmentTargetPattern`].
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

impl<'a> AssignmentTargetPattern<'a> {
    /// Convert an [`&AssignmentTargetPattern`] to a [`&ForStatementLeft`].
    ///
    /// [`&AssignmentTargetPattern`]: AssignmentTargetPattern
    /// [`&ForStatementLeft`]: ForStatementLeft
    #[inline]
    pub fn as_for_statement_left(&self) -> &ForStatementLeft<'a> {
        // SAFETY: Transmute is safe because discriminants + types are identical between
        // `parent` and `child` for the shared variants
        unsafe { &*std::ptr::from_ref(self).cast::<ForStatementLeft>() }
    }
}

impl<'a> TryFrom<ForStatementLeft<'a>> for AssignmentTargetPattern<'a> {
    type Error = ();

    /// Convert a [`ForStatementLeft`] to an [`AssignmentTargetPattern`].
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
    /// Convert an [`AssignmentTargetPattern`] to a [`ForStatementLeft`].
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

impl<'a> ExportDefaultDeclarationKind<'a> {
    /// Return if an [`ExportDefaultDeclarationKind`] is an [`Expression`].
    #[inline]
    pub fn is_expression(&self) -> bool {
        matches!(
            self,
            Self::BooleanLiteral(_)
                | Self::NullLiteral(_)
                | Self::NumericLiteral(_)
                | Self::BigIntLiteral(_)
                | Self::RegExpLiteral(_)
                | Self::StringLiteral(_)
                | Self::TemplateLiteral(_)
                | Self::Identifier(_)
                | Self::MetaProperty(_)
                | Self::Super(_)
                | Self::ArrayExpression(_)
                | Self::ArrowFunctionExpression(_)
                | Self::AssignmentExpression(_)
                | Self::AwaitExpression(_)
                | Self::BinaryExpression(_)
                | Self::CallExpression(_)
                | Self::ChainExpression(_)
                | Self::ClassExpression(_)
                | Self::ConditionalExpression(_)
                | Self::FunctionExpression(_)
                | Self::ImportExpression(_)
                | Self::LogicalExpression(_)
                | Self::NewExpression(_)
                | Self::ObjectExpression(_)
                | Self::ParenthesizedExpression(_)
                | Self::SequenceExpression(_)
                | Self::TaggedTemplateExpression(_)
                | Self::ThisExpression(_)
                | Self::UnaryExpression(_)
                | Self::UpdateExpression(_)
                | Self::YieldExpression(_)
                | Self::PrivateInExpression(_)
                | Self::JSXElement(_)
                | Self::JSXFragment(_)
                | Self::TSAsExpression(_)
                | Self::TSSatisfiesExpression(_)
                | Self::TSTypeAssertion(_)
                | Self::TSNonNullExpression(_)
                | Self::TSInstantiationExpression(_)
                | Self::V8IntrinsicExpression(_)
                | Self::ComputedMemberExpression(_)
                | Self::StaticMemberExpression(_)
                | Self::PrivateFieldExpression(_)
        )
    }

    /// Convert an [`ExportDefaultDeclarationKind`] to an [`Expression`].
    ///
    /// # Panics
    /// Panics if not convertible.
    #[inline]
    pub fn into_expression(self) -> Expression<'a> {
        Expression::try_from(self).unwrap()
    }

    /// Convert an [`&ExportDefaultDeclarationKind`] to an [`&Expression`].
    ///
    /// [`&ExportDefaultDeclarationKind`]: ExportDefaultDeclarationKind
    /// [`&Expression`]: Expression
    #[inline]
    pub fn as_expression(&self) -> Option<&Expression<'a>> {
        if self.is_expression() {
            // SAFETY: Transmute is safe because discriminants + types are identical between
            // `parent` and `child` for the shared variants
            Some(unsafe { &*std::ptr::from_ref(self).cast::<Expression>() })
        } else {
            None
        }
    }

    /// Convert an [`&mut ExportDefaultDeclarationKind`] to an [`&mut Expression`].
    ///
    /// [`&mut ExportDefaultDeclarationKind`]: ExportDefaultDeclarationKind
    /// [`&mut Expression`]: Expression
    #[inline]
    pub fn as_expression_mut(&mut self) -> Option<&mut Expression<'a>> {
        if self.is_expression() {
            // SAFETY: Transmute is safe because discriminants + types are identical between
            // `parent` and `child` for the shared variants
            Some(unsafe { &mut *std::ptr::from_mut(self).cast::<Expression>() })
        } else {
            None
        }
    }

    /// Convert an [`&ExportDefaultDeclarationKind`] to an [`&Expression`].
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

    /// Convert an [`&mut ExportDefaultDeclarationKind`] to an [`&mut Expression`].
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

impl<'a> Expression<'a> {
    /// Convert an [`&Expression`] to an [`&ExportDefaultDeclarationKind`].
    ///
    /// [`&Expression`]: Expression
    /// [`&ExportDefaultDeclarationKind`]: ExportDefaultDeclarationKind
    #[inline]
    pub fn as_export_default_declaration_kind(&self) -> &ExportDefaultDeclarationKind<'a> {
        // SAFETY: Transmute is safe because discriminants + types are identical between
        // `parent` and `child` for the shared variants
        unsafe { &*std::ptr::from_ref(self).cast::<ExportDefaultDeclarationKind>() }
    }
}

impl<'a> TryFrom<ExportDefaultDeclarationKind<'a>> for Expression<'a> {
    type Error = ();

    /// Convert an [`ExportDefaultDeclarationKind`] to an [`Expression`].
    ///
    /// # Errors
    /// Returns `Err` if not convertible.
    #[inline]
    fn try_from(value: ExportDefaultDeclarationKind<'a>) -> Result<Self, Self::Error> {
        // Compiler should implement this as a check of discriminant and then zero-cost transmute,
        // as discriminants for `parent` and `child` are aligned
        match value {
            ExportDefaultDeclarationKind::BooleanLiteral(o) => Ok(Expression::BooleanLiteral(o)),
            ExportDefaultDeclarationKind::NullLiteral(o) => Ok(Expression::NullLiteral(o)),
            ExportDefaultDeclarationKind::NumericLiteral(o) => Ok(Expression::NumericLiteral(o)),
            ExportDefaultDeclarationKind::BigIntLiteral(o) => Ok(Expression::BigIntLiteral(o)),
            ExportDefaultDeclarationKind::RegExpLiteral(o) => Ok(Expression::RegExpLiteral(o)),
            ExportDefaultDeclarationKind::StringLiteral(o) => Ok(Expression::StringLiteral(o)),
            ExportDefaultDeclarationKind::TemplateLiteral(o) => Ok(Expression::TemplateLiteral(o)),
            ExportDefaultDeclarationKind::Identifier(o) => Ok(Expression::Identifier(o)),
            ExportDefaultDeclarationKind::MetaProperty(o) => Ok(Expression::MetaProperty(o)),
            ExportDefaultDeclarationKind::Super(o) => Ok(Expression::Super(o)),
            ExportDefaultDeclarationKind::ArrayExpression(o) => Ok(Expression::ArrayExpression(o)),
            ExportDefaultDeclarationKind::ArrowFunctionExpression(o) => {
                Ok(Expression::ArrowFunctionExpression(o))
            }
            ExportDefaultDeclarationKind::AssignmentExpression(o) => {
                Ok(Expression::AssignmentExpression(o))
            }
            ExportDefaultDeclarationKind::AwaitExpression(o) => Ok(Expression::AwaitExpression(o)),
            ExportDefaultDeclarationKind::BinaryExpression(o) => {
                Ok(Expression::BinaryExpression(o))
            }
            ExportDefaultDeclarationKind::CallExpression(o) => Ok(Expression::CallExpression(o)),
            ExportDefaultDeclarationKind::ChainExpression(o) => Ok(Expression::ChainExpression(o)),
            ExportDefaultDeclarationKind::ClassExpression(o) => Ok(Expression::ClassExpression(o)),
            ExportDefaultDeclarationKind::ConditionalExpression(o) => {
                Ok(Expression::ConditionalExpression(o))
            }
            ExportDefaultDeclarationKind::FunctionExpression(o) => {
                Ok(Expression::FunctionExpression(o))
            }
            ExportDefaultDeclarationKind::ImportExpression(o) => {
                Ok(Expression::ImportExpression(o))
            }
            ExportDefaultDeclarationKind::LogicalExpression(o) => {
                Ok(Expression::LogicalExpression(o))
            }
            ExportDefaultDeclarationKind::NewExpression(o) => Ok(Expression::NewExpression(o)),
            ExportDefaultDeclarationKind::ObjectExpression(o) => {
                Ok(Expression::ObjectExpression(o))
            }
            ExportDefaultDeclarationKind::ParenthesizedExpression(o) => {
                Ok(Expression::ParenthesizedExpression(o))
            }
            ExportDefaultDeclarationKind::SequenceExpression(o) => {
                Ok(Expression::SequenceExpression(o))
            }
            ExportDefaultDeclarationKind::TaggedTemplateExpression(o) => {
                Ok(Expression::TaggedTemplateExpression(o))
            }
            ExportDefaultDeclarationKind::ThisExpression(o) => Ok(Expression::ThisExpression(o)),
            ExportDefaultDeclarationKind::UnaryExpression(o) => Ok(Expression::UnaryExpression(o)),
            ExportDefaultDeclarationKind::UpdateExpression(o) => {
                Ok(Expression::UpdateExpression(o))
            }
            ExportDefaultDeclarationKind::YieldExpression(o) => Ok(Expression::YieldExpression(o)),
            ExportDefaultDeclarationKind::PrivateInExpression(o) => {
                Ok(Expression::PrivateInExpression(o))
            }
            ExportDefaultDeclarationKind::JSXElement(o) => Ok(Expression::JSXElement(o)),
            ExportDefaultDeclarationKind::JSXFragment(o) => Ok(Expression::JSXFragment(o)),
            ExportDefaultDeclarationKind::TSAsExpression(o) => Ok(Expression::TSAsExpression(o)),
            ExportDefaultDeclarationKind::TSSatisfiesExpression(o) => {
                Ok(Expression::TSSatisfiesExpression(o))
            }
            ExportDefaultDeclarationKind::TSTypeAssertion(o) => Ok(Expression::TSTypeAssertion(o)),
            ExportDefaultDeclarationKind::TSNonNullExpression(o) => {
                Ok(Expression::TSNonNullExpression(o))
            }
            ExportDefaultDeclarationKind::TSInstantiationExpression(o) => {
                Ok(Expression::TSInstantiationExpression(o))
            }
            ExportDefaultDeclarationKind::V8IntrinsicExpression(o) => {
                Ok(Expression::V8IntrinsicExpression(o))
            }
            ExportDefaultDeclarationKind::ComputedMemberExpression(o) => {
                Ok(Expression::ComputedMemberExpression(o))
            }
            ExportDefaultDeclarationKind::StaticMemberExpression(o) => {
                Ok(Expression::StaticMemberExpression(o))
            }
            ExportDefaultDeclarationKind::PrivateFieldExpression(o) => {
                Ok(Expression::PrivateFieldExpression(o))
            }
            _ => Err(()),
        }
    }
}

impl<'a> From<Expression<'a>> for ExportDefaultDeclarationKind<'a> {
    /// Convert an [`Expression`] to an [`ExportDefaultDeclarationKind`].
    #[inline]
    fn from(value: Expression<'a>) -> Self {
        // Compiler should implement this as zero-cost transmute as discriminants
        // for `child` and `parent` are aligned
        match value {
            Expression::BooleanLiteral(o) => ExportDefaultDeclarationKind::BooleanLiteral(o),
            Expression::NullLiteral(o) => ExportDefaultDeclarationKind::NullLiteral(o),
            Expression::NumericLiteral(o) => ExportDefaultDeclarationKind::NumericLiteral(o),
            Expression::BigIntLiteral(o) => ExportDefaultDeclarationKind::BigIntLiteral(o),
            Expression::RegExpLiteral(o) => ExportDefaultDeclarationKind::RegExpLiteral(o),
            Expression::StringLiteral(o) => ExportDefaultDeclarationKind::StringLiteral(o),
            Expression::TemplateLiteral(o) => ExportDefaultDeclarationKind::TemplateLiteral(o),
            Expression::Identifier(o) => ExportDefaultDeclarationKind::Identifier(o),
            Expression::MetaProperty(o) => ExportDefaultDeclarationKind::MetaProperty(o),
            Expression::Super(o) => ExportDefaultDeclarationKind::Super(o),
            Expression::ArrayExpression(o) => ExportDefaultDeclarationKind::ArrayExpression(o),
            Expression::ArrowFunctionExpression(o) => {
                ExportDefaultDeclarationKind::ArrowFunctionExpression(o)
            }
            Expression::AssignmentExpression(o) => {
                ExportDefaultDeclarationKind::AssignmentExpression(o)
            }
            Expression::AwaitExpression(o) => ExportDefaultDeclarationKind::AwaitExpression(o),
            Expression::BinaryExpression(o) => ExportDefaultDeclarationKind::BinaryExpression(o),
            Expression::CallExpression(o) => ExportDefaultDeclarationKind::CallExpression(o),
            Expression::ChainExpression(o) => ExportDefaultDeclarationKind::ChainExpression(o),
            Expression::ClassExpression(o) => ExportDefaultDeclarationKind::ClassExpression(o),
            Expression::ConditionalExpression(o) => {
                ExportDefaultDeclarationKind::ConditionalExpression(o)
            }
            Expression::FunctionExpression(o) => {
                ExportDefaultDeclarationKind::FunctionExpression(o)
            }
            Expression::ImportExpression(o) => ExportDefaultDeclarationKind::ImportExpression(o),
            Expression::LogicalExpression(o) => ExportDefaultDeclarationKind::LogicalExpression(o),
            Expression::NewExpression(o) => ExportDefaultDeclarationKind::NewExpression(o),
            Expression::ObjectExpression(o) => ExportDefaultDeclarationKind::ObjectExpression(o),
            Expression::ParenthesizedExpression(o) => {
                ExportDefaultDeclarationKind::ParenthesizedExpression(o)
            }
            Expression::SequenceExpression(o) => {
                ExportDefaultDeclarationKind::SequenceExpression(o)
            }
            Expression::TaggedTemplateExpression(o) => {
                ExportDefaultDeclarationKind::TaggedTemplateExpression(o)
            }
            Expression::ThisExpression(o) => ExportDefaultDeclarationKind::ThisExpression(o),
            Expression::UnaryExpression(o) => ExportDefaultDeclarationKind::UnaryExpression(o),
            Expression::UpdateExpression(o) => ExportDefaultDeclarationKind::UpdateExpression(o),
            Expression::YieldExpression(o) => ExportDefaultDeclarationKind::YieldExpression(o),
            Expression::PrivateInExpression(o) => {
                ExportDefaultDeclarationKind::PrivateInExpression(o)
            }
            Expression::JSXElement(o) => ExportDefaultDeclarationKind::JSXElement(o),
            Expression::JSXFragment(o) => ExportDefaultDeclarationKind::JSXFragment(o),
            Expression::TSAsExpression(o) => ExportDefaultDeclarationKind::TSAsExpression(o),
            Expression::TSSatisfiesExpression(o) => {
                ExportDefaultDeclarationKind::TSSatisfiesExpression(o)
            }
            Expression::TSTypeAssertion(o) => ExportDefaultDeclarationKind::TSTypeAssertion(o),
            Expression::TSNonNullExpression(o) => {
                ExportDefaultDeclarationKind::TSNonNullExpression(o)
            }
            Expression::TSInstantiationExpression(o) => {
                ExportDefaultDeclarationKind::TSInstantiationExpression(o)
            }
            Expression::V8IntrinsicExpression(o) => {
                ExportDefaultDeclarationKind::V8IntrinsicExpression(o)
            }
            Expression::ComputedMemberExpression(o) => {
                ExportDefaultDeclarationKind::ComputedMemberExpression(o)
            }
            Expression::StaticMemberExpression(o) => {
                ExportDefaultDeclarationKind::StaticMemberExpression(o)
            }
            Expression::PrivateFieldExpression(o) => {
                ExportDefaultDeclarationKind::PrivateFieldExpression(o)
            }
        }
    }
}

impl<'a> ExportDefaultDeclarationKind<'a> {
    /// Return if an [`ExportDefaultDeclarationKind`] is a [`MemberExpression`].
    #[inline]
    pub fn is_member_expression(&self) -> bool {
        matches!(
            self,
            Self::ComputedMemberExpression(_)
                | Self::StaticMemberExpression(_)
                | Self::PrivateFieldExpression(_)
        )
    }

    /// Convert an [`ExportDefaultDeclarationKind`] to a [`MemberExpression`].
    ///
    /// # Panics
    /// Panics if not convertible.
    #[inline]
    pub fn into_member_expression(self) -> MemberExpression<'a> {
        MemberExpression::try_from(self).unwrap()
    }

    /// Convert an [`&ExportDefaultDeclarationKind`] to a [`&MemberExpression`].
    ///
    /// [`&ExportDefaultDeclarationKind`]: ExportDefaultDeclarationKind
    /// [`&MemberExpression`]: MemberExpression
    #[inline]
    pub fn as_member_expression(&self) -> Option<&MemberExpression<'a>> {
        if self.is_member_expression() {
            // SAFETY: Transmute is safe because discriminants + types are identical between
            // `parent` and `child` for the shared variants
            Some(unsafe { &*std::ptr::from_ref(self).cast::<MemberExpression>() })
        } else {
            None
        }
    }

    /// Convert an [`&mut ExportDefaultDeclarationKind`] to a [`&mut MemberExpression`].
    ///
    /// [`&mut ExportDefaultDeclarationKind`]: ExportDefaultDeclarationKind
    /// [`&mut MemberExpression`]: MemberExpression
    #[inline]
    pub fn as_member_expression_mut(&mut self) -> Option<&mut MemberExpression<'a>> {
        if self.is_member_expression() {
            // SAFETY: Transmute is safe because discriminants + types are identical between
            // `parent` and `child` for the shared variants
            Some(unsafe { &mut *std::ptr::from_mut(self).cast::<MemberExpression>() })
        } else {
            None
        }
    }

    /// Convert an [`&ExportDefaultDeclarationKind`] to a [`&MemberExpression`].
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

    /// Convert an [`&mut ExportDefaultDeclarationKind`] to a [`&mut MemberExpression`].
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

impl<'a> MemberExpression<'a> {
    /// Convert a [`&MemberExpression`] to an [`&ExportDefaultDeclarationKind`].
    ///
    /// [`&MemberExpression`]: MemberExpression
    /// [`&ExportDefaultDeclarationKind`]: ExportDefaultDeclarationKind
    #[inline]
    pub fn as_export_default_declaration_kind(&self) -> &ExportDefaultDeclarationKind<'a> {
        // SAFETY: Transmute is safe because discriminants + types are identical between
        // `parent` and `child` for the shared variants
        unsafe { &*std::ptr::from_ref(self).cast::<ExportDefaultDeclarationKind>() }
    }
}

impl<'a> TryFrom<ExportDefaultDeclarationKind<'a>> for MemberExpression<'a> {
    type Error = ();

    /// Convert an [`ExportDefaultDeclarationKind`] to a [`MemberExpression`].
    ///
    /// # Errors
    /// Returns `Err` if not convertible.
    #[inline]
    fn try_from(value: ExportDefaultDeclarationKind<'a>) -> Result<Self, Self::Error> {
        // Compiler should implement this as a check of discriminant and then zero-cost transmute,
        // as discriminants for `parent` and `child` are aligned
        match value {
            ExportDefaultDeclarationKind::ComputedMemberExpression(o) => {
                Ok(MemberExpression::ComputedMemberExpression(o))
            }
            ExportDefaultDeclarationKind::StaticMemberExpression(o) => {
                Ok(MemberExpression::StaticMemberExpression(o))
            }
            ExportDefaultDeclarationKind::PrivateFieldExpression(o) => {
                Ok(MemberExpression::PrivateFieldExpression(o))
            }
            _ => Err(()),
        }
    }
}

impl<'a> From<MemberExpression<'a>> for ExportDefaultDeclarationKind<'a> {
    /// Convert a [`MemberExpression`] to an [`ExportDefaultDeclarationKind`].
    #[inline]
    fn from(value: MemberExpression<'a>) -> Self {
        // Compiler should implement this as zero-cost transmute as discriminants
        // for `child` and `parent` are aligned
        match value {
            MemberExpression::ComputedMemberExpression(o) => {
                ExportDefaultDeclarationKind::ComputedMemberExpression(o)
            }
            MemberExpression::StaticMemberExpression(o) => {
                ExportDefaultDeclarationKind::StaticMemberExpression(o)
            }
            MemberExpression::PrivateFieldExpression(o) => {
                ExportDefaultDeclarationKind::PrivateFieldExpression(o)
            }
        }
    }
}

impl<'a> JSXExpression<'a> {
    /// Return if a [`JSXExpression`] is an [`Expression`].
    #[inline]
    pub fn is_expression(&self) -> bool {
        matches!(
            self,
            Self::BooleanLiteral(_)
                | Self::NullLiteral(_)
                | Self::NumericLiteral(_)
                | Self::BigIntLiteral(_)
                | Self::RegExpLiteral(_)
                | Self::StringLiteral(_)
                | Self::TemplateLiteral(_)
                | Self::Identifier(_)
                | Self::MetaProperty(_)
                | Self::Super(_)
                | Self::ArrayExpression(_)
                | Self::ArrowFunctionExpression(_)
                | Self::AssignmentExpression(_)
                | Self::AwaitExpression(_)
                | Self::BinaryExpression(_)
                | Self::CallExpression(_)
                | Self::ChainExpression(_)
                | Self::ClassExpression(_)
                | Self::ConditionalExpression(_)
                | Self::FunctionExpression(_)
                | Self::ImportExpression(_)
                | Self::LogicalExpression(_)
                | Self::NewExpression(_)
                | Self::ObjectExpression(_)
                | Self::ParenthesizedExpression(_)
                | Self::SequenceExpression(_)
                | Self::TaggedTemplateExpression(_)
                | Self::ThisExpression(_)
                | Self::UnaryExpression(_)
                | Self::UpdateExpression(_)
                | Self::YieldExpression(_)
                | Self::PrivateInExpression(_)
                | Self::JSXElement(_)
                | Self::JSXFragment(_)
                | Self::TSAsExpression(_)
                | Self::TSSatisfiesExpression(_)
                | Self::TSTypeAssertion(_)
                | Self::TSNonNullExpression(_)
                | Self::TSInstantiationExpression(_)
                | Self::V8IntrinsicExpression(_)
                | Self::ComputedMemberExpression(_)
                | Self::StaticMemberExpression(_)
                | Self::PrivateFieldExpression(_)
        )
    }

    /// Convert a [`JSXExpression`] to an [`Expression`].
    ///
    /// # Panics
    /// Panics if not convertible.
    #[inline]
    pub fn into_expression(self) -> Expression<'a> {
        Expression::try_from(self).unwrap()
    }

    /// Convert a [`&JSXExpression`] to an [`&Expression`].
    ///
    /// [`&JSXExpression`]: JSXExpression
    /// [`&Expression`]: Expression
    #[inline]
    pub fn as_expression(&self) -> Option<&Expression<'a>> {
        if self.is_expression() {
            // SAFETY: Transmute is safe because discriminants + types are identical between
            // `parent` and `child` for the shared variants
            Some(unsafe { &*std::ptr::from_ref(self).cast::<Expression>() })
        } else {
            None
        }
    }

    /// Convert a [`&mut JSXExpression`] to an [`&mut Expression`].
    ///
    /// [`&mut JSXExpression`]: JSXExpression
    /// [`&mut Expression`]: Expression
    #[inline]
    pub fn as_expression_mut(&mut self) -> Option<&mut Expression<'a>> {
        if self.is_expression() {
            // SAFETY: Transmute is safe because discriminants + types are identical between
            // `parent` and `child` for the shared variants
            Some(unsafe { &mut *std::ptr::from_mut(self).cast::<Expression>() })
        } else {
            None
        }
    }

    /// Convert a [`&JSXExpression`] to an [`&Expression`].
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

    /// Convert a [`&mut JSXExpression`] to an [`&mut Expression`].
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

impl<'a> Expression<'a> {
    /// Convert an [`&Expression`] to a [`&JSXExpression`].
    ///
    /// [`&Expression`]: Expression
    /// [`&JSXExpression`]: JSXExpression
    #[inline]
    pub fn as_jsx_expression(&self) -> &JSXExpression<'a> {
        // SAFETY: Transmute is safe because discriminants + types are identical between
        // `parent` and `child` for the shared variants
        unsafe { &*std::ptr::from_ref(self).cast::<JSXExpression>() }
    }
}

impl<'a> TryFrom<JSXExpression<'a>> for Expression<'a> {
    type Error = ();

    /// Convert a [`JSXExpression`] to an [`Expression`].
    ///
    /// # Errors
    /// Returns `Err` if not convertible.
    #[inline]
    fn try_from(value: JSXExpression<'a>) -> Result<Self, Self::Error> {
        // Compiler should implement this as a check of discriminant and then zero-cost transmute,
        // as discriminants for `parent` and `child` are aligned
        match value {
            JSXExpression::BooleanLiteral(o) => Ok(Expression::BooleanLiteral(o)),
            JSXExpression::NullLiteral(o) => Ok(Expression::NullLiteral(o)),
            JSXExpression::NumericLiteral(o) => Ok(Expression::NumericLiteral(o)),
            JSXExpression::BigIntLiteral(o) => Ok(Expression::BigIntLiteral(o)),
            JSXExpression::RegExpLiteral(o) => Ok(Expression::RegExpLiteral(o)),
            JSXExpression::StringLiteral(o) => Ok(Expression::StringLiteral(o)),
            JSXExpression::TemplateLiteral(o) => Ok(Expression::TemplateLiteral(o)),
            JSXExpression::Identifier(o) => Ok(Expression::Identifier(o)),
            JSXExpression::MetaProperty(o) => Ok(Expression::MetaProperty(o)),
            JSXExpression::Super(o) => Ok(Expression::Super(o)),
            JSXExpression::ArrayExpression(o) => Ok(Expression::ArrayExpression(o)),
            JSXExpression::ArrowFunctionExpression(o) => Ok(Expression::ArrowFunctionExpression(o)),
            JSXExpression::AssignmentExpression(o) => Ok(Expression::AssignmentExpression(o)),
            JSXExpression::AwaitExpression(o) => Ok(Expression::AwaitExpression(o)),
            JSXExpression::BinaryExpression(o) => Ok(Expression::BinaryExpression(o)),
            JSXExpression::CallExpression(o) => Ok(Expression::CallExpression(o)),
            JSXExpression::ChainExpression(o) => Ok(Expression::ChainExpression(o)),
            JSXExpression::ClassExpression(o) => Ok(Expression::ClassExpression(o)),
            JSXExpression::ConditionalExpression(o) => Ok(Expression::ConditionalExpression(o)),
            JSXExpression::FunctionExpression(o) => Ok(Expression::FunctionExpression(o)),
            JSXExpression::ImportExpression(o) => Ok(Expression::ImportExpression(o)),
            JSXExpression::LogicalExpression(o) => Ok(Expression::LogicalExpression(o)),
            JSXExpression::NewExpression(o) => Ok(Expression::NewExpression(o)),
            JSXExpression::ObjectExpression(o) => Ok(Expression::ObjectExpression(o)),
            JSXExpression::ParenthesizedExpression(o) => Ok(Expression::ParenthesizedExpression(o)),
            JSXExpression::SequenceExpression(o) => Ok(Expression::SequenceExpression(o)),
            JSXExpression::TaggedTemplateExpression(o) => {
                Ok(Expression::TaggedTemplateExpression(o))
            }
            JSXExpression::ThisExpression(o) => Ok(Expression::ThisExpression(o)),
            JSXExpression::UnaryExpression(o) => Ok(Expression::UnaryExpression(o)),
            JSXExpression::UpdateExpression(o) => Ok(Expression::UpdateExpression(o)),
            JSXExpression::YieldExpression(o) => Ok(Expression::YieldExpression(o)),
            JSXExpression::PrivateInExpression(o) => Ok(Expression::PrivateInExpression(o)),
            JSXExpression::JSXElement(o) => Ok(Expression::JSXElement(o)),
            JSXExpression::JSXFragment(o) => Ok(Expression::JSXFragment(o)),
            JSXExpression::TSAsExpression(o) => Ok(Expression::TSAsExpression(o)),
            JSXExpression::TSSatisfiesExpression(o) => Ok(Expression::TSSatisfiesExpression(o)),
            JSXExpression::TSTypeAssertion(o) => Ok(Expression::TSTypeAssertion(o)),
            JSXExpression::TSNonNullExpression(o) => Ok(Expression::TSNonNullExpression(o)),
            JSXExpression::TSInstantiationExpression(o) => {
                Ok(Expression::TSInstantiationExpression(o))
            }
            JSXExpression::V8IntrinsicExpression(o) => Ok(Expression::V8IntrinsicExpression(o)),
            JSXExpression::ComputedMemberExpression(o) => {
                Ok(Expression::ComputedMemberExpression(o))
            }
            JSXExpression::StaticMemberExpression(o) => Ok(Expression::StaticMemberExpression(o)),
            JSXExpression::PrivateFieldExpression(o) => Ok(Expression::PrivateFieldExpression(o)),
            _ => Err(()),
        }
    }
}

impl<'a> From<Expression<'a>> for JSXExpression<'a> {
    /// Convert an [`Expression`] to a [`JSXExpression`].
    #[inline]
    fn from(value: Expression<'a>) -> Self {
        // Compiler should implement this as zero-cost transmute as discriminants
        // for `child` and `parent` are aligned
        match value {
            Expression::BooleanLiteral(o) => JSXExpression::BooleanLiteral(o),
            Expression::NullLiteral(o) => JSXExpression::NullLiteral(o),
            Expression::NumericLiteral(o) => JSXExpression::NumericLiteral(o),
            Expression::BigIntLiteral(o) => JSXExpression::BigIntLiteral(o),
            Expression::RegExpLiteral(o) => JSXExpression::RegExpLiteral(o),
            Expression::StringLiteral(o) => JSXExpression::StringLiteral(o),
            Expression::TemplateLiteral(o) => JSXExpression::TemplateLiteral(o),
            Expression::Identifier(o) => JSXExpression::Identifier(o),
            Expression::MetaProperty(o) => JSXExpression::MetaProperty(o),
            Expression::Super(o) => JSXExpression::Super(o),
            Expression::ArrayExpression(o) => JSXExpression::ArrayExpression(o),
            Expression::ArrowFunctionExpression(o) => JSXExpression::ArrowFunctionExpression(o),
            Expression::AssignmentExpression(o) => JSXExpression::AssignmentExpression(o),
            Expression::AwaitExpression(o) => JSXExpression::AwaitExpression(o),
            Expression::BinaryExpression(o) => JSXExpression::BinaryExpression(o),
            Expression::CallExpression(o) => JSXExpression::CallExpression(o),
            Expression::ChainExpression(o) => JSXExpression::ChainExpression(o),
            Expression::ClassExpression(o) => JSXExpression::ClassExpression(o),
            Expression::ConditionalExpression(o) => JSXExpression::ConditionalExpression(o),
            Expression::FunctionExpression(o) => JSXExpression::FunctionExpression(o),
            Expression::ImportExpression(o) => JSXExpression::ImportExpression(o),
            Expression::LogicalExpression(o) => JSXExpression::LogicalExpression(o),
            Expression::NewExpression(o) => JSXExpression::NewExpression(o),
            Expression::ObjectExpression(o) => JSXExpression::ObjectExpression(o),
            Expression::ParenthesizedExpression(o) => JSXExpression::ParenthesizedExpression(o),
            Expression::SequenceExpression(o) => JSXExpression::SequenceExpression(o),
            Expression::TaggedTemplateExpression(o) => JSXExpression::TaggedTemplateExpression(o),
            Expression::ThisExpression(o) => JSXExpression::ThisExpression(o),
            Expression::UnaryExpression(o) => JSXExpression::UnaryExpression(o),
            Expression::UpdateExpression(o) => JSXExpression::UpdateExpression(o),
            Expression::YieldExpression(o) => JSXExpression::YieldExpression(o),
            Expression::PrivateInExpression(o) => JSXExpression::PrivateInExpression(o),
            Expression::JSXElement(o) => JSXExpression::JSXElement(o),
            Expression::JSXFragment(o) => JSXExpression::JSXFragment(o),
            Expression::TSAsExpression(o) => JSXExpression::TSAsExpression(o),
            Expression::TSSatisfiesExpression(o) => JSXExpression::TSSatisfiesExpression(o),
            Expression::TSTypeAssertion(o) => JSXExpression::TSTypeAssertion(o),
            Expression::TSNonNullExpression(o) => JSXExpression::TSNonNullExpression(o),
            Expression::TSInstantiationExpression(o) => JSXExpression::TSInstantiationExpression(o),
            Expression::V8IntrinsicExpression(o) => JSXExpression::V8IntrinsicExpression(o),
            Expression::ComputedMemberExpression(o) => JSXExpression::ComputedMemberExpression(o),
            Expression::StaticMemberExpression(o) => JSXExpression::StaticMemberExpression(o),
            Expression::PrivateFieldExpression(o) => JSXExpression::PrivateFieldExpression(o),
        }
    }
}

impl<'a> JSXExpression<'a> {
    /// Return if a [`JSXExpression`] is a [`MemberExpression`].
    #[inline]
    pub fn is_member_expression(&self) -> bool {
        matches!(
            self,
            Self::ComputedMemberExpression(_)
                | Self::StaticMemberExpression(_)
                | Self::PrivateFieldExpression(_)
        )
    }

    /// Convert a [`JSXExpression`] to a [`MemberExpression`].
    ///
    /// # Panics
    /// Panics if not convertible.
    #[inline]
    pub fn into_member_expression(self) -> MemberExpression<'a> {
        MemberExpression::try_from(self).unwrap()
    }

    /// Convert a [`&JSXExpression`] to a [`&MemberExpression`].
    ///
    /// [`&JSXExpression`]: JSXExpression
    /// [`&MemberExpression`]: MemberExpression
    #[inline]
    pub fn as_member_expression(&self) -> Option<&MemberExpression<'a>> {
        if self.is_member_expression() {
            // SAFETY: Transmute is safe because discriminants + types are identical between
            // `parent` and `child` for the shared variants
            Some(unsafe { &*std::ptr::from_ref(self).cast::<MemberExpression>() })
        } else {
            None
        }
    }

    /// Convert a [`&mut JSXExpression`] to a [`&mut MemberExpression`].
    ///
    /// [`&mut JSXExpression`]: JSXExpression
    /// [`&mut MemberExpression`]: MemberExpression
    #[inline]
    pub fn as_member_expression_mut(&mut self) -> Option<&mut MemberExpression<'a>> {
        if self.is_member_expression() {
            // SAFETY: Transmute is safe because discriminants + types are identical between
            // `parent` and `child` for the shared variants
            Some(unsafe { &mut *std::ptr::from_mut(self).cast::<MemberExpression>() })
        } else {
            None
        }
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

impl<'a> MemberExpression<'a> {
    /// Convert a [`&MemberExpression`] to a [`&JSXExpression`].
    ///
    /// [`&MemberExpression`]: MemberExpression
    /// [`&JSXExpression`]: JSXExpression
    #[inline]
    pub fn as_jsx_expression(&self) -> &JSXExpression<'a> {
        // SAFETY: Transmute is safe because discriminants + types are identical between
        // `parent` and `child` for the shared variants
        unsafe { &*std::ptr::from_ref(self).cast::<JSXExpression>() }
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
        // Compiler should implement this as a check of discriminant and then zero-cost transmute,
        // as discriminants for `parent` and `child` are aligned
        match value {
            JSXExpression::ComputedMemberExpression(o) => {
                Ok(MemberExpression::ComputedMemberExpression(o))
            }
            JSXExpression::StaticMemberExpression(o) => {
                Ok(MemberExpression::StaticMemberExpression(o))
            }
            JSXExpression::PrivateFieldExpression(o) => {
                Ok(MemberExpression::PrivateFieldExpression(o))
            }
            _ => Err(()),
        }
    }
}

impl<'a> From<MemberExpression<'a>> for JSXExpression<'a> {
    /// Convert a [`MemberExpression`] to a [`JSXExpression`].
    #[inline]
    fn from(value: MemberExpression<'a>) -> Self {
        // Compiler should implement this as zero-cost transmute as discriminants
        // for `child` and `parent` are aligned
        match value {
            MemberExpression::ComputedMemberExpression(o) => {
                JSXExpression::ComputedMemberExpression(o)
            }
            MemberExpression::StaticMemberExpression(o) => JSXExpression::StaticMemberExpression(o),
            MemberExpression::PrivateFieldExpression(o) => JSXExpression::PrivateFieldExpression(o),
        }
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
            Some(unsafe { &*std::ptr::from_ref(self).cast::<TSType>() })
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
            Some(unsafe { &mut *std::ptr::from_mut(self).cast::<TSType>() })
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
        unsafe { &*std::ptr::from_ref(self).cast::<TSTupleElement>() }
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
            Some(unsafe { &*std::ptr::from_ref(self).cast::<TSTypeName>() })
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
            Some(unsafe { &mut *std::ptr::from_mut(self).cast::<TSTypeName>() })
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
        unsafe { &*std::ptr::from_ref(self).cast::<TSTypeQueryExprName>() }
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

/// Macro for matching [`Expression`]'s variants.
///
/// Includes variants inherited from [`MemberExpression`].
#[macro_export]
macro_rules! match_expression {
    ($ty:ident) => {
        $ty::BooleanLiteral(_)
            | $ty::NullLiteral(_)
            | $ty::NumericLiteral(_)
            | $ty::BigIntLiteral(_)
            | $ty::RegExpLiteral(_)
            | $ty::StringLiteral(_)
            | $ty::TemplateLiteral(_)
            | $ty::Identifier(_)
            | $ty::MetaProperty(_)
            | $ty::Super(_)
            | $ty::ArrayExpression(_)
            | $ty::ArrowFunctionExpression(_)
            | $ty::AssignmentExpression(_)
            | $ty::AwaitExpression(_)
            | $ty::BinaryExpression(_)
            | $ty::CallExpression(_)
            | $ty::ChainExpression(_)
            | $ty::ClassExpression(_)
            | $ty::ConditionalExpression(_)
            | $ty::FunctionExpression(_)
            | $ty::ImportExpression(_)
            | $ty::LogicalExpression(_)
            | $ty::NewExpression(_)
            | $ty::ObjectExpression(_)
            | $ty::ParenthesizedExpression(_)
            | $ty::SequenceExpression(_)
            | $ty::TaggedTemplateExpression(_)
            | $ty::ThisExpression(_)
            | $ty::UnaryExpression(_)
            | $ty::UpdateExpression(_)
            | $ty::YieldExpression(_)
            | $ty::PrivateInExpression(_)
            | $ty::JSXElement(_)
            | $ty::JSXFragment(_)
            | $ty::TSAsExpression(_)
            | $ty::TSSatisfiesExpression(_)
            | $ty::TSTypeAssertion(_)
            | $ty::TSNonNullExpression(_)
            | $ty::TSInstantiationExpression(_)
            | $ty::V8IntrinsicExpression(_)
            | $ty::ComputedMemberExpression(_)
            | $ty::StaticMemberExpression(_)
            | $ty::PrivateFieldExpression(_)
    };
}
pub use match_expression;

/// Macro for matching [`MemberExpression`]'s variants.
#[macro_export]
macro_rules! match_member_expression {
    ($ty:ident) => {
        $ty::ComputedMemberExpression(_)
            | $ty::StaticMemberExpression(_)
            | $ty::PrivateFieldExpression(_)
    };
}
pub use match_member_expression;

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
            | $ty::ComputedMemberExpression(_)
            | $ty::StaticMemberExpression(_)
            | $ty::PrivateFieldExpression(_)
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
            | $ty::ComputedMemberExpression(_)
            | $ty::StaticMemberExpression(_)
            | $ty::PrivateFieldExpression(_)
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
