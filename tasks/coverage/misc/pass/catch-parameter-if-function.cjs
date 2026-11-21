// Annex B.3.3: a function declaration in an `if` clause behaves as if it were in its own block.
try {} catch (e) { if (true) function e() {} }
try {} catch (e) { if (false) ; else function e() {} }
