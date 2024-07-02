// https://github.com/typescript-eslint/typescript-eslint/issues/2942

declare function deco(...param: any): (...param: any) => any;

@deco({
  components: {
    val: true,
  },
})
class Foo {}
