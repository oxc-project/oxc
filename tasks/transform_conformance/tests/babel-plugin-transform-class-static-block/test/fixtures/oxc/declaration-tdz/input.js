const read = () => A;
class A {
  static { try { read(); console.log('initialized'); } catch (e) { console.log(e.name); } }
}
