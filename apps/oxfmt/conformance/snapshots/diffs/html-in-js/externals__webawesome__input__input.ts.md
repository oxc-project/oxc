# externals/webawesome/input/input.ts

## Option 1

`````json
{"printWidth":80}
`````

### Diff

`````diff
===================================================================
--- prettier
+++ oxfmt
@@ -118,18 +118,27 @@
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
 
@@ -185,9 +194,14 @@
   @property() step: number | "any";
 
   /** Controls whether and how text input is automatically capitalized as it is entered by the user. */
   @property() autocapitalize:
-    "off" | "none" | "on" | "sentences" | "words" | "characters";
+    | "off"
+    | "none"
+    | "on"
+    | "sentences"
+    | "words"
+    | "characters";
 
   /**
    * Indicates whether the browser's autocorrect feature is on or off. When set as an attribute, use `"off"` or `"on"`.
    * When set as a property, use `true` or `false`.
@@ -211,9 +225,15 @@
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
 
   /** Enables spell checking on the input. */
   @property({
     type: Boolean,
@@ -442,10 +462,15 @@
         <input
           part="input"
           id="input"
           class="control"
-          type=${this.type === "password" && this.passwordVisible ? "text" : this.type}
-          title=${this.title /* An empty title prevents browser validation tooltips from appearing on hover */}
+          type=${this.type === "password" && this.passwordVisible
+            ? "text"
+            : this.type}
+          title=${
+            this
+              .title /* An empty title prevents browser validation tooltips from appearing on hover */
+          }
           name=${ifDefined(this.name)}
           ?disabled=${this.disabled}
           ?readonly=${this.readonly}
           ?required=${this.required}
@@ -469,66 +494,62 @@
           @input=${this.handleInput}
           @keydown=${this.handleKeyDown}
         />
 
-        ${
-          isClearIconVisible
-            ? html`
-                <button
-                  part="clear-button"
-                  class="clear"
-                  type="button"
-                  aria-label=${this.localize.term("clearEntry")}
-                  @click=${this.handleClearClick}
-                  tabindex="-1"
-                >
-                  <slot name="clear-icon">
-                    <wa-icon
-                      name="circle-xmark"
-                      library="system"
-                      variant="regular"
-                    ></wa-icon>
-                  </slot>
-                </button>
-              `
-            : ""
-        }
-        ${
-          this.passwordToggle && !this.disabled
-            ? html`
-                <button
-                  part="password-toggle-button"
-                  class="password-toggle"
-                  type="button"
-                  aria-label=${this.localize.term(this.passwordVisible ? "hidePassword" : "showPassword")}
-                  @click=${this.handlePasswordToggle}
-                  tabindex="-1"
-                >
-                  ${
-                  !this.passwordVisible
-                    ? html`
-                        <slot name="show-password-icon">
-                          <wa-icon
-                            name="eye"
-                            library="system"
-                            variant="regular"
-                          ></wa-icon>
-                        </slot>
-                      `
-                    : html`
-                        <slot name="hide-password-icon">
-                          <wa-icon
-                            name="eye-slash"
-                            library="system"
-                            variant="regular"
-                          ></wa-icon>
-                        </slot>
-                      `
-                }
-                </button>
-              `
-            : ""
-        }
+        ${isClearIconVisible
+          ? html`
+              <button
+                part="clear-button"
+                class="clear"
+                type="button"
+                aria-label=${this.localize.term("clearEntry")}
+                @click=${this.handleClearClick}
+                tabindex="-1"
+              >
+                <slot name="clear-icon">
+                  <wa-icon
+                    name="circle-xmark"
+                    library="system"
+                    variant="regular"
+                  ></wa-icon>
+                </slot>
+              </button>
+            `
+          : ""}
+        ${this.passwordToggle && !this.disabled
+          ? html`
+              <button
+                part="password-toggle-button"
+                class="password-toggle"
+                type="button"
+                aria-label=${this.localize.term(
+                  this.passwordVisible ? "hidePassword" : "showPassword",
+                )}
+                @click=${this.handlePasswordToggle}
+                tabindex="-1"
+              >
+                ${!this.passwordVisible
+                  ? html`
+                      <slot name="show-password-icon">
+                        <wa-icon
+                          name="eye"
+                          library="system"
+                          variant="regular"
+                        ></wa-icon>
+                      </slot>
+                    `
+                  : html`
+                      <slot name="hide-password-icon">
+                        <wa-icon
+                          name="eye-slash"
+                          library="system"
+                          variant="regular"
+                        ></wa-icon>
+                      </slot>
+                    `}
+              </button>
+            `
+          : ""}
 
         <slot name="end" part="end" class="end"></slot>
       </div>
 

`````

### Actual (oxfmt)

`````ts
import { html, isServer, type PropertyValues } from "lit";
import { customElement, property, query, state } from "lit/decorators.js";
import { classMap } from "lit/directives/class-map.js";
import { ifDefined } from "lit/directives/if-defined.js";
import { live } from "lit/directives/live.js";
import { WaClearEvent } from "../../events/clear.js";
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
import styles from "./input.styles.js";

/**
 * @summary Inputs collect single-line data from the user, such as text, numbers, email addresses, and passwords. They
 *  support labels, hints, validation, and prefix or suffix slots.
 * @documentation https://webawesome.com/docs/components/input
 * @status stable
 * @since 2.0
 *
 * @dependency wa-icon
 *
 * @slot label - The input's label. Alternatively, you can use the `label` attribute.
 * @slot start - An element, such as `<wa-icon>`, placed at the start of the input control.
 * @slot end - An element, such as `<wa-icon>`, placed at the end of the input control.
 * @slot clear-icon - An icon to use in lieu of the default clear icon.
 * @slot show-password-icon - An icon to use in lieu of the default show password icon.
 * @slot hide-password-icon - An icon to use in lieu of the default hide password icon.
 * @slot hint - Text that describes how to use the input. Alternatively, you can use the `hint` attribute.
 *
 * @event blur - Emitted when the control loses focus.
 * @event change - Emitted when an alteration to the control's value is committed by the user.
 * @event focus - Emitted when the control gains focus.
 * @event input - Emitted when the control receives input.
 * @event wa-clear - Emitted when the clear button is activated.
 * @event wa-invalid - Emitted when the form control has been checked for validity and its constraints aren't satisfied.
 *
 * @csspart label - The label
 * @csspart hint - The hint's wrapper.
 * @csspart base - The wrapper being rendered as an input
 * @csspart input - The internal `<input>` control.
 * @csspart start - The container that wraps the `start` slot.
 * @csspart clear-button - The clear button.
 * @csspart password-toggle-button - The password toggle button.
 * @csspart end - The container that wraps the `end` slot.
 *
 * @cssstate blank - The input is empty.
 */
@customElement("wa-input")
export default class WaInput extends WebAwesomeFormAssociatedElement {
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

  /**
   * The type of input. Works the same as a native `<input>` element, but only a subset of types are supported. Defaults
   * to `text`.
   */
  @property({ reflect: true }) type:
    | "date"
    | "datetime-local"
    | "email"
    | "number"
    | "password"
    | "search"
    | "tel"
    | "text"
    | "time"
    | "url" = "text";

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

  /** Adds a clear button when the input is not empty. */
  @property({ attribute: "with-clear", type: Boolean }) withClear = false;

  /** Placeholder text to show as a hint when the input is empty. */
  @property() placeholder = "";

  /** Makes the input readonly. */
  @property({ type: Boolean, reflect: true }) readonly = false;

  /** Adds a button to toggle the password's visibility. Only applies to password types. */
  @property({ attribute: "password-toggle", type: Boolean }) passwordToggle =
    false;

  /** Determines whether or not the password is currently visible. Only applies to password input types. */
  @property({ attribute: "password-visible", type: Boolean }) passwordVisible =
    false;

  /** Hides the browser's built-in increment/decrement spin buttons for number inputs. */
  @property({ attribute: "without-spin-buttons", type: Boolean, reflect: true })
  withoutSpinButtons = false;

  /** Makes the input a required field. */
  @property({ type: Boolean, reflect: true }) required = false;

  /** A regular expression pattern to validate input against. */
  @property() pattern: string;

  /** The minimum length of input that will be considered valid. */
  @property({ type: Number }) minlength: number;

  /** The maximum length of input that will be considered valid. */
  @property({ type: Number }) maxlength: number;

  /** The input's minimum value. Only applies to date and number input types. */
  @property() min: number | string;

  /** The input's maximum value. Only applies to date and number input types. */
  @property() max: number | string;

  /**
   * Specifies the granularity that the value must adhere to, or the special value `any` which means no stepping is
   * implied, allowing any numeric value. Only applies to date and number input types.
   */
  @property() step: number | "any";

  /** Controls whether and how text input is automatically capitalized as it is entered by the user. */
  @property() autocapitalize:
    | "off"
    | "none"
    | "on"
    | "sentences"
    | "words"
    | "characters";

  /**
   * Indicates whether the browser's autocorrect feature is on or off. When set as an attribute, use `"off"` or `"on"`.
   * When set as a property, use `true` or `false`.
   */
  @property({
    type: Boolean,
    converter: {
      fromAttribute: (value) => (!value || value === "off" ? false : true),
      toAttribute: (value) => (value ? "on" : "off"),
    },
  })
  declare autocorrect: boolean;

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

  /** Enables spell checking on the input. */
  @property({
    type: Boolean,
    converter: {
      // Allow "true|false" attribute values but keep the property boolean
      fromAttribute: (value) => (!value || value === "false" ? false : true),
      toAttribute: (value) => (value ? "true" : "false"),
    },
  })
  spellcheck = true;

  /**
   * Tells the browser what type of data will be entered by the user, allowing it to display the appropriate virtual
   * keyboard on supportive devices.
   */
  @property() inputmode:
    | "none"
    | "text"
    | "decimal"
    | "numeric"
    | "tel"
    | "search"
    | "email"
    | "url";

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

  private handleChange(event: Event) {
    this.value = this.input.value;

    this.relayNativeEvent(event, { bubbles: true, composed: true });
  }

  private handleClearClick(event: MouseEvent) {
    event.preventDefault();

    if (this.value !== "") {
      this.value = "";

      this.updateComplete.then(() => {
        this.dispatchEvent(new WaClearEvent());
        this.dispatchEvent(
          new InputEvent("input", { bubbles: true, composed: true }),
        );
        this.dispatchEvent(
          new Event("change", { bubbles: true, composed: true }),
        );
      });
    }

    this.input.focus();
  }

  private handleInput() {
    this.value = this.input.value;
  }

  private handleKeyDown(event: KeyboardEvent) {
    submitOnEnter(event, this);
  }

  private handlePasswordToggle() {
    this.passwordVisible = !this.passwordVisible;
  }

  updated(changedProperties: PropertyValues<this>) {
    super.updated(changedProperties);

    if (
      changedProperties.has("value") ||
      changedProperties.has("defaultValue") ||
      changedProperties.has("type")
    ) {
      // Types where the browser sanitizes invalid input to an empty string. Mirror that behavior so `value` stays
      // consistent with the native input (e.g. setting `"abc"` on `type="number"` resolves to `""`).
      const sanitizingTypes = ["number", "date", "time", "datetime-local"];
      if (
        this.input &&
        sanitizingTypes.includes(this.type) &&
        this.value &&
        this.input.value !== this.value
      ) {
        this._value = this.input.value;
      }

      this.customStates.set("blank", !this.value);
      this.updateValidity();
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

  /** Sets the start and end positions of the text selection (0-based). */
  setSelectionRange(
    selectionStart: number,
    selectionEnd: number,
    selectionDirection: "forward" | "backward" | "none" = "none",
  ) {
    this.input.setSelectionRange(
      selectionStart,
      selectionEnd,
      selectionDirection,
    );
  }

  /** Replaces a range of text with a new string. */
  setRangeText(
    replacement: string,
    start?: number,
    end?: number,
    selectMode: "select" | "start" | "end" | "preserve" = "preserve",
  ) {
    const selectionStart = start ?? this.input.selectionStart!;
    const selectionEnd = end ?? this.input.selectionEnd!;

    this.input.setRangeText(
      replacement,
      selectionStart,
      selectionEnd,
      selectMode,
    );

    if (this.value !== this.input.value) {
      this.value = this.input.value;
    }
  }

  /** Displays the browser picker for an input element (only works if the browser supports it for the input type). */
  showPicker() {
    if ("showPicker" in HTMLInputElement.prototype) {
      this.input.showPicker();
    }
  }

  /** Increments the value of a numeric input type by the value of the step attribute. */
  stepUp() {
    this.input.stepUp();
    if (this.value !== this.input.value) {
      this.value = this.input.value;
    }
  }

  /** Decrements the value of a numeric input type by the value of the step attribute. */
  stepDown() {
    this.input.stepDown();
    if (this.value !== this.input.value) {
      this.value = this.input.value;
    }
  }

  formResetCallback() {
    this.value = null;

    if (this.input) {
      // Fixes https://github.com/shoelace-style/webawesome/issues/1640 where resetting an input would leave the "live" vlaue in place on the input in the shadow dom. This fixed that by manually forcing the value.
      // @ts-expect-error
      this.input.value = this.value;
    }

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
    const hasClearIcon = this.withClear && !this.disabled && !this.readonly;
    const isClearIconVisible =
      // prevents hydration mismatch errors.
      (isServer || this.hasUpdated) &&
      hasClearIcon &&
      (typeof this.value === "number" || (this.value && this.value.length > 0));

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

      <div part="base" class="text-field">
        <slot name="start" part="start" class="start"></slot>

        <input
          part="input"
          id="input"
          class="control"
          type=${this.type === "password" && this.passwordVisible
            ? "text"
            : this.type}
          title=${
            this
              .title /* An empty title prevents browser validation tooltips from appearing on hover */
          }
          name=${ifDefined(this.name)}
          ?disabled=${this.disabled}
          ?readonly=${this.readonly}
          ?required=${this.required}
          placeholder=${ifDefined(this.placeholder)}
          minlength=${ifDefined(this.minlength)}
          maxlength=${ifDefined(this.maxlength)}
          min=${ifDefined(this.min)}
          max=${ifDefined(this.max)}
          step=${ifDefined(this.step as number)}
          .value=${live(this.value ?? "")}
          autocapitalize=${ifDefined(this.autocapitalize)}
          autocomplete=${ifDefined(this.autocomplete)}
          autocorrect=${this.autocorrect ? "on" : "off"}
          ?autofocus=${this.autofocus}
          spellcheck=${this.spellcheck}
          pattern=${ifDefined(this.pattern)}
          enterkeyhint=${ifDefined(this.enterkeyhint)}
          inputmode=${ifDefined(this.inputmode)}
          aria-describedby="hint"
          @change=${this.handleChange}
          @input=${this.handleInput}
          @keydown=${this.handleKeyDown}
        />

        ${isClearIconVisible
          ? html`
              <button
                part="clear-button"
                class="clear"
                type="button"
                aria-label=${this.localize.term("clearEntry")}
                @click=${this.handleClearClick}
                tabindex="-1"
              >
                <slot name="clear-icon">
                  <wa-icon
                    name="circle-xmark"
                    library="system"
                    variant="regular"
                  ></wa-icon>
                </slot>
              </button>
            `
          : ""}
        ${this.passwordToggle && !this.disabled
          ? html`
              <button
                part="password-toggle-button"
                class="password-toggle"
                type="button"
                aria-label=${this.localize.term(
                  this.passwordVisible ? "hidePassword" : "showPassword",
                )}
                @click=${this.handlePasswordToggle}
                tabindex="-1"
              >
                ${!this.passwordVisible
                  ? html`
                      <slot name="show-password-icon">
                        <wa-icon
                          name="eye"
                          library="system"
                          variant="regular"
                        ></wa-icon>
                      </slot>
                    `
                  : html`
                      <slot name="hide-password-icon">
                        <wa-icon
                          name="eye-slash"
                          library="system"
                          variant="regular"
                        ></wa-icon>
                      </slot>
                    `}
              </button>
            `
          : ""}

        <slot name="end" part="end" class="end"></slot>
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
WaInput.disableWarning?.("change-in-update");

declare global {
  interface HTMLElementTagNameMap {
    "wa-input": WaInput;
  }
}

`````

### Expected (prettier)

`````ts
import { html, isServer, type PropertyValues } from "lit";
import { customElement, property, query, state } from "lit/decorators.js";
import { classMap } from "lit/directives/class-map.js";
import { ifDefined } from "lit/directives/if-defined.js";
import { live } from "lit/directives/live.js";
import { WaClearEvent } from "../../events/clear.js";
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
import styles from "./input.styles.js";

/**
 * @summary Inputs collect single-line data from the user, such as text, numbers, email addresses, and passwords. They
 *  support labels, hints, validation, and prefix or suffix slots.
 * @documentation https://webawesome.com/docs/components/input
 * @status stable
 * @since 2.0
 *
 * @dependency wa-icon
 *
 * @slot label - The input's label. Alternatively, you can use the `label` attribute.
 * @slot start - An element, such as `<wa-icon>`, placed at the start of the input control.
 * @slot end - An element, such as `<wa-icon>`, placed at the end of the input control.
 * @slot clear-icon - An icon to use in lieu of the default clear icon.
 * @slot show-password-icon - An icon to use in lieu of the default show password icon.
 * @slot hide-password-icon - An icon to use in lieu of the default hide password icon.
 * @slot hint - Text that describes how to use the input. Alternatively, you can use the `hint` attribute.
 *
 * @event blur - Emitted when the control loses focus.
 * @event change - Emitted when an alteration to the control's value is committed by the user.
 * @event focus - Emitted when the control gains focus.
 * @event input - Emitted when the control receives input.
 * @event wa-clear - Emitted when the clear button is activated.
 * @event wa-invalid - Emitted when the form control has been checked for validity and its constraints aren't satisfied.
 *
 * @csspart label - The label
 * @csspart hint - The hint's wrapper.
 * @csspart base - The wrapper being rendered as an input
 * @csspart input - The internal `<input>` control.
 * @csspart start - The container that wraps the `start` slot.
 * @csspart clear-button - The clear button.
 * @csspart password-toggle-button - The password toggle button.
 * @csspart end - The container that wraps the `end` slot.
 *
 * @cssstate blank - The input is empty.
 */
@customElement("wa-input")
export default class WaInput extends WebAwesomeFormAssociatedElement {
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

  /**
   * The type of input. Works the same as a native `<input>` element, but only a subset of types are supported. Defaults
   * to `text`.
   */
  @property({ reflect: true }) type:
    | "date"
    | "datetime-local"
    | "email"
    | "number"
    | "password"
    | "search"
    | "tel"
    | "text"
    | "time"
    | "url" = "text";

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

  /** Adds a clear button when the input is not empty. */
  @property({ attribute: "with-clear", type: Boolean }) withClear = false;

  /** Placeholder text to show as a hint when the input is empty. */
  @property() placeholder = "";

  /** Makes the input readonly. */
  @property({ type: Boolean, reflect: true }) readonly = false;

  /** Adds a button to toggle the password's visibility. Only applies to password types. */
  @property({ attribute: "password-toggle", type: Boolean }) passwordToggle =
    false;

  /** Determines whether or not the password is currently visible. Only applies to password input types. */
  @property({ attribute: "password-visible", type: Boolean }) passwordVisible =
    false;

  /** Hides the browser's built-in increment/decrement spin buttons for number inputs. */
  @property({ attribute: "without-spin-buttons", type: Boolean, reflect: true })
  withoutSpinButtons = false;

  /** Makes the input a required field. */
  @property({ type: Boolean, reflect: true }) required = false;

  /** A regular expression pattern to validate input against. */
  @property() pattern: string;

  /** The minimum length of input that will be considered valid. */
  @property({ type: Number }) minlength: number;

  /** The maximum length of input that will be considered valid. */
  @property({ type: Number }) maxlength: number;

  /** The input's minimum value. Only applies to date and number input types. */
  @property() min: number | string;

  /** The input's maximum value. Only applies to date and number input types. */
  @property() max: number | string;

  /**
   * Specifies the granularity that the value must adhere to, or the special value `any` which means no stepping is
   * implied, allowing any numeric value. Only applies to date and number input types.
   */
  @property() step: number | "any";

  /** Controls whether and how text input is automatically capitalized as it is entered by the user. */
  @property() autocapitalize:
    "off" | "none" | "on" | "sentences" | "words" | "characters";

  /**
   * Indicates whether the browser's autocorrect feature is on or off. When set as an attribute, use `"off"` or `"on"`.
   * When set as a property, use `true` or `false`.
   */
  @property({
    type: Boolean,
    converter: {
      fromAttribute: (value) => (!value || value === "off" ? false : true),
      toAttribute: (value) => (value ? "on" : "off"),
    },
  })
  declare autocorrect: boolean;

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

  /** Enables spell checking on the input. */
  @property({
    type: Boolean,
    converter: {
      // Allow "true|false" attribute values but keep the property boolean
      fromAttribute: (value) => (!value || value === "false" ? false : true),
      toAttribute: (value) => (value ? "true" : "false"),
    },
  })
  spellcheck = true;

  /**
   * Tells the browser what type of data will be entered by the user, allowing it to display the appropriate virtual
   * keyboard on supportive devices.
   */
  @property() inputmode:
    | "none"
    | "text"
    | "decimal"
    | "numeric"
    | "tel"
    | "search"
    | "email"
    | "url";

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

  private handleChange(event: Event) {
    this.value = this.input.value;

    this.relayNativeEvent(event, { bubbles: true, composed: true });
  }

  private handleClearClick(event: MouseEvent) {
    event.preventDefault();

    if (this.value !== "") {
      this.value = "";

      this.updateComplete.then(() => {
        this.dispatchEvent(new WaClearEvent());
        this.dispatchEvent(
          new InputEvent("input", { bubbles: true, composed: true }),
        );
        this.dispatchEvent(
          new Event("change", { bubbles: true, composed: true }),
        );
      });
    }

    this.input.focus();
  }

  private handleInput() {
    this.value = this.input.value;
  }

  private handleKeyDown(event: KeyboardEvent) {
    submitOnEnter(event, this);
  }

  private handlePasswordToggle() {
    this.passwordVisible = !this.passwordVisible;
  }

  updated(changedProperties: PropertyValues<this>) {
    super.updated(changedProperties);

    if (
      changedProperties.has("value") ||
      changedProperties.has("defaultValue") ||
      changedProperties.has("type")
    ) {
      // Types where the browser sanitizes invalid input to an empty string. Mirror that behavior so `value` stays
      // consistent with the native input (e.g. setting `"abc"` on `type="number"` resolves to `""`).
      const sanitizingTypes = ["number", "date", "time", "datetime-local"];
      if (
        this.input &&
        sanitizingTypes.includes(this.type) &&
        this.value &&
        this.input.value !== this.value
      ) {
        this._value = this.input.value;
      }

      this.customStates.set("blank", !this.value);
      this.updateValidity();
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

  /** Sets the start and end positions of the text selection (0-based). */
  setSelectionRange(
    selectionStart: number,
    selectionEnd: number,
    selectionDirection: "forward" | "backward" | "none" = "none",
  ) {
    this.input.setSelectionRange(
      selectionStart,
      selectionEnd,
      selectionDirection,
    );
  }

  /** Replaces a range of text with a new string. */
  setRangeText(
    replacement: string,
    start?: number,
    end?: number,
    selectMode: "select" | "start" | "end" | "preserve" = "preserve",
  ) {
    const selectionStart = start ?? this.input.selectionStart!;
    const selectionEnd = end ?? this.input.selectionEnd!;

    this.input.setRangeText(
      replacement,
      selectionStart,
      selectionEnd,
      selectMode,
    );

    if (this.value !== this.input.value) {
      this.value = this.input.value;
    }
  }

  /** Displays the browser picker for an input element (only works if the browser supports it for the input type). */
  showPicker() {
    if ("showPicker" in HTMLInputElement.prototype) {
      this.input.showPicker();
    }
  }

  /** Increments the value of a numeric input type by the value of the step attribute. */
  stepUp() {
    this.input.stepUp();
    if (this.value !== this.input.value) {
      this.value = this.input.value;
    }
  }

  /** Decrements the value of a numeric input type by the value of the step attribute. */
  stepDown() {
    this.input.stepDown();
    if (this.value !== this.input.value) {
      this.value = this.input.value;
    }
  }

  formResetCallback() {
    this.value = null;

    if (this.input) {
      // Fixes https://github.com/shoelace-style/webawesome/issues/1640 where resetting an input would leave the "live" vlaue in place on the input in the shadow dom. This fixed that by manually forcing the value.
      // @ts-expect-error
      this.input.value = this.value;
    }

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
    const hasClearIcon = this.withClear && !this.disabled && !this.readonly;
    const isClearIconVisible =
      // prevents hydration mismatch errors.
      (isServer || this.hasUpdated) &&
      hasClearIcon &&
      (typeof this.value === "number" || (this.value && this.value.length > 0));

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

      <div part="base" class="text-field">
        <slot name="start" part="start" class="start"></slot>

        <input
          part="input"
          id="input"
          class="control"
          type=${this.type === "password" && this.passwordVisible ? "text" : this.type}
          title=${this.title /* An empty title prevents browser validation tooltips from appearing on hover */}
          name=${ifDefined(this.name)}
          ?disabled=${this.disabled}
          ?readonly=${this.readonly}
          ?required=${this.required}
          placeholder=${ifDefined(this.placeholder)}
          minlength=${ifDefined(this.minlength)}
          maxlength=${ifDefined(this.maxlength)}
          min=${ifDefined(this.min)}
          max=${ifDefined(this.max)}
          step=${ifDefined(this.step as number)}
          .value=${live(this.value ?? "")}
          autocapitalize=${ifDefined(this.autocapitalize)}
          autocomplete=${ifDefined(this.autocomplete)}
          autocorrect=${this.autocorrect ? "on" : "off"}
          ?autofocus=${this.autofocus}
          spellcheck=${this.spellcheck}
          pattern=${ifDefined(this.pattern)}
          enterkeyhint=${ifDefined(this.enterkeyhint)}
          inputmode=${ifDefined(this.inputmode)}
          aria-describedby="hint"
          @change=${this.handleChange}
          @input=${this.handleInput}
          @keydown=${this.handleKeyDown}
        />

        ${
          isClearIconVisible
            ? html`
                <button
                  part="clear-button"
                  class="clear"
                  type="button"
                  aria-label=${this.localize.term("clearEntry")}
                  @click=${this.handleClearClick}
                  tabindex="-1"
                >
                  <slot name="clear-icon">
                    <wa-icon
                      name="circle-xmark"
                      library="system"
                      variant="regular"
                    ></wa-icon>
                  </slot>
                </button>
              `
            : ""
        }
        ${
          this.passwordToggle && !this.disabled
            ? html`
                <button
                  part="password-toggle-button"
                  class="password-toggle"
                  type="button"
                  aria-label=${this.localize.term(this.passwordVisible ? "hidePassword" : "showPassword")}
                  @click=${this.handlePasswordToggle}
                  tabindex="-1"
                >
                  ${
                  !this.passwordVisible
                    ? html`
                        <slot name="show-password-icon">
                          <wa-icon
                            name="eye"
                            library="system"
                            variant="regular"
                          ></wa-icon>
                        </slot>
                      `
                    : html`
                        <slot name="hide-password-icon">
                          <wa-icon
                            name="eye-slash"
                            library="system"
                            variant="regular"
                          ></wa-icon>
                        </slot>
                      `
                }
                </button>
              `
            : ""
        }

        <slot name="end" part="end" class="end"></slot>
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
WaInput.disableWarning?.("change-in-update");

declare global {
  interface HTMLElementTagNameMap {
    "wa-input": WaInput;
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
@@ -220,9 +220,16 @@
    * Tells the browser what type of data will be entered by the user, allowing it to display the appropriate virtual
    * keyboard on supportive devices.
    */
   @property() inputmode:
-    "none" | "text" | "decimal" | "numeric" | "tel" | "search" | "email" | "url";
+    | "none"
+    | "text"
+    | "decimal"
+    | "numeric"
+    | "tel"
+    | "search"
+    | "email"
+    | "url";
 
   /**
    * Only required for SSR. Set to `true` if you're slotting in a `label` element so the server-rendered markup
    * includes the label before the component hydrates on the client.
@@ -410,9 +417,12 @@
           part="input"
           id="input"
           class="control"
           type=${this.type === "password" && this.passwordVisible ? "text" : this.type}
-          title=${this.title /* An empty title prevents browser validation tooltips from appearing on hover */}
+          title=${
+            this
+              .title /* An empty title prevents browser validation tooltips from appearing on hover */
+          }
           name=${ifDefined(this.name)}
           ?disabled=${this.disabled}
           ?readonly=${this.readonly}
           ?required=${this.required}
@@ -436,54 +446,50 @@
           @input=${this.handleInput}
           @keydown=${this.handleKeyDown}
         />
 
-        ${
-          isClearIconVisible
-            ? html`
-                <button
-                  part="clear-button"
-                  class="clear"
-                  type="button"
-                  aria-label=${this.localize.term("clearEntry")}
-                  @click=${this.handleClearClick}
-                  tabindex="-1"
-                >
-                  <slot name="clear-icon">
-                    <wa-icon name="circle-xmark" library="system" variant="regular"></wa-icon>
-                  </slot>
-                </button>
-              `
-            : ""
-        }
-        ${
-          this.passwordToggle && !this.disabled
-            ? html`
-                <button
-                  part="password-toggle-button"
-                  class="password-toggle"
-                  type="button"
-                  aria-label=${this.localize.term(this.passwordVisible ? "hidePassword" : "showPassword")}
-                  @click=${this.handlePasswordToggle}
-                  tabindex="-1"
-                >
-                  ${
-                  !this.passwordVisible
-                    ? html`
-                        <slot name="show-password-icon">
-                          <wa-icon name="eye" library="system" variant="regular"></wa-icon>
-                        </slot>
-                      `
-                    : html`
-                        <slot name="hide-password-icon">
-                          <wa-icon name="eye-slash" library="system" variant="regular"></wa-icon>
-                        </slot>
-                      `
-                }
-                </button>
-              `
-            : ""
-        }
+        ${isClearIconVisible
+          ? html`
+              <button
+                part="clear-button"
+                class="clear"
+                type="button"
+                aria-label=${this.localize.term("clearEntry")}
+                @click=${this.handleClearClick}
+                tabindex="-1"
+              >
+                <slot name="clear-icon">
+                  <wa-icon name="circle-xmark" library="system" variant="regular"></wa-icon>
+                </slot>
+              </button>
+            `
+          : ""}
+        ${this.passwordToggle && !this.disabled
+          ? html`
+              <button
+                part="password-toggle-button"
+                class="password-toggle"
+                type="button"
+                aria-label=${this.localize.term(
+                  this.passwordVisible ? "hidePassword" : "showPassword",
+                )}
+                @click=${this.handlePasswordToggle}
+                tabindex="-1"
+              >
+                ${!this.passwordVisible
+                  ? html`
+                      <slot name="show-password-icon">
+                        <wa-icon name="eye" library="system" variant="regular"></wa-icon>
+                      </slot>
+                    `
+                  : html`
+                      <slot name="hide-password-icon">
+                        <wa-icon name="eye-slash" library="system" variant="regular"></wa-icon>
+                      </slot>
+                    `}
+              </button>
+            `
+          : ""}
 
         <slot name="end" part="end" class="end"></slot>
       </div>
 

`````

### Actual (oxfmt)

`````ts
import { html, isServer, type PropertyValues } from "lit";
import { customElement, property, query, state } from "lit/decorators.js";
import { classMap } from "lit/directives/class-map.js";
import { ifDefined } from "lit/directives/if-defined.js";
import { live } from "lit/directives/live.js";
import { WaClearEvent } from "../../events/clear.js";
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
import styles from "./input.styles.js";

/**
 * @summary Inputs collect single-line data from the user, such as text, numbers, email addresses, and passwords. They
 *  support labels, hints, validation, and prefix or suffix slots.
 * @documentation https://webawesome.com/docs/components/input
 * @status stable
 * @since 2.0
 *
 * @dependency wa-icon
 *
 * @slot label - The input's label. Alternatively, you can use the `label` attribute.
 * @slot start - An element, such as `<wa-icon>`, placed at the start of the input control.
 * @slot end - An element, such as `<wa-icon>`, placed at the end of the input control.
 * @slot clear-icon - An icon to use in lieu of the default clear icon.
 * @slot show-password-icon - An icon to use in lieu of the default show password icon.
 * @slot hide-password-icon - An icon to use in lieu of the default hide password icon.
 * @slot hint - Text that describes how to use the input. Alternatively, you can use the `hint` attribute.
 *
 * @event blur - Emitted when the control loses focus.
 * @event change - Emitted when an alteration to the control's value is committed by the user.
 * @event focus - Emitted when the control gains focus.
 * @event input - Emitted when the control receives input.
 * @event wa-clear - Emitted when the clear button is activated.
 * @event wa-invalid - Emitted when the form control has been checked for validity and its constraints aren't satisfied.
 *
 * @csspart label - The label
 * @csspart hint - The hint's wrapper.
 * @csspart base - The wrapper being rendered as an input
 * @csspart input - The internal `<input>` control.
 * @csspart start - The container that wraps the `start` slot.
 * @csspart clear-button - The clear button.
 * @csspart password-toggle-button - The password toggle button.
 * @csspart end - The container that wraps the `end` slot.
 *
 * @cssstate blank - The input is empty.
 */
@customElement("wa-input")
export default class WaInput extends WebAwesomeFormAssociatedElement {
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

  /**
   * The type of input. Works the same as a native `<input>` element, but only a subset of types are supported. Defaults
   * to `text`.
   */
  @property({ reflect: true }) type:
    | "date"
    | "datetime-local"
    | "email"
    | "number"
    | "password"
    | "search"
    | "tel"
    | "text"
    | "time"
    | "url" = "text";

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

  /** Adds a clear button when the input is not empty. */
  @property({ attribute: "with-clear", type: Boolean }) withClear = false;

  /** Placeholder text to show as a hint when the input is empty. */
  @property() placeholder = "";

  /** Makes the input readonly. */
  @property({ type: Boolean, reflect: true }) readonly = false;

  /** Adds a button to toggle the password's visibility. Only applies to password types. */
  @property({ attribute: "password-toggle", type: Boolean }) passwordToggle = false;

  /** Determines whether or not the password is currently visible. Only applies to password input types. */
  @property({ attribute: "password-visible", type: Boolean }) passwordVisible = false;

  /** Hides the browser's built-in increment/decrement spin buttons for number inputs. */
  @property({ attribute: "without-spin-buttons", type: Boolean, reflect: true })
  withoutSpinButtons = false;

  /** Makes the input a required field. */
  @property({ type: Boolean, reflect: true }) required = false;

  /** A regular expression pattern to validate input against. */
  @property() pattern: string;

  /** The minimum length of input that will be considered valid. */
  @property({ type: Number }) minlength: number;

  /** The maximum length of input that will be considered valid. */
  @property({ type: Number }) maxlength: number;

  /** The input's minimum value. Only applies to date and number input types. */
  @property() min: number | string;

  /** The input's maximum value. Only applies to date and number input types. */
  @property() max: number | string;

  /**
   * Specifies the granularity that the value must adhere to, or the special value `any` which means no stepping is
   * implied, allowing any numeric value. Only applies to date and number input types.
   */
  @property() step: number | "any";

  /** Controls whether and how text input is automatically capitalized as it is entered by the user. */
  @property() autocapitalize: "off" | "none" | "on" | "sentences" | "words" | "characters";

  /**
   * Indicates whether the browser's autocorrect feature is on or off. When set as an attribute, use `"off"` or `"on"`.
   * When set as a property, use `true` or `false`.
   */
  @property({
    type: Boolean,
    converter: {
      fromAttribute: (value) => (!value || value === "off" ? false : true),
      toAttribute: (value) => (value ? "on" : "off"),
    },
  })
  declare autocorrect: boolean;

  /**
   * Specifies what permission the browser has to provide assistance in filling out form field values. Refer to
   * [this page on MDN](https://developer.mozilla.org/en-US/docs/Web/HTML/Attributes/autocomplete) for available values.
   */
  @property() autocomplete: string;

  /** Indicates that the input should receive focus on page load. */
  @property({ type: Boolean }) autofocus: boolean;

  /** Used to customize the label or icon of the Enter key on virtual keyboards. */
  @property() enterkeyhint: "enter" | "done" | "go" | "next" | "previous" | "search" | "send";

  /** Enables spell checking on the input. */
  @property({
    type: Boolean,
    converter: {
      // Allow "true|false" attribute values but keep the property boolean
      fromAttribute: (value) => (!value || value === "false" ? false : true),
      toAttribute: (value) => (value ? "true" : "false"),
    },
  })
  spellcheck = true;

  /**
   * Tells the browser what type of data will be entered by the user, allowing it to display the appropriate virtual
   * keyboard on supportive devices.
   */
  @property() inputmode:
    | "none"
    | "text"
    | "decimal"
    | "numeric"
    | "tel"
    | "search"
    | "email"
    | "url";

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

  private handleChange(event: Event) {
    this.value = this.input.value;

    this.relayNativeEvent(event, { bubbles: true, composed: true });
  }

  private handleClearClick(event: MouseEvent) {
    event.preventDefault();

    if (this.value !== "") {
      this.value = "";

      this.updateComplete.then(() => {
        this.dispatchEvent(new WaClearEvent());
        this.dispatchEvent(new InputEvent("input", { bubbles: true, composed: true }));
        this.dispatchEvent(new Event("change", { bubbles: true, composed: true }));
      });
    }

    this.input.focus();
  }

  private handleInput() {
    this.value = this.input.value;
  }

  private handleKeyDown(event: KeyboardEvent) {
    submitOnEnter(event, this);
  }

  private handlePasswordToggle() {
    this.passwordVisible = !this.passwordVisible;
  }

  updated(changedProperties: PropertyValues<this>) {
    super.updated(changedProperties);

    if (
      changedProperties.has("value") ||
      changedProperties.has("defaultValue") ||
      changedProperties.has("type")
    ) {
      // Types where the browser sanitizes invalid input to an empty string. Mirror that behavior so `value` stays
      // consistent with the native input (e.g. setting `"abc"` on `type="number"` resolves to `""`).
      const sanitizingTypes = ["number", "date", "time", "datetime-local"];
      if (
        this.input &&
        sanitizingTypes.includes(this.type) &&
        this.value &&
        this.input.value !== this.value
      ) {
        this._value = this.input.value;
      }

      this.customStates.set("blank", !this.value);
      this.updateValidity();
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

  /** Sets the start and end positions of the text selection (0-based). */
  setSelectionRange(
    selectionStart: number,
    selectionEnd: number,
    selectionDirection: "forward" | "backward" | "none" = "none",
  ) {
    this.input.setSelectionRange(selectionStart, selectionEnd, selectionDirection);
  }

  /** Replaces a range of text with a new string. */
  setRangeText(
    replacement: string,
    start?: number,
    end?: number,
    selectMode: "select" | "start" | "end" | "preserve" = "preserve",
  ) {
    const selectionStart = start ?? this.input.selectionStart!;
    const selectionEnd = end ?? this.input.selectionEnd!;

    this.input.setRangeText(replacement, selectionStart, selectionEnd, selectMode);

    if (this.value !== this.input.value) {
      this.value = this.input.value;
    }
  }

  /** Displays the browser picker for an input element (only works if the browser supports it for the input type). */
  showPicker() {
    if ("showPicker" in HTMLInputElement.prototype) {
      this.input.showPicker();
    }
  }

  /** Increments the value of a numeric input type by the value of the step attribute. */
  stepUp() {
    this.input.stepUp();
    if (this.value !== this.input.value) {
      this.value = this.input.value;
    }
  }

  /** Decrements the value of a numeric input type by the value of the step attribute. */
  stepDown() {
    this.input.stepDown();
    if (this.value !== this.input.value) {
      this.value = this.input.value;
    }
  }

  formResetCallback() {
    this.value = null;

    if (this.input) {
      // Fixes https://github.com/shoelace-style/webawesome/issues/1640 where resetting an input would leave the "live" vlaue in place on the input in the shadow dom. This fixed that by manually forcing the value.
      // @ts-expect-error
      this.input.value = this.value;
    }

    super.formResetCallback();
  }

  render() {
    const hasLabelSlot = this.hasUpdated ? this.hasSlotController.test("label") : this.withLabel;
    const hasHintSlot = this.hasUpdated ? this.hasSlotController.test("hint") : this.withHint;
    const hasLabel = this.label ? true : !!hasLabelSlot;
    const hasHint = this.hint ? true : !!hasHintSlot;
    const hasClearIcon = this.withClear && !this.disabled && !this.readonly;
    const isClearIconVisible =
      // prevents hydration mismatch errors.
      (isServer || this.hasUpdated) &&
      hasClearIcon &&
      (typeof this.value === "number" || (this.value && this.value.length > 0));

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

      <div part="base" class="text-field">
        <slot name="start" part="start" class="start"></slot>

        <input
          part="input"
          id="input"
          class="control"
          type=${this.type === "password" && this.passwordVisible ? "text" : this.type}
          title=${
            this
              .title /* An empty title prevents browser validation tooltips from appearing on hover */
          }
          name=${ifDefined(this.name)}
          ?disabled=${this.disabled}
          ?readonly=${this.readonly}
          ?required=${this.required}
          placeholder=${ifDefined(this.placeholder)}
          minlength=${ifDefined(this.minlength)}
          maxlength=${ifDefined(this.maxlength)}
          min=${ifDefined(this.min)}
          max=${ifDefined(this.max)}
          step=${ifDefined(this.step as number)}
          .value=${live(this.value ?? "")}
          autocapitalize=${ifDefined(this.autocapitalize)}
          autocomplete=${ifDefined(this.autocomplete)}
          autocorrect=${this.autocorrect ? "on" : "off"}
          ?autofocus=${this.autofocus}
          spellcheck=${this.spellcheck}
          pattern=${ifDefined(this.pattern)}
          enterkeyhint=${ifDefined(this.enterkeyhint)}
          inputmode=${ifDefined(this.inputmode)}
          aria-describedby="hint"
          @change=${this.handleChange}
          @input=${this.handleInput}
          @keydown=${this.handleKeyDown}
        />

        ${isClearIconVisible
          ? html`
              <button
                part="clear-button"
                class="clear"
                type="button"
                aria-label=${this.localize.term("clearEntry")}
                @click=${this.handleClearClick}
                tabindex="-1"
              >
                <slot name="clear-icon">
                  <wa-icon name="circle-xmark" library="system" variant="regular"></wa-icon>
                </slot>
              </button>
            `
          : ""}
        ${this.passwordToggle && !this.disabled
          ? html`
              <button
                part="password-toggle-button"
                class="password-toggle"
                type="button"
                aria-label=${this.localize.term(
                  this.passwordVisible ? "hidePassword" : "showPassword",
                )}
                @click=${this.handlePasswordToggle}
                tabindex="-1"
              >
                ${!this.passwordVisible
                  ? html`
                      <slot name="show-password-icon">
                        <wa-icon name="eye" library="system" variant="regular"></wa-icon>
                      </slot>
                    `
                  : html`
                      <slot name="hide-password-icon">
                        <wa-icon name="eye-slash" library="system" variant="regular"></wa-icon>
                      </slot>
                    `}
              </button>
            `
          : ""}

        <slot name="end" part="end" class="end"></slot>
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
WaInput.disableWarning?.("change-in-update");

declare global {
  interface HTMLElementTagNameMap {
    "wa-input": WaInput;
  }
}

`````

### Expected (prettier)

`````ts
import { html, isServer, type PropertyValues } from "lit";
import { customElement, property, query, state } from "lit/decorators.js";
import { classMap } from "lit/directives/class-map.js";
import { ifDefined } from "lit/directives/if-defined.js";
import { live } from "lit/directives/live.js";
import { WaClearEvent } from "../../events/clear.js";
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
import styles from "./input.styles.js";

/**
 * @summary Inputs collect single-line data from the user, such as text, numbers, email addresses, and passwords. They
 *  support labels, hints, validation, and prefix or suffix slots.
 * @documentation https://webawesome.com/docs/components/input
 * @status stable
 * @since 2.0
 *
 * @dependency wa-icon
 *
 * @slot label - The input's label. Alternatively, you can use the `label` attribute.
 * @slot start - An element, such as `<wa-icon>`, placed at the start of the input control.
 * @slot end - An element, such as `<wa-icon>`, placed at the end of the input control.
 * @slot clear-icon - An icon to use in lieu of the default clear icon.
 * @slot show-password-icon - An icon to use in lieu of the default show password icon.
 * @slot hide-password-icon - An icon to use in lieu of the default hide password icon.
 * @slot hint - Text that describes how to use the input. Alternatively, you can use the `hint` attribute.
 *
 * @event blur - Emitted when the control loses focus.
 * @event change - Emitted when an alteration to the control's value is committed by the user.
 * @event focus - Emitted when the control gains focus.
 * @event input - Emitted when the control receives input.
 * @event wa-clear - Emitted when the clear button is activated.
 * @event wa-invalid - Emitted when the form control has been checked for validity and its constraints aren't satisfied.
 *
 * @csspart label - The label
 * @csspart hint - The hint's wrapper.
 * @csspart base - The wrapper being rendered as an input
 * @csspart input - The internal `<input>` control.
 * @csspart start - The container that wraps the `start` slot.
 * @csspart clear-button - The clear button.
 * @csspart password-toggle-button - The password toggle button.
 * @csspart end - The container that wraps the `end` slot.
 *
 * @cssstate blank - The input is empty.
 */
@customElement("wa-input")
export default class WaInput extends WebAwesomeFormAssociatedElement {
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

  /**
   * The type of input. Works the same as a native `<input>` element, but only a subset of types are supported. Defaults
   * to `text`.
   */
  @property({ reflect: true }) type:
    | "date"
    | "datetime-local"
    | "email"
    | "number"
    | "password"
    | "search"
    | "tel"
    | "text"
    | "time"
    | "url" = "text";

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

  /** Adds a clear button when the input is not empty. */
  @property({ attribute: "with-clear", type: Boolean }) withClear = false;

  /** Placeholder text to show as a hint when the input is empty. */
  @property() placeholder = "";

  /** Makes the input readonly. */
  @property({ type: Boolean, reflect: true }) readonly = false;

  /** Adds a button to toggle the password's visibility. Only applies to password types. */
  @property({ attribute: "password-toggle", type: Boolean }) passwordToggle = false;

  /** Determines whether or not the password is currently visible. Only applies to password input types. */
  @property({ attribute: "password-visible", type: Boolean }) passwordVisible = false;

  /** Hides the browser's built-in increment/decrement spin buttons for number inputs. */
  @property({ attribute: "without-spin-buttons", type: Boolean, reflect: true })
  withoutSpinButtons = false;

  /** Makes the input a required field. */
  @property({ type: Boolean, reflect: true }) required = false;

  /** A regular expression pattern to validate input against. */
  @property() pattern: string;

  /** The minimum length of input that will be considered valid. */
  @property({ type: Number }) minlength: number;

  /** The maximum length of input that will be considered valid. */
  @property({ type: Number }) maxlength: number;

  /** The input's minimum value. Only applies to date and number input types. */
  @property() min: number | string;

  /** The input's maximum value. Only applies to date and number input types. */
  @property() max: number | string;

  /**
   * Specifies the granularity that the value must adhere to, or the special value `any` which means no stepping is
   * implied, allowing any numeric value. Only applies to date and number input types.
   */
  @property() step: number | "any";

  /** Controls whether and how text input is automatically capitalized as it is entered by the user. */
  @property() autocapitalize: "off" | "none" | "on" | "sentences" | "words" | "characters";

  /**
   * Indicates whether the browser's autocorrect feature is on or off. When set as an attribute, use `"off"` or `"on"`.
   * When set as a property, use `true` or `false`.
   */
  @property({
    type: Boolean,
    converter: {
      fromAttribute: (value) => (!value || value === "off" ? false : true),
      toAttribute: (value) => (value ? "on" : "off"),
    },
  })
  declare autocorrect: boolean;

  /**
   * Specifies what permission the browser has to provide assistance in filling out form field values. Refer to
   * [this page on MDN](https://developer.mozilla.org/en-US/docs/Web/HTML/Attributes/autocomplete) for available values.
   */
  @property() autocomplete: string;

  /** Indicates that the input should receive focus on page load. */
  @property({ type: Boolean }) autofocus: boolean;

  /** Used to customize the label or icon of the Enter key on virtual keyboards. */
  @property() enterkeyhint: "enter" | "done" | "go" | "next" | "previous" | "search" | "send";

  /** Enables spell checking on the input. */
  @property({
    type: Boolean,
    converter: {
      // Allow "true|false" attribute values but keep the property boolean
      fromAttribute: (value) => (!value || value === "false" ? false : true),
      toAttribute: (value) => (value ? "true" : "false"),
    },
  })
  spellcheck = true;

  /**
   * Tells the browser what type of data will be entered by the user, allowing it to display the appropriate virtual
   * keyboard on supportive devices.
   */
  @property() inputmode:
    "none" | "text" | "decimal" | "numeric" | "tel" | "search" | "email" | "url";

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

  private handleChange(event: Event) {
    this.value = this.input.value;

    this.relayNativeEvent(event, { bubbles: true, composed: true });
  }

  private handleClearClick(event: MouseEvent) {
    event.preventDefault();

    if (this.value !== "") {
      this.value = "";

      this.updateComplete.then(() => {
        this.dispatchEvent(new WaClearEvent());
        this.dispatchEvent(new InputEvent("input", { bubbles: true, composed: true }));
        this.dispatchEvent(new Event("change", { bubbles: true, composed: true }));
      });
    }

    this.input.focus();
  }

  private handleInput() {
    this.value = this.input.value;
  }

  private handleKeyDown(event: KeyboardEvent) {
    submitOnEnter(event, this);
  }

  private handlePasswordToggle() {
    this.passwordVisible = !this.passwordVisible;
  }

  updated(changedProperties: PropertyValues<this>) {
    super.updated(changedProperties);

    if (
      changedProperties.has("value") ||
      changedProperties.has("defaultValue") ||
      changedProperties.has("type")
    ) {
      // Types where the browser sanitizes invalid input to an empty string. Mirror that behavior so `value` stays
      // consistent with the native input (e.g. setting `"abc"` on `type="number"` resolves to `""`).
      const sanitizingTypes = ["number", "date", "time", "datetime-local"];
      if (
        this.input &&
        sanitizingTypes.includes(this.type) &&
        this.value &&
        this.input.value !== this.value
      ) {
        this._value = this.input.value;
      }

      this.customStates.set("blank", !this.value);
      this.updateValidity();
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

  /** Sets the start and end positions of the text selection (0-based). */
  setSelectionRange(
    selectionStart: number,
    selectionEnd: number,
    selectionDirection: "forward" | "backward" | "none" = "none",
  ) {
    this.input.setSelectionRange(selectionStart, selectionEnd, selectionDirection);
  }

  /** Replaces a range of text with a new string. */
  setRangeText(
    replacement: string,
    start?: number,
    end?: number,
    selectMode: "select" | "start" | "end" | "preserve" = "preserve",
  ) {
    const selectionStart = start ?? this.input.selectionStart!;
    const selectionEnd = end ?? this.input.selectionEnd!;

    this.input.setRangeText(replacement, selectionStart, selectionEnd, selectMode);

    if (this.value !== this.input.value) {
      this.value = this.input.value;
    }
  }

  /** Displays the browser picker for an input element (only works if the browser supports it for the input type). */
  showPicker() {
    if ("showPicker" in HTMLInputElement.prototype) {
      this.input.showPicker();
    }
  }

  /** Increments the value of a numeric input type by the value of the step attribute. */
  stepUp() {
    this.input.stepUp();
    if (this.value !== this.input.value) {
      this.value = this.input.value;
    }
  }

  /** Decrements the value of a numeric input type by the value of the step attribute. */
  stepDown() {
    this.input.stepDown();
    if (this.value !== this.input.value) {
      this.value = this.input.value;
    }
  }

  formResetCallback() {
    this.value = null;

    if (this.input) {
      // Fixes https://github.com/shoelace-style/webawesome/issues/1640 where resetting an input would leave the "live" vlaue in place on the input in the shadow dom. This fixed that by manually forcing the value.
      // @ts-expect-error
      this.input.value = this.value;
    }

    super.formResetCallback();
  }

  render() {
    const hasLabelSlot = this.hasUpdated ? this.hasSlotController.test("label") : this.withLabel;
    const hasHintSlot = this.hasUpdated ? this.hasSlotController.test("hint") : this.withHint;
    const hasLabel = this.label ? true : !!hasLabelSlot;
    const hasHint = this.hint ? true : !!hasHintSlot;
    const hasClearIcon = this.withClear && !this.disabled && !this.readonly;
    const isClearIconVisible =
      // prevents hydration mismatch errors.
      (isServer || this.hasUpdated) &&
      hasClearIcon &&
      (typeof this.value === "number" || (this.value && this.value.length > 0));

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

      <div part="base" class="text-field">
        <slot name="start" part="start" class="start"></slot>

        <input
          part="input"
          id="input"
          class="control"
          type=${this.type === "password" && this.passwordVisible ? "text" : this.type}
          title=${this.title /* An empty title prevents browser validation tooltips from appearing on hover */}
          name=${ifDefined(this.name)}
          ?disabled=${this.disabled}
          ?readonly=${this.readonly}
          ?required=${this.required}
          placeholder=${ifDefined(this.placeholder)}
          minlength=${ifDefined(this.minlength)}
          maxlength=${ifDefined(this.maxlength)}
          min=${ifDefined(this.min)}
          max=${ifDefined(this.max)}
          step=${ifDefined(this.step as number)}
          .value=${live(this.value ?? "")}
          autocapitalize=${ifDefined(this.autocapitalize)}
          autocomplete=${ifDefined(this.autocomplete)}
          autocorrect=${this.autocorrect ? "on" : "off"}
          ?autofocus=${this.autofocus}
          spellcheck=${this.spellcheck}
          pattern=${ifDefined(this.pattern)}
          enterkeyhint=${ifDefined(this.enterkeyhint)}
          inputmode=${ifDefined(this.inputmode)}
          aria-describedby="hint"
          @change=${this.handleChange}
          @input=${this.handleInput}
          @keydown=${this.handleKeyDown}
        />

        ${
          isClearIconVisible
            ? html`
                <button
                  part="clear-button"
                  class="clear"
                  type="button"
                  aria-label=${this.localize.term("clearEntry")}
                  @click=${this.handleClearClick}
                  tabindex="-1"
                >
                  <slot name="clear-icon">
                    <wa-icon name="circle-xmark" library="system" variant="regular"></wa-icon>
                  </slot>
                </button>
              `
            : ""
        }
        ${
          this.passwordToggle && !this.disabled
            ? html`
                <button
                  part="password-toggle-button"
                  class="password-toggle"
                  type="button"
                  aria-label=${this.localize.term(this.passwordVisible ? "hidePassword" : "showPassword")}
                  @click=${this.handlePasswordToggle}
                  tabindex="-1"
                >
                  ${
                  !this.passwordVisible
                    ? html`
                        <slot name="show-password-icon">
                          <wa-icon name="eye" library="system" variant="regular"></wa-icon>
                        </slot>
                      `
                    : html`
                        <slot name="hide-password-icon">
                          <wa-icon name="eye-slash" library="system" variant="regular"></wa-icon>
                        </slot>
                      `
                }
                </button>
              `
            : ""
        }

        <slot name="end" part="end" class="end"></slot>
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
WaInput.disableWarning?.("change-in-update");

declare global {
  interface HTMLElementTagNameMap {
    "wa-input": WaInput;
  }
}

`````
