var T1, T2, T3, T4, T5;
(T1 => {
	const a = '1';
	T1['a'] = a;
	const b = '1' + '2';
	T1[T1['b'] = b] = 'b';
	const c = '1' + '2' + '3';
	T1[T1['c'] = c] = 'c';
	const d = 'a' - 'a';
	T1[T1['d'] = d] = 'd';
	const e = 'a' + 1;
	T1[T1['e'] = e] = 'e';
})(T1 ||= {});
(T2 => {
	const a = '1';
	T2['a'] = a;
	const b = '1' + '2';
	T2[T2['b'] = b] = 'b';
})(T2 ||= {});
(T3 => {
	const a = '1';
	T3['a'] = a;
	const b = '1' + '2';
	T3[T3['b'] = b] = 'b';
	const c = 1;
	T3[T3['c'] = c] = 'c';
	const d = 1 + 2;
	T3[T3['d'] = d] = 'd';
})(T3 ||= {});
(T4 => {
	const a = '1';
	T4['a'] = a;
})(T4 ||= {});
(T5 => {
	const a = '1' + '2';
	T5[T5['a'] = a] = 'a';
})(T5 ||= {});
