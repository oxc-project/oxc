class C {
	#_prop_accessor_storage;
	get prop() {
		return this.#_prop_accessor_storage;
	}
	set prop(value) {
		this.#_prop_accessor_storage = value;
	}
}
babelHelpers.decorate([dec], C.prototype, "prop", null);
