# externals/webawesome/popup/popup.ts

## Option 1

`````json
{"printWidth":80}
`````

### Diff

`````diff
===================================================================
--- prettier
+++ oxfmt
@@ -134,9 +134,12 @@
    * anchor as possible, considering available space and `arrow-padding`. A value of `start`, `end`, or `center` will
    * align the arrow to the start, end, or center of the popover instead.
    */
   @property({ attribute: "arrow-placement" }) arrowPlacement:
-    "start" | "end" | "center" | "anchor" = "anchor";
+    | "start"
+    | "end"
+    | "center"
+    | "anchor" = "anchor";
 
   /**
    * The amount of padding between the arrow and the edges of the popup. If the popup has a border-radius, for example,
    * this will prevent it from overflowing the corners.
@@ -175,9 +178,10 @@
    * the popup should be positioned using the best available fit based on available space or as it was initially
    * preferred.
    */
   @property({ attribute: "flip-fallback-strategy" }) flipFallbackStrategy:
-    "best-fit" | "initial" = "best-fit";
+    | "best-fit"
+    | "initial" = "best-fit";
 
   /**
    * The flip boundary describes clipping element(s) that overflow will be checked relative to when flipping. By
    * default, the boundary includes overflow ancestors that will cause the element to be clipped. If needed, you can
@@ -202,9 +206,11 @@
   @property({ attribute: "shift-padding", type: Number }) shiftPadding = 0;
 
   /** When set, this will cause the popup to automatically resize itself to prevent it from overflowing. */
   @property({ attribute: "auto-size" }) autoSize:
-    "horizontal" | "vertical" | "both";
+    | "horizontal"
+    | "vertical"
+    | "both";
 
   /** Syncs the popup's width or height to that of the anchor element. */
   @property() sync: "width" | "height" | "both";
 
@@ -663,9 +669,11 @@
           "popup-has-arrow": this.arrow,
         })}
       >
         <slot></slot>
-        ${this.arrow ? html`<div part="arrow" class="arrow" role="presentation"></div>` : ""}
+        ${this.arrow
+          ? html`<div part="arrow" class="arrow" role="presentation"></div>`
+          : ""}
       </div>
     `;
   }
 }

`````

### Actual (oxfmt)

`````ts
import {
  arrow,
  autoUpdate,
  computePosition,
  flip,
  getOverflowAncestors,
  offset,
  platform,
  shift,
  size,
} from "@floating-ui/dom";
import { offsetParent } from "composed-offset-position";
import type { PropertyValues } from "lit";
import { html } from "lit";
import { customElement, property, query } from "lit/decorators.js";
import { classMap } from "lit/directives/class-map.js";
import { WaRepositionEvent } from "../../events/reposition.js";
import WebAwesomeElement from "../../internal/webawesome-element.js";
import { LocalizeController } from "../../utilities/localize.js";
import styles from "./popup.styles.js";

export interface VirtualElement {
  getBoundingClientRect: () => DOMRect;
  contextElement?: Element;
}

function isVirtualElement(e: unknown): e is VirtualElement {
  return (
    e !== null &&
    typeof e === "object" &&
    "getBoundingClientRect" in e &&
    ("contextElement" in e ? e instanceof Element : true)
  );
}

const SUPPORTS_POPOVER =
  globalThis?.HTMLElement?.prototype.hasOwnProperty("popover");

/**
 * @summary Popups declaratively anchor one element to another and keep them positioned together as the page scrolls or
 *  resizes. Primarily a low-level building block for popovers, dropdowns, and tooltips.
 * @documentation https://webawesome.com/docs/components/popup
 * @status stable
 * @since 2.0
 *
 * @event wa-reposition - Emitted when the popup is repositioned. This event can fire a lot, so avoid putting expensive
 *  operations in your listener or consider debouncing it.
 *
 * @slot - The popup's content.
 * @slot anchor - The element the popup will be anchored to. If the anchor lives outside of the popup, you can use the
 *  `anchor` attribute or property instead.
 *
 * @csspart arrow - The arrow's container. Avoid setting `top|bottom|left|right` properties, as these values are
 *  assigned dynamically as the popup moves. This is most useful for applying a background color to match the popup, and
 *  maybe a border or box shadow.
 * @csspart popup - The popup's container. Useful for setting a background color, box shadow, etc.
 * @csspart hover-bridge - The hover bridge element. Only available when the `hover-bridge` option is enabled.
 *
 * @cssproperty [--arrow-size=6px] - The size of the arrow. Note that an arrow won't be shown unless the `arrow`
 *  attribute is used.
 * @cssproperty [--popup-border-width] - The width of any custom border applied to the popup. This is used to reposition
 *  the arrow to overlap to the inside edge of the popup border.
 * @cssproperty [--arrow-color=black] - The color of the arrow.
 * @cssproperty [--auto-size-available-width] - A read-only custom property that determines the amount of width the
 *  popup can be before overflowing. Useful for positioning child elements that need to overflow. This property is only
 *  available when using `auto-size`.
 * @cssproperty [--auto-size-available-height] - A read-only custom property that determines the amount of height the
 *  popup can be before overflowing. Useful for positioning child elements that need to overflow. This property is only
 *  available when using `auto-size`.
 * @cssproperty [--show-duration=100ms] - The show duration to use when applying built-in animation classes.
 * @cssproperty [--hide-duration=100ms] - The hide duration to use when applying built-in animation classes.
 */
@customElement("wa-popup")
export default class WaPopup extends WebAwesomeElement {
  static css = styles;

  private anchorEl: Element | VirtualElement | null;
  private cleanup: ReturnType<typeof autoUpdate> | undefined;
  private readonly localize = new LocalizeController(this);

  /** A reference to the internal popup container. Useful for animating and styling the popup with JavaScript. */
  @query(".popup") popup: HTMLElement;
  @query(".arrow") private arrowEl: HTMLElement;

  /**
   * The element the popup will be anchored to. If the anchor lives outside of the popup, you can provide the anchor
   * element `id`, a DOM element reference, or a `VirtualElement`. If the anchor lives inside the popup, use the
   * `anchor` slot instead.
   */
  @property() anchor: Element | string | VirtualElement;

  /**
   * Activates the positioning logic and shows the popup. When this attribute is removed, the positioning logic is torn
   * down and the popup will be hidden.
   */
  @property({ type: Boolean, reflect: true }) active = false;

  /**
   * The preferred placement of the popup. Note that the actual placement will vary as configured to keep the
   * panel inside of the viewport.
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
    | "left-end" = "top";

  /** The bounding box to use for flipping, shifting, and auto-sizing. */
  @property() boundary: "viewport" | "scroll" = "viewport";

  /** The distance in pixels from which to offset the panel away from its anchor. */
  @property({ type: Number }) distance = 0;

  /** The distance in pixels from which to offset the panel along its anchor. */
  @property({ type: Number }) skidding = 0;

  /**
   * Attaches an arrow to the popup. The arrow's size and color can be customized using the `--arrow-size` and
   * `--arrow-color` custom properties. For additional customizations, you can also target the arrow using
   * `::part(arrow)` in your stylesheet.
   */
  @property({ type: Boolean }) arrow = false;

  /**
   * The placement of the arrow. The default is `anchor`, which will align the arrow as close to the center of the
   * anchor as possible, considering available space and `arrow-padding`. A value of `start`, `end`, or `center` will
   * align the arrow to the start, end, or center of the popover instead.
   */
  @property({ attribute: "arrow-placement" }) arrowPlacement:
    | "start"
    | "end"
    | "center"
    | "anchor" = "anchor";

  /**
   * The amount of padding between the arrow and the edges of the popup. If the popup has a border-radius, for example,
   * this will prevent it from overflowing the corners.
   */
  @property({ attribute: "arrow-padding", type: Number }) arrowPadding = 10;

  /**
   * When set, placement of the popup will flip to the opposite site to keep it in view. You can use
   * `flipFallbackPlacements` to further configure how the fallback placement is determined.
   */
  @property({ type: Boolean }) flip = false;

  /**
   * If the preferred placement doesn't fit, popup will be tested in these fallback placements until one fits. Must be a
   * string of any number of placements separated by a space, e.g. "top bottom left". If no placement fits, the flip
   * fallback strategy will be used instead.
   * */
  @property({
    attribute: "flip-fallback-placements",
    converter: {
      fromAttribute: (value: string) => {
        return value
          .split(" ")
          .map((p) => p.trim())
          .filter((p) => p !== "");
      },
      toAttribute: (value: []) => {
        return value.join(" ");
      },
    },
  })
  flipFallbackPlacements = "";

  /**
   * When neither the preferred placement nor the fallback placements fit, this value will be used to determine whether
   * the popup should be positioned using the best available fit based on available space or as it was initially
   * preferred.
   */
  @property({ attribute: "flip-fallback-strategy" }) flipFallbackStrategy:
    | "best-fit"
    | "initial" = "best-fit";

  /**
   * The flip boundary describes clipping element(s) that overflow will be checked relative to when flipping. By
   * default, the boundary includes overflow ancestors that will cause the element to be clipped. If needed, you can
   * change the boundary by passing a reference to one or more elements to this property.
   */
  @property({ type: Object }) flipBoundary: Element | Element[];

  /** The amount of padding, in pixels, to exceed before the flip behavior will occur. */
  @property({ attribute: "flip-padding", type: Number }) flipPadding = 0;

  /** Moves the popup along the axis to keep it in view when clipped. */
  @property({ type: Boolean }) shift = false;

  /**
   * The shift boundary describes clipping element(s) that overflow will be checked relative to when shifting. By
   * default, the boundary includes overflow ancestors that will cause the element to be clipped. If needed, you can
   * change the boundary by passing a reference to one or more elements to this property.
   */
  @property({ type: Object }) shiftBoundary: Element | Element[];

  /** The amount of padding, in pixels, to exceed before the shift behavior will occur. */
  @property({ attribute: "shift-padding", type: Number }) shiftPadding = 0;

  /** When set, this will cause the popup to automatically resize itself to prevent it from overflowing. */
  @property({ attribute: "auto-size" }) autoSize:
    | "horizontal"
    | "vertical"
    | "both";

  /** Syncs the popup's width or height to that of the anchor element. */
  @property() sync: "width" | "height" | "both";

  /**
   * The auto-size boundary describes clipping element(s) that overflow will be checked relative to when resizing. By
   * default, the boundary includes overflow ancestors that will cause the element to be clipped. If needed, you can
   * change the boundary by passing a reference to one or more elements to this property.
   */
  @property({ type: Object }) autoSizeBoundary: Element | Element[];

  /** The amount of padding, in pixels, to exceed before the auto-size behavior will occur. */
  @property({ attribute: "auto-size-padding", type: Number }) autoSizePadding =
    0;

  /**
   * When a gap exists between the anchor and the popup element, this option will add a "hover bridge" that fills the
   * gap using an invisible element. This makes listening for events such as `mouseenter` and `mouseleave` more sane
   * because the pointer never technically leaves the element. The hover bridge will only be drawn when the popover is
   * active.
   */
  @property({ attribute: "hover-bridge", type: Boolean }) hoverBridge = false;

  async connectedCallback() {
    super.connectedCallback();

    // Start the positioner after the first update
    await this.updateComplete;
    this.start();
  }

  disconnectedCallback() {
    super.disconnectedCallback();
    this.stop();
  }

  async updated(changedProperties: PropertyValues<this>) {
    super.updated(changedProperties);

    // Start or stop the positioner when active changes
    if (changedProperties.has("active")) {
      if (this.active) {
        this.start();
      } else {
        this.stop();
      }
    }

    // Update the anchor when anchor changes
    if (changedProperties.has("anchor")) {
      this.handleAnchorChange();
    }

    // All other properties will trigger a reposition when active
    if (this.active) {
      await this.updateComplete;
      this.reposition();
    }
  }

  private async handleAnchorChange() {
    await this.stop();

    if (this.anchor && typeof this.anchor === "string") {
      // Locate the anchor by id
      const root = this.getRootNode() as Document | ShadowRoot;
      this.anchorEl = root.getElementById(this.anchor);
    } else if (
      this.anchor instanceof Element ||
      isVirtualElement(this.anchor)
    ) {
      // Use the anchor's reference
      this.anchorEl = this.anchor;
    } else {
      // Look for a slotted anchor
      this.anchorEl = this.querySelector<HTMLElement>('[slot="anchor"]');
    }

    // If the anchor is a `<slot>`, we'll use the first assigned element as the target since slots use
    // `display: contents`
    // and positioning can't be calculated on them
    if (this.anchorEl instanceof HTMLSlotElement) {
      this.anchorEl = this.anchorEl.assignedElements({
        flatten: true,
      })[0] as HTMLElement;
    }

    // If the anchor is valid, start it up
    if (this.anchorEl) {
      this.start();
    }
  }

  private start() {
    // We can't start the positioner without an anchor
    if (!this.anchorEl || !this.active || !this.isConnected) {
      return;
    }

    this.popup?.showPopover?.();

    this.cleanup = autoUpdate(this.anchorEl, this.popup, () => {
      this.reposition();
    });
  }

  private async stop(): Promise<void> {
    return new Promise((resolve) => {
      this.popup?.hidePopover?.();

      if (this.cleanup) {
        this.cleanup();
        this.cleanup = undefined;
        this.removeAttribute("data-current-placement");
        this.style.removeProperty("--auto-size-available-width");
        this.style.removeProperty("--auto-size-available-height");
        requestAnimationFrame(() => resolve());
      } else {
        resolve();
      }
    });
  }

  /** Forces the popup to recalculate and reposition itself. */
  reposition() {
    // Nothing to do if the popup is inactive or the anchor doesn't exist
    if (!this.active || !this.anchorEl || !this.popup) {
      return;
    }

    //
    // NOTE: Floating UI middlewares are order dependent: https://floating-ui.com/docs/middleware
    //
    const middleware = [
      // The offset middleware goes first
      offset({ mainAxis: this.distance, crossAxis: this.skidding }),
    ];

    // First we sync width/height
    if (this.sync) {
      middleware.push(
        size({
          apply: ({ rects }) => {
            const syncWidth = this.sync === "width" || this.sync === "both";
            const syncHeight = this.sync === "height" || this.sync === "both";
            this.popup.style.width = syncWidth
              ? `${rects.reference.width}px`
              : "";
            this.popup.style.height = syncHeight
              ? `${rects.reference.height}px`
              : "";
          },
        }),
      );
    } else {
      // Cleanup styles if we're not matching width/height
      this.popup.style.width = "";
      this.popup.style.height = "";
    }

    let defaultBoundary;

    if (
      SUPPORTS_POPOVER &&
      !isVirtualElement(this.anchor) &&
      this.boundary === "scroll"
    ) {
      // When using the Popover API, the floating element is no longer in the same DOM context
      // as the overflow ancestors so Floating-UI can't find them.
      // For flip, `elementContext: 'reference'` gets it to use the anchor element instead,
      // but the option is not available for shift() or size(), so we basically need to implement it ourselves.
      defaultBoundary = getOverflowAncestors(this.anchorEl as Element).filter(
        (el) => el instanceof Element,
      );
    }

    // Then we flip
    if (this.flip) {
      middleware.push(
        flip({
          boundary: this.flipBoundary || defaultBoundary,
          // @ts-expect-error - We're converting a string attribute to an array here
          fallbackPlacements: this.flipFallbackPlacements,
          fallbackStrategy:
            this.flipFallbackStrategy === "best-fit"
              ? "bestFit"
              : "initialPlacement",
          padding: this.flipPadding,
        }),
      );
    }

    // Then we shift
    if (this.shift) {
      middleware.push(
        shift({
          boundary: this.shiftBoundary || defaultBoundary,
          padding: this.shiftPadding,
        }),
      );
    }

    // Now we adjust the size as needed
    if (this.autoSize) {
      middleware.push(
        size({
          boundary: this.autoSizeBoundary || defaultBoundary,
          padding: this.autoSizePadding,
          apply: ({ availableWidth, availableHeight }) => {
            if (this.autoSize === "vertical" || this.autoSize === "both") {
              this.style.setProperty(
                "--auto-size-available-height",
                `${availableHeight}px`,
              );
            } else {
              this.style.removeProperty("--auto-size-available-height");
            }

            if (this.autoSize === "horizontal" || this.autoSize === "both") {
              this.style.setProperty(
                "--auto-size-available-width",
                `${availableWidth}px`,
              );
            } else {
              this.style.removeProperty("--auto-size-available-width");
            }
          },
        }),
      );
    } else {
      // Cleanup styles if we're no longer using auto-size
      this.style.removeProperty("--auto-size-available-width");
      this.style.removeProperty("--auto-size-available-height");
    }

    // Finally, we add an arrow
    if (this.arrow) {
      middleware.push(
        arrow({
          element: this.arrowEl,
          padding: this.arrowPadding,
        }),
      );
    }

    //
    // Use custom positioning logic if the strategy is absolute. Otherwise, fall back to the default logic.
    //
    // More info: https://github.com/shoelace-style/shoelace/issues/1135
    //
    const getOffsetParent = SUPPORTS_POPOVER
      ? (element: Element) => platform.getOffsetParent(element, offsetParent)
      : platform.getOffsetParent;

    computePosition(this.anchorEl, this.popup, {
      placement: this.placement,
      middleware,
      strategy: SUPPORTS_POPOVER ? "absolute" : "fixed",
      platform: {
        ...platform,
        getOffsetParent,
      },
    }).then(({ x, y, middlewareData, placement }) => {
      //
      // Even though we have our own localization utility, it uses different heuristics to determine RTL. Because of
      // that, we'll use the same approach that Floating UI uses.
      //
      // Source: https://github.com/floating-ui/floating-ui/blob/cb3b6ab07f95275730d3e6e46c702f8d4908b55c/packages/dom/src/utils/getDocumentRect.ts#L31
      //
      const isRtl = this.localize.dir() === "rtl";
      const staticSide = {
        top: "bottom",
        right: "left",
        bottom: "top",
        left: "right",
      }[placement.split("-")[0]]!;

      this.setAttribute("data-current-placement", placement);

      Object.assign(this.popup.style, {
        left: `${x}px`,
        top: `${y}px`,
      });

      if (this.arrow) {
        const arrowX = middlewareData.arrow!.x;
        const arrowY = middlewareData.arrow!.y;
        let top = "";
        let right = "";
        let bottom = "";
        let left = "";

        if (this.arrowPlacement === "start") {
          // Start
          const value =
            typeof arrowX === "number"
              ? `calc(${this.arrowPadding}px - var(--arrow-padding-offset))`
              : "";
          top =
            typeof arrowY === "number"
              ? `calc(${this.arrowPadding}px - var(--arrow-padding-offset))`
              : "";
          right = isRtl ? value : "";
          left = isRtl ? "" : value;
        } else if (this.arrowPlacement === "end") {
          // End
          const value =
            typeof arrowX === "number"
              ? `calc(${this.arrowPadding}px - var(--arrow-padding-offset))`
              : "";
          right = isRtl ? "" : value;
          left = isRtl ? value : "";
          bottom =
            typeof arrowY === "number"
              ? `calc(${this.arrowPadding}px - var(--arrow-padding-offset))`
              : "";
        } else if (this.arrowPlacement === "center") {
          // Center
          left =
            typeof arrowX === "number"
              ? `calc(50% - var(--arrow-size-diagonal))`
              : "";
          top =
            typeof arrowY === "number"
              ? `calc(50% - var(--arrow-size-diagonal))`
              : "";
        } else {
          // Anchor (default)
          left = typeof arrowX === "number" ? `${arrowX}px` : "";
          top = typeof arrowY === "number" ? `${arrowY}px` : "";
        }

        Object.assign(this.arrowEl.style, {
          top,
          right,
          bottom,
          left,
          [staticSide]:
            "calc(var(--arrow-base-offset) - var(--arrow-size-diagonal))",
        });
      }
    });

    // Wait until the new position is drawn before updating the hover bridge, otherwise it can get out of sync
    requestAnimationFrame(() => this.updateHoverBridge());

    this.dispatchEvent(new WaRepositionEvent());
  }

  private updateHoverBridge = () => {
    if (this.hoverBridge && this.anchorEl && this.popup) {
      const anchorRect = this.anchorEl.getBoundingClientRect();
      const popupRect = this.popup.getBoundingClientRect();
      const isVertical =
        this.placement.includes("top") || this.placement.includes("bottom");
      let topLeftX = 0;
      let topLeftY = 0;
      let topRightX = 0;
      let topRightY = 0;
      let bottomLeftX = 0;
      let bottomLeftY = 0;
      let bottomRightX = 0;
      let bottomRightY = 0;

      if (isVertical) {
        if (anchorRect.top < popupRect.top) {
          // Anchor is above
          topLeftX = anchorRect.left;
          topLeftY = anchorRect.bottom;
          topRightX = anchorRect.right;
          topRightY = anchorRect.bottom;

          bottomLeftX = popupRect.left;
          bottomLeftY = popupRect.top;
          bottomRightX = popupRect.right;
          bottomRightY = popupRect.top;
        } else {
          // Anchor is below
          topLeftX = popupRect.left;
          topLeftY = popupRect.bottom;
          topRightX = popupRect.right;
          topRightY = popupRect.bottom;

          bottomLeftX = anchorRect.left;
          bottomLeftY = anchorRect.top;
          bottomRightX = anchorRect.right;
          bottomRightY = anchorRect.top;
        }
      } else {
        if (anchorRect.left < popupRect.left) {
          // Anchor is on the left
          topLeftX = anchorRect.right;
          topLeftY = anchorRect.top;
          topRightX = popupRect.left;
          topRightY = popupRect.top;

          bottomLeftX = anchorRect.right;
          bottomLeftY = anchorRect.bottom;
          bottomRightX = popupRect.left;
          bottomRightY = popupRect.bottom;
        } else {
          // Anchor is on the right
          topLeftX = popupRect.right;
          topLeftY = popupRect.top;
          topRightX = anchorRect.left;
          topRightY = anchorRect.top;

          bottomLeftX = popupRect.right;
          bottomLeftY = popupRect.bottom;
          bottomRightX = anchorRect.left;
          bottomRightY = anchorRect.bottom;
        }
      }

      this.style.setProperty("--hover-bridge-top-left-x", `${topLeftX}px`);
      this.style.setProperty("--hover-bridge-top-left-y", `${topLeftY}px`);
      this.style.setProperty("--hover-bridge-top-right-x", `${topRightX}px`);
      this.style.setProperty("--hover-bridge-top-right-y", `${topRightY}px`);
      this.style.setProperty(
        "--hover-bridge-bottom-left-x",
        `${bottomLeftX}px`,
      );
      this.style.setProperty(
        "--hover-bridge-bottom-left-y",
        `${bottomLeftY}px`,
      );
      this.style.setProperty(
        "--hover-bridge-bottom-right-x",
        `${bottomRightX}px`,
      );
      this.style.setProperty(
        "--hover-bridge-bottom-right-y",
        `${bottomRightY}px`,
      );
    }
  };

  render() {
    return html`
      <slot name="anchor" @slotchange=${this.handleAnchorChange}></slot>

      <span
        part="hover-bridge"
        class=${classMap({
          "popup-hover-bridge": true,
          "popup-hover-bridge-visible": this.hoverBridge && this.active,
        })}
      ></span>

      <div
        popover="manual"
        part="popup"
        class=${classMap({
          popup: true,
          "popup-active": this.active,
          "popup-fixed": !SUPPORTS_POPOVER,
          "popup-has-arrow": this.arrow,
        })}
      >
        <slot></slot>
        ${this.arrow
          ? html`<div part="arrow" class="arrow" role="presentation"></div>`
          : ""}
      </div>
    `;
  }
}

declare global {
  interface HTMLElementTagNameMap {
    "wa-popup": WaPopup;
  }
}

`````

### Expected (prettier)

`````ts
import {
  arrow,
  autoUpdate,
  computePosition,
  flip,
  getOverflowAncestors,
  offset,
  platform,
  shift,
  size,
} from "@floating-ui/dom";
import { offsetParent } from "composed-offset-position";
import type { PropertyValues } from "lit";
import { html } from "lit";
import { customElement, property, query } from "lit/decorators.js";
import { classMap } from "lit/directives/class-map.js";
import { WaRepositionEvent } from "../../events/reposition.js";
import WebAwesomeElement from "../../internal/webawesome-element.js";
import { LocalizeController } from "../../utilities/localize.js";
import styles from "./popup.styles.js";

export interface VirtualElement {
  getBoundingClientRect: () => DOMRect;
  contextElement?: Element;
}

function isVirtualElement(e: unknown): e is VirtualElement {
  return (
    e !== null &&
    typeof e === "object" &&
    "getBoundingClientRect" in e &&
    ("contextElement" in e ? e instanceof Element : true)
  );
}

const SUPPORTS_POPOVER =
  globalThis?.HTMLElement?.prototype.hasOwnProperty("popover");

/**
 * @summary Popups declaratively anchor one element to another and keep them positioned together as the page scrolls or
 *  resizes. Primarily a low-level building block for popovers, dropdowns, and tooltips.
 * @documentation https://webawesome.com/docs/components/popup
 * @status stable
 * @since 2.0
 *
 * @event wa-reposition - Emitted when the popup is repositioned. This event can fire a lot, so avoid putting expensive
 *  operations in your listener or consider debouncing it.
 *
 * @slot - The popup's content.
 * @slot anchor - The element the popup will be anchored to. If the anchor lives outside of the popup, you can use the
 *  `anchor` attribute or property instead.
 *
 * @csspart arrow - The arrow's container. Avoid setting `top|bottom|left|right` properties, as these values are
 *  assigned dynamically as the popup moves. This is most useful for applying a background color to match the popup, and
 *  maybe a border or box shadow.
 * @csspart popup - The popup's container. Useful for setting a background color, box shadow, etc.
 * @csspart hover-bridge - The hover bridge element. Only available when the `hover-bridge` option is enabled.
 *
 * @cssproperty [--arrow-size=6px] - The size of the arrow. Note that an arrow won't be shown unless the `arrow`
 *  attribute is used.
 * @cssproperty [--popup-border-width] - The width of any custom border applied to the popup. This is used to reposition
 *  the arrow to overlap to the inside edge of the popup border.
 * @cssproperty [--arrow-color=black] - The color of the arrow.
 * @cssproperty [--auto-size-available-width] - A read-only custom property that determines the amount of width the
 *  popup can be before overflowing. Useful for positioning child elements that need to overflow. This property is only
 *  available when using `auto-size`.
 * @cssproperty [--auto-size-available-height] - A read-only custom property that determines the amount of height the
 *  popup can be before overflowing. Useful for positioning child elements that need to overflow. This property is only
 *  available when using `auto-size`.
 * @cssproperty [--show-duration=100ms] - The show duration to use when applying built-in animation classes.
 * @cssproperty [--hide-duration=100ms] - The hide duration to use when applying built-in animation classes.
 */
@customElement("wa-popup")
export default class WaPopup extends WebAwesomeElement {
  static css = styles;

  private anchorEl: Element | VirtualElement | null;
  private cleanup: ReturnType<typeof autoUpdate> | undefined;
  private readonly localize = new LocalizeController(this);

  /** A reference to the internal popup container. Useful for animating and styling the popup with JavaScript. */
  @query(".popup") popup: HTMLElement;
  @query(".arrow") private arrowEl: HTMLElement;

  /**
   * The element the popup will be anchored to. If the anchor lives outside of the popup, you can provide the anchor
   * element `id`, a DOM element reference, or a `VirtualElement`. If the anchor lives inside the popup, use the
   * `anchor` slot instead.
   */
  @property() anchor: Element | string | VirtualElement;

  /**
   * Activates the positioning logic and shows the popup. When this attribute is removed, the positioning logic is torn
   * down and the popup will be hidden.
   */
  @property({ type: Boolean, reflect: true }) active = false;

  /**
   * The preferred placement of the popup. Note that the actual placement will vary as configured to keep the
   * panel inside of the viewport.
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
    | "left-end" = "top";

  /** The bounding box to use for flipping, shifting, and auto-sizing. */
  @property() boundary: "viewport" | "scroll" = "viewport";

  /** The distance in pixels from which to offset the panel away from its anchor. */
  @property({ type: Number }) distance = 0;

  /** The distance in pixels from which to offset the panel along its anchor. */
  @property({ type: Number }) skidding = 0;

  /**
   * Attaches an arrow to the popup. The arrow's size and color can be customized using the `--arrow-size` and
   * `--arrow-color` custom properties. For additional customizations, you can also target the arrow using
   * `::part(arrow)` in your stylesheet.
   */
  @property({ type: Boolean }) arrow = false;

  /**
   * The placement of the arrow. The default is `anchor`, which will align the arrow as close to the center of the
   * anchor as possible, considering available space and `arrow-padding`. A value of `start`, `end`, or `center` will
   * align the arrow to the start, end, or center of the popover instead.
   */
  @property({ attribute: "arrow-placement" }) arrowPlacement:
    "start" | "end" | "center" | "anchor" = "anchor";

  /**
   * The amount of padding between the arrow and the edges of the popup. If the popup has a border-radius, for example,
   * this will prevent it from overflowing the corners.
   */
  @property({ attribute: "arrow-padding", type: Number }) arrowPadding = 10;

  /**
   * When set, placement of the popup will flip to the opposite site to keep it in view. You can use
   * `flipFallbackPlacements` to further configure how the fallback placement is determined.
   */
  @property({ type: Boolean }) flip = false;

  /**
   * If the preferred placement doesn't fit, popup will be tested in these fallback placements until one fits. Must be a
   * string of any number of placements separated by a space, e.g. "top bottom left". If no placement fits, the flip
   * fallback strategy will be used instead.
   * */
  @property({
    attribute: "flip-fallback-placements",
    converter: {
      fromAttribute: (value: string) => {
        return value
          .split(" ")
          .map((p) => p.trim())
          .filter((p) => p !== "");
      },
      toAttribute: (value: []) => {
        return value.join(" ");
      },
    },
  })
  flipFallbackPlacements = "";

  /**
   * When neither the preferred placement nor the fallback placements fit, this value will be used to determine whether
   * the popup should be positioned using the best available fit based on available space or as it was initially
   * preferred.
   */
  @property({ attribute: "flip-fallback-strategy" }) flipFallbackStrategy:
    "best-fit" | "initial" = "best-fit";

  /**
   * The flip boundary describes clipping element(s) that overflow will be checked relative to when flipping. By
   * default, the boundary includes overflow ancestors that will cause the element to be clipped. If needed, you can
   * change the boundary by passing a reference to one or more elements to this property.
   */
  @property({ type: Object }) flipBoundary: Element | Element[];

  /** The amount of padding, in pixels, to exceed before the flip behavior will occur. */
  @property({ attribute: "flip-padding", type: Number }) flipPadding = 0;

  /** Moves the popup along the axis to keep it in view when clipped. */
  @property({ type: Boolean }) shift = false;

  /**
   * The shift boundary describes clipping element(s) that overflow will be checked relative to when shifting. By
   * default, the boundary includes overflow ancestors that will cause the element to be clipped. If needed, you can
   * change the boundary by passing a reference to one or more elements to this property.
   */
  @property({ type: Object }) shiftBoundary: Element | Element[];

  /** The amount of padding, in pixels, to exceed before the shift behavior will occur. */
  @property({ attribute: "shift-padding", type: Number }) shiftPadding = 0;

  /** When set, this will cause the popup to automatically resize itself to prevent it from overflowing. */
  @property({ attribute: "auto-size" }) autoSize:
    "horizontal" | "vertical" | "both";

  /** Syncs the popup's width or height to that of the anchor element. */
  @property() sync: "width" | "height" | "both";

  /**
   * The auto-size boundary describes clipping element(s) that overflow will be checked relative to when resizing. By
   * default, the boundary includes overflow ancestors that will cause the element to be clipped. If needed, you can
   * change the boundary by passing a reference to one or more elements to this property.
   */
  @property({ type: Object }) autoSizeBoundary: Element | Element[];

  /** The amount of padding, in pixels, to exceed before the auto-size behavior will occur. */
  @property({ attribute: "auto-size-padding", type: Number }) autoSizePadding =
    0;

  /**
   * When a gap exists between the anchor and the popup element, this option will add a "hover bridge" that fills the
   * gap using an invisible element. This makes listening for events such as `mouseenter` and `mouseleave` more sane
   * because the pointer never technically leaves the element. The hover bridge will only be drawn when the popover is
   * active.
   */
  @property({ attribute: "hover-bridge", type: Boolean }) hoverBridge = false;

  async connectedCallback() {
    super.connectedCallback();

    // Start the positioner after the first update
    await this.updateComplete;
    this.start();
  }

  disconnectedCallback() {
    super.disconnectedCallback();
    this.stop();
  }

  async updated(changedProperties: PropertyValues<this>) {
    super.updated(changedProperties);

    // Start or stop the positioner when active changes
    if (changedProperties.has("active")) {
      if (this.active) {
        this.start();
      } else {
        this.stop();
      }
    }

    // Update the anchor when anchor changes
    if (changedProperties.has("anchor")) {
      this.handleAnchorChange();
    }

    // All other properties will trigger a reposition when active
    if (this.active) {
      await this.updateComplete;
      this.reposition();
    }
  }

  private async handleAnchorChange() {
    await this.stop();

    if (this.anchor && typeof this.anchor === "string") {
      // Locate the anchor by id
      const root = this.getRootNode() as Document | ShadowRoot;
      this.anchorEl = root.getElementById(this.anchor);
    } else if (
      this.anchor instanceof Element ||
      isVirtualElement(this.anchor)
    ) {
      // Use the anchor's reference
      this.anchorEl = this.anchor;
    } else {
      // Look for a slotted anchor
      this.anchorEl = this.querySelector<HTMLElement>('[slot="anchor"]');
    }

    // If the anchor is a `<slot>`, we'll use the first assigned element as the target since slots use
    // `display: contents`
    // and positioning can't be calculated on them
    if (this.anchorEl instanceof HTMLSlotElement) {
      this.anchorEl = this.anchorEl.assignedElements({
        flatten: true,
      })[0] as HTMLElement;
    }

    // If the anchor is valid, start it up
    if (this.anchorEl) {
      this.start();
    }
  }

  private start() {
    // We can't start the positioner without an anchor
    if (!this.anchorEl || !this.active || !this.isConnected) {
      return;
    }

    this.popup?.showPopover?.();

    this.cleanup = autoUpdate(this.anchorEl, this.popup, () => {
      this.reposition();
    });
  }

  private async stop(): Promise<void> {
    return new Promise((resolve) => {
      this.popup?.hidePopover?.();

      if (this.cleanup) {
        this.cleanup();
        this.cleanup = undefined;
        this.removeAttribute("data-current-placement");
        this.style.removeProperty("--auto-size-available-width");
        this.style.removeProperty("--auto-size-available-height");
        requestAnimationFrame(() => resolve());
      } else {
        resolve();
      }
    });
  }

  /** Forces the popup to recalculate and reposition itself. */
  reposition() {
    // Nothing to do if the popup is inactive or the anchor doesn't exist
    if (!this.active || !this.anchorEl || !this.popup) {
      return;
    }

    //
    // NOTE: Floating UI middlewares are order dependent: https://floating-ui.com/docs/middleware
    //
    const middleware = [
      // The offset middleware goes first
      offset({ mainAxis: this.distance, crossAxis: this.skidding }),
    ];

    // First we sync width/height
    if (this.sync) {
      middleware.push(
        size({
          apply: ({ rects }) => {
            const syncWidth = this.sync === "width" || this.sync === "both";
            const syncHeight = this.sync === "height" || this.sync === "both";
            this.popup.style.width = syncWidth
              ? `${rects.reference.width}px`
              : "";
            this.popup.style.height = syncHeight
              ? `${rects.reference.height}px`
              : "";
          },
        }),
      );
    } else {
      // Cleanup styles if we're not matching width/height
      this.popup.style.width = "";
      this.popup.style.height = "";
    }

    let defaultBoundary;

    if (
      SUPPORTS_POPOVER &&
      !isVirtualElement(this.anchor) &&
      this.boundary === "scroll"
    ) {
      // When using the Popover API, the floating element is no longer in the same DOM context
      // as the overflow ancestors so Floating-UI can't find them.
      // For flip, `elementContext: 'reference'` gets it to use the anchor element instead,
      // but the option is not available for shift() or size(), so we basically need to implement it ourselves.
      defaultBoundary = getOverflowAncestors(this.anchorEl as Element).filter(
        (el) => el instanceof Element,
      );
    }

    // Then we flip
    if (this.flip) {
      middleware.push(
        flip({
          boundary: this.flipBoundary || defaultBoundary,
          // @ts-expect-error - We're converting a string attribute to an array here
          fallbackPlacements: this.flipFallbackPlacements,
          fallbackStrategy:
            this.flipFallbackStrategy === "best-fit"
              ? "bestFit"
              : "initialPlacement",
          padding: this.flipPadding,
        }),
      );
    }

    // Then we shift
    if (this.shift) {
      middleware.push(
        shift({
          boundary: this.shiftBoundary || defaultBoundary,
          padding: this.shiftPadding,
        }),
      );
    }

    // Now we adjust the size as needed
    if (this.autoSize) {
      middleware.push(
        size({
          boundary: this.autoSizeBoundary || defaultBoundary,
          padding: this.autoSizePadding,
          apply: ({ availableWidth, availableHeight }) => {
            if (this.autoSize === "vertical" || this.autoSize === "both") {
              this.style.setProperty(
                "--auto-size-available-height",
                `${availableHeight}px`,
              );
            } else {
              this.style.removeProperty("--auto-size-available-height");
            }

            if (this.autoSize === "horizontal" || this.autoSize === "both") {
              this.style.setProperty(
                "--auto-size-available-width",
                `${availableWidth}px`,
              );
            } else {
              this.style.removeProperty("--auto-size-available-width");
            }
          },
        }),
      );
    } else {
      // Cleanup styles if we're no longer using auto-size
      this.style.removeProperty("--auto-size-available-width");
      this.style.removeProperty("--auto-size-available-height");
    }

    // Finally, we add an arrow
    if (this.arrow) {
      middleware.push(
        arrow({
          element: this.arrowEl,
          padding: this.arrowPadding,
        }),
      );
    }

    //
    // Use custom positioning logic if the strategy is absolute. Otherwise, fall back to the default logic.
    //
    // More info: https://github.com/shoelace-style/shoelace/issues/1135
    //
    const getOffsetParent = SUPPORTS_POPOVER
      ? (element: Element) => platform.getOffsetParent(element, offsetParent)
      : platform.getOffsetParent;

    computePosition(this.anchorEl, this.popup, {
      placement: this.placement,
      middleware,
      strategy: SUPPORTS_POPOVER ? "absolute" : "fixed",
      platform: {
        ...platform,
        getOffsetParent,
      },
    }).then(({ x, y, middlewareData, placement }) => {
      //
      // Even though we have our own localization utility, it uses different heuristics to determine RTL. Because of
      // that, we'll use the same approach that Floating UI uses.
      //
      // Source: https://github.com/floating-ui/floating-ui/blob/cb3b6ab07f95275730d3e6e46c702f8d4908b55c/packages/dom/src/utils/getDocumentRect.ts#L31
      //
      const isRtl = this.localize.dir() === "rtl";
      const staticSide = {
        top: "bottom",
        right: "left",
        bottom: "top",
        left: "right",
      }[placement.split("-")[0]]!;

      this.setAttribute("data-current-placement", placement);

      Object.assign(this.popup.style, {
        left: `${x}px`,
        top: `${y}px`,
      });

      if (this.arrow) {
        const arrowX = middlewareData.arrow!.x;
        const arrowY = middlewareData.arrow!.y;
        let top = "";
        let right = "";
        let bottom = "";
        let left = "";

        if (this.arrowPlacement === "start") {
          // Start
          const value =
            typeof arrowX === "number"
              ? `calc(${this.arrowPadding}px - var(--arrow-padding-offset))`
              : "";
          top =
            typeof arrowY === "number"
              ? `calc(${this.arrowPadding}px - var(--arrow-padding-offset))`
              : "";
          right = isRtl ? value : "";
          left = isRtl ? "" : value;
        } else if (this.arrowPlacement === "end") {
          // End
          const value =
            typeof arrowX === "number"
              ? `calc(${this.arrowPadding}px - var(--arrow-padding-offset))`
              : "";
          right = isRtl ? "" : value;
          left = isRtl ? value : "";
          bottom =
            typeof arrowY === "number"
              ? `calc(${this.arrowPadding}px - var(--arrow-padding-offset))`
              : "";
        } else if (this.arrowPlacement === "center") {
          // Center
          left =
            typeof arrowX === "number"
              ? `calc(50% - var(--arrow-size-diagonal))`
              : "";
          top =
            typeof arrowY === "number"
              ? `calc(50% - var(--arrow-size-diagonal))`
              : "";
        } else {
          // Anchor (default)
          left = typeof arrowX === "number" ? `${arrowX}px` : "";
          top = typeof arrowY === "number" ? `${arrowY}px` : "";
        }

        Object.assign(this.arrowEl.style, {
          top,
          right,
          bottom,
          left,
          [staticSide]:
            "calc(var(--arrow-base-offset) - var(--arrow-size-diagonal))",
        });
      }
    });

    // Wait until the new position is drawn before updating the hover bridge, otherwise it can get out of sync
    requestAnimationFrame(() => this.updateHoverBridge());

    this.dispatchEvent(new WaRepositionEvent());
  }

  private updateHoverBridge = () => {
    if (this.hoverBridge && this.anchorEl && this.popup) {
      const anchorRect = this.anchorEl.getBoundingClientRect();
      const popupRect = this.popup.getBoundingClientRect();
      const isVertical =
        this.placement.includes("top") || this.placement.includes("bottom");
      let topLeftX = 0;
      let topLeftY = 0;
      let topRightX = 0;
      let topRightY = 0;
      let bottomLeftX = 0;
      let bottomLeftY = 0;
      let bottomRightX = 0;
      let bottomRightY = 0;

      if (isVertical) {
        if (anchorRect.top < popupRect.top) {
          // Anchor is above
          topLeftX = anchorRect.left;
          topLeftY = anchorRect.bottom;
          topRightX = anchorRect.right;
          topRightY = anchorRect.bottom;

          bottomLeftX = popupRect.left;
          bottomLeftY = popupRect.top;
          bottomRightX = popupRect.right;
          bottomRightY = popupRect.top;
        } else {
          // Anchor is below
          topLeftX = popupRect.left;
          topLeftY = popupRect.bottom;
          topRightX = popupRect.right;
          topRightY = popupRect.bottom;

          bottomLeftX = anchorRect.left;
          bottomLeftY = anchorRect.top;
          bottomRightX = anchorRect.right;
          bottomRightY = anchorRect.top;
        }
      } else {
        if (anchorRect.left < popupRect.left) {
          // Anchor is on the left
          topLeftX = anchorRect.right;
          topLeftY = anchorRect.top;
          topRightX = popupRect.left;
          topRightY = popupRect.top;

          bottomLeftX = anchorRect.right;
          bottomLeftY = anchorRect.bottom;
          bottomRightX = popupRect.left;
          bottomRightY = popupRect.bottom;
        } else {
          // Anchor is on the right
          topLeftX = popupRect.right;
          topLeftY = popupRect.top;
          topRightX = anchorRect.left;
          topRightY = anchorRect.top;

          bottomLeftX = popupRect.right;
          bottomLeftY = popupRect.bottom;
          bottomRightX = anchorRect.left;
          bottomRightY = anchorRect.bottom;
        }
      }

      this.style.setProperty("--hover-bridge-top-left-x", `${topLeftX}px`);
      this.style.setProperty("--hover-bridge-top-left-y", `${topLeftY}px`);
      this.style.setProperty("--hover-bridge-top-right-x", `${topRightX}px`);
      this.style.setProperty("--hover-bridge-top-right-y", `${topRightY}px`);
      this.style.setProperty(
        "--hover-bridge-bottom-left-x",
        `${bottomLeftX}px`,
      );
      this.style.setProperty(
        "--hover-bridge-bottom-left-y",
        `${bottomLeftY}px`,
      );
      this.style.setProperty(
        "--hover-bridge-bottom-right-x",
        `${bottomRightX}px`,
      );
      this.style.setProperty(
        "--hover-bridge-bottom-right-y",
        `${bottomRightY}px`,
      );
    }
  };

  render() {
    return html`
      <slot name="anchor" @slotchange=${this.handleAnchorChange}></slot>

      <span
        part="hover-bridge"
        class=${classMap({
          "popup-hover-bridge": true,
          "popup-hover-bridge-visible": this.hoverBridge && this.active,
        })}
      ></span>

      <div
        popover="manual"
        part="popup"
        class=${classMap({
          popup: true,
          "popup-active": this.active,
          "popup-fixed": !SUPPORTS_POPOVER,
          "popup-has-arrow": this.arrow,
        })}
      >
        <slot></slot>
        ${this.arrow ? html`<div part="arrow" class="arrow" role="presentation"></div>` : ""}
      </div>
    `;
  }
}

declare global {
  interface HTMLElementTagNameMap {
    "wa-popup": WaPopup;
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
@@ -133,9 +133,12 @@
    * anchor as possible, considering available space and `arrow-padding`. A value of `start`, `end`, or `center` will
    * align the arrow to the start, end, or center of the popover instead.
    */
   @property({ attribute: "arrow-placement" }) arrowPlacement:
-    "start" | "end" | "center" | "anchor" = "anchor";
+    | "start"
+    | "end"
+    | "center"
+    | "anchor" = "anchor";
 
   /**
    * The amount of padding between the arrow and the edges of the popup. If the popup has a border-radius, for example,
    * this will prevent it from overflowing the corners.
@@ -616,15 +619,13 @@
           "popup-has-arrow": this.arrow,
         })}
       >
         <slot></slot>
-        ${
-          this.arrow
-            ? html`
-                <div part="arrow" class="arrow" role="presentation"></div>
-              `
-            : ""
-        }
+        ${this.arrow
+          ? html`
+              <div part="arrow" class="arrow" role="presentation"></div>
+            `
+          : ""}
       </div>
     `;
   }
 }

`````

### Actual (oxfmt)

`````ts
import {
  arrow,
  autoUpdate,
  computePosition,
  flip,
  getOverflowAncestors,
  offset,
  platform,
  shift,
  size,
} from "@floating-ui/dom";
import { offsetParent } from "composed-offset-position";
import type { PropertyValues } from "lit";
import { html } from "lit";
import { customElement, property, query } from "lit/decorators.js";
import { classMap } from "lit/directives/class-map.js";
import { WaRepositionEvent } from "../../events/reposition.js";
import WebAwesomeElement from "../../internal/webawesome-element.js";
import { LocalizeController } from "../../utilities/localize.js";
import styles from "./popup.styles.js";

export interface VirtualElement {
  getBoundingClientRect: () => DOMRect;
  contextElement?: Element;
}

function isVirtualElement(e: unknown): e is VirtualElement {
  return (
    e !== null &&
    typeof e === "object" &&
    "getBoundingClientRect" in e &&
    ("contextElement" in e ? e instanceof Element : true)
  );
}

const SUPPORTS_POPOVER = globalThis?.HTMLElement?.prototype.hasOwnProperty("popover");

/**
 * @summary Popups declaratively anchor one element to another and keep them positioned together as the page scrolls or
 *  resizes. Primarily a low-level building block for popovers, dropdowns, and tooltips.
 * @documentation https://webawesome.com/docs/components/popup
 * @status stable
 * @since 2.0
 *
 * @event wa-reposition - Emitted when the popup is repositioned. This event can fire a lot, so avoid putting expensive
 *  operations in your listener or consider debouncing it.
 *
 * @slot - The popup's content.
 * @slot anchor - The element the popup will be anchored to. If the anchor lives outside of the popup, you can use the
 *  `anchor` attribute or property instead.
 *
 * @csspart arrow - The arrow's container. Avoid setting `top|bottom|left|right` properties, as these values are
 *  assigned dynamically as the popup moves. This is most useful for applying a background color to match the popup, and
 *  maybe a border or box shadow.
 * @csspart popup - The popup's container. Useful for setting a background color, box shadow, etc.
 * @csspart hover-bridge - The hover bridge element. Only available when the `hover-bridge` option is enabled.
 *
 * @cssproperty [--arrow-size=6px] - The size of the arrow. Note that an arrow won't be shown unless the `arrow`
 *  attribute is used.
 * @cssproperty [--popup-border-width] - The width of any custom border applied to the popup. This is used to reposition
 *  the arrow to overlap to the inside edge of the popup border.
 * @cssproperty [--arrow-color=black] - The color of the arrow.
 * @cssproperty [--auto-size-available-width] - A read-only custom property that determines the amount of width the
 *  popup can be before overflowing. Useful for positioning child elements that need to overflow. This property is only
 *  available when using `auto-size`.
 * @cssproperty [--auto-size-available-height] - A read-only custom property that determines the amount of height the
 *  popup can be before overflowing. Useful for positioning child elements that need to overflow. This property is only
 *  available when using `auto-size`.
 * @cssproperty [--show-duration=100ms] - The show duration to use when applying built-in animation classes.
 * @cssproperty [--hide-duration=100ms] - The hide duration to use when applying built-in animation classes.
 */
@customElement("wa-popup")
export default class WaPopup extends WebAwesomeElement {
  static css = styles;

  private anchorEl: Element | VirtualElement | null;
  private cleanup: ReturnType<typeof autoUpdate> | undefined;
  private readonly localize = new LocalizeController(this);

  /** A reference to the internal popup container. Useful for animating and styling the popup with JavaScript. */
  @query(".popup") popup: HTMLElement;
  @query(".arrow") private arrowEl: HTMLElement;

  /**
   * The element the popup will be anchored to. If the anchor lives outside of the popup, you can provide the anchor
   * element `id`, a DOM element reference, or a `VirtualElement`. If the anchor lives inside the popup, use the
   * `anchor` slot instead.
   */
  @property() anchor: Element | string | VirtualElement;

  /**
   * Activates the positioning logic and shows the popup. When this attribute is removed, the positioning logic is torn
   * down and the popup will be hidden.
   */
  @property({ type: Boolean, reflect: true }) active = false;

  /**
   * The preferred placement of the popup. Note that the actual placement will vary as configured to keep the
   * panel inside of the viewport.
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
    | "left-end" = "top";

  /** The bounding box to use for flipping, shifting, and auto-sizing. */
  @property() boundary: "viewport" | "scroll" = "viewport";

  /** The distance in pixels from which to offset the panel away from its anchor. */
  @property({ type: Number }) distance = 0;

  /** The distance in pixels from which to offset the panel along its anchor. */
  @property({ type: Number }) skidding = 0;

  /**
   * Attaches an arrow to the popup. The arrow's size and color can be customized using the `--arrow-size` and
   * `--arrow-color` custom properties. For additional customizations, you can also target the arrow using
   * `::part(arrow)` in your stylesheet.
   */
  @property({ type: Boolean }) arrow = false;

  /**
   * The placement of the arrow. The default is `anchor`, which will align the arrow as close to the center of the
   * anchor as possible, considering available space and `arrow-padding`. A value of `start`, `end`, or `center` will
   * align the arrow to the start, end, or center of the popover instead.
   */
  @property({ attribute: "arrow-placement" }) arrowPlacement:
    | "start"
    | "end"
    | "center"
    | "anchor" = "anchor";

  /**
   * The amount of padding between the arrow and the edges of the popup. If the popup has a border-radius, for example,
   * this will prevent it from overflowing the corners.
   */
  @property({ attribute: "arrow-padding", type: Number }) arrowPadding = 10;

  /**
   * When set, placement of the popup will flip to the opposite site to keep it in view. You can use
   * `flipFallbackPlacements` to further configure how the fallback placement is determined.
   */
  @property({ type: Boolean }) flip = false;

  /**
   * If the preferred placement doesn't fit, popup will be tested in these fallback placements until one fits. Must be a
   * string of any number of placements separated by a space, e.g. "top bottom left". If no placement fits, the flip
   * fallback strategy will be used instead.
   * */
  @property({
    attribute: "flip-fallback-placements",
    converter: {
      fromAttribute: (value: string) => {
        return value
          .split(" ")
          .map((p) => p.trim())
          .filter((p) => p !== "");
      },
      toAttribute: (value: []) => {
        return value.join(" ");
      },
    },
  })
  flipFallbackPlacements = "";

  /**
   * When neither the preferred placement nor the fallback placements fit, this value will be used to determine whether
   * the popup should be positioned using the best available fit based on available space or as it was initially
   * preferred.
   */
  @property({ attribute: "flip-fallback-strategy" }) flipFallbackStrategy: "best-fit" | "initial" =
    "best-fit";

  /**
   * The flip boundary describes clipping element(s) that overflow will be checked relative to when flipping. By
   * default, the boundary includes overflow ancestors that will cause the element to be clipped. If needed, you can
   * change the boundary by passing a reference to one or more elements to this property.
   */
  @property({ type: Object }) flipBoundary: Element | Element[];

  /** The amount of padding, in pixels, to exceed before the flip behavior will occur. */
  @property({ attribute: "flip-padding", type: Number }) flipPadding = 0;

  /** Moves the popup along the axis to keep it in view when clipped. */
  @property({ type: Boolean }) shift = false;

  /**
   * The shift boundary describes clipping element(s) that overflow will be checked relative to when shifting. By
   * default, the boundary includes overflow ancestors that will cause the element to be clipped. If needed, you can
   * change the boundary by passing a reference to one or more elements to this property.
   */
  @property({ type: Object }) shiftBoundary: Element | Element[];

  /** The amount of padding, in pixels, to exceed before the shift behavior will occur. */
  @property({ attribute: "shift-padding", type: Number }) shiftPadding = 0;

  /** When set, this will cause the popup to automatically resize itself to prevent it from overflowing. */
  @property({ attribute: "auto-size" }) autoSize: "horizontal" | "vertical" | "both";

  /** Syncs the popup's width or height to that of the anchor element. */
  @property() sync: "width" | "height" | "both";

  /**
   * The auto-size boundary describes clipping element(s) that overflow will be checked relative to when resizing. By
   * default, the boundary includes overflow ancestors that will cause the element to be clipped. If needed, you can
   * change the boundary by passing a reference to one or more elements to this property.
   */
  @property({ type: Object }) autoSizeBoundary: Element | Element[];

  /** The amount of padding, in pixels, to exceed before the auto-size behavior will occur. */
  @property({ attribute: "auto-size-padding", type: Number }) autoSizePadding = 0;

  /**
   * When a gap exists between the anchor and the popup element, this option will add a "hover bridge" that fills the
   * gap using an invisible element. This makes listening for events such as `mouseenter` and `mouseleave` more sane
   * because the pointer never technically leaves the element. The hover bridge will only be drawn when the popover is
   * active.
   */
  @property({ attribute: "hover-bridge", type: Boolean }) hoverBridge = false;

  async connectedCallback() {
    super.connectedCallback();

    // Start the positioner after the first update
    await this.updateComplete;
    this.start();
  }

  disconnectedCallback() {
    super.disconnectedCallback();
    this.stop();
  }

  async updated(changedProperties: PropertyValues<this>) {
    super.updated(changedProperties);

    // Start or stop the positioner when active changes
    if (changedProperties.has("active")) {
      if (this.active) {
        this.start();
      } else {
        this.stop();
      }
    }

    // Update the anchor when anchor changes
    if (changedProperties.has("anchor")) {
      this.handleAnchorChange();
    }

    // All other properties will trigger a reposition when active
    if (this.active) {
      await this.updateComplete;
      this.reposition();
    }
  }

  private async handleAnchorChange() {
    await this.stop();

    if (this.anchor && typeof this.anchor === "string") {
      // Locate the anchor by id
      const root = this.getRootNode() as Document | ShadowRoot;
      this.anchorEl = root.getElementById(this.anchor);
    } else if (this.anchor instanceof Element || isVirtualElement(this.anchor)) {
      // Use the anchor's reference
      this.anchorEl = this.anchor;
    } else {
      // Look for a slotted anchor
      this.anchorEl = this.querySelector<HTMLElement>('[slot="anchor"]');
    }

    // If the anchor is a `<slot>`, we'll use the first assigned element as the target since slots use
    // `display: contents`
    // and positioning can't be calculated on them
    if (this.anchorEl instanceof HTMLSlotElement) {
      this.anchorEl = this.anchorEl.assignedElements({ flatten: true })[0] as HTMLElement;
    }

    // If the anchor is valid, start it up
    if (this.anchorEl) {
      this.start();
    }
  }

  private start() {
    // We can't start the positioner without an anchor
    if (!this.anchorEl || !this.active || !this.isConnected) {
      return;
    }

    this.popup?.showPopover?.();

    this.cleanup = autoUpdate(this.anchorEl, this.popup, () => {
      this.reposition();
    });
  }

  private async stop(): Promise<void> {
    return new Promise((resolve) => {
      this.popup?.hidePopover?.();

      if (this.cleanup) {
        this.cleanup();
        this.cleanup = undefined;
        this.removeAttribute("data-current-placement");
        this.style.removeProperty("--auto-size-available-width");
        this.style.removeProperty("--auto-size-available-height");
        requestAnimationFrame(() => resolve());
      } else {
        resolve();
      }
    });
  }

  /** Forces the popup to recalculate and reposition itself. */
  reposition() {
    // Nothing to do if the popup is inactive or the anchor doesn't exist
    if (!this.active || !this.anchorEl || !this.popup) {
      return;
    }

    //
    // NOTE: Floating UI middlewares are order dependent: https://floating-ui.com/docs/middleware
    //
    const middleware = [
      // The offset middleware goes first
      offset({ mainAxis: this.distance, crossAxis: this.skidding }),
    ];

    // First we sync width/height
    if (this.sync) {
      middleware.push(
        size({
          apply: ({ rects }) => {
            const syncWidth = this.sync === "width" || this.sync === "both";
            const syncHeight = this.sync === "height" || this.sync === "both";
            this.popup.style.width = syncWidth ? `${rects.reference.width}px` : "";
            this.popup.style.height = syncHeight ? `${rects.reference.height}px` : "";
          },
        }),
      );
    } else {
      // Cleanup styles if we're not matching width/height
      this.popup.style.width = "";
      this.popup.style.height = "";
    }

    let defaultBoundary;

    if (SUPPORTS_POPOVER && !isVirtualElement(this.anchor) && this.boundary === "scroll") {
      // When using the Popover API, the floating element is no longer in the same DOM context
      // as the overflow ancestors so Floating-UI can't find them.
      // For flip, `elementContext: 'reference'` gets it to use the anchor element instead,
      // but the option is not available for shift() or size(), so we basically need to implement it ourselves.
      defaultBoundary = getOverflowAncestors(this.anchorEl as Element).filter(
        (el) => el instanceof Element,
      );
    }

    // Then we flip
    if (this.flip) {
      middleware.push(
        flip({
          boundary: this.flipBoundary || defaultBoundary,
          // @ts-expect-error - We're converting a string attribute to an array here
          fallbackPlacements: this.flipFallbackPlacements,
          fallbackStrategy:
            this.flipFallbackStrategy === "best-fit" ? "bestFit" : "initialPlacement",
          padding: this.flipPadding,
        }),
      );
    }

    // Then we shift
    if (this.shift) {
      middleware.push(
        shift({
          boundary: this.shiftBoundary || defaultBoundary,
          padding: this.shiftPadding,
        }),
      );
    }

    // Now we adjust the size as needed
    if (this.autoSize) {
      middleware.push(
        size({
          boundary: this.autoSizeBoundary || defaultBoundary,
          padding: this.autoSizePadding,
          apply: ({ availableWidth, availableHeight }) => {
            if (this.autoSize === "vertical" || this.autoSize === "both") {
              this.style.setProperty("--auto-size-available-height", `${availableHeight}px`);
            } else {
              this.style.removeProperty("--auto-size-available-height");
            }

            if (this.autoSize === "horizontal" || this.autoSize === "both") {
              this.style.setProperty("--auto-size-available-width", `${availableWidth}px`);
            } else {
              this.style.removeProperty("--auto-size-available-width");
            }
          },
        }),
      );
    } else {
      // Cleanup styles if we're no longer using auto-size
      this.style.removeProperty("--auto-size-available-width");
      this.style.removeProperty("--auto-size-available-height");
    }

    // Finally, we add an arrow
    if (this.arrow) {
      middleware.push(
        arrow({
          element: this.arrowEl,
          padding: this.arrowPadding,
        }),
      );
    }

    //
    // Use custom positioning logic if the strategy is absolute. Otherwise, fall back to the default logic.
    //
    // More info: https://github.com/shoelace-style/shoelace/issues/1135
    //
    const getOffsetParent = SUPPORTS_POPOVER
      ? (element: Element) => platform.getOffsetParent(element, offsetParent)
      : platform.getOffsetParent;

    computePosition(this.anchorEl, this.popup, {
      placement: this.placement,
      middleware,
      strategy: SUPPORTS_POPOVER ? "absolute" : "fixed",
      platform: {
        ...platform,
        getOffsetParent,
      },
    }).then(({ x, y, middlewareData, placement }) => {
      //
      // Even though we have our own localization utility, it uses different heuristics to determine RTL. Because of
      // that, we'll use the same approach that Floating UI uses.
      //
      // Source: https://github.com/floating-ui/floating-ui/blob/cb3b6ab07f95275730d3e6e46c702f8d4908b55c/packages/dom/src/utils/getDocumentRect.ts#L31
      //
      const isRtl = this.localize.dir() === "rtl";
      const staticSide = { top: "bottom", right: "left", bottom: "top", left: "right" }[
        placement.split("-")[0]
      ]!;

      this.setAttribute("data-current-placement", placement);

      Object.assign(this.popup.style, {
        left: `${x}px`,
        top: `${y}px`,
      });

      if (this.arrow) {
        const arrowX = middlewareData.arrow!.x;
        const arrowY = middlewareData.arrow!.y;
        let top = "";
        let right = "";
        let bottom = "";
        let left = "";

        if (this.arrowPlacement === "start") {
          // Start
          const value =
            typeof arrowX === "number"
              ? `calc(${this.arrowPadding}px - var(--arrow-padding-offset))`
              : "";
          top =
            typeof arrowY === "number"
              ? `calc(${this.arrowPadding}px - var(--arrow-padding-offset))`
              : "";
          right = isRtl ? value : "";
          left = isRtl ? "" : value;
        } else if (this.arrowPlacement === "end") {
          // End
          const value =
            typeof arrowX === "number"
              ? `calc(${this.arrowPadding}px - var(--arrow-padding-offset))`
              : "";
          right = isRtl ? "" : value;
          left = isRtl ? value : "";
          bottom =
            typeof arrowY === "number"
              ? `calc(${this.arrowPadding}px - var(--arrow-padding-offset))`
              : "";
        } else if (this.arrowPlacement === "center") {
          // Center
          left = typeof arrowX === "number" ? `calc(50% - var(--arrow-size-diagonal))` : "";
          top = typeof arrowY === "number" ? `calc(50% - var(--arrow-size-diagonal))` : "";
        } else {
          // Anchor (default)
          left = typeof arrowX === "number" ? `${arrowX}px` : "";
          top = typeof arrowY === "number" ? `${arrowY}px` : "";
        }

        Object.assign(this.arrowEl.style, {
          top,
          right,
          bottom,
          left,
          [staticSide]: "calc(var(--arrow-base-offset) - var(--arrow-size-diagonal))",
        });
      }
    });

    // Wait until the new position is drawn before updating the hover bridge, otherwise it can get out of sync
    requestAnimationFrame(() => this.updateHoverBridge());

    this.dispatchEvent(new WaRepositionEvent());
  }

  private updateHoverBridge = () => {
    if (this.hoverBridge && this.anchorEl && this.popup) {
      const anchorRect = this.anchorEl.getBoundingClientRect();
      const popupRect = this.popup.getBoundingClientRect();
      const isVertical = this.placement.includes("top") || this.placement.includes("bottom");
      let topLeftX = 0;
      let topLeftY = 0;
      let topRightX = 0;
      let topRightY = 0;
      let bottomLeftX = 0;
      let bottomLeftY = 0;
      let bottomRightX = 0;
      let bottomRightY = 0;

      if (isVertical) {
        if (anchorRect.top < popupRect.top) {
          // Anchor is above
          topLeftX = anchorRect.left;
          topLeftY = anchorRect.bottom;
          topRightX = anchorRect.right;
          topRightY = anchorRect.bottom;

          bottomLeftX = popupRect.left;
          bottomLeftY = popupRect.top;
          bottomRightX = popupRect.right;
          bottomRightY = popupRect.top;
        } else {
          // Anchor is below
          topLeftX = popupRect.left;
          topLeftY = popupRect.bottom;
          topRightX = popupRect.right;
          topRightY = popupRect.bottom;

          bottomLeftX = anchorRect.left;
          bottomLeftY = anchorRect.top;
          bottomRightX = anchorRect.right;
          bottomRightY = anchorRect.top;
        }
      } else {
        if (anchorRect.left < popupRect.left) {
          // Anchor is on the left
          topLeftX = anchorRect.right;
          topLeftY = anchorRect.top;
          topRightX = popupRect.left;
          topRightY = popupRect.top;

          bottomLeftX = anchorRect.right;
          bottomLeftY = anchorRect.bottom;
          bottomRightX = popupRect.left;
          bottomRightY = popupRect.bottom;
        } else {
          // Anchor is on the right
          topLeftX = popupRect.right;
          topLeftY = popupRect.top;
          topRightX = anchorRect.left;
          topRightY = anchorRect.top;

          bottomLeftX = popupRect.right;
          bottomLeftY = popupRect.bottom;
          bottomRightX = anchorRect.left;
          bottomRightY = anchorRect.bottom;
        }
      }

      this.style.setProperty("--hover-bridge-top-left-x", `${topLeftX}px`);
      this.style.setProperty("--hover-bridge-top-left-y", `${topLeftY}px`);
      this.style.setProperty("--hover-bridge-top-right-x", `${topRightX}px`);
      this.style.setProperty("--hover-bridge-top-right-y", `${topRightY}px`);
      this.style.setProperty("--hover-bridge-bottom-left-x", `${bottomLeftX}px`);
      this.style.setProperty("--hover-bridge-bottom-left-y", `${bottomLeftY}px`);
      this.style.setProperty("--hover-bridge-bottom-right-x", `${bottomRightX}px`);
      this.style.setProperty("--hover-bridge-bottom-right-y", `${bottomRightY}px`);
    }
  };

  render() {
    return html`
      <slot name="anchor" @slotchange=${this.handleAnchorChange}></slot>

      <span
        part="hover-bridge"
        class=${classMap({
          "popup-hover-bridge": true,
          "popup-hover-bridge-visible": this.hoverBridge && this.active,
        })}
      ></span>

      <div
        popover="manual"
        part="popup"
        class=${classMap({
          popup: true,
          "popup-active": this.active,
          "popup-fixed": !SUPPORTS_POPOVER,
          "popup-has-arrow": this.arrow,
        })}
      >
        <slot></slot>
        ${this.arrow
          ? html`
              <div part="arrow" class="arrow" role="presentation"></div>
            `
          : ""}
      </div>
    `;
  }
}

declare global {
  interface HTMLElementTagNameMap {
    "wa-popup": WaPopup;
  }
}

`````

### Expected (prettier)

`````ts
import {
  arrow,
  autoUpdate,
  computePosition,
  flip,
  getOverflowAncestors,
  offset,
  platform,
  shift,
  size,
} from "@floating-ui/dom";
import { offsetParent } from "composed-offset-position";
import type { PropertyValues } from "lit";
import { html } from "lit";
import { customElement, property, query } from "lit/decorators.js";
import { classMap } from "lit/directives/class-map.js";
import { WaRepositionEvent } from "../../events/reposition.js";
import WebAwesomeElement from "../../internal/webawesome-element.js";
import { LocalizeController } from "../../utilities/localize.js";
import styles from "./popup.styles.js";

export interface VirtualElement {
  getBoundingClientRect: () => DOMRect;
  contextElement?: Element;
}

function isVirtualElement(e: unknown): e is VirtualElement {
  return (
    e !== null &&
    typeof e === "object" &&
    "getBoundingClientRect" in e &&
    ("contextElement" in e ? e instanceof Element : true)
  );
}

const SUPPORTS_POPOVER = globalThis?.HTMLElement?.prototype.hasOwnProperty("popover");

/**
 * @summary Popups declaratively anchor one element to another and keep them positioned together as the page scrolls or
 *  resizes. Primarily a low-level building block for popovers, dropdowns, and tooltips.
 * @documentation https://webawesome.com/docs/components/popup
 * @status stable
 * @since 2.0
 *
 * @event wa-reposition - Emitted when the popup is repositioned. This event can fire a lot, so avoid putting expensive
 *  operations in your listener or consider debouncing it.
 *
 * @slot - The popup's content.
 * @slot anchor - The element the popup will be anchored to. If the anchor lives outside of the popup, you can use the
 *  `anchor` attribute or property instead.
 *
 * @csspart arrow - The arrow's container. Avoid setting `top|bottom|left|right` properties, as these values are
 *  assigned dynamically as the popup moves. This is most useful for applying a background color to match the popup, and
 *  maybe a border or box shadow.
 * @csspart popup - The popup's container. Useful for setting a background color, box shadow, etc.
 * @csspart hover-bridge - The hover bridge element. Only available when the `hover-bridge` option is enabled.
 *
 * @cssproperty [--arrow-size=6px] - The size of the arrow. Note that an arrow won't be shown unless the `arrow`
 *  attribute is used.
 * @cssproperty [--popup-border-width] - The width of any custom border applied to the popup. This is used to reposition
 *  the arrow to overlap to the inside edge of the popup border.
 * @cssproperty [--arrow-color=black] - The color of the arrow.
 * @cssproperty [--auto-size-available-width] - A read-only custom property that determines the amount of width the
 *  popup can be before overflowing. Useful for positioning child elements that need to overflow. This property is only
 *  available when using `auto-size`.
 * @cssproperty [--auto-size-available-height] - A read-only custom property that determines the amount of height the
 *  popup can be before overflowing. Useful for positioning child elements that need to overflow. This property is only
 *  available when using `auto-size`.
 * @cssproperty [--show-duration=100ms] - The show duration to use when applying built-in animation classes.
 * @cssproperty [--hide-duration=100ms] - The hide duration to use when applying built-in animation classes.
 */
@customElement("wa-popup")
export default class WaPopup extends WebAwesomeElement {
  static css = styles;

  private anchorEl: Element | VirtualElement | null;
  private cleanup: ReturnType<typeof autoUpdate> | undefined;
  private readonly localize = new LocalizeController(this);

  /** A reference to the internal popup container. Useful for animating and styling the popup with JavaScript. */
  @query(".popup") popup: HTMLElement;
  @query(".arrow") private arrowEl: HTMLElement;

  /**
   * The element the popup will be anchored to. If the anchor lives outside of the popup, you can provide the anchor
   * element `id`, a DOM element reference, or a `VirtualElement`. If the anchor lives inside the popup, use the
   * `anchor` slot instead.
   */
  @property() anchor: Element | string | VirtualElement;

  /**
   * Activates the positioning logic and shows the popup. When this attribute is removed, the positioning logic is torn
   * down and the popup will be hidden.
   */
  @property({ type: Boolean, reflect: true }) active = false;

  /**
   * The preferred placement of the popup. Note that the actual placement will vary as configured to keep the
   * panel inside of the viewport.
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
    | "left-end" = "top";

  /** The bounding box to use for flipping, shifting, and auto-sizing. */
  @property() boundary: "viewport" | "scroll" = "viewport";

  /** The distance in pixels from which to offset the panel away from its anchor. */
  @property({ type: Number }) distance = 0;

  /** The distance in pixels from which to offset the panel along its anchor. */
  @property({ type: Number }) skidding = 0;

  /**
   * Attaches an arrow to the popup. The arrow's size and color can be customized using the `--arrow-size` and
   * `--arrow-color` custom properties. For additional customizations, you can also target the arrow using
   * `::part(arrow)` in your stylesheet.
   */
  @property({ type: Boolean }) arrow = false;

  /**
   * The placement of the arrow. The default is `anchor`, which will align the arrow as close to the center of the
   * anchor as possible, considering available space and `arrow-padding`. A value of `start`, `end`, or `center` will
   * align the arrow to the start, end, or center of the popover instead.
   */
  @property({ attribute: "arrow-placement" }) arrowPlacement:
    "start" | "end" | "center" | "anchor" = "anchor";

  /**
   * The amount of padding between the arrow and the edges of the popup. If the popup has a border-radius, for example,
   * this will prevent it from overflowing the corners.
   */
  @property({ attribute: "arrow-padding", type: Number }) arrowPadding = 10;

  /**
   * When set, placement of the popup will flip to the opposite site to keep it in view. You can use
   * `flipFallbackPlacements` to further configure how the fallback placement is determined.
   */
  @property({ type: Boolean }) flip = false;

  /**
   * If the preferred placement doesn't fit, popup will be tested in these fallback placements until one fits. Must be a
   * string of any number of placements separated by a space, e.g. "top bottom left". If no placement fits, the flip
   * fallback strategy will be used instead.
   * */
  @property({
    attribute: "flip-fallback-placements",
    converter: {
      fromAttribute: (value: string) => {
        return value
          .split(" ")
          .map((p) => p.trim())
          .filter((p) => p !== "");
      },
      toAttribute: (value: []) => {
        return value.join(" ");
      },
    },
  })
  flipFallbackPlacements = "";

  /**
   * When neither the preferred placement nor the fallback placements fit, this value will be used to determine whether
   * the popup should be positioned using the best available fit based on available space or as it was initially
   * preferred.
   */
  @property({ attribute: "flip-fallback-strategy" }) flipFallbackStrategy: "best-fit" | "initial" =
    "best-fit";

  /**
   * The flip boundary describes clipping element(s) that overflow will be checked relative to when flipping. By
   * default, the boundary includes overflow ancestors that will cause the element to be clipped. If needed, you can
   * change the boundary by passing a reference to one or more elements to this property.
   */
  @property({ type: Object }) flipBoundary: Element | Element[];

  /** The amount of padding, in pixels, to exceed before the flip behavior will occur. */
  @property({ attribute: "flip-padding", type: Number }) flipPadding = 0;

  /** Moves the popup along the axis to keep it in view when clipped. */
  @property({ type: Boolean }) shift = false;

  /**
   * The shift boundary describes clipping element(s) that overflow will be checked relative to when shifting. By
   * default, the boundary includes overflow ancestors that will cause the element to be clipped. If needed, you can
   * change the boundary by passing a reference to one or more elements to this property.
   */
  @property({ type: Object }) shiftBoundary: Element | Element[];

  /** The amount of padding, in pixels, to exceed before the shift behavior will occur. */
  @property({ attribute: "shift-padding", type: Number }) shiftPadding = 0;

  /** When set, this will cause the popup to automatically resize itself to prevent it from overflowing. */
  @property({ attribute: "auto-size" }) autoSize: "horizontal" | "vertical" | "both";

  /** Syncs the popup's width or height to that of the anchor element. */
  @property() sync: "width" | "height" | "both";

  /**
   * The auto-size boundary describes clipping element(s) that overflow will be checked relative to when resizing. By
   * default, the boundary includes overflow ancestors that will cause the element to be clipped. If needed, you can
   * change the boundary by passing a reference to one or more elements to this property.
   */
  @property({ type: Object }) autoSizeBoundary: Element | Element[];

  /** The amount of padding, in pixels, to exceed before the auto-size behavior will occur. */
  @property({ attribute: "auto-size-padding", type: Number }) autoSizePadding = 0;

  /**
   * When a gap exists between the anchor and the popup element, this option will add a "hover bridge" that fills the
   * gap using an invisible element. This makes listening for events such as `mouseenter` and `mouseleave` more sane
   * because the pointer never technically leaves the element. The hover bridge will only be drawn when the popover is
   * active.
   */
  @property({ attribute: "hover-bridge", type: Boolean }) hoverBridge = false;

  async connectedCallback() {
    super.connectedCallback();

    // Start the positioner after the first update
    await this.updateComplete;
    this.start();
  }

  disconnectedCallback() {
    super.disconnectedCallback();
    this.stop();
  }

  async updated(changedProperties: PropertyValues<this>) {
    super.updated(changedProperties);

    // Start or stop the positioner when active changes
    if (changedProperties.has("active")) {
      if (this.active) {
        this.start();
      } else {
        this.stop();
      }
    }

    // Update the anchor when anchor changes
    if (changedProperties.has("anchor")) {
      this.handleAnchorChange();
    }

    // All other properties will trigger a reposition when active
    if (this.active) {
      await this.updateComplete;
      this.reposition();
    }
  }

  private async handleAnchorChange() {
    await this.stop();

    if (this.anchor && typeof this.anchor === "string") {
      // Locate the anchor by id
      const root = this.getRootNode() as Document | ShadowRoot;
      this.anchorEl = root.getElementById(this.anchor);
    } else if (this.anchor instanceof Element || isVirtualElement(this.anchor)) {
      // Use the anchor's reference
      this.anchorEl = this.anchor;
    } else {
      // Look for a slotted anchor
      this.anchorEl = this.querySelector<HTMLElement>('[slot="anchor"]');
    }

    // If the anchor is a `<slot>`, we'll use the first assigned element as the target since slots use
    // `display: contents`
    // and positioning can't be calculated on them
    if (this.anchorEl instanceof HTMLSlotElement) {
      this.anchorEl = this.anchorEl.assignedElements({ flatten: true })[0] as HTMLElement;
    }

    // If the anchor is valid, start it up
    if (this.anchorEl) {
      this.start();
    }
  }

  private start() {
    // We can't start the positioner without an anchor
    if (!this.anchorEl || !this.active || !this.isConnected) {
      return;
    }

    this.popup?.showPopover?.();

    this.cleanup = autoUpdate(this.anchorEl, this.popup, () => {
      this.reposition();
    });
  }

  private async stop(): Promise<void> {
    return new Promise((resolve) => {
      this.popup?.hidePopover?.();

      if (this.cleanup) {
        this.cleanup();
        this.cleanup = undefined;
        this.removeAttribute("data-current-placement");
        this.style.removeProperty("--auto-size-available-width");
        this.style.removeProperty("--auto-size-available-height");
        requestAnimationFrame(() => resolve());
      } else {
        resolve();
      }
    });
  }

  /** Forces the popup to recalculate and reposition itself. */
  reposition() {
    // Nothing to do if the popup is inactive or the anchor doesn't exist
    if (!this.active || !this.anchorEl || !this.popup) {
      return;
    }

    //
    // NOTE: Floating UI middlewares are order dependent: https://floating-ui.com/docs/middleware
    //
    const middleware = [
      // The offset middleware goes first
      offset({ mainAxis: this.distance, crossAxis: this.skidding }),
    ];

    // First we sync width/height
    if (this.sync) {
      middleware.push(
        size({
          apply: ({ rects }) => {
            const syncWidth = this.sync === "width" || this.sync === "both";
            const syncHeight = this.sync === "height" || this.sync === "both";
            this.popup.style.width = syncWidth ? `${rects.reference.width}px` : "";
            this.popup.style.height = syncHeight ? `${rects.reference.height}px` : "";
          },
        }),
      );
    } else {
      // Cleanup styles if we're not matching width/height
      this.popup.style.width = "";
      this.popup.style.height = "";
    }

    let defaultBoundary;

    if (SUPPORTS_POPOVER && !isVirtualElement(this.anchor) && this.boundary === "scroll") {
      // When using the Popover API, the floating element is no longer in the same DOM context
      // as the overflow ancestors so Floating-UI can't find them.
      // For flip, `elementContext: 'reference'` gets it to use the anchor element instead,
      // but the option is not available for shift() or size(), so we basically need to implement it ourselves.
      defaultBoundary = getOverflowAncestors(this.anchorEl as Element).filter(
        (el) => el instanceof Element,
      );
    }

    // Then we flip
    if (this.flip) {
      middleware.push(
        flip({
          boundary: this.flipBoundary || defaultBoundary,
          // @ts-expect-error - We're converting a string attribute to an array here
          fallbackPlacements: this.flipFallbackPlacements,
          fallbackStrategy:
            this.flipFallbackStrategy === "best-fit" ? "bestFit" : "initialPlacement",
          padding: this.flipPadding,
        }),
      );
    }

    // Then we shift
    if (this.shift) {
      middleware.push(
        shift({
          boundary: this.shiftBoundary || defaultBoundary,
          padding: this.shiftPadding,
        }),
      );
    }

    // Now we adjust the size as needed
    if (this.autoSize) {
      middleware.push(
        size({
          boundary: this.autoSizeBoundary || defaultBoundary,
          padding: this.autoSizePadding,
          apply: ({ availableWidth, availableHeight }) => {
            if (this.autoSize === "vertical" || this.autoSize === "both") {
              this.style.setProperty("--auto-size-available-height", `${availableHeight}px`);
            } else {
              this.style.removeProperty("--auto-size-available-height");
            }

            if (this.autoSize === "horizontal" || this.autoSize === "both") {
              this.style.setProperty("--auto-size-available-width", `${availableWidth}px`);
            } else {
              this.style.removeProperty("--auto-size-available-width");
            }
          },
        }),
      );
    } else {
      // Cleanup styles if we're no longer using auto-size
      this.style.removeProperty("--auto-size-available-width");
      this.style.removeProperty("--auto-size-available-height");
    }

    // Finally, we add an arrow
    if (this.arrow) {
      middleware.push(
        arrow({
          element: this.arrowEl,
          padding: this.arrowPadding,
        }),
      );
    }

    //
    // Use custom positioning logic if the strategy is absolute. Otherwise, fall back to the default logic.
    //
    // More info: https://github.com/shoelace-style/shoelace/issues/1135
    //
    const getOffsetParent = SUPPORTS_POPOVER
      ? (element: Element) => platform.getOffsetParent(element, offsetParent)
      : platform.getOffsetParent;

    computePosition(this.anchorEl, this.popup, {
      placement: this.placement,
      middleware,
      strategy: SUPPORTS_POPOVER ? "absolute" : "fixed",
      platform: {
        ...platform,
        getOffsetParent,
      },
    }).then(({ x, y, middlewareData, placement }) => {
      //
      // Even though we have our own localization utility, it uses different heuristics to determine RTL. Because of
      // that, we'll use the same approach that Floating UI uses.
      //
      // Source: https://github.com/floating-ui/floating-ui/blob/cb3b6ab07f95275730d3e6e46c702f8d4908b55c/packages/dom/src/utils/getDocumentRect.ts#L31
      //
      const isRtl = this.localize.dir() === "rtl";
      const staticSide = { top: "bottom", right: "left", bottom: "top", left: "right" }[
        placement.split("-")[0]
      ]!;

      this.setAttribute("data-current-placement", placement);

      Object.assign(this.popup.style, {
        left: `${x}px`,
        top: `${y}px`,
      });

      if (this.arrow) {
        const arrowX = middlewareData.arrow!.x;
        const arrowY = middlewareData.arrow!.y;
        let top = "";
        let right = "";
        let bottom = "";
        let left = "";

        if (this.arrowPlacement === "start") {
          // Start
          const value =
            typeof arrowX === "number"
              ? `calc(${this.arrowPadding}px - var(--arrow-padding-offset))`
              : "";
          top =
            typeof arrowY === "number"
              ? `calc(${this.arrowPadding}px - var(--arrow-padding-offset))`
              : "";
          right = isRtl ? value : "";
          left = isRtl ? "" : value;
        } else if (this.arrowPlacement === "end") {
          // End
          const value =
            typeof arrowX === "number"
              ? `calc(${this.arrowPadding}px - var(--arrow-padding-offset))`
              : "";
          right = isRtl ? "" : value;
          left = isRtl ? value : "";
          bottom =
            typeof arrowY === "number"
              ? `calc(${this.arrowPadding}px - var(--arrow-padding-offset))`
              : "";
        } else if (this.arrowPlacement === "center") {
          // Center
          left = typeof arrowX === "number" ? `calc(50% - var(--arrow-size-diagonal))` : "";
          top = typeof arrowY === "number" ? `calc(50% - var(--arrow-size-diagonal))` : "";
        } else {
          // Anchor (default)
          left = typeof arrowX === "number" ? `${arrowX}px` : "";
          top = typeof arrowY === "number" ? `${arrowY}px` : "";
        }

        Object.assign(this.arrowEl.style, {
          top,
          right,
          bottom,
          left,
          [staticSide]: "calc(var(--arrow-base-offset) - var(--arrow-size-diagonal))",
        });
      }
    });

    // Wait until the new position is drawn before updating the hover bridge, otherwise it can get out of sync
    requestAnimationFrame(() => this.updateHoverBridge());

    this.dispatchEvent(new WaRepositionEvent());
  }

  private updateHoverBridge = () => {
    if (this.hoverBridge && this.anchorEl && this.popup) {
      const anchorRect = this.anchorEl.getBoundingClientRect();
      const popupRect = this.popup.getBoundingClientRect();
      const isVertical = this.placement.includes("top") || this.placement.includes("bottom");
      let topLeftX = 0;
      let topLeftY = 0;
      let topRightX = 0;
      let topRightY = 0;
      let bottomLeftX = 0;
      let bottomLeftY = 0;
      let bottomRightX = 0;
      let bottomRightY = 0;

      if (isVertical) {
        if (anchorRect.top < popupRect.top) {
          // Anchor is above
          topLeftX = anchorRect.left;
          topLeftY = anchorRect.bottom;
          topRightX = anchorRect.right;
          topRightY = anchorRect.bottom;

          bottomLeftX = popupRect.left;
          bottomLeftY = popupRect.top;
          bottomRightX = popupRect.right;
          bottomRightY = popupRect.top;
        } else {
          // Anchor is below
          topLeftX = popupRect.left;
          topLeftY = popupRect.bottom;
          topRightX = popupRect.right;
          topRightY = popupRect.bottom;

          bottomLeftX = anchorRect.left;
          bottomLeftY = anchorRect.top;
          bottomRightX = anchorRect.right;
          bottomRightY = anchorRect.top;
        }
      } else {
        if (anchorRect.left < popupRect.left) {
          // Anchor is on the left
          topLeftX = anchorRect.right;
          topLeftY = anchorRect.top;
          topRightX = popupRect.left;
          topRightY = popupRect.top;

          bottomLeftX = anchorRect.right;
          bottomLeftY = anchorRect.bottom;
          bottomRightX = popupRect.left;
          bottomRightY = popupRect.bottom;
        } else {
          // Anchor is on the right
          topLeftX = popupRect.right;
          topLeftY = popupRect.top;
          topRightX = anchorRect.left;
          topRightY = anchorRect.top;

          bottomLeftX = popupRect.right;
          bottomLeftY = popupRect.bottom;
          bottomRightX = anchorRect.left;
          bottomRightY = anchorRect.bottom;
        }
      }

      this.style.setProperty("--hover-bridge-top-left-x", `${topLeftX}px`);
      this.style.setProperty("--hover-bridge-top-left-y", `${topLeftY}px`);
      this.style.setProperty("--hover-bridge-top-right-x", `${topRightX}px`);
      this.style.setProperty("--hover-bridge-top-right-y", `${topRightY}px`);
      this.style.setProperty("--hover-bridge-bottom-left-x", `${bottomLeftX}px`);
      this.style.setProperty("--hover-bridge-bottom-left-y", `${bottomLeftY}px`);
      this.style.setProperty("--hover-bridge-bottom-right-x", `${bottomRightX}px`);
      this.style.setProperty("--hover-bridge-bottom-right-y", `${bottomRightY}px`);
    }
  };

  render() {
    return html`
      <slot name="anchor" @slotchange=${this.handleAnchorChange}></slot>

      <span
        part="hover-bridge"
        class=${classMap({
          "popup-hover-bridge": true,
          "popup-hover-bridge-visible": this.hoverBridge && this.active,
        })}
      ></span>

      <div
        popover="manual"
        part="popup"
        class=${classMap({
          popup: true,
          "popup-active": this.active,
          "popup-fixed": !SUPPORTS_POPOVER,
          "popup-has-arrow": this.arrow,
        })}
      >
        <slot></slot>
        ${
          this.arrow
            ? html`
                <div part="arrow" class="arrow" role="presentation"></div>
              `
            : ""
        }
      </div>
    `;
  }
}

declare global {
  interface HTMLElementTagNameMap {
    "wa-popup": WaPopup;
  }
}

`````
