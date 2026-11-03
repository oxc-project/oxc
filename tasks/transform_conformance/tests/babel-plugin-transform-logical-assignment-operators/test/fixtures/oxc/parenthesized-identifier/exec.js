let a, b = 1, c;
(a) ||= function () {};
(b) &&= () => {};
(c) ??= class {};
expect([a.name, b.name, c.name]).toEqual(["", "", ""]);
