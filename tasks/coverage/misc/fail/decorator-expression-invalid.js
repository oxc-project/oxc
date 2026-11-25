// Arguments after a parenthesized decorator expression
class A1 { @(a)(b) m() {} }
class A2 { @((c) => c)(1) m() {} }
@((c) => c)(A3) class A3 {}
@(a)(b) class A4 {}

// Member access or templates after a parenthesized decorator expression
class B1 { @(a).b m() {} }
class B2 { @(a).#b m() {} #b; }
class B3 { @(a)`` m() {} }
@(a).b class B4 {}

// Member access, calls or templates after a call
class C1 { @a()() m() {} }
class C2 { @a().b m() {} }
class C3 { @a.b.c(1)(2) m() {} }
class C4 { @a().b() m() {} }
class C5 { @a()`` m() {} }
@a().b class C6 {}
@a()() class C7 {}

// Heads other than an `IdentifierReference`
class D1 extends B { @super.x m() {} }
class D2 extends B { @super.x() m() {} }
class D3 { @this.x m() {} }
class D4 { @new a m() {} }
class D5 { @new a() m() {} }
class D6 { @a`` m() {} }
class D7 { @a.b`` m() {} }
class D8 { @import.meta.dec m() {} }
@this.x class D9 {}
