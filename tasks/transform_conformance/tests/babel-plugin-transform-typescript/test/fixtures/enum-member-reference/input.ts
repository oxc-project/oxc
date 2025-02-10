var x = 10;

enum Foo {
    a = 10,
    b = a,
    c = b + x,
}

enum Merge { x = Math.random() }
enum Merge { y = x }

enum NestOuter {
    a,
    b = (() => {
        enum NestInner {
            a = Math.random(),
            b = a
        }
        return NestInner.b;
    })()
}
