// Lexing ends in an error, so the token stream ends with `Undetermined` rather than `Eof`.
// This test case ensures that doesn't cause a panic.
foo();
'unterminated
