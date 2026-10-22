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
