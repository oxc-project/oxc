// Auto-generated code, DO NOT EDIT DIRECTLY!
// To edit this generated file you have to edit `tasks/ast_tools/src/generators/minifier_traverse/mod.rs`.

#![expect(
    clippy::cast_ptr_alignment,
    clippy::elidable_lifetime_names,
    clippy::ptr_as_ptr,
    clippy::ref_option,
    clippy::undocumented_unsafe_blocks
)]
#![allow(clippy::redundant_pub_crate)]

use std::{cell::Cell, marker::PhantomData, mem::offset_of};

use oxc_allocator::{Address, ArenaBox, ArenaVec, GetAddress};
use oxc_ast::ast::*;
use oxc_syntax::{node::NodeId, scope::ScopeId};

/// Type of [`Ancestor`].
/// Used in [`crate::TraverseCtx::retag_stack`].
#[repr(u16)]
#[derive(Clone, Copy)]
pub(crate) enum AncestorType {
    None = 0,
    ProgramHashbang = 1,
    ProgramDirectives = 2,
    ProgramBody = 3,
    ArrayExpressionElements = 4,
    ObjectExpressionProperties = 5,
    ObjectPropertyKey = 6,
    ObjectPropertyValue = 7,
    TemplateLiteralQuasis = 8,
    TemplateLiteralExpressions = 9,
    TaggedTemplateExpressionTag = 10,
    TaggedTemplateExpressionQuasi = 11,
    ComputedMemberExpressionObject = 12,
    ComputedMemberExpressionExpression = 13,
    StaticMemberExpressionObject = 14,
    StaticMemberExpressionProperty = 15,
    PrivateFieldExpressionObject = 16,
    PrivateFieldExpressionField = 17,
    CallExpressionCallee = 18,
    CallExpressionArguments = 19,
    NewExpressionCallee = 20,
    NewExpressionArguments = 21,
    MetaPropertyMeta = 22,
    MetaPropertyProperty = 23,
    SpreadElementArgument = 24,
    UpdateExpressionArgument = 25,
    UnaryExpressionArgument = 26,
    BinaryExpressionLeft = 27,
    BinaryExpressionRight = 28,
    PrivateInExpressionLeft = 29,
    PrivateInExpressionRight = 30,
    LogicalExpressionLeft = 31,
    LogicalExpressionRight = 32,
    ConditionalExpressionTest = 33,
    ConditionalExpressionConsequent = 34,
    ConditionalExpressionAlternate = 35,
    AssignmentExpressionLeft = 36,
    AssignmentExpressionRight = 37,
    ArrayAssignmentTargetElements = 38,
    ArrayAssignmentTargetRest = 39,
    ObjectAssignmentTargetProperties = 40,
    ObjectAssignmentTargetRest = 41,
    AssignmentTargetRestTarget = 42,
    AssignmentTargetWithDefaultBinding = 43,
    AssignmentTargetWithDefaultInit = 44,
    AssignmentTargetPropertyIdentifierBinding = 45,
    AssignmentTargetPropertyIdentifierInit = 46,
    AssignmentTargetPropertyPropertyName = 47,
    AssignmentTargetPropertyPropertyBinding = 48,
    SequenceExpressionExpressions = 49,
    AwaitExpressionArgument = 50,
    ChainExpressionExpression = 51,
    ParenthesizedExpressionExpression = 52,
    DirectiveExpression = 53,
    BlockStatementBody = 54,
    VariableDeclarationDeclarations = 55,
    VariableDeclaratorId = 56,
    VariableDeclaratorInit = 57,
    ExpressionStatementExpression = 58,
    IfStatementTest = 59,
    IfStatementConsequent = 60,
    IfStatementAlternate = 61,
    DoWhileStatementBody = 62,
    DoWhileStatementTest = 63,
    WhileStatementTest = 64,
    WhileStatementBody = 65,
    ForStatementInit = 66,
    ForStatementTest = 67,
    ForStatementUpdate = 68,
    ForStatementBody = 69,
    ForInStatementLeft = 70,
    ForInStatementRight = 71,
    ForInStatementBody = 72,
    ForOfStatementLeft = 73,
    ForOfStatementRight = 74,
    ForOfStatementBody = 75,
    ContinueStatementLabel = 76,
    BreakStatementLabel = 77,
    ReturnStatementArgument = 78,
    WithStatementObject = 79,
    WithStatementBody = 80,
    SwitchStatementDiscriminant = 81,
    SwitchStatementCases = 82,
    SwitchCaseTest = 83,
    SwitchCaseConsequent = 84,
    LabeledStatementLabel = 85,
    LabeledStatementBody = 86,
    ThrowStatementArgument = 87,
    TryStatementBlock = 88,
    TryStatementHandler = 89,
    TryStatementFinalizer = 90,
    CatchClauseParam = 91,
    CatchClauseBody = 92,
    CatchParameterPattern = 93,
    AssignmentPatternLeft = 94,
    AssignmentPatternRight = 95,
    ObjectPatternProperties = 96,
    ObjectPatternRest = 97,
    BindingPropertyKey = 98,
    BindingPropertyValue = 99,
    ArrayPatternElements = 100,
    ArrayPatternRest = 101,
    BindingRestElementArgument = 102,
    FunctionId = 103,
    FunctionParams = 104,
    FunctionBody = 105,
    FormalParametersItems = 106,
    FormalParametersRest = 107,
    FormalParameterDecorators = 108,
    FormalParameterPattern = 109,
    FormalParameterInitializer = 110,
    FormalParameterRestDecorators = 111,
    FormalParameterRestRest = 112,
    FunctionBodyDirectives = 113,
    FunctionBodyStatements = 114,
    ArrowFunctionExpressionParams = 115,
    ArrowFunctionExpressionBody = 116,
    YieldExpressionArgument = 117,
    ClassDecorators = 118,
    ClassId = 119,
    ClassSuperClass = 120,
    ClassBody = 121,
    ClassBodyBody = 122,
    MethodDefinitionDecorators = 123,
    MethodDefinitionKey = 124,
    MethodDefinitionValue = 125,
    PropertyDefinitionDecorators = 126,
    PropertyDefinitionKey = 127,
    PropertyDefinitionValue = 128,
    StaticBlockBody = 129,
    AccessorPropertyDecorators = 130,
    AccessorPropertyKey = 131,
    AccessorPropertyValue = 132,
    ImportExpressionSource = 133,
    ImportExpressionOptions = 134,
    ImportDeclarationSpecifiers = 135,
    ImportDeclarationSource = 136,
    ImportDeclarationWithClause = 137,
    ImportSpecifierImported = 138,
    ImportSpecifierLocal = 139,
    ImportDefaultSpecifierLocal = 140,
    ImportNamespaceSpecifierLocal = 141,
    WithClauseWithEntries = 142,
    ImportAttributeKey = 143,
    ImportAttributeValue = 144,
    ExportNamedDeclarationDeclaration = 145,
    ExportNamedDeclarationSpecifiers = 146,
    ExportNamedDeclarationSource = 147,
    ExportNamedDeclarationWithClause = 148,
    ExportDefaultDeclarationDeclaration = 149,
    ExportAllDeclarationExported = 150,
    ExportAllDeclarationSource = 151,
    ExportAllDeclarationWithClause = 152,
    ExportSpecifierLocal = 153,
    ExportSpecifierExported = 154,
    V8IntrinsicExpressionName = 155,
    V8IntrinsicExpressionArguments = 156,
    JSXElementOpeningElement = 157,
    JSXElementChildren = 158,
    JSXElementClosingElement = 159,
    JSXOpeningElementName = 160,
    JSXOpeningElementAttributes = 161,
    JSXClosingElementName = 162,
    JSXFragmentOpeningFragment = 163,
    JSXFragmentChildren = 164,
    JSXFragmentClosingFragment = 165,
    JSXNamespacedNameNamespace = 166,
    JSXNamespacedNameName = 167,
    JSXMemberExpressionObject = 168,
    JSXMemberExpressionProperty = 169,
    JSXExpressionContainerExpression = 170,
    JSXAttributeName = 171,
    JSXAttributeValue = 172,
    JSXSpreadAttributeArgument = 173,
    JSXSpreadChildExpression = 174,
    DecoratorExpression = 175,
}

/// Ancestor type used in AST traversal.
///
/// Encodes both the type of the parent, and child's location in the parent.
/// i.e. variants for `BinaryExpressionLeft` and `BinaryExpressionRight`, not just `BinaryExpression`.
///
/// `'a` is lifetime of AST nodes.
/// `'t` is lifetime of the `Ancestor` (which inherits lifetime from `&'t TraverseCtx'`).
/// i.e. `Ancestor`s can only exist within the body of `enter_*` and `exit_*` methods
/// and cannot "escape" from them.
//
// SAFETY
// * This type must be `#[repr(u16)]`.
// * Variant discriminants must correspond to those in `AncestorType`.
//
// These invariants make it possible to set the discriminant of an `Ancestor` without altering
// the "payload" pointer with:
// `*(ancestor as *mut _ as *mut AncestorType) = AncestorType::Program`.
// `TraverseCtx::retag_stack` uses this technique.
#[repr(C, u16)]
#[derive(Clone, Copy, Debug)]
pub enum Ancestor<'a, 't> {
    None = AncestorType::None as u16,
    ProgramHashbang(ProgramWithoutHashbang<'a, 't>) = AncestorType::ProgramHashbang as u16,
    ProgramDirectives(ProgramWithoutDirectives<'a, 't>) = AncestorType::ProgramDirectives as u16,
    ProgramBody(ProgramWithoutBody<'a, 't>) = AncestorType::ProgramBody as u16,
    ArrayExpressionElements(ArrayExpressionWithoutElements<'a, 't>) =
        AncestorType::ArrayExpressionElements as u16,
    ObjectExpressionProperties(ObjectExpressionWithoutProperties<'a, 't>) =
        AncestorType::ObjectExpressionProperties as u16,
    ObjectPropertyKey(ObjectPropertyWithoutKey<'a, 't>) = AncestorType::ObjectPropertyKey as u16,
    ObjectPropertyValue(ObjectPropertyWithoutValue<'a, 't>) =
        AncestorType::ObjectPropertyValue as u16,
    TemplateLiteralQuasis(TemplateLiteralWithoutQuasis<'a, 't>) =
        AncestorType::TemplateLiteralQuasis as u16,
    TemplateLiteralExpressions(TemplateLiteralWithoutExpressions<'a, 't>) =
        AncestorType::TemplateLiteralExpressions as u16,
    TaggedTemplateExpressionTag(TaggedTemplateExpressionWithoutTag<'a, 't>) =
        AncestorType::TaggedTemplateExpressionTag as u16,
    TaggedTemplateExpressionQuasi(TaggedTemplateExpressionWithoutQuasi<'a, 't>) =
        AncestorType::TaggedTemplateExpressionQuasi as u16,
    ComputedMemberExpressionObject(ComputedMemberExpressionWithoutObject<'a, 't>) =
        AncestorType::ComputedMemberExpressionObject as u16,
    ComputedMemberExpressionExpression(ComputedMemberExpressionWithoutExpression<'a, 't>) =
        AncestorType::ComputedMemberExpressionExpression as u16,
    StaticMemberExpressionObject(StaticMemberExpressionWithoutObject<'a, 't>) =
        AncestorType::StaticMemberExpressionObject as u16,
    StaticMemberExpressionProperty(StaticMemberExpressionWithoutProperty<'a, 't>) =
        AncestorType::StaticMemberExpressionProperty as u16,
    PrivateFieldExpressionObject(PrivateFieldExpressionWithoutObject<'a, 't>) =
        AncestorType::PrivateFieldExpressionObject as u16,
    PrivateFieldExpressionField(PrivateFieldExpressionWithoutField<'a, 't>) =
        AncestorType::PrivateFieldExpressionField as u16,
    CallExpressionCallee(CallExpressionWithoutCallee<'a, 't>) =
        AncestorType::CallExpressionCallee as u16,
    CallExpressionArguments(CallExpressionWithoutArguments<'a, 't>) =
        AncestorType::CallExpressionArguments as u16,
    NewExpressionCallee(NewExpressionWithoutCallee<'a, 't>) =
        AncestorType::NewExpressionCallee as u16,
    NewExpressionArguments(NewExpressionWithoutArguments<'a, 't>) =
        AncestorType::NewExpressionArguments as u16,
    MetaPropertyMeta(MetaPropertyWithoutMeta<'a, 't>) = AncestorType::MetaPropertyMeta as u16,
    MetaPropertyProperty(MetaPropertyWithoutProperty<'a, 't>) =
        AncestorType::MetaPropertyProperty as u16,
    SpreadElementArgument(SpreadElementWithoutArgument<'a, 't>) =
        AncestorType::SpreadElementArgument as u16,
    UpdateExpressionArgument(UpdateExpressionWithoutArgument<'a, 't>) =
        AncestorType::UpdateExpressionArgument as u16,
    UnaryExpressionArgument(UnaryExpressionWithoutArgument<'a, 't>) =
        AncestorType::UnaryExpressionArgument as u16,
    BinaryExpressionLeft(BinaryExpressionWithoutLeft<'a, 't>) =
        AncestorType::BinaryExpressionLeft as u16,
    BinaryExpressionRight(BinaryExpressionWithoutRight<'a, 't>) =
        AncestorType::BinaryExpressionRight as u16,
    PrivateInExpressionLeft(PrivateInExpressionWithoutLeft<'a, 't>) =
        AncestorType::PrivateInExpressionLeft as u16,
    PrivateInExpressionRight(PrivateInExpressionWithoutRight<'a, 't>) =
        AncestorType::PrivateInExpressionRight as u16,
    LogicalExpressionLeft(LogicalExpressionWithoutLeft<'a, 't>) =
        AncestorType::LogicalExpressionLeft as u16,
    LogicalExpressionRight(LogicalExpressionWithoutRight<'a, 't>) =
        AncestorType::LogicalExpressionRight as u16,
    ConditionalExpressionTest(ConditionalExpressionWithoutTest<'a, 't>) =
        AncestorType::ConditionalExpressionTest as u16,
    ConditionalExpressionConsequent(ConditionalExpressionWithoutConsequent<'a, 't>) =
        AncestorType::ConditionalExpressionConsequent as u16,
    ConditionalExpressionAlternate(ConditionalExpressionWithoutAlternate<'a, 't>) =
        AncestorType::ConditionalExpressionAlternate as u16,
    AssignmentExpressionLeft(AssignmentExpressionWithoutLeft<'a, 't>) =
        AncestorType::AssignmentExpressionLeft as u16,
    AssignmentExpressionRight(AssignmentExpressionWithoutRight<'a, 't>) =
        AncestorType::AssignmentExpressionRight as u16,
    ArrayAssignmentTargetElements(ArrayAssignmentTargetWithoutElements<'a, 't>) =
        AncestorType::ArrayAssignmentTargetElements as u16,
    ArrayAssignmentTargetRest(ArrayAssignmentTargetWithoutRest<'a, 't>) =
        AncestorType::ArrayAssignmentTargetRest as u16,
    ObjectAssignmentTargetProperties(ObjectAssignmentTargetWithoutProperties<'a, 't>) =
        AncestorType::ObjectAssignmentTargetProperties as u16,
    ObjectAssignmentTargetRest(ObjectAssignmentTargetWithoutRest<'a, 't>) =
        AncestorType::ObjectAssignmentTargetRest as u16,
    AssignmentTargetRestTarget(AssignmentTargetRestWithoutTarget<'a, 't>) =
        AncestorType::AssignmentTargetRestTarget as u16,
    AssignmentTargetWithDefaultBinding(AssignmentTargetWithDefaultWithoutBinding<'a, 't>) =
        AncestorType::AssignmentTargetWithDefaultBinding as u16,
    AssignmentTargetWithDefaultInit(AssignmentTargetWithDefaultWithoutInit<'a, 't>) =
        AncestorType::AssignmentTargetWithDefaultInit as u16,
    AssignmentTargetPropertyIdentifierBinding(
        AssignmentTargetPropertyIdentifierWithoutBinding<'a, 't>,
    ) = AncestorType::AssignmentTargetPropertyIdentifierBinding as u16,
    AssignmentTargetPropertyIdentifierInit(AssignmentTargetPropertyIdentifierWithoutInit<'a, 't>) =
        AncestorType::AssignmentTargetPropertyIdentifierInit as u16,
    AssignmentTargetPropertyPropertyName(AssignmentTargetPropertyPropertyWithoutName<'a, 't>) =
        AncestorType::AssignmentTargetPropertyPropertyName as u16,
    AssignmentTargetPropertyPropertyBinding(AssignmentTargetPropertyPropertyWithoutBinding<'a, 't>) =
        AncestorType::AssignmentTargetPropertyPropertyBinding as u16,
    SequenceExpressionExpressions(SequenceExpressionWithoutExpressions<'a, 't>) =
        AncestorType::SequenceExpressionExpressions as u16,
    AwaitExpressionArgument(AwaitExpressionWithoutArgument<'a, 't>) =
        AncestorType::AwaitExpressionArgument as u16,
    ChainExpressionExpression(ChainExpressionWithoutExpression<'a, 't>) =
        AncestorType::ChainExpressionExpression as u16,
    ParenthesizedExpressionExpression(ParenthesizedExpressionWithoutExpression<'a, 't>) =
        AncestorType::ParenthesizedExpressionExpression as u16,
    DirectiveExpression(DirectiveWithoutExpression<'a, 't>) =
        AncestorType::DirectiveExpression as u16,
    BlockStatementBody(BlockStatementWithoutBody<'a, 't>) = AncestorType::BlockStatementBody as u16,
    VariableDeclarationDeclarations(VariableDeclarationWithoutDeclarations<'a, 't>) =
        AncestorType::VariableDeclarationDeclarations as u16,
    VariableDeclaratorId(VariableDeclaratorWithoutId<'a, 't>) =
        AncestorType::VariableDeclaratorId as u16,
    VariableDeclaratorInit(VariableDeclaratorWithoutInit<'a, 't>) =
        AncestorType::VariableDeclaratorInit as u16,
    ExpressionStatementExpression(ExpressionStatementWithoutExpression<'a, 't>) =
        AncestorType::ExpressionStatementExpression as u16,
    IfStatementTest(IfStatementWithoutTest<'a, 't>) = AncestorType::IfStatementTest as u16,
    IfStatementConsequent(IfStatementWithoutConsequent<'a, 't>) =
        AncestorType::IfStatementConsequent as u16,
    IfStatementAlternate(IfStatementWithoutAlternate<'a, 't>) =
        AncestorType::IfStatementAlternate as u16,
    DoWhileStatementBody(DoWhileStatementWithoutBody<'a, 't>) =
        AncestorType::DoWhileStatementBody as u16,
    DoWhileStatementTest(DoWhileStatementWithoutTest<'a, 't>) =
        AncestorType::DoWhileStatementTest as u16,
    WhileStatementTest(WhileStatementWithoutTest<'a, 't>) = AncestorType::WhileStatementTest as u16,
    WhileStatementBody(WhileStatementWithoutBody<'a, 't>) = AncestorType::WhileStatementBody as u16,
    ForStatementInit(ForStatementWithoutInit<'a, 't>) = AncestorType::ForStatementInit as u16,
    ForStatementTest(ForStatementWithoutTest<'a, 't>) = AncestorType::ForStatementTest as u16,
    ForStatementUpdate(ForStatementWithoutUpdate<'a, 't>) = AncestorType::ForStatementUpdate as u16,
    ForStatementBody(ForStatementWithoutBody<'a, 't>) = AncestorType::ForStatementBody as u16,
    ForInStatementLeft(ForInStatementWithoutLeft<'a, 't>) = AncestorType::ForInStatementLeft as u16,
    ForInStatementRight(ForInStatementWithoutRight<'a, 't>) =
        AncestorType::ForInStatementRight as u16,
    ForInStatementBody(ForInStatementWithoutBody<'a, 't>) = AncestorType::ForInStatementBody as u16,
    ForOfStatementLeft(ForOfStatementWithoutLeft<'a, 't>) = AncestorType::ForOfStatementLeft as u16,
    ForOfStatementRight(ForOfStatementWithoutRight<'a, 't>) =
        AncestorType::ForOfStatementRight as u16,
    ForOfStatementBody(ForOfStatementWithoutBody<'a, 't>) = AncestorType::ForOfStatementBody as u16,
    ContinueStatementLabel(ContinueStatementWithoutLabel<'a, 't>) =
        AncestorType::ContinueStatementLabel as u16,
    BreakStatementLabel(BreakStatementWithoutLabel<'a, 't>) =
        AncestorType::BreakStatementLabel as u16,
    ReturnStatementArgument(ReturnStatementWithoutArgument<'a, 't>) =
        AncestorType::ReturnStatementArgument as u16,
    WithStatementObject(WithStatementWithoutObject<'a, 't>) =
        AncestorType::WithStatementObject as u16,
    WithStatementBody(WithStatementWithoutBody<'a, 't>) = AncestorType::WithStatementBody as u16,
    SwitchStatementDiscriminant(SwitchStatementWithoutDiscriminant<'a, 't>) =
        AncestorType::SwitchStatementDiscriminant as u16,
    SwitchStatementCases(SwitchStatementWithoutCases<'a, 't>) =
        AncestorType::SwitchStatementCases as u16,
    SwitchCaseTest(SwitchCaseWithoutTest<'a, 't>) = AncestorType::SwitchCaseTest as u16,
    SwitchCaseConsequent(SwitchCaseWithoutConsequent<'a, 't>) =
        AncestorType::SwitchCaseConsequent as u16,
    LabeledStatementLabel(LabeledStatementWithoutLabel<'a, 't>) =
        AncestorType::LabeledStatementLabel as u16,
    LabeledStatementBody(LabeledStatementWithoutBody<'a, 't>) =
        AncestorType::LabeledStatementBody as u16,
    ThrowStatementArgument(ThrowStatementWithoutArgument<'a, 't>) =
        AncestorType::ThrowStatementArgument as u16,
    TryStatementBlock(TryStatementWithoutBlock<'a, 't>) = AncestorType::TryStatementBlock as u16,
    TryStatementHandler(TryStatementWithoutHandler<'a, 't>) =
        AncestorType::TryStatementHandler as u16,
    TryStatementFinalizer(TryStatementWithoutFinalizer<'a, 't>) =
        AncestorType::TryStatementFinalizer as u16,
    CatchClauseParam(CatchClauseWithoutParam<'a, 't>) = AncestorType::CatchClauseParam as u16,
    CatchClauseBody(CatchClauseWithoutBody<'a, 't>) = AncestorType::CatchClauseBody as u16,
    CatchParameterPattern(CatchParameterWithoutPattern<'a, 't>) =
        AncestorType::CatchParameterPattern as u16,
    AssignmentPatternLeft(AssignmentPatternWithoutLeft<'a, 't>) =
        AncestorType::AssignmentPatternLeft as u16,
    AssignmentPatternRight(AssignmentPatternWithoutRight<'a, 't>) =
        AncestorType::AssignmentPatternRight as u16,
    ObjectPatternProperties(ObjectPatternWithoutProperties<'a, 't>) =
        AncestorType::ObjectPatternProperties as u16,
    ObjectPatternRest(ObjectPatternWithoutRest<'a, 't>) = AncestorType::ObjectPatternRest as u16,
    BindingPropertyKey(BindingPropertyWithoutKey<'a, 't>) = AncestorType::BindingPropertyKey as u16,
    BindingPropertyValue(BindingPropertyWithoutValue<'a, 't>) =
        AncestorType::BindingPropertyValue as u16,
    ArrayPatternElements(ArrayPatternWithoutElements<'a, 't>) =
        AncestorType::ArrayPatternElements as u16,
    ArrayPatternRest(ArrayPatternWithoutRest<'a, 't>) = AncestorType::ArrayPatternRest as u16,
    BindingRestElementArgument(BindingRestElementWithoutArgument<'a, 't>) =
        AncestorType::BindingRestElementArgument as u16,
    FunctionId(FunctionWithoutId<'a, 't>) = AncestorType::FunctionId as u16,
    FunctionParams(FunctionWithoutParams<'a, 't>) = AncestorType::FunctionParams as u16,
    FunctionBody(FunctionWithoutBody<'a, 't>) = AncestorType::FunctionBody as u16,
    FormalParametersItems(FormalParametersWithoutItems<'a, 't>) =
        AncestorType::FormalParametersItems as u16,
    FormalParametersRest(FormalParametersWithoutRest<'a, 't>) =
        AncestorType::FormalParametersRest as u16,
    FormalParameterDecorators(FormalParameterWithoutDecorators<'a, 't>) =
        AncestorType::FormalParameterDecorators as u16,
    FormalParameterPattern(FormalParameterWithoutPattern<'a, 't>) =
        AncestorType::FormalParameterPattern as u16,
    FormalParameterInitializer(FormalParameterWithoutInitializer<'a, 't>) =
        AncestorType::FormalParameterInitializer as u16,
    FormalParameterRestDecorators(FormalParameterRestWithoutDecorators<'a, 't>) =
        AncestorType::FormalParameterRestDecorators as u16,
    FormalParameterRestRest(FormalParameterRestWithoutRest<'a, 't>) =
        AncestorType::FormalParameterRestRest as u16,
    FunctionBodyDirectives(FunctionBodyWithoutDirectives<'a, 't>) =
        AncestorType::FunctionBodyDirectives as u16,
    FunctionBodyStatements(FunctionBodyWithoutStatements<'a, 't>) =
        AncestorType::FunctionBodyStatements as u16,
    ArrowFunctionExpressionParams(ArrowFunctionExpressionWithoutParams<'a, 't>) =
        AncestorType::ArrowFunctionExpressionParams as u16,
    ArrowFunctionExpressionBody(ArrowFunctionExpressionWithoutBody<'a, 't>) =
        AncestorType::ArrowFunctionExpressionBody as u16,
    YieldExpressionArgument(YieldExpressionWithoutArgument<'a, 't>) =
        AncestorType::YieldExpressionArgument as u16,
    ClassDecorators(ClassWithoutDecorators<'a, 't>) = AncestorType::ClassDecorators as u16,
    ClassId(ClassWithoutId<'a, 't>) = AncestorType::ClassId as u16,
    ClassSuperClass(ClassWithoutSuperClass<'a, 't>) = AncestorType::ClassSuperClass as u16,
    ClassBody(ClassWithoutBody<'a, 't>) = AncestorType::ClassBody as u16,
    ClassBodyBody(ClassBodyWithoutBody<'a, 't>) = AncestorType::ClassBodyBody as u16,
    MethodDefinitionDecorators(MethodDefinitionWithoutDecorators<'a, 't>) =
        AncestorType::MethodDefinitionDecorators as u16,
    MethodDefinitionKey(MethodDefinitionWithoutKey<'a, 't>) =
        AncestorType::MethodDefinitionKey as u16,
    MethodDefinitionValue(MethodDefinitionWithoutValue<'a, 't>) =
        AncestorType::MethodDefinitionValue as u16,
    PropertyDefinitionDecorators(PropertyDefinitionWithoutDecorators<'a, 't>) =
        AncestorType::PropertyDefinitionDecorators as u16,
    PropertyDefinitionKey(PropertyDefinitionWithoutKey<'a, 't>) =
        AncestorType::PropertyDefinitionKey as u16,
    PropertyDefinitionValue(PropertyDefinitionWithoutValue<'a, 't>) =
        AncestorType::PropertyDefinitionValue as u16,
    StaticBlockBody(StaticBlockWithoutBody<'a, 't>) = AncestorType::StaticBlockBody as u16,
    AccessorPropertyDecorators(AccessorPropertyWithoutDecorators<'a, 't>) =
        AncestorType::AccessorPropertyDecorators as u16,
    AccessorPropertyKey(AccessorPropertyWithoutKey<'a, 't>) =
        AncestorType::AccessorPropertyKey as u16,
    AccessorPropertyValue(AccessorPropertyWithoutValue<'a, 't>) =
        AncestorType::AccessorPropertyValue as u16,
    ImportExpressionSource(ImportExpressionWithoutSource<'a, 't>) =
        AncestorType::ImportExpressionSource as u16,
    ImportExpressionOptions(ImportExpressionWithoutOptions<'a, 't>) =
        AncestorType::ImportExpressionOptions as u16,
    ImportDeclarationSpecifiers(ImportDeclarationWithoutSpecifiers<'a, 't>) =
        AncestorType::ImportDeclarationSpecifiers as u16,
    ImportDeclarationSource(ImportDeclarationWithoutSource<'a, 't>) =
        AncestorType::ImportDeclarationSource as u16,
    ImportDeclarationWithClause(ImportDeclarationWithoutWithClause<'a, 't>) =
        AncestorType::ImportDeclarationWithClause as u16,
    ImportSpecifierImported(ImportSpecifierWithoutImported<'a, 't>) =
        AncestorType::ImportSpecifierImported as u16,
    ImportSpecifierLocal(ImportSpecifierWithoutLocal<'a, 't>) =
        AncestorType::ImportSpecifierLocal as u16,
    ImportDefaultSpecifierLocal(ImportDefaultSpecifierWithoutLocal<'a, 't>) =
        AncestorType::ImportDefaultSpecifierLocal as u16,
    ImportNamespaceSpecifierLocal(ImportNamespaceSpecifierWithoutLocal<'a, 't>) =
        AncestorType::ImportNamespaceSpecifierLocal as u16,
    WithClauseWithEntries(WithClauseWithoutWithEntries<'a, 't>) =
        AncestorType::WithClauseWithEntries as u16,
    ImportAttributeKey(ImportAttributeWithoutKey<'a, 't>) = AncestorType::ImportAttributeKey as u16,
    ImportAttributeValue(ImportAttributeWithoutValue<'a, 't>) =
        AncestorType::ImportAttributeValue as u16,
    ExportNamedDeclarationDeclaration(ExportNamedDeclarationWithoutDeclaration<'a, 't>) =
        AncestorType::ExportNamedDeclarationDeclaration as u16,
    ExportNamedDeclarationSpecifiers(ExportNamedDeclarationWithoutSpecifiers<'a, 't>) =
        AncestorType::ExportNamedDeclarationSpecifiers as u16,
    ExportNamedDeclarationSource(ExportNamedDeclarationWithoutSource<'a, 't>) =
        AncestorType::ExportNamedDeclarationSource as u16,
    ExportNamedDeclarationWithClause(ExportNamedDeclarationWithoutWithClause<'a, 't>) =
        AncestorType::ExportNamedDeclarationWithClause as u16,
    ExportDefaultDeclarationDeclaration(ExportDefaultDeclarationWithoutDeclaration<'a, 't>) =
        AncestorType::ExportDefaultDeclarationDeclaration as u16,
    ExportAllDeclarationExported(ExportAllDeclarationWithoutExported<'a, 't>) =
        AncestorType::ExportAllDeclarationExported as u16,
    ExportAllDeclarationSource(ExportAllDeclarationWithoutSource<'a, 't>) =
        AncestorType::ExportAllDeclarationSource as u16,
    ExportAllDeclarationWithClause(ExportAllDeclarationWithoutWithClause<'a, 't>) =
        AncestorType::ExportAllDeclarationWithClause as u16,
    ExportSpecifierLocal(ExportSpecifierWithoutLocal<'a, 't>) =
        AncestorType::ExportSpecifierLocal as u16,
    ExportSpecifierExported(ExportSpecifierWithoutExported<'a, 't>) =
        AncestorType::ExportSpecifierExported as u16,
    V8IntrinsicExpressionName(V8IntrinsicExpressionWithoutName<'a, 't>) =
        AncestorType::V8IntrinsicExpressionName as u16,
    V8IntrinsicExpressionArguments(V8IntrinsicExpressionWithoutArguments<'a, 't>) =
        AncestorType::V8IntrinsicExpressionArguments as u16,
    JSXElementOpeningElement(JSXElementWithoutOpeningElement<'a, 't>) =
        AncestorType::JSXElementOpeningElement as u16,
    JSXElementChildren(JSXElementWithoutChildren<'a, 't>) = AncestorType::JSXElementChildren as u16,
    JSXElementClosingElement(JSXElementWithoutClosingElement<'a, 't>) =
        AncestorType::JSXElementClosingElement as u16,
    JSXOpeningElementName(JSXOpeningElementWithoutName<'a, 't>) =
        AncestorType::JSXOpeningElementName as u16,
    JSXOpeningElementAttributes(JSXOpeningElementWithoutAttributes<'a, 't>) =
        AncestorType::JSXOpeningElementAttributes as u16,
    JSXClosingElementName(JSXClosingElementWithoutName<'a, 't>) =
        AncestorType::JSXClosingElementName as u16,
    JSXFragmentOpeningFragment(JSXFragmentWithoutOpeningFragment<'a, 't>) =
        AncestorType::JSXFragmentOpeningFragment as u16,
    JSXFragmentChildren(JSXFragmentWithoutChildren<'a, 't>) =
        AncestorType::JSXFragmentChildren as u16,
    JSXFragmentClosingFragment(JSXFragmentWithoutClosingFragment<'a, 't>) =
        AncestorType::JSXFragmentClosingFragment as u16,
    JSXNamespacedNameNamespace(JSXNamespacedNameWithoutNamespace<'a, 't>) =
        AncestorType::JSXNamespacedNameNamespace as u16,
    JSXNamespacedNameName(JSXNamespacedNameWithoutName<'a, 't>) =
        AncestorType::JSXNamespacedNameName as u16,
    JSXMemberExpressionObject(JSXMemberExpressionWithoutObject<'a, 't>) =
        AncestorType::JSXMemberExpressionObject as u16,
    JSXMemberExpressionProperty(JSXMemberExpressionWithoutProperty<'a, 't>) =
        AncestorType::JSXMemberExpressionProperty as u16,
    JSXExpressionContainerExpression(JSXExpressionContainerWithoutExpression<'a, 't>) =
        AncestorType::JSXExpressionContainerExpression as u16,
    JSXAttributeName(JSXAttributeWithoutName<'a, 't>) = AncestorType::JSXAttributeName as u16,
    JSXAttributeValue(JSXAttributeWithoutValue<'a, 't>) = AncestorType::JSXAttributeValue as u16,
    JSXSpreadAttributeArgument(JSXSpreadAttributeWithoutArgument<'a, 't>) =
        AncestorType::JSXSpreadAttributeArgument as u16,
    JSXSpreadChildExpression(JSXSpreadChildWithoutExpression<'a, 't>) =
        AncestorType::JSXSpreadChildExpression as u16,
    DecoratorExpression(DecoratorWithoutExpression<'a, 't>) =
        AncestorType::DecoratorExpression as u16,
}

impl<'a, 't> Ancestor<'a, 't> {
    #[inline]
    pub fn is_program(self) -> bool {
        matches!(self, Self::ProgramHashbang(_) | Self::ProgramDirectives(_) | Self::ProgramBody(_))
    }

    #[inline]
    pub fn is_array_expression(self) -> bool {
        matches!(self, Self::ArrayExpressionElements(_))
    }

    #[inline]
    pub fn is_object_expression(self) -> bool {
        matches!(self, Self::ObjectExpressionProperties(_))
    }

    #[inline]
    pub fn is_object_property(self) -> bool {
        matches!(self, Self::ObjectPropertyKey(_) | Self::ObjectPropertyValue(_))
    }

    #[inline]
    pub fn is_template_literal(self) -> bool {
        matches!(self, Self::TemplateLiteralQuasis(_) | Self::TemplateLiteralExpressions(_))
    }

    #[inline]
    pub fn is_tagged_template_expression(self) -> bool {
        matches!(
            self,
            Self::TaggedTemplateExpressionTag(_) | Self::TaggedTemplateExpressionQuasi(_)
        )
    }

    #[inline]
    pub fn is_computed_member_expression(self) -> bool {
        matches!(
            self,
            Self::ComputedMemberExpressionObject(_) | Self::ComputedMemberExpressionExpression(_)
        )
    }

    #[inline]
    pub fn is_static_member_expression(self) -> bool {
        matches!(
            self,
            Self::StaticMemberExpressionObject(_) | Self::StaticMemberExpressionProperty(_)
        )
    }

    #[inline]
    pub fn is_private_field_expression(self) -> bool {
        matches!(self, Self::PrivateFieldExpressionObject(_) | Self::PrivateFieldExpressionField(_))
    }

    #[inline]
    pub fn is_call_expression(self) -> bool {
        matches!(self, Self::CallExpressionCallee(_) | Self::CallExpressionArguments(_))
    }

    #[inline]
    pub fn is_new_expression(self) -> bool {
        matches!(self, Self::NewExpressionCallee(_) | Self::NewExpressionArguments(_))
    }

    #[inline]
    pub fn is_meta_property(self) -> bool {
        matches!(self, Self::MetaPropertyMeta(_) | Self::MetaPropertyProperty(_))
    }

    #[inline]
    pub fn is_spread_element(self) -> bool {
        matches!(self, Self::SpreadElementArgument(_))
    }

    #[inline]
    pub fn is_update_expression(self) -> bool {
        matches!(self, Self::UpdateExpressionArgument(_))
    }

    #[inline]
    pub fn is_unary_expression(self) -> bool {
        matches!(self, Self::UnaryExpressionArgument(_))
    }

    #[inline]
    pub fn is_binary_expression(self) -> bool {
        matches!(self, Self::BinaryExpressionLeft(_) | Self::BinaryExpressionRight(_))
    }

    #[inline]
    pub fn is_private_in_expression(self) -> bool {
        matches!(self, Self::PrivateInExpressionLeft(_) | Self::PrivateInExpressionRight(_))
    }

    #[inline]
    pub fn is_logical_expression(self) -> bool {
        matches!(self, Self::LogicalExpressionLeft(_) | Self::LogicalExpressionRight(_))
    }

    #[inline]
    pub fn is_conditional_expression(self) -> bool {
        matches!(
            self,
            Self::ConditionalExpressionTest(_)
                | Self::ConditionalExpressionConsequent(_)
                | Self::ConditionalExpressionAlternate(_)
        )
    }

    #[inline]
    pub fn is_assignment_expression(self) -> bool {
        matches!(self, Self::AssignmentExpressionLeft(_) | Self::AssignmentExpressionRight(_))
    }

    #[inline]
    pub fn is_array_assignment_target(self) -> bool {
        matches!(self, Self::ArrayAssignmentTargetElements(_) | Self::ArrayAssignmentTargetRest(_))
    }

    #[inline]
    pub fn is_object_assignment_target(self) -> bool {
        matches!(
            self,
            Self::ObjectAssignmentTargetProperties(_) | Self::ObjectAssignmentTargetRest(_)
        )
    }

    #[inline]
    pub fn is_assignment_target_rest(self) -> bool {
        matches!(self, Self::AssignmentTargetRestTarget(_))
    }

    #[inline]
    pub fn is_assignment_target_with_default(self) -> bool {
        matches!(
            self,
            Self::AssignmentTargetWithDefaultBinding(_) | Self::AssignmentTargetWithDefaultInit(_)
        )
    }

    #[inline]
    pub fn is_assignment_target_property_identifier(self) -> bool {
        matches!(
            self,
            Self::AssignmentTargetPropertyIdentifierBinding(_)
                | Self::AssignmentTargetPropertyIdentifierInit(_)
        )
    }

    #[inline]
    pub fn is_assignment_target_property_property(self) -> bool {
        matches!(
            self,
            Self::AssignmentTargetPropertyPropertyName(_)
                | Self::AssignmentTargetPropertyPropertyBinding(_)
        )
    }

    #[inline]
    pub fn is_sequence_expression(self) -> bool {
        matches!(self, Self::SequenceExpressionExpressions(_))
    }

    #[inline]
    pub fn is_await_expression(self) -> bool {
        matches!(self, Self::AwaitExpressionArgument(_))
    }

    #[inline]
    pub fn is_chain_expression(self) -> bool {
        matches!(self, Self::ChainExpressionExpression(_))
    }

    #[inline]
    pub fn is_parenthesized_expression(self) -> bool {
        matches!(self, Self::ParenthesizedExpressionExpression(_))
    }

    #[inline]
    pub fn is_directive(self) -> bool {
        matches!(self, Self::DirectiveExpression(_))
    }

    #[inline]
    pub fn is_block_statement(self) -> bool {
        matches!(self, Self::BlockStatementBody(_))
    }

    #[inline]
    pub fn is_variable_declaration(self) -> bool {
        matches!(self, Self::VariableDeclarationDeclarations(_))
    }

    #[inline]
    pub fn is_variable_declarator(self) -> bool {
        matches!(self, Self::VariableDeclaratorId(_) | Self::VariableDeclaratorInit(_))
    }

    #[inline]
    pub fn is_expression_statement(self) -> bool {
        matches!(self, Self::ExpressionStatementExpression(_))
    }

    #[inline]
    pub fn is_if_statement(self) -> bool {
        matches!(
            self,
            Self::IfStatementTest(_)
                | Self::IfStatementConsequent(_)
                | Self::IfStatementAlternate(_)
        )
    }

    #[inline]
    pub fn is_do_while_statement(self) -> bool {
        matches!(self, Self::DoWhileStatementBody(_) | Self::DoWhileStatementTest(_))
    }

    #[inline]
    pub fn is_while_statement(self) -> bool {
        matches!(self, Self::WhileStatementTest(_) | Self::WhileStatementBody(_))
    }

    #[inline]
    pub fn is_for_statement(self) -> bool {
        matches!(
            self,
            Self::ForStatementInit(_)
                | Self::ForStatementTest(_)
                | Self::ForStatementUpdate(_)
                | Self::ForStatementBody(_)
        )
    }

    #[inline]
    pub fn is_for_in_statement(self) -> bool {
        matches!(
            self,
            Self::ForInStatementLeft(_)
                | Self::ForInStatementRight(_)
                | Self::ForInStatementBody(_)
        )
    }

    #[inline]
    pub fn is_for_of_statement(self) -> bool {
        matches!(
            self,
            Self::ForOfStatementLeft(_)
                | Self::ForOfStatementRight(_)
                | Self::ForOfStatementBody(_)
        )
    }

    #[inline]
    pub fn is_continue_statement(self) -> bool {
        matches!(self, Self::ContinueStatementLabel(_))
    }

    #[inline]
    pub fn is_break_statement(self) -> bool {
        matches!(self, Self::BreakStatementLabel(_))
    }

    #[inline]
    pub fn is_return_statement(self) -> bool {
        matches!(self, Self::ReturnStatementArgument(_))
    }

    #[inline]
    pub fn is_with_statement(self) -> bool {
        matches!(self, Self::WithStatementObject(_) | Self::WithStatementBody(_))
    }

    #[inline]
    pub fn is_switch_statement(self) -> bool {
        matches!(self, Self::SwitchStatementDiscriminant(_) | Self::SwitchStatementCases(_))
    }

    #[inline]
    pub fn is_switch_case(self) -> bool {
        matches!(self, Self::SwitchCaseTest(_) | Self::SwitchCaseConsequent(_))
    }

    #[inline]
    pub fn is_labeled_statement(self) -> bool {
        matches!(self, Self::LabeledStatementLabel(_) | Self::LabeledStatementBody(_))
    }

    #[inline]
    pub fn is_throw_statement(self) -> bool {
        matches!(self, Self::ThrowStatementArgument(_))
    }

    #[inline]
    pub fn is_try_statement(self) -> bool {
        matches!(
            self,
            Self::TryStatementBlock(_)
                | Self::TryStatementHandler(_)
                | Self::TryStatementFinalizer(_)
        )
    }

    #[inline]
    pub fn is_catch_clause(self) -> bool {
        matches!(self, Self::CatchClauseParam(_) | Self::CatchClauseBody(_))
    }

    #[inline]
    pub fn is_catch_parameter(self) -> bool {
        matches!(self, Self::CatchParameterPattern(_))
    }

    #[inline]
    pub fn is_assignment_pattern(self) -> bool {
        matches!(self, Self::AssignmentPatternLeft(_) | Self::AssignmentPatternRight(_))
    }

    #[inline]
    pub fn is_object_pattern(self) -> bool {
        matches!(self, Self::ObjectPatternProperties(_) | Self::ObjectPatternRest(_))
    }

    #[inline]
    pub fn is_binding_property(self) -> bool {
        matches!(self, Self::BindingPropertyKey(_) | Self::BindingPropertyValue(_))
    }

    #[inline]
    pub fn is_array_pattern(self) -> bool {
        matches!(self, Self::ArrayPatternElements(_) | Self::ArrayPatternRest(_))
    }

    #[inline]
    pub fn is_binding_rest_element(self) -> bool {
        matches!(self, Self::BindingRestElementArgument(_))
    }

    #[inline]
    pub fn is_function(self) -> bool {
        matches!(self, Self::FunctionId(_) | Self::FunctionParams(_) | Self::FunctionBody(_))
    }

    #[inline]
    pub fn is_formal_parameters(self) -> bool {
        matches!(self, Self::FormalParametersItems(_) | Self::FormalParametersRest(_))
    }

    #[inline]
    pub fn is_formal_parameter(self) -> bool {
        matches!(
            self,
            Self::FormalParameterDecorators(_)
                | Self::FormalParameterPattern(_)
                | Self::FormalParameterInitializer(_)
        )
    }

    #[inline]
    pub fn is_formal_parameter_rest(self) -> bool {
        matches!(self, Self::FormalParameterRestDecorators(_) | Self::FormalParameterRestRest(_))
    }

    #[inline]
    pub fn is_function_body(self) -> bool {
        matches!(self, Self::FunctionBodyDirectives(_) | Self::FunctionBodyStatements(_))
    }

    #[inline]
    pub fn is_arrow_function_expression(self) -> bool {
        matches!(
            self,
            Self::ArrowFunctionExpressionParams(_) | Self::ArrowFunctionExpressionBody(_)
        )
    }

    #[inline]
    pub fn is_yield_expression(self) -> bool {
        matches!(self, Self::YieldExpressionArgument(_))
    }

    #[inline]
    pub fn is_class(self) -> bool {
        matches!(
            self,
            Self::ClassDecorators(_)
                | Self::ClassId(_)
                | Self::ClassSuperClass(_)
                | Self::ClassBody(_)
        )
    }

    #[inline]
    pub fn is_class_body(self) -> bool {
        matches!(self, Self::ClassBodyBody(_))
    }

    #[inline]
    pub fn is_method_definition(self) -> bool {
        matches!(
            self,
            Self::MethodDefinitionDecorators(_)
                | Self::MethodDefinitionKey(_)
                | Self::MethodDefinitionValue(_)
        )
    }

    #[inline]
    pub fn is_property_definition(self) -> bool {
        matches!(
            self,
            Self::PropertyDefinitionDecorators(_)
                | Self::PropertyDefinitionKey(_)
                | Self::PropertyDefinitionValue(_)
        )
    }

    #[inline]
    pub fn is_static_block(self) -> bool {
        matches!(self, Self::StaticBlockBody(_))
    }

    #[inline]
    pub fn is_accessor_property(self) -> bool {
        matches!(
            self,
            Self::AccessorPropertyDecorators(_)
                | Self::AccessorPropertyKey(_)
                | Self::AccessorPropertyValue(_)
        )
    }

    #[inline]
    pub fn is_import_expression(self) -> bool {
        matches!(self, Self::ImportExpressionSource(_) | Self::ImportExpressionOptions(_))
    }

    #[inline]
    pub fn is_import_declaration(self) -> bool {
        matches!(
            self,
            Self::ImportDeclarationSpecifiers(_)
                | Self::ImportDeclarationSource(_)
                | Self::ImportDeclarationWithClause(_)
        )
    }

    #[inline]
    pub fn is_import_specifier(self) -> bool {
        matches!(self, Self::ImportSpecifierImported(_) | Self::ImportSpecifierLocal(_))
    }

    #[inline]
    pub fn is_import_default_specifier(self) -> bool {
        matches!(self, Self::ImportDefaultSpecifierLocal(_))
    }

    #[inline]
    pub fn is_import_namespace_specifier(self) -> bool {
        matches!(self, Self::ImportNamespaceSpecifierLocal(_))
    }

    #[inline]
    pub fn is_with_clause(self) -> bool {
        matches!(self, Self::WithClauseWithEntries(_))
    }

    #[inline]
    pub fn is_import_attribute(self) -> bool {
        matches!(self, Self::ImportAttributeKey(_) | Self::ImportAttributeValue(_))
    }

    #[inline]
    pub fn is_export_named_declaration(self) -> bool {
        matches!(
            self,
            Self::ExportNamedDeclarationDeclaration(_)
                | Self::ExportNamedDeclarationSpecifiers(_)
                | Self::ExportNamedDeclarationSource(_)
                | Self::ExportNamedDeclarationWithClause(_)
        )
    }

    #[inline]
    pub fn is_export_default_declaration(self) -> bool {
        matches!(self, Self::ExportDefaultDeclarationDeclaration(_))
    }

    #[inline]
    pub fn is_export_all_declaration(self) -> bool {
        matches!(
            self,
            Self::ExportAllDeclarationExported(_)
                | Self::ExportAllDeclarationSource(_)
                | Self::ExportAllDeclarationWithClause(_)
        )
    }

    #[inline]
    pub fn is_export_specifier(self) -> bool {
        matches!(self, Self::ExportSpecifierLocal(_) | Self::ExportSpecifierExported(_))
    }

    #[inline]
    pub fn is_v8_intrinsic_expression(self) -> bool {
        matches!(self, Self::V8IntrinsicExpressionName(_) | Self::V8IntrinsicExpressionArguments(_))
    }

    #[inline]
    pub fn is_jsx_element(self) -> bool {
        matches!(
            self,
            Self::JSXElementOpeningElement(_)
                | Self::JSXElementChildren(_)
                | Self::JSXElementClosingElement(_)
        )
    }

    #[inline]
    pub fn is_jsx_opening_element(self) -> bool {
        matches!(self, Self::JSXOpeningElementName(_) | Self::JSXOpeningElementAttributes(_))
    }

    #[inline]
    pub fn is_jsx_closing_element(self) -> bool {
        matches!(self, Self::JSXClosingElementName(_))
    }

    #[inline]
    pub fn is_jsx_fragment(self) -> bool {
        matches!(
            self,
            Self::JSXFragmentOpeningFragment(_)
                | Self::JSXFragmentChildren(_)
                | Self::JSXFragmentClosingFragment(_)
        )
    }

    #[inline]
    pub fn is_jsx_namespaced_name(self) -> bool {
        matches!(self, Self::JSXNamespacedNameNamespace(_) | Self::JSXNamespacedNameName(_))
    }

    #[inline]
    pub fn is_jsx_member_expression(self) -> bool {
        matches!(self, Self::JSXMemberExpressionObject(_) | Self::JSXMemberExpressionProperty(_))
    }

    #[inline]
    pub fn is_jsx_expression_container(self) -> bool {
        matches!(self, Self::JSXExpressionContainerExpression(_))
    }

    #[inline]
    pub fn is_jsx_attribute(self) -> bool {
        matches!(self, Self::JSXAttributeName(_) | Self::JSXAttributeValue(_))
    }

    #[inline]
    pub fn is_jsx_spread_attribute(self) -> bool {
        matches!(self, Self::JSXSpreadAttributeArgument(_))
    }

    #[inline]
    pub fn is_jsx_spread_child(self) -> bool {
        matches!(self, Self::JSXSpreadChildExpression(_))
    }

    #[inline]
    pub fn is_decorator(self) -> bool {
        matches!(self, Self::DecoratorExpression(_))
    }

    #[inline]
    pub fn is_parent_of_statement(self) -> bool {
        matches!(
            self,
            Self::ProgramBody(_)
                | Self::BlockStatementBody(_)
                | Self::IfStatementConsequent(_)
                | Self::IfStatementAlternate(_)
                | Self::DoWhileStatementBody(_)
                | Self::WhileStatementBody(_)
                | Self::ForStatementBody(_)
                | Self::ForInStatementBody(_)
                | Self::ForOfStatementBody(_)
                | Self::WithStatementBody(_)
                | Self::SwitchCaseConsequent(_)
                | Self::LabeledStatementBody(_)
                | Self::FunctionBodyStatements(_)
                | Self::StaticBlockBody(_)
        )
    }

    #[inline]
    pub fn is_parent_of_array_expression_element(self) -> bool {
        matches!(self, Self::ArrayExpressionElements(_))
    }

    #[inline]
    pub fn is_parent_of_object_property_kind(self) -> bool {
        matches!(self, Self::ObjectExpressionProperties(_))
    }

    #[inline]
    pub fn is_parent_of_property_key(self) -> bool {
        matches!(
            self,
            Self::ObjectPropertyKey(_)
                | Self::AssignmentTargetPropertyPropertyName(_)
                | Self::BindingPropertyKey(_)
                | Self::MethodDefinitionKey(_)
                | Self::PropertyDefinitionKey(_)
                | Self::AccessorPropertyKey(_)
        )
    }

    #[inline]
    pub fn is_parent_of_expression(self) -> bool {
        matches!(
            self,
            Self::ObjectPropertyValue(_)
                | Self::TemplateLiteralExpressions(_)
                | Self::TaggedTemplateExpressionTag(_)
                | Self::ComputedMemberExpressionObject(_)
                | Self::ComputedMemberExpressionExpression(_)
                | Self::StaticMemberExpressionObject(_)
                | Self::PrivateFieldExpressionObject(_)
                | Self::CallExpressionCallee(_)
                | Self::NewExpressionCallee(_)
                | Self::SpreadElementArgument(_)
                | Self::UnaryExpressionArgument(_)
                | Self::BinaryExpressionLeft(_)
                | Self::BinaryExpressionRight(_)
                | Self::PrivateInExpressionRight(_)
                | Self::LogicalExpressionLeft(_)
                | Self::LogicalExpressionRight(_)
                | Self::ConditionalExpressionTest(_)
                | Self::ConditionalExpressionConsequent(_)
                | Self::ConditionalExpressionAlternate(_)
                | Self::AssignmentExpressionRight(_)
                | Self::AssignmentTargetWithDefaultInit(_)
                | Self::AssignmentTargetPropertyIdentifierInit(_)
                | Self::SequenceExpressionExpressions(_)
                | Self::AwaitExpressionArgument(_)
                | Self::ParenthesizedExpressionExpression(_)
                | Self::VariableDeclaratorInit(_)
                | Self::ExpressionStatementExpression(_)
                | Self::IfStatementTest(_)
                | Self::DoWhileStatementTest(_)
                | Self::WhileStatementTest(_)
                | Self::ForStatementTest(_)
                | Self::ForStatementUpdate(_)
                | Self::ForInStatementRight(_)
                | Self::ForOfStatementRight(_)
                | Self::ReturnStatementArgument(_)
                | Self::WithStatementObject(_)
                | Self::SwitchStatementDiscriminant(_)
                | Self::SwitchCaseTest(_)
                | Self::ThrowStatementArgument(_)
                | Self::AssignmentPatternRight(_)
                | Self::FormalParameterInitializer(_)
                | Self::YieldExpressionArgument(_)
                | Self::ClassSuperClass(_)
                | Self::PropertyDefinitionValue(_)
                | Self::AccessorPropertyValue(_)
                | Self::ImportExpressionSource(_)
                | Self::ImportExpressionOptions(_)
                | Self::JSXSpreadAttributeArgument(_)
                | Self::JSXSpreadChildExpression(_)
                | Self::DecoratorExpression(_)
        )
    }

    #[inline]
    pub fn is_parent_of_argument(self) -> bool {
        matches!(
            self,
            Self::CallExpressionArguments(_)
                | Self::NewExpressionArguments(_)
                | Self::V8IntrinsicExpressionArguments(_)
        )
    }

    #[inline]
    pub fn is_parent_of_simple_assignment_target(self) -> bool {
        matches!(self, Self::UpdateExpressionArgument(_))
    }

    #[inline]
    pub fn is_parent_of_assignment_target(self) -> bool {
        matches!(
            self,
            Self::AssignmentExpressionLeft(_)
                | Self::AssignmentTargetRestTarget(_)
                | Self::AssignmentTargetWithDefaultBinding(_)
        )
    }

    #[inline]
    pub fn is_parent_of_assignment_target_maybe_default(self) -> bool {
        matches!(
            self,
            Self::ArrayAssignmentTargetElements(_)
                | Self::AssignmentTargetPropertyPropertyBinding(_)
        )
    }

    #[inline]
    pub fn is_parent_of_assignment_target_property(self) -> bool {
        matches!(self, Self::ObjectAssignmentTargetProperties(_))
    }

    #[inline]
    pub fn is_parent_of_chain_element(self) -> bool {
        matches!(self, Self::ChainExpressionExpression(_))
    }

    #[inline]
    pub fn is_parent_of_binding_pattern(self) -> bool {
        matches!(
            self,
            Self::VariableDeclaratorId(_)
                | Self::CatchParameterPattern(_)
                | Self::AssignmentPatternLeft(_)
                | Self::BindingPropertyValue(_)
                | Self::ArrayPatternElements(_)
                | Self::BindingRestElementArgument(_)
                | Self::FormalParameterPattern(_)
        )
    }

    #[inline]
    pub fn is_parent_of_for_statement_init(self) -> bool {
        matches!(self, Self::ForStatementInit(_))
    }

    #[inline]
    pub fn is_parent_of_for_statement_left(self) -> bool {
        matches!(self, Self::ForInStatementLeft(_) | Self::ForOfStatementLeft(_))
    }

    #[inline]
    pub fn is_parent_of_class_element(self) -> bool {
        matches!(self, Self::ClassBodyBody(_))
    }

    #[inline]
    pub fn is_parent_of_import_declaration_specifier(self) -> bool {
        matches!(self, Self::ImportDeclarationSpecifiers(_))
    }

    #[inline]
    pub fn is_parent_of_module_export_name(self) -> bool {
        matches!(
            self,
            Self::ImportSpecifierImported(_)
                | Self::ExportAllDeclarationExported(_)
                | Self::ExportSpecifierLocal(_)
                | Self::ExportSpecifierExported(_)
        )
    }

    #[inline]
    pub fn is_parent_of_import_attribute_key(self) -> bool {
        matches!(self, Self::ImportAttributeKey(_))
    }

    #[inline]
    pub fn is_parent_of_declaration(self) -> bool {
        matches!(self, Self::ExportNamedDeclarationDeclaration(_))
    }

    #[inline]
    pub fn is_parent_of_export_default_declaration_kind(self) -> bool {
        matches!(self, Self::ExportDefaultDeclarationDeclaration(_))
    }

    #[inline]
    pub fn is_parent_of_jsx_child(self) -> bool {
        matches!(self, Self::JSXElementChildren(_) | Self::JSXFragmentChildren(_))
    }

    #[inline]
    pub fn is_parent_of_jsx_element_name(self) -> bool {
        matches!(self, Self::JSXOpeningElementName(_) | Self::JSXClosingElementName(_))
    }

    #[inline]
    pub fn is_parent_of_jsx_attribute_item(self) -> bool {
        matches!(self, Self::JSXOpeningElementAttributes(_))
    }

    #[inline]
    pub fn is_parent_of_jsx_member_expression_object(self) -> bool {
        matches!(self, Self::JSXMemberExpressionObject(_))
    }

    #[inline]
    pub fn is_parent_of_jsx_expression(self) -> bool {
        matches!(self, Self::JSXExpressionContainerExpression(_))
    }

    #[inline]
    pub fn is_parent_of_jsx_attribute_name(self) -> bool {
        matches!(self, Self::JSXAttributeName(_))
    }

    #[inline]
    pub fn is_parent_of_jsx_attribute_value(self) -> bool {
        matches!(self, Self::JSXAttributeValue(_))
    }
}

impl<'a, 't> GetAddress for Ancestor<'a, 't> {
    /// Get memory address of node represented by `Ancestor` in the arena.
    // Compiler should reduce this down to only a couple of assembly operations.
    #[inline]
    fn address(&self) -> Address {
        match self {
            Self::None => Address::DUMMY,
            Self::ProgramHashbang(a) => a.address(),
            Self::ProgramDirectives(a) => a.address(),
            Self::ProgramBody(a) => a.address(),
            Self::ArrayExpressionElements(a) => a.address(),
            Self::ObjectExpressionProperties(a) => a.address(),
            Self::ObjectPropertyKey(a) => a.address(),
            Self::ObjectPropertyValue(a) => a.address(),
            Self::TemplateLiteralQuasis(a) => a.address(),
            Self::TemplateLiteralExpressions(a) => a.address(),
            Self::TaggedTemplateExpressionTag(a) => a.address(),
            Self::TaggedTemplateExpressionQuasi(a) => a.address(),
            Self::ComputedMemberExpressionObject(a) => a.address(),
            Self::ComputedMemberExpressionExpression(a) => a.address(),
            Self::StaticMemberExpressionObject(a) => a.address(),
            Self::StaticMemberExpressionProperty(a) => a.address(),
            Self::PrivateFieldExpressionObject(a) => a.address(),
            Self::PrivateFieldExpressionField(a) => a.address(),
            Self::CallExpressionCallee(a) => a.address(),
            Self::CallExpressionArguments(a) => a.address(),
            Self::NewExpressionCallee(a) => a.address(),
            Self::NewExpressionArguments(a) => a.address(),
            Self::MetaPropertyMeta(a) => a.address(),
            Self::MetaPropertyProperty(a) => a.address(),
            Self::SpreadElementArgument(a) => a.address(),
            Self::UpdateExpressionArgument(a) => a.address(),
            Self::UnaryExpressionArgument(a) => a.address(),
            Self::BinaryExpressionLeft(a) => a.address(),
            Self::BinaryExpressionRight(a) => a.address(),
            Self::PrivateInExpressionLeft(a) => a.address(),
            Self::PrivateInExpressionRight(a) => a.address(),
            Self::LogicalExpressionLeft(a) => a.address(),
            Self::LogicalExpressionRight(a) => a.address(),
            Self::ConditionalExpressionTest(a) => a.address(),
            Self::ConditionalExpressionConsequent(a) => a.address(),
            Self::ConditionalExpressionAlternate(a) => a.address(),
            Self::AssignmentExpressionLeft(a) => a.address(),
            Self::AssignmentExpressionRight(a) => a.address(),
            Self::ArrayAssignmentTargetElements(a) => a.address(),
            Self::ArrayAssignmentTargetRest(a) => a.address(),
            Self::ObjectAssignmentTargetProperties(a) => a.address(),
            Self::ObjectAssignmentTargetRest(a) => a.address(),
            Self::AssignmentTargetRestTarget(a) => a.address(),
            Self::AssignmentTargetWithDefaultBinding(a) => a.address(),
            Self::AssignmentTargetWithDefaultInit(a) => a.address(),
            Self::AssignmentTargetPropertyIdentifierBinding(a) => a.address(),
            Self::AssignmentTargetPropertyIdentifierInit(a) => a.address(),
            Self::AssignmentTargetPropertyPropertyName(a) => a.address(),
            Self::AssignmentTargetPropertyPropertyBinding(a) => a.address(),
            Self::SequenceExpressionExpressions(a) => a.address(),
            Self::AwaitExpressionArgument(a) => a.address(),
            Self::ChainExpressionExpression(a) => a.address(),
            Self::ParenthesizedExpressionExpression(a) => a.address(),
            Self::DirectiveExpression(a) => a.address(),
            Self::BlockStatementBody(a) => a.address(),
            Self::VariableDeclarationDeclarations(a) => a.address(),
            Self::VariableDeclaratorId(a) => a.address(),
            Self::VariableDeclaratorInit(a) => a.address(),
            Self::ExpressionStatementExpression(a) => a.address(),
            Self::IfStatementTest(a) => a.address(),
            Self::IfStatementConsequent(a) => a.address(),
            Self::IfStatementAlternate(a) => a.address(),
            Self::DoWhileStatementBody(a) => a.address(),
            Self::DoWhileStatementTest(a) => a.address(),
            Self::WhileStatementTest(a) => a.address(),
            Self::WhileStatementBody(a) => a.address(),
            Self::ForStatementInit(a) => a.address(),
            Self::ForStatementTest(a) => a.address(),
            Self::ForStatementUpdate(a) => a.address(),
            Self::ForStatementBody(a) => a.address(),
            Self::ForInStatementLeft(a) => a.address(),
            Self::ForInStatementRight(a) => a.address(),
            Self::ForInStatementBody(a) => a.address(),
            Self::ForOfStatementLeft(a) => a.address(),
            Self::ForOfStatementRight(a) => a.address(),
            Self::ForOfStatementBody(a) => a.address(),
            Self::ContinueStatementLabel(a) => a.address(),
            Self::BreakStatementLabel(a) => a.address(),
            Self::ReturnStatementArgument(a) => a.address(),
            Self::WithStatementObject(a) => a.address(),
            Self::WithStatementBody(a) => a.address(),
            Self::SwitchStatementDiscriminant(a) => a.address(),
            Self::SwitchStatementCases(a) => a.address(),
            Self::SwitchCaseTest(a) => a.address(),
            Self::SwitchCaseConsequent(a) => a.address(),
            Self::LabeledStatementLabel(a) => a.address(),
            Self::LabeledStatementBody(a) => a.address(),
            Self::ThrowStatementArgument(a) => a.address(),
            Self::TryStatementBlock(a) => a.address(),
            Self::TryStatementHandler(a) => a.address(),
            Self::TryStatementFinalizer(a) => a.address(),
            Self::CatchClauseParam(a) => a.address(),
            Self::CatchClauseBody(a) => a.address(),
            Self::CatchParameterPattern(a) => a.address(),
            Self::AssignmentPatternLeft(a) => a.address(),
            Self::AssignmentPatternRight(a) => a.address(),
            Self::ObjectPatternProperties(a) => a.address(),
            Self::ObjectPatternRest(a) => a.address(),
            Self::BindingPropertyKey(a) => a.address(),
            Self::BindingPropertyValue(a) => a.address(),
            Self::ArrayPatternElements(a) => a.address(),
            Self::ArrayPatternRest(a) => a.address(),
            Self::BindingRestElementArgument(a) => a.address(),
            Self::FunctionId(a) => a.address(),
            Self::FunctionParams(a) => a.address(),
            Self::FunctionBody(a) => a.address(),
            Self::FormalParametersItems(a) => a.address(),
            Self::FormalParametersRest(a) => a.address(),
            Self::FormalParameterDecorators(a) => a.address(),
            Self::FormalParameterPattern(a) => a.address(),
            Self::FormalParameterInitializer(a) => a.address(),
            Self::FormalParameterRestDecorators(a) => a.address(),
            Self::FormalParameterRestRest(a) => a.address(),
            Self::FunctionBodyDirectives(a) => a.address(),
            Self::FunctionBodyStatements(a) => a.address(),
            Self::ArrowFunctionExpressionParams(a) => a.address(),
            Self::ArrowFunctionExpressionBody(a) => a.address(),
            Self::YieldExpressionArgument(a) => a.address(),
            Self::ClassDecorators(a) => a.address(),
            Self::ClassId(a) => a.address(),
            Self::ClassSuperClass(a) => a.address(),
            Self::ClassBody(a) => a.address(),
            Self::ClassBodyBody(a) => a.address(),
            Self::MethodDefinitionDecorators(a) => a.address(),
            Self::MethodDefinitionKey(a) => a.address(),
            Self::MethodDefinitionValue(a) => a.address(),
            Self::PropertyDefinitionDecorators(a) => a.address(),
            Self::PropertyDefinitionKey(a) => a.address(),
            Self::PropertyDefinitionValue(a) => a.address(),
            Self::StaticBlockBody(a) => a.address(),
            Self::AccessorPropertyDecorators(a) => a.address(),
            Self::AccessorPropertyKey(a) => a.address(),
            Self::AccessorPropertyValue(a) => a.address(),
            Self::ImportExpressionSource(a) => a.address(),
            Self::ImportExpressionOptions(a) => a.address(),
            Self::ImportDeclarationSpecifiers(a) => a.address(),
            Self::ImportDeclarationSource(a) => a.address(),
            Self::ImportDeclarationWithClause(a) => a.address(),
            Self::ImportSpecifierImported(a) => a.address(),
            Self::ImportSpecifierLocal(a) => a.address(),
            Self::ImportDefaultSpecifierLocal(a) => a.address(),
            Self::ImportNamespaceSpecifierLocal(a) => a.address(),
            Self::WithClauseWithEntries(a) => a.address(),
            Self::ImportAttributeKey(a) => a.address(),
            Self::ImportAttributeValue(a) => a.address(),
            Self::ExportNamedDeclarationDeclaration(a) => a.address(),
            Self::ExportNamedDeclarationSpecifiers(a) => a.address(),
            Self::ExportNamedDeclarationSource(a) => a.address(),
            Self::ExportNamedDeclarationWithClause(a) => a.address(),
            Self::ExportDefaultDeclarationDeclaration(a) => a.address(),
            Self::ExportAllDeclarationExported(a) => a.address(),
            Self::ExportAllDeclarationSource(a) => a.address(),
            Self::ExportAllDeclarationWithClause(a) => a.address(),
            Self::ExportSpecifierLocal(a) => a.address(),
            Self::ExportSpecifierExported(a) => a.address(),
            Self::V8IntrinsicExpressionName(a) => a.address(),
            Self::V8IntrinsicExpressionArguments(a) => a.address(),
            Self::JSXElementOpeningElement(a) => a.address(),
            Self::JSXElementChildren(a) => a.address(),
            Self::JSXElementClosingElement(a) => a.address(),
            Self::JSXOpeningElementName(a) => a.address(),
            Self::JSXOpeningElementAttributes(a) => a.address(),
            Self::JSXClosingElementName(a) => a.address(),
            Self::JSXFragmentOpeningFragment(a) => a.address(),
            Self::JSXFragmentChildren(a) => a.address(),
            Self::JSXFragmentClosingFragment(a) => a.address(),
            Self::JSXNamespacedNameNamespace(a) => a.address(),
            Self::JSXNamespacedNameName(a) => a.address(),
            Self::JSXMemberExpressionObject(a) => a.address(),
            Self::JSXMemberExpressionProperty(a) => a.address(),
            Self::JSXExpressionContainerExpression(a) => a.address(),
            Self::JSXAttributeName(a) => a.address(),
            Self::JSXAttributeValue(a) => a.address(),
            Self::JSXSpreadAttributeArgument(a) => a.address(),
            Self::JSXSpreadChildExpression(a) => a.address(),
            Self::DecoratorExpression(a) => a.address(),
        }
    }
}

pub(crate) const OFFSET_PROGRAM_NODE_ID: usize = offset_of!(Program, node_id);
pub(crate) const OFFSET_PROGRAM_SPAN: usize = offset_of!(Program, span);
pub(crate) const OFFSET_PROGRAM_SOURCE_TYPE: usize = offset_of!(Program, source_type);
pub(crate) const OFFSET_PROGRAM_SOURCE_TEXT: usize = offset_of!(Program, source_text);
pub(crate) const OFFSET_PROGRAM_COMMENTS: usize = offset_of!(Program, comments);
pub(crate) const OFFSET_PROGRAM_HASHBANG: usize = offset_of!(Program, hashbang);
pub(crate) const OFFSET_PROGRAM_DIRECTIVES: usize = offset_of!(Program, directives);
pub(crate) const OFFSET_PROGRAM_BODY: usize = offset_of!(Program, body);
pub(crate) const OFFSET_PROGRAM_SCOPE_ID: usize = offset_of!(Program, scope_id);

#[repr(transparent)]
#[derive(Clone, Copy, Debug)]
pub struct ProgramWithoutHashbang<'a, 't>(
    pub(crate) *const Program<'a>,
    pub(crate) PhantomData<&'t ()>,
);

impl<'a, 't> ProgramWithoutHashbang<'a, 't> {
    #[inline]
    pub fn node_id(self) -> &'t Cell<NodeId> {
        unsafe { &*((self.0 as *const u8).add(OFFSET_PROGRAM_NODE_ID) as *const Cell<NodeId>) }
    }

    #[inline]
    pub fn span(self) -> &'t Span {
        unsafe { &*((self.0 as *const u8).add(OFFSET_PROGRAM_SPAN) as *const Span) }
    }

    #[inline]
    pub fn source_type(self) -> &'t SourceType {
        unsafe { &*((self.0 as *const u8).add(OFFSET_PROGRAM_SOURCE_TYPE) as *const SourceType) }
    }

    #[inline]
    pub fn source_text(self) -> &'t &'a str {
        unsafe { &*((self.0 as *const u8).add(OFFSET_PROGRAM_SOURCE_TEXT) as *const &'a str) }
    }

    #[inline]
    pub fn comments(self) -> &'t ArenaVec<'a, Comment> {
        unsafe {
            &*((self.0 as *const u8).add(OFFSET_PROGRAM_COMMENTS) as *const ArenaVec<'a, Comment>)
        }
    }

    #[inline]
    pub fn directives(self) -> &'t ArenaVec<'a, Directive<'a>> {
        unsafe {
            &*((self.0 as *const u8).add(OFFSET_PROGRAM_DIRECTIVES)
                as *const ArenaVec<'a, Directive<'a>>)
        }
    }

    #[inline]
    pub fn body(self) -> &'t ArenaVec<'a, Statement<'a>> {
        unsafe {
            &*((self.0 as *const u8).add(OFFSET_PROGRAM_BODY) as *const ArenaVec<'a, Statement<'a>>)
        }
    }

    #[inline]
    pub fn scope_id(self) -> &'t Cell<Option<ScopeId>> {
        unsafe {
            &*((self.0 as *const u8).add(OFFSET_PROGRAM_SCOPE_ID) as *const Cell<Option<ScopeId>>)
        }
    }
}

impl<'a, 't> GetAddress for ProgramWithoutHashbang<'a, 't> {
    #[inline]
    fn address(&self) -> Address {
        unsafe { Address::from_ptr(self.0) }
    }
}

#[repr(transparent)]
#[derive(Clone, Copy, Debug)]
pub struct ProgramWithoutDirectives<'a, 't>(
    pub(crate) *const Program<'a>,
    pub(crate) PhantomData<&'t ()>,
);

impl<'a, 't> ProgramWithoutDirectives<'a, 't> {
    #[inline]
    pub fn node_id(self) -> &'t Cell<NodeId> {
        unsafe { &*((self.0 as *const u8).add(OFFSET_PROGRAM_NODE_ID) as *const Cell<NodeId>) }
    }

    #[inline]
    pub fn span(self) -> &'t Span {
        unsafe { &*((self.0 as *const u8).add(OFFSET_PROGRAM_SPAN) as *const Span) }
    }

    #[inline]
    pub fn source_type(self) -> &'t SourceType {
        unsafe { &*((self.0 as *const u8).add(OFFSET_PROGRAM_SOURCE_TYPE) as *const SourceType) }
    }

    #[inline]
    pub fn source_text(self) -> &'t &'a str {
        unsafe { &*((self.0 as *const u8).add(OFFSET_PROGRAM_SOURCE_TEXT) as *const &'a str) }
    }

    #[inline]
    pub fn comments(self) -> &'t ArenaVec<'a, Comment> {
        unsafe {
            &*((self.0 as *const u8).add(OFFSET_PROGRAM_COMMENTS) as *const ArenaVec<'a, Comment>)
        }
    }

    #[inline]
    pub fn hashbang(self) -> &'t Option<Hashbang<'a>> {
        unsafe {
            &*((self.0 as *const u8).add(OFFSET_PROGRAM_HASHBANG) as *const Option<Hashbang<'a>>)
        }
    }

    #[inline]
    pub fn body(self) -> &'t ArenaVec<'a, Statement<'a>> {
        unsafe {
            &*((self.0 as *const u8).add(OFFSET_PROGRAM_BODY) as *const ArenaVec<'a, Statement<'a>>)
        }
    }

    #[inline]
    pub fn scope_id(self) -> &'t Cell<Option<ScopeId>> {
        unsafe {
            &*((self.0 as *const u8).add(OFFSET_PROGRAM_SCOPE_ID) as *const Cell<Option<ScopeId>>)
        }
    }
}

impl<'a, 't> GetAddress for ProgramWithoutDirectives<'a, 't> {
    #[inline]
    fn address(&self) -> Address {
        unsafe { Address::from_ptr(self.0) }
    }
}

#[repr(transparent)]
#[derive(Clone, Copy, Debug)]
pub struct ProgramWithoutBody<'a, 't>(
    pub(crate) *const Program<'a>,
    pub(crate) PhantomData<&'t ()>,
);

impl<'a, 't> ProgramWithoutBody<'a, 't> {
    #[inline]
    pub fn node_id(self) -> &'t Cell<NodeId> {
        unsafe { &*((self.0 as *const u8).add(OFFSET_PROGRAM_NODE_ID) as *const Cell<NodeId>) }
    }

    #[inline]
    pub fn span(self) -> &'t Span {
        unsafe { &*((self.0 as *const u8).add(OFFSET_PROGRAM_SPAN) as *const Span) }
    }

    #[inline]
    pub fn source_type(self) -> &'t SourceType {
        unsafe { &*((self.0 as *const u8).add(OFFSET_PROGRAM_SOURCE_TYPE) as *const SourceType) }
    }

    #[inline]
    pub fn source_text(self) -> &'t &'a str {
        unsafe { &*((self.0 as *const u8).add(OFFSET_PROGRAM_SOURCE_TEXT) as *const &'a str) }
    }

    #[inline]
    pub fn comments(self) -> &'t ArenaVec<'a, Comment> {
        unsafe {
            &*((self.0 as *const u8).add(OFFSET_PROGRAM_COMMENTS) as *const ArenaVec<'a, Comment>)
        }
    }

    #[inline]
    pub fn hashbang(self) -> &'t Option<Hashbang<'a>> {
        unsafe {
            &*((self.0 as *const u8).add(OFFSET_PROGRAM_HASHBANG) as *const Option<Hashbang<'a>>)
        }
    }

    #[inline]
    pub fn directives(self) -> &'t ArenaVec<'a, Directive<'a>> {
        unsafe {
            &*((self.0 as *const u8).add(OFFSET_PROGRAM_DIRECTIVES)
                as *const ArenaVec<'a, Directive<'a>>)
        }
    }

    #[inline]
    pub fn scope_id(self) -> &'t Cell<Option<ScopeId>> {
        unsafe {
            &*((self.0 as *const u8).add(OFFSET_PROGRAM_SCOPE_ID) as *const Cell<Option<ScopeId>>)
        }
    }
}

impl<'a, 't> GetAddress for ProgramWithoutBody<'a, 't> {
    #[inline]
    fn address(&self) -> Address {
        unsafe { Address::from_ptr(self.0) }
    }
}

pub(crate) const OFFSET_ARRAY_EXPRESSION_NODE_ID: usize = offset_of!(ArrayExpression, node_id);
pub(crate) const OFFSET_ARRAY_EXPRESSION_SPAN: usize = offset_of!(ArrayExpression, span);
pub(crate) const OFFSET_ARRAY_EXPRESSION_ELEMENTS: usize = offset_of!(ArrayExpression, elements);

#[repr(transparent)]
#[derive(Clone, Copy, Debug)]
pub struct ArrayExpressionWithoutElements<'a, 't>(
    pub(crate) *const ArrayExpression<'a>,
    pub(crate) PhantomData<&'t ()>,
);

impl<'a, 't> ArrayExpressionWithoutElements<'a, 't> {
    #[inline]
    pub fn node_id(self) -> &'t Cell<NodeId> {
        unsafe {
            &*((self.0 as *const u8).add(OFFSET_ARRAY_EXPRESSION_NODE_ID) as *const Cell<NodeId>)
        }
    }

    #[inline]
    pub fn span(self) -> &'t Span {
        unsafe { &*((self.0 as *const u8).add(OFFSET_ARRAY_EXPRESSION_SPAN) as *const Span) }
    }
}

impl<'a, 't> GetAddress for ArrayExpressionWithoutElements<'a, 't> {
    #[inline]
    fn address(&self) -> Address {
        unsafe { Address::from_ptr(self.0) }
    }
}

pub(crate) const OFFSET_OBJECT_EXPRESSION_NODE_ID: usize = offset_of!(ObjectExpression, node_id);
pub(crate) const OFFSET_OBJECT_EXPRESSION_SPAN: usize = offset_of!(ObjectExpression, span);
pub(crate) const OFFSET_OBJECT_EXPRESSION_PROPERTIES: usize =
    offset_of!(ObjectExpression, properties);

#[repr(transparent)]
#[derive(Clone, Copy, Debug)]
pub struct ObjectExpressionWithoutProperties<'a, 't>(
    pub(crate) *const ObjectExpression<'a>,
    pub(crate) PhantomData<&'t ()>,
);

impl<'a, 't> ObjectExpressionWithoutProperties<'a, 't> {
    #[inline]
    pub fn node_id(self) -> &'t Cell<NodeId> {
        unsafe {
            &*((self.0 as *const u8).add(OFFSET_OBJECT_EXPRESSION_NODE_ID) as *const Cell<NodeId>)
        }
    }

    #[inline]
    pub fn span(self) -> &'t Span {
        unsafe { &*((self.0 as *const u8).add(OFFSET_OBJECT_EXPRESSION_SPAN) as *const Span) }
    }
}

impl<'a, 't> GetAddress for ObjectExpressionWithoutProperties<'a, 't> {
    #[inline]
    fn address(&self) -> Address {
        unsafe { Address::from_ptr(self.0) }
    }
}

pub(crate) const OFFSET_OBJECT_PROPERTY_NODE_ID: usize = offset_of!(ObjectProperty, node_id);
pub(crate) const OFFSET_OBJECT_PROPERTY_SPAN: usize = offset_of!(ObjectProperty, span);
pub(crate) const OFFSET_OBJECT_PROPERTY_KIND: usize = offset_of!(ObjectProperty, kind);
pub(crate) const OFFSET_OBJECT_PROPERTY_KEY: usize = offset_of!(ObjectProperty, key);
pub(crate) const OFFSET_OBJECT_PROPERTY_VALUE: usize = offset_of!(ObjectProperty, value);
pub(crate) const OFFSET_OBJECT_PROPERTY_METHOD: usize = offset_of!(ObjectProperty, method);
pub(crate) const OFFSET_OBJECT_PROPERTY_SHORTHAND: usize = offset_of!(ObjectProperty, shorthand);
pub(crate) const OFFSET_OBJECT_PROPERTY_COMPUTED: usize = offset_of!(ObjectProperty, computed);

#[repr(transparent)]
#[derive(Clone, Copy, Debug)]
pub struct ObjectPropertyWithoutKey<'a, 't>(
    pub(crate) *const ObjectProperty<'a>,
    pub(crate) PhantomData<&'t ()>,
);

impl<'a, 't> ObjectPropertyWithoutKey<'a, 't> {
    #[inline]
    pub fn node_id(self) -> &'t Cell<NodeId> {
        unsafe {
            &*((self.0 as *const u8).add(OFFSET_OBJECT_PROPERTY_NODE_ID) as *const Cell<NodeId>)
        }
    }

    #[inline]
    pub fn span(self) -> &'t Span {
        unsafe { &*((self.0 as *const u8).add(OFFSET_OBJECT_PROPERTY_SPAN) as *const Span) }
    }

    #[inline]
    pub fn kind(self) -> &'t PropertyKind {
        unsafe { &*((self.0 as *const u8).add(OFFSET_OBJECT_PROPERTY_KIND) as *const PropertyKind) }
    }

    #[inline]
    pub fn value(self) -> &'t Expression<'a> {
        unsafe {
            &*((self.0 as *const u8).add(OFFSET_OBJECT_PROPERTY_VALUE) as *const Expression<'a>)
        }
    }

    #[inline]
    pub fn method(self) -> &'t bool {
        unsafe { &*((self.0 as *const u8).add(OFFSET_OBJECT_PROPERTY_METHOD) as *const bool) }
    }

    #[inline]
    pub fn shorthand(self) -> &'t bool {
        unsafe { &*((self.0 as *const u8).add(OFFSET_OBJECT_PROPERTY_SHORTHAND) as *const bool) }
    }

    #[inline]
    pub fn computed(self) -> &'t bool {
        unsafe { &*((self.0 as *const u8).add(OFFSET_OBJECT_PROPERTY_COMPUTED) as *const bool) }
    }
}

impl<'a, 't> GetAddress for ObjectPropertyWithoutKey<'a, 't> {
    #[inline]
    fn address(&self) -> Address {
        unsafe { Address::from_ptr(self.0) }
    }
}

#[repr(transparent)]
#[derive(Clone, Copy, Debug)]
pub struct ObjectPropertyWithoutValue<'a, 't>(
    pub(crate) *const ObjectProperty<'a>,
    pub(crate) PhantomData<&'t ()>,
);

impl<'a, 't> ObjectPropertyWithoutValue<'a, 't> {
    #[inline]
    pub fn node_id(self) -> &'t Cell<NodeId> {
        unsafe {
            &*((self.0 as *const u8).add(OFFSET_OBJECT_PROPERTY_NODE_ID) as *const Cell<NodeId>)
        }
    }

    #[inline]
    pub fn span(self) -> &'t Span {
        unsafe { &*((self.0 as *const u8).add(OFFSET_OBJECT_PROPERTY_SPAN) as *const Span) }
    }

    #[inline]
    pub fn kind(self) -> &'t PropertyKind {
        unsafe { &*((self.0 as *const u8).add(OFFSET_OBJECT_PROPERTY_KIND) as *const PropertyKind) }
    }

    #[inline]
    pub fn key(self) -> &'t PropertyKey<'a> {
        unsafe {
            &*((self.0 as *const u8).add(OFFSET_OBJECT_PROPERTY_KEY) as *const PropertyKey<'a>)
        }
    }

    #[inline]
    pub fn method(self) -> &'t bool {
        unsafe { &*((self.0 as *const u8).add(OFFSET_OBJECT_PROPERTY_METHOD) as *const bool) }
    }

    #[inline]
    pub fn shorthand(self) -> &'t bool {
        unsafe { &*((self.0 as *const u8).add(OFFSET_OBJECT_PROPERTY_SHORTHAND) as *const bool) }
    }

    #[inline]
    pub fn computed(self) -> &'t bool {
        unsafe { &*((self.0 as *const u8).add(OFFSET_OBJECT_PROPERTY_COMPUTED) as *const bool) }
    }
}

impl<'a, 't> GetAddress for ObjectPropertyWithoutValue<'a, 't> {
    #[inline]
    fn address(&self) -> Address {
        unsafe { Address::from_ptr(self.0) }
    }
}

pub(crate) const OFFSET_TEMPLATE_LITERAL_NODE_ID: usize = offset_of!(TemplateLiteral, node_id);
pub(crate) const OFFSET_TEMPLATE_LITERAL_SPAN: usize = offset_of!(TemplateLiteral, span);
pub(crate) const OFFSET_TEMPLATE_LITERAL_QUASIS: usize = offset_of!(TemplateLiteral, quasis);
pub(crate) const OFFSET_TEMPLATE_LITERAL_EXPRESSIONS: usize =
    offset_of!(TemplateLiteral, expressions);

#[repr(transparent)]
#[derive(Clone, Copy, Debug)]
pub struct TemplateLiteralWithoutQuasis<'a, 't>(
    pub(crate) *const TemplateLiteral<'a>,
    pub(crate) PhantomData<&'t ()>,
);

impl<'a, 't> TemplateLiteralWithoutQuasis<'a, 't> {
    #[inline]
    pub fn node_id(self) -> &'t Cell<NodeId> {
        unsafe {
            &*((self.0 as *const u8).add(OFFSET_TEMPLATE_LITERAL_NODE_ID) as *const Cell<NodeId>)
        }
    }

    #[inline]
    pub fn span(self) -> &'t Span {
        unsafe { &*((self.0 as *const u8).add(OFFSET_TEMPLATE_LITERAL_SPAN) as *const Span) }
    }

    #[inline]
    pub fn expressions(self) -> &'t ArenaVec<'a, Expression<'a>> {
        unsafe {
            &*((self.0 as *const u8).add(OFFSET_TEMPLATE_LITERAL_EXPRESSIONS)
                as *const ArenaVec<'a, Expression<'a>>)
        }
    }
}

impl<'a, 't> GetAddress for TemplateLiteralWithoutQuasis<'a, 't> {
    #[inline]
    fn address(&self) -> Address {
        unsafe { Address::from_ptr(self.0) }
    }
}

#[repr(transparent)]
#[derive(Clone, Copy, Debug)]
pub struct TemplateLiteralWithoutExpressions<'a, 't>(
    pub(crate) *const TemplateLiteral<'a>,
    pub(crate) PhantomData<&'t ()>,
);

impl<'a, 't> TemplateLiteralWithoutExpressions<'a, 't> {
    #[inline]
    pub fn node_id(self) -> &'t Cell<NodeId> {
        unsafe {
            &*((self.0 as *const u8).add(OFFSET_TEMPLATE_LITERAL_NODE_ID) as *const Cell<NodeId>)
        }
    }

    #[inline]
    pub fn span(self) -> &'t Span {
        unsafe { &*((self.0 as *const u8).add(OFFSET_TEMPLATE_LITERAL_SPAN) as *const Span) }
    }

    #[inline]
    pub fn quasis(self) -> &'t ArenaVec<'a, TemplateElement<'a>> {
        unsafe {
            &*((self.0 as *const u8).add(OFFSET_TEMPLATE_LITERAL_QUASIS)
                as *const ArenaVec<'a, TemplateElement<'a>>)
        }
    }
}

impl<'a, 't> GetAddress for TemplateLiteralWithoutExpressions<'a, 't> {
    #[inline]
    fn address(&self) -> Address {
        unsafe { Address::from_ptr(self.0) }
    }
}

pub(crate) const OFFSET_TAGGED_TEMPLATE_EXPRESSION_NODE_ID: usize =
    offset_of!(TaggedTemplateExpression, node_id);
pub(crate) const OFFSET_TAGGED_TEMPLATE_EXPRESSION_SPAN: usize =
    offset_of!(TaggedTemplateExpression, span);
pub(crate) const OFFSET_TAGGED_TEMPLATE_EXPRESSION_TAG: usize =
    offset_of!(TaggedTemplateExpression, tag);
pub(crate) const OFFSET_TAGGED_TEMPLATE_EXPRESSION_TYPE_ARGUMENTS: usize =
    offset_of!(TaggedTemplateExpression, type_arguments);
pub(crate) const OFFSET_TAGGED_TEMPLATE_EXPRESSION_QUASI: usize =
    offset_of!(TaggedTemplateExpression, quasi);

#[repr(transparent)]
#[derive(Clone, Copy, Debug)]
pub struct TaggedTemplateExpressionWithoutTag<'a, 't>(
    pub(crate) *const TaggedTemplateExpression<'a>,
    pub(crate) PhantomData<&'t ()>,
);

impl<'a, 't> TaggedTemplateExpressionWithoutTag<'a, 't> {
    #[inline]
    pub fn node_id(self) -> &'t Cell<NodeId> {
        unsafe {
            &*((self.0 as *const u8).add(OFFSET_TAGGED_TEMPLATE_EXPRESSION_NODE_ID)
                as *const Cell<NodeId>)
        }
    }

    #[inline]
    pub fn span(self) -> &'t Span {
        unsafe {
            &*((self.0 as *const u8).add(OFFSET_TAGGED_TEMPLATE_EXPRESSION_SPAN) as *const Span)
        }
    }

    #[inline]
    pub fn type_arguments(self) -> &'t Option<ArenaBox<'a, TSTypeParameterInstantiation<'a>>> {
        unsafe {
            &*((self.0 as *const u8).add(OFFSET_TAGGED_TEMPLATE_EXPRESSION_TYPE_ARGUMENTS)
                as *const Option<ArenaBox<'a, TSTypeParameterInstantiation<'a>>>)
        }
    }

    #[inline]
    pub fn quasi(self) -> &'t TemplateLiteral<'a> {
        unsafe {
            &*((self.0 as *const u8).add(OFFSET_TAGGED_TEMPLATE_EXPRESSION_QUASI)
                as *const TemplateLiteral<'a>)
        }
    }
}

impl<'a, 't> GetAddress for TaggedTemplateExpressionWithoutTag<'a, 't> {
    #[inline]
    fn address(&self) -> Address {
        unsafe { Address::from_ptr(self.0) }
    }
}

#[repr(transparent)]
#[derive(Clone, Copy, Debug)]
pub struct TaggedTemplateExpressionWithoutQuasi<'a, 't>(
    pub(crate) *const TaggedTemplateExpression<'a>,
    pub(crate) PhantomData<&'t ()>,
);

impl<'a, 't> TaggedTemplateExpressionWithoutQuasi<'a, 't> {
    #[inline]
    pub fn node_id(self) -> &'t Cell<NodeId> {
        unsafe {
            &*((self.0 as *const u8).add(OFFSET_TAGGED_TEMPLATE_EXPRESSION_NODE_ID)
                as *const Cell<NodeId>)
        }
    }

    #[inline]
    pub fn span(self) -> &'t Span {
        unsafe {
            &*((self.0 as *const u8).add(OFFSET_TAGGED_TEMPLATE_EXPRESSION_SPAN) as *const Span)
        }
    }

    #[inline]
    pub fn tag(self) -> &'t Expression<'a> {
        unsafe {
            &*((self.0 as *const u8).add(OFFSET_TAGGED_TEMPLATE_EXPRESSION_TAG)
                as *const Expression<'a>)
        }
    }

    #[inline]
    pub fn type_arguments(self) -> &'t Option<ArenaBox<'a, TSTypeParameterInstantiation<'a>>> {
        unsafe {
            &*((self.0 as *const u8).add(OFFSET_TAGGED_TEMPLATE_EXPRESSION_TYPE_ARGUMENTS)
                as *const Option<ArenaBox<'a, TSTypeParameterInstantiation<'a>>>)
        }
    }
}

impl<'a, 't> GetAddress for TaggedTemplateExpressionWithoutQuasi<'a, 't> {
    #[inline]
    fn address(&self) -> Address {
        unsafe { Address::from_ptr(self.0) }
    }
}

pub(crate) const OFFSET_COMPUTED_MEMBER_EXPRESSION_NODE_ID: usize =
    offset_of!(ComputedMemberExpression, node_id);
pub(crate) const OFFSET_COMPUTED_MEMBER_EXPRESSION_SPAN: usize =
    offset_of!(ComputedMemberExpression, span);
pub(crate) const OFFSET_COMPUTED_MEMBER_EXPRESSION_OBJECT: usize =
    offset_of!(ComputedMemberExpression, object);
pub(crate) const OFFSET_COMPUTED_MEMBER_EXPRESSION_EXPRESSION: usize =
    offset_of!(ComputedMemberExpression, expression);
pub(crate) const OFFSET_COMPUTED_MEMBER_EXPRESSION_OPTIONAL: usize =
    offset_of!(ComputedMemberExpression, optional);

#[repr(transparent)]
#[derive(Clone, Copy, Debug)]
pub struct ComputedMemberExpressionWithoutObject<'a, 't>(
    pub(crate) *const ComputedMemberExpression<'a>,
    pub(crate) PhantomData<&'t ()>,
);

impl<'a, 't> ComputedMemberExpressionWithoutObject<'a, 't> {
    #[inline]
    pub fn node_id(self) -> &'t Cell<NodeId> {
        unsafe {
            &*((self.0 as *const u8).add(OFFSET_COMPUTED_MEMBER_EXPRESSION_NODE_ID)
                as *const Cell<NodeId>)
        }
    }

    #[inline]
    pub fn span(self) -> &'t Span {
        unsafe {
            &*((self.0 as *const u8).add(OFFSET_COMPUTED_MEMBER_EXPRESSION_SPAN) as *const Span)
        }
    }

    #[inline]
    pub fn expression(self) -> &'t Expression<'a> {
        unsafe {
            &*((self.0 as *const u8).add(OFFSET_COMPUTED_MEMBER_EXPRESSION_EXPRESSION)
                as *const Expression<'a>)
        }
    }

    #[inline]
    pub fn optional(self) -> &'t bool {
        unsafe {
            &*((self.0 as *const u8).add(OFFSET_COMPUTED_MEMBER_EXPRESSION_OPTIONAL) as *const bool)
        }
    }
}

impl<'a, 't> GetAddress for ComputedMemberExpressionWithoutObject<'a, 't> {
    #[inline]
    fn address(&self) -> Address {
        unsafe { Address::from_ptr(self.0) }
    }
}

#[repr(transparent)]
#[derive(Clone, Copy, Debug)]
pub struct ComputedMemberExpressionWithoutExpression<'a, 't>(
    pub(crate) *const ComputedMemberExpression<'a>,
    pub(crate) PhantomData<&'t ()>,
);

impl<'a, 't> ComputedMemberExpressionWithoutExpression<'a, 't> {
    #[inline]
    pub fn node_id(self) -> &'t Cell<NodeId> {
        unsafe {
            &*((self.0 as *const u8).add(OFFSET_COMPUTED_MEMBER_EXPRESSION_NODE_ID)
                as *const Cell<NodeId>)
        }
    }

    #[inline]
    pub fn span(self) -> &'t Span {
        unsafe {
            &*((self.0 as *const u8).add(OFFSET_COMPUTED_MEMBER_EXPRESSION_SPAN) as *const Span)
        }
    }

    #[inline]
    pub fn object(self) -> &'t Expression<'a> {
        unsafe {
            &*((self.0 as *const u8).add(OFFSET_COMPUTED_MEMBER_EXPRESSION_OBJECT)
                as *const Expression<'a>)
        }
    }

    #[inline]
    pub fn optional(self) -> &'t bool {
        unsafe {
            &*((self.0 as *const u8).add(OFFSET_COMPUTED_MEMBER_EXPRESSION_OPTIONAL) as *const bool)
        }
    }
}

impl<'a, 't> GetAddress for ComputedMemberExpressionWithoutExpression<'a, 't> {
    #[inline]
    fn address(&self) -> Address {
        unsafe { Address::from_ptr(self.0) }
    }
}

pub(crate) const OFFSET_STATIC_MEMBER_EXPRESSION_NODE_ID: usize =
    offset_of!(StaticMemberExpression, node_id);
pub(crate) const OFFSET_STATIC_MEMBER_EXPRESSION_SPAN: usize =
    offset_of!(StaticMemberExpression, span);
pub(crate) const OFFSET_STATIC_MEMBER_EXPRESSION_OBJECT: usize =
    offset_of!(StaticMemberExpression, object);
pub(crate) const OFFSET_STATIC_MEMBER_EXPRESSION_PROPERTY: usize =
    offset_of!(StaticMemberExpression, property);
pub(crate) const OFFSET_STATIC_MEMBER_EXPRESSION_OPTIONAL: usize =
    offset_of!(StaticMemberExpression, optional);

#[repr(transparent)]
#[derive(Clone, Copy, Debug)]
pub struct StaticMemberExpressionWithoutObject<'a, 't>(
    pub(crate) *const StaticMemberExpression<'a>,
    pub(crate) PhantomData<&'t ()>,
);

impl<'a, 't> StaticMemberExpressionWithoutObject<'a, 't> {
    #[inline]
    pub fn node_id(self) -> &'t Cell<NodeId> {
        unsafe {
            &*((self.0 as *const u8).add(OFFSET_STATIC_MEMBER_EXPRESSION_NODE_ID)
                as *const Cell<NodeId>)
        }
    }

    #[inline]
    pub fn span(self) -> &'t Span {
        unsafe {
            &*((self.0 as *const u8).add(OFFSET_STATIC_MEMBER_EXPRESSION_SPAN) as *const Span)
        }
    }

    #[inline]
    pub fn property(self) -> &'t IdentifierName<'a> {
        unsafe {
            &*((self.0 as *const u8).add(OFFSET_STATIC_MEMBER_EXPRESSION_PROPERTY)
                as *const IdentifierName<'a>)
        }
    }

    #[inline]
    pub fn optional(self) -> &'t bool {
        unsafe {
            &*((self.0 as *const u8).add(OFFSET_STATIC_MEMBER_EXPRESSION_OPTIONAL) as *const bool)
        }
    }
}

impl<'a, 't> GetAddress for StaticMemberExpressionWithoutObject<'a, 't> {
    #[inline]
    fn address(&self) -> Address {
        unsafe { Address::from_ptr(self.0) }
    }
}

#[repr(transparent)]
#[derive(Clone, Copy, Debug)]
pub struct StaticMemberExpressionWithoutProperty<'a, 't>(
    pub(crate) *const StaticMemberExpression<'a>,
    pub(crate) PhantomData<&'t ()>,
);

impl<'a, 't> StaticMemberExpressionWithoutProperty<'a, 't> {
    #[inline]
    pub fn node_id(self) -> &'t Cell<NodeId> {
        unsafe {
            &*((self.0 as *const u8).add(OFFSET_STATIC_MEMBER_EXPRESSION_NODE_ID)
                as *const Cell<NodeId>)
        }
    }

    #[inline]
    pub fn span(self) -> &'t Span {
        unsafe {
            &*((self.0 as *const u8).add(OFFSET_STATIC_MEMBER_EXPRESSION_SPAN) as *const Span)
        }
    }

    #[inline]
    pub fn object(self) -> &'t Expression<'a> {
        unsafe {
            &*((self.0 as *const u8).add(OFFSET_STATIC_MEMBER_EXPRESSION_OBJECT)
                as *const Expression<'a>)
        }
    }

    #[inline]
    pub fn optional(self) -> &'t bool {
        unsafe {
            &*((self.0 as *const u8).add(OFFSET_STATIC_MEMBER_EXPRESSION_OPTIONAL) as *const bool)
        }
    }
}

impl<'a, 't> GetAddress for StaticMemberExpressionWithoutProperty<'a, 't> {
    #[inline]
    fn address(&self) -> Address {
        unsafe { Address::from_ptr(self.0) }
    }
}

pub(crate) const OFFSET_PRIVATE_FIELD_EXPRESSION_NODE_ID: usize =
    offset_of!(PrivateFieldExpression, node_id);
pub(crate) const OFFSET_PRIVATE_FIELD_EXPRESSION_SPAN: usize =
    offset_of!(PrivateFieldExpression, span);
pub(crate) const OFFSET_PRIVATE_FIELD_EXPRESSION_OBJECT: usize =
    offset_of!(PrivateFieldExpression, object);
pub(crate) const OFFSET_PRIVATE_FIELD_EXPRESSION_FIELD: usize =
    offset_of!(PrivateFieldExpression, field);
pub(crate) const OFFSET_PRIVATE_FIELD_EXPRESSION_OPTIONAL: usize =
    offset_of!(PrivateFieldExpression, optional);

#[repr(transparent)]
#[derive(Clone, Copy, Debug)]
pub struct PrivateFieldExpressionWithoutObject<'a, 't>(
    pub(crate) *const PrivateFieldExpression<'a>,
    pub(crate) PhantomData<&'t ()>,
);

impl<'a, 't> PrivateFieldExpressionWithoutObject<'a, 't> {
    #[inline]
    pub fn node_id(self) -> &'t Cell<NodeId> {
        unsafe {
            &*((self.0 as *const u8).add(OFFSET_PRIVATE_FIELD_EXPRESSION_NODE_ID)
                as *const Cell<NodeId>)
        }
    }

    #[inline]
    pub fn span(self) -> &'t Span {
        unsafe {
            &*((self.0 as *const u8).add(OFFSET_PRIVATE_FIELD_EXPRESSION_SPAN) as *const Span)
        }
    }

    #[inline]
    pub fn field(self) -> &'t PrivateIdentifier<'a> {
        unsafe {
            &*((self.0 as *const u8).add(OFFSET_PRIVATE_FIELD_EXPRESSION_FIELD)
                as *const PrivateIdentifier<'a>)
        }
    }

    #[inline]
    pub fn optional(self) -> &'t bool {
        unsafe {
            &*((self.0 as *const u8).add(OFFSET_PRIVATE_FIELD_EXPRESSION_OPTIONAL) as *const bool)
        }
    }
}

impl<'a, 't> GetAddress for PrivateFieldExpressionWithoutObject<'a, 't> {
    #[inline]
    fn address(&self) -> Address {
        unsafe { Address::from_ptr(self.0) }
    }
}

#[repr(transparent)]
#[derive(Clone, Copy, Debug)]
pub struct PrivateFieldExpressionWithoutField<'a, 't>(
    pub(crate) *const PrivateFieldExpression<'a>,
    pub(crate) PhantomData<&'t ()>,
);

impl<'a, 't> PrivateFieldExpressionWithoutField<'a, 't> {
    #[inline]
    pub fn node_id(self) -> &'t Cell<NodeId> {
        unsafe {
            &*((self.0 as *const u8).add(OFFSET_PRIVATE_FIELD_EXPRESSION_NODE_ID)
                as *const Cell<NodeId>)
        }
    }

    #[inline]
    pub fn span(self) -> &'t Span {
        unsafe {
            &*((self.0 as *const u8).add(OFFSET_PRIVATE_FIELD_EXPRESSION_SPAN) as *const Span)
        }
    }

    #[inline]
    pub fn object(self) -> &'t Expression<'a> {
        unsafe {
            &*((self.0 as *const u8).add(OFFSET_PRIVATE_FIELD_EXPRESSION_OBJECT)
                as *const Expression<'a>)
        }
    }

    #[inline]
    pub fn optional(self) -> &'t bool {
        unsafe {
            &*((self.0 as *const u8).add(OFFSET_PRIVATE_FIELD_EXPRESSION_OPTIONAL) as *const bool)
        }
    }
}

impl<'a, 't> GetAddress for PrivateFieldExpressionWithoutField<'a, 't> {
    #[inline]
    fn address(&self) -> Address {
        unsafe { Address::from_ptr(self.0) }
    }
}

pub(crate) const OFFSET_CALL_EXPRESSION_NODE_ID: usize = offset_of!(CallExpression, node_id);
pub(crate) const OFFSET_CALL_EXPRESSION_SPAN: usize = offset_of!(CallExpression, span);
pub(crate) const OFFSET_CALL_EXPRESSION_CALLEE: usize = offset_of!(CallExpression, callee);
pub(crate) const OFFSET_CALL_EXPRESSION_TYPE_ARGUMENTS: usize =
    offset_of!(CallExpression, type_arguments);
pub(crate) const OFFSET_CALL_EXPRESSION_ARGUMENTS: usize = offset_of!(CallExpression, arguments);
pub(crate) const OFFSET_CALL_EXPRESSION_OPTIONAL: usize = offset_of!(CallExpression, optional);
pub(crate) const OFFSET_CALL_EXPRESSION_PURE: usize = offset_of!(CallExpression, pure);

#[repr(transparent)]
#[derive(Clone, Copy, Debug)]
pub struct CallExpressionWithoutCallee<'a, 't>(
    pub(crate) *const CallExpression<'a>,
    pub(crate) PhantomData<&'t ()>,
);

impl<'a, 't> CallExpressionWithoutCallee<'a, 't> {
    #[inline]
    pub fn node_id(self) -> &'t Cell<NodeId> {
        unsafe {
            &*((self.0 as *const u8).add(OFFSET_CALL_EXPRESSION_NODE_ID) as *const Cell<NodeId>)
        }
    }

    #[inline]
    pub fn span(self) -> &'t Span {
        unsafe { &*((self.0 as *const u8).add(OFFSET_CALL_EXPRESSION_SPAN) as *const Span) }
    }

    #[inline]
    pub fn type_arguments(self) -> &'t Option<ArenaBox<'a, TSTypeParameterInstantiation<'a>>> {
        unsafe {
            &*((self.0 as *const u8).add(OFFSET_CALL_EXPRESSION_TYPE_ARGUMENTS)
                as *const Option<ArenaBox<'a, TSTypeParameterInstantiation<'a>>>)
        }
    }

    #[inline]
    pub fn arguments(self) -> &'t ArenaVec<'a, Argument<'a>> {
        unsafe {
            &*((self.0 as *const u8).add(OFFSET_CALL_EXPRESSION_ARGUMENTS)
                as *const ArenaVec<'a, Argument<'a>>)
        }
    }

    #[inline]
    pub fn optional(self) -> &'t bool {
        unsafe { &*((self.0 as *const u8).add(OFFSET_CALL_EXPRESSION_OPTIONAL) as *const bool) }
    }

    #[inline]
    pub fn pure(self) -> &'t bool {
        unsafe { &*((self.0 as *const u8).add(OFFSET_CALL_EXPRESSION_PURE) as *const bool) }
    }
}

impl<'a, 't> GetAddress for CallExpressionWithoutCallee<'a, 't> {
    #[inline]
    fn address(&self) -> Address {
        unsafe { Address::from_ptr(self.0) }
    }
}

#[repr(transparent)]
#[derive(Clone, Copy, Debug)]
pub struct CallExpressionWithoutArguments<'a, 't>(
    pub(crate) *const CallExpression<'a>,
    pub(crate) PhantomData<&'t ()>,
);

impl<'a, 't> CallExpressionWithoutArguments<'a, 't> {
    #[inline]
    pub fn node_id(self) -> &'t Cell<NodeId> {
        unsafe {
            &*((self.0 as *const u8).add(OFFSET_CALL_EXPRESSION_NODE_ID) as *const Cell<NodeId>)
        }
    }

    #[inline]
    pub fn span(self) -> &'t Span {
        unsafe { &*((self.0 as *const u8).add(OFFSET_CALL_EXPRESSION_SPAN) as *const Span) }
    }

    #[inline]
    pub fn callee(self) -> &'t Expression<'a> {
        unsafe {
            &*((self.0 as *const u8).add(OFFSET_CALL_EXPRESSION_CALLEE) as *const Expression<'a>)
        }
    }

    #[inline]
    pub fn type_arguments(self) -> &'t Option<ArenaBox<'a, TSTypeParameterInstantiation<'a>>> {
        unsafe {
            &*((self.0 as *const u8).add(OFFSET_CALL_EXPRESSION_TYPE_ARGUMENTS)
                as *const Option<ArenaBox<'a, TSTypeParameterInstantiation<'a>>>)
        }
    }

    #[inline]
    pub fn optional(self) -> &'t bool {
        unsafe { &*((self.0 as *const u8).add(OFFSET_CALL_EXPRESSION_OPTIONAL) as *const bool) }
    }

    #[inline]
    pub fn pure(self) -> &'t bool {
        unsafe { &*((self.0 as *const u8).add(OFFSET_CALL_EXPRESSION_PURE) as *const bool) }
    }
}

impl<'a, 't> GetAddress for CallExpressionWithoutArguments<'a, 't> {
    #[inline]
    fn address(&self) -> Address {
        unsafe { Address::from_ptr(self.0) }
    }
}

pub(crate) const OFFSET_NEW_EXPRESSION_NODE_ID: usize = offset_of!(NewExpression, node_id);
pub(crate) const OFFSET_NEW_EXPRESSION_SPAN: usize = offset_of!(NewExpression, span);
pub(crate) const OFFSET_NEW_EXPRESSION_CALLEE: usize = offset_of!(NewExpression, callee);
pub(crate) const OFFSET_NEW_EXPRESSION_TYPE_ARGUMENTS: usize =
    offset_of!(NewExpression, type_arguments);
pub(crate) const OFFSET_NEW_EXPRESSION_ARGUMENTS: usize = offset_of!(NewExpression, arguments);
pub(crate) const OFFSET_NEW_EXPRESSION_PURE: usize = offset_of!(NewExpression, pure);

#[repr(transparent)]
#[derive(Clone, Copy, Debug)]
pub struct NewExpressionWithoutCallee<'a, 't>(
    pub(crate) *const NewExpression<'a>,
    pub(crate) PhantomData<&'t ()>,
);

impl<'a, 't> NewExpressionWithoutCallee<'a, 't> {
    #[inline]
    pub fn node_id(self) -> &'t Cell<NodeId> {
        unsafe {
            &*((self.0 as *const u8).add(OFFSET_NEW_EXPRESSION_NODE_ID) as *const Cell<NodeId>)
        }
    }

    #[inline]
    pub fn span(self) -> &'t Span {
        unsafe { &*((self.0 as *const u8).add(OFFSET_NEW_EXPRESSION_SPAN) as *const Span) }
    }

    #[inline]
    pub fn type_arguments(self) -> &'t Option<ArenaBox<'a, TSTypeParameterInstantiation<'a>>> {
        unsafe {
            &*((self.0 as *const u8).add(OFFSET_NEW_EXPRESSION_TYPE_ARGUMENTS)
                as *const Option<ArenaBox<'a, TSTypeParameterInstantiation<'a>>>)
        }
    }

    #[inline]
    pub fn arguments(self) -> &'t ArenaVec<'a, Argument<'a>> {
        unsafe {
            &*((self.0 as *const u8).add(OFFSET_NEW_EXPRESSION_ARGUMENTS)
                as *const ArenaVec<'a, Argument<'a>>)
        }
    }

    #[inline]
    pub fn pure(self) -> &'t bool {
        unsafe { &*((self.0 as *const u8).add(OFFSET_NEW_EXPRESSION_PURE) as *const bool) }
    }
}

impl<'a, 't> GetAddress for NewExpressionWithoutCallee<'a, 't> {
    #[inline]
    fn address(&self) -> Address {
        unsafe { Address::from_ptr(self.0) }
    }
}

#[repr(transparent)]
#[derive(Clone, Copy, Debug)]
pub struct NewExpressionWithoutArguments<'a, 't>(
    pub(crate) *const NewExpression<'a>,
    pub(crate) PhantomData<&'t ()>,
);

impl<'a, 't> NewExpressionWithoutArguments<'a, 't> {
    #[inline]
    pub fn node_id(self) -> &'t Cell<NodeId> {
        unsafe {
            &*((self.0 as *const u8).add(OFFSET_NEW_EXPRESSION_NODE_ID) as *const Cell<NodeId>)
        }
    }

    #[inline]
    pub fn span(self) -> &'t Span {
        unsafe { &*((self.0 as *const u8).add(OFFSET_NEW_EXPRESSION_SPAN) as *const Span) }
    }

    #[inline]
    pub fn callee(self) -> &'t Expression<'a> {
        unsafe {
            &*((self.0 as *const u8).add(OFFSET_NEW_EXPRESSION_CALLEE) as *const Expression<'a>)
        }
    }

    #[inline]
    pub fn type_arguments(self) -> &'t Option<ArenaBox<'a, TSTypeParameterInstantiation<'a>>> {
        unsafe {
            &*((self.0 as *const u8).add(OFFSET_NEW_EXPRESSION_TYPE_ARGUMENTS)
                as *const Option<ArenaBox<'a, TSTypeParameterInstantiation<'a>>>)
        }
    }

    #[inline]
    pub fn pure(self) -> &'t bool {
        unsafe { &*((self.0 as *const u8).add(OFFSET_NEW_EXPRESSION_PURE) as *const bool) }
    }
}

impl<'a, 't> GetAddress for NewExpressionWithoutArguments<'a, 't> {
    #[inline]
    fn address(&self) -> Address {
        unsafe { Address::from_ptr(self.0) }
    }
}

pub(crate) const OFFSET_META_PROPERTY_NODE_ID: usize = offset_of!(MetaProperty, node_id);
pub(crate) const OFFSET_META_PROPERTY_SPAN: usize = offset_of!(MetaProperty, span);
pub(crate) const OFFSET_META_PROPERTY_META: usize = offset_of!(MetaProperty, meta);
pub(crate) const OFFSET_META_PROPERTY_PROPERTY: usize = offset_of!(MetaProperty, property);

#[repr(transparent)]
#[derive(Clone, Copy, Debug)]
pub struct MetaPropertyWithoutMeta<'a, 't>(
    pub(crate) *const MetaProperty<'a>,
    pub(crate) PhantomData<&'t ()>,
);

impl<'a, 't> MetaPropertyWithoutMeta<'a, 't> {
    #[inline]
    pub fn node_id(self) -> &'t Cell<NodeId> {
        unsafe {
            &*((self.0 as *const u8).add(OFFSET_META_PROPERTY_NODE_ID) as *const Cell<NodeId>)
        }
    }

    #[inline]
    pub fn span(self) -> &'t Span {
        unsafe { &*((self.0 as *const u8).add(OFFSET_META_PROPERTY_SPAN) as *const Span) }
    }

    #[inline]
    pub fn property(self) -> &'t IdentifierName<'a> {
        unsafe {
            &*((self.0 as *const u8).add(OFFSET_META_PROPERTY_PROPERTY)
                as *const IdentifierName<'a>)
        }
    }
}

impl<'a, 't> GetAddress for MetaPropertyWithoutMeta<'a, 't> {
    #[inline]
    fn address(&self) -> Address {
        unsafe { Address::from_ptr(self.0) }
    }
}

#[repr(transparent)]
#[derive(Clone, Copy, Debug)]
pub struct MetaPropertyWithoutProperty<'a, 't>(
    pub(crate) *const MetaProperty<'a>,
    pub(crate) PhantomData<&'t ()>,
);

impl<'a, 't> MetaPropertyWithoutProperty<'a, 't> {
    #[inline]
    pub fn node_id(self) -> &'t Cell<NodeId> {
        unsafe {
            &*((self.0 as *const u8).add(OFFSET_META_PROPERTY_NODE_ID) as *const Cell<NodeId>)
        }
    }

    #[inline]
    pub fn span(self) -> &'t Span {
        unsafe { &*((self.0 as *const u8).add(OFFSET_META_PROPERTY_SPAN) as *const Span) }
    }

    #[inline]
    pub fn meta(self) -> &'t IdentifierName<'a> {
        unsafe {
            &*((self.0 as *const u8).add(OFFSET_META_PROPERTY_META) as *const IdentifierName<'a>)
        }
    }
}

impl<'a, 't> GetAddress for MetaPropertyWithoutProperty<'a, 't> {
    #[inline]
    fn address(&self) -> Address {
        unsafe { Address::from_ptr(self.0) }
    }
}

pub(crate) const OFFSET_SPREAD_ELEMENT_NODE_ID: usize = offset_of!(SpreadElement, node_id);
pub(crate) const OFFSET_SPREAD_ELEMENT_SPAN: usize = offset_of!(SpreadElement, span);
pub(crate) const OFFSET_SPREAD_ELEMENT_ARGUMENT: usize = offset_of!(SpreadElement, argument);

#[repr(transparent)]
#[derive(Clone, Copy, Debug)]
pub struct SpreadElementWithoutArgument<'a, 't>(
    pub(crate) *const SpreadElement<'a>,
    pub(crate) PhantomData<&'t ()>,
);

impl<'a, 't> SpreadElementWithoutArgument<'a, 't> {
    #[inline]
    pub fn node_id(self) -> &'t Cell<NodeId> {
        unsafe {
            &*((self.0 as *const u8).add(OFFSET_SPREAD_ELEMENT_NODE_ID) as *const Cell<NodeId>)
        }
    }

    #[inline]
    pub fn span(self) -> &'t Span {
        unsafe { &*((self.0 as *const u8).add(OFFSET_SPREAD_ELEMENT_SPAN) as *const Span) }
    }
}

impl<'a, 't> GetAddress for SpreadElementWithoutArgument<'a, 't> {
    #[inline]
    fn address(&self) -> Address {
        unsafe { Address::from_ptr(self.0) }
    }
}

pub(crate) const OFFSET_UPDATE_EXPRESSION_NODE_ID: usize = offset_of!(UpdateExpression, node_id);
pub(crate) const OFFSET_UPDATE_EXPRESSION_SPAN: usize = offset_of!(UpdateExpression, span);
pub(crate) const OFFSET_UPDATE_EXPRESSION_OPERATOR: usize = offset_of!(UpdateExpression, operator);
pub(crate) const OFFSET_UPDATE_EXPRESSION_PREFIX: usize = offset_of!(UpdateExpression, prefix);
pub(crate) const OFFSET_UPDATE_EXPRESSION_ARGUMENT: usize = offset_of!(UpdateExpression, argument);

#[repr(transparent)]
#[derive(Clone, Copy, Debug)]
pub struct UpdateExpressionWithoutArgument<'a, 't>(
    pub(crate) *const UpdateExpression<'a>,
    pub(crate) PhantomData<&'t ()>,
);

impl<'a, 't> UpdateExpressionWithoutArgument<'a, 't> {
    #[inline]
    pub fn node_id(self) -> &'t Cell<NodeId> {
        unsafe {
            &*((self.0 as *const u8).add(OFFSET_UPDATE_EXPRESSION_NODE_ID) as *const Cell<NodeId>)
        }
    }

    #[inline]
    pub fn span(self) -> &'t Span {
        unsafe { &*((self.0 as *const u8).add(OFFSET_UPDATE_EXPRESSION_SPAN) as *const Span) }
    }

    #[inline]
    pub fn operator(self) -> &'t UpdateOperator {
        unsafe {
            &*((self.0 as *const u8).add(OFFSET_UPDATE_EXPRESSION_OPERATOR)
                as *const UpdateOperator)
        }
    }

    #[inline]
    pub fn prefix(self) -> &'t bool {
        unsafe { &*((self.0 as *const u8).add(OFFSET_UPDATE_EXPRESSION_PREFIX) as *const bool) }
    }
}

impl<'a, 't> GetAddress for UpdateExpressionWithoutArgument<'a, 't> {
    #[inline]
    fn address(&self) -> Address {
        unsafe { Address::from_ptr(self.0) }
    }
}

pub(crate) const OFFSET_UNARY_EXPRESSION_NODE_ID: usize = offset_of!(UnaryExpression, node_id);
pub(crate) const OFFSET_UNARY_EXPRESSION_SPAN: usize = offset_of!(UnaryExpression, span);
pub(crate) const OFFSET_UNARY_EXPRESSION_OPERATOR: usize = offset_of!(UnaryExpression, operator);
pub(crate) const OFFSET_UNARY_EXPRESSION_ARGUMENT: usize = offset_of!(UnaryExpression, argument);

#[repr(transparent)]
#[derive(Clone, Copy, Debug)]
pub struct UnaryExpressionWithoutArgument<'a, 't>(
    pub(crate) *const UnaryExpression<'a>,
    pub(crate) PhantomData<&'t ()>,
);

impl<'a, 't> UnaryExpressionWithoutArgument<'a, 't> {
    #[inline]
    pub fn node_id(self) -> &'t Cell<NodeId> {
        unsafe {
            &*((self.0 as *const u8).add(OFFSET_UNARY_EXPRESSION_NODE_ID) as *const Cell<NodeId>)
        }
    }

    #[inline]
    pub fn span(self) -> &'t Span {
        unsafe { &*((self.0 as *const u8).add(OFFSET_UNARY_EXPRESSION_SPAN) as *const Span) }
    }

    #[inline]
    pub fn operator(self) -> &'t UnaryOperator {
        unsafe {
            &*((self.0 as *const u8).add(OFFSET_UNARY_EXPRESSION_OPERATOR) as *const UnaryOperator)
        }
    }
}

impl<'a, 't> GetAddress for UnaryExpressionWithoutArgument<'a, 't> {
    #[inline]
    fn address(&self) -> Address {
        unsafe { Address::from_ptr(self.0) }
    }
}

pub(crate) const OFFSET_BINARY_EXPRESSION_NODE_ID: usize = offset_of!(BinaryExpression, node_id);
pub(crate) const OFFSET_BINARY_EXPRESSION_SPAN: usize = offset_of!(BinaryExpression, span);
pub(crate) const OFFSET_BINARY_EXPRESSION_LEFT: usize = offset_of!(BinaryExpression, left);
pub(crate) const OFFSET_BINARY_EXPRESSION_OPERATOR: usize = offset_of!(BinaryExpression, operator);
pub(crate) const OFFSET_BINARY_EXPRESSION_RIGHT: usize = offset_of!(BinaryExpression, right);

#[repr(transparent)]
#[derive(Clone, Copy, Debug)]
pub struct BinaryExpressionWithoutLeft<'a, 't>(
    pub(crate) *const BinaryExpression<'a>,
    pub(crate) PhantomData<&'t ()>,
);

impl<'a, 't> BinaryExpressionWithoutLeft<'a, 't> {
    #[inline]
    pub fn node_id(self) -> &'t Cell<NodeId> {
        unsafe {
            &*((self.0 as *const u8).add(OFFSET_BINARY_EXPRESSION_NODE_ID) as *const Cell<NodeId>)
        }
    }

    #[inline]
    pub fn span(self) -> &'t Span {
        unsafe { &*((self.0 as *const u8).add(OFFSET_BINARY_EXPRESSION_SPAN) as *const Span) }
    }

    #[inline]
    pub fn operator(self) -> &'t BinaryOperator {
        unsafe {
            &*((self.0 as *const u8).add(OFFSET_BINARY_EXPRESSION_OPERATOR)
                as *const BinaryOperator)
        }
    }

    #[inline]
    pub fn right(self) -> &'t Expression<'a> {
        unsafe {
            &*((self.0 as *const u8).add(OFFSET_BINARY_EXPRESSION_RIGHT) as *const Expression<'a>)
        }
    }
}

impl<'a, 't> GetAddress for BinaryExpressionWithoutLeft<'a, 't> {
    #[inline]
    fn address(&self) -> Address {
        unsafe { Address::from_ptr(self.0) }
    }
}

#[repr(transparent)]
#[derive(Clone, Copy, Debug)]
pub struct BinaryExpressionWithoutRight<'a, 't>(
    pub(crate) *const BinaryExpression<'a>,
    pub(crate) PhantomData<&'t ()>,
);

impl<'a, 't> BinaryExpressionWithoutRight<'a, 't> {
    #[inline]
    pub fn node_id(self) -> &'t Cell<NodeId> {
        unsafe {
            &*((self.0 as *const u8).add(OFFSET_BINARY_EXPRESSION_NODE_ID) as *const Cell<NodeId>)
        }
    }

    #[inline]
    pub fn span(self) -> &'t Span {
        unsafe { &*((self.0 as *const u8).add(OFFSET_BINARY_EXPRESSION_SPAN) as *const Span) }
    }

    #[inline]
    pub fn left(self) -> &'t Expression<'a> {
        unsafe {
            &*((self.0 as *const u8).add(OFFSET_BINARY_EXPRESSION_LEFT) as *const Expression<'a>)
        }
    }

    #[inline]
    pub fn operator(self) -> &'t BinaryOperator {
        unsafe {
            &*((self.0 as *const u8).add(OFFSET_BINARY_EXPRESSION_OPERATOR)
                as *const BinaryOperator)
        }
    }
}

impl<'a, 't> GetAddress for BinaryExpressionWithoutRight<'a, 't> {
    #[inline]
    fn address(&self) -> Address {
        unsafe { Address::from_ptr(self.0) }
    }
}

pub(crate) const OFFSET_PRIVATE_IN_EXPRESSION_NODE_ID: usize =
    offset_of!(PrivateInExpression, node_id);
pub(crate) const OFFSET_PRIVATE_IN_EXPRESSION_SPAN: usize = offset_of!(PrivateInExpression, span);
pub(crate) const OFFSET_PRIVATE_IN_EXPRESSION_LEFT: usize = offset_of!(PrivateInExpression, left);
pub(crate) const OFFSET_PRIVATE_IN_EXPRESSION_RIGHT: usize = offset_of!(PrivateInExpression, right);

#[repr(transparent)]
#[derive(Clone, Copy, Debug)]
pub struct PrivateInExpressionWithoutLeft<'a, 't>(
    pub(crate) *const PrivateInExpression<'a>,
    pub(crate) PhantomData<&'t ()>,
);

impl<'a, 't> PrivateInExpressionWithoutLeft<'a, 't> {
    #[inline]
    pub fn node_id(self) -> &'t Cell<NodeId> {
        unsafe {
            &*((self.0 as *const u8).add(OFFSET_PRIVATE_IN_EXPRESSION_NODE_ID)
                as *const Cell<NodeId>)
        }
    }

    #[inline]
    pub fn span(self) -> &'t Span {
        unsafe { &*((self.0 as *const u8).add(OFFSET_PRIVATE_IN_EXPRESSION_SPAN) as *const Span) }
    }

    #[inline]
    pub fn right(self) -> &'t Expression<'a> {
        unsafe {
            &*((self.0 as *const u8).add(OFFSET_PRIVATE_IN_EXPRESSION_RIGHT)
                as *const Expression<'a>)
        }
    }
}

impl<'a, 't> GetAddress for PrivateInExpressionWithoutLeft<'a, 't> {
    #[inline]
    fn address(&self) -> Address {
        unsafe { Address::from_ptr(self.0) }
    }
}

#[repr(transparent)]
#[derive(Clone, Copy, Debug)]
pub struct PrivateInExpressionWithoutRight<'a, 't>(
    pub(crate) *const PrivateInExpression<'a>,
    pub(crate) PhantomData<&'t ()>,
);

impl<'a, 't> PrivateInExpressionWithoutRight<'a, 't> {
    #[inline]
    pub fn node_id(self) -> &'t Cell<NodeId> {
        unsafe {
            &*((self.0 as *const u8).add(OFFSET_PRIVATE_IN_EXPRESSION_NODE_ID)
                as *const Cell<NodeId>)
        }
    }

    #[inline]
    pub fn span(self) -> &'t Span {
        unsafe { &*((self.0 as *const u8).add(OFFSET_PRIVATE_IN_EXPRESSION_SPAN) as *const Span) }
    }

    #[inline]
    pub fn left(self) -> &'t PrivateIdentifier<'a> {
        unsafe {
            &*((self.0 as *const u8).add(OFFSET_PRIVATE_IN_EXPRESSION_LEFT)
                as *const PrivateIdentifier<'a>)
        }
    }
}

impl<'a, 't> GetAddress for PrivateInExpressionWithoutRight<'a, 't> {
    #[inline]
    fn address(&self) -> Address {
        unsafe { Address::from_ptr(self.0) }
    }
}

pub(crate) const OFFSET_LOGICAL_EXPRESSION_NODE_ID: usize = offset_of!(LogicalExpression, node_id);
pub(crate) const OFFSET_LOGICAL_EXPRESSION_SPAN: usize = offset_of!(LogicalExpression, span);
pub(crate) const OFFSET_LOGICAL_EXPRESSION_LEFT: usize = offset_of!(LogicalExpression, left);
pub(crate) const OFFSET_LOGICAL_EXPRESSION_OPERATOR: usize =
    offset_of!(LogicalExpression, operator);
pub(crate) const OFFSET_LOGICAL_EXPRESSION_RIGHT: usize = offset_of!(LogicalExpression, right);

#[repr(transparent)]
#[derive(Clone, Copy, Debug)]
pub struct LogicalExpressionWithoutLeft<'a, 't>(
    pub(crate) *const LogicalExpression<'a>,
    pub(crate) PhantomData<&'t ()>,
);

impl<'a, 't> LogicalExpressionWithoutLeft<'a, 't> {
    #[inline]
    pub fn node_id(self) -> &'t Cell<NodeId> {
        unsafe {
            &*((self.0 as *const u8).add(OFFSET_LOGICAL_EXPRESSION_NODE_ID) as *const Cell<NodeId>)
        }
    }

    #[inline]
    pub fn span(self) -> &'t Span {
        unsafe { &*((self.0 as *const u8).add(OFFSET_LOGICAL_EXPRESSION_SPAN) as *const Span) }
    }

    #[inline]
    pub fn operator(self) -> &'t LogicalOperator {
        unsafe {
            &*((self.0 as *const u8).add(OFFSET_LOGICAL_EXPRESSION_OPERATOR)
                as *const LogicalOperator)
        }
    }

    #[inline]
    pub fn right(self) -> &'t Expression<'a> {
        unsafe {
            &*((self.0 as *const u8).add(OFFSET_LOGICAL_EXPRESSION_RIGHT) as *const Expression<'a>)
        }
    }
}

impl<'a, 't> GetAddress for LogicalExpressionWithoutLeft<'a, 't> {
    #[inline]
    fn address(&self) -> Address {
        unsafe { Address::from_ptr(self.0) }
    }
}

#[repr(transparent)]
#[derive(Clone, Copy, Debug)]
pub struct LogicalExpressionWithoutRight<'a, 't>(
    pub(crate) *const LogicalExpression<'a>,
    pub(crate) PhantomData<&'t ()>,
);

impl<'a, 't> LogicalExpressionWithoutRight<'a, 't> {
    #[inline]
    pub fn node_id(self) -> &'t Cell<NodeId> {
        unsafe {
            &*((self.0 as *const u8).add(OFFSET_LOGICAL_EXPRESSION_NODE_ID) as *const Cell<NodeId>)
        }
    }

    #[inline]
    pub fn span(self) -> &'t Span {
        unsafe { &*((self.0 as *const u8).add(OFFSET_LOGICAL_EXPRESSION_SPAN) as *const Span) }
    }

    #[inline]
    pub fn left(self) -> &'t Expression<'a> {
        unsafe {
            &*((self.0 as *const u8).add(OFFSET_LOGICAL_EXPRESSION_LEFT) as *const Expression<'a>)
        }
    }

    #[inline]
    pub fn operator(self) -> &'t LogicalOperator {
        unsafe {
            &*((self.0 as *const u8).add(OFFSET_LOGICAL_EXPRESSION_OPERATOR)
                as *const LogicalOperator)
        }
    }
}

impl<'a, 't> GetAddress for LogicalExpressionWithoutRight<'a, 't> {
    #[inline]
    fn address(&self) -> Address {
        unsafe { Address::from_ptr(self.0) }
    }
}

pub(crate) const OFFSET_CONDITIONAL_EXPRESSION_NODE_ID: usize =
    offset_of!(ConditionalExpression, node_id);
pub(crate) const OFFSET_CONDITIONAL_EXPRESSION_SPAN: usize =
    offset_of!(ConditionalExpression, span);
pub(crate) const OFFSET_CONDITIONAL_EXPRESSION_TEST: usize =
    offset_of!(ConditionalExpression, test);
pub(crate) const OFFSET_CONDITIONAL_EXPRESSION_CONSEQUENT: usize =
    offset_of!(ConditionalExpression, consequent);
pub(crate) const OFFSET_CONDITIONAL_EXPRESSION_ALTERNATE: usize =
    offset_of!(ConditionalExpression, alternate);

#[repr(transparent)]
#[derive(Clone, Copy, Debug)]
pub struct ConditionalExpressionWithoutTest<'a, 't>(
    pub(crate) *const ConditionalExpression<'a>,
    pub(crate) PhantomData<&'t ()>,
);

impl<'a, 't> ConditionalExpressionWithoutTest<'a, 't> {
    #[inline]
    pub fn node_id(self) -> &'t Cell<NodeId> {
        unsafe {
            &*((self.0 as *const u8).add(OFFSET_CONDITIONAL_EXPRESSION_NODE_ID)
                as *const Cell<NodeId>)
        }
    }

    #[inline]
    pub fn span(self) -> &'t Span {
        unsafe { &*((self.0 as *const u8).add(OFFSET_CONDITIONAL_EXPRESSION_SPAN) as *const Span) }
    }

    #[inline]
    pub fn consequent(self) -> &'t Expression<'a> {
        unsafe {
            &*((self.0 as *const u8).add(OFFSET_CONDITIONAL_EXPRESSION_CONSEQUENT)
                as *const Expression<'a>)
        }
    }

    #[inline]
    pub fn alternate(self) -> &'t Expression<'a> {
        unsafe {
            &*((self.0 as *const u8).add(OFFSET_CONDITIONAL_EXPRESSION_ALTERNATE)
                as *const Expression<'a>)
        }
    }
}

impl<'a, 't> GetAddress for ConditionalExpressionWithoutTest<'a, 't> {
    #[inline]
    fn address(&self) -> Address {
        unsafe { Address::from_ptr(self.0) }
    }
}

#[repr(transparent)]
#[derive(Clone, Copy, Debug)]
pub struct ConditionalExpressionWithoutConsequent<'a, 't>(
    pub(crate) *const ConditionalExpression<'a>,
    pub(crate) PhantomData<&'t ()>,
);

impl<'a, 't> ConditionalExpressionWithoutConsequent<'a, 't> {
    #[inline]
    pub fn node_id(self) -> &'t Cell<NodeId> {
        unsafe {
            &*((self.0 as *const u8).add(OFFSET_CONDITIONAL_EXPRESSION_NODE_ID)
                as *const Cell<NodeId>)
        }
    }

    #[inline]
    pub fn span(self) -> &'t Span {
        unsafe { &*((self.0 as *const u8).add(OFFSET_CONDITIONAL_EXPRESSION_SPAN) as *const Span) }
    }

    #[inline]
    pub fn test(self) -> &'t Expression<'a> {
        unsafe {
            &*((self.0 as *const u8).add(OFFSET_CONDITIONAL_EXPRESSION_TEST)
                as *const Expression<'a>)
        }
    }

    #[inline]
    pub fn alternate(self) -> &'t Expression<'a> {
        unsafe {
            &*((self.0 as *const u8).add(OFFSET_CONDITIONAL_EXPRESSION_ALTERNATE)
                as *const Expression<'a>)
        }
    }
}

impl<'a, 't> GetAddress for ConditionalExpressionWithoutConsequent<'a, 't> {
    #[inline]
    fn address(&self) -> Address {
        unsafe { Address::from_ptr(self.0) }
    }
}

#[repr(transparent)]
#[derive(Clone, Copy, Debug)]
pub struct ConditionalExpressionWithoutAlternate<'a, 't>(
    pub(crate) *const ConditionalExpression<'a>,
    pub(crate) PhantomData<&'t ()>,
);

impl<'a, 't> ConditionalExpressionWithoutAlternate<'a, 't> {
    #[inline]
    pub fn node_id(self) -> &'t Cell<NodeId> {
        unsafe {
            &*((self.0 as *const u8).add(OFFSET_CONDITIONAL_EXPRESSION_NODE_ID)
                as *const Cell<NodeId>)
        }
    }

    #[inline]
    pub fn span(self) -> &'t Span {
        unsafe { &*((self.0 as *const u8).add(OFFSET_CONDITIONAL_EXPRESSION_SPAN) as *const Span) }
    }

    #[inline]
    pub fn test(self) -> &'t Expression<'a> {
        unsafe {
            &*((self.0 as *const u8).add(OFFSET_CONDITIONAL_EXPRESSION_TEST)
                as *const Expression<'a>)
        }
    }

    #[inline]
    pub fn consequent(self) -> &'t Expression<'a> {
        unsafe {
            &*((self.0 as *const u8).add(OFFSET_CONDITIONAL_EXPRESSION_CONSEQUENT)
                as *const Expression<'a>)
        }
    }
}

impl<'a, 't> GetAddress for ConditionalExpressionWithoutAlternate<'a, 't> {
    #[inline]
    fn address(&self) -> Address {
        unsafe { Address::from_ptr(self.0) }
    }
}

pub(crate) const OFFSET_ASSIGNMENT_EXPRESSION_NODE_ID: usize =
    offset_of!(AssignmentExpression, node_id);
pub(crate) const OFFSET_ASSIGNMENT_EXPRESSION_SPAN: usize = offset_of!(AssignmentExpression, span);
pub(crate) const OFFSET_ASSIGNMENT_EXPRESSION_OPERATOR: usize =
    offset_of!(AssignmentExpression, operator);
pub(crate) const OFFSET_ASSIGNMENT_EXPRESSION_LEFT: usize = offset_of!(AssignmentExpression, left);
pub(crate) const OFFSET_ASSIGNMENT_EXPRESSION_RIGHT: usize =
    offset_of!(AssignmentExpression, right);

#[repr(transparent)]
#[derive(Clone, Copy, Debug)]
pub struct AssignmentExpressionWithoutLeft<'a, 't>(
    pub(crate) *const AssignmentExpression<'a>,
    pub(crate) PhantomData<&'t ()>,
);

impl<'a, 't> AssignmentExpressionWithoutLeft<'a, 't> {
    #[inline]
    pub fn node_id(self) -> &'t Cell<NodeId> {
        unsafe {
            &*((self.0 as *const u8).add(OFFSET_ASSIGNMENT_EXPRESSION_NODE_ID)
                as *const Cell<NodeId>)
        }
    }

    #[inline]
    pub fn span(self) -> &'t Span {
        unsafe { &*((self.0 as *const u8).add(OFFSET_ASSIGNMENT_EXPRESSION_SPAN) as *const Span) }
    }

    #[inline]
    pub fn operator(self) -> &'t AssignmentOperator {
        unsafe {
            &*((self.0 as *const u8).add(OFFSET_ASSIGNMENT_EXPRESSION_OPERATOR)
                as *const AssignmentOperator)
        }
    }

    #[inline]
    pub fn right(self) -> &'t Expression<'a> {
        unsafe {
            &*((self.0 as *const u8).add(OFFSET_ASSIGNMENT_EXPRESSION_RIGHT)
                as *const Expression<'a>)
        }
    }
}

impl<'a, 't> GetAddress for AssignmentExpressionWithoutLeft<'a, 't> {
    #[inline]
    fn address(&self) -> Address {
        unsafe { Address::from_ptr(self.0) }
    }
}

#[repr(transparent)]
#[derive(Clone, Copy, Debug)]
pub struct AssignmentExpressionWithoutRight<'a, 't>(
    pub(crate) *const AssignmentExpression<'a>,
    pub(crate) PhantomData<&'t ()>,
);

impl<'a, 't> AssignmentExpressionWithoutRight<'a, 't> {
    #[inline]
    pub fn node_id(self) -> &'t Cell<NodeId> {
        unsafe {
            &*((self.0 as *const u8).add(OFFSET_ASSIGNMENT_EXPRESSION_NODE_ID)
                as *const Cell<NodeId>)
        }
    }

    #[inline]
    pub fn span(self) -> &'t Span {
        unsafe { &*((self.0 as *const u8).add(OFFSET_ASSIGNMENT_EXPRESSION_SPAN) as *const Span) }
    }

    #[inline]
    pub fn operator(self) -> &'t AssignmentOperator {
        unsafe {
            &*((self.0 as *const u8).add(OFFSET_ASSIGNMENT_EXPRESSION_OPERATOR)
                as *const AssignmentOperator)
        }
    }

    #[inline]
    pub fn left(self) -> &'t AssignmentTarget<'a> {
        unsafe {
            &*((self.0 as *const u8).add(OFFSET_ASSIGNMENT_EXPRESSION_LEFT)
                as *const AssignmentTarget<'a>)
        }
    }
}

impl<'a, 't> GetAddress for AssignmentExpressionWithoutRight<'a, 't> {
    #[inline]
    fn address(&self) -> Address {
        unsafe { Address::from_ptr(self.0) }
    }
}

pub(crate) const OFFSET_ARRAY_ASSIGNMENT_TARGET_NODE_ID: usize =
    offset_of!(ArrayAssignmentTarget, node_id);
pub(crate) const OFFSET_ARRAY_ASSIGNMENT_TARGET_SPAN: usize =
    offset_of!(ArrayAssignmentTarget, span);
pub(crate) const OFFSET_ARRAY_ASSIGNMENT_TARGET_ELEMENTS: usize =
    offset_of!(ArrayAssignmentTarget, elements);
pub(crate) const OFFSET_ARRAY_ASSIGNMENT_TARGET_REST: usize =
    offset_of!(ArrayAssignmentTarget, rest);

#[repr(transparent)]
#[derive(Clone, Copy, Debug)]
pub struct ArrayAssignmentTargetWithoutElements<'a, 't>(
    pub(crate) *const ArrayAssignmentTarget<'a>,
    pub(crate) PhantomData<&'t ()>,
);

impl<'a, 't> ArrayAssignmentTargetWithoutElements<'a, 't> {
    #[inline]
    pub fn node_id(self) -> &'t Cell<NodeId> {
        unsafe {
            &*((self.0 as *const u8).add(OFFSET_ARRAY_ASSIGNMENT_TARGET_NODE_ID)
                as *const Cell<NodeId>)
        }
    }

    #[inline]
    pub fn span(self) -> &'t Span {
        unsafe { &*((self.0 as *const u8).add(OFFSET_ARRAY_ASSIGNMENT_TARGET_SPAN) as *const Span) }
    }

    #[inline]
    pub fn rest(self) -> &'t Option<ArenaBox<'a, AssignmentTargetRest<'a>>> {
        unsafe {
            &*((self.0 as *const u8).add(OFFSET_ARRAY_ASSIGNMENT_TARGET_REST)
                as *const Option<ArenaBox<'a, AssignmentTargetRest<'a>>>)
        }
    }
}

impl<'a, 't> GetAddress for ArrayAssignmentTargetWithoutElements<'a, 't> {
    #[inline]
    fn address(&self) -> Address {
        unsafe { Address::from_ptr(self.0) }
    }
}

#[repr(transparent)]
#[derive(Clone, Copy, Debug)]
pub struct ArrayAssignmentTargetWithoutRest<'a, 't>(
    pub(crate) *const ArrayAssignmentTarget<'a>,
    pub(crate) PhantomData<&'t ()>,
);

impl<'a, 't> ArrayAssignmentTargetWithoutRest<'a, 't> {
    #[inline]
    pub fn node_id(self) -> &'t Cell<NodeId> {
        unsafe {
            &*((self.0 as *const u8).add(OFFSET_ARRAY_ASSIGNMENT_TARGET_NODE_ID)
                as *const Cell<NodeId>)
        }
    }

    #[inline]
    pub fn span(self) -> &'t Span {
        unsafe { &*((self.0 as *const u8).add(OFFSET_ARRAY_ASSIGNMENT_TARGET_SPAN) as *const Span) }
    }

    #[inline]
    pub fn elements(self) -> &'t ArenaVec<'a, Option<AssignmentTargetMaybeDefault<'a>>> {
        unsafe {
            &*((self.0 as *const u8).add(OFFSET_ARRAY_ASSIGNMENT_TARGET_ELEMENTS)
                as *const ArenaVec<'a, Option<AssignmentTargetMaybeDefault<'a>>>)
        }
    }
}

impl<'a, 't> GetAddress for ArrayAssignmentTargetWithoutRest<'a, 't> {
    #[inline]
    fn address(&self) -> Address {
        unsafe { Address::from_ptr(self.0) }
    }
}

pub(crate) const OFFSET_OBJECT_ASSIGNMENT_TARGET_NODE_ID: usize =
    offset_of!(ObjectAssignmentTarget, node_id);
pub(crate) const OFFSET_OBJECT_ASSIGNMENT_TARGET_SPAN: usize =
    offset_of!(ObjectAssignmentTarget, span);
pub(crate) const OFFSET_OBJECT_ASSIGNMENT_TARGET_PROPERTIES: usize =
    offset_of!(ObjectAssignmentTarget, properties);
pub(crate) const OFFSET_OBJECT_ASSIGNMENT_TARGET_REST: usize =
    offset_of!(ObjectAssignmentTarget, rest);

#[repr(transparent)]
#[derive(Clone, Copy, Debug)]
pub struct ObjectAssignmentTargetWithoutProperties<'a, 't>(
    pub(crate) *const ObjectAssignmentTarget<'a>,
    pub(crate) PhantomData<&'t ()>,
);

impl<'a, 't> ObjectAssignmentTargetWithoutProperties<'a, 't> {
    #[inline]
    pub fn node_id(self) -> &'t Cell<NodeId> {
        unsafe {
            &*((self.0 as *const u8).add(OFFSET_OBJECT_ASSIGNMENT_TARGET_NODE_ID)
                as *const Cell<NodeId>)
        }
    }

    #[inline]
    pub fn span(self) -> &'t Span {
        unsafe {
            &*((self.0 as *const u8).add(OFFSET_OBJECT_ASSIGNMENT_TARGET_SPAN) as *const Span)
        }
    }

    #[inline]
    pub fn rest(self) -> &'t Option<ArenaBox<'a, AssignmentTargetRest<'a>>> {
        unsafe {
            &*((self.0 as *const u8).add(OFFSET_OBJECT_ASSIGNMENT_TARGET_REST)
                as *const Option<ArenaBox<'a, AssignmentTargetRest<'a>>>)
        }
    }
}

impl<'a, 't> GetAddress for ObjectAssignmentTargetWithoutProperties<'a, 't> {
    #[inline]
    fn address(&self) -> Address {
        unsafe { Address::from_ptr(self.0) }
    }
}

#[repr(transparent)]
#[derive(Clone, Copy, Debug)]
pub struct ObjectAssignmentTargetWithoutRest<'a, 't>(
    pub(crate) *const ObjectAssignmentTarget<'a>,
    pub(crate) PhantomData<&'t ()>,
);

impl<'a, 't> ObjectAssignmentTargetWithoutRest<'a, 't> {
    #[inline]
    pub fn node_id(self) -> &'t Cell<NodeId> {
        unsafe {
            &*((self.0 as *const u8).add(OFFSET_OBJECT_ASSIGNMENT_TARGET_NODE_ID)
                as *const Cell<NodeId>)
        }
    }

    #[inline]
    pub fn span(self) -> &'t Span {
        unsafe {
            &*((self.0 as *const u8).add(OFFSET_OBJECT_ASSIGNMENT_TARGET_SPAN) as *const Span)
        }
    }

    #[inline]
    pub fn properties(self) -> &'t ArenaVec<'a, AssignmentTargetProperty<'a>> {
        unsafe {
            &*((self.0 as *const u8).add(OFFSET_OBJECT_ASSIGNMENT_TARGET_PROPERTIES)
                as *const ArenaVec<'a, AssignmentTargetProperty<'a>>)
        }
    }
}

impl<'a, 't> GetAddress for ObjectAssignmentTargetWithoutRest<'a, 't> {
    #[inline]
    fn address(&self) -> Address {
        unsafe { Address::from_ptr(self.0) }
    }
}

pub(crate) const OFFSET_ASSIGNMENT_TARGET_REST_NODE_ID: usize =
    offset_of!(AssignmentTargetRest, node_id);
pub(crate) const OFFSET_ASSIGNMENT_TARGET_REST_SPAN: usize = offset_of!(AssignmentTargetRest, span);
pub(crate) const OFFSET_ASSIGNMENT_TARGET_REST_TARGET: usize =
    offset_of!(AssignmentTargetRest, target);

#[repr(transparent)]
#[derive(Clone, Copy, Debug)]
pub struct AssignmentTargetRestWithoutTarget<'a, 't>(
    pub(crate) *const AssignmentTargetRest<'a>,
    pub(crate) PhantomData<&'t ()>,
);

impl<'a, 't> AssignmentTargetRestWithoutTarget<'a, 't> {
    #[inline]
    pub fn node_id(self) -> &'t Cell<NodeId> {
        unsafe {
            &*((self.0 as *const u8).add(OFFSET_ASSIGNMENT_TARGET_REST_NODE_ID)
                as *const Cell<NodeId>)
        }
    }

    #[inline]
    pub fn span(self) -> &'t Span {
        unsafe { &*((self.0 as *const u8).add(OFFSET_ASSIGNMENT_TARGET_REST_SPAN) as *const Span) }
    }
}

impl<'a, 't> GetAddress for AssignmentTargetRestWithoutTarget<'a, 't> {
    #[inline]
    fn address(&self) -> Address {
        unsafe { Address::from_ptr(self.0) }
    }
}

pub(crate) const OFFSET_ASSIGNMENT_TARGET_WITH_DEFAULT_NODE_ID: usize =
    offset_of!(AssignmentTargetWithDefault, node_id);
pub(crate) const OFFSET_ASSIGNMENT_TARGET_WITH_DEFAULT_SPAN: usize =
    offset_of!(AssignmentTargetWithDefault, span);
pub(crate) const OFFSET_ASSIGNMENT_TARGET_WITH_DEFAULT_BINDING: usize =
    offset_of!(AssignmentTargetWithDefault, binding);
pub(crate) const OFFSET_ASSIGNMENT_TARGET_WITH_DEFAULT_INIT: usize =
    offset_of!(AssignmentTargetWithDefault, init);

#[repr(transparent)]
#[derive(Clone, Copy, Debug)]
pub struct AssignmentTargetWithDefaultWithoutBinding<'a, 't>(
    pub(crate) *const AssignmentTargetWithDefault<'a>,
    pub(crate) PhantomData<&'t ()>,
);

impl<'a, 't> AssignmentTargetWithDefaultWithoutBinding<'a, 't> {
    #[inline]
    pub fn node_id(self) -> &'t Cell<NodeId> {
        unsafe {
            &*((self.0 as *const u8).add(OFFSET_ASSIGNMENT_TARGET_WITH_DEFAULT_NODE_ID)
                as *const Cell<NodeId>)
        }
    }

    #[inline]
    pub fn span(self) -> &'t Span {
        unsafe {
            &*((self.0 as *const u8).add(OFFSET_ASSIGNMENT_TARGET_WITH_DEFAULT_SPAN) as *const Span)
        }
    }

    #[inline]
    pub fn init(self) -> &'t Expression<'a> {
        unsafe {
            &*((self.0 as *const u8).add(OFFSET_ASSIGNMENT_TARGET_WITH_DEFAULT_INIT)
                as *const Expression<'a>)
        }
    }
}

impl<'a, 't> GetAddress for AssignmentTargetWithDefaultWithoutBinding<'a, 't> {
    #[inline]
    fn address(&self) -> Address {
        unsafe { Address::from_ptr(self.0) }
    }
}

#[repr(transparent)]
#[derive(Clone, Copy, Debug)]
pub struct AssignmentTargetWithDefaultWithoutInit<'a, 't>(
    pub(crate) *const AssignmentTargetWithDefault<'a>,
    pub(crate) PhantomData<&'t ()>,
);

impl<'a, 't> AssignmentTargetWithDefaultWithoutInit<'a, 't> {
    #[inline]
    pub fn node_id(self) -> &'t Cell<NodeId> {
        unsafe {
            &*((self.0 as *const u8).add(OFFSET_ASSIGNMENT_TARGET_WITH_DEFAULT_NODE_ID)
                as *const Cell<NodeId>)
        }
    }

    #[inline]
    pub fn span(self) -> &'t Span {
        unsafe {
            &*((self.0 as *const u8).add(OFFSET_ASSIGNMENT_TARGET_WITH_DEFAULT_SPAN) as *const Span)
        }
    }

    #[inline]
    pub fn binding(self) -> &'t AssignmentTarget<'a> {
        unsafe {
            &*((self.0 as *const u8).add(OFFSET_ASSIGNMENT_TARGET_WITH_DEFAULT_BINDING)
                as *const AssignmentTarget<'a>)
        }
    }
}

impl<'a, 't> GetAddress for AssignmentTargetWithDefaultWithoutInit<'a, 't> {
    #[inline]
    fn address(&self) -> Address {
        unsafe { Address::from_ptr(self.0) }
    }
}

pub(crate) const OFFSET_ASSIGNMENT_TARGET_PROPERTY_IDENTIFIER_NODE_ID: usize =
    offset_of!(AssignmentTargetPropertyIdentifier, node_id);
pub(crate) const OFFSET_ASSIGNMENT_TARGET_PROPERTY_IDENTIFIER_SPAN: usize =
    offset_of!(AssignmentTargetPropertyIdentifier, span);
pub(crate) const OFFSET_ASSIGNMENT_TARGET_PROPERTY_IDENTIFIER_BINDING: usize =
    offset_of!(AssignmentTargetPropertyIdentifier, binding);
pub(crate) const OFFSET_ASSIGNMENT_TARGET_PROPERTY_IDENTIFIER_INIT: usize =
    offset_of!(AssignmentTargetPropertyIdentifier, init);

#[repr(transparent)]
#[derive(Clone, Copy, Debug)]
pub struct AssignmentTargetPropertyIdentifierWithoutBinding<'a, 't>(
    pub(crate) *const AssignmentTargetPropertyIdentifier<'a>,
    pub(crate) PhantomData<&'t ()>,
);

impl<'a, 't> AssignmentTargetPropertyIdentifierWithoutBinding<'a, 't> {
    #[inline]
    pub fn node_id(self) -> &'t Cell<NodeId> {
        unsafe {
            &*((self.0 as *const u8).add(OFFSET_ASSIGNMENT_TARGET_PROPERTY_IDENTIFIER_NODE_ID)
                as *const Cell<NodeId>)
        }
    }

    #[inline]
    pub fn span(self) -> &'t Span {
        unsafe {
            &*((self.0 as *const u8).add(OFFSET_ASSIGNMENT_TARGET_PROPERTY_IDENTIFIER_SPAN)
                as *const Span)
        }
    }

    #[inline]
    pub fn init(self) -> &'t Option<Expression<'a>> {
        unsafe {
            &*((self.0 as *const u8).add(OFFSET_ASSIGNMENT_TARGET_PROPERTY_IDENTIFIER_INIT)
                as *const Option<Expression<'a>>)
        }
    }
}

impl<'a, 't> GetAddress for AssignmentTargetPropertyIdentifierWithoutBinding<'a, 't> {
    #[inline]
    fn address(&self) -> Address {
        unsafe { Address::from_ptr(self.0) }
    }
}

#[repr(transparent)]
#[derive(Clone, Copy, Debug)]
pub struct AssignmentTargetPropertyIdentifierWithoutInit<'a, 't>(
    pub(crate) *const AssignmentTargetPropertyIdentifier<'a>,
    pub(crate) PhantomData<&'t ()>,
);

impl<'a, 't> AssignmentTargetPropertyIdentifierWithoutInit<'a, 't> {
    #[inline]
    pub fn node_id(self) -> &'t Cell<NodeId> {
        unsafe {
            &*((self.0 as *const u8).add(OFFSET_ASSIGNMENT_TARGET_PROPERTY_IDENTIFIER_NODE_ID)
                as *const Cell<NodeId>)
        }
    }

    #[inline]
    pub fn span(self) -> &'t Span {
        unsafe {
            &*((self.0 as *const u8).add(OFFSET_ASSIGNMENT_TARGET_PROPERTY_IDENTIFIER_SPAN)
                as *const Span)
        }
    }

    #[inline]
    pub fn binding(self) -> &'t IdentifierReference<'a> {
        unsafe {
            &*((self.0 as *const u8).add(OFFSET_ASSIGNMENT_TARGET_PROPERTY_IDENTIFIER_BINDING)
                as *const IdentifierReference<'a>)
        }
    }
}

impl<'a, 't> GetAddress for AssignmentTargetPropertyIdentifierWithoutInit<'a, 't> {
    #[inline]
    fn address(&self) -> Address {
        unsafe { Address::from_ptr(self.0) }
    }
}

pub(crate) const OFFSET_ASSIGNMENT_TARGET_PROPERTY_PROPERTY_NODE_ID: usize =
    offset_of!(AssignmentTargetPropertyProperty, node_id);
pub(crate) const OFFSET_ASSIGNMENT_TARGET_PROPERTY_PROPERTY_SPAN: usize =
    offset_of!(AssignmentTargetPropertyProperty, span);
pub(crate) const OFFSET_ASSIGNMENT_TARGET_PROPERTY_PROPERTY_NAME: usize =
    offset_of!(AssignmentTargetPropertyProperty, name);
pub(crate) const OFFSET_ASSIGNMENT_TARGET_PROPERTY_PROPERTY_BINDING: usize =
    offset_of!(AssignmentTargetPropertyProperty, binding);
pub(crate) const OFFSET_ASSIGNMENT_TARGET_PROPERTY_PROPERTY_COMPUTED: usize =
    offset_of!(AssignmentTargetPropertyProperty, computed);

#[repr(transparent)]
#[derive(Clone, Copy, Debug)]
pub struct AssignmentTargetPropertyPropertyWithoutName<'a, 't>(
    pub(crate) *const AssignmentTargetPropertyProperty<'a>,
    pub(crate) PhantomData<&'t ()>,
);

impl<'a, 't> AssignmentTargetPropertyPropertyWithoutName<'a, 't> {
    #[inline]
    pub fn node_id(self) -> &'t Cell<NodeId> {
        unsafe {
            &*((self.0 as *const u8).add(OFFSET_ASSIGNMENT_TARGET_PROPERTY_PROPERTY_NODE_ID)
                as *const Cell<NodeId>)
        }
    }

    #[inline]
    pub fn span(self) -> &'t Span {
        unsafe {
            &*((self.0 as *const u8).add(OFFSET_ASSIGNMENT_TARGET_PROPERTY_PROPERTY_SPAN)
                as *const Span)
        }
    }

    #[inline]
    pub fn binding(self) -> &'t AssignmentTargetMaybeDefault<'a> {
        unsafe {
            &*((self.0 as *const u8).add(OFFSET_ASSIGNMENT_TARGET_PROPERTY_PROPERTY_BINDING)
                as *const AssignmentTargetMaybeDefault<'a>)
        }
    }

    #[inline]
    pub fn computed(self) -> &'t bool {
        unsafe {
            &*((self.0 as *const u8).add(OFFSET_ASSIGNMENT_TARGET_PROPERTY_PROPERTY_COMPUTED)
                as *const bool)
        }
    }
}

impl<'a, 't> GetAddress for AssignmentTargetPropertyPropertyWithoutName<'a, 't> {
    #[inline]
    fn address(&self) -> Address {
        unsafe { Address::from_ptr(self.0) }
    }
}

#[repr(transparent)]
#[derive(Clone, Copy, Debug)]
pub struct AssignmentTargetPropertyPropertyWithoutBinding<'a, 't>(
    pub(crate) *const AssignmentTargetPropertyProperty<'a>,
    pub(crate) PhantomData<&'t ()>,
);

impl<'a, 't> AssignmentTargetPropertyPropertyWithoutBinding<'a, 't> {
    #[inline]
    pub fn node_id(self) -> &'t Cell<NodeId> {
        unsafe {
            &*((self.0 as *const u8).add(OFFSET_ASSIGNMENT_TARGET_PROPERTY_PROPERTY_NODE_ID)
                as *const Cell<NodeId>)
        }
    }

    #[inline]
    pub fn span(self) -> &'t Span {
        unsafe {
            &*((self.0 as *const u8).add(OFFSET_ASSIGNMENT_TARGET_PROPERTY_PROPERTY_SPAN)
                as *const Span)
        }
    }

    #[inline]
    pub fn name(self) -> &'t PropertyKey<'a> {
        unsafe {
            &*((self.0 as *const u8).add(OFFSET_ASSIGNMENT_TARGET_PROPERTY_PROPERTY_NAME)
                as *const PropertyKey<'a>)
        }
    }

    #[inline]
    pub fn computed(self) -> &'t bool {
        unsafe {
            &*((self.0 as *const u8).add(OFFSET_ASSIGNMENT_TARGET_PROPERTY_PROPERTY_COMPUTED)
                as *const bool)
        }
    }
}

impl<'a, 't> GetAddress for AssignmentTargetPropertyPropertyWithoutBinding<'a, 't> {
    #[inline]
    fn address(&self) -> Address {
        unsafe { Address::from_ptr(self.0) }
    }
}

pub(crate) const OFFSET_SEQUENCE_EXPRESSION_NODE_ID: usize =
    offset_of!(SequenceExpression, node_id);
pub(crate) const OFFSET_SEQUENCE_EXPRESSION_SPAN: usize = offset_of!(SequenceExpression, span);
pub(crate) const OFFSET_SEQUENCE_EXPRESSION_EXPRESSIONS: usize =
    offset_of!(SequenceExpression, expressions);

#[repr(transparent)]
#[derive(Clone, Copy, Debug)]
pub struct SequenceExpressionWithoutExpressions<'a, 't>(
    pub(crate) *const SequenceExpression<'a>,
    pub(crate) PhantomData<&'t ()>,
);

impl<'a, 't> SequenceExpressionWithoutExpressions<'a, 't> {
    #[inline]
    pub fn node_id(self) -> &'t Cell<NodeId> {
        unsafe {
            &*((self.0 as *const u8).add(OFFSET_SEQUENCE_EXPRESSION_NODE_ID) as *const Cell<NodeId>)
        }
    }

    #[inline]
    pub fn span(self) -> &'t Span {
        unsafe { &*((self.0 as *const u8).add(OFFSET_SEQUENCE_EXPRESSION_SPAN) as *const Span) }
    }
}

impl<'a, 't> GetAddress for SequenceExpressionWithoutExpressions<'a, 't> {
    #[inline]
    fn address(&self) -> Address {
        unsafe { Address::from_ptr(self.0) }
    }
}

pub(crate) const OFFSET_AWAIT_EXPRESSION_NODE_ID: usize = offset_of!(AwaitExpression, node_id);
pub(crate) const OFFSET_AWAIT_EXPRESSION_SPAN: usize = offset_of!(AwaitExpression, span);
pub(crate) const OFFSET_AWAIT_EXPRESSION_ARGUMENT: usize = offset_of!(AwaitExpression, argument);

#[repr(transparent)]
#[derive(Clone, Copy, Debug)]
pub struct AwaitExpressionWithoutArgument<'a, 't>(
    pub(crate) *const AwaitExpression<'a>,
    pub(crate) PhantomData<&'t ()>,
);

impl<'a, 't> AwaitExpressionWithoutArgument<'a, 't> {
    #[inline]
    pub fn node_id(self) -> &'t Cell<NodeId> {
        unsafe {
            &*((self.0 as *const u8).add(OFFSET_AWAIT_EXPRESSION_NODE_ID) as *const Cell<NodeId>)
        }
    }

    #[inline]
    pub fn span(self) -> &'t Span {
        unsafe { &*((self.0 as *const u8).add(OFFSET_AWAIT_EXPRESSION_SPAN) as *const Span) }
    }
}

impl<'a, 't> GetAddress for AwaitExpressionWithoutArgument<'a, 't> {
    #[inline]
    fn address(&self) -> Address {
        unsafe { Address::from_ptr(self.0) }
    }
}

pub(crate) const OFFSET_CHAIN_EXPRESSION_NODE_ID: usize = offset_of!(ChainExpression, node_id);
pub(crate) const OFFSET_CHAIN_EXPRESSION_SPAN: usize = offset_of!(ChainExpression, span);
pub(crate) const OFFSET_CHAIN_EXPRESSION_EXPRESSION: usize =
    offset_of!(ChainExpression, expression);

#[repr(transparent)]
#[derive(Clone, Copy, Debug)]
pub struct ChainExpressionWithoutExpression<'a, 't>(
    pub(crate) *const ChainExpression<'a>,
    pub(crate) PhantomData<&'t ()>,
);

impl<'a, 't> ChainExpressionWithoutExpression<'a, 't> {
    #[inline]
    pub fn node_id(self) -> &'t Cell<NodeId> {
        unsafe {
            &*((self.0 as *const u8).add(OFFSET_CHAIN_EXPRESSION_NODE_ID) as *const Cell<NodeId>)
        }
    }

    #[inline]
    pub fn span(self) -> &'t Span {
        unsafe { &*((self.0 as *const u8).add(OFFSET_CHAIN_EXPRESSION_SPAN) as *const Span) }
    }
}

impl<'a, 't> GetAddress for ChainExpressionWithoutExpression<'a, 't> {
    #[inline]
    fn address(&self) -> Address {
        unsafe { Address::from_ptr(self.0) }
    }
}

pub(crate) const OFFSET_PARENTHESIZED_EXPRESSION_NODE_ID: usize =
    offset_of!(ParenthesizedExpression, node_id);
pub(crate) const OFFSET_PARENTHESIZED_EXPRESSION_SPAN: usize =
    offset_of!(ParenthesizedExpression, span);
pub(crate) const OFFSET_PARENTHESIZED_EXPRESSION_EXPRESSION: usize =
    offset_of!(ParenthesizedExpression, expression);

#[repr(transparent)]
#[derive(Clone, Copy, Debug)]
pub struct ParenthesizedExpressionWithoutExpression<'a, 't>(
    pub(crate) *const ParenthesizedExpression<'a>,
    pub(crate) PhantomData<&'t ()>,
);

impl<'a, 't> ParenthesizedExpressionWithoutExpression<'a, 't> {
    #[inline]
    pub fn node_id(self) -> &'t Cell<NodeId> {
        unsafe {
            &*((self.0 as *const u8).add(OFFSET_PARENTHESIZED_EXPRESSION_NODE_ID)
                as *const Cell<NodeId>)
        }
    }

    #[inline]
    pub fn span(self) -> &'t Span {
        unsafe {
            &*((self.0 as *const u8).add(OFFSET_PARENTHESIZED_EXPRESSION_SPAN) as *const Span)
        }
    }
}

impl<'a, 't> GetAddress for ParenthesizedExpressionWithoutExpression<'a, 't> {
    #[inline]
    fn address(&self) -> Address {
        unsafe { Address::from_ptr(self.0) }
    }
}

pub(crate) const OFFSET_DIRECTIVE_NODE_ID: usize = offset_of!(Directive, node_id);
pub(crate) const OFFSET_DIRECTIVE_SPAN: usize = offset_of!(Directive, span);
pub(crate) const OFFSET_DIRECTIVE_EXPRESSION: usize = offset_of!(Directive, expression);
pub(crate) const OFFSET_DIRECTIVE_DIRECTIVE: usize = offset_of!(Directive, directive);

#[repr(transparent)]
#[derive(Clone, Copy, Debug)]
pub struct DirectiveWithoutExpression<'a, 't>(
    pub(crate) *const Directive<'a>,
    pub(crate) PhantomData<&'t ()>,
);

impl<'a, 't> DirectiveWithoutExpression<'a, 't> {
    #[inline]
    pub fn node_id(self) -> &'t Cell<NodeId> {
        unsafe { &*((self.0 as *const u8).add(OFFSET_DIRECTIVE_NODE_ID) as *const Cell<NodeId>) }
    }

    #[inline]
    pub fn span(self) -> &'t Span {
        unsafe { &*((self.0 as *const u8).add(OFFSET_DIRECTIVE_SPAN) as *const Span) }
    }

    #[inline]
    pub fn directive(self) -> &'t Str<'a> {
        unsafe { &*((self.0 as *const u8).add(OFFSET_DIRECTIVE_DIRECTIVE) as *const Str<'a>) }
    }
}

impl<'a, 't> GetAddress for DirectiveWithoutExpression<'a, 't> {
    #[inline]
    fn address(&self) -> Address {
        unsafe { Address::from_ptr(self.0) }
    }
}

pub(crate) const OFFSET_BLOCK_STATEMENT_NODE_ID: usize = offset_of!(BlockStatement, node_id);
pub(crate) const OFFSET_BLOCK_STATEMENT_SPAN: usize = offset_of!(BlockStatement, span);
pub(crate) const OFFSET_BLOCK_STATEMENT_BODY: usize = offset_of!(BlockStatement, body);
pub(crate) const OFFSET_BLOCK_STATEMENT_SCOPE_ID: usize = offset_of!(BlockStatement, scope_id);

#[repr(transparent)]
#[derive(Clone, Copy, Debug)]
pub struct BlockStatementWithoutBody<'a, 't>(
    pub(crate) *const BlockStatement<'a>,
    pub(crate) PhantomData<&'t ()>,
);

impl<'a, 't> BlockStatementWithoutBody<'a, 't> {
    #[inline]
    pub fn node_id(self) -> &'t Cell<NodeId> {
        unsafe {
            &*((self.0 as *const u8).add(OFFSET_BLOCK_STATEMENT_NODE_ID) as *const Cell<NodeId>)
        }
    }

    #[inline]
    pub fn span(self) -> &'t Span {
        unsafe { &*((self.0 as *const u8).add(OFFSET_BLOCK_STATEMENT_SPAN) as *const Span) }
    }

    #[inline]
    pub fn scope_id(self) -> &'t Cell<Option<ScopeId>> {
        unsafe {
            &*((self.0 as *const u8).add(OFFSET_BLOCK_STATEMENT_SCOPE_ID)
                as *const Cell<Option<ScopeId>>)
        }
    }
}

impl<'a, 't> GetAddress for BlockStatementWithoutBody<'a, 't> {
    #[inline]
    fn address(&self) -> Address {
        unsafe { Address::from_ptr(self.0) }
    }
}

pub(crate) const OFFSET_VARIABLE_DECLARATION_NODE_ID: usize =
    offset_of!(VariableDeclaration, node_id);
pub(crate) const OFFSET_VARIABLE_DECLARATION_SPAN: usize = offset_of!(VariableDeclaration, span);
pub(crate) const OFFSET_VARIABLE_DECLARATION_KIND: usize = offset_of!(VariableDeclaration, kind);
pub(crate) const OFFSET_VARIABLE_DECLARATION_DECLARATIONS: usize =
    offset_of!(VariableDeclaration, declarations);
pub(crate) const OFFSET_VARIABLE_DECLARATION_DECLARE: usize =
    offset_of!(VariableDeclaration, declare);

#[repr(transparent)]
#[derive(Clone, Copy, Debug)]
pub struct VariableDeclarationWithoutDeclarations<'a, 't>(
    pub(crate) *const VariableDeclaration<'a>,
    pub(crate) PhantomData<&'t ()>,
);

impl<'a, 't> VariableDeclarationWithoutDeclarations<'a, 't> {
    #[inline]
    pub fn node_id(self) -> &'t Cell<NodeId> {
        unsafe {
            &*((self.0 as *const u8).add(OFFSET_VARIABLE_DECLARATION_NODE_ID)
                as *const Cell<NodeId>)
        }
    }

    #[inline]
    pub fn span(self) -> &'t Span {
        unsafe { &*((self.0 as *const u8).add(OFFSET_VARIABLE_DECLARATION_SPAN) as *const Span) }
    }

    #[inline]
    pub fn kind(self) -> &'t VariableDeclarationKind {
        unsafe {
            &*((self.0 as *const u8).add(OFFSET_VARIABLE_DECLARATION_KIND)
                as *const VariableDeclarationKind)
        }
    }

    #[inline]
    pub fn declare(self) -> &'t bool {
        unsafe { &*((self.0 as *const u8).add(OFFSET_VARIABLE_DECLARATION_DECLARE) as *const bool) }
    }
}

impl<'a, 't> GetAddress for VariableDeclarationWithoutDeclarations<'a, 't> {
    #[inline]
    fn address(&self) -> Address {
        unsafe { Address::from_ptr(self.0) }
    }
}

pub(crate) const OFFSET_VARIABLE_DECLARATOR_NODE_ID: usize =
    offset_of!(VariableDeclarator, node_id);
pub(crate) const OFFSET_VARIABLE_DECLARATOR_SPAN: usize = offset_of!(VariableDeclarator, span);
pub(crate) const OFFSET_VARIABLE_DECLARATOR_KIND: usize = offset_of!(VariableDeclarator, kind);
pub(crate) const OFFSET_VARIABLE_DECLARATOR_ID: usize = offset_of!(VariableDeclarator, id);
pub(crate) const OFFSET_VARIABLE_DECLARATOR_TYPE_ANNOTATION: usize =
    offset_of!(VariableDeclarator, type_annotation);
pub(crate) const OFFSET_VARIABLE_DECLARATOR_INIT: usize = offset_of!(VariableDeclarator, init);
pub(crate) const OFFSET_VARIABLE_DECLARATOR_DEFINITE: usize =
    offset_of!(VariableDeclarator, definite);

#[repr(transparent)]
#[derive(Clone, Copy, Debug)]
pub struct VariableDeclaratorWithoutId<'a, 't>(
    pub(crate) *const VariableDeclarator<'a>,
    pub(crate) PhantomData<&'t ()>,
);

impl<'a, 't> VariableDeclaratorWithoutId<'a, 't> {
    #[inline]
    pub fn node_id(self) -> &'t Cell<NodeId> {
        unsafe {
            &*((self.0 as *const u8).add(OFFSET_VARIABLE_DECLARATOR_NODE_ID) as *const Cell<NodeId>)
        }
    }

    #[inline]
    pub fn span(self) -> &'t Span {
        unsafe { &*((self.0 as *const u8).add(OFFSET_VARIABLE_DECLARATOR_SPAN) as *const Span) }
    }

    #[inline]
    pub fn kind(self) -> &'t VariableDeclarationKind {
        unsafe {
            &*((self.0 as *const u8).add(OFFSET_VARIABLE_DECLARATOR_KIND)
                as *const VariableDeclarationKind)
        }
    }

    #[inline]
    pub fn type_annotation(self) -> &'t Option<ArenaBox<'a, TSTypeAnnotation<'a>>> {
        unsafe {
            &*((self.0 as *const u8).add(OFFSET_VARIABLE_DECLARATOR_TYPE_ANNOTATION)
                as *const Option<ArenaBox<'a, TSTypeAnnotation<'a>>>)
        }
    }

    #[inline]
    pub fn init(self) -> &'t Option<Expression<'a>> {
        unsafe {
            &*((self.0 as *const u8).add(OFFSET_VARIABLE_DECLARATOR_INIT)
                as *const Option<Expression<'a>>)
        }
    }

    #[inline]
    pub fn definite(self) -> &'t bool {
        unsafe { &*((self.0 as *const u8).add(OFFSET_VARIABLE_DECLARATOR_DEFINITE) as *const bool) }
    }
}

impl<'a, 't> GetAddress for VariableDeclaratorWithoutId<'a, 't> {
    #[inline]
    fn address(&self) -> Address {
        unsafe { Address::from_ptr(self.0) }
    }
}

#[repr(transparent)]
#[derive(Clone, Copy, Debug)]
pub struct VariableDeclaratorWithoutInit<'a, 't>(
    pub(crate) *const VariableDeclarator<'a>,
    pub(crate) PhantomData<&'t ()>,
);

impl<'a, 't> VariableDeclaratorWithoutInit<'a, 't> {
    #[inline]
    pub fn node_id(self) -> &'t Cell<NodeId> {
        unsafe {
            &*((self.0 as *const u8).add(OFFSET_VARIABLE_DECLARATOR_NODE_ID) as *const Cell<NodeId>)
        }
    }

    #[inline]
    pub fn span(self) -> &'t Span {
        unsafe { &*((self.0 as *const u8).add(OFFSET_VARIABLE_DECLARATOR_SPAN) as *const Span) }
    }

    #[inline]
    pub fn kind(self) -> &'t VariableDeclarationKind {
        unsafe {
            &*((self.0 as *const u8).add(OFFSET_VARIABLE_DECLARATOR_KIND)
                as *const VariableDeclarationKind)
        }
    }

    #[inline]
    pub fn id(self) -> &'t BindingPattern<'a> {
        unsafe {
            &*((self.0 as *const u8).add(OFFSET_VARIABLE_DECLARATOR_ID)
                as *const BindingPattern<'a>)
        }
    }

    #[inline]
    pub fn type_annotation(self) -> &'t Option<ArenaBox<'a, TSTypeAnnotation<'a>>> {
        unsafe {
            &*((self.0 as *const u8).add(OFFSET_VARIABLE_DECLARATOR_TYPE_ANNOTATION)
                as *const Option<ArenaBox<'a, TSTypeAnnotation<'a>>>)
        }
    }

    #[inline]
    pub fn definite(self) -> &'t bool {
        unsafe { &*((self.0 as *const u8).add(OFFSET_VARIABLE_DECLARATOR_DEFINITE) as *const bool) }
    }
}

impl<'a, 't> GetAddress for VariableDeclaratorWithoutInit<'a, 't> {
    #[inline]
    fn address(&self) -> Address {
        unsafe { Address::from_ptr(self.0) }
    }
}

pub(crate) const OFFSET_EXPRESSION_STATEMENT_NODE_ID: usize =
    offset_of!(ExpressionStatement, node_id);
pub(crate) const OFFSET_EXPRESSION_STATEMENT_SPAN: usize = offset_of!(ExpressionStatement, span);
pub(crate) const OFFSET_EXPRESSION_STATEMENT_EXPRESSION: usize =
    offset_of!(ExpressionStatement, expression);

#[repr(transparent)]
#[derive(Clone, Copy, Debug)]
pub struct ExpressionStatementWithoutExpression<'a, 't>(
    pub(crate) *const ExpressionStatement<'a>,
    pub(crate) PhantomData<&'t ()>,
);

impl<'a, 't> ExpressionStatementWithoutExpression<'a, 't> {
    #[inline]
    pub fn node_id(self) -> &'t Cell<NodeId> {
        unsafe {
            &*((self.0 as *const u8).add(OFFSET_EXPRESSION_STATEMENT_NODE_ID)
                as *const Cell<NodeId>)
        }
    }

    #[inline]
    pub fn span(self) -> &'t Span {
        unsafe { &*((self.0 as *const u8).add(OFFSET_EXPRESSION_STATEMENT_SPAN) as *const Span) }
    }
}

impl<'a, 't> GetAddress for ExpressionStatementWithoutExpression<'a, 't> {
    #[inline]
    fn address(&self) -> Address {
        unsafe { Address::from_ptr(self.0) }
    }
}

pub(crate) const OFFSET_IF_STATEMENT_NODE_ID: usize = offset_of!(IfStatement, node_id);
pub(crate) const OFFSET_IF_STATEMENT_SPAN: usize = offset_of!(IfStatement, span);
pub(crate) const OFFSET_IF_STATEMENT_TEST: usize = offset_of!(IfStatement, test);
pub(crate) const OFFSET_IF_STATEMENT_CONSEQUENT: usize = offset_of!(IfStatement, consequent);
pub(crate) const OFFSET_IF_STATEMENT_ALTERNATE: usize = offset_of!(IfStatement, alternate);

#[repr(transparent)]
#[derive(Clone, Copy, Debug)]
pub struct IfStatementWithoutTest<'a, 't>(
    pub(crate) *const IfStatement<'a>,
    pub(crate) PhantomData<&'t ()>,
);

impl<'a, 't> IfStatementWithoutTest<'a, 't> {
    #[inline]
    pub fn node_id(self) -> &'t Cell<NodeId> {
        unsafe { &*((self.0 as *const u8).add(OFFSET_IF_STATEMENT_NODE_ID) as *const Cell<NodeId>) }
    }

    #[inline]
    pub fn span(self) -> &'t Span {
        unsafe { &*((self.0 as *const u8).add(OFFSET_IF_STATEMENT_SPAN) as *const Span) }
    }

    #[inline]
    pub fn consequent(self) -> &'t Statement<'a> {
        unsafe {
            &*((self.0 as *const u8).add(OFFSET_IF_STATEMENT_CONSEQUENT) as *const Statement<'a>)
        }
    }

    #[inline]
    pub fn alternate(self) -> &'t Option<Statement<'a>> {
        unsafe {
            &*((self.0 as *const u8).add(OFFSET_IF_STATEMENT_ALTERNATE)
                as *const Option<Statement<'a>>)
        }
    }
}

impl<'a, 't> GetAddress for IfStatementWithoutTest<'a, 't> {
    #[inline]
    fn address(&self) -> Address {
        unsafe { Address::from_ptr(self.0) }
    }
}

#[repr(transparent)]
#[derive(Clone, Copy, Debug)]
pub struct IfStatementWithoutConsequent<'a, 't>(
    pub(crate) *const IfStatement<'a>,
    pub(crate) PhantomData<&'t ()>,
);

impl<'a, 't> IfStatementWithoutConsequent<'a, 't> {
    #[inline]
    pub fn node_id(self) -> &'t Cell<NodeId> {
        unsafe { &*((self.0 as *const u8).add(OFFSET_IF_STATEMENT_NODE_ID) as *const Cell<NodeId>) }
    }

    #[inline]
    pub fn span(self) -> &'t Span {
        unsafe { &*((self.0 as *const u8).add(OFFSET_IF_STATEMENT_SPAN) as *const Span) }
    }

    #[inline]
    pub fn test(self) -> &'t Expression<'a> {
        unsafe { &*((self.0 as *const u8).add(OFFSET_IF_STATEMENT_TEST) as *const Expression<'a>) }
    }

    #[inline]
    pub fn alternate(self) -> &'t Option<Statement<'a>> {
        unsafe {
            &*((self.0 as *const u8).add(OFFSET_IF_STATEMENT_ALTERNATE)
                as *const Option<Statement<'a>>)
        }
    }
}

impl<'a, 't> GetAddress for IfStatementWithoutConsequent<'a, 't> {
    #[inline]
    fn address(&self) -> Address {
        unsafe { Address::from_ptr(self.0) }
    }
}

#[repr(transparent)]
#[derive(Clone, Copy, Debug)]
pub struct IfStatementWithoutAlternate<'a, 't>(
    pub(crate) *const IfStatement<'a>,
    pub(crate) PhantomData<&'t ()>,
);

impl<'a, 't> IfStatementWithoutAlternate<'a, 't> {
    #[inline]
    pub fn node_id(self) -> &'t Cell<NodeId> {
        unsafe { &*((self.0 as *const u8).add(OFFSET_IF_STATEMENT_NODE_ID) as *const Cell<NodeId>) }
    }

    #[inline]
    pub fn span(self) -> &'t Span {
        unsafe { &*((self.0 as *const u8).add(OFFSET_IF_STATEMENT_SPAN) as *const Span) }
    }

    #[inline]
    pub fn test(self) -> &'t Expression<'a> {
        unsafe { &*((self.0 as *const u8).add(OFFSET_IF_STATEMENT_TEST) as *const Expression<'a>) }
    }

    #[inline]
    pub fn consequent(self) -> &'t Statement<'a> {
        unsafe {
            &*((self.0 as *const u8).add(OFFSET_IF_STATEMENT_CONSEQUENT) as *const Statement<'a>)
        }
    }
}

impl<'a, 't> GetAddress for IfStatementWithoutAlternate<'a, 't> {
    #[inline]
    fn address(&self) -> Address {
        unsafe { Address::from_ptr(self.0) }
    }
}

pub(crate) const OFFSET_DO_WHILE_STATEMENT_NODE_ID: usize = offset_of!(DoWhileStatement, node_id);
pub(crate) const OFFSET_DO_WHILE_STATEMENT_SPAN: usize = offset_of!(DoWhileStatement, span);
pub(crate) const OFFSET_DO_WHILE_STATEMENT_BODY: usize = offset_of!(DoWhileStatement, body);
pub(crate) const OFFSET_DO_WHILE_STATEMENT_TEST: usize = offset_of!(DoWhileStatement, test);

#[repr(transparent)]
#[derive(Clone, Copy, Debug)]
pub struct DoWhileStatementWithoutBody<'a, 't>(
    pub(crate) *const DoWhileStatement<'a>,
    pub(crate) PhantomData<&'t ()>,
);

impl<'a, 't> DoWhileStatementWithoutBody<'a, 't> {
    #[inline]
    pub fn node_id(self) -> &'t Cell<NodeId> {
        unsafe {
            &*((self.0 as *const u8).add(OFFSET_DO_WHILE_STATEMENT_NODE_ID) as *const Cell<NodeId>)
        }
    }

    #[inline]
    pub fn span(self) -> &'t Span {
        unsafe { &*((self.0 as *const u8).add(OFFSET_DO_WHILE_STATEMENT_SPAN) as *const Span) }
    }

    #[inline]
    pub fn test(self) -> &'t Expression<'a> {
        unsafe {
            &*((self.0 as *const u8).add(OFFSET_DO_WHILE_STATEMENT_TEST) as *const Expression<'a>)
        }
    }
}

impl<'a, 't> GetAddress for DoWhileStatementWithoutBody<'a, 't> {
    #[inline]
    fn address(&self) -> Address {
        unsafe { Address::from_ptr(self.0) }
    }
}

#[repr(transparent)]
#[derive(Clone, Copy, Debug)]
pub struct DoWhileStatementWithoutTest<'a, 't>(
    pub(crate) *const DoWhileStatement<'a>,
    pub(crate) PhantomData<&'t ()>,
);

impl<'a, 't> DoWhileStatementWithoutTest<'a, 't> {
    #[inline]
    pub fn node_id(self) -> &'t Cell<NodeId> {
        unsafe {
            &*((self.0 as *const u8).add(OFFSET_DO_WHILE_STATEMENT_NODE_ID) as *const Cell<NodeId>)
        }
    }

    #[inline]
    pub fn span(self) -> &'t Span {
        unsafe { &*((self.0 as *const u8).add(OFFSET_DO_WHILE_STATEMENT_SPAN) as *const Span) }
    }

    #[inline]
    pub fn body(self) -> &'t Statement<'a> {
        unsafe {
            &*((self.0 as *const u8).add(OFFSET_DO_WHILE_STATEMENT_BODY) as *const Statement<'a>)
        }
    }
}

impl<'a, 't> GetAddress for DoWhileStatementWithoutTest<'a, 't> {
    #[inline]
    fn address(&self) -> Address {
        unsafe { Address::from_ptr(self.0) }
    }
}

pub(crate) const OFFSET_WHILE_STATEMENT_NODE_ID: usize = offset_of!(WhileStatement, node_id);
pub(crate) const OFFSET_WHILE_STATEMENT_SPAN: usize = offset_of!(WhileStatement, span);
pub(crate) const OFFSET_WHILE_STATEMENT_TEST: usize = offset_of!(WhileStatement, test);
pub(crate) const OFFSET_WHILE_STATEMENT_BODY: usize = offset_of!(WhileStatement, body);

#[repr(transparent)]
#[derive(Clone, Copy, Debug)]
pub struct WhileStatementWithoutTest<'a, 't>(
    pub(crate) *const WhileStatement<'a>,
    pub(crate) PhantomData<&'t ()>,
);

impl<'a, 't> WhileStatementWithoutTest<'a, 't> {
    #[inline]
    pub fn node_id(self) -> &'t Cell<NodeId> {
        unsafe {
            &*((self.0 as *const u8).add(OFFSET_WHILE_STATEMENT_NODE_ID) as *const Cell<NodeId>)
        }
    }

    #[inline]
    pub fn span(self) -> &'t Span {
        unsafe { &*((self.0 as *const u8).add(OFFSET_WHILE_STATEMENT_SPAN) as *const Span) }
    }

    #[inline]
    pub fn body(self) -> &'t Statement<'a> {
        unsafe {
            &*((self.0 as *const u8).add(OFFSET_WHILE_STATEMENT_BODY) as *const Statement<'a>)
        }
    }
}

impl<'a, 't> GetAddress for WhileStatementWithoutTest<'a, 't> {
    #[inline]
    fn address(&self) -> Address {
        unsafe { Address::from_ptr(self.0) }
    }
}

#[repr(transparent)]
#[derive(Clone, Copy, Debug)]
pub struct WhileStatementWithoutBody<'a, 't>(
    pub(crate) *const WhileStatement<'a>,
    pub(crate) PhantomData<&'t ()>,
);

impl<'a, 't> WhileStatementWithoutBody<'a, 't> {
    #[inline]
    pub fn node_id(self) -> &'t Cell<NodeId> {
        unsafe {
            &*((self.0 as *const u8).add(OFFSET_WHILE_STATEMENT_NODE_ID) as *const Cell<NodeId>)
        }
    }

    #[inline]
    pub fn span(self) -> &'t Span {
        unsafe { &*((self.0 as *const u8).add(OFFSET_WHILE_STATEMENT_SPAN) as *const Span) }
    }

    #[inline]
    pub fn test(self) -> &'t Expression<'a> {
        unsafe {
            &*((self.0 as *const u8).add(OFFSET_WHILE_STATEMENT_TEST) as *const Expression<'a>)
        }
    }
}

impl<'a, 't> GetAddress for WhileStatementWithoutBody<'a, 't> {
    #[inline]
    fn address(&self) -> Address {
        unsafe { Address::from_ptr(self.0) }
    }
}

pub(crate) const OFFSET_FOR_STATEMENT_NODE_ID: usize = offset_of!(ForStatement, node_id);
pub(crate) const OFFSET_FOR_STATEMENT_SPAN: usize = offset_of!(ForStatement, span);
pub(crate) const OFFSET_FOR_STATEMENT_INIT: usize = offset_of!(ForStatement, init);
pub(crate) const OFFSET_FOR_STATEMENT_TEST: usize = offset_of!(ForStatement, test);
pub(crate) const OFFSET_FOR_STATEMENT_UPDATE: usize = offset_of!(ForStatement, update);
pub(crate) const OFFSET_FOR_STATEMENT_BODY: usize = offset_of!(ForStatement, body);
pub(crate) const OFFSET_FOR_STATEMENT_SCOPE_ID: usize = offset_of!(ForStatement, scope_id);

#[repr(transparent)]
#[derive(Clone, Copy, Debug)]
pub struct ForStatementWithoutInit<'a, 't>(
    pub(crate) *const ForStatement<'a>,
    pub(crate) PhantomData<&'t ()>,
);

impl<'a, 't> ForStatementWithoutInit<'a, 't> {
    #[inline]
    pub fn node_id(self) -> &'t Cell<NodeId> {
        unsafe {
            &*((self.0 as *const u8).add(OFFSET_FOR_STATEMENT_NODE_ID) as *const Cell<NodeId>)
        }
    }

    #[inline]
    pub fn span(self) -> &'t Span {
        unsafe { &*((self.0 as *const u8).add(OFFSET_FOR_STATEMENT_SPAN) as *const Span) }
    }

    #[inline]
    pub fn test(self) -> &'t Option<Expression<'a>> {
        unsafe {
            &*((self.0 as *const u8).add(OFFSET_FOR_STATEMENT_TEST)
                as *const Option<Expression<'a>>)
        }
    }

    #[inline]
    pub fn update(self) -> &'t Option<Expression<'a>> {
        unsafe {
            &*((self.0 as *const u8).add(OFFSET_FOR_STATEMENT_UPDATE)
                as *const Option<Expression<'a>>)
        }
    }

    #[inline]
    pub fn body(self) -> &'t Statement<'a> {
        unsafe { &*((self.0 as *const u8).add(OFFSET_FOR_STATEMENT_BODY) as *const Statement<'a>) }
    }

    #[inline]
    pub fn scope_id(self) -> &'t Cell<Option<ScopeId>> {
        unsafe {
            &*((self.0 as *const u8).add(OFFSET_FOR_STATEMENT_SCOPE_ID)
                as *const Cell<Option<ScopeId>>)
        }
    }
}

impl<'a, 't> GetAddress for ForStatementWithoutInit<'a, 't> {
    #[inline]
    fn address(&self) -> Address {
        unsafe { Address::from_ptr(self.0) }
    }
}

#[repr(transparent)]
#[derive(Clone, Copy, Debug)]
pub struct ForStatementWithoutTest<'a, 't>(
    pub(crate) *const ForStatement<'a>,
    pub(crate) PhantomData<&'t ()>,
);

impl<'a, 't> ForStatementWithoutTest<'a, 't> {
    #[inline]
    pub fn node_id(self) -> &'t Cell<NodeId> {
        unsafe {
            &*((self.0 as *const u8).add(OFFSET_FOR_STATEMENT_NODE_ID) as *const Cell<NodeId>)
        }
    }

    #[inline]
    pub fn span(self) -> &'t Span {
        unsafe { &*((self.0 as *const u8).add(OFFSET_FOR_STATEMENT_SPAN) as *const Span) }
    }

    #[inline]
    pub fn init(self) -> &'t Option<ForStatementInit<'a>> {
        unsafe {
            &*((self.0 as *const u8).add(OFFSET_FOR_STATEMENT_INIT)
                as *const Option<ForStatementInit<'a>>)
        }
    }

    #[inline]
    pub fn update(self) -> &'t Option<Expression<'a>> {
        unsafe {
            &*((self.0 as *const u8).add(OFFSET_FOR_STATEMENT_UPDATE)
                as *const Option<Expression<'a>>)
        }
    }

    #[inline]
    pub fn body(self) -> &'t Statement<'a> {
        unsafe { &*((self.0 as *const u8).add(OFFSET_FOR_STATEMENT_BODY) as *const Statement<'a>) }
    }

    #[inline]
    pub fn scope_id(self) -> &'t Cell<Option<ScopeId>> {
        unsafe {
            &*((self.0 as *const u8).add(OFFSET_FOR_STATEMENT_SCOPE_ID)
                as *const Cell<Option<ScopeId>>)
        }
    }
}

impl<'a, 't> GetAddress for ForStatementWithoutTest<'a, 't> {
    #[inline]
    fn address(&self) -> Address {
        unsafe { Address::from_ptr(self.0) }
    }
}

#[repr(transparent)]
#[derive(Clone, Copy, Debug)]
pub struct ForStatementWithoutUpdate<'a, 't>(
    pub(crate) *const ForStatement<'a>,
    pub(crate) PhantomData<&'t ()>,
);

impl<'a, 't> ForStatementWithoutUpdate<'a, 't> {
    #[inline]
    pub fn node_id(self) -> &'t Cell<NodeId> {
        unsafe {
            &*((self.0 as *const u8).add(OFFSET_FOR_STATEMENT_NODE_ID) as *const Cell<NodeId>)
        }
    }

    #[inline]
    pub fn span(self) -> &'t Span {
        unsafe { &*((self.0 as *const u8).add(OFFSET_FOR_STATEMENT_SPAN) as *const Span) }
    }

    #[inline]
    pub fn init(self) -> &'t Option<ForStatementInit<'a>> {
        unsafe {
            &*((self.0 as *const u8).add(OFFSET_FOR_STATEMENT_INIT)
                as *const Option<ForStatementInit<'a>>)
        }
    }

    #[inline]
    pub fn test(self) -> &'t Option<Expression<'a>> {
        unsafe {
            &*((self.0 as *const u8).add(OFFSET_FOR_STATEMENT_TEST)
                as *const Option<Expression<'a>>)
        }
    }

    #[inline]
    pub fn body(self) -> &'t Statement<'a> {
        unsafe { &*((self.0 as *const u8).add(OFFSET_FOR_STATEMENT_BODY) as *const Statement<'a>) }
    }

    #[inline]
    pub fn scope_id(self) -> &'t Cell<Option<ScopeId>> {
        unsafe {
            &*((self.0 as *const u8).add(OFFSET_FOR_STATEMENT_SCOPE_ID)
                as *const Cell<Option<ScopeId>>)
        }
    }
}

impl<'a, 't> GetAddress for ForStatementWithoutUpdate<'a, 't> {
    #[inline]
    fn address(&self) -> Address {
        unsafe { Address::from_ptr(self.0) }
    }
}

#[repr(transparent)]
#[derive(Clone, Copy, Debug)]
pub struct ForStatementWithoutBody<'a, 't>(
    pub(crate) *const ForStatement<'a>,
    pub(crate) PhantomData<&'t ()>,
);

impl<'a, 't> ForStatementWithoutBody<'a, 't> {
    #[inline]
    pub fn node_id(self) -> &'t Cell<NodeId> {
        unsafe {
            &*((self.0 as *const u8).add(OFFSET_FOR_STATEMENT_NODE_ID) as *const Cell<NodeId>)
        }
    }

    #[inline]
    pub fn span(self) -> &'t Span {
        unsafe { &*((self.0 as *const u8).add(OFFSET_FOR_STATEMENT_SPAN) as *const Span) }
    }

    #[inline]
    pub fn init(self) -> &'t Option<ForStatementInit<'a>> {
        unsafe {
            &*((self.0 as *const u8).add(OFFSET_FOR_STATEMENT_INIT)
                as *const Option<ForStatementInit<'a>>)
        }
    }

    #[inline]
    pub fn test(self) -> &'t Option<Expression<'a>> {
        unsafe {
            &*((self.0 as *const u8).add(OFFSET_FOR_STATEMENT_TEST)
                as *const Option<Expression<'a>>)
        }
    }

    #[inline]
    pub fn update(self) -> &'t Option<Expression<'a>> {
        unsafe {
            &*((self.0 as *const u8).add(OFFSET_FOR_STATEMENT_UPDATE)
                as *const Option<Expression<'a>>)
        }
    }

    #[inline]
    pub fn scope_id(self) -> &'t Cell<Option<ScopeId>> {
        unsafe {
            &*((self.0 as *const u8).add(OFFSET_FOR_STATEMENT_SCOPE_ID)
                as *const Cell<Option<ScopeId>>)
        }
    }
}

impl<'a, 't> GetAddress for ForStatementWithoutBody<'a, 't> {
    #[inline]
    fn address(&self) -> Address {
        unsafe { Address::from_ptr(self.0) }
    }
}

pub(crate) const OFFSET_FOR_IN_STATEMENT_NODE_ID: usize = offset_of!(ForInStatement, node_id);
pub(crate) const OFFSET_FOR_IN_STATEMENT_SPAN: usize = offset_of!(ForInStatement, span);
pub(crate) const OFFSET_FOR_IN_STATEMENT_LEFT: usize = offset_of!(ForInStatement, left);
pub(crate) const OFFSET_FOR_IN_STATEMENT_RIGHT: usize = offset_of!(ForInStatement, right);
pub(crate) const OFFSET_FOR_IN_STATEMENT_BODY: usize = offset_of!(ForInStatement, body);
pub(crate) const OFFSET_FOR_IN_STATEMENT_SCOPE_ID: usize = offset_of!(ForInStatement, scope_id);

#[repr(transparent)]
#[derive(Clone, Copy, Debug)]
pub struct ForInStatementWithoutLeft<'a, 't>(
    pub(crate) *const ForInStatement<'a>,
    pub(crate) PhantomData<&'t ()>,
);

impl<'a, 't> ForInStatementWithoutLeft<'a, 't> {
    #[inline]
    pub fn node_id(self) -> &'t Cell<NodeId> {
        unsafe {
            &*((self.0 as *const u8).add(OFFSET_FOR_IN_STATEMENT_NODE_ID) as *const Cell<NodeId>)
        }
    }

    #[inline]
    pub fn span(self) -> &'t Span {
        unsafe { &*((self.0 as *const u8).add(OFFSET_FOR_IN_STATEMENT_SPAN) as *const Span) }
    }

    #[inline]
    pub fn right(self) -> &'t Expression<'a> {
        unsafe {
            &*((self.0 as *const u8).add(OFFSET_FOR_IN_STATEMENT_RIGHT) as *const Expression<'a>)
        }
    }

    #[inline]
    pub fn body(self) -> &'t Statement<'a> {
        unsafe {
            &*((self.0 as *const u8).add(OFFSET_FOR_IN_STATEMENT_BODY) as *const Statement<'a>)
        }
    }

    #[inline]
    pub fn scope_id(self) -> &'t Cell<Option<ScopeId>> {
        unsafe {
            &*((self.0 as *const u8).add(OFFSET_FOR_IN_STATEMENT_SCOPE_ID)
                as *const Cell<Option<ScopeId>>)
        }
    }
}

impl<'a, 't> GetAddress for ForInStatementWithoutLeft<'a, 't> {
    #[inline]
    fn address(&self) -> Address {
        unsafe { Address::from_ptr(self.0) }
    }
}

#[repr(transparent)]
#[derive(Clone, Copy, Debug)]
pub struct ForInStatementWithoutRight<'a, 't>(
    pub(crate) *const ForInStatement<'a>,
    pub(crate) PhantomData<&'t ()>,
);

impl<'a, 't> ForInStatementWithoutRight<'a, 't> {
    #[inline]
    pub fn node_id(self) -> &'t Cell<NodeId> {
        unsafe {
            &*((self.0 as *const u8).add(OFFSET_FOR_IN_STATEMENT_NODE_ID) as *const Cell<NodeId>)
        }
    }

    #[inline]
    pub fn span(self) -> &'t Span {
        unsafe { &*((self.0 as *const u8).add(OFFSET_FOR_IN_STATEMENT_SPAN) as *const Span) }
    }

    #[inline]
    pub fn left(self) -> &'t ForStatementLeft<'a> {
        unsafe {
            &*((self.0 as *const u8).add(OFFSET_FOR_IN_STATEMENT_LEFT)
                as *const ForStatementLeft<'a>)
        }
    }

    #[inline]
    pub fn body(self) -> &'t Statement<'a> {
        unsafe {
            &*((self.0 as *const u8).add(OFFSET_FOR_IN_STATEMENT_BODY) as *const Statement<'a>)
        }
    }

    #[inline]
    pub fn scope_id(self) -> &'t Cell<Option<ScopeId>> {
        unsafe {
            &*((self.0 as *const u8).add(OFFSET_FOR_IN_STATEMENT_SCOPE_ID)
                as *const Cell<Option<ScopeId>>)
        }
    }
}

impl<'a, 't> GetAddress for ForInStatementWithoutRight<'a, 't> {
    #[inline]
    fn address(&self) -> Address {
        unsafe { Address::from_ptr(self.0) }
    }
}

#[repr(transparent)]
#[derive(Clone, Copy, Debug)]
pub struct ForInStatementWithoutBody<'a, 't>(
    pub(crate) *const ForInStatement<'a>,
    pub(crate) PhantomData<&'t ()>,
);

impl<'a, 't> ForInStatementWithoutBody<'a, 't> {
    #[inline]
    pub fn node_id(self) -> &'t Cell<NodeId> {
        unsafe {
            &*((self.0 as *const u8).add(OFFSET_FOR_IN_STATEMENT_NODE_ID) as *const Cell<NodeId>)
        }
    }

    #[inline]
    pub fn span(self) -> &'t Span {
        unsafe { &*((self.0 as *const u8).add(OFFSET_FOR_IN_STATEMENT_SPAN) as *const Span) }
    }

    #[inline]
    pub fn left(self) -> &'t ForStatementLeft<'a> {
        unsafe {
            &*((self.0 as *const u8).add(OFFSET_FOR_IN_STATEMENT_LEFT)
                as *const ForStatementLeft<'a>)
        }
    }

    #[inline]
    pub fn right(self) -> &'t Expression<'a> {
        unsafe {
            &*((self.0 as *const u8).add(OFFSET_FOR_IN_STATEMENT_RIGHT) as *const Expression<'a>)
        }
    }

    #[inline]
    pub fn scope_id(self) -> &'t Cell<Option<ScopeId>> {
        unsafe {
            &*((self.0 as *const u8).add(OFFSET_FOR_IN_STATEMENT_SCOPE_ID)
                as *const Cell<Option<ScopeId>>)
        }
    }
}

impl<'a, 't> GetAddress for ForInStatementWithoutBody<'a, 't> {
    #[inline]
    fn address(&self) -> Address {
        unsafe { Address::from_ptr(self.0) }
    }
}

pub(crate) const OFFSET_FOR_OF_STATEMENT_NODE_ID: usize = offset_of!(ForOfStatement, node_id);
pub(crate) const OFFSET_FOR_OF_STATEMENT_SPAN: usize = offset_of!(ForOfStatement, span);
pub(crate) const OFFSET_FOR_OF_STATEMENT_AWAIT: usize = offset_of!(ForOfStatement, r#await);
pub(crate) const OFFSET_FOR_OF_STATEMENT_LEFT: usize = offset_of!(ForOfStatement, left);
pub(crate) const OFFSET_FOR_OF_STATEMENT_RIGHT: usize = offset_of!(ForOfStatement, right);
pub(crate) const OFFSET_FOR_OF_STATEMENT_BODY: usize = offset_of!(ForOfStatement, body);
pub(crate) const OFFSET_FOR_OF_STATEMENT_SCOPE_ID: usize = offset_of!(ForOfStatement, scope_id);

#[repr(transparent)]
#[derive(Clone, Copy, Debug)]
pub struct ForOfStatementWithoutLeft<'a, 't>(
    pub(crate) *const ForOfStatement<'a>,
    pub(crate) PhantomData<&'t ()>,
);

impl<'a, 't> ForOfStatementWithoutLeft<'a, 't> {
    #[inline]
    pub fn node_id(self) -> &'t Cell<NodeId> {
        unsafe {
            &*((self.0 as *const u8).add(OFFSET_FOR_OF_STATEMENT_NODE_ID) as *const Cell<NodeId>)
        }
    }

    #[inline]
    pub fn span(self) -> &'t Span {
        unsafe { &*((self.0 as *const u8).add(OFFSET_FOR_OF_STATEMENT_SPAN) as *const Span) }
    }

    #[inline]
    pub fn r#await(self) -> &'t bool {
        unsafe { &*((self.0 as *const u8).add(OFFSET_FOR_OF_STATEMENT_AWAIT) as *const bool) }
    }

    #[inline]
    pub fn right(self) -> &'t Expression<'a> {
        unsafe {
            &*((self.0 as *const u8).add(OFFSET_FOR_OF_STATEMENT_RIGHT) as *const Expression<'a>)
        }
    }

    #[inline]
    pub fn body(self) -> &'t Statement<'a> {
        unsafe {
            &*((self.0 as *const u8).add(OFFSET_FOR_OF_STATEMENT_BODY) as *const Statement<'a>)
        }
    }

    #[inline]
    pub fn scope_id(self) -> &'t Cell<Option<ScopeId>> {
        unsafe {
            &*((self.0 as *const u8).add(OFFSET_FOR_OF_STATEMENT_SCOPE_ID)
                as *const Cell<Option<ScopeId>>)
        }
    }
}

impl<'a, 't> GetAddress for ForOfStatementWithoutLeft<'a, 't> {
    #[inline]
    fn address(&self) -> Address {
        unsafe { Address::from_ptr(self.0) }
    }
}

#[repr(transparent)]
#[derive(Clone, Copy, Debug)]
pub struct ForOfStatementWithoutRight<'a, 't>(
    pub(crate) *const ForOfStatement<'a>,
    pub(crate) PhantomData<&'t ()>,
);

impl<'a, 't> ForOfStatementWithoutRight<'a, 't> {
    #[inline]
    pub fn node_id(self) -> &'t Cell<NodeId> {
        unsafe {
            &*((self.0 as *const u8).add(OFFSET_FOR_OF_STATEMENT_NODE_ID) as *const Cell<NodeId>)
        }
    }

    #[inline]
    pub fn span(self) -> &'t Span {
        unsafe { &*((self.0 as *const u8).add(OFFSET_FOR_OF_STATEMENT_SPAN) as *const Span) }
    }

    #[inline]
    pub fn r#await(self) -> &'t bool {
        unsafe { &*((self.0 as *const u8).add(OFFSET_FOR_OF_STATEMENT_AWAIT) as *const bool) }
    }

    #[inline]
    pub fn left(self) -> &'t ForStatementLeft<'a> {
        unsafe {
            &*((self.0 as *const u8).add(OFFSET_FOR_OF_STATEMENT_LEFT)
                as *const ForStatementLeft<'a>)
        }
    }

    #[inline]
    pub fn body(self) -> &'t Statement<'a> {
        unsafe {
            &*((self.0 as *const u8).add(OFFSET_FOR_OF_STATEMENT_BODY) as *const Statement<'a>)
        }
    }

    #[inline]
    pub fn scope_id(self) -> &'t Cell<Option<ScopeId>> {
        unsafe {
            &*((self.0 as *const u8).add(OFFSET_FOR_OF_STATEMENT_SCOPE_ID)
                as *const Cell<Option<ScopeId>>)
        }
    }
}

impl<'a, 't> GetAddress for ForOfStatementWithoutRight<'a, 't> {
    #[inline]
    fn address(&self) -> Address {
        unsafe { Address::from_ptr(self.0) }
    }
}

#[repr(transparent)]
#[derive(Clone, Copy, Debug)]
pub struct ForOfStatementWithoutBody<'a, 't>(
    pub(crate) *const ForOfStatement<'a>,
    pub(crate) PhantomData<&'t ()>,
);

impl<'a, 't> ForOfStatementWithoutBody<'a, 't> {
    #[inline]
    pub fn node_id(self) -> &'t Cell<NodeId> {
        unsafe {
            &*((self.0 as *const u8).add(OFFSET_FOR_OF_STATEMENT_NODE_ID) as *const Cell<NodeId>)
        }
    }

    #[inline]
    pub fn span(self) -> &'t Span {
        unsafe { &*((self.0 as *const u8).add(OFFSET_FOR_OF_STATEMENT_SPAN) as *const Span) }
    }

    #[inline]
    pub fn r#await(self) -> &'t bool {
        unsafe { &*((self.0 as *const u8).add(OFFSET_FOR_OF_STATEMENT_AWAIT) as *const bool) }
    }

    #[inline]
    pub fn left(self) -> &'t ForStatementLeft<'a> {
        unsafe {
            &*((self.0 as *const u8).add(OFFSET_FOR_OF_STATEMENT_LEFT)
                as *const ForStatementLeft<'a>)
        }
    }

    #[inline]
    pub fn right(self) -> &'t Expression<'a> {
        unsafe {
            &*((self.0 as *const u8).add(OFFSET_FOR_OF_STATEMENT_RIGHT) as *const Expression<'a>)
        }
    }

    #[inline]
    pub fn scope_id(self) -> &'t Cell<Option<ScopeId>> {
        unsafe {
            &*((self.0 as *const u8).add(OFFSET_FOR_OF_STATEMENT_SCOPE_ID)
                as *const Cell<Option<ScopeId>>)
        }
    }
}

impl<'a, 't> GetAddress for ForOfStatementWithoutBody<'a, 't> {
    #[inline]
    fn address(&self) -> Address {
        unsafe { Address::from_ptr(self.0) }
    }
}

pub(crate) const OFFSET_CONTINUE_STATEMENT_NODE_ID: usize = offset_of!(ContinueStatement, node_id);
pub(crate) const OFFSET_CONTINUE_STATEMENT_SPAN: usize = offset_of!(ContinueStatement, span);
pub(crate) const OFFSET_CONTINUE_STATEMENT_LABEL: usize = offset_of!(ContinueStatement, label);

#[repr(transparent)]
#[derive(Clone, Copy, Debug)]
pub struct ContinueStatementWithoutLabel<'a, 't>(
    pub(crate) *const ContinueStatement<'a>,
    pub(crate) PhantomData<&'t ()>,
);

impl<'a, 't> ContinueStatementWithoutLabel<'a, 't> {
    #[inline]
    pub fn node_id(self) -> &'t Cell<NodeId> {
        unsafe {
            &*((self.0 as *const u8).add(OFFSET_CONTINUE_STATEMENT_NODE_ID) as *const Cell<NodeId>)
        }
    }

    #[inline]
    pub fn span(self) -> &'t Span {
        unsafe { &*((self.0 as *const u8).add(OFFSET_CONTINUE_STATEMENT_SPAN) as *const Span) }
    }
}

impl<'a, 't> GetAddress for ContinueStatementWithoutLabel<'a, 't> {
    #[inline]
    fn address(&self) -> Address {
        unsafe { Address::from_ptr(self.0) }
    }
}

pub(crate) const OFFSET_BREAK_STATEMENT_NODE_ID: usize = offset_of!(BreakStatement, node_id);
pub(crate) const OFFSET_BREAK_STATEMENT_SPAN: usize = offset_of!(BreakStatement, span);
pub(crate) const OFFSET_BREAK_STATEMENT_LABEL: usize = offset_of!(BreakStatement, label);

#[repr(transparent)]
#[derive(Clone, Copy, Debug)]
pub struct BreakStatementWithoutLabel<'a, 't>(
    pub(crate) *const BreakStatement<'a>,
    pub(crate) PhantomData<&'t ()>,
);

impl<'a, 't> BreakStatementWithoutLabel<'a, 't> {
    #[inline]
    pub fn node_id(self) -> &'t Cell<NodeId> {
        unsafe {
            &*((self.0 as *const u8).add(OFFSET_BREAK_STATEMENT_NODE_ID) as *const Cell<NodeId>)
        }
    }

    #[inline]
    pub fn span(self) -> &'t Span {
        unsafe { &*((self.0 as *const u8).add(OFFSET_BREAK_STATEMENT_SPAN) as *const Span) }
    }
}

impl<'a, 't> GetAddress for BreakStatementWithoutLabel<'a, 't> {
    #[inline]
    fn address(&self) -> Address {
        unsafe { Address::from_ptr(self.0) }
    }
}

pub(crate) const OFFSET_RETURN_STATEMENT_NODE_ID: usize = offset_of!(ReturnStatement, node_id);
pub(crate) const OFFSET_RETURN_STATEMENT_SPAN: usize = offset_of!(ReturnStatement, span);
pub(crate) const OFFSET_RETURN_STATEMENT_ARGUMENT: usize = offset_of!(ReturnStatement, argument);

#[repr(transparent)]
#[derive(Clone, Copy, Debug)]
pub struct ReturnStatementWithoutArgument<'a, 't>(
    pub(crate) *const ReturnStatement<'a>,
    pub(crate) PhantomData<&'t ()>,
);

impl<'a, 't> ReturnStatementWithoutArgument<'a, 't> {
    #[inline]
    pub fn node_id(self) -> &'t Cell<NodeId> {
        unsafe {
            &*((self.0 as *const u8).add(OFFSET_RETURN_STATEMENT_NODE_ID) as *const Cell<NodeId>)
        }
    }

    #[inline]
    pub fn span(self) -> &'t Span {
        unsafe { &*((self.0 as *const u8).add(OFFSET_RETURN_STATEMENT_SPAN) as *const Span) }
    }
}

impl<'a, 't> GetAddress for ReturnStatementWithoutArgument<'a, 't> {
    #[inline]
    fn address(&self) -> Address {
        unsafe { Address::from_ptr(self.0) }
    }
}

pub(crate) const OFFSET_WITH_STATEMENT_NODE_ID: usize = offset_of!(WithStatement, node_id);
pub(crate) const OFFSET_WITH_STATEMENT_SPAN: usize = offset_of!(WithStatement, span);
pub(crate) const OFFSET_WITH_STATEMENT_OBJECT: usize = offset_of!(WithStatement, object);
pub(crate) const OFFSET_WITH_STATEMENT_BODY: usize = offset_of!(WithStatement, body);
pub(crate) const OFFSET_WITH_STATEMENT_SCOPE_ID: usize = offset_of!(WithStatement, scope_id);

#[repr(transparent)]
#[derive(Clone, Copy, Debug)]
pub struct WithStatementWithoutObject<'a, 't>(
    pub(crate) *const WithStatement<'a>,
    pub(crate) PhantomData<&'t ()>,
);

impl<'a, 't> WithStatementWithoutObject<'a, 't> {
    #[inline]
    pub fn node_id(self) -> &'t Cell<NodeId> {
        unsafe {
            &*((self.0 as *const u8).add(OFFSET_WITH_STATEMENT_NODE_ID) as *const Cell<NodeId>)
        }
    }

    #[inline]
    pub fn span(self) -> &'t Span {
        unsafe { &*((self.0 as *const u8).add(OFFSET_WITH_STATEMENT_SPAN) as *const Span) }
    }

    #[inline]
    pub fn body(self) -> &'t Statement<'a> {
        unsafe { &*((self.0 as *const u8).add(OFFSET_WITH_STATEMENT_BODY) as *const Statement<'a>) }
    }

    #[inline]
    pub fn scope_id(self) -> &'t Cell<Option<ScopeId>> {
        unsafe {
            &*((self.0 as *const u8).add(OFFSET_WITH_STATEMENT_SCOPE_ID)
                as *const Cell<Option<ScopeId>>)
        }
    }
}

impl<'a, 't> GetAddress for WithStatementWithoutObject<'a, 't> {
    #[inline]
    fn address(&self) -> Address {
        unsafe { Address::from_ptr(self.0) }
    }
}

#[repr(transparent)]
#[derive(Clone, Copy, Debug)]
pub struct WithStatementWithoutBody<'a, 't>(
    pub(crate) *const WithStatement<'a>,
    pub(crate) PhantomData<&'t ()>,
);

impl<'a, 't> WithStatementWithoutBody<'a, 't> {
    #[inline]
    pub fn node_id(self) -> &'t Cell<NodeId> {
        unsafe {
            &*((self.0 as *const u8).add(OFFSET_WITH_STATEMENT_NODE_ID) as *const Cell<NodeId>)
        }
    }

    #[inline]
    pub fn span(self) -> &'t Span {
        unsafe { &*((self.0 as *const u8).add(OFFSET_WITH_STATEMENT_SPAN) as *const Span) }
    }

    #[inline]
    pub fn object(self) -> &'t Expression<'a> {
        unsafe {
            &*((self.0 as *const u8).add(OFFSET_WITH_STATEMENT_OBJECT) as *const Expression<'a>)
        }
    }

    #[inline]
    pub fn scope_id(self) -> &'t Cell<Option<ScopeId>> {
        unsafe {
            &*((self.0 as *const u8).add(OFFSET_WITH_STATEMENT_SCOPE_ID)
                as *const Cell<Option<ScopeId>>)
        }
    }
}

impl<'a, 't> GetAddress for WithStatementWithoutBody<'a, 't> {
    #[inline]
    fn address(&self) -> Address {
        unsafe { Address::from_ptr(self.0) }
    }
}

pub(crate) const OFFSET_SWITCH_STATEMENT_NODE_ID: usize = offset_of!(SwitchStatement, node_id);
pub(crate) const OFFSET_SWITCH_STATEMENT_SPAN: usize = offset_of!(SwitchStatement, span);
pub(crate) const OFFSET_SWITCH_STATEMENT_DISCRIMINANT: usize =
    offset_of!(SwitchStatement, discriminant);
pub(crate) const OFFSET_SWITCH_STATEMENT_CASES: usize = offset_of!(SwitchStatement, cases);
pub(crate) const OFFSET_SWITCH_STATEMENT_SCOPE_ID: usize = offset_of!(SwitchStatement, scope_id);

#[repr(transparent)]
#[derive(Clone, Copy, Debug)]
pub struct SwitchStatementWithoutDiscriminant<'a, 't>(
    pub(crate) *const SwitchStatement<'a>,
    pub(crate) PhantomData<&'t ()>,
);

impl<'a, 't> SwitchStatementWithoutDiscriminant<'a, 't> {
    #[inline]
    pub fn node_id(self) -> &'t Cell<NodeId> {
        unsafe {
            &*((self.0 as *const u8).add(OFFSET_SWITCH_STATEMENT_NODE_ID) as *const Cell<NodeId>)
        }
    }

    #[inline]
    pub fn span(self) -> &'t Span {
        unsafe { &*((self.0 as *const u8).add(OFFSET_SWITCH_STATEMENT_SPAN) as *const Span) }
    }

    #[inline]
    pub fn cases(self) -> &'t ArenaVec<'a, SwitchCase<'a>> {
        unsafe {
            &*((self.0 as *const u8).add(OFFSET_SWITCH_STATEMENT_CASES)
                as *const ArenaVec<'a, SwitchCase<'a>>)
        }
    }

    #[inline]
    pub fn scope_id(self) -> &'t Cell<Option<ScopeId>> {
        unsafe {
            &*((self.0 as *const u8).add(OFFSET_SWITCH_STATEMENT_SCOPE_ID)
                as *const Cell<Option<ScopeId>>)
        }
    }
}

impl<'a, 't> GetAddress for SwitchStatementWithoutDiscriminant<'a, 't> {
    #[inline]
    fn address(&self) -> Address {
        unsafe { Address::from_ptr(self.0) }
    }
}

#[repr(transparent)]
#[derive(Clone, Copy, Debug)]
pub struct SwitchStatementWithoutCases<'a, 't>(
    pub(crate) *const SwitchStatement<'a>,
    pub(crate) PhantomData<&'t ()>,
);

impl<'a, 't> SwitchStatementWithoutCases<'a, 't> {
    #[inline]
    pub fn node_id(self) -> &'t Cell<NodeId> {
        unsafe {
            &*((self.0 as *const u8).add(OFFSET_SWITCH_STATEMENT_NODE_ID) as *const Cell<NodeId>)
        }
    }

    #[inline]
    pub fn span(self) -> &'t Span {
        unsafe { &*((self.0 as *const u8).add(OFFSET_SWITCH_STATEMENT_SPAN) as *const Span) }
    }

    #[inline]
    pub fn discriminant(self) -> &'t Expression<'a> {
        unsafe {
            &*((self.0 as *const u8).add(OFFSET_SWITCH_STATEMENT_DISCRIMINANT)
                as *const Expression<'a>)
        }
    }

    #[inline]
    pub fn scope_id(self) -> &'t Cell<Option<ScopeId>> {
        unsafe {
            &*((self.0 as *const u8).add(OFFSET_SWITCH_STATEMENT_SCOPE_ID)
                as *const Cell<Option<ScopeId>>)
        }
    }
}

impl<'a, 't> GetAddress for SwitchStatementWithoutCases<'a, 't> {
    #[inline]
    fn address(&self) -> Address {
        unsafe { Address::from_ptr(self.0) }
    }
}

pub(crate) const OFFSET_SWITCH_CASE_NODE_ID: usize = offset_of!(SwitchCase, node_id);
pub(crate) const OFFSET_SWITCH_CASE_SPAN: usize = offset_of!(SwitchCase, span);
pub(crate) const OFFSET_SWITCH_CASE_TEST: usize = offset_of!(SwitchCase, test);
pub(crate) const OFFSET_SWITCH_CASE_CONSEQUENT: usize = offset_of!(SwitchCase, consequent);

#[repr(transparent)]
#[derive(Clone, Copy, Debug)]
pub struct SwitchCaseWithoutTest<'a, 't>(
    pub(crate) *const SwitchCase<'a>,
    pub(crate) PhantomData<&'t ()>,
);

impl<'a, 't> SwitchCaseWithoutTest<'a, 't> {
    #[inline]
    pub fn node_id(self) -> &'t Cell<NodeId> {
        unsafe { &*((self.0 as *const u8).add(OFFSET_SWITCH_CASE_NODE_ID) as *const Cell<NodeId>) }
    }

    #[inline]
    pub fn span(self) -> &'t Span {
        unsafe { &*((self.0 as *const u8).add(OFFSET_SWITCH_CASE_SPAN) as *const Span) }
    }

    #[inline]
    pub fn consequent(self) -> &'t ArenaVec<'a, Statement<'a>> {
        unsafe {
            &*((self.0 as *const u8).add(OFFSET_SWITCH_CASE_CONSEQUENT)
                as *const ArenaVec<'a, Statement<'a>>)
        }
    }
}

impl<'a, 't> GetAddress for SwitchCaseWithoutTest<'a, 't> {
    #[inline]
    fn address(&self) -> Address {
        unsafe { Address::from_ptr(self.0) }
    }
}

#[repr(transparent)]
#[derive(Clone, Copy, Debug)]
pub struct SwitchCaseWithoutConsequent<'a, 't>(
    pub(crate) *const SwitchCase<'a>,
    pub(crate) PhantomData<&'t ()>,
);

impl<'a, 't> SwitchCaseWithoutConsequent<'a, 't> {
    #[inline]
    pub fn node_id(self) -> &'t Cell<NodeId> {
        unsafe { &*((self.0 as *const u8).add(OFFSET_SWITCH_CASE_NODE_ID) as *const Cell<NodeId>) }
    }

    #[inline]
    pub fn span(self) -> &'t Span {
        unsafe { &*((self.0 as *const u8).add(OFFSET_SWITCH_CASE_SPAN) as *const Span) }
    }

    #[inline]
    pub fn test(self) -> &'t Option<Expression<'a>> {
        unsafe {
            &*((self.0 as *const u8).add(OFFSET_SWITCH_CASE_TEST) as *const Option<Expression<'a>>)
        }
    }
}

impl<'a, 't> GetAddress for SwitchCaseWithoutConsequent<'a, 't> {
    #[inline]
    fn address(&self) -> Address {
        unsafe { Address::from_ptr(self.0) }
    }
}

pub(crate) const OFFSET_LABELED_STATEMENT_NODE_ID: usize = offset_of!(LabeledStatement, node_id);
pub(crate) const OFFSET_LABELED_STATEMENT_SPAN: usize = offset_of!(LabeledStatement, span);
pub(crate) const OFFSET_LABELED_STATEMENT_LABEL: usize = offset_of!(LabeledStatement, label);
pub(crate) const OFFSET_LABELED_STATEMENT_BODY: usize = offset_of!(LabeledStatement, body);

#[repr(transparent)]
#[derive(Clone, Copy, Debug)]
pub struct LabeledStatementWithoutLabel<'a, 't>(
    pub(crate) *const LabeledStatement<'a>,
    pub(crate) PhantomData<&'t ()>,
);

impl<'a, 't> LabeledStatementWithoutLabel<'a, 't> {
    #[inline]
    pub fn node_id(self) -> &'t Cell<NodeId> {
        unsafe {
            &*((self.0 as *const u8).add(OFFSET_LABELED_STATEMENT_NODE_ID) as *const Cell<NodeId>)
        }
    }

    #[inline]
    pub fn span(self) -> &'t Span {
        unsafe { &*((self.0 as *const u8).add(OFFSET_LABELED_STATEMENT_SPAN) as *const Span) }
    }

    #[inline]
    pub fn body(self) -> &'t Statement<'a> {
        unsafe {
            &*((self.0 as *const u8).add(OFFSET_LABELED_STATEMENT_BODY) as *const Statement<'a>)
        }
    }
}

impl<'a, 't> GetAddress for LabeledStatementWithoutLabel<'a, 't> {
    #[inline]
    fn address(&self) -> Address {
        unsafe { Address::from_ptr(self.0) }
    }
}

#[repr(transparent)]
#[derive(Clone, Copy, Debug)]
pub struct LabeledStatementWithoutBody<'a, 't>(
    pub(crate) *const LabeledStatement<'a>,
    pub(crate) PhantomData<&'t ()>,
);

impl<'a, 't> LabeledStatementWithoutBody<'a, 't> {
    #[inline]
    pub fn node_id(self) -> &'t Cell<NodeId> {
        unsafe {
            &*((self.0 as *const u8).add(OFFSET_LABELED_STATEMENT_NODE_ID) as *const Cell<NodeId>)
        }
    }

    #[inline]
    pub fn span(self) -> &'t Span {
        unsafe { &*((self.0 as *const u8).add(OFFSET_LABELED_STATEMENT_SPAN) as *const Span) }
    }

    #[inline]
    pub fn label(self) -> &'t LabelIdentifier<'a> {
        unsafe {
            &*((self.0 as *const u8).add(OFFSET_LABELED_STATEMENT_LABEL)
                as *const LabelIdentifier<'a>)
        }
    }
}

impl<'a, 't> GetAddress for LabeledStatementWithoutBody<'a, 't> {
    #[inline]
    fn address(&self) -> Address {
        unsafe { Address::from_ptr(self.0) }
    }
}

pub(crate) const OFFSET_THROW_STATEMENT_NODE_ID: usize = offset_of!(ThrowStatement, node_id);
pub(crate) const OFFSET_THROW_STATEMENT_SPAN: usize = offset_of!(ThrowStatement, span);
pub(crate) const OFFSET_THROW_STATEMENT_ARGUMENT: usize = offset_of!(ThrowStatement, argument);

#[repr(transparent)]
#[derive(Clone, Copy, Debug)]
pub struct ThrowStatementWithoutArgument<'a, 't>(
    pub(crate) *const ThrowStatement<'a>,
    pub(crate) PhantomData<&'t ()>,
);

impl<'a, 't> ThrowStatementWithoutArgument<'a, 't> {
    #[inline]
    pub fn node_id(self) -> &'t Cell<NodeId> {
        unsafe {
            &*((self.0 as *const u8).add(OFFSET_THROW_STATEMENT_NODE_ID) as *const Cell<NodeId>)
        }
    }

    #[inline]
    pub fn span(self) -> &'t Span {
        unsafe { &*((self.0 as *const u8).add(OFFSET_THROW_STATEMENT_SPAN) as *const Span) }
    }
}

impl<'a, 't> GetAddress for ThrowStatementWithoutArgument<'a, 't> {
    #[inline]
    fn address(&self) -> Address {
        unsafe { Address::from_ptr(self.0) }
    }
}

pub(crate) const OFFSET_TRY_STATEMENT_NODE_ID: usize = offset_of!(TryStatement, node_id);
pub(crate) const OFFSET_TRY_STATEMENT_SPAN: usize = offset_of!(TryStatement, span);
pub(crate) const OFFSET_TRY_STATEMENT_BLOCK: usize = offset_of!(TryStatement, block);
pub(crate) const OFFSET_TRY_STATEMENT_HANDLER: usize = offset_of!(TryStatement, handler);
pub(crate) const OFFSET_TRY_STATEMENT_FINALIZER: usize = offset_of!(TryStatement, finalizer);

#[repr(transparent)]
#[derive(Clone, Copy, Debug)]
pub struct TryStatementWithoutBlock<'a, 't>(
    pub(crate) *const TryStatement<'a>,
    pub(crate) PhantomData<&'t ()>,
);

impl<'a, 't> TryStatementWithoutBlock<'a, 't> {
    #[inline]
    pub fn node_id(self) -> &'t Cell<NodeId> {
        unsafe {
            &*((self.0 as *const u8).add(OFFSET_TRY_STATEMENT_NODE_ID) as *const Cell<NodeId>)
        }
    }

    #[inline]
    pub fn span(self) -> &'t Span {
        unsafe { &*((self.0 as *const u8).add(OFFSET_TRY_STATEMENT_SPAN) as *const Span) }
    }

    #[inline]
    pub fn handler(self) -> &'t Option<ArenaBox<'a, CatchClause<'a>>> {
        unsafe {
            &*((self.0 as *const u8).add(OFFSET_TRY_STATEMENT_HANDLER)
                as *const Option<ArenaBox<'a, CatchClause<'a>>>)
        }
    }

    #[inline]
    pub fn finalizer(self) -> &'t Option<ArenaBox<'a, BlockStatement<'a>>> {
        unsafe {
            &*((self.0 as *const u8).add(OFFSET_TRY_STATEMENT_FINALIZER)
                as *const Option<ArenaBox<'a, BlockStatement<'a>>>)
        }
    }
}

impl<'a, 't> GetAddress for TryStatementWithoutBlock<'a, 't> {
    #[inline]
    fn address(&self) -> Address {
        unsafe { Address::from_ptr(self.0) }
    }
}

#[repr(transparent)]
#[derive(Clone, Copy, Debug)]
pub struct TryStatementWithoutHandler<'a, 't>(
    pub(crate) *const TryStatement<'a>,
    pub(crate) PhantomData<&'t ()>,
);

impl<'a, 't> TryStatementWithoutHandler<'a, 't> {
    #[inline]
    pub fn node_id(self) -> &'t Cell<NodeId> {
        unsafe {
            &*((self.0 as *const u8).add(OFFSET_TRY_STATEMENT_NODE_ID) as *const Cell<NodeId>)
        }
    }

    #[inline]
    pub fn span(self) -> &'t Span {
        unsafe { &*((self.0 as *const u8).add(OFFSET_TRY_STATEMENT_SPAN) as *const Span) }
    }

    #[inline]
    pub fn block(self) -> &'t ArenaBox<'a, BlockStatement<'a>> {
        unsafe {
            &*((self.0 as *const u8).add(OFFSET_TRY_STATEMENT_BLOCK)
                as *const ArenaBox<'a, BlockStatement<'a>>)
        }
    }

    #[inline]
    pub fn finalizer(self) -> &'t Option<ArenaBox<'a, BlockStatement<'a>>> {
        unsafe {
            &*((self.0 as *const u8).add(OFFSET_TRY_STATEMENT_FINALIZER)
                as *const Option<ArenaBox<'a, BlockStatement<'a>>>)
        }
    }
}

impl<'a, 't> GetAddress for TryStatementWithoutHandler<'a, 't> {
    #[inline]
    fn address(&self) -> Address {
        unsafe { Address::from_ptr(self.0) }
    }
}

#[repr(transparent)]
#[derive(Clone, Copy, Debug)]
pub struct TryStatementWithoutFinalizer<'a, 't>(
    pub(crate) *const TryStatement<'a>,
    pub(crate) PhantomData<&'t ()>,
);

impl<'a, 't> TryStatementWithoutFinalizer<'a, 't> {
    #[inline]
    pub fn node_id(self) -> &'t Cell<NodeId> {
        unsafe {
            &*((self.0 as *const u8).add(OFFSET_TRY_STATEMENT_NODE_ID) as *const Cell<NodeId>)
        }
    }

    #[inline]
    pub fn span(self) -> &'t Span {
        unsafe { &*((self.0 as *const u8).add(OFFSET_TRY_STATEMENT_SPAN) as *const Span) }
    }

    #[inline]
    pub fn block(self) -> &'t ArenaBox<'a, BlockStatement<'a>> {
        unsafe {
            &*((self.0 as *const u8).add(OFFSET_TRY_STATEMENT_BLOCK)
                as *const ArenaBox<'a, BlockStatement<'a>>)
        }
    }

    #[inline]
    pub fn handler(self) -> &'t Option<ArenaBox<'a, CatchClause<'a>>> {
        unsafe {
            &*((self.0 as *const u8).add(OFFSET_TRY_STATEMENT_HANDLER)
                as *const Option<ArenaBox<'a, CatchClause<'a>>>)
        }
    }
}

impl<'a, 't> GetAddress for TryStatementWithoutFinalizer<'a, 't> {
    #[inline]
    fn address(&self) -> Address {
        unsafe { Address::from_ptr(self.0) }
    }
}

pub(crate) const OFFSET_CATCH_CLAUSE_NODE_ID: usize = offset_of!(CatchClause, node_id);
pub(crate) const OFFSET_CATCH_CLAUSE_SPAN: usize = offset_of!(CatchClause, span);
pub(crate) const OFFSET_CATCH_CLAUSE_PARAM: usize = offset_of!(CatchClause, param);
pub(crate) const OFFSET_CATCH_CLAUSE_BODY: usize = offset_of!(CatchClause, body);
pub(crate) const OFFSET_CATCH_CLAUSE_SCOPE_ID: usize = offset_of!(CatchClause, scope_id);

#[repr(transparent)]
#[derive(Clone, Copy, Debug)]
pub struct CatchClauseWithoutParam<'a, 't>(
    pub(crate) *const CatchClause<'a>,
    pub(crate) PhantomData<&'t ()>,
);

impl<'a, 't> CatchClauseWithoutParam<'a, 't> {
    #[inline]
    pub fn node_id(self) -> &'t Cell<NodeId> {
        unsafe { &*((self.0 as *const u8).add(OFFSET_CATCH_CLAUSE_NODE_ID) as *const Cell<NodeId>) }
    }

    #[inline]
    pub fn span(self) -> &'t Span {
        unsafe { &*((self.0 as *const u8).add(OFFSET_CATCH_CLAUSE_SPAN) as *const Span) }
    }

    #[inline]
    pub fn body(self) -> &'t ArenaBox<'a, BlockStatement<'a>> {
        unsafe {
            &*((self.0 as *const u8).add(OFFSET_CATCH_CLAUSE_BODY)
                as *const ArenaBox<'a, BlockStatement<'a>>)
        }
    }

    #[inline]
    pub fn scope_id(self) -> &'t Cell<Option<ScopeId>> {
        unsafe {
            &*((self.0 as *const u8).add(OFFSET_CATCH_CLAUSE_SCOPE_ID)
                as *const Cell<Option<ScopeId>>)
        }
    }
}

impl<'a, 't> GetAddress for CatchClauseWithoutParam<'a, 't> {
    #[inline]
    fn address(&self) -> Address {
        unsafe { Address::from_ptr(self.0) }
    }
}

#[repr(transparent)]
#[derive(Clone, Copy, Debug)]
pub struct CatchClauseWithoutBody<'a, 't>(
    pub(crate) *const CatchClause<'a>,
    pub(crate) PhantomData<&'t ()>,
);

impl<'a, 't> CatchClauseWithoutBody<'a, 't> {
    #[inline]
    pub fn node_id(self) -> &'t Cell<NodeId> {
        unsafe { &*((self.0 as *const u8).add(OFFSET_CATCH_CLAUSE_NODE_ID) as *const Cell<NodeId>) }
    }

    #[inline]
    pub fn span(self) -> &'t Span {
        unsafe { &*((self.0 as *const u8).add(OFFSET_CATCH_CLAUSE_SPAN) as *const Span) }
    }

    #[inline]
    pub fn param(self) -> &'t Option<CatchParameter<'a>> {
        unsafe {
            &*((self.0 as *const u8).add(OFFSET_CATCH_CLAUSE_PARAM)
                as *const Option<CatchParameter<'a>>)
        }
    }

    #[inline]
    pub fn scope_id(self) -> &'t Cell<Option<ScopeId>> {
        unsafe {
            &*((self.0 as *const u8).add(OFFSET_CATCH_CLAUSE_SCOPE_ID)
                as *const Cell<Option<ScopeId>>)
        }
    }
}

impl<'a, 't> GetAddress for CatchClauseWithoutBody<'a, 't> {
    #[inline]
    fn address(&self) -> Address {
        unsafe { Address::from_ptr(self.0) }
    }
}

pub(crate) const OFFSET_CATCH_PARAMETER_NODE_ID: usize = offset_of!(CatchParameter, node_id);
pub(crate) const OFFSET_CATCH_PARAMETER_SPAN: usize = offset_of!(CatchParameter, span);
pub(crate) const OFFSET_CATCH_PARAMETER_PATTERN: usize = offset_of!(CatchParameter, pattern);
pub(crate) const OFFSET_CATCH_PARAMETER_TYPE_ANNOTATION: usize =
    offset_of!(CatchParameter, type_annotation);

#[repr(transparent)]
#[derive(Clone, Copy, Debug)]
pub struct CatchParameterWithoutPattern<'a, 't>(
    pub(crate) *const CatchParameter<'a>,
    pub(crate) PhantomData<&'t ()>,
);

impl<'a, 't> CatchParameterWithoutPattern<'a, 't> {
    #[inline]
    pub fn node_id(self) -> &'t Cell<NodeId> {
        unsafe {
            &*((self.0 as *const u8).add(OFFSET_CATCH_PARAMETER_NODE_ID) as *const Cell<NodeId>)
        }
    }

    #[inline]
    pub fn span(self) -> &'t Span {
        unsafe { &*((self.0 as *const u8).add(OFFSET_CATCH_PARAMETER_SPAN) as *const Span) }
    }

    #[inline]
    pub fn type_annotation(self) -> &'t Option<ArenaBox<'a, TSTypeAnnotation<'a>>> {
        unsafe {
            &*((self.0 as *const u8).add(OFFSET_CATCH_PARAMETER_TYPE_ANNOTATION)
                as *const Option<ArenaBox<'a, TSTypeAnnotation<'a>>>)
        }
    }
}

impl<'a, 't> GetAddress for CatchParameterWithoutPattern<'a, 't> {
    #[inline]
    fn address(&self) -> Address {
        unsafe { Address::from_ptr(self.0) }
    }
}

pub(crate) const OFFSET_ASSIGNMENT_PATTERN_NODE_ID: usize = offset_of!(AssignmentPattern, node_id);
pub(crate) const OFFSET_ASSIGNMENT_PATTERN_SPAN: usize = offset_of!(AssignmentPattern, span);
pub(crate) const OFFSET_ASSIGNMENT_PATTERN_LEFT: usize = offset_of!(AssignmentPattern, left);
pub(crate) const OFFSET_ASSIGNMENT_PATTERN_RIGHT: usize = offset_of!(AssignmentPattern, right);

#[repr(transparent)]
#[derive(Clone, Copy, Debug)]
pub struct AssignmentPatternWithoutLeft<'a, 't>(
    pub(crate) *const AssignmentPattern<'a>,
    pub(crate) PhantomData<&'t ()>,
);

impl<'a, 't> AssignmentPatternWithoutLeft<'a, 't> {
    #[inline]
    pub fn node_id(self) -> &'t Cell<NodeId> {
        unsafe {
            &*((self.0 as *const u8).add(OFFSET_ASSIGNMENT_PATTERN_NODE_ID) as *const Cell<NodeId>)
        }
    }

    #[inline]
    pub fn span(self) -> &'t Span {
        unsafe { &*((self.0 as *const u8).add(OFFSET_ASSIGNMENT_PATTERN_SPAN) as *const Span) }
    }

    #[inline]
    pub fn right(self) -> &'t Expression<'a> {
        unsafe {
            &*((self.0 as *const u8).add(OFFSET_ASSIGNMENT_PATTERN_RIGHT) as *const Expression<'a>)
        }
    }
}

impl<'a, 't> GetAddress for AssignmentPatternWithoutLeft<'a, 't> {
    #[inline]
    fn address(&self) -> Address {
        unsafe { Address::from_ptr(self.0) }
    }
}

#[repr(transparent)]
#[derive(Clone, Copy, Debug)]
pub struct AssignmentPatternWithoutRight<'a, 't>(
    pub(crate) *const AssignmentPattern<'a>,
    pub(crate) PhantomData<&'t ()>,
);

impl<'a, 't> AssignmentPatternWithoutRight<'a, 't> {
    #[inline]
    pub fn node_id(self) -> &'t Cell<NodeId> {
        unsafe {
            &*((self.0 as *const u8).add(OFFSET_ASSIGNMENT_PATTERN_NODE_ID) as *const Cell<NodeId>)
        }
    }

    #[inline]
    pub fn span(self) -> &'t Span {
        unsafe { &*((self.0 as *const u8).add(OFFSET_ASSIGNMENT_PATTERN_SPAN) as *const Span) }
    }

    #[inline]
    pub fn left(self) -> &'t BindingPattern<'a> {
        unsafe {
            &*((self.0 as *const u8).add(OFFSET_ASSIGNMENT_PATTERN_LEFT)
                as *const BindingPattern<'a>)
        }
    }
}

impl<'a, 't> GetAddress for AssignmentPatternWithoutRight<'a, 't> {
    #[inline]
    fn address(&self) -> Address {
        unsafe { Address::from_ptr(self.0) }
    }
}

pub(crate) const OFFSET_OBJECT_PATTERN_NODE_ID: usize = offset_of!(ObjectPattern, node_id);
pub(crate) const OFFSET_OBJECT_PATTERN_SPAN: usize = offset_of!(ObjectPattern, span);
pub(crate) const OFFSET_OBJECT_PATTERN_PROPERTIES: usize = offset_of!(ObjectPattern, properties);
pub(crate) const OFFSET_OBJECT_PATTERN_REST: usize = offset_of!(ObjectPattern, rest);

#[repr(transparent)]
#[derive(Clone, Copy, Debug)]
pub struct ObjectPatternWithoutProperties<'a, 't>(
    pub(crate) *const ObjectPattern<'a>,
    pub(crate) PhantomData<&'t ()>,
);

impl<'a, 't> ObjectPatternWithoutProperties<'a, 't> {
    #[inline]
    pub fn node_id(self) -> &'t Cell<NodeId> {
        unsafe {
            &*((self.0 as *const u8).add(OFFSET_OBJECT_PATTERN_NODE_ID) as *const Cell<NodeId>)
        }
    }

    #[inline]
    pub fn span(self) -> &'t Span {
        unsafe { &*((self.0 as *const u8).add(OFFSET_OBJECT_PATTERN_SPAN) as *const Span) }
    }

    #[inline]
    pub fn rest(self) -> &'t Option<ArenaBox<'a, BindingRestElement<'a>>> {
        unsafe {
            &*((self.0 as *const u8).add(OFFSET_OBJECT_PATTERN_REST)
                as *const Option<ArenaBox<'a, BindingRestElement<'a>>>)
        }
    }
}

impl<'a, 't> GetAddress for ObjectPatternWithoutProperties<'a, 't> {
    #[inline]
    fn address(&self) -> Address {
        unsafe { Address::from_ptr(self.0) }
    }
}

#[repr(transparent)]
#[derive(Clone, Copy, Debug)]
pub struct ObjectPatternWithoutRest<'a, 't>(
    pub(crate) *const ObjectPattern<'a>,
    pub(crate) PhantomData<&'t ()>,
);

impl<'a, 't> ObjectPatternWithoutRest<'a, 't> {
    #[inline]
    pub fn node_id(self) -> &'t Cell<NodeId> {
        unsafe {
            &*((self.0 as *const u8).add(OFFSET_OBJECT_PATTERN_NODE_ID) as *const Cell<NodeId>)
        }
    }

    #[inline]
    pub fn span(self) -> &'t Span {
        unsafe { &*((self.0 as *const u8).add(OFFSET_OBJECT_PATTERN_SPAN) as *const Span) }
    }

    #[inline]
    pub fn properties(self) -> &'t ArenaVec<'a, BindingProperty<'a>> {
        unsafe {
            &*((self.0 as *const u8).add(OFFSET_OBJECT_PATTERN_PROPERTIES)
                as *const ArenaVec<'a, BindingProperty<'a>>)
        }
    }
}

impl<'a, 't> GetAddress for ObjectPatternWithoutRest<'a, 't> {
    #[inline]
    fn address(&self) -> Address {
        unsafe { Address::from_ptr(self.0) }
    }
}

pub(crate) const OFFSET_BINDING_PROPERTY_NODE_ID: usize = offset_of!(BindingProperty, node_id);
pub(crate) const OFFSET_BINDING_PROPERTY_SPAN: usize = offset_of!(BindingProperty, span);
pub(crate) const OFFSET_BINDING_PROPERTY_KEY: usize = offset_of!(BindingProperty, key);
pub(crate) const OFFSET_BINDING_PROPERTY_VALUE: usize = offset_of!(BindingProperty, value);
pub(crate) const OFFSET_BINDING_PROPERTY_SHORTHAND: usize = offset_of!(BindingProperty, shorthand);
pub(crate) const OFFSET_BINDING_PROPERTY_COMPUTED: usize = offset_of!(BindingProperty, computed);

#[repr(transparent)]
#[derive(Clone, Copy, Debug)]
pub struct BindingPropertyWithoutKey<'a, 't>(
    pub(crate) *const BindingProperty<'a>,
    pub(crate) PhantomData<&'t ()>,
);

impl<'a, 't> BindingPropertyWithoutKey<'a, 't> {
    #[inline]
    pub fn node_id(self) -> &'t Cell<NodeId> {
        unsafe {
            &*((self.0 as *const u8).add(OFFSET_BINDING_PROPERTY_NODE_ID) as *const Cell<NodeId>)
        }
    }

    #[inline]
    pub fn span(self) -> &'t Span {
        unsafe { &*((self.0 as *const u8).add(OFFSET_BINDING_PROPERTY_SPAN) as *const Span) }
    }

    #[inline]
    pub fn value(self) -> &'t BindingPattern<'a> {
        unsafe {
            &*((self.0 as *const u8).add(OFFSET_BINDING_PROPERTY_VALUE)
                as *const BindingPattern<'a>)
        }
    }

    #[inline]
    pub fn shorthand(self) -> &'t bool {
        unsafe { &*((self.0 as *const u8).add(OFFSET_BINDING_PROPERTY_SHORTHAND) as *const bool) }
    }

    #[inline]
    pub fn computed(self) -> &'t bool {
        unsafe { &*((self.0 as *const u8).add(OFFSET_BINDING_PROPERTY_COMPUTED) as *const bool) }
    }
}

impl<'a, 't> GetAddress for BindingPropertyWithoutKey<'a, 't> {
    #[inline]
    fn address(&self) -> Address {
        unsafe { Address::from_ptr(self.0) }
    }
}

#[repr(transparent)]
#[derive(Clone, Copy, Debug)]
pub struct BindingPropertyWithoutValue<'a, 't>(
    pub(crate) *const BindingProperty<'a>,
    pub(crate) PhantomData<&'t ()>,
);

impl<'a, 't> BindingPropertyWithoutValue<'a, 't> {
    #[inline]
    pub fn node_id(self) -> &'t Cell<NodeId> {
        unsafe {
            &*((self.0 as *const u8).add(OFFSET_BINDING_PROPERTY_NODE_ID) as *const Cell<NodeId>)
        }
    }

    #[inline]
    pub fn span(self) -> &'t Span {
        unsafe { &*((self.0 as *const u8).add(OFFSET_BINDING_PROPERTY_SPAN) as *const Span) }
    }

    #[inline]
    pub fn key(self) -> &'t PropertyKey<'a> {
        unsafe {
            &*((self.0 as *const u8).add(OFFSET_BINDING_PROPERTY_KEY) as *const PropertyKey<'a>)
        }
    }

    #[inline]
    pub fn shorthand(self) -> &'t bool {
        unsafe { &*((self.0 as *const u8).add(OFFSET_BINDING_PROPERTY_SHORTHAND) as *const bool) }
    }

    #[inline]
    pub fn computed(self) -> &'t bool {
        unsafe { &*((self.0 as *const u8).add(OFFSET_BINDING_PROPERTY_COMPUTED) as *const bool) }
    }
}

impl<'a, 't> GetAddress for BindingPropertyWithoutValue<'a, 't> {
    #[inline]
    fn address(&self) -> Address {
        unsafe { Address::from_ptr(self.0) }
    }
}

pub(crate) const OFFSET_ARRAY_PATTERN_NODE_ID: usize = offset_of!(ArrayPattern, node_id);
pub(crate) const OFFSET_ARRAY_PATTERN_SPAN: usize = offset_of!(ArrayPattern, span);
pub(crate) const OFFSET_ARRAY_PATTERN_ELEMENTS: usize = offset_of!(ArrayPattern, elements);
pub(crate) const OFFSET_ARRAY_PATTERN_REST: usize = offset_of!(ArrayPattern, rest);

#[repr(transparent)]
#[derive(Clone, Copy, Debug)]
pub struct ArrayPatternWithoutElements<'a, 't>(
    pub(crate) *const ArrayPattern<'a>,
    pub(crate) PhantomData<&'t ()>,
);

impl<'a, 't> ArrayPatternWithoutElements<'a, 't> {
    #[inline]
    pub fn node_id(self) -> &'t Cell<NodeId> {
        unsafe {
            &*((self.0 as *const u8).add(OFFSET_ARRAY_PATTERN_NODE_ID) as *const Cell<NodeId>)
        }
    }

    #[inline]
    pub fn span(self) -> &'t Span {
        unsafe { &*((self.0 as *const u8).add(OFFSET_ARRAY_PATTERN_SPAN) as *const Span) }
    }

    #[inline]
    pub fn rest(self) -> &'t Option<ArenaBox<'a, BindingRestElement<'a>>> {
        unsafe {
            &*((self.0 as *const u8).add(OFFSET_ARRAY_PATTERN_REST)
                as *const Option<ArenaBox<'a, BindingRestElement<'a>>>)
        }
    }
}

impl<'a, 't> GetAddress for ArrayPatternWithoutElements<'a, 't> {
    #[inline]
    fn address(&self) -> Address {
        unsafe { Address::from_ptr(self.0) }
    }
}

#[repr(transparent)]
#[derive(Clone, Copy, Debug)]
pub struct ArrayPatternWithoutRest<'a, 't>(
    pub(crate) *const ArrayPattern<'a>,
    pub(crate) PhantomData<&'t ()>,
);

impl<'a, 't> ArrayPatternWithoutRest<'a, 't> {
    #[inline]
    pub fn node_id(self) -> &'t Cell<NodeId> {
        unsafe {
            &*((self.0 as *const u8).add(OFFSET_ARRAY_PATTERN_NODE_ID) as *const Cell<NodeId>)
        }
    }

    #[inline]
    pub fn span(self) -> &'t Span {
        unsafe { &*((self.0 as *const u8).add(OFFSET_ARRAY_PATTERN_SPAN) as *const Span) }
    }

    #[inline]
    pub fn elements(self) -> &'t ArenaVec<'a, Option<BindingPattern<'a>>> {
        unsafe {
            &*((self.0 as *const u8).add(OFFSET_ARRAY_PATTERN_ELEMENTS)
                as *const ArenaVec<'a, Option<BindingPattern<'a>>>)
        }
    }
}

impl<'a, 't> GetAddress for ArrayPatternWithoutRest<'a, 't> {
    #[inline]
    fn address(&self) -> Address {
        unsafe { Address::from_ptr(self.0) }
    }
}

pub(crate) const OFFSET_BINDING_REST_ELEMENT_NODE_ID: usize =
    offset_of!(BindingRestElement, node_id);
pub(crate) const OFFSET_BINDING_REST_ELEMENT_SPAN: usize = offset_of!(BindingRestElement, span);
pub(crate) const OFFSET_BINDING_REST_ELEMENT_ARGUMENT: usize =
    offset_of!(BindingRestElement, argument);

#[repr(transparent)]
#[derive(Clone, Copy, Debug)]
pub struct BindingRestElementWithoutArgument<'a, 't>(
    pub(crate) *const BindingRestElement<'a>,
    pub(crate) PhantomData<&'t ()>,
);

impl<'a, 't> BindingRestElementWithoutArgument<'a, 't> {
    #[inline]
    pub fn node_id(self) -> &'t Cell<NodeId> {
        unsafe {
            &*((self.0 as *const u8).add(OFFSET_BINDING_REST_ELEMENT_NODE_ID)
                as *const Cell<NodeId>)
        }
    }

    #[inline]
    pub fn span(self) -> &'t Span {
        unsafe { &*((self.0 as *const u8).add(OFFSET_BINDING_REST_ELEMENT_SPAN) as *const Span) }
    }
}

impl<'a, 't> GetAddress for BindingRestElementWithoutArgument<'a, 't> {
    #[inline]
    fn address(&self) -> Address {
        unsafe { Address::from_ptr(self.0) }
    }
}

pub(crate) const OFFSET_FUNCTION_NODE_ID: usize = offset_of!(Function, node_id);
pub(crate) const OFFSET_FUNCTION_SPAN: usize = offset_of!(Function, span);
pub(crate) const OFFSET_FUNCTION_TYPE: usize = offset_of!(Function, r#type);
pub(crate) const OFFSET_FUNCTION_ID: usize = offset_of!(Function, id);
pub(crate) const OFFSET_FUNCTION_GENERATOR: usize = offset_of!(Function, generator);
pub(crate) const OFFSET_FUNCTION_ASYNC: usize = offset_of!(Function, r#async);
pub(crate) const OFFSET_FUNCTION_DECLARE: usize = offset_of!(Function, declare);
pub(crate) const OFFSET_FUNCTION_TYPE_PARAMETERS: usize = offset_of!(Function, type_parameters);
pub(crate) const OFFSET_FUNCTION_THIS_PARAM: usize = offset_of!(Function, this_param);
pub(crate) const OFFSET_FUNCTION_PARAMS: usize = offset_of!(Function, params);
pub(crate) const OFFSET_FUNCTION_RETURN_TYPE: usize = offset_of!(Function, return_type);
pub(crate) const OFFSET_FUNCTION_BODY: usize = offset_of!(Function, body);
pub(crate) const OFFSET_FUNCTION_SCOPE_ID: usize = offset_of!(Function, scope_id);
pub(crate) const OFFSET_FUNCTION_PURE: usize = offset_of!(Function, pure);
pub(crate) const OFFSET_FUNCTION_PIFE: usize = offset_of!(Function, pife);

#[repr(transparent)]
#[derive(Clone, Copy, Debug)]
pub struct FunctionWithoutId<'a, 't>(
    pub(crate) *const Function<'a>,
    pub(crate) PhantomData<&'t ()>,
);

impl<'a, 't> FunctionWithoutId<'a, 't> {
    #[inline]
    pub fn node_id(self) -> &'t Cell<NodeId> {
        unsafe { &*((self.0 as *const u8).add(OFFSET_FUNCTION_NODE_ID) as *const Cell<NodeId>) }
    }

    #[inline]
    pub fn span(self) -> &'t Span {
        unsafe { &*((self.0 as *const u8).add(OFFSET_FUNCTION_SPAN) as *const Span) }
    }

    #[inline]
    pub fn r#type(self) -> &'t FunctionType {
        unsafe { &*((self.0 as *const u8).add(OFFSET_FUNCTION_TYPE) as *const FunctionType) }
    }

    #[inline]
    pub fn generator(self) -> &'t bool {
        unsafe { &*((self.0 as *const u8).add(OFFSET_FUNCTION_GENERATOR) as *const bool) }
    }

    #[inline]
    pub fn r#async(self) -> &'t bool {
        unsafe { &*((self.0 as *const u8).add(OFFSET_FUNCTION_ASYNC) as *const bool) }
    }

    #[inline]
    pub fn declare(self) -> &'t bool {
        unsafe { &*((self.0 as *const u8).add(OFFSET_FUNCTION_DECLARE) as *const bool) }
    }

    #[inline]
    pub fn type_parameters(self) -> &'t Option<ArenaBox<'a, TSTypeParameterDeclaration<'a>>> {
        unsafe {
            &*((self.0 as *const u8).add(OFFSET_FUNCTION_TYPE_PARAMETERS)
                as *const Option<ArenaBox<'a, TSTypeParameterDeclaration<'a>>>)
        }
    }

    #[inline]
    pub fn this_param(self) -> &'t Option<ArenaBox<'a, TSThisParameter<'a>>> {
        unsafe {
            &*((self.0 as *const u8).add(OFFSET_FUNCTION_THIS_PARAM)
                as *const Option<ArenaBox<'a, TSThisParameter<'a>>>)
        }
    }

    #[inline]
    pub fn params(self) -> &'t ArenaBox<'a, FormalParameters<'a>> {
        unsafe {
            &*((self.0 as *const u8).add(OFFSET_FUNCTION_PARAMS)
                as *const ArenaBox<'a, FormalParameters<'a>>)
        }
    }

    #[inline]
    pub fn return_type(self) -> &'t Option<ArenaBox<'a, TSTypeAnnotation<'a>>> {
        unsafe {
            &*((self.0 as *const u8).add(OFFSET_FUNCTION_RETURN_TYPE)
                as *const Option<ArenaBox<'a, TSTypeAnnotation<'a>>>)
        }
    }

    #[inline]
    pub fn body(self) -> &'t Option<ArenaBox<'a, FunctionBody<'a>>> {
        unsafe {
            &*((self.0 as *const u8).add(OFFSET_FUNCTION_BODY)
                as *const Option<ArenaBox<'a, FunctionBody<'a>>>)
        }
    }

    #[inline]
    pub fn scope_id(self) -> &'t Cell<Option<ScopeId>> {
        unsafe {
            &*((self.0 as *const u8).add(OFFSET_FUNCTION_SCOPE_ID) as *const Cell<Option<ScopeId>>)
        }
    }

    #[inline]
    pub fn pure(self) -> &'t bool {
        unsafe { &*((self.0 as *const u8).add(OFFSET_FUNCTION_PURE) as *const bool) }
    }

    #[inline]
    pub fn pife(self) -> &'t bool {
        unsafe { &*((self.0 as *const u8).add(OFFSET_FUNCTION_PIFE) as *const bool) }
    }
}

impl<'a, 't> GetAddress for FunctionWithoutId<'a, 't> {
    #[inline]
    fn address(&self) -> Address {
        unsafe { Address::from_ptr(self.0) }
    }
}

#[repr(transparent)]
#[derive(Clone, Copy, Debug)]
pub struct FunctionWithoutParams<'a, 't>(
    pub(crate) *const Function<'a>,
    pub(crate) PhantomData<&'t ()>,
);

impl<'a, 't> FunctionWithoutParams<'a, 't> {
    #[inline]
    pub fn node_id(self) -> &'t Cell<NodeId> {
        unsafe { &*((self.0 as *const u8).add(OFFSET_FUNCTION_NODE_ID) as *const Cell<NodeId>) }
    }

    #[inline]
    pub fn span(self) -> &'t Span {
        unsafe { &*((self.0 as *const u8).add(OFFSET_FUNCTION_SPAN) as *const Span) }
    }

    #[inline]
    pub fn r#type(self) -> &'t FunctionType {
        unsafe { &*((self.0 as *const u8).add(OFFSET_FUNCTION_TYPE) as *const FunctionType) }
    }

    #[inline]
    pub fn id(self) -> &'t Option<BindingIdentifier<'a>> {
        unsafe {
            &*((self.0 as *const u8).add(OFFSET_FUNCTION_ID)
                as *const Option<BindingIdentifier<'a>>)
        }
    }

    #[inline]
    pub fn generator(self) -> &'t bool {
        unsafe { &*((self.0 as *const u8).add(OFFSET_FUNCTION_GENERATOR) as *const bool) }
    }

    #[inline]
    pub fn r#async(self) -> &'t bool {
        unsafe { &*((self.0 as *const u8).add(OFFSET_FUNCTION_ASYNC) as *const bool) }
    }

    #[inline]
    pub fn declare(self) -> &'t bool {
        unsafe { &*((self.0 as *const u8).add(OFFSET_FUNCTION_DECLARE) as *const bool) }
    }

    #[inline]
    pub fn type_parameters(self) -> &'t Option<ArenaBox<'a, TSTypeParameterDeclaration<'a>>> {
        unsafe {
            &*((self.0 as *const u8).add(OFFSET_FUNCTION_TYPE_PARAMETERS)
                as *const Option<ArenaBox<'a, TSTypeParameterDeclaration<'a>>>)
        }
    }

    #[inline]
    pub fn this_param(self) -> &'t Option<ArenaBox<'a, TSThisParameter<'a>>> {
        unsafe {
            &*((self.0 as *const u8).add(OFFSET_FUNCTION_THIS_PARAM)
                as *const Option<ArenaBox<'a, TSThisParameter<'a>>>)
        }
    }

    #[inline]
    pub fn return_type(self) -> &'t Option<ArenaBox<'a, TSTypeAnnotation<'a>>> {
        unsafe {
            &*((self.0 as *const u8).add(OFFSET_FUNCTION_RETURN_TYPE)
                as *const Option<ArenaBox<'a, TSTypeAnnotation<'a>>>)
        }
    }

    #[inline]
    pub fn body(self) -> &'t Option<ArenaBox<'a, FunctionBody<'a>>> {
        unsafe {
            &*((self.0 as *const u8).add(OFFSET_FUNCTION_BODY)
                as *const Option<ArenaBox<'a, FunctionBody<'a>>>)
        }
    }

    #[inline]
    pub fn scope_id(self) -> &'t Cell<Option<ScopeId>> {
        unsafe {
            &*((self.0 as *const u8).add(OFFSET_FUNCTION_SCOPE_ID) as *const Cell<Option<ScopeId>>)
        }
    }

    #[inline]
    pub fn pure(self) -> &'t bool {
        unsafe { &*((self.0 as *const u8).add(OFFSET_FUNCTION_PURE) as *const bool) }
    }

    #[inline]
    pub fn pife(self) -> &'t bool {
        unsafe { &*((self.0 as *const u8).add(OFFSET_FUNCTION_PIFE) as *const bool) }
    }
}

impl<'a, 't> GetAddress for FunctionWithoutParams<'a, 't> {
    #[inline]
    fn address(&self) -> Address {
        unsafe { Address::from_ptr(self.0) }
    }
}

#[repr(transparent)]
#[derive(Clone, Copy, Debug)]
pub struct FunctionWithoutBody<'a, 't>(
    pub(crate) *const Function<'a>,
    pub(crate) PhantomData<&'t ()>,
);

impl<'a, 't> FunctionWithoutBody<'a, 't> {
    #[inline]
    pub fn node_id(self) -> &'t Cell<NodeId> {
        unsafe { &*((self.0 as *const u8).add(OFFSET_FUNCTION_NODE_ID) as *const Cell<NodeId>) }
    }

    #[inline]
    pub fn span(self) -> &'t Span {
        unsafe { &*((self.0 as *const u8).add(OFFSET_FUNCTION_SPAN) as *const Span) }
    }

    #[inline]
    pub fn r#type(self) -> &'t FunctionType {
        unsafe { &*((self.0 as *const u8).add(OFFSET_FUNCTION_TYPE) as *const FunctionType) }
    }

    #[inline]
    pub fn id(self) -> &'t Option<BindingIdentifier<'a>> {
        unsafe {
            &*((self.0 as *const u8).add(OFFSET_FUNCTION_ID)
                as *const Option<BindingIdentifier<'a>>)
        }
    }

    #[inline]
    pub fn generator(self) -> &'t bool {
        unsafe { &*((self.0 as *const u8).add(OFFSET_FUNCTION_GENERATOR) as *const bool) }
    }

    #[inline]
    pub fn r#async(self) -> &'t bool {
        unsafe { &*((self.0 as *const u8).add(OFFSET_FUNCTION_ASYNC) as *const bool) }
    }

    #[inline]
    pub fn declare(self) -> &'t bool {
        unsafe { &*((self.0 as *const u8).add(OFFSET_FUNCTION_DECLARE) as *const bool) }
    }

    #[inline]
    pub fn type_parameters(self) -> &'t Option<ArenaBox<'a, TSTypeParameterDeclaration<'a>>> {
        unsafe {
            &*((self.0 as *const u8).add(OFFSET_FUNCTION_TYPE_PARAMETERS)
                as *const Option<ArenaBox<'a, TSTypeParameterDeclaration<'a>>>)
        }
    }

    #[inline]
    pub fn this_param(self) -> &'t Option<ArenaBox<'a, TSThisParameter<'a>>> {
        unsafe {
            &*((self.0 as *const u8).add(OFFSET_FUNCTION_THIS_PARAM)
                as *const Option<ArenaBox<'a, TSThisParameter<'a>>>)
        }
    }

    #[inline]
    pub fn params(self) -> &'t ArenaBox<'a, FormalParameters<'a>> {
        unsafe {
            &*((self.0 as *const u8).add(OFFSET_FUNCTION_PARAMS)
                as *const ArenaBox<'a, FormalParameters<'a>>)
        }
    }

    #[inline]
    pub fn return_type(self) -> &'t Option<ArenaBox<'a, TSTypeAnnotation<'a>>> {
        unsafe {
            &*((self.0 as *const u8).add(OFFSET_FUNCTION_RETURN_TYPE)
                as *const Option<ArenaBox<'a, TSTypeAnnotation<'a>>>)
        }
    }

    #[inline]
    pub fn scope_id(self) -> &'t Cell<Option<ScopeId>> {
        unsafe {
            &*((self.0 as *const u8).add(OFFSET_FUNCTION_SCOPE_ID) as *const Cell<Option<ScopeId>>)
        }
    }

    #[inline]
    pub fn pure(self) -> &'t bool {
        unsafe { &*((self.0 as *const u8).add(OFFSET_FUNCTION_PURE) as *const bool) }
    }

    #[inline]
    pub fn pife(self) -> &'t bool {
        unsafe { &*((self.0 as *const u8).add(OFFSET_FUNCTION_PIFE) as *const bool) }
    }
}

impl<'a, 't> GetAddress for FunctionWithoutBody<'a, 't> {
    #[inline]
    fn address(&self) -> Address {
        unsafe { Address::from_ptr(self.0) }
    }
}

pub(crate) const OFFSET_FORMAL_PARAMETERS_NODE_ID: usize = offset_of!(FormalParameters, node_id);
pub(crate) const OFFSET_FORMAL_PARAMETERS_SPAN: usize = offset_of!(FormalParameters, span);
pub(crate) const OFFSET_FORMAL_PARAMETERS_KIND: usize = offset_of!(FormalParameters, kind);
pub(crate) const OFFSET_FORMAL_PARAMETERS_ITEMS: usize = offset_of!(FormalParameters, items);
pub(crate) const OFFSET_FORMAL_PARAMETERS_REST: usize = offset_of!(FormalParameters, rest);

#[repr(transparent)]
#[derive(Clone, Copy, Debug)]
pub struct FormalParametersWithoutItems<'a, 't>(
    pub(crate) *const FormalParameters<'a>,
    pub(crate) PhantomData<&'t ()>,
);

impl<'a, 't> FormalParametersWithoutItems<'a, 't> {
    #[inline]
    pub fn node_id(self) -> &'t Cell<NodeId> {
        unsafe {
            &*((self.0 as *const u8).add(OFFSET_FORMAL_PARAMETERS_NODE_ID) as *const Cell<NodeId>)
        }
    }

    #[inline]
    pub fn span(self) -> &'t Span {
        unsafe { &*((self.0 as *const u8).add(OFFSET_FORMAL_PARAMETERS_SPAN) as *const Span) }
    }

    #[inline]
    pub fn kind(self) -> &'t FormalParameterKind {
        unsafe {
            &*((self.0 as *const u8).add(OFFSET_FORMAL_PARAMETERS_KIND)
                as *const FormalParameterKind)
        }
    }

    #[inline]
    pub fn rest(self) -> &'t Option<ArenaBox<'a, FormalParameterRest<'a>>> {
        unsafe {
            &*((self.0 as *const u8).add(OFFSET_FORMAL_PARAMETERS_REST)
                as *const Option<ArenaBox<'a, FormalParameterRest<'a>>>)
        }
    }
}

impl<'a, 't> GetAddress for FormalParametersWithoutItems<'a, 't> {
    #[inline]
    fn address(&self) -> Address {
        unsafe { Address::from_ptr(self.0) }
    }
}

#[repr(transparent)]
#[derive(Clone, Copy, Debug)]
pub struct FormalParametersWithoutRest<'a, 't>(
    pub(crate) *const FormalParameters<'a>,
    pub(crate) PhantomData<&'t ()>,
);

impl<'a, 't> FormalParametersWithoutRest<'a, 't> {
    #[inline]
    pub fn node_id(self) -> &'t Cell<NodeId> {
        unsafe {
            &*((self.0 as *const u8).add(OFFSET_FORMAL_PARAMETERS_NODE_ID) as *const Cell<NodeId>)
        }
    }

    #[inline]
    pub fn span(self) -> &'t Span {
        unsafe { &*((self.0 as *const u8).add(OFFSET_FORMAL_PARAMETERS_SPAN) as *const Span) }
    }

    #[inline]
    pub fn kind(self) -> &'t FormalParameterKind {
        unsafe {
            &*((self.0 as *const u8).add(OFFSET_FORMAL_PARAMETERS_KIND)
                as *const FormalParameterKind)
        }
    }

    #[inline]
    pub fn items(self) -> &'t ArenaVec<'a, FormalParameter<'a>> {
        unsafe {
            &*((self.0 as *const u8).add(OFFSET_FORMAL_PARAMETERS_ITEMS)
                as *const ArenaVec<'a, FormalParameter<'a>>)
        }
    }
}

impl<'a, 't> GetAddress for FormalParametersWithoutRest<'a, 't> {
    #[inline]
    fn address(&self) -> Address {
        unsafe { Address::from_ptr(self.0) }
    }
}

pub(crate) const OFFSET_FORMAL_PARAMETER_NODE_ID: usize = offset_of!(FormalParameter, node_id);
pub(crate) const OFFSET_FORMAL_PARAMETER_SPAN: usize = offset_of!(FormalParameter, span);
pub(crate) const OFFSET_FORMAL_PARAMETER_DECORATORS: usize =
    offset_of!(FormalParameter, decorators);
pub(crate) const OFFSET_FORMAL_PARAMETER_PATTERN: usize = offset_of!(FormalParameter, pattern);
pub(crate) const OFFSET_FORMAL_PARAMETER_TYPE_ANNOTATION: usize =
    offset_of!(FormalParameter, type_annotation);
pub(crate) const OFFSET_FORMAL_PARAMETER_INITIALIZER: usize =
    offset_of!(FormalParameter, initializer);
pub(crate) const OFFSET_FORMAL_PARAMETER_OPTIONAL: usize = offset_of!(FormalParameter, optional);
pub(crate) const OFFSET_FORMAL_PARAMETER_ACCESSIBILITY: usize =
    offset_of!(FormalParameter, accessibility);
pub(crate) const OFFSET_FORMAL_PARAMETER_READONLY: usize = offset_of!(FormalParameter, readonly);
pub(crate) const OFFSET_FORMAL_PARAMETER_OVERRIDE: usize = offset_of!(FormalParameter, r#override);

#[repr(transparent)]
#[derive(Clone, Copy, Debug)]
pub struct FormalParameterWithoutDecorators<'a, 't>(
    pub(crate) *const FormalParameter<'a>,
    pub(crate) PhantomData<&'t ()>,
);

impl<'a, 't> FormalParameterWithoutDecorators<'a, 't> {
    #[inline]
    pub fn node_id(self) -> &'t Cell<NodeId> {
        unsafe {
            &*((self.0 as *const u8).add(OFFSET_FORMAL_PARAMETER_NODE_ID) as *const Cell<NodeId>)
        }
    }

    #[inline]
    pub fn span(self) -> &'t Span {
        unsafe { &*((self.0 as *const u8).add(OFFSET_FORMAL_PARAMETER_SPAN) as *const Span) }
    }

    #[inline]
    pub fn pattern(self) -> &'t BindingPattern<'a> {
        unsafe {
            &*((self.0 as *const u8).add(OFFSET_FORMAL_PARAMETER_PATTERN)
                as *const BindingPattern<'a>)
        }
    }

    #[inline]
    pub fn type_annotation(self) -> &'t Option<ArenaBox<'a, TSTypeAnnotation<'a>>> {
        unsafe {
            &*((self.0 as *const u8).add(OFFSET_FORMAL_PARAMETER_TYPE_ANNOTATION)
                as *const Option<ArenaBox<'a, TSTypeAnnotation<'a>>>)
        }
    }

    #[inline]
    pub fn initializer(self) -> &'t Option<ArenaBox<'a, Expression<'a>>> {
        unsafe {
            &*((self.0 as *const u8).add(OFFSET_FORMAL_PARAMETER_INITIALIZER)
                as *const Option<ArenaBox<'a, Expression<'a>>>)
        }
    }

    #[inline]
    pub fn optional(self) -> &'t bool {
        unsafe { &*((self.0 as *const u8).add(OFFSET_FORMAL_PARAMETER_OPTIONAL) as *const bool) }
    }

    #[inline]
    pub fn accessibility(self) -> &'t Option<TSAccessibility> {
        unsafe {
            &*((self.0 as *const u8).add(OFFSET_FORMAL_PARAMETER_ACCESSIBILITY)
                as *const Option<TSAccessibility>)
        }
    }

    #[inline]
    pub fn readonly(self) -> &'t bool {
        unsafe { &*((self.0 as *const u8).add(OFFSET_FORMAL_PARAMETER_READONLY) as *const bool) }
    }

    #[inline]
    pub fn r#override(self) -> &'t bool {
        unsafe { &*((self.0 as *const u8).add(OFFSET_FORMAL_PARAMETER_OVERRIDE) as *const bool) }
    }
}

impl<'a, 't> GetAddress for FormalParameterWithoutDecorators<'a, 't> {
    #[inline]
    fn address(&self) -> Address {
        unsafe { Address::from_ptr(self.0) }
    }
}

#[repr(transparent)]
#[derive(Clone, Copy, Debug)]
pub struct FormalParameterWithoutPattern<'a, 't>(
    pub(crate) *const FormalParameter<'a>,
    pub(crate) PhantomData<&'t ()>,
);

impl<'a, 't> FormalParameterWithoutPattern<'a, 't> {
    #[inline]
    pub fn node_id(self) -> &'t Cell<NodeId> {
        unsafe {
            &*((self.0 as *const u8).add(OFFSET_FORMAL_PARAMETER_NODE_ID) as *const Cell<NodeId>)
        }
    }

    #[inline]
    pub fn span(self) -> &'t Span {
        unsafe { &*((self.0 as *const u8).add(OFFSET_FORMAL_PARAMETER_SPAN) as *const Span) }
    }

    #[inline]
    pub fn decorators(self) -> &'t ArenaVec<'a, Decorator<'a>> {
        unsafe {
            &*((self.0 as *const u8).add(OFFSET_FORMAL_PARAMETER_DECORATORS)
                as *const ArenaVec<'a, Decorator<'a>>)
        }
    }

    #[inline]
    pub fn type_annotation(self) -> &'t Option<ArenaBox<'a, TSTypeAnnotation<'a>>> {
        unsafe {
            &*((self.0 as *const u8).add(OFFSET_FORMAL_PARAMETER_TYPE_ANNOTATION)
                as *const Option<ArenaBox<'a, TSTypeAnnotation<'a>>>)
        }
    }

    #[inline]
    pub fn initializer(self) -> &'t Option<ArenaBox<'a, Expression<'a>>> {
        unsafe {
            &*((self.0 as *const u8).add(OFFSET_FORMAL_PARAMETER_INITIALIZER)
                as *const Option<ArenaBox<'a, Expression<'a>>>)
        }
    }

    #[inline]
    pub fn optional(self) -> &'t bool {
        unsafe { &*((self.0 as *const u8).add(OFFSET_FORMAL_PARAMETER_OPTIONAL) as *const bool) }
    }

    #[inline]
    pub fn accessibility(self) -> &'t Option<TSAccessibility> {
        unsafe {
            &*((self.0 as *const u8).add(OFFSET_FORMAL_PARAMETER_ACCESSIBILITY)
                as *const Option<TSAccessibility>)
        }
    }

    #[inline]
    pub fn readonly(self) -> &'t bool {
        unsafe { &*((self.0 as *const u8).add(OFFSET_FORMAL_PARAMETER_READONLY) as *const bool) }
    }

    #[inline]
    pub fn r#override(self) -> &'t bool {
        unsafe { &*((self.0 as *const u8).add(OFFSET_FORMAL_PARAMETER_OVERRIDE) as *const bool) }
    }
}

impl<'a, 't> GetAddress for FormalParameterWithoutPattern<'a, 't> {
    #[inline]
    fn address(&self) -> Address {
        unsafe { Address::from_ptr(self.0) }
    }
}

#[repr(transparent)]
#[derive(Clone, Copy, Debug)]
pub struct FormalParameterWithoutInitializer<'a, 't>(
    pub(crate) *const FormalParameter<'a>,
    pub(crate) PhantomData<&'t ()>,
);

impl<'a, 't> FormalParameterWithoutInitializer<'a, 't> {
    #[inline]
    pub fn node_id(self) -> &'t Cell<NodeId> {
        unsafe {
            &*((self.0 as *const u8).add(OFFSET_FORMAL_PARAMETER_NODE_ID) as *const Cell<NodeId>)
        }
    }

    #[inline]
    pub fn span(self) -> &'t Span {
        unsafe { &*((self.0 as *const u8).add(OFFSET_FORMAL_PARAMETER_SPAN) as *const Span) }
    }

    #[inline]
    pub fn decorators(self) -> &'t ArenaVec<'a, Decorator<'a>> {
        unsafe {
            &*((self.0 as *const u8).add(OFFSET_FORMAL_PARAMETER_DECORATORS)
                as *const ArenaVec<'a, Decorator<'a>>)
        }
    }

    #[inline]
    pub fn pattern(self) -> &'t BindingPattern<'a> {
        unsafe {
            &*((self.0 as *const u8).add(OFFSET_FORMAL_PARAMETER_PATTERN)
                as *const BindingPattern<'a>)
        }
    }

    #[inline]
    pub fn type_annotation(self) -> &'t Option<ArenaBox<'a, TSTypeAnnotation<'a>>> {
        unsafe {
            &*((self.0 as *const u8).add(OFFSET_FORMAL_PARAMETER_TYPE_ANNOTATION)
                as *const Option<ArenaBox<'a, TSTypeAnnotation<'a>>>)
        }
    }

    #[inline]
    pub fn optional(self) -> &'t bool {
        unsafe { &*((self.0 as *const u8).add(OFFSET_FORMAL_PARAMETER_OPTIONAL) as *const bool) }
    }

    #[inline]
    pub fn accessibility(self) -> &'t Option<TSAccessibility> {
        unsafe {
            &*((self.0 as *const u8).add(OFFSET_FORMAL_PARAMETER_ACCESSIBILITY)
                as *const Option<TSAccessibility>)
        }
    }

    #[inline]
    pub fn readonly(self) -> &'t bool {
        unsafe { &*((self.0 as *const u8).add(OFFSET_FORMAL_PARAMETER_READONLY) as *const bool) }
    }

    #[inline]
    pub fn r#override(self) -> &'t bool {
        unsafe { &*((self.0 as *const u8).add(OFFSET_FORMAL_PARAMETER_OVERRIDE) as *const bool) }
    }
}

impl<'a, 't> GetAddress for FormalParameterWithoutInitializer<'a, 't> {
    #[inline]
    fn address(&self) -> Address {
        unsafe { Address::from_ptr(self.0) }
    }
}

pub(crate) const OFFSET_FORMAL_PARAMETER_REST_NODE_ID: usize =
    offset_of!(FormalParameterRest, node_id);
pub(crate) const OFFSET_FORMAL_PARAMETER_REST_SPAN: usize = offset_of!(FormalParameterRest, span);
pub(crate) const OFFSET_FORMAL_PARAMETER_REST_DECORATORS: usize =
    offset_of!(FormalParameterRest, decorators);
pub(crate) const OFFSET_FORMAL_PARAMETER_REST_REST: usize = offset_of!(FormalParameterRest, rest);
pub(crate) const OFFSET_FORMAL_PARAMETER_REST_TYPE_ANNOTATION: usize =
    offset_of!(FormalParameterRest, type_annotation);

#[repr(transparent)]
#[derive(Clone, Copy, Debug)]
pub struct FormalParameterRestWithoutDecorators<'a, 't>(
    pub(crate) *const FormalParameterRest<'a>,
    pub(crate) PhantomData<&'t ()>,
);

impl<'a, 't> FormalParameterRestWithoutDecorators<'a, 't> {
    #[inline]
    pub fn node_id(self) -> &'t Cell<NodeId> {
        unsafe {
            &*((self.0 as *const u8).add(OFFSET_FORMAL_PARAMETER_REST_NODE_ID)
                as *const Cell<NodeId>)
        }
    }

    #[inline]
    pub fn span(self) -> &'t Span {
        unsafe { &*((self.0 as *const u8).add(OFFSET_FORMAL_PARAMETER_REST_SPAN) as *const Span) }
    }

    #[inline]
    pub fn rest(self) -> &'t BindingRestElement<'a> {
        unsafe {
            &*((self.0 as *const u8).add(OFFSET_FORMAL_PARAMETER_REST_REST)
                as *const BindingRestElement<'a>)
        }
    }

    #[inline]
    pub fn type_annotation(self) -> &'t Option<ArenaBox<'a, TSTypeAnnotation<'a>>> {
        unsafe {
            &*((self.0 as *const u8).add(OFFSET_FORMAL_PARAMETER_REST_TYPE_ANNOTATION)
                as *const Option<ArenaBox<'a, TSTypeAnnotation<'a>>>)
        }
    }
}

impl<'a, 't> GetAddress for FormalParameterRestWithoutDecorators<'a, 't> {
    #[inline]
    fn address(&self) -> Address {
        unsafe { Address::from_ptr(self.0) }
    }
}

#[repr(transparent)]
#[derive(Clone, Copy, Debug)]
pub struct FormalParameterRestWithoutRest<'a, 't>(
    pub(crate) *const FormalParameterRest<'a>,
    pub(crate) PhantomData<&'t ()>,
);

impl<'a, 't> FormalParameterRestWithoutRest<'a, 't> {
    #[inline]
    pub fn node_id(self) -> &'t Cell<NodeId> {
        unsafe {
            &*((self.0 as *const u8).add(OFFSET_FORMAL_PARAMETER_REST_NODE_ID)
                as *const Cell<NodeId>)
        }
    }

    #[inline]
    pub fn span(self) -> &'t Span {
        unsafe { &*((self.0 as *const u8).add(OFFSET_FORMAL_PARAMETER_REST_SPAN) as *const Span) }
    }

    #[inline]
    pub fn decorators(self) -> &'t ArenaVec<'a, Decorator<'a>> {
        unsafe {
            &*((self.0 as *const u8).add(OFFSET_FORMAL_PARAMETER_REST_DECORATORS)
                as *const ArenaVec<'a, Decorator<'a>>)
        }
    }

    #[inline]
    pub fn type_annotation(self) -> &'t Option<ArenaBox<'a, TSTypeAnnotation<'a>>> {
        unsafe {
            &*((self.0 as *const u8).add(OFFSET_FORMAL_PARAMETER_REST_TYPE_ANNOTATION)
                as *const Option<ArenaBox<'a, TSTypeAnnotation<'a>>>)
        }
    }
}

impl<'a, 't> GetAddress for FormalParameterRestWithoutRest<'a, 't> {
    #[inline]
    fn address(&self) -> Address {
        unsafe { Address::from_ptr(self.0) }
    }
}

pub(crate) const OFFSET_FUNCTION_BODY_NODE_ID: usize = offset_of!(FunctionBody, node_id);
pub(crate) const OFFSET_FUNCTION_BODY_SPAN: usize = offset_of!(FunctionBody, span);
pub(crate) const OFFSET_FUNCTION_BODY_DIRECTIVES: usize = offset_of!(FunctionBody, directives);
pub(crate) const OFFSET_FUNCTION_BODY_STATEMENTS: usize = offset_of!(FunctionBody, statements);

#[repr(transparent)]
#[derive(Clone, Copy, Debug)]
pub struct FunctionBodyWithoutDirectives<'a, 't>(
    pub(crate) *const FunctionBody<'a>,
    pub(crate) PhantomData<&'t ()>,
);

impl<'a, 't> FunctionBodyWithoutDirectives<'a, 't> {
    #[inline]
    pub fn node_id(self) -> &'t Cell<NodeId> {
        unsafe {
            &*((self.0 as *const u8).add(OFFSET_FUNCTION_BODY_NODE_ID) as *const Cell<NodeId>)
        }
    }

    #[inline]
    pub fn span(self) -> &'t Span {
        unsafe { &*((self.0 as *const u8).add(OFFSET_FUNCTION_BODY_SPAN) as *const Span) }
    }

    #[inline]
    pub fn statements(self) -> &'t ArenaVec<'a, Statement<'a>> {
        unsafe {
            &*((self.0 as *const u8).add(OFFSET_FUNCTION_BODY_STATEMENTS)
                as *const ArenaVec<'a, Statement<'a>>)
        }
    }
}

impl<'a, 't> GetAddress for FunctionBodyWithoutDirectives<'a, 't> {
    #[inline]
    fn address(&self) -> Address {
        unsafe { Address::from_ptr(self.0) }
    }
}

#[repr(transparent)]
#[derive(Clone, Copy, Debug)]
pub struct FunctionBodyWithoutStatements<'a, 't>(
    pub(crate) *const FunctionBody<'a>,
    pub(crate) PhantomData<&'t ()>,
);

impl<'a, 't> FunctionBodyWithoutStatements<'a, 't> {
    #[inline]
    pub fn node_id(self) -> &'t Cell<NodeId> {
        unsafe {
            &*((self.0 as *const u8).add(OFFSET_FUNCTION_BODY_NODE_ID) as *const Cell<NodeId>)
        }
    }

    #[inline]
    pub fn span(self) -> &'t Span {
        unsafe { &*((self.0 as *const u8).add(OFFSET_FUNCTION_BODY_SPAN) as *const Span) }
    }

    #[inline]
    pub fn directives(self) -> &'t ArenaVec<'a, Directive<'a>> {
        unsafe {
            &*((self.0 as *const u8).add(OFFSET_FUNCTION_BODY_DIRECTIVES)
                as *const ArenaVec<'a, Directive<'a>>)
        }
    }
}

impl<'a, 't> GetAddress for FunctionBodyWithoutStatements<'a, 't> {
    #[inline]
    fn address(&self) -> Address {
        unsafe { Address::from_ptr(self.0) }
    }
}

pub(crate) const OFFSET_ARROW_FUNCTION_EXPRESSION_NODE_ID: usize =
    offset_of!(ArrowFunctionExpression, node_id);
pub(crate) const OFFSET_ARROW_FUNCTION_EXPRESSION_SPAN: usize =
    offset_of!(ArrowFunctionExpression, span);
pub(crate) const OFFSET_ARROW_FUNCTION_EXPRESSION_EXPRESSION: usize =
    offset_of!(ArrowFunctionExpression, expression);
pub(crate) const OFFSET_ARROW_FUNCTION_EXPRESSION_ASYNC: usize =
    offset_of!(ArrowFunctionExpression, r#async);
pub(crate) const OFFSET_ARROW_FUNCTION_EXPRESSION_TYPE_PARAMETERS: usize =
    offset_of!(ArrowFunctionExpression, type_parameters);
pub(crate) const OFFSET_ARROW_FUNCTION_EXPRESSION_PARAMS: usize =
    offset_of!(ArrowFunctionExpression, params);
pub(crate) const OFFSET_ARROW_FUNCTION_EXPRESSION_RETURN_TYPE: usize =
    offset_of!(ArrowFunctionExpression, return_type);
pub(crate) const OFFSET_ARROW_FUNCTION_EXPRESSION_BODY: usize =
    offset_of!(ArrowFunctionExpression, body);
pub(crate) const OFFSET_ARROW_FUNCTION_EXPRESSION_SCOPE_ID: usize =
    offset_of!(ArrowFunctionExpression, scope_id);
pub(crate) const OFFSET_ARROW_FUNCTION_EXPRESSION_PURE: usize =
    offset_of!(ArrowFunctionExpression, pure);
pub(crate) const OFFSET_ARROW_FUNCTION_EXPRESSION_PIFE: usize =
    offset_of!(ArrowFunctionExpression, pife);

#[repr(transparent)]
#[derive(Clone, Copy, Debug)]
pub struct ArrowFunctionExpressionWithoutParams<'a, 't>(
    pub(crate) *const ArrowFunctionExpression<'a>,
    pub(crate) PhantomData<&'t ()>,
);

impl<'a, 't> ArrowFunctionExpressionWithoutParams<'a, 't> {
    #[inline]
    pub fn node_id(self) -> &'t Cell<NodeId> {
        unsafe {
            &*((self.0 as *const u8).add(OFFSET_ARROW_FUNCTION_EXPRESSION_NODE_ID)
                as *const Cell<NodeId>)
        }
    }

    #[inline]
    pub fn span(self) -> &'t Span {
        unsafe {
            &*((self.0 as *const u8).add(OFFSET_ARROW_FUNCTION_EXPRESSION_SPAN) as *const Span)
        }
    }

    #[inline]
    pub fn expression(self) -> &'t bool {
        unsafe {
            &*((self.0 as *const u8).add(OFFSET_ARROW_FUNCTION_EXPRESSION_EXPRESSION)
                as *const bool)
        }
    }

    #[inline]
    pub fn r#async(self) -> &'t bool {
        unsafe {
            &*((self.0 as *const u8).add(OFFSET_ARROW_FUNCTION_EXPRESSION_ASYNC) as *const bool)
        }
    }

    #[inline]
    pub fn type_parameters(self) -> &'t Option<ArenaBox<'a, TSTypeParameterDeclaration<'a>>> {
        unsafe {
            &*((self.0 as *const u8).add(OFFSET_ARROW_FUNCTION_EXPRESSION_TYPE_PARAMETERS)
                as *const Option<ArenaBox<'a, TSTypeParameterDeclaration<'a>>>)
        }
    }

    #[inline]
    pub fn return_type(self) -> &'t Option<ArenaBox<'a, TSTypeAnnotation<'a>>> {
        unsafe {
            &*((self.0 as *const u8).add(OFFSET_ARROW_FUNCTION_EXPRESSION_RETURN_TYPE)
                as *const Option<ArenaBox<'a, TSTypeAnnotation<'a>>>)
        }
    }

    #[inline]
    pub fn body(self) -> &'t ArenaBox<'a, FunctionBody<'a>> {
        unsafe {
            &*((self.0 as *const u8).add(OFFSET_ARROW_FUNCTION_EXPRESSION_BODY)
                as *const ArenaBox<'a, FunctionBody<'a>>)
        }
    }

    #[inline]
    pub fn scope_id(self) -> &'t Cell<Option<ScopeId>> {
        unsafe {
            &*((self.0 as *const u8).add(OFFSET_ARROW_FUNCTION_EXPRESSION_SCOPE_ID)
                as *const Cell<Option<ScopeId>>)
        }
    }

    #[inline]
    pub fn pure(self) -> &'t bool {
        unsafe {
            &*((self.0 as *const u8).add(OFFSET_ARROW_FUNCTION_EXPRESSION_PURE) as *const bool)
        }
    }

    #[inline]
    pub fn pife(self) -> &'t bool {
        unsafe {
            &*((self.0 as *const u8).add(OFFSET_ARROW_FUNCTION_EXPRESSION_PIFE) as *const bool)
        }
    }
}

impl<'a, 't> GetAddress for ArrowFunctionExpressionWithoutParams<'a, 't> {
    #[inline]
    fn address(&self) -> Address {
        unsafe { Address::from_ptr(self.0) }
    }
}

#[repr(transparent)]
#[derive(Clone, Copy, Debug)]
pub struct ArrowFunctionExpressionWithoutBody<'a, 't>(
    pub(crate) *const ArrowFunctionExpression<'a>,
    pub(crate) PhantomData<&'t ()>,
);

impl<'a, 't> ArrowFunctionExpressionWithoutBody<'a, 't> {
    #[inline]
    pub fn node_id(self) -> &'t Cell<NodeId> {
        unsafe {
            &*((self.0 as *const u8).add(OFFSET_ARROW_FUNCTION_EXPRESSION_NODE_ID)
                as *const Cell<NodeId>)
        }
    }

    #[inline]
    pub fn span(self) -> &'t Span {
        unsafe {
            &*((self.0 as *const u8).add(OFFSET_ARROW_FUNCTION_EXPRESSION_SPAN) as *const Span)
        }
    }

    #[inline]
    pub fn expression(self) -> &'t bool {
        unsafe {
            &*((self.0 as *const u8).add(OFFSET_ARROW_FUNCTION_EXPRESSION_EXPRESSION)
                as *const bool)
        }
    }

    #[inline]
    pub fn r#async(self) -> &'t bool {
        unsafe {
            &*((self.0 as *const u8).add(OFFSET_ARROW_FUNCTION_EXPRESSION_ASYNC) as *const bool)
        }
    }

    #[inline]
    pub fn type_parameters(self) -> &'t Option<ArenaBox<'a, TSTypeParameterDeclaration<'a>>> {
        unsafe {
            &*((self.0 as *const u8).add(OFFSET_ARROW_FUNCTION_EXPRESSION_TYPE_PARAMETERS)
                as *const Option<ArenaBox<'a, TSTypeParameterDeclaration<'a>>>)
        }
    }

    #[inline]
    pub fn params(self) -> &'t ArenaBox<'a, FormalParameters<'a>> {
        unsafe {
            &*((self.0 as *const u8).add(OFFSET_ARROW_FUNCTION_EXPRESSION_PARAMS)
                as *const ArenaBox<'a, FormalParameters<'a>>)
        }
    }

    #[inline]
    pub fn return_type(self) -> &'t Option<ArenaBox<'a, TSTypeAnnotation<'a>>> {
        unsafe {
            &*((self.0 as *const u8).add(OFFSET_ARROW_FUNCTION_EXPRESSION_RETURN_TYPE)
                as *const Option<ArenaBox<'a, TSTypeAnnotation<'a>>>)
        }
    }

    #[inline]
    pub fn scope_id(self) -> &'t Cell<Option<ScopeId>> {
        unsafe {
            &*((self.0 as *const u8).add(OFFSET_ARROW_FUNCTION_EXPRESSION_SCOPE_ID)
                as *const Cell<Option<ScopeId>>)
        }
    }

    #[inline]
    pub fn pure(self) -> &'t bool {
        unsafe {
            &*((self.0 as *const u8).add(OFFSET_ARROW_FUNCTION_EXPRESSION_PURE) as *const bool)
        }
    }

    #[inline]
    pub fn pife(self) -> &'t bool {
        unsafe {
            &*((self.0 as *const u8).add(OFFSET_ARROW_FUNCTION_EXPRESSION_PIFE) as *const bool)
        }
    }
}

impl<'a, 't> GetAddress for ArrowFunctionExpressionWithoutBody<'a, 't> {
    #[inline]
    fn address(&self) -> Address {
        unsafe { Address::from_ptr(self.0) }
    }
}

pub(crate) const OFFSET_YIELD_EXPRESSION_NODE_ID: usize = offset_of!(YieldExpression, node_id);
pub(crate) const OFFSET_YIELD_EXPRESSION_SPAN: usize = offset_of!(YieldExpression, span);
pub(crate) const OFFSET_YIELD_EXPRESSION_DELEGATE: usize = offset_of!(YieldExpression, delegate);
pub(crate) const OFFSET_YIELD_EXPRESSION_ARGUMENT: usize = offset_of!(YieldExpression, argument);

#[repr(transparent)]
#[derive(Clone, Copy, Debug)]
pub struct YieldExpressionWithoutArgument<'a, 't>(
    pub(crate) *const YieldExpression<'a>,
    pub(crate) PhantomData<&'t ()>,
);

impl<'a, 't> YieldExpressionWithoutArgument<'a, 't> {
    #[inline]
    pub fn node_id(self) -> &'t Cell<NodeId> {
        unsafe {
            &*((self.0 as *const u8).add(OFFSET_YIELD_EXPRESSION_NODE_ID) as *const Cell<NodeId>)
        }
    }

    #[inline]
    pub fn span(self) -> &'t Span {
        unsafe { &*((self.0 as *const u8).add(OFFSET_YIELD_EXPRESSION_SPAN) as *const Span) }
    }

    #[inline]
    pub fn delegate(self) -> &'t bool {
        unsafe { &*((self.0 as *const u8).add(OFFSET_YIELD_EXPRESSION_DELEGATE) as *const bool) }
    }
}

impl<'a, 't> GetAddress for YieldExpressionWithoutArgument<'a, 't> {
    #[inline]
    fn address(&self) -> Address {
        unsafe { Address::from_ptr(self.0) }
    }
}

pub(crate) const OFFSET_CLASS_NODE_ID: usize = offset_of!(Class, node_id);
pub(crate) const OFFSET_CLASS_SPAN: usize = offset_of!(Class, span);
pub(crate) const OFFSET_CLASS_TYPE: usize = offset_of!(Class, r#type);
pub(crate) const OFFSET_CLASS_DECORATORS: usize = offset_of!(Class, decorators);
pub(crate) const OFFSET_CLASS_ID: usize = offset_of!(Class, id);
pub(crate) const OFFSET_CLASS_TYPE_PARAMETERS: usize = offset_of!(Class, type_parameters);
pub(crate) const OFFSET_CLASS_SUPER_CLASS: usize = offset_of!(Class, super_class);
pub(crate) const OFFSET_CLASS_SUPER_TYPE_ARGUMENTS: usize = offset_of!(Class, super_type_arguments);
pub(crate) const OFFSET_CLASS_IMPLEMENTS: usize = offset_of!(Class, implements);
pub(crate) const OFFSET_CLASS_BODY: usize = offset_of!(Class, body);
pub(crate) const OFFSET_CLASS_ABSTRACT: usize = offset_of!(Class, r#abstract);
pub(crate) const OFFSET_CLASS_DECLARE: usize = offset_of!(Class, declare);
pub(crate) const OFFSET_CLASS_SCOPE_ID: usize = offset_of!(Class, scope_id);

#[repr(transparent)]
#[derive(Clone, Copy, Debug)]
pub struct ClassWithoutDecorators<'a, 't>(
    pub(crate) *const Class<'a>,
    pub(crate) PhantomData<&'t ()>,
);

impl<'a, 't> ClassWithoutDecorators<'a, 't> {
    #[inline]
    pub fn node_id(self) -> &'t Cell<NodeId> {
        unsafe { &*((self.0 as *const u8).add(OFFSET_CLASS_NODE_ID) as *const Cell<NodeId>) }
    }

    #[inline]
    pub fn span(self) -> &'t Span {
        unsafe { &*((self.0 as *const u8).add(OFFSET_CLASS_SPAN) as *const Span) }
    }

    #[inline]
    pub fn r#type(self) -> &'t ClassType {
        unsafe { &*((self.0 as *const u8).add(OFFSET_CLASS_TYPE) as *const ClassType) }
    }

    #[inline]
    pub fn id(self) -> &'t Option<BindingIdentifier<'a>> {
        unsafe {
            &*((self.0 as *const u8).add(OFFSET_CLASS_ID) as *const Option<BindingIdentifier<'a>>)
        }
    }

    #[inline]
    pub fn type_parameters(self) -> &'t Option<ArenaBox<'a, TSTypeParameterDeclaration<'a>>> {
        unsafe {
            &*((self.0 as *const u8).add(OFFSET_CLASS_TYPE_PARAMETERS)
                as *const Option<ArenaBox<'a, TSTypeParameterDeclaration<'a>>>)
        }
    }

    #[inline]
    pub fn super_class(self) -> &'t Option<Expression<'a>> {
        unsafe {
            &*((self.0 as *const u8).add(OFFSET_CLASS_SUPER_CLASS) as *const Option<Expression<'a>>)
        }
    }

    #[inline]
    pub fn super_type_arguments(
        self,
    ) -> &'t Option<ArenaBox<'a, TSTypeParameterInstantiation<'a>>> {
        unsafe {
            &*((self.0 as *const u8).add(OFFSET_CLASS_SUPER_TYPE_ARGUMENTS)
                as *const Option<ArenaBox<'a, TSTypeParameterInstantiation<'a>>>)
        }
    }

    #[inline]
    pub fn implements(self) -> &'t ArenaVec<'a, TSClassImplements<'a>> {
        unsafe {
            &*((self.0 as *const u8).add(OFFSET_CLASS_IMPLEMENTS)
                as *const ArenaVec<'a, TSClassImplements<'a>>)
        }
    }

    #[inline]
    pub fn body(self) -> &'t ArenaBox<'a, ClassBody<'a>> {
        unsafe {
            &*((self.0 as *const u8).add(OFFSET_CLASS_BODY) as *const ArenaBox<'a, ClassBody<'a>>)
        }
    }

    #[inline]
    pub fn r#abstract(self) -> &'t bool {
        unsafe { &*((self.0 as *const u8).add(OFFSET_CLASS_ABSTRACT) as *const bool) }
    }

    #[inline]
    pub fn declare(self) -> &'t bool {
        unsafe { &*((self.0 as *const u8).add(OFFSET_CLASS_DECLARE) as *const bool) }
    }

    #[inline]
    pub fn scope_id(self) -> &'t Cell<Option<ScopeId>> {
        unsafe {
            &*((self.0 as *const u8).add(OFFSET_CLASS_SCOPE_ID) as *const Cell<Option<ScopeId>>)
        }
    }
}

impl<'a, 't> GetAddress for ClassWithoutDecorators<'a, 't> {
    #[inline]
    fn address(&self) -> Address {
        unsafe { Address::from_ptr(self.0) }
    }
}

#[repr(transparent)]
#[derive(Clone, Copy, Debug)]
pub struct ClassWithoutId<'a, 't>(pub(crate) *const Class<'a>, pub(crate) PhantomData<&'t ()>);

impl<'a, 't> ClassWithoutId<'a, 't> {
    #[inline]
    pub fn node_id(self) -> &'t Cell<NodeId> {
        unsafe { &*((self.0 as *const u8).add(OFFSET_CLASS_NODE_ID) as *const Cell<NodeId>) }
    }

    #[inline]
    pub fn span(self) -> &'t Span {
        unsafe { &*((self.0 as *const u8).add(OFFSET_CLASS_SPAN) as *const Span) }
    }

    #[inline]
    pub fn r#type(self) -> &'t ClassType {
        unsafe { &*((self.0 as *const u8).add(OFFSET_CLASS_TYPE) as *const ClassType) }
    }

    #[inline]
    pub fn decorators(self) -> &'t ArenaVec<'a, Decorator<'a>> {
        unsafe {
            &*((self.0 as *const u8).add(OFFSET_CLASS_DECORATORS)
                as *const ArenaVec<'a, Decorator<'a>>)
        }
    }

    #[inline]
    pub fn type_parameters(self) -> &'t Option<ArenaBox<'a, TSTypeParameterDeclaration<'a>>> {
        unsafe {
            &*((self.0 as *const u8).add(OFFSET_CLASS_TYPE_PARAMETERS)
                as *const Option<ArenaBox<'a, TSTypeParameterDeclaration<'a>>>)
        }
    }

    #[inline]
    pub fn super_class(self) -> &'t Option<Expression<'a>> {
        unsafe {
            &*((self.0 as *const u8).add(OFFSET_CLASS_SUPER_CLASS) as *const Option<Expression<'a>>)
        }
    }

    #[inline]
    pub fn super_type_arguments(
        self,
    ) -> &'t Option<ArenaBox<'a, TSTypeParameterInstantiation<'a>>> {
        unsafe {
            &*((self.0 as *const u8).add(OFFSET_CLASS_SUPER_TYPE_ARGUMENTS)
                as *const Option<ArenaBox<'a, TSTypeParameterInstantiation<'a>>>)
        }
    }

    #[inline]
    pub fn implements(self) -> &'t ArenaVec<'a, TSClassImplements<'a>> {
        unsafe {
            &*((self.0 as *const u8).add(OFFSET_CLASS_IMPLEMENTS)
                as *const ArenaVec<'a, TSClassImplements<'a>>)
        }
    }

    #[inline]
    pub fn body(self) -> &'t ArenaBox<'a, ClassBody<'a>> {
        unsafe {
            &*((self.0 as *const u8).add(OFFSET_CLASS_BODY) as *const ArenaBox<'a, ClassBody<'a>>)
        }
    }

    #[inline]
    pub fn r#abstract(self) -> &'t bool {
        unsafe { &*((self.0 as *const u8).add(OFFSET_CLASS_ABSTRACT) as *const bool) }
    }

    #[inline]
    pub fn declare(self) -> &'t bool {
        unsafe { &*((self.0 as *const u8).add(OFFSET_CLASS_DECLARE) as *const bool) }
    }

    #[inline]
    pub fn scope_id(self) -> &'t Cell<Option<ScopeId>> {
        unsafe {
            &*((self.0 as *const u8).add(OFFSET_CLASS_SCOPE_ID) as *const Cell<Option<ScopeId>>)
        }
    }
}

impl<'a, 't> GetAddress for ClassWithoutId<'a, 't> {
    #[inline]
    fn address(&self) -> Address {
        unsafe { Address::from_ptr(self.0) }
    }
}

#[repr(transparent)]
#[derive(Clone, Copy, Debug)]
pub struct ClassWithoutSuperClass<'a, 't>(
    pub(crate) *const Class<'a>,
    pub(crate) PhantomData<&'t ()>,
);

impl<'a, 't> ClassWithoutSuperClass<'a, 't> {
    #[inline]
    pub fn node_id(self) -> &'t Cell<NodeId> {
        unsafe { &*((self.0 as *const u8).add(OFFSET_CLASS_NODE_ID) as *const Cell<NodeId>) }
    }

    #[inline]
    pub fn span(self) -> &'t Span {
        unsafe { &*((self.0 as *const u8).add(OFFSET_CLASS_SPAN) as *const Span) }
    }

    #[inline]
    pub fn r#type(self) -> &'t ClassType {
        unsafe { &*((self.0 as *const u8).add(OFFSET_CLASS_TYPE) as *const ClassType) }
    }

    #[inline]
    pub fn decorators(self) -> &'t ArenaVec<'a, Decorator<'a>> {
        unsafe {
            &*((self.0 as *const u8).add(OFFSET_CLASS_DECORATORS)
                as *const ArenaVec<'a, Decorator<'a>>)
        }
    }

    #[inline]
    pub fn id(self) -> &'t Option<BindingIdentifier<'a>> {
        unsafe {
            &*((self.0 as *const u8).add(OFFSET_CLASS_ID) as *const Option<BindingIdentifier<'a>>)
        }
    }

    #[inline]
    pub fn type_parameters(self) -> &'t Option<ArenaBox<'a, TSTypeParameterDeclaration<'a>>> {
        unsafe {
            &*((self.0 as *const u8).add(OFFSET_CLASS_TYPE_PARAMETERS)
                as *const Option<ArenaBox<'a, TSTypeParameterDeclaration<'a>>>)
        }
    }

    #[inline]
    pub fn super_type_arguments(
        self,
    ) -> &'t Option<ArenaBox<'a, TSTypeParameterInstantiation<'a>>> {
        unsafe {
            &*((self.0 as *const u8).add(OFFSET_CLASS_SUPER_TYPE_ARGUMENTS)
                as *const Option<ArenaBox<'a, TSTypeParameterInstantiation<'a>>>)
        }
    }

    #[inline]
    pub fn implements(self) -> &'t ArenaVec<'a, TSClassImplements<'a>> {
        unsafe {
            &*((self.0 as *const u8).add(OFFSET_CLASS_IMPLEMENTS)
                as *const ArenaVec<'a, TSClassImplements<'a>>)
        }
    }

    #[inline]
    pub fn body(self) -> &'t ArenaBox<'a, ClassBody<'a>> {
        unsafe {
            &*((self.0 as *const u8).add(OFFSET_CLASS_BODY) as *const ArenaBox<'a, ClassBody<'a>>)
        }
    }

    #[inline]
    pub fn r#abstract(self) -> &'t bool {
        unsafe { &*((self.0 as *const u8).add(OFFSET_CLASS_ABSTRACT) as *const bool) }
    }

    #[inline]
    pub fn declare(self) -> &'t bool {
        unsafe { &*((self.0 as *const u8).add(OFFSET_CLASS_DECLARE) as *const bool) }
    }

    #[inline]
    pub fn scope_id(self) -> &'t Cell<Option<ScopeId>> {
        unsafe {
            &*((self.0 as *const u8).add(OFFSET_CLASS_SCOPE_ID) as *const Cell<Option<ScopeId>>)
        }
    }
}

impl<'a, 't> GetAddress for ClassWithoutSuperClass<'a, 't> {
    #[inline]
    fn address(&self) -> Address {
        unsafe { Address::from_ptr(self.0) }
    }
}

#[repr(transparent)]
#[derive(Clone, Copy, Debug)]
pub struct ClassWithoutBody<'a, 't>(pub(crate) *const Class<'a>, pub(crate) PhantomData<&'t ()>);

impl<'a, 't> ClassWithoutBody<'a, 't> {
    #[inline]
    pub fn node_id(self) -> &'t Cell<NodeId> {
        unsafe { &*((self.0 as *const u8).add(OFFSET_CLASS_NODE_ID) as *const Cell<NodeId>) }
    }

    #[inline]
    pub fn span(self) -> &'t Span {
        unsafe { &*((self.0 as *const u8).add(OFFSET_CLASS_SPAN) as *const Span) }
    }

    #[inline]
    pub fn r#type(self) -> &'t ClassType {
        unsafe { &*((self.0 as *const u8).add(OFFSET_CLASS_TYPE) as *const ClassType) }
    }

    #[inline]
    pub fn decorators(self) -> &'t ArenaVec<'a, Decorator<'a>> {
        unsafe {
            &*((self.0 as *const u8).add(OFFSET_CLASS_DECORATORS)
                as *const ArenaVec<'a, Decorator<'a>>)
        }
    }

    #[inline]
    pub fn id(self) -> &'t Option<BindingIdentifier<'a>> {
        unsafe {
            &*((self.0 as *const u8).add(OFFSET_CLASS_ID) as *const Option<BindingIdentifier<'a>>)
        }
    }

    #[inline]
    pub fn type_parameters(self) -> &'t Option<ArenaBox<'a, TSTypeParameterDeclaration<'a>>> {
        unsafe {
            &*((self.0 as *const u8).add(OFFSET_CLASS_TYPE_PARAMETERS)
                as *const Option<ArenaBox<'a, TSTypeParameterDeclaration<'a>>>)
        }
    }

    #[inline]
    pub fn super_class(self) -> &'t Option<Expression<'a>> {
        unsafe {
            &*((self.0 as *const u8).add(OFFSET_CLASS_SUPER_CLASS) as *const Option<Expression<'a>>)
        }
    }

    #[inline]
    pub fn super_type_arguments(
        self,
    ) -> &'t Option<ArenaBox<'a, TSTypeParameterInstantiation<'a>>> {
        unsafe {
            &*((self.0 as *const u8).add(OFFSET_CLASS_SUPER_TYPE_ARGUMENTS)
                as *const Option<ArenaBox<'a, TSTypeParameterInstantiation<'a>>>)
        }
    }

    #[inline]
    pub fn implements(self) -> &'t ArenaVec<'a, TSClassImplements<'a>> {
        unsafe {
            &*((self.0 as *const u8).add(OFFSET_CLASS_IMPLEMENTS)
                as *const ArenaVec<'a, TSClassImplements<'a>>)
        }
    }

    #[inline]
    pub fn r#abstract(self) -> &'t bool {
        unsafe { &*((self.0 as *const u8).add(OFFSET_CLASS_ABSTRACT) as *const bool) }
    }

    #[inline]
    pub fn declare(self) -> &'t bool {
        unsafe { &*((self.0 as *const u8).add(OFFSET_CLASS_DECLARE) as *const bool) }
    }

    #[inline]
    pub fn scope_id(self) -> &'t Cell<Option<ScopeId>> {
        unsafe {
            &*((self.0 as *const u8).add(OFFSET_CLASS_SCOPE_ID) as *const Cell<Option<ScopeId>>)
        }
    }
}

impl<'a, 't> GetAddress for ClassWithoutBody<'a, 't> {
    #[inline]
    fn address(&self) -> Address {
        unsafe { Address::from_ptr(self.0) }
    }
}

pub(crate) const OFFSET_CLASS_BODY_NODE_ID: usize = offset_of!(ClassBody, node_id);
pub(crate) const OFFSET_CLASS_BODY_SPAN: usize = offset_of!(ClassBody, span);
pub(crate) const OFFSET_CLASS_BODY_BODY: usize = offset_of!(ClassBody, body);

#[repr(transparent)]
#[derive(Clone, Copy, Debug)]
pub struct ClassBodyWithoutBody<'a, 't>(
    pub(crate) *const ClassBody<'a>,
    pub(crate) PhantomData<&'t ()>,
);

impl<'a, 't> ClassBodyWithoutBody<'a, 't> {
    #[inline]
    pub fn node_id(self) -> &'t Cell<NodeId> {
        unsafe { &*((self.0 as *const u8).add(OFFSET_CLASS_BODY_NODE_ID) as *const Cell<NodeId>) }
    }

    #[inline]
    pub fn span(self) -> &'t Span {
        unsafe { &*((self.0 as *const u8).add(OFFSET_CLASS_BODY_SPAN) as *const Span) }
    }
}

impl<'a, 't> GetAddress for ClassBodyWithoutBody<'a, 't> {
    #[inline]
    fn address(&self) -> Address {
        unsafe { Address::from_ptr(self.0) }
    }
}

pub(crate) const OFFSET_METHOD_DEFINITION_NODE_ID: usize = offset_of!(MethodDefinition, node_id);
pub(crate) const OFFSET_METHOD_DEFINITION_SPAN: usize = offset_of!(MethodDefinition, span);
pub(crate) const OFFSET_METHOD_DEFINITION_TYPE: usize = offset_of!(MethodDefinition, r#type);
pub(crate) const OFFSET_METHOD_DEFINITION_DECORATORS: usize =
    offset_of!(MethodDefinition, decorators);
pub(crate) const OFFSET_METHOD_DEFINITION_KEY: usize = offset_of!(MethodDefinition, key);
pub(crate) const OFFSET_METHOD_DEFINITION_VALUE: usize = offset_of!(MethodDefinition, value);
pub(crate) const OFFSET_METHOD_DEFINITION_KIND: usize = offset_of!(MethodDefinition, kind);
pub(crate) const OFFSET_METHOD_DEFINITION_COMPUTED: usize = offset_of!(MethodDefinition, computed);
pub(crate) const OFFSET_METHOD_DEFINITION_STATIC: usize = offset_of!(MethodDefinition, r#static);
pub(crate) const OFFSET_METHOD_DEFINITION_OVERRIDE: usize =
    offset_of!(MethodDefinition, r#override);
pub(crate) const OFFSET_METHOD_DEFINITION_OPTIONAL: usize = offset_of!(MethodDefinition, optional);
pub(crate) const OFFSET_METHOD_DEFINITION_ACCESSIBILITY: usize =
    offset_of!(MethodDefinition, accessibility);

#[repr(transparent)]
#[derive(Clone, Copy, Debug)]
pub struct MethodDefinitionWithoutDecorators<'a, 't>(
    pub(crate) *const MethodDefinition<'a>,
    pub(crate) PhantomData<&'t ()>,
);

impl<'a, 't> MethodDefinitionWithoutDecorators<'a, 't> {
    #[inline]
    pub fn node_id(self) -> &'t Cell<NodeId> {
        unsafe {
            &*((self.0 as *const u8).add(OFFSET_METHOD_DEFINITION_NODE_ID) as *const Cell<NodeId>)
        }
    }

    #[inline]
    pub fn span(self) -> &'t Span {
        unsafe { &*((self.0 as *const u8).add(OFFSET_METHOD_DEFINITION_SPAN) as *const Span) }
    }

    #[inline]
    pub fn r#type(self) -> &'t MethodDefinitionType {
        unsafe {
            &*((self.0 as *const u8).add(OFFSET_METHOD_DEFINITION_TYPE)
                as *const MethodDefinitionType)
        }
    }

    #[inline]
    pub fn key(self) -> &'t PropertyKey<'a> {
        unsafe {
            &*((self.0 as *const u8).add(OFFSET_METHOD_DEFINITION_KEY) as *const PropertyKey<'a>)
        }
    }

    #[inline]
    pub fn value(self) -> &'t ArenaBox<'a, Function<'a>> {
        unsafe {
            &*((self.0 as *const u8).add(OFFSET_METHOD_DEFINITION_VALUE)
                as *const ArenaBox<'a, Function<'a>>)
        }
    }

    #[inline]
    pub fn kind(self) -> &'t MethodDefinitionKind {
        unsafe {
            &*((self.0 as *const u8).add(OFFSET_METHOD_DEFINITION_KIND)
                as *const MethodDefinitionKind)
        }
    }

    #[inline]
    pub fn computed(self) -> &'t bool {
        unsafe { &*((self.0 as *const u8).add(OFFSET_METHOD_DEFINITION_COMPUTED) as *const bool) }
    }

    #[inline]
    pub fn r#static(self) -> &'t bool {
        unsafe { &*((self.0 as *const u8).add(OFFSET_METHOD_DEFINITION_STATIC) as *const bool) }
    }

    #[inline]
    pub fn r#override(self) -> &'t bool {
        unsafe { &*((self.0 as *const u8).add(OFFSET_METHOD_DEFINITION_OVERRIDE) as *const bool) }
    }

    #[inline]
    pub fn optional(self) -> &'t bool {
        unsafe { &*((self.0 as *const u8).add(OFFSET_METHOD_DEFINITION_OPTIONAL) as *const bool) }
    }

    #[inline]
    pub fn accessibility(self) -> &'t Option<TSAccessibility> {
        unsafe {
            &*((self.0 as *const u8).add(OFFSET_METHOD_DEFINITION_ACCESSIBILITY)
                as *const Option<TSAccessibility>)
        }
    }
}

impl<'a, 't> GetAddress for MethodDefinitionWithoutDecorators<'a, 't> {
    #[inline]
    fn address(&self) -> Address {
        unsafe { Address::from_ptr(self.0) }
    }
}

#[repr(transparent)]
#[derive(Clone, Copy, Debug)]
pub struct MethodDefinitionWithoutKey<'a, 't>(
    pub(crate) *const MethodDefinition<'a>,
    pub(crate) PhantomData<&'t ()>,
);

impl<'a, 't> MethodDefinitionWithoutKey<'a, 't> {
    #[inline]
    pub fn node_id(self) -> &'t Cell<NodeId> {
        unsafe {
            &*((self.0 as *const u8).add(OFFSET_METHOD_DEFINITION_NODE_ID) as *const Cell<NodeId>)
        }
    }

    #[inline]
    pub fn span(self) -> &'t Span {
        unsafe { &*((self.0 as *const u8).add(OFFSET_METHOD_DEFINITION_SPAN) as *const Span) }
    }

    #[inline]
    pub fn r#type(self) -> &'t MethodDefinitionType {
        unsafe {
            &*((self.0 as *const u8).add(OFFSET_METHOD_DEFINITION_TYPE)
                as *const MethodDefinitionType)
        }
    }

    #[inline]
    pub fn decorators(self) -> &'t ArenaVec<'a, Decorator<'a>> {
        unsafe {
            &*((self.0 as *const u8).add(OFFSET_METHOD_DEFINITION_DECORATORS)
                as *const ArenaVec<'a, Decorator<'a>>)
        }
    }

    #[inline]
    pub fn value(self) -> &'t ArenaBox<'a, Function<'a>> {
        unsafe {
            &*((self.0 as *const u8).add(OFFSET_METHOD_DEFINITION_VALUE)
                as *const ArenaBox<'a, Function<'a>>)
        }
    }

    #[inline]
    pub fn kind(self) -> &'t MethodDefinitionKind {
        unsafe {
            &*((self.0 as *const u8).add(OFFSET_METHOD_DEFINITION_KIND)
                as *const MethodDefinitionKind)
        }
    }

    #[inline]
    pub fn computed(self) -> &'t bool {
        unsafe { &*((self.0 as *const u8).add(OFFSET_METHOD_DEFINITION_COMPUTED) as *const bool) }
    }

    #[inline]
    pub fn r#static(self) -> &'t bool {
        unsafe { &*((self.0 as *const u8).add(OFFSET_METHOD_DEFINITION_STATIC) as *const bool) }
    }

    #[inline]
    pub fn r#override(self) -> &'t bool {
        unsafe { &*((self.0 as *const u8).add(OFFSET_METHOD_DEFINITION_OVERRIDE) as *const bool) }
    }

    #[inline]
    pub fn optional(self) -> &'t bool {
        unsafe { &*((self.0 as *const u8).add(OFFSET_METHOD_DEFINITION_OPTIONAL) as *const bool) }
    }

    #[inline]
    pub fn accessibility(self) -> &'t Option<TSAccessibility> {
        unsafe {
            &*((self.0 as *const u8).add(OFFSET_METHOD_DEFINITION_ACCESSIBILITY)
                as *const Option<TSAccessibility>)
        }
    }
}

impl<'a, 't> GetAddress for MethodDefinitionWithoutKey<'a, 't> {
    #[inline]
    fn address(&self) -> Address {
        unsafe { Address::from_ptr(self.0) }
    }
}

#[repr(transparent)]
#[derive(Clone, Copy, Debug)]
pub struct MethodDefinitionWithoutValue<'a, 't>(
    pub(crate) *const MethodDefinition<'a>,
    pub(crate) PhantomData<&'t ()>,
);

impl<'a, 't> MethodDefinitionWithoutValue<'a, 't> {
    #[inline]
    pub fn node_id(self) -> &'t Cell<NodeId> {
        unsafe {
            &*((self.0 as *const u8).add(OFFSET_METHOD_DEFINITION_NODE_ID) as *const Cell<NodeId>)
        }
    }

    #[inline]
    pub fn span(self) -> &'t Span {
        unsafe { &*((self.0 as *const u8).add(OFFSET_METHOD_DEFINITION_SPAN) as *const Span) }
    }

    #[inline]
    pub fn r#type(self) -> &'t MethodDefinitionType {
        unsafe {
            &*((self.0 as *const u8).add(OFFSET_METHOD_DEFINITION_TYPE)
                as *const MethodDefinitionType)
        }
    }

    #[inline]
    pub fn decorators(self) -> &'t ArenaVec<'a, Decorator<'a>> {
        unsafe {
            &*((self.0 as *const u8).add(OFFSET_METHOD_DEFINITION_DECORATORS)
                as *const ArenaVec<'a, Decorator<'a>>)
        }
    }

    #[inline]
    pub fn key(self) -> &'t PropertyKey<'a> {
        unsafe {
            &*((self.0 as *const u8).add(OFFSET_METHOD_DEFINITION_KEY) as *const PropertyKey<'a>)
        }
    }

    #[inline]
    pub fn kind(self) -> &'t MethodDefinitionKind {
        unsafe {
            &*((self.0 as *const u8).add(OFFSET_METHOD_DEFINITION_KIND)
                as *const MethodDefinitionKind)
        }
    }

    #[inline]
    pub fn computed(self) -> &'t bool {
        unsafe { &*((self.0 as *const u8).add(OFFSET_METHOD_DEFINITION_COMPUTED) as *const bool) }
    }

    #[inline]
    pub fn r#static(self) -> &'t bool {
        unsafe { &*((self.0 as *const u8).add(OFFSET_METHOD_DEFINITION_STATIC) as *const bool) }
    }

    #[inline]
    pub fn r#override(self) -> &'t bool {
        unsafe { &*((self.0 as *const u8).add(OFFSET_METHOD_DEFINITION_OVERRIDE) as *const bool) }
    }

    #[inline]
    pub fn optional(self) -> &'t bool {
        unsafe { &*((self.0 as *const u8).add(OFFSET_METHOD_DEFINITION_OPTIONAL) as *const bool) }
    }

    #[inline]
    pub fn accessibility(self) -> &'t Option<TSAccessibility> {
        unsafe {
            &*((self.0 as *const u8).add(OFFSET_METHOD_DEFINITION_ACCESSIBILITY)
                as *const Option<TSAccessibility>)
        }
    }
}

impl<'a, 't> GetAddress for MethodDefinitionWithoutValue<'a, 't> {
    #[inline]
    fn address(&self) -> Address {
        unsafe { Address::from_ptr(self.0) }
    }
}

pub(crate) const OFFSET_PROPERTY_DEFINITION_NODE_ID: usize =
    offset_of!(PropertyDefinition, node_id);
pub(crate) const OFFSET_PROPERTY_DEFINITION_SPAN: usize = offset_of!(PropertyDefinition, span);
pub(crate) const OFFSET_PROPERTY_DEFINITION_TYPE: usize = offset_of!(PropertyDefinition, r#type);
pub(crate) const OFFSET_PROPERTY_DEFINITION_DECORATORS: usize =
    offset_of!(PropertyDefinition, decorators);
pub(crate) const OFFSET_PROPERTY_DEFINITION_KEY: usize = offset_of!(PropertyDefinition, key);
pub(crate) const OFFSET_PROPERTY_DEFINITION_TYPE_ANNOTATION: usize =
    offset_of!(PropertyDefinition, type_annotation);
pub(crate) const OFFSET_PROPERTY_DEFINITION_VALUE: usize = offset_of!(PropertyDefinition, value);
pub(crate) const OFFSET_PROPERTY_DEFINITION_COMPUTED: usize =
    offset_of!(PropertyDefinition, computed);
pub(crate) const OFFSET_PROPERTY_DEFINITION_STATIC: usize =
    offset_of!(PropertyDefinition, r#static);
pub(crate) const OFFSET_PROPERTY_DEFINITION_DECLARE: usize =
    offset_of!(PropertyDefinition, declare);
pub(crate) const OFFSET_PROPERTY_DEFINITION_OVERRIDE: usize =
    offset_of!(PropertyDefinition, r#override);
pub(crate) const OFFSET_PROPERTY_DEFINITION_OPTIONAL: usize =
    offset_of!(PropertyDefinition, optional);
pub(crate) const OFFSET_PROPERTY_DEFINITION_DEFINITE: usize =
    offset_of!(PropertyDefinition, definite);
pub(crate) const OFFSET_PROPERTY_DEFINITION_READONLY: usize =
    offset_of!(PropertyDefinition, readonly);
pub(crate) const OFFSET_PROPERTY_DEFINITION_ACCESSIBILITY: usize =
    offset_of!(PropertyDefinition, accessibility);

#[repr(transparent)]
#[derive(Clone, Copy, Debug)]
pub struct PropertyDefinitionWithoutDecorators<'a, 't>(
    pub(crate) *const PropertyDefinition<'a>,
    pub(crate) PhantomData<&'t ()>,
);

impl<'a, 't> PropertyDefinitionWithoutDecorators<'a, 't> {
    #[inline]
    pub fn node_id(self) -> &'t Cell<NodeId> {
        unsafe {
            &*((self.0 as *const u8).add(OFFSET_PROPERTY_DEFINITION_NODE_ID) as *const Cell<NodeId>)
        }
    }

    #[inline]
    pub fn span(self) -> &'t Span {
        unsafe { &*((self.0 as *const u8).add(OFFSET_PROPERTY_DEFINITION_SPAN) as *const Span) }
    }

    #[inline]
    pub fn r#type(self) -> &'t PropertyDefinitionType {
        unsafe {
            &*((self.0 as *const u8).add(OFFSET_PROPERTY_DEFINITION_TYPE)
                as *const PropertyDefinitionType)
        }
    }

    #[inline]
    pub fn key(self) -> &'t PropertyKey<'a> {
        unsafe {
            &*((self.0 as *const u8).add(OFFSET_PROPERTY_DEFINITION_KEY) as *const PropertyKey<'a>)
        }
    }

    #[inline]
    pub fn type_annotation(self) -> &'t Option<ArenaBox<'a, TSTypeAnnotation<'a>>> {
        unsafe {
            &*((self.0 as *const u8).add(OFFSET_PROPERTY_DEFINITION_TYPE_ANNOTATION)
                as *const Option<ArenaBox<'a, TSTypeAnnotation<'a>>>)
        }
    }

    #[inline]
    pub fn value(self) -> &'t Option<Expression<'a>> {
        unsafe {
            &*((self.0 as *const u8).add(OFFSET_PROPERTY_DEFINITION_VALUE)
                as *const Option<Expression<'a>>)
        }
    }

    #[inline]
    pub fn computed(self) -> &'t bool {
        unsafe { &*((self.0 as *const u8).add(OFFSET_PROPERTY_DEFINITION_COMPUTED) as *const bool) }
    }

    #[inline]
    pub fn r#static(self) -> &'t bool {
        unsafe { &*((self.0 as *const u8).add(OFFSET_PROPERTY_DEFINITION_STATIC) as *const bool) }
    }

    #[inline]
    pub fn declare(self) -> &'t bool {
        unsafe { &*((self.0 as *const u8).add(OFFSET_PROPERTY_DEFINITION_DECLARE) as *const bool) }
    }

    #[inline]
    pub fn r#override(self) -> &'t bool {
        unsafe { &*((self.0 as *const u8).add(OFFSET_PROPERTY_DEFINITION_OVERRIDE) as *const bool) }
    }

    #[inline]
    pub fn optional(self) -> &'t bool {
        unsafe { &*((self.0 as *const u8).add(OFFSET_PROPERTY_DEFINITION_OPTIONAL) as *const bool) }
    }

    #[inline]
    pub fn definite(self) -> &'t bool {
        unsafe { &*((self.0 as *const u8).add(OFFSET_PROPERTY_DEFINITION_DEFINITE) as *const bool) }
    }

    #[inline]
    pub fn readonly(self) -> &'t bool {
        unsafe { &*((self.0 as *const u8).add(OFFSET_PROPERTY_DEFINITION_READONLY) as *const bool) }
    }

    #[inline]
    pub fn accessibility(self) -> &'t Option<TSAccessibility> {
        unsafe {
            &*((self.0 as *const u8).add(OFFSET_PROPERTY_DEFINITION_ACCESSIBILITY)
                as *const Option<TSAccessibility>)
        }
    }
}

impl<'a, 't> GetAddress for PropertyDefinitionWithoutDecorators<'a, 't> {
    #[inline]
    fn address(&self) -> Address {
        unsafe { Address::from_ptr(self.0) }
    }
}

#[repr(transparent)]
#[derive(Clone, Copy, Debug)]
pub struct PropertyDefinitionWithoutKey<'a, 't>(
    pub(crate) *const PropertyDefinition<'a>,
    pub(crate) PhantomData<&'t ()>,
);

impl<'a, 't> PropertyDefinitionWithoutKey<'a, 't> {
    #[inline]
    pub fn node_id(self) -> &'t Cell<NodeId> {
        unsafe {
            &*((self.0 as *const u8).add(OFFSET_PROPERTY_DEFINITION_NODE_ID) as *const Cell<NodeId>)
        }
    }

    #[inline]
    pub fn span(self) -> &'t Span {
        unsafe { &*((self.0 as *const u8).add(OFFSET_PROPERTY_DEFINITION_SPAN) as *const Span) }
    }

    #[inline]
    pub fn r#type(self) -> &'t PropertyDefinitionType {
        unsafe {
            &*((self.0 as *const u8).add(OFFSET_PROPERTY_DEFINITION_TYPE)
                as *const PropertyDefinitionType)
        }
    }

    #[inline]
    pub fn decorators(self) -> &'t ArenaVec<'a, Decorator<'a>> {
        unsafe {
            &*((self.0 as *const u8).add(OFFSET_PROPERTY_DEFINITION_DECORATORS)
                as *const ArenaVec<'a, Decorator<'a>>)
        }
    }

    #[inline]
    pub fn type_annotation(self) -> &'t Option<ArenaBox<'a, TSTypeAnnotation<'a>>> {
        unsafe {
            &*((self.0 as *const u8).add(OFFSET_PROPERTY_DEFINITION_TYPE_ANNOTATION)
                as *const Option<ArenaBox<'a, TSTypeAnnotation<'a>>>)
        }
    }

    #[inline]
    pub fn value(self) -> &'t Option<Expression<'a>> {
        unsafe {
            &*((self.0 as *const u8).add(OFFSET_PROPERTY_DEFINITION_VALUE)
                as *const Option<Expression<'a>>)
        }
    }

    #[inline]
    pub fn computed(self) -> &'t bool {
        unsafe { &*((self.0 as *const u8).add(OFFSET_PROPERTY_DEFINITION_COMPUTED) as *const bool) }
    }

    #[inline]
    pub fn r#static(self) -> &'t bool {
        unsafe { &*((self.0 as *const u8).add(OFFSET_PROPERTY_DEFINITION_STATIC) as *const bool) }
    }

    #[inline]
    pub fn declare(self) -> &'t bool {
        unsafe { &*((self.0 as *const u8).add(OFFSET_PROPERTY_DEFINITION_DECLARE) as *const bool) }
    }

    #[inline]
    pub fn r#override(self) -> &'t bool {
        unsafe { &*((self.0 as *const u8).add(OFFSET_PROPERTY_DEFINITION_OVERRIDE) as *const bool) }
    }

    #[inline]
    pub fn optional(self) -> &'t bool {
        unsafe { &*((self.0 as *const u8).add(OFFSET_PROPERTY_DEFINITION_OPTIONAL) as *const bool) }
    }

    #[inline]
    pub fn definite(self) -> &'t bool {
        unsafe { &*((self.0 as *const u8).add(OFFSET_PROPERTY_DEFINITION_DEFINITE) as *const bool) }
    }

    #[inline]
    pub fn readonly(self) -> &'t bool {
        unsafe { &*((self.0 as *const u8).add(OFFSET_PROPERTY_DEFINITION_READONLY) as *const bool) }
    }

    #[inline]
    pub fn accessibility(self) -> &'t Option<TSAccessibility> {
        unsafe {
            &*((self.0 as *const u8).add(OFFSET_PROPERTY_DEFINITION_ACCESSIBILITY)
                as *const Option<TSAccessibility>)
        }
    }
}

impl<'a, 't> GetAddress for PropertyDefinitionWithoutKey<'a, 't> {
    #[inline]
    fn address(&self) -> Address {
        unsafe { Address::from_ptr(self.0) }
    }
}

#[repr(transparent)]
#[derive(Clone, Copy, Debug)]
pub struct PropertyDefinitionWithoutValue<'a, 't>(
    pub(crate) *const PropertyDefinition<'a>,
    pub(crate) PhantomData<&'t ()>,
);

impl<'a, 't> PropertyDefinitionWithoutValue<'a, 't> {
    #[inline]
    pub fn node_id(self) -> &'t Cell<NodeId> {
        unsafe {
            &*((self.0 as *const u8).add(OFFSET_PROPERTY_DEFINITION_NODE_ID) as *const Cell<NodeId>)
        }
    }

    #[inline]
    pub fn span(self) -> &'t Span {
        unsafe { &*((self.0 as *const u8).add(OFFSET_PROPERTY_DEFINITION_SPAN) as *const Span) }
    }

    #[inline]
    pub fn r#type(self) -> &'t PropertyDefinitionType {
        unsafe {
            &*((self.0 as *const u8).add(OFFSET_PROPERTY_DEFINITION_TYPE)
                as *const PropertyDefinitionType)
        }
    }

    #[inline]
    pub fn decorators(self) -> &'t ArenaVec<'a, Decorator<'a>> {
        unsafe {
            &*((self.0 as *const u8).add(OFFSET_PROPERTY_DEFINITION_DECORATORS)
                as *const ArenaVec<'a, Decorator<'a>>)
        }
    }

    #[inline]
    pub fn key(self) -> &'t PropertyKey<'a> {
        unsafe {
            &*((self.0 as *const u8).add(OFFSET_PROPERTY_DEFINITION_KEY) as *const PropertyKey<'a>)
        }
    }

    #[inline]
    pub fn type_annotation(self) -> &'t Option<ArenaBox<'a, TSTypeAnnotation<'a>>> {
        unsafe {
            &*((self.0 as *const u8).add(OFFSET_PROPERTY_DEFINITION_TYPE_ANNOTATION)
                as *const Option<ArenaBox<'a, TSTypeAnnotation<'a>>>)
        }
    }

    #[inline]
    pub fn computed(self) -> &'t bool {
        unsafe { &*((self.0 as *const u8).add(OFFSET_PROPERTY_DEFINITION_COMPUTED) as *const bool) }
    }

    #[inline]
    pub fn r#static(self) -> &'t bool {
        unsafe { &*((self.0 as *const u8).add(OFFSET_PROPERTY_DEFINITION_STATIC) as *const bool) }
    }

    #[inline]
    pub fn declare(self) -> &'t bool {
        unsafe { &*((self.0 as *const u8).add(OFFSET_PROPERTY_DEFINITION_DECLARE) as *const bool) }
    }

    #[inline]
    pub fn r#override(self) -> &'t bool {
        unsafe { &*((self.0 as *const u8).add(OFFSET_PROPERTY_DEFINITION_OVERRIDE) as *const bool) }
    }

    #[inline]
    pub fn optional(self) -> &'t bool {
        unsafe { &*((self.0 as *const u8).add(OFFSET_PROPERTY_DEFINITION_OPTIONAL) as *const bool) }
    }

    #[inline]
    pub fn definite(self) -> &'t bool {
        unsafe { &*((self.0 as *const u8).add(OFFSET_PROPERTY_DEFINITION_DEFINITE) as *const bool) }
    }

    #[inline]
    pub fn readonly(self) -> &'t bool {
        unsafe { &*((self.0 as *const u8).add(OFFSET_PROPERTY_DEFINITION_READONLY) as *const bool) }
    }

    #[inline]
    pub fn accessibility(self) -> &'t Option<TSAccessibility> {
        unsafe {
            &*((self.0 as *const u8).add(OFFSET_PROPERTY_DEFINITION_ACCESSIBILITY)
                as *const Option<TSAccessibility>)
        }
    }
}

impl<'a, 't> GetAddress for PropertyDefinitionWithoutValue<'a, 't> {
    #[inline]
    fn address(&self) -> Address {
        unsafe { Address::from_ptr(self.0) }
    }
}

pub(crate) const OFFSET_STATIC_BLOCK_NODE_ID: usize = offset_of!(StaticBlock, node_id);
pub(crate) const OFFSET_STATIC_BLOCK_SPAN: usize = offset_of!(StaticBlock, span);
pub(crate) const OFFSET_STATIC_BLOCK_BODY: usize = offset_of!(StaticBlock, body);
pub(crate) const OFFSET_STATIC_BLOCK_SCOPE_ID: usize = offset_of!(StaticBlock, scope_id);

#[repr(transparent)]
#[derive(Clone, Copy, Debug)]
pub struct StaticBlockWithoutBody<'a, 't>(
    pub(crate) *const StaticBlock<'a>,
    pub(crate) PhantomData<&'t ()>,
);

impl<'a, 't> StaticBlockWithoutBody<'a, 't> {
    #[inline]
    pub fn node_id(self) -> &'t Cell<NodeId> {
        unsafe { &*((self.0 as *const u8).add(OFFSET_STATIC_BLOCK_NODE_ID) as *const Cell<NodeId>) }
    }

    #[inline]
    pub fn span(self) -> &'t Span {
        unsafe { &*((self.0 as *const u8).add(OFFSET_STATIC_BLOCK_SPAN) as *const Span) }
    }

    #[inline]
    pub fn scope_id(self) -> &'t Cell<Option<ScopeId>> {
        unsafe {
            &*((self.0 as *const u8).add(OFFSET_STATIC_BLOCK_SCOPE_ID)
                as *const Cell<Option<ScopeId>>)
        }
    }
}

impl<'a, 't> GetAddress for StaticBlockWithoutBody<'a, 't> {
    #[inline]
    fn address(&self) -> Address {
        unsafe { Address::from_ptr(self.0) }
    }
}

pub(crate) const OFFSET_ACCESSOR_PROPERTY_NODE_ID: usize = offset_of!(AccessorProperty, node_id);
pub(crate) const OFFSET_ACCESSOR_PROPERTY_SPAN: usize = offset_of!(AccessorProperty, span);
pub(crate) const OFFSET_ACCESSOR_PROPERTY_TYPE: usize = offset_of!(AccessorProperty, r#type);
pub(crate) const OFFSET_ACCESSOR_PROPERTY_DECORATORS: usize =
    offset_of!(AccessorProperty, decorators);
pub(crate) const OFFSET_ACCESSOR_PROPERTY_KEY: usize = offset_of!(AccessorProperty, key);
pub(crate) const OFFSET_ACCESSOR_PROPERTY_TYPE_ANNOTATION: usize =
    offset_of!(AccessorProperty, type_annotation);
pub(crate) const OFFSET_ACCESSOR_PROPERTY_VALUE: usize = offset_of!(AccessorProperty, value);
pub(crate) const OFFSET_ACCESSOR_PROPERTY_COMPUTED: usize = offset_of!(AccessorProperty, computed);
pub(crate) const OFFSET_ACCESSOR_PROPERTY_STATIC: usize = offset_of!(AccessorProperty, r#static);
pub(crate) const OFFSET_ACCESSOR_PROPERTY_OVERRIDE: usize =
    offset_of!(AccessorProperty, r#override);
pub(crate) const OFFSET_ACCESSOR_PROPERTY_DEFINITE: usize = offset_of!(AccessorProperty, definite);
pub(crate) const OFFSET_ACCESSOR_PROPERTY_ACCESSIBILITY: usize =
    offset_of!(AccessorProperty, accessibility);

#[repr(transparent)]
#[derive(Clone, Copy, Debug)]
pub struct AccessorPropertyWithoutDecorators<'a, 't>(
    pub(crate) *const AccessorProperty<'a>,
    pub(crate) PhantomData<&'t ()>,
);

impl<'a, 't> AccessorPropertyWithoutDecorators<'a, 't> {
    #[inline]
    pub fn node_id(self) -> &'t Cell<NodeId> {
        unsafe {
            &*((self.0 as *const u8).add(OFFSET_ACCESSOR_PROPERTY_NODE_ID) as *const Cell<NodeId>)
        }
    }

    #[inline]
    pub fn span(self) -> &'t Span {
        unsafe { &*((self.0 as *const u8).add(OFFSET_ACCESSOR_PROPERTY_SPAN) as *const Span) }
    }

    #[inline]
    pub fn r#type(self) -> &'t AccessorPropertyType {
        unsafe {
            &*((self.0 as *const u8).add(OFFSET_ACCESSOR_PROPERTY_TYPE)
                as *const AccessorPropertyType)
        }
    }

    #[inline]
    pub fn key(self) -> &'t PropertyKey<'a> {
        unsafe {
            &*((self.0 as *const u8).add(OFFSET_ACCESSOR_PROPERTY_KEY) as *const PropertyKey<'a>)
        }
    }

    #[inline]
    pub fn type_annotation(self) -> &'t Option<ArenaBox<'a, TSTypeAnnotation<'a>>> {
        unsafe {
            &*((self.0 as *const u8).add(OFFSET_ACCESSOR_PROPERTY_TYPE_ANNOTATION)
                as *const Option<ArenaBox<'a, TSTypeAnnotation<'a>>>)
        }
    }

    #[inline]
    pub fn value(self) -> &'t Option<Expression<'a>> {
        unsafe {
            &*((self.0 as *const u8).add(OFFSET_ACCESSOR_PROPERTY_VALUE)
                as *const Option<Expression<'a>>)
        }
    }

    #[inline]
    pub fn computed(self) -> &'t bool {
        unsafe { &*((self.0 as *const u8).add(OFFSET_ACCESSOR_PROPERTY_COMPUTED) as *const bool) }
    }

    #[inline]
    pub fn r#static(self) -> &'t bool {
        unsafe { &*((self.0 as *const u8).add(OFFSET_ACCESSOR_PROPERTY_STATIC) as *const bool) }
    }

    #[inline]
    pub fn r#override(self) -> &'t bool {
        unsafe { &*((self.0 as *const u8).add(OFFSET_ACCESSOR_PROPERTY_OVERRIDE) as *const bool) }
    }

    #[inline]
    pub fn definite(self) -> &'t bool {
        unsafe { &*((self.0 as *const u8).add(OFFSET_ACCESSOR_PROPERTY_DEFINITE) as *const bool) }
    }

    #[inline]
    pub fn accessibility(self) -> &'t Option<TSAccessibility> {
        unsafe {
            &*((self.0 as *const u8).add(OFFSET_ACCESSOR_PROPERTY_ACCESSIBILITY)
                as *const Option<TSAccessibility>)
        }
    }
}

impl<'a, 't> GetAddress for AccessorPropertyWithoutDecorators<'a, 't> {
    #[inline]
    fn address(&self) -> Address {
        unsafe { Address::from_ptr(self.0) }
    }
}

#[repr(transparent)]
#[derive(Clone, Copy, Debug)]
pub struct AccessorPropertyWithoutKey<'a, 't>(
    pub(crate) *const AccessorProperty<'a>,
    pub(crate) PhantomData<&'t ()>,
);

impl<'a, 't> AccessorPropertyWithoutKey<'a, 't> {
    #[inline]
    pub fn node_id(self) -> &'t Cell<NodeId> {
        unsafe {
            &*((self.0 as *const u8).add(OFFSET_ACCESSOR_PROPERTY_NODE_ID) as *const Cell<NodeId>)
        }
    }

    #[inline]
    pub fn span(self) -> &'t Span {
        unsafe { &*((self.0 as *const u8).add(OFFSET_ACCESSOR_PROPERTY_SPAN) as *const Span) }
    }

    #[inline]
    pub fn r#type(self) -> &'t AccessorPropertyType {
        unsafe {
            &*((self.0 as *const u8).add(OFFSET_ACCESSOR_PROPERTY_TYPE)
                as *const AccessorPropertyType)
        }
    }

    #[inline]
    pub fn decorators(self) -> &'t ArenaVec<'a, Decorator<'a>> {
        unsafe {
            &*((self.0 as *const u8).add(OFFSET_ACCESSOR_PROPERTY_DECORATORS)
                as *const ArenaVec<'a, Decorator<'a>>)
        }
    }

    #[inline]
    pub fn type_annotation(self) -> &'t Option<ArenaBox<'a, TSTypeAnnotation<'a>>> {
        unsafe {
            &*((self.0 as *const u8).add(OFFSET_ACCESSOR_PROPERTY_TYPE_ANNOTATION)
                as *const Option<ArenaBox<'a, TSTypeAnnotation<'a>>>)
        }
    }

    #[inline]
    pub fn value(self) -> &'t Option<Expression<'a>> {
        unsafe {
            &*((self.0 as *const u8).add(OFFSET_ACCESSOR_PROPERTY_VALUE)
                as *const Option<Expression<'a>>)
        }
    }

    #[inline]
    pub fn computed(self) -> &'t bool {
        unsafe { &*((self.0 as *const u8).add(OFFSET_ACCESSOR_PROPERTY_COMPUTED) as *const bool) }
    }

    #[inline]
    pub fn r#static(self) -> &'t bool {
        unsafe { &*((self.0 as *const u8).add(OFFSET_ACCESSOR_PROPERTY_STATIC) as *const bool) }
    }

    #[inline]
    pub fn r#override(self) -> &'t bool {
        unsafe { &*((self.0 as *const u8).add(OFFSET_ACCESSOR_PROPERTY_OVERRIDE) as *const bool) }
    }

    #[inline]
    pub fn definite(self) -> &'t bool {
        unsafe { &*((self.0 as *const u8).add(OFFSET_ACCESSOR_PROPERTY_DEFINITE) as *const bool) }
    }

    #[inline]
    pub fn accessibility(self) -> &'t Option<TSAccessibility> {
        unsafe {
            &*((self.0 as *const u8).add(OFFSET_ACCESSOR_PROPERTY_ACCESSIBILITY)
                as *const Option<TSAccessibility>)
        }
    }
}

impl<'a, 't> GetAddress for AccessorPropertyWithoutKey<'a, 't> {
    #[inline]
    fn address(&self) -> Address {
        unsafe { Address::from_ptr(self.0) }
    }
}

#[repr(transparent)]
#[derive(Clone, Copy, Debug)]
pub struct AccessorPropertyWithoutValue<'a, 't>(
    pub(crate) *const AccessorProperty<'a>,
    pub(crate) PhantomData<&'t ()>,
);

impl<'a, 't> AccessorPropertyWithoutValue<'a, 't> {
    #[inline]
    pub fn node_id(self) -> &'t Cell<NodeId> {
        unsafe {
            &*((self.0 as *const u8).add(OFFSET_ACCESSOR_PROPERTY_NODE_ID) as *const Cell<NodeId>)
        }
    }

    #[inline]
    pub fn span(self) -> &'t Span {
        unsafe { &*((self.0 as *const u8).add(OFFSET_ACCESSOR_PROPERTY_SPAN) as *const Span) }
    }

    #[inline]
    pub fn r#type(self) -> &'t AccessorPropertyType {
        unsafe {
            &*((self.0 as *const u8).add(OFFSET_ACCESSOR_PROPERTY_TYPE)
                as *const AccessorPropertyType)
        }
    }

    #[inline]
    pub fn decorators(self) -> &'t ArenaVec<'a, Decorator<'a>> {
        unsafe {
            &*((self.0 as *const u8).add(OFFSET_ACCESSOR_PROPERTY_DECORATORS)
                as *const ArenaVec<'a, Decorator<'a>>)
        }
    }

    #[inline]
    pub fn key(self) -> &'t PropertyKey<'a> {
        unsafe {
            &*((self.0 as *const u8).add(OFFSET_ACCESSOR_PROPERTY_KEY) as *const PropertyKey<'a>)
        }
    }

    #[inline]
    pub fn type_annotation(self) -> &'t Option<ArenaBox<'a, TSTypeAnnotation<'a>>> {
        unsafe {
            &*((self.0 as *const u8).add(OFFSET_ACCESSOR_PROPERTY_TYPE_ANNOTATION)
                as *const Option<ArenaBox<'a, TSTypeAnnotation<'a>>>)
        }
    }

    #[inline]
    pub fn computed(self) -> &'t bool {
        unsafe { &*((self.0 as *const u8).add(OFFSET_ACCESSOR_PROPERTY_COMPUTED) as *const bool) }
    }

    #[inline]
    pub fn r#static(self) -> &'t bool {
        unsafe { &*((self.0 as *const u8).add(OFFSET_ACCESSOR_PROPERTY_STATIC) as *const bool) }
    }

    #[inline]
    pub fn r#override(self) -> &'t bool {
        unsafe { &*((self.0 as *const u8).add(OFFSET_ACCESSOR_PROPERTY_OVERRIDE) as *const bool) }
    }

    #[inline]
    pub fn definite(self) -> &'t bool {
        unsafe { &*((self.0 as *const u8).add(OFFSET_ACCESSOR_PROPERTY_DEFINITE) as *const bool) }
    }

    #[inline]
    pub fn accessibility(self) -> &'t Option<TSAccessibility> {
        unsafe {
            &*((self.0 as *const u8).add(OFFSET_ACCESSOR_PROPERTY_ACCESSIBILITY)
                as *const Option<TSAccessibility>)
        }
    }
}

impl<'a, 't> GetAddress for AccessorPropertyWithoutValue<'a, 't> {
    #[inline]
    fn address(&self) -> Address {
        unsafe { Address::from_ptr(self.0) }
    }
}

pub(crate) const OFFSET_IMPORT_EXPRESSION_NODE_ID: usize = offset_of!(ImportExpression, node_id);
pub(crate) const OFFSET_IMPORT_EXPRESSION_SPAN: usize = offset_of!(ImportExpression, span);
pub(crate) const OFFSET_IMPORT_EXPRESSION_SOURCE: usize = offset_of!(ImportExpression, source);
pub(crate) const OFFSET_IMPORT_EXPRESSION_OPTIONS: usize = offset_of!(ImportExpression, options);
pub(crate) const OFFSET_IMPORT_EXPRESSION_PHASE: usize = offset_of!(ImportExpression, phase);

#[repr(transparent)]
#[derive(Clone, Copy, Debug)]
pub struct ImportExpressionWithoutSource<'a, 't>(
    pub(crate) *const ImportExpression<'a>,
    pub(crate) PhantomData<&'t ()>,
);

impl<'a, 't> ImportExpressionWithoutSource<'a, 't> {
    #[inline]
    pub fn node_id(self) -> &'t Cell<NodeId> {
        unsafe {
            &*((self.0 as *const u8).add(OFFSET_IMPORT_EXPRESSION_NODE_ID) as *const Cell<NodeId>)
        }
    }

    #[inline]
    pub fn span(self) -> &'t Span {
        unsafe { &*((self.0 as *const u8).add(OFFSET_IMPORT_EXPRESSION_SPAN) as *const Span) }
    }

    #[inline]
    pub fn options(self) -> &'t Option<Expression<'a>> {
        unsafe {
            &*((self.0 as *const u8).add(OFFSET_IMPORT_EXPRESSION_OPTIONS)
                as *const Option<Expression<'a>>)
        }
    }

    #[inline]
    pub fn phase(self) -> &'t Option<ImportPhase> {
        unsafe {
            &*((self.0 as *const u8).add(OFFSET_IMPORT_EXPRESSION_PHASE)
                as *const Option<ImportPhase>)
        }
    }
}

impl<'a, 't> GetAddress for ImportExpressionWithoutSource<'a, 't> {
    #[inline]
    fn address(&self) -> Address {
        unsafe { Address::from_ptr(self.0) }
    }
}

#[repr(transparent)]
#[derive(Clone, Copy, Debug)]
pub struct ImportExpressionWithoutOptions<'a, 't>(
    pub(crate) *const ImportExpression<'a>,
    pub(crate) PhantomData<&'t ()>,
);

impl<'a, 't> ImportExpressionWithoutOptions<'a, 't> {
    #[inline]
    pub fn node_id(self) -> &'t Cell<NodeId> {
        unsafe {
            &*((self.0 as *const u8).add(OFFSET_IMPORT_EXPRESSION_NODE_ID) as *const Cell<NodeId>)
        }
    }

    #[inline]
    pub fn span(self) -> &'t Span {
        unsafe { &*((self.0 as *const u8).add(OFFSET_IMPORT_EXPRESSION_SPAN) as *const Span) }
    }

    #[inline]
    pub fn source(self) -> &'t Expression<'a> {
        unsafe {
            &*((self.0 as *const u8).add(OFFSET_IMPORT_EXPRESSION_SOURCE) as *const Expression<'a>)
        }
    }

    #[inline]
    pub fn phase(self) -> &'t Option<ImportPhase> {
        unsafe {
            &*((self.0 as *const u8).add(OFFSET_IMPORT_EXPRESSION_PHASE)
                as *const Option<ImportPhase>)
        }
    }
}

impl<'a, 't> GetAddress for ImportExpressionWithoutOptions<'a, 't> {
    #[inline]
    fn address(&self) -> Address {
        unsafe { Address::from_ptr(self.0) }
    }
}

pub(crate) const OFFSET_IMPORT_DECLARATION_NODE_ID: usize = offset_of!(ImportDeclaration, node_id);
pub(crate) const OFFSET_IMPORT_DECLARATION_SPAN: usize = offset_of!(ImportDeclaration, span);
pub(crate) const OFFSET_IMPORT_DECLARATION_SPECIFIERS: usize =
    offset_of!(ImportDeclaration, specifiers);
pub(crate) const OFFSET_IMPORT_DECLARATION_SOURCE: usize = offset_of!(ImportDeclaration, source);
pub(crate) const OFFSET_IMPORT_DECLARATION_PHASE: usize = offset_of!(ImportDeclaration, phase);
pub(crate) const OFFSET_IMPORT_DECLARATION_WITH_CLAUSE: usize =
    offset_of!(ImportDeclaration, with_clause);
pub(crate) const OFFSET_IMPORT_DECLARATION_IMPORT_KIND: usize =
    offset_of!(ImportDeclaration, import_kind);

#[repr(transparent)]
#[derive(Clone, Copy, Debug)]
pub struct ImportDeclarationWithoutSpecifiers<'a, 't>(
    pub(crate) *const ImportDeclaration<'a>,
    pub(crate) PhantomData<&'t ()>,
);

impl<'a, 't> ImportDeclarationWithoutSpecifiers<'a, 't> {
    #[inline]
    pub fn node_id(self) -> &'t Cell<NodeId> {
        unsafe {
            &*((self.0 as *const u8).add(OFFSET_IMPORT_DECLARATION_NODE_ID) as *const Cell<NodeId>)
        }
    }

    #[inline]
    pub fn span(self) -> &'t Span {
        unsafe { &*((self.0 as *const u8).add(OFFSET_IMPORT_DECLARATION_SPAN) as *const Span) }
    }

    #[inline]
    pub fn source(self) -> &'t StringLiteral<'a> {
        unsafe {
            &*((self.0 as *const u8).add(OFFSET_IMPORT_DECLARATION_SOURCE)
                as *const StringLiteral<'a>)
        }
    }

    #[inline]
    pub fn phase(self) -> &'t Option<ImportPhase> {
        unsafe {
            &*((self.0 as *const u8).add(OFFSET_IMPORT_DECLARATION_PHASE)
                as *const Option<ImportPhase>)
        }
    }

    #[inline]
    pub fn with_clause(self) -> &'t Option<ArenaBox<'a, WithClause<'a>>> {
        unsafe {
            &*((self.0 as *const u8).add(OFFSET_IMPORT_DECLARATION_WITH_CLAUSE)
                as *const Option<ArenaBox<'a, WithClause<'a>>>)
        }
    }

    #[inline]
    pub fn import_kind(self) -> &'t ImportOrExportKind {
        unsafe {
            &*((self.0 as *const u8).add(OFFSET_IMPORT_DECLARATION_IMPORT_KIND)
                as *const ImportOrExportKind)
        }
    }
}

impl<'a, 't> GetAddress for ImportDeclarationWithoutSpecifiers<'a, 't> {
    #[inline]
    fn address(&self) -> Address {
        unsafe { Address::from_ptr(self.0) }
    }
}

#[repr(transparent)]
#[derive(Clone, Copy, Debug)]
pub struct ImportDeclarationWithoutSource<'a, 't>(
    pub(crate) *const ImportDeclaration<'a>,
    pub(crate) PhantomData<&'t ()>,
);

impl<'a, 't> ImportDeclarationWithoutSource<'a, 't> {
    #[inline]
    pub fn node_id(self) -> &'t Cell<NodeId> {
        unsafe {
            &*((self.0 as *const u8).add(OFFSET_IMPORT_DECLARATION_NODE_ID) as *const Cell<NodeId>)
        }
    }

    #[inline]
    pub fn span(self) -> &'t Span {
        unsafe { &*((self.0 as *const u8).add(OFFSET_IMPORT_DECLARATION_SPAN) as *const Span) }
    }

    #[inline]
    pub fn specifiers(self) -> &'t Option<ArenaVec<'a, ImportDeclarationSpecifier<'a>>> {
        unsafe {
            &*((self.0 as *const u8).add(OFFSET_IMPORT_DECLARATION_SPECIFIERS)
                as *const Option<ArenaVec<'a, ImportDeclarationSpecifier<'a>>>)
        }
    }

    #[inline]
    pub fn phase(self) -> &'t Option<ImportPhase> {
        unsafe {
            &*((self.0 as *const u8).add(OFFSET_IMPORT_DECLARATION_PHASE)
                as *const Option<ImportPhase>)
        }
    }

    #[inline]
    pub fn with_clause(self) -> &'t Option<ArenaBox<'a, WithClause<'a>>> {
        unsafe {
            &*((self.0 as *const u8).add(OFFSET_IMPORT_DECLARATION_WITH_CLAUSE)
                as *const Option<ArenaBox<'a, WithClause<'a>>>)
        }
    }

    #[inline]
    pub fn import_kind(self) -> &'t ImportOrExportKind {
        unsafe {
            &*((self.0 as *const u8).add(OFFSET_IMPORT_DECLARATION_IMPORT_KIND)
                as *const ImportOrExportKind)
        }
    }
}

impl<'a, 't> GetAddress for ImportDeclarationWithoutSource<'a, 't> {
    #[inline]
    fn address(&self) -> Address {
        unsafe { Address::from_ptr(self.0) }
    }
}

#[repr(transparent)]
#[derive(Clone, Copy, Debug)]
pub struct ImportDeclarationWithoutWithClause<'a, 't>(
    pub(crate) *const ImportDeclaration<'a>,
    pub(crate) PhantomData<&'t ()>,
);

impl<'a, 't> ImportDeclarationWithoutWithClause<'a, 't> {
    #[inline]
    pub fn node_id(self) -> &'t Cell<NodeId> {
        unsafe {
            &*((self.0 as *const u8).add(OFFSET_IMPORT_DECLARATION_NODE_ID) as *const Cell<NodeId>)
        }
    }

    #[inline]
    pub fn span(self) -> &'t Span {
        unsafe { &*((self.0 as *const u8).add(OFFSET_IMPORT_DECLARATION_SPAN) as *const Span) }
    }

    #[inline]
    pub fn specifiers(self) -> &'t Option<ArenaVec<'a, ImportDeclarationSpecifier<'a>>> {
        unsafe {
            &*((self.0 as *const u8).add(OFFSET_IMPORT_DECLARATION_SPECIFIERS)
                as *const Option<ArenaVec<'a, ImportDeclarationSpecifier<'a>>>)
        }
    }

    #[inline]
    pub fn source(self) -> &'t StringLiteral<'a> {
        unsafe {
            &*((self.0 as *const u8).add(OFFSET_IMPORT_DECLARATION_SOURCE)
                as *const StringLiteral<'a>)
        }
    }

    #[inline]
    pub fn phase(self) -> &'t Option<ImportPhase> {
        unsafe {
            &*((self.0 as *const u8).add(OFFSET_IMPORT_DECLARATION_PHASE)
                as *const Option<ImportPhase>)
        }
    }

    #[inline]
    pub fn import_kind(self) -> &'t ImportOrExportKind {
        unsafe {
            &*((self.0 as *const u8).add(OFFSET_IMPORT_DECLARATION_IMPORT_KIND)
                as *const ImportOrExportKind)
        }
    }
}

impl<'a, 't> GetAddress for ImportDeclarationWithoutWithClause<'a, 't> {
    #[inline]
    fn address(&self) -> Address {
        unsafe { Address::from_ptr(self.0) }
    }
}

pub(crate) const OFFSET_IMPORT_SPECIFIER_NODE_ID: usize = offset_of!(ImportSpecifier, node_id);
pub(crate) const OFFSET_IMPORT_SPECIFIER_SPAN: usize = offset_of!(ImportSpecifier, span);
pub(crate) const OFFSET_IMPORT_SPECIFIER_IMPORTED: usize = offset_of!(ImportSpecifier, imported);
pub(crate) const OFFSET_IMPORT_SPECIFIER_LOCAL: usize = offset_of!(ImportSpecifier, local);
pub(crate) const OFFSET_IMPORT_SPECIFIER_IMPORT_KIND: usize =
    offset_of!(ImportSpecifier, import_kind);

#[repr(transparent)]
#[derive(Clone, Copy, Debug)]
pub struct ImportSpecifierWithoutImported<'a, 't>(
    pub(crate) *const ImportSpecifier<'a>,
    pub(crate) PhantomData<&'t ()>,
);

impl<'a, 't> ImportSpecifierWithoutImported<'a, 't> {
    #[inline]
    pub fn node_id(self) -> &'t Cell<NodeId> {
        unsafe {
            &*((self.0 as *const u8).add(OFFSET_IMPORT_SPECIFIER_NODE_ID) as *const Cell<NodeId>)
        }
    }

    #[inline]
    pub fn span(self) -> &'t Span {
        unsafe { &*((self.0 as *const u8).add(OFFSET_IMPORT_SPECIFIER_SPAN) as *const Span) }
    }

    #[inline]
    pub fn local(self) -> &'t BindingIdentifier<'a> {
        unsafe {
            &*((self.0 as *const u8).add(OFFSET_IMPORT_SPECIFIER_LOCAL)
                as *const BindingIdentifier<'a>)
        }
    }

    #[inline]
    pub fn import_kind(self) -> &'t ImportOrExportKind {
        unsafe {
            &*((self.0 as *const u8).add(OFFSET_IMPORT_SPECIFIER_IMPORT_KIND)
                as *const ImportOrExportKind)
        }
    }
}

impl<'a, 't> GetAddress for ImportSpecifierWithoutImported<'a, 't> {
    #[inline]
    fn address(&self) -> Address {
        unsafe { Address::from_ptr(self.0) }
    }
}

#[repr(transparent)]
#[derive(Clone, Copy, Debug)]
pub struct ImportSpecifierWithoutLocal<'a, 't>(
    pub(crate) *const ImportSpecifier<'a>,
    pub(crate) PhantomData<&'t ()>,
);

impl<'a, 't> ImportSpecifierWithoutLocal<'a, 't> {
    #[inline]
    pub fn node_id(self) -> &'t Cell<NodeId> {
        unsafe {
            &*((self.0 as *const u8).add(OFFSET_IMPORT_SPECIFIER_NODE_ID) as *const Cell<NodeId>)
        }
    }

    #[inline]
    pub fn span(self) -> &'t Span {
        unsafe { &*((self.0 as *const u8).add(OFFSET_IMPORT_SPECIFIER_SPAN) as *const Span) }
    }

    #[inline]
    pub fn imported(self) -> &'t ModuleExportName<'a> {
        unsafe {
            &*((self.0 as *const u8).add(OFFSET_IMPORT_SPECIFIER_IMPORTED)
                as *const ModuleExportName<'a>)
        }
    }

    #[inline]
    pub fn import_kind(self) -> &'t ImportOrExportKind {
        unsafe {
            &*((self.0 as *const u8).add(OFFSET_IMPORT_SPECIFIER_IMPORT_KIND)
                as *const ImportOrExportKind)
        }
    }
}

impl<'a, 't> GetAddress for ImportSpecifierWithoutLocal<'a, 't> {
    #[inline]
    fn address(&self) -> Address {
        unsafe { Address::from_ptr(self.0) }
    }
}

pub(crate) const OFFSET_IMPORT_DEFAULT_SPECIFIER_NODE_ID: usize =
    offset_of!(ImportDefaultSpecifier, node_id);
pub(crate) const OFFSET_IMPORT_DEFAULT_SPECIFIER_SPAN: usize =
    offset_of!(ImportDefaultSpecifier, span);
pub(crate) const OFFSET_IMPORT_DEFAULT_SPECIFIER_LOCAL: usize =
    offset_of!(ImportDefaultSpecifier, local);

#[repr(transparent)]
#[derive(Clone, Copy, Debug)]
pub struct ImportDefaultSpecifierWithoutLocal<'a, 't>(
    pub(crate) *const ImportDefaultSpecifier<'a>,
    pub(crate) PhantomData<&'t ()>,
);

impl<'a, 't> ImportDefaultSpecifierWithoutLocal<'a, 't> {
    #[inline]
    pub fn node_id(self) -> &'t Cell<NodeId> {
        unsafe {
            &*((self.0 as *const u8).add(OFFSET_IMPORT_DEFAULT_SPECIFIER_NODE_ID)
                as *const Cell<NodeId>)
        }
    }

    #[inline]
    pub fn span(self) -> &'t Span {
        unsafe {
            &*((self.0 as *const u8).add(OFFSET_IMPORT_DEFAULT_SPECIFIER_SPAN) as *const Span)
        }
    }
}

impl<'a, 't> GetAddress for ImportDefaultSpecifierWithoutLocal<'a, 't> {
    #[inline]
    fn address(&self) -> Address {
        unsafe { Address::from_ptr(self.0) }
    }
}

pub(crate) const OFFSET_IMPORT_NAMESPACE_SPECIFIER_NODE_ID: usize =
    offset_of!(ImportNamespaceSpecifier, node_id);
pub(crate) const OFFSET_IMPORT_NAMESPACE_SPECIFIER_SPAN: usize =
    offset_of!(ImportNamespaceSpecifier, span);
pub(crate) const OFFSET_IMPORT_NAMESPACE_SPECIFIER_LOCAL: usize =
    offset_of!(ImportNamespaceSpecifier, local);

#[repr(transparent)]
#[derive(Clone, Copy, Debug)]
pub struct ImportNamespaceSpecifierWithoutLocal<'a, 't>(
    pub(crate) *const ImportNamespaceSpecifier<'a>,
    pub(crate) PhantomData<&'t ()>,
);

impl<'a, 't> ImportNamespaceSpecifierWithoutLocal<'a, 't> {
    #[inline]
    pub fn node_id(self) -> &'t Cell<NodeId> {
        unsafe {
            &*((self.0 as *const u8).add(OFFSET_IMPORT_NAMESPACE_SPECIFIER_NODE_ID)
                as *const Cell<NodeId>)
        }
    }

    #[inline]
    pub fn span(self) -> &'t Span {
        unsafe {
            &*((self.0 as *const u8).add(OFFSET_IMPORT_NAMESPACE_SPECIFIER_SPAN) as *const Span)
        }
    }
}

impl<'a, 't> GetAddress for ImportNamespaceSpecifierWithoutLocal<'a, 't> {
    #[inline]
    fn address(&self) -> Address {
        unsafe { Address::from_ptr(self.0) }
    }
}

pub(crate) const OFFSET_WITH_CLAUSE_NODE_ID: usize = offset_of!(WithClause, node_id);
pub(crate) const OFFSET_WITH_CLAUSE_SPAN: usize = offset_of!(WithClause, span);
pub(crate) const OFFSET_WITH_CLAUSE_KEYWORD: usize = offset_of!(WithClause, keyword);
pub(crate) const OFFSET_WITH_CLAUSE_WITH_ENTRIES: usize = offset_of!(WithClause, with_entries);

#[repr(transparent)]
#[derive(Clone, Copy, Debug)]
pub struct WithClauseWithoutWithEntries<'a, 't>(
    pub(crate) *const WithClause<'a>,
    pub(crate) PhantomData<&'t ()>,
);

impl<'a, 't> WithClauseWithoutWithEntries<'a, 't> {
    #[inline]
    pub fn node_id(self) -> &'t Cell<NodeId> {
        unsafe { &*((self.0 as *const u8).add(OFFSET_WITH_CLAUSE_NODE_ID) as *const Cell<NodeId>) }
    }

    #[inline]
    pub fn span(self) -> &'t Span {
        unsafe { &*((self.0 as *const u8).add(OFFSET_WITH_CLAUSE_SPAN) as *const Span) }
    }

    #[inline]
    pub fn keyword(self) -> &'t WithClauseKeyword {
        unsafe {
            &*((self.0 as *const u8).add(OFFSET_WITH_CLAUSE_KEYWORD) as *const WithClauseKeyword)
        }
    }
}

impl<'a, 't> GetAddress for WithClauseWithoutWithEntries<'a, 't> {
    #[inline]
    fn address(&self) -> Address {
        unsafe { Address::from_ptr(self.0) }
    }
}

pub(crate) const OFFSET_IMPORT_ATTRIBUTE_NODE_ID: usize = offset_of!(ImportAttribute, node_id);
pub(crate) const OFFSET_IMPORT_ATTRIBUTE_SPAN: usize = offset_of!(ImportAttribute, span);
pub(crate) const OFFSET_IMPORT_ATTRIBUTE_KEY: usize = offset_of!(ImportAttribute, key);
pub(crate) const OFFSET_IMPORT_ATTRIBUTE_VALUE: usize = offset_of!(ImportAttribute, value);

#[repr(transparent)]
#[derive(Clone, Copy, Debug)]
pub struct ImportAttributeWithoutKey<'a, 't>(
    pub(crate) *const ImportAttribute<'a>,
    pub(crate) PhantomData<&'t ()>,
);

impl<'a, 't> ImportAttributeWithoutKey<'a, 't> {
    #[inline]
    pub fn node_id(self) -> &'t Cell<NodeId> {
        unsafe {
            &*((self.0 as *const u8).add(OFFSET_IMPORT_ATTRIBUTE_NODE_ID) as *const Cell<NodeId>)
        }
    }

    #[inline]
    pub fn span(self) -> &'t Span {
        unsafe { &*((self.0 as *const u8).add(OFFSET_IMPORT_ATTRIBUTE_SPAN) as *const Span) }
    }

    #[inline]
    pub fn value(self) -> &'t StringLiteral<'a> {
        unsafe {
            &*((self.0 as *const u8).add(OFFSET_IMPORT_ATTRIBUTE_VALUE) as *const StringLiteral<'a>)
        }
    }
}

impl<'a, 't> GetAddress for ImportAttributeWithoutKey<'a, 't> {
    #[inline]
    fn address(&self) -> Address {
        unsafe { Address::from_ptr(self.0) }
    }
}

#[repr(transparent)]
#[derive(Clone, Copy, Debug)]
pub struct ImportAttributeWithoutValue<'a, 't>(
    pub(crate) *const ImportAttribute<'a>,
    pub(crate) PhantomData<&'t ()>,
);

impl<'a, 't> ImportAttributeWithoutValue<'a, 't> {
    #[inline]
    pub fn node_id(self) -> &'t Cell<NodeId> {
        unsafe {
            &*((self.0 as *const u8).add(OFFSET_IMPORT_ATTRIBUTE_NODE_ID) as *const Cell<NodeId>)
        }
    }

    #[inline]
    pub fn span(self) -> &'t Span {
        unsafe { &*((self.0 as *const u8).add(OFFSET_IMPORT_ATTRIBUTE_SPAN) as *const Span) }
    }

    #[inline]
    pub fn key(self) -> &'t ImportAttributeKey<'a> {
        unsafe {
            &*((self.0 as *const u8).add(OFFSET_IMPORT_ATTRIBUTE_KEY)
                as *const ImportAttributeKey<'a>)
        }
    }
}

impl<'a, 't> GetAddress for ImportAttributeWithoutValue<'a, 't> {
    #[inline]
    fn address(&self) -> Address {
        unsafe { Address::from_ptr(self.0) }
    }
}

pub(crate) const OFFSET_EXPORT_NAMED_DECLARATION_NODE_ID: usize =
    offset_of!(ExportNamedDeclaration, node_id);
pub(crate) const OFFSET_EXPORT_NAMED_DECLARATION_SPAN: usize =
    offset_of!(ExportNamedDeclaration, span);
pub(crate) const OFFSET_EXPORT_NAMED_DECLARATION_DECLARATION: usize =
    offset_of!(ExportNamedDeclaration, declaration);
pub(crate) const OFFSET_EXPORT_NAMED_DECLARATION_SPECIFIERS: usize =
    offset_of!(ExportNamedDeclaration, specifiers);
pub(crate) const OFFSET_EXPORT_NAMED_DECLARATION_SOURCE: usize =
    offset_of!(ExportNamedDeclaration, source);
pub(crate) const OFFSET_EXPORT_NAMED_DECLARATION_EXPORT_KIND: usize =
    offset_of!(ExportNamedDeclaration, export_kind);
pub(crate) const OFFSET_EXPORT_NAMED_DECLARATION_WITH_CLAUSE: usize =
    offset_of!(ExportNamedDeclaration, with_clause);

#[repr(transparent)]
#[derive(Clone, Copy, Debug)]
pub struct ExportNamedDeclarationWithoutDeclaration<'a, 't>(
    pub(crate) *const ExportNamedDeclaration<'a>,
    pub(crate) PhantomData<&'t ()>,
);

impl<'a, 't> ExportNamedDeclarationWithoutDeclaration<'a, 't> {
    #[inline]
    pub fn node_id(self) -> &'t Cell<NodeId> {
        unsafe {
            &*((self.0 as *const u8).add(OFFSET_EXPORT_NAMED_DECLARATION_NODE_ID)
                as *const Cell<NodeId>)
        }
    }

    #[inline]
    pub fn span(self) -> &'t Span {
        unsafe {
            &*((self.0 as *const u8).add(OFFSET_EXPORT_NAMED_DECLARATION_SPAN) as *const Span)
        }
    }

    #[inline]
    pub fn specifiers(self) -> &'t ArenaVec<'a, ExportSpecifier<'a>> {
        unsafe {
            &*((self.0 as *const u8).add(OFFSET_EXPORT_NAMED_DECLARATION_SPECIFIERS)
                as *const ArenaVec<'a, ExportSpecifier<'a>>)
        }
    }

    #[inline]
    pub fn source(self) -> &'t Option<StringLiteral<'a>> {
        unsafe {
            &*((self.0 as *const u8).add(OFFSET_EXPORT_NAMED_DECLARATION_SOURCE)
                as *const Option<StringLiteral<'a>>)
        }
    }

    #[inline]
    pub fn export_kind(self) -> &'t ImportOrExportKind {
        unsafe {
            &*((self.0 as *const u8).add(OFFSET_EXPORT_NAMED_DECLARATION_EXPORT_KIND)
                as *const ImportOrExportKind)
        }
    }

    #[inline]
    pub fn with_clause(self) -> &'t Option<ArenaBox<'a, WithClause<'a>>> {
        unsafe {
            &*((self.0 as *const u8).add(OFFSET_EXPORT_NAMED_DECLARATION_WITH_CLAUSE)
                as *const Option<ArenaBox<'a, WithClause<'a>>>)
        }
    }
}

impl<'a, 't> GetAddress for ExportNamedDeclarationWithoutDeclaration<'a, 't> {
    #[inline]
    fn address(&self) -> Address {
        unsafe { Address::from_ptr(self.0) }
    }
}

#[repr(transparent)]
#[derive(Clone, Copy, Debug)]
pub struct ExportNamedDeclarationWithoutSpecifiers<'a, 't>(
    pub(crate) *const ExportNamedDeclaration<'a>,
    pub(crate) PhantomData<&'t ()>,
);

impl<'a, 't> ExportNamedDeclarationWithoutSpecifiers<'a, 't> {
    #[inline]
    pub fn node_id(self) -> &'t Cell<NodeId> {
        unsafe {
            &*((self.0 as *const u8).add(OFFSET_EXPORT_NAMED_DECLARATION_NODE_ID)
                as *const Cell<NodeId>)
        }
    }

    #[inline]
    pub fn span(self) -> &'t Span {
        unsafe {
            &*((self.0 as *const u8).add(OFFSET_EXPORT_NAMED_DECLARATION_SPAN) as *const Span)
        }
    }

    #[inline]
    pub fn declaration(self) -> &'t Option<Declaration<'a>> {
        unsafe {
            &*((self.0 as *const u8).add(OFFSET_EXPORT_NAMED_DECLARATION_DECLARATION)
                as *const Option<Declaration<'a>>)
        }
    }

    #[inline]
    pub fn source(self) -> &'t Option<StringLiteral<'a>> {
        unsafe {
            &*((self.0 as *const u8).add(OFFSET_EXPORT_NAMED_DECLARATION_SOURCE)
                as *const Option<StringLiteral<'a>>)
        }
    }

    #[inline]
    pub fn export_kind(self) -> &'t ImportOrExportKind {
        unsafe {
            &*((self.0 as *const u8).add(OFFSET_EXPORT_NAMED_DECLARATION_EXPORT_KIND)
                as *const ImportOrExportKind)
        }
    }

    #[inline]
    pub fn with_clause(self) -> &'t Option<ArenaBox<'a, WithClause<'a>>> {
        unsafe {
            &*((self.0 as *const u8).add(OFFSET_EXPORT_NAMED_DECLARATION_WITH_CLAUSE)
                as *const Option<ArenaBox<'a, WithClause<'a>>>)
        }
    }
}

impl<'a, 't> GetAddress for ExportNamedDeclarationWithoutSpecifiers<'a, 't> {
    #[inline]
    fn address(&self) -> Address {
        unsafe { Address::from_ptr(self.0) }
    }
}

#[repr(transparent)]
#[derive(Clone, Copy, Debug)]
pub struct ExportNamedDeclarationWithoutSource<'a, 't>(
    pub(crate) *const ExportNamedDeclaration<'a>,
    pub(crate) PhantomData<&'t ()>,
);

impl<'a, 't> ExportNamedDeclarationWithoutSource<'a, 't> {
    #[inline]
    pub fn node_id(self) -> &'t Cell<NodeId> {
        unsafe {
            &*((self.0 as *const u8).add(OFFSET_EXPORT_NAMED_DECLARATION_NODE_ID)
                as *const Cell<NodeId>)
        }
    }

    #[inline]
    pub fn span(self) -> &'t Span {
        unsafe {
            &*((self.0 as *const u8).add(OFFSET_EXPORT_NAMED_DECLARATION_SPAN) as *const Span)
        }
    }

    #[inline]
    pub fn declaration(self) -> &'t Option<Declaration<'a>> {
        unsafe {
            &*((self.0 as *const u8).add(OFFSET_EXPORT_NAMED_DECLARATION_DECLARATION)
                as *const Option<Declaration<'a>>)
        }
    }

    #[inline]
    pub fn specifiers(self) -> &'t ArenaVec<'a, ExportSpecifier<'a>> {
        unsafe {
            &*((self.0 as *const u8).add(OFFSET_EXPORT_NAMED_DECLARATION_SPECIFIERS)
                as *const ArenaVec<'a, ExportSpecifier<'a>>)
        }
    }

    #[inline]
    pub fn export_kind(self) -> &'t ImportOrExportKind {
        unsafe {
            &*((self.0 as *const u8).add(OFFSET_EXPORT_NAMED_DECLARATION_EXPORT_KIND)
                as *const ImportOrExportKind)
        }
    }

    #[inline]
    pub fn with_clause(self) -> &'t Option<ArenaBox<'a, WithClause<'a>>> {
        unsafe {
            &*((self.0 as *const u8).add(OFFSET_EXPORT_NAMED_DECLARATION_WITH_CLAUSE)
                as *const Option<ArenaBox<'a, WithClause<'a>>>)
        }
    }
}

impl<'a, 't> GetAddress for ExportNamedDeclarationWithoutSource<'a, 't> {
    #[inline]
    fn address(&self) -> Address {
        unsafe { Address::from_ptr(self.0) }
    }
}

#[repr(transparent)]
#[derive(Clone, Copy, Debug)]
pub struct ExportNamedDeclarationWithoutWithClause<'a, 't>(
    pub(crate) *const ExportNamedDeclaration<'a>,
    pub(crate) PhantomData<&'t ()>,
);

impl<'a, 't> ExportNamedDeclarationWithoutWithClause<'a, 't> {
    #[inline]
    pub fn node_id(self) -> &'t Cell<NodeId> {
        unsafe {
            &*((self.0 as *const u8).add(OFFSET_EXPORT_NAMED_DECLARATION_NODE_ID)
                as *const Cell<NodeId>)
        }
    }

    #[inline]
    pub fn span(self) -> &'t Span {
        unsafe {
            &*((self.0 as *const u8).add(OFFSET_EXPORT_NAMED_DECLARATION_SPAN) as *const Span)
        }
    }

    #[inline]
    pub fn declaration(self) -> &'t Option<Declaration<'a>> {
        unsafe {
            &*((self.0 as *const u8).add(OFFSET_EXPORT_NAMED_DECLARATION_DECLARATION)
                as *const Option<Declaration<'a>>)
        }
    }

    #[inline]
    pub fn specifiers(self) -> &'t ArenaVec<'a, ExportSpecifier<'a>> {
        unsafe {
            &*((self.0 as *const u8).add(OFFSET_EXPORT_NAMED_DECLARATION_SPECIFIERS)
                as *const ArenaVec<'a, ExportSpecifier<'a>>)
        }
    }

    #[inline]
    pub fn source(self) -> &'t Option<StringLiteral<'a>> {
        unsafe {
            &*((self.0 as *const u8).add(OFFSET_EXPORT_NAMED_DECLARATION_SOURCE)
                as *const Option<StringLiteral<'a>>)
        }
    }

    #[inline]
    pub fn export_kind(self) -> &'t ImportOrExportKind {
        unsafe {
            &*((self.0 as *const u8).add(OFFSET_EXPORT_NAMED_DECLARATION_EXPORT_KIND)
                as *const ImportOrExportKind)
        }
    }
}

impl<'a, 't> GetAddress for ExportNamedDeclarationWithoutWithClause<'a, 't> {
    #[inline]
    fn address(&self) -> Address {
        unsafe { Address::from_ptr(self.0) }
    }
}

pub(crate) const OFFSET_EXPORT_DEFAULT_DECLARATION_NODE_ID: usize =
    offset_of!(ExportDefaultDeclaration, node_id);
pub(crate) const OFFSET_EXPORT_DEFAULT_DECLARATION_SPAN: usize =
    offset_of!(ExportDefaultDeclaration, span);
pub(crate) const OFFSET_EXPORT_DEFAULT_DECLARATION_DECLARATION: usize =
    offset_of!(ExportDefaultDeclaration, declaration);

#[repr(transparent)]
#[derive(Clone, Copy, Debug)]
pub struct ExportDefaultDeclarationWithoutDeclaration<'a, 't>(
    pub(crate) *const ExportDefaultDeclaration<'a>,
    pub(crate) PhantomData<&'t ()>,
);

impl<'a, 't> ExportDefaultDeclarationWithoutDeclaration<'a, 't> {
    #[inline]
    pub fn node_id(self) -> &'t Cell<NodeId> {
        unsafe {
            &*((self.0 as *const u8).add(OFFSET_EXPORT_DEFAULT_DECLARATION_NODE_ID)
                as *const Cell<NodeId>)
        }
    }

    #[inline]
    pub fn span(self) -> &'t Span {
        unsafe {
            &*((self.0 as *const u8).add(OFFSET_EXPORT_DEFAULT_DECLARATION_SPAN) as *const Span)
        }
    }
}

impl<'a, 't> GetAddress for ExportDefaultDeclarationWithoutDeclaration<'a, 't> {
    #[inline]
    fn address(&self) -> Address {
        unsafe { Address::from_ptr(self.0) }
    }
}

pub(crate) const OFFSET_EXPORT_ALL_DECLARATION_NODE_ID: usize =
    offset_of!(ExportAllDeclaration, node_id);
pub(crate) const OFFSET_EXPORT_ALL_DECLARATION_SPAN: usize = offset_of!(ExportAllDeclaration, span);
pub(crate) const OFFSET_EXPORT_ALL_DECLARATION_EXPORTED: usize =
    offset_of!(ExportAllDeclaration, exported);
pub(crate) const OFFSET_EXPORT_ALL_DECLARATION_SOURCE: usize =
    offset_of!(ExportAllDeclaration, source);
pub(crate) const OFFSET_EXPORT_ALL_DECLARATION_WITH_CLAUSE: usize =
    offset_of!(ExportAllDeclaration, with_clause);
pub(crate) const OFFSET_EXPORT_ALL_DECLARATION_EXPORT_KIND: usize =
    offset_of!(ExportAllDeclaration, export_kind);

#[repr(transparent)]
#[derive(Clone, Copy, Debug)]
pub struct ExportAllDeclarationWithoutExported<'a, 't>(
    pub(crate) *const ExportAllDeclaration<'a>,
    pub(crate) PhantomData<&'t ()>,
);

impl<'a, 't> ExportAllDeclarationWithoutExported<'a, 't> {
    #[inline]
    pub fn node_id(self) -> &'t Cell<NodeId> {
        unsafe {
            &*((self.0 as *const u8).add(OFFSET_EXPORT_ALL_DECLARATION_NODE_ID)
                as *const Cell<NodeId>)
        }
    }

    #[inline]
    pub fn span(self) -> &'t Span {
        unsafe { &*((self.0 as *const u8).add(OFFSET_EXPORT_ALL_DECLARATION_SPAN) as *const Span) }
    }

    #[inline]
    pub fn source(self) -> &'t StringLiteral<'a> {
        unsafe {
            &*((self.0 as *const u8).add(OFFSET_EXPORT_ALL_DECLARATION_SOURCE)
                as *const StringLiteral<'a>)
        }
    }

    #[inline]
    pub fn with_clause(self) -> &'t Option<ArenaBox<'a, WithClause<'a>>> {
        unsafe {
            &*((self.0 as *const u8).add(OFFSET_EXPORT_ALL_DECLARATION_WITH_CLAUSE)
                as *const Option<ArenaBox<'a, WithClause<'a>>>)
        }
    }

    #[inline]
    pub fn export_kind(self) -> &'t ImportOrExportKind {
        unsafe {
            &*((self.0 as *const u8).add(OFFSET_EXPORT_ALL_DECLARATION_EXPORT_KIND)
                as *const ImportOrExportKind)
        }
    }
}

impl<'a, 't> GetAddress for ExportAllDeclarationWithoutExported<'a, 't> {
    #[inline]
    fn address(&self) -> Address {
        unsafe { Address::from_ptr(self.0) }
    }
}

#[repr(transparent)]
#[derive(Clone, Copy, Debug)]
pub struct ExportAllDeclarationWithoutSource<'a, 't>(
    pub(crate) *const ExportAllDeclaration<'a>,
    pub(crate) PhantomData<&'t ()>,
);

impl<'a, 't> ExportAllDeclarationWithoutSource<'a, 't> {
    #[inline]
    pub fn node_id(self) -> &'t Cell<NodeId> {
        unsafe {
            &*((self.0 as *const u8).add(OFFSET_EXPORT_ALL_DECLARATION_NODE_ID)
                as *const Cell<NodeId>)
        }
    }

    #[inline]
    pub fn span(self) -> &'t Span {
        unsafe { &*((self.0 as *const u8).add(OFFSET_EXPORT_ALL_DECLARATION_SPAN) as *const Span) }
    }

    #[inline]
    pub fn exported(self) -> &'t Option<ModuleExportName<'a>> {
        unsafe {
            &*((self.0 as *const u8).add(OFFSET_EXPORT_ALL_DECLARATION_EXPORTED)
                as *const Option<ModuleExportName<'a>>)
        }
    }

    #[inline]
    pub fn with_clause(self) -> &'t Option<ArenaBox<'a, WithClause<'a>>> {
        unsafe {
            &*((self.0 as *const u8).add(OFFSET_EXPORT_ALL_DECLARATION_WITH_CLAUSE)
                as *const Option<ArenaBox<'a, WithClause<'a>>>)
        }
    }

    #[inline]
    pub fn export_kind(self) -> &'t ImportOrExportKind {
        unsafe {
            &*((self.0 as *const u8).add(OFFSET_EXPORT_ALL_DECLARATION_EXPORT_KIND)
                as *const ImportOrExportKind)
        }
    }
}

impl<'a, 't> GetAddress for ExportAllDeclarationWithoutSource<'a, 't> {
    #[inline]
    fn address(&self) -> Address {
        unsafe { Address::from_ptr(self.0) }
    }
}

#[repr(transparent)]
#[derive(Clone, Copy, Debug)]
pub struct ExportAllDeclarationWithoutWithClause<'a, 't>(
    pub(crate) *const ExportAllDeclaration<'a>,
    pub(crate) PhantomData<&'t ()>,
);

impl<'a, 't> ExportAllDeclarationWithoutWithClause<'a, 't> {
    #[inline]
    pub fn node_id(self) -> &'t Cell<NodeId> {
        unsafe {
            &*((self.0 as *const u8).add(OFFSET_EXPORT_ALL_DECLARATION_NODE_ID)
                as *const Cell<NodeId>)
        }
    }

    #[inline]
    pub fn span(self) -> &'t Span {
        unsafe { &*((self.0 as *const u8).add(OFFSET_EXPORT_ALL_DECLARATION_SPAN) as *const Span) }
    }

    #[inline]
    pub fn exported(self) -> &'t Option<ModuleExportName<'a>> {
        unsafe {
            &*((self.0 as *const u8).add(OFFSET_EXPORT_ALL_DECLARATION_EXPORTED)
                as *const Option<ModuleExportName<'a>>)
        }
    }

    #[inline]
    pub fn source(self) -> &'t StringLiteral<'a> {
        unsafe {
            &*((self.0 as *const u8).add(OFFSET_EXPORT_ALL_DECLARATION_SOURCE)
                as *const StringLiteral<'a>)
        }
    }

    #[inline]
    pub fn export_kind(self) -> &'t ImportOrExportKind {
        unsafe {
            &*((self.0 as *const u8).add(OFFSET_EXPORT_ALL_DECLARATION_EXPORT_KIND)
                as *const ImportOrExportKind)
        }
    }
}

impl<'a, 't> GetAddress for ExportAllDeclarationWithoutWithClause<'a, 't> {
    #[inline]
    fn address(&self) -> Address {
        unsafe { Address::from_ptr(self.0) }
    }
}

pub(crate) const OFFSET_EXPORT_SPECIFIER_NODE_ID: usize = offset_of!(ExportSpecifier, node_id);
pub(crate) const OFFSET_EXPORT_SPECIFIER_SPAN: usize = offset_of!(ExportSpecifier, span);
pub(crate) const OFFSET_EXPORT_SPECIFIER_LOCAL: usize = offset_of!(ExportSpecifier, local);
pub(crate) const OFFSET_EXPORT_SPECIFIER_EXPORTED: usize = offset_of!(ExportSpecifier, exported);
pub(crate) const OFFSET_EXPORT_SPECIFIER_EXPORT_KIND: usize =
    offset_of!(ExportSpecifier, export_kind);

#[repr(transparent)]
#[derive(Clone, Copy, Debug)]
pub struct ExportSpecifierWithoutLocal<'a, 't>(
    pub(crate) *const ExportSpecifier<'a>,
    pub(crate) PhantomData<&'t ()>,
);

impl<'a, 't> ExportSpecifierWithoutLocal<'a, 't> {
    #[inline]
    pub fn node_id(self) -> &'t Cell<NodeId> {
        unsafe {
            &*((self.0 as *const u8).add(OFFSET_EXPORT_SPECIFIER_NODE_ID) as *const Cell<NodeId>)
        }
    }

    #[inline]
    pub fn span(self) -> &'t Span {
        unsafe { &*((self.0 as *const u8).add(OFFSET_EXPORT_SPECIFIER_SPAN) as *const Span) }
    }

    #[inline]
    pub fn exported(self) -> &'t ModuleExportName<'a> {
        unsafe {
            &*((self.0 as *const u8).add(OFFSET_EXPORT_SPECIFIER_EXPORTED)
                as *const ModuleExportName<'a>)
        }
    }

    #[inline]
    pub fn export_kind(self) -> &'t ImportOrExportKind {
        unsafe {
            &*((self.0 as *const u8).add(OFFSET_EXPORT_SPECIFIER_EXPORT_KIND)
                as *const ImportOrExportKind)
        }
    }
}

impl<'a, 't> GetAddress for ExportSpecifierWithoutLocal<'a, 't> {
    #[inline]
    fn address(&self) -> Address {
        unsafe { Address::from_ptr(self.0) }
    }
}

#[repr(transparent)]
#[derive(Clone, Copy, Debug)]
pub struct ExportSpecifierWithoutExported<'a, 't>(
    pub(crate) *const ExportSpecifier<'a>,
    pub(crate) PhantomData<&'t ()>,
);

impl<'a, 't> ExportSpecifierWithoutExported<'a, 't> {
    #[inline]
    pub fn node_id(self) -> &'t Cell<NodeId> {
        unsafe {
            &*((self.0 as *const u8).add(OFFSET_EXPORT_SPECIFIER_NODE_ID) as *const Cell<NodeId>)
        }
    }

    #[inline]
    pub fn span(self) -> &'t Span {
        unsafe { &*((self.0 as *const u8).add(OFFSET_EXPORT_SPECIFIER_SPAN) as *const Span) }
    }

    #[inline]
    pub fn local(self) -> &'t ModuleExportName<'a> {
        unsafe {
            &*((self.0 as *const u8).add(OFFSET_EXPORT_SPECIFIER_LOCAL)
                as *const ModuleExportName<'a>)
        }
    }

    #[inline]
    pub fn export_kind(self) -> &'t ImportOrExportKind {
        unsafe {
            &*((self.0 as *const u8).add(OFFSET_EXPORT_SPECIFIER_EXPORT_KIND)
                as *const ImportOrExportKind)
        }
    }
}

impl<'a, 't> GetAddress for ExportSpecifierWithoutExported<'a, 't> {
    #[inline]
    fn address(&self) -> Address {
        unsafe { Address::from_ptr(self.0) }
    }
}

pub(crate) const OFFSET_V8_INTRINSIC_EXPRESSION_NODE_ID: usize =
    offset_of!(V8IntrinsicExpression, node_id);
pub(crate) const OFFSET_V8_INTRINSIC_EXPRESSION_SPAN: usize =
    offset_of!(V8IntrinsicExpression, span);
pub(crate) const OFFSET_V8_INTRINSIC_EXPRESSION_NAME: usize =
    offset_of!(V8IntrinsicExpression, name);
pub(crate) const OFFSET_V8_INTRINSIC_EXPRESSION_ARGUMENTS: usize =
    offset_of!(V8IntrinsicExpression, arguments);

#[repr(transparent)]
#[derive(Clone, Copy, Debug)]
pub struct V8IntrinsicExpressionWithoutName<'a, 't>(
    pub(crate) *const V8IntrinsicExpression<'a>,
    pub(crate) PhantomData<&'t ()>,
);

impl<'a, 't> V8IntrinsicExpressionWithoutName<'a, 't> {
    #[inline]
    pub fn node_id(self) -> &'t Cell<NodeId> {
        unsafe {
            &*((self.0 as *const u8).add(OFFSET_V8_INTRINSIC_EXPRESSION_NODE_ID)
                as *const Cell<NodeId>)
        }
    }

    #[inline]
    pub fn span(self) -> &'t Span {
        unsafe { &*((self.0 as *const u8).add(OFFSET_V8_INTRINSIC_EXPRESSION_SPAN) as *const Span) }
    }

    #[inline]
    pub fn arguments(self) -> &'t ArenaVec<'a, Argument<'a>> {
        unsafe {
            &*((self.0 as *const u8).add(OFFSET_V8_INTRINSIC_EXPRESSION_ARGUMENTS)
                as *const ArenaVec<'a, Argument<'a>>)
        }
    }
}

impl<'a, 't> GetAddress for V8IntrinsicExpressionWithoutName<'a, 't> {
    #[inline]
    fn address(&self) -> Address {
        unsafe { Address::from_ptr(self.0) }
    }
}

#[repr(transparent)]
#[derive(Clone, Copy, Debug)]
pub struct V8IntrinsicExpressionWithoutArguments<'a, 't>(
    pub(crate) *const V8IntrinsicExpression<'a>,
    pub(crate) PhantomData<&'t ()>,
);

impl<'a, 't> V8IntrinsicExpressionWithoutArguments<'a, 't> {
    #[inline]
    pub fn node_id(self) -> &'t Cell<NodeId> {
        unsafe {
            &*((self.0 as *const u8).add(OFFSET_V8_INTRINSIC_EXPRESSION_NODE_ID)
                as *const Cell<NodeId>)
        }
    }

    #[inline]
    pub fn span(self) -> &'t Span {
        unsafe { &*((self.0 as *const u8).add(OFFSET_V8_INTRINSIC_EXPRESSION_SPAN) as *const Span) }
    }

    #[inline]
    pub fn name(self) -> &'t IdentifierName<'a> {
        unsafe {
            &*((self.0 as *const u8).add(OFFSET_V8_INTRINSIC_EXPRESSION_NAME)
                as *const IdentifierName<'a>)
        }
    }
}

impl<'a, 't> GetAddress for V8IntrinsicExpressionWithoutArguments<'a, 't> {
    #[inline]
    fn address(&self) -> Address {
        unsafe { Address::from_ptr(self.0) }
    }
}

pub(crate) const OFFSET_JSX_ELEMENT_NODE_ID: usize = offset_of!(JSXElement, node_id);
pub(crate) const OFFSET_JSX_ELEMENT_SPAN: usize = offset_of!(JSXElement, span);
pub(crate) const OFFSET_JSX_ELEMENT_OPENING_ELEMENT: usize =
    offset_of!(JSXElement, opening_element);
pub(crate) const OFFSET_JSX_ELEMENT_CHILDREN: usize = offset_of!(JSXElement, children);
pub(crate) const OFFSET_JSX_ELEMENT_CLOSING_ELEMENT: usize =
    offset_of!(JSXElement, closing_element);

#[repr(transparent)]
#[derive(Clone, Copy, Debug)]
pub struct JSXElementWithoutOpeningElement<'a, 't>(
    pub(crate) *const JSXElement<'a>,
    pub(crate) PhantomData<&'t ()>,
);

impl<'a, 't> JSXElementWithoutOpeningElement<'a, 't> {
    #[inline]
    pub fn node_id(self) -> &'t Cell<NodeId> {
        unsafe { &*((self.0 as *const u8).add(OFFSET_JSX_ELEMENT_NODE_ID) as *const Cell<NodeId>) }
    }

    #[inline]
    pub fn span(self) -> &'t Span {
        unsafe { &*((self.0 as *const u8).add(OFFSET_JSX_ELEMENT_SPAN) as *const Span) }
    }

    #[inline]
    pub fn children(self) -> &'t ArenaVec<'a, JSXChild<'a>> {
        unsafe {
            &*((self.0 as *const u8).add(OFFSET_JSX_ELEMENT_CHILDREN)
                as *const ArenaVec<'a, JSXChild<'a>>)
        }
    }

    #[inline]
    pub fn closing_element(self) -> &'t Option<ArenaBox<'a, JSXClosingElement<'a>>> {
        unsafe {
            &*((self.0 as *const u8).add(OFFSET_JSX_ELEMENT_CLOSING_ELEMENT)
                as *const Option<ArenaBox<'a, JSXClosingElement<'a>>>)
        }
    }
}

impl<'a, 't> GetAddress for JSXElementWithoutOpeningElement<'a, 't> {
    #[inline]
    fn address(&self) -> Address {
        unsafe { Address::from_ptr(self.0) }
    }
}

#[repr(transparent)]
#[derive(Clone, Copy, Debug)]
pub struct JSXElementWithoutChildren<'a, 't>(
    pub(crate) *const JSXElement<'a>,
    pub(crate) PhantomData<&'t ()>,
);

impl<'a, 't> JSXElementWithoutChildren<'a, 't> {
    #[inline]
    pub fn node_id(self) -> &'t Cell<NodeId> {
        unsafe { &*((self.0 as *const u8).add(OFFSET_JSX_ELEMENT_NODE_ID) as *const Cell<NodeId>) }
    }

    #[inline]
    pub fn span(self) -> &'t Span {
        unsafe { &*((self.0 as *const u8).add(OFFSET_JSX_ELEMENT_SPAN) as *const Span) }
    }

    #[inline]
    pub fn opening_element(self) -> &'t ArenaBox<'a, JSXOpeningElement<'a>> {
        unsafe {
            &*((self.0 as *const u8).add(OFFSET_JSX_ELEMENT_OPENING_ELEMENT)
                as *const ArenaBox<'a, JSXOpeningElement<'a>>)
        }
    }

    #[inline]
    pub fn closing_element(self) -> &'t Option<ArenaBox<'a, JSXClosingElement<'a>>> {
        unsafe {
            &*((self.0 as *const u8).add(OFFSET_JSX_ELEMENT_CLOSING_ELEMENT)
                as *const Option<ArenaBox<'a, JSXClosingElement<'a>>>)
        }
    }
}

impl<'a, 't> GetAddress for JSXElementWithoutChildren<'a, 't> {
    #[inline]
    fn address(&self) -> Address {
        unsafe { Address::from_ptr(self.0) }
    }
}

#[repr(transparent)]
#[derive(Clone, Copy, Debug)]
pub struct JSXElementWithoutClosingElement<'a, 't>(
    pub(crate) *const JSXElement<'a>,
    pub(crate) PhantomData<&'t ()>,
);

impl<'a, 't> JSXElementWithoutClosingElement<'a, 't> {
    #[inline]
    pub fn node_id(self) -> &'t Cell<NodeId> {
        unsafe { &*((self.0 as *const u8).add(OFFSET_JSX_ELEMENT_NODE_ID) as *const Cell<NodeId>) }
    }

    #[inline]
    pub fn span(self) -> &'t Span {
        unsafe { &*((self.0 as *const u8).add(OFFSET_JSX_ELEMENT_SPAN) as *const Span) }
    }

    #[inline]
    pub fn opening_element(self) -> &'t ArenaBox<'a, JSXOpeningElement<'a>> {
        unsafe {
            &*((self.0 as *const u8).add(OFFSET_JSX_ELEMENT_OPENING_ELEMENT)
                as *const ArenaBox<'a, JSXOpeningElement<'a>>)
        }
    }

    #[inline]
    pub fn children(self) -> &'t ArenaVec<'a, JSXChild<'a>> {
        unsafe {
            &*((self.0 as *const u8).add(OFFSET_JSX_ELEMENT_CHILDREN)
                as *const ArenaVec<'a, JSXChild<'a>>)
        }
    }
}

impl<'a, 't> GetAddress for JSXElementWithoutClosingElement<'a, 't> {
    #[inline]
    fn address(&self) -> Address {
        unsafe { Address::from_ptr(self.0) }
    }
}

pub(crate) const OFFSET_JSX_OPENING_ELEMENT_NODE_ID: usize = offset_of!(JSXOpeningElement, node_id);
pub(crate) const OFFSET_JSX_OPENING_ELEMENT_SPAN: usize = offset_of!(JSXOpeningElement, span);
pub(crate) const OFFSET_JSX_OPENING_ELEMENT_NAME: usize = offset_of!(JSXOpeningElement, name);
pub(crate) const OFFSET_JSX_OPENING_ELEMENT_TYPE_ARGUMENTS: usize =
    offset_of!(JSXOpeningElement, type_arguments);
pub(crate) const OFFSET_JSX_OPENING_ELEMENT_ATTRIBUTES: usize =
    offset_of!(JSXOpeningElement, attributes);

#[repr(transparent)]
#[derive(Clone, Copy, Debug)]
pub struct JSXOpeningElementWithoutName<'a, 't>(
    pub(crate) *const JSXOpeningElement<'a>,
    pub(crate) PhantomData<&'t ()>,
);

impl<'a, 't> JSXOpeningElementWithoutName<'a, 't> {
    #[inline]
    pub fn node_id(self) -> &'t Cell<NodeId> {
        unsafe {
            &*((self.0 as *const u8).add(OFFSET_JSX_OPENING_ELEMENT_NODE_ID) as *const Cell<NodeId>)
        }
    }

    #[inline]
    pub fn span(self) -> &'t Span {
        unsafe { &*((self.0 as *const u8).add(OFFSET_JSX_OPENING_ELEMENT_SPAN) as *const Span) }
    }

    #[inline]
    pub fn type_arguments(self) -> &'t Option<ArenaBox<'a, TSTypeParameterInstantiation<'a>>> {
        unsafe {
            &*((self.0 as *const u8).add(OFFSET_JSX_OPENING_ELEMENT_TYPE_ARGUMENTS)
                as *const Option<ArenaBox<'a, TSTypeParameterInstantiation<'a>>>)
        }
    }

    #[inline]
    pub fn attributes(self) -> &'t ArenaVec<'a, JSXAttributeItem<'a>> {
        unsafe {
            &*((self.0 as *const u8).add(OFFSET_JSX_OPENING_ELEMENT_ATTRIBUTES)
                as *const ArenaVec<'a, JSXAttributeItem<'a>>)
        }
    }
}

impl<'a, 't> GetAddress for JSXOpeningElementWithoutName<'a, 't> {
    #[inline]
    fn address(&self) -> Address {
        unsafe { Address::from_ptr(self.0) }
    }
}

#[repr(transparent)]
#[derive(Clone, Copy, Debug)]
pub struct JSXOpeningElementWithoutAttributes<'a, 't>(
    pub(crate) *const JSXOpeningElement<'a>,
    pub(crate) PhantomData<&'t ()>,
);

impl<'a, 't> JSXOpeningElementWithoutAttributes<'a, 't> {
    #[inline]
    pub fn node_id(self) -> &'t Cell<NodeId> {
        unsafe {
            &*((self.0 as *const u8).add(OFFSET_JSX_OPENING_ELEMENT_NODE_ID) as *const Cell<NodeId>)
        }
    }

    #[inline]
    pub fn span(self) -> &'t Span {
        unsafe { &*((self.0 as *const u8).add(OFFSET_JSX_OPENING_ELEMENT_SPAN) as *const Span) }
    }

    #[inline]
    pub fn name(self) -> &'t JSXElementName<'a> {
        unsafe {
            &*((self.0 as *const u8).add(OFFSET_JSX_OPENING_ELEMENT_NAME)
                as *const JSXElementName<'a>)
        }
    }

    #[inline]
    pub fn type_arguments(self) -> &'t Option<ArenaBox<'a, TSTypeParameterInstantiation<'a>>> {
        unsafe {
            &*((self.0 as *const u8).add(OFFSET_JSX_OPENING_ELEMENT_TYPE_ARGUMENTS)
                as *const Option<ArenaBox<'a, TSTypeParameterInstantiation<'a>>>)
        }
    }
}

impl<'a, 't> GetAddress for JSXOpeningElementWithoutAttributes<'a, 't> {
    #[inline]
    fn address(&self) -> Address {
        unsafe { Address::from_ptr(self.0) }
    }
}

pub(crate) const OFFSET_JSX_CLOSING_ELEMENT_NODE_ID: usize = offset_of!(JSXClosingElement, node_id);
pub(crate) const OFFSET_JSX_CLOSING_ELEMENT_SPAN: usize = offset_of!(JSXClosingElement, span);
pub(crate) const OFFSET_JSX_CLOSING_ELEMENT_NAME: usize = offset_of!(JSXClosingElement, name);

#[repr(transparent)]
#[derive(Clone, Copy, Debug)]
pub struct JSXClosingElementWithoutName<'a, 't>(
    pub(crate) *const JSXClosingElement<'a>,
    pub(crate) PhantomData<&'t ()>,
);

impl<'a, 't> JSXClosingElementWithoutName<'a, 't> {
    #[inline]
    pub fn node_id(self) -> &'t Cell<NodeId> {
        unsafe {
            &*((self.0 as *const u8).add(OFFSET_JSX_CLOSING_ELEMENT_NODE_ID) as *const Cell<NodeId>)
        }
    }

    #[inline]
    pub fn span(self) -> &'t Span {
        unsafe { &*((self.0 as *const u8).add(OFFSET_JSX_CLOSING_ELEMENT_SPAN) as *const Span) }
    }
}

impl<'a, 't> GetAddress for JSXClosingElementWithoutName<'a, 't> {
    #[inline]
    fn address(&self) -> Address {
        unsafe { Address::from_ptr(self.0) }
    }
}

pub(crate) const OFFSET_JSX_FRAGMENT_NODE_ID: usize = offset_of!(JSXFragment, node_id);
pub(crate) const OFFSET_JSX_FRAGMENT_SPAN: usize = offset_of!(JSXFragment, span);
pub(crate) const OFFSET_JSX_FRAGMENT_OPENING_FRAGMENT: usize =
    offset_of!(JSXFragment, opening_fragment);
pub(crate) const OFFSET_JSX_FRAGMENT_CHILDREN: usize = offset_of!(JSXFragment, children);
pub(crate) const OFFSET_JSX_FRAGMENT_CLOSING_FRAGMENT: usize =
    offset_of!(JSXFragment, closing_fragment);

#[repr(transparent)]
#[derive(Clone, Copy, Debug)]
pub struct JSXFragmentWithoutOpeningFragment<'a, 't>(
    pub(crate) *const JSXFragment<'a>,
    pub(crate) PhantomData<&'t ()>,
);

impl<'a, 't> JSXFragmentWithoutOpeningFragment<'a, 't> {
    #[inline]
    pub fn node_id(self) -> &'t Cell<NodeId> {
        unsafe { &*((self.0 as *const u8).add(OFFSET_JSX_FRAGMENT_NODE_ID) as *const Cell<NodeId>) }
    }

    #[inline]
    pub fn span(self) -> &'t Span {
        unsafe { &*((self.0 as *const u8).add(OFFSET_JSX_FRAGMENT_SPAN) as *const Span) }
    }

    #[inline]
    pub fn children(self) -> &'t ArenaVec<'a, JSXChild<'a>> {
        unsafe {
            &*((self.0 as *const u8).add(OFFSET_JSX_FRAGMENT_CHILDREN)
                as *const ArenaVec<'a, JSXChild<'a>>)
        }
    }

    #[inline]
    pub fn closing_fragment(self) -> &'t JSXClosingFragment {
        unsafe {
            &*((self.0 as *const u8).add(OFFSET_JSX_FRAGMENT_CLOSING_FRAGMENT)
                as *const JSXClosingFragment)
        }
    }
}

impl<'a, 't> GetAddress for JSXFragmentWithoutOpeningFragment<'a, 't> {
    #[inline]
    fn address(&self) -> Address {
        unsafe { Address::from_ptr(self.0) }
    }
}

#[repr(transparent)]
#[derive(Clone, Copy, Debug)]
pub struct JSXFragmentWithoutChildren<'a, 't>(
    pub(crate) *const JSXFragment<'a>,
    pub(crate) PhantomData<&'t ()>,
);

impl<'a, 't> JSXFragmentWithoutChildren<'a, 't> {
    #[inline]
    pub fn node_id(self) -> &'t Cell<NodeId> {
        unsafe { &*((self.0 as *const u8).add(OFFSET_JSX_FRAGMENT_NODE_ID) as *const Cell<NodeId>) }
    }

    #[inline]
    pub fn span(self) -> &'t Span {
        unsafe { &*((self.0 as *const u8).add(OFFSET_JSX_FRAGMENT_SPAN) as *const Span) }
    }

    #[inline]
    pub fn opening_fragment(self) -> &'t JSXOpeningFragment {
        unsafe {
            &*((self.0 as *const u8).add(OFFSET_JSX_FRAGMENT_OPENING_FRAGMENT)
                as *const JSXOpeningFragment)
        }
    }

    #[inline]
    pub fn closing_fragment(self) -> &'t JSXClosingFragment {
        unsafe {
            &*((self.0 as *const u8).add(OFFSET_JSX_FRAGMENT_CLOSING_FRAGMENT)
                as *const JSXClosingFragment)
        }
    }
}

impl<'a, 't> GetAddress for JSXFragmentWithoutChildren<'a, 't> {
    #[inline]
    fn address(&self) -> Address {
        unsafe { Address::from_ptr(self.0) }
    }
}

#[repr(transparent)]
#[derive(Clone, Copy, Debug)]
pub struct JSXFragmentWithoutClosingFragment<'a, 't>(
    pub(crate) *const JSXFragment<'a>,
    pub(crate) PhantomData<&'t ()>,
);

impl<'a, 't> JSXFragmentWithoutClosingFragment<'a, 't> {
    #[inline]
    pub fn node_id(self) -> &'t Cell<NodeId> {
        unsafe { &*((self.0 as *const u8).add(OFFSET_JSX_FRAGMENT_NODE_ID) as *const Cell<NodeId>) }
    }

    #[inline]
    pub fn span(self) -> &'t Span {
        unsafe { &*((self.0 as *const u8).add(OFFSET_JSX_FRAGMENT_SPAN) as *const Span) }
    }

    #[inline]
    pub fn opening_fragment(self) -> &'t JSXOpeningFragment {
        unsafe {
            &*((self.0 as *const u8).add(OFFSET_JSX_FRAGMENT_OPENING_FRAGMENT)
                as *const JSXOpeningFragment)
        }
    }

    #[inline]
    pub fn children(self) -> &'t ArenaVec<'a, JSXChild<'a>> {
        unsafe {
            &*((self.0 as *const u8).add(OFFSET_JSX_FRAGMENT_CHILDREN)
                as *const ArenaVec<'a, JSXChild<'a>>)
        }
    }
}

impl<'a, 't> GetAddress for JSXFragmentWithoutClosingFragment<'a, 't> {
    #[inline]
    fn address(&self) -> Address {
        unsafe { Address::from_ptr(self.0) }
    }
}

pub(crate) const OFFSET_JSX_NAMESPACED_NAME_NODE_ID: usize = offset_of!(JSXNamespacedName, node_id);
pub(crate) const OFFSET_JSX_NAMESPACED_NAME_SPAN: usize = offset_of!(JSXNamespacedName, span);
pub(crate) const OFFSET_JSX_NAMESPACED_NAME_NAMESPACE: usize =
    offset_of!(JSXNamespacedName, namespace);
pub(crate) const OFFSET_JSX_NAMESPACED_NAME_NAME: usize = offset_of!(JSXNamespacedName, name);

#[repr(transparent)]
#[derive(Clone, Copy, Debug)]
pub struct JSXNamespacedNameWithoutNamespace<'a, 't>(
    pub(crate) *const JSXNamespacedName<'a>,
    pub(crate) PhantomData<&'t ()>,
);

impl<'a, 't> JSXNamespacedNameWithoutNamespace<'a, 't> {
    #[inline]
    pub fn node_id(self) -> &'t Cell<NodeId> {
        unsafe {
            &*((self.0 as *const u8).add(OFFSET_JSX_NAMESPACED_NAME_NODE_ID) as *const Cell<NodeId>)
        }
    }

    #[inline]
    pub fn span(self) -> &'t Span {
        unsafe { &*((self.0 as *const u8).add(OFFSET_JSX_NAMESPACED_NAME_SPAN) as *const Span) }
    }

    #[inline]
    pub fn name(self) -> &'t JSXIdentifier<'a> {
        unsafe {
            &*((self.0 as *const u8).add(OFFSET_JSX_NAMESPACED_NAME_NAME)
                as *const JSXIdentifier<'a>)
        }
    }
}

impl<'a, 't> GetAddress for JSXNamespacedNameWithoutNamespace<'a, 't> {
    #[inline]
    fn address(&self) -> Address {
        unsafe { Address::from_ptr(self.0) }
    }
}

#[repr(transparent)]
#[derive(Clone, Copy, Debug)]
pub struct JSXNamespacedNameWithoutName<'a, 't>(
    pub(crate) *const JSXNamespacedName<'a>,
    pub(crate) PhantomData<&'t ()>,
);

impl<'a, 't> JSXNamespacedNameWithoutName<'a, 't> {
    #[inline]
    pub fn node_id(self) -> &'t Cell<NodeId> {
        unsafe {
            &*((self.0 as *const u8).add(OFFSET_JSX_NAMESPACED_NAME_NODE_ID) as *const Cell<NodeId>)
        }
    }

    #[inline]
    pub fn span(self) -> &'t Span {
        unsafe { &*((self.0 as *const u8).add(OFFSET_JSX_NAMESPACED_NAME_SPAN) as *const Span) }
    }

    #[inline]
    pub fn namespace(self) -> &'t JSXIdentifier<'a> {
        unsafe {
            &*((self.0 as *const u8).add(OFFSET_JSX_NAMESPACED_NAME_NAMESPACE)
                as *const JSXIdentifier<'a>)
        }
    }
}

impl<'a, 't> GetAddress for JSXNamespacedNameWithoutName<'a, 't> {
    #[inline]
    fn address(&self) -> Address {
        unsafe { Address::from_ptr(self.0) }
    }
}

pub(crate) const OFFSET_JSX_MEMBER_EXPRESSION_NODE_ID: usize =
    offset_of!(JSXMemberExpression, node_id);
pub(crate) const OFFSET_JSX_MEMBER_EXPRESSION_SPAN: usize = offset_of!(JSXMemberExpression, span);
pub(crate) const OFFSET_JSX_MEMBER_EXPRESSION_OBJECT: usize =
    offset_of!(JSXMemberExpression, object);
pub(crate) const OFFSET_JSX_MEMBER_EXPRESSION_PROPERTY: usize =
    offset_of!(JSXMemberExpression, property);

#[repr(transparent)]
#[derive(Clone, Copy, Debug)]
pub struct JSXMemberExpressionWithoutObject<'a, 't>(
    pub(crate) *const JSXMemberExpression<'a>,
    pub(crate) PhantomData<&'t ()>,
);

impl<'a, 't> JSXMemberExpressionWithoutObject<'a, 't> {
    #[inline]
    pub fn node_id(self) -> &'t Cell<NodeId> {
        unsafe {
            &*((self.0 as *const u8).add(OFFSET_JSX_MEMBER_EXPRESSION_NODE_ID)
                as *const Cell<NodeId>)
        }
    }

    #[inline]
    pub fn span(self) -> &'t Span {
        unsafe { &*((self.0 as *const u8).add(OFFSET_JSX_MEMBER_EXPRESSION_SPAN) as *const Span) }
    }

    #[inline]
    pub fn property(self) -> &'t JSXIdentifier<'a> {
        unsafe {
            &*((self.0 as *const u8).add(OFFSET_JSX_MEMBER_EXPRESSION_PROPERTY)
                as *const JSXIdentifier<'a>)
        }
    }
}

impl<'a, 't> GetAddress for JSXMemberExpressionWithoutObject<'a, 't> {
    #[inline]
    fn address(&self) -> Address {
        unsafe { Address::from_ptr(self.0) }
    }
}

#[repr(transparent)]
#[derive(Clone, Copy, Debug)]
pub struct JSXMemberExpressionWithoutProperty<'a, 't>(
    pub(crate) *const JSXMemberExpression<'a>,
    pub(crate) PhantomData<&'t ()>,
);

impl<'a, 't> JSXMemberExpressionWithoutProperty<'a, 't> {
    #[inline]
    pub fn node_id(self) -> &'t Cell<NodeId> {
        unsafe {
            &*((self.0 as *const u8).add(OFFSET_JSX_MEMBER_EXPRESSION_NODE_ID)
                as *const Cell<NodeId>)
        }
    }

    #[inline]
    pub fn span(self) -> &'t Span {
        unsafe { &*((self.0 as *const u8).add(OFFSET_JSX_MEMBER_EXPRESSION_SPAN) as *const Span) }
    }

    #[inline]
    pub fn object(self) -> &'t JSXMemberExpressionObject<'a> {
        unsafe {
            &*((self.0 as *const u8).add(OFFSET_JSX_MEMBER_EXPRESSION_OBJECT)
                as *const JSXMemberExpressionObject<'a>)
        }
    }
}

impl<'a, 't> GetAddress for JSXMemberExpressionWithoutProperty<'a, 't> {
    #[inline]
    fn address(&self) -> Address {
        unsafe { Address::from_ptr(self.0) }
    }
}

pub(crate) const OFFSET_JSX_EXPRESSION_CONTAINER_NODE_ID: usize =
    offset_of!(JSXExpressionContainer, node_id);
pub(crate) const OFFSET_JSX_EXPRESSION_CONTAINER_SPAN: usize =
    offset_of!(JSXExpressionContainer, span);
pub(crate) const OFFSET_JSX_EXPRESSION_CONTAINER_EXPRESSION: usize =
    offset_of!(JSXExpressionContainer, expression);

#[repr(transparent)]
#[derive(Clone, Copy, Debug)]
pub struct JSXExpressionContainerWithoutExpression<'a, 't>(
    pub(crate) *const JSXExpressionContainer<'a>,
    pub(crate) PhantomData<&'t ()>,
);

impl<'a, 't> JSXExpressionContainerWithoutExpression<'a, 't> {
    #[inline]
    pub fn node_id(self) -> &'t Cell<NodeId> {
        unsafe {
            &*((self.0 as *const u8).add(OFFSET_JSX_EXPRESSION_CONTAINER_NODE_ID)
                as *const Cell<NodeId>)
        }
    }

    #[inline]
    pub fn span(self) -> &'t Span {
        unsafe {
            &*((self.0 as *const u8).add(OFFSET_JSX_EXPRESSION_CONTAINER_SPAN) as *const Span)
        }
    }
}

impl<'a, 't> GetAddress for JSXExpressionContainerWithoutExpression<'a, 't> {
    #[inline]
    fn address(&self) -> Address {
        unsafe { Address::from_ptr(self.0) }
    }
}

pub(crate) const OFFSET_JSX_ATTRIBUTE_NODE_ID: usize = offset_of!(JSXAttribute, node_id);
pub(crate) const OFFSET_JSX_ATTRIBUTE_SPAN: usize = offset_of!(JSXAttribute, span);
pub(crate) const OFFSET_JSX_ATTRIBUTE_NAME: usize = offset_of!(JSXAttribute, name);
pub(crate) const OFFSET_JSX_ATTRIBUTE_VALUE: usize = offset_of!(JSXAttribute, value);

#[repr(transparent)]
#[derive(Clone, Copy, Debug)]
pub struct JSXAttributeWithoutName<'a, 't>(
    pub(crate) *const JSXAttribute<'a>,
    pub(crate) PhantomData<&'t ()>,
);

impl<'a, 't> JSXAttributeWithoutName<'a, 't> {
    #[inline]
    pub fn node_id(self) -> &'t Cell<NodeId> {
        unsafe {
            &*((self.0 as *const u8).add(OFFSET_JSX_ATTRIBUTE_NODE_ID) as *const Cell<NodeId>)
        }
    }

    #[inline]
    pub fn span(self) -> &'t Span {
        unsafe { &*((self.0 as *const u8).add(OFFSET_JSX_ATTRIBUTE_SPAN) as *const Span) }
    }

    #[inline]
    pub fn value(self) -> &'t Option<JSXAttributeValue<'a>> {
        unsafe {
            &*((self.0 as *const u8).add(OFFSET_JSX_ATTRIBUTE_VALUE)
                as *const Option<JSXAttributeValue<'a>>)
        }
    }
}

impl<'a, 't> GetAddress for JSXAttributeWithoutName<'a, 't> {
    #[inline]
    fn address(&self) -> Address {
        unsafe { Address::from_ptr(self.0) }
    }
}

#[repr(transparent)]
#[derive(Clone, Copy, Debug)]
pub struct JSXAttributeWithoutValue<'a, 't>(
    pub(crate) *const JSXAttribute<'a>,
    pub(crate) PhantomData<&'t ()>,
);

impl<'a, 't> JSXAttributeWithoutValue<'a, 't> {
    #[inline]
    pub fn node_id(self) -> &'t Cell<NodeId> {
        unsafe {
            &*((self.0 as *const u8).add(OFFSET_JSX_ATTRIBUTE_NODE_ID) as *const Cell<NodeId>)
        }
    }

    #[inline]
    pub fn span(self) -> &'t Span {
        unsafe { &*((self.0 as *const u8).add(OFFSET_JSX_ATTRIBUTE_SPAN) as *const Span) }
    }

    #[inline]
    pub fn name(self) -> &'t JSXAttributeName<'a> {
        unsafe {
            &*((self.0 as *const u8).add(OFFSET_JSX_ATTRIBUTE_NAME) as *const JSXAttributeName<'a>)
        }
    }
}

impl<'a, 't> GetAddress for JSXAttributeWithoutValue<'a, 't> {
    #[inline]
    fn address(&self) -> Address {
        unsafe { Address::from_ptr(self.0) }
    }
}

pub(crate) const OFFSET_JSX_SPREAD_ATTRIBUTE_NODE_ID: usize =
    offset_of!(JSXSpreadAttribute, node_id);
pub(crate) const OFFSET_JSX_SPREAD_ATTRIBUTE_SPAN: usize = offset_of!(JSXSpreadAttribute, span);
pub(crate) const OFFSET_JSX_SPREAD_ATTRIBUTE_ARGUMENT: usize =
    offset_of!(JSXSpreadAttribute, argument);

#[repr(transparent)]
#[derive(Clone, Copy, Debug)]
pub struct JSXSpreadAttributeWithoutArgument<'a, 't>(
    pub(crate) *const JSXSpreadAttribute<'a>,
    pub(crate) PhantomData<&'t ()>,
);

impl<'a, 't> JSXSpreadAttributeWithoutArgument<'a, 't> {
    #[inline]
    pub fn node_id(self) -> &'t Cell<NodeId> {
        unsafe {
            &*((self.0 as *const u8).add(OFFSET_JSX_SPREAD_ATTRIBUTE_NODE_ID)
                as *const Cell<NodeId>)
        }
    }

    #[inline]
    pub fn span(self) -> &'t Span {
        unsafe { &*((self.0 as *const u8).add(OFFSET_JSX_SPREAD_ATTRIBUTE_SPAN) as *const Span) }
    }
}

impl<'a, 't> GetAddress for JSXSpreadAttributeWithoutArgument<'a, 't> {
    #[inline]
    fn address(&self) -> Address {
        unsafe { Address::from_ptr(self.0) }
    }
}

pub(crate) const OFFSET_JSX_SPREAD_CHILD_NODE_ID: usize = offset_of!(JSXSpreadChild, node_id);
pub(crate) const OFFSET_JSX_SPREAD_CHILD_SPAN: usize = offset_of!(JSXSpreadChild, span);
pub(crate) const OFFSET_JSX_SPREAD_CHILD_EXPRESSION: usize = offset_of!(JSXSpreadChild, expression);

#[repr(transparent)]
#[derive(Clone, Copy, Debug)]
pub struct JSXSpreadChildWithoutExpression<'a, 't>(
    pub(crate) *const JSXSpreadChild<'a>,
    pub(crate) PhantomData<&'t ()>,
);

impl<'a, 't> JSXSpreadChildWithoutExpression<'a, 't> {
    #[inline]
    pub fn node_id(self) -> &'t Cell<NodeId> {
        unsafe {
            &*((self.0 as *const u8).add(OFFSET_JSX_SPREAD_CHILD_NODE_ID) as *const Cell<NodeId>)
        }
    }

    #[inline]
    pub fn span(self) -> &'t Span {
        unsafe { &*((self.0 as *const u8).add(OFFSET_JSX_SPREAD_CHILD_SPAN) as *const Span) }
    }
}

impl<'a, 't> GetAddress for JSXSpreadChildWithoutExpression<'a, 't> {
    #[inline]
    fn address(&self) -> Address {
        unsafe { Address::from_ptr(self.0) }
    }
}

pub(crate) const OFFSET_DECORATOR_NODE_ID: usize = offset_of!(Decorator, node_id);
pub(crate) const OFFSET_DECORATOR_SPAN: usize = offset_of!(Decorator, span);
pub(crate) const OFFSET_DECORATOR_EXPRESSION: usize = offset_of!(Decorator, expression);

#[repr(transparent)]
#[derive(Clone, Copy, Debug)]
pub struct DecoratorWithoutExpression<'a, 't>(
    pub(crate) *const Decorator<'a>,
    pub(crate) PhantomData<&'t ()>,
);

impl<'a, 't> DecoratorWithoutExpression<'a, 't> {
    #[inline]
    pub fn node_id(self) -> &'t Cell<NodeId> {
        unsafe { &*((self.0 as *const u8).add(OFFSET_DECORATOR_NODE_ID) as *const Cell<NodeId>) }
    }

    #[inline]
    pub fn span(self) -> &'t Span {
        unsafe { &*((self.0 as *const u8).add(OFFSET_DECORATOR_SPAN) as *const Span) }
    }
}

impl<'a, 't> GetAddress for DecoratorWithoutExpression<'a, 't> {
    #[inline]
    fn address(&self) -> Address {
        unsafe { Address::from_ptr(self.0) }
    }
}
