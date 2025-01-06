use oxc_ast::ast::*;
use oxc_syntax::identifier::is_identifier_name;
use oxc_traverse::{traverse_mut_with_ctx, ReusableTraverseCtx, Traverse, TraverseCtx};

use crate::{node_util::Ctx, CompressorPass};

/// Converts property accesses from quoted string or bracket access syntax to dot or unquoted string
/// syntax, where possible. Dot syntax is more compact.
///
/// <https://github.com/google/closure-compiler/blob/v20240609/src/com/google/javascript/jscomp/ConvertToDottedProperties.java>
pub struct ConvertToDottedProperties {
    pub(crate) changed: bool,
    in_fixed_loop: bool,
}

impl<'a> CompressorPass<'a> for ConvertToDottedProperties {
    fn build(&mut self, program: &mut Program<'a>, ctx: &mut ReusableTraverseCtx<'a>) {
        self.changed = true;
        traverse_mut_with_ctx(self, program, ctx);
    }
}

impl<'a> Traverse<'a> for ConvertToDottedProperties {
    fn exit_member_expression(
        &mut self,
        expr: &mut MemberExpression<'a>,
        ctx: &mut TraverseCtx<'a>,
    ) {
        if !self.in_fixed_loop {
            self.try_compress_computed_member_expression(expr, Ctx(ctx));
        }
    }
}

impl<'a> ConvertToDottedProperties {
    pub fn new(in_fixed_loop: bool) -> Self {
        Self { changed: false, in_fixed_loop }
    }

    /// `foo['bar']` -> `foo.bar`
    /// `foo?.['bar']` -> `foo?.bar`
    fn try_compress_computed_member_expression(
        &mut self,
        expr: &mut MemberExpression<'a>,
        ctx: Ctx<'a, '_>,
    ) {
        if let MemberExpression::ComputedMemberExpression(e) = expr {
            let Expression::StringLiteral(s) = &e.expression else { return };
            if !is_identifier_name(&s.value) {
                return;
            }
            let property = ctx.ast.identifier_name(s.span, s.value.clone());
            let object = ctx.ast.move_expression(&mut e.object);
            *expr = MemberExpression::StaticMemberExpression(
                ctx.ast.alloc_static_member_expression(e.span, object, property, e.optional),
            );
            self.changed = true;
        }
    }
}

#[cfg(test)]
mod test {
    use oxc_allocator::Allocator;

    use crate::tester;

    fn test(source_text: &str, expected: &str) {
        let allocator = Allocator::default();
        let mut pass = super::ConvertToDottedProperties::new(false);
        tester::test(&allocator, source_text, expected, &mut pass);
    }

    fn test_same(source_text: &str) {
        test(source_text, source_text);
    }

    #[test]
    fn test_computed_to_member_expression() {
        test("x['true']", "x.true");
        test_same("x['😊']");
    }

    #[test]
    fn test_convert_to_dotted_properties_convert() {
        test("a['p']", "a.p");
        test("a['_p_']", "a._p_");
        test("a['_']", "a._");
        test("a['$']", "a.$");
        test("a.b.c['p']", "a.b.c.p");
        test("a.b['c'].p", "a.b.c.p");
        test("a['p']();", "a.p();");
        test("a()['p']", "a().p");
        // ASCII in Unicode is always safe.
        test("a['\\u0041A']", "a.AA");
        // This is safe for ES5+. (keywords cannot be used for ES3)
        test("a['default']", "a.default");
        // This is safe for ES2015+. (\u1d17 was introduced in Unicode 3.1, ES2015+ uses Unicode 5.1+)
        test("a['\\u1d17A']", "a.\u{1d17}A");
        // Latin capital N with tilde - this is safe for ES3+.
        test("a['\\u00d1StuffAfter']", "a.\u{00d1}StuffAfter");
    }

    #[test]
    fn test_convert_to_dotted_properties_do_not_convert() {
        test_same("a[0]");
        test_same("a['']");
        test_same("a[' ']");
        test_same("a[',']");
        test_same("a[';']");
        test_same("a[':']");
        test_same("a['.']");
        test_same("a['0']");
        test_same("a['p ']");
        test_same("a['p' + '']");
        test_same("a[p]");
        test_same("a[P]");
        test_same("a[$]");
        test_same("a[p()]");
        // Ignorable control characters are ok in Java identifiers, but not in JS.
        test_same("a['A\\u0004']");
    }

    #[test]
    fn test_convert_to_dotted_properties_already_dotted() {
        test_same("a.b");
        test_same("var a = {b: 0};");
    }

    #[test]
    fn test_convert_to_dotted_properties_quoted_props() {
        test_same("({'':0})");
        test_same("({'1.0':0})");
        test_same("({'\\u1d17A':0})");
        test_same("({'a\\u0004b':0})");
    }

    #[test]
    fn test5746867() {
        test_same("var a = { '$\\\\' : 5 };");
        test_same("var a = { 'x\\\\u0041$\\\\' : 5 };");
    }

    #[test]
    fn test_convert_to_dotted_properties_optional_chaining() {
        test("data?.['name']", "data?.name");
        test("data?.['name']?.['first']", "data?.name?.first");
        test("data['name']?.['first']", "data.name?.first");
        test_same("a?.[0]");
        test_same("a?.['']");
        test_same("a?.[' ']");
        test_same("a?.[',']");
        test_same("a?.[';']");
        test_same("a?.[':']");
        test_same("a?.['.']");
        test_same("a?.['0']");
        test_same("a?.['p ']");
        test_same("a?.['p' + '']");
        test_same("a?.[p]");
        test_same("a?.[P]");
        test_same("a?.[$]");
        test_same("a?.[p()]");
        // This is safe for ES5+. (keywords cannot be used for ES3)
        test("a?.['default']", "a?.default");
    }

    #[test]
    #[ignore]
    fn test_convert_to_dotted_properties_computed_property_or_field() {
        test("const test1 = {['prop1']:87};", "const test1 = {prop1:87};");
        test(
            "const test1 = {['prop1']:87,['prop2']:bg,['prop3']:'hfd'};",
            "const test1 = {prop1:87,prop2:bg,prop3:'hfd'};",
        );
        test(
            "o = {['x']: async function(x) { return await x + 1; }};",
            "o = {x:async function (x) { return await x + 1; }};",
        );
        test("o = {['x']: function*(x) {}};", "o = {x: function*(x) {}};");
        test(
            "o = {['x']: async function*(x) { return await x + 1; }};",
            "o = {x:async function*(x) { return await x + 1; }};",
        );
        test("class C {'x' = 0;  ['y'] = 1;}", "class C { x= 0;y= 1;}");
        test("class C {'m'() {} }", "class C {m() {}}");

        test("const o = {'b'() {}, ['c']() {}};", "const o = {b: function() {}, c:function(){}};");
        test("o = {['x']: () => this};", "o = {x: () => this};");

        test("const o = {get ['d']() {}};", "const o = {get d() {}};");
        test("const o = { set ['e'](x) {}};", "const o = { set e(x) {}};");
        test(
            "class C {'m'() {}  ['n']() {} 'x' = 0;  ['y'] = 1;}",
            "class C {m() {}  n() {} x= 0;y= 1;}",
        );
        test(
            "const o = { get ['d']() {},  set ['e'](x) {}};",
            "const o = {get d() {},  set e(x){}};",
        );
        test(
            "const o = {['a']: 1,'b'() {}, ['c']() {},  get ['d']() {},  set ['e'](x) {}};",
            "const o = {a: 1,b: function() {}, c: function() {},  get d() {},  set e(x) {}};",
        );

        // test static keyword
        test(
            r"
                class C {
                'm'(){}
                ['n'](){}
                static 'x' = 0;
                static ['y'] = 1;}
            ",
            r"
                class C {
                m(){}
                n(){}
                static x = 0;
                static y= 1;}
            ",
        );
        test(
            r"
                window['MyClass'] = class {
                static ['Register'](){}
                };
            ",
            r"
                window.MyClass = class {
                static Register(){}
                };
            ",
        );
        test(
            r"
                class C {
                'method'(){}
                async ['method1'](){}
                *['method2'](){}
                static ['smethod'](){}
                static async ['smethod1'](){}
                static *['smethod2'](){}}
            ",
            r"
                class C {
                method(){}
                async method1(){}
                *method2(){}
                static smethod(){}
                static async smethod1(){}
                static *smethod2(){}}
            ",
        );

        test_same("const o = {[fn()]: 0}");
        test_same("const test1 = {[0]:87};");
        test_same("const test1 = {['default']:87};");
        test_same("class C { ['constructor']() {} }");
        test_same("class C { ['constructor'] = 0 }");
    }

    #[test]
    #[ignore]
    fn test_convert_to_dotted_properties_computed_property_with_default_value() {
        test("const {['o']: o = 0} = {};", "const {o:o = 0} = {};");
    }

    #[test]
    fn test_convert_to_dotted_properties_continue_optional_chaining() {
        test("const opt1 = window?.a?.['b'];", "const opt1 = window?.a?.b;");

        test("const opt2 = window?.a['b'];", "const opt2 = window?.a.b;");
        test(
            r"
                const chain =
                window['a'].x.y.b.x.y['c'].x.y?.d.x.y['e'].x.y
                ['f-f'].x.y?.['g-g'].x.y?.['h'].x.y['i'].x.y;
            ",
            r"
                const chain = window.a.x.y.b.x.y.c.x.y?.d.x.y.e.x.y
                ['f-f'].x.y?.['g-g'].x.y?.h.x.y.i.x.y;
            ",
        );
    }
}
