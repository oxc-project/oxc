// Each of these is a generic arrow function. Read as a JSX element, it is a syntax error.
type É = 1;

const a = <T,>(x: T) => x;
const b = <T = 1>(x: T) => x;
const c = <T extends 1>(x: T) => x;
const d = <const T,>(x: T) => x;
const e = < /* comment */ T,>(x: T) => x;
const f = <T /* comment */ ,>(x: T) => x;
const g = <T // comment
  ,>(x: T) => x;
const h = <T /* comment */ extends 1>(x: T) => x;
const i = <T,>(x: T) => x;
const j = <TT,>(x: TT) => x;
const k = <Ü,>(x: Ü) => x;
const l = <T
  extends É>(x: T) => x;
