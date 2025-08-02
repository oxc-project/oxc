class Foo {
	// Property with definite assignment assertion
	#a!: string;
	b!: string;
	method() {
		this.#a = "hello";
		this.b = "world";
	}
}
