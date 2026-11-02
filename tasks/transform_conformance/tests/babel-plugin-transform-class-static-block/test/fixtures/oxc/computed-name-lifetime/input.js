const classes = [];
for (const key of ['a', 'b']) {
  classes.push(class { [key] = class { static { this.seen = this.name; } }; });
}
console.log(new classes[0]().a.seen, new classes[1]().b.seen);

function* suspended() {
  const classes = [];
  while (classes.length < 2 && classes.push(class {
    [yield classes.length] = class { static { this.seen = this.name; } };
    static { Object.freeze(this); }
  })) {}
  return classes;
}
