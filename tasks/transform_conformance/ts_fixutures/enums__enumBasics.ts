var E1, E2, E3, E4, E5, E6, E7, E8, E9;
(E1 => {
	const A = 0;
	E1[E1['A'] = A] = 'A';
	const B = 1 + A;
	E1[E1['B'] = B] = 'B';
	const C = 1 + B;
	E1[E1['C'] = C] = 'C';
})(E1 ||= {});
var x = E1.A;
var e = E1;
var e;
var e;
var s = E1[e.A];
var s;
(E2 => {
	const A = 1;
	E2[E2['A'] = A] = 'A';
	const B = 2;
	E2[E2['B'] = B] = 'B';
	const C = 3;
	E2[E2['C'] = C] = 'C';
})(E2 ||= {});
(E3 => {
	const X = 'foo'.length;
	E3[E3['X'] = X] = 'X';
	const Y = 4 + 3;
	E3[E3['Y'] = Y] = 'Y';
	const Z =  +'foo';
	E3[E3['Z'] = Z] = 'Z';
})(E3 ||= {});
(E4 => {
	const X = 0;
	E4[E4['X'] = X] = 'X';
	const Y = 1 + X;
	E4[E4['Y'] = Y] = 'Y';
	const Z = 'foo'.length;
	E4[E4['Z'] = Z] = 'Z';
})(E4 ||= {});
(E5 => {
	const A = 0;
	E5[E5['A'] = A] = 'A';
	const B = 3;
	E5[E5['B'] = B] = 'B';
	const C = 1 + B;
	E5[E5['C'] = C] = 'C';
})(E5 ||= {});
(E6 => {
	const A = 0;
	E6[E6['A'] = A] = 'A';
	const B = 0;
	E6[E6['B'] = B] = 'B';
	const C = 1 + B;
	E6[E6['C'] = C] = 'C';
})(E6 ||= {});
(E7 => {
	const A = 'foo'['foo'];
	E7[E7['A'] = A] = 'A';
})(E7 ||= {});
(E8 => {
	const B = 'foo'['foo'];
	E8[E8['B'] = B] = 'B';
})(E8 ||= {});
(E9 => {
	const A = 0;
	E9[E9['A'] = A] = 'A';
	const B = A;
	E9[E9['B'] = B] = 'B';
})(E9 ||= {});
var doNotPropagate = [E8.B, E7.A, E4.Z, E3.X, E3.Y, E3.Z];
var doPropagate = [E9.A, E9.B, E6.B, E6.C, E6.A, E5.A, E5.B, E5.C];
