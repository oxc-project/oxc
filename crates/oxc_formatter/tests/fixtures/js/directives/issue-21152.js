// Issue #21152 - the blank line after a directive is preserved even when a
// trailing comment sits between the directive and the blank line
// (lines_after must count past the comment). One prologue directive per
// comment token type, each followed by the load-bearing blank line.
"use client"; // browser only

"use strict"; /* single line block comment */

"use asm"; /* multi
line
comment */

import { Foo } from "foo";
import { Bar } from "bar";
