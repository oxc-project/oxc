class Foo {
	constructor() {
		this.x = 1;
		this.self = this;
	}
	m() {
		return this.x;
	}
	getSelf() {
		return this;
	}
	test() {
		var _Foo, _Foo2, _Foo3, _o$Foo, _o$Foo2, _o$Foo3, _o$Foo$self$getSelf, _o$Foo$self$getSelf2, _o$Foo4, _o$Foo4$self, _o$Foo$self$getSelf3, _o$Foo$self$getSelf4, _o$Foo$self, _o$Foo$self2, _fn$Foo$self$getSelf, _fn$Foo$self$getSelf2, _fn, _fn$self, _fn$Foo$self$getSelf3, _fn$Foo$self$getSelf4, _fn$Foo$self, _fn$Foo$self2;
		const Foo = this;
		const o = { Foo };
		const fn = function() {
			return o;
		};
		(Foo === null || Foo === void 0 ? void 0 : (() => {}).call.bind((_Foo = Foo)["m"], _Foo))();
		(Foo === null || Foo === void 0 ? void 0 : (() => {}).call.bind((_Foo2 = Foo)["m"], _Foo2))().toString;
		(Foo === null || Foo === void 0 ? void 0 : (() => {}).call.bind((_Foo3 = Foo)["m"], _Foo3))().toString();
		(o === null || o === void 0 ? void 0 : (() => {}).call.bind((_o$Foo = o.Foo).m, _o$Foo))();
		(o === null || o === void 0 ? void 0 : (() => {}).call.bind((_o$Foo2 = o.Foo).m, _o$Foo2))().toString;
		(o === null || o === void 0 ? void 0 : (() => {}).call.bind((_o$Foo3 = o.Foo).m, _o$Foo3))().toString();
		((_o$Foo$self$getSelf = ((_o$Foo4 = o.Foo) === null || _o$Foo4 === void 0 ? void 0 : (() => {}).call.bind((_o$Foo4$self = _o$Foo4.self).getSelf, _o$Foo4$self))()) === null || _o$Foo$self$getSelf === void 0 ? void 0 : (() => {}).call.bind((_o$Foo$self$getSelf2 = _o$Foo$self$getSelf).m, _o$Foo$self$getSelf2))();
		((_o$Foo$self$getSelf3 = ((_o$Foo$self = o.Foo.self) === null || _o$Foo$self === void 0 ? void 0 : (() => {}).call.bind((_o$Foo$self2 = _o$Foo$self).getSelf, _o$Foo$self2))()) === null || _o$Foo$self$getSelf3 === void 0 ? void 0 : (() => {}).call.bind((_o$Foo$self$getSelf4 = _o$Foo$self$getSelf3).m, _o$Foo$self$getSelf4))();
		((_fn$Foo$self$getSelf = ((_fn = fn()) === null || _fn === void 0 || (_fn = _fn.Foo) === null || _fn === void 0 ? void 0 : (() => {}).call.bind((_fn$self = _fn.self).getSelf, _fn$self))()) === null || _fn$Foo$self$getSelf === void 0 ? void 0 : (() => {}).call.bind((_fn$Foo$self$getSelf2 = _fn$Foo$self$getSelf).m, _fn$Foo$self$getSelf2))();
		((_fn$Foo$self$getSelf3 = (fn === null || fn === void 0 || (_fn$Foo$self = fn().Foo.self) === null || _fn$Foo$self === void 0 ? void 0 : (() => {}).call.bind((_fn$Foo$self2 = _fn$Foo$self).getSelf, _fn$Foo$self2))()) === null || _fn$Foo$self$getSelf3 === void 0 ? void 0 : (() => {}).call.bind((_fn$Foo$self$getSelf4 = _fn$Foo$self$getSelf3).m, _fn$Foo$self$getSelf4))();
	}
}
new Foo().test();
