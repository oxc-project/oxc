# externals/webawesome/option/option.ts

## Option 1

`````json
{"printWidth":80}
`````

### Diff

`````diff
===================================================================
--- prettier
+++ oxfmt
@@ -220,24 +220,22 @@
   }
 
   render() {
     return html`
-      ${
-        this.selected
-          ? html`<wa-icon
-              part="checked-icon"
-              class="check"
-              name="check"
-              library="system"
-              variant="solid"
-              aria-hidden="true"
-            ></wa-icon>`
-          : html`<span
-              part="checked-icon"
-              class="check"
-              aria-hidden="true"
-            ></span>`
-      }
+      ${this.selected
+        ? html`<wa-icon
+            part="checked-icon"
+            class="check"
+            name="check"
+            library="system"
+            variant="solid"
+            aria-hidden="true"
+          ></wa-icon>`
+        : html`<span
+            part="checked-icon"
+            class="check"
+            aria-hidden="true"
+          ></span>`}
       <slot part="start" name="start" class="start"></slot>
       <slot
         part="label"
         class="label"

`````

### Actual (oxfmt)

`````ts
import type { PropertyValues } from "lit";
import { html } from "lit";
import { customElement, property, query, state } from "lit/decorators.js";
import getText from "../../internal/get-text.js";
import WebAwesomeElement from "../../internal/webawesome-element.js";
import { LocalizeController } from "../../utilities/localize.js";
import "../icon/icon.js";
import type WaSelect from "../select/select.js";
import styles from "./option.styles.js";

/**
 * @summary Options represent the individual choices inside a select or similar form control. Each option holds a value
 *  and the label shown to the user.
 * @documentation https://webawesome.com/docs/components/option
 * @status stable
 * @since 2.0
 *
 * @dependency wa-icon
 *
 * @slot - The option's label.
 * @slot start - An element, such as `<wa-icon>`, placed before the label.
 * @slot end - An element, such as `<wa-icon>`, placed after the label.
 *
 * @csspart checked-icon - The checked icon, a `<wa-icon>` element.
 * @csspart label - The option's label.
 * @csspart start - The container that wraps the `start` slot.
 * @csspart end - The container that wraps the `end` slot.
 *
 * @cssstate current - The user has keyed into the option, but hasn't selected it yet (shows a highlight)
 * @cssstate selected - The option is selected and has aria-selected="true"
 * @cssstate disabled - Applied when the option is disabled
 * @cssstate hover - Like `:hover` but works while dragging in Safari
 */
@customElement("wa-option")
export default class WaOption extends WebAwesomeElement {
  static css = styles;

  // @ts-expect-error - Controller is currently unused
  private readonly localize = new LocalizeController(this);
  private cachedDefaultLabel = "";
  private isInitialized = false;
  private isDefaultLabelDirty = true;

  @query(".label") defaultSlot: HTMLSlotElement;

  // Set via the parent select
  @state() current = false;

  /**
   * The option's value. When selected, the containing form control will receive this value. The value must be unique
   * from other options in the same group. Values may not contain spaces, as spaces are used as delimiters when listing
   * multiple values.
   */
  @property({ reflect: true }) value = "";

  /** Draws the option in a disabled state, preventing selection. */
  @property({ type: Boolean }) disabled = false;

  /** @internal */
  @property({ type: Boolean, attribute: false }) selected = false;

  /** Selects an option initially. */
  @property({ type: Boolean, attribute: "selected" }) defaultSelected = false;

  _label: string = "";
  /**
   * The option’s plain text label.
   * Usually automatically generated, but can be useful to provide manually for cases involving complex content.
   */
  @property()
  set label(value) {
    const oldValue = this._label;
    this._label = value || "";

    if (this._label !== oldValue) {
      this.requestUpdate("label", oldValue);
    }
  }

  get label(): string {
    if (this._label) {
      return this._label;
    }

    return this.defaultLabel;
  }

  /** The default label, generated from the element contents. Will be equal to `label` in most cases. */
  get defaultLabel(): string {
    if (this.isDefaultLabelDirty || !this.cachedDefaultLabel) {
      this.updateDefaultLabel();
    }
    return this.cachedDefaultLabel;
  }

  connectedCallback() {
    super.connectedCallback();
    this.setAttribute("role", "option");
    this.setAttribute("aria-selected", "false");

    this.addEventListener("mouseenter", this.handleHover);
    this.addEventListener("mouseleave", this.handleHover);
  }

  disconnectedCallback(): void {
    super.disconnectedCallback();

    this.removeEventListener("mouseenter", this.handleHover);
    this.removeEventListener("mouseleave", this.handleHover);
  }

  private handleDefaultSlotChange() {
    // Mark the default label as needing recalculation
    this.isDefaultLabelDirty = true;

    if (this.isInitialized) {
      // When the label changes, tell the parent <wa-select> to update. The parent's handleDefaultSlotChange already
      // calls selectionChanged() internally, so we don't need to call it separately here.
      customElements.whenDefined("wa-select").then(() => {
        const controller = this.closest("wa-select");
        if (controller) {
          controller.handleDefaultSlotChange();
        }
      });

      // When the label changes, tell the parent <wa-combobox> to update
      customElements.whenDefined("wa-combobox").then(() => {
        // We cast to <wa-select> because it shares the same API as combobox
        const controller = this.closest<WaSelect>("wa-combobox");
        if (controller) {
          controller.handleDefaultSlotChange();
        }
      });
    } else {
      this.isInitialized = true;
    }
  }

  private handleHover = (event: Event) => {
    // We need this because Safari doesn't honor :hover styles while dragging
    // Test case: https://codepen.io/leaverou/pen/VYZOOjy
    if (event.type === "mouseenter") {
      this.customStates.set("hover", true);
    } else if (event.type === "mouseleave") {
      this.customStates.set("hover", false);
    }
  };

  protected willUpdate(changedProperties: PropertyValues<this>): void {
    if (changedProperties.has("defaultSelected")) {
      // We cast to <wa-select> because it shares the same API as combobox
      if (!this.closest<WaSelect>("wa-combobox, wa-select")?.hasInteracted) {
        // Only sync if defaultSelected is becoming true
        // This prevents overwriting `selected` when it was set directly by frameworks like Vue
        if (this.defaultSelected) {
          const oldVal = this.selected;
          this.selected = this.defaultSelected;
          this.requestUpdate("selected", oldVal);
        }
      }
    }
    super.willUpdate(changedProperties);
  }

  updated(changedProperties: PropertyValues<this>) {
    super.updated(changedProperties);

    if (changedProperties.has("disabled")) {
      this.setAttribute("aria-disabled", this.disabled ? "true" : "false");
      this.customStates.set("disabled", this.disabled);
    }

    if (changedProperties.has("selected")) {
      this.setAttribute("aria-selected", this.selected ? "true" : "false");
      this.customStates.set("selected", this.selected);
    }

    if (changedProperties.has("value")) {
      // Ensure the value is a string. This ensures the next line doesn't error and allows framework users to pass numbers
      // instead of requiring them to cast the value to a string.
      if (typeof this.value !== "string") {
        this.value = String(this.value);
      }

      this.handleDefaultSlotChange();
    }

    if (changedProperties.has("current")) {
      this.customStates.set("current", this.current);
    }
  }

  protected firstUpdated(changedProperties: PropertyValues<this>) {
    super.firstUpdated(changedProperties);

    // If the `selected` property was set directly (e.g., by Vue's :selected binding),
    // notify the parent select to update its selection. This is needed because
    // Vue binds to the `selected` property instead of the `defaultSelected` property
    // when using `:selected="true"` syntax.
    if (this.selected && !this.defaultSelected) {
      const parent = this.closest<WaSelect>("wa-select, wa-combobox");
      if (parent && !parent.hasInteracted) {
        parent.selectionChanged?.();
      }
    }
  }

  private updateDefaultLabel() {
    let oldValue = this.cachedDefaultLabel;
    this.cachedDefaultLabel = getText(this).trim();
    this.isDefaultLabelDirty = false;
    let changed = this.cachedDefaultLabel !== oldValue;

    if (!this._label && changed) {
      // Uses default label, and it has changed
      this.requestUpdate("label", oldValue);
    }

    return changed;
  }

  render() {
    return html`
      ${this.selected
        ? html`<wa-icon
            part="checked-icon"
            class="check"
            name="check"
            library="system"
            variant="solid"
            aria-hidden="true"
          ></wa-icon>`
        : html`<span
            part="checked-icon"
            class="check"
            aria-hidden="true"
          ></span>`}
      <slot part="start" name="start" class="start"></slot>
      <slot
        part="label"
        class="label"
        @slotchange=${this.handleDefaultSlotChange}
      ></slot>
      <slot part="end" name="end" class="end"></slot>
    `;
  }
}

declare global {
  interface HTMLElementTagNameMap {
    "wa-option": WaOption;
  }
}

`````

### Expected (prettier)

`````ts
import type { PropertyValues } from "lit";
import { html } from "lit";
import { customElement, property, query, state } from "lit/decorators.js";
import getText from "../../internal/get-text.js";
import WebAwesomeElement from "../../internal/webawesome-element.js";
import { LocalizeController } from "../../utilities/localize.js";
import "../icon/icon.js";
import type WaSelect from "../select/select.js";
import styles from "./option.styles.js";

/**
 * @summary Options represent the individual choices inside a select or similar form control. Each option holds a value
 *  and the label shown to the user.
 * @documentation https://webawesome.com/docs/components/option
 * @status stable
 * @since 2.0
 *
 * @dependency wa-icon
 *
 * @slot - The option's label.
 * @slot start - An element, such as `<wa-icon>`, placed before the label.
 * @slot end - An element, such as `<wa-icon>`, placed after the label.
 *
 * @csspart checked-icon - The checked icon, a `<wa-icon>` element.
 * @csspart label - The option's label.
 * @csspart start - The container that wraps the `start` slot.
 * @csspart end - The container that wraps the `end` slot.
 *
 * @cssstate current - The user has keyed into the option, but hasn't selected it yet (shows a highlight)
 * @cssstate selected - The option is selected and has aria-selected="true"
 * @cssstate disabled - Applied when the option is disabled
 * @cssstate hover - Like `:hover` but works while dragging in Safari
 */
@customElement("wa-option")
export default class WaOption extends WebAwesomeElement {
  static css = styles;

  // @ts-expect-error - Controller is currently unused
  private readonly localize = new LocalizeController(this);
  private cachedDefaultLabel = "";
  private isInitialized = false;
  private isDefaultLabelDirty = true;

  @query(".label") defaultSlot: HTMLSlotElement;

  // Set via the parent select
  @state() current = false;

  /**
   * The option's value. When selected, the containing form control will receive this value. The value must be unique
   * from other options in the same group. Values may not contain spaces, as spaces are used as delimiters when listing
   * multiple values.
   */
  @property({ reflect: true }) value = "";

  /** Draws the option in a disabled state, preventing selection. */
  @property({ type: Boolean }) disabled = false;

  /** @internal */
  @property({ type: Boolean, attribute: false }) selected = false;

  /** Selects an option initially. */
  @property({ type: Boolean, attribute: "selected" }) defaultSelected = false;

  _label: string = "";
  /**
   * The option’s plain text label.
   * Usually automatically generated, but can be useful to provide manually for cases involving complex content.
   */
  @property()
  set label(value) {
    const oldValue = this._label;
    this._label = value || "";

    if (this._label !== oldValue) {
      this.requestUpdate("label", oldValue);
    }
  }

  get label(): string {
    if (this._label) {
      return this._label;
    }

    return this.defaultLabel;
  }

  /** The default label, generated from the element contents. Will be equal to `label` in most cases. */
  get defaultLabel(): string {
    if (this.isDefaultLabelDirty || !this.cachedDefaultLabel) {
      this.updateDefaultLabel();
    }
    return this.cachedDefaultLabel;
  }

  connectedCallback() {
    super.connectedCallback();
    this.setAttribute("role", "option");
    this.setAttribute("aria-selected", "false");

    this.addEventListener("mouseenter", this.handleHover);
    this.addEventListener("mouseleave", this.handleHover);
  }

  disconnectedCallback(): void {
    super.disconnectedCallback();

    this.removeEventListener("mouseenter", this.handleHover);
    this.removeEventListener("mouseleave", this.handleHover);
  }

  private handleDefaultSlotChange() {
    // Mark the default label as needing recalculation
    this.isDefaultLabelDirty = true;

    if (this.isInitialized) {
      // When the label changes, tell the parent <wa-select> to update. The parent's handleDefaultSlotChange already
      // calls selectionChanged() internally, so we don't need to call it separately here.
      customElements.whenDefined("wa-select").then(() => {
        const controller = this.closest("wa-select");
        if (controller) {
          controller.handleDefaultSlotChange();
        }
      });

      // When the label changes, tell the parent <wa-combobox> to update
      customElements.whenDefined("wa-combobox").then(() => {
        // We cast to <wa-select> because it shares the same API as combobox
        const controller = this.closest<WaSelect>("wa-combobox");
        if (controller) {
          controller.handleDefaultSlotChange();
        }
      });
    } else {
      this.isInitialized = true;
    }
  }

  private handleHover = (event: Event) => {
    // We need this because Safari doesn't honor :hover styles while dragging
    // Test case: https://codepen.io/leaverou/pen/VYZOOjy
    if (event.type === "mouseenter") {
      this.customStates.set("hover", true);
    } else if (event.type === "mouseleave") {
      this.customStates.set("hover", false);
    }
  };

  protected willUpdate(changedProperties: PropertyValues<this>): void {
    if (changedProperties.has("defaultSelected")) {
      // We cast to <wa-select> because it shares the same API as combobox
      if (!this.closest<WaSelect>("wa-combobox, wa-select")?.hasInteracted) {
        // Only sync if defaultSelected is becoming true
        // This prevents overwriting `selected` when it was set directly by frameworks like Vue
        if (this.defaultSelected) {
          const oldVal = this.selected;
          this.selected = this.defaultSelected;
          this.requestUpdate("selected", oldVal);
        }
      }
    }
    super.willUpdate(changedProperties);
  }

  updated(changedProperties: PropertyValues<this>) {
    super.updated(changedProperties);

    if (changedProperties.has("disabled")) {
      this.setAttribute("aria-disabled", this.disabled ? "true" : "false");
      this.customStates.set("disabled", this.disabled);
    }

    if (changedProperties.has("selected")) {
      this.setAttribute("aria-selected", this.selected ? "true" : "false");
      this.customStates.set("selected", this.selected);
    }

    if (changedProperties.has("value")) {
      // Ensure the value is a string. This ensures the next line doesn't error and allows framework users to pass numbers
      // instead of requiring them to cast the value to a string.
      if (typeof this.value !== "string") {
        this.value = String(this.value);
      }

      this.handleDefaultSlotChange();
    }

    if (changedProperties.has("current")) {
      this.customStates.set("current", this.current);
    }
  }

  protected firstUpdated(changedProperties: PropertyValues<this>) {
    super.firstUpdated(changedProperties);

    // If the `selected` property was set directly (e.g., by Vue's :selected binding),
    // notify the parent select to update its selection. This is needed because
    // Vue binds to the `selected` property instead of the `defaultSelected` property
    // when using `:selected="true"` syntax.
    if (this.selected && !this.defaultSelected) {
      const parent = this.closest<WaSelect>("wa-select, wa-combobox");
      if (parent && !parent.hasInteracted) {
        parent.selectionChanged?.();
      }
    }
  }

  private updateDefaultLabel() {
    let oldValue = this.cachedDefaultLabel;
    this.cachedDefaultLabel = getText(this).trim();
    this.isDefaultLabelDirty = false;
    let changed = this.cachedDefaultLabel !== oldValue;

    if (!this._label && changed) {
      // Uses default label, and it has changed
      this.requestUpdate("label", oldValue);
    }

    return changed;
  }

  render() {
    return html`
      ${
        this.selected
          ? html`<wa-icon
              part="checked-icon"
              class="check"
              name="check"
              library="system"
              variant="solid"
              aria-hidden="true"
            ></wa-icon>`
          : html`<span
              part="checked-icon"
              class="check"
              aria-hidden="true"
            ></span>`
      }
      <slot part="start" name="start" class="start"></slot>
      <slot
        part="label"
        class="label"
        @slotchange=${this.handleDefaultSlotChange}
      ></slot>
      <slot part="end" name="end" class="end"></slot>
    `;
  }
}

declare global {
  interface HTMLElementTagNameMap {
    "wa-option": WaOption;
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
@@ -220,24 +220,22 @@
   }
 
   render() {
     return html`
-      ${
-        this.selected
-          ? html`
-              <wa-icon
-                part="checked-icon"
-                class="check"
-                name="check"
-                library="system"
-                variant="solid"
-                aria-hidden="true"
-              ></wa-icon>
-            `
-          : html`
-              <span part="checked-icon" class="check" aria-hidden="true"></span>
-            `
-      }
+      ${this.selected
+        ? html`
+            <wa-icon
+              part="checked-icon"
+              class="check"
+              name="check"
+              library="system"
+              variant="solid"
+              aria-hidden="true"
+            ></wa-icon>
+          `
+        : html`
+            <span part="checked-icon" class="check" aria-hidden="true"></span>
+          `}
       <slot part="start" name="start" class="start"></slot>
       <slot part="label" class="label" @slotchange=${this.handleDefaultSlotChange}></slot>
       <slot part="end" name="end" class="end"></slot>
     `;

`````

### Actual (oxfmt)

`````ts
import type { PropertyValues } from "lit";
import { html } from "lit";
import { customElement, property, query, state } from "lit/decorators.js";
import getText from "../../internal/get-text.js";
import WebAwesomeElement from "../../internal/webawesome-element.js";
import { LocalizeController } from "../../utilities/localize.js";
import "../icon/icon.js";
import type WaSelect from "../select/select.js";
import styles from "./option.styles.js";

/**
 * @summary Options represent the individual choices inside a select or similar form control. Each option holds a value
 *  and the label shown to the user.
 * @documentation https://webawesome.com/docs/components/option
 * @status stable
 * @since 2.0
 *
 * @dependency wa-icon
 *
 * @slot - The option's label.
 * @slot start - An element, such as `<wa-icon>`, placed before the label.
 * @slot end - An element, such as `<wa-icon>`, placed after the label.
 *
 * @csspart checked-icon - The checked icon, a `<wa-icon>` element.
 * @csspart label - The option's label.
 * @csspart start - The container that wraps the `start` slot.
 * @csspart end - The container that wraps the `end` slot.
 *
 * @cssstate current - The user has keyed into the option, but hasn't selected it yet (shows a highlight)
 * @cssstate selected - The option is selected and has aria-selected="true"
 * @cssstate disabled - Applied when the option is disabled
 * @cssstate hover - Like `:hover` but works while dragging in Safari
 */
@customElement("wa-option")
export default class WaOption extends WebAwesomeElement {
  static css = styles;

  // @ts-expect-error - Controller is currently unused
  private readonly localize = new LocalizeController(this);
  private cachedDefaultLabel = "";
  private isInitialized = false;
  private isDefaultLabelDirty = true;

  @query(".label") defaultSlot: HTMLSlotElement;

  // Set via the parent select
  @state() current = false;

  /**
   * The option's value. When selected, the containing form control will receive this value. The value must be unique
   * from other options in the same group. Values may not contain spaces, as spaces are used as delimiters when listing
   * multiple values.
   */
  @property({ reflect: true }) value = "";

  /** Draws the option in a disabled state, preventing selection. */
  @property({ type: Boolean }) disabled = false;

  /** @internal */
  @property({ type: Boolean, attribute: false }) selected = false;

  /** Selects an option initially. */
  @property({ type: Boolean, attribute: "selected" }) defaultSelected = false;

  _label: string = "";
  /**
   * The option’s plain text label.
   * Usually automatically generated, but can be useful to provide manually for cases involving complex content.
   */
  @property()
  set label(value) {
    const oldValue = this._label;
    this._label = value || "";

    if (this._label !== oldValue) {
      this.requestUpdate("label", oldValue);
    }
  }

  get label(): string {
    if (this._label) {
      return this._label;
    }

    return this.defaultLabel;
  }

  /** The default label, generated from the element contents. Will be equal to `label` in most cases. */
  get defaultLabel(): string {
    if (this.isDefaultLabelDirty || !this.cachedDefaultLabel) {
      this.updateDefaultLabel();
    }
    return this.cachedDefaultLabel;
  }

  connectedCallback() {
    super.connectedCallback();
    this.setAttribute("role", "option");
    this.setAttribute("aria-selected", "false");

    this.addEventListener("mouseenter", this.handleHover);
    this.addEventListener("mouseleave", this.handleHover);
  }

  disconnectedCallback(): void {
    super.disconnectedCallback();

    this.removeEventListener("mouseenter", this.handleHover);
    this.removeEventListener("mouseleave", this.handleHover);
  }

  private handleDefaultSlotChange() {
    // Mark the default label as needing recalculation
    this.isDefaultLabelDirty = true;

    if (this.isInitialized) {
      // When the label changes, tell the parent <wa-select> to update. The parent's handleDefaultSlotChange already
      // calls selectionChanged() internally, so we don't need to call it separately here.
      customElements.whenDefined("wa-select").then(() => {
        const controller = this.closest("wa-select");
        if (controller) {
          controller.handleDefaultSlotChange();
        }
      });

      // When the label changes, tell the parent <wa-combobox> to update
      customElements.whenDefined("wa-combobox").then(() => {
        // We cast to <wa-select> because it shares the same API as combobox
        const controller = this.closest<WaSelect>("wa-combobox");
        if (controller) {
          controller.handleDefaultSlotChange();
        }
      });
    } else {
      this.isInitialized = true;
    }
  }

  private handleHover = (event: Event) => {
    // We need this because Safari doesn't honor :hover styles while dragging
    // Test case: https://codepen.io/leaverou/pen/VYZOOjy
    if (event.type === "mouseenter") {
      this.customStates.set("hover", true);
    } else if (event.type === "mouseleave") {
      this.customStates.set("hover", false);
    }
  };

  protected willUpdate(changedProperties: PropertyValues<this>): void {
    if (changedProperties.has("defaultSelected")) {
      // We cast to <wa-select> because it shares the same API as combobox
      if (!this.closest<WaSelect>("wa-combobox, wa-select")?.hasInteracted) {
        // Only sync if defaultSelected is becoming true
        // This prevents overwriting `selected` when it was set directly by frameworks like Vue
        if (this.defaultSelected) {
          const oldVal = this.selected;
          this.selected = this.defaultSelected;
          this.requestUpdate("selected", oldVal);
        }
      }
    }
    super.willUpdate(changedProperties);
  }

  updated(changedProperties: PropertyValues<this>) {
    super.updated(changedProperties);

    if (changedProperties.has("disabled")) {
      this.setAttribute("aria-disabled", this.disabled ? "true" : "false");
      this.customStates.set("disabled", this.disabled);
    }

    if (changedProperties.has("selected")) {
      this.setAttribute("aria-selected", this.selected ? "true" : "false");
      this.customStates.set("selected", this.selected);
    }

    if (changedProperties.has("value")) {
      // Ensure the value is a string. This ensures the next line doesn't error and allows framework users to pass numbers
      // instead of requiring them to cast the value to a string.
      if (typeof this.value !== "string") {
        this.value = String(this.value);
      }

      this.handleDefaultSlotChange();
    }

    if (changedProperties.has("current")) {
      this.customStates.set("current", this.current);
    }
  }

  protected firstUpdated(changedProperties: PropertyValues<this>) {
    super.firstUpdated(changedProperties);

    // If the `selected` property was set directly (e.g., by Vue's :selected binding),
    // notify the parent select to update its selection. This is needed because
    // Vue binds to the `selected` property instead of the `defaultSelected` property
    // when using `:selected="true"` syntax.
    if (this.selected && !this.defaultSelected) {
      const parent = this.closest<WaSelect>("wa-select, wa-combobox");
      if (parent && !parent.hasInteracted) {
        parent.selectionChanged?.();
      }
    }
  }

  private updateDefaultLabel() {
    let oldValue = this.cachedDefaultLabel;
    this.cachedDefaultLabel = getText(this).trim();
    this.isDefaultLabelDirty = false;
    let changed = this.cachedDefaultLabel !== oldValue;

    if (!this._label && changed) {
      // Uses default label, and it has changed
      this.requestUpdate("label", oldValue);
    }

    return changed;
  }

  render() {
    return html`
      ${this.selected
        ? html`
            <wa-icon
              part="checked-icon"
              class="check"
              name="check"
              library="system"
              variant="solid"
              aria-hidden="true"
            ></wa-icon>
          `
        : html`
            <span part="checked-icon" class="check" aria-hidden="true"></span>
          `}
      <slot part="start" name="start" class="start"></slot>
      <slot part="label" class="label" @slotchange=${this.handleDefaultSlotChange}></slot>
      <slot part="end" name="end" class="end"></slot>
    `;
  }
}

declare global {
  interface HTMLElementTagNameMap {
    "wa-option": WaOption;
  }
}

`````

### Expected (prettier)

`````ts
import type { PropertyValues } from "lit";
import { html } from "lit";
import { customElement, property, query, state } from "lit/decorators.js";
import getText from "../../internal/get-text.js";
import WebAwesomeElement from "../../internal/webawesome-element.js";
import { LocalizeController } from "../../utilities/localize.js";
import "../icon/icon.js";
import type WaSelect from "../select/select.js";
import styles from "./option.styles.js";

/**
 * @summary Options represent the individual choices inside a select or similar form control. Each option holds a value
 *  and the label shown to the user.
 * @documentation https://webawesome.com/docs/components/option
 * @status stable
 * @since 2.0
 *
 * @dependency wa-icon
 *
 * @slot - The option's label.
 * @slot start - An element, such as `<wa-icon>`, placed before the label.
 * @slot end - An element, such as `<wa-icon>`, placed after the label.
 *
 * @csspart checked-icon - The checked icon, a `<wa-icon>` element.
 * @csspart label - The option's label.
 * @csspart start - The container that wraps the `start` slot.
 * @csspart end - The container that wraps the `end` slot.
 *
 * @cssstate current - The user has keyed into the option, but hasn't selected it yet (shows a highlight)
 * @cssstate selected - The option is selected and has aria-selected="true"
 * @cssstate disabled - Applied when the option is disabled
 * @cssstate hover - Like `:hover` but works while dragging in Safari
 */
@customElement("wa-option")
export default class WaOption extends WebAwesomeElement {
  static css = styles;

  // @ts-expect-error - Controller is currently unused
  private readonly localize = new LocalizeController(this);
  private cachedDefaultLabel = "";
  private isInitialized = false;
  private isDefaultLabelDirty = true;

  @query(".label") defaultSlot: HTMLSlotElement;

  // Set via the parent select
  @state() current = false;

  /**
   * The option's value. When selected, the containing form control will receive this value. The value must be unique
   * from other options in the same group. Values may not contain spaces, as spaces are used as delimiters when listing
   * multiple values.
   */
  @property({ reflect: true }) value = "";

  /** Draws the option in a disabled state, preventing selection. */
  @property({ type: Boolean }) disabled = false;

  /** @internal */
  @property({ type: Boolean, attribute: false }) selected = false;

  /** Selects an option initially. */
  @property({ type: Boolean, attribute: "selected" }) defaultSelected = false;

  _label: string = "";
  /**
   * The option’s plain text label.
   * Usually automatically generated, but can be useful to provide manually for cases involving complex content.
   */
  @property()
  set label(value) {
    const oldValue = this._label;
    this._label = value || "";

    if (this._label !== oldValue) {
      this.requestUpdate("label", oldValue);
    }
  }

  get label(): string {
    if (this._label) {
      return this._label;
    }

    return this.defaultLabel;
  }

  /** The default label, generated from the element contents. Will be equal to `label` in most cases. */
  get defaultLabel(): string {
    if (this.isDefaultLabelDirty || !this.cachedDefaultLabel) {
      this.updateDefaultLabel();
    }
    return this.cachedDefaultLabel;
  }

  connectedCallback() {
    super.connectedCallback();
    this.setAttribute("role", "option");
    this.setAttribute("aria-selected", "false");

    this.addEventListener("mouseenter", this.handleHover);
    this.addEventListener("mouseleave", this.handleHover);
  }

  disconnectedCallback(): void {
    super.disconnectedCallback();

    this.removeEventListener("mouseenter", this.handleHover);
    this.removeEventListener("mouseleave", this.handleHover);
  }

  private handleDefaultSlotChange() {
    // Mark the default label as needing recalculation
    this.isDefaultLabelDirty = true;

    if (this.isInitialized) {
      // When the label changes, tell the parent <wa-select> to update. The parent's handleDefaultSlotChange already
      // calls selectionChanged() internally, so we don't need to call it separately here.
      customElements.whenDefined("wa-select").then(() => {
        const controller = this.closest("wa-select");
        if (controller) {
          controller.handleDefaultSlotChange();
        }
      });

      // When the label changes, tell the parent <wa-combobox> to update
      customElements.whenDefined("wa-combobox").then(() => {
        // We cast to <wa-select> because it shares the same API as combobox
        const controller = this.closest<WaSelect>("wa-combobox");
        if (controller) {
          controller.handleDefaultSlotChange();
        }
      });
    } else {
      this.isInitialized = true;
    }
  }

  private handleHover = (event: Event) => {
    // We need this because Safari doesn't honor :hover styles while dragging
    // Test case: https://codepen.io/leaverou/pen/VYZOOjy
    if (event.type === "mouseenter") {
      this.customStates.set("hover", true);
    } else if (event.type === "mouseleave") {
      this.customStates.set("hover", false);
    }
  };

  protected willUpdate(changedProperties: PropertyValues<this>): void {
    if (changedProperties.has("defaultSelected")) {
      // We cast to <wa-select> because it shares the same API as combobox
      if (!this.closest<WaSelect>("wa-combobox, wa-select")?.hasInteracted) {
        // Only sync if defaultSelected is becoming true
        // This prevents overwriting `selected` when it was set directly by frameworks like Vue
        if (this.defaultSelected) {
          const oldVal = this.selected;
          this.selected = this.defaultSelected;
          this.requestUpdate("selected", oldVal);
        }
      }
    }
    super.willUpdate(changedProperties);
  }

  updated(changedProperties: PropertyValues<this>) {
    super.updated(changedProperties);

    if (changedProperties.has("disabled")) {
      this.setAttribute("aria-disabled", this.disabled ? "true" : "false");
      this.customStates.set("disabled", this.disabled);
    }

    if (changedProperties.has("selected")) {
      this.setAttribute("aria-selected", this.selected ? "true" : "false");
      this.customStates.set("selected", this.selected);
    }

    if (changedProperties.has("value")) {
      // Ensure the value is a string. This ensures the next line doesn't error and allows framework users to pass numbers
      // instead of requiring them to cast the value to a string.
      if (typeof this.value !== "string") {
        this.value = String(this.value);
      }

      this.handleDefaultSlotChange();
    }

    if (changedProperties.has("current")) {
      this.customStates.set("current", this.current);
    }
  }

  protected firstUpdated(changedProperties: PropertyValues<this>) {
    super.firstUpdated(changedProperties);

    // If the `selected` property was set directly (e.g., by Vue's :selected binding),
    // notify the parent select to update its selection. This is needed because
    // Vue binds to the `selected` property instead of the `defaultSelected` property
    // when using `:selected="true"` syntax.
    if (this.selected && !this.defaultSelected) {
      const parent = this.closest<WaSelect>("wa-select, wa-combobox");
      if (parent && !parent.hasInteracted) {
        parent.selectionChanged?.();
      }
    }
  }

  private updateDefaultLabel() {
    let oldValue = this.cachedDefaultLabel;
    this.cachedDefaultLabel = getText(this).trim();
    this.isDefaultLabelDirty = false;
    let changed = this.cachedDefaultLabel !== oldValue;

    if (!this._label && changed) {
      // Uses default label, and it has changed
      this.requestUpdate("label", oldValue);
    }

    return changed;
  }

  render() {
    return html`
      ${
        this.selected
          ? html`
              <wa-icon
                part="checked-icon"
                class="check"
                name="check"
                library="system"
                variant="solid"
                aria-hidden="true"
              ></wa-icon>
            `
          : html`
              <span part="checked-icon" class="check" aria-hidden="true"></span>
            `
      }
      <slot part="start" name="start" class="start"></slot>
      <slot part="label" class="label" @slotchange=${this.handleDefaultSlotChange}></slot>
      <slot part="end" name="end" class="end"></slot>
    `;
  }
}

declare global {
  interface HTMLElementTagNameMap {
    "wa-option": WaOption;
  }
}

`````
