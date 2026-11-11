export default (class {}).foo;
export default (class {})["x"];
export default (class {}).getInstance();
export default (function () {}).toString();
export default (function () {}) + 1;
export default (function () {}) || 1;
export default (function () {})`t`;
export default (class {})();
// An ancestor with its own parens covers it
export default (class {}[1] = 1);
export default (function () {}, 1);
