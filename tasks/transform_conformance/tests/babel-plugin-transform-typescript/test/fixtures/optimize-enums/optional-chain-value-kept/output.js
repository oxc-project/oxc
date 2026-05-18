var Foo = /* @__PURE__ */ function(Foo) {
	Foo[Foo["A"] = 1] = "A";
	Foo[Foo["B"] = 2] = "B";
	return Foo;
}(Foo || {});
1;
2;
console.log(Foo);
