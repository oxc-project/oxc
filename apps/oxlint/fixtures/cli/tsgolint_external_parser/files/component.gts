const label: number = 42;

async function save(): Promise<void> {}

// Floating promise at module level: a tsgolint *rule* diagnostic.
save();

<template>
  {{! Type error inside the template: `toUpperCase` does not exist on `number`.
      Reported only under `--type-check`, and only once the content mapper has
      mapped this region back to its offset in the original `.gts`. }}
  <button {{on "click" save}}>{{label.toUpperCase}}</button>
</template>
