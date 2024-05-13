type Foo = {
    a: number
    b: string
}

interface Bar {
    a: number
    b: string
}

class Baz {
    a: number
    b: string

}

const x: Foo['a'] = 1
const y: Bar['a'] = 1
const z: Baz['a'] = 1

