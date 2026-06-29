# externals/webawesome/color-picker/color-picker.ts

## Option 1

`````json
{"printWidth":80}
`````

### Diff

`````diff
===================================================================
--- prettier
+++ oxfmt
@@ -227,9 +227,16 @@
   @property() format: "hex" | "rgb" | "hsl" | "hsv" = "hex";
 
   /** Determines the size of the color picker's trigger */
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
@@ -947,9 +954,16 @@
 
   /** Returns the current value as a string in the specified format. */
   getFormattedValue(
     format:
-      "hex" | "hexa" | "rgb" | "rgba" | "hsl" | "hsla" | "hsv" | "hsva" = "hex",
+      | "hex"
+      | "hexa"
+      | "rgb"
+      | "rgba"
+      | "hsl"
+      | "hsla"
+      | "hsv"
+      | "hsva" = "hex",
   ) {
     const currentColor = this.parseColor(
       `hsva(${this.hue}, ${this.saturation}%, ${this.brightness}%, ${this.alpha / 100})`,
     );
@@ -1218,9 +1232,11 @@
       >
         <div
           part="grid"
           class="grid"
-          style=${styleMap({ backgroundColor: this.getHexString(this.hue, 100, 100) })}
+          style=${styleMap({
+            backgroundColor: this.getHexString(this.hue, 100, 100),
+          })}
           @pointerdown=${this.handleGridDrag}
           @touchmove=${this.handleTouchMove}
         >
           <span
@@ -1271,52 +1287,50 @@
                 @keydown=${this.handleHueKeyDown}
               ></span>
             </div>
 
-            ${
-              this.opacity
-                ? html`
+            ${this.opacity
+              ? html`
+                  <div
+                    part="slider opacity-slider"
+                    class="alpha slider transparent-bg"
+                    @pointerdown="${this.handleAlphaDrag}"
+                    @touchmove=${this.handleTouchMove}
+                  >
                     <div
-                      part="slider opacity-slider"
-                      class="alpha slider transparent-bg"
-                      @pointerdown="${this.handleAlphaDrag}"
-                      @touchmove=${this.handleTouchMove}
-                    >
-                      <div
-                        class="alpha-gradient"
-                        style=${styleMap({
+                      class="alpha-gradient"
+                      style=${styleMap({
                         backgroundImage: `linear-gradient(
                           to right,
                           ${this.getHexString(this.hue, this.saturation, this.brightness, 0)} 0%,
                           ${this.getHexString(this.hue, this.saturation, this.brightness, 100)} 100%
                         )`,
                       })}
-                      ></div>
-                      <span
-                        part="slider-handle opacity-slider-handle"
-                        class="slider-handle"
-                        style=${styleMap({
+                    ></div>
+                    <span
+                      part="slider-handle opacity-slider-handle"
+                      class="slider-handle"
+                      style=${styleMap({
                         left: `${this.alpha}%`,
                         backgroundColor: this.getHexString(
                           this.hue,
                           this.saturation,
                           this.brightness,
                           this.alpha,
                         ),
                       })}
-                        role="slider"
-                        aria-label="alpha"
-                        aria-orientation="horizontal"
-                        aria-valuemin="0"
-                        aria-valuemax="100"
-                        aria-valuenow=${Math.round(this.alpha)}
-                        tabindex=${ifDefined(this.disabled ? undefined : "0")}
-                        @keydown=${this.handleAlphaKeyDown}
-                      ></span>
-                    </div>
-                  `
-                : ""
-            }
+                      role="slider"
+                      aria-label="alpha"
+                      aria-orientation="horizontal"
+                      aria-valuemin="0"
+                      aria-valuemax="100"
+                      aria-valuenow=${Math.round(this.alpha)}
+                      tabindex=${ifDefined(this.disabled ? undefined : "0")}
+                      @keydown=${this.handleAlphaKeyDown}
+                    ></span>
+                  </div>
+                `
+              : ""}
           </div>
 
           <button
             type="button"
@@ -1356,68 +1370,63 @@
             @focus=${this.stopNestedEventPropagation}
           ></wa-input>
 
           <wa-button-group>
-            ${
-              !this.withoutFormatToggle
-                ? html`
-                    <wa-button
-                      part="format-button"
-                      size="s"
-                      appearance="outlined"
-                      aria-label=${this.localize.term("toggleColorFormat")}
-                      exportparts="
+            ${!this.withoutFormatToggle
+              ? html`
+                  <wa-button
+                    part="format-button"
+                    size="s"
+                    appearance="outlined"
+                    aria-label=${this.localize.term("toggleColorFormat")}
+                    exportparts="
                       base:format-button__base,
                       start:format-button__start,
                       label:format-button__label,
                       end:format-button__end,
                       caret:format-button__caret
                     "
-                      @click=${this.handleFormatToggle}
-                      @blur=${this.stopNestedEventPropagation}
-                      @focus=${this.stopNestedEventPropagation}
-                    >
-                      ${this.setLetterCase(this.format)}
-                    </wa-button>
-                  `
-                : ""
-            }
-            ${
-              this.hasEyeDropper
-                ? html`
-                    <wa-button
-                      part="eyedropper-button"
-                      size="s"
-                      appearance="outlined"
-                      exportparts="
+                    @click=${this.handleFormatToggle}
+                    @blur=${this.stopNestedEventPropagation}
+                    @focus=${this.stopNestedEventPropagation}
+                  >
+                    ${this.setLetterCase(this.format)}
+                  </wa-button>
+                `
+              : ""}
+            ${this.hasEyeDropper
+              ? html`
+                  <wa-button
+                    part="eyedropper-button"
+                    size="s"
+                    appearance="outlined"
+                    exportparts="
                       base:eyedropper-button__base,
                       start:eyedropper-button__start,
                       label:eyedropper-button__label,
                       end:eyedropper-button__end,
                       caret:eyedropper-button__caret
                     "
-                      @click=${this.handleEyeDropper}
-                      @blur=${this.stopNestedEventPropagation}
-                      @focus=${this.stopNestedEventPropagation}
-                    >
-                      <wa-icon
-                        library="system"
-                        name="eyedropper"
-                        variant="solid"
-                        label=${this.localize.term("selectAColorFromTheScreen")}
-                      ></wa-icon>
-                    </wa-button>
-                  `
-                : ""
-            }
+                    @click=${this.handleEyeDropper}
+                    @blur=${this.stopNestedEventPropagation}
+                    @focus=${this.stopNestedEventPropagation}
+                  >
+                    <wa-icon
+                      library="system"
+                      name="eyedropper"
+                      variant="solid"
+                      label=${this.localize.term("selectAColorFromTheScreen")}
+                    ></wa-icon>
+                  </wa-button>
+                `
+              : ""}
           </wa-button-group>
         </div>
 
-        ${
-          normalizedSwatches.length > 0
-            ? html`
-                <div part="swatches" class="swatches">
-                  ${normalizedSwatches.map((swatch) => {
+        ${normalizedSwatches.length > 0
+          ? html`
+              <div part="swatches" class="swatches">
+                ${normalizedSwatches.map((swatch) => {
                   const parsedColor = this.parseColor(swatch.color);
 
                   // If we can't parse it, skip it
                   if (!parsedColor) {
@@ -1443,12 +1452,11 @@
                       ></div>
                     </div>
                   `;
                 })}
-                </div>
-              `
-            : ""
-        }
+              </div>
+            `
+          : ""}
       </div>
     `;
 
     // Render with popup

`````

### Actual (oxfmt)

`````ts
import { TinyColor } from "@ctrl/tinycolor";
import type { PropertyValues } from "lit";
import { html, isServer } from "lit";
import {
  customElement,
  eventOptions,
  property,
  query,
  state,
} from "lit/decorators.js";
import { classMap } from "lit/directives/class-map.js";
import { ifDefined } from "lit/directives/if-defined.js";
import { styleMap } from "lit/directives/style-map.js";
import { WaInvalidEvent } from "../../events/invalid.js";
import { animateWithClass } from "../../internal/animate.js";
import {
  isTopDismissible,
  registerDismissible,
  unregisterDismissible,
} from "../../internal/dismissible-stack.js";
import { drag } from "../../internal/drag.js";
import { waitForEvent } from "../../internal/event.js";
import { clamp } from "../../internal/math.js";
import { warnDeprecatedSize } from "../../internal/size.js";
import { HasSlotController } from "../../internal/slot.js";
import { RequiredValidator } from "../../internal/validators/required-validator.js";
import { watch } from "../../internal/watch.js";
import { WebAwesomeFormAssociatedElement } from "../../internal/webawesome-form-associated-element.js";
import formControlStyles from "../../styles/component/form-control.styles.js";
import sizeStyles from "../../styles/component/size.styles.js";
import visuallyHidden from "../../styles/component/visually-hidden.styles.js";
import { LocalizeController } from "../../utilities/localize.js";
import "../button-group/button-group.js";
import "../button/button.js";
import "../icon/icon.js";
import "../input/input.js";
import type WaInput from "../input/input.js";
import "../popup/popup.js";
import type WaPopup from "../popup/popup.js";
import styles from "./color-picker.styles.js";

export interface WaColorPickerSwatch {
  color: string;
  label: string;
}

interface EyeDropperConstructor {
  new (): EyeDropperInterface;
}

interface EyeDropperInterface {
  open: () => Promise<{ sRGBHex: string }>;
}

declare const EyeDropper: EyeDropperConstructor;

/**
 * @summary Color pickers let users choose a color from a visual palette or by entering a value. They support HEX, RGB,
 *  HSL, and HSV formats with optional alpha channel and swatch presets.
 * @documentation https://webawesome.com/docs/components/color-picker
 * @status stable
 * @since 2.0
 *
 * @dependency wa-button
 * @dependency wa-button-group
 * @dependency wa-input
 * @dependency wa-popup
 * @dependency wa-visually-hidden
 *
 * @slot label - The color picker's form label. Alternatively, you can use the `label` attribute.
 * @slot hint - The color picker's form hint. Alternatively, you can use the `hint` attribute.
 *
 * @event blur - Emitted when the color picker loses focus.
 * @event change - Emitted when the color picker's value changes.
 * @event focus - Emitted when the color picker receives focus.
 * @event input - Emitted when the color picker receives input.
 * @event wa-invalid - Emitted when the form control has been checked for validity and its constraints aren't satisfied.
 *
 * @csspart base - The component's base wrapper.
 * @csspart trigger - The color picker's dropdown trigger.
 * @csspart swatches - The container that holds the swatches.
 * @csspart swatch - Each individual swatch.
 * @csspart grid - The color grid.
 * @csspart grid-handle - The color grid's handle.
 * @csspart slider - Hue and opacity sliders.
 * @csspart slider-handle - Hue and opacity slider handles.
 * @csspart hue-slider - The hue slider.
 * @csspart hue-slider-handle - The hue slider's handle.
 * @csspart opacity-slider - The opacity slider.
 * @csspart opacity-slider-handle - The opacity slider's handle.
 * @csspart preview - The preview color.
 * @csspart input - The text input.
 * @csspart eyedropper-button - The eye dropper button.
 * @csspart eyedropper-button__base - The eye dropper button's exported `button` part.
 * @csspart eyedropper-button__start - The eye dropper button's exported `start` part.
 * @csspart eyedropper-button__label - The eye dropper button's exported `label` part.
 * @csspart eyedropper-button__end - The eye dropper button's exported `end` part.
 * @csspart eyedropper-button__caret - The eye dropper button's exported `caret` part.
 * @csspart format-button - The format button.
 * @csspart format-button__base - The format button's exported `button` part.
 * @csspart format-button__start - The format button's exported `start` part.
 * @csspart format-button__label - The format button's exported `label` part.
 * @csspart format-button__end - The format button's exported `end` part.
 * @csspart format-button__caret - The format button's exported `caret` part.
 *
 * @cssproperty --grid-width - The width of the color grid.
 * @cssproperty --grid-height - The height of the color grid.
 * @cssproperty --grid-handle-size - The size of the color grid's handle.
 * @cssproperty --slider-height - The height of the hue and alpha sliders.
 * @cssproperty --slider-handle-size - The diameter of the slider's handle.
 */
@customElement("wa-color-picker")
export default class WaColorPicker extends WebAwesomeFormAssociatedElement {
  static css = [visuallyHidden, sizeStyles, formControlStyles, styles];

  static shadowRootOptions = {
    ...WebAwesomeFormAssociatedElement.shadowRootOptions,
    delegatesFocus: true,
  };

  static get validators() {
    const validators = isServer ? [] : [RequiredValidator()];
    return [...super.validators, ...validators];
  }

  private readonly hasSlotController = new HasSlotController(
    this,
    "hint",
    "label",
  );

  private isSafeValue = false;
  private readonly localize = new LocalizeController(this);

  @query('[part~="base"]') base: HTMLElement;
  @query('[part~="input"]') input: WaInput;
  @query('[part~="form-control-label"]') triggerLabel: HTMLElement;
  @query('[part~="form-control-input"]') triggerButton: HTMLButtonElement;

  // @TODO: This is a hacky way to show the "Please fill out this field", do we want the old behavior where it opens the dropdown?
  //   or is the new behavior okay?
  get validationTarget() {
    // This puts the popup on the element only if the color picker is expanded.
    if (this.popup?.active) {
      return this.input;
    }

    // This puts popup on the color picker itself without needing to expand it to show the input.
    // This is necessary because form submissions expect the "anchor" to be currently shown.
    return this.trigger;
  }

  @query(".color-popup") popup: WaPopup;
  @query('[part~="preview"]') previewButton: HTMLButtonElement;
  @query('[part~="trigger"]') trigger: HTMLButtonElement;

  @state() private hasFocus = false;
  @state() private isDraggingGridHandle = false;
  @state() private isEmpty = true;
  @state() private inputValue = "";
  @state() private hue = 0;
  @state() private saturation = 100;
  @state() private brightness = 100;
  @state() private alpha = 100;

  private _value: string | null = null;

  /** The current value of the input, submitted as a name/value pair with form data. */
  get value() {
    if (this.valueHasChanged) {
      return this._value;
    }

    return this._value ?? this.defaultValue;
  }

  /**
   * The current value of the color picker. The value's format will vary based the `format` attribute. To get the value
   * in a specific format, use the `getFormattedValue()` method. The value is submitted as a name/value pair with form
   * data.
   */

  @state() set value(val: string | null) {
    if (this._value === val) {
      return;
    }

    this.valueHasChanged = true;
    this._value = val;
  }

  /** The default value of the form control. Primarily used for resetting the form control. */
  @property({ attribute: "value", reflect: true }) defaultValue: string | null =
    this.getAttribute("value") || null;

  /**
   * Only required for SSR. Set to `true` if you're slotting in a `label` element so the server-rendered markup
   * includes the label before the component hydrates on the client.
   */
  @property({ attribute: "with-label", reflect: true, type: Boolean })
  withLabel = false;

  /**
   * Only required for SSR. Set to `true` if you're slotting in a `hint` element so the server-rendered markup
   * includes the hint before the component hydrates on the client.
   */
  @property({ attribute: "with-hint", reflect: true, type: Boolean }) withHint =
    false;

  @state() private hasEyeDropper: boolean = false;

  /**
   * The color picker's label. This will not be displayed, but it will be announced by assistive devices. If you need to
   * display HTML, you can use the `label` slot` instead.
   */
  @property() label = "";

  /**
   * The color picker's hint. If you need to display HTML, use the `hint` slot instead.
   */
  @property({ attribute: "hint" }) hint = "";

  /**
   * The format to use. If opacity is enabled, these will translate to HEXA, RGBA, HSLA, and HSVA respectively. The color
   * picker will accept user input in any format (including CSS color names) and convert it to the desired format.
   */
  @property() format: "hex" | "rgb" | "hsl" | "hsv" = "hex";

  /** Determines the size of the color picker's trigger */
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

  /**
   * The preferred placement of the color picker's popup. Note that the actual placement will vary as configured to
   * keep the panel inside of the viewport.
   */
  @property({ reflect: true }) placement:
    | "top"
    | "top-start"
    | "top-end"
    | "bottom"
    | "bottom-start"
    | "bottom-end"
    | "right"
    | "right-start"
    | "right-end"
    | "left"
    | "left-start"
    | "left-end" = "bottom-start";

  /** Removes the button that lets users toggle between format.   */
  @property({ attribute: "without-format-toggle", type: Boolean })
  withoutFormatToggle = false;

  /** The name of the form control, submitted as a name/value pair with form data. */
  @property({ reflect: true }) name: string | null = null;

  /** Disables the color picker. */
  @property({ type: Boolean }) disabled = false;

  /**
   * Indicates whether or not the popup is open. You can toggle this attribute to show and hide the popup, or you
   * can use the `show()` and `hide()` methods and this attribute will reflect the popup's open state.
   */
  @property({ type: Boolean, reflect: true }) open = false;

  /** Shows the opacity slider. Enabling this will cause the formatted value to be HEXA, RGBA, or HSLA. */
  @property({ type: Boolean }) opacity = false;

  /** By default, values are lowercase. With this attribute, values will be uppercase instead. */
  @property({ type: Boolean }) uppercase = false;

  /**
   * One or more predefined color swatches to display as presets in the color picker. Can include any format the color
   * picker can parse, including HEX(A), RGB(A), HSL(A), HSV(A), and CSS color names. Each color must be separated by a
   * semicolon (`;`). Alternatively, you can pass an array of color values or an array of `{ color, label }` objects to
   * this property using JavaScript. When using objects with labels, the label will be used for the swatch's accessible
   * name instead of the raw color value.
   */
  @property() swatches: string | string[] | WaColorPickerSwatch[] = "";

  /** Makes the color picker a required field. */
  @property({ type: Boolean, reflect: true }) required = false;

  constructor() {
    super();

    if (!isServer) {
      this.addEventListener("focusin", this.handleFocusIn);
      this.addEventListener("focusout", this.handleFocusOut);
    }
  }

  private handleCopy() {
    this.input.select();
    document.execCommand("copy");
    this.previewButton.focus();

    // Show copied animation
    this.previewButton.classList.add("preview-color-copied");
    this.previewButton.addEventListener("animationend", () => {
      this.previewButton.classList.remove("preview-color-copied");
    });
  }

  private handleFocusIn = () => {
    this.hasFocus = true;
  };

  private handleFocusOut = () => {
    this.hasFocus = false;
  };

  private handleFormatToggle() {
    const formats = ["hex", "rgb", "hsl", "hsv"];
    const nextIndex = (formats.indexOf(this.format) + 1) % formats.length;
    this.format = formats[nextIndex] as "hex" | "rgb" | "hsl" | "hsv";
    this.setColor(this.value || "");

    this.updateComplete.then(() => {
      this.dispatchEvent(
        new Event("change", { bubbles: true, composed: true }),
      );
      this.dispatchEvent(
        new InputEvent("input", { bubbles: true, composed: true }),
      );
    });
  }

  private handleAlphaDrag(event: PointerEvent) {
    const container =
      this.shadowRoot!.querySelector<HTMLElement>(".slider.alpha")!;
    const handle = container.querySelector<HTMLElement>(".slider-handle")!;
    const { width } = container.getBoundingClientRect();
    let initialValue = this.value;
    let currentValue = this.value;

    handle.focus();
    event.preventDefault();

    drag(container, {
      onMove: (x) => {
        this.alpha = clamp((x / width) * 100, 0, 100);
        this.syncValues();

        if (this.value !== currentValue) {
          currentValue = this.value;
          this.updateComplete.then(() => {
            this.dispatchEvent(
              new InputEvent("input", { bubbles: true, composed: true }),
            );
          });
        }
      },
      onStop: () => {
        if (this.value !== initialValue) {
          initialValue = this.value;

          this.updateComplete.then(() => {
            this.dispatchEvent(
              new Event("change", { bubbles: true, composed: true }),
            );
          });
        }
      },
      initialEvent: event,
    });
  }

  private handleHueDrag(event: PointerEvent) {
    const container =
      this.shadowRoot!.querySelector<HTMLElement>(".slider.hue")!;
    const handle = container.querySelector<HTMLElement>(".slider-handle")!;
    const { width } = container.getBoundingClientRect();
    let initialValue = this.value;
    let currentValue = this.value;

    handle.focus();
    event.preventDefault();

    drag(container, {
      onMove: (x) => {
        this.hue = clamp((x / width) * 360, 0, 360);
        this.syncValues();

        if (this.value !== currentValue) {
          currentValue = this.value;
          this.updateComplete.then(() => {
            this.dispatchEvent(new InputEvent("input"));
          });
        }
      },
      onStop: () => {
        if (this.value !== initialValue) {
          initialValue = this.value;
          this.updateComplete.then(() => {
            this.dispatchEvent(
              new Event("change", { bubbles: true, composed: true }),
            );
          });
        }
      },
      initialEvent: event,
    });
  }

  private handleGridDrag(event: PointerEvent) {
    const grid = this.shadowRoot!.querySelector<HTMLElement>(".grid")!;
    const handle = grid.querySelector<HTMLElement>(".grid-handle")!;
    const { width, height } = grid.getBoundingClientRect();
    let initialValue = this.value;
    let currentValue = this.value;

    handle.focus();
    event.preventDefault();

    this.isDraggingGridHandle = true;

    drag(grid, {
      onMove: (x, y) => {
        this.saturation = clamp((x / width) * 100, 0, 100);
        this.brightness = clamp(100 - (y / height) * 100, 0, 100);
        this.syncValues();

        if (this.value !== currentValue) {
          currentValue = this.value;
          this.updateComplete.then(() => {
            this.dispatchEvent(
              new InputEvent("input", { bubbles: true, composed: true }),
            );
          });
        }
      },
      onStop: () => {
        this.isDraggingGridHandle = false;
        if (this.value !== initialValue) {
          initialValue = this.value;
          this.updateComplete.then(() => {
            this.dispatchEvent(
              new Event("change", { bubbles: true, composed: true }),
            );
          });
        }
      },
      initialEvent: event,
    });
  }

  private handleAlphaKeyDown(event: KeyboardEvent) {
    const increment = event.shiftKey ? 10 : 1;
    const oldValue = this.value;

    if (event.key === "ArrowLeft") {
      event.preventDefault();
      this.alpha = clamp(this.alpha - increment, 0, 100);
      this.syncValues();
    }

    if (event.key === "ArrowRight") {
      event.preventDefault();
      this.alpha = clamp(this.alpha + increment, 0, 100);
      this.syncValues();
    }

    if (event.key === "Home") {
      event.preventDefault();
      this.alpha = 0;
      this.syncValues();
    }

    if (event.key === "End") {
      event.preventDefault();
      this.alpha = 100;
      this.syncValues();
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
  }

  private handleHueKeyDown(event: KeyboardEvent) {
    const increment = event.shiftKey ? 10 : 1;
    const oldValue = this.value;

    if (event.key === "ArrowLeft") {
      event.preventDefault();
      this.hue = clamp(this.hue - increment, 0, 360);
      this.syncValues();
    }

    if (event.key === "ArrowRight") {
      event.preventDefault();
      this.hue = clamp(this.hue + increment, 0, 360);
      this.syncValues();
    }

    if (event.key === "Home") {
      event.preventDefault();
      this.hue = 0;
      this.syncValues();
    }

    if (event.key === "End") {
      event.preventDefault();
      this.hue = 360;
      this.syncValues();
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
  }

  private handleGridKeyDown(event: KeyboardEvent) {
    const increment = event.shiftKey ? 10 : 1;
    const oldValue = this.value;

    if (event.key === "ArrowLeft") {
      event.preventDefault();
      this.saturation = clamp(this.saturation - increment, 0, 100);
      this.syncValues();
    }

    if (event.key === "ArrowRight") {
      event.preventDefault();
      this.saturation = clamp(this.saturation + increment, 0, 100);
      this.syncValues();
    }

    if (event.key === "ArrowUp") {
      event.preventDefault();
      this.brightness = clamp(this.brightness + increment, 0, 100);
      this.syncValues();
    }

    if (event.key === "ArrowDown") {
      event.preventDefault();
      this.brightness = clamp(this.brightness - increment, 0, 100);
      this.syncValues();
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
  }

  private handleInputChange(event: Event) {
    const target = event.target as HTMLInputElement;
    const oldValue = this.value;

    // Prevent the `<wa-input>` element's `change` event from bubbling up
    event.stopPropagation();

    if (this.input.value) {
      this.setColor(target.value);
      target.value = this.value || "";
    } else {
      this.value = "";
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
  }

  private handleInputInput(event: InputEvent) {
    this.updateValidity();

    // Prevent the `<wa-input>` element's `input` event from bubbling up
    event.stopPropagation();
  }

  private handleInputKeyDown(event: KeyboardEvent) {
    if (event.key === "Enter") {
      const oldValue = this.value;

      if (this.input.value) {
        this.setColor(this.input.value);
        this.input.value = this.value;

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

        setTimeout(() => this.input.select());
      } else {
        this.hue = 0;
      }
    }
  }

  @eventOptions({ passive: false })
  private handleTouchMove(event: TouchEvent) {
    event.preventDefault();
  }

  private parseColor(colorString: string) {
    if (!colorString || colorString.trim() === "") {
      return null;
    }

    const color = new TinyColor(colorString);
    if (!color.isValid) {
      return null;
    }

    const hslColor = color.toHsl();
    const rgb = color.toRgb();
    const hsvColor = color.toHsv();

    // Checks for null RGB values
    if (!rgb || rgb.r == null || rgb.g == null || rgb.b == null) {
      return null;
    }

    // Adjust saturation and lightness from 0-1 to 0-100
    const hsl = {
      h: hslColor.h || 0,
      s: (hslColor.s || 0) * 100,
      l: (hslColor.l || 0) * 100,
      a: hslColor.a || 0,
    };

    const hex = color.toHexString();
    const hexa = color.toHex8String();

    // Adjust saturation and value from 0-1 to 0-100
    const hsv = {
      h: hsvColor.h || 0,
      s: (hsvColor.s || 0) * 100,
      v: (hsvColor.v || 0) * 100,
      a: hsvColor.a || 0,
    };

    return {
      hsl: {
        h: hsl.h,
        s: hsl.s,
        l: hsl.l,
        string: this.setLetterCase(
          `hsl(${Math.round(hsl.h)}, ${Math.round(hsl.s)}%, ${Math.round(hsl.l)}%)`,
        ),
      },
      hsla: {
        h: hsl.h,
        s: hsl.s,
        l: hsl.l,
        a: hsl.a,
        string: this.setLetterCase(
          `hsla(${Math.round(hsl.h)}, ${Math.round(hsl.s)}%, ${Math.round(hsl.l)}%, ${hsl.a.toFixed(2).toString()})`,
        ),
      },
      hsv: {
        h: hsv.h,
        s: hsv.s,
        v: hsv.v,
        string: this.setLetterCase(
          `hsv(${Math.round(hsv.h)}, ${Math.round(hsv.s)}%, ${Math.round(hsv.v)}%)`,
        ),
      },
      hsva: {
        h: hsv.h,
        s: hsv.s,
        v: hsv.v,
        a: hsv.a,
        string: this.setLetterCase(
          `hsva(${Math.round(hsv.h)}, ${Math.round(hsv.s)}%, ${Math.round(hsv.v)}%, ${hsv.a.toFixed(2).toString()})`,
        ),
      },
      rgb: {
        r: rgb.r,
        g: rgb.g,
        b: rgb.b,
        string: this.setLetterCase(
          `rgb(${Math.round(rgb.r)}, ${Math.round(rgb.g)}, ${Math.round(rgb.b)})`,
        ),
      },
      rgba: {
        r: rgb.r,
        g: rgb.g,
        b: rgb.b,
        a: rgb.a || 0,
        string: this.setLetterCase(
          `rgba(${Math.round(rgb.r)}, ${Math.round(rgb.g)}, ${Math.round(rgb.b)}, ${(rgb.a || 0).toFixed(2).toString()})`,
        ),
      },
      hex: this.setLetterCase(hex),
      hexa: this.setLetterCase(hexa),
    };
  }

  private setColor(colorString: string) {
    const newColor = this.parseColor(colorString);

    if (newColor === null) {
      return false;
    }

    this.hue = newColor.hsva.h;
    this.saturation = newColor.hsva.s;
    this.brightness = newColor.hsva.v;
    this.alpha = this.opacity ? newColor.hsva.a * 100 : 100;

    this.syncValues();

    return true;
  }

  private setLetterCase(string: string) {
    if (typeof string !== "string") {
      return "";
    }
    return this.uppercase ? string.toUpperCase() : string.toLowerCase();
  }

  private async syncValues() {
    const currentColor = this.parseColor(
      `hsva(${this.hue}, ${this.saturation}%, ${this.brightness}%, ${this.alpha / 100})`,
    );

    if (currentColor === null) {
      return;
    }

    // Update the value
    if (this.format === "hsl") {
      this.inputValue = this.opacity
        ? currentColor.hsla.string
        : currentColor.hsl.string;
    } else if (this.format === "rgb") {
      this.inputValue = this.opacity
        ? currentColor.rgba.string
        : currentColor.rgb.string;
    } else if (this.format === "hsv") {
      this.inputValue = this.opacity
        ? currentColor.hsva.string
        : currentColor.hsv.string;
    } else {
      this.inputValue = this.opacity ? currentColor.hexa : currentColor.hex;
    }

    // Setting this.value will trigger the watcher which parses the new value. We want to bypass that behavior because
    // we've already parsed the color here and conversion/rounding can lead to values changing slightly. When this
    // happens, dragging the grid handle becomes jumpy. After the next update, the usual behavior is restored.
    this.isSafeValue = true;
    this.value = this.inputValue;

    await this.updateComplete;
    this.isSafeValue = false;
  }

  private handleAfterHide() {
    this.previewButton.classList.remove("preview-color-copied");
    // Update validity so we get a new anchor.
    this.updateValidity();
  }

  private handleAfterShow() {
    // Update validity so we get a new anchor.
    this.updateValidity();
  }

  private handleEyeDropper() {
    if (!this.hasEyeDropper) {
      return;
    }

    const eyeDropper = new EyeDropper();

    eyeDropper
      .open()
      .then((colorSelectionResult) => {
        const oldValue = this.value;

        this.setColor(colorSelectionResult.sRGBHex);

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
      })
      .catch(() => {
        // The user canceled, do nothing
      });
  }

  private selectSwatch(color: string) {
    const oldValue = this.value;

    if (!this.disabled) {
      this.setColor(color);

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
    }
  }

  /** Generates a hex string from HSV values. Hue must be 0-360. All other arguments must be 0-100. */
  getHexString(
    hue: number,
    saturation: number,
    brightness: number,
    alpha = 100,
  ) {
    const color = new TinyColor(
      `hsva(${hue}, ${saturation}%, ${brightness}%, ${alpha / 100})`,
    );
    if (!color.isValid) {
      return "";
    }

    return color.toHex8String();
  }

  // Prevents nested components from leaking events
  private stopNestedEventPropagation(event: CustomEvent) {
    event.stopImmediatePropagation();
  }

  @watch("format", { waitUntilFirstUpdate: true })
  handleFormatChange() {
    this.syncValues();
  }

  @watch("opacity")
  handleOpacityChange() {
    this.alpha = 100;
  }

  protected willUpdate(changedProperties: PropertyValues<this>): void {
    super.willUpdate(changedProperties);

    // Its kind of bizarre, but this is required to get SSR to play nicely.
    if (changedProperties.has("value")) {
      this.handleValueChange(
        changedProperties.get("value") || "",
        this.value || "",
      );
    }
  }

  @watch("value")
  handleValueChange(oldValue: string | undefined, newValue: string) {
    this.isEmpty = !newValue;

    if (!newValue) {
      this.hue = 0;
      this.saturation = 0;
      this.brightness = 100;
      this.alpha = 100;
    }

    if (!this.isSafeValue) {
      const newColor = this.parseColor(newValue);

      if (newColor !== null) {
        this.inputValue = this.value || "";
        this.hue = newColor.hsva.h;
        this.saturation = newColor.hsva.s;
        this.brightness = newColor.hsva.v;
        this.alpha = newColor.hsva.a * 100;
        this.syncValues();
      } else {
        this.inputValue = oldValue ?? "";
      }
    }

    this.requestUpdate();
  }

  /** Sets focus on the color picker. */
  focus(options?: FocusOptions) {
    this.trigger.focus(options);
  }

  /** Removes focus from the color picker. */
  blur() {
    const elementToBlur = this.trigger;

    if (this.hasFocus) {
      // We don't know which element in the color picker has focus, so we'll move it to the trigger or base (inline) and
      // blur that instead. This results in document.activeElement becoming the `<body>`. This doesn't cause another
      // focus event because we're using focusin and something inside the color picker already has focus.
      elementToBlur.focus({ preventScroll: true });
      elementToBlur.blur();
    }

    if (this.popup?.active) {
      this.hide();
    }
  }

  /** Returns the current value as a string in the specified format. */
  getFormattedValue(
    format:
      | "hex"
      | "hexa"
      | "rgb"
      | "rgba"
      | "hsl"
      | "hsla"
      | "hsv"
      | "hsva" = "hex",
  ) {
    const currentColor = this.parseColor(
      `hsva(${this.hue}, ${this.saturation}%, ${this.brightness}%, ${this.alpha / 100})`,
    );

    if (currentColor === null) {
      return "";
    }

    switch (format) {
      case "hex":
        return currentColor.hex;
      case "hexa":
        return currentColor.hexa;
      case "rgb":
        return currentColor.rgb.string;
      case "rgba":
        return currentColor.rgba.string;
      case "hsl":
        return currentColor.hsl.string;
      case "hsla":
        return currentColor.hsla.string;
      case "hsv":
        return currentColor.hsv.string;
      case "hsva":
        return currentColor.hsva.string;
      default:
        return "";
    }
  }

  private reportValidityAfterShow = () => {
    // Remove the event so we don't emit "wa-invalid" twice
    this.removeEventListener("invalid", this.emitInvalid);

    this.reportValidity();

    this.addEventListener("invalid", this.emitInvalid);
  };

  /** Checks for validity and shows the browser's validation message if the control is invalid. */
  reportValidity() {
    // This won't get called when a form is submitted. This is only for manual calls.
    if (!this.validity.valid && !this.open) {
      // Show the popup so the browser can focus on it
      this.addEventListener("wa-after-show", this.reportValidityAfterShow, {
        once: true,
      });
      this.show();

      if (!this.disabled) {
        // By standards we have to emit a `wa-invalid` event here synchronously.
        this.dispatchEvent(new WaInvalidEvent());
      }

      return false;
    }

    return super.reportValidity();
  }

  formResetCallback() {
    this.value = this.defaultValue;

    super.formResetCallback();
  }

  firstUpdated(changedProperties: PropertyValues<this>): void {
    super.firstUpdated(changedProperties);

    this.hasEyeDropper = "EyeDropper" in window;
  }

  private handleKeyDown = (event: KeyboardEvent) => {
    // Close when escape is pressed inside an open popup. We need to listen on the panel itself and stop propagation
    // in case any ancestors are also listening for this key.
    if (this.open && event.key === "Escape" && isTopDismissible(this)) {
      event.stopPropagation();
      this.hide();
      this.focus();
    }
  };

  private handleDocumentKeyDown = (event: KeyboardEvent) => {
    // Close when escape or tab is pressed
    if (event.key === "Escape" && this.open && isTopDismissible(this)) {
      event.stopPropagation();
      this.focus();
      this.hide();
      return;
    }

    // Handle tabbing
    if (event.key === "Tab") {
      // Tabbing outside of the containing element closes the panel
      //
      // If the popup is used within a shadow DOM, we need to obtain the activeElement within that shadowRoot,
      // otherwise `document.activeElement` will only return the name of the parent shadow DOM element.
      setTimeout(() => {
        const activeElement =
          this.getRootNode() instanceof ShadowRoot
            ? document.activeElement?.shadowRoot?.activeElement
            : document.activeElement;

        if (
          !this ||
          activeElement?.closest(this.tagName.toLowerCase()) !== this
        ) {
          this.hide();
        }
      });
    }
  };

  private handleDocumentMouseDown = (event: MouseEvent) => {
    // Close when clicking outside of the popup panel and trigger
    const path = event.composedPath();

    // Check if click is inside the popup panel or the trigger element specifically
    const isInsideRelevantArea = path.some(
      (element) =>
        element instanceof Element &&
        (element.closest(".color-picker") || element === this.trigger),
    );

    if (this && !isInsideRelevantArea) {
      this.hide();
    }
  };

  handleTriggerClick() {
    if (this.open) {
      this.hide();
    } else {
      this.show();
      this.focus();
    }
  }

  async handleTriggerKeyDown(event: KeyboardEvent) {
    // When spacebar/enter is pressed, show the panel but don't focus on the menu. This let's the user press the same
    // key again to hide the menu in case they don't want to make a selection.
    if ([" ", "Enter"].includes(event.key)) {
      event.preventDefault();
      this.handleTriggerClick();
      return;
    }
  }

  handleTriggerKeyUp(event: KeyboardEvent) {
    // Prevent space from triggering a click event in Firefox
    if (event.key === " ") {
      event.preventDefault();
    }
  }

  updateAccessibleTrigger() {
    const accessibleTrigger = this.trigger;

    if (accessibleTrigger) {
      accessibleTrigger.setAttribute("aria-haspopup", "true");
      accessibleTrigger.setAttribute(
        "aria-expanded",
        this.open ? "true" : "false",
      );
    }
  }

  /** Shows the color picker panel. */
  async show() {
    if (this.open) {
      return undefined;
    }

    this.open = true;
    return waitForEvent(this, "wa-after-show");
  }

  /** Hides the color picker panel */
  async hide() {
    if (!this.open) {
      return undefined;
    }

    this.open = false;
    return waitForEvent(this, "wa-after-hide");
  }

  addOpenListeners() {
    this.base.addEventListener("keydown", this.handleKeyDown);
    document.addEventListener("keydown", this.handleDocumentKeyDown);
    document.addEventListener("mousedown", this.handleDocumentMouseDown);
    registerDismissible(this);
  }

  removeOpenListeners() {
    if (this.base) {
      this.base.removeEventListener("keydown", this.handleKeyDown);
    }
    document.removeEventListener("keydown", this.handleDocumentKeyDown);
    document.removeEventListener("mousedown", this.handleDocumentMouseDown);
    unregisterDismissible(this);
  }

  @watch("open", { waitUntilFirstUpdate: true })
  async handleOpenChange() {
    if (this.disabled) {
      this.open = false;
      return;
    }

    this.updateAccessibleTrigger();

    if (this.open) {
      // Show
      this.dispatchEvent(new CustomEvent("wa-show"));

      this.addOpenListeners();
      await this.updateComplete;
      this.base.hidden = false;
      this.popup.active = true;
      await animateWithClass(this.popup.popup, "show-with-scale");
      this.dispatchEvent(new CustomEvent("wa-after-show"));
    } else {
      // Hide
      this.dispatchEvent(new CustomEvent("wa-hide"));

      this.removeOpenListeners();
      await animateWithClass(this.popup.popup, "hide-with-scale");
      this.base.hidden = true;
      this.popup.active = false;
      this.dispatchEvent(new CustomEvent("wa-after-hide"));
    }
  }

  render() {
    const hasLabelSlot = !this.hasUpdated
      ? this.withLabel
      : this.withLabel || this.hasSlotController.test("label");
    const hasHintSlot = !this.hasUpdated
      ? this.withHint
      : this.withHint || this.hasSlotController.test("hint");
    const hasLabel = this.label ? true : !!hasLabelSlot;
    const hasHint = this.hint ? true : !!hasHintSlot;

    const gridHandleX = this.saturation;
    const gridHandleY = 100 - this.brightness;
    const normalizedSwatches: WaColorPickerSwatch[] = Array.isArray(
      this.swatches,
    )
      ? this.swatches.map((s) =>
          typeof s === "string" ? { color: s, label: s } : s,
        )
      : this.swatches
          .split(";")
          .filter((color) => color.trim() !== "")
          .map((color) => ({ color: color.trim(), label: color.trim() }));

    const colorPicker = html`
      <div
        part="base"
        class=${classMap({
          "color-picker": true,
        })}
        aria-disabled=${this.disabled ? "true" : "false"}
        tabindex="-1"
      >
        <div
          part="grid"
          class="grid"
          style=${styleMap({
            backgroundColor: this.getHexString(this.hue, 100, 100),
          })}
          @pointerdown=${this.handleGridDrag}
          @touchmove=${this.handleTouchMove}
        >
          <span
            part="grid-handle"
            class=${classMap({
              "grid-handle": true,
              "grid-handle-dragging": this.isDraggingGridHandle,
            })}
            style=${styleMap({
              top: `${gridHandleY}%`,
              left: `${gridHandleX}%`,
              backgroundColor: this.getHexString(
                this.hue,
                this.saturation,
                this.brightness,
                this.alpha,
              ),
            })}
            role="application"
            aria-label="HSV"
            tabindex=${ifDefined(this.disabled ? undefined : "0")}
            @keydown=${this.handleGridKeyDown}
          ></span>
        </div>

        <div class="controls">
          <div class="sliders">
            <div
              part="slider hue-slider"
              class="hue slider"
              @pointerdown=${this.handleHueDrag}
              @touchmove=${this.handleTouchMove}
            >
              <span
                part="slider-handle hue-slider-handle"
                class="slider-handle"
                style=${styleMap({
                  left: `${this.hue === 0 ? 0 : 100 / (360 / this.hue)}%`,
                  backgroundColor: this.getHexString(this.hue, 100, 100),
                })}
                role="slider"
                aria-label="hue"
                aria-orientation="horizontal"
                aria-valuemin="0"
                aria-valuemax="360"
                aria-valuenow=${`${Math.round(this.hue)}`}
                tabindex=${ifDefined(this.disabled ? undefined : "0")}
                @keydown=${this.handleHueKeyDown}
              ></span>
            </div>

            ${this.opacity
              ? html`
                  <div
                    part="slider opacity-slider"
                    class="alpha slider transparent-bg"
                    @pointerdown="${this.handleAlphaDrag}"
                    @touchmove=${this.handleTouchMove}
                  >
                    <div
                      class="alpha-gradient"
                      style=${styleMap({
                        backgroundImage: `linear-gradient(
                          to right,
                          ${this.getHexString(this.hue, this.saturation, this.brightness, 0)} 0%,
                          ${this.getHexString(this.hue, this.saturation, this.brightness, 100)} 100%
                        )`,
                      })}
                    ></div>
                    <span
                      part="slider-handle opacity-slider-handle"
                      class="slider-handle"
                      style=${styleMap({
                        left: `${this.alpha}%`,
                        backgroundColor: this.getHexString(
                          this.hue,
                          this.saturation,
                          this.brightness,
                          this.alpha,
                        ),
                      })}
                      role="slider"
                      aria-label="alpha"
                      aria-orientation="horizontal"
                      aria-valuemin="0"
                      aria-valuemax="100"
                      aria-valuenow=${Math.round(this.alpha)}
                      tabindex=${ifDefined(this.disabled ? undefined : "0")}
                      @keydown=${this.handleAlphaKeyDown}
                    ></span>
                  </div>
                `
              : ""}
          </div>

          <button
            type="button"
            part="preview"
            class="preview transparent-bg"
            aria-label=${this.localize.term("copy")}
            style=${styleMap({
              "--preview-color": this.getHexString(
                this.hue,
                this.saturation,
                this.brightness,
                this.alpha,
              ),
            })}
            @click=${this.handleCopy}
          ></button>
        </div>

        <div class="user-input" aria-live="polite">
          <wa-input
            part="input"
            type="text"
            name=${this.name}
            size="s"
            autocomplete="off"
            autocorrect="off"
            autocapitalize="off"
            spellcheck="false"
            .value=${this.isEmpty ? "" : this.inputValue}
            ?required=${this.required}
            ?disabled=${this.disabled}
            aria-label=${this.localize.term("currentValue")}
            @keydown=${this.handleInputKeyDown}
            @change=${this.handleInputChange}
            @input=${this.handleInputInput}
            @blur=${this.stopNestedEventPropagation}
            @focus=${this.stopNestedEventPropagation}
          ></wa-input>

          <wa-button-group>
            ${!this.withoutFormatToggle
              ? html`
                  <wa-button
                    part="format-button"
                    size="s"
                    appearance="outlined"
                    aria-label=${this.localize.term("toggleColorFormat")}
                    exportparts="
                      base:format-button__base,
                      start:format-button__start,
                      label:format-button__label,
                      end:format-button__end,
                      caret:format-button__caret
                    "
                    @click=${this.handleFormatToggle}
                    @blur=${this.stopNestedEventPropagation}
                    @focus=${this.stopNestedEventPropagation}
                  >
                    ${this.setLetterCase(this.format)}
                  </wa-button>
                `
              : ""}
            ${this.hasEyeDropper
              ? html`
                  <wa-button
                    part="eyedropper-button"
                    size="s"
                    appearance="outlined"
                    exportparts="
                      base:eyedropper-button__base,
                      start:eyedropper-button__start,
                      label:eyedropper-button__label,
                      end:eyedropper-button__end,
                      caret:eyedropper-button__caret
                    "
                    @click=${this.handleEyeDropper}
                    @blur=${this.stopNestedEventPropagation}
                    @focus=${this.stopNestedEventPropagation}
                  >
                    <wa-icon
                      library="system"
                      name="eyedropper"
                      variant="solid"
                      label=${this.localize.term("selectAColorFromTheScreen")}
                    ></wa-icon>
                  </wa-button>
                `
              : ""}
          </wa-button-group>
        </div>

        ${normalizedSwatches.length > 0
          ? html`
              <div part="swatches" class="swatches">
                ${normalizedSwatches.map((swatch) => {
                  const parsedColor = this.parseColor(swatch.color);

                  // If we can't parse it, skip it
                  if (!parsedColor) {
                    return "";
                  }

                  return html`
                    <div
                      part="swatch"
                      class="swatch transparent-bg"
                      tabindex=${ifDefined(this.disabled ? undefined : "0")}
                      role="button"
                      aria-label=${swatch.label}
                      @click=${() => this.selectSwatch(swatch.color)}
                      @keydown=${(event: KeyboardEvent) =>
                        !this.disabled &&
                        event.key === "Enter" &&
                        this.setColor(parsedColor.hexa)}
                    >
                      <div
                        class="swatch-color"
                        style=${styleMap({ backgroundColor: parsedColor.hexa })}
                      ></div>
                    </div>
                  `;
                })}
              </div>
            `
          : ""}
      </div>
    `;

    // Render with popup
    return html`
      <div
        class=${classMap({
          container: true,
          "form-control": true,
          "form-control-has-label": hasLabel,
        })}
        part="trigger-container form-control"
      >
        <div
          part="form-control-label"
          class=${classMap({
            label: true,
            "has-label": hasLabel,
          })}
          id="form-control-label"
        >
          <slot name="label">${this.label}</slot>
        </div>

        <button
          id="trigger"
          part="trigger form-control-input"
          class=${classMap({
            trigger: true,
            "trigger-empty": this.isEmpty,
            "transparent-bg": true,
            "form-control-input": true,
          })}
          style=${styleMap({
            color: this.getHexString(
              this.hue,
              this.saturation,
              this.brightness,
              this.alpha,
            ),
          })}
          type="button"
          aria-labelledby="form-control-label"
          aria-describedby="hint"
          .disabled=${this.disabled}
          @click=${this.handleTriggerClick}
          @keydown=${this.handleTriggerKeyDown}
          @keyup=${this.handleTriggerKeyUp}
        ></button>

        <slot
          id="hint"
          name="hint"
          part="hint"
          class=${classMap({
            "has-slotted": hasHint,
          })}
          >${this.hint}</slot
        >
      </div>

      <wa-popup
        class="color-popup"
        anchor="trigger"
        placement=${this.placement}
        distance="0"
        skidding="0"
        flip
        flip-fallback-strategy="best-fit"
        shift
        shift-padding="10"
        aria-disabled=${this.disabled ? "true" : "false"}
        @wa-after-show=${this.handleAfterShow}
        @wa-after-hide=${this.handleAfterHide}
      >
        ${colorPicker}
      </wa-popup>
    `;
  }
}

// The change-in-update warning is required for this component because:
//
// - The base class (WebAwesomeFormAssociatedElement) firstUpdated() calls updateValidity() which triggers
//    requestUpdate('validity').
// - HasSlotController calls host.requestUpdate() on slotchange events.
// - @watch('value') handler sets multiple @state properties (isEmpty, hue, saturation, brightness, alpha, inputValue)
//    and calls syncValues() and requestUpdate() during the update cycle to keep color state in sync.
// - @watch('opacity') and @watch('format') handlers set @state properties during update to synchronize color values.
// - firstUpdated() sets the @state property hasEyeDropper based on browser capability detection.
//
// See https://lit.dev/docs/tools/development/#development-build-runtime-warnings
WaColorPicker.disableWarning?.("change-in-update");

`````

### Expected (prettier)

`````ts
import { TinyColor } from "@ctrl/tinycolor";
import type { PropertyValues } from "lit";
import { html, isServer } from "lit";
import {
  customElement,
  eventOptions,
  property,
  query,
  state,
} from "lit/decorators.js";
import { classMap } from "lit/directives/class-map.js";
import { ifDefined } from "lit/directives/if-defined.js";
import { styleMap } from "lit/directives/style-map.js";
import { WaInvalidEvent } from "../../events/invalid.js";
import { animateWithClass } from "../../internal/animate.js";
import {
  isTopDismissible,
  registerDismissible,
  unregisterDismissible,
} from "../../internal/dismissible-stack.js";
import { drag } from "../../internal/drag.js";
import { waitForEvent } from "../../internal/event.js";
import { clamp } from "../../internal/math.js";
import { warnDeprecatedSize } from "../../internal/size.js";
import { HasSlotController } from "../../internal/slot.js";
import { RequiredValidator } from "../../internal/validators/required-validator.js";
import { watch } from "../../internal/watch.js";
import { WebAwesomeFormAssociatedElement } from "../../internal/webawesome-form-associated-element.js";
import formControlStyles from "../../styles/component/form-control.styles.js";
import sizeStyles from "../../styles/component/size.styles.js";
import visuallyHidden from "../../styles/component/visually-hidden.styles.js";
import { LocalizeController } from "../../utilities/localize.js";
import "../button-group/button-group.js";
import "../button/button.js";
import "../icon/icon.js";
import "../input/input.js";
import type WaInput from "../input/input.js";
import "../popup/popup.js";
import type WaPopup from "../popup/popup.js";
import styles from "./color-picker.styles.js";

export interface WaColorPickerSwatch {
  color: string;
  label: string;
}

interface EyeDropperConstructor {
  new (): EyeDropperInterface;
}

interface EyeDropperInterface {
  open: () => Promise<{ sRGBHex: string }>;
}

declare const EyeDropper: EyeDropperConstructor;

/**
 * @summary Color pickers let users choose a color from a visual palette or by entering a value. They support HEX, RGB,
 *  HSL, and HSV formats with optional alpha channel and swatch presets.
 * @documentation https://webawesome.com/docs/components/color-picker
 * @status stable
 * @since 2.0
 *
 * @dependency wa-button
 * @dependency wa-button-group
 * @dependency wa-input
 * @dependency wa-popup
 * @dependency wa-visually-hidden
 *
 * @slot label - The color picker's form label. Alternatively, you can use the `label` attribute.
 * @slot hint - The color picker's form hint. Alternatively, you can use the `hint` attribute.
 *
 * @event blur - Emitted when the color picker loses focus.
 * @event change - Emitted when the color picker's value changes.
 * @event focus - Emitted when the color picker receives focus.
 * @event input - Emitted when the color picker receives input.
 * @event wa-invalid - Emitted when the form control has been checked for validity and its constraints aren't satisfied.
 *
 * @csspart base - The component's base wrapper.
 * @csspart trigger - The color picker's dropdown trigger.
 * @csspart swatches - The container that holds the swatches.
 * @csspart swatch - Each individual swatch.
 * @csspart grid - The color grid.
 * @csspart grid-handle - The color grid's handle.
 * @csspart slider - Hue and opacity sliders.
 * @csspart slider-handle - Hue and opacity slider handles.
 * @csspart hue-slider - The hue slider.
 * @csspart hue-slider-handle - The hue slider's handle.
 * @csspart opacity-slider - The opacity slider.
 * @csspart opacity-slider-handle - The opacity slider's handle.
 * @csspart preview - The preview color.
 * @csspart input - The text input.
 * @csspart eyedropper-button - The eye dropper button.
 * @csspart eyedropper-button__base - The eye dropper button's exported `button` part.
 * @csspart eyedropper-button__start - The eye dropper button's exported `start` part.
 * @csspart eyedropper-button__label - The eye dropper button's exported `label` part.
 * @csspart eyedropper-button__end - The eye dropper button's exported `end` part.
 * @csspart eyedropper-button__caret - The eye dropper button's exported `caret` part.
 * @csspart format-button - The format button.
 * @csspart format-button__base - The format button's exported `button` part.
 * @csspart format-button__start - The format button's exported `start` part.
 * @csspart format-button__label - The format button's exported `label` part.
 * @csspart format-button__end - The format button's exported `end` part.
 * @csspart format-button__caret - The format button's exported `caret` part.
 *
 * @cssproperty --grid-width - The width of the color grid.
 * @cssproperty --grid-height - The height of the color grid.
 * @cssproperty --grid-handle-size - The size of the color grid's handle.
 * @cssproperty --slider-height - The height of the hue and alpha sliders.
 * @cssproperty --slider-handle-size - The diameter of the slider's handle.
 */
@customElement("wa-color-picker")
export default class WaColorPicker extends WebAwesomeFormAssociatedElement {
  static css = [visuallyHidden, sizeStyles, formControlStyles, styles];

  static shadowRootOptions = {
    ...WebAwesomeFormAssociatedElement.shadowRootOptions,
    delegatesFocus: true,
  };

  static get validators() {
    const validators = isServer ? [] : [RequiredValidator()];
    return [...super.validators, ...validators];
  }

  private readonly hasSlotController = new HasSlotController(
    this,
    "hint",
    "label",
  );

  private isSafeValue = false;
  private readonly localize = new LocalizeController(this);

  @query('[part~="base"]') base: HTMLElement;
  @query('[part~="input"]') input: WaInput;
  @query('[part~="form-control-label"]') triggerLabel: HTMLElement;
  @query('[part~="form-control-input"]') triggerButton: HTMLButtonElement;

  // @TODO: This is a hacky way to show the "Please fill out this field", do we want the old behavior where it opens the dropdown?
  //   or is the new behavior okay?
  get validationTarget() {
    // This puts the popup on the element only if the color picker is expanded.
    if (this.popup?.active) {
      return this.input;
    }

    // This puts popup on the color picker itself without needing to expand it to show the input.
    // This is necessary because form submissions expect the "anchor" to be currently shown.
    return this.trigger;
  }

  @query(".color-popup") popup: WaPopup;
  @query('[part~="preview"]') previewButton: HTMLButtonElement;
  @query('[part~="trigger"]') trigger: HTMLButtonElement;

  @state() private hasFocus = false;
  @state() private isDraggingGridHandle = false;
  @state() private isEmpty = true;
  @state() private inputValue = "";
  @state() private hue = 0;
  @state() private saturation = 100;
  @state() private brightness = 100;
  @state() private alpha = 100;

  private _value: string | null = null;

  /** The current value of the input, submitted as a name/value pair with form data. */
  get value() {
    if (this.valueHasChanged) {
      return this._value;
    }

    return this._value ?? this.defaultValue;
  }

  /**
   * The current value of the color picker. The value's format will vary based the `format` attribute. To get the value
   * in a specific format, use the `getFormattedValue()` method. The value is submitted as a name/value pair with form
   * data.
   */

  @state() set value(val: string | null) {
    if (this._value === val) {
      return;
    }

    this.valueHasChanged = true;
    this._value = val;
  }

  /** The default value of the form control. Primarily used for resetting the form control. */
  @property({ attribute: "value", reflect: true }) defaultValue: string | null =
    this.getAttribute("value") || null;

  /**
   * Only required for SSR. Set to `true` if you're slotting in a `label` element so the server-rendered markup
   * includes the label before the component hydrates on the client.
   */
  @property({ attribute: "with-label", reflect: true, type: Boolean })
  withLabel = false;

  /**
   * Only required for SSR. Set to `true` if you're slotting in a `hint` element so the server-rendered markup
   * includes the hint before the component hydrates on the client.
   */
  @property({ attribute: "with-hint", reflect: true, type: Boolean }) withHint =
    false;

  @state() private hasEyeDropper: boolean = false;

  /**
   * The color picker's label. This will not be displayed, but it will be announced by assistive devices. If you need to
   * display HTML, you can use the `label` slot` instead.
   */
  @property() label = "";

  /**
   * The color picker's hint. If you need to display HTML, use the `hint` slot instead.
   */
  @property({ attribute: "hint" }) hint = "";

  /**
   * The format to use. If opacity is enabled, these will translate to HEXA, RGBA, HSLA, and HSVA respectively. The color
   * picker will accept user input in any format (including CSS color names) and convert it to the desired format.
   */
  @property() format: "hex" | "rgb" | "hsl" | "hsv" = "hex";

  /** Determines the size of the color picker's trigger */
  @property({ reflect: true }) size:
    "xs" | "s" | "m" | "l" | "xl" | "small" | "medium" | "large" = "m";

  @watch("size")
  handleSizeChange() {
    warnDeprecatedSize(this.localName, this.size);
  }

  /**
   * The preferred placement of the color picker's popup. Note that the actual placement will vary as configured to
   * keep the panel inside of the viewport.
   */
  @property({ reflect: true }) placement:
    | "top"
    | "top-start"
    | "top-end"
    | "bottom"
    | "bottom-start"
    | "bottom-end"
    | "right"
    | "right-start"
    | "right-end"
    | "left"
    | "left-start"
    | "left-end" = "bottom-start";

  /** Removes the button that lets users toggle between format.   */
  @property({ attribute: "without-format-toggle", type: Boolean })
  withoutFormatToggle = false;

  /** The name of the form control, submitted as a name/value pair with form data. */
  @property({ reflect: true }) name: string | null = null;

  /** Disables the color picker. */
  @property({ type: Boolean }) disabled = false;

  /**
   * Indicates whether or not the popup is open. You can toggle this attribute to show and hide the popup, or you
   * can use the `show()` and `hide()` methods and this attribute will reflect the popup's open state.
   */
  @property({ type: Boolean, reflect: true }) open = false;

  /** Shows the opacity slider. Enabling this will cause the formatted value to be HEXA, RGBA, or HSLA. */
  @property({ type: Boolean }) opacity = false;

  /** By default, values are lowercase. With this attribute, values will be uppercase instead. */
  @property({ type: Boolean }) uppercase = false;

  /**
   * One or more predefined color swatches to display as presets in the color picker. Can include any format the color
   * picker can parse, including HEX(A), RGB(A), HSL(A), HSV(A), and CSS color names. Each color must be separated by a
   * semicolon (`;`). Alternatively, you can pass an array of color values or an array of `{ color, label }` objects to
   * this property using JavaScript. When using objects with labels, the label will be used for the swatch's accessible
   * name instead of the raw color value.
   */
  @property() swatches: string | string[] | WaColorPickerSwatch[] = "";

  /** Makes the color picker a required field. */
  @property({ type: Boolean, reflect: true }) required = false;

  constructor() {
    super();

    if (!isServer) {
      this.addEventListener("focusin", this.handleFocusIn);
      this.addEventListener("focusout", this.handleFocusOut);
    }
  }

  private handleCopy() {
    this.input.select();
    document.execCommand("copy");
    this.previewButton.focus();

    // Show copied animation
    this.previewButton.classList.add("preview-color-copied");
    this.previewButton.addEventListener("animationend", () => {
      this.previewButton.classList.remove("preview-color-copied");
    });
  }

  private handleFocusIn = () => {
    this.hasFocus = true;
  };

  private handleFocusOut = () => {
    this.hasFocus = false;
  };

  private handleFormatToggle() {
    const formats = ["hex", "rgb", "hsl", "hsv"];
    const nextIndex = (formats.indexOf(this.format) + 1) % formats.length;
    this.format = formats[nextIndex] as "hex" | "rgb" | "hsl" | "hsv";
    this.setColor(this.value || "");

    this.updateComplete.then(() => {
      this.dispatchEvent(
        new Event("change", { bubbles: true, composed: true }),
      );
      this.dispatchEvent(
        new InputEvent("input", { bubbles: true, composed: true }),
      );
    });
  }

  private handleAlphaDrag(event: PointerEvent) {
    const container =
      this.shadowRoot!.querySelector<HTMLElement>(".slider.alpha")!;
    const handle = container.querySelector<HTMLElement>(".slider-handle")!;
    const { width } = container.getBoundingClientRect();
    let initialValue = this.value;
    let currentValue = this.value;

    handle.focus();
    event.preventDefault();

    drag(container, {
      onMove: (x) => {
        this.alpha = clamp((x / width) * 100, 0, 100);
        this.syncValues();

        if (this.value !== currentValue) {
          currentValue = this.value;
          this.updateComplete.then(() => {
            this.dispatchEvent(
              new InputEvent("input", { bubbles: true, composed: true }),
            );
          });
        }
      },
      onStop: () => {
        if (this.value !== initialValue) {
          initialValue = this.value;

          this.updateComplete.then(() => {
            this.dispatchEvent(
              new Event("change", { bubbles: true, composed: true }),
            );
          });
        }
      },
      initialEvent: event,
    });
  }

  private handleHueDrag(event: PointerEvent) {
    const container =
      this.shadowRoot!.querySelector<HTMLElement>(".slider.hue")!;
    const handle = container.querySelector<HTMLElement>(".slider-handle")!;
    const { width } = container.getBoundingClientRect();
    let initialValue = this.value;
    let currentValue = this.value;

    handle.focus();
    event.preventDefault();

    drag(container, {
      onMove: (x) => {
        this.hue = clamp((x / width) * 360, 0, 360);
        this.syncValues();

        if (this.value !== currentValue) {
          currentValue = this.value;
          this.updateComplete.then(() => {
            this.dispatchEvent(new InputEvent("input"));
          });
        }
      },
      onStop: () => {
        if (this.value !== initialValue) {
          initialValue = this.value;
          this.updateComplete.then(() => {
            this.dispatchEvent(
              new Event("change", { bubbles: true, composed: true }),
            );
          });
        }
      },
      initialEvent: event,
    });
  }

  private handleGridDrag(event: PointerEvent) {
    const grid = this.shadowRoot!.querySelector<HTMLElement>(".grid")!;
    const handle = grid.querySelector<HTMLElement>(".grid-handle")!;
    const { width, height } = grid.getBoundingClientRect();
    let initialValue = this.value;
    let currentValue = this.value;

    handle.focus();
    event.preventDefault();

    this.isDraggingGridHandle = true;

    drag(grid, {
      onMove: (x, y) => {
        this.saturation = clamp((x / width) * 100, 0, 100);
        this.brightness = clamp(100 - (y / height) * 100, 0, 100);
        this.syncValues();

        if (this.value !== currentValue) {
          currentValue = this.value;
          this.updateComplete.then(() => {
            this.dispatchEvent(
              new InputEvent("input", { bubbles: true, composed: true }),
            );
          });
        }
      },
      onStop: () => {
        this.isDraggingGridHandle = false;
        if (this.value !== initialValue) {
          initialValue = this.value;
          this.updateComplete.then(() => {
            this.dispatchEvent(
              new Event("change", { bubbles: true, composed: true }),
            );
          });
        }
      },
      initialEvent: event,
    });
  }

  private handleAlphaKeyDown(event: KeyboardEvent) {
    const increment = event.shiftKey ? 10 : 1;
    const oldValue = this.value;

    if (event.key === "ArrowLeft") {
      event.preventDefault();
      this.alpha = clamp(this.alpha - increment, 0, 100);
      this.syncValues();
    }

    if (event.key === "ArrowRight") {
      event.preventDefault();
      this.alpha = clamp(this.alpha + increment, 0, 100);
      this.syncValues();
    }

    if (event.key === "Home") {
      event.preventDefault();
      this.alpha = 0;
      this.syncValues();
    }

    if (event.key === "End") {
      event.preventDefault();
      this.alpha = 100;
      this.syncValues();
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
  }

  private handleHueKeyDown(event: KeyboardEvent) {
    const increment = event.shiftKey ? 10 : 1;
    const oldValue = this.value;

    if (event.key === "ArrowLeft") {
      event.preventDefault();
      this.hue = clamp(this.hue - increment, 0, 360);
      this.syncValues();
    }

    if (event.key === "ArrowRight") {
      event.preventDefault();
      this.hue = clamp(this.hue + increment, 0, 360);
      this.syncValues();
    }

    if (event.key === "Home") {
      event.preventDefault();
      this.hue = 0;
      this.syncValues();
    }

    if (event.key === "End") {
      event.preventDefault();
      this.hue = 360;
      this.syncValues();
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
  }

  private handleGridKeyDown(event: KeyboardEvent) {
    const increment = event.shiftKey ? 10 : 1;
    const oldValue = this.value;

    if (event.key === "ArrowLeft") {
      event.preventDefault();
      this.saturation = clamp(this.saturation - increment, 0, 100);
      this.syncValues();
    }

    if (event.key === "ArrowRight") {
      event.preventDefault();
      this.saturation = clamp(this.saturation + increment, 0, 100);
      this.syncValues();
    }

    if (event.key === "ArrowUp") {
      event.preventDefault();
      this.brightness = clamp(this.brightness + increment, 0, 100);
      this.syncValues();
    }

    if (event.key === "ArrowDown") {
      event.preventDefault();
      this.brightness = clamp(this.brightness - increment, 0, 100);
      this.syncValues();
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
  }

  private handleInputChange(event: Event) {
    const target = event.target as HTMLInputElement;
    const oldValue = this.value;

    // Prevent the `<wa-input>` element's `change` event from bubbling up
    event.stopPropagation();

    if (this.input.value) {
      this.setColor(target.value);
      target.value = this.value || "";
    } else {
      this.value = "";
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
  }

  private handleInputInput(event: InputEvent) {
    this.updateValidity();

    // Prevent the `<wa-input>` element's `input` event from bubbling up
    event.stopPropagation();
  }

  private handleInputKeyDown(event: KeyboardEvent) {
    if (event.key === "Enter") {
      const oldValue = this.value;

      if (this.input.value) {
        this.setColor(this.input.value);
        this.input.value = this.value;

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

        setTimeout(() => this.input.select());
      } else {
        this.hue = 0;
      }
    }
  }

  @eventOptions({ passive: false })
  private handleTouchMove(event: TouchEvent) {
    event.preventDefault();
  }

  private parseColor(colorString: string) {
    if (!colorString || colorString.trim() === "") {
      return null;
    }

    const color = new TinyColor(colorString);
    if (!color.isValid) {
      return null;
    }

    const hslColor = color.toHsl();
    const rgb = color.toRgb();
    const hsvColor = color.toHsv();

    // Checks for null RGB values
    if (!rgb || rgb.r == null || rgb.g == null || rgb.b == null) {
      return null;
    }

    // Adjust saturation and lightness from 0-1 to 0-100
    const hsl = {
      h: hslColor.h || 0,
      s: (hslColor.s || 0) * 100,
      l: (hslColor.l || 0) * 100,
      a: hslColor.a || 0,
    };

    const hex = color.toHexString();
    const hexa = color.toHex8String();

    // Adjust saturation and value from 0-1 to 0-100
    const hsv = {
      h: hsvColor.h || 0,
      s: (hsvColor.s || 0) * 100,
      v: (hsvColor.v || 0) * 100,
      a: hsvColor.a || 0,
    };

    return {
      hsl: {
        h: hsl.h,
        s: hsl.s,
        l: hsl.l,
        string: this.setLetterCase(
          `hsl(${Math.round(hsl.h)}, ${Math.round(hsl.s)}%, ${Math.round(hsl.l)}%)`,
        ),
      },
      hsla: {
        h: hsl.h,
        s: hsl.s,
        l: hsl.l,
        a: hsl.a,
        string: this.setLetterCase(
          `hsla(${Math.round(hsl.h)}, ${Math.round(hsl.s)}%, ${Math.round(hsl.l)}%, ${hsl.a.toFixed(2).toString()})`,
        ),
      },
      hsv: {
        h: hsv.h,
        s: hsv.s,
        v: hsv.v,
        string: this.setLetterCase(
          `hsv(${Math.round(hsv.h)}, ${Math.round(hsv.s)}%, ${Math.round(hsv.v)}%)`,
        ),
      },
      hsva: {
        h: hsv.h,
        s: hsv.s,
        v: hsv.v,
        a: hsv.a,
        string: this.setLetterCase(
          `hsva(${Math.round(hsv.h)}, ${Math.round(hsv.s)}%, ${Math.round(hsv.v)}%, ${hsv.a.toFixed(2).toString()})`,
        ),
      },
      rgb: {
        r: rgb.r,
        g: rgb.g,
        b: rgb.b,
        string: this.setLetterCase(
          `rgb(${Math.round(rgb.r)}, ${Math.round(rgb.g)}, ${Math.round(rgb.b)})`,
        ),
      },
      rgba: {
        r: rgb.r,
        g: rgb.g,
        b: rgb.b,
        a: rgb.a || 0,
        string: this.setLetterCase(
          `rgba(${Math.round(rgb.r)}, ${Math.round(rgb.g)}, ${Math.round(rgb.b)}, ${(rgb.a || 0).toFixed(2).toString()})`,
        ),
      },
      hex: this.setLetterCase(hex),
      hexa: this.setLetterCase(hexa),
    };
  }

  private setColor(colorString: string) {
    const newColor = this.parseColor(colorString);

    if (newColor === null) {
      return false;
    }

    this.hue = newColor.hsva.h;
    this.saturation = newColor.hsva.s;
    this.brightness = newColor.hsva.v;
    this.alpha = this.opacity ? newColor.hsva.a * 100 : 100;

    this.syncValues();

    return true;
  }

  private setLetterCase(string: string) {
    if (typeof string !== "string") {
      return "";
    }
    return this.uppercase ? string.toUpperCase() : string.toLowerCase();
  }

  private async syncValues() {
    const currentColor = this.parseColor(
      `hsva(${this.hue}, ${this.saturation}%, ${this.brightness}%, ${this.alpha / 100})`,
    );

    if (currentColor === null) {
      return;
    }

    // Update the value
    if (this.format === "hsl") {
      this.inputValue = this.opacity
        ? currentColor.hsla.string
        : currentColor.hsl.string;
    } else if (this.format === "rgb") {
      this.inputValue = this.opacity
        ? currentColor.rgba.string
        : currentColor.rgb.string;
    } else if (this.format === "hsv") {
      this.inputValue = this.opacity
        ? currentColor.hsva.string
        : currentColor.hsv.string;
    } else {
      this.inputValue = this.opacity ? currentColor.hexa : currentColor.hex;
    }

    // Setting this.value will trigger the watcher which parses the new value. We want to bypass that behavior because
    // we've already parsed the color here and conversion/rounding can lead to values changing slightly. When this
    // happens, dragging the grid handle becomes jumpy. After the next update, the usual behavior is restored.
    this.isSafeValue = true;
    this.value = this.inputValue;

    await this.updateComplete;
    this.isSafeValue = false;
  }

  private handleAfterHide() {
    this.previewButton.classList.remove("preview-color-copied");
    // Update validity so we get a new anchor.
    this.updateValidity();
  }

  private handleAfterShow() {
    // Update validity so we get a new anchor.
    this.updateValidity();
  }

  private handleEyeDropper() {
    if (!this.hasEyeDropper) {
      return;
    }

    const eyeDropper = new EyeDropper();

    eyeDropper
      .open()
      .then((colorSelectionResult) => {
        const oldValue = this.value;

        this.setColor(colorSelectionResult.sRGBHex);

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
      })
      .catch(() => {
        // The user canceled, do nothing
      });
  }

  private selectSwatch(color: string) {
    const oldValue = this.value;

    if (!this.disabled) {
      this.setColor(color);

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
    }
  }

  /** Generates a hex string from HSV values. Hue must be 0-360. All other arguments must be 0-100. */
  getHexString(
    hue: number,
    saturation: number,
    brightness: number,
    alpha = 100,
  ) {
    const color = new TinyColor(
      `hsva(${hue}, ${saturation}%, ${brightness}%, ${alpha / 100})`,
    );
    if (!color.isValid) {
      return "";
    }

    return color.toHex8String();
  }

  // Prevents nested components from leaking events
  private stopNestedEventPropagation(event: CustomEvent) {
    event.stopImmediatePropagation();
  }

  @watch("format", { waitUntilFirstUpdate: true })
  handleFormatChange() {
    this.syncValues();
  }

  @watch("opacity")
  handleOpacityChange() {
    this.alpha = 100;
  }

  protected willUpdate(changedProperties: PropertyValues<this>): void {
    super.willUpdate(changedProperties);

    // Its kind of bizarre, but this is required to get SSR to play nicely.
    if (changedProperties.has("value")) {
      this.handleValueChange(
        changedProperties.get("value") || "",
        this.value || "",
      );
    }
  }

  @watch("value")
  handleValueChange(oldValue: string | undefined, newValue: string) {
    this.isEmpty = !newValue;

    if (!newValue) {
      this.hue = 0;
      this.saturation = 0;
      this.brightness = 100;
      this.alpha = 100;
    }

    if (!this.isSafeValue) {
      const newColor = this.parseColor(newValue);

      if (newColor !== null) {
        this.inputValue = this.value || "";
        this.hue = newColor.hsva.h;
        this.saturation = newColor.hsva.s;
        this.brightness = newColor.hsva.v;
        this.alpha = newColor.hsva.a * 100;
        this.syncValues();
      } else {
        this.inputValue = oldValue ?? "";
      }
    }

    this.requestUpdate();
  }

  /** Sets focus on the color picker. */
  focus(options?: FocusOptions) {
    this.trigger.focus(options);
  }

  /** Removes focus from the color picker. */
  blur() {
    const elementToBlur = this.trigger;

    if (this.hasFocus) {
      // We don't know which element in the color picker has focus, so we'll move it to the trigger or base (inline) and
      // blur that instead. This results in document.activeElement becoming the `<body>`. This doesn't cause another
      // focus event because we're using focusin and something inside the color picker already has focus.
      elementToBlur.focus({ preventScroll: true });
      elementToBlur.blur();
    }

    if (this.popup?.active) {
      this.hide();
    }
  }

  /** Returns the current value as a string in the specified format. */
  getFormattedValue(
    format:
      "hex" | "hexa" | "rgb" | "rgba" | "hsl" | "hsla" | "hsv" | "hsva" = "hex",
  ) {
    const currentColor = this.parseColor(
      `hsva(${this.hue}, ${this.saturation}%, ${this.brightness}%, ${this.alpha / 100})`,
    );

    if (currentColor === null) {
      return "";
    }

    switch (format) {
      case "hex":
        return currentColor.hex;
      case "hexa":
        return currentColor.hexa;
      case "rgb":
        return currentColor.rgb.string;
      case "rgba":
        return currentColor.rgba.string;
      case "hsl":
        return currentColor.hsl.string;
      case "hsla":
        return currentColor.hsla.string;
      case "hsv":
        return currentColor.hsv.string;
      case "hsva":
        return currentColor.hsva.string;
      default:
        return "";
    }
  }

  private reportValidityAfterShow = () => {
    // Remove the event so we don't emit "wa-invalid" twice
    this.removeEventListener("invalid", this.emitInvalid);

    this.reportValidity();

    this.addEventListener("invalid", this.emitInvalid);
  };

  /** Checks for validity and shows the browser's validation message if the control is invalid. */
  reportValidity() {
    // This won't get called when a form is submitted. This is only for manual calls.
    if (!this.validity.valid && !this.open) {
      // Show the popup so the browser can focus on it
      this.addEventListener("wa-after-show", this.reportValidityAfterShow, {
        once: true,
      });
      this.show();

      if (!this.disabled) {
        // By standards we have to emit a `wa-invalid` event here synchronously.
        this.dispatchEvent(new WaInvalidEvent());
      }

      return false;
    }

    return super.reportValidity();
  }

  formResetCallback() {
    this.value = this.defaultValue;

    super.formResetCallback();
  }

  firstUpdated(changedProperties: PropertyValues<this>): void {
    super.firstUpdated(changedProperties);

    this.hasEyeDropper = "EyeDropper" in window;
  }

  private handleKeyDown = (event: KeyboardEvent) => {
    // Close when escape is pressed inside an open popup. We need to listen on the panel itself and stop propagation
    // in case any ancestors are also listening for this key.
    if (this.open && event.key === "Escape" && isTopDismissible(this)) {
      event.stopPropagation();
      this.hide();
      this.focus();
    }
  };

  private handleDocumentKeyDown = (event: KeyboardEvent) => {
    // Close when escape or tab is pressed
    if (event.key === "Escape" && this.open && isTopDismissible(this)) {
      event.stopPropagation();
      this.focus();
      this.hide();
      return;
    }

    // Handle tabbing
    if (event.key === "Tab") {
      // Tabbing outside of the containing element closes the panel
      //
      // If the popup is used within a shadow DOM, we need to obtain the activeElement within that shadowRoot,
      // otherwise `document.activeElement` will only return the name of the parent shadow DOM element.
      setTimeout(() => {
        const activeElement =
          this.getRootNode() instanceof ShadowRoot
            ? document.activeElement?.shadowRoot?.activeElement
            : document.activeElement;

        if (
          !this ||
          activeElement?.closest(this.tagName.toLowerCase()) !== this
        ) {
          this.hide();
        }
      });
    }
  };

  private handleDocumentMouseDown = (event: MouseEvent) => {
    // Close when clicking outside of the popup panel and trigger
    const path = event.composedPath();

    // Check if click is inside the popup panel or the trigger element specifically
    const isInsideRelevantArea = path.some(
      (element) =>
        element instanceof Element &&
        (element.closest(".color-picker") || element === this.trigger),
    );

    if (this && !isInsideRelevantArea) {
      this.hide();
    }
  };

  handleTriggerClick() {
    if (this.open) {
      this.hide();
    } else {
      this.show();
      this.focus();
    }
  }

  async handleTriggerKeyDown(event: KeyboardEvent) {
    // When spacebar/enter is pressed, show the panel but don't focus on the menu. This let's the user press the same
    // key again to hide the menu in case they don't want to make a selection.
    if ([" ", "Enter"].includes(event.key)) {
      event.preventDefault();
      this.handleTriggerClick();
      return;
    }
  }

  handleTriggerKeyUp(event: KeyboardEvent) {
    // Prevent space from triggering a click event in Firefox
    if (event.key === " ") {
      event.preventDefault();
    }
  }

  updateAccessibleTrigger() {
    const accessibleTrigger = this.trigger;

    if (accessibleTrigger) {
      accessibleTrigger.setAttribute("aria-haspopup", "true");
      accessibleTrigger.setAttribute(
        "aria-expanded",
        this.open ? "true" : "false",
      );
    }
  }

  /** Shows the color picker panel. */
  async show() {
    if (this.open) {
      return undefined;
    }

    this.open = true;
    return waitForEvent(this, "wa-after-show");
  }

  /** Hides the color picker panel */
  async hide() {
    if (!this.open) {
      return undefined;
    }

    this.open = false;
    return waitForEvent(this, "wa-after-hide");
  }

  addOpenListeners() {
    this.base.addEventListener("keydown", this.handleKeyDown);
    document.addEventListener("keydown", this.handleDocumentKeyDown);
    document.addEventListener("mousedown", this.handleDocumentMouseDown);
    registerDismissible(this);
  }

  removeOpenListeners() {
    if (this.base) {
      this.base.removeEventListener("keydown", this.handleKeyDown);
    }
    document.removeEventListener("keydown", this.handleDocumentKeyDown);
    document.removeEventListener("mousedown", this.handleDocumentMouseDown);
    unregisterDismissible(this);
  }

  @watch("open", { waitUntilFirstUpdate: true })
  async handleOpenChange() {
    if (this.disabled) {
      this.open = false;
      return;
    }

    this.updateAccessibleTrigger();

    if (this.open) {
      // Show
      this.dispatchEvent(new CustomEvent("wa-show"));

      this.addOpenListeners();
      await this.updateComplete;
      this.base.hidden = false;
      this.popup.active = true;
      await animateWithClass(this.popup.popup, "show-with-scale");
      this.dispatchEvent(new CustomEvent("wa-after-show"));
    } else {
      // Hide
      this.dispatchEvent(new CustomEvent("wa-hide"));

      this.removeOpenListeners();
      await animateWithClass(this.popup.popup, "hide-with-scale");
      this.base.hidden = true;
      this.popup.active = false;
      this.dispatchEvent(new CustomEvent("wa-after-hide"));
    }
  }

  render() {
    const hasLabelSlot = !this.hasUpdated
      ? this.withLabel
      : this.withLabel || this.hasSlotController.test("label");
    const hasHintSlot = !this.hasUpdated
      ? this.withHint
      : this.withHint || this.hasSlotController.test("hint");
    const hasLabel = this.label ? true : !!hasLabelSlot;
    const hasHint = this.hint ? true : !!hasHintSlot;

    const gridHandleX = this.saturation;
    const gridHandleY = 100 - this.brightness;
    const normalizedSwatches: WaColorPickerSwatch[] = Array.isArray(
      this.swatches,
    )
      ? this.swatches.map((s) =>
          typeof s === "string" ? { color: s, label: s } : s,
        )
      : this.swatches
          .split(";")
          .filter((color) => color.trim() !== "")
          .map((color) => ({ color: color.trim(), label: color.trim() }));

    const colorPicker = html`
      <div
        part="base"
        class=${classMap({
          "color-picker": true,
        })}
        aria-disabled=${this.disabled ? "true" : "false"}
        tabindex="-1"
      >
        <div
          part="grid"
          class="grid"
          style=${styleMap({ backgroundColor: this.getHexString(this.hue, 100, 100) })}
          @pointerdown=${this.handleGridDrag}
          @touchmove=${this.handleTouchMove}
        >
          <span
            part="grid-handle"
            class=${classMap({
              "grid-handle": true,
              "grid-handle-dragging": this.isDraggingGridHandle,
            })}
            style=${styleMap({
              top: `${gridHandleY}%`,
              left: `${gridHandleX}%`,
              backgroundColor: this.getHexString(
                this.hue,
                this.saturation,
                this.brightness,
                this.alpha,
              ),
            })}
            role="application"
            aria-label="HSV"
            tabindex=${ifDefined(this.disabled ? undefined : "0")}
            @keydown=${this.handleGridKeyDown}
          ></span>
        </div>

        <div class="controls">
          <div class="sliders">
            <div
              part="slider hue-slider"
              class="hue slider"
              @pointerdown=${this.handleHueDrag}
              @touchmove=${this.handleTouchMove}
            >
              <span
                part="slider-handle hue-slider-handle"
                class="slider-handle"
                style=${styleMap({
                  left: `${this.hue === 0 ? 0 : 100 / (360 / this.hue)}%`,
                  backgroundColor: this.getHexString(this.hue, 100, 100),
                })}
                role="slider"
                aria-label="hue"
                aria-orientation="horizontal"
                aria-valuemin="0"
                aria-valuemax="360"
                aria-valuenow=${`${Math.round(this.hue)}`}
                tabindex=${ifDefined(this.disabled ? undefined : "0")}
                @keydown=${this.handleHueKeyDown}
              ></span>
            </div>

            ${
              this.opacity
                ? html`
                    <div
                      part="slider opacity-slider"
                      class="alpha slider transparent-bg"
                      @pointerdown="${this.handleAlphaDrag}"
                      @touchmove=${this.handleTouchMove}
                    >
                      <div
                        class="alpha-gradient"
                        style=${styleMap({
                        backgroundImage: `linear-gradient(
                          to right,
                          ${this.getHexString(this.hue, this.saturation, this.brightness, 0)} 0%,
                          ${this.getHexString(this.hue, this.saturation, this.brightness, 100)} 100%
                        )`,
                      })}
                      ></div>
                      <span
                        part="slider-handle opacity-slider-handle"
                        class="slider-handle"
                        style=${styleMap({
                        left: `${this.alpha}%`,
                        backgroundColor: this.getHexString(
                          this.hue,
                          this.saturation,
                          this.brightness,
                          this.alpha,
                        ),
                      })}
                        role="slider"
                        aria-label="alpha"
                        aria-orientation="horizontal"
                        aria-valuemin="0"
                        aria-valuemax="100"
                        aria-valuenow=${Math.round(this.alpha)}
                        tabindex=${ifDefined(this.disabled ? undefined : "0")}
                        @keydown=${this.handleAlphaKeyDown}
                      ></span>
                    </div>
                  `
                : ""
            }
          </div>

          <button
            type="button"
            part="preview"
            class="preview transparent-bg"
            aria-label=${this.localize.term("copy")}
            style=${styleMap({
              "--preview-color": this.getHexString(
                this.hue,
                this.saturation,
                this.brightness,
                this.alpha,
              ),
            })}
            @click=${this.handleCopy}
          ></button>
        </div>

        <div class="user-input" aria-live="polite">
          <wa-input
            part="input"
            type="text"
            name=${this.name}
            size="s"
            autocomplete="off"
            autocorrect="off"
            autocapitalize="off"
            spellcheck="false"
            .value=${this.isEmpty ? "" : this.inputValue}
            ?required=${this.required}
            ?disabled=${this.disabled}
            aria-label=${this.localize.term("currentValue")}
            @keydown=${this.handleInputKeyDown}
            @change=${this.handleInputChange}
            @input=${this.handleInputInput}
            @blur=${this.stopNestedEventPropagation}
            @focus=${this.stopNestedEventPropagation}
          ></wa-input>

          <wa-button-group>
            ${
              !this.withoutFormatToggle
                ? html`
                    <wa-button
                      part="format-button"
                      size="s"
                      appearance="outlined"
                      aria-label=${this.localize.term("toggleColorFormat")}
                      exportparts="
                      base:format-button__base,
                      start:format-button__start,
                      label:format-button__label,
                      end:format-button__end,
                      caret:format-button__caret
                    "
                      @click=${this.handleFormatToggle}
                      @blur=${this.stopNestedEventPropagation}
                      @focus=${this.stopNestedEventPropagation}
                    >
                      ${this.setLetterCase(this.format)}
                    </wa-button>
                  `
                : ""
            }
            ${
              this.hasEyeDropper
                ? html`
                    <wa-button
                      part="eyedropper-button"
                      size="s"
                      appearance="outlined"
                      exportparts="
                      base:eyedropper-button__base,
                      start:eyedropper-button__start,
                      label:eyedropper-button__label,
                      end:eyedropper-button__end,
                      caret:eyedropper-button__caret
                    "
                      @click=${this.handleEyeDropper}
                      @blur=${this.stopNestedEventPropagation}
                      @focus=${this.stopNestedEventPropagation}
                    >
                      <wa-icon
                        library="system"
                        name="eyedropper"
                        variant="solid"
                        label=${this.localize.term("selectAColorFromTheScreen")}
                      ></wa-icon>
                    </wa-button>
                  `
                : ""
            }
          </wa-button-group>
        </div>

        ${
          normalizedSwatches.length > 0
            ? html`
                <div part="swatches" class="swatches">
                  ${normalizedSwatches.map((swatch) => {
                  const parsedColor = this.parseColor(swatch.color);

                  // If we can't parse it, skip it
                  if (!parsedColor) {
                    return "";
                  }

                  return html`
                    <div
                      part="swatch"
                      class="swatch transparent-bg"
                      tabindex=${ifDefined(this.disabled ? undefined : "0")}
                      role="button"
                      aria-label=${swatch.label}
                      @click=${() => this.selectSwatch(swatch.color)}
                      @keydown=${(event: KeyboardEvent) =>
                        !this.disabled &&
                        event.key === "Enter" &&
                        this.setColor(parsedColor.hexa)}
                    >
                      <div
                        class="swatch-color"
                        style=${styleMap({ backgroundColor: parsedColor.hexa })}
                      ></div>
                    </div>
                  `;
                })}
                </div>
              `
            : ""
        }
      </div>
    `;

    // Render with popup
    return html`
      <div
        class=${classMap({
          container: true,
          "form-control": true,
          "form-control-has-label": hasLabel,
        })}
        part="trigger-container form-control"
      >
        <div
          part="form-control-label"
          class=${classMap({
            label: true,
            "has-label": hasLabel,
          })}
          id="form-control-label"
        >
          <slot name="label">${this.label}</slot>
        </div>

        <button
          id="trigger"
          part="trigger form-control-input"
          class=${classMap({
            trigger: true,
            "trigger-empty": this.isEmpty,
            "transparent-bg": true,
            "form-control-input": true,
          })}
          style=${styleMap({
            color: this.getHexString(
              this.hue,
              this.saturation,
              this.brightness,
              this.alpha,
            ),
          })}
          type="button"
          aria-labelledby="form-control-label"
          aria-describedby="hint"
          .disabled=${this.disabled}
          @click=${this.handleTriggerClick}
          @keydown=${this.handleTriggerKeyDown}
          @keyup=${this.handleTriggerKeyUp}
        ></button>

        <slot
          id="hint"
          name="hint"
          part="hint"
          class=${classMap({
            "has-slotted": hasHint,
          })}
          >${this.hint}</slot
        >
      </div>

      <wa-popup
        class="color-popup"
        anchor="trigger"
        placement=${this.placement}
        distance="0"
        skidding="0"
        flip
        flip-fallback-strategy="best-fit"
        shift
        shift-padding="10"
        aria-disabled=${this.disabled ? "true" : "false"}
        @wa-after-show=${this.handleAfterShow}
        @wa-after-hide=${this.handleAfterHide}
      >
        ${colorPicker}
      </wa-popup>
    `;
  }
}

// The change-in-update warning is required for this component because:
//
// - The base class (WebAwesomeFormAssociatedElement) firstUpdated() calls updateValidity() which triggers
//    requestUpdate('validity').
// - HasSlotController calls host.requestUpdate() on slotchange events.
// - @watch('value') handler sets multiple @state properties (isEmpty, hue, saturation, brightness, alpha, inputValue)
//    and calls syncValues() and requestUpdate() during the update cycle to keep color state in sync.
// - @watch('opacity') and @watch('format') handlers set @state properties during update to synchronize color values.
// - firstUpdated() sets the @state property hasEyeDropper based on browser capability detection.
//
// See https://lit.dev/docs/tools/development/#development-build-runtime-warnings
WaColorPicker.disableWarning?.("change-in-update");

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
@@ -1185,52 +1185,50 @@
                 @keydown=${this.handleHueKeyDown}
               ></span>
             </div>
 
-            ${
-              this.opacity
-                ? html`
+            ${this.opacity
+              ? html`
+                  <div
+                    part="slider opacity-slider"
+                    class="alpha slider transparent-bg"
+                    @pointerdown="${this.handleAlphaDrag}"
+                    @touchmove=${this.handleTouchMove}
+                  >
                     <div
-                      part="slider opacity-slider"
-                      class="alpha slider transparent-bg"
-                      @pointerdown="${this.handleAlphaDrag}"
-                      @touchmove=${this.handleTouchMove}
-                    >
-                      <div
-                        class="alpha-gradient"
-                        style=${styleMap({
+                      class="alpha-gradient"
+                      style=${styleMap({
                         backgroundImage: `linear-gradient(
                           to right,
                           ${this.getHexString(this.hue, this.saturation, this.brightness, 0)} 0%,
                           ${this.getHexString(this.hue, this.saturation, this.brightness, 100)} 100%
                         )`,
                       })}
-                      ></div>
-                      <span
-                        part="slider-handle opacity-slider-handle"
-                        class="slider-handle"
-                        style=${styleMap({
+                    ></div>
+                    <span
+                      part="slider-handle opacity-slider-handle"
+                      class="slider-handle"
+                      style=${styleMap({
                         left: `${this.alpha}%`,
                         backgroundColor: this.getHexString(
                           this.hue,
                           this.saturation,
                           this.brightness,
                           this.alpha,
                         ),
                       })}
-                        role="slider"
-                        aria-label="alpha"
-                        aria-orientation="horizontal"
-                        aria-valuemin="0"
-                        aria-valuemax="100"
-                        aria-valuenow=${Math.round(this.alpha)}
-                        tabindex=${ifDefined(this.disabled ? undefined : "0")}
-                        @keydown=${this.handleAlphaKeyDown}
-                      ></span>
-                    </div>
-                  `
-                : ""
-            }
+                      role="slider"
+                      aria-label="alpha"
+                      aria-orientation="horizontal"
+                      aria-valuemin="0"
+                      aria-valuemax="100"
+                      aria-valuenow=${Math.round(this.alpha)}
+                      tabindex=${ifDefined(this.disabled ? undefined : "0")}
+                      @keydown=${this.handleAlphaKeyDown}
+                    ></span>
+                  </div>
+                `
+              : ""}
           </div>
 
           <button
             type="button"
@@ -1270,68 +1268,63 @@
             @focus=${this.stopNestedEventPropagation}
           ></wa-input>
 
           <wa-button-group>
-            ${
-              !this.withoutFormatToggle
-                ? html`
-                    <wa-button
-                      part="format-button"
-                      size="s"
-                      appearance="outlined"
-                      aria-label=${this.localize.term("toggleColorFormat")}
-                      exportparts="
+            ${!this.withoutFormatToggle
+              ? html`
+                  <wa-button
+                    part="format-button"
+                    size="s"
+                    appearance="outlined"
+                    aria-label=${this.localize.term("toggleColorFormat")}
+                    exportparts="
                       base:format-button__base,
                       start:format-button__start,
                       label:format-button__label,
                       end:format-button__end,
                       caret:format-button__caret
                     "
-                      @click=${this.handleFormatToggle}
-                      @blur=${this.stopNestedEventPropagation}
-                      @focus=${this.stopNestedEventPropagation}
-                    >
-                      ${this.setLetterCase(this.format)}
-                    </wa-button>
-                  `
-                : ""
-            }
-            ${
-              this.hasEyeDropper
-                ? html`
-                    <wa-button
-                      part="eyedropper-button"
-                      size="s"
-                      appearance="outlined"
-                      exportparts="
+                    @click=${this.handleFormatToggle}
+                    @blur=${this.stopNestedEventPropagation}
+                    @focus=${this.stopNestedEventPropagation}
+                  >
+                    ${this.setLetterCase(this.format)}
+                  </wa-button>
+                `
+              : ""}
+            ${this.hasEyeDropper
+              ? html`
+                  <wa-button
+                    part="eyedropper-button"
+                    size="s"
+                    appearance="outlined"
+                    exportparts="
                       base:eyedropper-button__base,
                       start:eyedropper-button__start,
                       label:eyedropper-button__label,
                       end:eyedropper-button__end,
                       caret:eyedropper-button__caret
                     "
-                      @click=${this.handleEyeDropper}
-                      @blur=${this.stopNestedEventPropagation}
-                      @focus=${this.stopNestedEventPropagation}
-                    >
-                      <wa-icon
-                        library="system"
-                        name="eyedropper"
-                        variant="solid"
-                        label=${this.localize.term("selectAColorFromTheScreen")}
-                      ></wa-icon>
-                    </wa-button>
-                  `
-                : ""
-            }
+                    @click=${this.handleEyeDropper}
+                    @blur=${this.stopNestedEventPropagation}
+                    @focus=${this.stopNestedEventPropagation}
+                  >
+                    <wa-icon
+                      library="system"
+                      name="eyedropper"
+                      variant="solid"
+                      label=${this.localize.term("selectAColorFromTheScreen")}
+                    ></wa-icon>
+                  </wa-button>
+                `
+              : ""}
           </wa-button-group>
         </div>
 
-        ${
-          normalizedSwatches.length > 0
-            ? html`
-                <div part="swatches" class="swatches">
-                  ${normalizedSwatches.map((swatch) => {
+        ${normalizedSwatches.length > 0
+          ? html`
+              <div part="swatches" class="swatches">
+                ${normalizedSwatches.map((swatch) => {
                   const parsedColor = this.parseColor(swatch.color);
 
                   // If we can't parse it, skip it
                   if (!parsedColor) {
@@ -1355,12 +1348,11 @@
                       ></div>
                     </div>
                   `;
                 })}
-                </div>
-              `
-            : ""
-        }
+              </div>
+            `
+          : ""}
       </div>
     `;
 
     // Render with popup

`````

### Actual (oxfmt)

`````ts
import { TinyColor } from "@ctrl/tinycolor";
import type { PropertyValues } from "lit";
import { html, isServer } from "lit";
import { customElement, eventOptions, property, query, state } from "lit/decorators.js";
import { classMap } from "lit/directives/class-map.js";
import { ifDefined } from "lit/directives/if-defined.js";
import { styleMap } from "lit/directives/style-map.js";
import { WaInvalidEvent } from "../../events/invalid.js";
import { animateWithClass } from "../../internal/animate.js";
import {
  isTopDismissible,
  registerDismissible,
  unregisterDismissible,
} from "../../internal/dismissible-stack.js";
import { drag } from "../../internal/drag.js";
import { waitForEvent } from "../../internal/event.js";
import { clamp } from "../../internal/math.js";
import { warnDeprecatedSize } from "../../internal/size.js";
import { HasSlotController } from "../../internal/slot.js";
import { RequiredValidator } from "../../internal/validators/required-validator.js";
import { watch } from "../../internal/watch.js";
import { WebAwesomeFormAssociatedElement } from "../../internal/webawesome-form-associated-element.js";
import formControlStyles from "../../styles/component/form-control.styles.js";
import sizeStyles from "../../styles/component/size.styles.js";
import visuallyHidden from "../../styles/component/visually-hidden.styles.js";
import { LocalizeController } from "../../utilities/localize.js";
import "../button-group/button-group.js";
import "../button/button.js";
import "../icon/icon.js";
import "../input/input.js";
import type WaInput from "../input/input.js";
import "../popup/popup.js";
import type WaPopup from "../popup/popup.js";
import styles from "./color-picker.styles.js";

export interface WaColorPickerSwatch {
  color: string;
  label: string;
}

interface EyeDropperConstructor {
  new (): EyeDropperInterface;
}

interface EyeDropperInterface {
  open: () => Promise<{ sRGBHex: string }>;
}

declare const EyeDropper: EyeDropperConstructor;

/**
 * @summary Color pickers let users choose a color from a visual palette or by entering a value. They support HEX, RGB,
 *  HSL, and HSV formats with optional alpha channel and swatch presets.
 * @documentation https://webawesome.com/docs/components/color-picker
 * @status stable
 * @since 2.0
 *
 * @dependency wa-button
 * @dependency wa-button-group
 * @dependency wa-input
 * @dependency wa-popup
 * @dependency wa-visually-hidden
 *
 * @slot label - The color picker's form label. Alternatively, you can use the `label` attribute.
 * @slot hint - The color picker's form hint. Alternatively, you can use the `hint` attribute.
 *
 * @event blur - Emitted when the color picker loses focus.
 * @event change - Emitted when the color picker's value changes.
 * @event focus - Emitted when the color picker receives focus.
 * @event input - Emitted when the color picker receives input.
 * @event wa-invalid - Emitted when the form control has been checked for validity and its constraints aren't satisfied.
 *
 * @csspart base - The component's base wrapper.
 * @csspart trigger - The color picker's dropdown trigger.
 * @csspart swatches - The container that holds the swatches.
 * @csspart swatch - Each individual swatch.
 * @csspart grid - The color grid.
 * @csspart grid-handle - The color grid's handle.
 * @csspart slider - Hue and opacity sliders.
 * @csspart slider-handle - Hue and opacity slider handles.
 * @csspart hue-slider - The hue slider.
 * @csspart hue-slider-handle - The hue slider's handle.
 * @csspart opacity-slider - The opacity slider.
 * @csspart opacity-slider-handle - The opacity slider's handle.
 * @csspart preview - The preview color.
 * @csspart input - The text input.
 * @csspart eyedropper-button - The eye dropper button.
 * @csspart eyedropper-button__base - The eye dropper button's exported `button` part.
 * @csspart eyedropper-button__start - The eye dropper button's exported `start` part.
 * @csspart eyedropper-button__label - The eye dropper button's exported `label` part.
 * @csspart eyedropper-button__end - The eye dropper button's exported `end` part.
 * @csspart eyedropper-button__caret - The eye dropper button's exported `caret` part.
 * @csspart format-button - The format button.
 * @csspart format-button__base - The format button's exported `button` part.
 * @csspart format-button__start - The format button's exported `start` part.
 * @csspart format-button__label - The format button's exported `label` part.
 * @csspart format-button__end - The format button's exported `end` part.
 * @csspart format-button__caret - The format button's exported `caret` part.
 *
 * @cssproperty --grid-width - The width of the color grid.
 * @cssproperty --grid-height - The height of the color grid.
 * @cssproperty --grid-handle-size - The size of the color grid's handle.
 * @cssproperty --slider-height - The height of the hue and alpha sliders.
 * @cssproperty --slider-handle-size - The diameter of the slider's handle.
 */
@customElement("wa-color-picker")
export default class WaColorPicker extends WebAwesomeFormAssociatedElement {
  static css = [visuallyHidden, sizeStyles, formControlStyles, styles];

  static shadowRootOptions = {
    ...WebAwesomeFormAssociatedElement.shadowRootOptions,
    delegatesFocus: true,
  };

  static get validators() {
    const validators = isServer ? [] : [RequiredValidator()];
    return [...super.validators, ...validators];
  }

  private readonly hasSlotController = new HasSlotController(this, "hint", "label");

  private isSafeValue = false;
  private readonly localize = new LocalizeController(this);

  @query('[part~="base"]') base: HTMLElement;
  @query('[part~="input"]') input: WaInput;
  @query('[part~="form-control-label"]') triggerLabel: HTMLElement;
  @query('[part~="form-control-input"]') triggerButton: HTMLButtonElement;

  // @TODO: This is a hacky way to show the "Please fill out this field", do we want the old behavior where it opens the dropdown?
  //   or is the new behavior okay?
  get validationTarget() {
    // This puts the popup on the element only if the color picker is expanded.
    if (this.popup?.active) {
      return this.input;
    }

    // This puts popup on the color picker itself without needing to expand it to show the input.
    // This is necessary because form submissions expect the "anchor" to be currently shown.
    return this.trigger;
  }

  @query(".color-popup") popup: WaPopup;
  @query('[part~="preview"]') previewButton: HTMLButtonElement;
  @query('[part~="trigger"]') trigger: HTMLButtonElement;

  @state() private hasFocus = false;
  @state() private isDraggingGridHandle = false;
  @state() private isEmpty = true;
  @state() private inputValue = "";
  @state() private hue = 0;
  @state() private saturation = 100;
  @state() private brightness = 100;
  @state() private alpha = 100;

  private _value: string | null = null;

  /** The current value of the input, submitted as a name/value pair with form data. */
  get value() {
    if (this.valueHasChanged) {
      return this._value;
    }

    return this._value ?? this.defaultValue;
  }

  /**
   * The current value of the color picker. The value's format will vary based the `format` attribute. To get the value
   * in a specific format, use the `getFormattedValue()` method. The value is submitted as a name/value pair with form
   * data.
   */

  @state() set value(val: string | null) {
    if (this._value === val) {
      return;
    }

    this.valueHasChanged = true;
    this._value = val;
  }

  /** The default value of the form control. Primarily used for resetting the form control. */
  @property({ attribute: "value", reflect: true }) defaultValue: string | null =
    this.getAttribute("value") || null;

  /**
   * Only required for SSR. Set to `true` if you're slotting in a `label` element so the server-rendered markup
   * includes the label before the component hydrates on the client.
   */
  @property({ attribute: "with-label", reflect: true, type: Boolean }) withLabel = false;

  /**
   * Only required for SSR. Set to `true` if you're slotting in a `hint` element so the server-rendered markup
   * includes the hint before the component hydrates on the client.
   */
  @property({ attribute: "with-hint", reflect: true, type: Boolean }) withHint = false;

  @state() private hasEyeDropper: boolean = false;

  /**
   * The color picker's label. This will not be displayed, but it will be announced by assistive devices. If you need to
   * display HTML, you can use the `label` slot` instead.
   */
  @property() label = "";

  /**
   * The color picker's hint. If you need to display HTML, use the `hint` slot instead.
   */
  @property({ attribute: "hint" }) hint = "";

  /**
   * The format to use. If opacity is enabled, these will translate to HEXA, RGBA, HSLA, and HSVA respectively. The color
   * picker will accept user input in any format (including CSS color names) and convert it to the desired format.
   */
  @property() format: "hex" | "rgb" | "hsl" | "hsv" = "hex";

  /** Determines the size of the color picker's trigger */
  @property({ reflect: true }) size: "xs" | "s" | "m" | "l" | "xl" | "small" | "medium" | "large" =
    "m";

  @watch("size")
  handleSizeChange() {
    warnDeprecatedSize(this.localName, this.size);
  }

  /**
   * The preferred placement of the color picker's popup. Note that the actual placement will vary as configured to
   * keep the panel inside of the viewport.
   */
  @property({ reflect: true }) placement:
    | "top"
    | "top-start"
    | "top-end"
    | "bottom"
    | "bottom-start"
    | "bottom-end"
    | "right"
    | "right-start"
    | "right-end"
    | "left"
    | "left-start"
    | "left-end" = "bottom-start";

  /** Removes the button that lets users toggle between format.   */
  @property({ attribute: "without-format-toggle", type: Boolean }) withoutFormatToggle = false;

  /** The name of the form control, submitted as a name/value pair with form data. */
  @property({ reflect: true }) name: string | null = null;

  /** Disables the color picker. */
  @property({ type: Boolean }) disabled = false;

  /**
   * Indicates whether or not the popup is open. You can toggle this attribute to show and hide the popup, or you
   * can use the `show()` and `hide()` methods and this attribute will reflect the popup's open state.
   */
  @property({ type: Boolean, reflect: true }) open = false;

  /** Shows the opacity slider. Enabling this will cause the formatted value to be HEXA, RGBA, or HSLA. */
  @property({ type: Boolean }) opacity = false;

  /** By default, values are lowercase. With this attribute, values will be uppercase instead. */
  @property({ type: Boolean }) uppercase = false;

  /**
   * One or more predefined color swatches to display as presets in the color picker. Can include any format the color
   * picker can parse, including HEX(A), RGB(A), HSL(A), HSV(A), and CSS color names. Each color must be separated by a
   * semicolon (`;`). Alternatively, you can pass an array of color values or an array of `{ color, label }` objects to
   * this property using JavaScript. When using objects with labels, the label will be used for the swatch's accessible
   * name instead of the raw color value.
   */
  @property() swatches: string | string[] | WaColorPickerSwatch[] = "";

  /** Makes the color picker a required field. */
  @property({ type: Boolean, reflect: true }) required = false;

  constructor() {
    super();

    if (!isServer) {
      this.addEventListener("focusin", this.handleFocusIn);
      this.addEventListener("focusout", this.handleFocusOut);
    }
  }

  private handleCopy() {
    this.input.select();
    document.execCommand("copy");
    this.previewButton.focus();

    // Show copied animation
    this.previewButton.classList.add("preview-color-copied");
    this.previewButton.addEventListener("animationend", () => {
      this.previewButton.classList.remove("preview-color-copied");
    });
  }

  private handleFocusIn = () => {
    this.hasFocus = true;
  };

  private handleFocusOut = () => {
    this.hasFocus = false;
  };

  private handleFormatToggle() {
    const formats = ["hex", "rgb", "hsl", "hsv"];
    const nextIndex = (formats.indexOf(this.format) + 1) % formats.length;
    this.format = formats[nextIndex] as "hex" | "rgb" | "hsl" | "hsv";
    this.setColor(this.value || "");

    this.updateComplete.then(() => {
      this.dispatchEvent(new Event("change", { bubbles: true, composed: true }));
      this.dispatchEvent(new InputEvent("input", { bubbles: true, composed: true }));
    });
  }

  private handleAlphaDrag(event: PointerEvent) {
    const container = this.shadowRoot!.querySelector<HTMLElement>(".slider.alpha")!;
    const handle = container.querySelector<HTMLElement>(".slider-handle")!;
    const { width } = container.getBoundingClientRect();
    let initialValue = this.value;
    let currentValue = this.value;

    handle.focus();
    event.preventDefault();

    drag(container, {
      onMove: (x) => {
        this.alpha = clamp((x / width) * 100, 0, 100);
        this.syncValues();

        if (this.value !== currentValue) {
          currentValue = this.value;
          this.updateComplete.then(() => {
            this.dispatchEvent(new InputEvent("input", { bubbles: true, composed: true }));
          });
        }
      },
      onStop: () => {
        if (this.value !== initialValue) {
          initialValue = this.value;

          this.updateComplete.then(() => {
            this.dispatchEvent(new Event("change", { bubbles: true, composed: true }));
          });
        }
      },
      initialEvent: event,
    });
  }

  private handleHueDrag(event: PointerEvent) {
    const container = this.shadowRoot!.querySelector<HTMLElement>(".slider.hue")!;
    const handle = container.querySelector<HTMLElement>(".slider-handle")!;
    const { width } = container.getBoundingClientRect();
    let initialValue = this.value;
    let currentValue = this.value;

    handle.focus();
    event.preventDefault();

    drag(container, {
      onMove: (x) => {
        this.hue = clamp((x / width) * 360, 0, 360);
        this.syncValues();

        if (this.value !== currentValue) {
          currentValue = this.value;
          this.updateComplete.then(() => {
            this.dispatchEvent(new InputEvent("input"));
          });
        }
      },
      onStop: () => {
        if (this.value !== initialValue) {
          initialValue = this.value;
          this.updateComplete.then(() => {
            this.dispatchEvent(new Event("change", { bubbles: true, composed: true }));
          });
        }
      },
      initialEvent: event,
    });
  }

  private handleGridDrag(event: PointerEvent) {
    const grid = this.shadowRoot!.querySelector<HTMLElement>(".grid")!;
    const handle = grid.querySelector<HTMLElement>(".grid-handle")!;
    const { width, height } = grid.getBoundingClientRect();
    let initialValue = this.value;
    let currentValue = this.value;

    handle.focus();
    event.preventDefault();

    this.isDraggingGridHandle = true;

    drag(grid, {
      onMove: (x, y) => {
        this.saturation = clamp((x / width) * 100, 0, 100);
        this.brightness = clamp(100 - (y / height) * 100, 0, 100);
        this.syncValues();

        if (this.value !== currentValue) {
          currentValue = this.value;
          this.updateComplete.then(() => {
            this.dispatchEvent(new InputEvent("input", { bubbles: true, composed: true }));
          });
        }
      },
      onStop: () => {
        this.isDraggingGridHandle = false;
        if (this.value !== initialValue) {
          initialValue = this.value;
          this.updateComplete.then(() => {
            this.dispatchEvent(new Event("change", { bubbles: true, composed: true }));
          });
        }
      },
      initialEvent: event,
    });
  }

  private handleAlphaKeyDown(event: KeyboardEvent) {
    const increment = event.shiftKey ? 10 : 1;
    const oldValue = this.value;

    if (event.key === "ArrowLeft") {
      event.preventDefault();
      this.alpha = clamp(this.alpha - increment, 0, 100);
      this.syncValues();
    }

    if (event.key === "ArrowRight") {
      event.preventDefault();
      this.alpha = clamp(this.alpha + increment, 0, 100);
      this.syncValues();
    }

    if (event.key === "Home") {
      event.preventDefault();
      this.alpha = 0;
      this.syncValues();
    }

    if (event.key === "End") {
      event.preventDefault();
      this.alpha = 100;
      this.syncValues();
    }

    if (this.value !== oldValue) {
      this.updateComplete.then(() => {
        this.dispatchEvent(new InputEvent("input", { bubbles: true, composed: true }));
        this.dispatchEvent(new Event("change", { bubbles: true, composed: true }));
      });
    }
  }

  private handleHueKeyDown(event: KeyboardEvent) {
    const increment = event.shiftKey ? 10 : 1;
    const oldValue = this.value;

    if (event.key === "ArrowLeft") {
      event.preventDefault();
      this.hue = clamp(this.hue - increment, 0, 360);
      this.syncValues();
    }

    if (event.key === "ArrowRight") {
      event.preventDefault();
      this.hue = clamp(this.hue + increment, 0, 360);
      this.syncValues();
    }

    if (event.key === "Home") {
      event.preventDefault();
      this.hue = 0;
      this.syncValues();
    }

    if (event.key === "End") {
      event.preventDefault();
      this.hue = 360;
      this.syncValues();
    }

    if (this.value !== oldValue) {
      this.updateComplete.then(() => {
        this.dispatchEvent(new InputEvent("input", { bubbles: true, composed: true }));
        this.dispatchEvent(new Event("change", { bubbles: true, composed: true }));
      });
    }
  }

  private handleGridKeyDown(event: KeyboardEvent) {
    const increment = event.shiftKey ? 10 : 1;
    const oldValue = this.value;

    if (event.key === "ArrowLeft") {
      event.preventDefault();
      this.saturation = clamp(this.saturation - increment, 0, 100);
      this.syncValues();
    }

    if (event.key === "ArrowRight") {
      event.preventDefault();
      this.saturation = clamp(this.saturation + increment, 0, 100);
      this.syncValues();
    }

    if (event.key === "ArrowUp") {
      event.preventDefault();
      this.brightness = clamp(this.brightness + increment, 0, 100);
      this.syncValues();
    }

    if (event.key === "ArrowDown") {
      event.preventDefault();
      this.brightness = clamp(this.brightness - increment, 0, 100);
      this.syncValues();
    }

    if (this.value !== oldValue) {
      this.updateComplete.then(() => {
        this.dispatchEvent(new InputEvent("input", { bubbles: true, composed: true }));
        this.dispatchEvent(new Event("change", { bubbles: true, composed: true }));
      });
    }
  }

  private handleInputChange(event: Event) {
    const target = event.target as HTMLInputElement;
    const oldValue = this.value;

    // Prevent the `<wa-input>` element's `change` event from bubbling up
    event.stopPropagation();

    if (this.input.value) {
      this.setColor(target.value);
      target.value = this.value || "";
    } else {
      this.value = "";
    }

    if (this.value !== oldValue) {
      this.updateComplete.then(() => {
        this.dispatchEvent(new InputEvent("input", { bubbles: true, composed: true }));
        this.dispatchEvent(new Event("change", { bubbles: true, composed: true }));
      });
    }
  }

  private handleInputInput(event: InputEvent) {
    this.updateValidity();

    // Prevent the `<wa-input>` element's `input` event from bubbling up
    event.stopPropagation();
  }

  private handleInputKeyDown(event: KeyboardEvent) {
    if (event.key === "Enter") {
      const oldValue = this.value;

      if (this.input.value) {
        this.setColor(this.input.value);
        this.input.value = this.value;

        if (this.value !== oldValue) {
          this.updateComplete.then(() => {
            this.dispatchEvent(new InputEvent("input", { bubbles: true, composed: true }));
            this.dispatchEvent(new Event("change", { bubbles: true, composed: true }));
          });
        }

        setTimeout(() => this.input.select());
      } else {
        this.hue = 0;
      }
    }
  }

  @eventOptions({ passive: false })
  private handleTouchMove(event: TouchEvent) {
    event.preventDefault();
  }

  private parseColor(colorString: string) {
    if (!colorString || colorString.trim() === "") {
      return null;
    }

    const color = new TinyColor(colorString);
    if (!color.isValid) {
      return null;
    }

    const hslColor = color.toHsl();
    const rgb = color.toRgb();
    const hsvColor = color.toHsv();

    // Checks for null RGB values
    if (!rgb || rgb.r == null || rgb.g == null || rgb.b == null) {
      return null;
    }

    // Adjust saturation and lightness from 0-1 to 0-100
    const hsl = {
      h: hslColor.h || 0,
      s: (hslColor.s || 0) * 100,
      l: (hslColor.l || 0) * 100,
      a: hslColor.a || 0,
    };

    const hex = color.toHexString();
    const hexa = color.toHex8String();

    // Adjust saturation and value from 0-1 to 0-100
    const hsv = {
      h: hsvColor.h || 0,
      s: (hsvColor.s || 0) * 100,
      v: (hsvColor.v || 0) * 100,
      a: hsvColor.a || 0,
    };

    return {
      hsl: {
        h: hsl.h,
        s: hsl.s,
        l: hsl.l,
        string: this.setLetterCase(
          `hsl(${Math.round(hsl.h)}, ${Math.round(hsl.s)}%, ${Math.round(hsl.l)}%)`,
        ),
      },
      hsla: {
        h: hsl.h,
        s: hsl.s,
        l: hsl.l,
        a: hsl.a,
        string: this.setLetterCase(
          `hsla(${Math.round(hsl.h)}, ${Math.round(hsl.s)}%, ${Math.round(hsl.l)}%, ${hsl.a.toFixed(2).toString()})`,
        ),
      },
      hsv: {
        h: hsv.h,
        s: hsv.s,
        v: hsv.v,
        string: this.setLetterCase(
          `hsv(${Math.round(hsv.h)}, ${Math.round(hsv.s)}%, ${Math.round(hsv.v)}%)`,
        ),
      },
      hsva: {
        h: hsv.h,
        s: hsv.s,
        v: hsv.v,
        a: hsv.a,
        string: this.setLetterCase(
          `hsva(${Math.round(hsv.h)}, ${Math.round(hsv.s)}%, ${Math.round(hsv.v)}%, ${hsv.a.toFixed(2).toString()})`,
        ),
      },
      rgb: {
        r: rgb.r,
        g: rgb.g,
        b: rgb.b,
        string: this.setLetterCase(
          `rgb(${Math.round(rgb.r)}, ${Math.round(rgb.g)}, ${Math.round(rgb.b)})`,
        ),
      },
      rgba: {
        r: rgb.r,
        g: rgb.g,
        b: rgb.b,
        a: rgb.a || 0,
        string: this.setLetterCase(
          `rgba(${Math.round(rgb.r)}, ${Math.round(rgb.g)}, ${Math.round(rgb.b)}, ${(rgb.a || 0).toFixed(2).toString()})`,
        ),
      },
      hex: this.setLetterCase(hex),
      hexa: this.setLetterCase(hexa),
    };
  }

  private setColor(colorString: string) {
    const newColor = this.parseColor(colorString);

    if (newColor === null) {
      return false;
    }

    this.hue = newColor.hsva.h;
    this.saturation = newColor.hsva.s;
    this.brightness = newColor.hsva.v;
    this.alpha = this.opacity ? newColor.hsva.a * 100 : 100;

    this.syncValues();

    return true;
  }

  private setLetterCase(string: string) {
    if (typeof string !== "string") {
      return "";
    }
    return this.uppercase ? string.toUpperCase() : string.toLowerCase();
  }

  private async syncValues() {
    const currentColor = this.parseColor(
      `hsva(${this.hue}, ${this.saturation}%, ${this.brightness}%, ${this.alpha / 100})`,
    );

    if (currentColor === null) {
      return;
    }

    // Update the value
    if (this.format === "hsl") {
      this.inputValue = this.opacity ? currentColor.hsla.string : currentColor.hsl.string;
    } else if (this.format === "rgb") {
      this.inputValue = this.opacity ? currentColor.rgba.string : currentColor.rgb.string;
    } else if (this.format === "hsv") {
      this.inputValue = this.opacity ? currentColor.hsva.string : currentColor.hsv.string;
    } else {
      this.inputValue = this.opacity ? currentColor.hexa : currentColor.hex;
    }

    // Setting this.value will trigger the watcher which parses the new value. We want to bypass that behavior because
    // we've already parsed the color here and conversion/rounding can lead to values changing slightly. When this
    // happens, dragging the grid handle becomes jumpy. After the next update, the usual behavior is restored.
    this.isSafeValue = true;
    this.value = this.inputValue;

    await this.updateComplete;
    this.isSafeValue = false;
  }

  private handleAfterHide() {
    this.previewButton.classList.remove("preview-color-copied");
    // Update validity so we get a new anchor.
    this.updateValidity();
  }

  private handleAfterShow() {
    // Update validity so we get a new anchor.
    this.updateValidity();
  }

  private handleEyeDropper() {
    if (!this.hasEyeDropper) {
      return;
    }

    const eyeDropper = new EyeDropper();

    eyeDropper
      .open()
      .then((colorSelectionResult) => {
        const oldValue = this.value;

        this.setColor(colorSelectionResult.sRGBHex);

        if (this.value !== oldValue) {
          this.updateComplete.then(() => {
            this.dispatchEvent(new InputEvent("input", { bubbles: true, composed: true }));
            this.dispatchEvent(new Event("change", { bubbles: true, composed: true }));
          });
        }
      })
      .catch(() => {
        // The user canceled, do nothing
      });
  }

  private selectSwatch(color: string) {
    const oldValue = this.value;

    if (!this.disabled) {
      this.setColor(color);

      if (this.value !== oldValue) {
        this.updateComplete.then(() => {
          this.dispatchEvent(new InputEvent("input", { bubbles: true, composed: true }));
          this.dispatchEvent(new Event("change", { bubbles: true, composed: true }));
        });
      }
    }
  }

  /** Generates a hex string from HSV values. Hue must be 0-360. All other arguments must be 0-100. */
  getHexString(hue: number, saturation: number, brightness: number, alpha = 100) {
    const color = new TinyColor(`hsva(${hue}, ${saturation}%, ${brightness}%, ${alpha / 100})`);
    if (!color.isValid) {
      return "";
    }

    return color.toHex8String();
  }

  // Prevents nested components from leaking events
  private stopNestedEventPropagation(event: CustomEvent) {
    event.stopImmediatePropagation();
  }

  @watch("format", { waitUntilFirstUpdate: true })
  handleFormatChange() {
    this.syncValues();
  }

  @watch("opacity")
  handleOpacityChange() {
    this.alpha = 100;
  }

  protected willUpdate(changedProperties: PropertyValues<this>): void {
    super.willUpdate(changedProperties);

    // Its kind of bizarre, but this is required to get SSR to play nicely.
    if (changedProperties.has("value")) {
      this.handleValueChange(changedProperties.get("value") || "", this.value || "");
    }
  }

  @watch("value")
  handleValueChange(oldValue: string | undefined, newValue: string) {
    this.isEmpty = !newValue;

    if (!newValue) {
      this.hue = 0;
      this.saturation = 0;
      this.brightness = 100;
      this.alpha = 100;
    }

    if (!this.isSafeValue) {
      const newColor = this.parseColor(newValue);

      if (newColor !== null) {
        this.inputValue = this.value || "";
        this.hue = newColor.hsva.h;
        this.saturation = newColor.hsva.s;
        this.brightness = newColor.hsva.v;
        this.alpha = newColor.hsva.a * 100;
        this.syncValues();
      } else {
        this.inputValue = oldValue ?? "";
      }
    }

    this.requestUpdate();
  }

  /** Sets focus on the color picker. */
  focus(options?: FocusOptions) {
    this.trigger.focus(options);
  }

  /** Removes focus from the color picker. */
  blur() {
    const elementToBlur = this.trigger;

    if (this.hasFocus) {
      // We don't know which element in the color picker has focus, so we'll move it to the trigger or base (inline) and
      // blur that instead. This results in document.activeElement becoming the `<body>`. This doesn't cause another
      // focus event because we're using focusin and something inside the color picker already has focus.
      elementToBlur.focus({ preventScroll: true });
      elementToBlur.blur();
    }

    if (this.popup?.active) {
      this.hide();
    }
  }

  /** Returns the current value as a string in the specified format. */
  getFormattedValue(
    format: "hex" | "hexa" | "rgb" | "rgba" | "hsl" | "hsla" | "hsv" | "hsva" = "hex",
  ) {
    const currentColor = this.parseColor(
      `hsva(${this.hue}, ${this.saturation}%, ${this.brightness}%, ${this.alpha / 100})`,
    );

    if (currentColor === null) {
      return "";
    }

    switch (format) {
      case "hex":
        return currentColor.hex;
      case "hexa":
        return currentColor.hexa;
      case "rgb":
        return currentColor.rgb.string;
      case "rgba":
        return currentColor.rgba.string;
      case "hsl":
        return currentColor.hsl.string;
      case "hsla":
        return currentColor.hsla.string;
      case "hsv":
        return currentColor.hsv.string;
      case "hsva":
        return currentColor.hsva.string;
      default:
        return "";
    }
  }

  private reportValidityAfterShow = () => {
    // Remove the event so we don't emit "wa-invalid" twice
    this.removeEventListener("invalid", this.emitInvalid);

    this.reportValidity();

    this.addEventListener("invalid", this.emitInvalid);
  };

  /** Checks for validity and shows the browser's validation message if the control is invalid. */
  reportValidity() {
    // This won't get called when a form is submitted. This is only for manual calls.
    if (!this.validity.valid && !this.open) {
      // Show the popup so the browser can focus on it
      this.addEventListener("wa-after-show", this.reportValidityAfterShow, { once: true });
      this.show();

      if (!this.disabled) {
        // By standards we have to emit a `wa-invalid` event here synchronously.
        this.dispatchEvent(new WaInvalidEvent());
      }

      return false;
    }

    return super.reportValidity();
  }

  formResetCallback() {
    this.value = this.defaultValue;

    super.formResetCallback();
  }

  firstUpdated(changedProperties: PropertyValues<this>): void {
    super.firstUpdated(changedProperties);

    this.hasEyeDropper = "EyeDropper" in window;
  }

  private handleKeyDown = (event: KeyboardEvent) => {
    // Close when escape is pressed inside an open popup. We need to listen on the panel itself and stop propagation
    // in case any ancestors are also listening for this key.
    if (this.open && event.key === "Escape" && isTopDismissible(this)) {
      event.stopPropagation();
      this.hide();
      this.focus();
    }
  };

  private handleDocumentKeyDown = (event: KeyboardEvent) => {
    // Close when escape or tab is pressed
    if (event.key === "Escape" && this.open && isTopDismissible(this)) {
      event.stopPropagation();
      this.focus();
      this.hide();
      return;
    }

    // Handle tabbing
    if (event.key === "Tab") {
      // Tabbing outside of the containing element closes the panel
      //
      // If the popup is used within a shadow DOM, we need to obtain the activeElement within that shadowRoot,
      // otherwise `document.activeElement` will only return the name of the parent shadow DOM element.
      setTimeout(() => {
        const activeElement =
          this.getRootNode() instanceof ShadowRoot
            ? document.activeElement?.shadowRoot?.activeElement
            : document.activeElement;

        if (!this || activeElement?.closest(this.tagName.toLowerCase()) !== this) {
          this.hide();
        }
      });
    }
  };

  private handleDocumentMouseDown = (event: MouseEvent) => {
    // Close when clicking outside of the popup panel and trigger
    const path = event.composedPath();

    // Check if click is inside the popup panel or the trigger element specifically
    const isInsideRelevantArea = path.some(
      (element) =>
        element instanceof Element &&
        (element.closest(".color-picker") || element === this.trigger),
    );

    if (this && !isInsideRelevantArea) {
      this.hide();
    }
  };

  handleTriggerClick() {
    if (this.open) {
      this.hide();
    } else {
      this.show();
      this.focus();
    }
  }

  async handleTriggerKeyDown(event: KeyboardEvent) {
    // When spacebar/enter is pressed, show the panel but don't focus on the menu. This let's the user press the same
    // key again to hide the menu in case they don't want to make a selection.
    if ([" ", "Enter"].includes(event.key)) {
      event.preventDefault();
      this.handleTriggerClick();
      return;
    }
  }

  handleTriggerKeyUp(event: KeyboardEvent) {
    // Prevent space from triggering a click event in Firefox
    if (event.key === " ") {
      event.preventDefault();
    }
  }

  updateAccessibleTrigger() {
    const accessibleTrigger = this.trigger;

    if (accessibleTrigger) {
      accessibleTrigger.setAttribute("aria-haspopup", "true");
      accessibleTrigger.setAttribute("aria-expanded", this.open ? "true" : "false");
    }
  }

  /** Shows the color picker panel. */
  async show() {
    if (this.open) {
      return undefined;
    }

    this.open = true;
    return waitForEvent(this, "wa-after-show");
  }

  /** Hides the color picker panel */
  async hide() {
    if (!this.open) {
      return undefined;
    }

    this.open = false;
    return waitForEvent(this, "wa-after-hide");
  }

  addOpenListeners() {
    this.base.addEventListener("keydown", this.handleKeyDown);
    document.addEventListener("keydown", this.handleDocumentKeyDown);
    document.addEventListener("mousedown", this.handleDocumentMouseDown);
    registerDismissible(this);
  }

  removeOpenListeners() {
    if (this.base) {
      this.base.removeEventListener("keydown", this.handleKeyDown);
    }
    document.removeEventListener("keydown", this.handleDocumentKeyDown);
    document.removeEventListener("mousedown", this.handleDocumentMouseDown);
    unregisterDismissible(this);
  }

  @watch("open", { waitUntilFirstUpdate: true })
  async handleOpenChange() {
    if (this.disabled) {
      this.open = false;
      return;
    }

    this.updateAccessibleTrigger();

    if (this.open) {
      // Show
      this.dispatchEvent(new CustomEvent("wa-show"));

      this.addOpenListeners();
      await this.updateComplete;
      this.base.hidden = false;
      this.popup.active = true;
      await animateWithClass(this.popup.popup, "show-with-scale");
      this.dispatchEvent(new CustomEvent("wa-after-show"));
    } else {
      // Hide
      this.dispatchEvent(new CustomEvent("wa-hide"));

      this.removeOpenListeners();
      await animateWithClass(this.popup.popup, "hide-with-scale");
      this.base.hidden = true;
      this.popup.active = false;
      this.dispatchEvent(new CustomEvent("wa-after-hide"));
    }
  }

  render() {
    const hasLabelSlot = !this.hasUpdated
      ? this.withLabel
      : this.withLabel || this.hasSlotController.test("label");
    const hasHintSlot = !this.hasUpdated
      ? this.withHint
      : this.withHint || this.hasSlotController.test("hint");
    const hasLabel = this.label ? true : !!hasLabelSlot;
    const hasHint = this.hint ? true : !!hasHintSlot;

    const gridHandleX = this.saturation;
    const gridHandleY = 100 - this.brightness;
    const normalizedSwatches: WaColorPickerSwatch[] = Array.isArray(this.swatches)
      ? this.swatches.map((s) => (typeof s === "string" ? { color: s, label: s } : s))
      : this.swatches
          .split(";")
          .filter((color) => color.trim() !== "")
          .map((color) => ({ color: color.trim(), label: color.trim() }));

    const colorPicker = html`
      <div
        part="base"
        class=${classMap({
          "color-picker": true,
        })}
        aria-disabled=${this.disabled ? "true" : "false"}
        tabindex="-1"
      >
        <div
          part="grid"
          class="grid"
          style=${styleMap({ backgroundColor: this.getHexString(this.hue, 100, 100) })}
          @pointerdown=${this.handleGridDrag}
          @touchmove=${this.handleTouchMove}
        >
          <span
            part="grid-handle"
            class=${classMap({
              "grid-handle": true,
              "grid-handle-dragging": this.isDraggingGridHandle,
            })}
            style=${styleMap({
              top: `${gridHandleY}%`,
              left: `${gridHandleX}%`,
              backgroundColor: this.getHexString(
                this.hue,
                this.saturation,
                this.brightness,
                this.alpha,
              ),
            })}
            role="application"
            aria-label="HSV"
            tabindex=${ifDefined(this.disabled ? undefined : "0")}
            @keydown=${this.handleGridKeyDown}
          ></span>
        </div>

        <div class="controls">
          <div class="sliders">
            <div
              part="slider hue-slider"
              class="hue slider"
              @pointerdown=${this.handleHueDrag}
              @touchmove=${this.handleTouchMove}
            >
              <span
                part="slider-handle hue-slider-handle"
                class="slider-handle"
                style=${styleMap({
                  left: `${this.hue === 0 ? 0 : 100 / (360 / this.hue)}%`,
                  backgroundColor: this.getHexString(this.hue, 100, 100),
                })}
                role="slider"
                aria-label="hue"
                aria-orientation="horizontal"
                aria-valuemin="0"
                aria-valuemax="360"
                aria-valuenow=${`${Math.round(this.hue)}`}
                tabindex=${ifDefined(this.disabled ? undefined : "0")}
                @keydown=${this.handleHueKeyDown}
              ></span>
            </div>

            ${this.opacity
              ? html`
                  <div
                    part="slider opacity-slider"
                    class="alpha slider transparent-bg"
                    @pointerdown="${this.handleAlphaDrag}"
                    @touchmove=${this.handleTouchMove}
                  >
                    <div
                      class="alpha-gradient"
                      style=${styleMap({
                        backgroundImage: `linear-gradient(
                          to right,
                          ${this.getHexString(this.hue, this.saturation, this.brightness, 0)} 0%,
                          ${this.getHexString(this.hue, this.saturation, this.brightness, 100)} 100%
                        )`,
                      })}
                    ></div>
                    <span
                      part="slider-handle opacity-slider-handle"
                      class="slider-handle"
                      style=${styleMap({
                        left: `${this.alpha}%`,
                        backgroundColor: this.getHexString(
                          this.hue,
                          this.saturation,
                          this.brightness,
                          this.alpha,
                        ),
                      })}
                      role="slider"
                      aria-label="alpha"
                      aria-orientation="horizontal"
                      aria-valuemin="0"
                      aria-valuemax="100"
                      aria-valuenow=${Math.round(this.alpha)}
                      tabindex=${ifDefined(this.disabled ? undefined : "0")}
                      @keydown=${this.handleAlphaKeyDown}
                    ></span>
                  </div>
                `
              : ""}
          </div>

          <button
            type="button"
            part="preview"
            class="preview transparent-bg"
            aria-label=${this.localize.term("copy")}
            style=${styleMap({
              "--preview-color": this.getHexString(
                this.hue,
                this.saturation,
                this.brightness,
                this.alpha,
              ),
            })}
            @click=${this.handleCopy}
          ></button>
        </div>

        <div class="user-input" aria-live="polite">
          <wa-input
            part="input"
            type="text"
            name=${this.name}
            size="s"
            autocomplete="off"
            autocorrect="off"
            autocapitalize="off"
            spellcheck="false"
            .value=${this.isEmpty ? "" : this.inputValue}
            ?required=${this.required}
            ?disabled=${this.disabled}
            aria-label=${this.localize.term("currentValue")}
            @keydown=${this.handleInputKeyDown}
            @change=${this.handleInputChange}
            @input=${this.handleInputInput}
            @blur=${this.stopNestedEventPropagation}
            @focus=${this.stopNestedEventPropagation}
          ></wa-input>

          <wa-button-group>
            ${!this.withoutFormatToggle
              ? html`
                  <wa-button
                    part="format-button"
                    size="s"
                    appearance="outlined"
                    aria-label=${this.localize.term("toggleColorFormat")}
                    exportparts="
                      base:format-button__base,
                      start:format-button__start,
                      label:format-button__label,
                      end:format-button__end,
                      caret:format-button__caret
                    "
                    @click=${this.handleFormatToggle}
                    @blur=${this.stopNestedEventPropagation}
                    @focus=${this.stopNestedEventPropagation}
                  >
                    ${this.setLetterCase(this.format)}
                  </wa-button>
                `
              : ""}
            ${this.hasEyeDropper
              ? html`
                  <wa-button
                    part="eyedropper-button"
                    size="s"
                    appearance="outlined"
                    exportparts="
                      base:eyedropper-button__base,
                      start:eyedropper-button__start,
                      label:eyedropper-button__label,
                      end:eyedropper-button__end,
                      caret:eyedropper-button__caret
                    "
                    @click=${this.handleEyeDropper}
                    @blur=${this.stopNestedEventPropagation}
                    @focus=${this.stopNestedEventPropagation}
                  >
                    <wa-icon
                      library="system"
                      name="eyedropper"
                      variant="solid"
                      label=${this.localize.term("selectAColorFromTheScreen")}
                    ></wa-icon>
                  </wa-button>
                `
              : ""}
          </wa-button-group>
        </div>

        ${normalizedSwatches.length > 0
          ? html`
              <div part="swatches" class="swatches">
                ${normalizedSwatches.map((swatch) => {
                  const parsedColor = this.parseColor(swatch.color);

                  // If we can't parse it, skip it
                  if (!parsedColor) {
                    return "";
                  }

                  return html`
                    <div
                      part="swatch"
                      class="swatch transparent-bg"
                      tabindex=${ifDefined(this.disabled ? undefined : "0")}
                      role="button"
                      aria-label=${swatch.label}
                      @click=${() => this.selectSwatch(swatch.color)}
                      @keydown=${(event: KeyboardEvent) =>
                        !this.disabled && event.key === "Enter" && this.setColor(parsedColor.hexa)}
                    >
                      <div
                        class="swatch-color"
                        style=${styleMap({ backgroundColor: parsedColor.hexa })}
                      ></div>
                    </div>
                  `;
                })}
              </div>
            `
          : ""}
      </div>
    `;

    // Render with popup
    return html`
      <div
        class=${classMap({
          container: true,
          "form-control": true,
          "form-control-has-label": hasLabel,
        })}
        part="trigger-container form-control"
      >
        <div
          part="form-control-label"
          class=${classMap({
            label: true,
            "has-label": hasLabel,
          })}
          id="form-control-label"
        >
          <slot name="label">${this.label}</slot>
        </div>

        <button
          id="trigger"
          part="trigger form-control-input"
          class=${classMap({
            trigger: true,
            "trigger-empty": this.isEmpty,
            "transparent-bg": true,
            "form-control-input": true,
          })}
          style=${styleMap({
            color: this.getHexString(this.hue, this.saturation, this.brightness, this.alpha),
          })}
          type="button"
          aria-labelledby="form-control-label"
          aria-describedby="hint"
          .disabled=${this.disabled}
          @click=${this.handleTriggerClick}
          @keydown=${this.handleTriggerKeyDown}
          @keyup=${this.handleTriggerKeyUp}
        ></button>

        <slot
          id="hint"
          name="hint"
          part="hint"
          class=${classMap({
            "has-slotted": hasHint,
          })}
        >
          ${this.hint}
        </slot>
      </div>

      <wa-popup
        class="color-popup"
        anchor="trigger"
        placement=${this.placement}
        distance="0"
        skidding="0"
        flip
        flip-fallback-strategy="best-fit"
        shift
        shift-padding="10"
        aria-disabled=${this.disabled ? "true" : "false"}
        @wa-after-show=${this.handleAfterShow}
        @wa-after-hide=${this.handleAfterHide}
      >
        ${colorPicker}
      </wa-popup>
    `;
  }
}

// The change-in-update warning is required for this component because:
//
// - The base class (WebAwesomeFormAssociatedElement) firstUpdated() calls updateValidity() which triggers
//    requestUpdate('validity').
// - HasSlotController calls host.requestUpdate() on slotchange events.
// - @watch('value') handler sets multiple @state properties (isEmpty, hue, saturation, brightness, alpha, inputValue)
//    and calls syncValues() and requestUpdate() during the update cycle to keep color state in sync.
// - @watch('opacity') and @watch('format') handlers set @state properties during update to synchronize color values.
// - firstUpdated() sets the @state property hasEyeDropper based on browser capability detection.
//
// See https://lit.dev/docs/tools/development/#development-build-runtime-warnings
WaColorPicker.disableWarning?.("change-in-update");

`````

### Expected (prettier)

`````ts
import { TinyColor } from "@ctrl/tinycolor";
import type { PropertyValues } from "lit";
import { html, isServer } from "lit";
import { customElement, eventOptions, property, query, state } from "lit/decorators.js";
import { classMap } from "lit/directives/class-map.js";
import { ifDefined } from "lit/directives/if-defined.js";
import { styleMap } from "lit/directives/style-map.js";
import { WaInvalidEvent } from "../../events/invalid.js";
import { animateWithClass } from "../../internal/animate.js";
import {
  isTopDismissible,
  registerDismissible,
  unregisterDismissible,
} from "../../internal/dismissible-stack.js";
import { drag } from "../../internal/drag.js";
import { waitForEvent } from "../../internal/event.js";
import { clamp } from "../../internal/math.js";
import { warnDeprecatedSize } from "../../internal/size.js";
import { HasSlotController } from "../../internal/slot.js";
import { RequiredValidator } from "../../internal/validators/required-validator.js";
import { watch } from "../../internal/watch.js";
import { WebAwesomeFormAssociatedElement } from "../../internal/webawesome-form-associated-element.js";
import formControlStyles from "../../styles/component/form-control.styles.js";
import sizeStyles from "../../styles/component/size.styles.js";
import visuallyHidden from "../../styles/component/visually-hidden.styles.js";
import { LocalizeController } from "../../utilities/localize.js";
import "../button-group/button-group.js";
import "../button/button.js";
import "../icon/icon.js";
import "../input/input.js";
import type WaInput from "../input/input.js";
import "../popup/popup.js";
import type WaPopup from "../popup/popup.js";
import styles from "./color-picker.styles.js";

export interface WaColorPickerSwatch {
  color: string;
  label: string;
}

interface EyeDropperConstructor {
  new (): EyeDropperInterface;
}

interface EyeDropperInterface {
  open: () => Promise<{ sRGBHex: string }>;
}

declare const EyeDropper: EyeDropperConstructor;

/**
 * @summary Color pickers let users choose a color from a visual palette or by entering a value. They support HEX, RGB,
 *  HSL, and HSV formats with optional alpha channel and swatch presets.
 * @documentation https://webawesome.com/docs/components/color-picker
 * @status stable
 * @since 2.0
 *
 * @dependency wa-button
 * @dependency wa-button-group
 * @dependency wa-input
 * @dependency wa-popup
 * @dependency wa-visually-hidden
 *
 * @slot label - The color picker's form label. Alternatively, you can use the `label` attribute.
 * @slot hint - The color picker's form hint. Alternatively, you can use the `hint` attribute.
 *
 * @event blur - Emitted when the color picker loses focus.
 * @event change - Emitted when the color picker's value changes.
 * @event focus - Emitted when the color picker receives focus.
 * @event input - Emitted when the color picker receives input.
 * @event wa-invalid - Emitted when the form control has been checked for validity and its constraints aren't satisfied.
 *
 * @csspart base - The component's base wrapper.
 * @csspart trigger - The color picker's dropdown trigger.
 * @csspart swatches - The container that holds the swatches.
 * @csspart swatch - Each individual swatch.
 * @csspart grid - The color grid.
 * @csspart grid-handle - The color grid's handle.
 * @csspart slider - Hue and opacity sliders.
 * @csspart slider-handle - Hue and opacity slider handles.
 * @csspart hue-slider - The hue slider.
 * @csspart hue-slider-handle - The hue slider's handle.
 * @csspart opacity-slider - The opacity slider.
 * @csspart opacity-slider-handle - The opacity slider's handle.
 * @csspart preview - The preview color.
 * @csspart input - The text input.
 * @csspart eyedropper-button - The eye dropper button.
 * @csspart eyedropper-button__base - The eye dropper button's exported `button` part.
 * @csspart eyedropper-button__start - The eye dropper button's exported `start` part.
 * @csspart eyedropper-button__label - The eye dropper button's exported `label` part.
 * @csspart eyedropper-button__end - The eye dropper button's exported `end` part.
 * @csspart eyedropper-button__caret - The eye dropper button's exported `caret` part.
 * @csspart format-button - The format button.
 * @csspart format-button__base - The format button's exported `button` part.
 * @csspart format-button__start - The format button's exported `start` part.
 * @csspart format-button__label - The format button's exported `label` part.
 * @csspart format-button__end - The format button's exported `end` part.
 * @csspart format-button__caret - The format button's exported `caret` part.
 *
 * @cssproperty --grid-width - The width of the color grid.
 * @cssproperty --grid-height - The height of the color grid.
 * @cssproperty --grid-handle-size - The size of the color grid's handle.
 * @cssproperty --slider-height - The height of the hue and alpha sliders.
 * @cssproperty --slider-handle-size - The diameter of the slider's handle.
 */
@customElement("wa-color-picker")
export default class WaColorPicker extends WebAwesomeFormAssociatedElement {
  static css = [visuallyHidden, sizeStyles, formControlStyles, styles];

  static shadowRootOptions = {
    ...WebAwesomeFormAssociatedElement.shadowRootOptions,
    delegatesFocus: true,
  };

  static get validators() {
    const validators = isServer ? [] : [RequiredValidator()];
    return [...super.validators, ...validators];
  }

  private readonly hasSlotController = new HasSlotController(this, "hint", "label");

  private isSafeValue = false;
  private readonly localize = new LocalizeController(this);

  @query('[part~="base"]') base: HTMLElement;
  @query('[part~="input"]') input: WaInput;
  @query('[part~="form-control-label"]') triggerLabel: HTMLElement;
  @query('[part~="form-control-input"]') triggerButton: HTMLButtonElement;

  // @TODO: This is a hacky way to show the "Please fill out this field", do we want the old behavior where it opens the dropdown?
  //   or is the new behavior okay?
  get validationTarget() {
    // This puts the popup on the element only if the color picker is expanded.
    if (this.popup?.active) {
      return this.input;
    }

    // This puts popup on the color picker itself without needing to expand it to show the input.
    // This is necessary because form submissions expect the "anchor" to be currently shown.
    return this.trigger;
  }

  @query(".color-popup") popup: WaPopup;
  @query('[part~="preview"]') previewButton: HTMLButtonElement;
  @query('[part~="trigger"]') trigger: HTMLButtonElement;

  @state() private hasFocus = false;
  @state() private isDraggingGridHandle = false;
  @state() private isEmpty = true;
  @state() private inputValue = "";
  @state() private hue = 0;
  @state() private saturation = 100;
  @state() private brightness = 100;
  @state() private alpha = 100;

  private _value: string | null = null;

  /** The current value of the input, submitted as a name/value pair with form data. */
  get value() {
    if (this.valueHasChanged) {
      return this._value;
    }

    return this._value ?? this.defaultValue;
  }

  /**
   * The current value of the color picker. The value's format will vary based the `format` attribute. To get the value
   * in a specific format, use the `getFormattedValue()` method. The value is submitted as a name/value pair with form
   * data.
   */

  @state() set value(val: string | null) {
    if (this._value === val) {
      return;
    }

    this.valueHasChanged = true;
    this._value = val;
  }

  /** The default value of the form control. Primarily used for resetting the form control. */
  @property({ attribute: "value", reflect: true }) defaultValue: string | null =
    this.getAttribute("value") || null;

  /**
   * Only required for SSR. Set to `true` if you're slotting in a `label` element so the server-rendered markup
   * includes the label before the component hydrates on the client.
   */
  @property({ attribute: "with-label", reflect: true, type: Boolean }) withLabel = false;

  /**
   * Only required for SSR. Set to `true` if you're slotting in a `hint` element so the server-rendered markup
   * includes the hint before the component hydrates on the client.
   */
  @property({ attribute: "with-hint", reflect: true, type: Boolean }) withHint = false;

  @state() private hasEyeDropper: boolean = false;

  /**
   * The color picker's label. This will not be displayed, but it will be announced by assistive devices. If you need to
   * display HTML, you can use the `label` slot` instead.
   */
  @property() label = "";

  /**
   * The color picker's hint. If you need to display HTML, use the `hint` slot instead.
   */
  @property({ attribute: "hint" }) hint = "";

  /**
   * The format to use. If opacity is enabled, these will translate to HEXA, RGBA, HSLA, and HSVA respectively. The color
   * picker will accept user input in any format (including CSS color names) and convert it to the desired format.
   */
  @property() format: "hex" | "rgb" | "hsl" | "hsv" = "hex";

  /** Determines the size of the color picker's trigger */
  @property({ reflect: true }) size: "xs" | "s" | "m" | "l" | "xl" | "small" | "medium" | "large" =
    "m";

  @watch("size")
  handleSizeChange() {
    warnDeprecatedSize(this.localName, this.size);
  }

  /**
   * The preferred placement of the color picker's popup. Note that the actual placement will vary as configured to
   * keep the panel inside of the viewport.
   */
  @property({ reflect: true }) placement:
    | "top"
    | "top-start"
    | "top-end"
    | "bottom"
    | "bottom-start"
    | "bottom-end"
    | "right"
    | "right-start"
    | "right-end"
    | "left"
    | "left-start"
    | "left-end" = "bottom-start";

  /** Removes the button that lets users toggle between format.   */
  @property({ attribute: "without-format-toggle", type: Boolean }) withoutFormatToggle = false;

  /** The name of the form control, submitted as a name/value pair with form data. */
  @property({ reflect: true }) name: string | null = null;

  /** Disables the color picker. */
  @property({ type: Boolean }) disabled = false;

  /**
   * Indicates whether or not the popup is open. You can toggle this attribute to show and hide the popup, or you
   * can use the `show()` and `hide()` methods and this attribute will reflect the popup's open state.
   */
  @property({ type: Boolean, reflect: true }) open = false;

  /** Shows the opacity slider. Enabling this will cause the formatted value to be HEXA, RGBA, or HSLA. */
  @property({ type: Boolean }) opacity = false;

  /** By default, values are lowercase. With this attribute, values will be uppercase instead. */
  @property({ type: Boolean }) uppercase = false;

  /**
   * One or more predefined color swatches to display as presets in the color picker. Can include any format the color
   * picker can parse, including HEX(A), RGB(A), HSL(A), HSV(A), and CSS color names. Each color must be separated by a
   * semicolon (`;`). Alternatively, you can pass an array of color values or an array of `{ color, label }` objects to
   * this property using JavaScript. When using objects with labels, the label will be used for the swatch's accessible
   * name instead of the raw color value.
   */
  @property() swatches: string | string[] | WaColorPickerSwatch[] = "";

  /** Makes the color picker a required field. */
  @property({ type: Boolean, reflect: true }) required = false;

  constructor() {
    super();

    if (!isServer) {
      this.addEventListener("focusin", this.handleFocusIn);
      this.addEventListener("focusout", this.handleFocusOut);
    }
  }

  private handleCopy() {
    this.input.select();
    document.execCommand("copy");
    this.previewButton.focus();

    // Show copied animation
    this.previewButton.classList.add("preview-color-copied");
    this.previewButton.addEventListener("animationend", () => {
      this.previewButton.classList.remove("preview-color-copied");
    });
  }

  private handleFocusIn = () => {
    this.hasFocus = true;
  };

  private handleFocusOut = () => {
    this.hasFocus = false;
  };

  private handleFormatToggle() {
    const formats = ["hex", "rgb", "hsl", "hsv"];
    const nextIndex = (formats.indexOf(this.format) + 1) % formats.length;
    this.format = formats[nextIndex] as "hex" | "rgb" | "hsl" | "hsv";
    this.setColor(this.value || "");

    this.updateComplete.then(() => {
      this.dispatchEvent(new Event("change", { bubbles: true, composed: true }));
      this.dispatchEvent(new InputEvent("input", { bubbles: true, composed: true }));
    });
  }

  private handleAlphaDrag(event: PointerEvent) {
    const container = this.shadowRoot!.querySelector<HTMLElement>(".slider.alpha")!;
    const handle = container.querySelector<HTMLElement>(".slider-handle")!;
    const { width } = container.getBoundingClientRect();
    let initialValue = this.value;
    let currentValue = this.value;

    handle.focus();
    event.preventDefault();

    drag(container, {
      onMove: (x) => {
        this.alpha = clamp((x / width) * 100, 0, 100);
        this.syncValues();

        if (this.value !== currentValue) {
          currentValue = this.value;
          this.updateComplete.then(() => {
            this.dispatchEvent(new InputEvent("input", { bubbles: true, composed: true }));
          });
        }
      },
      onStop: () => {
        if (this.value !== initialValue) {
          initialValue = this.value;

          this.updateComplete.then(() => {
            this.dispatchEvent(new Event("change", { bubbles: true, composed: true }));
          });
        }
      },
      initialEvent: event,
    });
  }

  private handleHueDrag(event: PointerEvent) {
    const container = this.shadowRoot!.querySelector<HTMLElement>(".slider.hue")!;
    const handle = container.querySelector<HTMLElement>(".slider-handle")!;
    const { width } = container.getBoundingClientRect();
    let initialValue = this.value;
    let currentValue = this.value;

    handle.focus();
    event.preventDefault();

    drag(container, {
      onMove: (x) => {
        this.hue = clamp((x / width) * 360, 0, 360);
        this.syncValues();

        if (this.value !== currentValue) {
          currentValue = this.value;
          this.updateComplete.then(() => {
            this.dispatchEvent(new InputEvent("input"));
          });
        }
      },
      onStop: () => {
        if (this.value !== initialValue) {
          initialValue = this.value;
          this.updateComplete.then(() => {
            this.dispatchEvent(new Event("change", { bubbles: true, composed: true }));
          });
        }
      },
      initialEvent: event,
    });
  }

  private handleGridDrag(event: PointerEvent) {
    const grid = this.shadowRoot!.querySelector<HTMLElement>(".grid")!;
    const handle = grid.querySelector<HTMLElement>(".grid-handle")!;
    const { width, height } = grid.getBoundingClientRect();
    let initialValue = this.value;
    let currentValue = this.value;

    handle.focus();
    event.preventDefault();

    this.isDraggingGridHandle = true;

    drag(grid, {
      onMove: (x, y) => {
        this.saturation = clamp((x / width) * 100, 0, 100);
        this.brightness = clamp(100 - (y / height) * 100, 0, 100);
        this.syncValues();

        if (this.value !== currentValue) {
          currentValue = this.value;
          this.updateComplete.then(() => {
            this.dispatchEvent(new InputEvent("input", { bubbles: true, composed: true }));
          });
        }
      },
      onStop: () => {
        this.isDraggingGridHandle = false;
        if (this.value !== initialValue) {
          initialValue = this.value;
          this.updateComplete.then(() => {
            this.dispatchEvent(new Event("change", { bubbles: true, composed: true }));
          });
        }
      },
      initialEvent: event,
    });
  }

  private handleAlphaKeyDown(event: KeyboardEvent) {
    const increment = event.shiftKey ? 10 : 1;
    const oldValue = this.value;

    if (event.key === "ArrowLeft") {
      event.preventDefault();
      this.alpha = clamp(this.alpha - increment, 0, 100);
      this.syncValues();
    }

    if (event.key === "ArrowRight") {
      event.preventDefault();
      this.alpha = clamp(this.alpha + increment, 0, 100);
      this.syncValues();
    }

    if (event.key === "Home") {
      event.preventDefault();
      this.alpha = 0;
      this.syncValues();
    }

    if (event.key === "End") {
      event.preventDefault();
      this.alpha = 100;
      this.syncValues();
    }

    if (this.value !== oldValue) {
      this.updateComplete.then(() => {
        this.dispatchEvent(new InputEvent("input", { bubbles: true, composed: true }));
        this.dispatchEvent(new Event("change", { bubbles: true, composed: true }));
      });
    }
  }

  private handleHueKeyDown(event: KeyboardEvent) {
    const increment = event.shiftKey ? 10 : 1;
    const oldValue = this.value;

    if (event.key === "ArrowLeft") {
      event.preventDefault();
      this.hue = clamp(this.hue - increment, 0, 360);
      this.syncValues();
    }

    if (event.key === "ArrowRight") {
      event.preventDefault();
      this.hue = clamp(this.hue + increment, 0, 360);
      this.syncValues();
    }

    if (event.key === "Home") {
      event.preventDefault();
      this.hue = 0;
      this.syncValues();
    }

    if (event.key === "End") {
      event.preventDefault();
      this.hue = 360;
      this.syncValues();
    }

    if (this.value !== oldValue) {
      this.updateComplete.then(() => {
        this.dispatchEvent(new InputEvent("input", { bubbles: true, composed: true }));
        this.dispatchEvent(new Event("change", { bubbles: true, composed: true }));
      });
    }
  }

  private handleGridKeyDown(event: KeyboardEvent) {
    const increment = event.shiftKey ? 10 : 1;
    const oldValue = this.value;

    if (event.key === "ArrowLeft") {
      event.preventDefault();
      this.saturation = clamp(this.saturation - increment, 0, 100);
      this.syncValues();
    }

    if (event.key === "ArrowRight") {
      event.preventDefault();
      this.saturation = clamp(this.saturation + increment, 0, 100);
      this.syncValues();
    }

    if (event.key === "ArrowUp") {
      event.preventDefault();
      this.brightness = clamp(this.brightness + increment, 0, 100);
      this.syncValues();
    }

    if (event.key === "ArrowDown") {
      event.preventDefault();
      this.brightness = clamp(this.brightness - increment, 0, 100);
      this.syncValues();
    }

    if (this.value !== oldValue) {
      this.updateComplete.then(() => {
        this.dispatchEvent(new InputEvent("input", { bubbles: true, composed: true }));
        this.dispatchEvent(new Event("change", { bubbles: true, composed: true }));
      });
    }
  }

  private handleInputChange(event: Event) {
    const target = event.target as HTMLInputElement;
    const oldValue = this.value;

    // Prevent the `<wa-input>` element's `change` event from bubbling up
    event.stopPropagation();

    if (this.input.value) {
      this.setColor(target.value);
      target.value = this.value || "";
    } else {
      this.value = "";
    }

    if (this.value !== oldValue) {
      this.updateComplete.then(() => {
        this.dispatchEvent(new InputEvent("input", { bubbles: true, composed: true }));
        this.dispatchEvent(new Event("change", { bubbles: true, composed: true }));
      });
    }
  }

  private handleInputInput(event: InputEvent) {
    this.updateValidity();

    // Prevent the `<wa-input>` element's `input` event from bubbling up
    event.stopPropagation();
  }

  private handleInputKeyDown(event: KeyboardEvent) {
    if (event.key === "Enter") {
      const oldValue = this.value;

      if (this.input.value) {
        this.setColor(this.input.value);
        this.input.value = this.value;

        if (this.value !== oldValue) {
          this.updateComplete.then(() => {
            this.dispatchEvent(new InputEvent("input", { bubbles: true, composed: true }));
            this.dispatchEvent(new Event("change", { bubbles: true, composed: true }));
          });
        }

        setTimeout(() => this.input.select());
      } else {
        this.hue = 0;
      }
    }
  }

  @eventOptions({ passive: false })
  private handleTouchMove(event: TouchEvent) {
    event.preventDefault();
  }

  private parseColor(colorString: string) {
    if (!colorString || colorString.trim() === "") {
      return null;
    }

    const color = new TinyColor(colorString);
    if (!color.isValid) {
      return null;
    }

    const hslColor = color.toHsl();
    const rgb = color.toRgb();
    const hsvColor = color.toHsv();

    // Checks for null RGB values
    if (!rgb || rgb.r == null || rgb.g == null || rgb.b == null) {
      return null;
    }

    // Adjust saturation and lightness from 0-1 to 0-100
    const hsl = {
      h: hslColor.h || 0,
      s: (hslColor.s || 0) * 100,
      l: (hslColor.l || 0) * 100,
      a: hslColor.a || 0,
    };

    const hex = color.toHexString();
    const hexa = color.toHex8String();

    // Adjust saturation and value from 0-1 to 0-100
    const hsv = {
      h: hsvColor.h || 0,
      s: (hsvColor.s || 0) * 100,
      v: (hsvColor.v || 0) * 100,
      a: hsvColor.a || 0,
    };

    return {
      hsl: {
        h: hsl.h,
        s: hsl.s,
        l: hsl.l,
        string: this.setLetterCase(
          `hsl(${Math.round(hsl.h)}, ${Math.round(hsl.s)}%, ${Math.round(hsl.l)}%)`,
        ),
      },
      hsla: {
        h: hsl.h,
        s: hsl.s,
        l: hsl.l,
        a: hsl.a,
        string: this.setLetterCase(
          `hsla(${Math.round(hsl.h)}, ${Math.round(hsl.s)}%, ${Math.round(hsl.l)}%, ${hsl.a.toFixed(2).toString()})`,
        ),
      },
      hsv: {
        h: hsv.h,
        s: hsv.s,
        v: hsv.v,
        string: this.setLetterCase(
          `hsv(${Math.round(hsv.h)}, ${Math.round(hsv.s)}%, ${Math.round(hsv.v)}%)`,
        ),
      },
      hsva: {
        h: hsv.h,
        s: hsv.s,
        v: hsv.v,
        a: hsv.a,
        string: this.setLetterCase(
          `hsva(${Math.round(hsv.h)}, ${Math.round(hsv.s)}%, ${Math.round(hsv.v)}%, ${hsv.a.toFixed(2).toString()})`,
        ),
      },
      rgb: {
        r: rgb.r,
        g: rgb.g,
        b: rgb.b,
        string: this.setLetterCase(
          `rgb(${Math.round(rgb.r)}, ${Math.round(rgb.g)}, ${Math.round(rgb.b)})`,
        ),
      },
      rgba: {
        r: rgb.r,
        g: rgb.g,
        b: rgb.b,
        a: rgb.a || 0,
        string: this.setLetterCase(
          `rgba(${Math.round(rgb.r)}, ${Math.round(rgb.g)}, ${Math.round(rgb.b)}, ${(rgb.a || 0).toFixed(2).toString()})`,
        ),
      },
      hex: this.setLetterCase(hex),
      hexa: this.setLetterCase(hexa),
    };
  }

  private setColor(colorString: string) {
    const newColor = this.parseColor(colorString);

    if (newColor === null) {
      return false;
    }

    this.hue = newColor.hsva.h;
    this.saturation = newColor.hsva.s;
    this.brightness = newColor.hsva.v;
    this.alpha = this.opacity ? newColor.hsva.a * 100 : 100;

    this.syncValues();

    return true;
  }

  private setLetterCase(string: string) {
    if (typeof string !== "string") {
      return "";
    }
    return this.uppercase ? string.toUpperCase() : string.toLowerCase();
  }

  private async syncValues() {
    const currentColor = this.parseColor(
      `hsva(${this.hue}, ${this.saturation}%, ${this.brightness}%, ${this.alpha / 100})`,
    );

    if (currentColor === null) {
      return;
    }

    // Update the value
    if (this.format === "hsl") {
      this.inputValue = this.opacity ? currentColor.hsla.string : currentColor.hsl.string;
    } else if (this.format === "rgb") {
      this.inputValue = this.opacity ? currentColor.rgba.string : currentColor.rgb.string;
    } else if (this.format === "hsv") {
      this.inputValue = this.opacity ? currentColor.hsva.string : currentColor.hsv.string;
    } else {
      this.inputValue = this.opacity ? currentColor.hexa : currentColor.hex;
    }

    // Setting this.value will trigger the watcher which parses the new value. We want to bypass that behavior because
    // we've already parsed the color here and conversion/rounding can lead to values changing slightly. When this
    // happens, dragging the grid handle becomes jumpy. After the next update, the usual behavior is restored.
    this.isSafeValue = true;
    this.value = this.inputValue;

    await this.updateComplete;
    this.isSafeValue = false;
  }

  private handleAfterHide() {
    this.previewButton.classList.remove("preview-color-copied");
    // Update validity so we get a new anchor.
    this.updateValidity();
  }

  private handleAfterShow() {
    // Update validity so we get a new anchor.
    this.updateValidity();
  }

  private handleEyeDropper() {
    if (!this.hasEyeDropper) {
      return;
    }

    const eyeDropper = new EyeDropper();

    eyeDropper
      .open()
      .then((colorSelectionResult) => {
        const oldValue = this.value;

        this.setColor(colorSelectionResult.sRGBHex);

        if (this.value !== oldValue) {
          this.updateComplete.then(() => {
            this.dispatchEvent(new InputEvent("input", { bubbles: true, composed: true }));
            this.dispatchEvent(new Event("change", { bubbles: true, composed: true }));
          });
        }
      })
      .catch(() => {
        // The user canceled, do nothing
      });
  }

  private selectSwatch(color: string) {
    const oldValue = this.value;

    if (!this.disabled) {
      this.setColor(color);

      if (this.value !== oldValue) {
        this.updateComplete.then(() => {
          this.dispatchEvent(new InputEvent("input", { bubbles: true, composed: true }));
          this.dispatchEvent(new Event("change", { bubbles: true, composed: true }));
        });
      }
    }
  }

  /** Generates a hex string from HSV values. Hue must be 0-360. All other arguments must be 0-100. */
  getHexString(hue: number, saturation: number, brightness: number, alpha = 100) {
    const color = new TinyColor(`hsva(${hue}, ${saturation}%, ${brightness}%, ${alpha / 100})`);
    if (!color.isValid) {
      return "";
    }

    return color.toHex8String();
  }

  // Prevents nested components from leaking events
  private stopNestedEventPropagation(event: CustomEvent) {
    event.stopImmediatePropagation();
  }

  @watch("format", { waitUntilFirstUpdate: true })
  handleFormatChange() {
    this.syncValues();
  }

  @watch("opacity")
  handleOpacityChange() {
    this.alpha = 100;
  }

  protected willUpdate(changedProperties: PropertyValues<this>): void {
    super.willUpdate(changedProperties);

    // Its kind of bizarre, but this is required to get SSR to play nicely.
    if (changedProperties.has("value")) {
      this.handleValueChange(changedProperties.get("value") || "", this.value || "");
    }
  }

  @watch("value")
  handleValueChange(oldValue: string | undefined, newValue: string) {
    this.isEmpty = !newValue;

    if (!newValue) {
      this.hue = 0;
      this.saturation = 0;
      this.brightness = 100;
      this.alpha = 100;
    }

    if (!this.isSafeValue) {
      const newColor = this.parseColor(newValue);

      if (newColor !== null) {
        this.inputValue = this.value || "";
        this.hue = newColor.hsva.h;
        this.saturation = newColor.hsva.s;
        this.brightness = newColor.hsva.v;
        this.alpha = newColor.hsva.a * 100;
        this.syncValues();
      } else {
        this.inputValue = oldValue ?? "";
      }
    }

    this.requestUpdate();
  }

  /** Sets focus on the color picker. */
  focus(options?: FocusOptions) {
    this.trigger.focus(options);
  }

  /** Removes focus from the color picker. */
  blur() {
    const elementToBlur = this.trigger;

    if (this.hasFocus) {
      // We don't know which element in the color picker has focus, so we'll move it to the trigger or base (inline) and
      // blur that instead. This results in document.activeElement becoming the `<body>`. This doesn't cause another
      // focus event because we're using focusin and something inside the color picker already has focus.
      elementToBlur.focus({ preventScroll: true });
      elementToBlur.blur();
    }

    if (this.popup?.active) {
      this.hide();
    }
  }

  /** Returns the current value as a string in the specified format. */
  getFormattedValue(
    format: "hex" | "hexa" | "rgb" | "rgba" | "hsl" | "hsla" | "hsv" | "hsva" = "hex",
  ) {
    const currentColor = this.parseColor(
      `hsva(${this.hue}, ${this.saturation}%, ${this.brightness}%, ${this.alpha / 100})`,
    );

    if (currentColor === null) {
      return "";
    }

    switch (format) {
      case "hex":
        return currentColor.hex;
      case "hexa":
        return currentColor.hexa;
      case "rgb":
        return currentColor.rgb.string;
      case "rgba":
        return currentColor.rgba.string;
      case "hsl":
        return currentColor.hsl.string;
      case "hsla":
        return currentColor.hsla.string;
      case "hsv":
        return currentColor.hsv.string;
      case "hsva":
        return currentColor.hsva.string;
      default:
        return "";
    }
  }

  private reportValidityAfterShow = () => {
    // Remove the event so we don't emit "wa-invalid" twice
    this.removeEventListener("invalid", this.emitInvalid);

    this.reportValidity();

    this.addEventListener("invalid", this.emitInvalid);
  };

  /** Checks for validity and shows the browser's validation message if the control is invalid. */
  reportValidity() {
    // This won't get called when a form is submitted. This is only for manual calls.
    if (!this.validity.valid && !this.open) {
      // Show the popup so the browser can focus on it
      this.addEventListener("wa-after-show", this.reportValidityAfterShow, { once: true });
      this.show();

      if (!this.disabled) {
        // By standards we have to emit a `wa-invalid` event here synchronously.
        this.dispatchEvent(new WaInvalidEvent());
      }

      return false;
    }

    return super.reportValidity();
  }

  formResetCallback() {
    this.value = this.defaultValue;

    super.formResetCallback();
  }

  firstUpdated(changedProperties: PropertyValues<this>): void {
    super.firstUpdated(changedProperties);

    this.hasEyeDropper = "EyeDropper" in window;
  }

  private handleKeyDown = (event: KeyboardEvent) => {
    // Close when escape is pressed inside an open popup. We need to listen on the panel itself and stop propagation
    // in case any ancestors are also listening for this key.
    if (this.open && event.key === "Escape" && isTopDismissible(this)) {
      event.stopPropagation();
      this.hide();
      this.focus();
    }
  };

  private handleDocumentKeyDown = (event: KeyboardEvent) => {
    // Close when escape or tab is pressed
    if (event.key === "Escape" && this.open && isTopDismissible(this)) {
      event.stopPropagation();
      this.focus();
      this.hide();
      return;
    }

    // Handle tabbing
    if (event.key === "Tab") {
      // Tabbing outside of the containing element closes the panel
      //
      // If the popup is used within a shadow DOM, we need to obtain the activeElement within that shadowRoot,
      // otherwise `document.activeElement` will only return the name of the parent shadow DOM element.
      setTimeout(() => {
        const activeElement =
          this.getRootNode() instanceof ShadowRoot
            ? document.activeElement?.shadowRoot?.activeElement
            : document.activeElement;

        if (!this || activeElement?.closest(this.tagName.toLowerCase()) !== this) {
          this.hide();
        }
      });
    }
  };

  private handleDocumentMouseDown = (event: MouseEvent) => {
    // Close when clicking outside of the popup panel and trigger
    const path = event.composedPath();

    // Check if click is inside the popup panel or the trigger element specifically
    const isInsideRelevantArea = path.some(
      (element) =>
        element instanceof Element &&
        (element.closest(".color-picker") || element === this.trigger),
    );

    if (this && !isInsideRelevantArea) {
      this.hide();
    }
  };

  handleTriggerClick() {
    if (this.open) {
      this.hide();
    } else {
      this.show();
      this.focus();
    }
  }

  async handleTriggerKeyDown(event: KeyboardEvent) {
    // When spacebar/enter is pressed, show the panel but don't focus on the menu. This let's the user press the same
    // key again to hide the menu in case they don't want to make a selection.
    if ([" ", "Enter"].includes(event.key)) {
      event.preventDefault();
      this.handleTriggerClick();
      return;
    }
  }

  handleTriggerKeyUp(event: KeyboardEvent) {
    // Prevent space from triggering a click event in Firefox
    if (event.key === " ") {
      event.preventDefault();
    }
  }

  updateAccessibleTrigger() {
    const accessibleTrigger = this.trigger;

    if (accessibleTrigger) {
      accessibleTrigger.setAttribute("aria-haspopup", "true");
      accessibleTrigger.setAttribute("aria-expanded", this.open ? "true" : "false");
    }
  }

  /** Shows the color picker panel. */
  async show() {
    if (this.open) {
      return undefined;
    }

    this.open = true;
    return waitForEvent(this, "wa-after-show");
  }

  /** Hides the color picker panel */
  async hide() {
    if (!this.open) {
      return undefined;
    }

    this.open = false;
    return waitForEvent(this, "wa-after-hide");
  }

  addOpenListeners() {
    this.base.addEventListener("keydown", this.handleKeyDown);
    document.addEventListener("keydown", this.handleDocumentKeyDown);
    document.addEventListener("mousedown", this.handleDocumentMouseDown);
    registerDismissible(this);
  }

  removeOpenListeners() {
    if (this.base) {
      this.base.removeEventListener("keydown", this.handleKeyDown);
    }
    document.removeEventListener("keydown", this.handleDocumentKeyDown);
    document.removeEventListener("mousedown", this.handleDocumentMouseDown);
    unregisterDismissible(this);
  }

  @watch("open", { waitUntilFirstUpdate: true })
  async handleOpenChange() {
    if (this.disabled) {
      this.open = false;
      return;
    }

    this.updateAccessibleTrigger();

    if (this.open) {
      // Show
      this.dispatchEvent(new CustomEvent("wa-show"));

      this.addOpenListeners();
      await this.updateComplete;
      this.base.hidden = false;
      this.popup.active = true;
      await animateWithClass(this.popup.popup, "show-with-scale");
      this.dispatchEvent(new CustomEvent("wa-after-show"));
    } else {
      // Hide
      this.dispatchEvent(new CustomEvent("wa-hide"));

      this.removeOpenListeners();
      await animateWithClass(this.popup.popup, "hide-with-scale");
      this.base.hidden = true;
      this.popup.active = false;
      this.dispatchEvent(new CustomEvent("wa-after-hide"));
    }
  }

  render() {
    const hasLabelSlot = !this.hasUpdated
      ? this.withLabel
      : this.withLabel || this.hasSlotController.test("label");
    const hasHintSlot = !this.hasUpdated
      ? this.withHint
      : this.withHint || this.hasSlotController.test("hint");
    const hasLabel = this.label ? true : !!hasLabelSlot;
    const hasHint = this.hint ? true : !!hasHintSlot;

    const gridHandleX = this.saturation;
    const gridHandleY = 100 - this.brightness;
    const normalizedSwatches: WaColorPickerSwatch[] = Array.isArray(this.swatches)
      ? this.swatches.map((s) => (typeof s === "string" ? { color: s, label: s } : s))
      : this.swatches
          .split(";")
          .filter((color) => color.trim() !== "")
          .map((color) => ({ color: color.trim(), label: color.trim() }));

    const colorPicker = html`
      <div
        part="base"
        class=${classMap({
          "color-picker": true,
        })}
        aria-disabled=${this.disabled ? "true" : "false"}
        tabindex="-1"
      >
        <div
          part="grid"
          class="grid"
          style=${styleMap({ backgroundColor: this.getHexString(this.hue, 100, 100) })}
          @pointerdown=${this.handleGridDrag}
          @touchmove=${this.handleTouchMove}
        >
          <span
            part="grid-handle"
            class=${classMap({
              "grid-handle": true,
              "grid-handle-dragging": this.isDraggingGridHandle,
            })}
            style=${styleMap({
              top: `${gridHandleY}%`,
              left: `${gridHandleX}%`,
              backgroundColor: this.getHexString(
                this.hue,
                this.saturation,
                this.brightness,
                this.alpha,
              ),
            })}
            role="application"
            aria-label="HSV"
            tabindex=${ifDefined(this.disabled ? undefined : "0")}
            @keydown=${this.handleGridKeyDown}
          ></span>
        </div>

        <div class="controls">
          <div class="sliders">
            <div
              part="slider hue-slider"
              class="hue slider"
              @pointerdown=${this.handleHueDrag}
              @touchmove=${this.handleTouchMove}
            >
              <span
                part="slider-handle hue-slider-handle"
                class="slider-handle"
                style=${styleMap({
                  left: `${this.hue === 0 ? 0 : 100 / (360 / this.hue)}%`,
                  backgroundColor: this.getHexString(this.hue, 100, 100),
                })}
                role="slider"
                aria-label="hue"
                aria-orientation="horizontal"
                aria-valuemin="0"
                aria-valuemax="360"
                aria-valuenow=${`${Math.round(this.hue)}`}
                tabindex=${ifDefined(this.disabled ? undefined : "0")}
                @keydown=${this.handleHueKeyDown}
              ></span>
            </div>

            ${
              this.opacity
                ? html`
                    <div
                      part="slider opacity-slider"
                      class="alpha slider transparent-bg"
                      @pointerdown="${this.handleAlphaDrag}"
                      @touchmove=${this.handleTouchMove}
                    >
                      <div
                        class="alpha-gradient"
                        style=${styleMap({
                        backgroundImage: `linear-gradient(
                          to right,
                          ${this.getHexString(this.hue, this.saturation, this.brightness, 0)} 0%,
                          ${this.getHexString(this.hue, this.saturation, this.brightness, 100)} 100%
                        )`,
                      })}
                      ></div>
                      <span
                        part="slider-handle opacity-slider-handle"
                        class="slider-handle"
                        style=${styleMap({
                        left: `${this.alpha}%`,
                        backgroundColor: this.getHexString(
                          this.hue,
                          this.saturation,
                          this.brightness,
                          this.alpha,
                        ),
                      })}
                        role="slider"
                        aria-label="alpha"
                        aria-orientation="horizontal"
                        aria-valuemin="0"
                        aria-valuemax="100"
                        aria-valuenow=${Math.round(this.alpha)}
                        tabindex=${ifDefined(this.disabled ? undefined : "0")}
                        @keydown=${this.handleAlphaKeyDown}
                      ></span>
                    </div>
                  `
                : ""
            }
          </div>

          <button
            type="button"
            part="preview"
            class="preview transparent-bg"
            aria-label=${this.localize.term("copy")}
            style=${styleMap({
              "--preview-color": this.getHexString(
                this.hue,
                this.saturation,
                this.brightness,
                this.alpha,
              ),
            })}
            @click=${this.handleCopy}
          ></button>
        </div>

        <div class="user-input" aria-live="polite">
          <wa-input
            part="input"
            type="text"
            name=${this.name}
            size="s"
            autocomplete="off"
            autocorrect="off"
            autocapitalize="off"
            spellcheck="false"
            .value=${this.isEmpty ? "" : this.inputValue}
            ?required=${this.required}
            ?disabled=${this.disabled}
            aria-label=${this.localize.term("currentValue")}
            @keydown=${this.handleInputKeyDown}
            @change=${this.handleInputChange}
            @input=${this.handleInputInput}
            @blur=${this.stopNestedEventPropagation}
            @focus=${this.stopNestedEventPropagation}
          ></wa-input>

          <wa-button-group>
            ${
              !this.withoutFormatToggle
                ? html`
                    <wa-button
                      part="format-button"
                      size="s"
                      appearance="outlined"
                      aria-label=${this.localize.term("toggleColorFormat")}
                      exportparts="
                      base:format-button__base,
                      start:format-button__start,
                      label:format-button__label,
                      end:format-button__end,
                      caret:format-button__caret
                    "
                      @click=${this.handleFormatToggle}
                      @blur=${this.stopNestedEventPropagation}
                      @focus=${this.stopNestedEventPropagation}
                    >
                      ${this.setLetterCase(this.format)}
                    </wa-button>
                  `
                : ""
            }
            ${
              this.hasEyeDropper
                ? html`
                    <wa-button
                      part="eyedropper-button"
                      size="s"
                      appearance="outlined"
                      exportparts="
                      base:eyedropper-button__base,
                      start:eyedropper-button__start,
                      label:eyedropper-button__label,
                      end:eyedropper-button__end,
                      caret:eyedropper-button__caret
                    "
                      @click=${this.handleEyeDropper}
                      @blur=${this.stopNestedEventPropagation}
                      @focus=${this.stopNestedEventPropagation}
                    >
                      <wa-icon
                        library="system"
                        name="eyedropper"
                        variant="solid"
                        label=${this.localize.term("selectAColorFromTheScreen")}
                      ></wa-icon>
                    </wa-button>
                  `
                : ""
            }
          </wa-button-group>
        </div>

        ${
          normalizedSwatches.length > 0
            ? html`
                <div part="swatches" class="swatches">
                  ${normalizedSwatches.map((swatch) => {
                  const parsedColor = this.parseColor(swatch.color);

                  // If we can't parse it, skip it
                  if (!parsedColor) {
                    return "";
                  }

                  return html`
                    <div
                      part="swatch"
                      class="swatch transparent-bg"
                      tabindex=${ifDefined(this.disabled ? undefined : "0")}
                      role="button"
                      aria-label=${swatch.label}
                      @click=${() => this.selectSwatch(swatch.color)}
                      @keydown=${(event: KeyboardEvent) =>
                        !this.disabled && event.key === "Enter" && this.setColor(parsedColor.hexa)}
                    >
                      <div
                        class="swatch-color"
                        style=${styleMap({ backgroundColor: parsedColor.hexa })}
                      ></div>
                    </div>
                  `;
                })}
                </div>
              `
            : ""
        }
      </div>
    `;

    // Render with popup
    return html`
      <div
        class=${classMap({
          container: true,
          "form-control": true,
          "form-control-has-label": hasLabel,
        })}
        part="trigger-container form-control"
      >
        <div
          part="form-control-label"
          class=${classMap({
            label: true,
            "has-label": hasLabel,
          })}
          id="form-control-label"
        >
          <slot name="label">${this.label}</slot>
        </div>

        <button
          id="trigger"
          part="trigger form-control-input"
          class=${classMap({
            trigger: true,
            "trigger-empty": this.isEmpty,
            "transparent-bg": true,
            "form-control-input": true,
          })}
          style=${styleMap({
            color: this.getHexString(this.hue, this.saturation, this.brightness, this.alpha),
          })}
          type="button"
          aria-labelledby="form-control-label"
          aria-describedby="hint"
          .disabled=${this.disabled}
          @click=${this.handleTriggerClick}
          @keydown=${this.handleTriggerKeyDown}
          @keyup=${this.handleTriggerKeyUp}
        ></button>

        <slot
          id="hint"
          name="hint"
          part="hint"
          class=${classMap({
            "has-slotted": hasHint,
          })}
        >
          ${this.hint}
        </slot>
      </div>

      <wa-popup
        class="color-popup"
        anchor="trigger"
        placement=${this.placement}
        distance="0"
        skidding="0"
        flip
        flip-fallback-strategy="best-fit"
        shift
        shift-padding="10"
        aria-disabled=${this.disabled ? "true" : "false"}
        @wa-after-show=${this.handleAfterShow}
        @wa-after-hide=${this.handleAfterHide}
      >
        ${colorPicker}
      </wa-popup>
    `;
  }
}

// The change-in-update warning is required for this component because:
//
// - The base class (WebAwesomeFormAssociatedElement) firstUpdated() calls updateValidity() which triggers
//    requestUpdate('validity').
// - HasSlotController calls host.requestUpdate() on slotchange events.
// - @watch('value') handler sets multiple @state properties (isEmpty, hue, saturation, brightness, alpha, inputValue)
//    and calls syncValues() and requestUpdate() during the update cycle to keep color state in sync.
// - @watch('opacity') and @watch('format') handlers set @state properties during update to synchronize color values.
// - firstUpdated() sets the @state property hasEyeDropper based on browser capability detection.
//
// See https://lit.dev/docs/tools/development/#development-build-runtime-warnings
WaColorPicker.disableWarning?.("change-in-update");

`````
