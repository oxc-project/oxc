// see issue #36

// Foo.jsx
class Foo {
    // ES7 static members
    static bar = true;
}

export default Foo

export class Bar {
    static baz = false;

    render() {
        let {a, ...rest } = {a: 1, b: 2, c: 3}
    }
}
