# externals/webawesome/rating/rating.ts

## Option 1

`````json
{"printWidth":80}
`````

### Diff

`````diff
===================================================================
--- prettier
+++ oxfmt
@@ -164,9 +164,16 @@
   };
 
   /** The component's size. */
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

`````

### Actual (oxfmt)

`````ts
import { html, type PropertyValues } from "lit";
import { customElement, property, state } from "lit/decorators.js";
import { classMap } from "lit/directives/class-map.js";
import { styleMap } from "lit/directives/style-map.js";
import { unsafeHTML } from "lit/directives/unsafe-html.js";
import { WaHoverEvent } from "../../events/hover.js";
import { clamp } from "../../internal/math.js";
import { warnDeprecatedSize } from "../../internal/size.js";
import { RequiredValidator } from "../../internal/validators/required-validator.js";
import { watch } from "../../internal/watch.js";
import { WebAwesomeFormAssociatedElement } from "../../internal/webawesome-form-associated-element.js";
import sizeStyles from "../../styles/component/size.styles.js";
import { LocalizeController } from "../../utilities/localize.js";
import "../icon/icon.js";
import styles from "./rating.styles.js";

/**
 * @summary Ratings display a numeric score as a row of selectable symbols, typically stars. Use them to capture quick
 *  feedback or show an average rating for a product or piece of content.
 * @documentation https://webawesome.com/docs/components/rating
 * @status stable
 * @since 2.0
 *
 * @dependency wa-icon
 *
 * @event change - Emitted when the rating's value changes.
 * @event {{ phase: 'start' | 'move' | 'end', value: number }} wa-hover - Emitted when the user hovers over a value. The
 *  `phase` property indicates when hovering starts, moves to a new value, or ends. The `value` property tells what the
 *  rating's value would be if the user were to commit to the hovered value.
 * @event wa-invalid - Emitted when the form control has been checked for validity and its constraints aren't satisfied.
 *
 * @csspart base - The component's base wrapper.
 *
 * @cssproperty --symbol-color - The inactive color for symbols.
 * @cssproperty --symbol-color-active - The active color for symbols.
 * @cssproperty --symbol-spacing - The spacing to use around symbols.
 */
@customElement("wa-rating")
export default class WaRating extends WebAwesomeFormAssociatedElement {
  static css = [sizeStyles, styles];

  static get validators() {
    return [...super.validators, RequiredValidator()];
  }

  assumeInteractionOn = ["change"];

  private readonly localize = new LocalizeController(this);

  connectedCallback() {
    super.connectedCallback();
    this.setAttribute("role", "slider");
    this.setAttribute("aria-valuenow", String(this.value));
    this.setAttribute("aria-valuemin", "0");
    this.setAttribute("aria-valuemax", String(this.max));
    this.setAttribute("aria-disabled", this.disabled ? "true" : "false");
    this.setAttribute("aria-readonly", this.readonly ? "true" : "false");

    if (this.label) {
      this.setAttribute("aria-label", this.label);
    }

    if (!this.disabled && !this.readonly) {
      this.tabIndex = 0;
    } else {
      this.tabIndex = -1;
    }

    this.addEventListener("click", this.handleClick);
    this.addEventListener("keydown", this.handleKeyDown);
    this.addEventListener("pointerenter", this.handlePointerEnter);
    this.addEventListener("pointermove", this.handlePointerMove);
    this.addEventListener("pointerleave", this.handlePointerLeave);
    this.addEventListener("pointerdown", this.handlePointerDown);
    this.addEventListener("pointerup", this.handlePointerUp);
  }

  disconnectedCallback() {
    super.disconnectedCallback();
    this.removeEventListener("click", this.handleClick);
    this.removeEventListener("keydown", this.handleKeyDown);
    this.removeEventListener("pointerenter", this.handlePointerEnter);
    this.removeEventListener("pointermove", this.handlePointerMove);
    this.removeEventListener("pointerleave", this.handlePointerLeave);
    this.removeEventListener("pointerdown", this.handlePointerDown);
    this.removeEventListener("pointerup", this.handlePointerUp);
  }

  updated(changedProperties: PropertyValues<this>) {
    super.updated(changedProperties);

    if (changedProperties.has("value")) {
      this.setAttribute("aria-valuenow", String(this.value));
    }

    if (changedProperties.has("max")) {
      this.setAttribute("aria-valuemax", String(this.max));
    }

    if (changedProperties.has("disabled")) {
      this.setAttribute("aria-disabled", this.disabled ? "true" : "false");
      this.tabIndex = this.disabled || this.readonly ? -1 : 0;
    }

    if (changedProperties.has("readonly")) {
      this.setAttribute("aria-readonly", this.readonly ? "true" : "false");
      this.tabIndex = this.disabled || this.readonly ? -1 : 0;
    }

    if (changedProperties.has("label")) {
      if (this.label) {
        this.setAttribute("aria-label", this.label);
      } else {
        this.removeAttribute("aria-label");
      }
    }
  }

  @state() private hoverValue = 0;
  @state() private isHovering = false;

  /** The name of the rating, submitted as a name/value pair with form data. */
  @property() name: string | null = null;

  /** A label that describes the rating to assistive devices. */
  @property() label = "";

  /** The current rating. */
  @property({ type: Number }) value = 0;

  /** The default value of the form control. Used to reset the rating to its initial value. */
  @property({ type: Number, attribute: "default-value" }) defaultValue = 0;

  /** The highest rating to show. */
  @property({ type: Number }) max = 5;

  /**
   * The precision at which the rating will increase and decrease. For example, to allow half-star ratings, set this
   * attribute to `0.5`.
   */
  @property({ type: Number }) precision = 1;

  /** Makes the rating readonly. */
  @property({ type: Boolean, reflect: true }) readonly = false;

  /** Disables the rating. */
  @property({ type: Boolean }) declare disabled: boolean;

  /** Makes the rating a required field. */
  @property({ type: Boolean, reflect: true }) required = false;

  /**
   * A function that customizes the symbol to be rendered. The first and only argument is the rating's current value.
   * The function should return a string containing trusted HTML of the symbol to render at the specified value. Works
   * well with `<wa-icon>` elements.
   */
  @property() getSymbol: (value: number, isSelected: boolean) => string = (
    _value,
    isSelected,
  ) => {
    return isSelected
      ? '<wa-icon name="star" library="system" variant="solid"></wa-icon>'
      : '<wa-icon name="star" library="system" variant="regular"></wa-icon>';
  };

  /** The component's size. */
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

  private getValueFromPointerPosition(event: PointerEvent) {
    return this.getValueFromXCoordinate(event.clientX);
  }

  private getValueFromXCoordinate(coordinate: number) {
    const isRtl = this.localize.dir() === "rtl";
    const { left, right, width } = this.getBoundingClientRect();
    const value = isRtl
      ? this.roundToPrecision(
          ((right - coordinate) / width) * this.max,
          this.precision,
        )
      : this.roundToPrecision(
          ((coordinate - left) / width) * this.max,
          this.precision,
        );

    return clamp(value, 0, this.max);
  }

  private handleClick = (event: MouseEvent) => {
    if (this.disabled) {
      return;
    }

    this.setRatingValue(this.getValueFromXCoordinate(event.clientX));
    this.updateComplete.then(() => {
      this.dispatchEvent(
        new Event("change", { bubbles: true, composed: true }),
      );
    });
  };

  private setRatingValue(newValue: number) {
    if (this.disabled || this.readonly) {
      return;
    }

    this.value = newValue === this.value ? 0 : newValue;
    this.isHovering = false;
  }

  private handleKeyDown = (event: KeyboardEvent) => {
    const isLtr = this.matches(":dir(ltr)");
    const isRtl = this.localize.dir() === "rtl";
    const oldValue = this.value;

    if (this.disabled || this.readonly) {
      return;
    }

    if (
      event.key === "ArrowDown" ||
      (isLtr && event.key === "ArrowLeft") ||
      (isRtl && event.key === "ArrowRight")
    ) {
      const decrement = event.shiftKey ? 1 : this.precision;
      this.value = Math.max(0, this.value - decrement);
      event.preventDefault();
    }

    if (
      event.key === "ArrowUp" ||
      (isLtr && event.key === "ArrowRight") ||
      (isRtl && event.key === "ArrowLeft")
    ) {
      const increment = event.shiftKey ? 1 : this.precision;
      this.value = Math.min(this.max, this.value + increment);
      event.preventDefault();
    }

    if (event.key === "Home") {
      this.value = 0;
      event.preventDefault();
    }

    if (event.key === "End") {
      this.value = this.max;
      event.preventDefault();
    }

    if (this.value !== oldValue) {
      this.updateComplete.then(() => {
        this.dispatchEvent(
          new Event("change", { bubbles: true, composed: true }),
        );
      });
    }
  };

  private handlePointerEnter = (event: PointerEvent) => {
    this.isHovering = true;
    this.hoverValue = this.getValueFromPointerPosition(event);
  };

  private handlePointerMove = (event: PointerEvent) => {
    this.hoverValue = this.getValueFromPointerPosition(event);
  };

  private handlePointerLeave = () => {
    this.isHovering = false;
  };

  private handlePointerDown = (event: PointerEvent) => {
    if (event.button !== 0) {
      return;
    }

    this.isHovering = true;
    this.hoverValue = this.getValueFromPointerPosition(event);

    // Capture the pointer so pointermove/pointerup fire even outside the element (e.g. touch drag)
    this.setPointerCapture(event.pointerId);

    // Prevent scrolling on touch
    event.preventDefault();
  };

  private handlePointerUp = (event: PointerEvent) => {
    this.releasePointerCapture(event.pointerId);
    this.isHovering = false;
  };

  private roundToPrecision(numberToRound: number, precision = 0.5) {
    const multiplier = 1 / precision;
    return Math.ceil(numberToRound * multiplier) / multiplier;
  }

  @watch("hoverValue")
  handleHoverValueChange() {
    this.dispatchEvent(
      new WaHoverEvent({
        phase: "move",
        value: this.hoverValue,
      }),
    );
  }

  @watch("isHovering")
  handleIsHoveringChange() {
    this.dispatchEvent(
      new WaHoverEvent({
        phase: this.isHovering ? "start" : "end",
        value: this.hoverValue,
      }),
    );
  }

  formResetCallback() {
    this.value = this.defaultValue;
    super.formResetCallback();
  }

  render() {
    const isRtl = this.hasUpdated ? this.localize.dir() === "rtl" : this.dir;
    const counter = Array.from(Array(this.max).keys());
    let displayValue = 0;

    if (this.disabled || this.readonly) {
      displayValue = this.value;
    } else {
      displayValue = this.isHovering ? this.hoverValue : this.value;
    }

    return html`
      <div
        part="base"
        class=${classMap({
          rating: true,
          "rating-readonly": this.readonly,
          "rating-disabled": this.disabled,
        })}
      >
        <span class="symbols">
          ${counter.map((index) => {
            const isSelected = displayValue >= index + 1;

            if (displayValue > index && displayValue < index + 1) {
              // Users can click the current value to clear the rating. When this happens, we set this.isHovering to
              // false to prevent the hover state from confusing them as they move the mouse out of the control. This
              // extra mouseenter will reinstate it if they happen to mouse over an adjacent symbol.
              return html`
                <span
                  class=${classMap({
                    symbol: true,
                    "partial-symbol-container": true,
                    "symbol-hover":
                      this.isHovering && Math.ceil(displayValue) === index + 1,
                  })}
                  role="presentation"
                >
                  <div
                    style=${styleMap({
                      clipPath: isRtl
                        ? `inset(0 ${(displayValue - index) * 100}% 0 0)`
                        : `inset(0 0 0 ${(displayValue - index) * 100}%)`,
                    })}
                  >
                    ${unsafeHTML(this.getSymbol(index + 1, false))}
                  </div>
                  <div
                    class="partial-filled"
                    style=${styleMap({
                      clipPath: isRtl
                        ? `inset(0 0 0 ${100 - (displayValue - index) * 100}%)`
                        : `inset(0 ${100 - (displayValue - index) * 100}% 0 0)`,
                    })}
                  >
                    ${unsafeHTML(this.getSymbol(index + 1, true))}
                  </div>
                </span>
              `;
            }

            return html`
              <span
                class=${classMap({
                  symbol: true,
                  "symbol-hover":
                    this.isHovering && Math.ceil(displayValue) === index + 1,
                  "symbol-active": displayValue >= index + 1,
                })}
                role="presentation"
              >
                ${unsafeHTML(this.getSymbol(index + 1, isSelected))}
              </span>
            `;
          })}
        </span>
      </div>
    `;
  }
}

declare global {
  interface HTMLElementTagNameMap {
    "wa-rating": WaRating;
  }
}

`````

### Expected (prettier)

`````ts
import { html, type PropertyValues } from "lit";
import { customElement, property, state } from "lit/decorators.js";
import { classMap } from "lit/directives/class-map.js";
import { styleMap } from "lit/directives/style-map.js";
import { unsafeHTML } from "lit/directives/unsafe-html.js";
import { WaHoverEvent } from "../../events/hover.js";
import { clamp } from "../../internal/math.js";
import { warnDeprecatedSize } from "../../internal/size.js";
import { RequiredValidator } from "../../internal/validators/required-validator.js";
import { watch } from "../../internal/watch.js";
import { WebAwesomeFormAssociatedElement } from "../../internal/webawesome-form-associated-element.js";
import sizeStyles from "../../styles/component/size.styles.js";
import { LocalizeController } from "../../utilities/localize.js";
import "../icon/icon.js";
import styles from "./rating.styles.js";

/**
 * @summary Ratings display a numeric score as a row of selectable symbols, typically stars. Use them to capture quick
 *  feedback or show an average rating for a product or piece of content.
 * @documentation https://webawesome.com/docs/components/rating
 * @status stable
 * @since 2.0
 *
 * @dependency wa-icon
 *
 * @event change - Emitted when the rating's value changes.
 * @event {{ phase: 'start' | 'move' | 'end', value: number }} wa-hover - Emitted when the user hovers over a value. The
 *  `phase` property indicates when hovering starts, moves to a new value, or ends. The `value` property tells what the
 *  rating's value would be if the user were to commit to the hovered value.
 * @event wa-invalid - Emitted when the form control has been checked for validity and its constraints aren't satisfied.
 *
 * @csspart base - The component's base wrapper.
 *
 * @cssproperty --symbol-color - The inactive color for symbols.
 * @cssproperty --symbol-color-active - The active color for symbols.
 * @cssproperty --symbol-spacing - The spacing to use around symbols.
 */
@customElement("wa-rating")
export default class WaRating extends WebAwesomeFormAssociatedElement {
  static css = [sizeStyles, styles];

  static get validators() {
    return [...super.validators, RequiredValidator()];
  }

  assumeInteractionOn = ["change"];

  private readonly localize = new LocalizeController(this);

  connectedCallback() {
    super.connectedCallback();
    this.setAttribute("role", "slider");
    this.setAttribute("aria-valuenow", String(this.value));
    this.setAttribute("aria-valuemin", "0");
    this.setAttribute("aria-valuemax", String(this.max));
    this.setAttribute("aria-disabled", this.disabled ? "true" : "false");
    this.setAttribute("aria-readonly", this.readonly ? "true" : "false");

    if (this.label) {
      this.setAttribute("aria-label", this.label);
    }

    if (!this.disabled && !this.readonly) {
      this.tabIndex = 0;
    } else {
      this.tabIndex = -1;
    }

    this.addEventListener("click", this.handleClick);
    this.addEventListener("keydown", this.handleKeyDown);
    this.addEventListener("pointerenter", this.handlePointerEnter);
    this.addEventListener("pointermove", this.handlePointerMove);
    this.addEventListener("pointerleave", this.handlePointerLeave);
    this.addEventListener("pointerdown", this.handlePointerDown);
    this.addEventListener("pointerup", this.handlePointerUp);
  }

  disconnectedCallback() {
    super.disconnectedCallback();
    this.removeEventListener("click", this.handleClick);
    this.removeEventListener("keydown", this.handleKeyDown);
    this.removeEventListener("pointerenter", this.handlePointerEnter);
    this.removeEventListener("pointermove", this.handlePointerMove);
    this.removeEventListener("pointerleave", this.handlePointerLeave);
    this.removeEventListener("pointerdown", this.handlePointerDown);
    this.removeEventListener("pointerup", this.handlePointerUp);
  }

  updated(changedProperties: PropertyValues<this>) {
    super.updated(changedProperties);

    if (changedProperties.has("value")) {
      this.setAttribute("aria-valuenow", String(this.value));
    }

    if (changedProperties.has("max")) {
      this.setAttribute("aria-valuemax", String(this.max));
    }

    if (changedProperties.has("disabled")) {
      this.setAttribute("aria-disabled", this.disabled ? "true" : "false");
      this.tabIndex = this.disabled || this.readonly ? -1 : 0;
    }

    if (changedProperties.has("readonly")) {
      this.setAttribute("aria-readonly", this.readonly ? "true" : "false");
      this.tabIndex = this.disabled || this.readonly ? -1 : 0;
    }

    if (changedProperties.has("label")) {
      if (this.label) {
        this.setAttribute("aria-label", this.label);
      } else {
        this.removeAttribute("aria-label");
      }
    }
  }

  @state() private hoverValue = 0;
  @state() private isHovering = false;

  /** The name of the rating, submitted as a name/value pair with form data. */
  @property() name: string | null = null;

  /** A label that describes the rating to assistive devices. */
  @property() label = "";

  /** The current rating. */
  @property({ type: Number }) value = 0;

  /** The default value of the form control. Used to reset the rating to its initial value. */
  @property({ type: Number, attribute: "default-value" }) defaultValue = 0;

  /** The highest rating to show. */
  @property({ type: Number }) max = 5;

  /**
   * The precision at which the rating will increase and decrease. For example, to allow half-star ratings, set this
   * attribute to `0.5`.
   */
  @property({ type: Number }) precision = 1;

  /** Makes the rating readonly. */
  @property({ type: Boolean, reflect: true }) readonly = false;

  /** Disables the rating. */
  @property({ type: Boolean }) declare disabled: boolean;

  /** Makes the rating a required field. */
  @property({ type: Boolean, reflect: true }) required = false;

  /**
   * A function that customizes the symbol to be rendered. The first and only argument is the rating's current value.
   * The function should return a string containing trusted HTML of the symbol to render at the specified value. Works
   * well with `<wa-icon>` elements.
   */
  @property() getSymbol: (value: number, isSelected: boolean) => string = (
    _value,
    isSelected,
  ) => {
    return isSelected
      ? '<wa-icon name="star" library="system" variant="solid"></wa-icon>'
      : '<wa-icon name="star" library="system" variant="regular"></wa-icon>';
  };

  /** The component's size. */
  @property({ reflect: true }) size:
    "xs" | "s" | "m" | "l" | "xl" | "small" | "medium" | "large" = "m";

  @watch("size")
  handleSizeChange() {
    warnDeprecatedSize(this.localName, this.size);
  }

  private getValueFromPointerPosition(event: PointerEvent) {
    return this.getValueFromXCoordinate(event.clientX);
  }

  private getValueFromXCoordinate(coordinate: number) {
    const isRtl = this.localize.dir() === "rtl";
    const { left, right, width } = this.getBoundingClientRect();
    const value = isRtl
      ? this.roundToPrecision(
          ((right - coordinate) / width) * this.max,
          this.precision,
        )
      : this.roundToPrecision(
          ((coordinate - left) / width) * this.max,
          this.precision,
        );

    return clamp(value, 0, this.max);
  }

  private handleClick = (event: MouseEvent) => {
    if (this.disabled) {
      return;
    }

    this.setRatingValue(this.getValueFromXCoordinate(event.clientX));
    this.updateComplete.then(() => {
      this.dispatchEvent(
        new Event("change", { bubbles: true, composed: true }),
      );
    });
  };

  private setRatingValue(newValue: number) {
    if (this.disabled || this.readonly) {
      return;
    }

    this.value = newValue === this.value ? 0 : newValue;
    this.isHovering = false;
  }

  private handleKeyDown = (event: KeyboardEvent) => {
    const isLtr = this.matches(":dir(ltr)");
    const isRtl = this.localize.dir() === "rtl";
    const oldValue = this.value;

    if (this.disabled || this.readonly) {
      return;
    }

    if (
      event.key === "ArrowDown" ||
      (isLtr && event.key === "ArrowLeft") ||
      (isRtl && event.key === "ArrowRight")
    ) {
      const decrement = event.shiftKey ? 1 : this.precision;
      this.value = Math.max(0, this.value - decrement);
      event.preventDefault();
    }

    if (
      event.key === "ArrowUp" ||
      (isLtr && event.key === "ArrowRight") ||
      (isRtl && event.key === "ArrowLeft")
    ) {
      const increment = event.shiftKey ? 1 : this.precision;
      this.value = Math.min(this.max, this.value + increment);
      event.preventDefault();
    }

    if (event.key === "Home") {
      this.value = 0;
      event.preventDefault();
    }

    if (event.key === "End") {
      this.value = this.max;
      event.preventDefault();
    }

    if (this.value !== oldValue) {
      this.updateComplete.then(() => {
        this.dispatchEvent(
          new Event("change", { bubbles: true, composed: true }),
        );
      });
    }
  };

  private handlePointerEnter = (event: PointerEvent) => {
    this.isHovering = true;
    this.hoverValue = this.getValueFromPointerPosition(event);
  };

  private handlePointerMove = (event: PointerEvent) => {
    this.hoverValue = this.getValueFromPointerPosition(event);
  };

  private handlePointerLeave = () => {
    this.isHovering = false;
  };

  private handlePointerDown = (event: PointerEvent) => {
    if (event.button !== 0) {
      return;
    }

    this.isHovering = true;
    this.hoverValue = this.getValueFromPointerPosition(event);

    // Capture the pointer so pointermove/pointerup fire even outside the element (e.g. touch drag)
    this.setPointerCapture(event.pointerId);

    // Prevent scrolling on touch
    event.preventDefault();
  };

  private handlePointerUp = (event: PointerEvent) => {
    this.releasePointerCapture(event.pointerId);
    this.isHovering = false;
  };

  private roundToPrecision(numberToRound: number, precision = 0.5) {
    const multiplier = 1 / precision;
    return Math.ceil(numberToRound * multiplier) / multiplier;
  }

  @watch("hoverValue")
  handleHoverValueChange() {
    this.dispatchEvent(
      new WaHoverEvent({
        phase: "move",
        value: this.hoverValue,
      }),
    );
  }

  @watch("isHovering")
  handleIsHoveringChange() {
    this.dispatchEvent(
      new WaHoverEvent({
        phase: this.isHovering ? "start" : "end",
        value: this.hoverValue,
      }),
    );
  }

  formResetCallback() {
    this.value = this.defaultValue;
    super.formResetCallback();
  }

  render() {
    const isRtl = this.hasUpdated ? this.localize.dir() === "rtl" : this.dir;
    const counter = Array.from(Array(this.max).keys());
    let displayValue = 0;

    if (this.disabled || this.readonly) {
      displayValue = this.value;
    } else {
      displayValue = this.isHovering ? this.hoverValue : this.value;
    }

    return html`
      <div
        part="base"
        class=${classMap({
          rating: true,
          "rating-readonly": this.readonly,
          "rating-disabled": this.disabled,
        })}
      >
        <span class="symbols">
          ${counter.map((index) => {
            const isSelected = displayValue >= index + 1;

            if (displayValue > index && displayValue < index + 1) {
              // Users can click the current value to clear the rating. When this happens, we set this.isHovering to
              // false to prevent the hover state from confusing them as they move the mouse out of the control. This
              // extra mouseenter will reinstate it if they happen to mouse over an adjacent symbol.
              return html`
                <span
                  class=${classMap({
                    symbol: true,
                    "partial-symbol-container": true,
                    "symbol-hover":
                      this.isHovering && Math.ceil(displayValue) === index + 1,
                  })}
                  role="presentation"
                >
                  <div
                    style=${styleMap({
                      clipPath: isRtl
                        ? `inset(0 ${(displayValue - index) * 100}% 0 0)`
                        : `inset(0 0 0 ${(displayValue - index) * 100}%)`,
                    })}
                  >
                    ${unsafeHTML(this.getSymbol(index + 1, false))}
                  </div>
                  <div
                    class="partial-filled"
                    style=${styleMap({
                      clipPath: isRtl
                        ? `inset(0 0 0 ${100 - (displayValue - index) * 100}%)`
                        : `inset(0 ${100 - (displayValue - index) * 100}% 0 0)`,
                    })}
                  >
                    ${unsafeHTML(this.getSymbol(index + 1, true))}
                  </div>
                </span>
              `;
            }

            return html`
              <span
                class=${classMap({
                  symbol: true,
                  "symbol-hover":
                    this.isHovering && Math.ceil(displayValue) === index + 1,
                  "symbol-active": displayValue >= index + 1,
                })}
                role="presentation"
              >
                ${unsafeHTML(this.getSymbol(index + 1, isSelected))}
              </span>
            `;
          })}
        </span>
      </div>
    `;
  }
}

declare global {
  interface HTMLElementTagNameMap {
    "wa-rating": WaRating;
  }
}

`````
