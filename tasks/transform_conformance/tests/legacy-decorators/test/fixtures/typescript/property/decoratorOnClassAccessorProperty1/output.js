class C {
	#_a_accessor_storage;
	get a() {
		return this.#_a_accessor_storage;
	}
	set a(value) {
		this.#_a_accessor_storage = value;
	}
	static #_b_accessor_storage;
	static get b() {
		return C.#_b_accessor_storage;
	}
	static set b(value) {
		C.#_b_accessor_storage = value;
	}
	#_c_accessor_storage = "hello";
	get c() {
		return this.#_c_accessor_storage;
	}
	set c(value) {
		this.#_c_accessor_storage = value;
	}
}
babelHelpers.decorate([dec], C.prototype, "a", null);
babelHelpers.decorate([dec], C, "b", null);
babelHelpers.decorate([dec], C.prototype, "c", null);
