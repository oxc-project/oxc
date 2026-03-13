class C {
	#a_accessor_storage;
	get a() {
		return this.#a_accessor_storage;
	}
	set a(value) {
		this.#a_accessor_storage = value;
	}
	static #b_accessor_storage;
	static get b() {
		return this.#b_accessor_storage;
	}
	static set b(value) {
		this.#b_accessor_storage = value;
	}
	#c_accessor_storage = "hello";
	get c() {
		return this.#c_accessor_storage;
	}
	set c(value) {
		this.#c_accessor_storage = value;
	}
}
babelHelpers.decorate([dec], C.prototype, "a", null);
babelHelpers.decorate([dec], C, "b", null);
babelHelpers.decorate([dec], C.prototype, "c", null);
