import { REAL_SVELTE_FIXTURE_SPECS } from "../scripts/svelte-real-package-metadata.ts";
import { getFixtures } from "./utils.ts";

import type { Fixture } from "./utils.ts";

const fixturesByName = new Map(getFixtures().map((fixture) => [fixture.name, fixture]));

export function getRealSvelteFixtures(): Fixture[] {
  return REAL_SVELTE_FIXTURE_SPECS.map(({ name }) => getRealSvelteFixture(name));
}

export function getRealSvelteFixture(name: string): Fixture {
  const fixture = fixturesByName.get(name);
  if (fixture === undefined) {
    throw new Error(`Missing real-package Svelte fixture: ${name}`);
  }

  return fixture;
}
