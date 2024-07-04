const readonly = 1;
export const override = 2;
const accessor = 'foo'

function foo(accessor, readonly, y: readonly number[]) {}

class Foo {
    public accessor accessor;
    public override: number;
    public readonly readonly: number = 1;
}
class Bar extends Foo {
    constructor(public override override: number, override readonly readonly: number) {
        super()
    }
}

const x = { readonly, override };
const y = { readonly: readonly, override: override };
export { readonly };
export default readonly;
