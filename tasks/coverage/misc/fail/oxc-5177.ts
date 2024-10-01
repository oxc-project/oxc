// https://github.com/oxc-project/oxc/issues/5177
type Foo = 'foo' | 'bar'
export class Bang {
    accessor x?: Foo
}
