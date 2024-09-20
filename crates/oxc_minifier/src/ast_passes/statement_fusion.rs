use oxc_traverse::Traverse;

use crate::CompressorPass;

/// Statement Fusion
///
/// Tries to fuse all the statements in a block into a one statement by using COMMAs or statements.
///
/// <https://github.com/google/closure-compiler/blob/master/src/com/google/javascript/jscomp/StatementFusion.java>
pub struct StatementFusion;

impl<'a> CompressorPass<'a> for StatementFusion {}

impl<'a> Traverse<'a> for StatementFusion {}

impl StatementFusion {
    pub fn new() -> Self {
        Self {}
    }
}

#[cfg(test)]
mod test {
    use oxc_allocator::Allocator;

    use crate::tester;

    fn test(source_text: &str, expected: &str) {
        let allocator = Allocator::default();
        let mut pass = super::StatementFusion::new();
        tester::test(&allocator, source_text, expected, &mut pass);
    }

    fn test_same(source_text: &str) {
        test(source_text, source_text);
    }

    fn fuse(before: &str, after: &str) {
        test(
            &("function F(){if(CONDITION){".to_string() + before + "}}"),
            &("function F(){if(CONDITION){".to_string() + after + "}}"),
        );
    }

    fn fuse_same(code: &str) {
        test_same(&("function F(){if(CONDITION){".to_string() + code + "}}"));
    }

    #[test]
    #[ignore]
    fn nothing_to_do() {
        fuse_same("");
        fuse_same("a");
        fuse_same("a()");
        fuse_same("if(a()){}");
    }

    #[test]
    #[ignore]
    fn fold_block_with_statements() {
        fuse("a;b;c", "a,b,c");
        fuse("a();b();c();", "a(),b(),c()");
        fuse("a(),b();c(),d()", "a(),b(),c(),d()");
        fuse("a();b(),c(),d()", "a(),b(),c(),d()");
        fuse("a(),b(),c();d()", "a(),b(),c(),d()");
    }

    #[test]
    #[ignore]
    fn fold_block_into_if() {
        fuse("a;b;c;if(x){}", "if(a,b,c,x){}");
        fuse("a;b;c;if(x,y){}else{}", "if(a,b,c,x,y){}else{}");
        fuse("a;b;c;if(x,y){}", "if(a,b,c,x,y){}");
        fuse("a;b;c;if(x,y,z){}", "if(a,b,c,x,y,z){}");

        // Can't fuse if there are statements after the IF.
        fuse_same("a();if(a()){}a()");
    }

    #[test]
    #[ignore]
    fn fold_block_return() {
        fuse("a;b;c;return x", "return a,b,c,x");
        fuse("a;b;c;return x+y", "return a,b,c,x+y");

        // DeadAssignmentElimination would have cleaned it up anyways.
        fuse_same("a;b;c;return x;a;b;c");
    }

    #[test]
    #[ignore]
    fn fold_block_throw() {
        fuse("a;b;c;throw x", "throw a,b,c,x");
        fuse("a;b;c;throw x+y", "throw a,b,c,x+y");
        fuse_same("a;b;c;throw x;a;b;c");
    }

    #[test]
    #[ignore]
    fn fold_switch() {
        fuse("a;b;c;switch(x){}", "switch(a,b,c,x){}");
    }

    #[test]
    #[ignore]
    fn fuse_into_for_in1() {
        fuse("a;b;c;for(x in y){}", "for(x in a,b,c,y){}");
    }

    #[test]
    #[ignore]
    fn fuse_into_for_in2() {
        // This test case causes a parse warning in ES5 strict out, but is a parse error in ES6+ out.
        // setAcceptedLanguage(CompilerOptions.LanguageMode.ECMASCRIPT5_STRICT);
        // set_expect_parse_warnings_in_this_test();
        fuse_same("a();for(var x = b() in y){}");
    }

    #[test]
    #[ignore]
    fn fuse_into_vanilla_for1() {
        fuse("a;b;c;for(;g;){}", "for(a,b,c;g;){}");
        fuse("a;b;c;for(d;g;){}", "for(a,b,c,d;g;){}");
        fuse("a;b;c;for(d,e;g;){}", "for(a,b,c,d,e;g;){}");
        fuse_same("a();for(var x;g;){}");
    }

    #[test]
    #[ignore]
    fn fuse_into_vanilla_for2() {
        fuse_same("a;b;c;for(var d;g;){}");
        fuse_same("a;b;c;for(let d;g;){}");
        fuse_same("a;b;c;for(const d = 5;g;){}");
    }

    #[test]
    #[ignore]
    fn fuse_into_label() {
        fuse("a;b;c;label:for(x in y){}", "label:for(x in a,b,c,y){}");
        fuse("a;b;c;label:for(;g;){}", "label:for(a,b,c;g;){}");
        fuse("a;b;c;l1:l2:l3:for(;g;){}", "l1:l2:l3:for(a,b,c;g;){}");
        fuse_same("a;b;c;label:while(true){}");
    }

    #[test]
    #[ignore]
    fn fuse_into_block() {
        fuse("a;b;c;{d;e;f}", "{a,b,c,d,e,f}");
        fuse(
            "a;b; label: { if(q) break label; bar(); }",
            "label: { if(a,b,q) break label; bar(); }",
        );
        fuse_same("a;b;c;{var x;d;e;}");
        fuse_same("a;b;c;label:{break label;d;e;}");
    }

    #[test]
    #[ignore]
    fn no_fuse_into_while() {
        fuse_same("a;b;c;while(x){}");
    }

    #[test]
    #[ignore]
    fn no_fuse_into_do() {
        fuse_same("a;b;c;do{}while(x)");
    }

    #[test]
    #[ignore]
    fn no_fuse_into_block() {
        // Never fuse a statement into a block that contains let/const/class declarations, or you risk
        // colliding variable names. (unless the AST is normalized).
        fuse("a; {b;}", "{a,b;}");
        fuse("a; {b; var a = 1;}", "{a,b; var a = 1;}");
        fuse_same("a; { b; let a = 1; }");
        fuse_same("a; { b; const a = 1; }");
        fuse_same("a; { b; class a {} }");
        fuse_same("a; { b; function a() {} }");
        fuse_same("a; { b; const otherVariable = 1; }");

        // enable_normalize();
        test(
            "function f(a) { if (COND) { a; { b; let a = 1; } } }",
            "function f(a) { if (COND) { { a,b; let a$jscomp$1 = 1; } } }",
        );
        test(
            "function f(a) { if (COND) { a; { b; let otherVariable = 1; } } }",
            "function f(a) { if (COND) {  { a,b; let otherVariable = 1; } } }",
        );
    }

    #[test]
    #[ignore]
    fn no_global_scope_changes() {
        test_same("a,b,c");
    }

    #[test]
    #[ignore]
    fn no_function_block_changes() {
        test_same("function foo() { a,b,c }");
    }
}
