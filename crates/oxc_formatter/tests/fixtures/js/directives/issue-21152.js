// Issue #21152 - the blank line after the LAST directive survives a trailing comment;
// `end_of_line_comments_after` covers the last directive only, so one file per comment kind.
"use client"; // browser only

import { Foo } from "foo";
import { Bar } from "bar";
