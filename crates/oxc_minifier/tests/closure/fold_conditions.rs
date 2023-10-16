use crate::test;

#[test]
fn test_fold_not() {
    test("while(!(x==y)){a=b;}", "for(;x!=y;)a=b;");
    test("while(!(x!=y)){a=b;}", "for(;x==y;)a=b;");
    test("while(!(x===y)){a=b;}", "for(;x!==y;)a=b;");
    test("while(!(x!==y)){a=b;}", "for(;x===y;)a=b;");

    // Because !(x<NaN) != x>=NaN don't fold < and > cases.
    test("while(!(x>y)){a=b;}", "for(;!(x>y);)a=b;");
    test("while(!(x>=y)){a=b;}", "for(;!(x>=y);)a=b;");
    test("while(!(x<y)){a=b;}", "for(;!(x<y);)a=b;");
    test("while(!(x<=y)){a=b;}", "for(;!(x<=y);)a=b;");
    test("while(!(x<=NaN)){a=b;}", "for(;!(x<=NaN);)a=b;");

    // NOT forces a boolean context
    // test("x = !(y() && true)", "x=!y()");
    // This will be further optimized by PeepholeFoldConstants.
    // test("x = !true", "x=!1");
}
