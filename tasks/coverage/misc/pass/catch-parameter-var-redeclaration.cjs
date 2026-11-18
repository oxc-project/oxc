// Annex B.3.4: a simple catch parameter can be redeclared by `var`.
try {} catch (e) { var e; }
try {} catch (e) { var e = 1; }
try {} catch (e) { for (var e of []) {} }
try {} catch (e) { for (var e in {}) {} }
// Lexical declarations in nested blocks do not conflict with the catch parameter.
try {} catch (e) { { let e; } }
try {} catch (e) { { const e = 1; } }
try {} catch (e) { { class e {} } }
try {} catch (e) { { function e() {} } }
try {} catch ({ e }) { { let e; } }
try {} catch (e) { try {} catch (f) { let e; } }
try {} catch (e) { function f() { let e; } }
