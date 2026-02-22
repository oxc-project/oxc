/// Core HIR (High-level Intermediate Representation) types.
///
/// Port of `HIR/HIR.ts` from the React Compiler.
///
/// The HIR is the central data model of the compiler. It represents the program
/// as a control-flow graph of basic blocks, where each block contains a sequence
/// of instructions followed by a terminal node.
///
/// Pipeline: AST → (lowering) → HIR → (analysis) → Reactive Scopes → (codegen) → AST
use oxc_syntax::operator::{BinaryOperator, LogicalOperator, UnaryOperator, UpdateOperator};
use rustc_hash::{FxHashMap, FxHashSet};

use crate::compiler_error::SourceLocation;

use super::types::{PropertyLiteral, Type};

// =====================================================================================
// ID Types (opaque newtypes to prevent accidental misuse)
// =====================================================================================

/// Unique identifier for a basic block in the HIR control-flow graph.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct BlockId(pub u32);

/// Unique identifier for a reactive scope.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ScopeId(pub u32);

/// Unique identifier for an SSA identifier instance.
///
/// After EnterSSA, `id` uniquely identifies an SSA instance of a variable.
/// Before EnterSSA, `id` matches `declaration_id`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct IdentifierId(pub u32);

/// Unique identifier for a variable declaration in the original program.
///
/// If a value is reassigned, each reassigned value will have a distinct
/// `IdentifierId` (after SSA), but they share the same `DeclarationId`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct DeclarationId(pub u32);

/// Unique identifier for an instruction in the HIR.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct InstructionId(pub u32);

impl InstructionId {
    pub const ZERO: Self = Self(0);
}

impl std::fmt::Display for BlockId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "bb{}", self.0)
    }
}

impl std::fmt::Display for InstructionId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "${}", self.0)
    }
}

impl std::fmt::Display for IdentifierId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "#{}", self.0)
    }
}

// =====================================================================================
// Block & HIR types
// =====================================================================================

/// The kind of a basic block.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BlockKind {
    Block,
    Value,
    Loop,
    Sequence,
    Catch,
}

impl BlockKind {
    /// Returns true for "block" and "catch" block kinds which correspond to statements.
    pub fn is_statement(self) -> bool {
        matches!(self, BlockKind::Block | BlockKind::Catch)
    }

    /// Returns true for "value", "loop", and "sequence" block kinds which correspond to expressions.
    pub fn is_expression(self) -> bool {
        !self.is_statement()
    }
}

/// A basic block in the HIR control-flow graph.
#[derive(Debug, Clone)]
pub struct BasicBlock {
    pub kind: BlockKind,
    pub id: BlockId,
    pub instructions: Vec<Instruction>,
    pub terminal: Terminal,
    pub preds: FxHashSet<BlockId>,
    pub phis: Vec<Phi>,
}

/// The HIR control-flow graph: an entry block and a map of blocks.
///
/// Blocks are stored in reverse postorder: barring cycles, predecessors
/// appear before successors.
#[derive(Debug, Clone)]
pub struct Hir {
    pub entry: BlockId,
    pub blocks: FxHashMap<BlockId, BasicBlock>,
}

// =====================================================================================
// Phi nodes
// =====================================================================================

/// A phi node, representing the merge of values from different control-flow paths.
#[derive(Debug, Clone)]
pub struct Phi {
    pub id: u32,
    pub place: Place,
    pub operands: FxHashMap<BlockId, Place>,
}

// =====================================================================================
// Terminal nodes
// =====================================================================================

/// Terminal nodes represent statements that affect control flow.
#[derive(Debug, Clone)]
pub enum Terminal {
    Unsupported(UnsupportedTerminal),
    Unreachable(UnreachableTerminal),
    Throw(ThrowTerminal),
    Return(ReturnTerminal),
    Goto(GotoTerminal),
    If(IfTerminal),
    Branch(BranchTerminal),
    Switch(SwitchTerminal),
    For(ForTerminal),
    ForOf(ForOfTerminal),
    ForIn(ForInTerminal),
    DoWhile(DoWhileTerminal),
    While(WhileTerminal),
    Logical(LogicalTerminal),
    Ternary(TernaryTerminal),
    Optional(OptionalTerminal),
    Label(LabelTerminal),
    Sequence(SequenceTerminal),
    MaybeThrow(MaybeThrowTerminal),
    Try(TryTerminal),
    Scope(ReactiveScopeTerminal),
    PrunedScope(PrunedScopeTerminal),
}

impl Terminal {
    pub fn id(&self) -> InstructionId {
        match self {
            Terminal::Unsupported(t) => t.id,
            Terminal::Unreachable(t) => t.id,
            Terminal::Throw(t) => t.id,
            Terminal::Return(t) => t.id,
            Terminal::Goto(t) => t.id,
            Terminal::If(t) => t.id,
            Terminal::Branch(t) => t.id,
            Terminal::Switch(t) => t.id,
            Terminal::For(t) => t.id,
            Terminal::ForOf(t) => t.id,
            Terminal::ForIn(t) => t.id,
            Terminal::DoWhile(t) => t.id,
            Terminal::While(t) => t.id,
            Terminal::Logical(t) => t.id,
            Terminal::Ternary(t) => t.id,
            Terminal::Optional(t) => t.id,
            Terminal::Label(t) => t.id,
            Terminal::Sequence(t) => t.id,
            Terminal::MaybeThrow(t) => t.id,
            Terminal::Try(t) => t.id,
            Terminal::Scope(t) => t.id,
            Terminal::PrunedScope(t) => t.id,
        }
    }

    pub fn loc(&self) -> SourceLocation {
        match self {
            Terminal::Unsupported(t) => t.loc,
            Terminal::Unreachable(t) => t.loc,
            Terminal::Throw(t) => t.loc,
            Terminal::Return(t) => t.loc,
            Terminal::Goto(t) => t.loc,
            Terminal::If(t) => t.loc,
            Terminal::Branch(t) => t.loc,
            Terminal::Switch(t) => t.loc,
            Terminal::For(t) => t.loc,
            Terminal::ForOf(t) => t.loc,
            Terminal::ForIn(t) => t.loc,
            Terminal::DoWhile(t) => t.loc,
            Terminal::While(t) => t.loc,
            Terminal::Logical(t) => t.loc,
            Terminal::Ternary(t) => t.loc,
            Terminal::Optional(t) => t.loc,
            Terminal::Label(t) => t.loc,
            Terminal::Sequence(t) => t.loc,
            Terminal::MaybeThrow(t) => t.loc,
            Terminal::Try(t) => t.loc,
            Terminal::Scope(t) => t.loc,
            Terminal::PrunedScope(t) => t.loc,
        }
    }

    /// Returns the fallthrough block if this terminal has one.
    pub fn fallthrough(&self) -> Option<BlockId> {
        match self {
            Terminal::If(t) => Some(t.fallthrough),
            Terminal::Branch(t) => Some(t.fallthrough),
            Terminal::Switch(t) => Some(t.fallthrough),
            Terminal::For(t) => Some(t.fallthrough),
            Terminal::ForOf(t) => Some(t.fallthrough),
            Terminal::ForIn(t) => Some(t.fallthrough),
            Terminal::DoWhile(t) => Some(t.fallthrough),
            Terminal::While(t) => Some(t.fallthrough),
            Terminal::Logical(t) => Some(t.fallthrough),
            Terminal::Ternary(t) => Some(t.fallthrough),
            Terminal::Optional(t) => Some(t.fallthrough),
            Terminal::Label(t) => Some(t.fallthrough),
            Terminal::Sequence(t) => Some(t.fallthrough),
            Terminal::Try(t) => Some(t.fallthrough),
            Terminal::Scope(t) => Some(t.fallthrough),
            Terminal::PrunedScope(t) => Some(t.fallthrough),
            Terminal::Unsupported(_)
            | Terminal::Unreachable(_)
            | Terminal::Throw(_)
            | Terminal::Return(_)
            | Terminal::Goto(_)
            | Terminal::MaybeThrow(_) => None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct UnsupportedTerminal {
    pub id: InstructionId,
    pub loc: SourceLocation,
}

#[derive(Debug, Clone)]
pub struct UnreachableTerminal {
    pub id: InstructionId,
    pub loc: SourceLocation,
}

#[derive(Debug, Clone)]
pub struct ThrowTerminal {
    pub id: InstructionId,
    pub value: Place,
    pub loc: SourceLocation,
}

/// How a return statement was written in source.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReturnVariant {
    /// `() => { ... }` or `function() { ... }` (no explicit return)
    Void,
    /// `() => foo` (arrow function implicit return)
    Implicit,
    /// `() => { return ... }` or `function() { return ... }`
    Explicit,
}

#[derive(Debug, Clone)]
pub struct ReturnTerminal {
    pub id: InstructionId,
    pub value: Place,
    pub return_variant: ReturnVariant,
    pub loc: SourceLocation,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GotoVariant {
    Break,
    Continue,
    Try,
}

#[derive(Debug, Clone)]
pub struct GotoTerminal {
    pub id: InstructionId,
    pub block: BlockId,
    pub variant: GotoVariant,
    pub loc: SourceLocation,
}

#[derive(Debug, Clone)]
pub struct IfTerminal {
    pub id: InstructionId,
    pub test: Place,
    pub consequent: BlockId,
    pub alternate: BlockId,
    pub fallthrough: BlockId,
    pub loc: SourceLocation,
}

#[derive(Debug, Clone)]
pub struct BranchTerminal {
    pub id: InstructionId,
    pub test: Place,
    pub consequent: BlockId,
    pub alternate: BlockId,
    pub fallthrough: BlockId,
    pub loc: SourceLocation,
}

#[derive(Debug, Clone)]
pub struct Case {
    pub test: Option<Place>,
    pub block: BlockId,
}

#[derive(Debug, Clone)]
pub struct SwitchTerminal {
    pub id: InstructionId,
    pub test: Place,
    pub cases: Vec<Case>,
    pub fallthrough: BlockId,
    pub loc: SourceLocation,
}

#[derive(Debug, Clone)]
pub struct ForTerminal {
    pub id: InstructionId,
    pub init: BlockId,
    pub test: BlockId,
    pub update: Option<BlockId>,
    pub r#loop: BlockId,
    pub fallthrough: BlockId,
    pub loc: SourceLocation,
}

#[derive(Debug, Clone)]
pub struct ForOfTerminal {
    pub id: InstructionId,
    pub init: BlockId,
    pub test: BlockId,
    pub r#loop: BlockId,
    pub fallthrough: BlockId,
    pub loc: SourceLocation,
}

#[derive(Debug, Clone)]
pub struct ForInTerminal {
    pub id: InstructionId,
    pub init: BlockId,
    pub r#loop: BlockId,
    pub fallthrough: BlockId,
    pub loc: SourceLocation,
}

#[derive(Debug, Clone)]
pub struct DoWhileTerminal {
    pub id: InstructionId,
    pub r#loop: BlockId,
    pub test: BlockId,
    pub fallthrough: BlockId,
    pub loc: SourceLocation,
}

#[derive(Debug, Clone)]
pub struct WhileTerminal {
    pub id: InstructionId,
    pub test: BlockId,
    pub r#loop: BlockId,
    pub fallthrough: BlockId,
    pub loc: SourceLocation,
}

#[derive(Debug, Clone)]
pub struct LogicalTerminal {
    pub id: InstructionId,
    pub operator: LogicalOperator,
    pub test: BlockId,
    pub fallthrough: BlockId,
    pub loc: SourceLocation,
}

#[derive(Debug, Clone)]
pub struct TernaryTerminal {
    pub id: InstructionId,
    pub test: BlockId,
    pub fallthrough: BlockId,
    pub loc: SourceLocation,
}

#[derive(Debug, Clone)]
pub struct OptionalTerminal {
    pub id: InstructionId,
    pub optional: bool,
    pub test: BlockId,
    pub fallthrough: BlockId,
    pub loc: SourceLocation,
}

#[derive(Debug, Clone)]
pub struct LabelTerminal {
    pub id: InstructionId,
    pub block: BlockId,
    pub fallthrough: BlockId,
    pub loc: SourceLocation,
}

#[derive(Debug, Clone)]
pub struct SequenceTerminal {
    pub id: InstructionId,
    pub block: BlockId,
    pub fallthrough: BlockId,
    pub loc: SourceLocation,
}

#[derive(Debug, Clone)]
pub struct MaybeThrowTerminal {
    pub id: InstructionId,
    pub continuation: BlockId,
    pub handler: Option<BlockId>,
    pub loc: SourceLocation,
}

#[derive(Debug, Clone)]
pub struct TryTerminal {
    pub id: InstructionId,
    pub block: BlockId,
    pub handler_binding: Option<Place>,
    pub handler: BlockId,
    pub fallthrough: BlockId,
    pub loc: SourceLocation,
}

#[derive(Debug, Clone)]
pub struct ReactiveScopeTerminal {
    pub id: InstructionId,
    pub block: BlockId,
    pub scope: ReactiveScope,
    pub fallthrough: BlockId,
    pub loc: SourceLocation,
}

#[derive(Debug, Clone)]
pub struct PrunedScopeTerminal {
    pub id: InstructionId,
    pub block: BlockId,
    pub scope: ReactiveScope,
    pub fallthrough: BlockId,
    pub loc: SourceLocation,
}

// =====================================================================================
// Instructions
// =====================================================================================

/// An instruction in the HIR. Instructions represent expressions with all nesting flattened.
#[derive(Debug, Clone)]
pub struct Instruction {
    pub id: InstructionId,
    pub lvalue: Place,
    pub value: InstructionValue,
    pub effects: Option<Vec<crate::inference::aliasing_effects::AliasingEffect>>,
    pub loc: SourceLocation,
}

/// The kind of an instruction's lvalue binding.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InstructionKind {
    Const,
    Let,
    Reassign,
    Catch,
    HoistedConst,
    HoistedLet,
    HoistedFunction,
    Function,
}

impl InstructionKind {
    /// Convert a hoisted kind to its non-hoisted equivalent, or return `None`.
    pub fn convert_hoisted(self) -> Option<InstructionKind> {
        match self {
            InstructionKind::HoistedLet => Some(InstructionKind::Let),
            InstructionKind::HoistedConst => Some(InstructionKind::Const),
            InstructionKind::HoistedFunction => Some(InstructionKind::Function),
            InstructionKind::Let
            | InstructionKind::Const
            | InstructionKind::Function
            | InstructionKind::Reassign
            | InstructionKind::Catch => None,
        }
    }
}

/// An lvalue (left-hand side of assignment).
#[derive(Debug, Clone)]
pub struct LValue {
    pub place: Place,
    pub kind: InstructionKind,
}

/// An lvalue that is a destructuring pattern.
#[derive(Debug, Clone)]
pub struct LValuePattern {
    pub pattern: Pattern,
    pub kind: InstructionKind,
}

// =====================================================================================
// Patterns
// =====================================================================================

/// A destructuring pattern.
#[derive(Debug, Clone)]
pub enum Pattern {
    Array(ArrayPattern),
    Object(ObjectPattern),
}

/// A hole in a pattern (e.g., `[, x]`).
#[derive(Debug, Clone)]
pub struct Hole;

/// A spread pattern (e.g., `...rest`).
#[derive(Debug, Clone)]
pub struct SpreadPattern {
    pub place: Place,
}

/// An array destructuring pattern.
#[derive(Debug, Clone)]
pub struct ArrayPattern {
    pub items: Vec<ArrayPatternElement>,
    pub loc: SourceLocation,
}

/// An element in an array pattern.
#[derive(Debug, Clone)]
pub enum ArrayPatternElement {
    Place(Place),
    Spread(SpreadPattern),
    Hole,
}

/// An object destructuring pattern.
#[derive(Debug, Clone)]
pub struct ObjectPattern {
    pub properties: Vec<ObjectPatternProperty>,
    pub loc: SourceLocation,
}

/// A property or spread in an object pattern.
#[derive(Debug, Clone)]
pub enum ObjectPatternProperty {
    Property(ObjectProperty),
    Spread(SpreadPattern),
}

// =====================================================================================
// Object property keys
// =====================================================================================

/// The key of an object property.
#[derive(Debug, Clone)]
pub enum ObjectPropertyKey {
    String(String),
    Identifier(String),
    Computed(Place),
    Number(f64),
}

/// An object property definition.
#[derive(Debug, Clone)]
pub struct ObjectProperty {
    pub key: ObjectPropertyKey,
    pub property_type: ObjectPropertyType,
    pub place: Place,
}

/// Whether the property is a plain property or a method.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ObjectPropertyType {
    Property,
    Method,
}

// =====================================================================================
// Instruction values
// =====================================================================================

/// A lowered function (HIR function body).
#[derive(Debug, Clone)]
pub struct LoweredFunction {
    pub func: Box<HIRFunction>,
}

/// An object method value.
#[derive(Debug, Clone)]
pub struct ObjectMethodValue {
    pub loc: SourceLocation,
    pub lowered_func: LoweredFunction,
}

/// The value of an instruction. Values are not recursive — complex values
/// are defined by previous instructions storing to temporaries.
#[derive(Debug, Clone)]
pub enum InstructionValue {
    // Variable operations
    LoadLocal(LoadLocal),
    LoadContext(LoadContext),
    DeclareLocal(DeclareLocal),
    DeclareContext(DeclareContext),
    StoreLocal(StoreLocal),
    StoreContext(StoreContext),
    Destructure(Destructure),

    // Literals
    Primitive(PrimitiveValue),
    JsxText(JsxTextValue),
    RegExpLiteral(RegExpLiteral),
    TemplateLiteral(TemplateLiteral),
    TaggedTemplateExpression(TaggedTemplateExpression),

    // Expressions
    BinaryExpression(BinaryExpressionValue),
    UnaryExpression(UnaryExpressionValue),
    TypeCastExpression(TypeCastExpression),

    // Calls
    CallExpression(CallExpression),
    MethodCall(MethodCall),
    NewExpression(NewExpression),

    // Object/array construction
    ObjectExpression(ObjectExpression),
    ObjectMethod(ObjectMethodValue),
    ArrayExpression(ArrayExpression),

    // JSX
    JsxExpression(JsxExpression),
    JsxFragment(JsxFragment),

    // Property operations
    PropertyLoad(PropertyLoad),
    PropertyStore(PropertyStore),
    PropertyDelete(PropertyDelete),
    ComputedLoad(ComputedLoad),
    ComputedStore(ComputedStore),
    ComputedDelete(ComputedDelete),

    // Globals
    LoadGlobal(LoadGlobal),
    StoreGlobal(StoreGlobal),

    // Function expression
    FunctionExpression(FunctionExpressionValue),

    // Iteration
    GetIterator(GetIterator),
    IteratorNext(IteratorNext),
    NextPropertyOf(NextPropertyOf),

    // Update expressions
    PrefixUpdate(PrefixUpdate),
    PostfixUpdate(PostfixUpdate),

    // Await
    Await(AwaitValue),

    // Memoization markers
    StartMemoize(StartMemoize),
    FinishMemoize(FinishMemoize),

    // Debugger
    Debugger(DebuggerValue),

    // Meta
    MetaProperty(MetaProperty),

    // Catch-all for unsupported nodes
    UnsupportedNode(UnsupportedNode),
}

impl InstructionValue {
    pub fn loc(&self) -> SourceLocation {
        match self {
            InstructionValue::LoadLocal(v) => v.loc,
            InstructionValue::LoadContext(v) => v.loc,
            InstructionValue::DeclareLocal(v) => v.loc,
            InstructionValue::DeclareContext(v) => v.loc,
            InstructionValue::StoreLocal(v) => v.loc,
            InstructionValue::StoreContext(v) => v.loc,
            InstructionValue::Destructure(v) => v.loc,
            InstructionValue::Primitive(v) => v.loc,
            InstructionValue::JsxText(v) => v.loc,
            InstructionValue::RegExpLiteral(v) => v.loc,
            InstructionValue::TemplateLiteral(v) => v.loc,
            InstructionValue::TaggedTemplateExpression(v) => v.loc,
            InstructionValue::BinaryExpression(v) => v.loc,
            InstructionValue::UnaryExpression(v) => v.loc,
            InstructionValue::TypeCastExpression(v) => v.loc,
            InstructionValue::CallExpression(v) => v.loc,
            InstructionValue::MethodCall(v) => v.loc,
            InstructionValue::NewExpression(v) => v.loc,
            InstructionValue::ObjectExpression(v) => v.loc,
            InstructionValue::ObjectMethod(v) => v.loc,
            InstructionValue::ArrayExpression(v) => v.loc,
            InstructionValue::JsxExpression(v) => v.loc,
            InstructionValue::JsxFragment(v) => v.loc,
            InstructionValue::PropertyLoad(v) => v.loc,
            InstructionValue::PropertyStore(v) => v.loc,
            InstructionValue::PropertyDelete(v) => v.loc,
            InstructionValue::ComputedLoad(v) => v.loc,
            InstructionValue::ComputedStore(v) => v.loc,
            InstructionValue::ComputedDelete(v) => v.loc,
            InstructionValue::LoadGlobal(v) => v.loc,
            InstructionValue::StoreGlobal(v) => v.loc,
            InstructionValue::FunctionExpression(v) => v.loc,
            InstructionValue::GetIterator(v) => v.loc,
            InstructionValue::IteratorNext(v) => v.loc,
            InstructionValue::NextPropertyOf(v) => v.loc,
            InstructionValue::PrefixUpdate(v) => v.loc,
            InstructionValue::PostfixUpdate(v) => v.loc,
            InstructionValue::Await(v) => v.loc,
            InstructionValue::StartMemoize(v) => v.loc,
            InstructionValue::FinishMemoize(v) => v.loc,
            InstructionValue::Debugger(v) => v.loc,
            InstructionValue::MetaProperty(v) => v.loc,
            InstructionValue::UnsupportedNode(v) => v.loc,
        }
    }
}

// Individual instruction value types:

#[derive(Debug, Clone)]
pub struct LoadLocal {
    pub place: Place,
    pub loc: SourceLocation,
}

#[derive(Debug, Clone)]
pub struct LoadContext {
    pub place: Place,
    pub loc: SourceLocation,
}

#[derive(Debug, Clone)]
pub struct DeclareLocal {
    pub lvalue: LValue,
    pub loc: SourceLocation,
}

#[derive(Debug, Clone)]
pub struct DeclareContext {
    pub lvalue_kind: InstructionKind,
    pub lvalue_place: Place,
    pub loc: SourceLocation,
}

#[derive(Debug, Clone)]
pub struct StoreLocal {
    pub lvalue: LValue,
    pub value: Place,
    pub loc: SourceLocation,
}

#[derive(Debug, Clone)]
pub struct StoreContext {
    pub lvalue_kind: InstructionKind,
    pub lvalue_place: Place,
    pub value: Place,
    pub loc: SourceLocation,
}

#[derive(Debug, Clone)]
pub struct Destructure {
    pub lvalue: LValuePattern,
    pub value: Place,
    pub loc: SourceLocation,
}

/// A JavaScript primitive value.
#[derive(Debug, Clone)]
pub enum PrimitiveValueKind {
    Number(f64),
    Boolean(bool),
    String(String),
    Null,
    Undefined,
}

#[derive(Debug, Clone)]
pub struct PrimitiveValue {
    pub value: PrimitiveValueKind,
    pub loc: SourceLocation,
}

#[derive(Debug, Clone)]
pub struct JsxTextValue {
    pub value: String,
    pub loc: SourceLocation,
}

#[derive(Debug, Clone)]
pub struct RegExpLiteral {
    pub pattern: String,
    pub flags: String,
    pub loc: SourceLocation,
}

#[derive(Debug, Clone)]
pub struct TemplateLiteralQuasi {
    pub raw: String,
    pub cooked: Option<String>,
}

#[derive(Debug, Clone)]
pub struct TemplateLiteral {
    pub subexprs: Vec<Place>,
    pub quasis: Vec<TemplateLiteralQuasi>,
    pub loc: SourceLocation,
}

#[derive(Debug, Clone)]
pub struct TaggedTemplateExpression {
    pub tag: Place,
    pub value: TemplateLiteralQuasi,
    pub loc: SourceLocation,
}

#[derive(Debug, Clone)]
pub struct BinaryExpressionValue {
    pub operator: BinaryOperator,
    pub left: Place,
    pub right: Place,
    pub loc: SourceLocation,
}

#[derive(Debug, Clone)]
pub struct UnaryExpressionValue {
    pub operator: UnaryOperator,
    pub value: Place,
    pub loc: SourceLocation,
}

/// The kind of a TypeCast annotation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TypeAnnotationKind {
    Cast,
    As,
    Satisfies,
}

#[derive(Debug, Clone)]
pub struct TypeCastExpression {
    pub value: Place,
    pub type_: Type,
    pub annotation_kind: TypeAnnotationKind,
    pub loc: SourceLocation,
}

/// An element in a call argument list.
#[derive(Debug, Clone)]
pub enum CallArg {
    Place(Place),
    Spread(SpreadPattern),
}

#[derive(Debug, Clone)]
pub struct CallExpression {
    pub callee: Place,
    pub args: Vec<CallArg>,
    pub loc: SourceLocation,
}

#[derive(Debug, Clone)]
pub struct MethodCall {
    pub receiver: Place,
    pub property: Place,
    pub args: Vec<CallArg>,
    pub loc: SourceLocation,
}

#[derive(Debug, Clone)]
pub struct NewExpression {
    pub callee: Place,
    pub args: Vec<CallArg>,
    pub loc: SourceLocation,
}

#[derive(Debug, Clone)]
pub struct ObjectExpression {
    pub properties: Vec<ObjectPatternProperty>,
    pub loc: SourceLocation,
}

/// An element in an array expression.
#[derive(Debug, Clone)]
pub enum ArrayExpressionElement {
    Place(Place),
    Spread(SpreadPattern),
    Hole,
}

#[derive(Debug, Clone)]
pub struct ArrayExpression {
    pub elements: Vec<ArrayExpressionElement>,
    pub loc: SourceLocation,
}

/// A JSX tag — either a component/element place or a built-in HTML tag.
#[derive(Debug, Clone)]
pub enum JsxTag {
    Place(Place),
    BuiltIn(BuiltinTag),
}

/// A built-in HTML tag name (e.g., "div", "span").
#[derive(Debug, Clone)]
pub struct BuiltinTag {
    pub name: String,
    pub loc: SourceLocation,
}

/// A JSX attribute.
#[derive(Debug, Clone)]
pub enum JsxAttribute {
    Spread { argument: Place },
    Attribute { name: String, place: Place },
}

#[derive(Debug, Clone)]
pub struct JsxExpression {
    pub tag: JsxTag,
    pub props: Vec<JsxAttribute>,
    /// `None` means no children at all (self-closing).
    pub children: Option<Vec<Place>>,
    pub loc: SourceLocation,
    pub opening_loc: SourceLocation,
    pub closing_loc: SourceLocation,
}

#[derive(Debug, Clone)]
pub struct JsxFragment {
    pub children: Vec<Place>,
    pub loc: SourceLocation,
}

#[derive(Debug, Clone)]
pub struct PropertyLoad {
    pub object: Place,
    pub property: PropertyLiteral,
    pub loc: SourceLocation,
}

#[derive(Debug, Clone)]
pub struct PropertyStore {
    pub object: Place,
    pub property: PropertyLiteral,
    pub value: Place,
    pub loc: SourceLocation,
}

#[derive(Debug, Clone)]
pub struct PropertyDelete {
    pub object: Place,
    pub property: PropertyLiteral,
    pub loc: SourceLocation,
}

#[derive(Debug, Clone)]
pub struct ComputedLoad {
    pub object: Place,
    pub property: Place,
    pub loc: SourceLocation,
}

#[derive(Debug, Clone)]
pub struct ComputedStore {
    pub object: Place,
    pub property: Place,
    pub value: Place,
    pub loc: SourceLocation,
}

#[derive(Debug, Clone)]
pub struct ComputedDelete {
    pub object: Place,
    pub property: Place,
    pub loc: SourceLocation,
}

/// A non-local (imported or global) binding.
#[derive(Debug, Clone)]
pub enum NonLocalBinding {
    /// `import Foo from 'foo'`
    ImportDefault { name: String, module: String },
    /// `import * as Foo from 'foo'`
    ImportNamespace { name: String, module: String },
    /// `import {bar as baz} from 'foo'`
    ImportSpecifier { name: String, module: String, imported: String },
    /// Variable declared in module scope but outside current component/hook
    ModuleLocal { name: String },
    /// An unresolved binding
    Global { name: String },
}

impl NonLocalBinding {
    /// Get the local name of this binding.
    pub fn name(&self) -> &str {
        match self {
            NonLocalBinding::ImportDefault { name, .. }
            | NonLocalBinding::ImportNamespace { name, .. }
            | NonLocalBinding::ImportSpecifier { name, .. }
            | NonLocalBinding::ModuleLocal { name }
            | NonLocalBinding::Global { name } => name,
        }
    }
}

#[derive(Debug, Clone)]
pub struct LoadGlobal {
    pub binding: NonLocalBinding,
    pub loc: SourceLocation,
}

#[derive(Debug, Clone)]
pub struct StoreGlobal {
    pub name: String,
    pub value: Place,
    pub loc: SourceLocation,
}

/// The type of a function expression in the source.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FunctionExpressionType {
    ArrowFunctionExpression,
    FunctionExpression,
    FunctionDeclaration,
}

#[derive(Debug, Clone)]
pub struct FunctionExpressionValue {
    pub name: Option<ValidIdentifierName>,
    pub name_hint: Option<String>,
    pub lowered_func: LoweredFunction,
    pub expression_type: FunctionExpressionType,
    pub loc: SourceLocation,
}

#[derive(Debug, Clone)]
pub struct GetIterator {
    pub collection: Place,
    pub loc: SourceLocation,
}

#[derive(Debug, Clone)]
pub struct IteratorNext {
    pub iterator: Place,
    pub collection: Place,
    pub loc: SourceLocation,
}

#[derive(Debug, Clone)]
pub struct NextPropertyOf {
    pub value: Place,
    pub loc: SourceLocation,
}

#[derive(Debug, Clone)]
pub struct PrefixUpdate {
    pub lvalue: Place,
    pub operation: UpdateOperator,
    pub value: Place,
    pub loc: SourceLocation,
}

#[derive(Debug, Clone)]
pub struct PostfixUpdate {
    pub lvalue: Place,
    pub operation: UpdateOperator,
    pub value: Place,
    pub loc: SourceLocation,
}

#[derive(Debug, Clone)]
pub struct AwaitValue {
    pub value: Place,
    pub loc: SourceLocation,
}

#[derive(Debug, Clone)]
pub struct MetaProperty {
    pub meta: String,
    pub property: String,
    pub loc: SourceLocation,
}

#[derive(Debug, Clone)]
pub struct DebuggerValue {
    pub loc: SourceLocation,
}

#[derive(Debug, Clone)]
pub struct UnsupportedNode {
    pub loc: SourceLocation,
}

// =====================================================================================
// Memoization markers
// =====================================================================================

/// Dependency for manual memoization (useMemo/useCallback deps).
#[derive(Debug, Clone)]
pub struct ManualMemoDependency {
    pub root: ManualMemoDependencyRoot,
    pub path: DependencyPath,
    pub loc: SourceLocation,
}

#[derive(Debug, Clone)]
pub enum ManualMemoDependencyRoot {
    NamedLocal { value: Place, constant: bool },
    Global { identifier_name: String },
}

#[derive(Debug, Clone)]
pub struct StartMemoize {
    pub manual_memo_id: u32,
    pub deps: Option<Vec<ManualMemoDependency>>,
    pub deps_loc: Option<SourceLocation>,
    pub loc: SourceLocation,
}

#[derive(Debug, Clone)]
pub struct FinishMemoize {
    pub manual_memo_id: u32,
    pub decl: Place,
    pub pruned: bool,
    pub loc: SourceLocation,
}

// =====================================================================================
// Places & Identifiers
// =====================================================================================

/// A place where data may be read from / written to.
#[derive(Debug, Clone)]
pub struct Place {
    pub identifier: Identifier,
    pub effect: Effect,
    pub reactive: bool,
    pub loc: SourceLocation,
}

/// An identifier in the HIR — either a named variable or a temporary.
#[derive(Debug, Clone)]
pub struct Identifier {
    pub id: IdentifierId,
    pub declaration_id: DeclarationId,
    pub name: Option<IdentifierName>,
    pub mutable_range: MutableRange,
    pub scope: Option<Box<ReactiveScope>>,
    pub type_: Type,
    pub loc: SourceLocation,
}

/// The name of an identifier.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum IdentifierName {
    /// A validated identifier name from source code.
    Named(ValidIdentifierName),
    /// A promoted temporary (e.g., `#t0`).
    Promoted(String),
}

impl IdentifierName {
    /// Get the string value of this identifier name.
    pub fn value(&self) -> &str {
        match self {
            IdentifierName::Named(n) | IdentifierName::Promoted(n) => n,
        }
    }
}

/// A validated identifier name (not a reserved word, valid JS identifier).
pub type ValidIdentifierName = String;

/// Range in which an identifier is mutable. Start and End refer to `InstructionId`.
/// Start is inclusive, End is exclusive.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MutableRange {
    pub start: InstructionId,
    pub end: InstructionId,
}

impl Default for MutableRange {
    fn default() -> Self {
        Self { start: InstructionId(0), end: InstructionId(0) }
    }
}

// =====================================================================================
// Effects & Value kinds
// =====================================================================================

/// The effect with which a value is modified.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Effect {
    Unknown,
    Freeze,
    Read,
    Capture,
    ConditionallyMutateIterator,
    ConditionallyMutate,
    Mutate,
    Store,
}

impl Effect {
    /// Returns `true` if this effect is a mutable effect.
    pub fn is_mutable(self) -> bool {
        matches!(
            self,
            Effect::Capture
                | Effect::Store
                | Effect::ConditionallyMutate
                | Effect::ConditionallyMutateIterator
                | Effect::Mutate
        )
    }
}

impl std::fmt::Display for Effect {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Effect::Unknown => write!(f, "<unknown>"),
            Effect::Freeze => write!(f, "freeze"),
            Effect::Read => write!(f, "read"),
            Effect::Capture => write!(f, "capture"),
            Effect::ConditionallyMutateIterator => write!(f, "mutate-iterator?"),
            Effect::ConditionallyMutate => write!(f, "mutate?"),
            Effect::Mutate => write!(f, "mutate"),
            Effect::Store => write!(f, "store"),
        }
    }
}

/// Distinguishes different kinds of values relevant to inference.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ValueKind {
    MaybeFrozen,
    Frozen,
    Primitive,
    Global,
    Mutable,
    Context,
}

impl std::fmt::Display for ValueKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ValueKind::MaybeFrozen => write!(f, "maybefrozen"),
            ValueKind::Frozen => write!(f, "frozen"),
            ValueKind::Primitive => write!(f, "primitive"),
            ValueKind::Global => write!(f, "global"),
            ValueKind::Mutable => write!(f, "mutable"),
            ValueKind::Context => write!(f, "context"),
        }
    }
}

/// The reason for the kind of a value.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ValueReason {
    Global,
    JsxCaptured,
    HookCaptured,
    HookReturn,
    Effect,
    KnownReturnSignature,
    Context,
    State,
    ReducerState,
    ReactiveFunctionArgument,
    Other,
}

impl std::fmt::Display for ValueReason {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ValueReason::Global => write!(f, "global"),
            ValueReason::JsxCaptured => write!(f, "jsx-captured"),
            ValueReason::HookCaptured => write!(f, "hook-captured"),
            ValueReason::HookReturn => write!(f, "hook-return"),
            ValueReason::Effect => write!(f, "effect"),
            ValueReason::KnownReturnSignature => write!(f, "known-return-signature"),
            ValueReason::Context => write!(f, "context"),
            ValueReason::State => write!(f, "state"),
            ValueReason::ReducerState => write!(f, "reducer-state"),
            ValueReason::ReactiveFunctionArgument => write!(f, "reactive-function-argument"),
            ValueReason::Other => write!(f, "other"),
        }
    }
}

/// Abstract value: a combination of kind, reasons, and contexts.
#[derive(Debug, Clone)]
pub struct AbstractValue {
    pub kind: ValueKind,
    pub reason: FxHashSet<ValueReason>,
    pub context: FxHashSet<Place>,
}

// =====================================================================================
// Reactive Scopes
// =====================================================================================

/// A reactive scope — represents a region of code that will be memoized.
#[derive(Debug, Clone)]
pub struct ReactiveScope {
    pub id: ScopeId,
    pub range: MutableRange,
    pub dependencies: FxHashSet<ReactiveScopeDependency>,
    pub declarations: FxHashMap<IdentifierId, ReactiveScopeDeclaration>,
    pub reassignments: FxHashSet<IdentifierId>,
    pub early_return_value: Option<Box<EarlyReturnValue>>,
    pub merged: FxHashSet<ScopeId>,
    pub loc: SourceLocation,
}

#[derive(Debug, Clone)]
pub struct EarlyReturnValue {
    pub value: Identifier,
    pub loc: SourceLocation,
    pub label: BlockId,
}

#[derive(Debug, Clone)]
pub struct ReactiveScopeDeclaration {
    pub identifier: Identifier,
    pub scope: ReactiveScope,
}

/// A dependency path entry for property access chains.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct DependencyPathEntry {
    pub property: PropertyLiteral,
    pub optional: bool,
    pub loc: SourceLocation,
}

/// A dependency path — a chain of property accesses.
pub type DependencyPath = Vec<DependencyPathEntry>;

/// A reactive scope dependency — an identifier with a property path.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ReactiveScopeDependency {
    pub identifier_id: IdentifierId,
    pub reactive: bool,
    pub path: DependencyPath,
    pub loc: SourceLocation,
}

// =====================================================================================
// Reactive Function (tree-shaped IR for codegen)
// =====================================================================================

/// A React function in reactive form — the tree-shaped IR used for codegen.
#[derive(Debug, Clone)]
pub struct ReactiveFunction {
    pub loc: SourceLocation,
    pub id: Option<ValidIdentifierName>,
    pub name_hint: Option<String>,
    pub params: Vec<ReactiveParam>,
    pub generator: bool,
    pub is_async: bool,
    pub body: ReactiveBlock,
    pub directives: Vec<String>,
}

/// A parameter of a reactive function.
#[derive(Debug, Clone)]
pub enum ReactiveParam {
    Place(Place),
    Spread(SpreadPattern),
}

/// A block of reactive statements.
pub type ReactiveBlock = Vec<ReactiveStatement>;

/// A statement in a reactive block.
#[derive(Debug, Clone)]
pub enum ReactiveStatement {
    Instruction(ReactiveInstructionStatement),
    Terminal(Box<ReactiveTerminalStatement>),
    Scope(ReactiveScopeBlock),
    PrunedScope(PrunedReactiveScopeBlock),
}

#[derive(Debug, Clone)]
pub struct ReactiveInstructionStatement {
    pub instruction: ReactiveInstruction,
}

#[derive(Debug, Clone)]
pub struct ReactiveTerminalStatement {
    pub terminal: ReactiveTerminal,
    pub label: Option<ReactiveLabel>,
}

#[derive(Debug, Clone)]
pub struct ReactiveLabel {
    pub id: BlockId,
    pub implicit: bool,
}

#[derive(Debug, Clone)]
pub struct ReactiveScopeBlock {
    pub scope: ReactiveScope,
    pub instructions: ReactiveBlock,
}

#[derive(Debug, Clone)]
pub struct PrunedReactiveScopeBlock {
    pub scope: ReactiveScope,
    pub instructions: ReactiveBlock,
}

/// A reactive instruction.
#[derive(Debug, Clone)]
pub struct ReactiveInstruction {
    pub id: InstructionId,
    pub lvalue: Option<Place>,
    pub value: ReactiveValue,
    pub loc: SourceLocation,
}

/// A reactive value — may be a plain instruction value or a reactive-specific compound.
#[derive(Debug, Clone)]
pub enum ReactiveValue {
    Instruction(Box<InstructionValue>),
    Logical(ReactiveLogicalValue),
    Sequence(ReactiveSequenceValue),
    Ternary(ReactiveTernaryValue),
    OptionalCall(ReactiveOptionalCallValue),
}

#[derive(Debug, Clone)]
pub struct ReactiveLogicalValue {
    pub operator: LogicalOperator,
    pub left: Box<ReactiveValue>,
    pub right: Box<ReactiveValue>,
    pub loc: SourceLocation,
}

#[derive(Debug, Clone)]
pub struct ReactiveTernaryValue {
    pub test: Box<ReactiveValue>,
    pub consequent: Box<ReactiveValue>,
    pub alternate: Box<ReactiveValue>,
    pub loc: SourceLocation,
}

#[derive(Debug, Clone)]
pub struct ReactiveSequenceValue {
    pub instructions: Vec<ReactiveInstruction>,
    pub id: InstructionId,
    pub value: Box<ReactiveValue>,
    pub loc: SourceLocation,
}

#[derive(Debug, Clone)]
pub struct ReactiveOptionalCallValue {
    pub id: InstructionId,
    pub value: Box<ReactiveValue>,
    pub optional: bool,
    pub loc: SourceLocation,
}

/// A terminal in the reactive IR.
#[derive(Debug, Clone)]
pub enum ReactiveTerminal {
    Break(ReactiveBreakTerminal),
    Continue(ReactiveContinueTerminal),
    Return(ReactiveReturnTerminal),
    Throw(ReactiveThrowTerminal),
    Switch(Box<ReactiveSwitchTerminal>),
    DoWhile(Box<ReactiveDoWhileTerminal>),
    While(Box<ReactiveWhileTerminal>),
    For(Box<ReactiveForTerminal>),
    ForOf(Box<ReactiveForOfTerminal>),
    ForIn(Box<ReactiveForInTerminal>),
    If(Box<ReactiveIfTerminal>),
    Label(Box<ReactiveLabelTerminal>),
    Try(Box<ReactiveTryTerminal>),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReactiveTerminalTargetKind {
    Implicit,
    Labeled,
    Unlabeled,
}

#[derive(Debug, Clone)]
pub struct ReactiveBreakTerminal {
    pub target: BlockId,
    pub id: InstructionId,
    pub target_kind: ReactiveTerminalTargetKind,
    pub loc: SourceLocation,
}

#[derive(Debug, Clone)]
pub struct ReactiveContinueTerminal {
    pub target: BlockId,
    pub id: InstructionId,
    pub target_kind: ReactiveTerminalTargetKind,
    pub loc: SourceLocation,
}

#[derive(Debug, Clone)]
pub struct ReactiveReturnTerminal {
    pub value: Place,
    pub id: InstructionId,
    pub loc: SourceLocation,
}

#[derive(Debug, Clone)]
pub struct ReactiveThrowTerminal {
    pub value: Place,
    pub id: InstructionId,
    pub loc: SourceLocation,
}

#[derive(Debug, Clone)]
pub struct ReactiveSwitchTerminal {
    pub test: Place,
    pub cases: Vec<ReactiveSwitchCase>,
    pub id: InstructionId,
    pub loc: SourceLocation,
}

#[derive(Debug, Clone)]
pub struct ReactiveSwitchCase {
    pub test: Option<Place>,
    pub block: Option<ReactiveBlock>,
}

#[derive(Debug, Clone)]
pub struct ReactiveDoWhileTerminal {
    pub r#loop: ReactiveBlock,
    pub test: ReactiveValue,
    pub id: InstructionId,
    pub loc: SourceLocation,
}

#[derive(Debug, Clone)]
pub struct ReactiveWhileTerminal {
    pub test: ReactiveValue,
    pub r#loop: ReactiveBlock,
    pub id: InstructionId,
    pub loc: SourceLocation,
}

#[derive(Debug, Clone)]
pub struct ReactiveForTerminal {
    pub init: ReactiveValue,
    pub test: ReactiveValue,
    pub update: Option<ReactiveValue>,
    pub r#loop: ReactiveBlock,
    pub id: InstructionId,
    pub loc: SourceLocation,
}

#[derive(Debug, Clone)]
pub struct ReactiveForOfTerminal {
    pub init: ReactiveValue,
    pub test: ReactiveValue,
    pub r#loop: ReactiveBlock,
    pub id: InstructionId,
    pub loc: SourceLocation,
}

#[derive(Debug, Clone)]
pub struct ReactiveForInTerminal {
    pub init: ReactiveValue,
    pub r#loop: ReactiveBlock,
    pub id: InstructionId,
    pub loc: SourceLocation,
}

#[derive(Debug, Clone)]
pub struct ReactiveIfTerminal {
    pub test: Place,
    pub consequent: ReactiveBlock,
    pub alternate: Option<ReactiveBlock>,
    pub id: InstructionId,
    pub loc: SourceLocation,
}

#[derive(Debug, Clone)]
pub struct ReactiveLabelTerminal {
    pub block: ReactiveBlock,
    pub id: InstructionId,
    pub loc: SourceLocation,
}

#[derive(Debug, Clone)]
pub struct ReactiveTryTerminal {
    pub block: ReactiveBlock,
    pub handler_binding: Option<Place>,
    pub handler: ReactiveBlock,
    pub id: InstructionId,
    pub loc: SourceLocation,
}

// =====================================================================================
// HIR Function
// =====================================================================================

/// The type of a React function being compiled.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReactFunctionType {
    Component,
    Hook,
    Other,
}

/// A function lowered to HIR form.
#[derive(Debug, Clone)]
pub struct HIRFunction {
    pub loc: SourceLocation,
    pub id: Option<ValidIdentifierName>,
    pub name_hint: Option<String>,
    pub fn_type: ReactFunctionType,
    pub env: super::environment::Environment,
    pub params: Vec<ReactiveParam>,
    pub returns: Place,
    pub context: Vec<Place>,
    pub body: Hir,
    pub generator: bool,
    pub is_async: bool,
    pub directives: Vec<String>,
    pub aliasing_effects: Option<Vec<crate::inference::aliasing_effects::AliasingEffect>>,
}

// =====================================================================================
// Helper functions (type checks on identifiers)
// =====================================================================================

impl Identifier {
    pub fn is_object_method_type(&self) -> bool {
        self.type_ == Type::ObjectMethod
    }

    pub fn is_primitive_type(&self) -> bool {
        self.type_ == Type::Primitive
    }
}

impl Place {
    pub fn is_object_type(&self) -> bool {
        matches!(self.identifier.type_, Type::Object(_))
    }

    pub fn is_function_type(&self) -> bool {
        matches!(self.identifier.type_, Type::Function(_))
    }
}

/// Check if two dependency paths are equal.
pub fn are_equal_paths(a: &DependencyPath, b: &DependencyPath) -> bool {
    a.len() == b.len()
        && a.iter()
            .zip(b.iter())
            .all(|(ai, bi)| ai.property == bi.property && ai.optional == bi.optional)
}

/// Check if `subpath` is a prefix of `path`.
pub fn is_sub_path(subpath: &DependencyPath, path: &DependencyPath) -> bool {
    subpath.len() <= path.len()
        && subpath
            .iter()
            .zip(path.iter())
            .all(|(ai, bi)| ai.property == bi.property && ai.optional == bi.optional)
}

/// Check if `subpath` is a prefix of `path`, ignoring optional markers.
pub fn is_sub_path_ignoring_optionals(subpath: &DependencyPath, path: &DependencyPath) -> bool {
    subpath.len() <= path.len()
        && subpath.iter().zip(path.iter()).all(|(ai, bi)| ai.property == bi.property)
}
