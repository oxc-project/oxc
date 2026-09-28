new Foo<T>(// c
a);

f<T>(// c
a);

f<T>(// c
);

a.b<T>(// c
x);

a.b<T>(x).c<U>(// c
y);

f<T>(/* c */
a);

new Foo<T>(/* c */
a);

f<T>(/**
 * doc
 */
a);

// Before `(` or `<`: stays on the callee side
foo<T> // c
(a);
foo // c
<T>(a);
foo
// c
<T>(a);
foo /* c */ <T>(a);
new Foo // c
<T>(a);
a.b // c
<T>(x);
call // C4
?.();
foo // c
?.(a);
foo
/* c */
?.(a);
foo /* c */ ?.(a);
foo // c
?.<T>(a);
a.b // c
?.(x).c();
