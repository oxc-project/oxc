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

use crate::react_compiler_utils::OrderedMap;
use crate::react_compiler_utils::ordered_map::{ArenaOrderedMap, ArenaOrderedSet};
use oxc_allocator::{Allocator, CloneIn, CloneInSemanticIds, Vec as ArenaVec};
use oxc_ast::ast::*;
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

define_nonmax_u32_index_type! {
    /// Stable identity for an aliasing-effect diagnostic's analysis-relevant fields.
    pub struct DiagnosticId;
}

define_nonmax_u32_index_type! {
    /// Index into `Environment`'s source-specific aliasing-effect diagnostics.
    pub struct DiagnosticInstanceId;
}

/// Separates an aliasing diagnostic's analysis identity from its source-specific
/// payload. Effect interning uses `identity`, while emission uses `instance`.
#[derive(Debug, Clone, Copy)]
pub struct AliasingDiagnostic {
    identity: DiagnosticId,
    instance: DiagnosticInstanceId,
}

impl AliasingDiagnostic {
    pub(crate) fn new(identity: DiagnosticId, instance: DiagnosticInstanceId) -> Self {
        Self { identity, instance }
    }

    pub(crate) fn identity(self) -> DiagnosticId {
        self.identity
    }

    pub(crate) fn instance(self) -> DiagnosticInstanceId {
        self.instance
    }
}

impl BlockId {
    /// The entry block of a function's CFG. Blocks are numbered from 0, so the first
    /// block allocated is always the entry.
    pub const ENTRY: Self = Self::from_usize(0);

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
#[derive(Debug)]
pub struct HirFunction<'a> {
    pub span: Option<Span>,
    pub id: Option<Ident<'a>>,
    pub name_hint: Option<Ident<'a>>,
    pub fn_type: ReactFunctionType,
    pub params: ArenaVec<'a, ParamPattern>,
    pub returns: Place,
    pub context: ArenaVec<'a, Place>,
    pub body: HIR<'a>,
    pub instructions: ArenaVec<'a, Instruction<'a>>,
    pub generator: bool,
    pub is_async: bool,
    pub directives: ArenaVec<'a, Str<'a>>,
    pub aliasing_effects: Option<ArenaVec<'a, AliasingEffect<'a>>>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReactFunctionType {
    Component,
    Hook,
    Other,
}

#[derive(Debug, Clone, Copy)]
pub enum ParamPattern {
    Place(Place),
    Spread(SpreadPattern),
}

/// The HIR control-flow graph
#[derive(Debug)]
#[allow(clippy::upper_case_acronyms)]
pub struct HIR<'a> {
    pub entry: BlockId,
    pub blocks: OrderedMap<BlockId, BasicBlock<'a>>,
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
#[derive(Debug)]
pub struct BasicBlock<'a> {
    pub kind: BlockKind,
    pub id: BlockId,
    pub instructions: ArenaVec<'a, InstructionId>,
    pub terminal: Terminal<'a>,
    pub preds: ArenaOrderedSet<'a, BlockId>,
    pub phis: ArenaVec<'a, Phi<'a>>,
}

/// Phi node for SSA
#[derive(Debug)]
pub struct Phi<'a> {
    pub place: Place,
    pub operands: ArenaOrderedMap<'a, BlockId, Place>,
}

// =============================================================================
// Terminal enum
// =============================================================================

#[derive(Debug)]
pub enum Terminal<'a> {
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
        effects: Option<ArenaVec<'a, AliasingEffect<'a>>>,
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
        cases: ArenaVec<'a, Case>,
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
        effects: Option<ArenaVec<'a, AliasingEffect<'a>>>,
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

impl<'a> Terminal<'a> {
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

#[derive(Debug, Clone, Copy)]
pub struct Case {
    pub test: Option<Place>,
    pub block: BlockId,
}

// =============================================================================
// Instruction types
// =============================================================================

#[derive(Debug)]
pub struct Instruction<'a> {
    pub id: EvaluationOrder,
    pub lvalue: Place,
    pub value: InstructionValue<'a>,
    pub span: Option<Span>,
    pub effects: Option<ArenaVec<'a, AliasingEffect<'a>>>,
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

#[derive(Debug, Clone, Copy)]
pub struct LValue {
    pub place: Place,
    pub kind: InstructionKind,
}

#[derive(Debug)]
pub struct LValuePattern<'a> {
    pub pattern: Pattern<'a>,
    pub kind: InstructionKind,
}

#[derive(Debug)]
pub enum Pattern<'a> {
    Array(ArrayPattern<'a>),
    Object(ObjectPattern<'a>),
}

// =============================================================================
// InstructionValue enum
// =============================================================================

#[derive(Debug)]
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
        args: ArenaVec<'a, PlaceOrSpread>,
        span: Option<Span>,
    },
    CallExpression {
        callee: Place,
        args: ArenaVec<'a, PlaceOrSpread>,
        span: Option<Span>,
    },
    MethodCall {
        receiver: Place,
        property: Place,
        args: ArenaVec<'a, PlaceOrSpread>,
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
        props: ArenaVec<'a, JsxAttribute<'a>>,
        children: Option<ArenaVec<'a, Place>>,
        span: Option<Span>,
        opening_span: Option<Span>,
        closing_span: Option<Span>,
    },
    ObjectExpression {
        properties: ArenaVec<'a, ObjectPropertyOrSpread<'a>>,
        span: Option<Span>,
    },
    ObjectMethod {
        span: Option<Span>,
        lowered_func: LoweredFunction,
    },
    ArrayExpression {
        elements: ArenaVec<'a, ArrayElement>,
        span: Option<Span>,
    },
    JsxFragment {
        children: ArenaVec<'a, Place>,
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
        quasis: ArenaVec<'a, TemplateQuasi<'a>>,
        subexprs: ArenaVec<'a, Place>,
        span: Option<Span>,
    },
    TemplateLiteral {
        subexprs: ArenaVec<'a, Place>,
        quasis: ArenaVec<'a, TemplateQuasi<'a>>,
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
        deps: Option<ArenaVec<'a, ManualMemoDependency<'a>>>,
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

#[derive(Debug, Clone, Copy)]
pub struct TemplateQuasi<'a> {
    pub raw: Str<'a>,
    pub cooked: Option<Str<'a>>,
}

#[derive(Debug)]
pub struct ManualMemoDependency<'a> {
    pub root: ManualMemoDependencyRoot<'a>,
    pub path: ArenaVec<'a, DependencyPathEntry<'a>>,
    pub span: Option<Span>,
}

#[derive(Debug, Clone, Copy)]
pub enum ManualMemoDependencyRoot<'a> {
    NamedLocal { value: Place, constant: bool },
    Global { identifier_name: Ident<'a> },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DependencyPathEntry<'a> {
    pub property: PropertyLiteral<'a>,
    pub optional: bool,
    pub span: Option<Span>,
}

// =============================================================================
// Place, Identifier, and related types
// =============================================================================

#[derive(Debug, Clone, Copy)]
pub struct Place {
    pub identifier: IdentifierId,
    pub effect: Effect,
    pub reactive: bool,
    pub span: Option<Span>,
}

#[derive(Debug, Clone, Copy)]
pub struct Identifier<'a> {
    pub id: IdentifierId,
    pub declaration_id: DeclarationId,
    pub name: Option<IdentifierName<'a>>,
    pub mutable_range: MutableRange,
    pub scope: Option<ScopeId>,
    pub type_: TypeId,
    pub span: Option<Span>,
}

#[derive(Debug, Clone, Copy)]
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

#[derive(Debug, Clone, Copy)]
pub struct SpreadPattern {
    pub place: Place,
}

#[derive(Debug)]
pub struct ArrayPattern<'a> {
    pub items: ArenaVec<'a, ArrayPatternElement>,
}

#[derive(Debug, Clone, Copy)]
pub enum ArrayPatternElement {
    Place(Place),
    Spread(SpreadPattern),
    Hole,
}

#[derive(Debug)]
pub struct ObjectPattern<'a> {
    pub properties: ArenaVec<'a, ObjectPropertyOrSpread<'a>>,
}

#[derive(Debug, Clone, Copy)]
pub enum ObjectPropertyOrSpread<'a> {
    Property(ObjectProperty<'a>),
    Spread(SpreadPattern),
}

#[derive(Debug, Clone, Copy)]
pub struct ObjectProperty<'a> {
    pub key: ObjectPropertyKey<'a>,
    pub property_type: ObjectPropertyType,
    pub place: Place,
}

#[derive(Debug, Clone, Copy)]
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

#[derive(Debug, Clone, Copy)]
pub enum PlaceOrSpread {
    Place(Place),
    Spread(SpreadPattern),
}

#[derive(Debug, Clone, Copy)]
pub enum ArrayElement {
    Place(Place),
    Spread(SpreadPattern),
    Hole,
}

#[derive(Debug, Clone, Copy)]
pub struct LoweredFunction {
    pub func: FunctionId,
}

#[derive(Debug, Clone, Copy)]
pub struct BuiltinTag<'a> {
    pub name: Ident<'a>,
}

#[derive(Debug, Clone, Copy)]
pub enum JsxTag<'a> {
    Place(Place),
    Builtin(BuiltinTag<'a>),
}

#[derive(Debug, Clone, Copy)]
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

#[derive(Debug, Clone, Copy)]
pub enum VariableBinding<'a> {
    Identifier { identifier: IdentifierId, binding_kind: BindingKind },
    Global { name: Ident<'a> },
    ImportDefault { name: Ident<'a>, module: Str<'a> },
    ImportSpecifier { name: Ident<'a>, module: Str<'a>, imported: Ident<'a> },
    ImportNamespace { name: Ident<'a>, module: Str<'a> },
    ModuleLocal { name: Ident<'a> },
}

#[derive(Debug, Clone, Copy)]
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
    Var {
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

#[derive(Debug, Clone, Copy)]
pub enum PropertyNameKind<'a> {
    Literal { value: PropertyLiteral<'a> },
    Computed,
}

// =============================================================================
// ReactiveScope
// =============================================================================

#[derive(Debug)]
pub struct ReactiveScope<'a> {
    pub id: ScopeId,
    pub range: MutableRange,

    /// The inputs to this reactive scope (populated by later passes)
    pub dependencies: ArenaVec<'a, ReactiveScopeDependency<'a>>,

    /// The set of values produced by this scope (populated by later passes)
    pub declarations: ArenaVec<'a, (IdentifierId, ReactiveScopeDeclaration)>,

    /// Identifiers which are reassigned by this scope (populated by later passes)
    pub reassignments: ArenaVec<'a, IdentifierId>,

    /// If the scope contains an early return, this stores info about it (populated by later passes)
    pub early_return_value: Option<ReactiveScopeEarlyReturn>,

    /// Scopes that were merged into this one (populated by later passes)
    pub merged: ArenaVec<'a, ScopeId>,

    /// Source location spanning the scope
    pub span: Option<Span>,
}

/// A dependency of a reactive scope.
#[derive(Debug)]
pub struct ReactiveScopeDependency<'a> {
    pub identifier: IdentifierId,
    pub reactive: bool,
    pub path: ArenaVec<'a, DependencyPathEntry<'a>>,
    pub span: Option<Span>,
}

/// A declaration produced by a reactive scope.
#[derive(Debug, Clone, Copy)]
pub struct ReactiveScopeDeclaration {
    pub identifier: IdentifierId,
    pub scope: ScopeId,
}

// --- Arena `CloneIn` for the `ReactiveScope` subtree ------------------------
// Arena `Vec` is not `Clone`. HIR clones are same-arena and `Ident`/leaves borrow
// the source AST, so `Copy` ids/leaves clone via `*self` and only the arena `Vec`s
// recurse. Construction (`new_in`/`from_iter_in`) takes `&alloc`; `clone_in` takes
// `alloc`.

/// Trivial `CloneIn` for `Copy` HIR leaves/ids used inside arena `Vec`s.
macro_rules! impl_trivial_clone_in {
    ($($ty:ty),+ $(,)?) => {
        $(impl<'a> CloneIn<'a> for $ty {
            type Cloned = $ty;
            #[inline(always)]
            fn clone_in_impl(&self, _: CloneInSemanticIds, _: &'a Allocator) -> $ty {
                *self
            }
        })+
    };
}
impl_trivial_clone_in!(
    DependencyPathEntry<'a>,
    IdentifierId,
    ScopeId,
    BlockId,
    InstructionId,
    EvaluationOrder,
    TypeId,
    FunctionId,
    MutableRangeId,
    DiagnosticId,
    DiagnosticInstanceId,
    AliasingDiagnostic,
    DeclarationId,
    Place,
    Case,
    PlaceOrSpread,
    PlaceOrSpreadOrHole,
    ArrayElement,
    ArrayPatternElement,
    ObjectPropertyOrSpread<'a>,
    JsxAttribute<'a>,
    TemplateQuasi<'a>,
    ParamPattern,
    SpreadPattern,
);

impl<'a> CloneIn<'a> for ReactiveScopeDependency<'a> {
    type Cloned = ReactiveScopeDependency<'a>;
    fn clone_in_impl(&self, sem: CloneInSemanticIds, alloc: &'a Allocator) -> Self {
        ReactiveScopeDependency {
            identifier: self.identifier,
            reactive: self.reactive,
            path: self.path.clone_in_impl(sem, alloc),
            span: self.span,
        }
    }
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

use crate::react_compiler_hir::type_config::ValueKind;
use crate::react_compiler_hir::type_config::ValueReason;

/// Reason for a mutation, used for generating hints (e.g. rename to "Ref").
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MutationReason {
    AssignCurrentProperty,
}

/// Describes the aliasing/mutation/data-flow effects of an instruction or terminal.
/// Ported from TS `AliasingEffect` in `AliasingEffects.ts`.
#[derive(Debug)]
pub enum AliasingEffect<'a> {
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
        args: ArenaVec<'a, PlaceOrSpreadOrHole>,
        into: Place,
        /// Callee function `TypeId`, used to resolve the `FunctionSignature` from the
        /// environment on demand. Storing the id (rather than an `Rc<FunctionSignature>`)
        /// keeps this `Copy`/non-`Drop`, so `AliasingEffect` can be arena-allocated.
        signature: Option<TypeId>,
        span: Option<Span>,
    },
    /// Function expression creation with captures.
    CreateFunction { captures: ArenaVec<'a, Place>, function_id: FunctionId, into: Place },
    /// Mutation of a value known to be frozen (error).
    ///
    /// `error` references the diagnostic interned on `Environment`; the
    /// `OxcDiagnostic` itself is held out-of-band so this effect stays
    /// `Copy`/non-`Drop` and can be arena-allocated.
    MutateFrozen { place: Place, error: AliasingDiagnostic },
    /// Mutation of a global value (error).
    MutateGlobal { place: Place, error: AliasingDiagnostic },
    /// Side-effect not safe during render.
    Impure { place: Place, error: AliasingDiagnostic },
    /// Value is accessed during render.
    Render { place: Place },
}

/// Combined Place/Spread/Hole for Apply args.
#[derive(Debug, Clone, Copy)]
pub enum PlaceOrSpreadOrHole {
    Place(Place),
    Spread(SpreadPattern),
    Hole,
}

/// Aliasing signature for function calls.
/// Ported from TS `AliasingSignature` in `AliasingEffects.ts`.
#[derive(Debug)]
pub struct AliasingSignature<'a> {
    pub receiver: IdentifierId,
    pub params: ArenaVec<'a, IdentifierId>,
    pub rest: Option<IdentifierId>,
    pub returns: IdentifierId,
    pub effects: ArenaVec<'a, AliasingEffect<'a>>,
    pub temporaries: ArenaVec<'a, Place>,
}

// =============================================================================
// Arena `CloneIn` for the arena-carrying HIR types (same-arena, `Cloned = Self`)
// =============================================================================
// Copy leaves are copied by direct field assignment; only arena `Vec` /
// `ArenaOrderedMap` / `ArenaOrderedSet` fields recurse via `clone_in_impl`.
// `Option<ArenaVec<..>>` clones via `self.x.as_ref().map(|v| v.clone_in_impl(..))`.

impl<'a> CloneIn<'a> for HirFunction<'a> {
    type Cloned = HirFunction<'a>;
    fn clone_in_impl(&self, sem: CloneInSemanticIds, alloc: &'a Allocator) -> Self {
        HirFunction {
            span: self.span,
            id: self.id,
            name_hint: self.name_hint,
            fn_type: self.fn_type,
            params: self.params.clone_in_impl(sem, alloc),
            returns: self.returns,
            context: self.context.clone_in_impl(sem, alloc),
            body: self.body.clone_in_impl(sem, alloc),
            instructions: self.instructions.clone_in_impl(sem, alloc),
            generator: self.generator,
            is_async: self.is_async,
            directives: self.directives.clone_in_impl(sem, alloc),
            aliasing_effects: self.aliasing_effects.as_ref().map(|v| v.clone_in_impl(sem, alloc)),
        }
    }
}

impl<'a> CloneIn<'a> for HIR<'a> {
    type Cloned = HIR<'a>;
    fn clone_in_impl(&self, sem: CloneInSemanticIds, alloc: &'a Allocator) -> Self {
        let mut blocks = OrderedMap::default();
        for (id, block) in self.blocks.iter() {
            blocks.insert(*id, block.clone_in_impl(sem, alloc));
        }
        HIR { entry: self.entry, blocks }
    }
}

impl<'a> CloneIn<'a> for BasicBlock<'a> {
    type Cloned = BasicBlock<'a>;
    fn clone_in_impl(&self, sem: CloneInSemanticIds, alloc: &'a Allocator) -> Self {
        BasicBlock {
            kind: self.kind,
            id: self.id,
            instructions: self.instructions.clone_in_impl(sem, alloc),
            terminal: self.terminal.clone_in_impl(sem, alloc),
            preds: self.preds.clone_in_impl(sem, alloc),
            phis: self.phis.clone_in_impl(sem, alloc),
        }
    }
}

impl<'a> CloneIn<'a> for Phi<'a> {
    type Cloned = Phi<'a>;
    fn clone_in_impl(&self, sem: CloneInSemanticIds, alloc: &'a Allocator) -> Self {
        Phi { place: self.place, operands: self.operands.clone_in_impl(sem, alloc) }
    }
}

impl<'a> CloneIn<'a> for Terminal<'a> {
    type Cloned = Terminal<'a>;
    fn clone_in_impl(&self, sem: CloneInSemanticIds, alloc: &'a Allocator) -> Self {
        match self {
            Terminal::Unreachable { id, span } => Terminal::Unreachable { id: *id, span: *span },
            Terminal::Throw { value, id, span } => {
                Terminal::Throw { value: *value, id: *id, span: *span }
            }
            Terminal::Return { value, return_variant, id, span, effects } => Terminal::Return {
                value: *value,
                return_variant: *return_variant,
                id: *id,
                span: *span,
                effects: effects.as_ref().map(|v| v.clone_in_impl(sem, alloc)),
            },
            Terminal::Goto { block, variant, id, span } => {
                Terminal::Goto { block: *block, variant: *variant, id: *id, span: *span }
            }
            Terminal::If { test, consequent, alternate, fallthrough, id, span } => Terminal::If {
                test: *test,
                consequent: *consequent,
                alternate: *alternate,
                fallthrough: *fallthrough,
                id: *id,
                span: *span,
            },
            Terminal::Branch { test, consequent, alternate, fallthrough, id, span } => {
                Terminal::Branch {
                    test: *test,
                    consequent: *consequent,
                    alternate: *alternate,
                    fallthrough: *fallthrough,
                    id: *id,
                    span: *span,
                }
            }
            Terminal::Switch { test, cases, fallthrough, id, span } => Terminal::Switch {
                test: *test,
                cases: cases.clone_in_impl(sem, alloc),
                fallthrough: *fallthrough,
                id: *id,
                span: *span,
            },
            Terminal::DoWhile { loop_block, test, fallthrough, id, span } => Terminal::DoWhile {
                loop_block: *loop_block,
                test: *test,
                fallthrough: *fallthrough,
                id: *id,
                span: *span,
            },
            Terminal::While { test, loop_block, fallthrough, id, span } => Terminal::While {
                test: *test,
                loop_block: *loop_block,
                fallthrough: *fallthrough,
                id: *id,
                span: *span,
            },
            Terminal::For { init, test, update, loop_block, fallthrough, id, span } => {
                Terminal::For {
                    init: *init,
                    test: *test,
                    update: *update,
                    loop_block: *loop_block,
                    fallthrough: *fallthrough,
                    id: *id,
                    span: *span,
                }
            }
            Terminal::ForOf { init, test, loop_block, fallthrough, id, span } => Terminal::ForOf {
                init: *init,
                test: *test,
                loop_block: *loop_block,
                fallthrough: *fallthrough,
                id: *id,
                span: *span,
            },
            Terminal::ForIn { init, loop_block, fallthrough, id, span } => Terminal::ForIn {
                init: *init,
                loop_block: *loop_block,
                fallthrough: *fallthrough,
                id: *id,
                span: *span,
            },
            Terminal::Logical { operator, test, fallthrough, id, span } => Terminal::Logical {
                operator: *operator,
                test: *test,
                fallthrough: *fallthrough,
                id: *id,
                span: *span,
            },
            Terminal::Ternary { test, fallthrough, id, span } => {
                Terminal::Ternary { test: *test, fallthrough: *fallthrough, id: *id, span: *span }
            }
            Terminal::Optional { optional, test, fallthrough, id, span } => Terminal::Optional {
                optional: *optional,
                test: *test,
                fallthrough: *fallthrough,
                id: *id,
                span: *span,
            },
            Terminal::Label { block, fallthrough, id, span } => {
                Terminal::Label { block: *block, fallthrough: *fallthrough, id: *id, span: *span }
            }
            Terminal::Sequence { block, fallthrough, id, span } => Terminal::Sequence {
                block: *block,
                fallthrough: *fallthrough,
                id: *id,
                span: *span,
            },
            Terminal::MaybeThrow { continuation, handler, id, span, effects } => {
                Terminal::MaybeThrow {
                    continuation: *continuation,
                    handler: *handler,
                    id: *id,
                    span: *span,
                    effects: effects.as_ref().map(|v| v.clone_in_impl(sem, alloc)),
                }
            }
            Terminal::Try { block, handler_binding, handler, fallthrough, id, span } => {
                Terminal::Try {
                    block: *block,
                    handler_binding: *handler_binding,
                    handler: *handler,
                    fallthrough: *fallthrough,
                    id: *id,
                    span: *span,
                }
            }
            Terminal::Scope { fallthrough, block, scope, id, span } => Terminal::Scope {
                fallthrough: *fallthrough,
                block: *block,
                scope: *scope,
                id: *id,
                span: *span,
            },
            Terminal::PrunedScope { fallthrough, block, scope, id, span } => {
                Terminal::PrunedScope {
                    fallthrough: *fallthrough,
                    block: *block,
                    scope: *scope,
                    id: *id,
                    span: *span,
                }
            }
        }
    }
}

impl<'a> CloneIn<'a> for Instruction<'a> {
    type Cloned = Instruction<'a>;
    fn clone_in_impl(&self, sem: CloneInSemanticIds, alloc: &'a Allocator) -> Self {
        Instruction {
            id: self.id,
            lvalue: self.lvalue,
            value: self.value.clone_in_impl(sem, alloc),
            span: self.span,
            effects: self.effects.as_ref().map(|v| v.clone_in_impl(sem, alloc)),
        }
    }
}

impl<'a> CloneIn<'a> for InstructionValue<'a> {
    type Cloned = InstructionValue<'a>;
    fn clone_in_impl(&self, sem: CloneInSemanticIds, alloc: &'a Allocator) -> Self {
        match self {
            // --- Copy-only variants: field-copy ---
            InstructionValue::LoadLocal { place, span } => {
                InstructionValue::LoadLocal { place: *place, span: *span }
            }
            InstructionValue::LoadContext { place, span } => {
                InstructionValue::LoadContext { place: *place, span: *span }
            }
            InstructionValue::DeclareLocal { lvalue, span } => {
                InstructionValue::DeclareLocal { lvalue: *lvalue, span: *span }
            }
            InstructionValue::DeclareContext { lvalue, span } => {
                InstructionValue::DeclareContext { lvalue: *lvalue, span: *span }
            }
            InstructionValue::StoreLocal { lvalue, value, span } => {
                InstructionValue::StoreLocal { lvalue: *lvalue, value: *value, span: *span }
            }
            InstructionValue::StoreContext { lvalue, value, span } => {
                InstructionValue::StoreContext { lvalue: *lvalue, value: *value, span: *span }
            }
            InstructionValue::Primitive { value, span } => {
                InstructionValue::Primitive { value: *value, span: *span }
            }
            InstructionValue::JSXText { value, span } => {
                InstructionValue::JSXText { value: *value, span: *span }
            }
            InstructionValue::BinaryExpression { operator, left, right, span } => {
                InstructionValue::BinaryExpression {
                    operator: *operator,
                    left: *left,
                    right: *right,
                    span: *span,
                }
            }
            InstructionValue::UnaryExpression { operator, value, span } => {
                InstructionValue::UnaryExpression {
                    operator: *operator,
                    value: *value,
                    span: *span,
                }
            }
            InstructionValue::TypeCastExpression { value, cast, span } => {
                InstructionValue::TypeCastExpression { value: *value, cast: *cast, span: *span }
            }
            InstructionValue::ObjectMethod { span, lowered_func } => {
                InstructionValue::ObjectMethod { span: *span, lowered_func: *lowered_func }
            }
            InstructionValue::RegExpLiteral { pattern, flags, span } => {
                InstructionValue::RegExpLiteral { pattern: *pattern, flags: *flags, span: *span }
            }
            InstructionValue::MetaProperty { meta, property, span } => {
                InstructionValue::MetaProperty { meta: *meta, property: *property, span: *span }
            }
            InstructionValue::PropertyStore { object, property, value, span } => {
                InstructionValue::PropertyStore {
                    object: *object,
                    property: *property,
                    value: *value,
                    span: *span,
                }
            }
            InstructionValue::PropertyLoad { object, property, span } => {
                InstructionValue::PropertyLoad { object: *object, property: *property, span: *span }
            }
            InstructionValue::PropertyDelete { object, property, span } => {
                InstructionValue::PropertyDelete {
                    object: *object,
                    property: *property,
                    span: *span,
                }
            }
            InstructionValue::ComputedStore { object, property, value, span } => {
                InstructionValue::ComputedStore {
                    object: *object,
                    property: *property,
                    value: *value,
                    span: *span,
                }
            }
            InstructionValue::ComputedLoad { object, property, span } => {
                InstructionValue::ComputedLoad { object: *object, property: *property, span: *span }
            }
            InstructionValue::ComputedDelete { object, property, span } => {
                InstructionValue::ComputedDelete {
                    object: *object,
                    property: *property,
                    span: *span,
                }
            }
            InstructionValue::LoadGlobal { binding, span } => {
                InstructionValue::LoadGlobal { binding: *binding, span: *span }
            }
            InstructionValue::StoreGlobal { name, value, span } => {
                InstructionValue::StoreGlobal { name: *name, value: *value, span: *span }
            }
            InstructionValue::FunctionExpression {
                name,
                name_hint,
                lowered_func,
                expr_type,
                span,
            } => InstructionValue::FunctionExpression {
                name: *name,
                name_hint: *name_hint,
                lowered_func: *lowered_func,
                expr_type: *expr_type,
                span: *span,
            },
            InstructionValue::Await { value, span } => {
                InstructionValue::Await { value: *value, span: *span }
            }
            InstructionValue::GetIterator { collection, span } => {
                InstructionValue::GetIterator { collection: *collection, span: *span }
            }
            InstructionValue::IteratorNext { iterator, collection, span } => {
                InstructionValue::IteratorNext {
                    iterator: *iterator,
                    collection: *collection,
                    span: *span,
                }
            }
            InstructionValue::NextPropertyOf { value, span } => {
                InstructionValue::NextPropertyOf { value: *value, span: *span }
            }
            InstructionValue::PrefixUpdate { lvalue, operation, value, span } => {
                InstructionValue::PrefixUpdate {
                    lvalue: *lvalue,
                    operation: *operation,
                    value: *value,
                    span: *span,
                }
            }
            InstructionValue::PostfixUpdate { lvalue, operation, value, span } => {
                InstructionValue::PostfixUpdate {
                    lvalue: *lvalue,
                    operation: *operation,
                    value: *value,
                    span: *span,
                }
            }
            InstructionValue::Debugger { span } => InstructionValue::Debugger { span: *span },
            InstructionValue::FinishMemoize { manual_memo_id, decl, pruned, span } => {
                InstructionValue::FinishMemoize {
                    manual_memo_id: *manual_memo_id,
                    decl: *decl,
                    pruned: *pruned,
                    span: *span,
                }
            }
            // --- Arena-carrying variants: recurse ---
            InstructionValue::Destructure { lvalue, value, span } => {
                InstructionValue::Destructure {
                    lvalue: lvalue.clone_in_impl(sem, alloc),
                    value: *value,
                    span: *span,
                }
            }
            InstructionValue::NewExpression { callee, args, span } => {
                InstructionValue::NewExpression {
                    callee: *callee,
                    args: args.clone_in_impl(sem, alloc),
                    span: *span,
                }
            }
            InstructionValue::CallExpression { callee, args, span } => {
                InstructionValue::CallExpression {
                    callee: *callee,
                    args: args.clone_in_impl(sem, alloc),
                    span: *span,
                }
            }
            InstructionValue::MethodCall { receiver, property, args, span } => {
                InstructionValue::MethodCall {
                    receiver: *receiver,
                    property: *property,
                    args: args.clone_in_impl(sem, alloc),
                    span: *span,
                }
            }
            InstructionValue::JsxExpression {
                tag,
                props,
                children,
                span,
                opening_span,
                closing_span,
            } => InstructionValue::JsxExpression {
                tag: *tag,
                props: props.clone_in_impl(sem, alloc),
                children: children.as_ref().map(|v| v.clone_in_impl(sem, alloc)),
                span: *span,
                opening_span: *opening_span,
                closing_span: *closing_span,
            },
            InstructionValue::ObjectExpression { properties, span } => {
                InstructionValue::ObjectExpression {
                    properties: properties.clone_in_impl(sem, alloc),
                    span: *span,
                }
            }
            InstructionValue::ArrayExpression { elements, span } => {
                InstructionValue::ArrayExpression {
                    elements: elements.clone_in_impl(sem, alloc),
                    span: *span,
                }
            }
            InstructionValue::JsxFragment { children, span } => InstructionValue::JsxFragment {
                children: children.clone_in_impl(sem, alloc),
                span: *span,
            },
            InstructionValue::TaggedTemplateExpression { tag, quasis, subexprs, span } => {
                InstructionValue::TaggedTemplateExpression {
                    tag: *tag,
                    quasis: quasis.clone_in_impl(sem, alloc),
                    subexprs: subexprs.clone_in_impl(sem, alloc),
                    span: *span,
                }
            }
            InstructionValue::TemplateLiteral { subexprs, quasis, span } => {
                InstructionValue::TemplateLiteral {
                    subexprs: subexprs.clone_in_impl(sem, alloc),
                    quasis: quasis.clone_in_impl(sem, alloc),
                    span: *span,
                }
            }
            InstructionValue::StartMemoize {
                manual_memo_id,
                deps,
                deps_span,
                has_invalid_deps,
                span,
            } => InstructionValue::StartMemoize {
                manual_memo_id: *manual_memo_id,
                deps: deps.as_ref().map(|v| v.clone_in_impl(sem, alloc)),
                deps_span: *deps_span,
                has_invalid_deps: *has_invalid_deps,
                span: *span,
            },
        }
    }
}

impl<'a> CloneIn<'a> for LValuePattern<'a> {
    type Cloned = LValuePattern<'a>;
    fn clone_in_impl(&self, sem: CloneInSemanticIds, alloc: &'a Allocator) -> Self {
        LValuePattern { pattern: self.pattern.clone_in_impl(sem, alloc), kind: self.kind }
    }
}

impl<'a> CloneIn<'a> for Pattern<'a> {
    type Cloned = Pattern<'a>;
    fn clone_in_impl(&self, sem: CloneInSemanticIds, alloc: &'a Allocator) -> Self {
        match self {
            Pattern::Array(p) => Pattern::Array(p.clone_in_impl(sem, alloc)),
            Pattern::Object(p) => Pattern::Object(p.clone_in_impl(sem, alloc)),
        }
    }
}

impl<'a> CloneIn<'a> for ArrayPattern<'a> {
    type Cloned = ArrayPattern<'a>;
    fn clone_in_impl(&self, sem: CloneInSemanticIds, alloc: &'a Allocator) -> Self {
        ArrayPattern { items: self.items.clone_in_impl(sem, alloc) }
    }
}

impl<'a> CloneIn<'a> for ObjectPattern<'a> {
    type Cloned = ObjectPattern<'a>;
    fn clone_in_impl(&self, sem: CloneInSemanticIds, alloc: &'a Allocator) -> Self {
        ObjectPattern { properties: self.properties.clone_in_impl(sem, alloc) }
    }
}

impl<'a> CloneIn<'a> for ManualMemoDependency<'a> {
    type Cloned = ManualMemoDependency<'a>;
    fn clone_in_impl(&self, sem: CloneInSemanticIds, alloc: &'a Allocator) -> Self {
        ManualMemoDependency {
            root: self.root,
            path: self.path.clone_in_impl(sem, alloc),
            span: self.span,
        }
    }
}

impl<'a> CloneIn<'a> for AliasingEffect<'a> {
    type Cloned = AliasingEffect<'a>;
    fn clone_in_impl(&self, sem: CloneInSemanticIds, alloc: &'a Allocator) -> Self {
        match self {
            AliasingEffect::Freeze { value, reason } => {
                AliasingEffect::Freeze { value: *value, reason: *reason }
            }
            AliasingEffect::Mutate { value, reason } => {
                AliasingEffect::Mutate { value: *value, reason: *reason }
            }
            AliasingEffect::MutateConditionally { value } => {
                AliasingEffect::MutateConditionally { value: *value }
            }
            AliasingEffect::MutateTransitive { value } => {
                AliasingEffect::MutateTransitive { value: *value }
            }
            AliasingEffect::MutateTransitiveConditionally { value } => {
                AliasingEffect::MutateTransitiveConditionally { value: *value }
            }
            AliasingEffect::Capture { from, into } => {
                AliasingEffect::Capture { from: *from, into: *into }
            }
            AliasingEffect::Alias { from, into } => {
                AliasingEffect::Alias { from: *from, into: *into }
            }
            AliasingEffect::MaybeAlias { from, into } => {
                AliasingEffect::MaybeAlias { from: *from, into: *into }
            }
            AliasingEffect::Assign { from, into } => {
                AliasingEffect::Assign { from: *from, into: *into }
            }
            AliasingEffect::Create { into, value, reason } => {
                AliasingEffect::Create { into: *into, value: *value, reason: *reason }
            }
            AliasingEffect::CreateFrom { from, into } => {
                AliasingEffect::CreateFrom { from: *from, into: *into }
            }
            AliasingEffect::ImmutableCapture { from, into } => {
                AliasingEffect::ImmutableCapture { from: *from, into: *into }
            }
            AliasingEffect::Apply {
                receiver,
                function,
                mutates_function,
                args,
                into,
                signature,
                span,
            } => AliasingEffect::Apply {
                receiver: *receiver,
                function: *function,
                mutates_function: *mutates_function,
                args: args.clone_in_impl(sem, alloc),
                into: *into,
                signature: *signature,
                span: *span,
            },
            AliasingEffect::CreateFunction { captures, function_id, into } => {
                AliasingEffect::CreateFunction {
                    captures: captures.clone_in_impl(sem, alloc),
                    function_id: *function_id,
                    into: *into,
                }
            }
            AliasingEffect::MutateFrozen { place, error } => {
                AliasingEffect::MutateFrozen { place: *place, error: *error }
            }
            AliasingEffect::MutateGlobal { place, error } => {
                AliasingEffect::MutateGlobal { place: *place, error: *error }
            }
            AliasingEffect::Impure { place, error } => {
                AliasingEffect::Impure { place: *place, error: *error }
            }
            AliasingEffect::Render { place } => AliasingEffect::Render { place: *place },
        }
    }
}

impl<'a> CloneIn<'a> for AliasingSignature<'a> {
    type Cloned = AliasingSignature<'a>;
    fn clone_in_impl(&self, sem: CloneInSemanticIds, alloc: &'a Allocator) -> Self {
        AliasingSignature {
            receiver: self.receiver,
            params: self.params.clone_in_impl(sem, alloc),
            rest: self.rest,
            returns: self.returns,
            effects: self.effects.clone_in_impl(sem, alloc),
            temporaries: self.temporaries.clone_in_impl(sem, alloc),
        }
    }
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
