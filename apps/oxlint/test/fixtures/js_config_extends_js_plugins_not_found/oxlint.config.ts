import { defineConfig } from "#oxlint";

// @ts-expect-error - the fixture package has no type declarations
import shared from "shared-config";

export default defineConfig({
  extends: [shared],
});
