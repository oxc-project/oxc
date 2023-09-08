const importPath = './exports-for-dynamic-js';
class A {
    method() {
        const c = import(importPath)
    }
}


class B {
    method() {
        const c = import('i-do-not-exist')
    }
}
