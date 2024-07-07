export function foo(): void {}
foo.apply = () => {}

export const bar = (): void => {}
bar.call = ()=> {}


export namespace NS {
  export const goo = (): void => {}
  goo.length = 10
}

// unexported
const zoo = (): void => {}
zoo.toString = ()=> {}