import first from "source";
export { first, first as renamed, first as default };

export { later as forward };
import later from "other";

import { default as explicitDefault, named as alias } from "third";
export { explicitDefault, alias as renamedNamed };

import * as namespace from "fourth";
export { namespace };

import direct from "fifth";
export { direct as explicitlyReexported } from "sixth";
