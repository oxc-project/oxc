async function* test(x) {
	var E;
	(E => {
		const foo = await x;
		E[E['foo'] = foo] = 'foo';
		const baz = yield 1;
		E[E['baz'] = baz] = 'baz';
	})(E ||= {});
}
