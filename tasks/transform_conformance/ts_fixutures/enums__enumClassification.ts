var E01, E02, E03, E04, E05, E06, E07, E08, E10, E11, E12, E20;
(E01 => {
	const A = 0;
	E01[E01['A'] = A] = 'A';
})(E01 ||= {});
(E02 => {
	const A = 123;
	E02[E02['A'] = A] = 'A';
})(E02 ||= {});
(E03 => {
	const A = 'hello';
	E03['A'] = A;
})(E03 ||= {});
(E04 => {
	const A = 0;
	E04[E04['A'] = A] = 'A';
	const B = 1 + A;
	E04[E04['B'] = B] = 'B';
	const C = 1 + B;
	E04[E04['C'] = C] = 'C';
})(E04 ||= {});
(E05 => {
	const A = 0;
	E05[E05['A'] = A] = 'A';
	const B = 10;
	E05[E05['B'] = B] = 'B';
	const C = 1 + B;
	E05[E05['C'] = C] = 'C';
})(E05 ||= {});
(E06 => {
	const A = 'one';
	E06['A'] = A;
	const B = 'two';
	E06['B'] = B;
	const C = 'three';
	E06['C'] = C;
})(E06 ||= {});
(E07 => {
	const A = 0;
	E07[E07['A'] = A] = 'A';
	const B = 1 + A;
	E07[E07['B'] = B] = 'B';
	const C = 'hi';
	E07['C'] = C;
	const D = 10;
	E07[E07['D'] = D] = 'D';
	const E = 1 + D;
	E07[E07['E'] = E] = 'E';
	const F = 'bye';
	E07['F'] = F;
})(E07 ||= {});
(E08 => {
	const A = 10;
	E08[E08['A'] = A] = 'A';
	const B = 'hello';
	E08['B'] = B;
	const C = A;
	E08[E08['C'] = C] = 'C';
	const D = B;
	E08[E08['D'] = D] = 'D';
	const E = C;
	E08[E08['E'] = E] = 'E';
})(E08 ||= {});
(E10 => {
})(E10 ||= {});
(E11 => {
	const A =  +0;
	E11[E11['A'] = A] = 'A';
	const B = 1 + A;
	E11[E11['B'] = B] = 'B';
	const C = 1 + B;
	E11[E11['C'] = C] = 'C';
})(E11 ||= {});
(E12 => {
	const A = 1 << 0;
	E12[E12['A'] = A] = 'A';
	const B = 1 << 1;
	E12[E12['B'] = B] = 'B';
	const C = 1 << 2;
	E12[E12['C'] = C] = 'C';
})(E12 ||= {});
(E20 => {
	const A = 'foo'.length;
	E20[E20['A'] = A] = 'A';
	const B = A + 1;
	E20[E20['B'] = B] = 'B';
	const C =  +'123';
	E20[E20['C'] = C] = 'C';
	const D = Math.sin(1);
	E20[E20['D'] = D] = 'D';
})(E20 ||= {});
