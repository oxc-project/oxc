import { eslintCompatPlugin } from "#oxlint/plugins";

import type { Rule, Visitor } from "#oxlint/plugins";

// Record of all events, used to verify that `after` hooks run correctly
// even after an error in one rule's `before` hook.
// In ESLint compat mode, `before` is called inside `create`, so `create` throws.
// ESLint crashes on `create` errors, but pending `after` hooks are still run.
const events: string[] = [];

// Rule which records events. Runs before the throwing rule.
const trackingRule: Rule = {
  createOnce(context) {
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
        // oxlint-disable-next-line eslint/no-console
        console.error(`filename: ${context.filename}\nevents:\n${JSON.stringify(events, null, 2)}`);
      },
    } as unknown as Visitor; // TODO: Our types don't include CFG event handlers at present
  },
};

// Rule which throws in its `before` hook.
// In ESLint compat mode, this causes `create` to throw, which crashes ESLint.
// This rule's `after` hook should NOT be called (because `before` hook threw).
const throwInBeforeRule: Rule = {
  createOnce() {
    return {
      before() {
        events.push("before: throw-in-before");
        throw new Error("`before` hook threw");
      },
      Identifier() {
        events.push("Identifier: throw-in-before");
      },
      "Program:exit"() {
        events.push("Program:exit: throw-in-before");
      },
      onCodePathEnd() {
        events.push("onCodePathEnd: throw-in-before");
      },
      after() {
        // Should not be called because `before` hook threw,
        // so `setupAfterHook` was never called
        events.push("after: throw-in-before");
        // oxlint-disable-next-line eslint/no-console
        console.error("`after` hook in `throw-in-before` rule should not be called");
      },
    } as unknown as Visitor; // TODO: Our types don't include CFG event handlers at present
  },
};

// Rule which records events. Listed after the throwing rule in the plugin.
// ESLint crashes on throw-in-before's `create`, so this rule's `create` is never called.
// Outputs full event log via `console.error` in `after` hook.
const trackingLateRule: Rule = {
  createOnce() {
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
        console.error("`after` hook in `tracking-late` rule should not be called");
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
    "throw-in-before": throwInBeforeRule,
    "tracking-late": trackingLateRule,
  },
});
