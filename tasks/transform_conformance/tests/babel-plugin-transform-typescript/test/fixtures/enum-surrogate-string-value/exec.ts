enum E { x = "keep", "\uD800" = "x", B = E["\uD800"], C = B }
enum F { x = "keep", A = E["\uD800"] }
enum E { D = E["\uD800"] }
expect(E.x).toBe("keep");
expect(E.B).toBe("x");
expect(E.C).toBe("x");
expect(E.D).toBe("x");
expect(F.x).toBe("keep");
expect(F.A).toBe("x");
