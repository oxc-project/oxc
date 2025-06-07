import { dce, Dependency } from "mod";

@dce()
export abstract class AbstractClass {
	constructor(public dependency: Dependency) {}
}
