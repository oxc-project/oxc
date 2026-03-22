# relative-time/relative-time.test.ts

> html-in-js: Need to solve `label({ embed, hug }))` + `shouldExpandLastArg`

## Option 1

`````json
{"printWidth":80}
`````

### Diff

`````diff
===================================================================
--- prettier
+++ oxfmt
@@ -119,16 +119,15 @@
 
           it(`shows the correct relative time given a String object: ${testCase.expectedOutput}`, async () => {
             const dateString = testCase.date.toISOString();
 
-            const relativeTime: WaRelativeTime = await fixture<WaRelativeTime>(
-              html`
+            const relativeTime: WaRelativeTime =
+              await fixture<WaRelativeTime>(html`
                 <wa-relative-time
                   lang="en-US"
                   date="${dateString}"
                 ></wa-relative-time>
-              `,
-            );
+              `);
 
             await expectFormattedRelativeTimeToBe(
               relativeTime,
               testCase.expectedOutput,
@@ -136,27 +135,25 @@
           });
         });
 
         it("always shows numeric if requested via numeric property", async () => {
-          const relativeTime: WaRelativeTime = await fixture<WaRelativeTime>(
-            html`
+          const relativeTime: WaRelativeTime =
+            await fixture<WaRelativeTime>(html`
               <wa-relative-time
                 lang="en-US"
                 numeric="always"
               ></wa-relative-time>
-            `,
-          );
+            `);
           relativeTime.date = yesterday;
 
           await expectFormattedRelativeTimeToBe(relativeTime, "1 day ago");
         });
 
         it("shows human readable form if appropriate and numeric property is auto", async () => {
-          const relativeTime: WaRelativeTime = await fixture<WaRelativeTime>(
-            html`
+          const relativeTime: WaRelativeTime =
+            await fixture<WaRelativeTime>(html`
               <wa-relative-time lang="en-US" numeric="auto"></wa-relative-time>
-            `,
-          );
+            `);
           relativeTime.date = yesterday;
 
           await expectFormattedRelativeTimeToBe(relativeTime, "yesterday");
         });
@@ -175,17 +172,16 @@
         it("allows to use a short form of the unit", async () => {
           const twoYearsAgo = new Date(
             currentTime.getTime() - 2 * nonLeapYearInSeconds,
           );
-          const relativeTime: WaRelativeTime = await fixture<WaRelativeTime>(
-            html`
+          const relativeTime: WaRelativeTime =
+            await fixture<WaRelativeTime>(html`
               <wa-relative-time
                 lang="en-US"
                 numeric="always"
                 format="short"
               ></wa-relative-time>
-            `,
-          );
+            `);
           relativeTime.date = twoYearsAgo;
 
           await expectFormattedRelativeTimeToBe(relativeTime, "2 yr. ago");
         });
@@ -193,28 +189,26 @@
         it("allows to use a long form of the unit", async () => {
           const twoYearsAgo = new Date(
             currentTime.getTime() - 2 * nonLeapYearInSeconds,
           );
-          const relativeTime: WaRelativeTime = await fixture<WaRelativeTime>(
-            html`
+          const relativeTime: WaRelativeTime =
+            await fixture<WaRelativeTime>(html`
               <wa-relative-time
                 lang="en-US"
                 numeric="always"
                 format="long"
               ></wa-relative-time>
-            `,
-          );
+            `);
           relativeTime.date = twoYearsAgo;
 
           await expectFormattedRelativeTimeToBe(relativeTime, "2 years ago");
         });
 
         it("is formatted according to the requested locale", async () => {
-          const relativeTime: WaRelativeTime = await fixture<WaRelativeTime>(
-            html`
+          const relativeTime: WaRelativeTime =
+            await fixture<WaRelativeTime>(html`
               <wa-relative-time lang="de-DE" numeric="auto"></wa-relative-time>
-            `,
-          );
+            `);
           relativeTime.date = yesterday;
 
           await expectFormattedRelativeTimeToBe(relativeTime, "gestern");
         });

`````

### Actual (oxfmt)

`````ts
import { expect } from "@open-wc/testing";
import { html } from "lit";
import sinon from "sinon";
import type { hydratedFixture } from "../../internal/test/fixture.js";
import { clientFixture } from "../../internal/test/fixture.js";
import type WaRelativeTime from "./relative-time.js";

interface WaRelativeTimeTestCase {
  date: Date;
  expectedOutput: string;
}

const extractTimeElement = (
  relativeTime: WaRelativeTime,
): HTMLTimeElement | null => {
  return relativeTime.shadowRoot?.querySelector("time") || null;
};

const expectFormattedRelativeTimeToBe = async (
  relativeTime: WaRelativeTime,
  expectedOutput: string,
): Promise<void> => {
  await relativeTime.updateComplete;
  const textContent = extractTimeElement(relativeTime)?.textContent;
  expect(textContent).to.equal(expectedOutput);
};

const createRelativeTimeWithDate = async (
  relativeDate: Date,
  fixture: typeof hydratedFixture | typeof clientFixture,
): Promise<WaRelativeTime> => {
  const relativeTime: WaRelativeTime = await fixture<WaRelativeTime>(html`
    <wa-relative-time lang="en-US"></wa-relative-time>
  `);
  relativeTime.date = relativeDate;
  return relativeTime;
};

const minuteInSeconds = 60_000;
const hourInSeconds = minuteInSeconds * 60;
const dayInSeconds = hourInSeconds * 24;
const weekInSeconds = dayInSeconds * 7;
const monthInSeconds = dayInSeconds * 30;
const nonLeapYearInSeconds = dayInSeconds * 356;

const currentTime = new Date("2022-10-30T15:22:10.100Z");
const yesterday = new Date(currentTime.getTime() - dayInSeconds);
const testCases: WaRelativeTimeTestCase[] = [
  {
    date: new Date(currentTime.getTime() - minuteInSeconds),
    expectedOutput: "1 minute ago",
  },
  {
    date: new Date(currentTime.getTime() - hourInSeconds),
    expectedOutput: "1 hour ago",
  },
  {
    date: yesterday,
    expectedOutput: "yesterday",
  },
  {
    date: new Date(currentTime.getTime() - 4 * dayInSeconds),
    expectedOutput: "4 days ago",
  },
  {
    date: new Date(currentTime.getTime() - weekInSeconds),
    expectedOutput: "last week",
  },
  {
    date: new Date(currentTime.getTime() - monthInSeconds),
    expectedOutput: "last month",
  },
  {
    date: new Date(currentTime.getTime() - nonLeapYearInSeconds),
    expectedOutput: "last year",
  },
  {
    date: new Date(currentTime.getTime() + minuteInSeconds),
    expectedOutput: "in 1 minute",
  },
];

describe("wa-relative-time", () => {
  // @TODO: figure out why hydratedFixture behaves differently from clientFixture
  for (const fixture of [clientFixture]) {
    describe(`with "${fixture.type}" rendering`, () => {
      it("should pass accessibility tests", async () => {
        const relativeTime = await createRelativeTimeWithDate(
          currentTime,
          fixture,
        );

        await expect(relativeTime).to.be.accessible();
      });

      describe("handles time correctly", () => {
        let clock: sinon.SinonFakeTimers | null = null;

        beforeEach(() => {
          clock = sinon.useFakeTimers(currentTime);
        });

        afterEach(() => {
          clock?.restore();
        });

        testCases.forEach((testCase) => {
          it(`shows the correct relative time given a Date object: ${testCase.expectedOutput}`, async () => {
            const relativeTime = await createRelativeTimeWithDate(
              testCase.date,
              fixture,
            );

            await expectFormattedRelativeTimeToBe(
              relativeTime,
              testCase.expectedOutput,
            );
          });

          it(`shows the correct relative time given a String object: ${testCase.expectedOutput}`, async () => {
            const dateString = testCase.date.toISOString();

            const relativeTime: WaRelativeTime =
              await fixture<WaRelativeTime>(html`
                <wa-relative-time
                  lang="en-US"
                  date="${dateString}"
                ></wa-relative-time>
              `);

            await expectFormattedRelativeTimeToBe(
              relativeTime,
              testCase.expectedOutput,
            );
          });
        });

        it("always shows numeric if requested via numeric property", async () => {
          const relativeTime: WaRelativeTime =
            await fixture<WaRelativeTime>(html`
              <wa-relative-time
                lang="en-US"
                numeric="always"
              ></wa-relative-time>
            `);
          relativeTime.date = yesterday;

          await expectFormattedRelativeTimeToBe(relativeTime, "1 day ago");
        });

        it("shows human readable form if appropriate and numeric property is auto", async () => {
          const relativeTime: WaRelativeTime =
            await fixture<WaRelativeTime>(html`
              <wa-relative-time lang="en-US" numeric="auto"></wa-relative-time>
            `);
          relativeTime.date = yesterday;

          await expectFormattedRelativeTimeToBe(relativeTime, "yesterday");
        });

        it("shows the set date with the proper attributes at the time object", async () => {
          const relativeTime = await createRelativeTimeWithDate(
            yesterday,
            fixture,
          );

          await relativeTime.updateComplete;
          const timeElement = extractTimeElement(relativeTime);
          expect(timeElement?.dateTime).to.equal(yesterday.toISOString());
        });

        it("allows to use a short form of the unit", async () => {
          const twoYearsAgo = new Date(
            currentTime.getTime() - 2 * nonLeapYearInSeconds,
          );
          const relativeTime: WaRelativeTime =
            await fixture<WaRelativeTime>(html`
              <wa-relative-time
                lang="en-US"
                numeric="always"
                format="short"
              ></wa-relative-time>
            `);
          relativeTime.date = twoYearsAgo;

          await expectFormattedRelativeTimeToBe(relativeTime, "2 yr. ago");
        });

        it("allows to use a long form of the unit", async () => {
          const twoYearsAgo = new Date(
            currentTime.getTime() - 2 * nonLeapYearInSeconds,
          );
          const relativeTime: WaRelativeTime =
            await fixture<WaRelativeTime>(html`
              <wa-relative-time
                lang="en-US"
                numeric="always"
                format="long"
              ></wa-relative-time>
            `);
          relativeTime.date = twoYearsAgo;

          await expectFormattedRelativeTimeToBe(relativeTime, "2 years ago");
        });

        it("is formatted according to the requested locale", async () => {
          const relativeTime: WaRelativeTime =
            await fixture<WaRelativeTime>(html`
              <wa-relative-time lang="de-DE" numeric="auto"></wa-relative-time>
            `);
          relativeTime.date = yesterday;

          await expectFormattedRelativeTimeToBe(relativeTime, "gestern");
        });

        it("keeps the component in sync if requested", async () => {
          const relativeTime = await createRelativeTimeWithDate(
            yesterday,
            fixture,
          );
          relativeTime.sync = true;

          await expectFormattedRelativeTimeToBe(relativeTime, "yesterday");

          clock?.tick(dayInSeconds);

          await expectFormattedRelativeTimeToBe(relativeTime, "2 days ago");
        });
      });

      it("does not display a time element on invalid time string", async () => {
        const invalidDateString = "thisIsNotATimeString";

        const relativeTime: WaRelativeTime = await fixture<WaRelativeTime>(html`
          <wa-relative-time
            lang="en-US"
            date="${invalidDateString}"
          ></wa-relative-time>
        `);

        await relativeTime.updateComplete;
        expect(extractTimeElement(relativeTime)).to.be.null;
      });
    });
  }
});

`````

### Expected (prettier)

`````ts
import { expect } from "@open-wc/testing";
import { html } from "lit";
import sinon from "sinon";
import type { hydratedFixture } from "../../internal/test/fixture.js";
import { clientFixture } from "../../internal/test/fixture.js";
import type WaRelativeTime from "./relative-time.js";

interface WaRelativeTimeTestCase {
  date: Date;
  expectedOutput: string;
}

const extractTimeElement = (
  relativeTime: WaRelativeTime,
): HTMLTimeElement | null => {
  return relativeTime.shadowRoot?.querySelector("time") || null;
};

const expectFormattedRelativeTimeToBe = async (
  relativeTime: WaRelativeTime,
  expectedOutput: string,
): Promise<void> => {
  await relativeTime.updateComplete;
  const textContent = extractTimeElement(relativeTime)?.textContent;
  expect(textContent).to.equal(expectedOutput);
};

const createRelativeTimeWithDate = async (
  relativeDate: Date,
  fixture: typeof hydratedFixture | typeof clientFixture,
): Promise<WaRelativeTime> => {
  const relativeTime: WaRelativeTime = await fixture<WaRelativeTime>(html`
    <wa-relative-time lang="en-US"></wa-relative-time>
  `);
  relativeTime.date = relativeDate;
  return relativeTime;
};

const minuteInSeconds = 60_000;
const hourInSeconds = minuteInSeconds * 60;
const dayInSeconds = hourInSeconds * 24;
const weekInSeconds = dayInSeconds * 7;
const monthInSeconds = dayInSeconds * 30;
const nonLeapYearInSeconds = dayInSeconds * 356;

const currentTime = new Date("2022-10-30T15:22:10.100Z");
const yesterday = new Date(currentTime.getTime() - dayInSeconds);
const testCases: WaRelativeTimeTestCase[] = [
  {
    date: new Date(currentTime.getTime() - minuteInSeconds),
    expectedOutput: "1 minute ago",
  },
  {
    date: new Date(currentTime.getTime() - hourInSeconds),
    expectedOutput: "1 hour ago",
  },
  {
    date: yesterday,
    expectedOutput: "yesterday",
  },
  {
    date: new Date(currentTime.getTime() - 4 * dayInSeconds),
    expectedOutput: "4 days ago",
  },
  {
    date: new Date(currentTime.getTime() - weekInSeconds),
    expectedOutput: "last week",
  },
  {
    date: new Date(currentTime.getTime() - monthInSeconds),
    expectedOutput: "last month",
  },
  {
    date: new Date(currentTime.getTime() - nonLeapYearInSeconds),
    expectedOutput: "last year",
  },
  {
    date: new Date(currentTime.getTime() + minuteInSeconds),
    expectedOutput: "in 1 minute",
  },
];

describe("wa-relative-time", () => {
  // @TODO: figure out why hydratedFixture behaves differently from clientFixture
  for (const fixture of [clientFixture]) {
    describe(`with "${fixture.type}" rendering`, () => {
      it("should pass accessibility tests", async () => {
        const relativeTime = await createRelativeTimeWithDate(
          currentTime,
          fixture,
        );

        await expect(relativeTime).to.be.accessible();
      });

      describe("handles time correctly", () => {
        let clock: sinon.SinonFakeTimers | null = null;

        beforeEach(() => {
          clock = sinon.useFakeTimers(currentTime);
        });

        afterEach(() => {
          clock?.restore();
        });

        testCases.forEach((testCase) => {
          it(`shows the correct relative time given a Date object: ${testCase.expectedOutput}`, async () => {
            const relativeTime = await createRelativeTimeWithDate(
              testCase.date,
              fixture,
            );

            await expectFormattedRelativeTimeToBe(
              relativeTime,
              testCase.expectedOutput,
            );
          });

          it(`shows the correct relative time given a String object: ${testCase.expectedOutput}`, async () => {
            const dateString = testCase.date.toISOString();

            const relativeTime: WaRelativeTime = await fixture<WaRelativeTime>(
              html`
                <wa-relative-time
                  lang="en-US"
                  date="${dateString}"
                ></wa-relative-time>
              `,
            );

            await expectFormattedRelativeTimeToBe(
              relativeTime,
              testCase.expectedOutput,
            );
          });
        });

        it("always shows numeric if requested via numeric property", async () => {
          const relativeTime: WaRelativeTime = await fixture<WaRelativeTime>(
            html`
              <wa-relative-time
                lang="en-US"
                numeric="always"
              ></wa-relative-time>
            `,
          );
          relativeTime.date = yesterday;

          await expectFormattedRelativeTimeToBe(relativeTime, "1 day ago");
        });

        it("shows human readable form if appropriate and numeric property is auto", async () => {
          const relativeTime: WaRelativeTime = await fixture<WaRelativeTime>(
            html`
              <wa-relative-time lang="en-US" numeric="auto"></wa-relative-time>
            `,
          );
          relativeTime.date = yesterday;

          await expectFormattedRelativeTimeToBe(relativeTime, "yesterday");
        });

        it("shows the set date with the proper attributes at the time object", async () => {
          const relativeTime = await createRelativeTimeWithDate(
            yesterday,
            fixture,
          );

          await relativeTime.updateComplete;
          const timeElement = extractTimeElement(relativeTime);
          expect(timeElement?.dateTime).to.equal(yesterday.toISOString());
        });

        it("allows to use a short form of the unit", async () => {
          const twoYearsAgo = new Date(
            currentTime.getTime() - 2 * nonLeapYearInSeconds,
          );
          const relativeTime: WaRelativeTime = await fixture<WaRelativeTime>(
            html`
              <wa-relative-time
                lang="en-US"
                numeric="always"
                format="short"
              ></wa-relative-time>
            `,
          );
          relativeTime.date = twoYearsAgo;

          await expectFormattedRelativeTimeToBe(relativeTime, "2 yr. ago");
        });

        it("allows to use a long form of the unit", async () => {
          const twoYearsAgo = new Date(
            currentTime.getTime() - 2 * nonLeapYearInSeconds,
          );
          const relativeTime: WaRelativeTime = await fixture<WaRelativeTime>(
            html`
              <wa-relative-time
                lang="en-US"
                numeric="always"
                format="long"
              ></wa-relative-time>
            `,
          );
          relativeTime.date = twoYearsAgo;

          await expectFormattedRelativeTimeToBe(relativeTime, "2 years ago");
        });

        it("is formatted according to the requested locale", async () => {
          const relativeTime: WaRelativeTime = await fixture<WaRelativeTime>(
            html`
              <wa-relative-time lang="de-DE" numeric="auto"></wa-relative-time>
            `,
          );
          relativeTime.date = yesterday;

          await expectFormattedRelativeTimeToBe(relativeTime, "gestern");
        });

        it("keeps the component in sync if requested", async () => {
          const relativeTime = await createRelativeTimeWithDate(
            yesterday,
            fixture,
          );
          relativeTime.sync = true;

          await expectFormattedRelativeTimeToBe(relativeTime, "yesterday");

          clock?.tick(dayInSeconds);

          await expectFormattedRelativeTimeToBe(relativeTime, "2 days ago");
        });
      });

      it("does not display a time element on invalid time string", async () => {
        const invalidDateString = "thisIsNotATimeString";

        const relativeTime: WaRelativeTime = await fixture<WaRelativeTime>(html`
          <wa-relative-time
            lang="en-US"
            date="${invalidDateString}"
          ></wa-relative-time>
        `);

        await relativeTime.updateComplete;
        expect(extractTimeElement(relativeTime)).to.be.null;
      });
    });
  }
});

`````
