var x = 10;
var Foo = function(Foo) {
       Foo[Foo['a'] = 10] = 'a';
       Foo[Foo['b'] = 10] = 'b';
       Foo[Foo['c'] = Foo.b + x] = 'c';
       return Foo;
}(Foo || {});
