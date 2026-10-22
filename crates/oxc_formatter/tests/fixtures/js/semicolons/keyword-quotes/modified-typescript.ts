// DIVERGENCES.md#modified-keyword-class-field-boundary
abstract class AbstractProperties {
  x = 1;
  abstract in: number;
  y = 1;
  abstract instanceof: number;
}
abstract class AbstractMethods {
  x = 1;
  abstract in(): void;
  y = 1;
  abstract instanceof(): void;
}
class DecoratedMembers {
  x = 1;
  @dec in;
  y = 1;
  @dec instanceof() {}
}
abstract class OrdinaryMembers {
  x = 1;
  abstract property: number;
  y = 1;
  abstract method(): void;
}
class OrdinaryDecoratedMembers {
  x = 1;
  @dec property;
  y = 1;
  @dec method() {}
}
