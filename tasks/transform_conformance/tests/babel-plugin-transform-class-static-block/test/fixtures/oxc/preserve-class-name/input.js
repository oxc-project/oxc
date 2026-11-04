const A = class { static { this.seen = this.name; } };
const object = { [Symbol.for("key")]: class { static { this.seen = this.name; } } };
