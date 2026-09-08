// A star-aligned multiline block comment before the value breaks after the operator even
// when the value follows it on the comment's last line (Prettier's `isIndentableBlockComment`);
// other multiline block comments keep the inline layout.
fnString = /**
 * multi-line
 */ value;
var fnString = /**
 * multi-line
 */ value;
const style = /** @type {{
  width: number,
}} */ ({
  width,
});
const style2 = /**
 * @type {{
 *   width: number,
 * }}
 */ ({
  width,
});
// An argument-less call stays poorly breakable with a dangling comment
call = call(/* call argument long long long long long long long long long long long long long long */);
// A commented lone argument is never short
node.id = this.flowParseTypeAnnotatableIdentifier(/*allowPrimitiveOverride*/ true);
