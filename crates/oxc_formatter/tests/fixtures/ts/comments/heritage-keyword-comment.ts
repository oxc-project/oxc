// A comment after `extends` stays after it, past type parameters too.
class WithTypeParameters<T> extends
// c
B {}
class BeforeKeyword<T>
  // a
  extends B {}
class Implements implements
// c
B {}
class ImplementsBlock implements
/* c */
B {}
class ExtendsImplements extends X implements
// c
B {}
const AnonymousImplements = class implements
// c
B {};
interface Extends extends
// c
B {}
interface ExtendsMany<T> extends
// c
B, C {}
interface BeforeExtends // a
  extends B {}
