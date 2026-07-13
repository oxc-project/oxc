# externals/webawesome/dropdown-item/dropdown-item.ts

## Option 1

`````json
{"printWidth":80}
`````

### Diff

`````diff
===================================================================
--- prettier
+++ oxfmt
@@ -55,9 +55,16 @@
   /**
    * @internal The dropdown item's size.
    */
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
@@ -293,21 +300,19 @@
   }
 
   render() {
     return html`
-      ${
-        this.type === "checkbox"
-          ? html`
-              <wa-icon
-                id="check"
-                part="checkmark"
-                exportparts="svg:checkmark__svg"
-                library="system"
-                name="check"
-              ></wa-icon>
-            `
-          : ""
-      }
+      ${this.type === "checkbox"
+        ? html`
+            <wa-icon
+              id="check"
+              part="checkmark"
+              exportparts="svg:checkmark__svg"
+              library="system"
+              name="check"
+            ></wa-icon>
+          `
+        : ""}
 
       <span id="icon" part="icon">
         <slot name="icon"></slot>
       </span>
@@ -319,38 +324,34 @@
       <span id="details" part="details">
         <slot name="details"></slot>
       </span>
 
-      ${
-        this.hasSubmenu
-          ? html`
-              <wa-icon
-                id="submenu-indicator"
-                part="submenu-icon"
-                exportparts="svg:submenu-icon__svg"
-                library="system"
-                name="chevron-right"
-              ></wa-icon>
-            `
-          : ""
-      }
-      ${
-        this.hasSubmenu
-          ? html`
-              <div
-                id="submenu"
-                part="submenu"
-                popover="manual"
-                role="menu"
-                tabindex="-1"
-                aria-orientation="vertical"
-                hidden
-              >
-                <slot name="submenu"></slot>
-              </div>
-            `
-          : ""
-      }
+      ${this.hasSubmenu
+        ? html`
+            <wa-icon
+              id="submenu-indicator"
+              part="submenu-icon"
+              exportparts="svg:submenu-icon__svg"
+              library="system"
+              name="chevron-right"
+            ></wa-icon>
+          `
+        : ""}
+      ${this.hasSubmenu
+        ? html`
+            <div
+              id="submenu"
+              part="submenu"
+              popover="manual"
+              role="menu"
+              tabindex="-1"
+              aria-orientation="vertical"
+              hidden
+            >
+              <slot name="submenu"></slot>
+            </div>
+          `
+        : ""}
     `;
   }
 }
 

`````

### Actual (oxfmt)

`````ts
import type { PropertyValues } from "lit";
import { html } from "lit";
import { customElement, property, query, state } from "lit/decorators.js";
import { animateWithClass } from "../../internal/animate.js";
import { warnDeprecatedSize } from "../../internal/size.js";
import { HasSlotController } from "../../internal/slot.js";
import { watch } from "../../internal/watch.js";
import WebAwesomeElement from "../../internal/webawesome-element.js";
import "../icon/icon.js";
import styles from "./dropdown-item.styles.js";

/**
 * @summary Dropdown items represent selectable entries within a dropdown menu, including standard actions, checkable
 *  items, and submenu triggers.
 * @documentation https://webawesome.com/docs/components/dropdown-item
 * @status stable
 * @since 3.0
 *
 * @dependency wa-icon
 *
 * @event blur - Emitted when the dropdown item loses focus.
 * @event focus - Emitted when the dropdown item gains focus.
 *
 * @slot - The dropdown item's label.
 * @slot icon - An optional icon to display before the label.
 * @slot details - Additional content or details to display after the label.
 * @slot submenu - Submenu items, typically `<wa-dropdown-item>` elements, to create a nested menu.
 *
 * @csspart checkmark - The checkmark icon (a `<wa-icon>` element) when the item is a checkbox.
 * @csspart icon - The container for the icon slot.
 * @csspart label - The container for the label slot.
 * @csspart details - The container for the details slot.
 * @csspart submenu-icon - The submenu indicator icon (a `<wa-icon>` element).
 * @csspart submenu - The submenu container.
 */
@customElement("wa-dropdown-item")
export default class WaDropdownItem extends WebAwesomeElement {
  static css = styles;

  private readonly hasSlotController = new HasSlotController(
    this,
    "[default]",
    "start",
    "end",
  );

  @query("#submenu") submenuElement: HTMLDivElement;

  /** @internal The controller will set this property to true when the item is active. */
  @property({ type: Boolean }) active = false;

  /** The type of menu item to render. */
  @property({ reflect: true }) variant: "danger" | "default" = "default";

  /**
   * @internal The dropdown item's size.
   */
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
   * @internal The controller will set this property to true when at least one checkbox exists in the dropdown. This
   * allows non-checkbox items to draw additional space to align properly with checkbox items.
   */
  @property({ attribute: "checkbox-adjacent", type: Boolean, reflect: true })
  checkboxAdjacent = false;

  /**
   * @internal The controller will set this property to true when at least one item with a submenu exists in the
   * dropdown. This allows non-submenu items to draw additional space to align properly with items that have submenus.
   */
  @property({ attribute: "submenu-adjacent", type: Boolean, reflect: true })
  submenuAdjacent = false;

  /**
   * An optional value for the menu item. This is useful for determining which item was selected when listening to the
   * dropdown's `wa-select` event.
   */
  @property() value: string;

  /** Set to `checkbox` to make the item a checkbox. */
  @property({ reflect: true }) type: "normal" | "checkbox" = "normal";

  /** Set to true to check the dropdown item. Only valid when `type` is `checkbox`. */
  @property({ type: Boolean }) checked = false;

  /** Disables the dropdown item. */
  @property({ type: Boolean, reflect: true }) disabled = false;

  /** Whether the submenu is currently open. */
  @property({ type: Boolean, reflect: true }) submenuOpen = false;

  /** @internal Store whether this item has a submenu */
  @state() hasSubmenu = false;

  connectedCallback() {
    super.connectedCallback();
    this.addEventListener("click", this.handleHostClick);
    this.addEventListener("mouseenter", this.handleMouseEnter.bind(this));
    this.shadowRoot!.addEventListener("click", this.handleClick, {
      capture: true,
    });
    this.shadowRoot!.addEventListener("slotchange", this.handleSlotChange);
  }

  disconnectedCallback() {
    super.disconnectedCallback();
    this.closeSubmenu();
    this.removeEventListener("click", this.handleHostClick);
    this.removeEventListener("mouseenter", this.handleMouseEnter);
    this.shadowRoot!.removeEventListener("click", this.handleClick, {
      capture: true,
    });
    this.shadowRoot!.removeEventListener("slotchange", this.handleSlotChange);
  }

  firstUpdated() {
    this.setAttribute("tabindex", "-1");
    this.hasSubmenu = this.hasSlotController.test("submenu");
    this.updateHasSubmenuState();
  }

  updated(changedProperties: PropertyValues<this>) {
    if (changedProperties.has("active")) {
      this.setAttribute("tabindex", this.active ? "0" : "-1");
      this.customStates.set("active", this.active);
    }

    if (changedProperties.has("checked")) {
      if (this.type === "checkbox") {
        this.setAttribute("aria-checked", this.checked ? "true" : "false");
      } else {
        this.removeAttribute("aria-checked");
      }
      this.customStates.set("checked", this.checked);
    }

    if (changedProperties.has("disabled")) {
      this.setAttribute("aria-disabled", this.disabled ? "true" : "false");
      this.customStates.set("disabled", this.disabled);
    }

    if (changedProperties.has("type")) {
      if (this.type === "checkbox") {
        this.setAttribute("role", "menuitemcheckbox");
        this.setAttribute("aria-checked", this.checked ? "true" : "false");
      } else {
        this.setAttribute("role", "menuitem");
        this.removeAttribute("aria-checked");
      }
    }

    if (changedProperties.has("submenuOpen")) {
      this.customStates.set("submenu-open", this.submenuOpen);
      if (this.submenuOpen) {
        this.openSubmenu();
      } else {
        this.closeSubmenu();
      }
    }
  }

  private handleSlotChange = () => {
    this.hasSubmenu = this.hasSlotController.test("submenu");
    this.updateHasSubmenuState();

    if (this.hasSubmenu) {
      this.setAttribute("aria-haspopup", "menu");
      this.setAttribute("aria-expanded", this.submenuOpen ? "true" : "false");
    } else {
      this.removeAttribute("aria-haspopup");
      this.removeAttribute("aria-expanded");
    }
  };

  /** Update the has-submenu custom state */
  private updateHasSubmenuState() {
    this.customStates.set("has-submenu", this.hasSubmenu);
  }

  /** Opens the submenu. */
  async openSubmenu() {
    const submenu = this.submenuElement;
    if (!this.hasSubmenu || !submenu || !this.isConnected) return;

    // Notify parent dropdown to handle positioning
    this.notifyParentOfOpening();

    // Use Popover API to show the submenu
    submenu.showPopover?.();
    submenu.hidden = false;
    submenu.setAttribute("data-visible", "");
    this.submenuOpen = true;
    this.setAttribute("aria-expanded", "true");

    // Animate the submenu
    await animateWithClass(submenu, "show");

    // Set focus to the first submenu item
    setTimeout(() => {
      const items = this.getSubmenuItems();
      if (items.length > 0) {
        items.forEach((item, index) => (item.active = index === 0));
        items[0].focus({ preventScroll: true });
      }
    }, 0);
  }

  /** Notifies the parent dropdown that this item is opening its submenu */
  private notifyParentOfOpening() {
    // First notify the parent that we're about to open
    const event = new CustomEvent("submenu-opening", {
      bubbles: true,
      composed: true,
      detail: { item: this },
    });
    this.dispatchEvent(event);

    // Find sibling items that have open submenus and close them
    const parent = this.parentElement;
    if (parent) {
      const siblings = [...parent.children].filter(
        (el) =>
          el !== this &&
          el.localName === "wa-dropdown-item" &&
          el.getAttribute("slot") === this.getAttribute("slot") &&
          (el as WaDropdownItem).submenuOpen,
      ) as WaDropdownItem[];

      // Close each sibling submenu with animation
      siblings.forEach((sibling) => {
        sibling.submenuOpen = false;
      });
    }
  }

  /** Closes the submenu. */
  async closeSubmenu() {
    const submenu = this.submenuElement;
    if (!this.hasSubmenu || !submenu) return;

    this.submenuOpen = false;
    this.setAttribute("aria-expanded", "false");

    if (!submenu.hidden) {
      await animateWithClass(submenu, "hide");
      if (submenu?.isConnected) {
        submenu.hidden = true;
        submenu.removeAttribute("data-visible");
        submenu.hidePopover?.();
      }
    }
  }

  /** Gets all dropdown items in the submenu. */
  private getSubmenuItems(): WaDropdownItem[] {
    // Only get direct children with slot="submenu", not nested ones
    return [...this.children].filter(
      (el) =>
        el.localName === "wa-dropdown-item" &&
        el.getAttribute("slot") === "submenu" &&
        !el.hasAttribute("disabled"),
    ) as WaDropdownItem[];
  }

  /** Prevents click events from firing on the host when the item is disabled (e.g. programmatic .click() calls). */
  private handleHostClick = (event: MouseEvent) => {
    if (this.disabled) {
      event.preventDefault();
      event.stopImmediatePropagation();
    }
  };

  /** Prevents click events from firing when the item is disabled. */
  private handleClick = (event: MouseEvent) => {
    if (this.disabled) {
      event.preventDefault();
      event.stopImmediatePropagation();
    }
  };

  /** Handles mouse enter to open the submenu */
  private handleMouseEnter() {
    if (this.hasSubmenu && !this.disabled) {
      this.notifyParentOfOpening();
      this.submenuOpen = true;
    }
  }

  render() {
    return html`
      ${this.type === "checkbox"
        ? html`
            <wa-icon
              id="check"
              part="checkmark"
              exportparts="svg:checkmark__svg"
              library="system"
              name="check"
            ></wa-icon>
          `
        : ""}

      <span id="icon" part="icon">
        <slot name="icon"></slot>
      </span>

      <span id="label" part="label">
        <slot></slot>
      </span>

      <span id="details" part="details">
        <slot name="details"></slot>
      </span>

      ${this.hasSubmenu
        ? html`
            <wa-icon
              id="submenu-indicator"
              part="submenu-icon"
              exportparts="svg:submenu-icon__svg"
              library="system"
              name="chevron-right"
            ></wa-icon>
          `
        : ""}
      ${this.hasSubmenu
        ? html`
            <div
              id="submenu"
              part="submenu"
              popover="manual"
              role="menu"
              tabindex="-1"
              aria-orientation="vertical"
              hidden
            >
              <slot name="submenu"></slot>
            </div>
          `
        : ""}
    `;
  }
}

declare global {
  interface HTMLElementTagNameMap {
    "wa-dropdown-item": WaDropdownItem;
  }
}

`````

### Expected (prettier)

`````ts
import type { PropertyValues } from "lit";
import { html } from "lit";
import { customElement, property, query, state } from "lit/decorators.js";
import { animateWithClass } from "../../internal/animate.js";
import { warnDeprecatedSize } from "../../internal/size.js";
import { HasSlotController } from "../../internal/slot.js";
import { watch } from "../../internal/watch.js";
import WebAwesomeElement from "../../internal/webawesome-element.js";
import "../icon/icon.js";
import styles from "./dropdown-item.styles.js";

/**
 * @summary Dropdown items represent selectable entries within a dropdown menu, including standard actions, checkable
 *  items, and submenu triggers.
 * @documentation https://webawesome.com/docs/components/dropdown-item
 * @status stable
 * @since 3.0
 *
 * @dependency wa-icon
 *
 * @event blur - Emitted when the dropdown item loses focus.
 * @event focus - Emitted when the dropdown item gains focus.
 *
 * @slot - The dropdown item's label.
 * @slot icon - An optional icon to display before the label.
 * @slot details - Additional content or details to display after the label.
 * @slot submenu - Submenu items, typically `<wa-dropdown-item>` elements, to create a nested menu.
 *
 * @csspart checkmark - The checkmark icon (a `<wa-icon>` element) when the item is a checkbox.
 * @csspart icon - The container for the icon slot.
 * @csspart label - The container for the label slot.
 * @csspart details - The container for the details slot.
 * @csspart submenu-icon - The submenu indicator icon (a `<wa-icon>` element).
 * @csspart submenu - The submenu container.
 */
@customElement("wa-dropdown-item")
export default class WaDropdownItem extends WebAwesomeElement {
  static css = styles;

  private readonly hasSlotController = new HasSlotController(
    this,
    "[default]",
    "start",
    "end",
  );

  @query("#submenu") submenuElement: HTMLDivElement;

  /** @internal The controller will set this property to true when the item is active. */
  @property({ type: Boolean }) active = false;

  /** The type of menu item to render. */
  @property({ reflect: true }) variant: "danger" | "default" = "default";

  /**
   * @internal The dropdown item's size.
   */
  @property({ reflect: true }) size:
    "xs" | "s" | "m" | "l" | "xl" | "small" | "medium" | "large" = "m";

  @watch("size")
  handleSizeChange() {
    warnDeprecatedSize(this.localName, this.size);
  }

  /**
   * @internal The controller will set this property to true when at least one checkbox exists in the dropdown. This
   * allows non-checkbox items to draw additional space to align properly with checkbox items.
   */
  @property({ attribute: "checkbox-adjacent", type: Boolean, reflect: true })
  checkboxAdjacent = false;

  /**
   * @internal The controller will set this property to true when at least one item with a submenu exists in the
   * dropdown. This allows non-submenu items to draw additional space to align properly with items that have submenus.
   */
  @property({ attribute: "submenu-adjacent", type: Boolean, reflect: true })
  submenuAdjacent = false;

  /**
   * An optional value for the menu item. This is useful for determining which item was selected when listening to the
   * dropdown's `wa-select` event.
   */
  @property() value: string;

  /** Set to `checkbox` to make the item a checkbox. */
  @property({ reflect: true }) type: "normal" | "checkbox" = "normal";

  /** Set to true to check the dropdown item. Only valid when `type` is `checkbox`. */
  @property({ type: Boolean }) checked = false;

  /** Disables the dropdown item. */
  @property({ type: Boolean, reflect: true }) disabled = false;

  /** Whether the submenu is currently open. */
  @property({ type: Boolean, reflect: true }) submenuOpen = false;

  /** @internal Store whether this item has a submenu */
  @state() hasSubmenu = false;

  connectedCallback() {
    super.connectedCallback();
    this.addEventListener("click", this.handleHostClick);
    this.addEventListener("mouseenter", this.handleMouseEnter.bind(this));
    this.shadowRoot!.addEventListener("click", this.handleClick, {
      capture: true,
    });
    this.shadowRoot!.addEventListener("slotchange", this.handleSlotChange);
  }

  disconnectedCallback() {
    super.disconnectedCallback();
    this.closeSubmenu();
    this.removeEventListener("click", this.handleHostClick);
    this.removeEventListener("mouseenter", this.handleMouseEnter);
    this.shadowRoot!.removeEventListener("click", this.handleClick, {
      capture: true,
    });
    this.shadowRoot!.removeEventListener("slotchange", this.handleSlotChange);
  }

  firstUpdated() {
    this.setAttribute("tabindex", "-1");
    this.hasSubmenu = this.hasSlotController.test("submenu");
    this.updateHasSubmenuState();
  }

  updated(changedProperties: PropertyValues<this>) {
    if (changedProperties.has("active")) {
      this.setAttribute("tabindex", this.active ? "0" : "-1");
      this.customStates.set("active", this.active);
    }

    if (changedProperties.has("checked")) {
      if (this.type === "checkbox") {
        this.setAttribute("aria-checked", this.checked ? "true" : "false");
      } else {
        this.removeAttribute("aria-checked");
      }
      this.customStates.set("checked", this.checked);
    }

    if (changedProperties.has("disabled")) {
      this.setAttribute("aria-disabled", this.disabled ? "true" : "false");
      this.customStates.set("disabled", this.disabled);
    }

    if (changedProperties.has("type")) {
      if (this.type === "checkbox") {
        this.setAttribute("role", "menuitemcheckbox");
        this.setAttribute("aria-checked", this.checked ? "true" : "false");
      } else {
        this.setAttribute("role", "menuitem");
        this.removeAttribute("aria-checked");
      }
    }

    if (changedProperties.has("submenuOpen")) {
      this.customStates.set("submenu-open", this.submenuOpen);
      if (this.submenuOpen) {
        this.openSubmenu();
      } else {
        this.closeSubmenu();
      }
    }
  }

  private handleSlotChange = () => {
    this.hasSubmenu = this.hasSlotController.test("submenu");
    this.updateHasSubmenuState();

    if (this.hasSubmenu) {
      this.setAttribute("aria-haspopup", "menu");
      this.setAttribute("aria-expanded", this.submenuOpen ? "true" : "false");
    } else {
      this.removeAttribute("aria-haspopup");
      this.removeAttribute("aria-expanded");
    }
  };

  /** Update the has-submenu custom state */
  private updateHasSubmenuState() {
    this.customStates.set("has-submenu", this.hasSubmenu);
  }

  /** Opens the submenu. */
  async openSubmenu() {
    const submenu = this.submenuElement;
    if (!this.hasSubmenu || !submenu || !this.isConnected) return;

    // Notify parent dropdown to handle positioning
    this.notifyParentOfOpening();

    // Use Popover API to show the submenu
    submenu.showPopover?.();
    submenu.hidden = false;
    submenu.setAttribute("data-visible", "");
    this.submenuOpen = true;
    this.setAttribute("aria-expanded", "true");

    // Animate the submenu
    await animateWithClass(submenu, "show");

    // Set focus to the first submenu item
    setTimeout(() => {
      const items = this.getSubmenuItems();
      if (items.length > 0) {
        items.forEach((item, index) => (item.active = index === 0));
        items[0].focus({ preventScroll: true });
      }
    }, 0);
  }

  /** Notifies the parent dropdown that this item is opening its submenu */
  private notifyParentOfOpening() {
    // First notify the parent that we're about to open
    const event = new CustomEvent("submenu-opening", {
      bubbles: true,
      composed: true,
      detail: { item: this },
    });
    this.dispatchEvent(event);

    // Find sibling items that have open submenus and close them
    const parent = this.parentElement;
    if (parent) {
      const siblings = [...parent.children].filter(
        (el) =>
          el !== this &&
          el.localName === "wa-dropdown-item" &&
          el.getAttribute("slot") === this.getAttribute("slot") &&
          (el as WaDropdownItem).submenuOpen,
      ) as WaDropdownItem[];

      // Close each sibling submenu with animation
      siblings.forEach((sibling) => {
        sibling.submenuOpen = false;
      });
    }
  }

  /** Closes the submenu. */
  async closeSubmenu() {
    const submenu = this.submenuElement;
    if (!this.hasSubmenu || !submenu) return;

    this.submenuOpen = false;
    this.setAttribute("aria-expanded", "false");

    if (!submenu.hidden) {
      await animateWithClass(submenu, "hide");
      if (submenu?.isConnected) {
        submenu.hidden = true;
        submenu.removeAttribute("data-visible");
        submenu.hidePopover?.();
      }
    }
  }

  /** Gets all dropdown items in the submenu. */
  private getSubmenuItems(): WaDropdownItem[] {
    // Only get direct children with slot="submenu", not nested ones
    return [...this.children].filter(
      (el) =>
        el.localName === "wa-dropdown-item" &&
        el.getAttribute("slot") === "submenu" &&
        !el.hasAttribute("disabled"),
    ) as WaDropdownItem[];
  }

  /** Prevents click events from firing on the host when the item is disabled (e.g. programmatic .click() calls). */
  private handleHostClick = (event: MouseEvent) => {
    if (this.disabled) {
      event.preventDefault();
      event.stopImmediatePropagation();
    }
  };

  /** Prevents click events from firing when the item is disabled. */
  private handleClick = (event: MouseEvent) => {
    if (this.disabled) {
      event.preventDefault();
      event.stopImmediatePropagation();
    }
  };

  /** Handles mouse enter to open the submenu */
  private handleMouseEnter() {
    if (this.hasSubmenu && !this.disabled) {
      this.notifyParentOfOpening();
      this.submenuOpen = true;
    }
  }

  render() {
    return html`
      ${
        this.type === "checkbox"
          ? html`
              <wa-icon
                id="check"
                part="checkmark"
                exportparts="svg:checkmark__svg"
                library="system"
                name="check"
              ></wa-icon>
            `
          : ""
      }

      <span id="icon" part="icon">
        <slot name="icon"></slot>
      </span>

      <span id="label" part="label">
        <slot></slot>
      </span>

      <span id="details" part="details">
        <slot name="details"></slot>
      </span>

      ${
        this.hasSubmenu
          ? html`
              <wa-icon
                id="submenu-indicator"
                part="submenu-icon"
                exportparts="svg:submenu-icon__svg"
                library="system"
                name="chevron-right"
              ></wa-icon>
            `
          : ""
      }
      ${
        this.hasSubmenu
          ? html`
              <div
                id="submenu"
                part="submenu"
                popover="manual"
                role="menu"
                tabindex="-1"
                aria-orientation="vertical"
                hidden
              >
                <slot name="submenu"></slot>
              </div>
            `
          : ""
      }
    `;
  }
}

declare global {
  interface HTMLElementTagNameMap {
    "wa-dropdown-item": WaDropdownItem;
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
@@ -284,21 +284,19 @@
   }
 
   render() {
     return html`
-      ${
-        this.type === "checkbox"
-          ? html`
-              <wa-icon
-                id="check"
-                part="checkmark"
-                exportparts="svg:checkmark__svg"
-                library="system"
-                name="check"
-              ></wa-icon>
-            `
-          : ""
-      }
+      ${this.type === "checkbox"
+        ? html`
+            <wa-icon
+              id="check"
+              part="checkmark"
+              exportparts="svg:checkmark__svg"
+              library="system"
+              name="check"
+            ></wa-icon>
+          `
+        : ""}
 
       <span id="icon" part="icon">
         <slot name="icon"></slot>
       </span>
@@ -310,38 +308,34 @@
       <span id="details" part="details">
         <slot name="details"></slot>
       </span>
 
-      ${
-        this.hasSubmenu
-          ? html`
-              <wa-icon
-                id="submenu-indicator"
-                part="submenu-icon"
-                exportparts="svg:submenu-icon__svg"
-                library="system"
-                name="chevron-right"
-              ></wa-icon>
-            `
-          : ""
-      }
-      ${
-        this.hasSubmenu
-          ? html`
-              <div
-                id="submenu"
-                part="submenu"
-                popover="manual"
-                role="menu"
-                tabindex="-1"
-                aria-orientation="vertical"
-                hidden
-              >
-                <slot name="submenu"></slot>
-              </div>
-            `
-          : ""
-      }
+      ${this.hasSubmenu
+        ? html`
+            <wa-icon
+              id="submenu-indicator"
+              part="submenu-icon"
+              exportparts="svg:submenu-icon__svg"
+              library="system"
+              name="chevron-right"
+            ></wa-icon>
+          `
+        : ""}
+      ${this.hasSubmenu
+        ? html`
+            <div
+              id="submenu"
+              part="submenu"
+              popover="manual"
+              role="menu"
+              tabindex="-1"
+              aria-orientation="vertical"
+              hidden
+            >
+              <slot name="submenu"></slot>
+            </div>
+          `
+        : ""}
     `;
   }
 }
 

`````

### Actual (oxfmt)

`````ts
import type { PropertyValues } from "lit";
import { html } from "lit";
import { customElement, property, query, state } from "lit/decorators.js";
import { animateWithClass } from "../../internal/animate.js";
import { warnDeprecatedSize } from "../../internal/size.js";
import { HasSlotController } from "../../internal/slot.js";
import { watch } from "../../internal/watch.js";
import WebAwesomeElement from "../../internal/webawesome-element.js";
import "../icon/icon.js";
import styles from "./dropdown-item.styles.js";

/**
 * @summary Dropdown items represent selectable entries within a dropdown menu, including standard actions, checkable
 *  items, and submenu triggers.
 * @documentation https://webawesome.com/docs/components/dropdown-item
 * @status stable
 * @since 3.0
 *
 * @dependency wa-icon
 *
 * @event blur - Emitted when the dropdown item loses focus.
 * @event focus - Emitted when the dropdown item gains focus.
 *
 * @slot - The dropdown item's label.
 * @slot icon - An optional icon to display before the label.
 * @slot details - Additional content or details to display after the label.
 * @slot submenu - Submenu items, typically `<wa-dropdown-item>` elements, to create a nested menu.
 *
 * @csspart checkmark - The checkmark icon (a `<wa-icon>` element) when the item is a checkbox.
 * @csspart icon - The container for the icon slot.
 * @csspart label - The container for the label slot.
 * @csspart details - The container for the details slot.
 * @csspart submenu-icon - The submenu indicator icon (a `<wa-icon>` element).
 * @csspart submenu - The submenu container.
 */
@customElement("wa-dropdown-item")
export default class WaDropdownItem extends WebAwesomeElement {
  static css = styles;

  private readonly hasSlotController = new HasSlotController(this, "[default]", "start", "end");

  @query("#submenu") submenuElement: HTMLDivElement;

  /** @internal The controller will set this property to true when the item is active. */
  @property({ type: Boolean }) active = false;

  /** The type of menu item to render. */
  @property({ reflect: true }) variant: "danger" | "default" = "default";

  /**
   * @internal The dropdown item's size.
   */
  @property({ reflect: true }) size: "xs" | "s" | "m" | "l" | "xl" | "small" | "medium" | "large" =
    "m";

  @watch("size")
  handleSizeChange() {
    warnDeprecatedSize(this.localName, this.size);
  }

  /**
   * @internal The controller will set this property to true when at least one checkbox exists in the dropdown. This
   * allows non-checkbox items to draw additional space to align properly with checkbox items.
   */
  @property({ attribute: "checkbox-adjacent", type: Boolean, reflect: true }) checkboxAdjacent =
    false;

  /**
   * @internal The controller will set this property to true when at least one item with a submenu exists in the
   * dropdown. This allows non-submenu items to draw additional space to align properly with items that have submenus.
   */
  @property({ attribute: "submenu-adjacent", type: Boolean, reflect: true }) submenuAdjacent =
    false;

  /**
   * An optional value for the menu item. This is useful for determining which item was selected when listening to the
   * dropdown's `wa-select` event.
   */
  @property() value: string;

  /** Set to `checkbox` to make the item a checkbox. */
  @property({ reflect: true }) type: "normal" | "checkbox" = "normal";

  /** Set to true to check the dropdown item. Only valid when `type` is `checkbox`. */
  @property({ type: Boolean }) checked = false;

  /** Disables the dropdown item. */
  @property({ type: Boolean, reflect: true }) disabled = false;

  /** Whether the submenu is currently open. */
  @property({ type: Boolean, reflect: true }) submenuOpen = false;

  /** @internal Store whether this item has a submenu */
  @state() hasSubmenu = false;

  connectedCallback() {
    super.connectedCallback();
    this.addEventListener("click", this.handleHostClick);
    this.addEventListener("mouseenter", this.handleMouseEnter.bind(this));
    this.shadowRoot!.addEventListener("click", this.handleClick, { capture: true });
    this.shadowRoot!.addEventListener("slotchange", this.handleSlotChange);
  }

  disconnectedCallback() {
    super.disconnectedCallback();
    this.closeSubmenu();
    this.removeEventListener("click", this.handleHostClick);
    this.removeEventListener("mouseenter", this.handleMouseEnter);
    this.shadowRoot!.removeEventListener("click", this.handleClick, { capture: true });
    this.shadowRoot!.removeEventListener("slotchange", this.handleSlotChange);
  }

  firstUpdated() {
    this.setAttribute("tabindex", "-1");
    this.hasSubmenu = this.hasSlotController.test("submenu");
    this.updateHasSubmenuState();
  }

  updated(changedProperties: PropertyValues<this>) {
    if (changedProperties.has("active")) {
      this.setAttribute("tabindex", this.active ? "0" : "-1");
      this.customStates.set("active", this.active);
    }

    if (changedProperties.has("checked")) {
      if (this.type === "checkbox") {
        this.setAttribute("aria-checked", this.checked ? "true" : "false");
      } else {
        this.removeAttribute("aria-checked");
      }
      this.customStates.set("checked", this.checked);
    }

    if (changedProperties.has("disabled")) {
      this.setAttribute("aria-disabled", this.disabled ? "true" : "false");
      this.customStates.set("disabled", this.disabled);
    }

    if (changedProperties.has("type")) {
      if (this.type === "checkbox") {
        this.setAttribute("role", "menuitemcheckbox");
        this.setAttribute("aria-checked", this.checked ? "true" : "false");
      } else {
        this.setAttribute("role", "menuitem");
        this.removeAttribute("aria-checked");
      }
    }

    if (changedProperties.has("submenuOpen")) {
      this.customStates.set("submenu-open", this.submenuOpen);
      if (this.submenuOpen) {
        this.openSubmenu();
      } else {
        this.closeSubmenu();
      }
    }
  }

  private handleSlotChange = () => {
    this.hasSubmenu = this.hasSlotController.test("submenu");
    this.updateHasSubmenuState();

    if (this.hasSubmenu) {
      this.setAttribute("aria-haspopup", "menu");
      this.setAttribute("aria-expanded", this.submenuOpen ? "true" : "false");
    } else {
      this.removeAttribute("aria-haspopup");
      this.removeAttribute("aria-expanded");
    }
  };

  /** Update the has-submenu custom state */
  private updateHasSubmenuState() {
    this.customStates.set("has-submenu", this.hasSubmenu);
  }

  /** Opens the submenu. */
  async openSubmenu() {
    const submenu = this.submenuElement;
    if (!this.hasSubmenu || !submenu || !this.isConnected) return;

    // Notify parent dropdown to handle positioning
    this.notifyParentOfOpening();

    // Use Popover API to show the submenu
    submenu.showPopover?.();
    submenu.hidden = false;
    submenu.setAttribute("data-visible", "");
    this.submenuOpen = true;
    this.setAttribute("aria-expanded", "true");

    // Animate the submenu
    await animateWithClass(submenu, "show");

    // Set focus to the first submenu item
    setTimeout(() => {
      const items = this.getSubmenuItems();
      if (items.length > 0) {
        items.forEach((item, index) => (item.active = index === 0));
        items[0].focus({ preventScroll: true });
      }
    }, 0);
  }

  /** Notifies the parent dropdown that this item is opening its submenu */
  private notifyParentOfOpening() {
    // First notify the parent that we're about to open
    const event = new CustomEvent("submenu-opening", {
      bubbles: true,
      composed: true,
      detail: { item: this },
    });
    this.dispatchEvent(event);

    // Find sibling items that have open submenus and close them
    const parent = this.parentElement;
    if (parent) {
      const siblings = [...parent.children].filter(
        (el) =>
          el !== this &&
          el.localName === "wa-dropdown-item" &&
          el.getAttribute("slot") === this.getAttribute("slot") &&
          (el as WaDropdownItem).submenuOpen,
      ) as WaDropdownItem[];

      // Close each sibling submenu with animation
      siblings.forEach((sibling) => {
        sibling.submenuOpen = false;
      });
    }
  }

  /** Closes the submenu. */
  async closeSubmenu() {
    const submenu = this.submenuElement;
    if (!this.hasSubmenu || !submenu) return;

    this.submenuOpen = false;
    this.setAttribute("aria-expanded", "false");

    if (!submenu.hidden) {
      await animateWithClass(submenu, "hide");
      if (submenu?.isConnected) {
        submenu.hidden = true;
        submenu.removeAttribute("data-visible");
        submenu.hidePopover?.();
      }
    }
  }

  /** Gets all dropdown items in the submenu. */
  private getSubmenuItems(): WaDropdownItem[] {
    // Only get direct children with slot="submenu", not nested ones
    return [...this.children].filter(
      (el) =>
        el.localName === "wa-dropdown-item" &&
        el.getAttribute("slot") === "submenu" &&
        !el.hasAttribute("disabled"),
    ) as WaDropdownItem[];
  }

  /** Prevents click events from firing on the host when the item is disabled (e.g. programmatic .click() calls). */
  private handleHostClick = (event: MouseEvent) => {
    if (this.disabled) {
      event.preventDefault();
      event.stopImmediatePropagation();
    }
  };

  /** Prevents click events from firing when the item is disabled. */
  private handleClick = (event: MouseEvent) => {
    if (this.disabled) {
      event.preventDefault();
      event.stopImmediatePropagation();
    }
  };

  /** Handles mouse enter to open the submenu */
  private handleMouseEnter() {
    if (this.hasSubmenu && !this.disabled) {
      this.notifyParentOfOpening();
      this.submenuOpen = true;
    }
  }

  render() {
    return html`
      ${this.type === "checkbox"
        ? html`
            <wa-icon
              id="check"
              part="checkmark"
              exportparts="svg:checkmark__svg"
              library="system"
              name="check"
            ></wa-icon>
          `
        : ""}

      <span id="icon" part="icon">
        <slot name="icon"></slot>
      </span>

      <span id="label" part="label">
        <slot></slot>
      </span>

      <span id="details" part="details">
        <slot name="details"></slot>
      </span>

      ${this.hasSubmenu
        ? html`
            <wa-icon
              id="submenu-indicator"
              part="submenu-icon"
              exportparts="svg:submenu-icon__svg"
              library="system"
              name="chevron-right"
            ></wa-icon>
          `
        : ""}
      ${this.hasSubmenu
        ? html`
            <div
              id="submenu"
              part="submenu"
              popover="manual"
              role="menu"
              tabindex="-1"
              aria-orientation="vertical"
              hidden
            >
              <slot name="submenu"></slot>
            </div>
          `
        : ""}
    `;
  }
}

declare global {
  interface HTMLElementTagNameMap {
    "wa-dropdown-item": WaDropdownItem;
  }
}

`````

### Expected (prettier)

`````ts
import type { PropertyValues } from "lit";
import { html } from "lit";
import { customElement, property, query, state } from "lit/decorators.js";
import { animateWithClass } from "../../internal/animate.js";
import { warnDeprecatedSize } from "../../internal/size.js";
import { HasSlotController } from "../../internal/slot.js";
import { watch } from "../../internal/watch.js";
import WebAwesomeElement from "../../internal/webawesome-element.js";
import "../icon/icon.js";
import styles from "./dropdown-item.styles.js";

/**
 * @summary Dropdown items represent selectable entries within a dropdown menu, including standard actions, checkable
 *  items, and submenu triggers.
 * @documentation https://webawesome.com/docs/components/dropdown-item
 * @status stable
 * @since 3.0
 *
 * @dependency wa-icon
 *
 * @event blur - Emitted when the dropdown item loses focus.
 * @event focus - Emitted when the dropdown item gains focus.
 *
 * @slot - The dropdown item's label.
 * @slot icon - An optional icon to display before the label.
 * @slot details - Additional content or details to display after the label.
 * @slot submenu - Submenu items, typically `<wa-dropdown-item>` elements, to create a nested menu.
 *
 * @csspart checkmark - The checkmark icon (a `<wa-icon>` element) when the item is a checkbox.
 * @csspart icon - The container for the icon slot.
 * @csspart label - The container for the label slot.
 * @csspart details - The container for the details slot.
 * @csspart submenu-icon - The submenu indicator icon (a `<wa-icon>` element).
 * @csspart submenu - The submenu container.
 */
@customElement("wa-dropdown-item")
export default class WaDropdownItem extends WebAwesomeElement {
  static css = styles;

  private readonly hasSlotController = new HasSlotController(this, "[default]", "start", "end");

  @query("#submenu") submenuElement: HTMLDivElement;

  /** @internal The controller will set this property to true when the item is active. */
  @property({ type: Boolean }) active = false;

  /** The type of menu item to render. */
  @property({ reflect: true }) variant: "danger" | "default" = "default";

  /**
   * @internal The dropdown item's size.
   */
  @property({ reflect: true }) size: "xs" | "s" | "m" | "l" | "xl" | "small" | "medium" | "large" =
    "m";

  @watch("size")
  handleSizeChange() {
    warnDeprecatedSize(this.localName, this.size);
  }

  /**
   * @internal The controller will set this property to true when at least one checkbox exists in the dropdown. This
   * allows non-checkbox items to draw additional space to align properly with checkbox items.
   */
  @property({ attribute: "checkbox-adjacent", type: Boolean, reflect: true }) checkboxAdjacent =
    false;

  /**
   * @internal The controller will set this property to true when at least one item with a submenu exists in the
   * dropdown. This allows non-submenu items to draw additional space to align properly with items that have submenus.
   */
  @property({ attribute: "submenu-adjacent", type: Boolean, reflect: true }) submenuAdjacent =
    false;

  /**
   * An optional value for the menu item. This is useful for determining which item was selected when listening to the
   * dropdown's `wa-select` event.
   */
  @property() value: string;

  /** Set to `checkbox` to make the item a checkbox. */
  @property({ reflect: true }) type: "normal" | "checkbox" = "normal";

  /** Set to true to check the dropdown item. Only valid when `type` is `checkbox`. */
  @property({ type: Boolean }) checked = false;

  /** Disables the dropdown item. */
  @property({ type: Boolean, reflect: true }) disabled = false;

  /** Whether the submenu is currently open. */
  @property({ type: Boolean, reflect: true }) submenuOpen = false;

  /** @internal Store whether this item has a submenu */
  @state() hasSubmenu = false;

  connectedCallback() {
    super.connectedCallback();
    this.addEventListener("click", this.handleHostClick);
    this.addEventListener("mouseenter", this.handleMouseEnter.bind(this));
    this.shadowRoot!.addEventListener("click", this.handleClick, { capture: true });
    this.shadowRoot!.addEventListener("slotchange", this.handleSlotChange);
  }

  disconnectedCallback() {
    super.disconnectedCallback();
    this.closeSubmenu();
    this.removeEventListener("click", this.handleHostClick);
    this.removeEventListener("mouseenter", this.handleMouseEnter);
    this.shadowRoot!.removeEventListener("click", this.handleClick, { capture: true });
    this.shadowRoot!.removeEventListener("slotchange", this.handleSlotChange);
  }

  firstUpdated() {
    this.setAttribute("tabindex", "-1");
    this.hasSubmenu = this.hasSlotController.test("submenu");
    this.updateHasSubmenuState();
  }

  updated(changedProperties: PropertyValues<this>) {
    if (changedProperties.has("active")) {
      this.setAttribute("tabindex", this.active ? "0" : "-1");
      this.customStates.set("active", this.active);
    }

    if (changedProperties.has("checked")) {
      if (this.type === "checkbox") {
        this.setAttribute("aria-checked", this.checked ? "true" : "false");
      } else {
        this.removeAttribute("aria-checked");
      }
      this.customStates.set("checked", this.checked);
    }

    if (changedProperties.has("disabled")) {
      this.setAttribute("aria-disabled", this.disabled ? "true" : "false");
      this.customStates.set("disabled", this.disabled);
    }

    if (changedProperties.has("type")) {
      if (this.type === "checkbox") {
        this.setAttribute("role", "menuitemcheckbox");
        this.setAttribute("aria-checked", this.checked ? "true" : "false");
      } else {
        this.setAttribute("role", "menuitem");
        this.removeAttribute("aria-checked");
      }
    }

    if (changedProperties.has("submenuOpen")) {
      this.customStates.set("submenu-open", this.submenuOpen);
      if (this.submenuOpen) {
        this.openSubmenu();
      } else {
        this.closeSubmenu();
      }
    }
  }

  private handleSlotChange = () => {
    this.hasSubmenu = this.hasSlotController.test("submenu");
    this.updateHasSubmenuState();

    if (this.hasSubmenu) {
      this.setAttribute("aria-haspopup", "menu");
      this.setAttribute("aria-expanded", this.submenuOpen ? "true" : "false");
    } else {
      this.removeAttribute("aria-haspopup");
      this.removeAttribute("aria-expanded");
    }
  };

  /** Update the has-submenu custom state */
  private updateHasSubmenuState() {
    this.customStates.set("has-submenu", this.hasSubmenu);
  }

  /** Opens the submenu. */
  async openSubmenu() {
    const submenu = this.submenuElement;
    if (!this.hasSubmenu || !submenu || !this.isConnected) return;

    // Notify parent dropdown to handle positioning
    this.notifyParentOfOpening();

    // Use Popover API to show the submenu
    submenu.showPopover?.();
    submenu.hidden = false;
    submenu.setAttribute("data-visible", "");
    this.submenuOpen = true;
    this.setAttribute("aria-expanded", "true");

    // Animate the submenu
    await animateWithClass(submenu, "show");

    // Set focus to the first submenu item
    setTimeout(() => {
      const items = this.getSubmenuItems();
      if (items.length > 0) {
        items.forEach((item, index) => (item.active = index === 0));
        items[0].focus({ preventScroll: true });
      }
    }, 0);
  }

  /** Notifies the parent dropdown that this item is opening its submenu */
  private notifyParentOfOpening() {
    // First notify the parent that we're about to open
    const event = new CustomEvent("submenu-opening", {
      bubbles: true,
      composed: true,
      detail: { item: this },
    });
    this.dispatchEvent(event);

    // Find sibling items that have open submenus and close them
    const parent = this.parentElement;
    if (parent) {
      const siblings = [...parent.children].filter(
        (el) =>
          el !== this &&
          el.localName === "wa-dropdown-item" &&
          el.getAttribute("slot") === this.getAttribute("slot") &&
          (el as WaDropdownItem).submenuOpen,
      ) as WaDropdownItem[];

      // Close each sibling submenu with animation
      siblings.forEach((sibling) => {
        sibling.submenuOpen = false;
      });
    }
  }

  /** Closes the submenu. */
  async closeSubmenu() {
    const submenu = this.submenuElement;
    if (!this.hasSubmenu || !submenu) return;

    this.submenuOpen = false;
    this.setAttribute("aria-expanded", "false");

    if (!submenu.hidden) {
      await animateWithClass(submenu, "hide");
      if (submenu?.isConnected) {
        submenu.hidden = true;
        submenu.removeAttribute("data-visible");
        submenu.hidePopover?.();
      }
    }
  }

  /** Gets all dropdown items in the submenu. */
  private getSubmenuItems(): WaDropdownItem[] {
    // Only get direct children with slot="submenu", not nested ones
    return [...this.children].filter(
      (el) =>
        el.localName === "wa-dropdown-item" &&
        el.getAttribute("slot") === "submenu" &&
        !el.hasAttribute("disabled"),
    ) as WaDropdownItem[];
  }

  /** Prevents click events from firing on the host when the item is disabled (e.g. programmatic .click() calls). */
  private handleHostClick = (event: MouseEvent) => {
    if (this.disabled) {
      event.preventDefault();
      event.stopImmediatePropagation();
    }
  };

  /** Prevents click events from firing when the item is disabled. */
  private handleClick = (event: MouseEvent) => {
    if (this.disabled) {
      event.preventDefault();
      event.stopImmediatePropagation();
    }
  };

  /** Handles mouse enter to open the submenu */
  private handleMouseEnter() {
    if (this.hasSubmenu && !this.disabled) {
      this.notifyParentOfOpening();
      this.submenuOpen = true;
    }
  }

  render() {
    return html`
      ${
        this.type === "checkbox"
          ? html`
              <wa-icon
                id="check"
                part="checkmark"
                exportparts="svg:checkmark__svg"
                library="system"
                name="check"
              ></wa-icon>
            `
          : ""
      }

      <span id="icon" part="icon">
        <slot name="icon"></slot>
      </span>

      <span id="label" part="label">
        <slot></slot>
      </span>

      <span id="details" part="details">
        <slot name="details"></slot>
      </span>

      ${
        this.hasSubmenu
          ? html`
              <wa-icon
                id="submenu-indicator"
                part="submenu-icon"
                exportparts="svg:submenu-icon__svg"
                library="system"
                name="chevron-right"
              ></wa-icon>
            `
          : ""
      }
      ${
        this.hasSubmenu
          ? html`
              <div
                id="submenu"
                part="submenu"
                popover="manual"
                role="menu"
                tabindex="-1"
                aria-orientation="vertical"
                hidden
              >
                <slot name="submenu"></slot>
              </div>
            `
          : ""
      }
    `;
  }
}

declare global {
  interface HTMLElementTagNameMap {
    "wa-dropdown-item": WaDropdownItem;
  }
}

`````
