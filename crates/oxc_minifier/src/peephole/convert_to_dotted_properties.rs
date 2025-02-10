use oxc_ast::ast::*;
use oxc_syntax::identifier::is_identifier_name;

use crate::ctx::Ctx;

use super::LatePeepholeOptimizations;

impl<'a> LatePeepholeOptimizations {
    /// Converts property accesses from quoted string or bracket access syntax to dot or unquoted string
    /// syntax, where possible. Dot syntax is more compact.
    ///
    /// <https://github.com/google/closure-compiler/blob/v20240609/src/com/google/javascript/jscomp/ConvertToDottedProperties.java>
    ///
    /// `foo['bar']` -> `foo.bar`
    /// `foo?.['bar']` -> `foo?.bar`
    pub fn convert_to_dotted_properties(expr: &mut MemberExpression<'a>, ctx: Ctx<'a, '_>) {
        let MemberExpression::ComputedMemberExpression(e) = expr else { return };
        let Expression::StringLiteral(s) = &e.expression else { return };
        if is_identifier_name(&s.value) {
            let property = ctx.ast.identifier_name(s.span, s.value);
            let object = ctx.ast.move_expression(&mut e.object);
            *expr = MemberExpression::StaticMemberExpression(
                ctx.ast.alloc_static_member_expression(e.span, object, property, e.optional),
            );
            return;
        }
        let v = s.value.as_str();
        if e.optional {
            return;
        }
        if let Some(n) = Ctx::string_to_equivalent_number_value(v) {
            e.expression = ctx.ast.expression_numeric_literal(s.span, n, None, NumberBase::Decimal);
        }
    }
}

#[cfg(test)]
mod test {
    use crate::tester::{test, test_same};

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
        test_same("a['p ']");
        test("a['p' + '']", "a.p");
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
        test("({'':0})", "");
        test("({'1.0':0})", "");
        test("({'\\u1d17A':0})", "");
        test("({'a\\u0004b':0})", "");
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
        test("a?.['p' + '']", "a?.p");
        test_same("a?.[p]");
        test_same("a?.[P]");
        test_same("a?.[$]");
        test_same("a?.[p()]");
        // This is safe for ES5+. (keywords cannot be used for ES3)
        test("a?.['default']", "a?.default");
    }

    #[test]
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

        test("const o = {'b'() {}, ['c']() {}};", "const o = {b() {}, c(){}};");
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
            "const o = {a: 1,b() {}, c() {},  get d() {},  set e(x) {}};",
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
        test("const test1 = {[0]:87};", "const test1 = {0:87}");
        test("const test1 = {['default']:87};", "const test1 = {default:87};");
        test_same("class C { ['constructor']() {} }");
        test_same("class C { ['constructor'] = 0 }");
    }

    #[test]
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

    #[test]
    fn test_index() {
        test("x['y']", "x.y;");
        test_same("x['y z']");
        test("x?.['y']", "x?.y;");
        test_same("x?.['y z']");
        test("x?.['y']()", "x?.y();");
        test_same("x?.['y z']()");
        test("x['y' + 'z']", "x.yz");
        test("x?.['y' + 'z']", "x?.yz");
        test("x['0']", "x[0];");
        test("x['123']", "x[123];");
        test("x['-123']", "x[-123];");
        test_same("x['-0']");
        test_same("x['+0']");
        test_same("x['01']");
        test_same("x['-01']");
        test_same("x['0x1']");
        test_same("x['-0x1']");
        test("x['2147483647']", "x[2147483647]");
        test_same("x['2147483648']");
        test("x['-2147483648']", "x[-2147483648]");
        test_same("x['-2147483649']");
    }
}
