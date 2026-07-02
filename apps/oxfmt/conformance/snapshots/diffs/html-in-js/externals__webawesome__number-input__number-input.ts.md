# externals/webawesome/number-input/number-input.ts

## Option 1

`````json
{"printWidth":80}
`````

### Diff

`````diff
===================================================================
--- prettier
+++ oxfmt
@@ -104,18 +104,27 @@
     this.getAttribute("value") || null;
 
   /** The input's size. */
   @property({ reflect: true }) size:
-    "xs" | "s" | "m" | "l" | "xl" | "small" | "medium" | "large" = "m";
+    | "xs"
+    | "s"
+    | "m"
+    | "l"
+    | "xl"
+    | "small"
+    | "medium"
+    | "large" = "m";
 
   @watch("size")
   handleSizeChange() {
     warnDeprecatedSize(this.localName, this.size);
   }
 
   /** The input's visual appearance. */
   @property({ reflect: true }) appearance:
-    "filled" | "outlined" | "filled-outlined" = "outlined";
+    | "filled"
+    | "outlined"
+    | "filled-outlined" = "outlined";
 
   /** Draws a pill-style input with rounded edges. */
   @property({ type: Boolean, reflect: true }) pill = false;
 
@@ -160,9 +169,15 @@
   @property({ type: Boolean }) autofocus: boolean;
 
   /** Used to customize the label or icon of the Enter key on virtual keyboards. */
   @property() enterkeyhint:
-    "enter" | "done" | "go" | "next" | "previous" | "search" | "send";
+    | "enter"
+    | "done"
+    | "go"
+    | "next"
+    | "previous"
+    | "search"
+    | "send";
 
   /**
    * Tells the browser what type of data will be entered by the user, allowing it to display the appropriate virtual
    * keyboard on supportive devices.
@@ -346,28 +361,27 @@
         <slot name="label">${this.label}</slot>
       </label>
 
       <div part="base" class="number-field">
-        ${
-          !this.withoutSteppers
-            ? html`
-                <button
-                  part="stepper stepper-decrement"
-                  class="stepper stepper-decrement"
-                  type="button"
-                  tabindex="-1"
-                  aria-label=${this.localize.term("decrement")}
-                  ?disabled=${this.disabled || this.readonly || this.isAtMin}
-                  @pointerdown=${this.handleStepperPointerDown}
-                  @pointerup=${(event: PointerEvent) => this.handleStepperPointerUp("down", event)}
-                >
-                  <slot name="decrement-icon">
-                    <wa-icon name="minus" library="system"></wa-icon>
-                  </slot>
-                </button>
-              `
-            : ""
-        }
+        ${!this.withoutSteppers
+          ? html`
+              <button
+                part="stepper stepper-decrement"
+                class="stepper stepper-decrement"
+                type="button"
+                tabindex="-1"
+                aria-label=${this.localize.term("decrement")}
+                ?disabled=${this.disabled || this.readonly || this.isAtMin}
+                @pointerdown=${this.handleStepperPointerDown}
+                @pointerup=${(event: PointerEvent) =>
+                  this.handleStepperPointerUp("down", event)}
+              >
+                <slot name="decrement-icon">
+                  <wa-icon name="minus" library="system"></wa-icon>
+                </slot>
+              </button>
+            `
+          : ""}
 
         <slot name="start" part="start" class="start"></slot>
 
         <input
@@ -375,9 +389,12 @@
           id="input"
           class="control"
           type="number"
           inputmode=${ifDefined(this.inputmode)}
-          title=${this.title /* An empty title prevents browser validation tooltips from appearing on hover */}
+          title=${
+            this
+              .title /* An empty title prevents browser validation tooltips from appearing on hover */
+          }
           name=${ifDefined(this.name)}
           ?disabled=${this.disabled}
           ?readonly=${this.readonly}
           ?required=${this.required}
@@ -396,28 +413,27 @@
         />
 
         <slot name="end" part="end" class="end"></slot>
 
-        ${
-          !this.withoutSteppers
-            ? html`
-                <button
-                  part="stepper stepper-increment"
-                  class="stepper stepper-increment"
-                  type="button"
-                  tabindex="-1"
-                  aria-label=${this.localize.term("increment")}
-                  ?disabled=${this.disabled || this.readonly || this.isAtMax}
-                  @pointerdown=${this.handleStepperPointerDown}
-                  @pointerup=${(event: PointerEvent) => this.handleStepperPointerUp("up", event)}
-                >
-                  <slot name="increment-icon">
-                    <wa-icon name="plus" library="system"></wa-icon>
-                  </slot>
-                </button>
-              `
-            : ""
-        }
+        ${!this.withoutSteppers
+          ? html`
+              <button
+                part="stepper stepper-increment"
+                class="stepper stepper-increment"
+                type="button"
+                tabindex="-1"
+                aria-label=${this.localize.term("increment")}
+                ?disabled=${this.disabled || this.readonly || this.isAtMax}
+                @pointerdown=${this.handleStepperPointerDown}
+                @pointerup=${(event: PointerEvent) =>
+                  this.handleStepperPointerUp("up", event)}
+              >
+                <slot name="increment-icon">
+                  <wa-icon name="plus" library="system"></wa-icon>
+                </slot>
+              </button>
+            `
+          : ""}
       </div>
 
       <slot
         id="hint"

`````

### Actual (oxfmt)

`````ts
import { html, type PropertyValues } from "lit";
import { customElement, property, query, state } from "lit/decorators.js";
import { classMap } from "lit/directives/class-map.js";
import { ifDefined } from "lit/directives/if-defined.js";
import { live } from "lit/directives/live.js";
import { warnDeprecatedSize } from "../../internal/size.js";
import { HasSlotController } from "../../internal/slot.js";
import { submitOnEnter } from "../../internal/submit-on-enter.js";
import { MirrorValidator } from "../../internal/validators/mirror-validator.js";
import { watch } from "../../internal/watch.js";
import { WebAwesomeFormAssociatedElement } from "../../internal/webawesome-form-associated-element.js";
import formControlStyles from "../../styles/component/form-control.styles.js";
import sizeStyles from "../../styles/component/size.styles.js";
import { LocalizeController } from "../../utilities/localize.js";
import "../icon/icon.js";
import styles from "./number-input.styles.js";

/**
 * @summary Number inputs let users enter and edit numeric values, with optional stepper buttons for incrementing and
 *  decrementing. Use them for quantities, measurements, and other numeric form fields.
 * @documentation https://webawesome.com/docs/components/number-input
 * @status stable
 * @since 3.2
 *
 * @dependency wa-icon
 *
 * @slot label - The input's label. Alternatively, you can use the `label` attribute.
 * @slot start - An element, such as `<wa-icon>`, placed at the start of the input control.
 * @slot end - An element, such as `<wa-icon>`, placed at the end of the input control (before steppers).
 * @slot increment-icon - An icon to use in lieu of the default increment icon.
 * @slot decrement-icon - An icon to use in lieu of the default decrement icon.
 * @slot hint - Text that describes how to use the input. Alternatively, you can use the `hint` attribute.
 *
 * @event blur - Emitted when the control loses focus.
 * @event change - Emitted when an alteration to the control's value is committed by the user.
 * @event focus - Emitted when the control gains focus.
 * @event input - Emitted when the control receives input.
 * @event beforeinput - Emitted before the value changes. Can be cancelled with `event.preventDefault()` to prevent the
 *  value from changing.
 * @event wa-invalid - Emitted when the form control has been checked for validity and its constraints aren't satisfied.
 *
 * @csspart label - The label element.
 * @csspart form-control-label - Alias for the label element.
 * @csspart hint - The hint element.
 * @csspart base - The wrapper containing the input and steppers.
 * @csspart input - The internal `<input>` control.
 * @csspart start - The container that wraps the `start` slot.
 * @csspart end - The container that wraps the `end` slot.
 * @csspart stepper - Both stepper buttons (for shared styling).
 * @csspart stepper-increment - The increment (+) button on the end side.
 * @csspart stepper-decrement - The decrement (-) button on the start side.
 *
 * @cssstate blank - The input is empty.
 * @cssstate focused - The input has focus.
 */
@customElement("wa-number-input")
export default class WaNumberInput extends WebAwesomeFormAssociatedElement {
  static css = [sizeStyles, formControlStyles, styles];

  static shadowRootOptions = {
    ...WebAwesomeFormAssociatedElement.shadowRootOptions,
    delegatesFocus: true,
  };

  static get validators() {
    return [...super.validators, MirrorValidator()];
  }

  assumeInteractionOn = ["blur", "input"];
  private readonly hasSlotController = new HasSlotController(
    this,
    "hint",
    "label",
  );
  private readonly localize = new LocalizeController(this);

  @query("input") input: HTMLInputElement;

  @property() title = ""; // make reactive to pass through

  private _value: string | null = null;

  /** The current value of the input, submitted as a name/value pair with form data. */
  get value() {
    if (this.valueHasChanged) {
      return this._value;
    }

    return this._value ?? this.defaultValue;
  }

  @state()
  set value(val: string | null) {
    if (this._value === val) {
      return;
    }

    this.valueHasChanged = true;
    this._value = val;
  }

  /** The default value of the form control. Primarily used for resetting the form control. */
  @property({ attribute: "value", reflect: true }) defaultValue: string | null =
    this.getAttribute("value") || null;

  /** The input's size. */
  @property({ reflect: true }) size:
    | "xs"
    | "s"
    | "m"
    | "l"
    | "xl"
    | "small"
    | "medium"
    | "large" = "m";

  @watch("size")
  handleSizeChange() {
    warnDeprecatedSize(this.localName, this.size);
  }

  /** The input's visual appearance. */
  @property({ reflect: true }) appearance:
    | "filled"
    | "outlined"
    | "filled-outlined" = "outlined";

  /** Draws a pill-style input with rounded edges. */
  @property({ type: Boolean, reflect: true }) pill = false;

  /** The input's label. If you need to display HTML, use the `label` slot instead. */
  @property() label = "";

  /** The input's hint. If you need to display HTML, use the `hint` slot instead. */
  @property({ attribute: "hint" }) hint = "";

  /** Placeholder text to show as a hint when the input is empty. */
  @property() placeholder = "";

  /** Makes the input readonly. */
  @property({ type: Boolean, reflect: true }) readonly = false;

  /** Makes the input a required field. */
  @property({ type: Boolean, reflect: true }) required = false;

  /** The input's minimum value. */
  @property({ type: Number }) min: number;

  /** The input's maximum value. */
  @property({ type: Number }) max: number;

  /**
   * Specifies the granularity that the value must adhere to, or the special value `any` which means no stepping is
   * implied, allowing any numeric value.
   */
  @property() step: number | "any" = 1;

  /** Hides the increment/decrement stepper buttons. */
  @property({ attribute: "without-steppers", type: Boolean }) withoutSteppers =
    false;

  /**
   * Specifies what permission the browser has to provide assistance in filling out form field values. Refer to
   * [this page on MDN](https://developer.mozilla.org/en-US/docs/Web/HTML/Attributes/autocomplete) for available values.
   */
  @property() autocomplete: string;

  /** Indicates that the input should receive focus on page load. */
  @property({ type: Boolean }) autofocus: boolean;

  /** Used to customize the label or icon of the Enter key on virtual keyboards. */
  @property() enterkeyhint:
    | "enter"
    | "done"
    | "go"
    | "next"
    | "previous"
    | "search"
    | "send";

  /**
   * Tells the browser what type of data will be entered by the user, allowing it to display the appropriate virtual
   * keyboard on supportive devices.
   */
  @property() inputmode: "numeric" | "decimal" = "numeric";

  /**
   * Only required for SSR. Set to `true` if you're slotting in a `label` element so the server-rendered markup
   * includes the label before the component hydrates on the client.
   */
  @property({ attribute: "with-label", type: Boolean }) withLabel = false;

  /**
   * Only required for SSR. Set to `true` if you're slotting in a `hint` element so the server-rendered markup
   * includes the hint before the component hydrates on the client.
   */
  @property({ attribute: "with-hint", type: Boolean }) withHint = false;

  /** Returns true if the value is at or below the minimum. */
  private get isAtMin(): boolean {
    if (this.min === undefined) return false;
    const numValue = parseFloat(this.value || "");
    return !isNaN(numValue) && numValue <= this.min;
  }

  /** Returns true if the value is at or above the maximum. */
  private get isAtMax(): boolean {
    if (this.max === undefined) return false;
    const numValue = parseFloat(this.value || "");
    return !isNaN(numValue) && numValue >= this.max;
  }

  private handleChange(event: Event) {
    this.value = this.input.value;

    this.relayNativeEvent(event, { bubbles: true, composed: true });
  }

  private handleInput() {
    this.value = this.input.value;
  }

  private handleKeyDown(event: KeyboardEvent) {
    submitOnEnter(event, this);

    // Sync value after arrow key changes
    if (event.key === "ArrowUp" || event.key === "ArrowDown") {
      requestAnimationFrame(() => {
        if (this.value !== this.input.value) {
          this.value = this.input.value;
        }
      });
    }
  }

  private handleStepperPointerUp(
    direction: "up" | "down",
    event: PointerEvent,
  ) {
    if (this.disabled || this.readonly) return;

    const beforeInputEvent = new InputEvent("beforeinput", {
      bubbles: true,
      cancelable: true,
      composed: true,
    });
    this.dispatchEvent(beforeInputEvent);
    if (beforeInputEvent.defaultPrevented) return;

    if (direction === "up") {
      this.input.stepUp();
    } else {
      this.input.stepDown();
    }

    if (this.value !== this.input.value) {
      this.value = this.input.value;
    }

    this.dispatchEvent(
      new InputEvent("input", { bubbles: true, composed: true }),
    );
    this.dispatchEvent(new Event("change", { bubbles: true, composed: true }));

    // Avoid focusing the input on touch to prevent the virtual keyboard from showing
    if (event.pointerType !== "touch") {
      this.input.focus();
    }
  }

  private handleStepperPointerDown(event: PointerEvent) {
    // Avoid focusing the input on touch to prevent the virtual keyboard from showing
    if (event.pointerType === "touch") return;

    event.preventDefault();
    this.input.focus();
  }

  updated(changedProperties: PropertyValues<this>) {
    super.updated(changedProperties);

    if (
      changedProperties.has("value") ||
      changedProperties.has("defaultValue")
    ) {
      // The browser sanitizes invalid numeric input to an empty string. Mirror that behavior so `value` stays
      // consistent with the native input (e.g. setting `"abc"` resolves to `""`).
      if (this.input && this.value && this.input.value !== this.value) {
        this._value = this.input.value;
      }

      this.customStates.set("blank", !this.value);
    }
  }

  @watch("step", { waitUntilFirstUpdate: true })
  handleStepChange() {
    // If step changes, the value may become invalid so we need to recheck after the update. We set the new step
    // imperatively so we don't have to wait for the next render to report the updated validity.
    this.input.step = String(this.step);
    this.updateValidity();
  }

  /** Sets focus on the input. */
  focus(options?: FocusOptions) {
    this.input.focus(options);
  }

  /** Removes focus from the input. */
  blur() {
    this.input.blur();
  }

  /** Selects all the text in the input. */
  select() {
    this.input.select();
  }

  /** Increments the value by the step amount. */
  stepUp() {
    this.input.stepUp();
    if (this.value !== this.input.value) {
      this.value = this.input.value;
    }
  }

  /** Decrements the value by the step amount. */
  stepDown() {
    this.input.stepDown();
    if (this.value !== this.input.value) {
      this.value = this.input.value;
    }
  }

  formResetCallback() {
    this.value = this.defaultValue;

    super.formResetCallback();
  }

  render() {
    const hasLabelSlot = this.hasUpdated
      ? this.hasSlotController.test("label")
      : this.withLabel;
    const hasHintSlot = this.hasUpdated
      ? this.hasSlotController.test("hint")
      : this.withHint;
    const hasLabel = this.label ? true : !!hasLabelSlot;
    const hasHint = this.hint ? true : !!hasHintSlot;

    return html`
      <label
        part="form-control-label label"
        class=${classMap({
          label: true,
          "has-label": hasLabel,
        })}
        for="input"
        aria-hidden=${hasLabel ? "false" : "true"}
      >
        <slot name="label">${this.label}</slot>
      </label>

      <div part="base" class="number-field">
        ${!this.withoutSteppers
          ? html`
              <button
                part="stepper stepper-decrement"
                class="stepper stepper-decrement"
                type="button"
                tabindex="-1"
                aria-label=${this.localize.term("decrement")}
                ?disabled=${this.disabled || this.readonly || this.isAtMin}
                @pointerdown=${this.handleStepperPointerDown}
                @pointerup=${(event: PointerEvent) =>
                  this.handleStepperPointerUp("down", event)}
              >
                <slot name="decrement-icon">
                  <wa-icon name="minus" library="system"></wa-icon>
                </slot>
              </button>
            `
          : ""}

        <slot name="start" part="start" class="start"></slot>

        <input
          part="input"
          id="input"
          class="control"
          type="number"
          inputmode=${ifDefined(this.inputmode)}
          title=${
            this
              .title /* An empty title prevents browser validation tooltips from appearing on hover */
          }
          name=${ifDefined(this.name)}
          ?disabled=${this.disabled}
          ?readonly=${this.readonly}
          ?required=${this.required}
          placeholder=${ifDefined(this.placeholder)}
          min=${ifDefined(this.min)}
          max=${ifDefined(this.max)}
          step=${ifDefined(this.step as number)}
          .value=${live(this.value ?? "")}
          autocomplete=${ifDefined(this.autocomplete)}
          ?autofocus=${this.autofocus}
          enterkeyhint=${ifDefined(this.enterkeyhint)}
          aria-describedby="hint"
          @change=${this.handleChange}
          @input=${this.handleInput}
          @keydown=${this.handleKeyDown}
        />

        <slot name="end" part="end" class="end"></slot>

        ${!this.withoutSteppers
          ? html`
              <button
                part="stepper stepper-increment"
                class="stepper stepper-increment"
                type="button"
                tabindex="-1"
                aria-label=${this.localize.term("increment")}
                ?disabled=${this.disabled || this.readonly || this.isAtMax}
                @pointerdown=${this.handleStepperPointerDown}
                @pointerup=${(event: PointerEvent) =>
                  this.handleStepperPointerUp("up", event)}
              >
                <slot name="increment-icon">
                  <wa-icon name="plus" library="system"></wa-icon>
                </slot>
              </button>
            `
          : ""}
      </div>

      <slot
        id="hint"
        part="hint"
        name="hint"
        class=${classMap({
          "has-slotted": hasHint,
        })}
        aria-hidden=${hasHint ? "false" : "true"}
        >${this.hint}</slot
      >
    `;
  }
}

// The change-in-update warning is required for this component because the form-associated base class calls
// updateValidity() in firstUpdated(), which triggers requestUpdate('validity') to sync the validation state after the
// first render when the validation target is available. Additionally, HasSlotController triggers requestUpdate() on
// initial slotchange events. See https://lit.dev/docs/tools/development/#development-build-runtime-warnings
WaNumberInput.disableWarning?.("change-in-update");

declare global {
  interface HTMLElementTagNameMap {
    "wa-number-input": WaNumberInput;
  }
}

`````

### Expected (prettier)

`````ts
import { html, type PropertyValues } from "lit";
import { customElement, property, query, state } from "lit/decorators.js";
import { classMap } from "lit/directives/class-map.js";
import { ifDefined } from "lit/directives/if-defined.js";
import { live } from "lit/directives/live.js";
import { warnDeprecatedSize } from "../../internal/size.js";
import { HasSlotController } from "../../internal/slot.js";
import { submitOnEnter } from "../../internal/submit-on-enter.js";
import { MirrorValidator } from "../../internal/validators/mirror-validator.js";
import { watch } from "../../internal/watch.js";
import { WebAwesomeFormAssociatedElement } from "../../internal/webawesome-form-associated-element.js";
import formControlStyles from "../../styles/component/form-control.styles.js";
import sizeStyles from "../../styles/component/size.styles.js";
import { LocalizeController } from "../../utilities/localize.js";
import "../icon/icon.js";
import styles from "./number-input.styles.js";

/**
 * @summary Number inputs let users enter and edit numeric values, with optional stepper buttons for incrementing and
 *  decrementing. Use them for quantities, measurements, and other numeric form fields.
 * @documentation https://webawesome.com/docs/components/number-input
 * @status stable
 * @since 3.2
 *
 * @dependency wa-icon
 *
 * @slot label - The input's label. Alternatively, you can use the `label` attribute.
 * @slot start - An element, such as `<wa-icon>`, placed at the start of the input control.
 * @slot end - An element, such as `<wa-icon>`, placed at the end of the input control (before steppers).
 * @slot increment-icon - An icon to use in lieu of the default increment icon.
 * @slot decrement-icon - An icon to use in lieu of the default decrement icon.
 * @slot hint - Text that describes how to use the input. Alternatively, you can use the `hint` attribute.
 *
 * @event blur - Emitted when the control loses focus.
 * @event change - Emitted when an alteration to the control's value is committed by the user.
 * @event focus - Emitted when the control gains focus.
 * @event input - Emitted when the control receives input.
 * @event beforeinput - Emitted before the value changes. Can be cancelled with `event.preventDefault()` to prevent the
 *  value from changing.
 * @event wa-invalid - Emitted when the form control has been checked for validity and its constraints aren't satisfied.
 *
 * @csspart label - The label element.
 * @csspart form-control-label - Alias for the label element.
 * @csspart hint - The hint element.
 * @csspart base - The wrapper containing the input and steppers.
 * @csspart input - The internal `<input>` control.
 * @csspart start - The container that wraps the `start` slot.
 * @csspart end - The container that wraps the `end` slot.
 * @csspart stepper - Both stepper buttons (for shared styling).
 * @csspart stepper-increment - The increment (+) button on the end side.
 * @csspart stepper-decrement - The decrement (-) button on the start side.
 *
 * @cssstate blank - The input is empty.
 * @cssstate focused - The input has focus.
 */
@customElement("wa-number-input")
export default class WaNumberInput extends WebAwesomeFormAssociatedElement {
  static css = [sizeStyles, formControlStyles, styles];

  static shadowRootOptions = {
    ...WebAwesomeFormAssociatedElement.shadowRootOptions,
    delegatesFocus: true,
  };

  static get validators() {
    return [...super.validators, MirrorValidator()];
  }

  assumeInteractionOn = ["blur", "input"];
  private readonly hasSlotController = new HasSlotController(
    this,
    "hint",
    "label",
  );
  private readonly localize = new LocalizeController(this);

  @query("input") input: HTMLInputElement;

  @property() title = ""; // make reactive to pass through

  private _value: string | null = null;

  /** The current value of the input, submitted as a name/value pair with form data. */
  get value() {
    if (this.valueHasChanged) {
      return this._value;
    }

    return this._value ?? this.defaultValue;
  }

  @state()
  set value(val: string | null) {
    if (this._value === val) {
      return;
    }

    this.valueHasChanged = true;
    this._value = val;
  }

  /** The default value of the form control. Primarily used for resetting the form control. */
  @property({ attribute: "value", reflect: true }) defaultValue: string | null =
    this.getAttribute("value") || null;

  /** The input's size. */
  @property({ reflect: true }) size:
    "xs" | "s" | "m" | "l" | "xl" | "small" | "medium" | "large" = "m";

  @watch("size")
  handleSizeChange() {
    warnDeprecatedSize(this.localName, this.size);
  }

  /** The input's visual appearance. */
  @property({ reflect: true }) appearance:
    "filled" | "outlined" | "filled-outlined" = "outlined";

  /** Draws a pill-style input with rounded edges. */
  @property({ type: Boolean, reflect: true }) pill = false;

  /** The input's label. If you need to display HTML, use the `label` slot instead. */
  @property() label = "";

  /** The input's hint. If you need to display HTML, use the `hint` slot instead. */
  @property({ attribute: "hint" }) hint = "";

  /** Placeholder text to show as a hint when the input is empty. */
  @property() placeholder = "";

  /** Makes the input readonly. */
  @property({ type: Boolean, reflect: true }) readonly = false;

  /** Makes the input a required field. */
  @property({ type: Boolean, reflect: true }) required = false;

  /** The input's minimum value. */
  @property({ type: Number }) min: number;

  /** The input's maximum value. */
  @property({ type: Number }) max: number;

  /**
   * Specifies the granularity that the value must adhere to, or the special value `any` which means no stepping is
   * implied, allowing any numeric value.
   */
  @property() step: number | "any" = 1;

  /** Hides the increment/decrement stepper buttons. */
  @property({ attribute: "without-steppers", type: Boolean }) withoutSteppers =
    false;

  /**
   * Specifies what permission the browser has to provide assistance in filling out form field values. Refer to
   * [this page on MDN](https://developer.mozilla.org/en-US/docs/Web/HTML/Attributes/autocomplete) for available values.
   */
  @property() autocomplete: string;

  /** Indicates that the input should receive focus on page load. */
  @property({ type: Boolean }) autofocus: boolean;

  /** Used to customize the label or icon of the Enter key on virtual keyboards. */
  @property() enterkeyhint:
    "enter" | "done" | "go" | "next" | "previous" | "search" | "send";

  /**
   * Tells the browser what type of data will be entered by the user, allowing it to display the appropriate virtual
   * keyboard on supportive devices.
   */
  @property() inputmode: "numeric" | "decimal" = "numeric";

  /**
   * Only required for SSR. Set to `true` if you're slotting in a `label` element so the server-rendered markup
   * includes the label before the component hydrates on the client.
   */
  @property({ attribute: "with-label", type: Boolean }) withLabel = false;

  /**
   * Only required for SSR. Set to `true` if you're slotting in a `hint` element so the server-rendered markup
   * includes the hint before the component hydrates on the client.
   */
  @property({ attribute: "with-hint", type: Boolean }) withHint = false;

  /** Returns true if the value is at or below the minimum. */
  private get isAtMin(): boolean {
    if (this.min === undefined) return false;
    const numValue = parseFloat(this.value || "");
    return !isNaN(numValue) && numValue <= this.min;
  }

  /** Returns true if the value is at or above the maximum. */
  private get isAtMax(): boolean {
    if (this.max === undefined) return false;
    const numValue = parseFloat(this.value || "");
    return !isNaN(numValue) && numValue >= this.max;
  }

  private handleChange(event: Event) {
    this.value = this.input.value;

    this.relayNativeEvent(event, { bubbles: true, composed: true });
  }

  private handleInput() {
    this.value = this.input.value;
  }

  private handleKeyDown(event: KeyboardEvent) {
    submitOnEnter(event, this);

    // Sync value after arrow key changes
    if (event.key === "ArrowUp" || event.key === "ArrowDown") {
      requestAnimationFrame(() => {
        if (this.value !== this.input.value) {
          this.value = this.input.value;
        }
      });
    }
  }

  private handleStepperPointerUp(
    direction: "up" | "down",
    event: PointerEvent,
  ) {
    if (this.disabled || this.readonly) return;

    const beforeInputEvent = new InputEvent("beforeinput", {
      bubbles: true,
      cancelable: true,
      composed: true,
    });
    this.dispatchEvent(beforeInputEvent);
    if (beforeInputEvent.defaultPrevented) return;

    if (direction === "up") {
      this.input.stepUp();
    } else {
      this.input.stepDown();
    }

    if (this.value !== this.input.value) {
      this.value = this.input.value;
    }

    this.dispatchEvent(
      new InputEvent("input", { bubbles: true, composed: true }),
    );
    this.dispatchEvent(new Event("change", { bubbles: true, composed: true }));

    // Avoid focusing the input on touch to prevent the virtual keyboard from showing
    if (event.pointerType !== "touch") {
      this.input.focus();
    }
  }

  private handleStepperPointerDown(event: PointerEvent) {
    // Avoid focusing the input on touch to prevent the virtual keyboard from showing
    if (event.pointerType === "touch") return;

    event.preventDefault();
    this.input.focus();
  }

  updated(changedProperties: PropertyValues<this>) {
    super.updated(changedProperties);

    if (
      changedProperties.has("value") ||
      changedProperties.has("defaultValue")
    ) {
      // The browser sanitizes invalid numeric input to an empty string. Mirror that behavior so `value` stays
      // consistent with the native input (e.g. setting `"abc"` resolves to `""`).
      if (this.input && this.value && this.input.value !== this.value) {
        this._value = this.input.value;
      }

      this.customStates.set("blank", !this.value);
    }
  }

  @watch("step", { waitUntilFirstUpdate: true })
  handleStepChange() {
    // If step changes, the value may become invalid so we need to recheck after the update. We set the new step
    // imperatively so we don't have to wait for the next render to report the updated validity.
    this.input.step = String(this.step);
    this.updateValidity();
  }

  /** Sets focus on the input. */
  focus(options?: FocusOptions) {
    this.input.focus(options);
  }

  /** Removes focus from the input. */
  blur() {
    this.input.blur();
  }

  /** Selects all the text in the input. */
  select() {
    this.input.select();
  }

  /** Increments the value by the step amount. */
  stepUp() {
    this.input.stepUp();
    if (this.value !== this.input.value) {
      this.value = this.input.value;
    }
  }

  /** Decrements the value by the step amount. */
  stepDown() {
    this.input.stepDown();
    if (this.value !== this.input.value) {
      this.value = this.input.value;
    }
  }

  formResetCallback() {
    this.value = this.defaultValue;

    super.formResetCallback();
  }

  render() {
    const hasLabelSlot = this.hasUpdated
      ? this.hasSlotController.test("label")
      : this.withLabel;
    const hasHintSlot = this.hasUpdated
      ? this.hasSlotController.test("hint")
      : this.withHint;
    const hasLabel = this.label ? true : !!hasLabelSlot;
    const hasHint = this.hint ? true : !!hasHintSlot;

    return html`
      <label
        part="form-control-label label"
        class=${classMap({
          label: true,
          "has-label": hasLabel,
        })}
        for="input"
        aria-hidden=${hasLabel ? "false" : "true"}
      >
        <slot name="label">${this.label}</slot>
      </label>

      <div part="base" class="number-field">
        ${
          !this.withoutSteppers
            ? html`
                <button
                  part="stepper stepper-decrement"
                  class="stepper stepper-decrement"
                  type="button"
                  tabindex="-1"
                  aria-label=${this.localize.term("decrement")}
                  ?disabled=${this.disabled || this.readonly || this.isAtMin}
                  @pointerdown=${this.handleStepperPointerDown}
                  @pointerup=${(event: PointerEvent) => this.handleStepperPointerUp("down", event)}
                >
                  <slot name="decrement-icon">
                    <wa-icon name="minus" library="system"></wa-icon>
                  </slot>
                </button>
              `
            : ""
        }

        <slot name="start" part="start" class="start"></slot>

        <input
          part="input"
          id="input"
          class="control"
          type="number"
          inputmode=${ifDefined(this.inputmode)}
          title=${this.title /* An empty title prevents browser validation tooltips from appearing on hover */}
          name=${ifDefined(this.name)}
          ?disabled=${this.disabled}
          ?readonly=${this.readonly}
          ?required=${this.required}
          placeholder=${ifDefined(this.placeholder)}
          min=${ifDefined(this.min)}
          max=${ifDefined(this.max)}
          step=${ifDefined(this.step as number)}
          .value=${live(this.value ?? "")}
          autocomplete=${ifDefined(this.autocomplete)}
          ?autofocus=${this.autofocus}
          enterkeyhint=${ifDefined(this.enterkeyhint)}
          aria-describedby="hint"
          @change=${this.handleChange}
          @input=${this.handleInput}
          @keydown=${this.handleKeyDown}
        />

        <slot name="end" part="end" class="end"></slot>

        ${
          !this.withoutSteppers
            ? html`
                <button
                  part="stepper stepper-increment"
                  class="stepper stepper-increment"
                  type="button"
                  tabindex="-1"
                  aria-label=${this.localize.term("increment")}
                  ?disabled=${this.disabled || this.readonly || this.isAtMax}
                  @pointerdown=${this.handleStepperPointerDown}
                  @pointerup=${(event: PointerEvent) => this.handleStepperPointerUp("up", event)}
                >
                  <slot name="increment-icon">
                    <wa-icon name="plus" library="system"></wa-icon>
                  </slot>
                </button>
              `
            : ""
        }
      </div>

      <slot
        id="hint"
        part="hint"
        name="hint"
        class=${classMap({
          "has-slotted": hasHint,
        })}
        aria-hidden=${hasHint ? "false" : "true"}
        >${this.hint}</slot
      >
    `;
  }
}

// The change-in-update warning is required for this component because the form-associated base class calls
// updateValidity() in firstUpdated(), which triggers requestUpdate('validity') to sync the validation state after the
// first render when the validation target is available. Additionally, HasSlotController triggers requestUpdate() on
// initial slotchange events. See https://lit.dev/docs/tools/development/#development-build-runtime-warnings
WaNumberInput.disableWarning?.("change-in-update");

declare global {
  interface HTMLElementTagNameMap {
    "wa-number-input": WaNumberInput;
  }
}

`````

## Option 2

`````json
{"printWidth":100,"htmlWhitespaceSensitivity":"ignore"}
`````

### Diff

`````diff
===================================================================
--- prettier
+++ oxfmt
@@ -327,28 +327,26 @@
         <slot name="label">${this.label}</slot>
       </label>
 
       <div part="base" class="number-field">
-        ${
-          !this.withoutSteppers
-            ? html`
-                <button
-                  part="stepper stepper-decrement"
-                  class="stepper stepper-decrement"
-                  type="button"
-                  tabindex="-1"
-                  aria-label=${this.localize.term("decrement")}
-                  ?disabled=${this.disabled || this.readonly || this.isAtMin}
-                  @pointerdown=${this.handleStepperPointerDown}
-                  @pointerup=${(event: PointerEvent) => this.handleStepperPointerUp("down", event)}
-                >
-                  <slot name="decrement-icon">
-                    <wa-icon name="minus" library="system"></wa-icon>
-                  </slot>
-                </button>
-              `
-            : ""
-        }
+        ${!this.withoutSteppers
+          ? html`
+              <button
+                part="stepper stepper-decrement"
+                class="stepper stepper-decrement"
+                type="button"
+                tabindex="-1"
+                aria-label=${this.localize.term("decrement")}
+                ?disabled=${this.disabled || this.readonly || this.isAtMin}
+                @pointerdown=${this.handleStepperPointerDown}
+                @pointerup=${(event: PointerEvent) => this.handleStepperPointerUp("down", event)}
+              >
+                <slot name="decrement-icon">
+                  <wa-icon name="minus" library="system"></wa-icon>
+                </slot>
+              </button>
+            `
+          : ""}
 
         <slot name="start" part="start" class="start"></slot>
 
         <input
@@ -356,9 +354,12 @@
           id="input"
           class="control"
           type="number"
           inputmode=${ifDefined(this.inputmode)}
-          title=${this.title /* An empty title prevents browser validation tooltips from appearing on hover */}
+          title=${
+            this
+              .title /* An empty title prevents browser validation tooltips from appearing on hover */
+          }
           name=${ifDefined(this.name)}
           ?disabled=${this.disabled}
           ?readonly=${this.readonly}
           ?required=${this.required}
@@ -377,28 +378,26 @@
         />
 
         <slot name="end" part="end" class="end"></slot>
 
-        ${
-          !this.withoutSteppers
-            ? html`
-                <button
-                  part="stepper stepper-increment"
-                  class="stepper stepper-increment"
-                  type="button"
-                  tabindex="-1"
-                  aria-label=${this.localize.term("increment")}
-                  ?disabled=${this.disabled || this.readonly || this.isAtMax}
-                  @pointerdown=${this.handleStepperPointerDown}
-                  @pointerup=${(event: PointerEvent) => this.handleStepperPointerUp("up", event)}
-                >
-                  <slot name="increment-icon">
-                    <wa-icon name="plus" library="system"></wa-icon>
-                  </slot>
-                </button>
-              `
-            : ""
-        }
+        ${!this.withoutSteppers
+          ? html`
+              <button
+                part="stepper stepper-increment"
+                class="stepper stepper-increment"
+                type="button"
+                tabindex="-1"
+                aria-label=${this.localize.term("increment")}
+                ?disabled=${this.disabled || this.readonly || this.isAtMax}
+                @pointerdown=${this.handleStepperPointerDown}
+                @pointerup=${(event: PointerEvent) => this.handleStepperPointerUp("up", event)}
+              >
+                <slot name="increment-icon">
+                  <wa-icon name="plus" library="system"></wa-icon>
+                </slot>
+              </button>
+            `
+          : ""}
       </div>
 
       <slot
         id="hint"

`````

### Actual (oxfmt)

`````ts
import { html, type PropertyValues } from "lit";
import { customElement, property, query, state } from "lit/decorators.js";
import { classMap } from "lit/directives/class-map.js";
import { ifDefined } from "lit/directives/if-defined.js";
import { live } from "lit/directives/live.js";
import { warnDeprecatedSize } from "../../internal/size.js";
import { HasSlotController } from "../../internal/slot.js";
import { submitOnEnter } from "../../internal/submit-on-enter.js";
import { MirrorValidator } from "../../internal/validators/mirror-validator.js";
import { watch } from "../../internal/watch.js";
import { WebAwesomeFormAssociatedElement } from "../../internal/webawesome-form-associated-element.js";
import formControlStyles from "../../styles/component/form-control.styles.js";
import sizeStyles from "../../styles/component/size.styles.js";
import { LocalizeController } from "../../utilities/localize.js";
import "../icon/icon.js";
import styles from "./number-input.styles.js";

/**
 * @summary Number inputs let users enter and edit numeric values, with optional stepper buttons for incrementing and
 *  decrementing. Use them for quantities, measurements, and other numeric form fields.
 * @documentation https://webawesome.com/docs/components/number-input
 * @status stable
 * @since 3.2
 *
 * @dependency wa-icon
 *
 * @slot label - The input's label. Alternatively, you can use the `label` attribute.
 * @slot start - An element, such as `<wa-icon>`, placed at the start of the input control.
 * @slot end - An element, such as `<wa-icon>`, placed at the end of the input control (before steppers).
 * @slot increment-icon - An icon to use in lieu of the default increment icon.
 * @slot decrement-icon - An icon to use in lieu of the default decrement icon.
 * @slot hint - Text that describes how to use the input. Alternatively, you can use the `hint` attribute.
 *
 * @event blur - Emitted when the control loses focus.
 * @event change - Emitted when an alteration to the control's value is committed by the user.
 * @event focus - Emitted when the control gains focus.
 * @event input - Emitted when the control receives input.
 * @event beforeinput - Emitted before the value changes. Can be cancelled with `event.preventDefault()` to prevent the
 *  value from changing.
 * @event wa-invalid - Emitted when the form control has been checked for validity and its constraints aren't satisfied.
 *
 * @csspart label - The label element.
 * @csspart form-control-label - Alias for the label element.
 * @csspart hint - The hint element.
 * @csspart base - The wrapper containing the input and steppers.
 * @csspart input - The internal `<input>` control.
 * @csspart start - The container that wraps the `start` slot.
 * @csspart end - The container that wraps the `end` slot.
 * @csspart stepper - Both stepper buttons (for shared styling).
 * @csspart stepper-increment - The increment (+) button on the end side.
 * @csspart stepper-decrement - The decrement (-) button on the start side.
 *
 * @cssstate blank - The input is empty.
 * @cssstate focused - The input has focus.
 */
@customElement("wa-number-input")
export default class WaNumberInput extends WebAwesomeFormAssociatedElement {
  static css = [sizeStyles, formControlStyles, styles];

  static shadowRootOptions = {
    ...WebAwesomeFormAssociatedElement.shadowRootOptions,
    delegatesFocus: true,
  };

  static get validators() {
    return [...super.validators, MirrorValidator()];
  }

  assumeInteractionOn = ["blur", "input"];
  private readonly hasSlotController = new HasSlotController(this, "hint", "label");
  private readonly localize = new LocalizeController(this);

  @query("input") input: HTMLInputElement;

  @property() title = ""; // make reactive to pass through

  private _value: string | null = null;

  /** The current value of the input, submitted as a name/value pair with form data. */
  get value() {
    if (this.valueHasChanged) {
      return this._value;
    }

    return this._value ?? this.defaultValue;
  }

  @state()
  set value(val: string | null) {
    if (this._value === val) {
      return;
    }

    this.valueHasChanged = true;
    this._value = val;
  }

  /** The default value of the form control. Primarily used for resetting the form control. */
  @property({ attribute: "value", reflect: true }) defaultValue: string | null =
    this.getAttribute("value") || null;

  /** The input's size. */
  @property({ reflect: true }) size: "xs" | "s" | "m" | "l" | "xl" | "small" | "medium" | "large" =
    "m";

  @watch("size")
  handleSizeChange() {
    warnDeprecatedSize(this.localName, this.size);
  }

  /** The input's visual appearance. */
  @property({ reflect: true }) appearance: "filled" | "outlined" | "filled-outlined" = "outlined";

  /** Draws a pill-style input with rounded edges. */
  @property({ type: Boolean, reflect: true }) pill = false;

  /** The input's label. If you need to display HTML, use the `label` slot instead. */
  @property() label = "";

  /** The input's hint. If you need to display HTML, use the `hint` slot instead. */
  @property({ attribute: "hint" }) hint = "";

  /** Placeholder text to show as a hint when the input is empty. */
  @property() placeholder = "";

  /** Makes the input readonly. */
  @property({ type: Boolean, reflect: true }) readonly = false;

  /** Makes the input a required field. */
  @property({ type: Boolean, reflect: true }) required = false;

  /** The input's minimum value. */
  @property({ type: Number }) min: number;

  /** The input's maximum value. */
  @property({ type: Number }) max: number;

  /**
   * Specifies the granularity that the value must adhere to, or the special value `any` which means no stepping is
   * implied, allowing any numeric value.
   */
  @property() step: number | "any" = 1;

  /** Hides the increment/decrement stepper buttons. */
  @property({ attribute: "without-steppers", type: Boolean }) withoutSteppers = false;

  /**
   * Specifies what permission the browser has to provide assistance in filling out form field values. Refer to
   * [this page on MDN](https://developer.mozilla.org/en-US/docs/Web/HTML/Attributes/autocomplete) for available values.
   */
  @property() autocomplete: string;

  /** Indicates that the input should receive focus on page load. */
  @property({ type: Boolean }) autofocus: boolean;

  /** Used to customize the label or icon of the Enter key on virtual keyboards. */
  @property() enterkeyhint: "enter" | "done" | "go" | "next" | "previous" | "search" | "send";

  /**
   * Tells the browser what type of data will be entered by the user, allowing it to display the appropriate virtual
   * keyboard on supportive devices.
   */
  @property() inputmode: "numeric" | "decimal" = "numeric";

  /**
   * Only required for SSR. Set to `true` if you're slotting in a `label` element so the server-rendered markup
   * includes the label before the component hydrates on the client.
   */
  @property({ attribute: "with-label", type: Boolean }) withLabel = false;

  /**
   * Only required for SSR. Set to `true` if you're slotting in a `hint` element so the server-rendered markup
   * includes the hint before the component hydrates on the client.
   */
  @property({ attribute: "with-hint", type: Boolean }) withHint = false;

  /** Returns true if the value is at or below the minimum. */
  private get isAtMin(): boolean {
    if (this.min === undefined) return false;
    const numValue = parseFloat(this.value || "");
    return !isNaN(numValue) && numValue <= this.min;
  }

  /** Returns true if the value is at or above the maximum. */
  private get isAtMax(): boolean {
    if (this.max === undefined) return false;
    const numValue = parseFloat(this.value || "");
    return !isNaN(numValue) && numValue >= this.max;
  }

  private handleChange(event: Event) {
    this.value = this.input.value;

    this.relayNativeEvent(event, { bubbles: true, composed: true });
  }

  private handleInput() {
    this.value = this.input.value;
  }

  private handleKeyDown(event: KeyboardEvent) {
    submitOnEnter(event, this);

    // Sync value after arrow key changes
    if (event.key === "ArrowUp" || event.key === "ArrowDown") {
      requestAnimationFrame(() => {
        if (this.value !== this.input.value) {
          this.value = this.input.value;
        }
      });
    }
  }

  private handleStepperPointerUp(direction: "up" | "down", event: PointerEvent) {
    if (this.disabled || this.readonly) return;

    const beforeInputEvent = new InputEvent("beforeinput", {
      bubbles: true,
      cancelable: true,
      composed: true,
    });
    this.dispatchEvent(beforeInputEvent);
    if (beforeInputEvent.defaultPrevented) return;

    if (direction === "up") {
      this.input.stepUp();
    } else {
      this.input.stepDown();
    }

    if (this.value !== this.input.value) {
      this.value = this.input.value;
    }

    this.dispatchEvent(new InputEvent("input", { bubbles: true, composed: true }));
    this.dispatchEvent(new Event("change", { bubbles: true, composed: true }));

    // Avoid focusing the input on touch to prevent the virtual keyboard from showing
    if (event.pointerType !== "touch") {
      this.input.focus();
    }
  }

  private handleStepperPointerDown(event: PointerEvent) {
    // Avoid focusing the input on touch to prevent the virtual keyboard from showing
    if (event.pointerType === "touch") return;

    event.preventDefault();
    this.input.focus();
  }

  updated(changedProperties: PropertyValues<this>) {
    super.updated(changedProperties);

    if (changedProperties.has("value") || changedProperties.has("defaultValue")) {
      // The browser sanitizes invalid numeric input to an empty string. Mirror that behavior so `value` stays
      // consistent with the native input (e.g. setting `"abc"` resolves to `""`).
      if (this.input && this.value && this.input.value !== this.value) {
        this._value = this.input.value;
      }

      this.customStates.set("blank", !this.value);
    }
  }

  @watch("step", { waitUntilFirstUpdate: true })
  handleStepChange() {
    // If step changes, the value may become invalid so we need to recheck after the update. We set the new step
    // imperatively so we don't have to wait for the next render to report the updated validity.
    this.input.step = String(this.step);
    this.updateValidity();
  }

  /** Sets focus on the input. */
  focus(options?: FocusOptions) {
    this.input.focus(options);
  }

  /** Removes focus from the input. */
  blur() {
    this.input.blur();
  }

  /** Selects all the text in the input. */
  select() {
    this.input.select();
  }

  /** Increments the value by the step amount. */
  stepUp() {
    this.input.stepUp();
    if (this.value !== this.input.value) {
      this.value = this.input.value;
    }
  }

  /** Decrements the value by the step amount. */
  stepDown() {
    this.input.stepDown();
    if (this.value !== this.input.value) {
      this.value = this.input.value;
    }
  }

  formResetCallback() {
    this.value = this.defaultValue;

    super.formResetCallback();
  }

  render() {
    const hasLabelSlot = this.hasUpdated ? this.hasSlotController.test("label") : this.withLabel;
    const hasHintSlot = this.hasUpdated ? this.hasSlotController.test("hint") : this.withHint;
    const hasLabel = this.label ? true : !!hasLabelSlot;
    const hasHint = this.hint ? true : !!hasHintSlot;

    return html`
      <label
        part="form-control-label label"
        class=${classMap({
          label: true,
          "has-label": hasLabel,
        })}
        for="input"
        aria-hidden=${hasLabel ? "false" : "true"}
      >
        <slot name="label">${this.label}</slot>
      </label>

      <div part="base" class="number-field">
        ${!this.withoutSteppers
          ? html`
              <button
                part="stepper stepper-decrement"
                class="stepper stepper-decrement"
                type="button"
                tabindex="-1"
                aria-label=${this.localize.term("decrement")}
                ?disabled=${this.disabled || this.readonly || this.isAtMin}
                @pointerdown=${this.handleStepperPointerDown}
                @pointerup=${(event: PointerEvent) => this.handleStepperPointerUp("down", event)}
              >
                <slot name="decrement-icon">
                  <wa-icon name="minus" library="system"></wa-icon>
                </slot>
              </button>
            `
          : ""}

        <slot name="start" part="start" class="start"></slot>

        <input
          part="input"
          id="input"
          class="control"
          type="number"
          inputmode=${ifDefined(this.inputmode)}
          title=${
            this
              .title /* An empty title prevents browser validation tooltips from appearing on hover */
          }
          name=${ifDefined(this.name)}
          ?disabled=${this.disabled}
          ?readonly=${this.readonly}
          ?required=${this.required}
          placeholder=${ifDefined(this.placeholder)}
          min=${ifDefined(this.min)}
          max=${ifDefined(this.max)}
          step=${ifDefined(this.step as number)}
          .value=${live(this.value ?? "")}
          autocomplete=${ifDefined(this.autocomplete)}
          ?autofocus=${this.autofocus}
          enterkeyhint=${ifDefined(this.enterkeyhint)}
          aria-describedby="hint"
          @change=${this.handleChange}
          @input=${this.handleInput}
          @keydown=${this.handleKeyDown}
        />

        <slot name="end" part="end" class="end"></slot>

        ${!this.withoutSteppers
          ? html`
              <button
                part="stepper stepper-increment"
                class="stepper stepper-increment"
                type="button"
                tabindex="-1"
                aria-label=${this.localize.term("increment")}
                ?disabled=${this.disabled || this.readonly || this.isAtMax}
                @pointerdown=${this.handleStepperPointerDown}
                @pointerup=${(event: PointerEvent) => this.handleStepperPointerUp("up", event)}
              >
                <slot name="increment-icon">
                  <wa-icon name="plus" library="system"></wa-icon>
                </slot>
              </button>
            `
          : ""}
      </div>

      <slot
        id="hint"
        part="hint"
        name="hint"
        class=${classMap({
          "has-slotted": hasHint,
        })}
        aria-hidden=${hasHint ? "false" : "true"}
      >
        ${this.hint}
      </slot>
    `;
  }
}

// The change-in-update warning is required for this component because the form-associated base class calls
// updateValidity() in firstUpdated(), which triggers requestUpdate('validity') to sync the validation state after the
// first render when the validation target is available. Additionally, HasSlotController triggers requestUpdate() on
// initial slotchange events. See https://lit.dev/docs/tools/development/#development-build-runtime-warnings
WaNumberInput.disableWarning?.("change-in-update");

declare global {
  interface HTMLElementTagNameMap {
    "wa-number-input": WaNumberInput;
  }
}

`````

### Expected (prettier)

`````ts
import { html, type PropertyValues } from "lit";
import { customElement, property, query, state } from "lit/decorators.js";
import { classMap } from "lit/directives/class-map.js";
import { ifDefined } from "lit/directives/if-defined.js";
import { live } from "lit/directives/live.js";
import { warnDeprecatedSize } from "../../internal/size.js";
import { HasSlotController } from "../../internal/slot.js";
import { submitOnEnter } from "../../internal/submit-on-enter.js";
import { MirrorValidator } from "../../internal/validators/mirror-validator.js";
import { watch } from "../../internal/watch.js";
import { WebAwesomeFormAssociatedElement } from "../../internal/webawesome-form-associated-element.js";
import formControlStyles from "../../styles/component/form-control.styles.js";
import sizeStyles from "../../styles/component/size.styles.js";
import { LocalizeController } from "../../utilities/localize.js";
import "../icon/icon.js";
import styles from "./number-input.styles.js";

/**
 * @summary Number inputs let users enter and edit numeric values, with optional stepper buttons for incrementing and
 *  decrementing. Use them for quantities, measurements, and other numeric form fields.
 * @documentation https://webawesome.com/docs/components/number-input
 * @status stable
 * @since 3.2
 *
 * @dependency wa-icon
 *
 * @slot label - The input's label. Alternatively, you can use the `label` attribute.
 * @slot start - An element, such as `<wa-icon>`, placed at the start of the input control.
 * @slot end - An element, such as `<wa-icon>`, placed at the end of the input control (before steppers).
 * @slot increment-icon - An icon to use in lieu of the default increment icon.
 * @slot decrement-icon - An icon to use in lieu of the default decrement icon.
 * @slot hint - Text that describes how to use the input. Alternatively, you can use the `hint` attribute.
 *
 * @event blur - Emitted when the control loses focus.
 * @event change - Emitted when an alteration to the control's value is committed by the user.
 * @event focus - Emitted when the control gains focus.
 * @event input - Emitted when the control receives input.
 * @event beforeinput - Emitted before the value changes. Can be cancelled with `event.preventDefault()` to prevent the
 *  value from changing.
 * @event wa-invalid - Emitted when the form control has been checked for validity and its constraints aren't satisfied.
 *
 * @csspart label - The label element.
 * @csspart form-control-label - Alias for the label element.
 * @csspart hint - The hint element.
 * @csspart base - The wrapper containing the input and steppers.
 * @csspart input - The internal `<input>` control.
 * @csspart start - The container that wraps the `start` slot.
 * @csspart end - The container that wraps the `end` slot.
 * @csspart stepper - Both stepper buttons (for shared styling).
 * @csspart stepper-increment - The increment (+) button on the end side.
 * @csspart stepper-decrement - The decrement (-) button on the start side.
 *
 * @cssstate blank - The input is empty.
 * @cssstate focused - The input has focus.
 */
@customElement("wa-number-input")
export default class WaNumberInput extends WebAwesomeFormAssociatedElement {
  static css = [sizeStyles, formControlStyles, styles];

  static shadowRootOptions = {
    ...WebAwesomeFormAssociatedElement.shadowRootOptions,
    delegatesFocus: true,
  };

  static get validators() {
    return [...super.validators, MirrorValidator()];
  }

  assumeInteractionOn = ["blur", "input"];
  private readonly hasSlotController = new HasSlotController(this, "hint", "label");
  private readonly localize = new LocalizeController(this);

  @query("input") input: HTMLInputElement;

  @property() title = ""; // make reactive to pass through

  private _value: string | null = null;

  /** The current value of the input, submitted as a name/value pair with form data. */
  get value() {
    if (this.valueHasChanged) {
      return this._value;
    }

    return this._value ?? this.defaultValue;
  }

  @state()
  set value(val: string | null) {
    if (this._value === val) {
      return;
    }

    this.valueHasChanged = true;
    this._value = val;
  }

  /** The default value of the form control. Primarily used for resetting the form control. */
  @property({ attribute: "value", reflect: true }) defaultValue: string | null =
    this.getAttribute("value") || null;

  /** The input's size. */
  @property({ reflect: true }) size: "xs" | "s" | "m" | "l" | "xl" | "small" | "medium" | "large" =
    "m";

  @watch("size")
  handleSizeChange() {
    warnDeprecatedSize(this.localName, this.size);
  }

  /** The input's visual appearance. */
  @property({ reflect: true }) appearance: "filled" | "outlined" | "filled-outlined" = "outlined";

  /** Draws a pill-style input with rounded edges. */
  @property({ type: Boolean, reflect: true }) pill = false;

  /** The input's label. If you need to display HTML, use the `label` slot instead. */
  @property() label = "";

  /** The input's hint. If you need to display HTML, use the `hint` slot instead. */
  @property({ attribute: "hint" }) hint = "";

  /** Placeholder text to show as a hint when the input is empty. */
  @property() placeholder = "";

  /** Makes the input readonly. */
  @property({ type: Boolean, reflect: true }) readonly = false;

  /** Makes the input a required field. */
  @property({ type: Boolean, reflect: true }) required = false;

  /** The input's minimum value. */
  @property({ type: Number }) min: number;

  /** The input's maximum value. */
  @property({ type: Number }) max: number;

  /**
   * Specifies the granularity that the value must adhere to, or the special value `any` which means no stepping is
   * implied, allowing any numeric value.
   */
  @property() step: number | "any" = 1;

  /** Hides the increment/decrement stepper buttons. */
  @property({ attribute: "without-steppers", type: Boolean }) withoutSteppers = false;

  /**
   * Specifies what permission the browser has to provide assistance in filling out form field values. Refer to
   * [this page on MDN](https://developer.mozilla.org/en-US/docs/Web/HTML/Attributes/autocomplete) for available values.
   */
  @property() autocomplete: string;

  /** Indicates that the input should receive focus on page load. */
  @property({ type: Boolean }) autofocus: boolean;

  /** Used to customize the label or icon of the Enter key on virtual keyboards. */
  @property() enterkeyhint: "enter" | "done" | "go" | "next" | "previous" | "search" | "send";

  /**
   * Tells the browser what type of data will be entered by the user, allowing it to display the appropriate virtual
   * keyboard on supportive devices.
   */
  @property() inputmode: "numeric" | "decimal" = "numeric";

  /**
   * Only required for SSR. Set to `true` if you're slotting in a `label` element so the server-rendered markup
   * includes the label before the component hydrates on the client.
   */
  @property({ attribute: "with-label", type: Boolean }) withLabel = false;

  /**
   * Only required for SSR. Set to `true` if you're slotting in a `hint` element so the server-rendered markup
   * includes the hint before the component hydrates on the client.
   */
  @property({ attribute: "with-hint", type: Boolean }) withHint = false;

  /** Returns true if the value is at or below the minimum. */
  private get isAtMin(): boolean {
    if (this.min === undefined) return false;
    const numValue = parseFloat(this.value || "");
    return !isNaN(numValue) && numValue <= this.min;
  }

  /** Returns true if the value is at or above the maximum. */
  private get isAtMax(): boolean {
    if (this.max === undefined) return false;
    const numValue = parseFloat(this.value || "");
    return !isNaN(numValue) && numValue >= this.max;
  }

  private handleChange(event: Event) {
    this.value = this.input.value;

    this.relayNativeEvent(event, { bubbles: true, composed: true });
  }

  private handleInput() {
    this.value = this.input.value;
  }

  private handleKeyDown(event: KeyboardEvent) {
    submitOnEnter(event, this);

    // Sync value after arrow key changes
    if (event.key === "ArrowUp" || event.key === "ArrowDown") {
      requestAnimationFrame(() => {
        if (this.value !== this.input.value) {
          this.value = this.input.value;
        }
      });
    }
  }

  private handleStepperPointerUp(direction: "up" | "down", event: PointerEvent) {
    if (this.disabled || this.readonly) return;

    const beforeInputEvent = new InputEvent("beforeinput", {
      bubbles: true,
      cancelable: true,
      composed: true,
    });
    this.dispatchEvent(beforeInputEvent);
    if (beforeInputEvent.defaultPrevented) return;

    if (direction === "up") {
      this.input.stepUp();
    } else {
      this.input.stepDown();
    }

    if (this.value !== this.input.value) {
      this.value = this.input.value;
    }

    this.dispatchEvent(new InputEvent("input", { bubbles: true, composed: true }));
    this.dispatchEvent(new Event("change", { bubbles: true, composed: true }));

    // Avoid focusing the input on touch to prevent the virtual keyboard from showing
    if (event.pointerType !== "touch") {
      this.input.focus();
    }
  }

  private handleStepperPointerDown(event: PointerEvent) {
    // Avoid focusing the input on touch to prevent the virtual keyboard from showing
    if (event.pointerType === "touch") return;

    event.preventDefault();
    this.input.focus();
  }

  updated(changedProperties: PropertyValues<this>) {
    super.updated(changedProperties);

    if (changedProperties.has("value") || changedProperties.has("defaultValue")) {
      // The browser sanitizes invalid numeric input to an empty string. Mirror that behavior so `value` stays
      // consistent with the native input (e.g. setting `"abc"` resolves to `""`).
      if (this.input && this.value && this.input.value !== this.value) {
        this._value = this.input.value;
      }

      this.customStates.set("blank", !this.value);
    }
  }

  @watch("step", { waitUntilFirstUpdate: true })
  handleStepChange() {
    // If step changes, the value may become invalid so we need to recheck after the update. We set the new step
    // imperatively so we don't have to wait for the next render to report the updated validity.
    this.input.step = String(this.step);
    this.updateValidity();
  }

  /** Sets focus on the input. */
  focus(options?: FocusOptions) {
    this.input.focus(options);
  }

  /** Removes focus from the input. */
  blur() {
    this.input.blur();
  }

  /** Selects all the text in the input. */
  select() {
    this.input.select();
  }

  /** Increments the value by the step amount. */
  stepUp() {
    this.input.stepUp();
    if (this.value !== this.input.value) {
      this.value = this.input.value;
    }
  }

  /** Decrements the value by the step amount. */
  stepDown() {
    this.input.stepDown();
    if (this.value !== this.input.value) {
      this.value = this.input.value;
    }
  }

  formResetCallback() {
    this.value = this.defaultValue;

    super.formResetCallback();
  }

  render() {
    const hasLabelSlot = this.hasUpdated ? this.hasSlotController.test("label") : this.withLabel;
    const hasHintSlot = this.hasUpdated ? this.hasSlotController.test("hint") : this.withHint;
    const hasLabel = this.label ? true : !!hasLabelSlot;
    const hasHint = this.hint ? true : !!hasHintSlot;

    return html`
      <label
        part="form-control-label label"
        class=${classMap({
          label: true,
          "has-label": hasLabel,
        })}
        for="input"
        aria-hidden=${hasLabel ? "false" : "true"}
      >
        <slot name="label">${this.label}</slot>
      </label>

      <div part="base" class="number-field">
        ${
          !this.withoutSteppers
            ? html`
                <button
                  part="stepper stepper-decrement"
                  class="stepper stepper-decrement"
                  type="button"
                  tabindex="-1"
                  aria-label=${this.localize.term("decrement")}
                  ?disabled=${this.disabled || this.readonly || this.isAtMin}
                  @pointerdown=${this.handleStepperPointerDown}
                  @pointerup=${(event: PointerEvent) => this.handleStepperPointerUp("down", event)}
                >
                  <slot name="decrement-icon">
                    <wa-icon name="minus" library="system"></wa-icon>
                  </slot>
                </button>
              `
            : ""
        }

        <slot name="start" part="start" class="start"></slot>

        <input
          part="input"
          id="input"
          class="control"
          type="number"
          inputmode=${ifDefined(this.inputmode)}
          title=${this.title /* An empty title prevents browser validation tooltips from appearing on hover */}
          name=${ifDefined(this.name)}
          ?disabled=${this.disabled}
          ?readonly=${this.readonly}
          ?required=${this.required}
          placeholder=${ifDefined(this.placeholder)}
          min=${ifDefined(this.min)}
          max=${ifDefined(this.max)}
          step=${ifDefined(this.step as number)}
          .value=${live(this.value ?? "")}
          autocomplete=${ifDefined(this.autocomplete)}
          ?autofocus=${this.autofocus}
          enterkeyhint=${ifDefined(this.enterkeyhint)}
          aria-describedby="hint"
          @change=${this.handleChange}
          @input=${this.handleInput}
          @keydown=${this.handleKeyDown}
        />

        <slot name="end" part="end" class="end"></slot>

        ${
          !this.withoutSteppers
            ? html`
                <button
                  part="stepper stepper-increment"
                  class="stepper stepper-increment"
                  type="button"
                  tabindex="-1"
                  aria-label=${this.localize.term("increment")}
                  ?disabled=${this.disabled || this.readonly || this.isAtMax}
                  @pointerdown=${this.handleStepperPointerDown}
                  @pointerup=${(event: PointerEvent) => this.handleStepperPointerUp("up", event)}
                >
                  <slot name="increment-icon">
                    <wa-icon name="plus" library="system"></wa-icon>
                  </slot>
                </button>
              `
            : ""
        }
      </div>

      <slot
        id="hint"
        part="hint"
        name="hint"
        class=${classMap({
          "has-slotted": hasHint,
        })}
        aria-hidden=${hasHint ? "false" : "true"}
      >
        ${this.hint}
      </slot>
    `;
  }
}

// The change-in-update warning is required for this component because the form-associated base class calls
// updateValidity() in firstUpdated(), which triggers requestUpdate('validity') to sync the validation state after the
// first render when the validation target is available. Additionally, HasSlotController triggers requestUpdate() on
// initial slotchange events. See https://lit.dev/docs/tools/development/#development-build-runtime-warnings
WaNumberInput.disableWarning?.("change-in-update");

declare global {
  interface HTMLElementTagNameMap {
    "wa-number-input": WaNumberInput;
  }
}

`````
