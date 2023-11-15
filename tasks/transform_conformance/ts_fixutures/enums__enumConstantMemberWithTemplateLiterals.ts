var T1, T2, T3, T4, T5, T6;
(T1 => {
	const a = `1`;
	T1['a'] = a;
})(T1 ||= {});
(T2 => {
	const a = `1`;
	T2['a'] = a;
	const b = '2';
	T2['b'] = b;
	const c = 3;
	T2[T2['c'] = c] = 'c';
})(T2 ||= {});
(T3 => {
	const a = `1` + `1`;
	T3[T3['a'] = a] = 'a';
})(T3 ||= {});
(T4 => {
	const a = `1`;
	T4['a'] = a;
	const b = `1` + `1`;
	T4[T4['b'] = b] = 'b';
	const c = `1` + '2';
	T4[T4['c'] = c] = 'c';
	const d = '2' + `1`;
	T4[T4['d'] = d] = 'd';
	const e = '2' + `1` + `1`;
	T4[T4['e'] = e] = 'e';
})(T4 ||= {});
(T5 => {
	const a = `1`;
	T5['a'] = a;
	const b = `1` + `2`;
	T5[T5['b'] = b] = 'b';
	const c = `1` + `2` + `3`;
	T5[T5['c'] = c] = 'c';
	const d = 1;
	T5[T5['d'] = d] = 'd';
	const e = `1` - `1`;
	T5[T5['e'] = e] = 'e';
	const f = `1` + 1;
	T5[T5['f'] = f] = 'f';
	const g = `1${'2'}3`;
	T5['g'] = g;
	const h = `1`.length;
	T5[T5['h'] = h] = 'h';
})(T5 ||= {});
(T6 => {
	const a = 1;
	T6[T6['a'] = a] = 'a';
	const b = `12`.length;
	T6[T6['b'] = b] = 'b';
})(T6 ||= {});
