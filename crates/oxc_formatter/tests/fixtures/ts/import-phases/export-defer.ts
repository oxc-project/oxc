export defer { a, b as c } from "./a";

export defer * as ns from "./b";

export defer { d } from "./c" with { type: "json" };
