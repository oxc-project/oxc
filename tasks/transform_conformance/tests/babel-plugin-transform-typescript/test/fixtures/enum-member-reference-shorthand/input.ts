enum Foo {
    a = Math.random(),
    b = ({ a }).a,
    __proto__ = Math.random(),
    c = ({ __proto__ }).__proto__,
}
