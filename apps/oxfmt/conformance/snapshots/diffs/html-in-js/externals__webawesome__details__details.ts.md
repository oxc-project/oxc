# externals/webawesome/details/details.ts

## Option 1

`````json
{"printWidth":80}
`````

### Diff

`````diff
===================================================================
--- prettier
+++ oxfmt
@@ -79,13 +79,17 @@
   @property({ type: Boolean, reflect: true }) disabled = false;
 
   /** The element's visual appearance. */
   @property({ reflect: true }) appearance:
-    "filled" | "outlined" | "filled-outlined" | "plain" = "outlined";
+    | "filled"
+    | "outlined"
+    | "filled-outlined"
+    | "plain" = "outlined";
 
   /** The location of the expand/collapse icon. */
   @property({ attribute: "icon-placement", reflect: true }) iconPlacement:
-    "start" | "end" = "end";
+    | "start"
+    | "end" = "end";
 
   disconnectedCallback() {
     super.disconnectedCallback();
     this.detailsObserver?.disconnect();

`````

### Actual (oxfmt)

`````ts
import type { PropertyValues } from "lit";
import { html } from "lit";
import { customElement, property, query, state } from "lit/decorators.js";
import { classMap } from "lit/directives/class-map.js";
import { WaAfterHideEvent } from "../../events/after-hide.js";
import { WaAfterShowEvent } from "../../events/after-show.js";
import { WaHideEvent } from "../../events/hide.js";
import { WaShowEvent } from "../../events/show.js";
import { animate, parseDuration } from "../../internal/animate.js";
import { waitForEvent } from "../../internal/event.js";
import { watch } from "../../internal/watch.js";
import WebAwesomeElement from "../../internal/webawesome-element.js";
import { WebAwesomeFormAssociatedElement } from "../../internal/webawesome-form-associated-element.js";
import { LocalizeController } from "../../utilities/localize.js";
import "../icon/icon.js";
import styles from "./details.styles.js";

/**
 * @summary Details display a brief summary and expand to reveal additional content. Use them to progressively disclose
 *  information, group related FAQs, or hide advanced options.
 * @documentation https://webawesome.com/docs/components/details
 * @status stable
 * @since 2.0
 *
 * @dependency wa-icon
 *
 * @slot - The details' main content.
 * @slot summary - The details' summary. Alternatively, you can use the `summary` attribute.
 * @slot expand-icon - Optional expand icon to use instead of the default. Works best with `<wa-icon>`.
 * @slot collapse-icon - Optional collapse icon to use instead of the default. Works best with `<wa-icon>`.
 *
 * @event wa-show - Emitted when the details opens.
 * @event wa-after-show - Emitted after the details opens and all animations are complete.
 * @event wa-hide - Emitted when the details closes.
 * @event wa-after-hide - Emitted after the details closes and all animations are complete.
 *
 * @csspart base - The inner `<details>` element used to render the component.
 *                 Styles you apply to the component are automatically applied to this part, so you usually don't need to deal with it unless you need to set the `display` property.
 * @csspart header - The header that wraps both the summary and the expand/collapse icon.
 * @csspart summary - The container that wraps the summary.
 * @csspart icon - The container that wraps the expand/collapse icons.
 * @csspart content - The details content.
 *
 * @cssproperty --spacing - The amount of space around and between the details' content. Expects a single value.
 * @cssproperty [--show-duration=200ms] - The show duration to use when applying built-in animation classes.
 * @cssproperty [--hide-duration=200ms] - The hide duration to use when applying built-in animation classes.
 *
 * @cssstate animating - Applied when the details is animating expand/collapse.
 */
@customElement("wa-details")
export default class WaDetails extends WebAwesomeElement {
  static css = styles;

  private detailsObserver: MutationObserver;
  private readonly localize = new LocalizeController(this);

  @query("details") details: HTMLDetailsElement;
  @query("summary") header: HTMLElement;
  @query(".body") body: HTMLElement;
  @query(".expand-icon-slot") expandIconSlot: HTMLSlotElement;

  private animationGeneration = 0;

  @state() isAnimating = false;

  /**
   * Indicates whether or not the details is open. You can toggle this attribute to show and hide the details, or you
   * can use the `show()` and `hide()` methods and this attribute will reflect the details' open state.
   */
  @property({ type: Boolean, reflect: true }) open = false;

  /** The summary to show in the header. If you need to display HTML, use the `summary` slot instead. */
  @property() summary: string;

  /** Groups related details elements. When one opens, others with the same name will close. */
  @property({ reflect: true }) name: string;

  /** Disables the details so it can't be toggled. */
  @property({ type: Boolean, reflect: true }) disabled = false;

  /** The element's visual appearance. */
  @property({ reflect: true }) appearance:
    | "filled"
    | "outlined"
    | "filled-outlined"
    | "plain" = "outlined";

  /** The location of the expand/collapse icon. */
  @property({ attribute: "icon-placement", reflect: true }) iconPlacement:
    | "start"
    | "end" = "end";

  disconnectedCallback() {
    super.disconnectedCallback();
    this.detailsObserver?.disconnect();
  }

  firstUpdated() {
    this.body.style.height = this.open ? "auto" : "0";
    if (this.open) {
      this.details.open = true;
    }

    this.detailsObserver = new MutationObserver((changes) => {
      for (const change of changes) {
        if (change.type === "attributes" && change.attributeName === "open") {
          if (this.details.open) {
            this.show();
          } else {
            this.hide();
          }
        }
      }
    });
    this.detailsObserver.observe(this.details, { attributes: true });
  }

  updated(changedProperties: PropertyValues<this>) {
    if (changedProperties.has("isAnimating")) {
      this.customStates.set("animating", this.isAnimating);
    }
  }

  private handleSummaryClick(event: MouseEvent) {
    const eventPath = event.composedPath() as HTMLElement[];

    // Check if any element in the path is interactive
    const hasInteractiveElement = eventPath.some((element) => {
      if (!(element instanceof HTMLElement)) return false;

      // Check native interactive elements
      const tagName = element.tagName?.toLowerCase();
      if (["a", "button", "input", "textarea", "select"].includes(tagName)) {
        return true;
      }

      // Check for Web Awesome form controls
      if (element instanceof WebAwesomeFormAssociatedElement) {
        return !("disabled" in element) || !element.disabled;
      }

      return false;
    });

    if (hasInteractiveElement) {
      // Let interactive elements handle their own clicks, fixes #309
      return;
    }

    event.preventDefault();

    if (!this.disabled) {
      if (this.open) {
        this.hide();
      } else {
        this.show();
      }
      this.header.focus();
    }
  }

  private handleSummaryKeyDown(event: KeyboardEvent) {
    if (event.key === "Enter" || event.key === " ") {
      event.preventDefault();

      if (this.open) {
        this.hide();
      } else {
        this.show();
      }
    }

    if (event.key === "ArrowUp" || event.key === "ArrowLeft") {
      event.preventDefault();
      this.hide();
    }

    if (event.key === "ArrowDown" || event.key === "ArrowRight") {
      event.preventDefault();
      this.show();
    }
  }

  /** Closes other <wa-details> elements in the same document when they have the same name. */
  private closeOthersWithSameName() {
    if (!this.name) return;

    const root = this.getRootNode() as Document | ShadowRoot;
    const otherDetails = root.querySelectorAll(
      `wa-details[name="${this.name}"]`,
    ) as NodeListOf<WaDetails>;

    otherDetails.forEach((detail) => {
      if (detail !== this && detail.open) {
        detail.open = false;
      }
    });
  }

  @watch("open", { waitUntilFirstUpdate: true })
  async handleOpenChange() {
    this.animationGeneration++;
    const generation = this.animationGeneration;

    if (this.open) {
      this.details.open = true;
      // Show
      const waShow = new WaShowEvent();
      this.dispatchEvent(waShow);
      if (waShow.defaultPrevented) {
        this.open = false;
        this.details.open = false;
        return;
      }

      // Close other details with the same name
      this.closeOthersWithSameName();

      this.isAnimating = true;
      const duration = parseDuration(
        getComputedStyle(this.body).getPropertyValue("--show-duration"),
      );
      // We can't animate to 'auto', so use the scroll height for now
      await animate(
        this.body,
        [
          { height: "0", opacity: "0" },
          { height: `${this.body.scrollHeight}px`, opacity: "1" },
        ],
        {
          duration,
          easing: "linear",
        },
      );

      // If a newer animation has started, handle the final state
      if (this.animationGeneration !== generation) {
        return;
      }

      this.body.style.height = "auto";
      this.isAnimating = false;

      this.dispatchEvent(new WaAfterShowEvent());
    } else {
      // Hide
      const waHide = new WaHideEvent();
      this.dispatchEvent(waHide);
      if (waHide.defaultPrevented) {
        this.details.open = true;
        this.open = true;
        return;
      }

      this.isAnimating = true;
      const duration = parseDuration(
        getComputedStyle(this.body).getPropertyValue("--hide-duration"),
      );
      // We can't animate from 'auto', so use the scroll height for now
      await animate(
        this.body,
        [
          { height: `${this.body.scrollHeight}px`, opacity: "1" },
          { height: "0", opacity: "0" },
        ],
        { duration, easing: "linear" },
      );

      // If a newer animation has started, handle the final state
      if (this.animationGeneration !== generation) {
        return;
      }

      this.body.style.height = "0";
      this.isAnimating = false;
      this.details.open = false;
      this.dispatchEvent(new WaAfterHideEvent());
    }
  }

  /** Shows the details. */
  async show() {
    if (this.open || this.disabled) {
      return undefined;
    }

    this.open = true;
    return waitForEvent(this, "wa-after-show");
  }

  /** Hides the details */
  async hide() {
    if (!this.open || this.disabled) {
      return undefined;
    }

    this.open = false;
    return waitForEvent(this, "wa-after-hide");
  }

  render() {
    const isRtl = !this.hasUpdated
      ? this.dir === "rtl"
      : this.localize.dir() === "rtl";

    return html`
      <details part="base">
        <summary
          part="header"
          role="button"
          aria-expanded=${this.open ? "true" : "false"}
          aria-controls="content"
          aria-disabled=${this.disabled ? "true" : "false"}
          tabindex=${this.disabled ? "-1" : "0"}
          @click=${this.handleSummaryClick}
          @keydown=${this.handleSummaryKeyDown}
        >
          <slot name="summary" part="summary">${this.summary}</slot>

          <span part="icon">
            <slot name="expand-icon">
              <wa-icon
                library="system"
                variant="solid"
                name=${isRtl ? "chevron-left" : "chevron-right"}
              ></wa-icon>
            </slot>
            <slot name="collapse-icon">
              <wa-icon
                library="system"
                variant="solid"
                name=${isRtl ? "chevron-left" : "chevron-right"}
              ></wa-icon>
            </slot>
          </span>
        </summary>

        <div
          class=${classMap({
            body: true,
            animating: this.isAnimating,
          })}
          role="region"
          aria-labelledby="header"
        >
          <slot part="content" id="content" class="content"></slot>
        </div>
      </details>
    `;
  }
}

declare global {
  interface HTMLElementTagNameMap {
    "wa-details": WaDetails;
  }
}

`````

### Expected (prettier)

`````ts
import type { PropertyValues } from "lit";
import { html } from "lit";
import { customElement, property, query, state } from "lit/decorators.js";
import { classMap } from "lit/directives/class-map.js";
import { WaAfterHideEvent } from "../../events/after-hide.js";
import { WaAfterShowEvent } from "../../events/after-show.js";
import { WaHideEvent } from "../../events/hide.js";
import { WaShowEvent } from "../../events/show.js";
import { animate, parseDuration } from "../../internal/animate.js";
import { waitForEvent } from "../../internal/event.js";
import { watch } from "../../internal/watch.js";
import WebAwesomeElement from "../../internal/webawesome-element.js";
import { WebAwesomeFormAssociatedElement } from "../../internal/webawesome-form-associated-element.js";
import { LocalizeController } from "../../utilities/localize.js";
import "../icon/icon.js";
import styles from "./details.styles.js";

/**
 * @summary Details display a brief summary and expand to reveal additional content. Use them to progressively disclose
 *  information, group related FAQs, or hide advanced options.
 * @documentation https://webawesome.com/docs/components/details
 * @status stable
 * @since 2.0
 *
 * @dependency wa-icon
 *
 * @slot - The details' main content.
 * @slot summary - The details' summary. Alternatively, you can use the `summary` attribute.
 * @slot expand-icon - Optional expand icon to use instead of the default. Works best with `<wa-icon>`.
 * @slot collapse-icon - Optional collapse icon to use instead of the default. Works best with `<wa-icon>`.
 *
 * @event wa-show - Emitted when the details opens.
 * @event wa-after-show - Emitted after the details opens and all animations are complete.
 * @event wa-hide - Emitted when the details closes.
 * @event wa-after-hide - Emitted after the details closes and all animations are complete.
 *
 * @csspart base - The inner `<details>` element used to render the component.
 *                 Styles you apply to the component are automatically applied to this part, so you usually don't need to deal with it unless you need to set the `display` property.
 * @csspart header - The header that wraps both the summary and the expand/collapse icon.
 * @csspart summary - The container that wraps the summary.
 * @csspart icon - The container that wraps the expand/collapse icons.
 * @csspart content - The details content.
 *
 * @cssproperty --spacing - The amount of space around and between the details' content. Expects a single value.
 * @cssproperty [--show-duration=200ms] - The show duration to use when applying built-in animation classes.
 * @cssproperty [--hide-duration=200ms] - The hide duration to use when applying built-in animation classes.
 *
 * @cssstate animating - Applied when the details is animating expand/collapse.
 */
@customElement("wa-details")
export default class WaDetails extends WebAwesomeElement {
  static css = styles;

  private detailsObserver: MutationObserver;
  private readonly localize = new LocalizeController(this);

  @query("details") details: HTMLDetailsElement;
  @query("summary") header: HTMLElement;
  @query(".body") body: HTMLElement;
  @query(".expand-icon-slot") expandIconSlot: HTMLSlotElement;

  private animationGeneration = 0;

  @state() isAnimating = false;

  /**
   * Indicates whether or not the details is open. You can toggle this attribute to show and hide the details, or you
   * can use the `show()` and `hide()` methods and this attribute will reflect the details' open state.
   */
  @property({ type: Boolean, reflect: true }) open = false;

  /** The summary to show in the header. If you need to display HTML, use the `summary` slot instead. */
  @property() summary: string;

  /** Groups related details elements. When one opens, others with the same name will close. */
  @property({ reflect: true }) name: string;

  /** Disables the details so it can't be toggled. */
  @property({ type: Boolean, reflect: true }) disabled = false;

  /** The element's visual appearance. */
  @property({ reflect: true }) appearance:
    "filled" | "outlined" | "filled-outlined" | "plain" = "outlined";

  /** The location of the expand/collapse icon. */
  @property({ attribute: "icon-placement", reflect: true }) iconPlacement:
    "start" | "end" = "end";

  disconnectedCallback() {
    super.disconnectedCallback();
    this.detailsObserver?.disconnect();
  }

  firstUpdated() {
    this.body.style.height = this.open ? "auto" : "0";
    if (this.open) {
      this.details.open = true;
    }

    this.detailsObserver = new MutationObserver((changes) => {
      for (const change of changes) {
        if (change.type === "attributes" && change.attributeName === "open") {
          if (this.details.open) {
            this.show();
          } else {
            this.hide();
          }
        }
      }
    });
    this.detailsObserver.observe(this.details, { attributes: true });
  }

  updated(changedProperties: PropertyValues<this>) {
    if (changedProperties.has("isAnimating")) {
      this.customStates.set("animating", this.isAnimating);
    }
  }

  private handleSummaryClick(event: MouseEvent) {
    const eventPath = event.composedPath() as HTMLElement[];

    // Check if any element in the path is interactive
    const hasInteractiveElement = eventPath.some((element) => {
      if (!(element instanceof HTMLElement)) return false;

      // Check native interactive elements
      const tagName = element.tagName?.toLowerCase();
      if (["a", "button", "input", "textarea", "select"].includes(tagName)) {
        return true;
      }

      // Check for Web Awesome form controls
      if (element instanceof WebAwesomeFormAssociatedElement) {
        return !("disabled" in element) || !element.disabled;
      }

      return false;
    });

    if (hasInteractiveElement) {
      // Let interactive elements handle their own clicks, fixes #309
      return;
    }

    event.preventDefault();

    if (!this.disabled) {
      if (this.open) {
        this.hide();
      } else {
        this.show();
      }
      this.header.focus();
    }
  }

  private handleSummaryKeyDown(event: KeyboardEvent) {
    if (event.key === "Enter" || event.key === " ") {
      event.preventDefault();

      if (this.open) {
        this.hide();
      } else {
        this.show();
      }
    }

    if (event.key === "ArrowUp" || event.key === "ArrowLeft") {
      event.preventDefault();
      this.hide();
    }

    if (event.key === "ArrowDown" || event.key === "ArrowRight") {
      event.preventDefault();
      this.show();
    }
  }

  /** Closes other <wa-details> elements in the same document when they have the same name. */
  private closeOthersWithSameName() {
    if (!this.name) return;

    const root = this.getRootNode() as Document | ShadowRoot;
    const otherDetails = root.querySelectorAll(
      `wa-details[name="${this.name}"]`,
    ) as NodeListOf<WaDetails>;

    otherDetails.forEach((detail) => {
      if (detail !== this && detail.open) {
        detail.open = false;
      }
    });
  }

  @watch("open", { waitUntilFirstUpdate: true })
  async handleOpenChange() {
    this.animationGeneration++;
    const generation = this.animationGeneration;

    if (this.open) {
      this.details.open = true;
      // Show
      const waShow = new WaShowEvent();
      this.dispatchEvent(waShow);
      if (waShow.defaultPrevented) {
        this.open = false;
        this.details.open = false;
        return;
      }

      // Close other details with the same name
      this.closeOthersWithSameName();

      this.isAnimating = true;
      const duration = parseDuration(
        getComputedStyle(this.body).getPropertyValue("--show-duration"),
      );
      // We can't animate to 'auto', so use the scroll height for now
      await animate(
        this.body,
        [
          { height: "0", opacity: "0" },
          { height: `${this.body.scrollHeight}px`, opacity: "1" },
        ],
        {
          duration,
          easing: "linear",
        },
      );

      // If a newer animation has started, handle the final state
      if (this.animationGeneration !== generation) {
        return;
      }

      this.body.style.height = "auto";
      this.isAnimating = false;

      this.dispatchEvent(new WaAfterShowEvent());
    } else {
      // Hide
      const waHide = new WaHideEvent();
      this.dispatchEvent(waHide);
      if (waHide.defaultPrevented) {
        this.details.open = true;
        this.open = true;
        return;
      }

      this.isAnimating = true;
      const duration = parseDuration(
        getComputedStyle(this.body).getPropertyValue("--hide-duration"),
      );
      // We can't animate from 'auto', so use the scroll height for now
      await animate(
        this.body,
        [
          { height: `${this.body.scrollHeight}px`, opacity: "1" },
          { height: "0", opacity: "0" },
        ],
        { duration, easing: "linear" },
      );

      // If a newer animation has started, handle the final state
      if (this.animationGeneration !== generation) {
        return;
      }

      this.body.style.height = "0";
      this.isAnimating = false;
      this.details.open = false;
      this.dispatchEvent(new WaAfterHideEvent());
    }
  }

  /** Shows the details. */
  async show() {
    if (this.open || this.disabled) {
      return undefined;
    }

    this.open = true;
    return waitForEvent(this, "wa-after-show");
  }

  /** Hides the details */
  async hide() {
    if (!this.open || this.disabled) {
      return undefined;
    }

    this.open = false;
    return waitForEvent(this, "wa-after-hide");
  }

  render() {
    const isRtl = !this.hasUpdated
      ? this.dir === "rtl"
      : this.localize.dir() === "rtl";

    return html`
      <details part="base">
        <summary
          part="header"
          role="button"
          aria-expanded=${this.open ? "true" : "false"}
          aria-controls="content"
          aria-disabled=${this.disabled ? "true" : "false"}
          tabindex=${this.disabled ? "-1" : "0"}
          @click=${this.handleSummaryClick}
          @keydown=${this.handleSummaryKeyDown}
        >
          <slot name="summary" part="summary">${this.summary}</slot>

          <span part="icon">
            <slot name="expand-icon">
              <wa-icon
                library="system"
                variant="solid"
                name=${isRtl ? "chevron-left" : "chevron-right"}
              ></wa-icon>
            </slot>
            <slot name="collapse-icon">
              <wa-icon
                library="system"
                variant="solid"
                name=${isRtl ? "chevron-left" : "chevron-right"}
              ></wa-icon>
            </slot>
          </span>
        </summary>

        <div
          class=${classMap({
            body: true,
            animating: this.isAnimating,
          })}
          role="region"
          aria-labelledby="header"
        >
          <slot part="content" id="content" class="content"></slot>
        </div>
      </details>
    `;
  }
}

declare global {
  interface HTMLElementTagNameMap {
    "wa-details": WaDetails;
  }
}

`````
