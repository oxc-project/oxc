import { singleton, log, deco, C } from "dec";

@log("Problem")
@singleton()
export class Problem extends C {
  @deco()
  run() {
    return super.run();
  }
}
