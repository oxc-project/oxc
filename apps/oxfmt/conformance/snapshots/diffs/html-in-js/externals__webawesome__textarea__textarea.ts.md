# externals/webawesome/textarea/textarea.ts

## Option 1

`````json
{"printWidth":80}
`````

### Diff

`````diff
===================================================================
--- prettier
+++ oxfmt
@@ -94,18 +94,27 @@
     this.getAttribute("value") ?? "";
 
   /** The textarea's size. */
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
 
   /** The textarea's visual appearance. */
   @property({ reflect: true }) appearance:
-    "filled" | "outlined" | "filled-outlined" = "outlined";
+    | "filled"
+    | "outlined"
+    | "filled-outlined" = "outlined";
 
   /** The textarea's label. If you need to display HTML, use the `label` slot instead. */
   @property() label = "";
 
@@ -119,9 +128,13 @@
   @property({ type: Number }) rows = 4;
 
   /** Controls how the textarea can be resized. */
   @property({ reflect: true }) resize:
-    "none" | "vertical" | "horizontal" | "both" | "auto" = "vertical";
+    | "none"
+    | "vertical"
+    | "horizontal"
+    | "both"
+    | "auto" = "vertical";
 
   /** Disables the textarea. */
   @property({ type: Boolean }) disabled = false;
 
@@ -138,9 +151,14 @@
   @property({ type: Number }) maxlength: number;
 
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
@@ -164,9 +182,15 @@
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
 
   /** Enables spell checking on the textarea. */
   @property({
     type: Boolean,
@@ -457,9 +481,12 @@
         <textarea
           part="textarea"
           id="input"
           class="control"
-          title=${this.title /* An empty title prevents browser validation tooltips from appearing on hover */}
+          title=${
+            this
+              .title /* An empty title prevents browser validation tooltips from appearing on hover */
+          }
           name=${ifDefined(this.name)}
           .value=${live(this.value)}
           ?disabled=${this.disabled}
           ?readonly=${this.readonly}
@@ -504,20 +531,18 @@
           })}
           >${this.hint}</slot
         >
 
-        ${
-          this.withCount
-            ? html`
-                <div part="count" class="count" aria-hidden="true">
-                  ${countText}
-                </div>
-                <div class="wa-visually-hidden-force" aria-live="polite">
-                  ${this.announcedCountText}
-                </div>
-              `
-            : ""
-        }
+        ${this.withCount
+          ? html`
+              <div part="count" class="count" aria-hidden="true">
+                ${countText}
+              </div>
+              <div class="wa-visually-hidden-force" aria-live="polite">
+                ${this.announcedCountText}
+              </div>
+            `
+          : ""}
       </div>
     `;
   }
 }

`````

### Actual (oxfmt)

`````ts
import type { PropertyValues } from "lit";
import { html } from "lit";
import { customElement, property, query, state } from "lit/decorators.js";
import { classMap } from "lit/directives/class-map.js";
import { ifDefined } from "lit/directives/if-defined.js";
import { live } from "lit/directives/live.js";
import { warnDeprecatedSize } from "../../internal/size.js";
import { HasSlotController } from "../../internal/slot.js";
import { MirrorValidator } from "../../internal/validators/mirror-validator.js";
import { watch } from "../../internal/watch.js";
import { WebAwesomeFormAssociatedElement } from "../../internal/webawesome-form-associated-element.js";
import formControlStyles from "../../styles/component/form-control.styles.js";
import sizeStyles from "../../styles/component/size.styles.js";
import visuallyHidden from "../../styles/component/visually-hidden.styles.js";
import { LocalizeController } from "../../utilities/localize.js";
import styles from "./textarea.styles.js";

/**
 * @summary Textareas collect multi-line text input from the user, with optional resizing and character counting.
 * @documentation https://webawesome.com/docs/components/textarea
 * @status stable
 * @since 2.0
 *
 * @slot label - The textarea's label. Alternatively, you can use the `label` attribute.
 * @slot hint - Text that describes how to use the input. Alternatively, you can use the `hint` attribute.
 *
 * @event blur - Emitted when the control loses focus.
 * @event change - Emitted when an alteration to the control's value is committed by the user.
 * @event focus - Emitted when the control gains focus.
 * @event input - Emitted when the control receives input.
 * @event wa-invalid - Emitted when the form control has been checked for validity and its constraints aren't satisfied.
 *
 * @csspart label - The label
 * @csspart form-control-input - The input's wrapper.
 * @csspart hint - The hint's wrapper.
 * @csspart textarea - The internal `<textarea>` control.
 * @csspart base - The wrapper around the `<textarea>` control.
 * @csspart count - The character count element, rendered when the `with-count` attribute is present.
 *
 * @cssstate blank - The textarea is empty.
 */
@customElement("wa-textarea")
export default class WaTextarea extends WebAwesomeFormAssociatedElement {
  static css = [styles, formControlStyles, sizeStyles, visuallyHidden];

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
  private resizeObserver?: ResizeObserver;
  private countAnnounceTimeout: ReturnType<typeof setTimeout>;

  @state() private announcedCountText = "";

  @query(".control") input: HTMLTextAreaElement;
  @query('[part~="base"]') base: HTMLDivElement;
  @query(".size-adjuster") sizeAdjuster: HTMLTextAreaElement;

  @property() title = ""; // make reactive to pass through

  /** The name of the textarea, submitted as a name/value pair with form data. */
  @property({ reflect: true }) name: string | null = null;

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
  @property({ attribute: "value", reflect: true }) defaultValue: string =
    this.getAttribute("value") ?? "";

  /** The textarea's size. */
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

  /** The textarea's visual appearance. */
  @property({ reflect: true }) appearance:
    | "filled"
    | "outlined"
    | "filled-outlined" = "outlined";

  /** The textarea's label. If you need to display HTML, use the `label` slot instead. */
  @property() label = "";

  /** The textarea's hint. If you need to display HTML, use the `hint` slot instead. */
  @property({ attribute: "hint" }) hint = "";

  /** Placeholder text to show as a hint when the input is empty. */
  @property() placeholder = "";

  /** The number of rows to display by default. */
  @property({ type: Number }) rows = 4;

  /** Controls how the textarea can be resized. */
  @property({ reflect: true }) resize:
    | "none"
    | "vertical"
    | "horizontal"
    | "both"
    | "auto" = "vertical";

  /** Disables the textarea. */
  @property({ type: Boolean }) disabled = false;

  /** Makes the textarea readonly. */
  @property({ type: Boolean, reflect: true }) readonly = false;

  /** Makes the textarea a required field. */
  @property({ type: Boolean, reflect: true }) required = false;

  /** The minimum length of input that will be considered valid. */
  @property({ type: Number }) minlength: number;

  /** The maximum length of input that will be considered valid. */
  @property({ type: Number }) maxlength: number;

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

  /** Enables spell checking on the textarea. */
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

  /** Shows a character count below the textarea. When `maxlength` is set, shows remaining characters instead. */
  @property({ attribute: "with-count", type: Boolean, reflect: true })
  withCount = false;

  connectedCallback() {
    super.connectedCallback();

    this.updateComplete.then(() => {
      this.setTextareaDimensions();
      this.updateResizeObserver();

      if (this.didSSR && this.input && this.value !== this.input.value) {
        const value = this.input.value;

        this.value = value;
      }
    });
  }

  disconnectedCallback() {
    super.disconnectedCallback();
    clearTimeout(this.countAnnounceTimeout);
    this.resizeObserver?.disconnect();
    this.resizeObserver = undefined;
  }

  /** Creates or destroys the resize observer based on the current resize mode. */
  private updateResizeObserver() {
    // The resize observer is only needed for manual resize modes (vertical, horizontal, both)
    // to sync the base wrapper dimensions with the textarea.
    const needsObserver = this.resize !== "none" && this.resize !== "auto";

    if (needsObserver && !this.resizeObserver && this.input) {
      this.resizeObserver = new ResizeObserver(() =>
        this.setTextareaDimensions(),
      );
      this.resizeObserver.observe(this.input);
    } else if (!needsObserver && this.resizeObserver) {
      this.resizeObserver.disconnect();
      this.resizeObserver = undefined;
    }
  }

  private handleBlur() {
    this.checkValidity();
  }

  private handleChange(event: Event) {
    this.valueHasChanged = true;
    this.value = this.input.value;
    this.setTextareaDimensions();
    this.checkValidity();
    this.relayNativeEvent(event, { bubbles: true, composed: true });
  }

  private handleInput(event: InputEvent) {
    this.valueHasChanged = true;
    this.value = this.input.value;
    this.relayNativeEvent(event, { bubbles: true, composed: true });
    this.scheduleCountAnnouncement();
  }

  private scheduleCountAnnouncement() {
    clearTimeout(this.countAnnounceTimeout);
    this.countAnnounceTimeout = setTimeout(() => {
      const currentLength = (this.value ?? "").length;
      this.announcedCountText =
        this.maxlength != null
          ? this.localize.term(
              "numCharactersRemaining",
              this.maxlength - currentLength,
            )
          : this.localize.term("numCharacters", currentLength);
    }, 1000);
  }

  private setTextareaDimensions() {
    if (this.resize === "none") {
      // just in case this is called via a property changing.
      this.base.style.width = ``;
      this.base.style.height = ``;
      return;
    }

    if (this.resize === "auto") {
      // This prevents layout shifts. We use `clientHeight` instead of `scrollHeight` to account for if the `<textarea>`
      // has a max-height set on it. In my tests, this has worked fine. Im not aware of any edge cases. [Konnor]
      // Let's switch to `field-sizing: content` once it has better support: https://caniuse.com/mdn-css_properties_field-sizing [Lea]
      this.sizeAdjuster.style.height = `${this.input.clientHeight}px`;
      this.input.style.height = "auto";
      this.input.style.height = `${this.input.scrollHeight}px`;

      this.base.style.width = ``;
      this.base.style.height = ``;
      return;
    }

    // handles vertical, horizontal, and both resizers:

    // These should always be set by a manual resize operation , so its reasonable to expect px.
    if (this.input.style.width) {
      const width = Number(this.input.style.width.split(/px/)[0]) + 2;
      this.base.style.width = `${width}px`;
    }

    if (this.input.style.height) {
      const height = Number(this.input.style.height.split(/px/)[0]) + 2;
      this.base.style.height = `${height}px`;
    }
  }

  @watch("rows", { waitUntilFirstUpdate: true })
  handleRowsChange() {
    this.setTextareaDimensions();
  }

  @watch("value", { waitUntilFirstUpdate: true })
  async handleValueChange() {
    await this.updateComplete;
    this.checkValidity();
    this.setTextareaDimensions();
  }

  updated(changedProperties: PropertyValues<this>) {
    if (changedProperties.has("resize")) {
      this.setTextareaDimensions();
      this.updateResizeObserver();
    }

    super.updated(changedProperties);

    if (changedProperties.has("value")) {
      this.customStates.set("blank", !this.value);
    }
  }

  /** Sets focus on the textarea. */
  focus(options?: FocusOptions) {
    this.input.focus(options);
  }

  /** Removes focus from the textarea. */
  blur() {
    this.input.blur();
  }

  /** Selects all the text in the textarea. */
  select() {
    this.input.select();
  }

  /** Gets or sets the textarea's scroll position. */
  scrollPosition(position?: {
    top?: number;
    left?: number;
  }): { top: number; left: number } | undefined {
    if (position) {
      if (typeof position.top === "number") this.input.scrollTop = position.top;
      if (typeof position.left === "number")
        this.input.scrollLeft = position.left;
      return undefined;
    }

    return {
      top: this.input.scrollTop,
      left: this.input.scrollTop,
    };
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
    const selectionStart = start ?? this.input.selectionStart;
    const selectionEnd = end ?? this.input.selectionEnd;

    this.input.setRangeText(
      replacement,
      selectionStart,
      selectionEnd,
      selectMode,
    );

    if (this.value !== this.input.value) {
      this.value = this.input.value;
      this.setTextareaDimensions();
    }
  }

  formResetCallback() {
    this._value = null;

    if (this.input) {
      this.input.value = this.value || "";
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

    // We use .length intentionally here instead of Intl.Segmenter so the count stays consistent with the browser's
    // native maxlength enforcement, which also counts UTF-16 code units.
    const currentLength = (this.value ?? "").length;
    const countText =
      this.maxlength != null
        ? this.localize.term(
            "numCharactersRemaining",
            this.maxlength - currentLength,
          )
        : this.localize.term("numCharacters", currentLength);

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

      <div part="base" class="textarea">
        <textarea
          part="textarea"
          id="input"
          class="control"
          title=${
            this
              .title /* An empty title prevents browser validation tooltips from appearing on hover */
          }
          name=${ifDefined(this.name)}
          .value=${live(this.value)}
          ?disabled=${this.disabled}
          ?readonly=${this.readonly}
          ?required=${this.required}
          placeholder=${ifDefined(this.placeholder)}
          rows=${ifDefined(this.rows)}
          minlength=${ifDefined(this.minlength)}
          maxlength=${ifDefined(this.maxlength)}
          autocapitalize=${ifDefined(this.autocapitalize)}
          autocorrect=${ifDefined(this.autocorrect)}
          ?autofocus=${this.autofocus}
          spellcheck=${ifDefined(this.spellcheck)}
          enterkeyhint=${ifDefined(this.enterkeyhint)}
          inputmode=${ifDefined(this.inputmode)}
          aria-describedby="hint"
          @change=${this.handleChange}
          @input=${this.handleInput}
          @blur=${this.handleBlur}
        ></textarea>

        <!-- This "adjuster" exists to prevent layout shifting. https://github.com/shoelace-style/shoelace/issues/2180 -->
        <div
          part="textarea-adjuster"
          class="size-adjuster"
          ?hidden=${this.resize !== "auto"}
        ></div>
      </div>

      <div
        class=${classMap({
          footer: true,
          "has-count": this.withCount,
        })}
      >
        <slot
          id="hint"
          name="hint"
          part="hint"
          aria-hidden=${hasHint ? "false" : "true"}
          class=${classMap({
            "has-slotted": hasHint,
          })}
          >${this.hint}</slot
        >

        ${this.withCount
          ? html`
              <div part="count" class="count" aria-hidden="true">
                ${countText}
              </div>
              <div class="wa-visually-hidden-force" aria-live="polite">
                ${this.announcedCountText}
              </div>
            `
          : ""}
      </div>
    `;
  }
}

// The change-in-update warning is required for this component because the base class (WebAwesomeFormAssociatedElement)
// calls updateValidity() in firstUpdated(), which triggers requestUpdate('validity') to sync validation state after
// the DOM is available. This second update cycle cannot be avoided since validation requires the rendered DOM for
// anchoring. See https://lit.dev/docs/tools/development/#development-build-runtime-warnings
WaTextarea.disableWarning?.("change-in-update");

declare global {
  interface HTMLElementTagNameMap {
    "wa-textarea": WaTextarea;
  }
}

`````

### Expected (prettier)

`````ts
import type { PropertyValues } from "lit";
import { html } from "lit";
import { customElement, property, query, state } from "lit/decorators.js";
import { classMap } from "lit/directives/class-map.js";
import { ifDefined } from "lit/directives/if-defined.js";
import { live } from "lit/directives/live.js";
import { warnDeprecatedSize } from "../../internal/size.js";
import { HasSlotController } from "../../internal/slot.js";
import { MirrorValidator } from "../../internal/validators/mirror-validator.js";
import { watch } from "../../internal/watch.js";
import { WebAwesomeFormAssociatedElement } from "../../internal/webawesome-form-associated-element.js";
import formControlStyles from "../../styles/component/form-control.styles.js";
import sizeStyles from "../../styles/component/size.styles.js";
import visuallyHidden from "../../styles/component/visually-hidden.styles.js";
import { LocalizeController } from "../../utilities/localize.js";
import styles from "./textarea.styles.js";

/**
 * @summary Textareas collect multi-line text input from the user, with optional resizing and character counting.
 * @documentation https://webawesome.com/docs/components/textarea
 * @status stable
 * @since 2.0
 *
 * @slot label - The textarea's label. Alternatively, you can use the `label` attribute.
 * @slot hint - Text that describes how to use the input. Alternatively, you can use the `hint` attribute.
 *
 * @event blur - Emitted when the control loses focus.
 * @event change - Emitted when an alteration to the control's value is committed by the user.
 * @event focus - Emitted when the control gains focus.
 * @event input - Emitted when the control receives input.
 * @event wa-invalid - Emitted when the form control has been checked for validity and its constraints aren't satisfied.
 *
 * @csspart label - The label
 * @csspart form-control-input - The input's wrapper.
 * @csspart hint - The hint's wrapper.
 * @csspart textarea - The internal `<textarea>` control.
 * @csspart base - The wrapper around the `<textarea>` control.
 * @csspart count - The character count element, rendered when the `with-count` attribute is present.
 *
 * @cssstate blank - The textarea is empty.
 */
@customElement("wa-textarea")
export default class WaTextarea extends WebAwesomeFormAssociatedElement {
  static css = [styles, formControlStyles, sizeStyles, visuallyHidden];

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
  private resizeObserver?: ResizeObserver;
  private countAnnounceTimeout: ReturnType<typeof setTimeout>;

  @state() private announcedCountText = "";

  @query(".control") input: HTMLTextAreaElement;
  @query('[part~="base"]') base: HTMLDivElement;
  @query(".size-adjuster") sizeAdjuster: HTMLTextAreaElement;

  @property() title = ""; // make reactive to pass through

  /** The name of the textarea, submitted as a name/value pair with form data. */
  @property({ reflect: true }) name: string | null = null;

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
  @property({ attribute: "value", reflect: true }) defaultValue: string =
    this.getAttribute("value") ?? "";

  /** The textarea's size. */
  @property({ reflect: true }) size:
    "xs" | "s" | "m" | "l" | "xl" | "small" | "medium" | "large" = "m";

  @watch("size")
  handleSizeChange() {
    warnDeprecatedSize(this.localName, this.size);
  }

  /** The textarea's visual appearance. */
  @property({ reflect: true }) appearance:
    "filled" | "outlined" | "filled-outlined" = "outlined";

  /** The textarea's label. If you need to display HTML, use the `label` slot instead. */
  @property() label = "";

  /** The textarea's hint. If you need to display HTML, use the `hint` slot instead. */
  @property({ attribute: "hint" }) hint = "";

  /** Placeholder text to show as a hint when the input is empty. */
  @property() placeholder = "";

  /** The number of rows to display by default. */
  @property({ type: Number }) rows = 4;

  /** Controls how the textarea can be resized. */
  @property({ reflect: true }) resize:
    "none" | "vertical" | "horizontal" | "both" | "auto" = "vertical";

  /** Disables the textarea. */
  @property({ type: Boolean }) disabled = false;

  /** Makes the textarea readonly. */
  @property({ type: Boolean, reflect: true }) readonly = false;

  /** Makes the textarea a required field. */
  @property({ type: Boolean, reflect: true }) required = false;

  /** The minimum length of input that will be considered valid. */
  @property({ type: Number }) minlength: number;

  /** The maximum length of input that will be considered valid. */
  @property({ type: Number }) maxlength: number;

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

  /** Enables spell checking on the textarea. */
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

  /** Shows a character count below the textarea. When `maxlength` is set, shows remaining characters instead. */
  @property({ attribute: "with-count", type: Boolean, reflect: true })
  withCount = false;

  connectedCallback() {
    super.connectedCallback();

    this.updateComplete.then(() => {
      this.setTextareaDimensions();
      this.updateResizeObserver();

      if (this.didSSR && this.input && this.value !== this.input.value) {
        const value = this.input.value;

        this.value = value;
      }
    });
  }

  disconnectedCallback() {
    super.disconnectedCallback();
    clearTimeout(this.countAnnounceTimeout);
    this.resizeObserver?.disconnect();
    this.resizeObserver = undefined;
  }

  /** Creates or destroys the resize observer based on the current resize mode. */
  private updateResizeObserver() {
    // The resize observer is only needed for manual resize modes (vertical, horizontal, both)
    // to sync the base wrapper dimensions with the textarea.
    const needsObserver = this.resize !== "none" && this.resize !== "auto";

    if (needsObserver && !this.resizeObserver && this.input) {
      this.resizeObserver = new ResizeObserver(() =>
        this.setTextareaDimensions(),
      );
      this.resizeObserver.observe(this.input);
    } else if (!needsObserver && this.resizeObserver) {
      this.resizeObserver.disconnect();
      this.resizeObserver = undefined;
    }
  }

  private handleBlur() {
    this.checkValidity();
  }

  private handleChange(event: Event) {
    this.valueHasChanged = true;
    this.value = this.input.value;
    this.setTextareaDimensions();
    this.checkValidity();
    this.relayNativeEvent(event, { bubbles: true, composed: true });
  }

  private handleInput(event: InputEvent) {
    this.valueHasChanged = true;
    this.value = this.input.value;
    this.relayNativeEvent(event, { bubbles: true, composed: true });
    this.scheduleCountAnnouncement();
  }

  private scheduleCountAnnouncement() {
    clearTimeout(this.countAnnounceTimeout);
    this.countAnnounceTimeout = setTimeout(() => {
      const currentLength = (this.value ?? "").length;
      this.announcedCountText =
        this.maxlength != null
          ? this.localize.term(
              "numCharactersRemaining",
              this.maxlength - currentLength,
            )
          : this.localize.term("numCharacters", currentLength);
    }, 1000);
  }

  private setTextareaDimensions() {
    if (this.resize === "none") {
      // just in case this is called via a property changing.
      this.base.style.width = ``;
      this.base.style.height = ``;
      return;
    }

    if (this.resize === "auto") {
      // This prevents layout shifts. We use `clientHeight` instead of `scrollHeight` to account for if the `<textarea>`
      // has a max-height set on it. In my tests, this has worked fine. Im not aware of any edge cases. [Konnor]
      // Let's switch to `field-sizing: content` once it has better support: https://caniuse.com/mdn-css_properties_field-sizing [Lea]
      this.sizeAdjuster.style.height = `${this.input.clientHeight}px`;
      this.input.style.height = "auto";
      this.input.style.height = `${this.input.scrollHeight}px`;

      this.base.style.width = ``;
      this.base.style.height = ``;
      return;
    }

    // handles vertical, horizontal, and both resizers:

    // These should always be set by a manual resize operation , so its reasonable to expect px.
    if (this.input.style.width) {
      const width = Number(this.input.style.width.split(/px/)[0]) + 2;
      this.base.style.width = `${width}px`;
    }

    if (this.input.style.height) {
      const height = Number(this.input.style.height.split(/px/)[0]) + 2;
      this.base.style.height = `${height}px`;
    }
  }

  @watch("rows", { waitUntilFirstUpdate: true })
  handleRowsChange() {
    this.setTextareaDimensions();
  }

  @watch("value", { waitUntilFirstUpdate: true })
  async handleValueChange() {
    await this.updateComplete;
    this.checkValidity();
    this.setTextareaDimensions();
  }

  updated(changedProperties: PropertyValues<this>) {
    if (changedProperties.has("resize")) {
      this.setTextareaDimensions();
      this.updateResizeObserver();
    }

    super.updated(changedProperties);

    if (changedProperties.has("value")) {
      this.customStates.set("blank", !this.value);
    }
  }

  /** Sets focus on the textarea. */
  focus(options?: FocusOptions) {
    this.input.focus(options);
  }

  /** Removes focus from the textarea. */
  blur() {
    this.input.blur();
  }

  /** Selects all the text in the textarea. */
  select() {
    this.input.select();
  }

  /** Gets or sets the textarea's scroll position. */
  scrollPosition(position?: {
    top?: number;
    left?: number;
  }): { top: number; left: number } | undefined {
    if (position) {
      if (typeof position.top === "number") this.input.scrollTop = position.top;
      if (typeof position.left === "number")
        this.input.scrollLeft = position.left;
      return undefined;
    }

    return {
      top: this.input.scrollTop,
      left: this.input.scrollTop,
    };
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
    const selectionStart = start ?? this.input.selectionStart;
    const selectionEnd = end ?? this.input.selectionEnd;

    this.input.setRangeText(
      replacement,
      selectionStart,
      selectionEnd,
      selectMode,
    );

    if (this.value !== this.input.value) {
      this.value = this.input.value;
      this.setTextareaDimensions();
    }
  }

  formResetCallback() {
    this._value = null;

    if (this.input) {
      this.input.value = this.value || "";
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

    // We use .length intentionally here instead of Intl.Segmenter so the count stays consistent with the browser's
    // native maxlength enforcement, which also counts UTF-16 code units.
    const currentLength = (this.value ?? "").length;
    const countText =
      this.maxlength != null
        ? this.localize.term(
            "numCharactersRemaining",
            this.maxlength - currentLength,
          )
        : this.localize.term("numCharacters", currentLength);

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

      <div part="base" class="textarea">
        <textarea
          part="textarea"
          id="input"
          class="control"
          title=${this.title /* An empty title prevents browser validation tooltips from appearing on hover */}
          name=${ifDefined(this.name)}
          .value=${live(this.value)}
          ?disabled=${this.disabled}
          ?readonly=${this.readonly}
          ?required=${this.required}
          placeholder=${ifDefined(this.placeholder)}
          rows=${ifDefined(this.rows)}
          minlength=${ifDefined(this.minlength)}
          maxlength=${ifDefined(this.maxlength)}
          autocapitalize=${ifDefined(this.autocapitalize)}
          autocorrect=${ifDefined(this.autocorrect)}
          ?autofocus=${this.autofocus}
          spellcheck=${ifDefined(this.spellcheck)}
          enterkeyhint=${ifDefined(this.enterkeyhint)}
          inputmode=${ifDefined(this.inputmode)}
          aria-describedby="hint"
          @change=${this.handleChange}
          @input=${this.handleInput}
          @blur=${this.handleBlur}
        ></textarea>

        <!-- This "adjuster" exists to prevent layout shifting. https://github.com/shoelace-style/shoelace/issues/2180 -->
        <div
          part="textarea-adjuster"
          class="size-adjuster"
          ?hidden=${this.resize !== "auto"}
        ></div>
      </div>

      <div
        class=${classMap({
          footer: true,
          "has-count": this.withCount,
        })}
      >
        <slot
          id="hint"
          name="hint"
          part="hint"
          aria-hidden=${hasHint ? "false" : "true"}
          class=${classMap({
            "has-slotted": hasHint,
          })}
          >${this.hint}</slot
        >

        ${
          this.withCount
            ? html`
                <div part="count" class="count" aria-hidden="true">
                  ${countText}
                </div>
                <div class="wa-visually-hidden-force" aria-live="polite">
                  ${this.announcedCountText}
                </div>
              `
            : ""
        }
      </div>
    `;
  }
}

// The change-in-update warning is required for this component because the base class (WebAwesomeFormAssociatedElement)
// calls updateValidity() in firstUpdated(), which triggers requestUpdate('validity') to sync validation state after
// the DOM is available. This second update cycle cannot be avoided since validation requires the rendered DOM for
// anchoring. See https://lit.dev/docs/tools/development/#development-build-runtime-warnings
WaTextarea.disableWarning?.("change-in-update");

declare global {
  interface HTMLElementTagNameMap {
    "wa-textarea": WaTextarea;
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
@@ -175,9 +175,16 @@
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
@@ -420,9 +427,12 @@
         <textarea
           part="textarea"
           id="input"
           class="control"
-          title=${this.title /* An empty title prevents browser validation tooltips from appearing on hover */}
+          title=${
+            this
+              .title /* An empty title prevents browser validation tooltips from appearing on hover */
+          }
           name=${ifDefined(this.name)}
           .value=${live(this.value)}
           ?disabled=${this.disabled}
           ?readonly=${this.readonly}
@@ -464,18 +474,16 @@
         >
           ${this.hint}
         </slot>
 
-        ${
-          this.withCount
-            ? html`
-                <div part="count" class="count" aria-hidden="true">${countText}</div>
-                <div class="wa-visually-hidden-force" aria-live="polite">
-                  ${this.announcedCountText}
-                </div>
-              `
-            : ""
-        }
+        ${this.withCount
+          ? html`
+              <div part="count" class="count" aria-hidden="true">${countText}</div>
+              <div class="wa-visually-hidden-force" aria-live="polite">
+                ${this.announcedCountText}
+              </div>
+            `
+          : ""}
       </div>
     `;
   }
 }

`````

### Actual (oxfmt)

`````ts
import type { PropertyValues } from "lit";
import { html } from "lit";
import { customElement, property, query, state } from "lit/decorators.js";
import { classMap } from "lit/directives/class-map.js";
import { ifDefined } from "lit/directives/if-defined.js";
import { live } from "lit/directives/live.js";
import { warnDeprecatedSize } from "../../internal/size.js";
import { HasSlotController } from "../../internal/slot.js";
import { MirrorValidator } from "../../internal/validators/mirror-validator.js";
import { watch } from "../../internal/watch.js";
import { WebAwesomeFormAssociatedElement } from "../../internal/webawesome-form-associated-element.js";
import formControlStyles from "../../styles/component/form-control.styles.js";
import sizeStyles from "../../styles/component/size.styles.js";
import visuallyHidden from "../../styles/component/visually-hidden.styles.js";
import { LocalizeController } from "../../utilities/localize.js";
import styles from "./textarea.styles.js";

/**
 * @summary Textareas collect multi-line text input from the user, with optional resizing and character counting.
 * @documentation https://webawesome.com/docs/components/textarea
 * @status stable
 * @since 2.0
 *
 * @slot label - The textarea's label. Alternatively, you can use the `label` attribute.
 * @slot hint - Text that describes how to use the input. Alternatively, you can use the `hint` attribute.
 *
 * @event blur - Emitted when the control loses focus.
 * @event change - Emitted when an alteration to the control's value is committed by the user.
 * @event focus - Emitted when the control gains focus.
 * @event input - Emitted when the control receives input.
 * @event wa-invalid - Emitted when the form control has been checked for validity and its constraints aren't satisfied.
 *
 * @csspart label - The label
 * @csspart form-control-input - The input's wrapper.
 * @csspart hint - The hint's wrapper.
 * @csspart textarea - The internal `<textarea>` control.
 * @csspart base - The wrapper around the `<textarea>` control.
 * @csspart count - The character count element, rendered when the `with-count` attribute is present.
 *
 * @cssstate blank - The textarea is empty.
 */
@customElement("wa-textarea")
export default class WaTextarea extends WebAwesomeFormAssociatedElement {
  static css = [styles, formControlStyles, sizeStyles, visuallyHidden];

  static get validators() {
    return [...super.validators, MirrorValidator()];
  }

  assumeInteractionOn = ["blur", "input"];
  private readonly hasSlotController = new HasSlotController(this, "hint", "label");
  private readonly localize = new LocalizeController(this);
  private resizeObserver?: ResizeObserver;
  private countAnnounceTimeout: ReturnType<typeof setTimeout>;

  @state() private announcedCountText = "";

  @query(".control") input: HTMLTextAreaElement;
  @query('[part~="base"]') base: HTMLDivElement;
  @query(".size-adjuster") sizeAdjuster: HTMLTextAreaElement;

  @property() title = ""; // make reactive to pass through

  /** The name of the textarea, submitted as a name/value pair with form data. */
  @property({ reflect: true }) name: string | null = null;

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
  @property({ attribute: "value", reflect: true }) defaultValue: string =
    this.getAttribute("value") ?? "";

  /** The textarea's size. */
  @property({ reflect: true }) size: "xs" | "s" | "m" | "l" | "xl" | "small" | "medium" | "large" =
    "m";

  @watch("size")
  handleSizeChange() {
    warnDeprecatedSize(this.localName, this.size);
  }

  /** The textarea's visual appearance. */
  @property({ reflect: true }) appearance: "filled" | "outlined" | "filled-outlined" = "outlined";

  /** The textarea's label. If you need to display HTML, use the `label` slot instead. */
  @property() label = "";

  /** The textarea's hint. If you need to display HTML, use the `hint` slot instead. */
  @property({ attribute: "hint" }) hint = "";

  /** Placeholder text to show as a hint when the input is empty. */
  @property() placeholder = "";

  /** The number of rows to display by default. */
  @property({ type: Number }) rows = 4;

  /** Controls how the textarea can be resized. */
  @property({ reflect: true }) resize: "none" | "vertical" | "horizontal" | "both" | "auto" =
    "vertical";

  /** Disables the textarea. */
  @property({ type: Boolean }) disabled = false;

  /** Makes the textarea readonly. */
  @property({ type: Boolean, reflect: true }) readonly = false;

  /** Makes the textarea a required field. */
  @property({ type: Boolean, reflect: true }) required = false;

  /** The minimum length of input that will be considered valid. */
  @property({ type: Number }) minlength: number;

  /** The maximum length of input that will be considered valid. */
  @property({ type: Number }) maxlength: number;

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

  /** Enables spell checking on the textarea. */
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

  /** Shows a character count below the textarea. When `maxlength` is set, shows remaining characters instead. */
  @property({ attribute: "with-count", type: Boolean, reflect: true }) withCount = false;

  connectedCallback() {
    super.connectedCallback();

    this.updateComplete.then(() => {
      this.setTextareaDimensions();
      this.updateResizeObserver();

      if (this.didSSR && this.input && this.value !== this.input.value) {
        const value = this.input.value;

        this.value = value;
      }
    });
  }

  disconnectedCallback() {
    super.disconnectedCallback();
    clearTimeout(this.countAnnounceTimeout);
    this.resizeObserver?.disconnect();
    this.resizeObserver = undefined;
  }

  /** Creates or destroys the resize observer based on the current resize mode. */
  private updateResizeObserver() {
    // The resize observer is only needed for manual resize modes (vertical, horizontal, both)
    // to sync the base wrapper dimensions with the textarea.
    const needsObserver = this.resize !== "none" && this.resize !== "auto";

    if (needsObserver && !this.resizeObserver && this.input) {
      this.resizeObserver = new ResizeObserver(() => this.setTextareaDimensions());
      this.resizeObserver.observe(this.input);
    } else if (!needsObserver && this.resizeObserver) {
      this.resizeObserver.disconnect();
      this.resizeObserver = undefined;
    }
  }

  private handleBlur() {
    this.checkValidity();
  }

  private handleChange(event: Event) {
    this.valueHasChanged = true;
    this.value = this.input.value;
    this.setTextareaDimensions();
    this.checkValidity();
    this.relayNativeEvent(event, { bubbles: true, composed: true });
  }

  private handleInput(event: InputEvent) {
    this.valueHasChanged = true;
    this.value = this.input.value;
    this.relayNativeEvent(event, { bubbles: true, composed: true });
    this.scheduleCountAnnouncement();
  }

  private scheduleCountAnnouncement() {
    clearTimeout(this.countAnnounceTimeout);
    this.countAnnounceTimeout = setTimeout(() => {
      const currentLength = (this.value ?? "").length;
      this.announcedCountText =
        this.maxlength != null
          ? this.localize.term("numCharactersRemaining", this.maxlength - currentLength)
          : this.localize.term("numCharacters", currentLength);
    }, 1000);
  }

  private setTextareaDimensions() {
    if (this.resize === "none") {
      // just in case this is called via a property changing.
      this.base.style.width = ``;
      this.base.style.height = ``;
      return;
    }

    if (this.resize === "auto") {
      // This prevents layout shifts. We use `clientHeight` instead of `scrollHeight` to account for if the `<textarea>`
      // has a max-height set on it. In my tests, this has worked fine. Im not aware of any edge cases. [Konnor]
      // Let's switch to `field-sizing: content` once it has better support: https://caniuse.com/mdn-css_properties_field-sizing [Lea]
      this.sizeAdjuster.style.height = `${this.input.clientHeight}px`;
      this.input.style.height = "auto";
      this.input.style.height = `${this.input.scrollHeight}px`;

      this.base.style.width = ``;
      this.base.style.height = ``;
      return;
    }

    // handles vertical, horizontal, and both resizers:

    // These should always be set by a manual resize operation , so its reasonable to expect px.
    if (this.input.style.width) {
      const width = Number(this.input.style.width.split(/px/)[0]) + 2;
      this.base.style.width = `${width}px`;
    }

    if (this.input.style.height) {
      const height = Number(this.input.style.height.split(/px/)[0]) + 2;
      this.base.style.height = `${height}px`;
    }
  }

  @watch("rows", { waitUntilFirstUpdate: true })
  handleRowsChange() {
    this.setTextareaDimensions();
  }

  @watch("value", { waitUntilFirstUpdate: true })
  async handleValueChange() {
    await this.updateComplete;
    this.checkValidity();
    this.setTextareaDimensions();
  }

  updated(changedProperties: PropertyValues<this>) {
    if (changedProperties.has("resize")) {
      this.setTextareaDimensions();
      this.updateResizeObserver();
    }

    super.updated(changedProperties);

    if (changedProperties.has("value")) {
      this.customStates.set("blank", !this.value);
    }
  }

  /** Sets focus on the textarea. */
  focus(options?: FocusOptions) {
    this.input.focus(options);
  }

  /** Removes focus from the textarea. */
  blur() {
    this.input.blur();
  }

  /** Selects all the text in the textarea. */
  select() {
    this.input.select();
  }

  /** Gets or sets the textarea's scroll position. */
  scrollPosition(position?: {
    top?: number;
    left?: number;
  }): { top: number; left: number } | undefined {
    if (position) {
      if (typeof position.top === "number") this.input.scrollTop = position.top;
      if (typeof position.left === "number") this.input.scrollLeft = position.left;
      return undefined;
    }

    return {
      top: this.input.scrollTop,
      left: this.input.scrollTop,
    };
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
    const selectionStart = start ?? this.input.selectionStart;
    const selectionEnd = end ?? this.input.selectionEnd;

    this.input.setRangeText(replacement, selectionStart, selectionEnd, selectMode);

    if (this.value !== this.input.value) {
      this.value = this.input.value;
      this.setTextareaDimensions();
    }
  }

  formResetCallback() {
    this._value = null;

    if (this.input) {
      this.input.value = this.value || "";
    }

    super.formResetCallback();
  }

  render() {
    const hasLabelSlot = this.hasUpdated ? this.hasSlotController.test("label") : this.withLabel;
    const hasHintSlot = this.hasUpdated ? this.hasSlotController.test("hint") : this.withHint;
    const hasLabel = this.label ? true : !!hasLabelSlot;
    const hasHint = this.hint ? true : !!hasHintSlot;

    // We use .length intentionally here instead of Intl.Segmenter so the count stays consistent with the browser's
    // native maxlength enforcement, which also counts UTF-16 code units.
    const currentLength = (this.value ?? "").length;
    const countText =
      this.maxlength != null
        ? this.localize.term("numCharactersRemaining", this.maxlength - currentLength)
        : this.localize.term("numCharacters", currentLength);

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

      <div part="base" class="textarea">
        <textarea
          part="textarea"
          id="input"
          class="control"
          title=${
            this
              .title /* An empty title prevents browser validation tooltips from appearing on hover */
          }
          name=${ifDefined(this.name)}
          .value=${live(this.value)}
          ?disabled=${this.disabled}
          ?readonly=${this.readonly}
          ?required=${this.required}
          placeholder=${ifDefined(this.placeholder)}
          rows=${ifDefined(this.rows)}
          minlength=${ifDefined(this.minlength)}
          maxlength=${ifDefined(this.maxlength)}
          autocapitalize=${ifDefined(this.autocapitalize)}
          autocorrect=${ifDefined(this.autocorrect)}
          ?autofocus=${this.autofocus}
          spellcheck=${ifDefined(this.spellcheck)}
          enterkeyhint=${ifDefined(this.enterkeyhint)}
          inputmode=${ifDefined(this.inputmode)}
          aria-describedby="hint"
          @change=${this.handleChange}
          @input=${this.handleInput}
          @blur=${this.handleBlur}
        ></textarea>

        <!-- This "adjuster" exists to prevent layout shifting. https://github.com/shoelace-style/shoelace/issues/2180 -->
        <div part="textarea-adjuster" class="size-adjuster" ?hidden=${this.resize !== "auto"}></div>
      </div>

      <div
        class=${classMap({
          footer: true,
          "has-count": this.withCount,
        })}
      >
        <slot
          id="hint"
          name="hint"
          part="hint"
          aria-hidden=${hasHint ? "false" : "true"}
          class=${classMap({
            "has-slotted": hasHint,
          })}
        >
          ${this.hint}
        </slot>

        ${this.withCount
          ? html`
              <div part="count" class="count" aria-hidden="true">${countText}</div>
              <div class="wa-visually-hidden-force" aria-live="polite">
                ${this.announcedCountText}
              </div>
            `
          : ""}
      </div>
    `;
  }
}

// The change-in-update warning is required for this component because the base class (WebAwesomeFormAssociatedElement)
// calls updateValidity() in firstUpdated(), which triggers requestUpdate('validity') to sync validation state after
// the DOM is available. This second update cycle cannot be avoided since validation requires the rendered DOM for
// anchoring. See https://lit.dev/docs/tools/development/#development-build-runtime-warnings
WaTextarea.disableWarning?.("change-in-update");

declare global {
  interface HTMLElementTagNameMap {
    "wa-textarea": WaTextarea;
  }
}

`````

### Expected (prettier)

`````ts
import type { PropertyValues } from "lit";
import { html } from "lit";
import { customElement, property, query, state } from "lit/decorators.js";
import { classMap } from "lit/directives/class-map.js";
import { ifDefined } from "lit/directives/if-defined.js";
import { live } from "lit/directives/live.js";
import { warnDeprecatedSize } from "../../internal/size.js";
import { HasSlotController } from "../../internal/slot.js";
import { MirrorValidator } from "../../internal/validators/mirror-validator.js";
import { watch } from "../../internal/watch.js";
import { WebAwesomeFormAssociatedElement } from "../../internal/webawesome-form-associated-element.js";
import formControlStyles from "../../styles/component/form-control.styles.js";
import sizeStyles from "../../styles/component/size.styles.js";
import visuallyHidden from "../../styles/component/visually-hidden.styles.js";
import { LocalizeController } from "../../utilities/localize.js";
import styles from "./textarea.styles.js";

/**
 * @summary Textareas collect multi-line text input from the user, with optional resizing and character counting.
 * @documentation https://webawesome.com/docs/components/textarea
 * @status stable
 * @since 2.0
 *
 * @slot label - The textarea's label. Alternatively, you can use the `label` attribute.
 * @slot hint - Text that describes how to use the input. Alternatively, you can use the `hint` attribute.
 *
 * @event blur - Emitted when the control loses focus.
 * @event change - Emitted when an alteration to the control's value is committed by the user.
 * @event focus - Emitted when the control gains focus.
 * @event input - Emitted when the control receives input.
 * @event wa-invalid - Emitted when the form control has been checked for validity and its constraints aren't satisfied.
 *
 * @csspart label - The label
 * @csspart form-control-input - The input's wrapper.
 * @csspart hint - The hint's wrapper.
 * @csspart textarea - The internal `<textarea>` control.
 * @csspart base - The wrapper around the `<textarea>` control.
 * @csspart count - The character count element, rendered when the `with-count` attribute is present.
 *
 * @cssstate blank - The textarea is empty.
 */
@customElement("wa-textarea")
export default class WaTextarea extends WebAwesomeFormAssociatedElement {
  static css = [styles, formControlStyles, sizeStyles, visuallyHidden];

  static get validators() {
    return [...super.validators, MirrorValidator()];
  }

  assumeInteractionOn = ["blur", "input"];
  private readonly hasSlotController = new HasSlotController(this, "hint", "label");
  private readonly localize = new LocalizeController(this);
  private resizeObserver?: ResizeObserver;
  private countAnnounceTimeout: ReturnType<typeof setTimeout>;

  @state() private announcedCountText = "";

  @query(".control") input: HTMLTextAreaElement;
  @query('[part~="base"]') base: HTMLDivElement;
  @query(".size-adjuster") sizeAdjuster: HTMLTextAreaElement;

  @property() title = ""; // make reactive to pass through

  /** The name of the textarea, submitted as a name/value pair with form data. */
  @property({ reflect: true }) name: string | null = null;

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
  @property({ attribute: "value", reflect: true }) defaultValue: string =
    this.getAttribute("value") ?? "";

  /** The textarea's size. */
  @property({ reflect: true }) size: "xs" | "s" | "m" | "l" | "xl" | "small" | "medium" | "large" =
    "m";

  @watch("size")
  handleSizeChange() {
    warnDeprecatedSize(this.localName, this.size);
  }

  /** The textarea's visual appearance. */
  @property({ reflect: true }) appearance: "filled" | "outlined" | "filled-outlined" = "outlined";

  /** The textarea's label. If you need to display HTML, use the `label` slot instead. */
  @property() label = "";

  /** The textarea's hint. If you need to display HTML, use the `hint` slot instead. */
  @property({ attribute: "hint" }) hint = "";

  /** Placeholder text to show as a hint when the input is empty. */
  @property() placeholder = "";

  /** The number of rows to display by default. */
  @property({ type: Number }) rows = 4;

  /** Controls how the textarea can be resized. */
  @property({ reflect: true }) resize: "none" | "vertical" | "horizontal" | "both" | "auto" =
    "vertical";

  /** Disables the textarea. */
  @property({ type: Boolean }) disabled = false;

  /** Makes the textarea readonly. */
  @property({ type: Boolean, reflect: true }) readonly = false;

  /** Makes the textarea a required field. */
  @property({ type: Boolean, reflect: true }) required = false;

  /** The minimum length of input that will be considered valid. */
  @property({ type: Number }) minlength: number;

  /** The maximum length of input that will be considered valid. */
  @property({ type: Number }) maxlength: number;

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

  /** Enables spell checking on the textarea. */
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

  /** Shows a character count below the textarea. When `maxlength` is set, shows remaining characters instead. */
  @property({ attribute: "with-count", type: Boolean, reflect: true }) withCount = false;

  connectedCallback() {
    super.connectedCallback();

    this.updateComplete.then(() => {
      this.setTextareaDimensions();
      this.updateResizeObserver();

      if (this.didSSR && this.input && this.value !== this.input.value) {
        const value = this.input.value;

        this.value = value;
      }
    });
  }

  disconnectedCallback() {
    super.disconnectedCallback();
    clearTimeout(this.countAnnounceTimeout);
    this.resizeObserver?.disconnect();
    this.resizeObserver = undefined;
  }

  /** Creates or destroys the resize observer based on the current resize mode. */
  private updateResizeObserver() {
    // The resize observer is only needed for manual resize modes (vertical, horizontal, both)
    // to sync the base wrapper dimensions with the textarea.
    const needsObserver = this.resize !== "none" && this.resize !== "auto";

    if (needsObserver && !this.resizeObserver && this.input) {
      this.resizeObserver = new ResizeObserver(() => this.setTextareaDimensions());
      this.resizeObserver.observe(this.input);
    } else if (!needsObserver && this.resizeObserver) {
      this.resizeObserver.disconnect();
      this.resizeObserver = undefined;
    }
  }

  private handleBlur() {
    this.checkValidity();
  }

  private handleChange(event: Event) {
    this.valueHasChanged = true;
    this.value = this.input.value;
    this.setTextareaDimensions();
    this.checkValidity();
    this.relayNativeEvent(event, { bubbles: true, composed: true });
  }

  private handleInput(event: InputEvent) {
    this.valueHasChanged = true;
    this.value = this.input.value;
    this.relayNativeEvent(event, { bubbles: true, composed: true });
    this.scheduleCountAnnouncement();
  }

  private scheduleCountAnnouncement() {
    clearTimeout(this.countAnnounceTimeout);
    this.countAnnounceTimeout = setTimeout(() => {
      const currentLength = (this.value ?? "").length;
      this.announcedCountText =
        this.maxlength != null
          ? this.localize.term("numCharactersRemaining", this.maxlength - currentLength)
          : this.localize.term("numCharacters", currentLength);
    }, 1000);
  }

  private setTextareaDimensions() {
    if (this.resize === "none") {
      // just in case this is called via a property changing.
      this.base.style.width = ``;
      this.base.style.height = ``;
      return;
    }

    if (this.resize === "auto") {
      // This prevents layout shifts. We use `clientHeight` instead of `scrollHeight` to account for if the `<textarea>`
      // has a max-height set on it. In my tests, this has worked fine. Im not aware of any edge cases. [Konnor]
      // Let's switch to `field-sizing: content` once it has better support: https://caniuse.com/mdn-css_properties_field-sizing [Lea]
      this.sizeAdjuster.style.height = `${this.input.clientHeight}px`;
      this.input.style.height = "auto";
      this.input.style.height = `${this.input.scrollHeight}px`;

      this.base.style.width = ``;
      this.base.style.height = ``;
      return;
    }

    // handles vertical, horizontal, and both resizers:

    // These should always be set by a manual resize operation , so its reasonable to expect px.
    if (this.input.style.width) {
      const width = Number(this.input.style.width.split(/px/)[0]) + 2;
      this.base.style.width = `${width}px`;
    }

    if (this.input.style.height) {
      const height = Number(this.input.style.height.split(/px/)[0]) + 2;
      this.base.style.height = `${height}px`;
    }
  }

  @watch("rows", { waitUntilFirstUpdate: true })
  handleRowsChange() {
    this.setTextareaDimensions();
  }

  @watch("value", { waitUntilFirstUpdate: true })
  async handleValueChange() {
    await this.updateComplete;
    this.checkValidity();
    this.setTextareaDimensions();
  }

  updated(changedProperties: PropertyValues<this>) {
    if (changedProperties.has("resize")) {
      this.setTextareaDimensions();
      this.updateResizeObserver();
    }

    super.updated(changedProperties);

    if (changedProperties.has("value")) {
      this.customStates.set("blank", !this.value);
    }
  }

  /** Sets focus on the textarea. */
  focus(options?: FocusOptions) {
    this.input.focus(options);
  }

  /** Removes focus from the textarea. */
  blur() {
    this.input.blur();
  }

  /** Selects all the text in the textarea. */
  select() {
    this.input.select();
  }

  /** Gets or sets the textarea's scroll position. */
  scrollPosition(position?: {
    top?: number;
    left?: number;
  }): { top: number; left: number } | undefined {
    if (position) {
      if (typeof position.top === "number") this.input.scrollTop = position.top;
      if (typeof position.left === "number") this.input.scrollLeft = position.left;
      return undefined;
    }

    return {
      top: this.input.scrollTop,
      left: this.input.scrollTop,
    };
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
    const selectionStart = start ?? this.input.selectionStart;
    const selectionEnd = end ?? this.input.selectionEnd;

    this.input.setRangeText(replacement, selectionStart, selectionEnd, selectMode);

    if (this.value !== this.input.value) {
      this.value = this.input.value;
      this.setTextareaDimensions();
    }
  }

  formResetCallback() {
    this._value = null;

    if (this.input) {
      this.input.value = this.value || "";
    }

    super.formResetCallback();
  }

  render() {
    const hasLabelSlot = this.hasUpdated ? this.hasSlotController.test("label") : this.withLabel;
    const hasHintSlot = this.hasUpdated ? this.hasSlotController.test("hint") : this.withHint;
    const hasLabel = this.label ? true : !!hasLabelSlot;
    const hasHint = this.hint ? true : !!hasHintSlot;

    // We use .length intentionally here instead of Intl.Segmenter so the count stays consistent with the browser's
    // native maxlength enforcement, which also counts UTF-16 code units.
    const currentLength = (this.value ?? "").length;
    const countText =
      this.maxlength != null
        ? this.localize.term("numCharactersRemaining", this.maxlength - currentLength)
        : this.localize.term("numCharacters", currentLength);

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

      <div part="base" class="textarea">
        <textarea
          part="textarea"
          id="input"
          class="control"
          title=${this.title /* An empty title prevents browser validation tooltips from appearing on hover */}
          name=${ifDefined(this.name)}
          .value=${live(this.value)}
          ?disabled=${this.disabled}
          ?readonly=${this.readonly}
          ?required=${this.required}
          placeholder=${ifDefined(this.placeholder)}
          rows=${ifDefined(this.rows)}
          minlength=${ifDefined(this.minlength)}
          maxlength=${ifDefined(this.maxlength)}
          autocapitalize=${ifDefined(this.autocapitalize)}
          autocorrect=${ifDefined(this.autocorrect)}
          ?autofocus=${this.autofocus}
          spellcheck=${ifDefined(this.spellcheck)}
          enterkeyhint=${ifDefined(this.enterkeyhint)}
          inputmode=${ifDefined(this.inputmode)}
          aria-describedby="hint"
          @change=${this.handleChange}
          @input=${this.handleInput}
          @blur=${this.handleBlur}
        ></textarea>

        <!-- This "adjuster" exists to prevent layout shifting. https://github.com/shoelace-style/shoelace/issues/2180 -->
        <div part="textarea-adjuster" class="size-adjuster" ?hidden=${this.resize !== "auto"}></div>
      </div>

      <div
        class=${classMap({
          footer: true,
          "has-count": this.withCount,
        })}
      >
        <slot
          id="hint"
          name="hint"
          part="hint"
          aria-hidden=${hasHint ? "false" : "true"}
          class=${classMap({
            "has-slotted": hasHint,
          })}
        >
          ${this.hint}
        </slot>

        ${
          this.withCount
            ? html`
                <div part="count" class="count" aria-hidden="true">${countText}</div>
                <div class="wa-visually-hidden-force" aria-live="polite">
                  ${this.announcedCountText}
                </div>
              `
            : ""
        }
      </div>
    `;
  }
}

// The change-in-update warning is required for this component because the base class (WebAwesomeFormAssociatedElement)
// calls updateValidity() in firstUpdated(), which triggers requestUpdate('validity') to sync validation state after
// the DOM is available. This second update cycle cannot be avoided since validation requires the rendered DOM for
// anchoring. See https://lit.dev/docs/tools/development/#development-build-runtime-warnings
WaTextarea.disableWarning?.("change-in-update");

declare global {
  interface HTMLElementTagNameMap {
    "wa-textarea": WaTextarea;
  }
}

`````
