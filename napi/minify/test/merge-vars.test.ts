// Cases ported from UglifyJS `test/compress/merge_vars.js`.
// https://github.com/mishoo/UglifyJS/blob/master/test/compress/merge_vars.js
//
// UglifyJS is released under the BSD-2-Clause license:
//
// Copyright 2012-2019 (c) Mihai Bazon <mihai.bazon@gmail.com>
//
// Redistribution and use in source and binary forms, with or without
// modification, are permitted provided that the following conditions
// are met:
//
//     * Redistributions of source code must retain the above
//       copyright notice, this list of conditions and the following
//       disclaimer.
//
//     * Redistributions in binary form must reproduce the above
//       copyright notice, this list of conditions and the following
//       disclaimer in the documentation and/or other materials
//       provided with the distribution.
//
// THIS SOFTWARE IS PROVIDED BY THE COPYRIGHT HOLDER “AS IS” AND ANY
// EXPRESS OR IMPLIED WARRANTIES, INCLUDING, BUT NOT LIMITED TO, THE
// IMPLIED WARRANTIES OF MERCHANTABILITY AND FITNESS FOR A PARTICULAR
// PURPOSE ARE DISCLAIMED. IN NO EVENT SHALL THE COPYRIGHT HOLDER BE
// LIABLE FOR ANY DIRECT, INDIRECT, INCIDENTAL, SPECIAL, EXEMPLARY,
// OR CONSEQUENTIAL DAMAGES (INCLUDING, BUT NOT LIMITED TO,
// PROCUREMENT OF SUBSTITUTE GOODS OR SERVICES; LOSS OF USE, DATA, OR
// PROFITS; OR BUSINESS INTERRUPTION) HOWEVER CAUSED AND ON ANY
// THEORY OF LIABILITY, WHETHER IN CONTRACT, STRICT LIABILITY, OR
// TORT (INCLUDING NEGLIGENCE OR OTHERWISE) ARISING IN ANY WAY OUT OF
// THE USE OF THIS SOFTWARE, EVEN IF ADVISED OF THE POSSIBILITY OF
// SUCH DAMAGE.

import { inspect } from "node:util";
import { runInNewContext } from "node:vm";
import { describe, expect, it } from "vitest";

import { minifySync } from "../index";

function format(value: unknown): string {
  return inspect(value, { depth: 3 }).replace(/\[(Function|class)[^\]]*\]/g, "[$1]");
}

function run(code: string): { output: string[]; error?: string } {
  const output: string[] = [];
  const log = (...args: unknown[]) => {
    output.push(args.map(format).join(" "));
  };
  try {
    runInNewContext(code, { console: { log } }, { timeout: 1000 });
  } catch (error) {
    return {
      output,
      error: error instanceof Object ? String((error as Error).name) : String(error),
    };
  }
  return { output };
}

const cases: [string, string][] = [
  [
    "merge",
    `
      var a = "foo";
      console.log(a);
      function f(b) {
          var c;
          console.log(b);
          c = "bar";
          console.log(c);
      }
      f("baz");
      var d = "moo";
      console.log(d);
    `,
  ],
  [
    "merge_toplevel",
    `
      var a = "foo";
      console.log(a);
      function f(b) {
          var c;
          console.log(b);
          c = "bar";
          console.log(c);
      }
      f("baz");
      var d = "moo";
      console.log(d);
    `,
  ],
  [
    "segment",
    `
      var a = "foo";
      console.log(a);
      for (var c, i = 0; i < 1; i++) {
          var b = "bar";
          console.log(b);
          c = "baz";
          console.log(c);
      }
      var d = "moo";
      console.log(d);
    `,
  ],
  [
    "init_scope_vars",
    `
      Function.prototype.call();
    `,
  ],
  [
    "binary_branch",
    `
      console.log(function(a) {
          var b = "FAIL", c;
          a && (c = b);
          return c || "PASS";
      }());
    `,
  ],
  [
    "conditional_branch",
    `
      console.log(function(a) {
          var b = "FAIL", c;
          a ? (c = b) : void 0;
          return c || "PASS";
      }());
    `,
  ],
  [
    "conditional_chain_1",
    `
      function f(a, b) {
          var c, d;
          if (a && (c = a))
              console.log(c);
          else
              b || (d = b) ? console.log("foo") : console.log(d);
      }
      f("", null);
      f("", true);
      f(42, null);
      f(42, true);
    `,
  ],
  [
    "conditional_chain_2",
    `
      function f(a, b) {
          var c, d;
          if (a && (c = a))
              console.log(c);
          else
              b || (d = b) ? console.log(c) : console.log(d);
      }
      f("", null);
      f("", true);
      f(42, null);
      f(42, true);
    `,
  ],
  [
    "conditional_chain_3",
    `
      function f(a, b) {
          var c, d;
          if (a && (c = a) || b || (d = b))
              console.log(c);
          else
              console.log(d);
      }
      f("", null);
      f("", true);
      f(42, null);
      f(42, true);
    `,
  ],
  [
    "conditional_chain_4",
    `
      function f(a, b) {
          var c, d;
          if (a && b ? c = a : d = b)
              console.log(c);
          else
              console.log(d);
      }
      f("", null);
      f("", true);
      f(42, null);
      f(42, true);
    `,
  ],
  [
    "if_branch",
    `
      console.log(function(a) {
          var b = "FAIL", c;
          if (a) c = b;
          return c || "PASS";
      }());
    `,
  ],
  [
    "switch_branch",
    `
      console.log(function(a) {
          var b = "FAIL", c;
          switch (a) {
            case 1:
              c = b;
              break;
          }
          return c || "PASS";
      }());
    `,
  ],
  [
    "try_branch",
    `
      console.log(function(a) {
          var b = "FAIL", c;
          try {
              a && F();
          } catch (e) {
              c = b;
          }
          return c || "PASS";
      }());
    `,
  ],
  [
    "read_before_assign_1",
    `
      var c = 0;
      c = 0;
      (function() {
          var a = console.log(++a);
          a;
      })();
      c;
    `,
  ],
  [
    "read_before_assign_2",
    `
      console.log(function(a, a) {
          while (b)
              return "FAIL";
          var b = 1;
          return "PASS";
      }(0, []));
    `,
  ],
  [
    "collapse_vars_1",
    `
      var a = a && a.p;
      var b = "PASS";
      var b = b && console.log(b);
    `,
  ],
  [
    "collapse_vars_2",
    `
      "use strict";
      var log = console.log;
      (function g(a) {
          var b = a;
          var c = Math.random();
          var c = b;
          log(c);
          return c;
      })("PASS");
    `,
  ],
  [
    "not_redefined",
    `
      var log = console.log;
      (function() {
          return f("PASS");
          function f(a) {
              const b = a;
              const c = log(b);
              const d = log;
              c && log(d);
          }
      })();
    `,
  ],
  [
    "issue_4103",
    `
      function f(a) {
          console.log(a);
      }
      var b = 0;
      var c = f(b++ + (c %= 1 >> console.log(c = 0)));
      b;
    `,
  ],
  [
    "issue_4107_1",
    `
      (function() {
          function f(b, b, c) {
              var d = 1 && a, a = console || c;
              console.log(typeof a);
          }
          f();
      })();
      console.log(typeof a);
    `,
  ],
  [
    "issue_4107_2",
    `
      (function() {
          function f(b, b, a) {
              var d = 1 && c, c = console || a;
              console.log(typeof c);
          }
          f();
      })();
      console.log(typeof a);
    `,
  ],
  [
    "issue_4109",
    `
      var a = "foo";
      try {
          throw "bar";
      } catch (e) {
          console.log(e);
      } finally {
          var o = a;
          for (var k in o);
          (function() {
              a++;
          });
      }
      console.log(a);
    `,
  ],
  [
    "issue_4110",
    `
      while (a)
          var c;
      var b, a = c += b = a;
      console.log(b);
    `,
  ],
  [
    "issue_4111",
    `
      var a = 0;
      if (a)
          a = 0;
      else
          for (var b = 0; --b && ++a < 2;) {
              var o = console, k;
              for (k in o);
          }
      console.log(a);
    `,
  ],
  [
    "issue_4112",
    `
      console.log(typeof function() {
          try {
              throw 42;
          } catch (e) {
              var o = e;
              for (e in o);
              var a = function() {};
              console.log(typeof a);
              return a;
          }
      }());
    `,
  ],
  [
    "issue_4115",
    `
      L: {
          var o = typeof console;
          for (var k in o)
              break L;
          var a = 0;
      }
      console.log(typeof a);
    `,
  ],
  [
    "cross_branch_1_1",
    `
      var a;
      function f() {
          var x, y;
          if (a)
              x = "foo";
          console.log(x);
          y = "bar";
          console.log(y);
      }
      a = 0;
      f();
      a = 1;
      f();
    `,
  ],
  [
    "cross_branch_1_2",
    `
      var a;
      function f() {
          var x, y;
          if (a) {
              x = "foo";
              console.log(x);
          }
          y = "bar";
          console.log(y);
      }
      a = 0;
      f();
      a = 1;
      f();
    `,
  ],
  [
    "cross_branch_1_3",
    `
      var a;
      function f() {
          var x, y;
          if (a) {
              x = "foo";
              console.log(x);
              y = "bar";
          }
          console.log(y);
      }
      a = 0;
      f();
      a = 1;
      f();
    `,
  ],
  [
    "cross_branch_1_4",
    `
      var a;
      function f() {
          var x, y;
          x = "foo";
          if (a)
              console.log(x);
          y = "bar";
          console.log(y);
      }
      a = 0;
      f();
      a = 1;
      f();
    `,
  ],
  [
    "cross_branch_1_5",
    `
      var a;
      function f() {
          var x, y;
          x = "foo";
          if (a) {
              console.log(x);
              y = "bar";
          }
          console.log(y);
      }
      a = 0;
      f();
      a = 1;
      f();
    `,
  ],
  [
    "cross_branch_1_6",
    `
      var a;
      function f() {
          var x, y;
          x = "foo";
          if (a) {
              console.log(x);
              y = "bar";
              console.log(y);
          }
      }
      a = 0;
      f();
      a = 1;
      f();
    `,
  ],
  [
    "cross_branch_1_7",
    `
      var a;
      function f() {
          var x, y;
          x = "foo";
          console.log(x);
          if (a)
              y = "bar";
          console.log(y);
      }
      a = 0;
      f();
      a = 1;
      f();
    `,
  ],
  [
    "cross_branch_1_8",
    `
      var a;
      function f() {
          var x, y;
          x = "foo";
          console.log(x);
          if (a) {
              y = "bar";
              console.log(y);
          }
      }
      a = 0;
      f();
      a = 1;
      f();
    `,
  ],
  [
    "cross_branch_1_9",
    `
      var a;
      function f() {
          var x, y;
          x = "foo";
          console.log(x);
          y = "bar";
          if (a)
              console.log(y);
      }
      a = 0;
      f();
      a = 1;
      f();
    `,
  ],
  [
    "cross_branch_2a_1",
    `
      var a, b;
      function f() {
          var x, y;
          if (a) {
              if (b)
                  x = "foo";
              console.log(x);
          }
          y = "bar";
          console.log(y);
      }
      a = 0, b = 0;
      f();
      a = 1, b = 0;
      f();
      a = 0, b = 1;
      f();
      a = 1, b = 1;
      f();
    `,
  ],
  [
    "cross_branch_2a_2",
    `
      var a, b;
      function f() {
          var x, y;
          if (a) {
              x = "foo";
              if (b)
                  console.log(x);
          }
          y = "bar";
          console.log(y);
      }
      a = 0, b = 0;
      f();
      a = 1, b = 0;
      f();
      a = 0, b = 1;
      f();
      a = 1, b = 1;
      f();
    `,
  ],
  [
    "cross_branch_2a_3",
    `
      var a, b;
      function f() {
          var x, y;
          if (a) {
              if (b)
                  x = "foo";
              console.log(x);
              y = "bar";
          }
          console.log(y);
      }
      a = 0, b = 0;
      f();
      a = 1, b = 0;
      f();
      a = 0, b = 1;
      f();
      a = 1, b = 1;
      f();
    `,
  ],
  [
    "cross_branch_2a_4",
    `
      var a, b;
      function f() {
          var x, y;
          if (a) {
              if (b) {
                  x = "foo";
                  console.log(x);
              }
              y = "bar";
          }
          console.log(y);
      }
      a = 0, b = 0;
      f();
      a = 1, b = 0;
      f();
      a = 0, b = 1;
      f();
      a = 1, b = 1;
      f();
    `,
  ],
  [
    "cross_branch_2a_5",
    `
      var a, b;
      function f() {
          var x, y;
          if (a) {
              x = "foo";
              if (b)
                  console.log(x);
              y = "bar";
          }
          console.log(y);
      }
      a = 0, b = 0;
      f();
      a = 1, b = 0;
      f();
      a = 0, b = 1;
      f();
      a = 1, b = 1;
      f();
    `,
  ],
  [
    "cross_branch_2a_6",
    `
      var a, b;
      function f() {
          var x, y;
          if (a) {
              x = "foo";
              if (b) {
                  console.log(x);
                  y = "bar";
              }
          }
          console.log(y);
      }
      a = 0, b = 0;
      f();
      a = 1, b = 0;
      f();
      a = 0, b = 1;
      f();
      a = 1, b = 1;
      f();
    `,
  ],
  [
    "cross_branch_2a_7",
    `
      var a, b;
      function f() {
          var x, y;
          if (a) {
              x = "foo";
              console.log(x);
              if (b)
                  y = "bar";
          }
          console.log(y);
      }
      a = 0, b = 0;
      f();
      a = 1, b = 0;
      f();
      a = 0, b = 1;
      f();
      a = 1, b = 1;
      f();
    `,
  ],
  [
    "cross_branch_2a_8",
    `
      var a, b;
      function f() {
          var x, y;
          x = "foo";
          if (a) {
              if (b)
                  console.log(x);
              y = "bar";
          }
          console.log(y);
      }
      a = 0, b = 0;
      f();
      a = 1, b = 0;
      f();
      a = 0, b = 1;
      f();
      a = 1, b = 1;
      f();
    `,
  ],
  [
    "cross_branch_2a_9",
    `
      var a, b;
      function f() {
          var x, y;
          x = "foo";
          if (a) {
              console.log(x);
              if (b)
                  y = "bar";
          }
          console.log(y);
      }
      a = 0, b = 0;
      f();
      a = 1, b = 0;
      f();
      a = 0, b = 1;
      f();
      a = 1, b = 1;
      f();
    `,
  ],
  [
    "cross_branch_2a_10",
    `
      var a, b;
      function f() {
          var x, y;
          x = "foo";
          if (a) {
              if (b)
                  console.log(x);
              y = "bar";
              console.log(y);
          }
      }
      a = 0, b = 0;
      f();
      a = 1, b = 0;
      f();
      a = 0, b = 1;
      f();
      a = 1, b = 1;
      f();
    `,
  ],
  [
    "cross_branch_2a_11",
    `
      var a, b;
      function f() {
          var x, y;
          x = "foo";
          if (a) {
              if (b) {
                  console.log(x);
                  y = "bar";
              }
              console.log(y);
          }
      }
      a = 0, b = 0;
      f();
      a = 1, b = 0;
      f();
      a = 0, b = 1;
      f();
      a = 1, b = 1;
      f();
    `,
  ],
  [
    "cross_branch_2a_12",
    `
      var a, b;
      function f() {
          var x, y;
          x = "foo";
          if (a) {
              console.log(x);
              if (b)
                  y = "bar";
              console.log(y);
          }
      }
      a = 0, b = 0;
      f();
      a = 1, b = 0;
      f();
      a = 0, b = 1;
      f();
      a = 1, b = 1;
      f();
    `,
  ],
  [
    "cross_branch_2a_13",
    `
      var a, b;
      function f() {
          var x, y;
          x = "foo";
          if (a) {
              console.log(x);
              if (b) {
                  y = "bar";
                  console.log(y);
              }
          }
      }
      a = 0, b = 0;
      f();
      a = 1, b = 0;
      f();
      a = 0, b = 1;
      f();
      a = 1, b = 1;
      f();
    `,
  ],
  [
    "cross_branch_2a_14",
    `
      var a, b;
      function f() {
          var x, y;
          x = "foo";
          if (a) {
              console.log(x);
              y = "bar";
              if (b)
                  console.log(y);
          }
      }
      a = 0, b = 0;
      f();
      a = 1, b = 0;
      f();
      a = 0, b = 1;
      f();
      a = 1, b = 1;
      f();
    `,
  ],
  [
    "cross_branch_2a_15",
    `
      var a, b;
      function f() {
          var x, y;
          x = "foo";
          console.log(x);
          if (a) {
              if (b)
                  y = "bar";
              console.log(y);
          }
      }
      a = 0, b = 0;
      f();
      a = 1, b = 0;
      f();
      a = 0, b = 1;
      f();
      a = 1, b = 1;
      f();
    `,
  ],
  [
    "cross_branch_2a_16",
    `
      var a, b;
      function f() {
          var x, y;
          x = "foo";
          console.log(x);
          if (a) {
              y = "bar";
              if (b)
                  console.log(y);
          }
      }
      a = 0, b = 0;
      f();
      a = 1, b = 0;
      f();
      a = 0, b = 1;
      f();
      a = 1, b = 1;
      f();
    `,
  ],
  [
    "cross_branch_2b_1",
    `
      var a, b;
      function f() {
          var x, y;
          if (a)
              x = "foo";
          if (b)
              console.log(x);
          y = "bar";
          console.log(y);
      }
      a = 0, b = 0;
      f();
      a = 1, b = 0;
      f();
      a = 0, b = 1;
      f();
      a = 1, b = 1;
      f();
    `,
  ],
  [
    "cross_branch_2b_2",
    `
      var a, b;
      function f() {
          var x, y;
          if (a)
              x = "foo";
          if (b) {
              console.log(x);
              y = "bar";
          }
          console.log(y);
      }
      a = 0, b = 0;
      f();
      a = 1, b = 0;
      f();
      a = 0, b = 1;
      f();
      a = 1, b = 1;
      f();
    `,
  ],
  [
    "cross_branch_2b_3",
    `
      var a, b;
      function f() {
          var x, y;
          if (a)
              x = "foo";
          if (b) {
              console.log(x);
              y = "bar";
              console.log(y);
          }
      }
      a = 0, b = 0;
      f();
      a = 1, b = 0;
      f();
      a = 0, b = 1;
      f();
      a = 1, b = 1;
      f();
    `,
  ],
  [
    "cross_branch_2b_4",
    `
      var a, b;
      function f() {
          var x, y;
          if (a)
              x = "foo";
          console.log(x);
          if (b)
              y = "bar";
          console.log(y);
      }
      a = 0, b = 0;
      f();
      a = 1, b = 0;
      f();
      a = 0, b = 1;
      f();
      a = 1, b = 1;
      f();
    `,
  ],
  [
    "cross_branch_2b_5",
    `
      var a, b;
      function f() {
          var x, y;
          if (a)
              x = "foo";
          console.log(x);
          if (b) {
              y = "bar";
              console.log(y);
          }
      }
      a = 0, b = 0;
      f();
      a = 1, b = 0;
      f();
      a = 0, b = 1;
      f();
      a = 1, b = 1;
      f();
    `,
  ],
  [
    "cross_branch_2b_6",
    `
      var a, b;
      function f() {
          var x, y;
          if (a) {
              x = "foo";
              console.log(x);
          }
          if (b)
              y = "bar";
          console.log(y);
      }
      a = 0, b = 0;
      f();
      a = 1, b = 0;
      f();
      a = 0, b = 1;
      f();
      a = 1, b = 1;
      f();
    `,
  ],
  [
    "cross_branch_2b_7",
    `
      var a, b;
      function f() {
          var x, y;
          if (a) {
              x = "foo";
              console.log(x);
          }
          if (b) {
              y = "bar";
              console.log(y);
          }
      }
      a = 0, b = 0;
      f();
      a = 1, b = 0;
      f();
      a = 0, b = 1;
      f();
      a = 1, b = 1;
      f();
    `,
  ],
  [
    "cross_branch_2b_8",
    `
      var a, b;
      function f() {
          var x, y;
          if (a)
              x = "foo";
          console.log(x);
          y = "bar";
          if (b)
              console.log(y);
      }
      a = 0, b = 0;
      f();
      a = 1, b = 0;
      f();
      a = 0, b = 1;
      f();
      a = 1, b = 1;
      f();
    `,
  ],
  [
    "cross_branch_2b_9",
    `
      var a, b;
      function f() {
          var x, y;
          if (a) {
              x = "foo";
              console.log(x);
          }
          y = "bar";
          if (b)
              console.log(y);
      }
      a = 0, b = 0;
      f();
      a = 1, b = 0;
      f();
      a = 0, b = 1;
      f();
      a = 1, b = 1;
      f();
    `,
  ],
  [
    "cross_branch_2b_10",
    `
      var a, b;
      function f() {
          var x, y;
          if (a) {
              x = "foo";
              console.log(x);
              y = "bar";
          }
          if (b)
              console.log(y);
      }
      a = 0, b = 0;
      f();
      a = 1, b = 0;
      f();
      a = 0, b = 1;
      f();
      a = 1, b = 1;
      f();
    `,
  ],
  [
    "cross_branch_2b_11",
    `
      var a, b;
      function f() {
          var x, y;
          x = "foo";
          if (a)
              console.log(x);
          if (b)
              y = "bar";
          console.log(y);
      }
      a = 0, b = 0;
      f();
      a = 1, b = 0;
      f();
      a = 0, b = 1;
      f();
      a = 1, b = 1;
      f();
    `,
  ],
  [
    "cross_branch_2b_12",
    `
      var a, b;
      function f() {
          var x, y;
          x = "foo";
          if (a)
              console.log(x);
          if (b) {
              y = "bar";
              console.log(y);
          }
      }
      a = 0, b = 0;
      f();
      a = 1, b = 0;
      f();
      a = 0, b = 1;
      f();
      a = 1, b = 1;
      f();
    `,
  ],
  [
    "cross_branch_2b_13",
    `
      var a, b;
      function f() {
          var x, y;
          x = "foo";
          if (a)
              console.log(x);
          y = "bar";
          if (b)
              console.log(y);
      }
      a = 0, b = 0;
      f();
      a = 1, b = 0;
      f();
      a = 0, b = 1;
      f();
      a = 1, b = 1;
      f();
    `,
  ],
  [
    "cross_branch_2b_14",
    `
      var a, b;
      function f() {
          var x, y;
          x = "foo";
          if (a) {
              console.log(x);
              y = "bar";
          }
          if (b)
              console.log(y);
      }
      a = 0, b = 0;
      f();
      a = 1, b = 0;
      f();
      a = 0, b = 1;
      f();
      a = 1, b = 1;
      f();
    `,
  ],
  [
    "cross_branch_2b_15",
    `
      var a, b;
      function f() {
          var x, y;
          x = "foo";
          console.log(x);
          if (a)
              y = "bar";
          if (b)
              console.log(y);
      }
      a = 0, b = 0;
      f();
      a = 1, b = 0;
      f();
      a = 0, b = 1;
      f();
      a = 1, b = 1;
      f();
    `,
  ],
  [
    "issue_4126_1",
    `
      function f(a) {
          try {
              console.log("PASS");
          } catch (e) {
              var b = a;
          } finally {
              var c = b;
          }
          console.log(c);
      }
      f("FAIL");
    `,
  ],
  [
    "issue_4126_2",
    `
      try {
          var a = function() {
              var b = 0;
              function f() {
                  b;
              }
              THROW(b);
          }();
      } catch (e) {
          console.log(a);
      }
    `,
  ],
  [
    "issue_4130",
    `
      var a = 2;
      while (a)
          try {
              console.log(a);
          } catch (e) {
              var b = 0;
          } finally {
              b && console.log("FAIL");
              var c = --a;
              for (var k in c)
                  c;
          }
    `,
  ],
  [
    "issue_4135",
    `
      var a = 0, b = 0;
      --b;
      a++;
      if (!a)
          var c = function() {
              var d = 0;
              function f() {
                  d && d.p;
              }
              f();
              this;
          }(a++);
      console.log(a, b, c);
    `,
  ],
  [
    "issue_4139",
    `
      try {
          console.log;
      } catch (e) {
          var a, arguments = 0;
      } finally {
          a = typeof arguments;
          console.log(a);
      }
    `,
  ],
  [
    "lambda_reuse",
    `
      var a, b, f = function() {
          console.log(a);
      };
      f();
      a = "PASS";
      b = "FAIL";
      f();
      if (console.log(typeof b))
          console.log(b);
    `,
  ],
  [
    "conditional_write",
    `
      var a = "FAIL", b;
      if (console)
          a = "PASS";
      b = [a, 42].join();
      console.log(b);
    `,
  ],
  [
    "issue_4155",
    `
      (function() {
          try {
              throw "PASS";
          } catch (e) {
              var a;
              (function() {
                  console.log(e, a);
              })(a = NaN);
          }
          var e = function() {};
          e && console.log(typeof e);
      })();
    `,
  ],
  [
    "issue_4157_1",
    `
      (function() {
          try {
              for (var a = "FAIL"; a; a++)
                  return;
              var b = 0;
          } finally {
              console.log(b);
          }
      })();
    `,
  ],
  [
    "issue_4157_2",
    `
      (function() {
          try {
              throw "FAIL";
          } catch (e) {
              for (var a = e; a; a++)
                  return;
              var b = 0;
          } finally {
              console.log(b);
          }
      })();
    `,
  ],
  [
    "issue_4168",
    `
      var o = {
          f: function(a, b, c) {
              var d = a.d;
              var e = b.e;
              var f = c.f;
              this.g(arguments);
              if (d)
                  console.log(e, f);
          },
          g: function(args) {
              console.log(args[0], args[1], args[2]);
          },
      };
      o.f("PASS", true, 42);
    `,
  ],
  [
    "issue_4168_use_strict",
    `
      "use strict";
      var o = {
          f: function(a, b, c) {
              var d = a.d;
              var e = b.e;
              var f = c.f;
              this.g(arguments);
              if (d)
                  console.log(e, f);
          },
          g: function(args) {
              console.log(args[0], args[1], args[2]);
          },
      };
      o.f("PASS", true, 42);
    `,
  ],
  [
    "issue_4237_1",
    `
      console.log(function(a) {
          do {
              var b = a++;
              if (b)
                  return "FAIL";
              continue;
              var c = 42;
          } while ("undefined" != typeof c);
          return "PASS";
      }(0));
    `,
  ],
  [
    "issue_4237_2",
    `
      console.log(function(a) {
          do {
              switch (0) {
                case 0:
                  var b = a++;
                default:
                  while (b)
                      return "FAIL";
              }
              try {
                  var c = 0;
              } finally {
                  continue;
              }
              var d = 0;
          } while ("undefined" != typeof d);
          return "PASS";
      }(0));
    `,
  ],
  [
    "issue_4253",
    `
      switch (0) {
        default:
          var a = "FAIL";
          a = a && a;
          try {
              break;
          } catch (e) {}
          var b = 42;
      }
      console.log(b);
    `,
  ],
  [
    "issue_4255",
    `
      L: for (var a = 2; --a;)
          for (var b = 0; console.log(b); --b)
              break L;
    `,
  ],
  [
    "issue_4257",
    `
      var a = 0;
      for (var i = 0; i < 2; i++)
          switch (--a) {
            case 0:
              var b = 0;
              break;
            case 0:
            default:
              var c = 1 + (0 | (b && A));
              console.log(c);
          }
    `,
  ],
  [
    "issue_4628",
    `
      (function() {
          try {
              console;
          } finally {
              var b = a;
          }
          for (var a in "foo");
          console.log(b);
      })();
    `,
  ],
  [
    "issue_4653",
    `
      var a = 1, b;
      function f(c, d) {
          c || console.log(d);
      }
      f(a++ + (b = b), b |= console.log(a));
    `,
  ],
  [
    "issue_4759",
    `
      var i = 2, a = 1, b, c, d;
      while (i--) {
          try {
              if (1 != b) {
                  d = [];
                  null.p;
                  c = d;
              } else {
                  b = 0;
                  a = c;
              }
          } catch (e) {}
          b = a;
      }
      console.log(a);
    `,
  ],
  [
    "issue_4761",
    `
      var a = "FAIL", b;
      try {
          !a && --a && (b = 0)[console] || console.log(b);
      } catch (e) {}
    `,
  ],
  [
    "issue_4956_1",
    `
      var a, b;
      function f(c) {
          switch (c) {
            case 0:
              a = { p: 42 };

            case 1:
              b = a.p;
              console.log(b);
          }
      }
      f(0);
      f(1);
    `,
  ],
  [
    "issue_4956_2",
    `
      var a, b;
      function f(c) {
          if (0 == c) {
              console;
              a = { p: 42 };
          }
          b = a.p;
          if (1 == c)
              console.log(b);
      }
      f(0);
      f(1);
    `,
  ],
  [
    "issue_5182",
    `
      try {
          var con = console;
      } catch (x) {}
      global.log = con.log;
      var jump = function(x) {
          console.log("JUMP:", x * 10);
          return x + x;
      };
      var jump2 = jump;
      var run = function(x) {
          console.log("RUN:", x * -10);
          return x * x;
      };
      var run2 = run;
      var bar = (x, y) => {
          console.log("BAR:", x + y);
          return x - y;
      };
      var bar2 = bar;
      var obj = {
          foo: bar2,
          go: run2,
          not_used: jump2,
      };
      console.log(obj.foo(1, 2), global.log("PASS"));
    `,
  ],
  [
    "issue_5420",
    `
      do {
          var a = "FAIL 1";
          a && a.p;
          a = "FAIL 2";
          try {
              continue;
          } catch (e) {}
          var b = "FAIL 3";
      } while (console.log(b || "PASS"));
    `,
  ],
  [
    "issue_5451",
    `
      A = 1;
      var a = 1, b;
      console.log(function f() {
          return a-- && f(b = A, b);
      }());
    `,
  ],
  [
    "issue_5471_1",
    `
      var a = "FAIL 1";
      function f(b, c) {
          function g() {
              if (console)
                  return 42;
              else
                  c = "FAIL 2";
          }
          var d = g();
          console.log(c || "PASS");
          var e = function h() {
              while (b && e);
          }();
      }
      f(a++) && a;
    `,
  ],
  [
    "issue_5471_2",
    `
      var a = "FAIL 1";
      function f(b, c) {
          function g() {
              if (console)
                  return 42;
              else
                  c = "FAIL 2";
          }
          var d = g();
          console.log(c || "PASS");
          var e = function h() {
              while (b && e);
          }();
      }
      f(a++) && a;
    `,
  ],
  [
    "issue_5714",
    `
      "use strict";
      console.log(function() {
          var i = 1;
          while (i--) {
              var a = function f(b) {
                  console.log(b);
                  var c = function(d) {
                      console.log(typeof d);
                  }(console);
              }();
              var e = 42;
          }
          return e;
      }());
    `,
  ],
  [
    "issue_5770_1",
    `
      L: do {
          if (console)
              for (var a = "FAIL 1"; a; a--)
                  continue L;
          var b = "FAIL 2";
      } while (console.log(b || "PASS"));
    `,
  ],
  [
    "issue_5770_2",
    `
      L: do {
          for (var a = "FAIL 1"; a; a--)
              continue L;
          var b = "FAIL 2";
      } while (console.log(b || "PASS"));
    `,
  ],
  [
    "issue_5772_1",
    `
      (function(a) {
          while (--a)
              return;
          var b = console.log("foo") && (c = 42) ? 0 : console.log(c);
          var c = b;
      })();
    `,
  ],
  [
    "issue_5772_2",
    `
      (function(a) {
          while (--a)
              return;
          var b;
          var c = console.log("foo") && (b = 1) ? 2 : 3;
          console.log(b, c);
      })();
    `,
  ],
];

describe("merge vars", () => {
  const variants = [];
  for (const compress of [false, true]) {
    for (const toplevel of [false, true]) {
      variants.push({ compress, mangle: { toplevel } });
    }
  }
  variants.push({ compress: false, mangle: { toplevel: true, mergeVars: false } });

  for (const options of variants) {
    it.each(cases)(`preserves %s with ${JSON.stringify(options)}`, (_name, code) => {
      const result = minifySync("input.js", code, options);
      expect(result.errors).toEqual([]);
      expect(run(result.code)).toEqual(run(code));
    });
  }
});
