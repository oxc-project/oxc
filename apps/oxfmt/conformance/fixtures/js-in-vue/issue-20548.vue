<script lang="ts">
  // Arrow function chain: SoftOrSpace -> StartGroup -> Space
  const fn = (a: string) => (b: number) => { return b }

  // Interface extends member expression: SoftOrSpace -> StartGroup -> Space
  interface Foo extends Bar.Baz { x: number; y: string; z: boolean; aaa: number; bbb: string; }

  // Class extends member expression: SoftOrSpace -> StartGroup -> Space
  class Foo extends Bar.Baz { x: number = 1; y: string = "hello"; z: boolean = true; aaa: number = 2; bbb: string = "world"; }
</script>
