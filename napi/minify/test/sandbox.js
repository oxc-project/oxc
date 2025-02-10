// https://github.com/terser/terser/blob/v5.37.0/test/sandbox.js

import vm from 'vm';

// import { Console } from "console";

// global.console = new Console({
// stdout: process.stdout,
// stderr: process.stderr,
// colorMode: false
// });

function safe_log(arg, level) {
  if (arg) {
    switch (typeof arg) {
      case 'function':
        return arg.toString();
      case 'object':
        if (/Error$/.test(arg.name)) return arg.toString();
        arg.constructor.toString();
        if (level--) {
          for (var key in arg) {
            if (!Object.getOwnPropertyDescriptor(arg, key).get) {
              arg[key] = safe_log(arg[key], level);
            }
          }
        }
    }
  }
  return arg;
}

function strip_func_ids(text) {
  return text.toString().replace(/F[0-9]{6}N/g, '<F<>N>');
}

var FUNC_TOSTRING = `
    [ Array, Boolean, Error, Function, Number, Object, RegExp, String].forEach(function(f) {
        f.toString = Function.prototype.toString;
    });
    Function.prototype.toString = function() {
        var id = 100000;
        return function() {
            var n = this.name;
            if (!/^F[0-9]{6}N$/.test(n)) {
                n = "F" + ++id + "N";
                ${
  Object.getOwnPropertyDescriptor(Function.prototype, 'name').configurable
    ? 'Object.defineProperty(this, "name", { get: () => n });'
    : ''
}
            }
            return "function " + n + "() {...}";
        }
    }();
`;

export function run_code(code, prepend_code = '') {
  var stdout = '';
  var original_write = process.stdout.write;
  process.stdout.write = function(chunk) {
    stdout += chunk;
  };
  try {
    const global = {
      console: {
        log: function(msg) {
          if (arguments.length == 1 && typeof msg == 'string') {
            return console.log(msg);
          }
          return console.log.apply(
            console,
            [].map.call(arguments, function(arg) {
              return safe_log(arg, 3);
            }),
          );
        },
      },
      id: x => x,
      leak: () => {},
      pass: () => {
        global.console.log('PASS');
      },
      fail: () => {
        global.console.log('FAIL');
      },
    };
    global.global = global;
    vm.runInNewContext(
      [
        FUNC_TOSTRING,
        '!function() {',
        prepend_code + code,
        '}();',
      ].join('\n'),
      global,
    );
    return stdout;
  } catch (ex) {
    return ex;
  } finally {
    process.stdout.write = original_write;
  }
}

export function same_stdout(expected, actual) {
  return typeof expected == typeof actual && strip_func_ids(expected) == strip_func_ids(actual);
}
