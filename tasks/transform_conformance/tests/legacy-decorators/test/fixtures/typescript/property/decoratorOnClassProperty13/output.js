class C {
	#prop_accessor_storage;
	get prop() {
		return this.#prop_accessor_storage;
	}
	set prop(value) {
		this.#prop_accessor_storage = value;
	}
}
babelHelpers.decorate([
	dec
], C.prototype, "prop", null);
