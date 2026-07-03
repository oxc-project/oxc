# externals/webawesome/drawer/drawer.ts

## Option 1

`````json
{"printWidth":80}
`````

### Diff

`````diff
===================================================================
--- prettier
+++ oxfmt
@@ -270,51 +270,50 @@
         @cancel=${this.handleDialogCancel}
         @click=${this.handleDialogClick}
         @pointerdown=${this.handleDialogPointerDown}
       >
-        ${
-          hasHeader
-            ? html`
-                <header part="header" class="header">
-                  <h2 part="title" class="title" id="title">
-                    <!-- If there's no label, use an invisible character to prevent the header from collapsing -->
-                    <slot name="label">
-                      ${this.label.length > 0 ? this.label : String.fromCharCode(8203)}
-                    </slot>
-                  </h2>
-                  <div part="header-actions" class="header-actions">
-                    <slot name="header-actions"></slot>
-                    <wa-button
-                      part="close-button"
-                      exportparts="base:close-button__base"
-                      class="close"
-                      appearance="plain"
-                      @click="${(event: PointerEvent) => this.requestClose(event.target as Element)}"
-                    >
-                      <wa-icon
-                        name="xmark"
-                        label=${this.localize.term("close")}
-                        library="system"
-                        variant="solid"
-                      ></wa-icon>
-                    </wa-button>
-                  </div>
-                </header>
-              `
-            : ""
-        }
+        ${hasHeader
+          ? html`
+              <header part="header" class="header">
+                <h2 part="title" class="title" id="title">
+                  <!-- If there's no label, use an invisible character to prevent the header from collapsing -->
+                  <slot name="label">
+                    ${this.label.length > 0
+                      ? this.label
+                      : String.fromCharCode(8203)}
+                  </slot>
+                </h2>
+                <div part="header-actions" class="header-actions">
+                  <slot name="header-actions"></slot>
+                  <wa-button
+                    part="close-button"
+                    exportparts="base:close-button__base"
+                    class="close"
+                    appearance="plain"
+                    @click="${(event: PointerEvent) =>
+                      this.requestClose(event.target as Element)}"
+                  >
+                    <wa-icon
+                      name="xmark"
+                      label=${this.localize.term("close")}
+                      library="system"
+                      variant="solid"
+                    ></wa-icon>
+                  </wa-button>
+                </div>
+              </header>
+            `
+          : ""}
 
         <div part="body" class="body"><slot></slot></div>
 
-        ${
-          hasFooter
-            ? html`
-                <footer part="footer" class="footer">
-                  <slot name="footer"></slot>
-                </footer>
-              `
-            : ""
-        }
+        ${hasFooter
+          ? html`
+              <footer part="footer" class="footer">
+                <slot name="footer"></slot>
+              </footer>
+            `
+          : ""}
       </dialog>
     `;
   }
 }

`````

### Actual (oxfmt)

`````ts
import { html, isServer } from "lit";
import { customElement, property, query } from "lit/decorators.js";
import { classMap } from "lit/directives/class-map.js";
import { WaAfterHideEvent } from "../../events/after-hide.js";
import { WaAfterShowEvent } from "../../events/after-show.js";
import { WaHideEvent } from "../../events/hide.js";
import { WaShowEvent } from "../../events/show.js";
import { animateWithClass } from "../../internal/animate.js";
import {
  isTopDismissible,
  registerDismissible,
  unregisterDismissible,
} from "../../internal/dismissible-stack.js";
import { parseSpaceDelimitedTokens } from "../../internal/parse.js";
import {
  lockBodyScrolling,
  unlockBodyScrolling,
} from "../../internal/scroll.js";
import { HasSlotController } from "../../internal/slot.js";
import { watch } from "../../internal/watch.js";
import WebAwesomeElement from "../../internal/webawesome-element.js";
import { LocalizeController } from "../../utilities/localize.js";
import "../button/button.js";
import styles from "./drawer.styles.js";

/**
 * @summary Drawers slide in from the edge of a container to expose additional options and information without
 *  navigating away. Useful for navigation menus, filters, and secondary content.
 * @documentation https://webawesome.com/docs/components/drawer
 * @status stable
 * @since 2.0
 *
 * @dependency wa-button
 *
 * @slot - The drawer's main content.
 * @slot label - The drawer's label. Alternatively, you can use the `label` attribute.
 * @slot header-actions - Optional actions to add to the header. Works best with `<wa-button>`.
 * @slot footer - The drawer's footer, usually one or more buttons representing various options.
 *
 * @event wa-show - Emitted when the drawer opens.
 * @event wa-after-show - Emitted after the drawer opens and all animations are complete.
 * @event wa-hide - Emitted when the drawer closes.
 * @event wa-after-hide - Emitted after the drawer closes and all animations are complete.
 * @event {{ source: Element }} wa-hide - Emitted when the drawer is requesting to close. Calling
 *  `event.preventDefault()` will prevent the drawer from closing. You can inspect `event.detail.source` to see which
 *  element caused the drawer to close. If the source is the drawer element itself, the user has pressed [[Escape]] or
 *  the drawer has been closed programmatically. Avoid using this unless closing the drawer will result in destructive
 *  behavior such as data loss.
 *
 * @csspart dialog - The drawer's internal `<dialog>` element.
 * @csspart header - The drawer's header. This element wraps the title and header actions.
 * @csspart header-actions - Optional actions to add to the header. Works best with `<wa-button>`.
 * @csspart title - The drawer's title.
 * @csspart close-button - The close button, a `<wa-button>`.
 * @csspart close-button__base - The close button's exported `base` part.
 * @csspart body - The drawer's body.
 * @csspart footer - The drawer's footer.
 *
 * @cssproperty --spacing - The amount of space around and between the drawer's content.
 * @cssproperty --size - The preferred size of the drawer. This will be applied to the drawer's width or height
 *   depending on its `placement`. Note that the drawer will shrink to accommodate smaller screens.
 * @cssproperty [--backdrop-filter=none] - A filter to apply to the backdrop behind the drawer.
 * @cssproperty [--show-duration=200ms] - The animation duration when showing the drawer.
 * @cssproperty [--hide-duration=200ms] - The animation duration when hiding the drawer.
 *
 * @property modal - Exposes the internal modal utility that controls focus trapping. To temporarily disable focus
 *   trapping and allow third-party modals spawned from an active Shoelace modal, call `modal.activateExternal()` when
 *   the third-party modal opens. Upon closing, call `modal.deactivateExternal()` to restore Shoelace's focus trapping.
 */
@customElement("wa-drawer")
export default class WaDrawer extends WebAwesomeElement {
  static css = styles;

  private readonly localize = new LocalizeController(this);
  private readonly hasSlotController = new HasSlotController(
    this,
    "footer",
    "header-actions",
    "label",
  );
  private originalTrigger: HTMLElement | null;

  @query(".drawer") drawer: HTMLDialogElement;

  /** Indicates whether or not the drawer is open. Toggle this attribute to show and hide the drawer. */
  @property({ type: Boolean, reflect: true }) open = false;

  /**
   * The drawer's label as displayed in the header. You should always include a relevant label, as it is required for
   * proper accessibility. If you need to display HTML, use the `label` slot instead.
   */
  @property({ reflect: true }) label = "";

  /** The direction from which the drawer will open. */
  @property({ reflect: true }) placement: "top" | "end" | "bottom" | "start" =
    "end";

  /** Disables the header. This will also remove the default close button. */
  @property({ attribute: "without-header", type: Boolean, reflect: true })
  withoutHeader = false;

  /** When enabled, the drawer will be closed when the user clicks outside of it. */
  @property({ attribute: "light-dismiss", type: Boolean }) lightDismiss = true;

  /**
   * Only required for SSR. Set to `true` if you're slotting in a `footer` element so the server-rendered markup
   * includes the footer before the component hydrates on the client.
   */
  @property({ attribute: "with-footer", type: Boolean }) withFooter = false;

  firstUpdated() {
    if (isServer) {
      return;
    }
    if (this.open) {
      this.addOpenListeners();
      this.drawer.showModal();
      lockBodyScrolling(this);
    }
  }

  disconnectedCallback() {
    super.disconnectedCallback();
    unlockBodyScrolling(this);
    this.removeOpenListeners();
  }

  private async requestClose(source: Element) {
    // Hide
    const waHideEvent = new WaHideEvent({ source });
    this.dispatchEvent(waHideEvent);

    if (waHideEvent.defaultPrevented) {
      this.open = true;
      animateWithClass(this.drawer, "pulse");
      return;
    }

    this.removeOpenListeners();

    await animateWithClass(this.drawer, "hide");

    this.open = false;
    this.drawer.close();
    unlockBodyScrolling(this);

    // Restore focus to the original trigger
    const trigger = this.originalTrigger;
    if (typeof trigger?.focus === "function") {
      setTimeout(() => trigger.focus());
    }

    this.dispatchEvent(new WaAfterHideEvent());
  }

  private addOpenListeners() {
    document.addEventListener("keydown", this.handleDocumentKeyDown);
    registerDismissible(this);
  }

  private removeOpenListeners() {
    document.removeEventListener("keydown", this.handleDocumentKeyDown);
    unregisterDismissible(this);
  }

  private handleDialogCancel(event: Event) {
    event.preventDefault();

    if (
      !this.drawer.classList.contains("hide") &&
      event.target === this.drawer &&
      isTopDismissible(this)
    ) {
      this.requestClose(this.drawer);
    }
  }

  private handleDialogClick(event: MouseEvent) {
    const target = event.target as HTMLElement;
    const button = target.closest('[data-drawer="close"]');

    // Close when a button with [data-drawer="close"] is clicked
    if (button) {
      event.stopPropagation();
      this.requestClose(button);
    }
  }

  private async handleDialogPointerDown(event: PointerEvent) {
    // Detect when the backdrop is clicked
    if (event.target === this.drawer) {
      if (this.lightDismiss) {
        this.requestClose(this.drawer);
      } else {
        await animateWithClass(this.drawer, "pulse");
      }
    }
  }

  private handleDocumentKeyDown = (event: KeyboardEvent) => {
    if (event.key === "Escape" && this.open && isTopDismissible(this)) {
      event.preventDefault();
      event.stopPropagation();
      this.requestClose(this.drawer);
    }
  };

  @watch("open", { waitUntilFirstUpdate: true })
  handleOpenChange() {
    // Open or close the drawer
    if (this.open && !this.drawer.open) {
      this.show();
    } else if (this.drawer.open) {
      this.open = true;
      this.requestClose(this.drawer);
    }
  }

  /** Shows the drawer. */
  private async show() {
    // Show
    const waShowEvent = new WaShowEvent();
    this.dispatchEvent(waShowEvent);
    if (waShowEvent.defaultPrevented) {
      this.open = false;
      return;
    }

    // Show
    this.addOpenListeners();
    this.originalTrigger = document.activeElement as HTMLElement;
    this.open = true;
    this.drawer.showModal();

    lockBodyScrolling(this);

    // Set focus on autocomplete if it exists
    requestAnimationFrame(() => {
      const elementToFocus =
        this.querySelector<HTMLButtonElement>("[autofocus]");
      if (elementToFocus && typeof elementToFocus.focus === "function") {
        elementToFocus.focus();
      } else {
        this.drawer.focus();
      }
    });

    await animateWithClass(this.drawer, "show");

    this.dispatchEvent(new WaAfterShowEvent());
  }

  render() {
    const hasHeader = !this.withoutHeader;
    const hasFooter = this.hasUpdated
      ? this.hasSlotController.test("footer")
      : this.withFooter;

    return html`
      <dialog
        part="dialog"
        class=${classMap({
          drawer: true,
          open: this.open,
          top: this.placement === "top",
          end: this.placement === "end",
          bottom: this.placement === "bottom",
          start: this.placement === "start",
        })}
        @cancel=${this.handleDialogCancel}
        @click=${this.handleDialogClick}
        @pointerdown=${this.handleDialogPointerDown}
      >
        ${hasHeader
          ? html`
              <header part="header" class="header">
                <h2 part="title" class="title" id="title">
                  <!-- If there's no label, use an invisible character to prevent the header from collapsing -->
                  <slot name="label">
                    ${this.label.length > 0
                      ? this.label
                      : String.fromCharCode(8203)}
                  </slot>
                </h2>
                <div part="header-actions" class="header-actions">
                  <slot name="header-actions"></slot>
                  <wa-button
                    part="close-button"
                    exportparts="base:close-button__base"
                    class="close"
                    appearance="plain"
                    @click="${(event: PointerEvent) =>
                      this.requestClose(event.target as Element)}"
                  >
                    <wa-icon
                      name="xmark"
                      label=${this.localize.term("close")}
                      library="system"
                      variant="solid"
                    ></wa-icon>
                  </wa-button>
                </div>
              </header>
            `
          : ""}

        <div part="body" class="body"><slot></slot></div>

        ${hasFooter
          ? html`
              <footer part="footer" class="footer">
                <slot name="footer"></slot>
              </footer>
            `
          : ""}
      </dialog>
    `;
  }
}

if (!isServer) {
  //
  // Watch for data-drawer="open *" clicks
  //
  document.addEventListener("click", (event: MouseEvent) => {
    const drawerAttrEl = (event.target as Element).closest("[data-drawer]");

    if (drawerAttrEl instanceof Element) {
      const [command, id] = parseSpaceDelimitedTokens(
        drawerAttrEl.getAttribute("data-drawer") || "",
      );

      if (command === "open" && id?.length) {
        const doc = drawerAttrEl.getRootNode() as Document | ShadowRoot;
        const drawer = doc.getElementById(id) as WaDrawer;

        if (drawer?.localName === "wa-drawer") {
          drawer.open = true;
        } else {
          console.warn(
            `A drawer with an ID of "${id}" could not be found in this document.`,
          );
        }
      }
    }
  });

  //
  // Ugly, but it fixes light dismiss in Safari: https://bugs.webkit.org/show_bug.cgi?id=267688
  //
  // [Mar 27, 2026] - This bug was fixed in Safari 18.3 beta so this can be removed in a year or so.
  //
  document.addEventListener("pointerdown", () => {
    /* empty */
  });
}

declare global {
  interface HTMLElementTagNameMap {
    "wa-drawer": WaDrawer;
  }
}

`````

### Expected (prettier)

`````ts
import { html, isServer } from "lit";
import { customElement, property, query } from "lit/decorators.js";
import { classMap } from "lit/directives/class-map.js";
import { WaAfterHideEvent } from "../../events/after-hide.js";
import { WaAfterShowEvent } from "../../events/after-show.js";
import { WaHideEvent } from "../../events/hide.js";
import { WaShowEvent } from "../../events/show.js";
import { animateWithClass } from "../../internal/animate.js";
import {
  isTopDismissible,
  registerDismissible,
  unregisterDismissible,
} from "../../internal/dismissible-stack.js";
import { parseSpaceDelimitedTokens } from "../../internal/parse.js";
import {
  lockBodyScrolling,
  unlockBodyScrolling,
} from "../../internal/scroll.js";
import { HasSlotController } from "../../internal/slot.js";
import { watch } from "../../internal/watch.js";
import WebAwesomeElement from "../../internal/webawesome-element.js";
import { LocalizeController } from "../../utilities/localize.js";
import "../button/button.js";
import styles from "./drawer.styles.js";

/**
 * @summary Drawers slide in from the edge of a container to expose additional options and information without
 *  navigating away. Useful for navigation menus, filters, and secondary content.
 * @documentation https://webawesome.com/docs/components/drawer
 * @status stable
 * @since 2.0
 *
 * @dependency wa-button
 *
 * @slot - The drawer's main content.
 * @slot label - The drawer's label. Alternatively, you can use the `label` attribute.
 * @slot header-actions - Optional actions to add to the header. Works best with `<wa-button>`.
 * @slot footer - The drawer's footer, usually one or more buttons representing various options.
 *
 * @event wa-show - Emitted when the drawer opens.
 * @event wa-after-show - Emitted after the drawer opens and all animations are complete.
 * @event wa-hide - Emitted when the drawer closes.
 * @event wa-after-hide - Emitted after the drawer closes and all animations are complete.
 * @event {{ source: Element }} wa-hide - Emitted when the drawer is requesting to close. Calling
 *  `event.preventDefault()` will prevent the drawer from closing. You can inspect `event.detail.source` to see which
 *  element caused the drawer to close. If the source is the drawer element itself, the user has pressed [[Escape]] or
 *  the drawer has been closed programmatically. Avoid using this unless closing the drawer will result in destructive
 *  behavior such as data loss.
 *
 * @csspart dialog - The drawer's internal `<dialog>` element.
 * @csspart header - The drawer's header. This element wraps the title and header actions.
 * @csspart header-actions - Optional actions to add to the header. Works best with `<wa-button>`.
 * @csspart title - The drawer's title.
 * @csspart close-button - The close button, a `<wa-button>`.
 * @csspart close-button__base - The close button's exported `base` part.
 * @csspart body - The drawer's body.
 * @csspart footer - The drawer's footer.
 *
 * @cssproperty --spacing - The amount of space around and between the drawer's content.
 * @cssproperty --size - The preferred size of the drawer. This will be applied to the drawer's width or height
 *   depending on its `placement`. Note that the drawer will shrink to accommodate smaller screens.
 * @cssproperty [--backdrop-filter=none] - A filter to apply to the backdrop behind the drawer.
 * @cssproperty [--show-duration=200ms] - The animation duration when showing the drawer.
 * @cssproperty [--hide-duration=200ms] - The animation duration when hiding the drawer.
 *
 * @property modal - Exposes the internal modal utility that controls focus trapping. To temporarily disable focus
 *   trapping and allow third-party modals spawned from an active Shoelace modal, call `modal.activateExternal()` when
 *   the third-party modal opens. Upon closing, call `modal.deactivateExternal()` to restore Shoelace's focus trapping.
 */
@customElement("wa-drawer")
export default class WaDrawer extends WebAwesomeElement {
  static css = styles;

  private readonly localize = new LocalizeController(this);
  private readonly hasSlotController = new HasSlotController(
    this,
    "footer",
    "header-actions",
    "label",
  );
  private originalTrigger: HTMLElement | null;

  @query(".drawer") drawer: HTMLDialogElement;

  /** Indicates whether or not the drawer is open. Toggle this attribute to show and hide the drawer. */
  @property({ type: Boolean, reflect: true }) open = false;

  /**
   * The drawer's label as displayed in the header. You should always include a relevant label, as it is required for
   * proper accessibility. If you need to display HTML, use the `label` slot instead.
   */
  @property({ reflect: true }) label = "";

  /** The direction from which the drawer will open. */
  @property({ reflect: true }) placement: "top" | "end" | "bottom" | "start" =
    "end";

  /** Disables the header. This will also remove the default close button. */
  @property({ attribute: "without-header", type: Boolean, reflect: true })
  withoutHeader = false;

  /** When enabled, the drawer will be closed when the user clicks outside of it. */
  @property({ attribute: "light-dismiss", type: Boolean }) lightDismiss = true;

  /**
   * Only required for SSR. Set to `true` if you're slotting in a `footer` element so the server-rendered markup
   * includes the footer before the component hydrates on the client.
   */
  @property({ attribute: "with-footer", type: Boolean }) withFooter = false;

  firstUpdated() {
    if (isServer) {
      return;
    }
    if (this.open) {
      this.addOpenListeners();
      this.drawer.showModal();
      lockBodyScrolling(this);
    }
  }

  disconnectedCallback() {
    super.disconnectedCallback();
    unlockBodyScrolling(this);
    this.removeOpenListeners();
  }

  private async requestClose(source: Element) {
    // Hide
    const waHideEvent = new WaHideEvent({ source });
    this.dispatchEvent(waHideEvent);

    if (waHideEvent.defaultPrevented) {
      this.open = true;
      animateWithClass(this.drawer, "pulse");
      return;
    }

    this.removeOpenListeners();

    await animateWithClass(this.drawer, "hide");

    this.open = false;
    this.drawer.close();
    unlockBodyScrolling(this);

    // Restore focus to the original trigger
    const trigger = this.originalTrigger;
    if (typeof trigger?.focus === "function") {
      setTimeout(() => trigger.focus());
    }

    this.dispatchEvent(new WaAfterHideEvent());
  }

  private addOpenListeners() {
    document.addEventListener("keydown", this.handleDocumentKeyDown);
    registerDismissible(this);
  }

  private removeOpenListeners() {
    document.removeEventListener("keydown", this.handleDocumentKeyDown);
    unregisterDismissible(this);
  }

  private handleDialogCancel(event: Event) {
    event.preventDefault();

    if (
      !this.drawer.classList.contains("hide") &&
      event.target === this.drawer &&
      isTopDismissible(this)
    ) {
      this.requestClose(this.drawer);
    }
  }

  private handleDialogClick(event: MouseEvent) {
    const target = event.target as HTMLElement;
    const button = target.closest('[data-drawer="close"]');

    // Close when a button with [data-drawer="close"] is clicked
    if (button) {
      event.stopPropagation();
      this.requestClose(button);
    }
  }

  private async handleDialogPointerDown(event: PointerEvent) {
    // Detect when the backdrop is clicked
    if (event.target === this.drawer) {
      if (this.lightDismiss) {
        this.requestClose(this.drawer);
      } else {
        await animateWithClass(this.drawer, "pulse");
      }
    }
  }

  private handleDocumentKeyDown = (event: KeyboardEvent) => {
    if (event.key === "Escape" && this.open && isTopDismissible(this)) {
      event.preventDefault();
      event.stopPropagation();
      this.requestClose(this.drawer);
    }
  };

  @watch("open", { waitUntilFirstUpdate: true })
  handleOpenChange() {
    // Open or close the drawer
    if (this.open && !this.drawer.open) {
      this.show();
    } else if (this.drawer.open) {
      this.open = true;
      this.requestClose(this.drawer);
    }
  }

  /** Shows the drawer. */
  private async show() {
    // Show
    const waShowEvent = new WaShowEvent();
    this.dispatchEvent(waShowEvent);
    if (waShowEvent.defaultPrevented) {
      this.open = false;
      return;
    }

    // Show
    this.addOpenListeners();
    this.originalTrigger = document.activeElement as HTMLElement;
    this.open = true;
    this.drawer.showModal();

    lockBodyScrolling(this);

    // Set focus on autocomplete if it exists
    requestAnimationFrame(() => {
      const elementToFocus =
        this.querySelector<HTMLButtonElement>("[autofocus]");
      if (elementToFocus && typeof elementToFocus.focus === "function") {
        elementToFocus.focus();
      } else {
        this.drawer.focus();
      }
    });

    await animateWithClass(this.drawer, "show");

    this.dispatchEvent(new WaAfterShowEvent());
  }

  render() {
    const hasHeader = !this.withoutHeader;
    const hasFooter = this.hasUpdated
      ? this.hasSlotController.test("footer")
      : this.withFooter;

    return html`
      <dialog
        part="dialog"
        class=${classMap({
          drawer: true,
          open: this.open,
          top: this.placement === "top",
          end: this.placement === "end",
          bottom: this.placement === "bottom",
          start: this.placement === "start",
        })}
        @cancel=${this.handleDialogCancel}
        @click=${this.handleDialogClick}
        @pointerdown=${this.handleDialogPointerDown}
      >
        ${
          hasHeader
            ? html`
                <header part="header" class="header">
                  <h2 part="title" class="title" id="title">
                    <!-- If there's no label, use an invisible character to prevent the header from collapsing -->
                    <slot name="label">
                      ${this.label.length > 0 ? this.label : String.fromCharCode(8203)}
                    </slot>
                  </h2>
                  <div part="header-actions" class="header-actions">
                    <slot name="header-actions"></slot>
                    <wa-button
                      part="close-button"
                      exportparts="base:close-button__base"
                      class="close"
                      appearance="plain"
                      @click="${(event: PointerEvent) => this.requestClose(event.target as Element)}"
                    >
                      <wa-icon
                        name="xmark"
                        label=${this.localize.term("close")}
                        library="system"
                        variant="solid"
                      ></wa-icon>
                    </wa-button>
                  </div>
                </header>
              `
            : ""
        }

        <div part="body" class="body"><slot></slot></div>

        ${
          hasFooter
            ? html`
                <footer part="footer" class="footer">
                  <slot name="footer"></slot>
                </footer>
              `
            : ""
        }
      </dialog>
    `;
  }
}

if (!isServer) {
  //
  // Watch for data-drawer="open *" clicks
  //
  document.addEventListener("click", (event: MouseEvent) => {
    const drawerAttrEl = (event.target as Element).closest("[data-drawer]");

    if (drawerAttrEl instanceof Element) {
      const [command, id] = parseSpaceDelimitedTokens(
        drawerAttrEl.getAttribute("data-drawer") || "",
      );

      if (command === "open" && id?.length) {
        const doc = drawerAttrEl.getRootNode() as Document | ShadowRoot;
        const drawer = doc.getElementById(id) as WaDrawer;

        if (drawer?.localName === "wa-drawer") {
          drawer.open = true;
        } else {
          console.warn(
            `A drawer with an ID of "${id}" could not be found in this document.`,
          );
        }
      }
    }
  });

  //
  // Ugly, but it fixes light dismiss in Safari: https://bugs.webkit.org/show_bug.cgi?id=267688
  //
  // [Mar 27, 2026] - This bug was fixed in Safari 18.3 beta so this can be removed in a year or so.
  //
  document.addEventListener("pointerdown", () => {
    /* empty */
  });
}

declare global {
  interface HTMLElementTagNameMap {
    "wa-drawer": WaDrawer;
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
@@ -262,51 +262,47 @@
         @cancel=${this.handleDialogCancel}
         @click=${this.handleDialogClick}
         @pointerdown=${this.handleDialogPointerDown}
       >
-        ${
-          hasHeader
-            ? html`
-                <header part="header" class="header">
-                  <h2 part="title" class="title" id="title">
-                    <!-- If there's no label, use an invisible character to prevent the header from collapsing -->
-                    <slot name="label">
-                      ${this.label.length > 0 ? this.label : String.fromCharCode(8203)}
-                    </slot>
-                  </h2>
-                  <div part="header-actions" class="header-actions">
-                    <slot name="header-actions"></slot>
-                    <wa-button
-                      part="close-button"
-                      exportparts="base:close-button__base"
-                      class="close"
-                      appearance="plain"
-                      @click="${(event: PointerEvent) => this.requestClose(event.target as Element)}"
-                    >
-                      <wa-icon
-                        name="xmark"
-                        label=${this.localize.term("close")}
-                        library="system"
-                        variant="solid"
-                      ></wa-icon>
-                    </wa-button>
-                  </div>
-                </header>
-              `
-            : ""
-        }
+        ${hasHeader
+          ? html`
+              <header part="header" class="header">
+                <h2 part="title" class="title" id="title">
+                  <!-- If there's no label, use an invisible character to prevent the header from collapsing -->
+                  <slot name="label">
+                    ${this.label.length > 0 ? this.label : String.fromCharCode(8203)}
+                  </slot>
+                </h2>
+                <div part="header-actions" class="header-actions">
+                  <slot name="header-actions"></slot>
+                  <wa-button
+                    part="close-button"
+                    exportparts="base:close-button__base"
+                    class="close"
+                    appearance="plain"
+                    @click="${(event: PointerEvent) => this.requestClose(event.target as Element)}"
+                  >
+                    <wa-icon
+                      name="xmark"
+                      label=${this.localize.term("close")}
+                      library="system"
+                      variant="solid"
+                    ></wa-icon>
+                  </wa-button>
+                </div>
+              </header>
+            `
+          : ""}
 
         <div part="body" class="body"><slot></slot></div>
 
-        ${
-          hasFooter
-            ? html`
-                <footer part="footer" class="footer">
-                  <slot name="footer"></slot>
-                </footer>
-              `
-            : ""
-        }
+        ${hasFooter
+          ? html`
+              <footer part="footer" class="footer">
+                <slot name="footer"></slot>
+              </footer>
+            `
+          : ""}
       </dialog>
     `;
   }
 }

`````

### Actual (oxfmt)

`````ts
import { html, isServer } from "lit";
import { customElement, property, query } from "lit/decorators.js";
import { classMap } from "lit/directives/class-map.js";
import { WaAfterHideEvent } from "../../events/after-hide.js";
import { WaAfterShowEvent } from "../../events/after-show.js";
import { WaHideEvent } from "../../events/hide.js";
import { WaShowEvent } from "../../events/show.js";
import { animateWithClass } from "../../internal/animate.js";
import {
  isTopDismissible,
  registerDismissible,
  unregisterDismissible,
} from "../../internal/dismissible-stack.js";
import { parseSpaceDelimitedTokens } from "../../internal/parse.js";
import { lockBodyScrolling, unlockBodyScrolling } from "../../internal/scroll.js";
import { HasSlotController } from "../../internal/slot.js";
import { watch } from "../../internal/watch.js";
import WebAwesomeElement from "../../internal/webawesome-element.js";
import { LocalizeController } from "../../utilities/localize.js";
import "../button/button.js";
import styles from "./drawer.styles.js";

/**
 * @summary Drawers slide in from the edge of a container to expose additional options and information without
 *  navigating away. Useful for navigation menus, filters, and secondary content.
 * @documentation https://webawesome.com/docs/components/drawer
 * @status stable
 * @since 2.0
 *
 * @dependency wa-button
 *
 * @slot - The drawer's main content.
 * @slot label - The drawer's label. Alternatively, you can use the `label` attribute.
 * @slot header-actions - Optional actions to add to the header. Works best with `<wa-button>`.
 * @slot footer - The drawer's footer, usually one or more buttons representing various options.
 *
 * @event wa-show - Emitted when the drawer opens.
 * @event wa-after-show - Emitted after the drawer opens and all animations are complete.
 * @event wa-hide - Emitted when the drawer closes.
 * @event wa-after-hide - Emitted after the drawer closes and all animations are complete.
 * @event {{ source: Element }} wa-hide - Emitted when the drawer is requesting to close. Calling
 *  `event.preventDefault()` will prevent the drawer from closing. You can inspect `event.detail.source` to see which
 *  element caused the drawer to close. If the source is the drawer element itself, the user has pressed [[Escape]] or
 *  the drawer has been closed programmatically. Avoid using this unless closing the drawer will result in destructive
 *  behavior such as data loss.
 *
 * @csspart dialog - The drawer's internal `<dialog>` element.
 * @csspart header - The drawer's header. This element wraps the title and header actions.
 * @csspart header-actions - Optional actions to add to the header. Works best with `<wa-button>`.
 * @csspart title - The drawer's title.
 * @csspart close-button - The close button, a `<wa-button>`.
 * @csspart close-button__base - The close button's exported `base` part.
 * @csspart body - The drawer's body.
 * @csspart footer - The drawer's footer.
 *
 * @cssproperty --spacing - The amount of space around and between the drawer's content.
 * @cssproperty --size - The preferred size of the drawer. This will be applied to the drawer's width or height
 *   depending on its `placement`. Note that the drawer will shrink to accommodate smaller screens.
 * @cssproperty [--backdrop-filter=none] - A filter to apply to the backdrop behind the drawer.
 * @cssproperty [--show-duration=200ms] - The animation duration when showing the drawer.
 * @cssproperty [--hide-duration=200ms] - The animation duration when hiding the drawer.
 *
 * @property modal - Exposes the internal modal utility that controls focus trapping. To temporarily disable focus
 *   trapping and allow third-party modals spawned from an active Shoelace modal, call `modal.activateExternal()` when
 *   the third-party modal opens. Upon closing, call `modal.deactivateExternal()` to restore Shoelace's focus trapping.
 */
@customElement("wa-drawer")
export default class WaDrawer extends WebAwesomeElement {
  static css = styles;

  private readonly localize = new LocalizeController(this);
  private readonly hasSlotController = new HasSlotController(
    this,
    "footer",
    "header-actions",
    "label",
  );
  private originalTrigger: HTMLElement | null;

  @query(".drawer") drawer: HTMLDialogElement;

  /** Indicates whether or not the drawer is open. Toggle this attribute to show and hide the drawer. */
  @property({ type: Boolean, reflect: true }) open = false;

  /**
   * The drawer's label as displayed in the header. You should always include a relevant label, as it is required for
   * proper accessibility. If you need to display HTML, use the `label` slot instead.
   */
  @property({ reflect: true }) label = "";

  /** The direction from which the drawer will open. */
  @property({ reflect: true }) placement: "top" | "end" | "bottom" | "start" = "end";

  /** Disables the header. This will also remove the default close button. */
  @property({ attribute: "without-header", type: Boolean, reflect: true }) withoutHeader = false;

  /** When enabled, the drawer will be closed when the user clicks outside of it. */
  @property({ attribute: "light-dismiss", type: Boolean }) lightDismiss = true;

  /**
   * Only required for SSR. Set to `true` if you're slotting in a `footer` element so the server-rendered markup
   * includes the footer before the component hydrates on the client.
   */
  @property({ attribute: "with-footer", type: Boolean }) withFooter = false;

  firstUpdated() {
    if (isServer) {
      return;
    }
    if (this.open) {
      this.addOpenListeners();
      this.drawer.showModal();
      lockBodyScrolling(this);
    }
  }

  disconnectedCallback() {
    super.disconnectedCallback();
    unlockBodyScrolling(this);
    this.removeOpenListeners();
  }

  private async requestClose(source: Element) {
    // Hide
    const waHideEvent = new WaHideEvent({ source });
    this.dispatchEvent(waHideEvent);

    if (waHideEvent.defaultPrevented) {
      this.open = true;
      animateWithClass(this.drawer, "pulse");
      return;
    }

    this.removeOpenListeners();

    await animateWithClass(this.drawer, "hide");

    this.open = false;
    this.drawer.close();
    unlockBodyScrolling(this);

    // Restore focus to the original trigger
    const trigger = this.originalTrigger;
    if (typeof trigger?.focus === "function") {
      setTimeout(() => trigger.focus());
    }

    this.dispatchEvent(new WaAfterHideEvent());
  }

  private addOpenListeners() {
    document.addEventListener("keydown", this.handleDocumentKeyDown);
    registerDismissible(this);
  }

  private removeOpenListeners() {
    document.removeEventListener("keydown", this.handleDocumentKeyDown);
    unregisterDismissible(this);
  }

  private handleDialogCancel(event: Event) {
    event.preventDefault();

    if (
      !this.drawer.classList.contains("hide") &&
      event.target === this.drawer &&
      isTopDismissible(this)
    ) {
      this.requestClose(this.drawer);
    }
  }

  private handleDialogClick(event: MouseEvent) {
    const target = event.target as HTMLElement;
    const button = target.closest('[data-drawer="close"]');

    // Close when a button with [data-drawer="close"] is clicked
    if (button) {
      event.stopPropagation();
      this.requestClose(button);
    }
  }

  private async handleDialogPointerDown(event: PointerEvent) {
    // Detect when the backdrop is clicked
    if (event.target === this.drawer) {
      if (this.lightDismiss) {
        this.requestClose(this.drawer);
      } else {
        await animateWithClass(this.drawer, "pulse");
      }
    }
  }

  private handleDocumentKeyDown = (event: KeyboardEvent) => {
    if (event.key === "Escape" && this.open && isTopDismissible(this)) {
      event.preventDefault();
      event.stopPropagation();
      this.requestClose(this.drawer);
    }
  };

  @watch("open", { waitUntilFirstUpdate: true })
  handleOpenChange() {
    // Open or close the drawer
    if (this.open && !this.drawer.open) {
      this.show();
    } else if (this.drawer.open) {
      this.open = true;
      this.requestClose(this.drawer);
    }
  }

  /** Shows the drawer. */
  private async show() {
    // Show
    const waShowEvent = new WaShowEvent();
    this.dispatchEvent(waShowEvent);
    if (waShowEvent.defaultPrevented) {
      this.open = false;
      return;
    }

    // Show
    this.addOpenListeners();
    this.originalTrigger = document.activeElement as HTMLElement;
    this.open = true;
    this.drawer.showModal();

    lockBodyScrolling(this);

    // Set focus on autocomplete if it exists
    requestAnimationFrame(() => {
      const elementToFocus = this.querySelector<HTMLButtonElement>("[autofocus]");
      if (elementToFocus && typeof elementToFocus.focus === "function") {
        elementToFocus.focus();
      } else {
        this.drawer.focus();
      }
    });

    await animateWithClass(this.drawer, "show");

    this.dispatchEvent(new WaAfterShowEvent());
  }

  render() {
    const hasHeader = !this.withoutHeader;
    const hasFooter = this.hasUpdated ? this.hasSlotController.test("footer") : this.withFooter;

    return html`
      <dialog
        part="dialog"
        class=${classMap({
          drawer: true,
          open: this.open,
          top: this.placement === "top",
          end: this.placement === "end",
          bottom: this.placement === "bottom",
          start: this.placement === "start",
        })}
        @cancel=${this.handleDialogCancel}
        @click=${this.handleDialogClick}
        @pointerdown=${this.handleDialogPointerDown}
      >
        ${hasHeader
          ? html`
              <header part="header" class="header">
                <h2 part="title" class="title" id="title">
                  <!-- If there's no label, use an invisible character to prevent the header from collapsing -->
                  <slot name="label">
                    ${this.label.length > 0 ? this.label : String.fromCharCode(8203)}
                  </slot>
                </h2>
                <div part="header-actions" class="header-actions">
                  <slot name="header-actions"></slot>
                  <wa-button
                    part="close-button"
                    exportparts="base:close-button__base"
                    class="close"
                    appearance="plain"
                    @click="${(event: PointerEvent) => this.requestClose(event.target as Element)}"
                  >
                    <wa-icon
                      name="xmark"
                      label=${this.localize.term("close")}
                      library="system"
                      variant="solid"
                    ></wa-icon>
                  </wa-button>
                </div>
              </header>
            `
          : ""}

        <div part="body" class="body"><slot></slot></div>

        ${hasFooter
          ? html`
              <footer part="footer" class="footer">
                <slot name="footer"></slot>
              </footer>
            `
          : ""}
      </dialog>
    `;
  }
}

if (!isServer) {
  //
  // Watch for data-drawer="open *" clicks
  //
  document.addEventListener("click", (event: MouseEvent) => {
    const drawerAttrEl = (event.target as Element).closest("[data-drawer]");

    if (drawerAttrEl instanceof Element) {
      const [command, id] = parseSpaceDelimitedTokens(
        drawerAttrEl.getAttribute("data-drawer") || "",
      );

      if (command === "open" && id?.length) {
        const doc = drawerAttrEl.getRootNode() as Document | ShadowRoot;
        const drawer = doc.getElementById(id) as WaDrawer;

        if (drawer?.localName === "wa-drawer") {
          drawer.open = true;
        } else {
          console.warn(`A drawer with an ID of "${id}" could not be found in this document.`);
        }
      }
    }
  });

  //
  // Ugly, but it fixes light dismiss in Safari: https://bugs.webkit.org/show_bug.cgi?id=267688
  //
  // [Mar 27, 2026] - This bug was fixed in Safari 18.3 beta so this can be removed in a year or so.
  //
  document.addEventListener("pointerdown", () => {
    /* empty */
  });
}

declare global {
  interface HTMLElementTagNameMap {
    "wa-drawer": WaDrawer;
  }
}

`````

### Expected (prettier)

`````ts
import { html, isServer } from "lit";
import { customElement, property, query } from "lit/decorators.js";
import { classMap } from "lit/directives/class-map.js";
import { WaAfterHideEvent } from "../../events/after-hide.js";
import { WaAfterShowEvent } from "../../events/after-show.js";
import { WaHideEvent } from "../../events/hide.js";
import { WaShowEvent } from "../../events/show.js";
import { animateWithClass } from "../../internal/animate.js";
import {
  isTopDismissible,
  registerDismissible,
  unregisterDismissible,
} from "../../internal/dismissible-stack.js";
import { parseSpaceDelimitedTokens } from "../../internal/parse.js";
import { lockBodyScrolling, unlockBodyScrolling } from "../../internal/scroll.js";
import { HasSlotController } from "../../internal/slot.js";
import { watch } from "../../internal/watch.js";
import WebAwesomeElement from "../../internal/webawesome-element.js";
import { LocalizeController } from "../../utilities/localize.js";
import "../button/button.js";
import styles from "./drawer.styles.js";

/**
 * @summary Drawers slide in from the edge of a container to expose additional options and information without
 *  navigating away. Useful for navigation menus, filters, and secondary content.
 * @documentation https://webawesome.com/docs/components/drawer
 * @status stable
 * @since 2.0
 *
 * @dependency wa-button
 *
 * @slot - The drawer's main content.
 * @slot label - The drawer's label. Alternatively, you can use the `label` attribute.
 * @slot header-actions - Optional actions to add to the header. Works best with `<wa-button>`.
 * @slot footer - The drawer's footer, usually one or more buttons representing various options.
 *
 * @event wa-show - Emitted when the drawer opens.
 * @event wa-after-show - Emitted after the drawer opens and all animations are complete.
 * @event wa-hide - Emitted when the drawer closes.
 * @event wa-after-hide - Emitted after the drawer closes and all animations are complete.
 * @event {{ source: Element }} wa-hide - Emitted when the drawer is requesting to close. Calling
 *  `event.preventDefault()` will prevent the drawer from closing. You can inspect `event.detail.source` to see which
 *  element caused the drawer to close. If the source is the drawer element itself, the user has pressed [[Escape]] or
 *  the drawer has been closed programmatically. Avoid using this unless closing the drawer will result in destructive
 *  behavior such as data loss.
 *
 * @csspart dialog - The drawer's internal `<dialog>` element.
 * @csspart header - The drawer's header. This element wraps the title and header actions.
 * @csspart header-actions - Optional actions to add to the header. Works best with `<wa-button>`.
 * @csspart title - The drawer's title.
 * @csspart close-button - The close button, a `<wa-button>`.
 * @csspart close-button__base - The close button's exported `base` part.
 * @csspart body - The drawer's body.
 * @csspart footer - The drawer's footer.
 *
 * @cssproperty --spacing - The amount of space around and between the drawer's content.
 * @cssproperty --size - The preferred size of the drawer. This will be applied to the drawer's width or height
 *   depending on its `placement`. Note that the drawer will shrink to accommodate smaller screens.
 * @cssproperty [--backdrop-filter=none] - A filter to apply to the backdrop behind the drawer.
 * @cssproperty [--show-duration=200ms] - The animation duration when showing the drawer.
 * @cssproperty [--hide-duration=200ms] - The animation duration when hiding the drawer.
 *
 * @property modal - Exposes the internal modal utility that controls focus trapping. To temporarily disable focus
 *   trapping and allow third-party modals spawned from an active Shoelace modal, call `modal.activateExternal()` when
 *   the third-party modal opens. Upon closing, call `modal.deactivateExternal()` to restore Shoelace's focus trapping.
 */
@customElement("wa-drawer")
export default class WaDrawer extends WebAwesomeElement {
  static css = styles;

  private readonly localize = new LocalizeController(this);
  private readonly hasSlotController = new HasSlotController(
    this,
    "footer",
    "header-actions",
    "label",
  );
  private originalTrigger: HTMLElement | null;

  @query(".drawer") drawer: HTMLDialogElement;

  /** Indicates whether or not the drawer is open. Toggle this attribute to show and hide the drawer. */
  @property({ type: Boolean, reflect: true }) open = false;

  /**
   * The drawer's label as displayed in the header. You should always include a relevant label, as it is required for
   * proper accessibility. If you need to display HTML, use the `label` slot instead.
   */
  @property({ reflect: true }) label = "";

  /** The direction from which the drawer will open. */
  @property({ reflect: true }) placement: "top" | "end" | "bottom" | "start" = "end";

  /** Disables the header. This will also remove the default close button. */
  @property({ attribute: "without-header", type: Boolean, reflect: true }) withoutHeader = false;

  /** When enabled, the drawer will be closed when the user clicks outside of it. */
  @property({ attribute: "light-dismiss", type: Boolean }) lightDismiss = true;

  /**
   * Only required for SSR. Set to `true` if you're slotting in a `footer` element so the server-rendered markup
   * includes the footer before the component hydrates on the client.
   */
  @property({ attribute: "with-footer", type: Boolean }) withFooter = false;

  firstUpdated() {
    if (isServer) {
      return;
    }
    if (this.open) {
      this.addOpenListeners();
      this.drawer.showModal();
      lockBodyScrolling(this);
    }
  }

  disconnectedCallback() {
    super.disconnectedCallback();
    unlockBodyScrolling(this);
    this.removeOpenListeners();
  }

  private async requestClose(source: Element) {
    // Hide
    const waHideEvent = new WaHideEvent({ source });
    this.dispatchEvent(waHideEvent);

    if (waHideEvent.defaultPrevented) {
      this.open = true;
      animateWithClass(this.drawer, "pulse");
      return;
    }

    this.removeOpenListeners();

    await animateWithClass(this.drawer, "hide");

    this.open = false;
    this.drawer.close();
    unlockBodyScrolling(this);

    // Restore focus to the original trigger
    const trigger = this.originalTrigger;
    if (typeof trigger?.focus === "function") {
      setTimeout(() => trigger.focus());
    }

    this.dispatchEvent(new WaAfterHideEvent());
  }

  private addOpenListeners() {
    document.addEventListener("keydown", this.handleDocumentKeyDown);
    registerDismissible(this);
  }

  private removeOpenListeners() {
    document.removeEventListener("keydown", this.handleDocumentKeyDown);
    unregisterDismissible(this);
  }

  private handleDialogCancel(event: Event) {
    event.preventDefault();

    if (
      !this.drawer.classList.contains("hide") &&
      event.target === this.drawer &&
      isTopDismissible(this)
    ) {
      this.requestClose(this.drawer);
    }
  }

  private handleDialogClick(event: MouseEvent) {
    const target = event.target as HTMLElement;
    const button = target.closest('[data-drawer="close"]');

    // Close when a button with [data-drawer="close"] is clicked
    if (button) {
      event.stopPropagation();
      this.requestClose(button);
    }
  }

  private async handleDialogPointerDown(event: PointerEvent) {
    // Detect when the backdrop is clicked
    if (event.target === this.drawer) {
      if (this.lightDismiss) {
        this.requestClose(this.drawer);
      } else {
        await animateWithClass(this.drawer, "pulse");
      }
    }
  }

  private handleDocumentKeyDown = (event: KeyboardEvent) => {
    if (event.key === "Escape" && this.open && isTopDismissible(this)) {
      event.preventDefault();
      event.stopPropagation();
      this.requestClose(this.drawer);
    }
  };

  @watch("open", { waitUntilFirstUpdate: true })
  handleOpenChange() {
    // Open or close the drawer
    if (this.open && !this.drawer.open) {
      this.show();
    } else if (this.drawer.open) {
      this.open = true;
      this.requestClose(this.drawer);
    }
  }

  /** Shows the drawer. */
  private async show() {
    // Show
    const waShowEvent = new WaShowEvent();
    this.dispatchEvent(waShowEvent);
    if (waShowEvent.defaultPrevented) {
      this.open = false;
      return;
    }

    // Show
    this.addOpenListeners();
    this.originalTrigger = document.activeElement as HTMLElement;
    this.open = true;
    this.drawer.showModal();

    lockBodyScrolling(this);

    // Set focus on autocomplete if it exists
    requestAnimationFrame(() => {
      const elementToFocus = this.querySelector<HTMLButtonElement>("[autofocus]");
      if (elementToFocus && typeof elementToFocus.focus === "function") {
        elementToFocus.focus();
      } else {
        this.drawer.focus();
      }
    });

    await animateWithClass(this.drawer, "show");

    this.dispatchEvent(new WaAfterShowEvent());
  }

  render() {
    const hasHeader = !this.withoutHeader;
    const hasFooter = this.hasUpdated ? this.hasSlotController.test("footer") : this.withFooter;

    return html`
      <dialog
        part="dialog"
        class=${classMap({
          drawer: true,
          open: this.open,
          top: this.placement === "top",
          end: this.placement === "end",
          bottom: this.placement === "bottom",
          start: this.placement === "start",
        })}
        @cancel=${this.handleDialogCancel}
        @click=${this.handleDialogClick}
        @pointerdown=${this.handleDialogPointerDown}
      >
        ${
          hasHeader
            ? html`
                <header part="header" class="header">
                  <h2 part="title" class="title" id="title">
                    <!-- If there's no label, use an invisible character to prevent the header from collapsing -->
                    <slot name="label">
                      ${this.label.length > 0 ? this.label : String.fromCharCode(8203)}
                    </slot>
                  </h2>
                  <div part="header-actions" class="header-actions">
                    <slot name="header-actions"></slot>
                    <wa-button
                      part="close-button"
                      exportparts="base:close-button__base"
                      class="close"
                      appearance="plain"
                      @click="${(event: PointerEvent) => this.requestClose(event.target as Element)}"
                    >
                      <wa-icon
                        name="xmark"
                        label=${this.localize.term("close")}
                        library="system"
                        variant="solid"
                      ></wa-icon>
                    </wa-button>
                  </div>
                </header>
              `
            : ""
        }

        <div part="body" class="body"><slot></slot></div>

        ${
          hasFooter
            ? html`
                <footer part="footer" class="footer">
                  <slot name="footer"></slot>
                </footer>
              `
            : ""
        }
      </dialog>
    `;
  }
}

if (!isServer) {
  //
  // Watch for data-drawer="open *" clicks
  //
  document.addEventListener("click", (event: MouseEvent) => {
    const drawerAttrEl = (event.target as Element).closest("[data-drawer]");

    if (drawerAttrEl instanceof Element) {
      const [command, id] = parseSpaceDelimitedTokens(
        drawerAttrEl.getAttribute("data-drawer") || "",
      );

      if (command === "open" && id?.length) {
        const doc = drawerAttrEl.getRootNode() as Document | ShadowRoot;
        const drawer = doc.getElementById(id) as WaDrawer;

        if (drawer?.localName === "wa-drawer") {
          drawer.open = true;
        } else {
          console.warn(`A drawer with an ID of "${id}" could not be found in this document.`);
        }
      }
    }
  });

  //
  // Ugly, but it fixes light dismiss in Safari: https://bugs.webkit.org/show_bug.cgi?id=267688
  //
  // [Mar 27, 2026] - This bug was fixed in Safari 18.3 beta so this can be removed in a year or so.
  //
  document.addEventListener("pointerdown", () => {
    /* empty */
  });
}

declare global {
  interface HTMLElementTagNameMap {
    "wa-drawer": WaDrawer;
  }
}

`````
