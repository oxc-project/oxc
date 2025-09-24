// Examples of incorrect code for no-implied-eval rule

setTimeout('alert("Hi!");', 100);

setInterval('alert("Hi!");', 100);

setImmediate('alert("Hi!")');

window.setTimeout('count = 5', 10);

window.setInterval('foo = bar', 10);

const fn = new Function('a', 'b', 'return a + b');