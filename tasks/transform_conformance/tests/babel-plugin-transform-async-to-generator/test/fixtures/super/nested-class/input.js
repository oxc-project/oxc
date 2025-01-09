class Root {}
class Outer extends Root {
  value = 0
  async method() {
    () => super.value;

    class Inner extends Outer {
      normal() {
        console.log(super.value);
      }

      async method() {
        () => super.value;
      }
    }

    () => super.value;
  }
}
