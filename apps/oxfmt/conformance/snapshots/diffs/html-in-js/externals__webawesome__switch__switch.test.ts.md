# externals/webawesome/switch/switch.test.ts

## Option 1

`````json
{"printWidth":80}
`````

### Diff

`````diff
===================================================================
--- prettier
+++ oxfmt
@@ -478,9 +478,12 @@
           const el = await fixture<HTMLDivElement>(html`
             <div
               style="display: flex; flex-direction: column; overflow: auto; max-height: 400px;"
             >
-              ${Array.from({ length: 41 }, () => html`<wa-switch>Switch</wa-switch>`)}
+              ${Array.from(
+                { length: 41 },
+                () => html`<wa-switch>Switch</wa-switch>`,
+              )}
             </div>
           `);
 
           const switches = el.querySelectorAll<WaSwitch>("wa-switch");

`````

### Actual (oxfmt)

`````ts
import { aTimeout, expect, oneEvent, waitUntil } from "@open-wc/testing";
import { sendKeys } from "@web/test-runner-commands";
import { html } from "lit";
import sinon from "sinon";
import { expectEvent } from "../../internal/test/expect-event.js";
import { fixtures } from "../../internal/test/fixture.js";
import { runFormControlBaseTests } from "../../internal/test/form-control-base-tests.js";
import type WaSwitch from "./switch.js";

describe("<wa-switch>", () => {
  runFormControlBaseTests("wa-switch");

  for (const fixture of fixtures) {
    describe(`with "${fixture.type}" rendering`, () => {
      describe("accessibility", () => {
        it("should pass accessibility tests", async () => {
          const el = await fixture<WaSwitch>(
            html`<wa-switch>Switch</wa-switch>`,
          );
          await expect(el).to.be.accessible();
        });

        it('should have role="switch" on the internal input', async () => {
          const el = await fixture<WaSwitch>(
            html`<wa-switch>Switch</wa-switch>`,
          );
          const input = el.shadowRoot!.querySelector("input")!;
          expect(input.getAttribute("role")).to.equal("switch");
        });

        it("should set aria-checked to match checked state", async () => {
          const el = await fixture<WaSwitch>(
            html`<wa-switch>Switch</wa-switch>`,
          );
          const input = el.shadowRoot!.querySelector("input")!;

          expect(input.getAttribute("aria-checked")).to.equal("false");

          el.checked = true;
          await el.updateComplete;

          expect(input.getAttribute("aria-checked")).to.equal("true");
        });
      });

      describe("properties", () => {
        it("should have correct default property values", async () => {
          const el = await fixture<WaSwitch>(html`<wa-switch></wa-switch>`);

          expect(el.name).to.equal(null);
          expect(el.value).to.equal("on");
          expect(el.title).to.equal("");
          expect(el.disabled).to.be.false;
          expect(el.required).to.be.false;
          expect(el.checked).to.be.false;
          expect(el.defaultChecked).to.be.false;
          expect(el.hint).to.equal("");
          expect(el.size).to.equal("m");
        });

        it("should reflect the checked attribute as defaultChecked", async () => {
          const el = await fixture<WaSwitch>(
            html`<wa-switch checked></wa-switch>`,
          );
          expect(el.defaultChecked).to.be.true;
          expect(el.checked).to.be.true;
        });

        it('should default value to "on" when no value attribute is set', async () => {
          const el = await fixture<WaSwitch>(html`<wa-switch></wa-switch>`);
          expect(el.value).to.equal("on");
        });

        it("should return the value regardless of checked state", async () => {
          const el = await fixture<WaSwitch>(
            html`<wa-switch value="myvalue" checked></wa-switch>`,
          );

          expect(el.checked).to.be.true;
          expect(el.value).to.equal("myvalue");

          el.checked = false;
          await el.updateComplete;

          expect(el.checked).to.be.false;
          expect(el.value).to.equal("myvalue");
        });

        it("should have title on the internal input if title attribute is set", async () => {
          const el = await fixture<WaSwitch>(
            html`<wa-switch title="Test"></wa-switch>`,
          );
          const input = el.shadowRoot!.querySelector("input")!;
          expect(input.title).to.equal("Test");
        });

        it("should be disabled with the disabled attribute", async () => {
          const el = await fixture<WaSwitch>(
            html`<wa-switch disabled></wa-switch>`,
          );
          const input =
            el.shadowRoot!.querySelector<HTMLInputElement>("input")!;
          expect(input.disabled).to.be.true;
        });

        it("should update checked when set programmatically", async () => {
          const el = await fixture<WaSwitch>(html`<wa-switch></wa-switch>`);
          expect(el.checked).to.equal(false);

          el.checked = true;
          await el.updateComplete;

          expect(el.checked).to.equal(true);
        });

        it("should be valid by default", async () => {
          const el = await fixture<WaSwitch>(html`<wa-switch></wa-switch>`);
          expect(el.checkValidity()).to.be.true;
        });
      });

      describe("events", () => {
        it("should emit change and input when clicked", async () => {
          const el = await fixture<WaSwitch>(html`<wa-switch></wa-switch>`);

          await expectEvent(el, ["change", "input"], () => el.click());

          expect(el.checked).to.be.true;
        });

        it("should emit change and input when toggled with spacebar", async () => {
          const el = await fixture<WaSwitch>(html`<wa-switch></wa-switch>`);

          el.focus();
          await expectEvent(el, ["change", "input"], () =>
            sendKeys({ press: " " }),
          );

          expect(el.checked).to.be.true;
        });

        it("should emit change and input when toggled with the right arrow", async () => {
          const el = await fixture<WaSwitch>(html`<wa-switch></wa-switch>`);

          el.focus();
          await expectEvent(el, ["change", "input"], () =>
            sendKeys({ press: "ArrowRight" }),
          );

          expect(el.checked).to.be.true;
        });

        it("should emit change and input when toggled with the left arrow", async () => {
          const el = await fixture<WaSwitch>(
            html`<wa-switch checked></wa-switch>`,
          );

          el.focus();
          await expectEvent(el, ["change", "input"], () =>
            sendKeys({ press: "ArrowLeft" }),
          );

          expect(el.checked).to.be.false;
        });

        it("should not emit change or input when checked is set by JavaScript", async () => {
          const el = await fixture<WaSwitch>(html`<wa-switch></wa-switch>`);
          el.addEventListener("change", () =>
            expect.fail("change incorrectly emitted"),
          );
          el.addEventListener("input", () =>
            expect.fail("input incorrectly emitted"),
          );

          el.checked = true;
          await el.updateComplete;
          el.checked = false;
          await el.updateComplete;
        });
      });

      describe("slots", () => {
        it("should render the default slot for label content", async () => {
          const el = await fixture<WaSwitch>(
            html`<wa-switch>Switch Label</wa-switch>`,
          );
          const labelSlot =
            el.shadowRoot!.querySelector<HTMLSlotElement>("slot:not([name])")!;
          expect(labelSlot).to.exist;
        });

        it("should display hint text via the hint attribute", async () => {
          const el = await fixture<WaSwitch>(
            html`<wa-switch hint="Help text">Switch</wa-switch>`,
          );
          const hintSlot = el.shadowRoot!.querySelector('[part="hint"]')!;
          expect(hintSlot.textContent).to.contain("Help text");
        });

        it("should display hint text via the hint slot", async () => {
          const el = await fixture<WaSwitch>(html`
            <wa-switch>
              Switch
              <span slot="hint">Slotted hint</span>
            </wa-switch>
          `);
          const hintSlot =
            el.shadowRoot!.querySelector<HTMLSlotElement>('slot[name="hint"]')!;
          expect(hintSlot).to.exist;
        });
      });

      describe("keyboard navigation", () => {
        it("should toggle on with Space key", async () => {
          const el = await fixture<WaSwitch>(html`<wa-switch></wa-switch>`);
          el.focus();
          await sendKeys({ press: " " });
          await el.updateComplete;

          expect(el.checked).to.be.true;
        });

        it("should toggle off with Space key", async () => {
          const el = await fixture<WaSwitch>(
            html`<wa-switch checked></wa-switch>`,
          );
          el.focus();
          await sendKeys({ press: " " });
          await el.updateComplete;

          expect(el.checked).to.be.false;
        });

        it("should turn on with ArrowRight key", async () => {
          const el = await fixture<WaSwitch>(html`<wa-switch></wa-switch>`);
          el.focus();
          await sendKeys({ press: "ArrowRight" });
          await el.updateComplete;

          expect(el.checked).to.be.true;
        });

        it("should turn off with ArrowLeft key", async () => {
          const el = await fixture<WaSwitch>(
            html`<wa-switch checked></wa-switch>`,
          );
          el.focus();
          await sendKeys({ press: "ArrowLeft" });
          await el.updateComplete;

          expect(el.checked).to.be.false;
        });
      });

      describe("form integration", () => {
        it("should submit the correct value when a value is provided", async () => {
          const form = await fixture<HTMLFormElement>(html`
            <form>
              <wa-switch name="a" value="1" checked></wa-switch>
              <wa-button type="submit">Submit</wa-button>
            </form>
          `);
          const button = form.querySelector("wa-button")!;
          const submitHandler = sinon.spy((event: SubmitEvent) => {
            formData = new FormData(form);
            event.preventDefault();
          });
          let formData: FormData;

          form.addEventListener("submit", submitHandler);
          button.click();

          await waitUntil(() => submitHandler.calledOnce);
          expect(formData!.get("a")).to.equal("1");
        });

        it('should submit "on" when no value is provided', async () => {
          const form = await fixture<HTMLFormElement>(html`
            <form>
              <wa-switch name="a" checked></wa-switch>
              <wa-button type="submit">Submit</wa-button>
            </form>
          `);
          const button = form.querySelector("wa-button")!;
          const submitHandler = sinon.spy((event: SubmitEvent) => {
            formData = new FormData(form);
            event.preventDefault();
          });
          let formData: FormData;

          form.addEventListener("submit", submitHandler);
          button.click();

          await waitUntil(() => submitHandler.calledOnce);
          expect(formData!.get("a")).to.equal("on");
        });

        it("should not submit a value when unchecked", async () => {
          const form = await fixture<HTMLFormElement>(html`
            <form>
              <wa-switch name="a" value="1"></wa-switch>
            </form>
          `);
          const formData = new FormData(form);
          expect(formData.get("a")).to.be.null;
        });

        it("should show a constraint validation error when setCustomValidity() is called", async () => {
          const form = await fixture<HTMLFormElement>(html`
            <form>
              <wa-switch name="a" value="1" checked></wa-switch>
              <wa-button type="submit">Submit</wa-button>
            </form>
          `);
          const button = form.querySelector("wa-button")!;
          const waSwitch = form.querySelector("wa-switch")!;
          const submitHandler = sinon.spy((event: SubmitEvent) =>
            event.preventDefault(),
          );

          waSwitch.setCustomValidity("Invalid selection");
          form.addEventListener("submit", submitHandler);
          button.click();
          await aTimeout(100);

          expect(submitHandler).to.not.have.been.called;
        });

        it("should be invalid when required and unchecked", async () => {
          const el = await fixture<WaSwitch>(
            html`<wa-switch required></wa-switch>`,
          );
          expect(el.checkValidity()).to.be.false;
        });

        it("should be valid when required and checked", async () => {
          const el = await fixture<WaSwitch>(
            html`<wa-switch required checked></wa-switch>`,
          );
          expect(el.checkValidity()).to.be.true;
        });

        it("should be present in form data when using the form attribute and located outside of a <form>", async () => {
          const el = await fixture<HTMLFormElement>(html`
            <div>
              <form id="f">
                <wa-button type="submit">Submit</wa-button>
              </form>
              <wa-switch form="f" name="a" value="1" checked></wa-switch>
            </div>
          `);
          const form = el.querySelector("form")!;
          const formData = new FormData(form);
          expect(formData.get("a")).to.equal("1");
        });

        it("should reset the element to its initial value on form reset", async () => {
          const form = await fixture<HTMLFormElement>(html`
            <form>
              <wa-switch name="a" value="1" checked></wa-switch>
              <wa-button type="reset">Reset</wa-button>
            </form>
          `);
          const button = form.querySelector("wa-button")!;
          const switchEl = form.querySelector("wa-switch")!;

          switchEl.checked = false;
          await switchEl.updateComplete;

          setTimeout(() => button.click());
          await oneEvent(form, "reset");
          await switchEl.updateComplete;

          expect(switchEl.checked).to.be.true;

          switchEl.defaultChecked = false;

          setTimeout(() => button.click());
          await oneEvent(form, "reset");
          await switchEl.updateComplete;

          expect(switchEl.checked).to.be.false;
        });
      });

      describe("CSS parts and states", () => {
        it("should expose CSS parts", async () => {
          const el = await fixture<WaSwitch>(
            html`<wa-switch hint="Help">Switch</wa-switch>`,
          );
          expect(el.shadowRoot!.querySelector('[part="base"]')).to.exist;
          expect(el.shadowRoot!.querySelector('[part="control"]')).to.exist;
          expect(el.shadowRoot!.querySelector('[part="thumb"]')).to.exist;
          expect(el.shadowRoot!.querySelector('[part="label"]')).to.exist;
          expect(el.shadowRoot!.querySelector('[part="hint"]')).to.exist;
        });

        it("should set :state(checked) when checked", async () => {
          const el = await fixture<WaSwitch>(
            html`<wa-switch checked></wa-switch>`,
          );
          expect(el.customStates.has("checked")).to.be.true;
        });

        it("should not have :state(checked) when unchecked", async () => {
          const el = await fixture<WaSwitch>(html`<wa-switch></wa-switch>`);
          expect(el.customStates.has("checked")).to.be.false;
        });

        it("should toggle :state(checked) when clicked", async () => {
          const el = await fixture<WaSwitch>(html`<wa-switch></wa-switch>`);
          expect(el.customStates.has("checked")).to.be.false;

          el.click();
          await el.updateComplete;

          expect(el.customStates.has("checked")).to.be.true;
        });

        it("should receive validation states even when novalidate is used on the parent form", async () => {
          const el = await fixture<HTMLFormElement>(
            html`<form novalidate><wa-switch required></wa-switch></form>`,
          );
          const waSwitch = el.querySelector<WaSwitch>("wa-switch")!;

          expect(waSwitch.customStates.has("required")).to.be.true;
          expect(waSwitch.customStates.has("optional")).to.be.false;
          expect(waSwitch.customStates.has("invalid")).to.be.true;
          expect(waSwitch.customStates.has("valid")).to.be.false;
          expect(waSwitch.customStates.has("user-invalid")).to.be.false;
          expect(waSwitch.customStates.has("user-valid")).to.be.false;
        });

        it("should set :state(user-valid) after user interaction when valid", async () => {
          const el = await fixture<WaSwitch>(
            html`<wa-switch required></wa-switch>`,
          );

          // Initially no user states
          expect(el.customStates.has("user-valid")).to.be.false;
          expect(el.customStates.has("user-invalid")).to.be.false;

          // Click to check (satisfies required)
          el.click();
          await el.updateComplete;

          expect(el.customStates.has("user-valid")).to.be.true;
          expect(el.customStates.has("user-invalid")).to.be.false;
        });

        it("should set :state(user-invalid) after user interaction when invalid", async () => {
          const el = await fixture<WaSwitch>(
            html`<wa-switch required checked></wa-switch>`,
          );

          // Click to uncheck (violates required)
          el.click();
          await el.updateComplete;

          expect(el.customStates.has("user-invalid")).to.be.true;
          expect(el.customStates.has("user-valid")).to.be.false;
        });
      });

      describe("regression tests", () => {
        it("should hide the native input with correct positioning for overflow scroll", async () => {
          // https://github.com/shoelace-style/shoelace/issues/1169
          const el = await fixture<WaSwitch>(html`<wa-switch></wa-switch>`);
          const control = el.shadowRoot!.querySelector(".switch")!;
          const input = el.shadowRoot!.querySelector(".input")!;

          expect(getComputedStyle(control).position).to.equal("relative");
          expect(getComputedStyle(input).position).to.equal("absolute");
        });

        it("should not jump the page when focusing a switch at the bottom of an overflow container", async () => {
          // https://github.com/shoelace-style/shoelace/issues/1169
          const el = await fixture<HTMLDivElement>(html`
            <div
              style="display: flex; flex-direction: column; overflow: auto; max-height: 400px;"
            >
              ${Array.from(
                { length: 41 },
                () => html`<wa-switch>Switch</wa-switch>`,
              )}
            </div>
          `);

          const switches = el.querySelectorAll<WaSwitch>("wa-switch");
          const lastSwitch = switches[switches.length - 1];

          expect(window.scrollY).to.equal(0);
          await aTimeout(10);
          lastSwitch.focus();
          await aTimeout(10);
          expect(window.scrollY).to.equal(0);
        });
      });
    });
  }
});

`````

### Expected (prettier)

`````ts
import { aTimeout, expect, oneEvent, waitUntil } from "@open-wc/testing";
import { sendKeys } from "@web/test-runner-commands";
import { html } from "lit";
import sinon from "sinon";
import { expectEvent } from "../../internal/test/expect-event.js";
import { fixtures } from "../../internal/test/fixture.js";
import { runFormControlBaseTests } from "../../internal/test/form-control-base-tests.js";
import type WaSwitch from "./switch.js";

describe("<wa-switch>", () => {
  runFormControlBaseTests("wa-switch");

  for (const fixture of fixtures) {
    describe(`with "${fixture.type}" rendering`, () => {
      describe("accessibility", () => {
        it("should pass accessibility tests", async () => {
          const el = await fixture<WaSwitch>(
            html`<wa-switch>Switch</wa-switch>`,
          );
          await expect(el).to.be.accessible();
        });

        it('should have role="switch" on the internal input', async () => {
          const el = await fixture<WaSwitch>(
            html`<wa-switch>Switch</wa-switch>`,
          );
          const input = el.shadowRoot!.querySelector("input")!;
          expect(input.getAttribute("role")).to.equal("switch");
        });

        it("should set aria-checked to match checked state", async () => {
          const el = await fixture<WaSwitch>(
            html`<wa-switch>Switch</wa-switch>`,
          );
          const input = el.shadowRoot!.querySelector("input")!;

          expect(input.getAttribute("aria-checked")).to.equal("false");

          el.checked = true;
          await el.updateComplete;

          expect(input.getAttribute("aria-checked")).to.equal("true");
        });
      });

      describe("properties", () => {
        it("should have correct default property values", async () => {
          const el = await fixture<WaSwitch>(html`<wa-switch></wa-switch>`);

          expect(el.name).to.equal(null);
          expect(el.value).to.equal("on");
          expect(el.title).to.equal("");
          expect(el.disabled).to.be.false;
          expect(el.required).to.be.false;
          expect(el.checked).to.be.false;
          expect(el.defaultChecked).to.be.false;
          expect(el.hint).to.equal("");
          expect(el.size).to.equal("m");
        });

        it("should reflect the checked attribute as defaultChecked", async () => {
          const el = await fixture<WaSwitch>(
            html`<wa-switch checked></wa-switch>`,
          );
          expect(el.defaultChecked).to.be.true;
          expect(el.checked).to.be.true;
        });

        it('should default value to "on" when no value attribute is set', async () => {
          const el = await fixture<WaSwitch>(html`<wa-switch></wa-switch>`);
          expect(el.value).to.equal("on");
        });

        it("should return the value regardless of checked state", async () => {
          const el = await fixture<WaSwitch>(
            html`<wa-switch value="myvalue" checked></wa-switch>`,
          );

          expect(el.checked).to.be.true;
          expect(el.value).to.equal("myvalue");

          el.checked = false;
          await el.updateComplete;

          expect(el.checked).to.be.false;
          expect(el.value).to.equal("myvalue");
        });

        it("should have title on the internal input if title attribute is set", async () => {
          const el = await fixture<WaSwitch>(
            html`<wa-switch title="Test"></wa-switch>`,
          );
          const input = el.shadowRoot!.querySelector("input")!;
          expect(input.title).to.equal("Test");
        });

        it("should be disabled with the disabled attribute", async () => {
          const el = await fixture<WaSwitch>(
            html`<wa-switch disabled></wa-switch>`,
          );
          const input =
            el.shadowRoot!.querySelector<HTMLInputElement>("input")!;
          expect(input.disabled).to.be.true;
        });

        it("should update checked when set programmatically", async () => {
          const el = await fixture<WaSwitch>(html`<wa-switch></wa-switch>`);
          expect(el.checked).to.equal(false);

          el.checked = true;
          await el.updateComplete;

          expect(el.checked).to.equal(true);
        });

        it("should be valid by default", async () => {
          const el = await fixture<WaSwitch>(html`<wa-switch></wa-switch>`);
          expect(el.checkValidity()).to.be.true;
        });
      });

      describe("events", () => {
        it("should emit change and input when clicked", async () => {
          const el = await fixture<WaSwitch>(html`<wa-switch></wa-switch>`);

          await expectEvent(el, ["change", "input"], () => el.click());

          expect(el.checked).to.be.true;
        });

        it("should emit change and input when toggled with spacebar", async () => {
          const el = await fixture<WaSwitch>(html`<wa-switch></wa-switch>`);

          el.focus();
          await expectEvent(el, ["change", "input"], () =>
            sendKeys({ press: " " }),
          );

          expect(el.checked).to.be.true;
        });

        it("should emit change and input when toggled with the right arrow", async () => {
          const el = await fixture<WaSwitch>(html`<wa-switch></wa-switch>`);

          el.focus();
          await expectEvent(el, ["change", "input"], () =>
            sendKeys({ press: "ArrowRight" }),
          );

          expect(el.checked).to.be.true;
        });

        it("should emit change and input when toggled with the left arrow", async () => {
          const el = await fixture<WaSwitch>(
            html`<wa-switch checked></wa-switch>`,
          );

          el.focus();
          await expectEvent(el, ["change", "input"], () =>
            sendKeys({ press: "ArrowLeft" }),
          );

          expect(el.checked).to.be.false;
        });

        it("should not emit change or input when checked is set by JavaScript", async () => {
          const el = await fixture<WaSwitch>(html`<wa-switch></wa-switch>`);
          el.addEventListener("change", () =>
            expect.fail("change incorrectly emitted"),
          );
          el.addEventListener("input", () =>
            expect.fail("input incorrectly emitted"),
          );

          el.checked = true;
          await el.updateComplete;
          el.checked = false;
          await el.updateComplete;
        });
      });

      describe("slots", () => {
        it("should render the default slot for label content", async () => {
          const el = await fixture<WaSwitch>(
            html`<wa-switch>Switch Label</wa-switch>`,
          );
          const labelSlot =
            el.shadowRoot!.querySelector<HTMLSlotElement>("slot:not([name])")!;
          expect(labelSlot).to.exist;
        });

        it("should display hint text via the hint attribute", async () => {
          const el = await fixture<WaSwitch>(
            html`<wa-switch hint="Help text">Switch</wa-switch>`,
          );
          const hintSlot = el.shadowRoot!.querySelector('[part="hint"]')!;
          expect(hintSlot.textContent).to.contain("Help text");
        });

        it("should display hint text via the hint slot", async () => {
          const el = await fixture<WaSwitch>(html`
            <wa-switch>
              Switch
              <span slot="hint">Slotted hint</span>
            </wa-switch>
          `);
          const hintSlot =
            el.shadowRoot!.querySelector<HTMLSlotElement>('slot[name="hint"]')!;
          expect(hintSlot).to.exist;
        });
      });

      describe("keyboard navigation", () => {
        it("should toggle on with Space key", async () => {
          const el = await fixture<WaSwitch>(html`<wa-switch></wa-switch>`);
          el.focus();
          await sendKeys({ press: " " });
          await el.updateComplete;

          expect(el.checked).to.be.true;
        });

        it("should toggle off with Space key", async () => {
          const el = await fixture<WaSwitch>(
            html`<wa-switch checked></wa-switch>`,
          );
          el.focus();
          await sendKeys({ press: " " });
          await el.updateComplete;

          expect(el.checked).to.be.false;
        });

        it("should turn on with ArrowRight key", async () => {
          const el = await fixture<WaSwitch>(html`<wa-switch></wa-switch>`);
          el.focus();
          await sendKeys({ press: "ArrowRight" });
          await el.updateComplete;

          expect(el.checked).to.be.true;
        });

        it("should turn off with ArrowLeft key", async () => {
          const el = await fixture<WaSwitch>(
            html`<wa-switch checked></wa-switch>`,
          );
          el.focus();
          await sendKeys({ press: "ArrowLeft" });
          await el.updateComplete;

          expect(el.checked).to.be.false;
        });
      });

      describe("form integration", () => {
        it("should submit the correct value when a value is provided", async () => {
          const form = await fixture<HTMLFormElement>(html`
            <form>
              <wa-switch name="a" value="1" checked></wa-switch>
              <wa-button type="submit">Submit</wa-button>
            </form>
          `);
          const button = form.querySelector("wa-button")!;
          const submitHandler = sinon.spy((event: SubmitEvent) => {
            formData = new FormData(form);
            event.preventDefault();
          });
          let formData: FormData;

          form.addEventListener("submit", submitHandler);
          button.click();

          await waitUntil(() => submitHandler.calledOnce);
          expect(formData!.get("a")).to.equal("1");
        });

        it('should submit "on" when no value is provided', async () => {
          const form = await fixture<HTMLFormElement>(html`
            <form>
              <wa-switch name="a" checked></wa-switch>
              <wa-button type="submit">Submit</wa-button>
            </form>
          `);
          const button = form.querySelector("wa-button")!;
          const submitHandler = sinon.spy((event: SubmitEvent) => {
            formData = new FormData(form);
            event.preventDefault();
          });
          let formData: FormData;

          form.addEventListener("submit", submitHandler);
          button.click();

          await waitUntil(() => submitHandler.calledOnce);
          expect(formData!.get("a")).to.equal("on");
        });

        it("should not submit a value when unchecked", async () => {
          const form = await fixture<HTMLFormElement>(html`
            <form>
              <wa-switch name="a" value="1"></wa-switch>
            </form>
          `);
          const formData = new FormData(form);
          expect(formData.get("a")).to.be.null;
        });

        it("should show a constraint validation error when setCustomValidity() is called", async () => {
          const form = await fixture<HTMLFormElement>(html`
            <form>
              <wa-switch name="a" value="1" checked></wa-switch>
              <wa-button type="submit">Submit</wa-button>
            </form>
          `);
          const button = form.querySelector("wa-button")!;
          const waSwitch = form.querySelector("wa-switch")!;
          const submitHandler = sinon.spy((event: SubmitEvent) =>
            event.preventDefault(),
          );

          waSwitch.setCustomValidity("Invalid selection");
          form.addEventListener("submit", submitHandler);
          button.click();
          await aTimeout(100);

          expect(submitHandler).to.not.have.been.called;
        });

        it("should be invalid when required and unchecked", async () => {
          const el = await fixture<WaSwitch>(
            html`<wa-switch required></wa-switch>`,
          );
          expect(el.checkValidity()).to.be.false;
        });

        it("should be valid when required and checked", async () => {
          const el = await fixture<WaSwitch>(
            html`<wa-switch required checked></wa-switch>`,
          );
          expect(el.checkValidity()).to.be.true;
        });

        it("should be present in form data when using the form attribute and located outside of a <form>", async () => {
          const el = await fixture<HTMLFormElement>(html`
            <div>
              <form id="f">
                <wa-button type="submit">Submit</wa-button>
              </form>
              <wa-switch form="f" name="a" value="1" checked></wa-switch>
            </div>
          `);
          const form = el.querySelector("form")!;
          const formData = new FormData(form);
          expect(formData.get("a")).to.equal("1");
        });

        it("should reset the element to its initial value on form reset", async () => {
          const form = await fixture<HTMLFormElement>(html`
            <form>
              <wa-switch name="a" value="1" checked></wa-switch>
              <wa-button type="reset">Reset</wa-button>
            </form>
          `);
          const button = form.querySelector("wa-button")!;
          const switchEl = form.querySelector("wa-switch")!;

          switchEl.checked = false;
          await switchEl.updateComplete;

          setTimeout(() => button.click());
          await oneEvent(form, "reset");
          await switchEl.updateComplete;

          expect(switchEl.checked).to.be.true;

          switchEl.defaultChecked = false;

          setTimeout(() => button.click());
          await oneEvent(form, "reset");
          await switchEl.updateComplete;

          expect(switchEl.checked).to.be.false;
        });
      });

      describe("CSS parts and states", () => {
        it("should expose CSS parts", async () => {
          const el = await fixture<WaSwitch>(
            html`<wa-switch hint="Help">Switch</wa-switch>`,
          );
          expect(el.shadowRoot!.querySelector('[part="base"]')).to.exist;
          expect(el.shadowRoot!.querySelector('[part="control"]')).to.exist;
          expect(el.shadowRoot!.querySelector('[part="thumb"]')).to.exist;
          expect(el.shadowRoot!.querySelector('[part="label"]')).to.exist;
          expect(el.shadowRoot!.querySelector('[part="hint"]')).to.exist;
        });

        it("should set :state(checked) when checked", async () => {
          const el = await fixture<WaSwitch>(
            html`<wa-switch checked></wa-switch>`,
          );
          expect(el.customStates.has("checked")).to.be.true;
        });

        it("should not have :state(checked) when unchecked", async () => {
          const el = await fixture<WaSwitch>(html`<wa-switch></wa-switch>`);
          expect(el.customStates.has("checked")).to.be.false;
        });

        it("should toggle :state(checked) when clicked", async () => {
          const el = await fixture<WaSwitch>(html`<wa-switch></wa-switch>`);
          expect(el.customStates.has("checked")).to.be.false;

          el.click();
          await el.updateComplete;

          expect(el.customStates.has("checked")).to.be.true;
        });

        it("should receive validation states even when novalidate is used on the parent form", async () => {
          const el = await fixture<HTMLFormElement>(
            html`<form novalidate><wa-switch required></wa-switch></form>`,
          );
          const waSwitch = el.querySelector<WaSwitch>("wa-switch")!;

          expect(waSwitch.customStates.has("required")).to.be.true;
          expect(waSwitch.customStates.has("optional")).to.be.false;
          expect(waSwitch.customStates.has("invalid")).to.be.true;
          expect(waSwitch.customStates.has("valid")).to.be.false;
          expect(waSwitch.customStates.has("user-invalid")).to.be.false;
          expect(waSwitch.customStates.has("user-valid")).to.be.false;
        });

        it("should set :state(user-valid) after user interaction when valid", async () => {
          const el = await fixture<WaSwitch>(
            html`<wa-switch required></wa-switch>`,
          );

          // Initially no user states
          expect(el.customStates.has("user-valid")).to.be.false;
          expect(el.customStates.has("user-invalid")).to.be.false;

          // Click to check (satisfies required)
          el.click();
          await el.updateComplete;

          expect(el.customStates.has("user-valid")).to.be.true;
          expect(el.customStates.has("user-invalid")).to.be.false;
        });

        it("should set :state(user-invalid) after user interaction when invalid", async () => {
          const el = await fixture<WaSwitch>(
            html`<wa-switch required checked></wa-switch>`,
          );

          // Click to uncheck (violates required)
          el.click();
          await el.updateComplete;

          expect(el.customStates.has("user-invalid")).to.be.true;
          expect(el.customStates.has("user-valid")).to.be.false;
        });
      });

      describe("regression tests", () => {
        it("should hide the native input with correct positioning for overflow scroll", async () => {
          // https://github.com/shoelace-style/shoelace/issues/1169
          const el = await fixture<WaSwitch>(html`<wa-switch></wa-switch>`);
          const control = el.shadowRoot!.querySelector(".switch")!;
          const input = el.shadowRoot!.querySelector(".input")!;

          expect(getComputedStyle(control).position).to.equal("relative");
          expect(getComputedStyle(input).position).to.equal("absolute");
        });

        it("should not jump the page when focusing a switch at the bottom of an overflow container", async () => {
          // https://github.com/shoelace-style/shoelace/issues/1169
          const el = await fixture<HTMLDivElement>(html`
            <div
              style="display: flex; flex-direction: column; overflow: auto; max-height: 400px;"
            >
              ${Array.from({ length: 41 }, () => html`<wa-switch>Switch</wa-switch>`)}
            </div>
          `);

          const switches = el.querySelectorAll<WaSwitch>("wa-switch");
          const lastSwitch = switches[switches.length - 1];

          expect(window.scrollY).to.equal(0);
          await aTimeout(10);
          lastSwitch.focus();
          await aTimeout(10);
          expect(window.scrollY).to.equal(0);
        });
      });
    });
  }
});

`````
