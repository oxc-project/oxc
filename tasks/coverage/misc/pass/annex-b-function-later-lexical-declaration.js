function laterLet() { { function g() {} } let g = 1; return g; }
function laterClass() { { function g() {} } class g {} return g; }
function laterInEnclosingBlock() { { { function g() {} } let g; } }
function ifStatementFunctionThenLet() { { function g() {} } if (false) function g() {} let g = 1; return g; }
function undoThenRehoist() { { { function g() {} } let g; } function nested() { { function g() {} } } { function g() {} } let g = 7; return g; }
{ function g() {} }
let g;
