# externals/webawesome/dropdown/dropdown.ts

## Option 1

`````json
{"printWidth":80}
`````

### Diff

`````diff
===================================================================
--- prettier
+++ oxfmt
@@ -80,9 +80,16 @@
   @property({ type: Boolean, reflect: true }) open = false;
 
   /** The dropdown's size. */
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
// dropdown.ts
import {
  autoUpdate,
  computePosition,
  flip,
  offset,
  shift,
} from "@floating-ui/dom";
import type { PropertyValues } from "lit";
import { html } from "lit";
import { customElement, property, query } from "lit/decorators.js";
import { WaAfterHideEvent } from "../../events/after-hide.js";
import { WaAfterShowEvent } from "../../events/after-show.js";
import { WaHideEvent } from "../../events/hide.js";
import { WaSelectEvent } from "../../events/select.js";
import { WaShowEvent } from "../../events/show.js";
import { activeElements } from "../../internal/active-elements.js";
import { animateWithClass } from "../../internal/animate.js";
import {
  isTopDismissible,
  registerDismissible,
  unregisterDismissible,
} from "../../internal/dismissible-stack.js";
import { uniqueId } from "../../internal/math.js";
import { warnDeprecatedSize } from "../../internal/size.js";
import { watch } from "../../internal/watch.js";
import WebAwesomeElement from "../../internal/webawesome-element.js";
import sizeStyles from "../../styles/component/size.styles.js";
import { LocalizeController } from "../../utilities/localize.js";
import type WaButton from "../button/button.js";
import "../dropdown-item/dropdown-item.js";
import type WaDropdownItem from "../dropdown-item/dropdown-item.js";
import "../popup/popup.js";
import type WaPopup from "../popup/popup.js";
import styles from "./dropdown.styles.js";

const openDropdowns = new Set<WaDropdown>();

/**
 * @summary Dropdowns display a list of options triggered by a button or other element. They support keyboard
 *  navigation, submenus, and checkable items for building menus and context actions.
 * @documentation https://webawesome.com/docs/components/dropdown
 * @status stable
 * @since 2.0
 *
 * @dependency wa-dropdown-item
 * @dependency wa-popup
 *
 * @event wa-show - Emitted when the dropdown is about to show.
 * @event wa-after-show - Emitted after the dropdown has been shown.
 * @event wa-hide - Emitted when the dropdown is about to hide.
 * @event wa-after-hide - Emitted after the dropdown has been hidden.
 * @event wa-select - Emitted when an item in the dropdown is selected.
 *
 * @slot - The dropdown's items, typically `<wa-dropdown-item>` elements.
 * @slot trigger - The element that triggers the dropdown, such as a `<wa-button>` or `<button>`.
 *
 * @csspart base - The component's host element.
 * @csspart menu - The dropdown menu container.
 *
 * @cssproperty --show-duration - The duration of the show animation.
 * @cssproperty --hide-duration - The duration of the hide animation.
 */
@customElement("wa-dropdown")
export default class WaDropdown extends WebAwesomeElement {
  static css = [sizeStyles, styles];

  private submenuCleanups: Map<WaDropdownItem, ReturnType<typeof autoUpdate>> =
    new Map();
  private readonly localize = new LocalizeController(this);
  private userTypedQuery = "";
  private userTypedTimeout: ReturnType<typeof setTimeout>;
  private openSubmenuStack: WaDropdownItem[] = [];

  @query("slot:not([name])") defaultSlot: HTMLSlotElement;
  @query("#menu") private menu: HTMLDivElement;
  @query("wa-popup") private popup: WaPopup;

  /** Opens or closes the dropdown. */
  @property({ type: Boolean, reflect: true }) open = false;

  /** The dropdown's size. */
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
   * The placement of the dropdown menu in reference to the trigger. The menu will shift to a more optimal location if
   * the preferred placement doesn't have enough room.
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

  /** The distance of the dropdown menu from its trigger. */
  @property({ type: Number }) distance = 0;

  /** The offset of the dropdown menu along its trigger. */
  @property({ type: Number }) skidding = 0;

  disconnectedCallback() {
    super.disconnectedCallback();
    clearInterval(this.userTypedTimeout);
    this.closeAllSubmenus();

    // Clean up all submenu positioning
    this.submenuCleanups.forEach((cleanup) => cleanup());
    this.submenuCleanups.clear();

    document.removeEventListener("mousemove", this.handleGlobalMouseMove);
    document.removeEventListener("keydown", this.handleDocumentKeyDown);
    document.removeEventListener("pointerdown", this.handleDocumentPointerDown);
    unregisterDismissible(this);
  }

  firstUpdated() {
    this.syncAriaAttributes();
  }

  async updated(changedProperties: PropertyValues) {
    if (changedProperties.has("open")) {
      const previousOpen = changedProperties.get("open");
      // check if the previous value is the same
      // (if they are, do not trigger menu showing / hiding)
      if (previousOpen === this.open) {
        return;
      }
      // check if we are changing from undefined to false
      // (if we are, we can skip menu hiding)
      if (previousOpen === undefined && this.open === false) {
        return;
      }

      this.customStates.set("open", this.open);

      if (this.open) {
        await this.showMenu();
      } else {
        this.closeAllSubmenus();
        await this.hideMenu();
      }
    }

    if (changedProperties.has("size")) {
      this.syncItemSizes();
    }
  }

  /** Gets all dropdown items slotted in the menu. */
  private getItems(includeDisabled = false): WaDropdownItem[] {
    const items = (
      this.defaultSlot?.assignedElements({ flatten: true }) ?? []
    ).filter((el) => el.localName === "wa-dropdown-item") as WaDropdownItem[];

    return includeDisabled ? items : items.filter((item) => !item.disabled);
  }

  /** Gets all dropdown items in a specific submenu. */
  private getSubmenuItems(
    parentItem: WaDropdownItem,
    includeDisabled = false,
  ): WaDropdownItem[] {
    // Find the submenu slot within the parent item
    const submenuSlot =
      parentItem.shadowRoot?.querySelector<HTMLSlotElement>(
        'slot[name="submenu"]',
      ) || parentItem.querySelector<HTMLSlotElement>('slot[name="submenu"]');
    if (!submenuSlot) {
      return [];
    }

    // Get the items from the submenu slot
    const items = submenuSlot
      .assignedElements({ flatten: true })
      .filter((el) => el.localName === "wa-dropdown-item") as WaDropdownItem[];

    return includeDisabled ? items : items.filter((item) => !item.disabled);
  }

  /** Syncs item sizes with the dropdown's size property. */
  private syncItemSizes() {
    const items = (
      this.defaultSlot?.assignedElements({ flatten: true }) ?? []
    ).filter((el) => el.localName === "wa-dropdown-item") as WaDropdownItem[];
    items.forEach((item) => (item.size = this.size));
  }

  /** Handles the submenu navigation stack */
  private addToSubmenuStack(item: WaDropdownItem) {
    const index = this.openSubmenuStack.indexOf(item);
    if (index !== -1) {
      this.openSubmenuStack = this.openSubmenuStack.slice(0, index + 1);
    } else {
      this.openSubmenuStack.push(item);
    }
  }

  /** Removes the last item from the submenu stack */
  private removeFromSubmenuStack() {
    return this.openSubmenuStack.pop();
  }

  /** Gets the current active submenu item */
  private getCurrentSubmenuItem(): WaDropdownItem | undefined {
    return this.openSubmenuStack.length > 0
      ? this.openSubmenuStack[this.openSubmenuStack.length - 1]
      : undefined;
  }

  /** Closes all submenus in the dropdown. */
  private closeAllSubmenus() {
    const items = this.getItems(true);
    items.forEach((item) => {
      item.submenuOpen = false;
    });
    this.openSubmenuStack = [];
  }

  /** Closes sibling submenus at the same level as the specified item. */
  private closeSiblingSubmenus(item: WaDropdownItem) {
    const parentDropdownItem = item.closest<WaDropdownItem>(
      'wa-dropdown-item:not([slot="submenu"])',
    );
    let siblingItems: WaDropdownItem[];

    if (parentDropdownItem) {
      siblingItems = this.getSubmenuItems(parentDropdownItem, true);
    } else {
      siblingItems = this.getItems(true);
    }

    siblingItems.forEach((siblingItem) => {
      if (siblingItem !== item && siblingItem.submenuOpen) {
        siblingItem.submenuOpen = false;
      }
    });

    if (!this.openSubmenuStack.includes(item)) {
      this.openSubmenuStack.push(item);
    }
  }

  /** Get the slotted trigger button, a <wa-button> or <button> element */
  private getTrigger(): HTMLButtonElement | WaButton | null {
    return this.querySelector<WaButton | HTMLButtonElement>('[slot="trigger"]');
  }

  /** Shows the dropdown menu. This should only be called from within updated(). */
  private async showMenu() {
    const anchor = this.getTrigger();
    if (!anchor || !this.popup || !this.menu) return;

    const showEvent = new WaShowEvent();
    this.dispatchEvent(showEvent);
    if (showEvent.defaultPrevented) {
      this.open = false;
      return;
    }

    // if this dropdown is already open, do nothing
    // (this can happen when wa-hide was cancelled)
    if (this.popup.active) {
      return;
    }

    openDropdowns.forEach((dropdown) => (dropdown.open = false));

    this.popup.active = true; // Use wa-popup's active property instead of showPopover
    this.open = true;
    openDropdowns.add(this);
    registerDismissible(this);
    this.syncAriaAttributes();
    document.addEventListener("keydown", this.handleDocumentKeyDown);
    document.addEventListener("pointerdown", this.handleDocumentPointerDown);
    document.addEventListener("mousemove", this.handleGlobalMouseMove);

    // In case its still trying to hide, remove the class to cancel the hide animation.
    this.menu.classList.remove("hide");
    await animateWithClass(this.menu, "show"); // Animate the menu div

    const items = this.getItems();
    if (items.length > 0) {
      items.forEach((item, index) => (item.active = index === 0));
      items[0].focus({ preventScroll: true });
    }

    this.dispatchEvent(new WaAfterShowEvent());
  }

  /** Hides the dropdown menu. This should only be called from within updated(). */
  private async hideMenu() {
    if (!this.popup || !this.menu) return;

    const hideEvent = new WaHideEvent({ source: this });
    this.dispatchEvent(hideEvent);
    if (hideEvent.defaultPrevented) {
      this.open = true;
      return;
    }

    this.open = false;
    openDropdowns.delete(this);
    unregisterDismissible(this);
    this.syncAriaAttributes();
    document.removeEventListener("keydown", this.handleDocumentKeyDown);
    document.removeEventListener("pointerdown", this.handleDocumentPointerDown);
    document.removeEventListener("mousemove", this.handleGlobalMouseMove);

    this.menu.classList.remove("show");
    await animateWithClass(this.menu, "hide"); // Animate before hiding

    // Sometimes this ends up out of sync. So make sure it aligns with `open`
    this.popup.active = this.open; // Hide using wa-popup
    this.dispatchEvent(new WaAfterHideEvent());
  }

  /** Handles key down events when the menu is open */
  private handleDocumentKeyDown = async (event: KeyboardEvent) => {
    const isRtl = this.localize.dir() === "rtl";

    if (event.key === "Escape" && this.open && isTopDismissible(this)) {
      const trigger = this.getTrigger();

      event.preventDefault();
      event.stopPropagation();

      this.open = false;
      trigger?.focus({ preventScroll: true });
      return;
    }

    const activeElement = [...activeElements()].find(
      (el) => el.localName === "wa-dropdown-item",
    );
    const isFocusedOnItem = activeElement?.localName === "wa-dropdown-item";
    const currentSubmenuItem = this.getCurrentSubmenuItem();
    const isInSubmenu = !!currentSubmenuItem;

    let items: WaDropdownItem[];
    let activeItem: WaDropdownItem | undefined;
    let activeItemIndex: number;

    if (isInSubmenu) {
      items = this.getSubmenuItems(currentSubmenuItem);
      activeItem = items.find((item) => item.active || item === activeElement);
      activeItemIndex = activeItem ? items.indexOf(activeItem) : -1;
    } else {
      items = this.getItems();
      activeItem = items.find((item) => item.active || item === activeElement);
      activeItemIndex = activeItem ? items.indexOf(activeItem) : -1;
    }

    let itemToSelect: WaDropdownItem | undefined;

    if (event.key === "ArrowUp") {
      event.preventDefault();
      event.stopPropagation();
      if (activeItemIndex > 0) {
        itemToSelect = items[activeItemIndex - 1];
      } else {
        itemToSelect = items[items.length - 1];
      }
    }

    if (event.key === "ArrowDown") {
      event.preventDefault();
      event.stopPropagation();
      if (activeItemIndex !== -1 && activeItemIndex < items.length - 1) {
        itemToSelect = items[activeItemIndex + 1];
      } else {
        itemToSelect = items[0];
      }
    }

    if (
      event.key === (isRtl ? "ArrowLeft" : "ArrowRight") &&
      isFocusedOnItem &&
      activeItem
    ) {
      if (activeItem.hasSubmenu) {
        event.preventDefault();
        event.stopPropagation();

        activeItem.submenuOpen = true;
        this.addToSubmenuStack(activeItem);

        setTimeout(() => {
          const submenuItems = this.getSubmenuItems(activeItem!);
          if (submenuItems.length > 0) {
            submenuItems.forEach((item, index) => (item.active = index === 0));
            submenuItems[0].focus({ preventScroll: true });
          }
        }, 0);

        return;
      }
    }

    if (event.key === (isRtl ? "ArrowRight" : "ArrowLeft") && isInSubmenu) {
      event.preventDefault();
      event.stopPropagation();

      const removedItem = this.removeFromSubmenuStack();
      if (removedItem) {
        removedItem.submenuOpen = false;

        setTimeout(() => {
          removedItem.focus({ preventScroll: true });
          removedItem.active = true;

          const parentItems =
            removedItem.slot === "submenu"
              ? this.getSubmenuItems(
                  removedItem.parentElement as WaDropdownItem,
                )
              : this.getItems();

          parentItems.forEach((item) => {
            if (item !== removedItem) {
              item.active = false;
            }
          });
        }, 0);
      }

      return;
    }

    if (event.key === "Home" || event.key === "End") {
      event.preventDefault();
      event.stopPropagation();
      itemToSelect = event.key === "Home" ? items[0] : items[items.length - 1];
    }

    if (event.key === "Tab") {
      await this.hideMenu();
    }

    if (
      event.key.length === 1 &&
      !(event.metaKey || event.ctrlKey || event.altKey) &&
      !(event.key === " " && this.userTypedQuery === "")
    ) {
      clearTimeout(this.userTypedTimeout);
      this.userTypedTimeout = setTimeout(() => {
        this.userTypedQuery = "";
      }, 1000);

      this.userTypedQuery += event.key;

      items.some((item) => {
        const label = (item.textContent || "").trim().toLowerCase();
        const selectionQuery = this.userTypedQuery.trim().toLowerCase();

        if (label.startsWith(selectionQuery)) {
          itemToSelect = item;
          return true;
        }

        return false;
      });
    }

    if (itemToSelect) {
      event.preventDefault();
      event.stopPropagation();
      items.forEach((item) => (item.active = item === itemToSelect));
      itemToSelect.focus({ preventScroll: true });
      return;
    }

    if (
      (event.key === "Enter" ||
        (event.key === " " && this.userTypedQuery === "")) &&
      isFocusedOnItem &&
      activeItem
    ) {
      event.preventDefault();
      event.stopPropagation();

      if (activeItem.hasSubmenu) {
        activeItem.submenuOpen = true;
        this.addToSubmenuStack(activeItem);

        setTimeout(() => {
          const submenuItems = this.getSubmenuItems(activeItem!);
          if (submenuItems.length > 0) {
            submenuItems.forEach((item, index) => (item.active = index === 0));
            submenuItems[0].focus({ preventScroll: true });
          }
        }, 0);
      } else {
        this.makeSelection(activeItem);
      }
    }
  };

  /** Handles pointer down events when the dropdown is open. */
  private handleDocumentPointerDown = (event: PointerEvent) => {
    const path = event.composedPath();
    const isInDropdownHierarchy = path.some((el) => {
      if (el instanceof HTMLElement) {
        return el === this || el.closest('wa-dropdown, [part="submenu"]');
      }
      return false;
    });

    if (!isInDropdownHierarchy) {
      this.open = false;
    }
  };

  /** Handles clicks on the menu. */
  private handleMenuClick(event: MouseEvent) {
    const item = (event.target as Element).closest("wa-dropdown-item");

    if (!item || item.disabled) return;

    if (item.hasSubmenu) {
      if (!item.submenuOpen) {
        this.closeSiblingSubmenus(item);
        this.addToSubmenuStack(item);
        item.submenuOpen = true;
      }

      event.stopPropagation();
      return;
    }

    this.makeSelection(item);
  }

  /** Prepares dropdown items when they get added or removed */
  private async handleMenuSlotChange() {
    const items = this.getItems(true);
    await Promise.all(items.map((item) => item.updateComplete));

    this.syncItemSizes();

    const hasCheckbox = items.some((item) => item.type === "checkbox");
    const hasSubmenu = items.some((item) => item.hasSubmenu);

    items.forEach((item, index) => {
      item.active = index === 0;
      item.checkboxAdjacent = hasCheckbox;
      item.submenuAdjacent = hasSubmenu;
    });
  }

  /** Toggles the dropdown menu */
  private handleTriggerClick() {
    this.open = !this.open;
  }

  /** Handles submenu opening events */
  private handleSubmenuOpening(event: CustomEvent) {
    const openingItem = event.detail.item as WaDropdownItem;
    this.closeSiblingSubmenus(openingItem);
    this.addToSubmenuStack(openingItem);

    this.setupSubmenuPosition(openingItem);
    this.processSubmenuItems(openingItem);
  }

  /** Sets up submenu positioning with autoUpdate */
  private setupSubmenuPosition(item: WaDropdownItem) {
    if (!item.submenuElement) return;

    this.cleanupSubmenuPosition(item);

    const cleanup = autoUpdate(item, item.submenuElement, () => {
      this.positionSubmenu(item);
      this.updateSafeTriangleCoordinates(item);
    });

    this.submenuCleanups.set(item, cleanup);

    const submenuSlot = item.submenuElement.querySelector(
      'slot[name="submenu"]',
    );
    if (submenuSlot) {
      submenuSlot.removeEventListener(
        "slotchange",
        WaDropdown.handleSubmenuSlotChange,
      );
      submenuSlot.addEventListener(
        "slotchange",
        WaDropdown.handleSubmenuSlotChange,
      );
      WaDropdown.handleSubmenuSlotChange({
        target: submenuSlot,
      } as unknown as Event);
    }
  }

  private static handleSubmenuSlotChange(event: Event) {
    const slot = event.target as HTMLSlotElement;
    if (!slot) return;

    const items = slot
      .assignedElements()
      .filter((el) => el.localName === "wa-dropdown-item") as WaDropdownItem[];

    if (items.length === 0) return;

    const hasSubmenuItems = items.some((item) => item.hasSubmenu);
    const hasCheckboxItems = items.some((item) => item.type === "checkbox");

    items.forEach((item) => {
      item.submenuAdjacent = hasSubmenuItems;
      item.checkboxAdjacent = hasCheckboxItems;
    });
  }

  private processSubmenuItems(item: WaDropdownItem) {
    if (!item.submenuElement) return;

    const submenuItems = this.getSubmenuItems(item, true);
    const hasSubmenuItems = submenuItems.some((subItem) => subItem.hasSubmenu);

    submenuItems.forEach((subItem) => {
      subItem.submenuAdjacent = hasSubmenuItems;
    });
  }

  /** Cleans up submenu positioning */
  private cleanupSubmenuPosition(item: WaDropdownItem) {
    const cleanup = this.submenuCleanups.get(item);
    if (cleanup) {
      cleanup();
      this.submenuCleanups.delete(item);
    }
  }

  /** Positions a submenu relative to its parent item */
  private positionSubmenu(item: WaDropdownItem) {
    if (!item.submenuElement) return;

    const isRtl = this.localize.dir() === "rtl";
    const placement = isRtl ? "left-start" : "right-start";

    computePosition(item, item.submenuElement, {
      placement: placement,
      middleware: [
        offset({
          mainAxis: 0,
          crossAxis: -5,
        }),
        flip({
          fallbackStrategy: "bestFit",
        }),
        shift({
          padding: 8,
        }),
      ],
    }).then(({ x, y, placement }) => {
      item.submenuElement.setAttribute("data-placement", placement);

      Object.assign(item.submenuElement.style, {
        left: `${x}px`,
        top: `${y}px`,
      });
    });
  }

  /** Updates the safe triangle coordinates for a submenu */
  private updateSafeTriangleCoordinates(item: WaDropdownItem) {
    if (!item.submenuElement || !item.submenuOpen) return;

    const isKeyboardNavigation =
      document.activeElement?.matches(":focus-visible");

    if (isKeyboardNavigation) {
      item.submenuElement.style.setProperty("--safe-triangle-visible", "none");
      return;
    }

    item.submenuElement.style.setProperty("--safe-triangle-visible", "block");

    const submenuRect = item.submenuElement.getBoundingClientRect();
    const isRtl = this.localize.dir() === "rtl";

    item.submenuElement.style.setProperty(
      "--safe-triangle-submenu-start-x",
      `${isRtl ? submenuRect.right : submenuRect.left}px`,
    );
    item.submenuElement.style.setProperty(
      "--safe-triangle-submenu-start-y",
      `${submenuRect.top}px`,
    );
    item.submenuElement.style.setProperty(
      "--safe-triangle-submenu-end-x",
      `${isRtl ? submenuRect.right : submenuRect.left}px`,
    );
    item.submenuElement.style.setProperty(
      "--safe-triangle-submenu-end-y",
      `${submenuRect.bottom}px`,
    );
  }

  /** Handle global mouse movement for safe triangle logic */
  private handleGlobalMouseMove = (event: MouseEvent) => {
    const currentSubmenuItem = this.getCurrentSubmenuItem();
    if (!currentSubmenuItem?.submenuOpen || !currentSubmenuItem.submenuElement)
      return;

    const submenuRect =
      currentSubmenuItem.submenuElement.getBoundingClientRect();
    const isRtl = this.localize.dir() === "rtl";
    const submenuEdgeX = isRtl ? submenuRect.right : submenuRect.left;

    const constrainedX = isRtl
      ? Math.max(event.clientX, submenuEdgeX)
      : Math.min(event.clientX, submenuEdgeX);
    const constrainedY = Math.max(
      submenuRect.top,
      Math.min(event.clientY, submenuRect.bottom),
    );

    currentSubmenuItem.submenuElement.style.setProperty(
      "--safe-triangle-cursor-x",
      `${constrainedX}px`,
    );
    currentSubmenuItem.submenuElement.style.setProperty(
      "--safe-triangle-cursor-y",
      `${constrainedY}px`,
    );

    // Calculate these up front since this event cant fire a lot.
    const composedPath = event.composedPath();
    const submenuItemHovered = currentSubmenuItem.matches(":hover");
    const submenuElementHovered = Boolean(
      currentSubmenuItem.submenuElement?.matches(":hover"),
    );

    const isOverItem =
      submenuItemHovered ||
      !!composedPath.find((el) => el === currentSubmenuItem);

    const isOverSubmenu =
      submenuElementHovered ||
      !!composedPath.find(
        (el) =>
          el instanceof HTMLElement &&
          el.closest('[part="submenu"]') === currentSubmenuItem.submenuElement,
      );

    if (!isOverItem && !isOverSubmenu) {
      setTimeout(() => {
        if (!submenuItemHovered && !submenuElementHovered) {
          currentSubmenuItem.submenuOpen = false;
        }
      }, 100);
    }
  };

  /** Makes a selection, emits the wa-select event, and closes the dropdown. */
  private makeSelection(item: WaDropdownItem) {
    const trigger = this.getTrigger();

    if (item.disabled) {
      return;
    }

    if (item.type === "checkbox") {
      item.checked = !item.checked;
    }

    const selectEvent = new WaSelectEvent({ item });
    this.dispatchEvent(selectEvent);

    if (!selectEvent.defaultPrevented) {
      this.open = false;
      trigger?.focus({ preventScroll: true });
    }
  }

  /** Syncs aria attributes on the slotted trigger element and the menu based on the dropdown's current state */
  private async syncAriaAttributes() {
    const trigger = this.getTrigger();
    let nativeButton: HTMLButtonElement | undefined;

    if (!trigger) {
      return;
    }

    if (trigger.localName === "wa-button") {
      await customElements.whenDefined("wa-button");
      await (trigger as WaButton).updateComplete;
      nativeButton =
        trigger.shadowRoot!.querySelector<HTMLButtonElement>('[part="base"]')!;
    } else {
      nativeButton = trigger as HTMLButtonElement;
    }

    if (!nativeButton.hasAttribute("id")) {
      nativeButton.setAttribute("id", uniqueId("wa-dropdown-trigger-"));
    }

    nativeButton.setAttribute("aria-haspopup", "menu");
    nativeButton.setAttribute("aria-expanded", this.open ? "true" : "false");

    this.menu?.setAttribute("aria-expanded", "false");
  }

  render() {
    // On initial render, we want to use this.open, for everything else, we sync off of this.popup.active to get animations working.
    let active = this.hasUpdated ? this.popup?.active : this.open;

    return html`
      <wa-popup
        placement=${this.placement}
        distance=${this.distance}
        skidding=${this.skidding}
        ?active=${active}
        flip
        flip-fallback-strategy="best-fit"
        shift
        shift-padding="10"
        auto-size="vertical"
        auto-size-padding="10"
      >
        <slot
          name="trigger"
          slot="anchor"
          @click=${this.handleTriggerClick}
          @slotchange=${this.syncAriaAttributes}
        ></slot>
        <div
          id="menu"
          part="menu"
          role="menu"
          tabindex="-1"
          aria-orientation="vertical"
          @click=${this.handleMenuClick}
          @submenu-opening=${this.handleSubmenuOpening}
        >
          <slot @slotchange=${this.handleMenuSlotChange}></slot>
        </div>
      </wa-popup>
    `;
  }
}

declare global {
  interface HTMLElementTagNameMap {
    "wa-dropdown": WaDropdown;
  }
}

`````

### Expected (prettier)

`````ts
// dropdown.ts
import {
  autoUpdate,
  computePosition,
  flip,
  offset,
  shift,
} from "@floating-ui/dom";
import type { PropertyValues } from "lit";
import { html } from "lit";
import { customElement, property, query } from "lit/decorators.js";
import { WaAfterHideEvent } from "../../events/after-hide.js";
import { WaAfterShowEvent } from "../../events/after-show.js";
import { WaHideEvent } from "../../events/hide.js";
import { WaSelectEvent } from "../../events/select.js";
import { WaShowEvent } from "../../events/show.js";
import { activeElements } from "../../internal/active-elements.js";
import { animateWithClass } from "../../internal/animate.js";
import {
  isTopDismissible,
  registerDismissible,
  unregisterDismissible,
} from "../../internal/dismissible-stack.js";
import { uniqueId } from "../../internal/math.js";
import { warnDeprecatedSize } from "../../internal/size.js";
import { watch } from "../../internal/watch.js";
import WebAwesomeElement from "../../internal/webawesome-element.js";
import sizeStyles from "../../styles/component/size.styles.js";
import { LocalizeController } from "../../utilities/localize.js";
import type WaButton from "../button/button.js";
import "../dropdown-item/dropdown-item.js";
import type WaDropdownItem from "../dropdown-item/dropdown-item.js";
import "../popup/popup.js";
import type WaPopup from "../popup/popup.js";
import styles from "./dropdown.styles.js";

const openDropdowns = new Set<WaDropdown>();

/**
 * @summary Dropdowns display a list of options triggered by a button or other element. They support keyboard
 *  navigation, submenus, and checkable items for building menus and context actions.
 * @documentation https://webawesome.com/docs/components/dropdown
 * @status stable
 * @since 2.0
 *
 * @dependency wa-dropdown-item
 * @dependency wa-popup
 *
 * @event wa-show - Emitted when the dropdown is about to show.
 * @event wa-after-show - Emitted after the dropdown has been shown.
 * @event wa-hide - Emitted when the dropdown is about to hide.
 * @event wa-after-hide - Emitted after the dropdown has been hidden.
 * @event wa-select - Emitted when an item in the dropdown is selected.
 *
 * @slot - The dropdown's items, typically `<wa-dropdown-item>` elements.
 * @slot trigger - The element that triggers the dropdown, such as a `<wa-button>` or `<button>`.
 *
 * @csspart base - The component's host element.
 * @csspart menu - The dropdown menu container.
 *
 * @cssproperty --show-duration - The duration of the show animation.
 * @cssproperty --hide-duration - The duration of the hide animation.
 */
@customElement("wa-dropdown")
export default class WaDropdown extends WebAwesomeElement {
  static css = [sizeStyles, styles];

  private submenuCleanups: Map<WaDropdownItem, ReturnType<typeof autoUpdate>> =
    new Map();
  private readonly localize = new LocalizeController(this);
  private userTypedQuery = "";
  private userTypedTimeout: ReturnType<typeof setTimeout>;
  private openSubmenuStack: WaDropdownItem[] = [];

  @query("slot:not([name])") defaultSlot: HTMLSlotElement;
  @query("#menu") private menu: HTMLDivElement;
  @query("wa-popup") private popup: WaPopup;

  /** Opens or closes the dropdown. */
  @property({ type: Boolean, reflect: true }) open = false;

  /** The dropdown's size. */
  @property({ reflect: true }) size:
    "xs" | "s" | "m" | "l" | "xl" | "small" | "medium" | "large" = "m";

  @watch("size")
  handleSizeChange() {
    warnDeprecatedSize(this.localName, this.size);
  }

  /**
   * The placement of the dropdown menu in reference to the trigger. The menu will shift to a more optimal location if
   * the preferred placement doesn't have enough room.
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

  /** The distance of the dropdown menu from its trigger. */
  @property({ type: Number }) distance = 0;

  /** The offset of the dropdown menu along its trigger. */
  @property({ type: Number }) skidding = 0;

  disconnectedCallback() {
    super.disconnectedCallback();
    clearInterval(this.userTypedTimeout);
    this.closeAllSubmenus();

    // Clean up all submenu positioning
    this.submenuCleanups.forEach((cleanup) => cleanup());
    this.submenuCleanups.clear();

    document.removeEventListener("mousemove", this.handleGlobalMouseMove);
    document.removeEventListener("keydown", this.handleDocumentKeyDown);
    document.removeEventListener("pointerdown", this.handleDocumentPointerDown);
    unregisterDismissible(this);
  }

  firstUpdated() {
    this.syncAriaAttributes();
  }

  async updated(changedProperties: PropertyValues) {
    if (changedProperties.has("open")) {
      const previousOpen = changedProperties.get("open");
      // check if the previous value is the same
      // (if they are, do not trigger menu showing / hiding)
      if (previousOpen === this.open) {
        return;
      }
      // check if we are changing from undefined to false
      // (if we are, we can skip menu hiding)
      if (previousOpen === undefined && this.open === false) {
        return;
      }

      this.customStates.set("open", this.open);

      if (this.open) {
        await this.showMenu();
      } else {
        this.closeAllSubmenus();
        await this.hideMenu();
      }
    }

    if (changedProperties.has("size")) {
      this.syncItemSizes();
    }
  }

  /** Gets all dropdown items slotted in the menu. */
  private getItems(includeDisabled = false): WaDropdownItem[] {
    const items = (
      this.defaultSlot?.assignedElements({ flatten: true }) ?? []
    ).filter((el) => el.localName === "wa-dropdown-item") as WaDropdownItem[];

    return includeDisabled ? items : items.filter((item) => !item.disabled);
  }

  /** Gets all dropdown items in a specific submenu. */
  private getSubmenuItems(
    parentItem: WaDropdownItem,
    includeDisabled = false,
  ): WaDropdownItem[] {
    // Find the submenu slot within the parent item
    const submenuSlot =
      parentItem.shadowRoot?.querySelector<HTMLSlotElement>(
        'slot[name="submenu"]',
      ) || parentItem.querySelector<HTMLSlotElement>('slot[name="submenu"]');
    if (!submenuSlot) {
      return [];
    }

    // Get the items from the submenu slot
    const items = submenuSlot
      .assignedElements({ flatten: true })
      .filter((el) => el.localName === "wa-dropdown-item") as WaDropdownItem[];

    return includeDisabled ? items : items.filter((item) => !item.disabled);
  }

  /** Syncs item sizes with the dropdown's size property. */
  private syncItemSizes() {
    const items = (
      this.defaultSlot?.assignedElements({ flatten: true }) ?? []
    ).filter((el) => el.localName === "wa-dropdown-item") as WaDropdownItem[];
    items.forEach((item) => (item.size = this.size));
  }

  /** Handles the submenu navigation stack */
  private addToSubmenuStack(item: WaDropdownItem) {
    const index = this.openSubmenuStack.indexOf(item);
    if (index !== -1) {
      this.openSubmenuStack = this.openSubmenuStack.slice(0, index + 1);
    } else {
      this.openSubmenuStack.push(item);
    }
  }

  /** Removes the last item from the submenu stack */
  private removeFromSubmenuStack() {
    return this.openSubmenuStack.pop();
  }

  /** Gets the current active submenu item */
  private getCurrentSubmenuItem(): WaDropdownItem | undefined {
    return this.openSubmenuStack.length > 0
      ? this.openSubmenuStack[this.openSubmenuStack.length - 1]
      : undefined;
  }

  /** Closes all submenus in the dropdown. */
  private closeAllSubmenus() {
    const items = this.getItems(true);
    items.forEach((item) => {
      item.submenuOpen = false;
    });
    this.openSubmenuStack = [];
  }

  /** Closes sibling submenus at the same level as the specified item. */
  private closeSiblingSubmenus(item: WaDropdownItem) {
    const parentDropdownItem = item.closest<WaDropdownItem>(
      'wa-dropdown-item:not([slot="submenu"])',
    );
    let siblingItems: WaDropdownItem[];

    if (parentDropdownItem) {
      siblingItems = this.getSubmenuItems(parentDropdownItem, true);
    } else {
      siblingItems = this.getItems(true);
    }

    siblingItems.forEach((siblingItem) => {
      if (siblingItem !== item && siblingItem.submenuOpen) {
        siblingItem.submenuOpen = false;
      }
    });

    if (!this.openSubmenuStack.includes(item)) {
      this.openSubmenuStack.push(item);
    }
  }

  /** Get the slotted trigger button, a <wa-button> or <button> element */
  private getTrigger(): HTMLButtonElement | WaButton | null {
    return this.querySelector<WaButton | HTMLButtonElement>('[slot="trigger"]');
  }

  /** Shows the dropdown menu. This should only be called from within updated(). */
  private async showMenu() {
    const anchor = this.getTrigger();
    if (!anchor || !this.popup || !this.menu) return;

    const showEvent = new WaShowEvent();
    this.dispatchEvent(showEvent);
    if (showEvent.defaultPrevented) {
      this.open = false;
      return;
    }

    // if this dropdown is already open, do nothing
    // (this can happen when wa-hide was cancelled)
    if (this.popup.active) {
      return;
    }

    openDropdowns.forEach((dropdown) => (dropdown.open = false));

    this.popup.active = true; // Use wa-popup's active property instead of showPopover
    this.open = true;
    openDropdowns.add(this);
    registerDismissible(this);
    this.syncAriaAttributes();
    document.addEventListener("keydown", this.handleDocumentKeyDown);
    document.addEventListener("pointerdown", this.handleDocumentPointerDown);
    document.addEventListener("mousemove", this.handleGlobalMouseMove);

    // In case its still trying to hide, remove the class to cancel the hide animation.
    this.menu.classList.remove("hide");
    await animateWithClass(this.menu, "show"); // Animate the menu div

    const items = this.getItems();
    if (items.length > 0) {
      items.forEach((item, index) => (item.active = index === 0));
      items[0].focus({ preventScroll: true });
    }

    this.dispatchEvent(new WaAfterShowEvent());
  }

  /** Hides the dropdown menu. This should only be called from within updated(). */
  private async hideMenu() {
    if (!this.popup || !this.menu) return;

    const hideEvent = new WaHideEvent({ source: this });
    this.dispatchEvent(hideEvent);
    if (hideEvent.defaultPrevented) {
      this.open = true;
      return;
    }

    this.open = false;
    openDropdowns.delete(this);
    unregisterDismissible(this);
    this.syncAriaAttributes();
    document.removeEventListener("keydown", this.handleDocumentKeyDown);
    document.removeEventListener("pointerdown", this.handleDocumentPointerDown);
    document.removeEventListener("mousemove", this.handleGlobalMouseMove);

    this.menu.classList.remove("show");
    await animateWithClass(this.menu, "hide"); // Animate before hiding

    // Sometimes this ends up out of sync. So make sure it aligns with `open`
    this.popup.active = this.open; // Hide using wa-popup
    this.dispatchEvent(new WaAfterHideEvent());
  }

  /** Handles key down events when the menu is open */
  private handleDocumentKeyDown = async (event: KeyboardEvent) => {
    const isRtl = this.localize.dir() === "rtl";

    if (event.key === "Escape" && this.open && isTopDismissible(this)) {
      const trigger = this.getTrigger();

      event.preventDefault();
      event.stopPropagation();

      this.open = false;
      trigger?.focus({ preventScroll: true });
      return;
    }

    const activeElement = [...activeElements()].find(
      (el) => el.localName === "wa-dropdown-item",
    );
    const isFocusedOnItem = activeElement?.localName === "wa-dropdown-item";
    const currentSubmenuItem = this.getCurrentSubmenuItem();
    const isInSubmenu = !!currentSubmenuItem;

    let items: WaDropdownItem[];
    let activeItem: WaDropdownItem | undefined;
    let activeItemIndex: number;

    if (isInSubmenu) {
      items = this.getSubmenuItems(currentSubmenuItem);
      activeItem = items.find((item) => item.active || item === activeElement);
      activeItemIndex = activeItem ? items.indexOf(activeItem) : -1;
    } else {
      items = this.getItems();
      activeItem = items.find((item) => item.active || item === activeElement);
      activeItemIndex = activeItem ? items.indexOf(activeItem) : -1;
    }

    let itemToSelect: WaDropdownItem | undefined;

    if (event.key === "ArrowUp") {
      event.preventDefault();
      event.stopPropagation();
      if (activeItemIndex > 0) {
        itemToSelect = items[activeItemIndex - 1];
      } else {
        itemToSelect = items[items.length - 1];
      }
    }

    if (event.key === "ArrowDown") {
      event.preventDefault();
      event.stopPropagation();
      if (activeItemIndex !== -1 && activeItemIndex < items.length - 1) {
        itemToSelect = items[activeItemIndex + 1];
      } else {
        itemToSelect = items[0];
      }
    }

    if (
      event.key === (isRtl ? "ArrowLeft" : "ArrowRight") &&
      isFocusedOnItem &&
      activeItem
    ) {
      if (activeItem.hasSubmenu) {
        event.preventDefault();
        event.stopPropagation();

        activeItem.submenuOpen = true;
        this.addToSubmenuStack(activeItem);

        setTimeout(() => {
          const submenuItems = this.getSubmenuItems(activeItem!);
          if (submenuItems.length > 0) {
            submenuItems.forEach((item, index) => (item.active = index === 0));
            submenuItems[0].focus({ preventScroll: true });
          }
        }, 0);

        return;
      }
    }

    if (event.key === (isRtl ? "ArrowRight" : "ArrowLeft") && isInSubmenu) {
      event.preventDefault();
      event.stopPropagation();

      const removedItem = this.removeFromSubmenuStack();
      if (removedItem) {
        removedItem.submenuOpen = false;

        setTimeout(() => {
          removedItem.focus({ preventScroll: true });
          removedItem.active = true;

          const parentItems =
            removedItem.slot === "submenu"
              ? this.getSubmenuItems(
                  removedItem.parentElement as WaDropdownItem,
                )
              : this.getItems();

          parentItems.forEach((item) => {
            if (item !== removedItem) {
              item.active = false;
            }
          });
        }, 0);
      }

      return;
    }

    if (event.key === "Home" || event.key === "End") {
      event.preventDefault();
      event.stopPropagation();
      itemToSelect = event.key === "Home" ? items[0] : items[items.length - 1];
    }

    if (event.key === "Tab") {
      await this.hideMenu();
    }

    if (
      event.key.length === 1 &&
      !(event.metaKey || event.ctrlKey || event.altKey) &&
      !(event.key === " " && this.userTypedQuery === "")
    ) {
      clearTimeout(this.userTypedTimeout);
      this.userTypedTimeout = setTimeout(() => {
        this.userTypedQuery = "";
      }, 1000);

      this.userTypedQuery += event.key;

      items.some((item) => {
        const label = (item.textContent || "").trim().toLowerCase();
        const selectionQuery = this.userTypedQuery.trim().toLowerCase();

        if (label.startsWith(selectionQuery)) {
          itemToSelect = item;
          return true;
        }

        return false;
      });
    }

    if (itemToSelect) {
      event.preventDefault();
      event.stopPropagation();
      items.forEach((item) => (item.active = item === itemToSelect));
      itemToSelect.focus({ preventScroll: true });
      return;
    }

    if (
      (event.key === "Enter" ||
        (event.key === " " && this.userTypedQuery === "")) &&
      isFocusedOnItem &&
      activeItem
    ) {
      event.preventDefault();
      event.stopPropagation();

      if (activeItem.hasSubmenu) {
        activeItem.submenuOpen = true;
        this.addToSubmenuStack(activeItem);

        setTimeout(() => {
          const submenuItems = this.getSubmenuItems(activeItem!);
          if (submenuItems.length > 0) {
            submenuItems.forEach((item, index) => (item.active = index === 0));
            submenuItems[0].focus({ preventScroll: true });
          }
        }, 0);
      } else {
        this.makeSelection(activeItem);
      }
    }
  };

  /** Handles pointer down events when the dropdown is open. */
  private handleDocumentPointerDown = (event: PointerEvent) => {
    const path = event.composedPath();
    const isInDropdownHierarchy = path.some((el) => {
      if (el instanceof HTMLElement) {
        return el === this || el.closest('wa-dropdown, [part="submenu"]');
      }
      return false;
    });

    if (!isInDropdownHierarchy) {
      this.open = false;
    }
  };

  /** Handles clicks on the menu. */
  private handleMenuClick(event: MouseEvent) {
    const item = (event.target as Element).closest("wa-dropdown-item");

    if (!item || item.disabled) return;

    if (item.hasSubmenu) {
      if (!item.submenuOpen) {
        this.closeSiblingSubmenus(item);
        this.addToSubmenuStack(item);
        item.submenuOpen = true;
      }

      event.stopPropagation();
      return;
    }

    this.makeSelection(item);
  }

  /** Prepares dropdown items when they get added or removed */
  private async handleMenuSlotChange() {
    const items = this.getItems(true);
    await Promise.all(items.map((item) => item.updateComplete));

    this.syncItemSizes();

    const hasCheckbox = items.some((item) => item.type === "checkbox");
    const hasSubmenu = items.some((item) => item.hasSubmenu);

    items.forEach((item, index) => {
      item.active = index === 0;
      item.checkboxAdjacent = hasCheckbox;
      item.submenuAdjacent = hasSubmenu;
    });
  }

  /** Toggles the dropdown menu */
  private handleTriggerClick() {
    this.open = !this.open;
  }

  /** Handles submenu opening events */
  private handleSubmenuOpening(event: CustomEvent) {
    const openingItem = event.detail.item as WaDropdownItem;
    this.closeSiblingSubmenus(openingItem);
    this.addToSubmenuStack(openingItem);

    this.setupSubmenuPosition(openingItem);
    this.processSubmenuItems(openingItem);
  }

  /** Sets up submenu positioning with autoUpdate */
  private setupSubmenuPosition(item: WaDropdownItem) {
    if (!item.submenuElement) return;

    this.cleanupSubmenuPosition(item);

    const cleanup = autoUpdate(item, item.submenuElement, () => {
      this.positionSubmenu(item);
      this.updateSafeTriangleCoordinates(item);
    });

    this.submenuCleanups.set(item, cleanup);

    const submenuSlot = item.submenuElement.querySelector(
      'slot[name="submenu"]',
    );
    if (submenuSlot) {
      submenuSlot.removeEventListener(
        "slotchange",
        WaDropdown.handleSubmenuSlotChange,
      );
      submenuSlot.addEventListener(
        "slotchange",
        WaDropdown.handleSubmenuSlotChange,
      );
      WaDropdown.handleSubmenuSlotChange({
        target: submenuSlot,
      } as unknown as Event);
    }
  }

  private static handleSubmenuSlotChange(event: Event) {
    const slot = event.target as HTMLSlotElement;
    if (!slot) return;

    const items = slot
      .assignedElements()
      .filter((el) => el.localName === "wa-dropdown-item") as WaDropdownItem[];

    if (items.length === 0) return;

    const hasSubmenuItems = items.some((item) => item.hasSubmenu);
    const hasCheckboxItems = items.some((item) => item.type === "checkbox");

    items.forEach((item) => {
      item.submenuAdjacent = hasSubmenuItems;
      item.checkboxAdjacent = hasCheckboxItems;
    });
  }

  private processSubmenuItems(item: WaDropdownItem) {
    if (!item.submenuElement) return;

    const submenuItems = this.getSubmenuItems(item, true);
    const hasSubmenuItems = submenuItems.some((subItem) => subItem.hasSubmenu);

    submenuItems.forEach((subItem) => {
      subItem.submenuAdjacent = hasSubmenuItems;
    });
  }

  /** Cleans up submenu positioning */
  private cleanupSubmenuPosition(item: WaDropdownItem) {
    const cleanup = this.submenuCleanups.get(item);
    if (cleanup) {
      cleanup();
      this.submenuCleanups.delete(item);
    }
  }

  /** Positions a submenu relative to its parent item */
  private positionSubmenu(item: WaDropdownItem) {
    if (!item.submenuElement) return;

    const isRtl = this.localize.dir() === "rtl";
    const placement = isRtl ? "left-start" : "right-start";

    computePosition(item, item.submenuElement, {
      placement: placement,
      middleware: [
        offset({
          mainAxis: 0,
          crossAxis: -5,
        }),
        flip({
          fallbackStrategy: "bestFit",
        }),
        shift({
          padding: 8,
        }),
      ],
    }).then(({ x, y, placement }) => {
      item.submenuElement.setAttribute("data-placement", placement);

      Object.assign(item.submenuElement.style, {
        left: `${x}px`,
        top: `${y}px`,
      });
    });
  }

  /** Updates the safe triangle coordinates for a submenu */
  private updateSafeTriangleCoordinates(item: WaDropdownItem) {
    if (!item.submenuElement || !item.submenuOpen) return;

    const isKeyboardNavigation =
      document.activeElement?.matches(":focus-visible");

    if (isKeyboardNavigation) {
      item.submenuElement.style.setProperty("--safe-triangle-visible", "none");
      return;
    }

    item.submenuElement.style.setProperty("--safe-triangle-visible", "block");

    const submenuRect = item.submenuElement.getBoundingClientRect();
    const isRtl = this.localize.dir() === "rtl";

    item.submenuElement.style.setProperty(
      "--safe-triangle-submenu-start-x",
      `${isRtl ? submenuRect.right : submenuRect.left}px`,
    );
    item.submenuElement.style.setProperty(
      "--safe-triangle-submenu-start-y",
      `${submenuRect.top}px`,
    );
    item.submenuElement.style.setProperty(
      "--safe-triangle-submenu-end-x",
      `${isRtl ? submenuRect.right : submenuRect.left}px`,
    );
    item.submenuElement.style.setProperty(
      "--safe-triangle-submenu-end-y",
      `${submenuRect.bottom}px`,
    );
  }

  /** Handle global mouse movement for safe triangle logic */
  private handleGlobalMouseMove = (event: MouseEvent) => {
    const currentSubmenuItem = this.getCurrentSubmenuItem();
    if (!currentSubmenuItem?.submenuOpen || !currentSubmenuItem.submenuElement)
      return;

    const submenuRect =
      currentSubmenuItem.submenuElement.getBoundingClientRect();
    const isRtl = this.localize.dir() === "rtl";
    const submenuEdgeX = isRtl ? submenuRect.right : submenuRect.left;

    const constrainedX = isRtl
      ? Math.max(event.clientX, submenuEdgeX)
      : Math.min(event.clientX, submenuEdgeX);
    const constrainedY = Math.max(
      submenuRect.top,
      Math.min(event.clientY, submenuRect.bottom),
    );

    currentSubmenuItem.submenuElement.style.setProperty(
      "--safe-triangle-cursor-x",
      `${constrainedX}px`,
    );
    currentSubmenuItem.submenuElement.style.setProperty(
      "--safe-triangle-cursor-y",
      `${constrainedY}px`,
    );

    // Calculate these up front since this event cant fire a lot.
    const composedPath = event.composedPath();
    const submenuItemHovered = currentSubmenuItem.matches(":hover");
    const submenuElementHovered = Boolean(
      currentSubmenuItem.submenuElement?.matches(":hover"),
    );

    const isOverItem =
      submenuItemHovered ||
      !!composedPath.find((el) => el === currentSubmenuItem);

    const isOverSubmenu =
      submenuElementHovered ||
      !!composedPath.find(
        (el) =>
          el instanceof HTMLElement &&
          el.closest('[part="submenu"]') === currentSubmenuItem.submenuElement,
      );

    if (!isOverItem && !isOverSubmenu) {
      setTimeout(() => {
        if (!submenuItemHovered && !submenuElementHovered) {
          currentSubmenuItem.submenuOpen = false;
        }
      }, 100);
    }
  };

  /** Makes a selection, emits the wa-select event, and closes the dropdown. */
  private makeSelection(item: WaDropdownItem) {
    const trigger = this.getTrigger();

    if (item.disabled) {
      return;
    }

    if (item.type === "checkbox") {
      item.checked = !item.checked;
    }

    const selectEvent = new WaSelectEvent({ item });
    this.dispatchEvent(selectEvent);

    if (!selectEvent.defaultPrevented) {
      this.open = false;
      trigger?.focus({ preventScroll: true });
    }
  }

  /** Syncs aria attributes on the slotted trigger element and the menu based on the dropdown's current state */
  private async syncAriaAttributes() {
    const trigger = this.getTrigger();
    let nativeButton: HTMLButtonElement | undefined;

    if (!trigger) {
      return;
    }

    if (trigger.localName === "wa-button") {
      await customElements.whenDefined("wa-button");
      await (trigger as WaButton).updateComplete;
      nativeButton =
        trigger.shadowRoot!.querySelector<HTMLButtonElement>('[part="base"]')!;
    } else {
      nativeButton = trigger as HTMLButtonElement;
    }

    if (!nativeButton.hasAttribute("id")) {
      nativeButton.setAttribute("id", uniqueId("wa-dropdown-trigger-"));
    }

    nativeButton.setAttribute("aria-haspopup", "menu");
    nativeButton.setAttribute("aria-expanded", this.open ? "true" : "false");

    this.menu?.setAttribute("aria-expanded", "false");
  }

  render() {
    // On initial render, we want to use this.open, for everything else, we sync off of this.popup.active to get animations working.
    let active = this.hasUpdated ? this.popup?.active : this.open;

    return html`
      <wa-popup
        placement=${this.placement}
        distance=${this.distance}
        skidding=${this.skidding}
        ?active=${active}
        flip
        flip-fallback-strategy="best-fit"
        shift
        shift-padding="10"
        auto-size="vertical"
        auto-size-padding="10"
      >
        <slot
          name="trigger"
          slot="anchor"
          @click=${this.handleTriggerClick}
          @slotchange=${this.syncAriaAttributes}
        ></slot>
        <div
          id="menu"
          part="menu"
          role="menu"
          tabindex="-1"
          aria-orientation="vertical"
          @click=${this.handleMenuClick}
          @submenu-opening=${this.handleSubmenuOpening}
        >
          <slot @slotchange=${this.handleMenuSlotChange}></slot>
        </div>
      </wa-popup>
    `;
  }
}

declare global {
  interface HTMLElementTagNameMap {
    "wa-dropdown": WaDropdown;
  }
}

`````
