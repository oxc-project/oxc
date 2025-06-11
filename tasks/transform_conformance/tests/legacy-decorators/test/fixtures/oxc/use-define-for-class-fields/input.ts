class Cls {
	x: number;
	y = 1;
	@dce
	z;
}

class StaticCls {
	static x: number;
	static y = 1;
	@dce
	static z;
}
