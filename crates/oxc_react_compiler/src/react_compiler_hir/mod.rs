pub mod default_module_type_provider;
pub mod dominator;
pub mod environment;
pub mod environment_config;
pub mod globals;
pub mod object_shape;
pub mod raw;
pub mod reactive;
pub mod type_config;
pub mod visitors;

use crate::react_compiler_utils::FxIndexMap;
use crate::react_compiler_utils::FxIndexSet;
use oxc_ast::ast::*;
use oxc_diagnostics::OxcDiagnostic;
use oxc_index::define_nonmax_u32_index_type;
use oxc_str::{Ident, Str};
use oxc_syntax::number::ToJsString;
pub use raw::RawTypeCategory;
pub use reactive::*;

/// Sentinel value for generated/synthetic source locations (a node with no span).
pub const GENERATED_SOURCE: Option<Span> = None;

// =============================================================================
// ID newtypes
// =============================================================================

define_nonmax_u32_index_type! {
    pub struct BlockId;
}

define_nonmax_u32_index_type! {
    pub struct IdentifierId;
}

define_nonmax_u32_index_type! {
    /// Index into the flat instruction table on HirFunction.
    pub struct InstructionId;
}

define_nonmax_u32_index_type! {
    /// Evaluation order assigned to instructions and terminals during numbering.
    /// This was previously called InstructionId in the TypeScript compiler.
    pub struct EvaluationOrder;
}

define_nonmax_u32_index_type! {
    pub struct DeclarationId;
}

define_nonmax_u32_index_type! {
    pub struct ScopeId;
}

define_nonmax_u32_index_type! {
    pub struct TypeId;
}

define_nonmax_u32_index_type! {
    pub struct FunctionId;
}

define_nonmax_u32_index_type! {
    pub struct MutableRangeId;
}

impl BlockId {
    /// Placeholder id for the never-read block that the final `terminate()` installs as
    /// `current`. Uses the largest representable index so it can never alias a real block
    /// (blocks are numbered from 0).
    pub const PLACEHOLDER: Self = Self::from_usize(Self::MAX_INDEX);
}

impl EvaluationOrder {
    /// Placeholder order for instructions, terminals, and ranges before
    /// `mark_instruction_ids` assigns the real, 1-based orders. `0` is never a valid
    /// assigned order.
    pub const UNSET: Self = Self::from_usize(0);
}

// =============================================================================
// FloatValue wrapper
// =============================================================================

/// Wrapper around f64 that stores raw bytes for deterministic equality and hashing.
/// This allows use in FxHashMap keys and ensures NaN == NaN (bitwise comparison).
#[derive(Debug, Clone, Copy)]
pub struct FloatValue(u64);

impl FloatValue {
    pub fn new(value: f64) -> Self {
        FloatValue(value.to_bits())
    }

    pub fn value(self) -> f64 {
        f64::from_bits(self.0)
    }
}

impl From<f64> for FloatValue {
    fn from(value: f64) -> Self {
        FloatValue::new(value)
    }
}

impl From<FloatValue> for f64 {
    fn from(value: FloatValue) -> Self {
        value.value()
    }
}

impl PartialEq for FloatValue {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

impl Eq for FloatValue {}

impl std::hash::Hash for FloatValue {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.0.hash(state);
    }
}

impl std::fmt::Display for FloatValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.value().to_js_string())
    }
}

// =============================================================================
// Core HIR types
// =============================================================================

/// A function lowered to HIR form
#[derive(Debug, Clone)]
pub struct HirFunction<'a> {
    pub span: Option<Span>,
    pub id: Option<Ident<'a>>,
    pub name_hint: Option<Ident<'a>>,
    pub fn_type: ReactFunctionType,
    pub params: Vec<ParamPattern>,
    pub returns: Place,
    pub context: Vec<Place>,
    pub body: HIR,
    pub instructions: Vec<Instruction<'a>>,
    pub generator: bool,
    pub is_async: bool,
    pub directives: Vec<Str<'a>>,
    pub aliasing_effects: Option<Vec<AliasingEffect>>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReactFunctionType {
    Component,
    Hook,
    Other,
}

#[derive(Debug, Clone)]
pub enum ParamPattern {
    Place(Place),
    Spread(SpreadPattern),
}

/// The HIR control-flow graph
#[derive(Debug, Clone)]
#[allow(clippy::upper_case_acronyms)]
pub struct HIR {
    pub entry: BlockId,
    pub blocks: FxIndexMap<BlockId, BasicBlock>,
}

/// Block kinds
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BlockKind {
    Block,
    Value,
    Loop,
    Sequence,
    Catch,
}

impl std::fmt::Display for BlockKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BlockKind::Block => write!(f, "block"),
            BlockKind::Value => write!(f, "value"),
            BlockKind::Loop => write!(f, "loop"),
            BlockKind::Sequence => write!(f, "sequence"),
            BlockKind::Catch => write!(f, "catch"),
        }
    }
}

/// A basic block in the CFG
#[derive(Debug, Clone)]
pub struct BasicBlock {
    pub kind: BlockKind,
    pub id: BlockId,
    pub instructions: Vec<InstructionId>,
    pub terminal: Terminal,
    pub preds: FxIndexSet<BlockId>,
    pub phis: Vec<Phi>,
}

/// Phi node for SSA
#[derive(Debug, Clone)]
pub struct Phi {
    pub place: Place,
    pub operands: FxIndexMap<BlockId, Place>,
}

// =============================================================================
// Terminal enum
// =============================================================================

#[derive(Debug, Clone)]
pub enum Terminal {
    Unreachable {
        id: EvaluationOrder,
        span: Option<Span>,
    },
    Throw {
        value: Place,
        id: EvaluationOrder,
        span: Option<Span>,
    },
    Return {
        value: Place,
        return_variant: ReturnVariant,
        id: EvaluationOrder,
        span: Option<Span>,
        effects: Option<Vec<AliasingEffect>>,
    },
    Goto {
        block: BlockId,
        variant: GotoVariant,
        id: EvaluationOrder,
        span: Option<Span>,
    },
    If {
        test: Place,
        consequent: BlockId,
        alternate: BlockId,
        fallthrough: BlockId,
        id: EvaluationOrder,
        span: Option<Span>,
    },
    Branch {
        test: Place,
        consequent: BlockId,
        alternate: BlockId,
        fallthrough: BlockId,
        id: EvaluationOrder,
        span: Option<Span>,
    },
    Switch {
        test: Place,
        cases: Vec<Case>,
        fallthrough: BlockId,
        id: EvaluationOrder,
        span: Option<Span>,
    },
    DoWhile {
        loop_block: BlockId,
        test: BlockId,
        fallthrough: BlockId,
        id: EvaluationOrder,
        span: Option<Span>,
    },
    While {
        test: BlockId,
        loop_block: BlockId,
        fallthrough: BlockId,
        id: EvaluationOrder,
        span: Option<Span>,
    },
    For {
        init: BlockId,
        test: BlockId,
        update: Option<BlockId>,
        loop_block: BlockId,
        fallthrough: BlockId,
        id: EvaluationOrder,
        span: Option<Span>,
    },
    ForOf {
        init: BlockId,
        test: BlockId,
        loop_block: BlockId,
        fallthrough: BlockId,
        id: EvaluationOrder,
        span: Option<Span>,
    },
    ForIn {
        init: BlockId,
        loop_block: BlockId,
        fallthrough: BlockId,
        id: EvaluationOrder,
        span: Option<Span>,
    },
    Logical {
        operator: LogicalOperator,
        test: BlockId,
        fallthrough: BlockId,
        id: EvaluationOrder,
        span: Option<Span>,
    },
    Ternary {
        test: BlockId,
        fallthrough: BlockId,
        id: EvaluationOrder,
        span: Option<Span>,
    },
    Optional {
        optional: bool,
        test: BlockId,
        fallthrough: BlockId,
        id: EvaluationOrder,
        span: Option<Span>,
    },
    Label {
        block: BlockId,
        fallthrough: BlockId,
        id: EvaluationOrder,
        span: Option<Span>,
    },
    Sequence {
        block: BlockId,
        fallthrough: BlockId,
        id: EvaluationOrder,
        span: Option<Span>,
    },
    MaybeThrow {
        continuation: BlockId,
        handler: Option<BlockId>,
        id: EvaluationOrder,
        span: Option<Span>,
        effects: Option<Vec<AliasingEffect>>,
    },
    Try {
        block: BlockId,
        handler_binding: Option<Place>,
        handler: BlockId,
        fallthrough: BlockId,
        id: EvaluationOrder,
        span: Option<Span>,
    },
    Scope {
        fallthrough: BlockId,
        block: BlockId,
        scope: ScopeId,
        id: EvaluationOrder,
        span: Option<Span>,
    },
    PrunedScope {
        fallthrough: BlockId,
        block: BlockId,
        scope: ScopeId,
        id: EvaluationOrder,
        span: Option<Span>,
    },
}

impl Terminal {
    /// Get the evaluation order of this terminal
    pub fn evaluation_order(&self) -> EvaluationOrder {
        match self {
            Terminal::Unreachable { id, .. }
            | Terminal::Throw { id, .. }
            | Terminal::Return { id, .. }
            | Terminal::Goto { id, .. }
            | Terminal::If { id, .. }
            | Terminal::Branch { id, .. }
            | Terminal::Switch { id, .. }
            | Terminal::DoWhile { id, .. }
            | Terminal::While { id, .. }
            | Terminal::For { id, .. }
            | Terminal::ForOf { id, .. }
            | Terminal::ForIn { id, .. }
            | Terminal::Logical { id, .. }
            | Terminal::Ternary { id, .. }
            | Terminal::Optional { id, .. }
            | Terminal::Label { id, .. }
            | Terminal::Sequence { id, .. }
            | Terminal::MaybeThrow { id, .. }
            | Terminal::Try { id, .. }
            | Terminal::Scope { id, .. }
            | Terminal::PrunedScope { id, .. } => *id,
        }
    }

    /// Get the source location of this terminal
    pub fn span(&self) -> Option<&Span> {
        match self {
            Terminal::Unreachable { span, .. }
            | Terminal::Throw { span, .. }
            | Terminal::Return { span, .. }
            | Terminal::Goto { span, .. }
            | Terminal::If { span, .. }
            | Terminal::Branch { span, .. }
            | Terminal::Switch { span, .. }
            | Terminal::DoWhile { span, .. }
            | Terminal::While { span, .. }
            | Terminal::For { span, .. }
            | Terminal::ForOf { span, .. }
            | Terminal::ForIn { span, .. }
            | Terminal::Logical { span, .. }
            | Terminal::Ternary { span, .. }
            | Terminal::Optional { span, .. }
            | Terminal::Label { span, .. }
            | Terminal::Sequence { span, .. }
            | Terminal::MaybeThrow { span, .. }
            | Terminal::Try { span, .. }
            | Terminal::Scope { span, .. }
            | Terminal::PrunedScope { span, .. } => span.as_ref(),
        }
    }

    /// Set the evaluation order of this terminal
    pub fn set_evaluation_order(&mut self, new_id: EvaluationOrder) {
        match self {
            Terminal::Unreachable { id, .. }
            | Terminal::Throw { id, .. }
            | Terminal::Return { id, .. }
            | Terminal::Goto { id, .. }
            | Terminal::If { id, .. }
            | Terminal::Branch { id, .. }
            | Terminal::Switch { id, .. }
            | Terminal::DoWhile { id, .. }
            | Terminal::While { id, .. }
            | Terminal::For { id, .. }
            | Terminal::ForOf { id, .. }
            | Terminal::ForIn { id, .. }
            | Terminal::Logical { id, .. }
            | Terminal::Ternary { id, .. }
            | Terminal::Optional { id, .. }
            | Terminal::Label { id, .. }
            | Terminal::Sequence { id, .. }
            | Terminal::MaybeThrow { id, .. }
            | Terminal::Try { id, .. }
            | Terminal::Scope { id, .. }
            | Terminal::PrunedScope { id, .. } => *id = new_id,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReturnVariant {
    Void,
    Implicit,
    Explicit,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GotoVariant {
    Break,
    Continue,
    Try,
}

#[derive(Debug, Clone)]
pub struct Case {
    pub test: Option<Place>,
    pub block: BlockId,
}

// =============================================================================
// Instruction types
// =============================================================================

#[derive(Debug, Clone)]
pub struct Instruction<'a> {
    pub id: EvaluationOrder,
    pub lvalue: Place,
    pub value: InstructionValue<'a>,
    pub span: Option<Span>,
    pub effects: Option<Vec<AliasingEffect>>,
}

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

#[derive(Debug, Clone)]
pub struct LValue {
    pub place: Place,
    pub kind: InstructionKind,
}

#[derive(Debug, Clone)]
pub struct LValuePattern<'a> {
    pub pattern: Pattern<'a>,
    pub kind: InstructionKind,
}

#[derive(Debug, Clone)]
pub enum Pattern<'a> {
    Array(ArrayPattern),
    Object(ObjectPattern<'a>),
}

// =============================================================================
// InstructionValue enum
// =============================================================================

#[derive(Debug, Clone)]
pub enum InstructionValue<'a> {
    LoadLocal {
        place: Place,
        span: Option<Span>,
    },
    LoadContext {
        place: Place,
        span: Option<Span>,
    },
    DeclareLocal {
        lvalue: LValue,
        span: Option<Span>,
    },
    DeclareContext {
        lvalue: LValue,
        span: Option<Span>,
    },
    StoreLocal {
        lvalue: LValue,
        value: Place,
        span: Option<Span>,
    },
    StoreContext {
        lvalue: LValue,
        value: Place,
        span: Option<Span>,
    },
    Destructure {
        lvalue: LValuePattern<'a>,
        value: Place,
        span: Option<Span>,
    },
    Primitive {
        value: PrimitiveValue<'a>,
        span: Option<Span>,
    },
    JSXText {
        value: Str<'a>,
        span: Option<Span>,
    },
    BinaryExpression {
        operator: BinaryOperator,
        left: Place,
        right: Place,
        span: Option<Span>,
    },
    NewExpression {
        callee: Place,
        args: Vec<PlaceOrSpread>,
        span: Option<Span>,
    },
    CallExpression {
        callee: Place,
        args: Vec<PlaceOrSpread>,
        span: Option<Span>,
    },
    MethodCall {
        receiver: Place,
        property: Place,
        args: Vec<PlaceOrSpread>,
        span: Option<Span>,
    },
    UnaryExpression {
        operator: UnaryOperator,
        value: Place,
        span: Option<Span>,
    },
    TypeCastExpression {
        value: Place,
        cast: TypeCast<'a>,
        span: Option<Span>,
    },
    JsxExpression {
        tag: JsxTag<'a>,
        props: Vec<JsxAttribute<'a>>,
        children: Option<Vec<Place>>,
        span: Option<Span>,
        opening_span: Option<Span>,
        closing_span: Option<Span>,
    },
    ObjectExpression {
        properties: Vec<ObjectPropertyOrSpread<'a>>,
        span: Option<Span>,
    },
    ObjectMethod {
        span: Option<Span>,
        lowered_func: LoweredFunction,
    },
    ArrayExpression {
        elements: Vec<ArrayElement>,
        span: Option<Span>,
    },
    JsxFragment {
        children: Vec<Place>,
        span: Option<Span>,
    },
    RegExpLiteral {
        pattern: Str<'a>,
        flags: Str<'a>,
        span: Option<Span>,
    },
    MetaProperty {
        meta: Ident<'a>,
        property: Ident<'a>,
        span: Option<Span>,
    },
    PropertyStore {
        object: Place,
        property: PropertyLiteral<'a>,
        value: Place,
        span: Option<Span>,
    },
    PropertyLoad {
        object: Place,
        property: PropertyLiteral<'a>,
        span: Option<Span>,
    },
    PropertyDelete {
        object: Place,
        property: PropertyLiteral<'a>,
        span: Option<Span>,
    },
    ComputedStore {
        object: Place,
        property: Place,
        value: Place,
        span: Option<Span>,
    },
    ComputedLoad {
        object: Place,
        property: Place,
        span: Option<Span>,
    },
    ComputedDelete {
        object: Place,
        property: Place,
        span: Option<Span>,
    },
    LoadGlobal {
        binding: NonLocalBinding<'a>,
        span: Option<Span>,
    },
    StoreGlobal {
        name: Ident<'a>,
        value: Place,
        span: Option<Span>,
    },
    FunctionExpression {
        name: Option<Ident<'a>>,
        name_hint: Option<Ident<'a>>,
        lowered_func: LoweredFunction,
        expr_type: FunctionExpressionType,
        span: Option<Span>,
    },
    TaggedTemplateExpression {
        tag: Place,
        // Mirrors `TemplateLiteral`: `subexprs.len() == quasis.len() - 1`.
        // Upstream's HIR models only a single quasi with no interpolation; the
        // oxc port extends it to support `tag`-ed templates with `${...}`
        // interpolations (a deliberate divergence from the TS reference).
        quasis: Vec<TemplateQuasi<'a>>,
        subexprs: Vec<Place>,
        span: Option<Span>,
    },
    TemplateLiteral {
        subexprs: Vec<Place>,
        quasis: Vec<TemplateQuasi<'a>>,
        span: Option<Span>,
    },
    Await {
        value: Place,
        span: Option<Span>,
    },
    GetIterator {
        collection: Place,
        span: Option<Span>,
    },
    IteratorNext {
        iterator: Place,
        collection: Place,
        span: Option<Span>,
    },
    NextPropertyOf {
        value: Place,
        span: Option<Span>,
    },
    PrefixUpdate {
        lvalue: Place,
        operation: UpdateOperator,
        value: Place,
        span: Option<Span>,
    },
    PostfixUpdate {
        lvalue: Place,
        operation: UpdateOperator,
        value: Place,
        span: Option<Span>,
    },
    Debugger {
        span: Option<Span>,
    },
    StartMemoize {
        manual_memo_id: u32,
        deps: Option<Vec<ManualMemoDependency<'a>>>,
        deps_span: Option<Option<Span>>,
        has_invalid_deps: bool,
        span: Option<Span>,
    },
    FinishMemoize {
        manual_memo_id: u32,
        decl: Place,
        pruned: bool,
        span: Option<Span>,
    },
}

/// A preserved TS type-cast wrapper, aligned with the oxc AST node it was
/// lowered from. The type subtree is an arena clone made at lowering; codegen
/// re-emits it, applying identifier renames via semantic reference ids.
#[derive(Debug, Clone, Copy)]
pub enum TypeCast<'a> {
    /// `expr as T` (`TSAsExpression`) and `<T>expr` (`TSTypeAssertion`)
    As(&'a TSType<'a>),
    /// `expr satisfies T` (`TSSatisfiesExpression`)
    Satisfies(&'a TSType<'a>),
}

impl<'a> InstructionValue<'a> {
    pub fn span(&self) -> Option<&Span> {
        match self {
            InstructionValue::LoadLocal { span, .. }
            | InstructionValue::LoadContext { span, .. }
            | InstructionValue::DeclareLocal { span, .. }
            | InstructionValue::DeclareContext { span, .. }
            | InstructionValue::StoreLocal { span, .. }
            | InstructionValue::StoreContext { span, .. }
            | InstructionValue::Destructure { span, .. }
            | InstructionValue::Primitive { span, .. }
            | InstructionValue::JSXText { span, .. }
            | InstructionValue::BinaryExpression { span, .. }
            | InstructionValue::NewExpression { span, .. }
            | InstructionValue::CallExpression { span, .. }
            | InstructionValue::MethodCall { span, .. }
            | InstructionValue::UnaryExpression { span, .. }
            | InstructionValue::TypeCastExpression { span, .. }
            | InstructionValue::JsxExpression { span, .. }
            | InstructionValue::ObjectExpression { span, .. }
            | InstructionValue::ObjectMethod { span, .. }
            | InstructionValue::ArrayExpression { span, .. }
            | InstructionValue::JsxFragment { span, .. }
            | InstructionValue::RegExpLiteral { span, .. }
            | InstructionValue::MetaProperty { span, .. }
            | InstructionValue::PropertyStore { span, .. }
            | InstructionValue::PropertyLoad { span, .. }
            | InstructionValue::PropertyDelete { span, .. }
            | InstructionValue::ComputedStore { span, .. }
            | InstructionValue::ComputedLoad { span, .. }
            | InstructionValue::ComputedDelete { span, .. }
            | InstructionValue::LoadGlobal { span, .. }
            | InstructionValue::StoreGlobal { span, .. }
            | InstructionValue::FunctionExpression { span, .. }
            | InstructionValue::TaggedTemplateExpression { span, .. }
            | InstructionValue::TemplateLiteral { span, .. }
            | InstructionValue::Await { span, .. }
            | InstructionValue::GetIterator { span, .. }
            | InstructionValue::IteratorNext { span, .. }
            | InstructionValue::NextPropertyOf { span, .. }
            | InstructionValue::PrefixUpdate { span, .. }
            | InstructionValue::PostfixUpdate { span, .. }
            | InstructionValue::Debugger { span, .. }
            | InstructionValue::StartMemoize { span, .. }
            | InstructionValue::FinishMemoize { span, .. } => span.as_ref(),
        }
    }
}

// =============================================================================
// Supporting types
// =============================================================================

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PrimitiveValue<'a> {
    Null,
    Undefined,
    Boolean(bool),
    Number(FloatValue),
    String(Str<'a>),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnaryOperator {
    Minus,
    Plus,
    Not,
    BitwiseNot,
    TypeOf,
    Void,
}

impl std::fmt::Display for UnaryOperator {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            UnaryOperator::Minus => write!(f, "-"),
            UnaryOperator::Plus => write!(f, "+"),
            UnaryOperator::Not => write!(f, "!"),
            UnaryOperator::BitwiseNot => write!(f, "~"),
            UnaryOperator::TypeOf => write!(f, "typeof"),
            UnaryOperator::Void => write!(f, "void"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FunctionExpressionType {
    ArrowFunctionExpression,
    FunctionExpression,
    FunctionDeclaration,
}

#[derive(Debug, Clone)]
pub struct TemplateQuasi<'a> {
    pub raw: Str<'a>,
    pub cooked: Option<Str<'a>>,
}

#[derive(Debug, Clone)]
pub struct ManualMemoDependency<'a> {
    pub root: ManualMemoDependencyRoot<'a>,
    pub path: Vec<DependencyPathEntry<'a>>,
    pub span: Option<Span>,
}

#[derive(Debug, Clone)]
pub enum ManualMemoDependencyRoot<'a> {
    NamedLocal { value: Place, constant: bool },
    Global { identifier_name: Ident<'a> },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DependencyPathEntry<'a> {
    pub property: PropertyLiteral<'a>,
    pub optional: bool,
    pub span: Option<Span>,
}

// =============================================================================
// Place, Identifier, and related types
// =============================================================================

#[derive(Debug, Clone)]
pub struct Place {
    pub identifier: IdentifierId,
    pub effect: Effect,
    pub reactive: bool,
    pub span: Option<Span>,
}

#[derive(Debug, Clone)]
pub struct Identifier<'a> {
    pub id: IdentifierId,
    pub declaration_id: DeclarationId,
    pub name: Option<IdentifierName<'a>>,
    pub mutable_range: MutableRange,
    pub scope: Option<ScopeId>,
    pub type_: TypeId,
    pub span: Option<Span>,
}

#[derive(Debug, Clone)]
pub struct MutableRange {
    /// Unique identity for this logical range. Cloning preserves the ID
    /// (same logical range); use `Environment::new_mutable_range()` to create
    /// a range with a fresh ID.
    pub id: MutableRangeId,
    pub start: EvaluationOrder,
    pub end: EvaluationOrder,
}

impl MutableRange {
    /// Returns true if the given evaluation order falls within this mutable range.
    /// Corresponds to TS `inRange({id}, range)` / `isMutable(instr, place)`.
    pub fn contains(&self, eval_order: EvaluationOrder) -> bool {
        eval_order >= self.start && eval_order < self.end
    }

    /// Returns true if this range has the same identity as `other`.
    /// In the TS compiler, this corresponds to checking whether two mutableRange
    /// references point to the same JS object (=== identity).
    pub fn same_range(&self, other: &MutableRange) -> bool {
        self.id == other.id
    }
}

#[derive(Debug, Clone, Copy)]
pub enum IdentifierName<'a> {
    Named(Ident<'a>),
    Promoted(Ident<'a>),
}

impl<'a> IdentifierName<'a> {
    pub fn value(&self) -> &'a str {
        match self {
            IdentifierName::Named(v) | IdentifierName::Promoted(v) => v.as_str(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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
    /// Returns true if this effect represents a mutable operation.
    /// Mutable effects are: Capture, Store, ConditionallyMutate,
    /// ConditionallyMutateIterator, and Mutate.
    pub fn is_mutable(&self) -> bool {
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

#[derive(Debug, Clone)]
pub struct SpreadPattern {
    pub place: Place,
}

#[derive(Debug, Clone)]
pub struct ArrayPattern {
    pub items: Vec<ArrayPatternElement>,
}

#[derive(Debug, Clone)]
pub enum ArrayPatternElement {
    Place(Place),
    Spread(SpreadPattern),
    Hole,
}

#[derive(Debug, Clone)]
pub struct ObjectPattern<'a> {
    pub properties: Vec<ObjectPropertyOrSpread<'a>>,
}

#[derive(Debug, Clone)]
pub enum ObjectPropertyOrSpread<'a> {
    Property(ObjectProperty<'a>),
    Spread(SpreadPattern),
}

#[derive(Debug, Clone)]
pub struct ObjectProperty<'a> {
    pub key: ObjectPropertyKey<'a>,
    pub property_type: ObjectPropertyType,
    pub place: Place,
}

#[derive(Debug, Clone)]
pub enum ObjectPropertyKey<'a> {
    String { name: Ident<'a> },
    Identifier { name: Ident<'a> },
    Computed { name: Place },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ObjectPropertyType {
    Property,
    Method,
}

impl std::fmt::Display for ObjectPropertyType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ObjectPropertyType::Property => write!(f, "property"),
            ObjectPropertyType::Method => write!(f, "method"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PropertyLiteral<'a> {
    String(Ident<'a>),
    Number(FloatValue),
}

impl PropertyLiteral<'_> {
    pub fn is_string(&self, value: &str) -> bool {
        matches!(self, Self::String(s) if *s == value)
    }
}

impl std::fmt::Display for PropertyLiteral<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PropertyLiteral::String(s) => write!(f, "{}", s),
            PropertyLiteral::Number(n) => write!(f, "{}", n),
        }
    }
}

#[derive(Debug, Clone)]
pub enum PlaceOrSpread {
    Place(Place),
    Spread(SpreadPattern),
}

#[derive(Debug, Clone)]
pub enum ArrayElement {
    Place(Place),
    Spread(SpreadPattern),
    Hole,
}

#[derive(Debug, Clone)]
pub struct LoweredFunction {
    pub func: FunctionId,
}

#[derive(Debug, Clone)]
pub struct BuiltinTag<'a> {
    pub name: Ident<'a>,
}

#[derive(Debug, Clone)]
pub enum JsxTag<'a> {
    Place(Place),
    Builtin(BuiltinTag<'a>),
}

#[derive(Debug, Clone)]
pub enum JsxAttribute<'a> {
    SpreadAttribute { argument: Place },
    Attribute { name: Ident<'a>, place: Place },
}

// =============================================================================
// Variable Binding types
// =============================================================================

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BindingKind {
    Var,
    Let,
    Const,
    Param,
    Module,
    Hoisted,
    Local,
    Unknown,
}

#[derive(Debug, Clone)]
pub enum VariableBinding<'a> {
    Identifier { identifier: IdentifierId, binding_kind: BindingKind },
    Global { name: Ident<'a> },
    ImportDefault { name: Ident<'a>, module: Str<'a> },
    ImportSpecifier { name: Ident<'a>, module: Str<'a>, imported: Ident<'a> },
    ImportNamespace { name: Ident<'a>, module: Str<'a> },
    ModuleLocal { name: Ident<'a> },
}

#[derive(Debug, Clone)]
pub enum NonLocalBinding<'a> {
    ImportDefault { name: Ident<'a>, module: Str<'a> },
    ImportSpecifier { name: Ident<'a>, module: Str<'a>, imported: Ident<'a> },
    ImportNamespace { name: Ident<'a>, module: Str<'a> },
    ModuleLocal { name: Ident<'a> },
    Global { name: Ident<'a> },
}

impl<'a> NonLocalBinding<'a> {
    /// Returns the `name` field common to all variants.
    pub fn name(&self) -> Ident<'a> {
        match self {
            NonLocalBinding::ImportDefault { name, .. }
            | NonLocalBinding::ImportSpecifier { name, .. }
            | NonLocalBinding::ImportNamespace { name, .. }
            | NonLocalBinding::ModuleLocal { name, .. }
            | NonLocalBinding::Global { name, .. } => *name,
        }
    }
}

// =============================================================================
// Type system (from Types.ts)
// =============================================================================

#[derive(Debug, Clone)]
pub enum Type<'a> {
    Primitive,
    Function {
        shape_id: Option<Ident<'a>>,
        return_type: Box<Type<'a>>,
        is_constructor: bool,
    },
    Object {
        shape_id: Option<Ident<'a>>,
    },
    #[allow(clippy::enum_variant_names)]
    TypeVar {
        id: TypeId,
    },
    Poly,
    Phi {
        operands: Vec<Type<'a>>,
    },
    Property {
        object_type: Box<Type<'a>>,
        object_name: Ident<'a>,
        property_name: PropertyNameKind<'a>,
    },
    ObjectMethod,
}

#[derive(Debug, Clone)]
pub enum PropertyNameKind<'a> {
    Literal { value: PropertyLiteral<'a> },
    Computed,
}

// =============================================================================
// ReactiveScope
// =============================================================================

#[derive(Debug, Clone)]
pub struct ReactiveScope<'a> {
    pub id: ScopeId,
    pub range: MutableRange,

    /// The inputs to this reactive scope (populated by later passes)
    pub dependencies: Vec<ReactiveScopeDependency<'a>>,

    /// The set of values produced by this scope (populated by later passes)
    pub declarations: Vec<(IdentifierId, ReactiveScopeDeclaration)>,

    /// Identifiers which are reassigned by this scope (populated by later passes)
    pub reassignments: Vec<IdentifierId>,

    /// If the scope contains an early return, this stores info about it (populated by later passes)
    pub early_return_value: Option<ReactiveScopeEarlyReturn>,

    /// Scopes that were merged into this one (populated by later passes)
    pub merged: Vec<ScopeId>,

    /// Source location spanning the scope
    pub span: Option<Span>,
}

/// A dependency of a reactive scope.
#[derive(Debug, Clone)]
pub struct ReactiveScopeDependency<'a> {
    pub identifier: IdentifierId,
    pub reactive: bool,
    pub path: Vec<DependencyPathEntry<'a>>,
    pub span: Option<Span>,
}

/// A declaration produced by a reactive scope.
#[derive(Debug, Clone)]
pub struct ReactiveScopeDeclaration {
    pub identifier: IdentifierId,
    pub scope: ScopeId,
}

/// Early return value info for a reactive scope.
#[derive(Debug, Clone)]
pub struct ReactiveScopeEarlyReturn {
    pub value: IdentifierId,
    pub span: Option<Span>,
    pub label: BlockId,
}

// =============================================================================
// Aliasing effects (runtime types, from AliasingEffects.ts)
// =============================================================================

use crate::react_compiler_hir::object_shape::FunctionSignature;
use crate::react_compiler_hir::type_config::ValueKind;
use crate::react_compiler_hir::type_config::ValueReason;

/// Reason for a mutation, used for generating hints (e.g. rename to "Ref").
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MutationReason {
    AssignCurrentProperty,
}

/// Describes the aliasing/mutation/data-flow effects of an instruction or terminal.
/// Ported from TS `AliasingEffect` in `AliasingEffects.ts`.
#[derive(Debug, Clone)]
#[allow(clippy::large_enum_variant)]
pub enum AliasingEffect {
    /// Marks the given value and its direct aliases as frozen.
    Freeze { value: Place, reason: ValueReason },
    /// Mutate the value and any direct aliases.
    Mutate { value: Place, reason: Option<MutationReason> },
    /// Mutate the value conditionally (only if mutable).
    MutateConditionally { value: Place },
    /// Mutate the value and transitive captures.
    MutateTransitive { value: Place },
    /// Mutate the value and transitive captures conditionally.
    MutateTransitiveConditionally { value: Place },
    /// Information flow from `from` to `into` (non-aliasing capture).
    Capture { from: Place, into: Place },
    /// Direct aliasing: mutation of `into` implies mutation of `from`.
    Alias { from: Place, into: Place },
    /// Potential aliasing relationship.
    MaybeAlias { from: Place, into: Place },
    /// Direct assignment: `into = from`.
    Assign { from: Place, into: Place },
    /// Creates a value of the given kind at the given place.
    Create { into: Place, value: ValueKind, reason: ValueReason },
    /// Creates a new value with the same kind as the source.
    CreateFrom { from: Place, into: Place },
    /// Immutable data flow (escape analysis only, no mutable range influence).
    ImmutableCapture { from: Place, into: Place },
    /// Function call application.
    Apply {
        receiver: Place,
        function: Place,
        mutates_function: bool,
        args: Vec<PlaceOrSpreadOrHole>,
        into: Place,
        signature: Option<FunctionSignature>,
        span: Option<Span>,
    },
    /// Function expression creation with captures.
    CreateFunction { captures: Vec<Place>, function_id: FunctionId, into: Place },
    /// Mutation of a value known to be frozen (error).
    MutateFrozen { place: Place, error: OxcDiagnostic },
    /// Mutation of a global value (error).
    MutateGlobal { place: Place, error: OxcDiagnostic },
    /// Side-effect not safe during render.
    Impure { place: Place, error: OxcDiagnostic },
    /// Value is accessed during render.
    Render { place: Place },
}

/// Combined Place/Spread/Hole for Apply args.
#[derive(Debug, Clone)]
pub enum PlaceOrSpreadOrHole {
    Place(Place),
    Spread(SpreadPattern),
    Hole,
}

/// Aliasing signature for function calls.
/// Ported from TS `AliasingSignature` in `AliasingEffects.ts`.
#[derive(Debug, Clone)]
pub struct AliasingSignature {
    pub receiver: IdentifierId,
    pub params: Vec<IdentifierId>,
    pub rest: Option<IdentifierId>,
    pub returns: IdentifierId,
    pub effects: Vec<AliasingEffect>,
    pub temporaries: Vec<Place>,
}

// =============================================================================
// Type helper functions (ported from HIR.ts)
// =============================================================================

use crate::react_compiler_hir::object_shape::BUILT_IN_ARRAY_ID;
use crate::react_compiler_hir::object_shape::BUILT_IN_JSX_ID;
use crate::react_compiler_hir::object_shape::BUILT_IN_PROPS_ID;
use crate::react_compiler_hir::object_shape::BUILT_IN_REF_VALUE_ID;
use crate::react_compiler_hir::object_shape::BUILT_IN_USE_OPERATOR_ID;
use crate::react_compiler_hir::object_shape::BUILT_IN_USE_REF_ID;

/// Returns true if the type (looked up via identifier) is primitive.
pub fn is_primitive_type(ty: &Type) -> bool {
    matches!(ty, Type::Primitive)
}

/// Returns true if the type is the props object.
pub fn is_props_type(ty: &Type) -> bool {
    matches!(ty, Type::Object { shape_id: Some(id) } if *id == BUILT_IN_PROPS_ID)
}

/// Returns true if the type is an array.
pub fn is_array_type(ty: &Type) -> bool {
    matches!(ty, Type::Object { shape_id: Some(id) } if *id == BUILT_IN_ARRAY_ID)
}

/// Returns true if the type is JSX.
pub fn is_jsx_type(ty: &Type) -> bool {
    matches!(ty, Type::Object { shape_id: Some(id) } if *id == BUILT_IN_JSX_ID)
}

/// Returns true if the identifier type is a ref value.
pub fn is_ref_value_type(ty: &Type) -> bool {
    matches!(ty, Type::Object { shape_id: Some(id) } if *id == BUILT_IN_REF_VALUE_ID)
}

/// Returns true if the identifier type is useRef.
pub fn is_use_ref_type(ty: &Type) -> bool {
    matches!(ty, Type::Object { shape_id: Some(id) } if *id == BUILT_IN_USE_REF_ID)
}

/// Returns true if the type is a ref or ref value.
pub fn is_ref_or_ref_value(ty: &Type) -> bool {
    is_use_ref_type(ty) || is_ref_value_type(ty)
}

/// Returns true if the type is a useState result (BuiltInUseState).
pub fn is_use_state_type(ty: &Type) -> bool {
    matches!(ty, Type::Object { shape_id: Some(id) } if *id == object_shape::BUILT_IN_USE_STATE_ID)
}

/// Returns true if the type is a setState function (BuiltInSetState).
pub fn is_set_state_type(ty: &Type) -> bool {
    matches!(ty, Type::Function { shape_id: Some(id), .. } if *id == object_shape::BUILT_IN_SET_STATE_ID)
}

/// Returns true if the type is a useEffect hook.
pub fn is_use_effect_hook_type(ty: &Type) -> bool {
    matches!(ty, Type::Function { shape_id: Some(id), .. } if *id == object_shape::BUILT_IN_USE_EFFECT_HOOK_ID)
}

/// Returns true if the type is a useLayoutEffect hook.
pub fn is_use_layout_effect_hook_type(ty: &Type) -> bool {
    matches!(ty, Type::Function { shape_id: Some(id), .. } if *id == object_shape::BUILT_IN_USE_LAYOUT_EFFECT_HOOK_ID)
}

/// Returns true if the type is a useInsertionEffect hook.
pub fn is_use_insertion_effect_hook_type(ty: &Type) -> bool {
    matches!(ty, Type::Function { shape_id: Some(id), .. } if *id == object_shape::BUILT_IN_USE_INSERTION_EFFECT_HOOK_ID)
}

/// Returns true if the type is a useEffectEvent function.
pub fn is_use_effect_event_type(ty: &Type) -> bool {
    matches!(ty, Type::Function { shape_id: Some(id), .. } if *id == object_shape::BUILT_IN_USE_EFFECT_EVENT_ID)
}

/// Returns true if the type is a ref or ref-like mutable type (e.g. Reanimated shared values).
pub fn is_ref_or_ref_like_mutable_type(ty: &Type) -> bool {
    matches!(ty, Type::Object { shape_id: Some(id) }
        if *id == object_shape::BUILT_IN_USE_REF_ID || *id == object_shape::REANIMATED_SHARED_VALUE_ID)
}

/// Returns true if the type is the `use()` operator (React.use).
pub fn is_use_operator_type(ty: &Type) -> bool {
    matches!(
        ty,
        Type::Function { shape_id: Some(id), .. }
            if *id == BUILT_IN_USE_OPERATOR_ID
    )
}

/// Returns true if the type is a plain object (BuiltInObject).
pub fn is_plain_object_type(ty: &Type) -> bool {
    matches!(ty, Type::Object { shape_id: Some(id) } if *id == object_shape::BUILT_IN_OBJECT_ID)
}

/// Returns true if the type is a startTransition function (BuiltInStartTransition).
pub fn is_start_transition_type(ty: &Type) -> bool {
    matches!(ty, Type::Function { shape_id: Some(id), .. } if *id == object_shape::BUILT_IN_START_TRANSITION_ID)
}
