var x = 10;
var Foo = function(Foo) {
  Foo[Foo['a'] = 10] = 'a';
  Foo[Foo['b'] = 10] = 'b';
  Foo[Foo['c'] = Foo.b + x] = 'c';
  return Foo;
}(Foo || {});
var Merge = function(Merge) {
  Merge[Merge['x'] = Math.random()] = 'x';
  return Merge;
}(Merge || {});
Merge = function(Merge) {
  Merge[Merge['y'] = Merge.x] = 'y';
  return Merge;
}(Merge || {});
var NestOuter = function(NestOuter) {
  NestOuter[NestOuter['a'] = 0] = 'a';
  NestOuter[NestOuter['b'] = (() => {
    let NestInner = function(NestInner) {
      NestInner[NestInner['a'] = Math.random()] = 'a';
      NestInner[NestInner['b'] = NestInner.a] = 'b';
      return NestInner;
    }({});
    return NestInner.b;
  })()] = 'b';
  return NestOuter;
}(NestOuter || {});