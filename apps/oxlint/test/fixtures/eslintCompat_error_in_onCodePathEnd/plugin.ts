import { eslintCompatPlugin } from "#oxlint/plugins";

import type { Rule, Visitor } from "#oxlint/plugins";

// Record of all events, used to verify that `after` hooks run in correct order
// even after an error during AST visitation
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

// Rule which throws during AST visitation.
// Its `after` hook should still be called.
const throwInVisitRule: Rule = {
  createOnce() {
    return {
      before() {
        events.push("before: throw-in-visit");
      },
      Identifier() {
        events.push("Identifier: throw-in-visit");
      },
      "Program:exit"() {
        events.push("Program:exit: throw-in-visit");
      },
      onCodePathEnd() {
        events.push("onCodePathEnd: throw-in-visit");
        throw new Error("`onCodePathEnd` CFG event handler threw");
      },
      after() {
        events.push("after: throw-in-visit");
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
    "throw-in-visit": throwInVisitRule,
    "tracking-late": trackingLateRule,
  },
});
