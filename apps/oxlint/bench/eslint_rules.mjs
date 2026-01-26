import { Bench } from "tinybench";
import { withCodSpeed } from "@codspeed/tinybench-plugin";
import {
  parse,
  lintFileImpl,
  registerPlugin,
  registeredRules,
  diagnostics,
  setOptions,
  DEFAULT_OPTIONS_ID,
} from "#oxlint";

// @ts-expect-error - ESLint internal API
import { builtinRules } from "../conformance/submodules/eslint/lib/unsupported-api.js";

// Source: https://cdn.jsdelivr.net/gh/oxc-project/benchmark-files@main/RadixUIAdoptionSection.jsx
const SOURCE_CODE = `import React from 'react';
import NextLink from 'next/link';
import { Box, Button, Container, Grid, Heading, Section, Text } from '@radix-ui/themes';
import { MarketingCaption } from './MarketingCaption';

export const AdoptionSection = () => {
  return (
    <Section size={{ initial: '2', md: '3' }}>
      <Container mx={{ initial: '5', xs: '6', sm: '7', md: '9' }}>
        <Box mb="8">
          <MarketingCaption mb="1">Transition to Radix Primitives</MarketingCaption>
          <Heading as="h2" size="7" mb="5">
            Adoption made easy
          </Heading>
          <NextLink href="/primitives/docs" passHref legacyBehavior>
            <Button asChild size={{ initial: '3', xs: '4' }} color="gray" highContrast>
              <a>
                Go to docs
                <svg
                  width="14"
                  height="14"
                  viewBox="0 0 12 12"
                  xmlns="http://www.w3.org/2000/svg"
                  fill="currentcolor"
                  style={{ opacity: 1, marginRight: -3 }}
                >
                  <path d="M6.39205 11.6023L5.36932 10.5909L8.92045 7.03977H0V5.5625H8.92045L5.36932 2.01705L6.39205 1L11.6932 6.30114L6.39205 11.6023Z" />
                </svg>
              </a>
            </Button>
          </NextLink>
        </Box>

        <Grid columns={{ initial: '1', sm: '2' }} gap={{ initial: '7', sm: '5' }}>
          <Box>
            <Heading as="h3" size="4" mb="2">
              Incremental adoption
            </Heading>
            <Text as="p" size="3" mr={{ sm: '5', md: '7', lg: '9' }}>
              Each component is its own independently versioned package, so new components can be
              added alongside your existing code. No need to disrupt feature work with a huge
              rewrite{'\\u2060'}—you can start small and add more components one by one.
            </Text>
          </Box>

          <Box>
            <Heading as="h3" size="4" mb="2">
              Detailed docs and TypeScript support
            </Heading>
            <Text as="p" size="3" mr={{ sm: '5', md: '7', lg: '9' }}>
              Radix documentation contains real-world examples, extensive API references,
              accessibility details, and full TypeScript support. All components share a similar
              API, creating a consistent developer experience. You will love working with Radix
              Primitives.
            </Text>
          </Box>
        </Grid>
      </Container>
    </Section>
  );
};`;

const FILENAME = "test.jsx";

// Constants for lintFileImpl parameters
const BUFFER_ID = 0; // Pre-parsed buffer is stored at index 0
const RULE_ID = 0; // Single rule at index 0
const EMPTY_SETTINGS = "{}";
const EMPTY_GLOBALS = "{}";

// Collect all rules upfront
const rules = Array.from(builtinRules.entries());
console.log(`Benchmarking ${rules.length} ESLint rules...`);

// Parse the source code once before benchmarking
console.log("Parsing source code...");
parse(FILENAME, SOURCE_CODE, {
  lang: "jsx",
  allowReturnOutsideFunction: true,
});
console.log("Source code parsed successfully.");

// Set up default options for all rules
// Format: { options: [defaultOptions, ...], ruleIds: [0, ...] }
// defaultOptions is an empty array [] for rules with no options
setOptions(JSON.stringify({ options: [[]], ruleIds: [0] }));

const bench = withCodSpeed(new Bench());

// Single benchmark that runs all rules using the pre-parsed buffer
bench.add("all-eslint-rules", () => {
  for (const [ruleName, rule] of rules) {
    try {
      // Create a plugin for this rule
      const plugin = {
        meta: { name: "eslint" },
        rules: { [ruleName]: rule },
      };

      // Register the plugin (null = no plugin name override, false = not a builtin plugin)
      registerPlugin(plugin, null, false);

      // Lint using the pre-parsed buffer
      lintFileImpl(
        FILENAME,
        BUFFER_ID,
        null, // Buffer already stored, no need to pass it again
        [RULE_ID],
        [DEFAULT_OPTIONS_ID],
        EMPTY_SETTINGS,
        EMPTY_GLOBALS,
      );
    } catch {
      // Some rules may fail on this code, but we're benchmarking execution time
    } finally {
      // Reset state for next rule
      registeredRules.length = 0;
      diagnostics.length = 0;
    }
  }
});

console.log("Running benchmarks...");
await bench.run();
console.table(bench.table());
