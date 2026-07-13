# externals/webawesome/breadcrumb-item/breadcrumb-item.ts

## Option 1

`````json
{"printWidth":80}
`````

### Diff

`````diff
===================================================================
--- prettier
+++ oxfmt
@@ -76,41 +76,35 @@
       <span part="start" class="start">
         <slot name="start"></slot>
       </span>
 
-      ${
-        this.renderType === "link"
-          ? html`
-              <a
-                part="label"
-                class="label label-link"
-                href="${this.href!}"
-                target="${ifDefined(this.target ? this.target : undefined)}"
-                rel=${ifDefined(this.target ? this.rel : undefined)}
-              >
-                <slot></slot>
-              </a>
-            `
-          : ""
-      }
-      ${
-        this.renderType === "button"
-          ? html`
-              <button part="label" type="button" class="label label-button">
-                <slot @slotchange=${this.handleSlotChange}></slot>
-              </button>
-            `
-          : ""
-      }
-      ${
-        this.renderType === "dropdown"
-          ? html`
-              <div part="label" class="label label-dropdown">
-                <slot @slotchange=${this.handleSlotChange}></slot>
-              </div>
-            `
-          : ""
-      }
+      ${this.renderType === "link"
+        ? html`
+            <a
+              part="label"
+              class="label label-link"
+              href="${this.href!}"
+              target="${ifDefined(this.target ? this.target : undefined)}"
+              rel=${ifDefined(this.target ? this.rel : undefined)}
+            >
+              <slot></slot>
+            </a>
+          `
+        : ""}
+      ${this.renderType === "button"
+        ? html`
+            <button part="label" type="button" class="label label-button">
+              <slot @slotchange=${this.handleSlotChange}></slot>
+            </button>
+          `
+        : ""}
+      ${this.renderType === "dropdown"
+        ? html`
+            <div part="label" class="label label-dropdown">
+              <slot @slotchange=${this.handleSlotChange}></slot>
+            </div>
+          `
+        : ""}
 
       <span part="end" class="end">
         <slot name="end"></slot>
       </span>

`````

### Actual (oxfmt)

`````ts
import { html } from "lit";
import { customElement, property, query, state } from "lit/decorators.js";
import { ifDefined } from "lit/directives/if-defined.js";
import { watch } from "../../internal/watch.js";
import WebAwesomeElement from "../../internal/webawesome-element.js";
import styles from "./breadcrumb-item.styles.js";

/**
 * @summary Breadcrumb items represent individual links inside a breadcrumb, typically one per level of the site
 *  hierarchy.
 * @documentation https://webawesome.com/docs/components/breadcrumb-item
 * @status stable
 * @since 2.0
 *
 * @slot - The breadcrumb item's label.
 * @slot start - An element, such as `<wa-icon>`, placed before the label.
 * @slot end - An element, such as `<wa-icon>`, placed after the label.
 * @slot separator - The separator to use for the breadcrumb item. This will only change the separator for this item. If
 * you want to change it for all items in the group, set the separator on `<wa-breadcrumb>` instead.
 *
 * @csspart label - The breadcrumb item's label.
 * @csspart start - The container that wraps the `start` slot.
 * @csspart end - The container that wraps the `end` slot.
 * @csspart separator - The container that wraps the separator.
 */
@customElement("wa-breadcrumb-item")
export default class WaBreadcrumbItem extends WebAwesomeElement {
  static css = styles;

  @query("slot:not([name])") defaultSlot: HTMLSlotElement;

  @state() private renderType: "button" | "link" | "dropdown" = "button";

  /**
   * Optional URL to direct the user to when the breadcrumb item is activated. When set, a link will be rendered
   * internally. When unset, a button will be rendered instead.
   */
  @property() href?: string;

  /** Tells the browser where to open the link. Only used when `href` is set. */
  @property() target?: "_blank" | "_parent" | "_self" | "_top";

  /** The `rel` attribute to use on the link. Only used when `href` is set. */
  @property() rel = "noreferrer noopener";

  private setRenderType() {
    const hasDropdown =
      this.defaultSlot
        .assignedElements({ flatten: true })
        .filter((i) => i.tagName.toLowerCase() === "wa-dropdown").length > 0;

    if (this.href) {
      this.renderType = "link";
      return;
    }

    if (hasDropdown) {
      this.renderType = "dropdown";
      return;
    }

    this.renderType = "button";
  }

  @watch("href", { waitUntilFirstUpdate: true })
  hrefChanged() {
    this.setRenderType();
  }

  handleSlotChange() {
    this.setRenderType();
  }

  render() {
    return html`
      <span part="start" class="start">
        <slot name="start"></slot>
      </span>

      ${this.renderType === "link"
        ? html`
            <a
              part="label"
              class="label label-link"
              href="${this.href!}"
              target="${ifDefined(this.target ? this.target : undefined)}"
              rel=${ifDefined(this.target ? this.rel : undefined)}
            >
              <slot></slot>
            </a>
          `
        : ""}
      ${this.renderType === "button"
        ? html`
            <button part="label" type="button" class="label label-button">
              <slot @slotchange=${this.handleSlotChange}></slot>
            </button>
          `
        : ""}
      ${this.renderType === "dropdown"
        ? html`
            <div part="label" class="label label-dropdown">
              <slot @slotchange=${this.handleSlotChange}></slot>
            </div>
          `
        : ""}

      <span part="end" class="end">
        <slot name="end"></slot>
      </span>

      <span part="separator" class="separator" aria-hidden="true">
        <slot name="separator"></slot>
      </span>
    `;
  }
}

declare global {
  interface HTMLElementTagNameMap {
    "wa-breadcrumb-item": WaBreadcrumbItem;
  }
}

`````

### Expected (prettier)

`````ts
import { html } from "lit";
import { customElement, property, query, state } from "lit/decorators.js";
import { ifDefined } from "lit/directives/if-defined.js";
import { watch } from "../../internal/watch.js";
import WebAwesomeElement from "../../internal/webawesome-element.js";
import styles from "./breadcrumb-item.styles.js";

/**
 * @summary Breadcrumb items represent individual links inside a breadcrumb, typically one per level of the site
 *  hierarchy.
 * @documentation https://webawesome.com/docs/components/breadcrumb-item
 * @status stable
 * @since 2.0
 *
 * @slot - The breadcrumb item's label.
 * @slot start - An element, such as `<wa-icon>`, placed before the label.
 * @slot end - An element, such as `<wa-icon>`, placed after the label.
 * @slot separator - The separator to use for the breadcrumb item. This will only change the separator for this item. If
 * you want to change it for all items in the group, set the separator on `<wa-breadcrumb>` instead.
 *
 * @csspart label - The breadcrumb item's label.
 * @csspart start - The container that wraps the `start` slot.
 * @csspart end - The container that wraps the `end` slot.
 * @csspart separator - The container that wraps the separator.
 */
@customElement("wa-breadcrumb-item")
export default class WaBreadcrumbItem extends WebAwesomeElement {
  static css = styles;

  @query("slot:not([name])") defaultSlot: HTMLSlotElement;

  @state() private renderType: "button" | "link" | "dropdown" = "button";

  /**
   * Optional URL to direct the user to when the breadcrumb item is activated. When set, a link will be rendered
   * internally. When unset, a button will be rendered instead.
   */
  @property() href?: string;

  /** Tells the browser where to open the link. Only used when `href` is set. */
  @property() target?: "_blank" | "_parent" | "_self" | "_top";

  /** The `rel` attribute to use on the link. Only used when `href` is set. */
  @property() rel = "noreferrer noopener";

  private setRenderType() {
    const hasDropdown =
      this.defaultSlot
        .assignedElements({ flatten: true })
        .filter((i) => i.tagName.toLowerCase() === "wa-dropdown").length > 0;

    if (this.href) {
      this.renderType = "link";
      return;
    }

    if (hasDropdown) {
      this.renderType = "dropdown";
      return;
    }

    this.renderType = "button";
  }

  @watch("href", { waitUntilFirstUpdate: true })
  hrefChanged() {
    this.setRenderType();
  }

  handleSlotChange() {
    this.setRenderType();
  }

  render() {
    return html`
      <span part="start" class="start">
        <slot name="start"></slot>
      </span>

      ${
        this.renderType === "link"
          ? html`
              <a
                part="label"
                class="label label-link"
                href="${this.href!}"
                target="${ifDefined(this.target ? this.target : undefined)}"
                rel=${ifDefined(this.target ? this.rel : undefined)}
              >
                <slot></slot>
              </a>
            `
          : ""
      }
      ${
        this.renderType === "button"
          ? html`
              <button part="label" type="button" class="label label-button">
                <slot @slotchange=${this.handleSlotChange}></slot>
              </button>
            `
          : ""
      }
      ${
        this.renderType === "dropdown"
          ? html`
              <div part="label" class="label label-dropdown">
                <slot @slotchange=${this.handleSlotChange}></slot>
              </div>
            `
          : ""
      }

      <span part="end" class="end">
        <slot name="end"></slot>
      </span>

      <span part="separator" class="separator" aria-hidden="true">
        <slot name="separator"></slot>
      </span>
    `;
  }
}

declare global {
  interface HTMLElementTagNameMap {
    "wa-breadcrumb-item": WaBreadcrumbItem;
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
@@ -76,41 +76,35 @@
       <span part="start" class="start">
         <slot name="start"></slot>
       </span>
 
-      ${
-        this.renderType === "link"
-          ? html`
-              <a
-                part="label"
-                class="label label-link"
-                href="${this.href!}"
-                target="${ifDefined(this.target ? this.target : undefined)}"
-                rel=${ifDefined(this.target ? this.rel : undefined)}
-              >
-                <slot></slot>
-              </a>
-            `
-          : ""
-      }
-      ${
-        this.renderType === "button"
-          ? html`
-              <button part="label" type="button" class="label label-button">
-                <slot @slotchange=${this.handleSlotChange}></slot>
-              </button>
-            `
-          : ""
-      }
-      ${
-        this.renderType === "dropdown"
-          ? html`
-              <div part="label" class="label label-dropdown">
-                <slot @slotchange=${this.handleSlotChange}></slot>
-              </div>
-            `
-          : ""
-      }
+      ${this.renderType === "link"
+        ? html`
+            <a
+              part="label"
+              class="label label-link"
+              href="${this.href!}"
+              target="${ifDefined(this.target ? this.target : undefined)}"
+              rel=${ifDefined(this.target ? this.rel : undefined)}
+            >
+              <slot></slot>
+            </a>
+          `
+        : ""}
+      ${this.renderType === "button"
+        ? html`
+            <button part="label" type="button" class="label label-button">
+              <slot @slotchange=${this.handleSlotChange}></slot>
+            </button>
+          `
+        : ""}
+      ${this.renderType === "dropdown"
+        ? html`
+            <div part="label" class="label label-dropdown">
+              <slot @slotchange=${this.handleSlotChange}></slot>
+            </div>
+          `
+        : ""}
 
       <span part="end" class="end">
         <slot name="end"></slot>
       </span>

`````

### Actual (oxfmt)

`````ts
import { html } from "lit";
import { customElement, property, query, state } from "lit/decorators.js";
import { ifDefined } from "lit/directives/if-defined.js";
import { watch } from "../../internal/watch.js";
import WebAwesomeElement from "../../internal/webawesome-element.js";
import styles from "./breadcrumb-item.styles.js";

/**
 * @summary Breadcrumb items represent individual links inside a breadcrumb, typically one per level of the site
 *  hierarchy.
 * @documentation https://webawesome.com/docs/components/breadcrumb-item
 * @status stable
 * @since 2.0
 *
 * @slot - The breadcrumb item's label.
 * @slot start - An element, such as `<wa-icon>`, placed before the label.
 * @slot end - An element, such as `<wa-icon>`, placed after the label.
 * @slot separator - The separator to use for the breadcrumb item. This will only change the separator for this item. If
 * you want to change it for all items in the group, set the separator on `<wa-breadcrumb>` instead.
 *
 * @csspart label - The breadcrumb item's label.
 * @csspart start - The container that wraps the `start` slot.
 * @csspart end - The container that wraps the `end` slot.
 * @csspart separator - The container that wraps the separator.
 */
@customElement("wa-breadcrumb-item")
export default class WaBreadcrumbItem extends WebAwesomeElement {
  static css = styles;

  @query("slot:not([name])") defaultSlot: HTMLSlotElement;

  @state() private renderType: "button" | "link" | "dropdown" = "button";

  /**
   * Optional URL to direct the user to when the breadcrumb item is activated. When set, a link will be rendered
   * internally. When unset, a button will be rendered instead.
   */
  @property() href?: string;

  /** Tells the browser where to open the link. Only used when `href` is set. */
  @property() target?: "_blank" | "_parent" | "_self" | "_top";

  /** The `rel` attribute to use on the link. Only used when `href` is set. */
  @property() rel = "noreferrer noopener";

  private setRenderType() {
    const hasDropdown =
      this.defaultSlot
        .assignedElements({ flatten: true })
        .filter((i) => i.tagName.toLowerCase() === "wa-dropdown").length > 0;

    if (this.href) {
      this.renderType = "link";
      return;
    }

    if (hasDropdown) {
      this.renderType = "dropdown";
      return;
    }

    this.renderType = "button";
  }

  @watch("href", { waitUntilFirstUpdate: true })
  hrefChanged() {
    this.setRenderType();
  }

  handleSlotChange() {
    this.setRenderType();
  }

  render() {
    return html`
      <span part="start" class="start">
        <slot name="start"></slot>
      </span>

      ${this.renderType === "link"
        ? html`
            <a
              part="label"
              class="label label-link"
              href="${this.href!}"
              target="${ifDefined(this.target ? this.target : undefined)}"
              rel=${ifDefined(this.target ? this.rel : undefined)}
            >
              <slot></slot>
            </a>
          `
        : ""}
      ${this.renderType === "button"
        ? html`
            <button part="label" type="button" class="label label-button">
              <slot @slotchange=${this.handleSlotChange}></slot>
            </button>
          `
        : ""}
      ${this.renderType === "dropdown"
        ? html`
            <div part="label" class="label label-dropdown">
              <slot @slotchange=${this.handleSlotChange}></slot>
            </div>
          `
        : ""}

      <span part="end" class="end">
        <slot name="end"></slot>
      </span>

      <span part="separator" class="separator" aria-hidden="true">
        <slot name="separator"></slot>
      </span>
    `;
  }
}

declare global {
  interface HTMLElementTagNameMap {
    "wa-breadcrumb-item": WaBreadcrumbItem;
  }
}

`````

### Expected (prettier)

`````ts
import { html } from "lit";
import { customElement, property, query, state } from "lit/decorators.js";
import { ifDefined } from "lit/directives/if-defined.js";
import { watch } from "../../internal/watch.js";
import WebAwesomeElement from "../../internal/webawesome-element.js";
import styles from "./breadcrumb-item.styles.js";

/**
 * @summary Breadcrumb items represent individual links inside a breadcrumb, typically one per level of the site
 *  hierarchy.
 * @documentation https://webawesome.com/docs/components/breadcrumb-item
 * @status stable
 * @since 2.0
 *
 * @slot - The breadcrumb item's label.
 * @slot start - An element, such as `<wa-icon>`, placed before the label.
 * @slot end - An element, such as `<wa-icon>`, placed after the label.
 * @slot separator - The separator to use for the breadcrumb item. This will only change the separator for this item. If
 * you want to change it for all items in the group, set the separator on `<wa-breadcrumb>` instead.
 *
 * @csspart label - The breadcrumb item's label.
 * @csspart start - The container that wraps the `start` slot.
 * @csspart end - The container that wraps the `end` slot.
 * @csspart separator - The container that wraps the separator.
 */
@customElement("wa-breadcrumb-item")
export default class WaBreadcrumbItem extends WebAwesomeElement {
  static css = styles;

  @query("slot:not([name])") defaultSlot: HTMLSlotElement;

  @state() private renderType: "button" | "link" | "dropdown" = "button";

  /**
   * Optional URL to direct the user to when the breadcrumb item is activated. When set, a link will be rendered
   * internally. When unset, a button will be rendered instead.
   */
  @property() href?: string;

  /** Tells the browser where to open the link. Only used when `href` is set. */
  @property() target?: "_blank" | "_parent" | "_self" | "_top";

  /** The `rel` attribute to use on the link. Only used when `href` is set. */
  @property() rel = "noreferrer noopener";

  private setRenderType() {
    const hasDropdown =
      this.defaultSlot
        .assignedElements({ flatten: true })
        .filter((i) => i.tagName.toLowerCase() === "wa-dropdown").length > 0;

    if (this.href) {
      this.renderType = "link";
      return;
    }

    if (hasDropdown) {
      this.renderType = "dropdown";
      return;
    }

    this.renderType = "button";
  }

  @watch("href", { waitUntilFirstUpdate: true })
  hrefChanged() {
    this.setRenderType();
  }

  handleSlotChange() {
    this.setRenderType();
  }

  render() {
    return html`
      <span part="start" class="start">
        <slot name="start"></slot>
      </span>

      ${
        this.renderType === "link"
          ? html`
              <a
                part="label"
                class="label label-link"
                href="${this.href!}"
                target="${ifDefined(this.target ? this.target : undefined)}"
                rel=${ifDefined(this.target ? this.rel : undefined)}
              >
                <slot></slot>
              </a>
            `
          : ""
      }
      ${
        this.renderType === "button"
          ? html`
              <button part="label" type="button" class="label label-button">
                <slot @slotchange=${this.handleSlotChange}></slot>
              </button>
            `
          : ""
      }
      ${
        this.renderType === "dropdown"
          ? html`
              <div part="label" class="label label-dropdown">
                <slot @slotchange=${this.handleSlotChange}></slot>
              </div>
            `
          : ""
      }

      <span part="end" class="end">
        <slot name="end"></slot>
      </span>

      <span part="separator" class="separator" aria-hidden="true">
        <slot name="separator"></slot>
      </span>
    `;
  }
}

declare global {
  interface HTMLElementTagNameMap {
    "wa-breadcrumb-item": WaBreadcrumbItem;
  }
}

`````
