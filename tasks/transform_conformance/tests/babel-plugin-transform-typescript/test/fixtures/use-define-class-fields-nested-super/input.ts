class Foo {}

class Bar extends Foo {
	test = "initial";
	constructor() {
		super(), console.log();
	}
}

class Baz extends Foo {
	constructor(public code: string) {
		super(), console.log(this);
	}
}
