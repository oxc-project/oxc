class Cls {
	x: number;
	y = 1;
	@dce
	z;

	#x: number;
	#y = 1;
	@dce
	#z;

	[x]: number;
	[y] = 1;
	@dce
	[z]: number;
}

class ClsWithConstructor extends Cls {
	constructor() {
		console.log("before super()");
		super();
		console.log("after super()");
	}

	x: number;
	y = 1;
	@dce
	z;

	[x]: number;
	[y] = 1;
	@dce
	[z]: number;
}

class StaticCls {
	static x: number;
	static y = 1;
	@dce
	static z;

	static [x]: number;
	static [y] = 1;
	@dce
	static [z]: number;
}

class ClsWithComputedKeyMethods {
	[x()]() {}
	[y()] = 1;
	[z()]() {}
	[a()] = 2;
	[b()];
	[c()] = 3;
	accessor [e()] = 4;
}

class StaticClsWithComputedKeyMethods {
	static [x()]() {}
	static [y()] = 1;
	static [z()]() {}
	static [a()] = 2;
	static [b()];
	static [c()] = 3;
	static accessor [e()] = 4;
}
