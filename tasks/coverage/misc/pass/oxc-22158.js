// https://github.com/oxc-project/oxc/issues/22158
// `await` in initializer of an `export var` declaration must not be
// reported as a duplicated export when parsed with unambiguous `.js`.
export var a = await + 1;
export var b = await(1);
export var c = await + 0;
