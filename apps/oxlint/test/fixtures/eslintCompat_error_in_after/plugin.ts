import { eslintCompatPlugin } from "#oxlint/plugins";

import type { Rule, Visitor } from "#oxlint/plugins";

// Record of all events, used to verify that other rules' `after` hooks run
// even after an error in one rule's `after` hook
const events: string[] = [];

// Rule which records events. Runs before the throwing rule.
const trackingRule: Rule = {
  createOnce() {
    return {
      before() {
        events.push("before: tracking");
      },
      Identifier() {
        events.push("Identifier: tracking");
      },
      "Program:exit"() {
        events.push("Program:exit: tracking");
      },
      onCodePathEnd() {
        events.push("onCodePathEnd: tracking");
      },
      after() {
        events.push("after: tracking");
      },
    } as unknown as Visitor; // TODO: Our types don't include CFG event handlers at present
  },
};

// Rule which throws in its `after` hook.
// Other rules' `after` hooks should still run.
const throwInAfterRule: Rule = {
  createOnce() {
    return {
      before() {
        events.push("before: throw-in-after");
      },
      Identifier() {
        events.push("Identifier: throw-in-after");
      },
      "Program:exit"() {
        events.push("Program:exit: throw-in-after");
      },
      onCodePathEnd() {
        events.push("onCodePathEnd: throw-in-after");
      },
      after() {
        events.push("after: throw-in-after");
        throw new Error("`after` hook threw");
      },
    } as unknown as Visitor; // TODO: Our types don't include CFG event handlers at present
  },
};

// Rule which records events. Runs after the throwing rule.
// Outputs full event log via `console.error` in `after` hook.
const trackingLateRule: Rule = {
  createOnce(context) {
    return {
      before() {
        events.push("before: tracking-late");
      },
      Identifier() {
        events.push("Identifier: tracking-late");
      },
      "Program:exit"() {
        events.push("Program:exit: tracking-late");
      },
      onCodePathEnd() {
        events.push("onCodePathEnd: tracking-late");
      },
      after() {
        events.push("after: tracking-late");
        // oxlint-disable-next-line eslint/no-console
        console.error(`filename: ${context.filename}\nevents:\n${JSON.stringify(events, null, 2)}`);
      },
    } as unknown as Visitor; // TODO: Our types don't include CFG event handlers at present
  },
};

export default eslintCompatPlugin({
  meta: {
    name: "eslint-compat-plugin",
  },
  rules: {
    tracking: trackingRule,
    "throw-in-after": throwInAfterRule,
    "tracking-late": trackingLateRule,
  },
});
