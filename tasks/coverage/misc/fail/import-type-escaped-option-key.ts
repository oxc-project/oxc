type Escaped = import("x", { "\u0077ith": { type: "json" } });
type LoneSurrogate = import("x", { "\uD800": { type: "json" } });
