var E1, E2, E3, E5, E6;
(E1 => {
	const a = 1;
	E1[E1['a'] = a] = 'a';
	const b = 1 + a;
	E1[E1['b'] = b] = 'b';
})(E1 ||= {});
(E2 => {
	const a =  -1;
	E2[E2['a'] = a] = 'a';
	const b = 1 + a;
	E2[E2['b'] = b] = 'b';
})(E2 ||= {});
(E3 => {
	const a = 0.1;
	E3[E3['a'] = a] = 'a';
	const b = 1 + a;
	E3[E3['b'] = b] = 'b';
})(E3 ||= {});
(E5 => {
	const a = 1 / 0;
	E5[E5['a'] = a] = 'a';
	const b = 2 / 0.0;
	E5[E5['b'] = b] = 'b';
	const c = 1.0 / 0.0;
	E5[E5['c'] = c] = 'c';
	const d = 0.0 / 0.0;
	E5[E5['d'] = d] = 'd';
	const e = NaN;
	E5[E5['e'] = e] = 'e';
	const f = Infinity;
	E5[E5['f'] = f] = 'f';
	const g =  -Infinity;
	E5[E5['g'] = g] = 'g';
})(E5 ||= {});
(E6 => {
	const a = 1 / 0;
	E6[E6['a'] = a] = 'a';
	const b = 2 / 0.0;
	E6[E6['b'] = b] = 'b';
	const c = 1.0 / 0.0;
	E6[E6['c'] = c] = 'c';
	const d = 0.0 / 0.0;
	E6[E6['d'] = d] = 'd';
	const e = NaN;
	E6[E6['e'] = e] = 'e';
	const f = Infinity;
	E6[E6['f'] = f] = 'f';
	const g =  -Infinity;
	E6[E6['g'] = g] = 'g';
})(E6 ||= {});
