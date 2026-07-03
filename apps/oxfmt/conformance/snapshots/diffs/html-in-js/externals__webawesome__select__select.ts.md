# externals/webawesome/select/select.ts

## Option 1

`````json
{"printWidth":80}
`````

### Diff

`````diff
===================================================================
--- prettier
+++ oxfmt
@@ -231,9 +231,16 @@
   }
 
   /** The select's size. */
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
@@ -265,9 +272,11 @@
   @property({ type: Boolean, reflect: true }) open = false;
 
   /** The select's visual appearance. */
   @property({ reflect: true }) appearance:
-    "filled" | "outlined" | "filled-outlined" = "outlined";
+    | "filled"
+    | "outlined"
+    | "filled-outlined" = "outlined";
 
   /** Draws a pill-style select with rounded edges. */
   @property({ type: Boolean, reflect: true }) pill = false;
 
@@ -319,9 +328,10 @@
         ?pill=${this.pill}
         size=${this.size}
         with-remove
         data-value=${option.value}
-        @wa-remove=${(event: WaRemoveEvent) => this.handleTagRemove(event, option)}
+        @wa-remove=${(event: WaRemoveEvent) =>
+          this.handleTagRemove(event, option)}
       >
         ${option.label}
       </wa-tag>
     `;
@@ -1133,53 +1143,51 @@
                 @focus=${this.handleFocus}
               />
 
               <!-- Tags need to wait for first hydration before populating otherwise it will create a hydration mismatch. -->
-              ${
-                this.multiple && this.hasUpdated
-                  ? html`<div
-                      part="tags"
-                      class="tags"
-                      @wa-remove=${this.handleTagRemove}
-                    >
-                      ${this.tags}
-                    </div>`
-                  : ""
-              }
+              ${this.multiple && this.hasUpdated
+                ? html`<div
+                    part="tags"
+                    class="tags"
+                    @wa-remove=${this.handleTagRemove}
+                  >
+                    ${this.tags}
+                  </div>`
+                : ""}
 
               <input
                 class="value-input"
                 type="text"
                 ?disabled=${this.disabled}
                 ?required=${this.required}
-                .value=${Array.isArray(this.value) ? this.value.join(", ") : this.value}
+                .value=${Array.isArray(this.value)
+                  ? this.value.join(", ")
+                  : this.value}
                 tabindex="-1"
                 aria-hidden="true"
                 @focus=${() => this.focus()}
               />
 
-              ${
-                hasClearIcon
-                  ? html`
-                      <button
-                        part="clear-button"
-                        type="button"
-                        aria-label=${this.localize.term("clearEntry")}
-                        @mousedown=${this.handleClearMouseDown}
-                        @click=${this.handleClearClick}
-                        tabindex="-1"
-                      >
-                        <slot name="clear-icon">
-                          <wa-icon
-                            name="circle-xmark"
-                            library="system"
-                            variant="regular"
-                          ></wa-icon>
-                        </slot>
-                      </button>
-                    `
-                  : ""
-              }
+              ${hasClearIcon
+                ? html`
+                    <button
+                      part="clear-button"
+                      type="button"
+                      aria-label=${this.localize.term("clearEntry")}
+                      @mousedown=${this.handleClearMouseDown}
+                      @click=${this.handleClearClick}
+                      tabindex="-1"
+                    >
+                      <slot name="clear-icon">
+                        <wa-icon
+                          name="circle-xmark"
+                          library="system"
+                          variant="regular"
+                        ></wa-icon>
+                      </slot>
+                    </button>
+                  `
+                : ""}
 
               <slot name="end" part="end" class="end"></slot>
 
               <slot name="expand-icon" part="expand-icon" class="expand-icon">

`````

### Actual (oxfmt)

`````ts
import type { PropertyValues, TemplateResult } from "lit";
import { html, isServer } from "lit";
import { customElement, property, query, state } from "lit/decorators.js";
import { classMap } from "lit/directives/class-map.js";
import { unsafeHTML } from "lit/directives/unsafe-html.js";
import { WaAfterHideEvent } from "../../events/after-hide.js";
import { WaAfterShowEvent } from "../../events/after-show.js";
import { WaClearEvent } from "../../events/clear.js";
import { WaHideEvent } from "../../events/hide.js";
import { WaRemoveEvent } from "../../events/remove.js";
import { WaShowEvent } from "../../events/show.js";
import { animateWithClass } from "../../internal/animate.js";
import {
  isTopDismissible,
  registerDismissible,
  unregisterDismissible,
} from "../../internal/dismissible-stack.js";
import { waitForEvent } from "../../internal/event.js";
import { scrollIntoView } from "../../internal/scroll.js";
import { warnDeprecatedSize } from "../../internal/size.js";
import { HasSlotController } from "../../internal/slot.js";
import { RequiredValidator } from "../../internal/validators/required-validator.js";
import { watch } from "../../internal/watch.js";
import { WebAwesomeFormAssociatedElement } from "../../internal/webawesome-form-associated-element.js";
import formControlStyles from "../../styles/component/form-control.styles.js";
import sizeStyles from "../../styles/component/size.styles.js";
import { LocalizeController } from "../../utilities/localize.js";
import "../icon/icon.js";
import "../option/option.js";
import type WaOption from "../option/option.js";
import "../popup/popup.js";
import type WaPopup from "../popup/popup.js";
import "../tag/tag.js";
import styles from "./select.styles.js";

/**
 * @summary Selects let users choose one or more values from a dropdown list of predefined options. Use them in forms
 *  when a fixed set of choices needs to fit in limited space.
 * @documentation https://webawesome.com/docs/components/select
 * @status stable
 * @since 2.0
 *
 * @dependency wa-icon
 * @dependency wa-popup
 * @dependency wa-tag
 * @dependency wa-option
 *
 * @slot - The listbox options. Must be `<wa-option>` elements. You can use `<wa-divider>` to group items visually.
 * @slot label - The input's label. Alternatively, you can use the `label` attribute.
 * @slot start - An element, such as `<wa-icon>`, placed at the start of the combobox.
 * @slot end - An element, such as `<wa-icon>`, placed at the end of the combobox.
 * @slot clear-icon - An icon to use in lieu of the default clear icon.
 * @slot expand-icon - The icon to show when the control is expanded and collapsed. Rotates on open and close.
 * @slot hint - Text that describes how to use the input. Alternatively, you can use the `hint` attribute.
 *
 * @event change - Emitted when the control's value changes.
 * @event input - Emitted when the control receives input.
 * @event focus - Emitted when the control gains focus.
 * @event blur - Emitted when the control loses focus.
 * @event wa-clear - Emitted when the control's value is cleared.
 * @event wa-show - Emitted when the select's menu opens.
 * @event wa-after-show - Emitted after the select's menu opens and all animations are complete.
 * @event wa-hide - Emitted when the select's menu closes.
 * @event wa-after-hide - Emitted after the select's menu closes and all animations are complete.
 * @event wa-invalid - Emitted when the form control has been checked for validity and its constraints aren't satisfied.
 *
 * @csspart form-control - The form control that wraps the label, input, and hint.
 * @csspart form-control-label - The label's wrapper.
 * @csspart form-control-input - The select's wrapper.
 * @csspart hint - The hint's wrapper.
 * @csspart combobox - The container the wraps the start, end, value, clear icon, and expand button.
 * @csspart start - The container that wraps the `start` slot.
 * @csspart end - The container that wraps the `end` slot.
 * @csspart display-input - The element that displays the selected option's label, an `<input>` element.
 * @csspart listbox - The listbox container where options are slotted.
 * @csspart tags - The container that houses option tags when `multiselect` is used.
 * @csspart tag - The individual tags that represent each multiselect option.
 * @csspart tag__content - The tag's content part.
 * @csspart tag__remove-button - The tag's remove button.
 * @csspart tag__remove-button__base - The tag's remove button base part.
 * @csspart clear-button - The clear button.
 * @csspart expand-icon - The container that wraps the expand icon.
 *
 * @cssproperty [--show-duration=100ms] - The duration of the show animation.
 * @cssproperty [--hide-duration=100ms] - The duration of the hide animation.
 * @cssproperty [--tag-max-size=10ch] - When using `multiple`, the max size of tags before their content is truncated.
 *
 * @cssstate blank - The select is empty.
 */
@customElement("wa-select")
export default class WaSelect extends WebAwesomeFormAssociatedElement {
  static css = [styles, formControlStyles, sizeStyles];

  static get validators() {
    const validators = isServer
      ? []
      : [
          RequiredValidator({
            validationElement: Object.assign(document.createElement("select"), {
              required: true,
            }),
          }),
        ];
    return [...super.validators, ...validators];
  }

  assumeInteractionOn = ["blur", "input"];

  private cachedOptions: WaOption[] | null = null;
  private readonly hasSlotController = new HasSlotController(
    this,
    "hint",
    "label",
  );
  private readonly localize = new LocalizeController(this);
  private selectionOrder: Map<string, number> = new Map();
  private typeToSelectString = "";
  private typeToSelectTimeout: number;
  private slotChangePending = false;

  @query(".select") popup: WaPopup;
  @query(".combobox") combobox: HTMLSlotElement;
  @query(".display-input") displayInput: HTMLInputElement;
  @query(".value-input") valueInput: HTMLInputElement;
  @query(".listbox") listbox: HTMLSlotElement;

  /** Where to anchor native constraint validation */
  get validationTarget() {
    return this.valueInput;
  }

  @state() displayLabel = "";
  @state() currentOption: WaOption;
  @state() selectedOptions: WaOption[] = [];
  /** @internal */
  optionValues: Set<string | null> | undefined;

  /** The name of the select, submitted as a name/value pair with form data. */
  @property({ reflect: true }) name = "";

  private _defaultValue: null | string | string[] = null;

  @property({
    attribute: false,
  })
  set defaultValue(val: null | string | string[]) {
    this._defaultValue = this.convertDefaultValue(val);
  }

  get defaultValue() {
    return this.convertDefaultValue(this._defaultValue);
  }

  private rawValuesEqual(
    a: string[] | null | undefined,
    b: string[] | null | undefined,
  ): boolean {
    if (a == null && b == null) return true;
    if (a == null || b == null) return false;
    if (a.length !== b.length) return false;
    return a.every((v, i) => v === b[i]);
  }

  /**
   * @private
   * A converter for defaultValue from array to string if its multiple. Also fixes some hydration issues.
   */
  private convertDefaultValue(val: typeof this.defaultValue) {
    // For some reason this can go off before we've fully updated. So check the attribute too.
    const isMultiple = this.multiple || this.hasAttribute("multiple");

    if (!isMultiple && Array.isArray(val)) {
      val = val[0];
    }

    return val;
  }

  private _value: string[] | undefined | null;

  /** The select's value. This will be a string for single select or an array for multi-select. */
  @property({ attribute: "value", reflect: false })
  set value(val: string | string[] | null) {
    let oldValue = this.value;

    if ((val as any) instanceof FormData) {
      val = (val as unknown as FormData).getAll(this.name) as string[];
    }

    if (val != null && !Array.isArray(val)) {
      val = [val];
    }

    const oldRawValue = this._value;
    this._value = val ?? null;

    // Compare raw internal values to detect actual changes. We can't rely on the getter because it filters through
    // optionValues, which may be empty when options aren't in the DOM yet (common with frameworks that set properties
    // before appending children).
    if (!this.rawValuesEqual(oldRawValue, this._value)) {
      this.valueHasChanged = true;
      this.requestUpdate("value", oldValue);
    }
  }

  get value() {
    let value = this._value ?? this.defaultValue ?? null;

    if (value != null) {
      value = Array.isArray(value) ? value : [value];
    }

    // Build optionValues from the cached options list. This is rebuilt each time the getter is called rather than
    // cached, because caching created stale-state bugs when the value was set before options existed in the DOM. The
    // underlying getAllOptions() is already cached via cachedOptions, so this is cheap.
    this.optionValues = new Set(
      this.getAllOptions()
        .filter((option) => !option.disabled)
        .map((option) => option.value),
    );

    // Drop values not in the DOM
    let ret: null | string | string[] = value;
    if (value != null) {
      ret = value.filter((v) => this.optionValues!.has(v));
      ret = this.multiple ? ret : ret[0];
      ret = ret ?? null;
    }

    return ret;
  }

  /** The select's size. */
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

  /** Placeholder text to show as a hint when the select is empty. */
  @property() placeholder = "";

  /** Allows more than one option to be selected. */
  @property({ type: Boolean, reflect: true }) multiple = false;

  /**
   * The maximum number of selected options to show when `multiple` is true. After the maximum, "+n" will be shown to
   * indicate the number of additional items that are selected. Set to 0 to remove the limit.
   */
  @property({ attribute: "max-options-visible", type: Number })
  maxOptionsVisible = 3;

  /** Disables the select control. */
  @property({ type: Boolean }) disabled = false;

  /** Adds a clear button when the select is not empty. */
  @property({ attribute: "with-clear", type: Boolean }) withClear = false;

  /**
   * Indicates whether or not the select is open. You can toggle this attribute to show and hide the menu, or you can
   * use the `show()` and `hide()` methods and this attribute will reflect the select's open state.
   */
  @property({ type: Boolean, reflect: true }) open = false;

  /** The select's visual appearance. */
  @property({ reflect: true }) appearance:
    | "filled"
    | "outlined"
    | "filled-outlined" = "outlined";

  /** Draws a pill-style select with rounded edges. */
  @property({ type: Boolean, reflect: true }) pill = false;

  /** The select's label. If you need to display HTML, use the `label` slot instead. */
  @property() label = "";

  /**
   * The preferred placement of the select's menu. Note that the actual placement may vary as needed to keep the listbox
   * inside of the viewport.
   */
  @property({ reflect: true }) placement: "top" | "bottom" = "bottom";

  /** The select's hint. If you need to display HTML, use the `hint` slot instead. */
  @property({ attribute: "hint" }) hint = "";

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

  /** The select's required attribute. */
  @property({ type: Boolean, reflect: true }) required = false;

  /**
   * A function that customizes the tags to be rendered when multiple=true. The first argument is the option, the second
   * is the current tag's index.  The function should return either a Lit TemplateResult or a string containing trusted
   * HTML of the symbol to render at the specified value.
   */
  @property({ attribute: false }) getTag: (
    option: WaOption,
    index: number,
  ) => TemplateResult | string | HTMLElement = (option) => {
    return html`
      <wa-tag
        part="tag"
        exportparts="
            base:tag__base,
            content:tag__content,
            remove-button:tag__remove-button,
            remove-button__base:tag__remove-button__base
          "
        ?pill=${this.pill}
        size=${this.size}
        with-remove
        data-value=${option.value}
        @wa-remove=${(event: WaRemoveEvent) =>
          this.handleTagRemove(event, option)}
      >
        ${option.label}
      </wa-tag>
    `;
  };

  connectedCallback() {
    super.connectedCallback();

    // Call processSlotChange directly so initial setup is synchronous.
    // Subsequent option additions will be batched via handleDefaultSlotChange.
    this.processSlotChange();

    // Because this is a form control, it shouldn't be opened initially
    this.open = false;
  }

  disconnectedCallback() {
    super.disconnectedCallback();
    this.removeOpenListeners();
    this.cachedOptions = null;
  }

  private updateDefaultValue() {
    const allOptions = this.getAllOptions();
    const defaultSelectedOptions = allOptions.filter(
      (el) => el.hasAttribute("selected") || el.defaultSelected,
    );
    if (defaultSelectedOptions.length > 0) {
      const selectedValues = defaultSelectedOptions.map((el) => el.value);
      this._defaultValue = this.multiple ? selectedValues : selectedValues[0];
    }
    if (this.hasAttribute("value")) {
      this._defaultValue = this.getAttribute("value") || null;
    }
  }

  private addOpenListeners() {
    //
    // Listen on the root node instead of the document in case the elements are inside a shadow root
    //
    // https://github.com/shoelace-style/shoelace/issues/1763
    //
    document.addEventListener("focusin", this.handleDocumentFocusIn);
    document.addEventListener("keydown", this.handleDocumentKeyDown);
    document.addEventListener("mousedown", this.handleDocumentMouseDown);
    registerDismissible(this);

    // If the component is rendered in a shadow root, we need to attach the focusin listener there too
    if (this.getRootNode() !== document) {
      this.getRootNode().addEventListener(
        "focusin",
        this.handleDocumentFocusIn,
      );
    }
  }

  private removeOpenListeners() {
    document.removeEventListener("focusin", this.handleDocumentFocusIn);
    document.removeEventListener("keydown", this.handleDocumentKeyDown);
    document.removeEventListener("mousedown", this.handleDocumentMouseDown);
    unregisterDismissible(this);

    if (this.getRootNode() !== document) {
      this.getRootNode().removeEventListener(
        "focusin",
        this.handleDocumentFocusIn,
      );
    }
  }

  private handleFocus() {
    this.displayInput.setSelectionRange(0, 0);
  }

  private handleDocumentFocusIn = (event: KeyboardEvent) => {
    // Close when focusing out of the select
    const path = event.composedPath();
    if (this && !path.includes(this)) {
      this.hide();
    }
  };

  private handleDocumentKeyDown = (event: KeyboardEvent) => {
    const target = event.target as HTMLElement;
    const isClearButton = target.closest('[part~="clear-button"]') !== null;
    const isButton = target.closest("wa-button") !== null;

    // Ignore presses when the target is a button (e.g. the remove button in `<wa-tag>`)
    if (isClearButton || isButton) {
      return;
    }

    // Close when pressing escape
    if (event.key === "Escape" && this.open && isTopDismissible(this)) {
      event.preventDefault();
      event.stopPropagation();
      this.hide();
      this.displayInput.focus({ preventScroll: true });
    }

    // Handle enter and space. When pressing space, we allow for type to select behaviors so if there's anything in the
    // buffer we _don't_ close it.
    if (
      event.key === "Enter" ||
      (event.key === " " && this.typeToSelectString === "")
    ) {
      event.preventDefault();
      event.stopImmediatePropagation();

      // If it's not open, open it
      if (!this.open) {
        this.show();
        return;
      }

      // If it is open, update the value based on the current selection and close it
      if (this.currentOption && !this.currentOption.disabled) {
        this.valueHasChanged = true;
        this.hasInteracted = true;
        if (this.multiple) {
          this.toggleOptionSelection(this.currentOption);
        } else {
          this.setSelectedOptions(this.currentOption);
        }

        // Emit after updating
        this.updateComplete.then(() => {
          this.dispatchEvent(
            new InputEvent("input", { bubbles: true, composed: true }),
          );
          this.dispatchEvent(
            new Event("change", { bubbles: true, composed: true }),
          );
        });

        if (!this.multiple) {
          this.hide();
          this.displayInput.focus({ preventScroll: true });
        }
      }

      return;
    }

    // Navigate options
    if (["ArrowUp", "ArrowDown", "Home", "End"].includes(event.key)) {
      const allOptions = this.getAllOptions();
      const currentIndex = allOptions.indexOf(this.currentOption);
      let newIndex = Math.max(0, currentIndex);

      // Prevent scrolling
      event.preventDefault();

      // Open it
      if (!this.open) {
        this.show();

        // If an option is already selected, stop here because we want that one to remain highlighted when the listbox
        // opens for the first time
        if (this.currentOption) {
          return;
        }
      }

      if (event.key === "ArrowDown") {
        newIndex = currentIndex + 1;
        if (newIndex > allOptions.length - 1) newIndex = 0;
      } else if (event.key === "ArrowUp") {
        newIndex = currentIndex - 1;
        if (newIndex < 0) newIndex = allOptions.length - 1;
      } else if (event.key === "Home") {
        newIndex = 0;
      } else if (event.key === "End") {
        newIndex = allOptions.length - 1;
      }

      this.setCurrentOption(allOptions[newIndex]);
    }

    // All other "printable" keys trigger type to select
    if (event.key?.length === 1 || event.key === "Backspace") {
      const allOptions = this.getAllOptions();

      // Don't block important key combos like CMD+R
      if (event.metaKey || event.ctrlKey || event.altKey) {
        return;
      }

      // Open, unless the key that triggered is backspace
      if (!this.open) {
        if (event.key === "Backspace") {
          return;
        }

        this.show();
      }

      event.stopPropagation();
      event.preventDefault();

      clearTimeout(this.typeToSelectTimeout);
      this.typeToSelectTimeout = window.setTimeout(
        () => (this.typeToSelectString = ""),
        1000,
      );

      if (event.key === "Backspace") {
        this.typeToSelectString = this.typeToSelectString.slice(0, -1);
      } else {
        this.typeToSelectString += event.key.toLowerCase();
      }

      for (const option of allOptions) {
        const label = option.label.toLowerCase();

        if (label.startsWith(this.typeToSelectString)) {
          this.setCurrentOption(option);
          break;
        }
      }
    }
  };

  private handleDocumentMouseDown = (event: MouseEvent) => {
    // Close when clicking outside of the select
    const path = event.composedPath();
    if (this && !path.includes(this)) {
      this.hide();
    }
  };

  private handleLabelClick() {
    this.displayInput.focus();
  }

  private handleComboboxClick(event: MouseEvent) {
    event.preventDefault();
  }

  private handleComboboxMouseDown(event: MouseEvent) {
    const path = event.composedPath();
    const isButton = path.some(
      (el) => el instanceof Element && el.tagName.toLowerCase() === "wa-button",
    );

    // Ignore disabled controls and clicks on tags (remove buttons)
    if (this.disabled || isButton) {
      return;
    }

    event.preventDefault();
    this.displayInput.focus({ preventScroll: true });
    this.open = !this.open;
  }

  private handleComboboxKeyDown(event: KeyboardEvent) {
    event.stopPropagation();
    this.handleDocumentKeyDown(event);
  }

  private handleClearClick(event: MouseEvent) {
    event.stopPropagation();

    this.hasInteracted = true;
    this.valueHasChanged = true;

    if (this.value !== null) {
      this.displayLabel = "";
      this.selectionOrder.clear();
      this.setSelectedOptions([]);
      this.displayInput.focus({ preventScroll: true });

      // Emit after update
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
  }

  private handleClearMouseDown(event: MouseEvent) {
    // Don't lose focus or propagate events when clicking the clear button
    event.stopPropagation();
    event.preventDefault();
  }

  private handleOptionClick(event: MouseEvent) {
    const target = event.target as HTMLElement;
    const option = target.closest("wa-option");

    if (option && !option.disabled) {
      this.hasInteracted = true;
      this.valueHasChanged = true;

      if (this.multiple) {
        this.toggleOptionSelection(option);
      } else {
        this.setSelectedOptions(option);
      }

      // Set focus after updating so the value is announced by screen readers
      this.updateComplete.then(() =>
        this.displayInput.focus({ preventScroll: true }),
      );

      this.requestUpdate("value");

      // Emit after updating
      this.updateComplete.then(() => {
        this.dispatchEvent(
          new InputEvent("input", { bubbles: true, composed: true }),
        );
        this.dispatchEvent(
          new Event("change", { bubbles: true, composed: true }),
        );
      });

      if (!this.multiple) {
        this.hide();
        this.displayInput.focus({ preventScroll: true });
      }
    }
  }

  /* @internal - used by options to update labels */
  public handleDefaultSlotChange() {
    if (this.slotChangePending) return;
    this.slotChangePending = true;
    queueMicrotask(() => {
      this.slotChangePending = false;
      this.processSlotChange();
    });
  }

  private processSlotChange() {
    if (!customElements.get("wa-option")) {
      customElements
        .whenDefined("wa-option")
        .then(() => this.handleDefaultSlotChange());
    }

    // Invalidate the options cache since slots have changed
    this.cachedOptions = null;

    const allOptions = this.getAllOptions();

    // Update defaultValue if it hasn't been explicitly set and we have selected options
    this.updateDefaultValue();

    let value = this.value;

    if (value == null || (!this.valueHasChanged && !this.hasInteracted)) {
      this.selectionChanged();
      return;
    }

    if (!Array.isArray(value)) {
      value = [value];
    }

    // Select only the options that match the new value
    const selectedOptions = allOptions.filter((el) => value.includes(el.value));
    this.setSelectedOptions(selectedOptions);
  }

  private handleTagRemove(event: WaRemoveEvent, directOption?: WaOption) {
    event.stopPropagation();

    if (this.disabled) return;

    // Mark as interacted so selectionChanged() uses the correct filter logic
    this.hasInteracted = true;
    this.valueHasChanged = true;

    // Use the directly provided option if available (from getTag method)
    let option = directOption;

    // If no direct option was provided, find the option from the data-value attribute
    if (!option) {
      const tagElement = (event.target as Element).closest(
        "wa-tag[data-value]",
      ) as HTMLElement | null;

      if (tagElement) {
        const value = tagElement.dataset.value;
        option = this.selectedOptions.find((opt) => opt.value === value);
      }
    }

    if (option) {
      this.toggleOptionSelection(option, false);

      // Emit after updating
      this.updateComplete.then(() => {
        this.dispatchEvent(
          new InputEvent("input", { bubbles: true, composed: true }),
        );
        this.dispatchEvent(
          new Event("change", { bubbles: true, composed: true }),
        );
      });
    }
  }

  // Gets an array of all `<wa-option>` elements
  private getAllOptions() {
    if (this.cachedOptions) return this.cachedOptions;
    if (!this?.querySelectorAll) {
      return [];
    }
    this.cachedOptions = [...this.querySelectorAll<WaOption>("wa-option")];
    return this.cachedOptions;
  }

  // Gets the first `<wa-option>` element
  private getFirstOption() {
    return this.querySelector<WaOption>("wa-option");
  }

  // Sets the current option, which is the option the user is currently interacting with (e.g. via keyboard). Only one
  // option may be "current" at a time.
  private setCurrentOption(option: WaOption | null) {
    const allOptions = this.getAllOptions();

    // Clear selection
    allOptions.forEach((el) => {
      el.current = false;
      el.tabIndex = -1;
    });

    // Select the target option
    if (option) {
      this.currentOption = option;
      option.current = true;
      option.tabIndex = 0;
      option.focus({ preventScroll: true });
    }
  }

  // Sets the selected option(s)
  private setSelectedOptions(option: WaOption | WaOption[]) {
    const allOptions = this.getAllOptions();
    const newSelectedOptions = Array.isArray(option) ? option : [option];

    // Clear existing selection
    allOptions.forEach((el) => {
      if (newSelectedOptions.includes(el)) {
        return;
      }
      el.selected = false;
    });

    // Set the new selection
    if (newSelectedOptions.length) {
      newSelectedOptions.forEach((el) => (el.selected = true));
    }

    // Update selection, value, and display label
    this.selectionChanged();
  }

  // Toggles an option's selected state
  private toggleOptionSelection(option: WaOption, force?: boolean) {
    if (force === true || force === false) {
      option.selected = force;
    } else {
      option.selected = !option.selected;
    }

    this.selectionChanged();
  }

  // @internal This method must be called whenever the selection changes. It will update the selected options cache, the
  // current value, and the display value. The option component uses it internally to update labels as they change.
  public selectionChanged() {
    const options = this.getAllOptions();

    // Update selected options cache
    const newSelectedOptions = options.filter((el) => {
      if (!this.hasInteracted && !this.valueHasChanged) {
        const defaultValue = this.defaultValue;
        const defaultValues = Array.isArray(defaultValue)
          ? defaultValue
          : [defaultValue];
        return (
          el.hasAttribute("selected") ||
          el.defaultSelected ||
          el.selected ||
          defaultValues?.includes(el.value)
        );
      }

      return el.selected;
    });

    // Update the selection order map
    const newSelectedValues = new Set(newSelectedOptions.map((el) => el.value));

    // Remove deselected options from the order map
    for (const value of this.selectionOrder.keys()) {
      if (!newSelectedValues.has(value)) {
        this.selectionOrder.delete(value);
      }
    }

    // Add newly selected options
    const maxOrder =
      this.selectionOrder.size > 0
        ? Math.max(...this.selectionOrder.values())
        : -1;
    let nextOrder = maxOrder + 1;
    for (const option of newSelectedOptions) {
      if (!this.selectionOrder.has(option.value)) {
        this.selectionOrder.set(option.value, nextOrder++);
      }
    }

    // Sort options by selection order
    this.selectedOptions = newSelectedOptions.sort((a, b) => {
      const orderA = this.selectionOrder.get(a.value) ?? 0;
      const orderB = this.selectionOrder.get(b.value) ?? 0;
      return orderA - orderB;
    });

    let selectedValues = new Set(this.selectedOptions.map((el) => el.value));

    // Toggle values present in the DOM from this.value, while preserving options NOT present in the DOM (for lazy loading)
    // Note that options NOT present in the DOM will be moved to the end after this
    if (selectedValues.size > 0 || this._value) {
      const oldValue = this._value;
      if (this._value == null) {
        // First time it's set
        let value = this.defaultValue ?? [];
        this._value = Array.isArray(value) ? value : [value];
      }

      // Filter out values that are in the DOM
      this._value =
        this._value?.filter((value) => !this.optionValues?.has(value)) ?? null;
      this._value?.unshift(...selectedValues);
      this.requestUpdate("value", oldValue);
    }

    // Update the value and display label
    if (this.multiple) {
      if (this.placeholder && !this.value?.length) {
        // When no items are selected, keep the value empty so the placeholder shows
        this.displayLabel = "";
      } else {
        this.displayLabel = this.localize.term(
          "numOptionsSelected",
          this.selectedOptions.length,
        );
      }
    } else {
      const selectedOption = this.selectedOptions[0];
      this.displayLabel = selectedOption?.label ?? "";
    }

    // Update validity
    this.updateComplete.then(() => {
      this.updateValidity();
    });
  }

  protected get tags() {
    return this.selectedOptions.map((option, index) => {
      if (index < this.maxOptionsVisible || this.maxOptionsVisible <= 0) {
        const tag = this.getTag(option, index);
        if (!tag) return null;
        return typeof tag === "string" ? unsafeHTML(tag) : tag;
      } else if (index === this.maxOptionsVisible) {
        // Hit tag limit
        return html`
          <wa-tag
            part="tag"
            exportparts="
              base:tag__base,
              content:tag__content,
              remove-button:tag__remove-button,
              remove-button__base:tag__remove-button__base
            "
            >+${this.selectedOptions.length - index}</wa-tag
          >
        `;
      }
      return null;
    });
  }

  updated(changedProperties: PropertyValues<this>) {
    super.updated(changedProperties);

    if (
      changedProperties.has("value") ||
      changedProperties.has("displayLabel")
    ) {
      this.customStates.set("blank", !this.value && !this.displayLabel);
    }
  }

  @watch("disabled", { waitUntilFirstUpdate: true })
  handleDisabledChange() {
    // Close the listbox when the control is open and disabled
    if (this.disabled && this.open) {
      this.open = false;
    }
  }

  @watch("value", { waitUntilFirstUpdate: true })
  handleValueChange() {
    const allOptions = this.getAllOptions();
    const value = Array.isArray(this.value) ? this.value : [this.value];

    // Select only the options that match the new value
    const selectedOptions = allOptions.filter((el) => value.includes(el.value));
    this.setSelectedOptions(selectedOptions);
    this.updateValidity();
  }

  @watch("open", { waitUntilFirstUpdate: true })
  async handleOpenChange() {
    if (this.open && !this.disabled) {
      // Reset the current option
      this.setCurrentOption(this.selectedOptions[0] || this.getFirstOption());

      // Show
      const waShowEvent = new WaShowEvent();
      this.dispatchEvent(waShowEvent);
      if (waShowEvent.defaultPrevented) {
        this.open = false;
        return;
      }

      this.addOpenListeners();
      this.listbox.hidden = false;
      this.popup.active = true;

      // Select the appropriate option based on value after the listbox opens
      requestAnimationFrame(() => {
        this.setCurrentOption(this.currentOption);
      });

      await animateWithClass(this.popup.popup, "show");

      // Make sure the current option is scrolled into view (required for Safari)
      if (this.currentOption) {
        scrollIntoView(this.currentOption, this.listbox, "vertical", "auto");
      }

      this.dispatchEvent(new WaAfterShowEvent());
    } else {
      // Hide
      const waHideEvent = new WaHideEvent();
      this.dispatchEvent(waHideEvent);
      if (waHideEvent.defaultPrevented) {
        this.open = false;
        return;
      }

      this.removeOpenListeners();
      await animateWithClass(this.popup.popup, "hide");
      this.listbox.hidden = true;
      this.popup.active = false;

      this.dispatchEvent(new WaAfterHideEvent());
    }
  }

  /** Shows the listbox. */
  async show() {
    if (this.open || this.disabled) {
      this.open = false;
      return undefined;
    }

    this.open = true;
    return waitForEvent(this, "wa-after-show");
  }

  /** Hides the listbox. */
  async hide() {
    if (!this.open || this.disabled) {
      this.open = false;
      return undefined;
    }

    this.open = false;
    return waitForEvent(this, "wa-after-hide");
  }

  /** Sets focus on the control. */
  focus(options?: FocusOptions) {
    this.displayInput.focus(options);
  }

  /** Removes focus from the control. */
  blur() {
    this.displayInput.blur();
  }

  formResetCallback() {
    this.selectionOrder.clear();
    this.value = this.defaultValue;
    super.formResetCallback();
    this.handleValueChange();

    this.updateComplete.then(() => {
      this.dispatchEvent(
        new InputEvent("input", { bubbles: true, composed: true }),
      );
      this.dispatchEvent(
        new Event("change", { bubbles: true, composed: true }),
      );
    });
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
    const hasClearIcon =
      (this.hasUpdated || isServer) &&
      this.withClear &&
      !this.disabled &&
      (this.displayLabel || (this.value && this.value.length > 0));

    return html`
      <div
        part="form-control"
        class=${classMap({
          "form-control": true,
          "form-control-has-label": hasLabel,
        })}
      >
        <label
          id="label"
          part="form-control-label label"
          class=${classMap({
            label: true,
            "has-label": hasLabel,
          })}
          aria-hidden=${hasLabel ? "false" : "true"}
          @click=${this.handleLabelClick}
        >
          <slot name="label">${this.label}</slot>
        </label>

        <div part="form-control-input" class="form-control-input">
          <wa-popup
            class=${classMap({
              select: true,
              open: this.open,
              disabled: this.disabled,
              enabled: !this.disabled,
              multiple: this.multiple,
            })}
            placement=${this.placement}
            flip
            shift
            sync="width"
            auto-size="vertical"
            auto-size-padding="10"
          >
            <div
              part="combobox"
              class="combobox"
              slot="anchor"
              @keydown=${this.handleComboboxKeyDown}
              @mousedown=${this.handleComboboxMouseDown}
              @click=${this.handleComboboxClick}
            >
              <slot part="start" name="start" class="start"></slot>

              <input
                part="display-input"
                class="display-input"
                type="text"
                placeholder=${this.placeholder}
                .disabled=${this.disabled}
                .value=${this.displayLabel}
                ?required=${this.required}
                autocomplete="off"
                spellcheck="false"
                autocapitalize="off"
                readonly
                aria-invalid=${
                  !this.validity.valid
                  /** aria-invalid is required because readonly inputs are technically always valid so it never reads 'invalid data' for screen readers. */
                }
                aria-controls="listbox"
                aria-expanded=${this.open ? "true" : "false"}
                aria-haspopup="listbox"
                aria-labelledby="label"
                aria-disabled=${this.disabled ? "true" : "false"}
                aria-describedby="hint"
                role="combobox"
                tabindex="0"
                @focus=${this.handleFocus}
              />

              <!-- Tags need to wait for first hydration before populating otherwise it will create a hydration mismatch. -->
              ${this.multiple && this.hasUpdated
                ? html`<div
                    part="tags"
                    class="tags"
                    @wa-remove=${this.handleTagRemove}
                  >
                    ${this.tags}
                  </div>`
                : ""}

              <input
                class="value-input"
                type="text"
                ?disabled=${this.disabled}
                ?required=${this.required}
                .value=${Array.isArray(this.value)
                  ? this.value.join(", ")
                  : this.value}
                tabindex="-1"
                aria-hidden="true"
                @focus=${() => this.focus()}
              />

              ${hasClearIcon
                ? html`
                    <button
                      part="clear-button"
                      type="button"
                      aria-label=${this.localize.term("clearEntry")}
                      @mousedown=${this.handleClearMouseDown}
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

              <slot name="end" part="end" class="end"></slot>

              <slot name="expand-icon" part="expand-icon" class="expand-icon">
                <wa-icon
                  library="system"
                  name="chevron-down"
                  variant="solid"
                ></wa-icon>
              </slot>
            </div>

            <div
              id="listbox"
              role="listbox"
              aria-expanded=${this.open ? "true" : "false"}
              aria-multiselectable=${this.multiple ? "true" : "false"}
              aria-labelledby="label"
              part="listbox"
              class="listbox"
              tabindex="-1"
              @mouseup=${this.handleOptionClick}
            >
              <slot @slotchange=${this.handleDefaultSlotChange}></slot>
            </div>
          </wa-popup>
        </div>

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
      </div>
    `;
  }
}

// The change-in-update warning is required for this component because the form-associated base class calls
// updateValidity() in firstUpdated(), which triggers requestUpdate('validity') to sync the validation state after the
// first render when the validation target is available. Additionally, HasSlotController triggers requestUpdate() on
// initial slotchange events, and selectionChanged() sets @state properties (displayLabel, selectedOptions) in response
// to slot content changes. See https://lit.dev/docs/tools/development/#development-build-runtime-warnings
WaSelect.disableWarning?.("change-in-update");

declare global {
  interface HTMLElementTagNameMap {
    "wa-select": WaSelect;
  }
}

`````

### Expected (prettier)

`````ts
import type { PropertyValues, TemplateResult } from "lit";
import { html, isServer } from "lit";
import { customElement, property, query, state } from "lit/decorators.js";
import { classMap } from "lit/directives/class-map.js";
import { unsafeHTML } from "lit/directives/unsafe-html.js";
import { WaAfterHideEvent } from "../../events/after-hide.js";
import { WaAfterShowEvent } from "../../events/after-show.js";
import { WaClearEvent } from "../../events/clear.js";
import { WaHideEvent } from "../../events/hide.js";
import { WaRemoveEvent } from "../../events/remove.js";
import { WaShowEvent } from "../../events/show.js";
import { animateWithClass } from "../../internal/animate.js";
import {
  isTopDismissible,
  registerDismissible,
  unregisterDismissible,
} from "../../internal/dismissible-stack.js";
import { waitForEvent } from "../../internal/event.js";
import { scrollIntoView } from "../../internal/scroll.js";
import { warnDeprecatedSize } from "../../internal/size.js";
import { HasSlotController } from "../../internal/slot.js";
import { RequiredValidator } from "../../internal/validators/required-validator.js";
import { watch } from "../../internal/watch.js";
import { WebAwesomeFormAssociatedElement } from "../../internal/webawesome-form-associated-element.js";
import formControlStyles from "../../styles/component/form-control.styles.js";
import sizeStyles from "../../styles/component/size.styles.js";
import { LocalizeController } from "../../utilities/localize.js";
import "../icon/icon.js";
import "../option/option.js";
import type WaOption from "../option/option.js";
import "../popup/popup.js";
import type WaPopup from "../popup/popup.js";
import "../tag/tag.js";
import styles from "./select.styles.js";

/**
 * @summary Selects let users choose one or more values from a dropdown list of predefined options. Use them in forms
 *  when a fixed set of choices needs to fit in limited space.
 * @documentation https://webawesome.com/docs/components/select
 * @status stable
 * @since 2.0
 *
 * @dependency wa-icon
 * @dependency wa-popup
 * @dependency wa-tag
 * @dependency wa-option
 *
 * @slot - The listbox options. Must be `<wa-option>` elements. You can use `<wa-divider>` to group items visually.
 * @slot label - The input's label. Alternatively, you can use the `label` attribute.
 * @slot start - An element, such as `<wa-icon>`, placed at the start of the combobox.
 * @slot end - An element, such as `<wa-icon>`, placed at the end of the combobox.
 * @slot clear-icon - An icon to use in lieu of the default clear icon.
 * @slot expand-icon - The icon to show when the control is expanded and collapsed. Rotates on open and close.
 * @slot hint - Text that describes how to use the input. Alternatively, you can use the `hint` attribute.
 *
 * @event change - Emitted when the control's value changes.
 * @event input - Emitted when the control receives input.
 * @event focus - Emitted when the control gains focus.
 * @event blur - Emitted when the control loses focus.
 * @event wa-clear - Emitted when the control's value is cleared.
 * @event wa-show - Emitted when the select's menu opens.
 * @event wa-after-show - Emitted after the select's menu opens and all animations are complete.
 * @event wa-hide - Emitted when the select's menu closes.
 * @event wa-after-hide - Emitted after the select's menu closes and all animations are complete.
 * @event wa-invalid - Emitted when the form control has been checked for validity and its constraints aren't satisfied.
 *
 * @csspart form-control - The form control that wraps the label, input, and hint.
 * @csspart form-control-label - The label's wrapper.
 * @csspart form-control-input - The select's wrapper.
 * @csspart hint - The hint's wrapper.
 * @csspart combobox - The container the wraps the start, end, value, clear icon, and expand button.
 * @csspart start - The container that wraps the `start` slot.
 * @csspart end - The container that wraps the `end` slot.
 * @csspart display-input - The element that displays the selected option's label, an `<input>` element.
 * @csspart listbox - The listbox container where options are slotted.
 * @csspart tags - The container that houses option tags when `multiselect` is used.
 * @csspart tag - The individual tags that represent each multiselect option.
 * @csspart tag__content - The tag's content part.
 * @csspart tag__remove-button - The tag's remove button.
 * @csspart tag__remove-button__base - The tag's remove button base part.
 * @csspart clear-button - The clear button.
 * @csspart expand-icon - The container that wraps the expand icon.
 *
 * @cssproperty [--show-duration=100ms] - The duration of the show animation.
 * @cssproperty [--hide-duration=100ms] - The duration of the hide animation.
 * @cssproperty [--tag-max-size=10ch] - When using `multiple`, the max size of tags before their content is truncated.
 *
 * @cssstate blank - The select is empty.
 */
@customElement("wa-select")
export default class WaSelect extends WebAwesomeFormAssociatedElement {
  static css = [styles, formControlStyles, sizeStyles];

  static get validators() {
    const validators = isServer
      ? []
      : [
          RequiredValidator({
            validationElement: Object.assign(document.createElement("select"), {
              required: true,
            }),
          }),
        ];
    return [...super.validators, ...validators];
  }

  assumeInteractionOn = ["blur", "input"];

  private cachedOptions: WaOption[] | null = null;
  private readonly hasSlotController = new HasSlotController(
    this,
    "hint",
    "label",
  );
  private readonly localize = new LocalizeController(this);
  private selectionOrder: Map<string, number> = new Map();
  private typeToSelectString = "";
  private typeToSelectTimeout: number;
  private slotChangePending = false;

  @query(".select") popup: WaPopup;
  @query(".combobox") combobox: HTMLSlotElement;
  @query(".display-input") displayInput: HTMLInputElement;
  @query(".value-input") valueInput: HTMLInputElement;
  @query(".listbox") listbox: HTMLSlotElement;

  /** Where to anchor native constraint validation */
  get validationTarget() {
    return this.valueInput;
  }

  @state() displayLabel = "";
  @state() currentOption: WaOption;
  @state() selectedOptions: WaOption[] = [];
  /** @internal */
  optionValues: Set<string | null> | undefined;

  /** The name of the select, submitted as a name/value pair with form data. */
  @property({ reflect: true }) name = "";

  private _defaultValue: null | string | string[] = null;

  @property({
    attribute: false,
  })
  set defaultValue(val: null | string | string[]) {
    this._defaultValue = this.convertDefaultValue(val);
  }

  get defaultValue() {
    return this.convertDefaultValue(this._defaultValue);
  }

  private rawValuesEqual(
    a: string[] | null | undefined,
    b: string[] | null | undefined,
  ): boolean {
    if (a == null && b == null) return true;
    if (a == null || b == null) return false;
    if (a.length !== b.length) return false;
    return a.every((v, i) => v === b[i]);
  }

  /**
   * @private
   * A converter for defaultValue from array to string if its multiple. Also fixes some hydration issues.
   */
  private convertDefaultValue(val: typeof this.defaultValue) {
    // For some reason this can go off before we've fully updated. So check the attribute too.
    const isMultiple = this.multiple || this.hasAttribute("multiple");

    if (!isMultiple && Array.isArray(val)) {
      val = val[0];
    }

    return val;
  }

  private _value: string[] | undefined | null;

  /** The select's value. This will be a string for single select or an array for multi-select. */
  @property({ attribute: "value", reflect: false })
  set value(val: string | string[] | null) {
    let oldValue = this.value;

    if ((val as any) instanceof FormData) {
      val = (val as unknown as FormData).getAll(this.name) as string[];
    }

    if (val != null && !Array.isArray(val)) {
      val = [val];
    }

    const oldRawValue = this._value;
    this._value = val ?? null;

    // Compare raw internal values to detect actual changes. We can't rely on the getter because it filters through
    // optionValues, which may be empty when options aren't in the DOM yet (common with frameworks that set properties
    // before appending children).
    if (!this.rawValuesEqual(oldRawValue, this._value)) {
      this.valueHasChanged = true;
      this.requestUpdate("value", oldValue);
    }
  }

  get value() {
    let value = this._value ?? this.defaultValue ?? null;

    if (value != null) {
      value = Array.isArray(value) ? value : [value];
    }

    // Build optionValues from the cached options list. This is rebuilt each time the getter is called rather than
    // cached, because caching created stale-state bugs when the value was set before options existed in the DOM. The
    // underlying getAllOptions() is already cached via cachedOptions, so this is cheap.
    this.optionValues = new Set(
      this.getAllOptions()
        .filter((option) => !option.disabled)
        .map((option) => option.value),
    );

    // Drop values not in the DOM
    let ret: null | string | string[] = value;
    if (value != null) {
      ret = value.filter((v) => this.optionValues!.has(v));
      ret = this.multiple ? ret : ret[0];
      ret = ret ?? null;
    }

    return ret;
  }

  /** The select's size. */
  @property({ reflect: true }) size:
    "xs" | "s" | "m" | "l" | "xl" | "small" | "medium" | "large" = "m";

  @watch("size")
  handleSizeChange() {
    warnDeprecatedSize(this.localName, this.size);
  }

  /** Placeholder text to show as a hint when the select is empty. */
  @property() placeholder = "";

  /** Allows more than one option to be selected. */
  @property({ type: Boolean, reflect: true }) multiple = false;

  /**
   * The maximum number of selected options to show when `multiple` is true. After the maximum, "+n" will be shown to
   * indicate the number of additional items that are selected. Set to 0 to remove the limit.
   */
  @property({ attribute: "max-options-visible", type: Number })
  maxOptionsVisible = 3;

  /** Disables the select control. */
  @property({ type: Boolean }) disabled = false;

  /** Adds a clear button when the select is not empty. */
  @property({ attribute: "with-clear", type: Boolean }) withClear = false;

  /**
   * Indicates whether or not the select is open. You can toggle this attribute to show and hide the menu, or you can
   * use the `show()` and `hide()` methods and this attribute will reflect the select's open state.
   */
  @property({ type: Boolean, reflect: true }) open = false;

  /** The select's visual appearance. */
  @property({ reflect: true }) appearance:
    "filled" | "outlined" | "filled-outlined" = "outlined";

  /** Draws a pill-style select with rounded edges. */
  @property({ type: Boolean, reflect: true }) pill = false;

  /** The select's label. If you need to display HTML, use the `label` slot instead. */
  @property() label = "";

  /**
   * The preferred placement of the select's menu. Note that the actual placement may vary as needed to keep the listbox
   * inside of the viewport.
   */
  @property({ reflect: true }) placement: "top" | "bottom" = "bottom";

  /** The select's hint. If you need to display HTML, use the `hint` slot instead. */
  @property({ attribute: "hint" }) hint = "";

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

  /** The select's required attribute. */
  @property({ type: Boolean, reflect: true }) required = false;

  /**
   * A function that customizes the tags to be rendered when multiple=true. The first argument is the option, the second
   * is the current tag's index.  The function should return either a Lit TemplateResult or a string containing trusted
   * HTML of the symbol to render at the specified value.
   */
  @property({ attribute: false }) getTag: (
    option: WaOption,
    index: number,
  ) => TemplateResult | string | HTMLElement = (option) => {
    return html`
      <wa-tag
        part="tag"
        exportparts="
            base:tag__base,
            content:tag__content,
            remove-button:tag__remove-button,
            remove-button__base:tag__remove-button__base
          "
        ?pill=${this.pill}
        size=${this.size}
        with-remove
        data-value=${option.value}
        @wa-remove=${(event: WaRemoveEvent) => this.handleTagRemove(event, option)}
      >
        ${option.label}
      </wa-tag>
    `;
  };

  connectedCallback() {
    super.connectedCallback();

    // Call processSlotChange directly so initial setup is synchronous.
    // Subsequent option additions will be batched via handleDefaultSlotChange.
    this.processSlotChange();

    // Because this is a form control, it shouldn't be opened initially
    this.open = false;
  }

  disconnectedCallback() {
    super.disconnectedCallback();
    this.removeOpenListeners();
    this.cachedOptions = null;
  }

  private updateDefaultValue() {
    const allOptions = this.getAllOptions();
    const defaultSelectedOptions = allOptions.filter(
      (el) => el.hasAttribute("selected") || el.defaultSelected,
    );
    if (defaultSelectedOptions.length > 0) {
      const selectedValues = defaultSelectedOptions.map((el) => el.value);
      this._defaultValue = this.multiple ? selectedValues : selectedValues[0];
    }
    if (this.hasAttribute("value")) {
      this._defaultValue = this.getAttribute("value") || null;
    }
  }

  private addOpenListeners() {
    //
    // Listen on the root node instead of the document in case the elements are inside a shadow root
    //
    // https://github.com/shoelace-style/shoelace/issues/1763
    //
    document.addEventListener("focusin", this.handleDocumentFocusIn);
    document.addEventListener("keydown", this.handleDocumentKeyDown);
    document.addEventListener("mousedown", this.handleDocumentMouseDown);
    registerDismissible(this);

    // If the component is rendered in a shadow root, we need to attach the focusin listener there too
    if (this.getRootNode() !== document) {
      this.getRootNode().addEventListener(
        "focusin",
        this.handleDocumentFocusIn,
      );
    }
  }

  private removeOpenListeners() {
    document.removeEventListener("focusin", this.handleDocumentFocusIn);
    document.removeEventListener("keydown", this.handleDocumentKeyDown);
    document.removeEventListener("mousedown", this.handleDocumentMouseDown);
    unregisterDismissible(this);

    if (this.getRootNode() !== document) {
      this.getRootNode().removeEventListener(
        "focusin",
        this.handleDocumentFocusIn,
      );
    }
  }

  private handleFocus() {
    this.displayInput.setSelectionRange(0, 0);
  }

  private handleDocumentFocusIn = (event: KeyboardEvent) => {
    // Close when focusing out of the select
    const path = event.composedPath();
    if (this && !path.includes(this)) {
      this.hide();
    }
  };

  private handleDocumentKeyDown = (event: KeyboardEvent) => {
    const target = event.target as HTMLElement;
    const isClearButton = target.closest('[part~="clear-button"]') !== null;
    const isButton = target.closest("wa-button") !== null;

    // Ignore presses when the target is a button (e.g. the remove button in `<wa-tag>`)
    if (isClearButton || isButton) {
      return;
    }

    // Close when pressing escape
    if (event.key === "Escape" && this.open && isTopDismissible(this)) {
      event.preventDefault();
      event.stopPropagation();
      this.hide();
      this.displayInput.focus({ preventScroll: true });
    }

    // Handle enter and space. When pressing space, we allow for type to select behaviors so if there's anything in the
    // buffer we _don't_ close it.
    if (
      event.key === "Enter" ||
      (event.key === " " && this.typeToSelectString === "")
    ) {
      event.preventDefault();
      event.stopImmediatePropagation();

      // If it's not open, open it
      if (!this.open) {
        this.show();
        return;
      }

      // If it is open, update the value based on the current selection and close it
      if (this.currentOption && !this.currentOption.disabled) {
        this.valueHasChanged = true;
        this.hasInteracted = true;
        if (this.multiple) {
          this.toggleOptionSelection(this.currentOption);
        } else {
          this.setSelectedOptions(this.currentOption);
        }

        // Emit after updating
        this.updateComplete.then(() => {
          this.dispatchEvent(
            new InputEvent("input", { bubbles: true, composed: true }),
          );
          this.dispatchEvent(
            new Event("change", { bubbles: true, composed: true }),
          );
        });

        if (!this.multiple) {
          this.hide();
          this.displayInput.focus({ preventScroll: true });
        }
      }

      return;
    }

    // Navigate options
    if (["ArrowUp", "ArrowDown", "Home", "End"].includes(event.key)) {
      const allOptions = this.getAllOptions();
      const currentIndex = allOptions.indexOf(this.currentOption);
      let newIndex = Math.max(0, currentIndex);

      // Prevent scrolling
      event.preventDefault();

      // Open it
      if (!this.open) {
        this.show();

        // If an option is already selected, stop here because we want that one to remain highlighted when the listbox
        // opens for the first time
        if (this.currentOption) {
          return;
        }
      }

      if (event.key === "ArrowDown") {
        newIndex = currentIndex + 1;
        if (newIndex > allOptions.length - 1) newIndex = 0;
      } else if (event.key === "ArrowUp") {
        newIndex = currentIndex - 1;
        if (newIndex < 0) newIndex = allOptions.length - 1;
      } else if (event.key === "Home") {
        newIndex = 0;
      } else if (event.key === "End") {
        newIndex = allOptions.length - 1;
      }

      this.setCurrentOption(allOptions[newIndex]);
    }

    // All other "printable" keys trigger type to select
    if (event.key?.length === 1 || event.key === "Backspace") {
      const allOptions = this.getAllOptions();

      // Don't block important key combos like CMD+R
      if (event.metaKey || event.ctrlKey || event.altKey) {
        return;
      }

      // Open, unless the key that triggered is backspace
      if (!this.open) {
        if (event.key === "Backspace") {
          return;
        }

        this.show();
      }

      event.stopPropagation();
      event.preventDefault();

      clearTimeout(this.typeToSelectTimeout);
      this.typeToSelectTimeout = window.setTimeout(
        () => (this.typeToSelectString = ""),
        1000,
      );

      if (event.key === "Backspace") {
        this.typeToSelectString = this.typeToSelectString.slice(0, -1);
      } else {
        this.typeToSelectString += event.key.toLowerCase();
      }

      for (const option of allOptions) {
        const label = option.label.toLowerCase();

        if (label.startsWith(this.typeToSelectString)) {
          this.setCurrentOption(option);
          break;
        }
      }
    }
  };

  private handleDocumentMouseDown = (event: MouseEvent) => {
    // Close when clicking outside of the select
    const path = event.composedPath();
    if (this && !path.includes(this)) {
      this.hide();
    }
  };

  private handleLabelClick() {
    this.displayInput.focus();
  }

  private handleComboboxClick(event: MouseEvent) {
    event.preventDefault();
  }

  private handleComboboxMouseDown(event: MouseEvent) {
    const path = event.composedPath();
    const isButton = path.some(
      (el) => el instanceof Element && el.tagName.toLowerCase() === "wa-button",
    );

    // Ignore disabled controls and clicks on tags (remove buttons)
    if (this.disabled || isButton) {
      return;
    }

    event.preventDefault();
    this.displayInput.focus({ preventScroll: true });
    this.open = !this.open;
  }

  private handleComboboxKeyDown(event: KeyboardEvent) {
    event.stopPropagation();
    this.handleDocumentKeyDown(event);
  }

  private handleClearClick(event: MouseEvent) {
    event.stopPropagation();

    this.hasInteracted = true;
    this.valueHasChanged = true;

    if (this.value !== null) {
      this.displayLabel = "";
      this.selectionOrder.clear();
      this.setSelectedOptions([]);
      this.displayInput.focus({ preventScroll: true });

      // Emit after update
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
  }

  private handleClearMouseDown(event: MouseEvent) {
    // Don't lose focus or propagate events when clicking the clear button
    event.stopPropagation();
    event.preventDefault();
  }

  private handleOptionClick(event: MouseEvent) {
    const target = event.target as HTMLElement;
    const option = target.closest("wa-option");

    if (option && !option.disabled) {
      this.hasInteracted = true;
      this.valueHasChanged = true;

      if (this.multiple) {
        this.toggleOptionSelection(option);
      } else {
        this.setSelectedOptions(option);
      }

      // Set focus after updating so the value is announced by screen readers
      this.updateComplete.then(() =>
        this.displayInput.focus({ preventScroll: true }),
      );

      this.requestUpdate("value");

      // Emit after updating
      this.updateComplete.then(() => {
        this.dispatchEvent(
          new InputEvent("input", { bubbles: true, composed: true }),
        );
        this.dispatchEvent(
          new Event("change", { bubbles: true, composed: true }),
        );
      });

      if (!this.multiple) {
        this.hide();
        this.displayInput.focus({ preventScroll: true });
      }
    }
  }

  /* @internal - used by options to update labels */
  public handleDefaultSlotChange() {
    if (this.slotChangePending) return;
    this.slotChangePending = true;
    queueMicrotask(() => {
      this.slotChangePending = false;
      this.processSlotChange();
    });
  }

  private processSlotChange() {
    if (!customElements.get("wa-option")) {
      customElements
        .whenDefined("wa-option")
        .then(() => this.handleDefaultSlotChange());
    }

    // Invalidate the options cache since slots have changed
    this.cachedOptions = null;

    const allOptions = this.getAllOptions();

    // Update defaultValue if it hasn't been explicitly set and we have selected options
    this.updateDefaultValue();

    let value = this.value;

    if (value == null || (!this.valueHasChanged && !this.hasInteracted)) {
      this.selectionChanged();
      return;
    }

    if (!Array.isArray(value)) {
      value = [value];
    }

    // Select only the options that match the new value
    const selectedOptions = allOptions.filter((el) => value.includes(el.value));
    this.setSelectedOptions(selectedOptions);
  }

  private handleTagRemove(event: WaRemoveEvent, directOption?: WaOption) {
    event.stopPropagation();

    if (this.disabled) return;

    // Mark as interacted so selectionChanged() uses the correct filter logic
    this.hasInteracted = true;
    this.valueHasChanged = true;

    // Use the directly provided option if available (from getTag method)
    let option = directOption;

    // If no direct option was provided, find the option from the data-value attribute
    if (!option) {
      const tagElement = (event.target as Element).closest(
        "wa-tag[data-value]",
      ) as HTMLElement | null;

      if (tagElement) {
        const value = tagElement.dataset.value;
        option = this.selectedOptions.find((opt) => opt.value === value);
      }
    }

    if (option) {
      this.toggleOptionSelection(option, false);

      // Emit after updating
      this.updateComplete.then(() => {
        this.dispatchEvent(
          new InputEvent("input", { bubbles: true, composed: true }),
        );
        this.dispatchEvent(
          new Event("change", { bubbles: true, composed: true }),
        );
      });
    }
  }

  // Gets an array of all `<wa-option>` elements
  private getAllOptions() {
    if (this.cachedOptions) return this.cachedOptions;
    if (!this?.querySelectorAll) {
      return [];
    }
    this.cachedOptions = [...this.querySelectorAll<WaOption>("wa-option")];
    return this.cachedOptions;
  }

  // Gets the first `<wa-option>` element
  private getFirstOption() {
    return this.querySelector<WaOption>("wa-option");
  }

  // Sets the current option, which is the option the user is currently interacting with (e.g. via keyboard). Only one
  // option may be "current" at a time.
  private setCurrentOption(option: WaOption | null) {
    const allOptions = this.getAllOptions();

    // Clear selection
    allOptions.forEach((el) => {
      el.current = false;
      el.tabIndex = -1;
    });

    // Select the target option
    if (option) {
      this.currentOption = option;
      option.current = true;
      option.tabIndex = 0;
      option.focus({ preventScroll: true });
    }
  }

  // Sets the selected option(s)
  private setSelectedOptions(option: WaOption | WaOption[]) {
    const allOptions = this.getAllOptions();
    const newSelectedOptions = Array.isArray(option) ? option : [option];

    // Clear existing selection
    allOptions.forEach((el) => {
      if (newSelectedOptions.includes(el)) {
        return;
      }
      el.selected = false;
    });

    // Set the new selection
    if (newSelectedOptions.length) {
      newSelectedOptions.forEach((el) => (el.selected = true));
    }

    // Update selection, value, and display label
    this.selectionChanged();
  }

  // Toggles an option's selected state
  private toggleOptionSelection(option: WaOption, force?: boolean) {
    if (force === true || force === false) {
      option.selected = force;
    } else {
      option.selected = !option.selected;
    }

    this.selectionChanged();
  }

  // @internal This method must be called whenever the selection changes. It will update the selected options cache, the
  // current value, and the display value. The option component uses it internally to update labels as they change.
  public selectionChanged() {
    const options = this.getAllOptions();

    // Update selected options cache
    const newSelectedOptions = options.filter((el) => {
      if (!this.hasInteracted && !this.valueHasChanged) {
        const defaultValue = this.defaultValue;
        const defaultValues = Array.isArray(defaultValue)
          ? defaultValue
          : [defaultValue];
        return (
          el.hasAttribute("selected") ||
          el.defaultSelected ||
          el.selected ||
          defaultValues?.includes(el.value)
        );
      }

      return el.selected;
    });

    // Update the selection order map
    const newSelectedValues = new Set(newSelectedOptions.map((el) => el.value));

    // Remove deselected options from the order map
    for (const value of this.selectionOrder.keys()) {
      if (!newSelectedValues.has(value)) {
        this.selectionOrder.delete(value);
      }
    }

    // Add newly selected options
    const maxOrder =
      this.selectionOrder.size > 0
        ? Math.max(...this.selectionOrder.values())
        : -1;
    let nextOrder = maxOrder + 1;
    for (const option of newSelectedOptions) {
      if (!this.selectionOrder.has(option.value)) {
        this.selectionOrder.set(option.value, nextOrder++);
      }
    }

    // Sort options by selection order
    this.selectedOptions = newSelectedOptions.sort((a, b) => {
      const orderA = this.selectionOrder.get(a.value) ?? 0;
      const orderB = this.selectionOrder.get(b.value) ?? 0;
      return orderA - orderB;
    });

    let selectedValues = new Set(this.selectedOptions.map((el) => el.value));

    // Toggle values present in the DOM from this.value, while preserving options NOT present in the DOM (for lazy loading)
    // Note that options NOT present in the DOM will be moved to the end after this
    if (selectedValues.size > 0 || this._value) {
      const oldValue = this._value;
      if (this._value == null) {
        // First time it's set
        let value = this.defaultValue ?? [];
        this._value = Array.isArray(value) ? value : [value];
      }

      // Filter out values that are in the DOM
      this._value =
        this._value?.filter((value) => !this.optionValues?.has(value)) ?? null;
      this._value?.unshift(...selectedValues);
      this.requestUpdate("value", oldValue);
    }

    // Update the value and display label
    if (this.multiple) {
      if (this.placeholder && !this.value?.length) {
        // When no items are selected, keep the value empty so the placeholder shows
        this.displayLabel = "";
      } else {
        this.displayLabel = this.localize.term(
          "numOptionsSelected",
          this.selectedOptions.length,
        );
      }
    } else {
      const selectedOption = this.selectedOptions[0];
      this.displayLabel = selectedOption?.label ?? "";
    }

    // Update validity
    this.updateComplete.then(() => {
      this.updateValidity();
    });
  }

  protected get tags() {
    return this.selectedOptions.map((option, index) => {
      if (index < this.maxOptionsVisible || this.maxOptionsVisible <= 0) {
        const tag = this.getTag(option, index);
        if (!tag) return null;
        return typeof tag === "string" ? unsafeHTML(tag) : tag;
      } else if (index === this.maxOptionsVisible) {
        // Hit tag limit
        return html`
          <wa-tag
            part="tag"
            exportparts="
              base:tag__base,
              content:tag__content,
              remove-button:tag__remove-button,
              remove-button__base:tag__remove-button__base
            "
            >+${this.selectedOptions.length - index}</wa-tag
          >
        `;
      }
      return null;
    });
  }

  updated(changedProperties: PropertyValues<this>) {
    super.updated(changedProperties);

    if (
      changedProperties.has("value") ||
      changedProperties.has("displayLabel")
    ) {
      this.customStates.set("blank", !this.value && !this.displayLabel);
    }
  }

  @watch("disabled", { waitUntilFirstUpdate: true })
  handleDisabledChange() {
    // Close the listbox when the control is open and disabled
    if (this.disabled && this.open) {
      this.open = false;
    }
  }

  @watch("value", { waitUntilFirstUpdate: true })
  handleValueChange() {
    const allOptions = this.getAllOptions();
    const value = Array.isArray(this.value) ? this.value : [this.value];

    // Select only the options that match the new value
    const selectedOptions = allOptions.filter((el) => value.includes(el.value));
    this.setSelectedOptions(selectedOptions);
    this.updateValidity();
  }

  @watch("open", { waitUntilFirstUpdate: true })
  async handleOpenChange() {
    if (this.open && !this.disabled) {
      // Reset the current option
      this.setCurrentOption(this.selectedOptions[0] || this.getFirstOption());

      // Show
      const waShowEvent = new WaShowEvent();
      this.dispatchEvent(waShowEvent);
      if (waShowEvent.defaultPrevented) {
        this.open = false;
        return;
      }

      this.addOpenListeners();
      this.listbox.hidden = false;
      this.popup.active = true;

      // Select the appropriate option based on value after the listbox opens
      requestAnimationFrame(() => {
        this.setCurrentOption(this.currentOption);
      });

      await animateWithClass(this.popup.popup, "show");

      // Make sure the current option is scrolled into view (required for Safari)
      if (this.currentOption) {
        scrollIntoView(this.currentOption, this.listbox, "vertical", "auto");
      }

      this.dispatchEvent(new WaAfterShowEvent());
    } else {
      // Hide
      const waHideEvent = new WaHideEvent();
      this.dispatchEvent(waHideEvent);
      if (waHideEvent.defaultPrevented) {
        this.open = false;
        return;
      }

      this.removeOpenListeners();
      await animateWithClass(this.popup.popup, "hide");
      this.listbox.hidden = true;
      this.popup.active = false;

      this.dispatchEvent(new WaAfterHideEvent());
    }
  }

  /** Shows the listbox. */
  async show() {
    if (this.open || this.disabled) {
      this.open = false;
      return undefined;
    }

    this.open = true;
    return waitForEvent(this, "wa-after-show");
  }

  /** Hides the listbox. */
  async hide() {
    if (!this.open || this.disabled) {
      this.open = false;
      return undefined;
    }

    this.open = false;
    return waitForEvent(this, "wa-after-hide");
  }

  /** Sets focus on the control. */
  focus(options?: FocusOptions) {
    this.displayInput.focus(options);
  }

  /** Removes focus from the control. */
  blur() {
    this.displayInput.blur();
  }

  formResetCallback() {
    this.selectionOrder.clear();
    this.value = this.defaultValue;
    super.formResetCallback();
    this.handleValueChange();

    this.updateComplete.then(() => {
      this.dispatchEvent(
        new InputEvent("input", { bubbles: true, composed: true }),
      );
      this.dispatchEvent(
        new Event("change", { bubbles: true, composed: true }),
      );
    });
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
    const hasClearIcon =
      (this.hasUpdated || isServer) &&
      this.withClear &&
      !this.disabled &&
      (this.displayLabel || (this.value && this.value.length > 0));

    return html`
      <div
        part="form-control"
        class=${classMap({
          "form-control": true,
          "form-control-has-label": hasLabel,
        })}
      >
        <label
          id="label"
          part="form-control-label label"
          class=${classMap({
            label: true,
            "has-label": hasLabel,
          })}
          aria-hidden=${hasLabel ? "false" : "true"}
          @click=${this.handleLabelClick}
        >
          <slot name="label">${this.label}</slot>
        </label>

        <div part="form-control-input" class="form-control-input">
          <wa-popup
            class=${classMap({
              select: true,
              open: this.open,
              disabled: this.disabled,
              enabled: !this.disabled,
              multiple: this.multiple,
            })}
            placement=${this.placement}
            flip
            shift
            sync="width"
            auto-size="vertical"
            auto-size-padding="10"
          >
            <div
              part="combobox"
              class="combobox"
              slot="anchor"
              @keydown=${this.handleComboboxKeyDown}
              @mousedown=${this.handleComboboxMouseDown}
              @click=${this.handleComboboxClick}
            >
              <slot part="start" name="start" class="start"></slot>

              <input
                part="display-input"
                class="display-input"
                type="text"
                placeholder=${this.placeholder}
                .disabled=${this.disabled}
                .value=${this.displayLabel}
                ?required=${this.required}
                autocomplete="off"
                spellcheck="false"
                autocapitalize="off"
                readonly
                aria-invalid=${
                  !this.validity.valid
                  /** aria-invalid is required because readonly inputs are technically always valid so it never reads 'invalid data' for screen readers. */
                }
                aria-controls="listbox"
                aria-expanded=${this.open ? "true" : "false"}
                aria-haspopup="listbox"
                aria-labelledby="label"
                aria-disabled=${this.disabled ? "true" : "false"}
                aria-describedby="hint"
                role="combobox"
                tabindex="0"
                @focus=${this.handleFocus}
              />

              <!-- Tags need to wait for first hydration before populating otherwise it will create a hydration mismatch. -->
              ${
                this.multiple && this.hasUpdated
                  ? html`<div
                      part="tags"
                      class="tags"
                      @wa-remove=${this.handleTagRemove}
                    >
                      ${this.tags}
                    </div>`
                  : ""
              }

              <input
                class="value-input"
                type="text"
                ?disabled=${this.disabled}
                ?required=${this.required}
                .value=${Array.isArray(this.value) ? this.value.join(", ") : this.value}
                tabindex="-1"
                aria-hidden="true"
                @focus=${() => this.focus()}
              />

              ${
                hasClearIcon
                  ? html`
                      <button
                        part="clear-button"
                        type="button"
                        aria-label=${this.localize.term("clearEntry")}
                        @mousedown=${this.handleClearMouseDown}
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

              <slot name="end" part="end" class="end"></slot>

              <slot name="expand-icon" part="expand-icon" class="expand-icon">
                <wa-icon
                  library="system"
                  name="chevron-down"
                  variant="solid"
                ></wa-icon>
              </slot>
            </div>

            <div
              id="listbox"
              role="listbox"
              aria-expanded=${this.open ? "true" : "false"}
              aria-multiselectable=${this.multiple ? "true" : "false"}
              aria-labelledby="label"
              part="listbox"
              class="listbox"
              tabindex="-1"
              @mouseup=${this.handleOptionClick}
            >
              <slot @slotchange=${this.handleDefaultSlotChange}></slot>
            </div>
          </wa-popup>
        </div>

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
      </div>
    `;
  }
}

// The change-in-update warning is required for this component because the form-associated base class calls
// updateValidity() in firstUpdated(), which triggers requestUpdate('validity') to sync the validation state after the
// first render when the validation target is available. Additionally, HasSlotController triggers requestUpdate() on
// initial slotchange events, and selectionChanged() sets @state properties (displayLabel, selectedOptions) in response
// to slot content changes. See https://lit.dev/docs/tools/development/#development-build-runtime-warnings
WaSelect.disableWarning?.("change-in-update");

declare global {
  interface HTMLElementTagNameMap {
    "wa-select": WaSelect;
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
@@ -1071,17 +1071,15 @@
                 @focus=${this.handleFocus}
               />
 
               <!-- Tags need to wait for first hydration before populating otherwise it will create a hydration mismatch. -->
-              ${
-                this.multiple && this.hasUpdated
-                  ? html`
-                      <div part="tags" class="tags" @wa-remove=${this.handleTagRemove}>
-                        ${this.tags}
-                      </div>
-                    `
-                  : ""
-              }
+              ${this.multiple && this.hasUpdated
+                ? html`
+                    <div part="tags" class="tags" @wa-remove=${this.handleTagRemove}>
+                      ${this.tags}
+                    </div>
+                  `
+                : ""}
 
               <input
                 class="value-input"
                 type="text"
@@ -1092,26 +1090,24 @@
                 aria-hidden="true"
                 @focus=${() => this.focus()}
               />
 
-              ${
-                hasClearIcon
-                  ? html`
-                      <button
-                        part="clear-button"
-                        type="button"
-                        aria-label=${this.localize.term("clearEntry")}
-                        @mousedown=${this.handleClearMouseDown}
-                        @click=${this.handleClearClick}
-                        tabindex="-1"
-                      >
-                        <slot name="clear-icon">
-                          <wa-icon name="circle-xmark" library="system" variant="regular"></wa-icon>
-                        </slot>
-                      </button>
-                    `
-                  : ""
-              }
+              ${hasClearIcon
+                ? html`
+                    <button
+                      part="clear-button"
+                      type="button"
+                      aria-label=${this.localize.term("clearEntry")}
+                      @mousedown=${this.handleClearMouseDown}
+                      @click=${this.handleClearClick}
+                      tabindex="-1"
+                    >
+                      <slot name="clear-icon">
+                        <wa-icon name="circle-xmark" library="system" variant="regular"></wa-icon>
+                      </slot>
+                    </button>
+                  `
+                : ""}
 
               <slot name="end" part="end" class="end"></slot>
 
               <slot name="expand-icon" part="expand-icon" class="expand-icon">

`````

### Actual (oxfmt)

`````ts
import type { PropertyValues, TemplateResult } from "lit";
import { html, isServer } from "lit";
import { customElement, property, query, state } from "lit/decorators.js";
import { classMap } from "lit/directives/class-map.js";
import { unsafeHTML } from "lit/directives/unsafe-html.js";
import { WaAfterHideEvent } from "../../events/after-hide.js";
import { WaAfterShowEvent } from "../../events/after-show.js";
import { WaClearEvent } from "../../events/clear.js";
import { WaHideEvent } from "../../events/hide.js";
import { WaRemoveEvent } from "../../events/remove.js";
import { WaShowEvent } from "../../events/show.js";
import { animateWithClass } from "../../internal/animate.js";
import {
  isTopDismissible,
  registerDismissible,
  unregisterDismissible,
} from "../../internal/dismissible-stack.js";
import { waitForEvent } from "../../internal/event.js";
import { scrollIntoView } from "../../internal/scroll.js";
import { warnDeprecatedSize } from "../../internal/size.js";
import { HasSlotController } from "../../internal/slot.js";
import { RequiredValidator } from "../../internal/validators/required-validator.js";
import { watch } from "../../internal/watch.js";
import { WebAwesomeFormAssociatedElement } from "../../internal/webawesome-form-associated-element.js";
import formControlStyles from "../../styles/component/form-control.styles.js";
import sizeStyles from "../../styles/component/size.styles.js";
import { LocalizeController } from "../../utilities/localize.js";
import "../icon/icon.js";
import "../option/option.js";
import type WaOption from "../option/option.js";
import "../popup/popup.js";
import type WaPopup from "../popup/popup.js";
import "../tag/tag.js";
import styles from "./select.styles.js";

/**
 * @summary Selects let users choose one or more values from a dropdown list of predefined options. Use them in forms
 *  when a fixed set of choices needs to fit in limited space.
 * @documentation https://webawesome.com/docs/components/select
 * @status stable
 * @since 2.0
 *
 * @dependency wa-icon
 * @dependency wa-popup
 * @dependency wa-tag
 * @dependency wa-option
 *
 * @slot - The listbox options. Must be `<wa-option>` elements. You can use `<wa-divider>` to group items visually.
 * @slot label - The input's label. Alternatively, you can use the `label` attribute.
 * @slot start - An element, such as `<wa-icon>`, placed at the start of the combobox.
 * @slot end - An element, such as `<wa-icon>`, placed at the end of the combobox.
 * @slot clear-icon - An icon to use in lieu of the default clear icon.
 * @slot expand-icon - The icon to show when the control is expanded and collapsed. Rotates on open and close.
 * @slot hint - Text that describes how to use the input. Alternatively, you can use the `hint` attribute.
 *
 * @event change - Emitted when the control's value changes.
 * @event input - Emitted when the control receives input.
 * @event focus - Emitted when the control gains focus.
 * @event blur - Emitted when the control loses focus.
 * @event wa-clear - Emitted when the control's value is cleared.
 * @event wa-show - Emitted when the select's menu opens.
 * @event wa-after-show - Emitted after the select's menu opens and all animations are complete.
 * @event wa-hide - Emitted when the select's menu closes.
 * @event wa-after-hide - Emitted after the select's menu closes and all animations are complete.
 * @event wa-invalid - Emitted when the form control has been checked for validity and its constraints aren't satisfied.
 *
 * @csspart form-control - The form control that wraps the label, input, and hint.
 * @csspart form-control-label - The label's wrapper.
 * @csspart form-control-input - The select's wrapper.
 * @csspart hint - The hint's wrapper.
 * @csspart combobox - The container the wraps the start, end, value, clear icon, and expand button.
 * @csspart start - The container that wraps the `start` slot.
 * @csspart end - The container that wraps the `end` slot.
 * @csspart display-input - The element that displays the selected option's label, an `<input>` element.
 * @csspart listbox - The listbox container where options are slotted.
 * @csspart tags - The container that houses option tags when `multiselect` is used.
 * @csspart tag - The individual tags that represent each multiselect option.
 * @csspart tag__content - The tag's content part.
 * @csspart tag__remove-button - The tag's remove button.
 * @csspart tag__remove-button__base - The tag's remove button base part.
 * @csspart clear-button - The clear button.
 * @csspart expand-icon - The container that wraps the expand icon.
 *
 * @cssproperty [--show-duration=100ms] - The duration of the show animation.
 * @cssproperty [--hide-duration=100ms] - The duration of the hide animation.
 * @cssproperty [--tag-max-size=10ch] - When using `multiple`, the max size of tags before their content is truncated.
 *
 * @cssstate blank - The select is empty.
 */
@customElement("wa-select")
export default class WaSelect extends WebAwesomeFormAssociatedElement {
  static css = [styles, formControlStyles, sizeStyles];

  static get validators() {
    const validators = isServer
      ? []
      : [
          RequiredValidator({
            validationElement: Object.assign(document.createElement("select"), { required: true }),
          }),
        ];
    return [...super.validators, ...validators];
  }

  assumeInteractionOn = ["blur", "input"];

  private cachedOptions: WaOption[] | null = null;
  private readonly hasSlotController = new HasSlotController(this, "hint", "label");
  private readonly localize = new LocalizeController(this);
  private selectionOrder: Map<string, number> = new Map();
  private typeToSelectString = "";
  private typeToSelectTimeout: number;
  private slotChangePending = false;

  @query(".select") popup: WaPopup;
  @query(".combobox") combobox: HTMLSlotElement;
  @query(".display-input") displayInput: HTMLInputElement;
  @query(".value-input") valueInput: HTMLInputElement;
  @query(".listbox") listbox: HTMLSlotElement;

  /** Where to anchor native constraint validation */
  get validationTarget() {
    return this.valueInput;
  }

  @state() displayLabel = "";
  @state() currentOption: WaOption;
  @state() selectedOptions: WaOption[] = [];
  /** @internal */
  optionValues: Set<string | null> | undefined;

  /** The name of the select, submitted as a name/value pair with form data. */
  @property({ reflect: true }) name = "";

  private _defaultValue: null | string | string[] = null;

  @property({
    attribute: false,
  })
  set defaultValue(val: null | string | string[]) {
    this._defaultValue = this.convertDefaultValue(val);
  }

  get defaultValue() {
    return this.convertDefaultValue(this._defaultValue);
  }

  private rawValuesEqual(a: string[] | null | undefined, b: string[] | null | undefined): boolean {
    if (a == null && b == null) return true;
    if (a == null || b == null) return false;
    if (a.length !== b.length) return false;
    return a.every((v, i) => v === b[i]);
  }

  /**
   * @private
   * A converter for defaultValue from array to string if its multiple. Also fixes some hydration issues.
   */
  private convertDefaultValue(val: typeof this.defaultValue) {
    // For some reason this can go off before we've fully updated. So check the attribute too.
    const isMultiple = this.multiple || this.hasAttribute("multiple");

    if (!isMultiple && Array.isArray(val)) {
      val = val[0];
    }

    return val;
  }

  private _value: string[] | undefined | null;

  /** The select's value. This will be a string for single select or an array for multi-select. */
  @property({ attribute: "value", reflect: false })
  set value(val: string | string[] | null) {
    let oldValue = this.value;

    if ((val as any) instanceof FormData) {
      val = (val as unknown as FormData).getAll(this.name) as string[];
    }

    if (val != null && !Array.isArray(val)) {
      val = [val];
    }

    const oldRawValue = this._value;
    this._value = val ?? null;

    // Compare raw internal values to detect actual changes. We can't rely on the getter because it filters through
    // optionValues, which may be empty when options aren't in the DOM yet (common with frameworks that set properties
    // before appending children).
    if (!this.rawValuesEqual(oldRawValue, this._value)) {
      this.valueHasChanged = true;
      this.requestUpdate("value", oldValue);
    }
  }

  get value() {
    let value = this._value ?? this.defaultValue ?? null;

    if (value != null) {
      value = Array.isArray(value) ? value : [value];
    }

    // Build optionValues from the cached options list. This is rebuilt each time the getter is called rather than
    // cached, because caching created stale-state bugs when the value was set before options existed in the DOM. The
    // underlying getAllOptions() is already cached via cachedOptions, so this is cheap.
    this.optionValues = new Set(
      this.getAllOptions()
        .filter((option) => !option.disabled)
        .map((option) => option.value),
    );

    // Drop values not in the DOM
    let ret: null | string | string[] = value;
    if (value != null) {
      ret = value.filter((v) => this.optionValues!.has(v));
      ret = this.multiple ? ret : ret[0];
      ret = ret ?? null;
    }

    return ret;
  }

  /** The select's size. */
  @property({ reflect: true }) size: "xs" | "s" | "m" | "l" | "xl" | "small" | "medium" | "large" =
    "m";

  @watch("size")
  handleSizeChange() {
    warnDeprecatedSize(this.localName, this.size);
  }

  /** Placeholder text to show as a hint when the select is empty. */
  @property() placeholder = "";

  /** Allows more than one option to be selected. */
  @property({ type: Boolean, reflect: true }) multiple = false;

  /**
   * The maximum number of selected options to show when `multiple` is true. After the maximum, "+n" will be shown to
   * indicate the number of additional items that are selected. Set to 0 to remove the limit.
   */
  @property({ attribute: "max-options-visible", type: Number }) maxOptionsVisible = 3;

  /** Disables the select control. */
  @property({ type: Boolean }) disabled = false;

  /** Adds a clear button when the select is not empty. */
  @property({ attribute: "with-clear", type: Boolean }) withClear = false;

  /**
   * Indicates whether or not the select is open. You can toggle this attribute to show and hide the menu, or you can
   * use the `show()` and `hide()` methods and this attribute will reflect the select's open state.
   */
  @property({ type: Boolean, reflect: true }) open = false;

  /** The select's visual appearance. */
  @property({ reflect: true }) appearance: "filled" | "outlined" | "filled-outlined" = "outlined";

  /** Draws a pill-style select with rounded edges. */
  @property({ type: Boolean, reflect: true }) pill = false;

  /** The select's label. If you need to display HTML, use the `label` slot instead. */
  @property() label = "";

  /**
   * The preferred placement of the select's menu. Note that the actual placement may vary as needed to keep the listbox
   * inside of the viewport.
   */
  @property({ reflect: true }) placement: "top" | "bottom" = "bottom";

  /** The select's hint. If you need to display HTML, use the `hint` slot instead. */
  @property({ attribute: "hint" }) hint = "";

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

  /** The select's required attribute. */
  @property({ type: Boolean, reflect: true }) required = false;

  /**
   * A function that customizes the tags to be rendered when multiple=true. The first argument is the option, the second
   * is the current tag's index.  The function should return either a Lit TemplateResult or a string containing trusted
   * HTML of the symbol to render at the specified value.
   */
  @property({ attribute: false }) getTag: (
    option: WaOption,
    index: number,
  ) => TemplateResult | string | HTMLElement = (option) => {
    return html`
      <wa-tag
        part="tag"
        exportparts="
            base:tag__base,
            content:tag__content,
            remove-button:tag__remove-button,
            remove-button__base:tag__remove-button__base
          "
        ?pill=${this.pill}
        size=${this.size}
        with-remove
        data-value=${option.value}
        @wa-remove=${(event: WaRemoveEvent) => this.handleTagRemove(event, option)}
      >
        ${option.label}
      </wa-tag>
    `;
  };

  connectedCallback() {
    super.connectedCallback();

    // Call processSlotChange directly so initial setup is synchronous.
    // Subsequent option additions will be batched via handleDefaultSlotChange.
    this.processSlotChange();

    // Because this is a form control, it shouldn't be opened initially
    this.open = false;
  }

  disconnectedCallback() {
    super.disconnectedCallback();
    this.removeOpenListeners();
    this.cachedOptions = null;
  }

  private updateDefaultValue() {
    const allOptions = this.getAllOptions();
    const defaultSelectedOptions = allOptions.filter(
      (el) => el.hasAttribute("selected") || el.defaultSelected,
    );
    if (defaultSelectedOptions.length > 0) {
      const selectedValues = defaultSelectedOptions.map((el) => el.value);
      this._defaultValue = this.multiple ? selectedValues : selectedValues[0];
    }
    if (this.hasAttribute("value")) {
      this._defaultValue = this.getAttribute("value") || null;
    }
  }

  private addOpenListeners() {
    //
    // Listen on the root node instead of the document in case the elements are inside a shadow root
    //
    // https://github.com/shoelace-style/shoelace/issues/1763
    //
    document.addEventListener("focusin", this.handleDocumentFocusIn);
    document.addEventListener("keydown", this.handleDocumentKeyDown);
    document.addEventListener("mousedown", this.handleDocumentMouseDown);
    registerDismissible(this);

    // If the component is rendered in a shadow root, we need to attach the focusin listener there too
    if (this.getRootNode() !== document) {
      this.getRootNode().addEventListener("focusin", this.handleDocumentFocusIn);
    }
  }

  private removeOpenListeners() {
    document.removeEventListener("focusin", this.handleDocumentFocusIn);
    document.removeEventListener("keydown", this.handleDocumentKeyDown);
    document.removeEventListener("mousedown", this.handleDocumentMouseDown);
    unregisterDismissible(this);

    if (this.getRootNode() !== document) {
      this.getRootNode().removeEventListener("focusin", this.handleDocumentFocusIn);
    }
  }

  private handleFocus() {
    this.displayInput.setSelectionRange(0, 0);
  }

  private handleDocumentFocusIn = (event: KeyboardEvent) => {
    // Close when focusing out of the select
    const path = event.composedPath();
    if (this && !path.includes(this)) {
      this.hide();
    }
  };

  private handleDocumentKeyDown = (event: KeyboardEvent) => {
    const target = event.target as HTMLElement;
    const isClearButton = target.closest('[part~="clear-button"]') !== null;
    const isButton = target.closest("wa-button") !== null;

    // Ignore presses when the target is a button (e.g. the remove button in `<wa-tag>`)
    if (isClearButton || isButton) {
      return;
    }

    // Close when pressing escape
    if (event.key === "Escape" && this.open && isTopDismissible(this)) {
      event.preventDefault();
      event.stopPropagation();
      this.hide();
      this.displayInput.focus({ preventScroll: true });
    }

    // Handle enter and space. When pressing space, we allow for type to select behaviors so if there's anything in the
    // buffer we _don't_ close it.
    if (event.key === "Enter" || (event.key === " " && this.typeToSelectString === "")) {
      event.preventDefault();
      event.stopImmediatePropagation();

      // If it's not open, open it
      if (!this.open) {
        this.show();
        return;
      }

      // If it is open, update the value based on the current selection and close it
      if (this.currentOption && !this.currentOption.disabled) {
        this.valueHasChanged = true;
        this.hasInteracted = true;
        if (this.multiple) {
          this.toggleOptionSelection(this.currentOption);
        } else {
          this.setSelectedOptions(this.currentOption);
        }

        // Emit after updating
        this.updateComplete.then(() => {
          this.dispatchEvent(new InputEvent("input", { bubbles: true, composed: true }));
          this.dispatchEvent(new Event("change", { bubbles: true, composed: true }));
        });

        if (!this.multiple) {
          this.hide();
          this.displayInput.focus({ preventScroll: true });
        }
      }

      return;
    }

    // Navigate options
    if (["ArrowUp", "ArrowDown", "Home", "End"].includes(event.key)) {
      const allOptions = this.getAllOptions();
      const currentIndex = allOptions.indexOf(this.currentOption);
      let newIndex = Math.max(0, currentIndex);

      // Prevent scrolling
      event.preventDefault();

      // Open it
      if (!this.open) {
        this.show();

        // If an option is already selected, stop here because we want that one to remain highlighted when the listbox
        // opens for the first time
        if (this.currentOption) {
          return;
        }
      }

      if (event.key === "ArrowDown") {
        newIndex = currentIndex + 1;
        if (newIndex > allOptions.length - 1) newIndex = 0;
      } else if (event.key === "ArrowUp") {
        newIndex = currentIndex - 1;
        if (newIndex < 0) newIndex = allOptions.length - 1;
      } else if (event.key === "Home") {
        newIndex = 0;
      } else if (event.key === "End") {
        newIndex = allOptions.length - 1;
      }

      this.setCurrentOption(allOptions[newIndex]);
    }

    // All other "printable" keys trigger type to select
    if (event.key?.length === 1 || event.key === "Backspace") {
      const allOptions = this.getAllOptions();

      // Don't block important key combos like CMD+R
      if (event.metaKey || event.ctrlKey || event.altKey) {
        return;
      }

      // Open, unless the key that triggered is backspace
      if (!this.open) {
        if (event.key === "Backspace") {
          return;
        }

        this.show();
      }

      event.stopPropagation();
      event.preventDefault();

      clearTimeout(this.typeToSelectTimeout);
      this.typeToSelectTimeout = window.setTimeout(() => (this.typeToSelectString = ""), 1000);

      if (event.key === "Backspace") {
        this.typeToSelectString = this.typeToSelectString.slice(0, -1);
      } else {
        this.typeToSelectString += event.key.toLowerCase();
      }

      for (const option of allOptions) {
        const label = option.label.toLowerCase();

        if (label.startsWith(this.typeToSelectString)) {
          this.setCurrentOption(option);
          break;
        }
      }
    }
  };

  private handleDocumentMouseDown = (event: MouseEvent) => {
    // Close when clicking outside of the select
    const path = event.composedPath();
    if (this && !path.includes(this)) {
      this.hide();
    }
  };

  private handleLabelClick() {
    this.displayInput.focus();
  }

  private handleComboboxClick(event: MouseEvent) {
    event.preventDefault();
  }

  private handleComboboxMouseDown(event: MouseEvent) {
    const path = event.composedPath();
    const isButton = path.some(
      (el) => el instanceof Element && el.tagName.toLowerCase() === "wa-button",
    );

    // Ignore disabled controls and clicks on tags (remove buttons)
    if (this.disabled || isButton) {
      return;
    }

    event.preventDefault();
    this.displayInput.focus({ preventScroll: true });
    this.open = !this.open;
  }

  private handleComboboxKeyDown(event: KeyboardEvent) {
    event.stopPropagation();
    this.handleDocumentKeyDown(event);
  }

  private handleClearClick(event: MouseEvent) {
    event.stopPropagation();

    this.hasInteracted = true;
    this.valueHasChanged = true;

    if (this.value !== null) {
      this.displayLabel = "";
      this.selectionOrder.clear();
      this.setSelectedOptions([]);
      this.displayInput.focus({ preventScroll: true });

      // Emit after update
      this.updateComplete.then(() => {
        this.dispatchEvent(new WaClearEvent());
        this.dispatchEvent(new InputEvent("input", { bubbles: true, composed: true }));
        this.dispatchEvent(new Event("change", { bubbles: true, composed: true }));
      });
    }
  }

  private handleClearMouseDown(event: MouseEvent) {
    // Don't lose focus or propagate events when clicking the clear button
    event.stopPropagation();
    event.preventDefault();
  }

  private handleOptionClick(event: MouseEvent) {
    const target = event.target as HTMLElement;
    const option = target.closest("wa-option");

    if (option && !option.disabled) {
      this.hasInteracted = true;
      this.valueHasChanged = true;

      if (this.multiple) {
        this.toggleOptionSelection(option);
      } else {
        this.setSelectedOptions(option);
      }

      // Set focus after updating so the value is announced by screen readers
      this.updateComplete.then(() => this.displayInput.focus({ preventScroll: true }));

      this.requestUpdate("value");

      // Emit after updating
      this.updateComplete.then(() => {
        this.dispatchEvent(new InputEvent("input", { bubbles: true, composed: true }));
        this.dispatchEvent(new Event("change", { bubbles: true, composed: true }));
      });

      if (!this.multiple) {
        this.hide();
        this.displayInput.focus({ preventScroll: true });
      }
    }
  }

  /* @internal - used by options to update labels */
  public handleDefaultSlotChange() {
    if (this.slotChangePending) return;
    this.slotChangePending = true;
    queueMicrotask(() => {
      this.slotChangePending = false;
      this.processSlotChange();
    });
  }

  private processSlotChange() {
    if (!customElements.get("wa-option")) {
      customElements.whenDefined("wa-option").then(() => this.handleDefaultSlotChange());
    }

    // Invalidate the options cache since slots have changed
    this.cachedOptions = null;

    const allOptions = this.getAllOptions();

    // Update defaultValue if it hasn't been explicitly set and we have selected options
    this.updateDefaultValue();

    let value = this.value;

    if (value == null || (!this.valueHasChanged && !this.hasInteracted)) {
      this.selectionChanged();
      return;
    }

    if (!Array.isArray(value)) {
      value = [value];
    }

    // Select only the options that match the new value
    const selectedOptions = allOptions.filter((el) => value.includes(el.value));
    this.setSelectedOptions(selectedOptions);
  }

  private handleTagRemove(event: WaRemoveEvent, directOption?: WaOption) {
    event.stopPropagation();

    if (this.disabled) return;

    // Mark as interacted so selectionChanged() uses the correct filter logic
    this.hasInteracted = true;
    this.valueHasChanged = true;

    // Use the directly provided option if available (from getTag method)
    let option = directOption;

    // If no direct option was provided, find the option from the data-value attribute
    if (!option) {
      const tagElement = (event.target as Element).closest(
        "wa-tag[data-value]",
      ) as HTMLElement | null;

      if (tagElement) {
        const value = tagElement.dataset.value;
        option = this.selectedOptions.find((opt) => opt.value === value);
      }
    }

    if (option) {
      this.toggleOptionSelection(option, false);

      // Emit after updating
      this.updateComplete.then(() => {
        this.dispatchEvent(new InputEvent("input", { bubbles: true, composed: true }));
        this.dispatchEvent(new Event("change", { bubbles: true, composed: true }));
      });
    }
  }

  // Gets an array of all `<wa-option>` elements
  private getAllOptions() {
    if (this.cachedOptions) return this.cachedOptions;
    if (!this?.querySelectorAll) {
      return [];
    }
    this.cachedOptions = [...this.querySelectorAll<WaOption>("wa-option")];
    return this.cachedOptions;
  }

  // Gets the first `<wa-option>` element
  private getFirstOption() {
    return this.querySelector<WaOption>("wa-option");
  }

  // Sets the current option, which is the option the user is currently interacting with (e.g. via keyboard). Only one
  // option may be "current" at a time.
  private setCurrentOption(option: WaOption | null) {
    const allOptions = this.getAllOptions();

    // Clear selection
    allOptions.forEach((el) => {
      el.current = false;
      el.tabIndex = -1;
    });

    // Select the target option
    if (option) {
      this.currentOption = option;
      option.current = true;
      option.tabIndex = 0;
      option.focus({ preventScroll: true });
    }
  }

  // Sets the selected option(s)
  private setSelectedOptions(option: WaOption | WaOption[]) {
    const allOptions = this.getAllOptions();
    const newSelectedOptions = Array.isArray(option) ? option : [option];

    // Clear existing selection
    allOptions.forEach((el) => {
      if (newSelectedOptions.includes(el)) {
        return;
      }
      el.selected = false;
    });

    // Set the new selection
    if (newSelectedOptions.length) {
      newSelectedOptions.forEach((el) => (el.selected = true));
    }

    // Update selection, value, and display label
    this.selectionChanged();
  }

  // Toggles an option's selected state
  private toggleOptionSelection(option: WaOption, force?: boolean) {
    if (force === true || force === false) {
      option.selected = force;
    } else {
      option.selected = !option.selected;
    }

    this.selectionChanged();
  }

  // @internal This method must be called whenever the selection changes. It will update the selected options cache, the
  // current value, and the display value. The option component uses it internally to update labels as they change.
  public selectionChanged() {
    const options = this.getAllOptions();

    // Update selected options cache
    const newSelectedOptions = options.filter((el) => {
      if (!this.hasInteracted && !this.valueHasChanged) {
        const defaultValue = this.defaultValue;
        const defaultValues = Array.isArray(defaultValue) ? defaultValue : [defaultValue];
        return (
          el.hasAttribute("selected") ||
          el.defaultSelected ||
          el.selected ||
          defaultValues?.includes(el.value)
        );
      }

      return el.selected;
    });

    // Update the selection order map
    const newSelectedValues = new Set(newSelectedOptions.map((el) => el.value));

    // Remove deselected options from the order map
    for (const value of this.selectionOrder.keys()) {
      if (!newSelectedValues.has(value)) {
        this.selectionOrder.delete(value);
      }
    }

    // Add newly selected options
    const maxOrder = this.selectionOrder.size > 0 ? Math.max(...this.selectionOrder.values()) : -1;
    let nextOrder = maxOrder + 1;
    for (const option of newSelectedOptions) {
      if (!this.selectionOrder.has(option.value)) {
        this.selectionOrder.set(option.value, nextOrder++);
      }
    }

    // Sort options by selection order
    this.selectedOptions = newSelectedOptions.sort((a, b) => {
      const orderA = this.selectionOrder.get(a.value) ?? 0;
      const orderB = this.selectionOrder.get(b.value) ?? 0;
      return orderA - orderB;
    });

    let selectedValues = new Set(this.selectedOptions.map((el) => el.value));

    // Toggle values present in the DOM from this.value, while preserving options NOT present in the DOM (for lazy loading)
    // Note that options NOT present in the DOM will be moved to the end after this
    if (selectedValues.size > 0 || this._value) {
      const oldValue = this._value;
      if (this._value == null) {
        // First time it's set
        let value = this.defaultValue ?? [];
        this._value = Array.isArray(value) ? value : [value];
      }

      // Filter out values that are in the DOM
      this._value = this._value?.filter((value) => !this.optionValues?.has(value)) ?? null;
      this._value?.unshift(...selectedValues);
      this.requestUpdate("value", oldValue);
    }

    // Update the value and display label
    if (this.multiple) {
      if (this.placeholder && !this.value?.length) {
        // When no items are selected, keep the value empty so the placeholder shows
        this.displayLabel = "";
      } else {
        this.displayLabel = this.localize.term("numOptionsSelected", this.selectedOptions.length);
      }
    } else {
      const selectedOption = this.selectedOptions[0];
      this.displayLabel = selectedOption?.label ?? "";
    }

    // Update validity
    this.updateComplete.then(() => {
      this.updateValidity();
    });
  }

  protected get tags() {
    return this.selectedOptions.map((option, index) => {
      if (index < this.maxOptionsVisible || this.maxOptionsVisible <= 0) {
        const tag = this.getTag(option, index);
        if (!tag) return null;
        return typeof tag === "string" ? unsafeHTML(tag) : tag;
      } else if (index === this.maxOptionsVisible) {
        // Hit tag limit
        return html`
          <wa-tag
            part="tag"
            exportparts="
              base:tag__base,
              content:tag__content,
              remove-button:tag__remove-button,
              remove-button__base:tag__remove-button__base
            "
          >
            +${this.selectedOptions.length - index}
          </wa-tag>
        `;
      }
      return null;
    });
  }

  updated(changedProperties: PropertyValues<this>) {
    super.updated(changedProperties);

    if (changedProperties.has("value") || changedProperties.has("displayLabel")) {
      this.customStates.set("blank", !this.value && !this.displayLabel);
    }
  }

  @watch("disabled", { waitUntilFirstUpdate: true })
  handleDisabledChange() {
    // Close the listbox when the control is open and disabled
    if (this.disabled && this.open) {
      this.open = false;
    }
  }

  @watch("value", { waitUntilFirstUpdate: true })
  handleValueChange() {
    const allOptions = this.getAllOptions();
    const value = Array.isArray(this.value) ? this.value : [this.value];

    // Select only the options that match the new value
    const selectedOptions = allOptions.filter((el) => value.includes(el.value));
    this.setSelectedOptions(selectedOptions);
    this.updateValidity();
  }

  @watch("open", { waitUntilFirstUpdate: true })
  async handleOpenChange() {
    if (this.open && !this.disabled) {
      // Reset the current option
      this.setCurrentOption(this.selectedOptions[0] || this.getFirstOption());

      // Show
      const waShowEvent = new WaShowEvent();
      this.dispatchEvent(waShowEvent);
      if (waShowEvent.defaultPrevented) {
        this.open = false;
        return;
      }

      this.addOpenListeners();
      this.listbox.hidden = false;
      this.popup.active = true;

      // Select the appropriate option based on value after the listbox opens
      requestAnimationFrame(() => {
        this.setCurrentOption(this.currentOption);
      });

      await animateWithClass(this.popup.popup, "show");

      // Make sure the current option is scrolled into view (required for Safari)
      if (this.currentOption) {
        scrollIntoView(this.currentOption, this.listbox, "vertical", "auto");
      }

      this.dispatchEvent(new WaAfterShowEvent());
    } else {
      // Hide
      const waHideEvent = new WaHideEvent();
      this.dispatchEvent(waHideEvent);
      if (waHideEvent.defaultPrevented) {
        this.open = false;
        return;
      }

      this.removeOpenListeners();
      await animateWithClass(this.popup.popup, "hide");
      this.listbox.hidden = true;
      this.popup.active = false;

      this.dispatchEvent(new WaAfterHideEvent());
    }
  }

  /** Shows the listbox. */
  async show() {
    if (this.open || this.disabled) {
      this.open = false;
      return undefined;
    }

    this.open = true;
    return waitForEvent(this, "wa-after-show");
  }

  /** Hides the listbox. */
  async hide() {
    if (!this.open || this.disabled) {
      this.open = false;
      return undefined;
    }

    this.open = false;
    return waitForEvent(this, "wa-after-hide");
  }

  /** Sets focus on the control. */
  focus(options?: FocusOptions) {
    this.displayInput.focus(options);
  }

  /** Removes focus from the control. */
  blur() {
    this.displayInput.blur();
  }

  formResetCallback() {
    this.selectionOrder.clear();
    this.value = this.defaultValue;
    super.formResetCallback();
    this.handleValueChange();

    this.updateComplete.then(() => {
      this.dispatchEvent(new InputEvent("input", { bubbles: true, composed: true }));
      this.dispatchEvent(new Event("change", { bubbles: true, composed: true }));
    });
  }

  render() {
    const hasLabelSlot = this.hasUpdated ? this.hasSlotController.test("label") : this.withLabel;
    const hasHintSlot = this.hasUpdated ? this.hasSlotController.test("hint") : this.withHint;
    const hasLabel = this.label ? true : !!hasLabelSlot;
    const hasHint = this.hint ? true : !!hasHintSlot;
    const hasClearIcon =
      (this.hasUpdated || isServer) &&
      this.withClear &&
      !this.disabled &&
      (this.displayLabel || (this.value && this.value.length > 0));

    return html`
      <div
        part="form-control"
        class=${classMap({
          "form-control": true,
          "form-control-has-label": hasLabel,
        })}
      >
        <label
          id="label"
          part="form-control-label label"
          class=${classMap({
            label: true,
            "has-label": hasLabel,
          })}
          aria-hidden=${hasLabel ? "false" : "true"}
          @click=${this.handleLabelClick}
        >
          <slot name="label">${this.label}</slot>
        </label>

        <div part="form-control-input" class="form-control-input">
          <wa-popup
            class=${classMap({
              select: true,
              open: this.open,
              disabled: this.disabled,
              enabled: !this.disabled,
              multiple: this.multiple,
            })}
            placement=${this.placement}
            flip
            shift
            sync="width"
            auto-size="vertical"
            auto-size-padding="10"
          >
            <div
              part="combobox"
              class="combobox"
              slot="anchor"
              @keydown=${this.handleComboboxKeyDown}
              @mousedown=${this.handleComboboxMouseDown}
              @click=${this.handleComboboxClick}
            >
              <slot part="start" name="start" class="start"></slot>

              <input
                part="display-input"
                class="display-input"
                type="text"
                placeholder=${this.placeholder}
                .disabled=${this.disabled}
                .value=${this.displayLabel}
                ?required=${this.required}
                autocomplete="off"
                spellcheck="false"
                autocapitalize="off"
                readonly
                aria-invalid=${
                  !this.validity.valid
                  /** aria-invalid is required because readonly inputs are technically always valid so it never reads 'invalid data' for screen readers. */
                }
                aria-controls="listbox"
                aria-expanded=${this.open ? "true" : "false"}
                aria-haspopup="listbox"
                aria-labelledby="label"
                aria-disabled=${this.disabled ? "true" : "false"}
                aria-describedby="hint"
                role="combobox"
                tabindex="0"
                @focus=${this.handleFocus}
              />

              <!-- Tags need to wait for first hydration before populating otherwise it will create a hydration mismatch. -->
              ${this.multiple && this.hasUpdated
                ? html`
                    <div part="tags" class="tags" @wa-remove=${this.handleTagRemove}>
                      ${this.tags}
                    </div>
                  `
                : ""}

              <input
                class="value-input"
                type="text"
                ?disabled=${this.disabled}
                ?required=${this.required}
                .value=${Array.isArray(this.value) ? this.value.join(", ") : this.value}
                tabindex="-1"
                aria-hidden="true"
                @focus=${() => this.focus()}
              />

              ${hasClearIcon
                ? html`
                    <button
                      part="clear-button"
                      type="button"
                      aria-label=${this.localize.term("clearEntry")}
                      @mousedown=${this.handleClearMouseDown}
                      @click=${this.handleClearClick}
                      tabindex="-1"
                    >
                      <slot name="clear-icon">
                        <wa-icon name="circle-xmark" library="system" variant="regular"></wa-icon>
                      </slot>
                    </button>
                  `
                : ""}

              <slot name="end" part="end" class="end"></slot>

              <slot name="expand-icon" part="expand-icon" class="expand-icon">
                <wa-icon library="system" name="chevron-down" variant="solid"></wa-icon>
              </slot>
            </div>

            <div
              id="listbox"
              role="listbox"
              aria-expanded=${this.open ? "true" : "false"}
              aria-multiselectable=${this.multiple ? "true" : "false"}
              aria-labelledby="label"
              part="listbox"
              class="listbox"
              tabindex="-1"
              @mouseup=${this.handleOptionClick}
            >
              <slot @slotchange=${this.handleDefaultSlotChange}></slot>
            </div>
          </wa-popup>
        </div>

        <slot
          id="hint"
          name="hint"
          part="hint"
          class=${classMap({
            "has-slotted": hasHint,
          })}
          aria-hidden=${hasHint ? "false" : "true"}
        >
          ${this.hint}
        </slot>
      </div>
    `;
  }
}

// The change-in-update warning is required for this component because the form-associated base class calls
// updateValidity() in firstUpdated(), which triggers requestUpdate('validity') to sync the validation state after the
// first render when the validation target is available. Additionally, HasSlotController triggers requestUpdate() on
// initial slotchange events, and selectionChanged() sets @state properties (displayLabel, selectedOptions) in response
// to slot content changes. See https://lit.dev/docs/tools/development/#development-build-runtime-warnings
WaSelect.disableWarning?.("change-in-update");

declare global {
  interface HTMLElementTagNameMap {
    "wa-select": WaSelect;
  }
}

`````

### Expected (prettier)

`````ts
import type { PropertyValues, TemplateResult } from "lit";
import { html, isServer } from "lit";
import { customElement, property, query, state } from "lit/decorators.js";
import { classMap } from "lit/directives/class-map.js";
import { unsafeHTML } from "lit/directives/unsafe-html.js";
import { WaAfterHideEvent } from "../../events/after-hide.js";
import { WaAfterShowEvent } from "../../events/after-show.js";
import { WaClearEvent } from "../../events/clear.js";
import { WaHideEvent } from "../../events/hide.js";
import { WaRemoveEvent } from "../../events/remove.js";
import { WaShowEvent } from "../../events/show.js";
import { animateWithClass } from "../../internal/animate.js";
import {
  isTopDismissible,
  registerDismissible,
  unregisterDismissible,
} from "../../internal/dismissible-stack.js";
import { waitForEvent } from "../../internal/event.js";
import { scrollIntoView } from "../../internal/scroll.js";
import { warnDeprecatedSize } from "../../internal/size.js";
import { HasSlotController } from "../../internal/slot.js";
import { RequiredValidator } from "../../internal/validators/required-validator.js";
import { watch } from "../../internal/watch.js";
import { WebAwesomeFormAssociatedElement } from "../../internal/webawesome-form-associated-element.js";
import formControlStyles from "../../styles/component/form-control.styles.js";
import sizeStyles from "../../styles/component/size.styles.js";
import { LocalizeController } from "../../utilities/localize.js";
import "../icon/icon.js";
import "../option/option.js";
import type WaOption from "../option/option.js";
import "../popup/popup.js";
import type WaPopup from "../popup/popup.js";
import "../tag/tag.js";
import styles from "./select.styles.js";

/**
 * @summary Selects let users choose one or more values from a dropdown list of predefined options. Use them in forms
 *  when a fixed set of choices needs to fit in limited space.
 * @documentation https://webawesome.com/docs/components/select
 * @status stable
 * @since 2.0
 *
 * @dependency wa-icon
 * @dependency wa-popup
 * @dependency wa-tag
 * @dependency wa-option
 *
 * @slot - The listbox options. Must be `<wa-option>` elements. You can use `<wa-divider>` to group items visually.
 * @slot label - The input's label. Alternatively, you can use the `label` attribute.
 * @slot start - An element, such as `<wa-icon>`, placed at the start of the combobox.
 * @slot end - An element, such as `<wa-icon>`, placed at the end of the combobox.
 * @slot clear-icon - An icon to use in lieu of the default clear icon.
 * @slot expand-icon - The icon to show when the control is expanded and collapsed. Rotates on open and close.
 * @slot hint - Text that describes how to use the input. Alternatively, you can use the `hint` attribute.
 *
 * @event change - Emitted when the control's value changes.
 * @event input - Emitted when the control receives input.
 * @event focus - Emitted when the control gains focus.
 * @event blur - Emitted when the control loses focus.
 * @event wa-clear - Emitted when the control's value is cleared.
 * @event wa-show - Emitted when the select's menu opens.
 * @event wa-after-show - Emitted after the select's menu opens and all animations are complete.
 * @event wa-hide - Emitted when the select's menu closes.
 * @event wa-after-hide - Emitted after the select's menu closes and all animations are complete.
 * @event wa-invalid - Emitted when the form control has been checked for validity and its constraints aren't satisfied.
 *
 * @csspart form-control - The form control that wraps the label, input, and hint.
 * @csspart form-control-label - The label's wrapper.
 * @csspart form-control-input - The select's wrapper.
 * @csspart hint - The hint's wrapper.
 * @csspart combobox - The container the wraps the start, end, value, clear icon, and expand button.
 * @csspart start - The container that wraps the `start` slot.
 * @csspart end - The container that wraps the `end` slot.
 * @csspart display-input - The element that displays the selected option's label, an `<input>` element.
 * @csspart listbox - The listbox container where options are slotted.
 * @csspart tags - The container that houses option tags when `multiselect` is used.
 * @csspart tag - The individual tags that represent each multiselect option.
 * @csspart tag__content - The tag's content part.
 * @csspart tag__remove-button - The tag's remove button.
 * @csspart tag__remove-button__base - The tag's remove button base part.
 * @csspart clear-button - The clear button.
 * @csspart expand-icon - The container that wraps the expand icon.
 *
 * @cssproperty [--show-duration=100ms] - The duration of the show animation.
 * @cssproperty [--hide-duration=100ms] - The duration of the hide animation.
 * @cssproperty [--tag-max-size=10ch] - When using `multiple`, the max size of tags before their content is truncated.
 *
 * @cssstate blank - The select is empty.
 */
@customElement("wa-select")
export default class WaSelect extends WebAwesomeFormAssociatedElement {
  static css = [styles, formControlStyles, sizeStyles];

  static get validators() {
    const validators = isServer
      ? []
      : [
          RequiredValidator({
            validationElement: Object.assign(document.createElement("select"), { required: true }),
          }),
        ];
    return [...super.validators, ...validators];
  }

  assumeInteractionOn = ["blur", "input"];

  private cachedOptions: WaOption[] | null = null;
  private readonly hasSlotController = new HasSlotController(this, "hint", "label");
  private readonly localize = new LocalizeController(this);
  private selectionOrder: Map<string, number> = new Map();
  private typeToSelectString = "";
  private typeToSelectTimeout: number;
  private slotChangePending = false;

  @query(".select") popup: WaPopup;
  @query(".combobox") combobox: HTMLSlotElement;
  @query(".display-input") displayInput: HTMLInputElement;
  @query(".value-input") valueInput: HTMLInputElement;
  @query(".listbox") listbox: HTMLSlotElement;

  /** Where to anchor native constraint validation */
  get validationTarget() {
    return this.valueInput;
  }

  @state() displayLabel = "";
  @state() currentOption: WaOption;
  @state() selectedOptions: WaOption[] = [];
  /** @internal */
  optionValues: Set<string | null> | undefined;

  /** The name of the select, submitted as a name/value pair with form data. */
  @property({ reflect: true }) name = "";

  private _defaultValue: null | string | string[] = null;

  @property({
    attribute: false,
  })
  set defaultValue(val: null | string | string[]) {
    this._defaultValue = this.convertDefaultValue(val);
  }

  get defaultValue() {
    return this.convertDefaultValue(this._defaultValue);
  }

  private rawValuesEqual(a: string[] | null | undefined, b: string[] | null | undefined): boolean {
    if (a == null && b == null) return true;
    if (a == null || b == null) return false;
    if (a.length !== b.length) return false;
    return a.every((v, i) => v === b[i]);
  }

  /**
   * @private
   * A converter for defaultValue from array to string if its multiple. Also fixes some hydration issues.
   */
  private convertDefaultValue(val: typeof this.defaultValue) {
    // For some reason this can go off before we've fully updated. So check the attribute too.
    const isMultiple = this.multiple || this.hasAttribute("multiple");

    if (!isMultiple && Array.isArray(val)) {
      val = val[0];
    }

    return val;
  }

  private _value: string[] | undefined | null;

  /** The select's value. This will be a string for single select or an array for multi-select. */
  @property({ attribute: "value", reflect: false })
  set value(val: string | string[] | null) {
    let oldValue = this.value;

    if ((val as any) instanceof FormData) {
      val = (val as unknown as FormData).getAll(this.name) as string[];
    }

    if (val != null && !Array.isArray(val)) {
      val = [val];
    }

    const oldRawValue = this._value;
    this._value = val ?? null;

    // Compare raw internal values to detect actual changes. We can't rely on the getter because it filters through
    // optionValues, which may be empty when options aren't in the DOM yet (common with frameworks that set properties
    // before appending children).
    if (!this.rawValuesEqual(oldRawValue, this._value)) {
      this.valueHasChanged = true;
      this.requestUpdate("value", oldValue);
    }
  }

  get value() {
    let value = this._value ?? this.defaultValue ?? null;

    if (value != null) {
      value = Array.isArray(value) ? value : [value];
    }

    // Build optionValues from the cached options list. This is rebuilt each time the getter is called rather than
    // cached, because caching created stale-state bugs when the value was set before options existed in the DOM. The
    // underlying getAllOptions() is already cached via cachedOptions, so this is cheap.
    this.optionValues = new Set(
      this.getAllOptions()
        .filter((option) => !option.disabled)
        .map((option) => option.value),
    );

    // Drop values not in the DOM
    let ret: null | string | string[] = value;
    if (value != null) {
      ret = value.filter((v) => this.optionValues!.has(v));
      ret = this.multiple ? ret : ret[0];
      ret = ret ?? null;
    }

    return ret;
  }

  /** The select's size. */
  @property({ reflect: true }) size: "xs" | "s" | "m" | "l" | "xl" | "small" | "medium" | "large" =
    "m";

  @watch("size")
  handleSizeChange() {
    warnDeprecatedSize(this.localName, this.size);
  }

  /** Placeholder text to show as a hint when the select is empty. */
  @property() placeholder = "";

  /** Allows more than one option to be selected. */
  @property({ type: Boolean, reflect: true }) multiple = false;

  /**
   * The maximum number of selected options to show when `multiple` is true. After the maximum, "+n" will be shown to
   * indicate the number of additional items that are selected. Set to 0 to remove the limit.
   */
  @property({ attribute: "max-options-visible", type: Number }) maxOptionsVisible = 3;

  /** Disables the select control. */
  @property({ type: Boolean }) disabled = false;

  /** Adds a clear button when the select is not empty. */
  @property({ attribute: "with-clear", type: Boolean }) withClear = false;

  /**
   * Indicates whether or not the select is open. You can toggle this attribute to show and hide the menu, or you can
   * use the `show()` and `hide()` methods and this attribute will reflect the select's open state.
   */
  @property({ type: Boolean, reflect: true }) open = false;

  /** The select's visual appearance. */
  @property({ reflect: true }) appearance: "filled" | "outlined" | "filled-outlined" = "outlined";

  /** Draws a pill-style select with rounded edges. */
  @property({ type: Boolean, reflect: true }) pill = false;

  /** The select's label. If you need to display HTML, use the `label` slot instead. */
  @property() label = "";

  /**
   * The preferred placement of the select's menu. Note that the actual placement may vary as needed to keep the listbox
   * inside of the viewport.
   */
  @property({ reflect: true }) placement: "top" | "bottom" = "bottom";

  /** The select's hint. If you need to display HTML, use the `hint` slot instead. */
  @property({ attribute: "hint" }) hint = "";

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

  /** The select's required attribute. */
  @property({ type: Boolean, reflect: true }) required = false;

  /**
   * A function that customizes the tags to be rendered when multiple=true. The first argument is the option, the second
   * is the current tag's index.  The function should return either a Lit TemplateResult or a string containing trusted
   * HTML of the symbol to render at the specified value.
   */
  @property({ attribute: false }) getTag: (
    option: WaOption,
    index: number,
  ) => TemplateResult | string | HTMLElement = (option) => {
    return html`
      <wa-tag
        part="tag"
        exportparts="
            base:tag__base,
            content:tag__content,
            remove-button:tag__remove-button,
            remove-button__base:tag__remove-button__base
          "
        ?pill=${this.pill}
        size=${this.size}
        with-remove
        data-value=${option.value}
        @wa-remove=${(event: WaRemoveEvent) => this.handleTagRemove(event, option)}
      >
        ${option.label}
      </wa-tag>
    `;
  };

  connectedCallback() {
    super.connectedCallback();

    // Call processSlotChange directly so initial setup is synchronous.
    // Subsequent option additions will be batched via handleDefaultSlotChange.
    this.processSlotChange();

    // Because this is a form control, it shouldn't be opened initially
    this.open = false;
  }

  disconnectedCallback() {
    super.disconnectedCallback();
    this.removeOpenListeners();
    this.cachedOptions = null;
  }

  private updateDefaultValue() {
    const allOptions = this.getAllOptions();
    const defaultSelectedOptions = allOptions.filter(
      (el) => el.hasAttribute("selected") || el.defaultSelected,
    );
    if (defaultSelectedOptions.length > 0) {
      const selectedValues = defaultSelectedOptions.map((el) => el.value);
      this._defaultValue = this.multiple ? selectedValues : selectedValues[0];
    }
    if (this.hasAttribute("value")) {
      this._defaultValue = this.getAttribute("value") || null;
    }
  }

  private addOpenListeners() {
    //
    // Listen on the root node instead of the document in case the elements are inside a shadow root
    //
    // https://github.com/shoelace-style/shoelace/issues/1763
    //
    document.addEventListener("focusin", this.handleDocumentFocusIn);
    document.addEventListener("keydown", this.handleDocumentKeyDown);
    document.addEventListener("mousedown", this.handleDocumentMouseDown);
    registerDismissible(this);

    // If the component is rendered in a shadow root, we need to attach the focusin listener there too
    if (this.getRootNode() !== document) {
      this.getRootNode().addEventListener("focusin", this.handleDocumentFocusIn);
    }
  }

  private removeOpenListeners() {
    document.removeEventListener("focusin", this.handleDocumentFocusIn);
    document.removeEventListener("keydown", this.handleDocumentKeyDown);
    document.removeEventListener("mousedown", this.handleDocumentMouseDown);
    unregisterDismissible(this);

    if (this.getRootNode() !== document) {
      this.getRootNode().removeEventListener("focusin", this.handleDocumentFocusIn);
    }
  }

  private handleFocus() {
    this.displayInput.setSelectionRange(0, 0);
  }

  private handleDocumentFocusIn = (event: KeyboardEvent) => {
    // Close when focusing out of the select
    const path = event.composedPath();
    if (this && !path.includes(this)) {
      this.hide();
    }
  };

  private handleDocumentKeyDown = (event: KeyboardEvent) => {
    const target = event.target as HTMLElement;
    const isClearButton = target.closest('[part~="clear-button"]') !== null;
    const isButton = target.closest("wa-button") !== null;

    // Ignore presses when the target is a button (e.g. the remove button in `<wa-tag>`)
    if (isClearButton || isButton) {
      return;
    }

    // Close when pressing escape
    if (event.key === "Escape" && this.open && isTopDismissible(this)) {
      event.preventDefault();
      event.stopPropagation();
      this.hide();
      this.displayInput.focus({ preventScroll: true });
    }

    // Handle enter and space. When pressing space, we allow for type to select behaviors so if there's anything in the
    // buffer we _don't_ close it.
    if (event.key === "Enter" || (event.key === " " && this.typeToSelectString === "")) {
      event.preventDefault();
      event.stopImmediatePropagation();

      // If it's not open, open it
      if (!this.open) {
        this.show();
        return;
      }

      // If it is open, update the value based on the current selection and close it
      if (this.currentOption && !this.currentOption.disabled) {
        this.valueHasChanged = true;
        this.hasInteracted = true;
        if (this.multiple) {
          this.toggleOptionSelection(this.currentOption);
        } else {
          this.setSelectedOptions(this.currentOption);
        }

        // Emit after updating
        this.updateComplete.then(() => {
          this.dispatchEvent(new InputEvent("input", { bubbles: true, composed: true }));
          this.dispatchEvent(new Event("change", { bubbles: true, composed: true }));
        });

        if (!this.multiple) {
          this.hide();
          this.displayInput.focus({ preventScroll: true });
        }
      }

      return;
    }

    // Navigate options
    if (["ArrowUp", "ArrowDown", "Home", "End"].includes(event.key)) {
      const allOptions = this.getAllOptions();
      const currentIndex = allOptions.indexOf(this.currentOption);
      let newIndex = Math.max(0, currentIndex);

      // Prevent scrolling
      event.preventDefault();

      // Open it
      if (!this.open) {
        this.show();

        // If an option is already selected, stop here because we want that one to remain highlighted when the listbox
        // opens for the first time
        if (this.currentOption) {
          return;
        }
      }

      if (event.key === "ArrowDown") {
        newIndex = currentIndex + 1;
        if (newIndex > allOptions.length - 1) newIndex = 0;
      } else if (event.key === "ArrowUp") {
        newIndex = currentIndex - 1;
        if (newIndex < 0) newIndex = allOptions.length - 1;
      } else if (event.key === "Home") {
        newIndex = 0;
      } else if (event.key === "End") {
        newIndex = allOptions.length - 1;
      }

      this.setCurrentOption(allOptions[newIndex]);
    }

    // All other "printable" keys trigger type to select
    if (event.key?.length === 1 || event.key === "Backspace") {
      const allOptions = this.getAllOptions();

      // Don't block important key combos like CMD+R
      if (event.metaKey || event.ctrlKey || event.altKey) {
        return;
      }

      // Open, unless the key that triggered is backspace
      if (!this.open) {
        if (event.key === "Backspace") {
          return;
        }

        this.show();
      }

      event.stopPropagation();
      event.preventDefault();

      clearTimeout(this.typeToSelectTimeout);
      this.typeToSelectTimeout = window.setTimeout(() => (this.typeToSelectString = ""), 1000);

      if (event.key === "Backspace") {
        this.typeToSelectString = this.typeToSelectString.slice(0, -1);
      } else {
        this.typeToSelectString += event.key.toLowerCase();
      }

      for (const option of allOptions) {
        const label = option.label.toLowerCase();

        if (label.startsWith(this.typeToSelectString)) {
          this.setCurrentOption(option);
          break;
        }
      }
    }
  };

  private handleDocumentMouseDown = (event: MouseEvent) => {
    // Close when clicking outside of the select
    const path = event.composedPath();
    if (this && !path.includes(this)) {
      this.hide();
    }
  };

  private handleLabelClick() {
    this.displayInput.focus();
  }

  private handleComboboxClick(event: MouseEvent) {
    event.preventDefault();
  }

  private handleComboboxMouseDown(event: MouseEvent) {
    const path = event.composedPath();
    const isButton = path.some(
      (el) => el instanceof Element && el.tagName.toLowerCase() === "wa-button",
    );

    // Ignore disabled controls and clicks on tags (remove buttons)
    if (this.disabled || isButton) {
      return;
    }

    event.preventDefault();
    this.displayInput.focus({ preventScroll: true });
    this.open = !this.open;
  }

  private handleComboboxKeyDown(event: KeyboardEvent) {
    event.stopPropagation();
    this.handleDocumentKeyDown(event);
  }

  private handleClearClick(event: MouseEvent) {
    event.stopPropagation();

    this.hasInteracted = true;
    this.valueHasChanged = true;

    if (this.value !== null) {
      this.displayLabel = "";
      this.selectionOrder.clear();
      this.setSelectedOptions([]);
      this.displayInput.focus({ preventScroll: true });

      // Emit after update
      this.updateComplete.then(() => {
        this.dispatchEvent(new WaClearEvent());
        this.dispatchEvent(new InputEvent("input", { bubbles: true, composed: true }));
        this.dispatchEvent(new Event("change", { bubbles: true, composed: true }));
      });
    }
  }

  private handleClearMouseDown(event: MouseEvent) {
    // Don't lose focus or propagate events when clicking the clear button
    event.stopPropagation();
    event.preventDefault();
  }

  private handleOptionClick(event: MouseEvent) {
    const target = event.target as HTMLElement;
    const option = target.closest("wa-option");

    if (option && !option.disabled) {
      this.hasInteracted = true;
      this.valueHasChanged = true;

      if (this.multiple) {
        this.toggleOptionSelection(option);
      } else {
        this.setSelectedOptions(option);
      }

      // Set focus after updating so the value is announced by screen readers
      this.updateComplete.then(() => this.displayInput.focus({ preventScroll: true }));

      this.requestUpdate("value");

      // Emit after updating
      this.updateComplete.then(() => {
        this.dispatchEvent(new InputEvent("input", { bubbles: true, composed: true }));
        this.dispatchEvent(new Event("change", { bubbles: true, composed: true }));
      });

      if (!this.multiple) {
        this.hide();
        this.displayInput.focus({ preventScroll: true });
      }
    }
  }

  /* @internal - used by options to update labels */
  public handleDefaultSlotChange() {
    if (this.slotChangePending) return;
    this.slotChangePending = true;
    queueMicrotask(() => {
      this.slotChangePending = false;
      this.processSlotChange();
    });
  }

  private processSlotChange() {
    if (!customElements.get("wa-option")) {
      customElements.whenDefined("wa-option").then(() => this.handleDefaultSlotChange());
    }

    // Invalidate the options cache since slots have changed
    this.cachedOptions = null;

    const allOptions = this.getAllOptions();

    // Update defaultValue if it hasn't been explicitly set and we have selected options
    this.updateDefaultValue();

    let value = this.value;

    if (value == null || (!this.valueHasChanged && !this.hasInteracted)) {
      this.selectionChanged();
      return;
    }

    if (!Array.isArray(value)) {
      value = [value];
    }

    // Select only the options that match the new value
    const selectedOptions = allOptions.filter((el) => value.includes(el.value));
    this.setSelectedOptions(selectedOptions);
  }

  private handleTagRemove(event: WaRemoveEvent, directOption?: WaOption) {
    event.stopPropagation();

    if (this.disabled) return;

    // Mark as interacted so selectionChanged() uses the correct filter logic
    this.hasInteracted = true;
    this.valueHasChanged = true;

    // Use the directly provided option if available (from getTag method)
    let option = directOption;

    // If no direct option was provided, find the option from the data-value attribute
    if (!option) {
      const tagElement = (event.target as Element).closest(
        "wa-tag[data-value]",
      ) as HTMLElement | null;

      if (tagElement) {
        const value = tagElement.dataset.value;
        option = this.selectedOptions.find((opt) => opt.value === value);
      }
    }

    if (option) {
      this.toggleOptionSelection(option, false);

      // Emit after updating
      this.updateComplete.then(() => {
        this.dispatchEvent(new InputEvent("input", { bubbles: true, composed: true }));
        this.dispatchEvent(new Event("change", { bubbles: true, composed: true }));
      });
    }
  }

  // Gets an array of all `<wa-option>` elements
  private getAllOptions() {
    if (this.cachedOptions) return this.cachedOptions;
    if (!this?.querySelectorAll) {
      return [];
    }
    this.cachedOptions = [...this.querySelectorAll<WaOption>("wa-option")];
    return this.cachedOptions;
  }

  // Gets the first `<wa-option>` element
  private getFirstOption() {
    return this.querySelector<WaOption>("wa-option");
  }

  // Sets the current option, which is the option the user is currently interacting with (e.g. via keyboard). Only one
  // option may be "current" at a time.
  private setCurrentOption(option: WaOption | null) {
    const allOptions = this.getAllOptions();

    // Clear selection
    allOptions.forEach((el) => {
      el.current = false;
      el.tabIndex = -1;
    });

    // Select the target option
    if (option) {
      this.currentOption = option;
      option.current = true;
      option.tabIndex = 0;
      option.focus({ preventScroll: true });
    }
  }

  // Sets the selected option(s)
  private setSelectedOptions(option: WaOption | WaOption[]) {
    const allOptions = this.getAllOptions();
    const newSelectedOptions = Array.isArray(option) ? option : [option];

    // Clear existing selection
    allOptions.forEach((el) => {
      if (newSelectedOptions.includes(el)) {
        return;
      }
      el.selected = false;
    });

    // Set the new selection
    if (newSelectedOptions.length) {
      newSelectedOptions.forEach((el) => (el.selected = true));
    }

    // Update selection, value, and display label
    this.selectionChanged();
  }

  // Toggles an option's selected state
  private toggleOptionSelection(option: WaOption, force?: boolean) {
    if (force === true || force === false) {
      option.selected = force;
    } else {
      option.selected = !option.selected;
    }

    this.selectionChanged();
  }

  // @internal This method must be called whenever the selection changes. It will update the selected options cache, the
  // current value, and the display value. The option component uses it internally to update labels as they change.
  public selectionChanged() {
    const options = this.getAllOptions();

    // Update selected options cache
    const newSelectedOptions = options.filter((el) => {
      if (!this.hasInteracted && !this.valueHasChanged) {
        const defaultValue = this.defaultValue;
        const defaultValues = Array.isArray(defaultValue) ? defaultValue : [defaultValue];
        return (
          el.hasAttribute("selected") ||
          el.defaultSelected ||
          el.selected ||
          defaultValues?.includes(el.value)
        );
      }

      return el.selected;
    });

    // Update the selection order map
    const newSelectedValues = new Set(newSelectedOptions.map((el) => el.value));

    // Remove deselected options from the order map
    for (const value of this.selectionOrder.keys()) {
      if (!newSelectedValues.has(value)) {
        this.selectionOrder.delete(value);
      }
    }

    // Add newly selected options
    const maxOrder = this.selectionOrder.size > 0 ? Math.max(...this.selectionOrder.values()) : -1;
    let nextOrder = maxOrder + 1;
    for (const option of newSelectedOptions) {
      if (!this.selectionOrder.has(option.value)) {
        this.selectionOrder.set(option.value, nextOrder++);
      }
    }

    // Sort options by selection order
    this.selectedOptions = newSelectedOptions.sort((a, b) => {
      const orderA = this.selectionOrder.get(a.value) ?? 0;
      const orderB = this.selectionOrder.get(b.value) ?? 0;
      return orderA - orderB;
    });

    let selectedValues = new Set(this.selectedOptions.map((el) => el.value));

    // Toggle values present in the DOM from this.value, while preserving options NOT present in the DOM (for lazy loading)
    // Note that options NOT present in the DOM will be moved to the end after this
    if (selectedValues.size > 0 || this._value) {
      const oldValue = this._value;
      if (this._value == null) {
        // First time it's set
        let value = this.defaultValue ?? [];
        this._value = Array.isArray(value) ? value : [value];
      }

      // Filter out values that are in the DOM
      this._value = this._value?.filter((value) => !this.optionValues?.has(value)) ?? null;
      this._value?.unshift(...selectedValues);
      this.requestUpdate("value", oldValue);
    }

    // Update the value and display label
    if (this.multiple) {
      if (this.placeholder && !this.value?.length) {
        // When no items are selected, keep the value empty so the placeholder shows
        this.displayLabel = "";
      } else {
        this.displayLabel = this.localize.term("numOptionsSelected", this.selectedOptions.length);
      }
    } else {
      const selectedOption = this.selectedOptions[0];
      this.displayLabel = selectedOption?.label ?? "";
    }

    // Update validity
    this.updateComplete.then(() => {
      this.updateValidity();
    });
  }

  protected get tags() {
    return this.selectedOptions.map((option, index) => {
      if (index < this.maxOptionsVisible || this.maxOptionsVisible <= 0) {
        const tag = this.getTag(option, index);
        if (!tag) return null;
        return typeof tag === "string" ? unsafeHTML(tag) : tag;
      } else if (index === this.maxOptionsVisible) {
        // Hit tag limit
        return html`
          <wa-tag
            part="tag"
            exportparts="
              base:tag__base,
              content:tag__content,
              remove-button:tag__remove-button,
              remove-button__base:tag__remove-button__base
            "
          >
            +${this.selectedOptions.length - index}
          </wa-tag>
        `;
      }
      return null;
    });
  }

  updated(changedProperties: PropertyValues<this>) {
    super.updated(changedProperties);

    if (changedProperties.has("value") || changedProperties.has("displayLabel")) {
      this.customStates.set("blank", !this.value && !this.displayLabel);
    }
  }

  @watch("disabled", { waitUntilFirstUpdate: true })
  handleDisabledChange() {
    // Close the listbox when the control is open and disabled
    if (this.disabled && this.open) {
      this.open = false;
    }
  }

  @watch("value", { waitUntilFirstUpdate: true })
  handleValueChange() {
    const allOptions = this.getAllOptions();
    const value = Array.isArray(this.value) ? this.value : [this.value];

    // Select only the options that match the new value
    const selectedOptions = allOptions.filter((el) => value.includes(el.value));
    this.setSelectedOptions(selectedOptions);
    this.updateValidity();
  }

  @watch("open", { waitUntilFirstUpdate: true })
  async handleOpenChange() {
    if (this.open && !this.disabled) {
      // Reset the current option
      this.setCurrentOption(this.selectedOptions[0] || this.getFirstOption());

      // Show
      const waShowEvent = new WaShowEvent();
      this.dispatchEvent(waShowEvent);
      if (waShowEvent.defaultPrevented) {
        this.open = false;
        return;
      }

      this.addOpenListeners();
      this.listbox.hidden = false;
      this.popup.active = true;

      // Select the appropriate option based on value after the listbox opens
      requestAnimationFrame(() => {
        this.setCurrentOption(this.currentOption);
      });

      await animateWithClass(this.popup.popup, "show");

      // Make sure the current option is scrolled into view (required for Safari)
      if (this.currentOption) {
        scrollIntoView(this.currentOption, this.listbox, "vertical", "auto");
      }

      this.dispatchEvent(new WaAfterShowEvent());
    } else {
      // Hide
      const waHideEvent = new WaHideEvent();
      this.dispatchEvent(waHideEvent);
      if (waHideEvent.defaultPrevented) {
        this.open = false;
        return;
      }

      this.removeOpenListeners();
      await animateWithClass(this.popup.popup, "hide");
      this.listbox.hidden = true;
      this.popup.active = false;

      this.dispatchEvent(new WaAfterHideEvent());
    }
  }

  /** Shows the listbox. */
  async show() {
    if (this.open || this.disabled) {
      this.open = false;
      return undefined;
    }

    this.open = true;
    return waitForEvent(this, "wa-after-show");
  }

  /** Hides the listbox. */
  async hide() {
    if (!this.open || this.disabled) {
      this.open = false;
      return undefined;
    }

    this.open = false;
    return waitForEvent(this, "wa-after-hide");
  }

  /** Sets focus on the control. */
  focus(options?: FocusOptions) {
    this.displayInput.focus(options);
  }

  /** Removes focus from the control. */
  blur() {
    this.displayInput.blur();
  }

  formResetCallback() {
    this.selectionOrder.clear();
    this.value = this.defaultValue;
    super.formResetCallback();
    this.handleValueChange();

    this.updateComplete.then(() => {
      this.dispatchEvent(new InputEvent("input", { bubbles: true, composed: true }));
      this.dispatchEvent(new Event("change", { bubbles: true, composed: true }));
    });
  }

  render() {
    const hasLabelSlot = this.hasUpdated ? this.hasSlotController.test("label") : this.withLabel;
    const hasHintSlot = this.hasUpdated ? this.hasSlotController.test("hint") : this.withHint;
    const hasLabel = this.label ? true : !!hasLabelSlot;
    const hasHint = this.hint ? true : !!hasHintSlot;
    const hasClearIcon =
      (this.hasUpdated || isServer) &&
      this.withClear &&
      !this.disabled &&
      (this.displayLabel || (this.value && this.value.length > 0));

    return html`
      <div
        part="form-control"
        class=${classMap({
          "form-control": true,
          "form-control-has-label": hasLabel,
        })}
      >
        <label
          id="label"
          part="form-control-label label"
          class=${classMap({
            label: true,
            "has-label": hasLabel,
          })}
          aria-hidden=${hasLabel ? "false" : "true"}
          @click=${this.handleLabelClick}
        >
          <slot name="label">${this.label}</slot>
        </label>

        <div part="form-control-input" class="form-control-input">
          <wa-popup
            class=${classMap({
              select: true,
              open: this.open,
              disabled: this.disabled,
              enabled: !this.disabled,
              multiple: this.multiple,
            })}
            placement=${this.placement}
            flip
            shift
            sync="width"
            auto-size="vertical"
            auto-size-padding="10"
          >
            <div
              part="combobox"
              class="combobox"
              slot="anchor"
              @keydown=${this.handleComboboxKeyDown}
              @mousedown=${this.handleComboboxMouseDown}
              @click=${this.handleComboboxClick}
            >
              <slot part="start" name="start" class="start"></slot>

              <input
                part="display-input"
                class="display-input"
                type="text"
                placeholder=${this.placeholder}
                .disabled=${this.disabled}
                .value=${this.displayLabel}
                ?required=${this.required}
                autocomplete="off"
                spellcheck="false"
                autocapitalize="off"
                readonly
                aria-invalid=${
                  !this.validity.valid
                  /** aria-invalid is required because readonly inputs are technically always valid so it never reads 'invalid data' for screen readers. */
                }
                aria-controls="listbox"
                aria-expanded=${this.open ? "true" : "false"}
                aria-haspopup="listbox"
                aria-labelledby="label"
                aria-disabled=${this.disabled ? "true" : "false"}
                aria-describedby="hint"
                role="combobox"
                tabindex="0"
                @focus=${this.handleFocus}
              />

              <!-- Tags need to wait for first hydration before populating otherwise it will create a hydration mismatch. -->
              ${
                this.multiple && this.hasUpdated
                  ? html`
                      <div part="tags" class="tags" @wa-remove=${this.handleTagRemove}>
                        ${this.tags}
                      </div>
                    `
                  : ""
              }

              <input
                class="value-input"
                type="text"
                ?disabled=${this.disabled}
                ?required=${this.required}
                .value=${Array.isArray(this.value) ? this.value.join(", ") : this.value}
                tabindex="-1"
                aria-hidden="true"
                @focus=${() => this.focus()}
              />

              ${
                hasClearIcon
                  ? html`
                      <button
                        part="clear-button"
                        type="button"
                        aria-label=${this.localize.term("clearEntry")}
                        @mousedown=${this.handleClearMouseDown}
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

              <slot name="end" part="end" class="end"></slot>

              <slot name="expand-icon" part="expand-icon" class="expand-icon">
                <wa-icon library="system" name="chevron-down" variant="solid"></wa-icon>
              </slot>
            </div>

            <div
              id="listbox"
              role="listbox"
              aria-expanded=${this.open ? "true" : "false"}
              aria-multiselectable=${this.multiple ? "true" : "false"}
              aria-labelledby="label"
              part="listbox"
              class="listbox"
              tabindex="-1"
              @mouseup=${this.handleOptionClick}
            >
              <slot @slotchange=${this.handleDefaultSlotChange}></slot>
            </div>
          </wa-popup>
        </div>

        <slot
          id="hint"
          name="hint"
          part="hint"
          class=${classMap({
            "has-slotted": hasHint,
          })}
          aria-hidden=${hasHint ? "false" : "true"}
        >
          ${this.hint}
        </slot>
      </div>
    `;
  }
}

// The change-in-update warning is required for this component because the form-associated base class calls
// updateValidity() in firstUpdated(), which triggers requestUpdate('validity') to sync the validation state after the
// first render when the validation target is available. Additionally, HasSlotController triggers requestUpdate() on
// initial slotchange events, and selectionChanged() sets @state properties (displayLabel, selectedOptions) in response
// to slot content changes. See https://lit.dev/docs/tools/development/#development-build-runtime-warnings
WaSelect.disableWarning?.("change-in-update");

declare global {
  interface HTMLElementTagNameMap {
    "wa-select": WaSelect;
  }
}

`````
