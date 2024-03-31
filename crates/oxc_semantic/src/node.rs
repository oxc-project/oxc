use petgraph::stable_graph::NodeIndex;

// TODO: remove `AstNodeTrait` alias when the `AstNode` struct got removed.
use oxc_ast::{AstKind, AstNode as AstNodeTrait};
use oxc_index::IndexVec;

use crate::scope::ScopeId;

pub use oxc_syntax::node::{AstNodeId, NodeFlags};

/// Semantic node contains all the semantic information about an ast node.
#[derive(Debug, Clone, Copy)]
pub struct AstNode<'a> {
    id: AstNodeId,
    /// A pointer to the ast node, which resides in the `bumpalo` memory arena.
    kind: AstKind<'a>,

    /// Associated Scope (initialized by binding)
    scope_id: ScopeId,

    /// Associated NodeIndex in CFG (initialized by control_flow)
    cfg_ix: NodeIndex,

    flags: NodeFlags,
}

impl<'a> AstNode<'a> {
    pub fn new(kind: AstKind<'a>, scope_id: ScopeId, cfg_ix: NodeIndex, flags: NodeFlags) -> Self {
        Self { id: AstNodeId::new(0), kind, scope_id, cfg_ix, flags }
    }

    pub fn id(&self) -> AstNodeId {
        #[allow(clippy::enum_glob_use)]
        use AstKind::*;
        match self.kind {
            // These absoulutly have to change.
            // | JSXMemberExpressionObject(_) // enum, refactored as the subject of this experiment
            | PropertyKey(_) // enum
            | Argument(_) // enum
            | AssignmentTarget(_) // enum
            | SimpleAssignmentTarget(_) // enum
            | ArrayExpressionElement(_) // enum
            | ExpressionArrayElement(_) // enum
            | ModuleDeclaration(_) // enum
            | JSXElementName(_) // enum
            // | FinallyClause(_) // block,
                                  // fixable via minor refactoring in 2 of our linter rules.
                                  // isn't related to the issue at hand and is already fixed in
                                  // this experiment.
                => self.id,
            _ => self.kind.ast_node_id().unwrap_or(AstNodeId::new(0)),
        }
        // These are good to go, Some are using enums but no linter rules are based on them so we
        // can change our assumptions but I prefer to refactor all enums.
        //
        // | JSXNamespacedName(_)
        // | Program(_)
        // | Directive(_)
        // | Hashbang(_)
        // | BlockStatement(_)
        // | BreakStatement(_)
        // | ContinueStatement(_)
        // | DebuggerStatement(_)
        // | DoWhileStatement(_)
        // | EmptyStatement(_)
        // | ExpressionStatement(_)
        // | ForInStatement(_)
        // | ForOfStatement(_)
        // | ForStatement(_)
        // | ForStatementInit(_)
        // | IfStatement(_)
        // | LabeledStatement(_)
        // | ReturnStatement(_)
        // | SwitchStatement(_)
        // | ThrowStatement(_)
        // | TryStatement(_)
        // | WhileStatement(_)
        // | WithStatement(_)
        // | SwitchCase(_)
        // | CatchClause(_)
        // | VariableDeclaration(_)
        // | VariableDeclarator(_)
        // | UsingDeclaration(_)
        // | IdentifierName(_)
        // | IdentifierReference(_)
        // | BindingIdentifier(_)
        // | LabelIdentifier(_)
        // | PrivateIdentifier(_)
        // | NumericLiteral(_)
        // | StringLiteral(_)
        // | BooleanLiteral(_)
        // | NullLiteral(_)
        // | BigintLiteral(_)
        // | RegExpLiteral(_)
        // | TemplateLiteral(_)
        // | MetaProperty(_)
        // | Super(_)
        // | ArrayExpression(_)
        // | ArrowFunctionExpression(_)
        // | AssignmentExpression(_)
        // | AwaitExpression(_)
        // | BinaryExpression(_)
        // | CallExpression(_)
        // | ChainExpression(_)
        // | ConditionalExpression(_)
        // | LogicalExpression(_)
        // | MemberExpression(_)
        // | NewExpression(_)
        // | ObjectExpression(_)
        // | ParenthesizedExpression(_)
        // | SequenceExpression(_)
        // | TaggedTemplateExpression(_)
        // | ThisExpression(_)
        // | UnaryExpression(_)
        // | UpdateExpression(_)
        // | YieldExpression(_)
        // | ImportExpression(_)
        // | PrivateInExpression(_)
        // | ObjectProperty(_)
        // | AssignmentTargetWithDefault(_)
        // | Elision(_)
        // | SpreadElement(_)
        // | BindingRestElement(_)
        // | Function(_)
        // | FunctionBody(_)
        // | FormalParameters(_)
        // | FormalParameter(_)
        // | Class(_)
        // | ClassBody(_)
        // | ClassHeritage(_)
        // | StaticBlock(_)
        // | PropertyDefinition(_)
        // | MethodDefinition(_)
        // | ArrayPattern(_)
        // | ObjectPattern(_)
        // | AssignmentPattern(_)
        // | Decorator(_)
        // | ImportDeclaration(_)
        // | ImportSpecifier(_)
        // | ImportDefaultSpecifier(_)
        // | ImportNamespaceSpecifier(_)
        // | ExportDefaultDeclaration(_)
        // | ExportNamedDeclaration(_)
        // | ExportAllDeclaration(_)
        // | JSXElement(_)
        // | JSXFragment(_)
        // | JSXOpeningElement(_)
        // | JSXClosingElement(_)
        // | JSXExpressionContainer(_)
        // | JSXAttributeItem(_)
        // | JSXSpreadAttribute(_)
        // | JSXText(_)
        // | JSXIdentifier(_)
        // | JSXMemberExpression(_)
        // | TSModuleBlock(_)
        // | TSAnyKeyword(_)
        // | TSIntersectionType(_)
        // | TSLiteralType(_)
        // | TSMethodSignature(_)
        // | TSNullKeyword(_)
        // | TSTypeLiteral(_)
        // | TSTypeReference(_)
        // | TSUnionType(_)
        // | TSVoidKeyword(_)
        // | TSIndexedAccessType(_)
        // | TSAsExpression(_)
        // | TSSatisfiesExpression(_)
        // | TSNonNullExpression(_)
        // | TSInstantiationExpression(_)
        // | TSEnumDeclaration(_)
        // | TSEnumMember(_)
        // | TSImportEqualsDeclaration(_)
        // | TSTypeName(_)
        // | TSExternalModuleReference(_)
        // | TSQualifiedName(_)
        // | TSInterfaceDeclaration(_)
        // | TSModuleDeclaration(_)
        // | TSTypeAliasDeclaration(_)
        // | TSTypeAnnotation(_)
        // | TSTypeQuery(_)
        // | TSTypeAssertion(_)
        // | TSTypeParameter(_)
        // | TSTypeParameterDeclaration(_)
        // | TSTypeParameterInstantiation(_)
        // | TSImportType(_)
        // | TSPropertySignature(_)
    }

    pub fn cfg_ix(&self) -> NodeIndex {
        self.cfg_ix
    }

    pub fn kind(&self) -> AstKind<'a> {
        self.kind
    }

    pub fn scope_id(&self) -> ScopeId {
        self.scope_id
    }

    pub fn flags(&self) -> NodeFlags {
        self.flags
    }

    pub fn flags_mut(&mut self) -> &mut NodeFlags {
        &mut self.flags
    }
}

/// Untyped AST nodes flattened into an vec
#[derive(Debug, Default)]
pub struct AstNodes<'a> {
    nodes: IndexVec<AstNodeId, AstNode<'a>>,
    parent_ids: IndexVec<AstNodeId, Option<AstNodeId>>,
}

impl<'a> AstNodes<'a> {
    pub fn iter(&self) -> impl Iterator<Item = &AstNode<'a>> + '_ {
        self.nodes.iter()
    }

    /// Walk up the AST, iterating over each parent node.
    ///
    /// The first node produced by this iterator is the first parent of the node
    /// pointed to by `node_id`. The last node will usually be a `Program`.
    pub fn iter_parents(&self, node_id: AstNodeId) -> impl Iterator<Item = &AstNode<'a>> + '_ {
        let curr = Some(self.get_node(node_id));
        AstNodeParentIter { curr, nodes: self }
    }

    pub fn kind(&self, ast_node_id: AstNodeId) -> AstKind<'a> {
        self.nodes[ast_node_id].kind
    }

    pub fn parent_id(&self, ast_node_id: AstNodeId) -> Option<AstNodeId> {
        self.parent_ids[ast_node_id]
    }

    pub fn parent_kind(&self, ast_node_id: AstNodeId) -> Option<AstKind<'a>> {
        self.parent_id(ast_node_id).map(|node_id| self.kind(node_id))
    }

    pub fn parent_node(&self, ast_node_id: AstNodeId) -> Option<&AstNode<'a>> {
        self.parent_id(ast_node_id).map(|node_id| self.get_node(node_id))
    }

    pub fn get_node(&self, ast_node_id: AstNodeId) -> &AstNode<'a> {
        &self.nodes[ast_node_id]
    }

    pub fn get_node_mut(&mut self, ast_node_id: AstNodeId) -> &mut AstNode<'a> {
        &mut self.nodes[ast_node_id]
    }

    /// Walk up the AST, iterating over each parent node.
    ///
    /// The first node produced by this iterator is the first parent of the node
    /// pointed to by `node_id`. The last node will usually be a `Program`.
    pub fn ancestors(&self, ast_node_id: AstNodeId) -> impl Iterator<Item = AstNodeId> + '_ {
        let parent_ids = &self.parent_ids;
        std::iter::successors(Some(ast_node_id), |node_id| parent_ids[*node_id])
    }

    pub fn add_node(&mut self, node: AstNode<'a>, parent_id: Option<AstNodeId>) -> AstNodeId {
        let mut node = node;
        let ast_node_id = self.parent_ids.push(parent_id);
        node.id = ast_node_id;
        node.kind.set_ast_node_id(Some(ast_node_id));
        self.nodes.push(node);
        ast_node_id
    }
}

#[derive(Debug)]
pub struct AstNodeParentIter<'s, 'a> {
    curr: Option<&'s AstNode<'a>>,
    nodes: &'s AstNodes<'a>,
}

impl<'s, 'a> Iterator for AstNodeParentIter<'s, 'a> {
    type Item = &'s AstNode<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        let next = self.curr;
        self.curr = self.curr.and_then(|curr| self.nodes.parent_node(curr.id()));

        next
    }
}
