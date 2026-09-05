const styleClass = "primary";
const unusedLocal = "never used";

export default class NativeRules {
  count = 0;

  isZero(value: number): boolean {
    debugger;
    return value == 0;
  }

  <template>
    <span class={{styleClass}}>{{this.count}}</span>
  </template>
}
