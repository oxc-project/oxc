// `rules-of-hooks` must be skipped for loader-derived Glimmer sources: `useX`
// helpers are common in Ember and would false-positive. `useThing` calls a hook
// conditionally, which the rule would otherwise flag. The `debugger` statement
// proves other native rules still run.
import { useState } from "ember-hook";

export function useThing(cond) {
  if (cond) {
    useState();
  }
}

debugger;

export default <template>{{useThing}}</template>;
