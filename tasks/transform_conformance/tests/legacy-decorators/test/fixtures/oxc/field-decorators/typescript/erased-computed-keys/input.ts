import { key } from "./key";

abstract class A {
  @dec declare [k()]: number;
  @dec abstract [j()]: number;
  // Used as a value now, so the import stays.
  @dec declare [key]: string;
}
