# externals/webawesome/radio-group/radio-group.ts

## Option 1

`````json
{"printWidth":80}
`````

### Diff

`````diff
===================================================================
--- prettier
+++ oxfmt
@@ -108,9 +108,16 @@
     this.getAttribute("value") || null;
 
   /** The radio group's size. When present, this size will be applied to all `<wa-radio>` items inside. */
   @property({ reflect: true }) size:
-    "xs" | "s" | "m" | "l" | "xl" | "small" | "medium" | "large";
+    | "xs"
+    | "s"
+    | "m"
+    | "l"
+    | "xl"
+    | "small"
+    | "medium"
+    | "large";
 
   @watch("size")
   handleSizeChange() {
     warnDeprecatedSize(this.localName, this.size);

`````

### Actual (oxfmt)

`````ts
import type { PropertyValues } from "lit";
import { html, isServer } from "lit";
import { customElement, property, query, state } from "lit/decorators.js";
import { classMap } from "lit/directives/class-map.js";
import { uniqueId } from "../../internal/math.js";
import { warnDeprecatedSize } from "../../internal/size.js";
import { HasSlotController } from "../../internal/slot.js";
import { RequiredValidator } from "../../internal/validators/required-validator.js";
import { watch } from "../../internal/watch.js";
import { WebAwesomeFormAssociatedElement } from "../../internal/webawesome-form-associated-element.js";
import formControlStyles from "../../styles/component/form-control.styles.js";
import sizeStyles from "../../styles/component/size.styles.js";
import "../radio/radio.js";
import type WaRadio from "../radio/radio.js";
import styles from "./radio-group.styles.js";

/**
 * @summary Radio groups wrap a set of radios so they function as a single form control with one shared value. They
 *  handle keyboard navigation, labeling, and validation for the group as a whole.
 * @documentation https://webawesome.com/docs/components/radio-group
 * @status stable
 * @since 2.0
 *
 * @dependency wa-radio
 *
 * @slot - The default slot where `<wa-radio>` elements are placed.
 * @slot label - The radio group's label. Required for proper accessibility. Alternatively, you can use the `label`
 *  attribute.
 * @slot hint - Text that describes how to use the radio group. Alternatively, you can use the `hint` attribute.
 *
 * @event change - Emitted when the radio group's selected value changes.
 * @event input - Emitted when the radio group receives user input.
 * @event wa-invalid - Emitted when the form control has been checked for validity and its constraints aren't satisfied.
 *
 * @csspart form-control - The form control that wraps the label, input, and hint.
 * @csspart form-control-label - The label's wrapper.
 * @csspart form-control-input - The input's wrapper.
 * @csspart radios - The wrapper than surrounds radio items, styled as a flex container by default.
 * @csspart hint - The hint's wrapper.
 */
@customElement("wa-radio-group")
export default class WaRadioGroup extends WebAwesomeFormAssociatedElement {
  static css = [sizeStyles, formControlStyles, styles];

  static get validators() {
    const validators = isServer
      ? []
      : [
          RequiredValidator({
            validationElement: Object.assign(document.createElement("input"), {
              required: true,
              type: "radio",
              // we need an id that's guaranteed to be unique; users will never see this
              name: uniqueId("__wa-radio"),
            }),
          }),
        ];
    return [...super.validators, ...validators];
  }

  private readonly hasSlotController = new HasSlotController(
    this,
    "hint",
    "label",
  );

  @query("slot:not([name])") defaultSlot: HTMLSlotElement;

  /**
   * The radio group's label. Required for proper accessibility. If you need to display HTML, use the `label` slot
   * instead.
   */
  @property() label = "";

  /** The radio groups's hint. If you need to display HTML, use the `hint` slot instead. */
  @property({ attribute: "hint" }) hint = "";

  /** The name of the radio group, submitted as a name/value pair with form data. */
  @property({ reflect: true }) name: string | null = null;

  /** Disables the radio group and all child radios. */
  @property({ type: Boolean, reflect: true }) disabled = false;

  /** The orientation in which to show radio items. */
  @property({ reflect: true }) orientation: "horizontal" | "vertical" =
    "vertical";

  private _value: string | null = null;

  get value() {
    if (this.valueHasChanged) {
      return this._value;
    }

    return this._value ?? this.defaultValue;
  }

  /** The current value of the radio group, submitted as a name/value pair with form data. */
  @state()
  set value(val: string | number | null) {
    if (typeof val === "number") val = String(val);
    this.valueHasChanged = true;
    this._value = val;
  }

  /** The default value of the form control. Primarily used for resetting the form control. */
  @property({ attribute: "value", reflect: true }) defaultValue: string | null =
    this.getAttribute("value") || null;

  /** The radio group's size. When present, this size will be applied to all `<wa-radio>` items inside. */
  @property({ reflect: true }) size:
    | "xs"
    | "s"
    | "m"
    | "l"
    | "xl"
    | "small"
    | "medium"
    | "large";

  @watch("size")
  handleSizeChange() {
    warnDeprecatedSize(this.localName, this.size);
  }

  /** Ensures a child radio is checked before allowing the containing form to submit. */
  @property({ type: Boolean, reflect: true }) required = false;

  /**
   * Only required for SSR. Set to `true` if you're slotting in a `label` element so the server-rendered markup
   * includes the label before the component hydrates on the client.
   */
  @property({ type: Boolean, attribute: "with-label" }) withLabel = false;

  /**
   * Only required for SSR. Set to `true` if you're slotting in a `hint` element so the server-rendered markup
   * includes the hint before the component hydrates on the client.
   */
  @property({ type: Boolean, attribute: "with-hint" }) withHint = false;

  //
  // We need this because if we don't have it, FormValidation yells at us that it's "not focusable".
  //   If we use `this.tabIndex = -1` we can't focus the radio inside.
  //
  static shadowRootOptions = {
    ...WebAwesomeFormAssociatedElement.shadowRootOptions,
    delegatesFocus: true,
  };

  constructor() {
    super();

    if (!isServer) {
      this.addEventListener("keydown", this.handleKeyDown);
      this.addEventListener("click", this.handleRadioClick);
    }
  }

  /**
   * We use the first available radio as the validationTarget similar to native HTML that shows the validation popup on
   * the first radio element.
   */
  get validationTarget() {
    if (isServer) return undefined;

    const radio = this.querySelector<WaRadio>(":is(wa-radio):not([disabled])");
    if (!radio) return undefined;

    return radio;
  }

  updated(changedProperties: PropertyValues<this>) {
    if (
      changedProperties.has("disabled") ||
      changedProperties.has("size") ||
      changedProperties.has("value") ||
      changedProperties.has("defaultValue")
    ) {
      this.syncRadioElements();
    }
  }

  formResetCallback(
    ...args: Parameters<WebAwesomeFormAssociatedElement["formResetCallback"]>
  ) {
    this._value = null;

    super.formResetCallback(...args);

    this.syncRadioElements();
  }

  private handleRadioClick = (e: Event) => {
    const clickedRadio = (e.target as HTMLElement).closest<WaRadio>("wa-radio");

    if (
      !clickedRadio ||
      clickedRadio.disabled ||
      (clickedRadio as any).forceDisabled ||
      this.disabled
    ) {
      return;
    }

    const oldValue = this.value;
    this.value = clickedRadio.value;
    clickedRadio.checked = true;

    const radios = this.getAllRadios();
    for (const radio of radios) {
      if (clickedRadio === radio) {
        continue;
      }

      radio.checked = false;
      radio.setAttribute("tabindex", "-1");
    }

    if (this.value !== oldValue) {
      this.updateComplete.then(() => {
        this.dispatchEvent(
          new InputEvent("input", { bubbles: true, composed: true }),
        );
        this.dispatchEvent(
          new Event("change", { bubbles: true, composed: true }),
        );
      });
    }
  };

  private getAllRadios() {
    return [...this.querySelectorAll<WaRadio>("wa-radio")];
  }

  private handleLabelClick() {
    this.focus();
  }

  private async syncRadioElements() {
    const radios = this.getAllRadios();

    // Set positioning data attributes and properties
    radios.forEach((radio, index) => {
      if (this.size) radio.setAttribute("size", this.size);
      radio.toggleAttribute(
        "data-wa-radio-horizontal",
        this.orientation !== "vertical",
      );
      radio.toggleAttribute(
        "data-wa-radio-vertical",
        this.orientation === "vertical",
      );
      radio.toggleAttribute("data-wa-radio-first", index === 0);
      radio.toggleAttribute(
        "data-wa-radio-inner",
        index !== 0 && index !== radios.length - 1,
      );
      radio.toggleAttribute("data-wa-radio-last", index === radios.length - 1);

      // Set forceDisabled state based on radio group's disabled state
      (radio as WaRadio).forceDisabled = this.disabled;
    });

    await Promise.all(
      radios.map(async (radio) => {
        await radio.updateComplete;

        if (!radio.disabled && radio.value === this.value) {
          radio.checked = true;
        } else {
          radio.checked = false;
        }
      }),
    );

    // Manage tabIndex based on disabled state and checked status
    if (this.disabled) {
      // If radio group is disabled, all radios should not be tabbable
      radios.forEach((radio) => {
        radio.tabIndex = -1;
      });
    } else {
      // Normal tabbing behavior
      const enabledRadios = radios.filter((radio) => !radio.disabled);
      const checkedRadio = enabledRadios.find((radio) => radio.checked);

      if (enabledRadios.length > 0) {
        if (checkedRadio) {
          // If there's a checked radio, it should be tabbable
          enabledRadios.forEach((radio) => {
            radio.tabIndex = radio.checked ? 0 : -1;
          });
        } else {
          // If no radio is checked, first enabled radio should be tabbable
          enabledRadios.forEach((radio, index) => {
            radio.tabIndex = index === 0 ? 0 : -1;
          });
        }
      }

      // Disabled radios should never be tabbable
      radios
        .filter((radio) => radio.disabled)
        .forEach((radio) => {
          radio.tabIndex = -1;
        });
    }
  }

  private handleKeyDown(event: KeyboardEvent) {
    if (
      !["ArrowUp", "ArrowDown", "ArrowLeft", "ArrowRight", " "].includes(
        event.key,
      ) ||
      this.disabled
    ) {
      return;
    }

    const radios = this.getAllRadios().filter((radio) => !radio.disabled);

    if (radios.length <= 0) {
      return;
    }

    event.preventDefault();

    const oldValue = this.value;

    const checkedRadio = radios.find((radio) => radio.checked) ?? radios[0];
    const incr =
      event.key === " "
        ? 0
        : ["ArrowUp", "ArrowLeft"].includes(event.key)
          ? -1
          : 1;
    let index = radios.indexOf(checkedRadio) + incr;

    if (!index) index = 0;

    if (index < 0) {
      index = radios.length - 1;
    }

    if (index > radios.length - 1) {
      index = 0;
    }

    const hasRadioButtons = radios.some(
      (radio) => radio.tagName.toLowerCase() === "wa-radio-button",
    );

    this.getAllRadios().forEach((radio) => {
      radio.checked = false;

      if (!hasRadioButtons) {
        radio.setAttribute("tabindex", "-1");
      }
    });

    this.value = radios[index].value;
    radios[index].checked = true;

    if (!hasRadioButtons) {
      radios[index].setAttribute("tabindex", "0");
      radios[index].focus();
    } else {
      radios[index].shadowRoot!.querySelector("button")!.focus();
    }

    if (this.value !== oldValue) {
      this.updateComplete.then(() => {
        this.dispatchEvent(
          new InputEvent("input", { bubbles: true, composed: true }),
        );
        this.dispatchEvent(
          new Event("change", { bubbles: true, composed: true }),
        );
      });
    }

    event.preventDefault();
  }

  /** Sets focus on the radio group. */
  public focus(options?: FocusOptions) {
    if (this.disabled) return;

    const radios = this.getAllRadios();
    const checked = radios.find((radio) => radio.checked);
    const firstEnabledRadio = radios.find((radio) => !radio.disabled);
    const radioToFocus = checked || firstEnabledRadio;

    // Call focus for the checked radio. If no radio is checked, focus the first one that isn't disabled.
    if (radioToFocus) {
      radioToFocus.focus(options);
    }
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
      <fieldset
        part="form-control"
        class=${classMap({
          "form-control": true,
          "form-control-radio-group": true,
          "form-control-has-label": hasLabel,
        })}
        role="radiogroup"
        aria-labelledby="label"
        aria-describedby="hint"
        aria-errormessage="error-message"
        aria-orientation=${this.orientation}
      >
        <label
          part="form-control-label"
          id="label"
          class=${classMap({
            label: true,
            "has-label": hasLabel,
          })}
          aria-hidden=${hasLabel ? "false" : "true"}
          @click=${this.handleLabelClick}
        >
          <slot name="label">${this.label}</slot>
        </label>

        <slot
          part="form-control-input"
          @slotchange=${this.syncRadioElements}
        ></slot>

        <slot
          id="hint"
          name="hint"
          part="hint"
          class=${classMap({
            "has-slotted": hasHint,
          })}
          aria-hidden=${hasHint ? "false" : "true"}
          >${this.hint}</slot
        >
      </fieldset>
    `;
  }
}

// The change-in-update warning is required for this component because HasSlotController calls requestUpdate() in
// response to slotchange events after first render, and the form validation system calls requestUpdate('validity')
// during firstUpdated() to initialize constraint validation state. Both are essential for correct behavior.
// See https://lit.dev/docs/tools/development/#development-build-runtime-warnings
WaRadioGroup.disableWarning?.("change-in-update");

declare global {
  interface HTMLElementTagNameMap {
    "wa-radio-group": WaRadioGroup;
  }
}

`````

### Expected (prettier)

`````ts
import type { PropertyValues } from "lit";
import { html, isServer } from "lit";
import { customElement, property, query, state } from "lit/decorators.js";
import { classMap } from "lit/directives/class-map.js";
import { uniqueId } from "../../internal/math.js";
import { warnDeprecatedSize } from "../../internal/size.js";
import { HasSlotController } from "../../internal/slot.js";
import { RequiredValidator } from "../../internal/validators/required-validator.js";
import { watch } from "../../internal/watch.js";
import { WebAwesomeFormAssociatedElement } from "../../internal/webawesome-form-associated-element.js";
import formControlStyles from "../../styles/component/form-control.styles.js";
import sizeStyles from "../../styles/component/size.styles.js";
import "../radio/radio.js";
import type WaRadio from "../radio/radio.js";
import styles from "./radio-group.styles.js";

/**
 * @summary Radio groups wrap a set of radios so they function as a single form control with one shared value. They
 *  handle keyboard navigation, labeling, and validation for the group as a whole.
 * @documentation https://webawesome.com/docs/components/radio-group
 * @status stable
 * @since 2.0
 *
 * @dependency wa-radio
 *
 * @slot - The default slot where `<wa-radio>` elements are placed.
 * @slot label - The radio group's label. Required for proper accessibility. Alternatively, you can use the `label`
 *  attribute.
 * @slot hint - Text that describes how to use the radio group. Alternatively, you can use the `hint` attribute.
 *
 * @event change - Emitted when the radio group's selected value changes.
 * @event input - Emitted when the radio group receives user input.
 * @event wa-invalid - Emitted when the form control has been checked for validity and its constraints aren't satisfied.
 *
 * @csspart form-control - The form control that wraps the label, input, and hint.
 * @csspart form-control-label - The label's wrapper.
 * @csspart form-control-input - The input's wrapper.
 * @csspart radios - The wrapper than surrounds radio items, styled as a flex container by default.
 * @csspart hint - The hint's wrapper.
 */
@customElement("wa-radio-group")
export default class WaRadioGroup extends WebAwesomeFormAssociatedElement {
  static css = [sizeStyles, formControlStyles, styles];

  static get validators() {
    const validators = isServer
      ? []
      : [
          RequiredValidator({
            validationElement: Object.assign(document.createElement("input"), {
              required: true,
              type: "radio",
              // we need an id that's guaranteed to be unique; users will never see this
              name: uniqueId("__wa-radio"),
            }),
          }),
        ];
    return [...super.validators, ...validators];
  }

  private readonly hasSlotController = new HasSlotController(
    this,
    "hint",
    "label",
  );

  @query("slot:not([name])") defaultSlot: HTMLSlotElement;

  /**
   * The radio group's label. Required for proper accessibility. If you need to display HTML, use the `label` slot
   * instead.
   */
  @property() label = "";

  /** The radio groups's hint. If you need to display HTML, use the `hint` slot instead. */
  @property({ attribute: "hint" }) hint = "";

  /** The name of the radio group, submitted as a name/value pair with form data. */
  @property({ reflect: true }) name: string | null = null;

  /** Disables the radio group and all child radios. */
  @property({ type: Boolean, reflect: true }) disabled = false;

  /** The orientation in which to show radio items. */
  @property({ reflect: true }) orientation: "horizontal" | "vertical" =
    "vertical";

  private _value: string | null = null;

  get value() {
    if (this.valueHasChanged) {
      return this._value;
    }

    return this._value ?? this.defaultValue;
  }

  /** The current value of the radio group, submitted as a name/value pair with form data. */
  @state()
  set value(val: string | number | null) {
    if (typeof val === "number") val = String(val);
    this.valueHasChanged = true;
    this._value = val;
  }

  /** The default value of the form control. Primarily used for resetting the form control. */
  @property({ attribute: "value", reflect: true }) defaultValue: string | null =
    this.getAttribute("value") || null;

  /** The radio group's size. When present, this size will be applied to all `<wa-radio>` items inside. */
  @property({ reflect: true }) size:
    "xs" | "s" | "m" | "l" | "xl" | "small" | "medium" | "large";

  @watch("size")
  handleSizeChange() {
    warnDeprecatedSize(this.localName, this.size);
  }

  /** Ensures a child radio is checked before allowing the containing form to submit. */
  @property({ type: Boolean, reflect: true }) required = false;

  /**
   * Only required for SSR. Set to `true` if you're slotting in a `label` element so the server-rendered markup
   * includes the label before the component hydrates on the client.
   */
  @property({ type: Boolean, attribute: "with-label" }) withLabel = false;

  /**
   * Only required for SSR. Set to `true` if you're slotting in a `hint` element so the server-rendered markup
   * includes the hint before the component hydrates on the client.
   */
  @property({ type: Boolean, attribute: "with-hint" }) withHint = false;

  //
  // We need this because if we don't have it, FormValidation yells at us that it's "not focusable".
  //   If we use `this.tabIndex = -1` we can't focus the radio inside.
  //
  static shadowRootOptions = {
    ...WebAwesomeFormAssociatedElement.shadowRootOptions,
    delegatesFocus: true,
  };

  constructor() {
    super();

    if (!isServer) {
      this.addEventListener("keydown", this.handleKeyDown);
      this.addEventListener("click", this.handleRadioClick);
    }
  }

  /**
   * We use the first available radio as the validationTarget similar to native HTML that shows the validation popup on
   * the first radio element.
   */
  get validationTarget() {
    if (isServer) return undefined;

    const radio = this.querySelector<WaRadio>(":is(wa-radio):not([disabled])");
    if (!radio) return undefined;

    return radio;
  }

  updated(changedProperties: PropertyValues<this>) {
    if (
      changedProperties.has("disabled") ||
      changedProperties.has("size") ||
      changedProperties.has("value") ||
      changedProperties.has("defaultValue")
    ) {
      this.syncRadioElements();
    }
  }

  formResetCallback(
    ...args: Parameters<WebAwesomeFormAssociatedElement["formResetCallback"]>
  ) {
    this._value = null;

    super.formResetCallback(...args);

    this.syncRadioElements();
  }

  private handleRadioClick = (e: Event) => {
    const clickedRadio = (e.target as HTMLElement).closest<WaRadio>("wa-radio");

    if (
      !clickedRadio ||
      clickedRadio.disabled ||
      (clickedRadio as any).forceDisabled ||
      this.disabled
    ) {
      return;
    }

    const oldValue = this.value;
    this.value = clickedRadio.value;
    clickedRadio.checked = true;

    const radios = this.getAllRadios();
    for (const radio of radios) {
      if (clickedRadio === radio) {
        continue;
      }

      radio.checked = false;
      radio.setAttribute("tabindex", "-1");
    }

    if (this.value !== oldValue) {
      this.updateComplete.then(() => {
        this.dispatchEvent(
          new InputEvent("input", { bubbles: true, composed: true }),
        );
        this.dispatchEvent(
          new Event("change", { bubbles: true, composed: true }),
        );
      });
    }
  };

  private getAllRadios() {
    return [...this.querySelectorAll<WaRadio>("wa-radio")];
  }

  private handleLabelClick() {
    this.focus();
  }

  private async syncRadioElements() {
    const radios = this.getAllRadios();

    // Set positioning data attributes and properties
    radios.forEach((radio, index) => {
      if (this.size) radio.setAttribute("size", this.size);
      radio.toggleAttribute(
        "data-wa-radio-horizontal",
        this.orientation !== "vertical",
      );
      radio.toggleAttribute(
        "data-wa-radio-vertical",
        this.orientation === "vertical",
      );
      radio.toggleAttribute("data-wa-radio-first", index === 0);
      radio.toggleAttribute(
        "data-wa-radio-inner",
        index !== 0 && index !== radios.length - 1,
      );
      radio.toggleAttribute("data-wa-radio-last", index === radios.length - 1);

      // Set forceDisabled state based on radio group's disabled state
      (radio as WaRadio).forceDisabled = this.disabled;
    });

    await Promise.all(
      radios.map(async (radio) => {
        await radio.updateComplete;

        if (!radio.disabled && radio.value === this.value) {
          radio.checked = true;
        } else {
          radio.checked = false;
        }
      }),
    );

    // Manage tabIndex based on disabled state and checked status
    if (this.disabled) {
      // If radio group is disabled, all radios should not be tabbable
      radios.forEach((radio) => {
        radio.tabIndex = -1;
      });
    } else {
      // Normal tabbing behavior
      const enabledRadios = radios.filter((radio) => !radio.disabled);
      const checkedRadio = enabledRadios.find((radio) => radio.checked);

      if (enabledRadios.length > 0) {
        if (checkedRadio) {
          // If there's a checked radio, it should be tabbable
          enabledRadios.forEach((radio) => {
            radio.tabIndex = radio.checked ? 0 : -1;
          });
        } else {
          // If no radio is checked, first enabled radio should be tabbable
          enabledRadios.forEach((radio, index) => {
            radio.tabIndex = index === 0 ? 0 : -1;
          });
        }
      }

      // Disabled radios should never be tabbable
      radios
        .filter((radio) => radio.disabled)
        .forEach((radio) => {
          radio.tabIndex = -1;
        });
    }
  }

  private handleKeyDown(event: KeyboardEvent) {
    if (
      !["ArrowUp", "ArrowDown", "ArrowLeft", "ArrowRight", " "].includes(
        event.key,
      ) ||
      this.disabled
    ) {
      return;
    }

    const radios = this.getAllRadios().filter((radio) => !radio.disabled);

    if (radios.length <= 0) {
      return;
    }

    event.preventDefault();

    const oldValue = this.value;

    const checkedRadio = radios.find((radio) => radio.checked) ?? radios[0];
    const incr =
      event.key === " "
        ? 0
        : ["ArrowUp", "ArrowLeft"].includes(event.key)
          ? -1
          : 1;
    let index = radios.indexOf(checkedRadio) + incr;

    if (!index) index = 0;

    if (index < 0) {
      index = radios.length - 1;
    }

    if (index > radios.length - 1) {
      index = 0;
    }

    const hasRadioButtons = radios.some(
      (radio) => radio.tagName.toLowerCase() === "wa-radio-button",
    );

    this.getAllRadios().forEach((radio) => {
      radio.checked = false;

      if (!hasRadioButtons) {
        radio.setAttribute("tabindex", "-1");
      }
    });

    this.value = radios[index].value;
    radios[index].checked = true;

    if (!hasRadioButtons) {
      radios[index].setAttribute("tabindex", "0");
      radios[index].focus();
    } else {
      radios[index].shadowRoot!.querySelector("button")!.focus();
    }

    if (this.value !== oldValue) {
      this.updateComplete.then(() => {
        this.dispatchEvent(
          new InputEvent("input", { bubbles: true, composed: true }),
        );
        this.dispatchEvent(
          new Event("change", { bubbles: true, composed: true }),
        );
      });
    }

    event.preventDefault();
  }

  /** Sets focus on the radio group. */
  public focus(options?: FocusOptions) {
    if (this.disabled) return;

    const radios = this.getAllRadios();
    const checked = radios.find((radio) => radio.checked);
    const firstEnabledRadio = radios.find((radio) => !radio.disabled);
    const radioToFocus = checked || firstEnabledRadio;

    // Call focus for the checked radio. If no radio is checked, focus the first one that isn't disabled.
    if (radioToFocus) {
      radioToFocus.focus(options);
    }
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
      <fieldset
        part="form-control"
        class=${classMap({
          "form-control": true,
          "form-control-radio-group": true,
          "form-control-has-label": hasLabel,
        })}
        role="radiogroup"
        aria-labelledby="label"
        aria-describedby="hint"
        aria-errormessage="error-message"
        aria-orientation=${this.orientation}
      >
        <label
          part="form-control-label"
          id="label"
          class=${classMap({
            label: true,
            "has-label": hasLabel,
          })}
          aria-hidden=${hasLabel ? "false" : "true"}
          @click=${this.handleLabelClick}
        >
          <slot name="label">${this.label}</slot>
        </label>

        <slot
          part="form-control-input"
          @slotchange=${this.syncRadioElements}
        ></slot>

        <slot
          id="hint"
          name="hint"
          part="hint"
          class=${classMap({
            "has-slotted": hasHint,
          })}
          aria-hidden=${hasHint ? "false" : "true"}
          >${this.hint}</slot
        >
      </fieldset>
    `;
  }
}

// The change-in-update warning is required for this component because HasSlotController calls requestUpdate() in
// response to slotchange events after first render, and the form validation system calls requestUpdate('validity')
// during firstUpdated() to initialize constraint validation state. Both are essential for correct behavior.
// See https://lit.dev/docs/tools/development/#development-build-runtime-warnings
WaRadioGroup.disableWarning?.("change-in-update");

declare global {
  interface HTMLElementTagNameMap {
    "wa-radio-group": WaRadioGroup;
  }
}

`````
