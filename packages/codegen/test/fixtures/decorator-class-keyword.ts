@dec class A {}
@dec
class B {}
@dec /* class */
class C {}
@dec class /* class */ D {}
@dec class // class
E {}
@dec("//") class F {}
x = @dec class<T> {};
x = @dec class {};
x = @dec class extends Base {};
@dec abstract class G {}
