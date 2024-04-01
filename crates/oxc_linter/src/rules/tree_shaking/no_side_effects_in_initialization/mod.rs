use oxc_ast::AstKind;
use oxc_diagnostics::{
    miette::{self, Diagnostic},
    thiserror::{self, Error},
};
use oxc_macros::declare_oxc_lint;
use oxc_span::{CompactStr, Span};

use crate::{context::LintContext, rule::Rule};

use self::listener_map::{ListenerMap, NodeListenerOptions};

mod listener_map;

#[derive(Debug, Error, Diagnostic)]
enum NoSideEffectsDiagnostic {
    #[error("eslint-plugin-tree-shaking(no-side-effects-in-initialization): Cannot determine side-effects of assignment to `{0}`")]
    #[diagnostic(severity(warning))]
    Assignment(CompactStr, #[label] Span),

    #[error("eslint-plugin-tree-shaking(no-side-effects-in-initialization): Cannot determine side-effects of mutating")]
    #[diagnostic(severity(warning))]
    Mutation(#[label] Span),

    #[error("eslint-plugin-tree-shaking(no-side-effects-in-initialization): Cannot determine side-effects of mutating `{0}`")]
    #[diagnostic(severity(warning))]
    MutationWithName(CompactStr, #[label] Span),

    #[error("eslint-plugin-tree-shaking(no-side-effects-in-initialization): Cannot determine side-effects of mutating function return value")]
    #[diagnostic(severity(warning))]
    MutationOfFunctionReturnValue(#[label] Span),

    #[error("eslint-plugin-tree-shaking(no-side-effects-in-initialization): Cannot determine side-effects of calling")]
    #[diagnostic(severity(warning))]
    Call(#[label] Span),

    #[error("eslint-plugin-tree-shaking(no-side-effects-in-initialization): Cannot determine side-effects of calling function return value")]
    #[diagnostic(severity(warning))]
    CallReturnValue(#[label] Span),

    #[error("eslint-plugin-tree-shaking(no-side-effects-in-initialization): Cannot determine side-effects of calling global function `{0}`")]
    #[diagnostic(severity(warning))]
    CallGlobal(CompactStr, #[label] Span),
}

/// <https://github.com/lukastaegert/eslint-plugin-tree-shaking/blob/master/src/rules/no-side-effects-in-initialization.ts>
#[derive(Debug, Default, Clone)]
pub struct NoSideEffectsInInitialization;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Marks all side-effects in module initialization that will interfere with tree-shaking.
    ///
    /// This plugin is intended as a means for library developers to identify patterns that will
    /// interfere with the tree-shaking algorithm of their module bundler (i.e. rollup or webpack).
    ///
    /// ### Why is this bad?
    ///
    /// ### Example
    ///
    /// ```javascript
    /// myGlobal = 17; // Cannot determine side-effects of assignment to global variable
    /// const x = { [globalFunction()]: "myString" }; // Cannot determine side-effects of calling global function
    /// export default 42;
    /// ```
    NoSideEffectsInInitialization,
    nursery
);

impl Rule for NoSideEffectsInInitialization {
    fn run_once(&self, ctx: &LintContext) {
        let Some(root) = ctx.nodes().iter().next() else { return };
        let AstKind::Program(program) = root.kind() else { return };
        let node_listener_options = NodeListenerOptions::new(ctx);
        program.report_effects(&node_listener_options);
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        // ArrayExpression
        "[]",
        // "const x = []",
        // "const x = [ext,ext]",
        // "const x = [1,,2,]",
        // // ArrayPattern
        // "const [x] = []",
        // "const [,x,] = []",
        // // ArrowFunctionExpression
        // "const x = a=>{a(); ext()}",
        // // ArrowFunctionExpression when called
        // "(()=>{})()",
        // "(a=>{})()",
        // "((...a)=>{})()",
        // "(({a})=>{})()",
        // // ArrowFunctionExpression when mutated
        // "const x = ()=>{}; x.y = 1",
        // // AssignmentExpression
        "var x;x = {}",
        "var x;x += 1",
        "const x = {}; x.y = 1",
        r#"const x = {}; x["y"] = 1"#,
        "function x(){this.y = 1}; const z = new x()",
        "let x = 1; x = 2 + 3",
        "let x; x = 2 + 3",
        // // AssignmentPattern
        // "const {x = ext} = {}",
        // "const {x: y = ext} = {}",
        // "const {[ext]: x = ext} = {}",
        // "const x = ()=>{}, {y = x()} = {}",
        // // BinaryExpression
        // "const x = 1 + 2",
        // "if (1-1) ext()",
        // // BlockStatement
        // "{}",
        // "const x = ()=>{};{const x = ext}x()",
        // "const x = ext;{const x = ()=>{}; x()}",
        // // BreakStatement
        // "while(true){break}",
        // // CallExpression
        "(a=>{const y = a})(ext, ext)",
        "const x = ()=>{}, y = ()=>{}; x(y())",
        // // CatchClause
        // "try {} catch (error) {}",
        // "const x = ()=>{}; try {} catch (error) {const x = ext}; x()",
        // "const x = ext; try {} catch (error) {const x = ()=>{}; x()}",
        // // ClassBody
        // "class x {a(){ext()}}",
        // // ClassBody when called
        // "class x {a(){ext()}}; const y = new x()",
        // "class x {constructor(){}}; const y = new x()",
        // "class y{}; class x extends y{}; const z = new x()",
        // // ClassDeclaration
        // "class x extends ext {}",
        // // ClassDeclaration when called
        // "class x {}; const y = new x()",
        // // ClassExpression
        // "const x = class extends ext {}",
        // // ClassExpression when called
        // "const x = new (class {})()",
        // // ClassProperty
        // "class x {y}",
        // "class x {y = 1}",
        // "class x {y = ext()}",
        // // ConditionalExpression
        // "const x = ext ? 1 : 2",
        // "const x = true ? 1 : ext()",
        // "const x = false ? ext() : 2",
        // "if (true ? false : true) ext()",
        // "ext ? 1 : ext.x",
        // "ext ? ext.x : 1",
        // // ConditionalExpression when called
        // "const x = ()=>{}, y = ()=>{};(ext ? x : y)()",
        // "const x = ()=>{}; (true ? x : ext)()",
        // "const x = ()=>{}; (false ? ext : x)()",
        // // ContinueStatement
        // "while(true){continue}",
        // // DoWhileStatement
        // "do {} while(true)",
        // "do {} while(ext > 0)",
        // "const x = ()=>{}; do x(); while(true)",
        // // EmptyStatement
        // ";",
        // // ExportAllDeclaration
        // r#"export * from "import""#,
        // // ExportDefaultDeclaration
        // "export default ext",
        // "const x = ext; export default x",
        // "export default function(){}",
        // "export default (function(){})",
        // "const x = function(){}; export default /* tree-shaking no-side-effects-when-called */ x",
        // "export default /* tree-shaking no-side-effects-when-called */ function(){}",
        // // ExportNamedDeclaration
        // "export const x = ext",
        // "export function x(){ext()}",
        // "const x = ext; export {x}",
        // r#"export {x} from "import""#,
        // r#"export {x as y} from "import""#,
        // r#"export {x as default} from "import""#,
        // "export const /* tree-shaking no-side-effects-when-called */ x = function(){}",
        // "export function /* tree-shaking no-side-effects-when-called */ x(){}",
        // "const x = function(){}; export {/* tree-shaking no-side-effects-when-called */ x}",
        // // ExpressionStatement
        // "const x = 1",
        // // ForInStatement
        // "for(const x in ext){x = 1}",
        // "let x; for(x in ext){}",
        // // ForStatement
        // "for(let i = 0; i < 3; i++){i++}",
        // "for(;;){}",
        // // FunctionDeclaration
        // "function x(a){a(); ext()}",
        // // FunctionDeclaration when called
        // "function x(){}; x()",
        // "function x(a){}; x()",
        // "function x(...a){}; x()",
        // "function x({a}){}; x()",
        // // FunctionDeclaration when mutated
        // "function x(){}; x.y = 1",
        // // FunctionExpression
        // "const x = function (a){a(); ext()}",
        // // FunctionExpression when called
        // "(function (){}())",
        // "(function (a){}())",
        // "(function (...a){}())",
        // "(function ({a}){}())",
        // // Identifier
        // "var x;x = 1",
        // // Identifier when called
        // "const x = ()=>{};x(ext)",
        // "function x(){};x(ext)",
        // "var x = ()=>{};x(ext)",
        // "const x = ()=>{}, y = ()=>{x()}; y()",
        // "const x = ext, y = ()=>{const x = ()=>{}; x()}; y()",
        // // Identifier when mutated
        // "const x = {}; x.y = ext",
        // // IfStatement
        // "let y;if (ext > 0) {y = 1} else {y = 2}",
        // "if (false) {ext()}",
        // "if (true) {} else {ext()}",
        // // ImportDeclaration
        // r#"import "import""#,
        // r#"import x from "import-default""#,
        // r#"import {x} from "import""#,
        // r#"import {x as y} from "import""#,
        // r#"import * as x from "import""#,
        // r#"import /* tree-shaking no-side-effects-when-called */ x from "import-default-no-effects"; x()"#,
        // r#"import /* test */ /*tree-shaking  no-side-effects-when-called */ x from "import-default-no-effects"; x()"#,
        // r#"import /* tree-shaking  no-side-effects-when-called*/ /* test */ x from "import-default-no-effects"; x()"#,
        // r#"import {/* tree-shaking  no-side-effects-when-called */ x} from "import-no-effects"; x()"#,
        // r#"import {x as /* tree-shaking  no-side-effects-when-called */ y} from "import-no-effects"; y()"#,
        // r#"import {x} from "import"; /*@__PURE__*/ x()"#,
        // r#"import {x} from "import"; /* @__PURE__ */ x()"#,
        // // JSXAttribute
        // r#"class X {}; const x = <X test="3"/>"#,
        // "class X {}; const x = <X test={3}/>",
        // "class X {}; const x = <X test=<X/>/>",
        // // JSXElement
        // "class X {}; const x = <X/>",
        // "class X {}; const x = <X>Text</X>",
        // // JSXEmptyExpression
        // "class X {}; const x = <X>{}</X>",
        // // JSXExpressionContainer
        // "class X {}; const x = <X>{3}</X>",
        // // JSXIdentifier
        // "class X {}; const x = <X/>",
        // "const X = class {constructor() {this.x = 1}}; const x = <X/>",
        // // JSXOpeningElement
        // "class X {}; const x = <X/>",
        // "class X {}; const x = <X></X>",
        // r#"class X {}; const x = <X test="3"/>"#,
        // // JSXSpreadAttribute
        // "class X {}; const x = <X {...{x: 3}}/>",
        // // LabeledStatement
        // "loop: for(;true;){continue loop}",
        // // Literal
        // "const x = 3",
        // "if (false) ext()",
        // r#""use strict""#,
        // // LogicalExpression
        // "const x = 3 || 4",
        // "true || ext()",
        // "false && ext()",
        // "if (false && false) ext()",
        // "if (true && false) ext()",
        // "if (false && true) ext()",
        // "if (false || false) ext()",
        // // MemberExpression
        // "const x = ext.y",
        // r#"const x = ext["y"]"#,
        // "let x = ()=>{}; x.y = 1",
        // // MemberExpression when called
        // "const x = Object.keys({})",
        // // MemberExpression when mutated
        // "const x = {};x.y = ext",
        // "const x = {y: 1};delete x.y",
        // // MetaProperty
        // "function x(){const y = new.target}; x()",
        // // MethodDefinition
        // "class x {a(){}}",
        // "class x {static a(){}}",
        // // NewExpression
        // "const x = new (function (){this.x = 1})()",
        // "function x(){this.y = 1}; const z = new x()",
        // "/*@__PURE__*/ new ext()",
        // // ObjectExpression
        // "const x = {y: ext}",
        // r#"const x = {["y"]: ext}"#,
        // "const x = {};x.y = ext",
        // // ObjectPattern
        // "const {x} = {}",
        // "const {[ext]: x} = {}",
        // // RestElement
        // "const [...x] = []",
        // // ReturnStatement
        // "(()=>{return})()",
        // "(()=>{return 1})()",
        // // SequenceExpression
        // "let x = 1; x++, x++",
        // "if (ext, false) ext()",
        // // SwitchCase
        // "switch(ext){case ext:const x = 1;break;default:}",
        // // SwitchStatement
        // "switch(ext){}",
        // "const x = ()=>{}; switch(ext){case 1:const x = ext}; x()",
        // "const x = ext; switch(ext){case 1:const x = ()=>{}; x()}",
        // // TaggedTemplateExpression
        // "const x = ()=>{}; const y = x``",
        // // TemplateLiteral
        // "const x = ``",
        // "const x = `Literal`",
        // "const x = `Literal ${ext}`",
        // r#"const x = ()=>"a"; const y = `Literal ${x()}`"#,
        // // ThisExpression
        // "const y = this.x",
        // // ThisExpression when mutated
        // "const y = new (function (){this.x = 1})()",
        // "const y = new (function (){{this.x = 1}})()",
        // "const y = new (function (){(()=>{this.x = 1})()})()",
        // "function x(){this.y = 1}; const y = new x()",
        // // TryStatement
        // "try {} catch (error) {}",
        // "try {} finally {}",
        // "try {} catch (error) {} finally {}",
        // // UnaryExpression
        // "!ext",
        // "const x = {};delete x.y",
        // r#"const x = {};delete x["y"]"#,
        // // UpdateExpression
        // "let x=1;x++",
        // "const x = {};x.y++",
        // // VariableDeclaration
        // "const x = 1",
        // // VariableDeclarator
        // "var x, y",
        // "var x = 1, y = 2",
        // "const x = 1, y = 2",
        // "let x = 1, y = 2",
        // "const {x} = {}",
        // // WhileStatement
        // "while(true){}",
        // "while(ext > 0){}",
        // "const x = ()=>{}; while(true)x()",
        // // YieldExpression
        // "function* x(){const a = yield}; x()",
        // "function* x(){yield ext}; x()",
        // // Supports TypeScript nodes
        // "interface Blub {}",
    ];

    let fail = vec![
        // // ArrayExpression
        // "const x = [ext()]",
        // "const x = [,,ext(),]",
        // // ArrayPattern
        // "const [x = ext()] = []",
        // "const [,x = ext(),] = []",
        // // ArrowFunctionExpression when called
        // "(()=>{ext()})()",
        // "(({a = ext()})=>{})()",
        // "(a=>{a()})(ext)",
        // "((...a)=>{a()})(ext)",
        // "(({a})=>{a()})(ext)",
        // "(a=>{a.x = 1})(ext)",
        // "(a=>{const b = a;b.x = 1})(ext)",
        // "((...a)=>{a.x = 1})(ext)",
        // "(({a})=>{a.x = 1})(ext)",
        // // AssignmentExpression
        "ext = 1",
        "ext += 1",
        "ext.x = 1",
        "const x = {};x[ext()] = 1",
        // "this.x = 1",
        // // AssignmentPattern
        // "const {x = ext()} = {}",
        // "const {y: {x = ext()} = {}} = {}",
        // // AwaitExpression
        // "const x = async ()=>{await ext()}; x()",
        // // BinaryExpression
        // "const x = 1 + ext()",
        // "const x = ext() + 1",
        // // BlockStatement
        // "{ext()}",
        // "var x=()=>{};{var x=ext}x()",
        // "var x=ext;{x(); var x=()=>{}}",
        // // CallExpression
        "(()=>{})(ext(), 1)",
        "(()=>{})(1, ext())",
        // // CallExpression when called
        "const x = ()=>ext; const y = x(); y()",
        // // CallExpression when mutated
        "const x = ()=>ext; const y = x(); y.z = 1",
        // // CatchClause
        // "try {} catch (error) {ext()}",
        // "var x=()=>{}; try {} catch (error) {var x=ext}; x()",
        // // ClassBody
        // "class x {[ext()](){}}",
        // // ClassBody when called
        // "class x {constructor(){ext()}}; new x()",
        // "class x {constructor(){ext()}}; const y = new x()",
        // "class x extends ext {}; const y =  new x()",
        // "class y {constructor(){ext()}}; class x extends y {}; const z =  new x()",
        // "class y {constructor(){ext()}}; class x extends y {constructor(){super()}}; const z = new x()",
        // "class y{}; class x extends y{constructor(){super()}}; const z = new x()",
        // // ClassDeclaration
        // "class x extends ext() {}",
        // "class x {[ext()](){}}",
        // // ClassDeclaration when called
        // "class x {constructor(){ext()}}; new x()",
        // "class x {constructor(){ext()}}; const y = new x()",
        // "class x extends ext {}; const y = new x()",
        // // ClassExpression
        // "const x = class extends ext() {}",
        // "const x = class {[ext()](){}}",
        // // ClassExpression when called
        // "new (class {constructor(){ext()}})()",
        // "const x = new (class {constructor(){ext()}})()",
        // "const x = new (class extends ext {})()",
        // // ClassProperty
        // "class x {[ext()] = 1}",
        // // ClassProperty when called
        // "class x {y = ext()}; new x()",
        // // ConditionalExpression
        // "const x = ext() ? 1 : 2",
        // "const x = ext ? ext() : 2",
        // "const x = ext ? 1 : ext()",
        // "if (false ? false : true) ext()",
        // // ConditionalExpression when called
        // "const x = ()=>{}; (true ? ext : x)()",
        // "const x = ()=>{}; (false ? x : ext)()",
        // "const x = ()=>{}; (ext ? x : ext)()",
        // // DebuggerStatement
        // "debugger",
        // // DoWhileStatement
        // "do {} while(ext())",
        // "do ext(); while(true)",
        // "do {ext()} while(true)",
        // // ExportDefaultDeclaration
        // "export default ext()",
        // "export default /* tree-shaking no-side-effects-when-called */ ext",
        // "const x = ext; export default /* tree-shaking no-side-effects-when-called */ x",
        // // ExportNamedDeclaration
        // "export const x = ext()",
        // "export const /* tree-shaking no-side-effects-when-called */ x = ext",
        // "export function /* tree-shaking no-side-effects-when-called */ x(){ext()}",
        // "const x = ext; export {/* tree-shaking no-side-effects-when-called */ x}",
        // // ExpressionStatement
        // "ext()",
        // // ForInStatement
        // "for(ext in {a: 1}){}",
        // "for(const x in ext()){}",
        // "for(const x in {a: 1}){ext()}",
        // "for(const x in {a: 1}) ext()",
        // // ForOfStatement
        // "for(ext of {a: 1}){}",
        // "for(const x of ext()){}",
        // "for(const x of {a: 1}){ext()}",
        // "for(const x of {a: 1}) ext()",
        // // ForStatement
        // "for(ext();;){}",
        // "for(;ext();){}",
        // "for(;true;ext()){}",
        // "for(;true;) ext()",
        // "for(;true;){ext()}",
        // // FunctionDeclaration when called
        // "function x(){ext()}; x()",
        // "function x(){ext()}; const y = new x()",
        // "function x(){ext()}; new x()",
        // "function x(a = ext()){}; x()",
        // "function x(a){a()}; x(ext)",
        // "function x(...a){a()}; x(ext)",
        // "function x({a}){a()}; x(ext)",
        // "function x(a){a(); a(); a()}; x(ext)",
        // "function x(a){a.y = 1}; x(ext)",
        // "function x(...a){a.y = 1}; x(ext)",
        // "function x({a}){a.y = 1}; x(ext)",
        // "function x(a){a.y = 1; a.y = 2; a.y = 3}; x(ext)",
        // "function x(){ext = 1}; x(); x(); x()",
        // "function x(){ext = 1}; const y = new x(); y = new x(); y = new x()",
        // // FunctionExpression when called
        // "(function (){ext()}())",
        // "const x = new (function (){ext()})()",
        // "new (function (){ext()})()",
        // "(function ({a = ext()}){}())",
        // "(function (a){a()}(ext))",
        // "(function (...a){a()}(ext))",
        // "(function ({a}){a()}(ext))",
        // "(function (a){a.x = 1}(ext))",
        // "(function (a){const b = a;b.x = 1}(ext))",
        // "(function (...a){a.x = 1}(ext))",
        // "(function ({a}){a.x = 1}(ext))",
        // // Identifier when called
        // "ext()",
        // "const x = ext; x()",
        // "let x = ()=>{}; x = ext; x()",
        // "var x = ()=>{}; var x = ext; x()",
        // "const x = ()=>{ext()}; x()",
        // "const x = ()=>{ext = 1}; x(); x(); x()",
        // "let x = ()=>{}; const y = ()=>{x()}; x = ext; y()",
        // "var x = ()=>{}; const y = ()=>{x()}; var x = ext; y()",
        // "const x = ()=>{}; const {y} = x(); y()",
        // "const x = ()=>{}; const [y] = x(); y()",
        // // Identifier when mutated
        // "var x = ext; x.y = 1",
        // "var x = {}; x = ext; x.y = 1",
        // "var x = {}; var x = ext; x.y = 1",
        // "var x = {}; x = ext; x.y = 1; x.y = 1; x.y = 1",
        // "const x = {y:ext}; const {y} = x; y.z = 1",
        // // IfStatement
        // "if (ext()>0){}",
        // "if (1>0){ext()}",
        // "if (1<0){} else {ext()}",
        // "if (ext>0){ext()} else {ext()}",
        // // ImportDeclaration
        // r#"import x from "import-default"; x()"#,
        // r#"import x from "import-default"; x.z = 1"#,
        // r#"import {x} from "import"; x()"#,
        // r#"import {x} from "import"; x.z = 1"#,
        // r#"import {x as y} from "import"; y()"#,
        // r#"import {x as y} from "import"; y.a = 1"#,
        // r#"import * as y from "import"; y.x()"#,
        // r#"import * as y from "import"; y.x = 1"#,
        // // JSXAttribute
        // "class X {}; const x = <X test={ext()}/>",
        // "class X {}; class Y {constructor(){ext()}}; const x = <X test=<Y/>/>",
        // // JSXElement
        // "class X {constructor(){ext()}}; const x = <X/>",
        // "class X {}; const x = <X>{ext()}</X>",
        // // JSXExpressionContainer
        // "class X {}; const x = <X>{ext()}</X>",
        // // JSXIdentifier
        // "class X {constructor(){ext()}}; const x = <X/>",
        // "const X = class {constructor(){ext()}}; const x = <X/>",
        // "const x = <Ext/>",
        // // JSXMemberExpression
        // "const X = {Y: ext}; const x = <X.Y />",
        // // JSXOpeningElement
        // "class X {}; const x = <X test={ext()}/>",
        // // JSXSpreadAttribute
        // "class X {}; const x = <X {...{x: ext()}}/>",
        // // LabeledStatement
        // "loop: for(;true;){ext()}",
        // // Literal
        // "if (true) ext()",
        // // LogicalExpression
        // "ext() && true",
        // "true && ext()",
        // "false || ext()",
        // "if (true && true) ext()",
        // "if (false || true) ext()",
        // "if (true || false) ext()",
        // "if (true || true) ext()",
        // // MemberExpression
        // "const x = {};const y = x[ext()]",
        // // MemberExpression when called
        // "ext.x()",
        // "const x = {}; x.y()",
        // "const x = ()=>{}; x().y()",
        // "const Object = {}; const x = Object.keys({})",
        // "const x = {}; x[ext()]()",
        // // MemberExpression when mutated
        // "const x = {y: ext};x.y.z = 1",
        // "const x = {y:ext};const y = x.y; y.z = 1",
        // "const x = {y: ext};delete x.y.z",
        // // MethodDefinition
        // "class x {static [ext()](){}}",
        // // NewExpression
        // "const x = new ext()",
        // "new ext()",
        // // ObjectExpression
        // "const x = {y: ext()}",
        // r#"const x = {["y"]: ext()}"#,
        // "const x = {[ext()]: 1}",
        // // ObjectPattern
        // "const {[ext()]: x} = {}",
        // // ReturnStatement
        // "(()=>{return ext()})()",
        // // SequenceExpression
        // "ext(), 1",
        // "1, ext()",
        // "if (1, true) ext()",
        // "if (1, ext) ext()",
        // // Super when called
        // "class y {constructor(){ext()}}; class x extends y {constructor(){super()}}; const z = new x()",
        // "class y{}; class x extends y{constructor(){super(); super.test()}}; const z = new x()",
        // "class y{}; class x extends y{constructor(){super()}}; const z = new x()",
        // // SwitchCase
        // "switch(ext){case ext():}",
        // "switch(ext){case 1:ext()}",
        // // SwitchStatement
        // "switch(ext()){}",
        // "var x=()=>{}; switch(ext){case 1:var x=ext}; x()",
        // // TaggedTemplateExpression
        // "const x = ext``",
        // "ext``",
        // "const x = ()=>{}; const y = x`${ext()}`",
        // // TemplateLiteral
        // "const x = `Literal ${ext()}`",
        // // ThisExpression when mutated
        // "this.x = 1",
        // "(()=>{this.x = 1})()",
        // "(function(){this.x = 1}())",
        // "const y = new (function (){(function(){this.x = 1}())})()",
        // "function x(){this.y = 1}; x()",
        // // ThrowStatement
        // r#"throw new Error("Hello Error")"#,
        // // TryStatement
        // "try {ext()} catch (error) {}",
        // "try {} finally {ext()}",
        // // UnaryExpression
        // "!ext()",
        // "delete ext.x",
        // r#"delete ext["x"]"#,
        // "const x = ()=>{};delete x()",
        // // UpdateExpression
        // "ext++",
        // "const x = {};x[ext()]++",
        // // VariableDeclaration
        // "const x = ext()",
        // // VariableDeclarator
        // "var x = ext(),y = ext()",
        // "const x = ext(),y = ext()",
        // "let x = ext(),y = ext()",
        // "const {x = ext()} = {}",
        // // WhileStatement
        // "while(ext()){}",
        // "while(true)ext()",
        // "while(true){ext()}",
        // // YieldExpression
        // "function* x(){yield ext()}; x()",
        // // YieldExpression when called
        // "function* x(){yield ext()}; x()"
    ];

    Tester::new(NoSideEffectsInInitialization::NAME, pass, fail).test_and_snapshot();
}
