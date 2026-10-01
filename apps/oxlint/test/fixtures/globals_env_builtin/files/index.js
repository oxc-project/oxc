// ES builtins should resolve, even though `env` in config does not include `builtin`
Number.isFinite(1);
new Intl.NumberFormat();

// Enabled by `browser` env
window.alert("hello");

// Not defined anywhere
notDefinedAnywhere;
