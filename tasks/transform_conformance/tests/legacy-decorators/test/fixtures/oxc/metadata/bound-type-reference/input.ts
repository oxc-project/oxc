import { BoundTypeReference } from "./output";

console.log(BoundTypeReference)

class Example {
	constructor(@dce count: BoundTypeReference) {}
  prop: BoundTypeReference.Value = 1;
}
