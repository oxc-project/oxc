for (class { static { a in b; } };;);
for (let C = class { static { a in b; } };;);

// Restore the outer initializer's disabled `in` context after the static block.
for (class { static { a in b; } }.x in c);
