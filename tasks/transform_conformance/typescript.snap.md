# typescript/tests/cases/conformance/enums/awaitAndYield.ts
```error

  × `await` is only allowed within async functions and at the top levels of modules
   ╭─[typescript/tests/cases/conformance/enums/awaitAndYield.ts:5:15]
 4 │     enum E {
 5 │         foo = await x,
   ·               ─────
 6 │         baz = yield 1,
   ╰────


```

# typescript/tests/cases/conformance/enums/enumBasics.ts
```typescript
var E1 = (E1 => {
	const A = 0;
	E1[E1['A'] = A] = 'A';
	const B = 1 + A;
	E1[E1['B'] = B] = 'B';
	const C = 1 + B;
	E1[E1['C'] = C] = 'C';
	return E1;
})(E1 || {});
var x = E1.A;
var e = E1;
var e;
var e;
var s = E1[e.A];
var s;
var E2 = (E2 => {
	const A = 1;
	E2[E2['A'] = A] = 'A';
	const B = 2;
	E2[E2['B'] = B] = 'B';
	const C = 3;
	E2[E2['C'] = C] = 'C';
	return E2;
})(E2 || {});
var E3 = (E3 => {
	const X = 'foo'.length;
	E3[E3['X'] = X] = 'X';
	const Y = 4 + 3;
	E3[E3['Y'] = Y] = 'Y';
	const Z =  +'foo';
	E3[E3['Z'] = Z] = 'Z';
	return E3;
})(E3 || {});
var E4 = (E4 => {
	const X = 0;
	E4[E4['X'] = X] = 'X';
	const Y = 1 + X;
	E4[E4['Y'] = Y] = 'Y';
	const Z = 'foo'.length;
	E4[E4['Z'] = Z] = 'Z';
	return E4;
})(E4 || {});
var E5 = (E5 => {
	const A = 0;
	E5[E5['A'] = A] = 'A';
	const B = 3;
	E5[E5['B'] = B] = 'B';
	const C = 1 + B;
	E5[E5['C'] = C] = 'C';
	return E5;
})(E5 || {});
var E6 = (E6 => {
	const A = 0;
	E6[E6['A'] = A] = 'A';
	const B = 0;
	E6[E6['B'] = B] = 'B';
	const C = 1 + B;
	E6[E6['C'] = C] = 'C';
	return E6;
})(E6 || {});
var E7 = (E7 => {
	const A = 'foo'['foo'];
	E7[E7['A'] = A] = 'A';
	return E7;
})(E7 || {});
var E8 = (E8 => {
	const B = 'foo'['foo'];
	E8[E8['B'] = B] = 'B';
	return E8;
})(E8 || {});
var E9 = (E9 => {
	const A = 0;
	E9[E9['A'] = A] = 'A';
	const B = A;
	E9[E9['B'] = B] = 'B';
	return E9;
})(E9 || {});
var doNotPropagate = [E8.B, E7.A, E4.Z, E3.X, E3.Y, E3.Z];
var doPropagate = [E9.A, E9.B, E6.B, E6.C, E6.A, E5.A, E5.B, E5.C];

```

# typescript/tests/cases/conformance/enums/enumClassification.ts
```typescript
var E01 = (E01 => {
	const A = 0;
	E01[E01['A'] = A] = 'A';
	return E01;
})(E01 || {});
var E02 = (E02 => {
	const A = 123;
	E02[E02['A'] = A] = 'A';
	return E02;
})(E02 || {});
var E03 = (E03 => {
	const A = 'hello';
	E03['A'] = A;
	return E03;
})(E03 || {});
var E04 = (E04 => {
	const A = 0;
	E04[E04['A'] = A] = 'A';
	const B = 1 + A;
	E04[E04['B'] = B] = 'B';
	const C = 1 + B;
	E04[E04['C'] = C] = 'C';
	return E04;
})(E04 || {});
var E05 = (E05 => {
	const A = 0;
	E05[E05['A'] = A] = 'A';
	const B = 10;
	E05[E05['B'] = B] = 'B';
	const C = 1 + B;
	E05[E05['C'] = C] = 'C';
	return E05;
})(E05 || {});
var E06 = (E06 => {
	const A = 'one';
	E06['A'] = A;
	const B = 'two';
	E06['B'] = B;
	const C = 'three';
	E06['C'] = C;
	return E06;
})(E06 || {});
var E07 = (E07 => {
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
	return E07;
})(E07 || {});
var E08 = (E08 => {
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
	return E08;
})(E08 || {});
var E10 = (E10 => {
	return E10;
})(E10 || {});
var E11 = (E11 => {
	const A =  +0;
	E11[E11['A'] = A] = 'A';
	const B = 1 + A;
	E11[E11['B'] = B] = 'B';
	const C = 1 + B;
	E11[E11['C'] = C] = 'C';
	return E11;
})(E11 || {});
var E12 = (E12 => {
	const A = 1 << 0;
	E12[E12['A'] = A] = 'A';
	const B = 1 << 1;
	E12[E12['B'] = B] = 'B';
	const C = 1 << 2;
	E12[E12['C'] = C] = 'C';
	return E12;
})(E12 || {});
var E20 = (E20 => {
	const A = 'foo'.length;
	E20[E20['A'] = A] = 'A';
	const B = A + 1;
	E20[E20['B'] = B] = 'B';
	const C =  +'123';
	E20[E20['C'] = C] = 'C';
	const D = Math.sin(1);
	E20[E20['D'] = D] = 'D';
	return E20;
})(E20 || {});

```

# typescript/tests/cases/conformance/enums/enumConstantMemberWithString.ts
```typescript
var T1 = (T1 => {
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
	return T1;
})(T1 || {});
var T2 = (T2 => {
	const a = '1';
	T2['a'] = a;
	const b = '1' + '2';
	T2[T2['b'] = b] = 'b';
	return T2;
})(T2 || {});
var T3 = (T3 => {
	const a = '1';
	T3['a'] = a;
	const b = '1' + '2';
	T3[T3['b'] = b] = 'b';
	const c = 1;
	T3[T3['c'] = c] = 'c';
	const d = 1 + 2;
	T3[T3['d'] = d] = 'd';
	return T3;
})(T3 || {});
var T4 = (T4 => {
	const a = '1';
	T4['a'] = a;
	return T4;
})(T4 || {});
var T5 = (T5 => {
	const a = '1' + '2';
	T5[T5['a'] = a] = 'a';
	return T5;
})(T5 || {});

```

# typescript/tests/cases/conformance/enums/enumConstantMemberWithStringEmitDeclaration.ts
```typescript
var T1 = (T1 => {
	const a = '1';
	T1['a'] = a;
	const b = '1' + '2';
	T1[T1['b'] = b] = 'b';
	const c = '1' + '2' + '3';
	T1[T1['c'] = c] = 'c';
	return T1;
})(T1 || {});
var T2 = (T2 => {
	const a = '1';
	T2['a'] = a;
	const b = '1' + '2';
	T2[T2['b'] = b] = 'b';
	return T2;
})(T2 || {});
var T3 = (T3 => {
	const a = '1';
	T3['a'] = a;
	const b = '1' + '2';
	T3[T3['b'] = b] = 'b';
	return T3;
})(T3 || {});
var T4 = (T4 => {
	const a = '1';
	T4['a'] = a;
	return T4;
})(T4 || {});
var T5 = (T5 => {
	const a = '1' + '2';
	T5[T5['a'] = a] = 'a';
	return T5;
})(T5 || {});

```

# typescript/tests/cases/conformance/enums/enumConstantMemberWithTemplateLiterals.ts
```typescript
var T1 = (T1 => {
	const a = `1`;
	T1['a'] = a;
	return T1;
})(T1 || {});
var T2 = (T2 => {
	const a = `1`;
	T2['a'] = a;
	const b = '2';
	T2['b'] = b;
	const c = 3;
	T2[T2['c'] = c] = 'c';
	return T2;
})(T2 || {});
var T3 = (T3 => {
	const a = `1` + `1`;
	T3[T3['a'] = a] = 'a';
	return T3;
})(T3 || {});
var T4 = (T4 => {
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
	return T4;
})(T4 || {});
var T5 = (T5 => {
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
	return T5;
})(T5 || {});
var T6 = (T6 => {
	const a = 1;
	T6[T6['a'] = a] = 'a';
	const b = `12`.length;
	T6[T6['b'] = b] = 'b';
	return T6;
})(T6 || {});

```

# typescript/tests/cases/conformance/enums/enumConstantMemberWithTemplateLiteralsEmitDeclaration.ts
```typescript
var T1 = (T1 => {
	const a = `1`;
	T1['a'] = a;
	return T1;
})(T1 || {});
var T2 = (T2 => {
	const a = `1`;
	T2['a'] = a;
	const b = '2';
	T2['b'] = b;
	const c = 3;
	T2[T2['c'] = c] = 'c';
	return T2;
})(T2 || {});
var T3 = (T3 => {
	const a = `1` + `1`;
	T3[T3['a'] = a] = 'a';
	return T3;
})(T3 || {});
var T4 = (T4 => {
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
	return T4;
})(T4 || {});
var T5 = (T5 => {
	const a = `1`;
	T5['a'] = a;
	const b = `1` + `2`;
	T5[T5['b'] = b] = 'b';
	const c = `1` + `2` + `3`;
	T5[T5['c'] = c] = 'c';
	const d = 1;
	T5[T5['d'] = d] = 'd';
	return T5;
})(T5 || {});
var T6 = (T6 => {
	const a = 1;
	T6[T6['a'] = a] = 'a';
	const b = `12`.length;
	T6[T6['b'] = b] = 'b';
	return T6;
})(T6 || {});

```

# typescript/tests/cases/conformance/enums/enumConstantMembers.ts
```typescript
var E1 = (E1 => {
	const a = 1;
	E1[E1['a'] = a] = 'a';
	const b = 1 + a;
	E1[E1['b'] = b] = 'b';
	return E1;
})(E1 || {});
var E2 = (E2 => {
	const a =  -1;
	E2[E2['a'] = a] = 'a';
	const b = 1 + a;
	E2[E2['b'] = b] = 'b';
	return E2;
})(E2 || {});
var E3 = (E3 => {
	const a = 0.1;
	E3[E3['a'] = a] = 'a';
	const b = 1 + a;
	E3[E3['b'] = b] = 'b';
	return E3;
})(E3 || {});
var E5 = (E5 => {
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
	return E5;
})(E5 || {});
var E6 = (E6 => {
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
	return E6;
})(E6 || {});

```

# typescript/tests/cases/conformance/enums/enumErrorOnConstantBindingWithInitializer.ts
```typescript
const {value='123'} = thing;
var E = (E => {
	const test = value;
	E[E['test'] = test] = 'test';
	return E;
})(E || {});

```

# typescript/tests/cases/conformance/enums/enumErrors.ts
```error

  × Expected `,` but found `;`
    ╭─[typescript/tests/cases/conformance/enums/enumErrors.ts:48:18]
 47 │ 
 48 │     postSemicolon;
    ·                  ┬
    ·                  ╰── `,` expected
 49 │     postColonValueComma: 2,
    ╰────


```

# typescript/tests/cases/conformance/enums/enumExportMergingES6.ts
```typescript
export var Animals = (Animals => {
	const Cat = 1;
	Animals[Animals['Cat'] = Cat] = 'Cat';
	return Animals;
})(Animals || {});
var Animals = (Animals => {
	const Dog = 2;
	Animals[Animals['Dog'] = Dog] = 'Dog';
	return Animals;
})(Animals || {});
var Animals = (Animals => {
	const CatDog = Cat | Dog;
	Animals[Animals['CatDog'] = CatDog] = 'CatDog';
	return Animals;
})(Animals || {});

```

# typescript/tests/cases/conformance/enums/enumMerging.ts
```typescript
let M1;
(function(_M1) {
	var EImpl1 = (EImpl1 => {
		const A = 0;
		EImpl1[EImpl1['A'] = A] = 'A';
		const B = 1 + A;
		EImpl1[EImpl1['B'] = B] = 'B';
		const C = 1 + B;
		EImpl1[EImpl1['C'] = C] = 'C';
		return EImpl1;
	})(EImpl1 || {});
	var EImpl1 = (EImpl1 => {
		const D = 1;
		EImpl1[EImpl1['D'] = D] = 'D';
		const E = 1 + D;
		EImpl1[EImpl1['E'] = E] = 'E';
		const F = 1 + E;
		EImpl1[EImpl1['F'] = F] = 'F';
		return EImpl1;
	})(EImpl1 || {});
export 	var EConst1 = (EConst1 => {
		const A = 3;
		EConst1[EConst1['A'] = A] = 'A';
		const B = 2;
		EConst1[EConst1['B'] = B] = 'B';
		const C = 1;
		EConst1[EConst1['C'] = C] = 'C';
		return EConst1;
	})(EConst1 || {});
	var EConst1 = (EConst1 => {
		const D = 7;
		EConst1[EConst1['D'] = D] = 'D';
		const E = 9;
		EConst1[EConst1['E'] = E] = 'E';
		const F = 8;
		EConst1[EConst1['F'] = F] = 'F';
		return EConst1;
	})(EConst1 || {});
	var x = [EConst1.A, EConst1.B, EConst1.C, EConst1.D, EConst1.E, EConst1.F];
})(M1 || (M1 = {}));
let M2;
(function(_M2) {
export 	var EComp2 = (EComp2 => {
		const A = 'foo'.length;
		EComp2[EComp2['A'] = A] = 'A';
		const B = 'foo'.length;
		EComp2[EComp2['B'] = B] = 'B';
		const C = 'foo'.length;
		EComp2[EComp2['C'] = C] = 'C';
		return EComp2;
	})(EComp2 || {});
	var EComp2 = (EComp2 => {
		const D = 'foo'.length;
		EComp2[EComp2['D'] = D] = 'D';
		const E = 'foo'.length;
		EComp2[EComp2['E'] = E] = 'E';
		const F = 'foo'.length;
		EComp2[EComp2['F'] = F] = 'F';
		return EComp2;
	})(EComp2 || {});
	var x = [EComp2.A, EComp2.B, EComp2.C, EComp2.D, EComp2.E, EComp2.F];
})(M2 || (M2 = {}));
let M3;
(function(_M3) {
	var EInit = (EInit => {
		const A = 0;
		EInit[EInit['A'] = A] = 'A';
		const B = 1 + A;
		EInit[EInit['B'] = B] = 'B';
		return EInit;
	})(EInit || {});
	var EInit = (EInit => {
		const C = 1;
		EInit[EInit['C'] = C] = 'C';
		const D = 1 + C;
		EInit[EInit['D'] = D] = 'D';
		const E = 1 + D;
		EInit[EInit['E'] = E] = 'E';
		return EInit;
	})(EInit || {});
})(M3 || (M3 = {}));
let M4;
(function(_M4) {
export 	var Color = (Color => {
		const Red = 0;
		Color[Color['Red'] = Red] = 'Red';
		const Green = 1 + Red;
		Color[Color['Green'] = Green] = 'Green';
		const Blue = 1 + Green;
		Color[Color['Blue'] = Blue] = 'Blue';
		return Color;
	})(Color || {});
})(M4 || (M4 = {}));
let M5;
(function(_M5) {
	var Color = (Color => {
		const Red = 0;
		Color[Color['Red'] = Red] = 'Red';
		const Green = 1 + Red;
		Color[Color['Green'] = Green] = 'Green';
		const Blue = 1 + Green;
		Color[Color['Blue'] = Blue] = 'Blue';
		return Color;
	})(Color || {});
})(M5 || (M5 = {}));
let M6;
(function(_M6) {
	(function(_A) {
		var Color = (Color => {
			const Red = 0;
			Color[Color['Red'] = Red] = 'Red';
			const Green = 1 + Red;
			Color[Color['Green'] = Green] = 'Green';
			const Blue = 1 + Green;
			Color[Color['Blue'] = Blue] = 'Blue';
			return Color;
		})(Color || {});
	})(A || (A = {}));
})(M6 || (M6 = {}));
let M6;
(function(_M62) {
export 	let A;
	(function(_A2) {
		var Color = (Color => {
			const Yellow = 1;
			Color[Color['Yellow'] = Yellow] = 'Yellow';
			return Color;
		})(Color || {});
	})(A || (A = {}));
	var t = A.Color.Yellow;
	t = A.Color.Red;
})(M6 || (M6 = {}));

```

# typescript/tests/cases/conformance/enums/enumMergingErrors.ts
```typescript
let M;
(function(_M) {
export 	var E1 = (E1 => {
		const A = 0;
		E1[E1['A'] = A] = 'A';
		return E1;
	})(E1 || {});
export 	var E2 = (E2 => {
		const C = 0;
		E2[E2['C'] = C] = 'C';
		return E2;
	})(E2 || {});
export 	var E3 = (E3 => {
		const A = 0;
		E3[E3['A'] = A] = 'A';
		return E3;
	})(E3 || {});
})(M || (M = {}));
let M;
(function(_M2) {
	var E1 = (E1 => {
		const B = 'foo'.length;
		E1[E1['B'] = B] = 'B';
		return E1;
	})(E1 || {});
	var E2 = (E2 => {
		const B = 'foo'.length;
		E2[E2['B'] = B] = 'B';
		return E2;
	})(E2 || {});
	var E3 = (E3 => {
		const C = 0;
		E3[E3['C'] = C] = 'C';
		return E3;
	})(E3 || {});
})(M || (M = {}));
let M;
(function(_M3) {
	var E1 = (E1 => {
		const C = 0;
		E1[E1['C'] = C] = 'C';
		return E1;
	})(E1 || {});
	var E2 = (E2 => {
		const A = 0;
		E2[E2['A'] = A] = 'A';
		return E2;
	})(E2 || {});
	var E3 = (E3 => {
		const B = 'foo'.length;
		E3[E3['B'] = B] = 'B';
		return E3;
	})(E3 || {});
})(M || (M = {}));
let M1;
(function(_M1) {
	var E1 = (E1 => {
		const A = 0;
		E1[E1['A'] = A] = 'A';
		return E1;
	})(E1 || {});
})(M1 || (M1 = {}));
let M1;
(function(_M12) {
	var E1 = (E1 => {
		const B = 0;
		E1[E1['B'] = B] = 'B';
		return E1;
	})(E1 || {});
})(M1 || (M1 = {}));
let M1;
(function(_M13) {
	var E1 = (E1 => {
		const C = 0;
		E1[E1['C'] = C] = 'C';
		return E1;
	})(E1 || {});
})(M1 || (M1 = {}));
let M2;
(function(_M2) {
	var E1 = (E1 => {
		const A = 0;
		E1[E1['A'] = A] = 'A';
		return E1;
	})(E1 || {});
})(M2 || (M2 = {}));
let M2;
(function(_M22) {
	var E1 = (E1 => {
		const B = 0;
		E1[E1['B'] = B] = 'B';
		return E1;
	})(E1 || {});
})(M2 || (M2 = {}));
let M2;
(function(_M23) {
	var E1 = (E1 => {
		const C = 0;
		E1[E1['C'] = C] = 'C';
		return E1;
	})(E1 || {});
})(M2 || (M2 = {}));

```

# typescript/tests/cases/conformance/enums/enumShadowedInfinityNaN.ts
```typescript
{
	let Infinity = {};
	var En = (En => {
		const X = Infinity;
		En[En['X'] = X] = 'X';
		return En;
	})(En || {});
}
{
	let NaN = {};
	var En = (En => {
		const X = NaN;
		En[En['X'] = X] = 'X';
		return En;
	})(En || {});
}

```

# babel/packages/babel-plugin-transform-typescript/test/fixtures/enum/boolean-value/input.ts
```typescript
var E = (E => {
	const A = true;
	E[E['A'] = A] = 'A';
	return E;
})(E || {});

```

# babel/packages/babel-plugin-transform-typescript/test/fixtures/enum/const/input.ts
```typescript
var E = (E => {
	return E;
})(E || {});

```

# babel/packages/babel-plugin-transform-typescript/test/fixtures/enum/constant-folding/input.ts
```typescript
var E = (E => {
	const a = 0;
	E[E['a'] = a] = 'a';
	const b = 1 | 2;
	E[E['b'] = b] = 'b';
	const c = 1 & 3;
	E[E['c'] = c] = 'c';
	const d = 4 >> 1;
	E[E['d'] = d] = 'd';
	const e = 8 >>> 1;
	E[E['e'] = e] = 'e';
	const f = 1 << 3;
	E[E['f'] = f] = 'f';
	const g = 2 ^ 7;
	E[E['g'] = g] = 'g';
	const h = 2 * 3;
	E[E['h'] = h] = 'h';
	const i = 2 / 3;
	E[E['i'] = i] = 'i';
	const j = 2 + 5;
	E[E['j'] = j] = 'j';
	const k = 2 - 4;
	E[E['k'] = k] = 'k';
	const l = 2.5 % 2;
	E[E['l'] = l] = 'l';
	const m = 2 ** 33;
	E[E['m'] = m] = 'm';
	const n =  +9;
	E[E['n'] = n] = 'n';
	const o =  -1;
	E[E['o'] = o] = 'o';
	const p =  ~2;
	E[E['p'] = p] = 'p';
	const q = 1 + 2 - 3 * 4 /  -5;
	E[E['q'] = q] = 'q';
	const r = 1 + q;
	E[E['r'] = r] = 'r';
	return E;
})(E || {});

```

# babel/packages/babel-plugin-transform-typescript/test/fixtures/enum/enum-merging-inner-references/input.ts
```typescript
var Animals = (Animals => {
	const Cat = 1;
	Animals[Animals['Cat'] = Cat] = 'Cat';
	const Dog = 2;
	Animals[Animals['Dog'] = Dog] = 'Dog';
	return Animals;
})(Animals || {});
var Animals = (Animals => {
	const CatDog = Cat - Dog;
	Animals[Animals['CatDog'] = CatDog] = 'CatDog';
	return Animals;
})(Animals || {});

```

# babel/packages/babel-plugin-transform-typescript/test/fixtures/enum/enum-merging-inner-references-shadow/input.ts
```typescript
const Cat = 10;
const Dog = 20;
var Animals = (Animals => {
	const Cat = 1;
	Animals[Animals['Cat'] = Cat] = 'Cat';
	return Animals;
})(Animals || {});
var Animals = (Animals => {
	const Dog = 2;
	Animals[Animals['Dog'] = Dog] = 'Dog';
	return Animals;
})(Animals || {});
var Animals = (Animals => {
	const CatDog = Cat | Dog;
	Animals[Animals['CatDog'] = CatDog] = 'CatDog';
	return Animals;
})(Animals || {});

```

# babel/packages/babel-plugin-transform-typescript/test/fixtures/enum/export/input.ts
```typescript
export var E = (E => {
	const A = 1;
	E[E['A'] = A] = 'A';
	return E;
})(E || {});
var E = (E => {
	const B = 2;
	E[E['B'] = B] = 'B';
	return E;
})(E || {});

```

# babel/packages/babel-plugin-transform-typescript/test/fixtures/enum/inferred/input.ts
```typescript
var E = (E => {
	const x = 0;
	E[E['x'] = x] = 'x';
	const y = 1 + x;
	E[E['y'] = y] = 'y';
	return E;
})(E || {});

```

# babel/packages/babel-plugin-transform-typescript/test/fixtures/enum/inner-references/input.ts
```typescript
var E = (E => {
	const a = 10;
	E[E['a'] = a] = 'a';
	const b = a;
	E[E['b'] = b] = 'b';
	return E;
})(E || {});

```

# babel/packages/babel-plugin-transform-typescript/test/fixtures/enum/mix-references/input.ts
```typescript
var x = 10;
var Foo = (Foo => {
	const a = 10;
	Foo[Foo['a'] = a] = 'a';
	const b = a;
	Foo[Foo['b'] = b] = 'b';
	const c = b + x;
	Foo[Foo['c'] = c] = 'c';
	return Foo;
})(Foo || {});
var Bar = (Bar => {
	const D = Foo.a;
	Bar[Bar['D'] = D] = 'D';
	const E = D;
	Bar[Bar['E'] = E] = 'E';
	const F = Math.E;
	Bar[Bar['F'] = F] = 'F';
	const G = E + Foo.c;
	Bar[Bar['G'] = G] = 'G';
	return Bar;
})(Bar || {});
var Baz = (Baz => {
	const a = 0;
	Baz[Baz['a'] = a] = 'a';
	const b = 1;
	Baz[Baz['b'] = b] = 'b';
	const x = a.toString();
	Baz[Baz['x'] = x] = 'x';
	return Baz;
})(Baz || {});
var A = (A => {
	const a = 0;
	A[A['a'] = a] = 'a';
	const b = (() => {
		let a = 1;
		return a + 1;
	})();
	A[A['b'] = b] = 'b';
	const c = (() => {
		return a + 2;
	})();
	A[A['c'] = c] = 'c';
	return A;
})(A || {});

```

# babel/packages/babel-plugin-transform-typescript/test/fixtures/enum/non-foldable-constant/input.ts
```typescript
var E = (E => {
	const a = Math.sin(1);
	E[E['a'] = a] = 'a';
	const b = 1 + a;
	E[E['b'] = b] = 'b';
	return E;
})(E || {});

```

# babel/packages/babel-plugin-transform-typescript/test/fixtures/enum/non-scoped/input.ts
```typescript
var E = (E => {
	const x = 1;
	E[E['x'] = x] = 'x';
	const y = 2;
	E[E['y'] = y] = 'y';
	return E;
})(E || {});
var E = (E => {
	const z = 3;
	E[E['z'] = z] = 'z';
	return E;
})(E || {});

```

# babel/packages/babel-plugin-transform-typescript/test/fixtures/enum/outer-references/input.ts
```typescript
var socketType = (socketType => {
	const SOCKET = 0;
	socketType[socketType['SOCKET'] = SOCKET] = 'SOCKET';
	const SERVER = 1 + SOCKET;
	socketType[socketType['SERVER'] = SERVER] = 'SERVER';
	const IPC = 1 + SERVER;
	socketType[socketType['IPC'] = IPC] = 'IPC';
	return socketType;
})(socketType || {});
var constants = (constants => {
	const SOCKET = socketType.SOCKET;
	constants[constants['SOCKET'] = SOCKET] = 'SOCKET';
	const SERVER = socketType.SERVER;
	constants[constants['SERVER'] = SERVER] = 'SERVER';
	const IPC = socketType.IPC;
	constants[constants['IPC'] = IPC] = 'IPC';
	const UV_READABLE = 1 + IPC;
	constants[constants['UV_READABLE'] = UV_READABLE] = 'UV_READABLE';
	const UV_WRITABLE = 1 + UV_READABLE;
	constants[constants['UV_WRITABLE'] = UV_WRITABLE] = 'UV_WRITABLE';
	return constants;
})(constants || {});

```

# babel/packages/babel-plugin-transform-typescript/test/fixtures/enum/scoped/input.ts
```typescript
{
	var E = (E => {
		return E;
	})(E || {});
}

```

# babel/packages/babel-plugin-transform-typescript/test/fixtures/enum/string-value/input.ts
```typescript
var E = (E => {
	const A = 0;
	E[E['A'] = A] = 'A';
	const B = '';
	E['B'] = B;
	const A2 = A;
	E[E['A2'] = A2] = 'A2';
	const B2 = B;
	E[E['B2'] = B2] = 'B2';
	return E;
})(E || {});

```

# babel/packages/babel-plugin-transform-typescript/test/fixtures/enum/string-value-template/input.ts
```typescript
var E = (E => {
	const A = `Hey`;
	E['A'] = A;
	return E;
})(E || {});

```

# babel/packages/babel-plugin-transform-typescript/test/fixtures/enum/string-values-computed/input.ts
```typescript
var E = (E => {
	const A = 'HALLO' + 'WERLD';
	E[E['A'] = A] = 'A';
	return E;
})(E || {});

```

# babel/packages/babel-plugin-transform-typescript/test/fixtures/enum/ts5.0-const-foldable/input.ts
```typescript
const BaseValue = 10;
const Prefix = '/data';
var Values = (Values => {
	const First = BaseValue;
	Values[Values['First'] = First] = 'First';
	const Second = 1 + First;
	Values[Values['Second'] = Second] = 'Second';
	const Third = 1 + Second;
	Values[Values['Third'] = Third] = 'Third';
	return Values;
})(Values || {});
const xxx = 100 + Values.First;
const yyy = xxx;
var Routes = (Routes => {
	const Parts = `${Prefix}/parts`;
	Routes['Parts'] = Parts;
	const Invoices = `${Prefix}/invoices`;
	Routes['Invoices'] = Invoices;
	const x = `${Values.First}/x`;
	Routes['x'] = x;
	const y = `${yyy}/y`;
	Routes['y'] = y;
	return Routes;
})(Routes || {});

```

