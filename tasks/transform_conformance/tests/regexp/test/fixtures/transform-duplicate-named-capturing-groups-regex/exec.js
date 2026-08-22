const regexp = /(?<named>a)|(?<named>b)|c/d;

expect("a".match(regexp).groups).toEqual({ named: "a" });
expect("b".match(regexp).groups).toEqual({ named: "b" });
expect("c".match(regexp).groups).toEqual({ named: undefined });
expect("b".match(regexp).indices.groups).toEqual({ named: [0, 1] });

expect("a".replace(regexp, "[$<named>]")).toBe("[a]");
expect("b".replace(regexp, "[$<named>]")).toBe("[b]");
expect("c".replace(regexp, "[$<named>]")).toBe("[]");

expect(/(?:(?<named>a)|(?<named>b))\k<named>/.test("aa")).toBe(true);
expect(/(?:(?<named>a)|(?<named>b))\k<named>/.test("bb")).toBe(true);
expect(/(?:(?<named>a)|(?<named>b))\k<named>/.test("ab")).toBe(false);

const repeated = /(?:(?:(?<named>a)|(?<named>b))\k<named>){2}/.exec("aabb");
expect(repeated.groups).toEqual({ named: "b" });

const ordered = /(?<y>a)(?<x>a)|(?<x>b)(?<y>b)/.exec("bb");
expect(Object.keys(ordered.groups)).toEqual(["y", "x"]);

const proto = /(?<__proto__>a)|(?<__proto__>b)/.exec("b");
expect(proto.groups.__proto__).toBe("b");
