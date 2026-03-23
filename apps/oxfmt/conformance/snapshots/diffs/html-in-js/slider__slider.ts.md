# slider/slider.ts

> `@decorator` + union type: https://github.com/oxc-project/oxc/issues/20519

## Option 1

`````json
{"printWidth":80}
`````

### Diff

`````diff
===================================================================
--- prettier
+++ oxfmt
@@ -198,10 +198,13 @@
   @property({ attribute: "tooltip-distance", type: Number }) tooltipDistance =
     8;
 
   /** The placement of the tooltip in reference to the slider's thumb. */
-  @property({ attribute: "tooltip-placement", reflect: true })
-  tooltipPlacement: "top" | "right" | "bottom" | "left" = "top";
+  @property({ attribute: "tooltip-placement", reflect: true }) tooltipPlacement:
+    | "top"
+    | "right"
+    | "bottom"
+    | "left" = "top";
 
   /** Draws markers at each step along the slider. */
   @property({ attribute: "with-markers", type: Boolean }) withMarkers = false;
 

`````

### Actual (oxfmt)

`````ts
import type { PropertyValues } from "lit";
import { html } from "lit";
import { customElement, property, query, state } from "lit/decorators.js";
import { classMap } from "lit/directives/class-map.js";
import { styleMap } from "lit/directives/style-map.js";
import { DraggableElement } from "../../internal/drag.js";
import { clamp } from "../../internal/math.js";
import { HasSlotController } from "../../internal/slot.js";
import { submitOnEnter } from "../../internal/submit-on-enter.js";
import { SliderValidator } from "../../internal/validators/slider-validator.js";
import { WebAwesomeFormAssociatedElement } from "../../internal/webawesome-form-associated-element.js";
import formControlStyles from "../../styles/component/form-control.styles.js";
import sizeStyles from "../../styles/component/size.styles.js";
import { LocalizeController } from "../../utilities/localize.js";
import "../tooltip/tooltip.js";
import type WaTooltip from "../tooltip/tooltip.js";
import styles from "./slider.styles.js";

/**
 * <wa-slider>
 *
 * @summary Ranges allow the user to select a single value within a given range using a slider.
 * @documentation https://webawesome.com/docs/components/range
 * @status stable
 * @since 2.0
 *
 * @dependency wa-tooltip
 *
 * @slot label - The slider label. Alternatively, you can use the `label` attribute.
 * @slot hint - Text that describes how to use the input. Alternatively, you can use the `hint` attribute.
 *  instead.
 * @slot reference - One or more reference labels to show visually below the slider.
 *
 * @event blur - Emitted when the control loses focus.
 * @event change - Emitted when an alteration to the control's value is committed by the user.
 * @event focus - Emitted when the control gains focus.
 * @event input - Emitted when the control receives input.
 * @event wa-invalid - Emitted when the form control has been checked for validity and its constraints aren't satisfied.
 *
 * @csspart label - The element that contains the sliders's label.
 * @csspart hint - The element that contains the slider's description.
 * @csspart slider - The focusable element with `role="slider"`. Contains the track and reference slot.
 * @csspart track - The slider's track.
 * @csspart indicator - The colored indicator that shows from the start of the slider to the current value.
 * @csspart markers - The container that holds all the markers when `with-markers` is used.
 * @csspart marker - The individual markers that are shown when `with-markers` is used.
 * @csspart references - The container that holds references that get slotted in.
 * @csspart thumb - The slider's thumb.
 * @csspart thumb-min - The min value thumb in a range slider.
 * @csspart thumb-max - The max value thumb in a range slider.
 * @csspart tooltip - The tooltip, a `<wa-tooltip>` element.
 * @csspart tooltip__tooltip - The tooltip's `tooltip` part.
 * @csspart tooltip__content - The tooltip's `content` part.
 * @csspart tooltip__arrow - The tooltip's `arrow` part.
 *
 * @cssstate disabled - Applied when the slider is disabled.
 * @cssstate dragging - Applied when the slider is being dragged.
 * @cssstate focused - Applied when the slider has focus.
 * @cssstate user-valid - Applied when the slider is valid and the user has sufficiently interacted with it.
 * @cssstate user-invalid - Applied when the slider is invalid and the user has sufficiently interacted with it.
 *
 * @cssproperty [--track-size=0.75em] - The height or width of the slider's track.
 * @cssproperty [--marker-width=0.1875em] - The width of each individual marker.
 * @cssproperty [--marker-height=0.1875em] - The height of each individual marker.
 * @cssproperty [--thumb-width=1.25em] - The width of the thumb.
 * @cssproperty [--thumb-height=1.25em] - The height of the thumb.
 */
@customElement("wa-slider")
export default class WaSlider extends WebAwesomeFormAssociatedElement {
  static formAssociated = true;
  static observeSlots = true;
  static css = [sizeStyles, formControlStyles, styles];

  static get validators() {
    return [...super.validators, SliderValidator()];
  }

  private draggableTrack: DraggableElement;
  private draggableThumbMin: DraggableElement | null = null;
  private draggableThumbMax: DraggableElement | null = null;
  private readonly hasSlotController = new HasSlotController(
    this,
    "hint",
    "label",
  );
  private readonly localize = new LocalizeController(this);
  private trackBoundingClientRect: DOMRect;
  private valueWhenDraggingStarted: number | undefined | null;
  private activeThumb: "min" | "max" | null = null;
  private lastTrackPosition: number | null = null; // Track last position for direction detection

  protected get focusableAnchor() {
    return this.isRange ? this.thumbMin || this.slider : this.slider;
  }

  /** Override validation target to point to the focusable element */
  get validationTarget() {
    return this.focusableAnchor;
  }

  @query("#slider") slider: HTMLElement;
  @query("#thumb") thumb: HTMLElement;
  @query("#thumb-min") thumbMin: HTMLElement;
  @query("#thumb-max") thumbMax: HTMLElement;
  @query("#track") track: HTMLElement;
  @query("#tooltip") tooltip: WaTooltip;

  /**
   * The slider's label. If you need to provide HTML in the label, use the `label` slot instead.
   */
  @property() label: string = "";

  /** The slider hint. If you need to display HTML, use the hint slot instead. */
  @property({ attribute: "hint" }) hint = "";

  /** The name of the slider. This will be submitted with the form as a name/value pair. */
  @property({ reflect: true }) name: string;

  /** The minimum value of a range selection. Used only when range attribute is set. */
  @property({ type: Number, attribute: "min-value" }) minValue = 0;

  /** The maximum value of a range selection. Used only when range attribute is set. */
  @property({ type: Number, attribute: "max-value" }) maxValue = 50;

  /** The default value of the form control. Primarily used for resetting the form control. */
  @property({ attribute: "value", reflect: true, type: Number })
  defaultValue: number =
    this.getAttribute("value") == null
      ? this.minValue
      : Number(this.getAttribute("value"));

  private _value: number | null = null;

  /** The current value of the slider, submitted as a name/value pair with form data. */
  get value(): number {
    if (this.valueHasChanged) {
      const val = this._value ?? this.minValue ?? 0;
      return clamp(val, this.min, this.max);
    }

    const val = this._value ?? this.defaultValue;
    return clamp(val, this.min, this.max);
  }

  @state()
  set value(val: number | null) {
    val = Number(val) ?? this.minValue;

    if (this._value === val) {
      return;
    }

    this.valueHasChanged = true;
    this._value = val;
  }

  /** Converts the slider to a range slider with two thumbs. */
  @property({ type: Boolean, reflect: true }) range = false;

  /** Get if this is a range slider */
  get isRange(): boolean {
    return this.range;
  }

  /** Disables the slider. */
  @property({ type: Boolean }) disabled = false;

  /** Makes the slider a read-only field. */
  @property({ type: Boolean, reflect: true }) readonly = false;

  /** The orientation of the slider. */
  @property({ reflect: true }) orientation: "horizontal" | "vertical" =
    "horizontal";

  /** The slider's size. */
  @property({ reflect: true }) size: "small" | "medium" | "large" = "medium";

  /** The starting value from which to draw the slider's fill, which is based on its current value. */
  @property({ attribute: "indicator-offset", type: Number })
  indicatorOffset: number;

  /** The minimum value allowed. */
  @property({ type: Number }) min: number = 0;

  /** The maximum value allowed. */
  @property({ type: Number }) max: number = 100;

  /** The granularity the value must adhere to when incrementing and decrementing. */
  @property({ type: Number }) step: number = 1;

  /** Makes the slider a required field. */
  @property({ type: Boolean, reflect: true }) required = false;

  /** Tells the browser to focus the slider when the page loads or a dialog is shown. */
  @property({ type: Boolean }) autofocus: boolean;

  /** The distance of the tooltip from the slider's thumb. */
  @property({ attribute: "tooltip-distance", type: Number }) tooltipDistance =
    8;

  /** The placement of the tooltip in reference to the slider's thumb. */
  @property({ attribute: "tooltip-placement", reflect: true }) tooltipPlacement:
    | "top"
    | "right"
    | "bottom"
    | "left" = "top";

  /** Draws markers at each step along the slider. */
  @property({ attribute: "with-markers", type: Boolean }) withMarkers = false;

  /** Draws a tooltip above the thumb when the control has focus or is dragged. */
  @property({ attribute: "with-tooltip", type: Boolean }) withTooltip = false;

  /**
   * A custom formatting function to apply to the value. This will be shown in the tooltip and announced by screen
   * readers. Must be set with JavaScript. Property only.
   */
  @property({ attribute: false }) valueFormatter: (value: number) => string;

  firstUpdated() {
    // Setup dragging based on range or single thumb mode
    if (this.isRange) {
      // Enable dragging on both thumbs for range slider
      this.draggableThumbMin = new DraggableElement(this.thumbMin, {
        start: () => {
          this.activeThumb = "min";
          this.trackBoundingClientRect = this.track.getBoundingClientRect();
          this.valueWhenDraggingStarted = this.minValue;
          this.customStates.set("dragging", true);
          this.showRangeTooltips();
        },
        move: (x, y) => {
          this.setThumbValueFromCoordinates(x, y, "min");
        },
        stop: () => {
          if (this.minValue !== this.valueWhenDraggingStarted) {
            this.updateComplete.then(() => {
              this.dispatchEvent(
                new Event("change", { bubbles: true, composed: true }),
              );
            });
            this.hasInteracted = true;
          }
          this.hideRangeTooltips();
          this.customStates.set("dragging", false);
          this.valueWhenDraggingStarted = undefined;
          this.activeThumb = null;
        },
      });

      this.draggableThumbMax = new DraggableElement(this.thumbMax, {
        start: () => {
          this.activeThumb = "max";
          this.trackBoundingClientRect = this.track.getBoundingClientRect();
          this.valueWhenDraggingStarted = this.maxValue;
          this.customStates.set("dragging", true);
          this.showRangeTooltips();
        },
        move: (x, y) => {
          this.setThumbValueFromCoordinates(x, y, "max");
        },
        stop: () => {
          if (this.maxValue !== this.valueWhenDraggingStarted) {
            this.updateComplete.then(() => {
              this.dispatchEvent(
                new Event("change", { bubbles: true, composed: true }),
              );
            });
            this.hasInteracted = true;
          }
          this.hideRangeTooltips();
          this.customStates.set("dragging", false);
          this.valueWhenDraggingStarted = undefined;
          this.activeThumb = null;
        },
      });

      // Enable track dragging for finding the closest thumb
      this.draggableTrack = new DraggableElement(this.track, {
        start: (x, y) => {
          this.trackBoundingClientRect = this.track.getBoundingClientRect();

          // When a drag starts, we need to determine which thumb to move
          // If the thumbs are in nearly the same position, we prioritize the one that's already active
          // or the one that received focus most recently
          if (this.activeThumb) {
            // Keep using the already active thumb (useful for keyboard interactions)
            this.valueWhenDraggingStarted =
              this.activeThumb === "min" ? this.minValue : this.maxValue;
          } else {
            // Otherwise select by closest distance
            const value = this.getValueFromCoordinates(x, y);
            const minDistance = Math.abs(value - this.minValue);
            const maxDistance = Math.abs(value - this.maxValue);

            if (minDistance === maxDistance) {
              // If distances are equal, prioritize the max thumb when value is higher than both thumbs
              // and min thumb when value is lower than both thumbs
              if (value > this.maxValue) {
                this.activeThumb = "max";
              } else if (value < this.minValue) {
                this.activeThumb = "min";
              } else {
                // If the value is between the thumbs and they're at the same distance,
                // prioritize the thumb that's in the direction of movement
                const isRtl = this.localize.dir() === "rtl";
                const isVertical = this.orientation === "vertical";
                const position = isVertical ? y : x;
                const previousPosition = this.lastTrackPosition || position;
                this.lastTrackPosition = position;

                // Determine direction of movement
                const movingForward =
                  (position > previousPosition !== isRtl && !isVertical) ||
                  (position < previousPosition && isVertical);

                this.activeThumb = movingForward ? "max" : "min";
              }
            } else {
              // Select the closest thumb
              this.activeThumb = minDistance <= maxDistance ? "min" : "max";
            }

            this.valueWhenDraggingStarted =
              this.activeThumb === "min" ? this.minValue : this.maxValue;
          }

          this.customStates.set("dragging", true);
          this.setThumbValueFromCoordinates(x, y, this.activeThumb);
          this.showRangeTooltips();
        },
        move: (x, y) => {
          if (this.activeThumb) {
            this.setThumbValueFromCoordinates(x, y, this.activeThumb);
          }
        },
        stop: () => {
          if (this.activeThumb) {
            const currentValue =
              this.activeThumb === "min" ? this.minValue : this.maxValue;
            if (currentValue !== this.valueWhenDraggingStarted) {
              this.updateComplete.then(() => {
                this.dispatchEvent(
                  new Event("change", { bubbles: true, composed: true }),
                );
              });
              this.hasInteracted = true;
            }
          }
          this.hideRangeTooltips();
          this.customStates.set("dragging", false);
          this.valueWhenDraggingStarted = undefined;
          this.activeThumb = null;
        },
      });
    } else {
      // Single thumb mode - original behavior
      this.draggableTrack = new DraggableElement(this.slider, {
        start: (x, y) => {
          this.trackBoundingClientRect = this.track.getBoundingClientRect();
          this.valueWhenDraggingStarted = this.value;
          this.customStates.set("dragging", true);
          this.setValueFromCoordinates(x, y);
          this.showTooltip();
        },
        move: (x, y) => {
          this.setValueFromCoordinates(x, y);
        },
        stop: () => {
          if (this.value !== this.valueWhenDraggingStarted) {
            this.updateComplete.then(() => {
              this.dispatchEvent(
                new Event("change", { bubbles: true, composed: true }),
              );
            });

            this.hasInteracted = true;
          }
          this.hideTooltip();
          this.customStates.set("dragging", false);
          this.valueWhenDraggingStarted = undefined;
        },
      });
    }
  }

  updated(changedProperties: PropertyValues<this>) {
    // Handle range mode changes
    if (changedProperties.has("range")) {
      this.requestUpdate();
    }

    if (this.isRange) {
      // Handle min/max values for range mode
      if (
        changedProperties.has("minValue") ||
        changedProperties.has("maxValue")
      ) {
        // Ensure min doesn't exceed max
        this.minValue = clamp(this.minValue, this.min, this.maxValue);
        this.maxValue = clamp(this.maxValue, this.minValue, this.max);
        // Update form value
        this.updateFormValue();
      }
    } else {
      // Handle value for single thumb mode
      if (changedProperties.has("value")) {
        this.setValue(String(this.value));
      }
    }

    // Handle min/max
    if (changedProperties.has("min") || changedProperties.has("max")) {
      if (this.isRange) {
        this.minValue = clamp(this.minValue, this.min, this.max);
        this.maxValue = clamp(this.maxValue, this.min, this.max);
      }
    }

    // Handle disabled
    if (changedProperties.has("disabled")) {
      this.customStates.set("disabled", this.disabled);
    }

    // Disable dragging when disabled or readonly
    if (
      changedProperties.has("disabled") ||
      changedProperties.has("readonly")
    ) {
      const enabled = !(this.disabled || this.readonly);

      if (this.isRange) {
        if (this.draggableThumbMin) this.draggableThumbMin.toggle(enabled);
        if (this.draggableThumbMax) this.draggableThumbMax.toggle(enabled);
      }

      if (this.draggableTrack) {
        this.draggableTrack.toggle(enabled);
      }
    }

    super.updated(changedProperties);
  }

  /** @internal Called when a containing fieldset is disabled. */
  formDisabledCallback(isDisabled: boolean) {
    this.disabled = isDisabled;
  }

  /** @internal Called when the form is reset. */
  formResetCallback() {
    if (this.isRange) {
      this.minValue = parseFloat(
        this.getAttribute("min-value") ?? String(this.min),
      );
      this.maxValue = parseFloat(
        this.getAttribute("max-value") ?? String(this.max),
      );
    } else {
      this._value = null;
      this.defaultValue =
        this.defaultValue ??
        parseFloat(this.getAttribute("value") ?? String(this.min));
    }
    this.valueHasChanged = false;
    this.hasInteracted = false;
    super.formResetCallback();
  }

  /** Clamps a number to min/max while ensuring it's a valid step interval. */
  private clampAndRoundToStep(value: number) {
    const stepPrecision = (String(this.step).split(".")[1] || "").replace(
      /0+$/g,
      "",
    ).length;

    // Ensure we're working with numbers (in case the user passes strings to the respective properties)
    const step = Number(this.step);
    const min = Number(this.min);
    const max = Number(this.max);

    value = Math.round(value / step) * step;
    value = clamp(value, min, max);

    return parseFloat(value.toFixed(stepPrecision));
  }

  /** Given a value, returns its percentage within a range of min/max. */
  private getPercentageFromValue(value: number) {
    return ((value - this.min) / (this.max - this.min)) * 100;
  }

  /** Converts coordinates to slider value */
  private getValueFromCoordinates(x: number, y: number) {
    const isRtl = this.localize.dir() === "rtl";
    const isVertical = this.orientation === "vertical";
    const { top, right, bottom, left, height, width } =
      this.trackBoundingClientRect;
    const pointerPosition = isVertical ? y : x;
    const sliderCoords = isVertical
      ? { start: top, end: bottom, size: height }
      : { start: left, end: right, size: width };
    const relativePosition = isVertical
      ? sliderCoords.end - pointerPosition
      : isRtl
        ? sliderCoords.end - pointerPosition
        : pointerPosition - sliderCoords.start;
    const percentage = relativePosition / sliderCoords.size;
    return this.clampAndRoundToStep(
      this.min + (this.max - this.min) * percentage,
    );
  }

  private handleBlur() {
    // Only hide tooltips if neither thumb has focus
    if (this.isRange) {
      // Allow a subsequent focus event to fire on the other thumb if the user is tabbing
      requestAnimationFrame(() => {
        const focusedElement = this.shadowRoot?.activeElement;
        const thumbHasFocus =
          focusedElement === this.thumbMin || focusedElement === this.thumbMax;
        if (!thumbHasFocus) {
          this.hideRangeTooltips();
        }
      });
    } else {
      this.hideTooltip();
    }
    this.customStates.set("focused", false);
    this.dispatchEvent(
      new FocusEvent("blur", { bubbles: true, composed: true }),
    );
  }

  private handleFocus(event: FocusEvent) {
    const target = event.target as HTMLElement;

    // Handle focus for specific thumbs in range mode
    if (this.isRange) {
      if (target === this.thumbMin) {
        this.activeThumb = "min";
      } else if (target === this.thumbMax) {
        this.activeThumb = "max";
      }
      this.showRangeTooltips();
    } else {
      this.showTooltip();
    }

    this.customStates.set("focused", true);
    this.dispatchEvent(
      new FocusEvent("focus", { bubbles: true, composed: true }),
    );
  }

  private handleKeyDown(event: KeyboardEvent) {
    const isRtl = this.localize.dir() === "rtl";
    const target = event.target as HTMLElement;

    if (this.disabled || this.readonly) return;

    // For range slider, determine which thumb is active
    if (this.isRange) {
      if (target === this.thumbMin) {
        this.activeThumb = "min";
      } else if (target === this.thumbMax) {
        this.activeThumb = "max";
      }

      if (!this.activeThumb) return;
    }

    // Get current value based on slider mode
    const current = this.isRange
      ? this.activeThumb === "min"
        ? this.minValue
        : this.maxValue
      : this.value;

    let newValue = current;

    // Handle key presses
    switch (event.key) {
      // Increase
      case "ArrowUp":
      case isRtl ? "ArrowLeft" : "ArrowRight":
        event.preventDefault();
        newValue = this.clampAndRoundToStep(current + this.step);
        break;

      // Decrease
      case "ArrowDown":
      case isRtl ? "ArrowRight" : "ArrowLeft":
        event.preventDefault();
        newValue = this.clampAndRoundToStep(current - this.step);
        break;

      // Minimum value
      case "Home":
        event.preventDefault();
        newValue =
          this.isRange && this.activeThumb === "min"
            ? this.min
            : this.isRange
              ? this.minValue
              : this.min;
        break;

      // Maximum value
      case "End":
        event.preventDefault();
        newValue =
          this.isRange && this.activeThumb === "max"
            ? this.max
            : this.isRange
              ? this.maxValue
              : this.max;
        break;

      // Move up 10%
      case "PageUp":
        event.preventDefault();
        const stepUp = Math.max(
          current + (this.max - this.min) / 10,
          current + this.step, // make sure we at least move up to the next step
        );
        newValue = this.clampAndRoundToStep(stepUp);
        break;

      // Move down 10%
      case "PageDown":
        event.preventDefault();
        const stepDown = Math.min(
          current - (this.max - this.min) / 10,
          current - this.step, // make sure we at least move down to the previous step
        );
        newValue = this.clampAndRoundToStep(stepDown);
        break;

      // Handle form submission on Enter
      case "Enter":
        submitOnEnter(event, this);
        return;
    }

    // If no value change, exit early
    if (newValue === current) return;

    // Apply the new value with appropriate constraints
    if (this.isRange) {
      if (this.activeThumb === "min") {
        if (newValue > this.maxValue) {
          // If min thumb exceeds max thumb, move both
          this.maxValue = newValue;
          this.minValue = newValue;
        } else {
          this.minValue = Math.max(this.min, newValue);
        }
      } else {
        if (newValue < this.minValue) {
          // If max thumb goes below min thumb, move both
          this.minValue = newValue;
          this.maxValue = newValue;
        } else {
          this.maxValue = Math.min(this.max, newValue);
        }
      }
      this.updateFormValue();
    } else {
      this.value = clamp(newValue, this.min, this.max);
    }

    // Dispatch events
    this.updateComplete.then(() => {
      this.dispatchEvent(
        new InputEvent("input", { bubbles: true, composed: true }),
      );
      this.dispatchEvent(
        new Event("change", { bubbles: true, composed: true }),
      );
    });
    this.hasInteracted = true;
  }

  private handleLabelPointerDown(event: PointerEvent) {
    event.preventDefault();

    if (!this.disabled) {
      if (this.isRange) {
        this.thumbMin?.focus();
      } else {
        this.slider.focus();
      }
    }
  }

  private setValueFromCoordinates(x: number, y: number) {
    const oldValue = this.value;
    this.value = this.getValueFromCoordinates(x, y);

    // Dispatch input events when the value changes by dragging
    if (this.value !== oldValue) {
      this.updateComplete.then(() => {
        this.dispatchEvent(
          new InputEvent("input", { bubbles: true, composed: true }),
        );
      });
    }
  }

  private setThumbValueFromCoordinates(
    x: number,
    y: number,
    thumb: "min" | "max",
  ) {
    const value = this.getValueFromCoordinates(x, y);
    const oldValue = thumb === "min" ? this.minValue : this.maxValue;

    if (thumb === "min") {
      // If min thumb is being dragged and would exceed max thumb
      if (value > this.maxValue) {
        // Move both thumbs, keeping their distance at 0
        this.maxValue = value;
        this.minValue = value;
      } else {
        // Normal case - just move min thumb
        this.minValue = Math.max(this.min, value);
      }
    } else {
      // thumb === 'max'
      // If max thumb is being dragged and would go below min thumb
      if (value < this.minValue) {
        // Move both thumbs, keeping their distance at 0
        this.minValue = value;
        this.maxValue = value;
      } else {
        // Normal case - just move max thumb
        this.maxValue = Math.min(this.max, value);
      }
    }

    // Dispatch input events
    if (oldValue !== (thumb === "min" ? this.minValue : this.maxValue)) {
      this.updateFormValue();
      this.updateComplete.then(() => {
        this.dispatchEvent(
          new InputEvent("input", { bubbles: true, composed: true }),
        );
      });
    }
  }

  private showTooltip() {
    if (this.withTooltip && this.tooltip) {
      this.tooltip.open = true;
    }
  }

  private hideTooltip() {
    if (this.withTooltip && this.tooltip) {
      this.tooltip.open = false;
    }
  }

  private showRangeTooltips() {
    if (!this.withTooltip) return;

    // Show only the active tooltip, hide the other
    const tooltipMin = this.shadowRoot?.getElementById(
      "tooltip-thumb-min",
    ) as WaTooltip;
    const tooltipMax = this.shadowRoot?.getElementById(
      "tooltip-thumb-max",
    ) as WaTooltip;

    if (this.activeThumb === "min") {
      if (tooltipMin) tooltipMin.open = true;
      if (tooltipMax) tooltipMax.open = false;
    } else if (this.activeThumb === "max") {
      if (tooltipMax) tooltipMax.open = true;
      if (tooltipMin) tooltipMin.open = false;
    }
  }

  private hideRangeTooltips() {
    if (!this.withTooltip) return;

    const tooltipMin = this.shadowRoot?.getElementById(
      "tooltip-thumb-min",
    ) as WaTooltip;
    const tooltipMax = this.shadowRoot?.getElementById(
      "tooltip-thumb-max",
    ) as WaTooltip;

    if (tooltipMin) tooltipMin.open = false;
    if (tooltipMax) tooltipMax.open = false;
  }

  /** Updates the form value submission for range sliders */
  private updateFormValue() {
    if (this.isRange) {
      // Submit both values using FormData for range sliders
      const formData = new FormData();
      formData.append(this.name || "", String(this.minValue));
      formData.append(this.name || "", String(this.maxValue));
      this.setValue(formData);
    }
  }

  /** Sets focus to the slider. */
  public focus() {
    if (this.isRange) {
      this.thumbMin?.focus();
    } else {
      this.slider.focus();
    }
  }

  /** Removes focus from the slider. */
  public blur() {
    if (this.isRange) {
      if (document.activeElement === this.thumbMin) {
        this.thumbMin.blur();
      } else if (document.activeElement === this.thumbMax) {
        this.thumbMax.blur();
      }
    } else {
      this.slider.blur();
    }
  }

  /**
   * Decreases the slider's value by `step`. This is a programmatic change, so `input` and `change` events will not be
   * emitted when this is called.
   */
  public stepDown() {
    if (this.isRange) {
      // If in range mode, default to stepping down the min value
      const newValue = this.clampAndRoundToStep(this.minValue - this.step);
      this.minValue = clamp(newValue, this.min, this.maxValue);
      this.updateFormValue();
    } else {
      const newValue = this.clampAndRoundToStep(this.value - this.step);
      this.value = newValue;
    }
  }

  /**
   * Increases the slider's value by `step`. This is a programmatic change, so `input` and `change` events will not be
   * emitted when this is called.
   */
  public stepUp() {
    if (this.isRange) {
      // If in range mode, default to stepping up the max value
      const newValue = this.clampAndRoundToStep(this.maxValue + this.step);
      this.maxValue = clamp(newValue, this.minValue, this.max);
      this.updateFormValue();
    } else {
      const newValue = this.clampAndRoundToStep(this.value + this.step);
      this.value = newValue;
    }
  }

  render() {
    const hasLabelSlot = this.hasSlotController.test("label");
    const hasHintSlot = this.hasSlotController.test("hint");
    const hasLabel = this.label ? true : !!hasLabelSlot;
    const hasHint = this.hint ? true : !!hasHintSlot;
    const hasReference = this.hasSlotController.test("reference");

    const sliderClasses = classMap({
      small: this.size === "small",
      medium: this.size === "medium",
      large: this.size === "large",
      horizontal: this.orientation === "horizontal",
      vertical: this.orientation === "vertical",
      disabled: this.disabled,
    });

    // Calculate marker positions
    const markers = [];
    if (this.withMarkers) {
      for (let i = this.min; i <= this.max; i += this.step) {
        markers.push(this.getPercentageFromValue(i));
      }
    }

    // Common UI fragments
    const label = html`
      <label
        id="label"
        part="label"
        for=${this.isRange ? "thumb-min" : "text-box"}
        class=${classMap({ vh: !hasLabel, "has-label": hasLabel })}
        @pointerdown=${this.handleLabelPointerDown}
      >
        <slot name="label">${this.label}</slot>
      </label>
    `;

    const hint = html`
      <div
        id="hint"
        part="hint"
        class=${classMap({
          "has-slotted": hasHint,
        })}
      >
        <slot name="hint">${this.hint}</slot>
      </div>
    `;

    const markersTemplate = this.withMarkers
      ? html`
          <div id="markers" part="markers">
            ${markers.map(
              (marker) =>
                html`<span
                  part="marker"
                  class="marker"
                  style=${styleMap({ "--position": `${marker}%` })}
                ></span>`,
            )}
          </div>
        `
      : "";

    const referencesTemplate = hasReference
      ? html`
          <div id="references" part="references" aria-hidden="true">
            <slot name="reference"></slot>
          </div>
        `
      : "";

    // Create tooltip template function
    const createTooltip = (thumbId: string, value: number) =>
      this.withTooltip
        ? html`
            <wa-tooltip
              id=${`tooltip${thumbId !== "thumb" ? "-" + thumbId : ""}`}
              part="tooltip"
              exportparts="
                base:tooltip__base,
                body:tooltip__body,
                arrow:tooltip__arrow
              "
              trigger="manual"
              distance=${this.tooltipDistance}
              placement=${this.tooltipPlacement}
              for=${thumbId}
              activation="manual"
              dir=${this.localize.dir()}
            >
              <span aria-hidden="true">
                ${typeof this.valueFormatter === "function"
                  ? this.valueFormatter(value)
                  : this.localize.number(value)}
              </span>
            </wa-tooltip>
          `
        : "";

    // Render based on mode
    if (this.isRange) {
      // Range slider mode
      const minThumbPosition = clamp(
        this.getPercentageFromValue(this.minValue),
        0,
        100,
      );
      const maxThumbPosition = clamp(
        this.getPercentageFromValue(this.maxValue),
        0,
        100,
      );

      return html`
        ${label}

        <div id="slider" part="slider" class=${sliderClasses}>
          <div id="track" part="track">
            <div
              id="indicator"
              part="indicator"
              style=${styleMap({
                "--start": `${Math.min(minThumbPosition, maxThumbPosition)}%`,
                "--end": `${Math.max(minThumbPosition, maxThumbPosition)}%`,
              })}
            ></div>

            ${markersTemplate}

            <span
              id="thumb-min"
              part="thumb thumb-min"
              style=${styleMap({ "--position": `${minThumbPosition}%` })}
              role="slider"
              aria-valuemin=${this.min}
              aria-valuenow=${this.minValue}
              aria-valuetext=${typeof this.valueFormatter === "function"
                ? this.valueFormatter(this.minValue)
                : this.localize.number(this.minValue)}
              aria-valuemax=${this.max}
              aria-label="${this.label
                ? `${this.label} (minimum value)`
                : "Minimum value"}"
              aria-orientation=${this.orientation}
              aria-disabled=${this.disabled ? "true" : "false"}
              aria-readonly=${this.readonly ? "true" : "false"}
              tabindex=${this.disabled ? -1 : 0}
              @blur=${this.handleBlur}
              @focus=${this.handleFocus}
              @keydown=${this.handleKeyDown}
            ></span>

            <span
              id="thumb-max"
              part="thumb thumb-max"
              style=${styleMap({ "--position": `${maxThumbPosition}%` })}
              role="slider"
              aria-valuemin=${this.min}
              aria-valuenow=${this.maxValue}
              aria-valuetext=${typeof this.valueFormatter === "function"
                ? this.valueFormatter(this.maxValue)
                : this.localize.number(this.maxValue)}
              aria-valuemax=${this.max}
              aria-label="${this.label
                ? `${this.label} (maximum value)`
                : "Maximum value"}"
              aria-orientation=${this.orientation}
              aria-disabled=${this.disabled ? "true" : "false"}
              aria-readonly=${this.readonly ? "true" : "false"}
              tabindex=${this.disabled ? -1 : 0}
              @blur=${this.handleBlur}
              @focus=${this.handleFocus}
              @keydown=${this.handleKeyDown}
            ></span>
          </div>

          ${referencesTemplate} ${hint}
        </div>

        ${createTooltip("thumb-min", this.minValue)}
        ${createTooltip("thumb-max", this.maxValue)}
      `;
    } else {
      // Single thumb mode
      const thumbPosition = clamp(
        this.getPercentageFromValue(this.value),
        0,
        100,
      );
      const indicatorOffsetPosition = clamp(
        this.getPercentageFromValue(
          typeof this.indicatorOffset === "number"
            ? this.indicatorOffset
            : this.min,
        ),
        0,
        100,
      );

      return html`
        ${label}

        <div
          id="slider"
          part="slider"
          class=${sliderClasses}
          role="slider"
          aria-disabled=${this.disabled ? "true" : "false"}
          aria-readonly=${this.disabled ? "true" : "false"}
          aria-orientation=${this.orientation}
          aria-valuemin=${this.min}
          aria-valuenow=${this.value}
          aria-valuetext=${typeof this.valueFormatter === "function"
            ? this.valueFormatter(this.value)
            : this.localize.number(this.value)}
          aria-valuemax=${this.max}
          aria-labelledby="label"
          aria-describedby="hint"
          tabindex=${this.disabled ? -1 : 0}
          @blur=${this.handleBlur}
          @focus=${this.handleFocus}
          @keydown=${this.handleKeyDown}
        >
          <div id="track" part="track">
            <div
              id="indicator"
              part="indicator"
              style=${styleMap({
                "--start": `${indicatorOffsetPosition}%`,
                "--end": `${thumbPosition}%`,
              })}
            ></div>

            ${markersTemplate}
            <span
              id="thumb"
              part="thumb"
              style=${styleMap({ "--position": `${thumbPosition}%` })}
            ></span>
          </div>

          ${referencesTemplate} ${hint}
        </div>

        ${createTooltip("thumb", this.value)}
      `;
    }
  }
}

declare global {
  interface HTMLElementTagNameMap {
    "wa-slider": WaSlider;
  }
}

`````

### Expected (prettier)

`````ts
import type { PropertyValues } from "lit";
import { html } from "lit";
import { customElement, property, query, state } from "lit/decorators.js";
import { classMap } from "lit/directives/class-map.js";
import { styleMap } from "lit/directives/style-map.js";
import { DraggableElement } from "../../internal/drag.js";
import { clamp } from "../../internal/math.js";
import { HasSlotController } from "../../internal/slot.js";
import { submitOnEnter } from "../../internal/submit-on-enter.js";
import { SliderValidator } from "../../internal/validators/slider-validator.js";
import { WebAwesomeFormAssociatedElement } from "../../internal/webawesome-form-associated-element.js";
import formControlStyles from "../../styles/component/form-control.styles.js";
import sizeStyles from "../../styles/component/size.styles.js";
import { LocalizeController } from "../../utilities/localize.js";
import "../tooltip/tooltip.js";
import type WaTooltip from "../tooltip/tooltip.js";
import styles from "./slider.styles.js";

/**
 * <wa-slider>
 *
 * @summary Ranges allow the user to select a single value within a given range using a slider.
 * @documentation https://webawesome.com/docs/components/range
 * @status stable
 * @since 2.0
 *
 * @dependency wa-tooltip
 *
 * @slot label - The slider label. Alternatively, you can use the `label` attribute.
 * @slot hint - Text that describes how to use the input. Alternatively, you can use the `hint` attribute.
 *  instead.
 * @slot reference - One or more reference labels to show visually below the slider.
 *
 * @event blur - Emitted when the control loses focus.
 * @event change - Emitted when an alteration to the control's value is committed by the user.
 * @event focus - Emitted when the control gains focus.
 * @event input - Emitted when the control receives input.
 * @event wa-invalid - Emitted when the form control has been checked for validity and its constraints aren't satisfied.
 *
 * @csspart label - The element that contains the sliders's label.
 * @csspart hint - The element that contains the slider's description.
 * @csspart slider - The focusable element with `role="slider"`. Contains the track and reference slot.
 * @csspart track - The slider's track.
 * @csspart indicator - The colored indicator that shows from the start of the slider to the current value.
 * @csspart markers - The container that holds all the markers when `with-markers` is used.
 * @csspart marker - The individual markers that are shown when `with-markers` is used.
 * @csspart references - The container that holds references that get slotted in.
 * @csspart thumb - The slider's thumb.
 * @csspart thumb-min - The min value thumb in a range slider.
 * @csspart thumb-max - The max value thumb in a range slider.
 * @csspart tooltip - The tooltip, a `<wa-tooltip>` element.
 * @csspart tooltip__tooltip - The tooltip's `tooltip` part.
 * @csspart tooltip__content - The tooltip's `content` part.
 * @csspart tooltip__arrow - The tooltip's `arrow` part.
 *
 * @cssstate disabled - Applied when the slider is disabled.
 * @cssstate dragging - Applied when the slider is being dragged.
 * @cssstate focused - Applied when the slider has focus.
 * @cssstate user-valid - Applied when the slider is valid and the user has sufficiently interacted with it.
 * @cssstate user-invalid - Applied when the slider is invalid and the user has sufficiently interacted with it.
 *
 * @cssproperty [--track-size=0.75em] - The height or width of the slider's track.
 * @cssproperty [--marker-width=0.1875em] - The width of each individual marker.
 * @cssproperty [--marker-height=0.1875em] - The height of each individual marker.
 * @cssproperty [--thumb-width=1.25em] - The width of the thumb.
 * @cssproperty [--thumb-height=1.25em] - The height of the thumb.
 */
@customElement("wa-slider")
export default class WaSlider extends WebAwesomeFormAssociatedElement {
  static formAssociated = true;
  static observeSlots = true;
  static css = [sizeStyles, formControlStyles, styles];

  static get validators() {
    return [...super.validators, SliderValidator()];
  }

  private draggableTrack: DraggableElement;
  private draggableThumbMin: DraggableElement | null = null;
  private draggableThumbMax: DraggableElement | null = null;
  private readonly hasSlotController = new HasSlotController(
    this,
    "hint",
    "label",
  );
  private readonly localize = new LocalizeController(this);
  private trackBoundingClientRect: DOMRect;
  private valueWhenDraggingStarted: number | undefined | null;
  private activeThumb: "min" | "max" | null = null;
  private lastTrackPosition: number | null = null; // Track last position for direction detection

  protected get focusableAnchor() {
    return this.isRange ? this.thumbMin || this.slider : this.slider;
  }

  /** Override validation target to point to the focusable element */
  get validationTarget() {
    return this.focusableAnchor;
  }

  @query("#slider") slider: HTMLElement;
  @query("#thumb") thumb: HTMLElement;
  @query("#thumb-min") thumbMin: HTMLElement;
  @query("#thumb-max") thumbMax: HTMLElement;
  @query("#track") track: HTMLElement;
  @query("#tooltip") tooltip: WaTooltip;

  /**
   * The slider's label. If you need to provide HTML in the label, use the `label` slot instead.
   */
  @property() label: string = "";

  /** The slider hint. If you need to display HTML, use the hint slot instead. */
  @property({ attribute: "hint" }) hint = "";

  /** The name of the slider. This will be submitted with the form as a name/value pair. */
  @property({ reflect: true }) name: string;

  /** The minimum value of a range selection. Used only when range attribute is set. */
  @property({ type: Number, attribute: "min-value" }) minValue = 0;

  /** The maximum value of a range selection. Used only when range attribute is set. */
  @property({ type: Number, attribute: "max-value" }) maxValue = 50;

  /** The default value of the form control. Primarily used for resetting the form control. */
  @property({ attribute: "value", reflect: true, type: Number })
  defaultValue: number =
    this.getAttribute("value") == null
      ? this.minValue
      : Number(this.getAttribute("value"));

  private _value: number | null = null;

  /** The current value of the slider, submitted as a name/value pair with form data. */
  get value(): number {
    if (this.valueHasChanged) {
      const val = this._value ?? this.minValue ?? 0;
      return clamp(val, this.min, this.max);
    }

    const val = this._value ?? this.defaultValue;
    return clamp(val, this.min, this.max);
  }

  @state()
  set value(val: number | null) {
    val = Number(val) ?? this.minValue;

    if (this._value === val) {
      return;
    }

    this.valueHasChanged = true;
    this._value = val;
  }

  /** Converts the slider to a range slider with two thumbs. */
  @property({ type: Boolean, reflect: true }) range = false;

  /** Get if this is a range slider */
  get isRange(): boolean {
    return this.range;
  }

  /** Disables the slider. */
  @property({ type: Boolean }) disabled = false;

  /** Makes the slider a read-only field. */
  @property({ type: Boolean, reflect: true }) readonly = false;

  /** The orientation of the slider. */
  @property({ reflect: true }) orientation: "horizontal" | "vertical" =
    "horizontal";

  /** The slider's size. */
  @property({ reflect: true }) size: "small" | "medium" | "large" = "medium";

  /** The starting value from which to draw the slider's fill, which is based on its current value. */
  @property({ attribute: "indicator-offset", type: Number })
  indicatorOffset: number;

  /** The minimum value allowed. */
  @property({ type: Number }) min: number = 0;

  /** The maximum value allowed. */
  @property({ type: Number }) max: number = 100;

  /** The granularity the value must adhere to when incrementing and decrementing. */
  @property({ type: Number }) step: number = 1;

  /** Makes the slider a required field. */
  @property({ type: Boolean, reflect: true }) required = false;

  /** Tells the browser to focus the slider when the page loads or a dialog is shown. */
  @property({ type: Boolean }) autofocus: boolean;

  /** The distance of the tooltip from the slider's thumb. */
  @property({ attribute: "tooltip-distance", type: Number }) tooltipDistance =
    8;

  /** The placement of the tooltip in reference to the slider's thumb. */
  @property({ attribute: "tooltip-placement", reflect: true })
  tooltipPlacement: "top" | "right" | "bottom" | "left" = "top";

  /** Draws markers at each step along the slider. */
  @property({ attribute: "with-markers", type: Boolean }) withMarkers = false;

  /** Draws a tooltip above the thumb when the control has focus or is dragged. */
  @property({ attribute: "with-tooltip", type: Boolean }) withTooltip = false;

  /**
   * A custom formatting function to apply to the value. This will be shown in the tooltip and announced by screen
   * readers. Must be set with JavaScript. Property only.
   */
  @property({ attribute: false }) valueFormatter: (value: number) => string;

  firstUpdated() {
    // Setup dragging based on range or single thumb mode
    if (this.isRange) {
      // Enable dragging on both thumbs for range slider
      this.draggableThumbMin = new DraggableElement(this.thumbMin, {
        start: () => {
          this.activeThumb = "min";
          this.trackBoundingClientRect = this.track.getBoundingClientRect();
          this.valueWhenDraggingStarted = this.minValue;
          this.customStates.set("dragging", true);
          this.showRangeTooltips();
        },
        move: (x, y) => {
          this.setThumbValueFromCoordinates(x, y, "min");
        },
        stop: () => {
          if (this.minValue !== this.valueWhenDraggingStarted) {
            this.updateComplete.then(() => {
              this.dispatchEvent(
                new Event("change", { bubbles: true, composed: true }),
              );
            });
            this.hasInteracted = true;
          }
          this.hideRangeTooltips();
          this.customStates.set("dragging", false);
          this.valueWhenDraggingStarted = undefined;
          this.activeThumb = null;
        },
      });

      this.draggableThumbMax = new DraggableElement(this.thumbMax, {
        start: () => {
          this.activeThumb = "max";
          this.trackBoundingClientRect = this.track.getBoundingClientRect();
          this.valueWhenDraggingStarted = this.maxValue;
          this.customStates.set("dragging", true);
          this.showRangeTooltips();
        },
        move: (x, y) => {
          this.setThumbValueFromCoordinates(x, y, "max");
        },
        stop: () => {
          if (this.maxValue !== this.valueWhenDraggingStarted) {
            this.updateComplete.then(() => {
              this.dispatchEvent(
                new Event("change", { bubbles: true, composed: true }),
              );
            });
            this.hasInteracted = true;
          }
          this.hideRangeTooltips();
          this.customStates.set("dragging", false);
          this.valueWhenDraggingStarted = undefined;
          this.activeThumb = null;
        },
      });

      // Enable track dragging for finding the closest thumb
      this.draggableTrack = new DraggableElement(this.track, {
        start: (x, y) => {
          this.trackBoundingClientRect = this.track.getBoundingClientRect();

          // When a drag starts, we need to determine which thumb to move
          // If the thumbs are in nearly the same position, we prioritize the one that's already active
          // or the one that received focus most recently
          if (this.activeThumb) {
            // Keep using the already active thumb (useful for keyboard interactions)
            this.valueWhenDraggingStarted =
              this.activeThumb === "min" ? this.minValue : this.maxValue;
          } else {
            // Otherwise select by closest distance
            const value = this.getValueFromCoordinates(x, y);
            const minDistance = Math.abs(value - this.minValue);
            const maxDistance = Math.abs(value - this.maxValue);

            if (minDistance === maxDistance) {
              // If distances are equal, prioritize the max thumb when value is higher than both thumbs
              // and min thumb when value is lower than both thumbs
              if (value > this.maxValue) {
                this.activeThumb = "max";
              } else if (value < this.minValue) {
                this.activeThumb = "min";
              } else {
                // If the value is between the thumbs and they're at the same distance,
                // prioritize the thumb that's in the direction of movement
                const isRtl = this.localize.dir() === "rtl";
                const isVertical = this.orientation === "vertical";
                const position = isVertical ? y : x;
                const previousPosition = this.lastTrackPosition || position;
                this.lastTrackPosition = position;

                // Determine direction of movement
                const movingForward =
                  (position > previousPosition !== isRtl && !isVertical) ||
                  (position < previousPosition && isVertical);

                this.activeThumb = movingForward ? "max" : "min";
              }
            } else {
              // Select the closest thumb
              this.activeThumb = minDistance <= maxDistance ? "min" : "max";
            }

            this.valueWhenDraggingStarted =
              this.activeThumb === "min" ? this.minValue : this.maxValue;
          }

          this.customStates.set("dragging", true);
          this.setThumbValueFromCoordinates(x, y, this.activeThumb);
          this.showRangeTooltips();
        },
        move: (x, y) => {
          if (this.activeThumb) {
            this.setThumbValueFromCoordinates(x, y, this.activeThumb);
          }
        },
        stop: () => {
          if (this.activeThumb) {
            const currentValue =
              this.activeThumb === "min" ? this.minValue : this.maxValue;
            if (currentValue !== this.valueWhenDraggingStarted) {
              this.updateComplete.then(() => {
                this.dispatchEvent(
                  new Event("change", { bubbles: true, composed: true }),
                );
              });
              this.hasInteracted = true;
            }
          }
          this.hideRangeTooltips();
          this.customStates.set("dragging", false);
          this.valueWhenDraggingStarted = undefined;
          this.activeThumb = null;
        },
      });
    } else {
      // Single thumb mode - original behavior
      this.draggableTrack = new DraggableElement(this.slider, {
        start: (x, y) => {
          this.trackBoundingClientRect = this.track.getBoundingClientRect();
          this.valueWhenDraggingStarted = this.value;
          this.customStates.set("dragging", true);
          this.setValueFromCoordinates(x, y);
          this.showTooltip();
        },
        move: (x, y) => {
          this.setValueFromCoordinates(x, y);
        },
        stop: () => {
          if (this.value !== this.valueWhenDraggingStarted) {
            this.updateComplete.then(() => {
              this.dispatchEvent(
                new Event("change", { bubbles: true, composed: true }),
              );
            });

            this.hasInteracted = true;
          }
          this.hideTooltip();
          this.customStates.set("dragging", false);
          this.valueWhenDraggingStarted = undefined;
        },
      });
    }
  }

  updated(changedProperties: PropertyValues<this>) {
    // Handle range mode changes
    if (changedProperties.has("range")) {
      this.requestUpdate();
    }

    if (this.isRange) {
      // Handle min/max values for range mode
      if (
        changedProperties.has("minValue") ||
        changedProperties.has("maxValue")
      ) {
        // Ensure min doesn't exceed max
        this.minValue = clamp(this.minValue, this.min, this.maxValue);
        this.maxValue = clamp(this.maxValue, this.minValue, this.max);
        // Update form value
        this.updateFormValue();
      }
    } else {
      // Handle value for single thumb mode
      if (changedProperties.has("value")) {
        this.setValue(String(this.value));
      }
    }

    // Handle min/max
    if (changedProperties.has("min") || changedProperties.has("max")) {
      if (this.isRange) {
        this.minValue = clamp(this.minValue, this.min, this.max);
        this.maxValue = clamp(this.maxValue, this.min, this.max);
      }
    }

    // Handle disabled
    if (changedProperties.has("disabled")) {
      this.customStates.set("disabled", this.disabled);
    }

    // Disable dragging when disabled or readonly
    if (
      changedProperties.has("disabled") ||
      changedProperties.has("readonly")
    ) {
      const enabled = !(this.disabled || this.readonly);

      if (this.isRange) {
        if (this.draggableThumbMin) this.draggableThumbMin.toggle(enabled);
        if (this.draggableThumbMax) this.draggableThumbMax.toggle(enabled);
      }

      if (this.draggableTrack) {
        this.draggableTrack.toggle(enabled);
      }
    }

    super.updated(changedProperties);
  }

  /** @internal Called when a containing fieldset is disabled. */
  formDisabledCallback(isDisabled: boolean) {
    this.disabled = isDisabled;
  }

  /** @internal Called when the form is reset. */
  formResetCallback() {
    if (this.isRange) {
      this.minValue = parseFloat(
        this.getAttribute("min-value") ?? String(this.min),
      );
      this.maxValue = parseFloat(
        this.getAttribute("max-value") ?? String(this.max),
      );
    } else {
      this._value = null;
      this.defaultValue =
        this.defaultValue ??
        parseFloat(this.getAttribute("value") ?? String(this.min));
    }
    this.valueHasChanged = false;
    this.hasInteracted = false;
    super.formResetCallback();
  }

  /** Clamps a number to min/max while ensuring it's a valid step interval. */
  private clampAndRoundToStep(value: number) {
    const stepPrecision = (String(this.step).split(".")[1] || "").replace(
      /0+$/g,
      "",
    ).length;

    // Ensure we're working with numbers (in case the user passes strings to the respective properties)
    const step = Number(this.step);
    const min = Number(this.min);
    const max = Number(this.max);

    value = Math.round(value / step) * step;
    value = clamp(value, min, max);

    return parseFloat(value.toFixed(stepPrecision));
  }

  /** Given a value, returns its percentage within a range of min/max. */
  private getPercentageFromValue(value: number) {
    return ((value - this.min) / (this.max - this.min)) * 100;
  }

  /** Converts coordinates to slider value */
  private getValueFromCoordinates(x: number, y: number) {
    const isRtl = this.localize.dir() === "rtl";
    const isVertical = this.orientation === "vertical";
    const { top, right, bottom, left, height, width } =
      this.trackBoundingClientRect;
    const pointerPosition = isVertical ? y : x;
    const sliderCoords = isVertical
      ? { start: top, end: bottom, size: height }
      : { start: left, end: right, size: width };
    const relativePosition = isVertical
      ? sliderCoords.end - pointerPosition
      : isRtl
        ? sliderCoords.end - pointerPosition
        : pointerPosition - sliderCoords.start;
    const percentage = relativePosition / sliderCoords.size;
    return this.clampAndRoundToStep(
      this.min + (this.max - this.min) * percentage,
    );
  }

  private handleBlur() {
    // Only hide tooltips if neither thumb has focus
    if (this.isRange) {
      // Allow a subsequent focus event to fire on the other thumb if the user is tabbing
      requestAnimationFrame(() => {
        const focusedElement = this.shadowRoot?.activeElement;
        const thumbHasFocus =
          focusedElement === this.thumbMin || focusedElement === this.thumbMax;
        if (!thumbHasFocus) {
          this.hideRangeTooltips();
        }
      });
    } else {
      this.hideTooltip();
    }
    this.customStates.set("focused", false);
    this.dispatchEvent(
      new FocusEvent("blur", { bubbles: true, composed: true }),
    );
  }

  private handleFocus(event: FocusEvent) {
    const target = event.target as HTMLElement;

    // Handle focus for specific thumbs in range mode
    if (this.isRange) {
      if (target === this.thumbMin) {
        this.activeThumb = "min";
      } else if (target === this.thumbMax) {
        this.activeThumb = "max";
      }
      this.showRangeTooltips();
    } else {
      this.showTooltip();
    }

    this.customStates.set("focused", true);
    this.dispatchEvent(
      new FocusEvent("focus", { bubbles: true, composed: true }),
    );
  }

  private handleKeyDown(event: KeyboardEvent) {
    const isRtl = this.localize.dir() === "rtl";
    const target = event.target as HTMLElement;

    if (this.disabled || this.readonly) return;

    // For range slider, determine which thumb is active
    if (this.isRange) {
      if (target === this.thumbMin) {
        this.activeThumb = "min";
      } else if (target === this.thumbMax) {
        this.activeThumb = "max";
      }

      if (!this.activeThumb) return;
    }

    // Get current value based on slider mode
    const current = this.isRange
      ? this.activeThumb === "min"
        ? this.minValue
        : this.maxValue
      : this.value;

    let newValue = current;

    // Handle key presses
    switch (event.key) {
      // Increase
      case "ArrowUp":
      case isRtl ? "ArrowLeft" : "ArrowRight":
        event.preventDefault();
        newValue = this.clampAndRoundToStep(current + this.step);
        break;

      // Decrease
      case "ArrowDown":
      case isRtl ? "ArrowRight" : "ArrowLeft":
        event.preventDefault();
        newValue = this.clampAndRoundToStep(current - this.step);
        break;

      // Minimum value
      case "Home":
        event.preventDefault();
        newValue =
          this.isRange && this.activeThumb === "min"
            ? this.min
            : this.isRange
              ? this.minValue
              : this.min;
        break;

      // Maximum value
      case "End":
        event.preventDefault();
        newValue =
          this.isRange && this.activeThumb === "max"
            ? this.max
            : this.isRange
              ? this.maxValue
              : this.max;
        break;

      // Move up 10%
      case "PageUp":
        event.preventDefault();
        const stepUp = Math.max(
          current + (this.max - this.min) / 10,
          current + this.step, // make sure we at least move up to the next step
        );
        newValue = this.clampAndRoundToStep(stepUp);
        break;

      // Move down 10%
      case "PageDown":
        event.preventDefault();
        const stepDown = Math.min(
          current - (this.max - this.min) / 10,
          current - this.step, // make sure we at least move down to the previous step
        );
        newValue = this.clampAndRoundToStep(stepDown);
        break;

      // Handle form submission on Enter
      case "Enter":
        submitOnEnter(event, this);
        return;
    }

    // If no value change, exit early
    if (newValue === current) return;

    // Apply the new value with appropriate constraints
    if (this.isRange) {
      if (this.activeThumb === "min") {
        if (newValue > this.maxValue) {
          // If min thumb exceeds max thumb, move both
          this.maxValue = newValue;
          this.minValue = newValue;
        } else {
          this.minValue = Math.max(this.min, newValue);
        }
      } else {
        if (newValue < this.minValue) {
          // If max thumb goes below min thumb, move both
          this.minValue = newValue;
          this.maxValue = newValue;
        } else {
          this.maxValue = Math.min(this.max, newValue);
        }
      }
      this.updateFormValue();
    } else {
      this.value = clamp(newValue, this.min, this.max);
    }

    // Dispatch events
    this.updateComplete.then(() => {
      this.dispatchEvent(
        new InputEvent("input", { bubbles: true, composed: true }),
      );
      this.dispatchEvent(
        new Event("change", { bubbles: true, composed: true }),
      );
    });
    this.hasInteracted = true;
  }

  private handleLabelPointerDown(event: PointerEvent) {
    event.preventDefault();

    if (!this.disabled) {
      if (this.isRange) {
        this.thumbMin?.focus();
      } else {
        this.slider.focus();
      }
    }
  }

  private setValueFromCoordinates(x: number, y: number) {
    const oldValue = this.value;
    this.value = this.getValueFromCoordinates(x, y);

    // Dispatch input events when the value changes by dragging
    if (this.value !== oldValue) {
      this.updateComplete.then(() => {
        this.dispatchEvent(
          new InputEvent("input", { bubbles: true, composed: true }),
        );
      });
    }
  }

  private setThumbValueFromCoordinates(
    x: number,
    y: number,
    thumb: "min" | "max",
  ) {
    const value = this.getValueFromCoordinates(x, y);
    const oldValue = thumb === "min" ? this.minValue : this.maxValue;

    if (thumb === "min") {
      // If min thumb is being dragged and would exceed max thumb
      if (value > this.maxValue) {
        // Move both thumbs, keeping their distance at 0
        this.maxValue = value;
        this.minValue = value;
      } else {
        // Normal case - just move min thumb
        this.minValue = Math.max(this.min, value);
      }
    } else {
      // thumb === 'max'
      // If max thumb is being dragged and would go below min thumb
      if (value < this.minValue) {
        // Move both thumbs, keeping their distance at 0
        this.minValue = value;
        this.maxValue = value;
      } else {
        // Normal case - just move max thumb
        this.maxValue = Math.min(this.max, value);
      }
    }

    // Dispatch input events
    if (oldValue !== (thumb === "min" ? this.minValue : this.maxValue)) {
      this.updateFormValue();
      this.updateComplete.then(() => {
        this.dispatchEvent(
          new InputEvent("input", { bubbles: true, composed: true }),
        );
      });
    }
  }

  private showTooltip() {
    if (this.withTooltip && this.tooltip) {
      this.tooltip.open = true;
    }
  }

  private hideTooltip() {
    if (this.withTooltip && this.tooltip) {
      this.tooltip.open = false;
    }
  }

  private showRangeTooltips() {
    if (!this.withTooltip) return;

    // Show only the active tooltip, hide the other
    const tooltipMin = this.shadowRoot?.getElementById(
      "tooltip-thumb-min",
    ) as WaTooltip;
    const tooltipMax = this.shadowRoot?.getElementById(
      "tooltip-thumb-max",
    ) as WaTooltip;

    if (this.activeThumb === "min") {
      if (tooltipMin) tooltipMin.open = true;
      if (tooltipMax) tooltipMax.open = false;
    } else if (this.activeThumb === "max") {
      if (tooltipMax) tooltipMax.open = true;
      if (tooltipMin) tooltipMin.open = false;
    }
  }

  private hideRangeTooltips() {
    if (!this.withTooltip) return;

    const tooltipMin = this.shadowRoot?.getElementById(
      "tooltip-thumb-min",
    ) as WaTooltip;
    const tooltipMax = this.shadowRoot?.getElementById(
      "tooltip-thumb-max",
    ) as WaTooltip;

    if (tooltipMin) tooltipMin.open = false;
    if (tooltipMax) tooltipMax.open = false;
  }

  /** Updates the form value submission for range sliders */
  private updateFormValue() {
    if (this.isRange) {
      // Submit both values using FormData for range sliders
      const formData = new FormData();
      formData.append(this.name || "", String(this.minValue));
      formData.append(this.name || "", String(this.maxValue));
      this.setValue(formData);
    }
  }

  /** Sets focus to the slider. */
  public focus() {
    if (this.isRange) {
      this.thumbMin?.focus();
    } else {
      this.slider.focus();
    }
  }

  /** Removes focus from the slider. */
  public blur() {
    if (this.isRange) {
      if (document.activeElement === this.thumbMin) {
        this.thumbMin.blur();
      } else if (document.activeElement === this.thumbMax) {
        this.thumbMax.blur();
      }
    } else {
      this.slider.blur();
    }
  }

  /**
   * Decreases the slider's value by `step`. This is a programmatic change, so `input` and `change` events will not be
   * emitted when this is called.
   */
  public stepDown() {
    if (this.isRange) {
      // If in range mode, default to stepping down the min value
      const newValue = this.clampAndRoundToStep(this.minValue - this.step);
      this.minValue = clamp(newValue, this.min, this.maxValue);
      this.updateFormValue();
    } else {
      const newValue = this.clampAndRoundToStep(this.value - this.step);
      this.value = newValue;
    }
  }

  /**
   * Increases the slider's value by `step`. This is a programmatic change, so `input` and `change` events will not be
   * emitted when this is called.
   */
  public stepUp() {
    if (this.isRange) {
      // If in range mode, default to stepping up the max value
      const newValue = this.clampAndRoundToStep(this.maxValue + this.step);
      this.maxValue = clamp(newValue, this.minValue, this.max);
      this.updateFormValue();
    } else {
      const newValue = this.clampAndRoundToStep(this.value + this.step);
      this.value = newValue;
    }
  }

  render() {
    const hasLabelSlot = this.hasSlotController.test("label");
    const hasHintSlot = this.hasSlotController.test("hint");
    const hasLabel = this.label ? true : !!hasLabelSlot;
    const hasHint = this.hint ? true : !!hasHintSlot;
    const hasReference = this.hasSlotController.test("reference");

    const sliderClasses = classMap({
      small: this.size === "small",
      medium: this.size === "medium",
      large: this.size === "large",
      horizontal: this.orientation === "horizontal",
      vertical: this.orientation === "vertical",
      disabled: this.disabled,
    });

    // Calculate marker positions
    const markers = [];
    if (this.withMarkers) {
      for (let i = this.min; i <= this.max; i += this.step) {
        markers.push(this.getPercentageFromValue(i));
      }
    }

    // Common UI fragments
    const label = html`
      <label
        id="label"
        part="label"
        for=${this.isRange ? "thumb-min" : "text-box"}
        class=${classMap({ vh: !hasLabel, "has-label": hasLabel })}
        @pointerdown=${this.handleLabelPointerDown}
      >
        <slot name="label">${this.label}</slot>
      </label>
    `;

    const hint = html`
      <div
        id="hint"
        part="hint"
        class=${classMap({
          "has-slotted": hasHint,
        })}
      >
        <slot name="hint">${this.hint}</slot>
      </div>
    `;

    const markersTemplate = this.withMarkers
      ? html`
          <div id="markers" part="markers">
            ${markers.map(
              (marker) =>
                html`<span
                  part="marker"
                  class="marker"
                  style=${styleMap({ "--position": `${marker}%` })}
                ></span>`,
            )}
          </div>
        `
      : "";

    const referencesTemplate = hasReference
      ? html`
          <div id="references" part="references" aria-hidden="true">
            <slot name="reference"></slot>
          </div>
        `
      : "";

    // Create tooltip template function
    const createTooltip = (thumbId: string, value: number) =>
      this.withTooltip
        ? html`
            <wa-tooltip
              id=${`tooltip${thumbId !== "thumb" ? "-" + thumbId : ""}`}
              part="tooltip"
              exportparts="
                base:tooltip__base,
                body:tooltip__body,
                arrow:tooltip__arrow
              "
              trigger="manual"
              distance=${this.tooltipDistance}
              placement=${this.tooltipPlacement}
              for=${thumbId}
              activation="manual"
              dir=${this.localize.dir()}
            >
              <span aria-hidden="true">
                ${typeof this.valueFormatter === "function"
                  ? this.valueFormatter(value)
                  : this.localize.number(value)}
              </span>
            </wa-tooltip>
          `
        : "";

    // Render based on mode
    if (this.isRange) {
      // Range slider mode
      const minThumbPosition = clamp(
        this.getPercentageFromValue(this.minValue),
        0,
        100,
      );
      const maxThumbPosition = clamp(
        this.getPercentageFromValue(this.maxValue),
        0,
        100,
      );

      return html`
        ${label}

        <div id="slider" part="slider" class=${sliderClasses}>
          <div id="track" part="track">
            <div
              id="indicator"
              part="indicator"
              style=${styleMap({
                "--start": `${Math.min(minThumbPosition, maxThumbPosition)}%`,
                "--end": `${Math.max(minThumbPosition, maxThumbPosition)}%`,
              })}
            ></div>

            ${markersTemplate}

            <span
              id="thumb-min"
              part="thumb thumb-min"
              style=${styleMap({ "--position": `${minThumbPosition}%` })}
              role="slider"
              aria-valuemin=${this.min}
              aria-valuenow=${this.minValue}
              aria-valuetext=${typeof this.valueFormatter === "function"
                ? this.valueFormatter(this.minValue)
                : this.localize.number(this.minValue)}
              aria-valuemax=${this.max}
              aria-label="${this.label
                ? `${this.label} (minimum value)`
                : "Minimum value"}"
              aria-orientation=${this.orientation}
              aria-disabled=${this.disabled ? "true" : "false"}
              aria-readonly=${this.readonly ? "true" : "false"}
              tabindex=${this.disabled ? -1 : 0}
              @blur=${this.handleBlur}
              @focus=${this.handleFocus}
              @keydown=${this.handleKeyDown}
            ></span>

            <span
              id="thumb-max"
              part="thumb thumb-max"
              style=${styleMap({ "--position": `${maxThumbPosition}%` })}
              role="slider"
              aria-valuemin=${this.min}
              aria-valuenow=${this.maxValue}
              aria-valuetext=${typeof this.valueFormatter === "function"
                ? this.valueFormatter(this.maxValue)
                : this.localize.number(this.maxValue)}
              aria-valuemax=${this.max}
              aria-label="${this.label
                ? `${this.label} (maximum value)`
                : "Maximum value"}"
              aria-orientation=${this.orientation}
              aria-disabled=${this.disabled ? "true" : "false"}
              aria-readonly=${this.readonly ? "true" : "false"}
              tabindex=${this.disabled ? -1 : 0}
              @blur=${this.handleBlur}
              @focus=${this.handleFocus}
              @keydown=${this.handleKeyDown}
            ></span>
          </div>

          ${referencesTemplate} ${hint}
        </div>

        ${createTooltip("thumb-min", this.minValue)}
        ${createTooltip("thumb-max", this.maxValue)}
      `;
    } else {
      // Single thumb mode
      const thumbPosition = clamp(
        this.getPercentageFromValue(this.value),
        0,
        100,
      );
      const indicatorOffsetPosition = clamp(
        this.getPercentageFromValue(
          typeof this.indicatorOffset === "number"
            ? this.indicatorOffset
            : this.min,
        ),
        0,
        100,
      );

      return html`
        ${label}

        <div
          id="slider"
          part="slider"
          class=${sliderClasses}
          role="slider"
          aria-disabled=${this.disabled ? "true" : "false"}
          aria-readonly=${this.disabled ? "true" : "false"}
          aria-orientation=${this.orientation}
          aria-valuemin=${this.min}
          aria-valuenow=${this.value}
          aria-valuetext=${typeof this.valueFormatter === "function"
            ? this.valueFormatter(this.value)
            : this.localize.number(this.value)}
          aria-valuemax=${this.max}
          aria-labelledby="label"
          aria-describedby="hint"
          tabindex=${this.disabled ? -1 : 0}
          @blur=${this.handleBlur}
          @focus=${this.handleFocus}
          @keydown=${this.handleKeyDown}
        >
          <div id="track" part="track">
            <div
              id="indicator"
              part="indicator"
              style=${styleMap({
                "--start": `${indicatorOffsetPosition}%`,
                "--end": `${thumbPosition}%`,
              })}
            ></div>

            ${markersTemplate}
            <span
              id="thumb"
              part="thumb"
              style=${styleMap({ "--position": `${thumbPosition}%` })}
            ></span>
          </div>

          ${referencesTemplate} ${hint}
        </div>

        ${createTooltip("thumb", this.value)}
      `;
    }
  }
}

declare global {
  interface HTMLElementTagNameMap {
    "wa-slider": WaSlider;
  }
}

`````
