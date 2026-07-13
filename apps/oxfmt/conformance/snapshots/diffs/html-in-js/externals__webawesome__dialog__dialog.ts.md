# externals/webawesome/dialog/dialog.ts

## Option 1

`````json
{"printWidth":80}
`````

### Diff

`````diff
===================================================================
--- prettier
+++ oxfmt
@@ -252,51 +252,50 @@
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
import styles from "./dialog.styles.js";

/**
 * @summary Dialogs appear above the page and require the user's immediate attention. Use them for confirmations, forms,
 *  or focused tasks that interrupt the main flow.
 * @documentation https://webawesome.com/docs/components/dialog
 * @status stable
 * @since 2.0
 *
 * @dependency wa-button
 *
 * @slot - The dialog's main content.
 * @slot label - The dialog's label. Alternatively, you can use the `label` attribute.
 * @slot header-actions - Optional actions to add to the header. Works best with `<wa-button>`.
 * @slot footer - The dialog's footer, usually one or more buttons representing various options.
 *
 * @event wa-show - Emitted when the dialog opens.
 * @event wa-after-show - Emitted after the dialog opens and all animations are complete.
 * @event {{ source: Element }} wa-hide - Emitted when the dialog is requested to close. Calling
 *  `event.preventDefault()` will prevent the dialog from closing. You can inspect `event.detail.source` to see which
 *  element caused the dialog to close. If the source is the dialog element itself, the user has pressed [[Escape]] or
 *  the dialog has been closed programmatically. Avoid using this unless closing the dialog will result in destructive
 *  behavior such as data loss.
 * @event wa-after-hide - Emitted after the dialog closes and all animations are complete.
 *
 * @csspart dialog - The dialog's internal `<dialog>` element.
 * @csspart header - The dialog's header. This element wraps the title and header actions.
 * @csspart header-actions - Optional actions to add to the header. Works best with `<wa-button>`.
 * @csspart title - The dialog's title.
 * @csspart close-button - The close button, a `<wa-button>`.
 * @csspart close-button__base - The close button's exported `base` part.
 * @csspart body - The dialog's body.
 * @csspart footer - The dialog's footer.
 *
 * @cssproperty --spacing - The amount of space around and between the dialog's content.
 * @cssproperty --width - The preferred width of the dialog. Note that the dialog will shrink to accommodate smaller screens.
 * @cssproperty [--backdrop-filter=none] - A filter to apply to the backdrop behind the dialog.
 * @cssproperty [--show-duration=200ms] - The animation duration when showing the dialog.
 * @cssproperty [--hide-duration=200ms] - The animation duration when hiding the dialog.
 */
@customElement("wa-dialog")
export default class WaDialog extends WebAwesomeElement {
  static css = styles;

  private readonly localize = new LocalizeController(this);
  private readonly hasSlotController = new HasSlotController(
    this,
    "footer",
    "header-actions",
    "label",
  );
  private originalTrigger: HTMLElement | null;

  @query(".dialog") dialog: HTMLDialogElement;

  /** Indicates whether or not the dialog is open. Toggle this attribute to show and hide the dialog. */
  @property({ type: Boolean, reflect: true }) open = false;

  /**
   * The dialog's label as displayed in the header. You should always include a relevant label, as it is required for
   * proper accessibility. If you need to display HTML, use the `label` slot instead.
   */
  @property({ reflect: true }) label = "";

  /** Disables the header. This will also remove the default close button. */
  @property({ attribute: "without-header", type: Boolean, reflect: true })
  withoutHeader = false;

  /** When enabled, the dialog will be closed when the user clicks outside of it. */
  @property({ attribute: "light-dismiss", type: Boolean }) lightDismiss = false;

  /**
   * Only required for SSR. Set to `true` if you're slotting in a `footer` element so the server-rendered markup
   * includes the footer before the component hydrates on the client.
   */
  @property({ attribute: "with-footer", type: Boolean }) withFooter = false;

  firstUpdated() {
    if (this.open) {
      this.addOpenListeners();
      this.dialog.showModal();
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
      animateWithClass(this.dialog, "pulse");
      return;
    }

    this.removeOpenListeners();

    await animateWithClass(this.dialog, "hide");

    this.open = false;
    this.dialog.close();
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
      !this.dialog.classList.contains("hide") &&
      event.target === this.dialog &&
      isTopDismissible(this)
    ) {
      this.requestClose(this.dialog);
    }
  }

  private handleDialogClick(event: MouseEvent) {
    const target = event.target as HTMLElement;
    const button = target.closest('[data-dialog="close"]');

    // Close when a button with [data-dialog="close"] is clicked
    if (button) {
      event.stopPropagation();
      this.requestClose(button);
    }
  }

  private async handleDialogPointerDown(event: PointerEvent) {
    // Detect when the backdrop is clicked
    if (event.target === this.dialog) {
      if (this.lightDismiss) {
        this.requestClose(this.dialog);
      } else {
        await animateWithClass(this.dialog, "pulse");
      }
    }
  }

  private handleDocumentKeyDown = (event: KeyboardEvent) => {
    if (event.key === "Escape" && this.open && isTopDismissible(this)) {
      event.preventDefault();
      event.stopPropagation();
      this.requestClose(this.dialog);
    }
  };

  @watch("open", { waitUntilFirstUpdate: true })
  handleOpenChange() {
    // Open or close the dialog
    if (this.open && !this.dialog.open) {
      this.show();
    } else if (!this.open && this.dialog.open) {
      this.open = true;
      this.requestClose(this.dialog);
    }
  }

  /** Shows the dialog. */
  private async show() {
    // Show
    const waShowEvent = new WaShowEvent();
    this.dispatchEvent(waShowEvent);
    if (waShowEvent.defaultPrevented) {
      this.open = false;
      return;
    }

    this.addOpenListeners();
    this.originalTrigger = document.activeElement as HTMLElement;
    this.open = true;
    this.dialog.showModal();

    lockBodyScrolling(this);

    // Set focus on autocomplete if it exists
    requestAnimationFrame(() => {
      const elementToFocus =
        this.querySelector<HTMLButtonElement>("[autofocus]");
      if (elementToFocus && typeof elementToFocus.focus === "function") {
        elementToFocus.focus();
      } else {
        this.dialog.focus();
      }
    });

    await animateWithClass(this.dialog, "show");

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
          dialog: true,
          open: this.open,
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

// Ugly, but it fixes light dismiss in Safari: https://bugs.webkit.org/show_bug.cgi?id=267688
if (!isServer) {
  //
  // Watch for data-dialog="open *" clicks
  //
  document.addEventListener("click", (event: MouseEvent) => {
    const dialogAttrEl = (event.target as Element).closest("[data-dialog]");

    if (dialogAttrEl instanceof Element) {
      const [command, id] = parseSpaceDelimitedTokens(
        dialogAttrEl.getAttribute("data-dialog") || "",
      );

      if (command === "open" && id?.length) {
        const doc = dialogAttrEl.getRootNode() as Document | ShadowRoot;
        const dialog = doc.getElementById(id) as WaDialog;

        if (dialog?.localName === "wa-dialog") {
          dialog.open = true;
        } else {
          console.warn(
            `A dialog with an ID of "${id}" could not be found in this document.`,
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
    "wa-dialog": WaDialog;
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
import styles from "./dialog.styles.js";

/**
 * @summary Dialogs appear above the page and require the user's immediate attention. Use them for confirmations, forms,
 *  or focused tasks that interrupt the main flow.
 * @documentation https://webawesome.com/docs/components/dialog
 * @status stable
 * @since 2.0
 *
 * @dependency wa-button
 *
 * @slot - The dialog's main content.
 * @slot label - The dialog's label. Alternatively, you can use the `label` attribute.
 * @slot header-actions - Optional actions to add to the header. Works best with `<wa-button>`.
 * @slot footer - The dialog's footer, usually one or more buttons representing various options.
 *
 * @event wa-show - Emitted when the dialog opens.
 * @event wa-after-show - Emitted after the dialog opens and all animations are complete.
 * @event {{ source: Element }} wa-hide - Emitted when the dialog is requested to close. Calling
 *  `event.preventDefault()` will prevent the dialog from closing. You can inspect `event.detail.source` to see which
 *  element caused the dialog to close. If the source is the dialog element itself, the user has pressed [[Escape]] or
 *  the dialog has been closed programmatically. Avoid using this unless closing the dialog will result in destructive
 *  behavior such as data loss.
 * @event wa-after-hide - Emitted after the dialog closes and all animations are complete.
 *
 * @csspart dialog - The dialog's internal `<dialog>` element.
 * @csspart header - The dialog's header. This element wraps the title and header actions.
 * @csspart header-actions - Optional actions to add to the header. Works best with `<wa-button>`.
 * @csspart title - The dialog's title.
 * @csspart close-button - The close button, a `<wa-button>`.
 * @csspart close-button__base - The close button's exported `base` part.
 * @csspart body - The dialog's body.
 * @csspart footer - The dialog's footer.
 *
 * @cssproperty --spacing - The amount of space around and between the dialog's content.
 * @cssproperty --width - The preferred width of the dialog. Note that the dialog will shrink to accommodate smaller screens.
 * @cssproperty [--backdrop-filter=none] - A filter to apply to the backdrop behind the dialog.
 * @cssproperty [--show-duration=200ms] - The animation duration when showing the dialog.
 * @cssproperty [--hide-duration=200ms] - The animation duration when hiding the dialog.
 */
@customElement("wa-dialog")
export default class WaDialog extends WebAwesomeElement {
  static css = styles;

  private readonly localize = new LocalizeController(this);
  private readonly hasSlotController = new HasSlotController(
    this,
    "footer",
    "header-actions",
    "label",
  );
  private originalTrigger: HTMLElement | null;

  @query(".dialog") dialog: HTMLDialogElement;

  /** Indicates whether or not the dialog is open. Toggle this attribute to show and hide the dialog. */
  @property({ type: Boolean, reflect: true }) open = false;

  /**
   * The dialog's label as displayed in the header. You should always include a relevant label, as it is required for
   * proper accessibility. If you need to display HTML, use the `label` slot instead.
   */
  @property({ reflect: true }) label = "";

  /** Disables the header. This will also remove the default close button. */
  @property({ attribute: "without-header", type: Boolean, reflect: true })
  withoutHeader = false;

  /** When enabled, the dialog will be closed when the user clicks outside of it. */
  @property({ attribute: "light-dismiss", type: Boolean }) lightDismiss = false;

  /**
   * Only required for SSR. Set to `true` if you're slotting in a `footer` element so the server-rendered markup
   * includes the footer before the component hydrates on the client.
   */
  @property({ attribute: "with-footer", type: Boolean }) withFooter = false;

  firstUpdated() {
    if (this.open) {
      this.addOpenListeners();
      this.dialog.showModal();
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
      animateWithClass(this.dialog, "pulse");
      return;
    }

    this.removeOpenListeners();

    await animateWithClass(this.dialog, "hide");

    this.open = false;
    this.dialog.close();
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
      !this.dialog.classList.contains("hide") &&
      event.target === this.dialog &&
      isTopDismissible(this)
    ) {
      this.requestClose(this.dialog);
    }
  }

  private handleDialogClick(event: MouseEvent) {
    const target = event.target as HTMLElement;
    const button = target.closest('[data-dialog="close"]');

    // Close when a button with [data-dialog="close"] is clicked
    if (button) {
      event.stopPropagation();
      this.requestClose(button);
    }
  }

  private async handleDialogPointerDown(event: PointerEvent) {
    // Detect when the backdrop is clicked
    if (event.target === this.dialog) {
      if (this.lightDismiss) {
        this.requestClose(this.dialog);
      } else {
        await animateWithClass(this.dialog, "pulse");
      }
    }
  }

  private handleDocumentKeyDown = (event: KeyboardEvent) => {
    if (event.key === "Escape" && this.open && isTopDismissible(this)) {
      event.preventDefault();
      event.stopPropagation();
      this.requestClose(this.dialog);
    }
  };

  @watch("open", { waitUntilFirstUpdate: true })
  handleOpenChange() {
    // Open or close the dialog
    if (this.open && !this.dialog.open) {
      this.show();
    } else if (!this.open && this.dialog.open) {
      this.open = true;
      this.requestClose(this.dialog);
    }
  }

  /** Shows the dialog. */
  private async show() {
    // Show
    const waShowEvent = new WaShowEvent();
    this.dispatchEvent(waShowEvent);
    if (waShowEvent.defaultPrevented) {
      this.open = false;
      return;
    }

    this.addOpenListeners();
    this.originalTrigger = document.activeElement as HTMLElement;
    this.open = true;
    this.dialog.showModal();

    lockBodyScrolling(this);

    // Set focus on autocomplete if it exists
    requestAnimationFrame(() => {
      const elementToFocus =
        this.querySelector<HTMLButtonElement>("[autofocus]");
      if (elementToFocus && typeof elementToFocus.focus === "function") {
        elementToFocus.focus();
      } else {
        this.dialog.focus();
      }
    });

    await animateWithClass(this.dialog, "show");

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
          dialog: true,
          open: this.open,
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

// Ugly, but it fixes light dismiss in Safari: https://bugs.webkit.org/show_bug.cgi?id=267688
if (!isServer) {
  //
  // Watch for data-dialog="open *" clicks
  //
  document.addEventListener("click", (event: MouseEvent) => {
    const dialogAttrEl = (event.target as Element).closest("[data-dialog]");

    if (dialogAttrEl instanceof Element) {
      const [command, id] = parseSpaceDelimitedTokens(
        dialogAttrEl.getAttribute("data-dialog") || "",
      );

      if (command === "open" && id?.length) {
        const doc = dialogAttrEl.getRootNode() as Document | ShadowRoot;
        const dialog = doc.getElementById(id) as WaDialog;

        if (dialog?.localName === "wa-dialog") {
          dialog.open = true;
        } else {
          console.warn(
            `A dialog with an ID of "${id}" could not be found in this document.`,
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
    "wa-dialog": WaDialog;
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
@@ -245,51 +245,47 @@
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
import styles from "./dialog.styles.js";

/**
 * @summary Dialogs appear above the page and require the user's immediate attention. Use them for confirmations, forms,
 *  or focused tasks that interrupt the main flow.
 * @documentation https://webawesome.com/docs/components/dialog
 * @status stable
 * @since 2.0
 *
 * @dependency wa-button
 *
 * @slot - The dialog's main content.
 * @slot label - The dialog's label. Alternatively, you can use the `label` attribute.
 * @slot header-actions - Optional actions to add to the header. Works best with `<wa-button>`.
 * @slot footer - The dialog's footer, usually one or more buttons representing various options.
 *
 * @event wa-show - Emitted when the dialog opens.
 * @event wa-after-show - Emitted after the dialog opens and all animations are complete.
 * @event {{ source: Element }} wa-hide - Emitted when the dialog is requested to close. Calling
 *  `event.preventDefault()` will prevent the dialog from closing. You can inspect `event.detail.source` to see which
 *  element caused the dialog to close. If the source is the dialog element itself, the user has pressed [[Escape]] or
 *  the dialog has been closed programmatically. Avoid using this unless closing the dialog will result in destructive
 *  behavior such as data loss.
 * @event wa-after-hide - Emitted after the dialog closes and all animations are complete.
 *
 * @csspart dialog - The dialog's internal `<dialog>` element.
 * @csspart header - The dialog's header. This element wraps the title and header actions.
 * @csspart header-actions - Optional actions to add to the header. Works best with `<wa-button>`.
 * @csspart title - The dialog's title.
 * @csspart close-button - The close button, a `<wa-button>`.
 * @csspart close-button__base - The close button's exported `base` part.
 * @csspart body - The dialog's body.
 * @csspart footer - The dialog's footer.
 *
 * @cssproperty --spacing - The amount of space around and between the dialog's content.
 * @cssproperty --width - The preferred width of the dialog. Note that the dialog will shrink to accommodate smaller screens.
 * @cssproperty [--backdrop-filter=none] - A filter to apply to the backdrop behind the dialog.
 * @cssproperty [--show-duration=200ms] - The animation duration when showing the dialog.
 * @cssproperty [--hide-duration=200ms] - The animation duration when hiding the dialog.
 */
@customElement("wa-dialog")
export default class WaDialog extends WebAwesomeElement {
  static css = styles;

  private readonly localize = new LocalizeController(this);
  private readonly hasSlotController = new HasSlotController(
    this,
    "footer",
    "header-actions",
    "label",
  );
  private originalTrigger: HTMLElement | null;

  @query(".dialog") dialog: HTMLDialogElement;

  /** Indicates whether or not the dialog is open. Toggle this attribute to show and hide the dialog. */
  @property({ type: Boolean, reflect: true }) open = false;

  /**
   * The dialog's label as displayed in the header. You should always include a relevant label, as it is required for
   * proper accessibility. If you need to display HTML, use the `label` slot instead.
   */
  @property({ reflect: true }) label = "";

  /** Disables the header. This will also remove the default close button. */
  @property({ attribute: "without-header", type: Boolean, reflect: true }) withoutHeader = false;

  /** When enabled, the dialog will be closed when the user clicks outside of it. */
  @property({ attribute: "light-dismiss", type: Boolean }) lightDismiss = false;

  /**
   * Only required for SSR. Set to `true` if you're slotting in a `footer` element so the server-rendered markup
   * includes the footer before the component hydrates on the client.
   */
  @property({ attribute: "with-footer", type: Boolean }) withFooter = false;

  firstUpdated() {
    if (this.open) {
      this.addOpenListeners();
      this.dialog.showModal();
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
      animateWithClass(this.dialog, "pulse");
      return;
    }

    this.removeOpenListeners();

    await animateWithClass(this.dialog, "hide");

    this.open = false;
    this.dialog.close();
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
      !this.dialog.classList.contains("hide") &&
      event.target === this.dialog &&
      isTopDismissible(this)
    ) {
      this.requestClose(this.dialog);
    }
  }

  private handleDialogClick(event: MouseEvent) {
    const target = event.target as HTMLElement;
    const button = target.closest('[data-dialog="close"]');

    // Close when a button with [data-dialog="close"] is clicked
    if (button) {
      event.stopPropagation();
      this.requestClose(button);
    }
  }

  private async handleDialogPointerDown(event: PointerEvent) {
    // Detect when the backdrop is clicked
    if (event.target === this.dialog) {
      if (this.lightDismiss) {
        this.requestClose(this.dialog);
      } else {
        await animateWithClass(this.dialog, "pulse");
      }
    }
  }

  private handleDocumentKeyDown = (event: KeyboardEvent) => {
    if (event.key === "Escape" && this.open && isTopDismissible(this)) {
      event.preventDefault();
      event.stopPropagation();
      this.requestClose(this.dialog);
    }
  };

  @watch("open", { waitUntilFirstUpdate: true })
  handleOpenChange() {
    // Open or close the dialog
    if (this.open && !this.dialog.open) {
      this.show();
    } else if (!this.open && this.dialog.open) {
      this.open = true;
      this.requestClose(this.dialog);
    }
  }

  /** Shows the dialog. */
  private async show() {
    // Show
    const waShowEvent = new WaShowEvent();
    this.dispatchEvent(waShowEvent);
    if (waShowEvent.defaultPrevented) {
      this.open = false;
      return;
    }

    this.addOpenListeners();
    this.originalTrigger = document.activeElement as HTMLElement;
    this.open = true;
    this.dialog.showModal();

    lockBodyScrolling(this);

    // Set focus on autocomplete if it exists
    requestAnimationFrame(() => {
      const elementToFocus = this.querySelector<HTMLButtonElement>("[autofocus]");
      if (elementToFocus && typeof elementToFocus.focus === "function") {
        elementToFocus.focus();
      } else {
        this.dialog.focus();
      }
    });

    await animateWithClass(this.dialog, "show");

    this.dispatchEvent(new WaAfterShowEvent());
  }

  render() {
    const hasHeader = !this.withoutHeader;
    const hasFooter = this.hasUpdated ? this.hasSlotController.test("footer") : this.withFooter;

    return html`
      <dialog
        part="dialog"
        class=${classMap({
          dialog: true,
          open: this.open,
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

// Ugly, but it fixes light dismiss in Safari: https://bugs.webkit.org/show_bug.cgi?id=267688
if (!isServer) {
  //
  // Watch for data-dialog="open *" clicks
  //
  document.addEventListener("click", (event: MouseEvent) => {
    const dialogAttrEl = (event.target as Element).closest("[data-dialog]");

    if (dialogAttrEl instanceof Element) {
      const [command, id] = parseSpaceDelimitedTokens(
        dialogAttrEl.getAttribute("data-dialog") || "",
      );

      if (command === "open" && id?.length) {
        const doc = dialogAttrEl.getRootNode() as Document | ShadowRoot;
        const dialog = doc.getElementById(id) as WaDialog;

        if (dialog?.localName === "wa-dialog") {
          dialog.open = true;
        } else {
          console.warn(`A dialog with an ID of "${id}" could not be found in this document.`);
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
    "wa-dialog": WaDialog;
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
import styles from "./dialog.styles.js";

/**
 * @summary Dialogs appear above the page and require the user's immediate attention. Use them for confirmations, forms,
 *  or focused tasks that interrupt the main flow.
 * @documentation https://webawesome.com/docs/components/dialog
 * @status stable
 * @since 2.0
 *
 * @dependency wa-button
 *
 * @slot - The dialog's main content.
 * @slot label - The dialog's label. Alternatively, you can use the `label` attribute.
 * @slot header-actions - Optional actions to add to the header. Works best with `<wa-button>`.
 * @slot footer - The dialog's footer, usually one or more buttons representing various options.
 *
 * @event wa-show - Emitted when the dialog opens.
 * @event wa-after-show - Emitted after the dialog opens and all animations are complete.
 * @event {{ source: Element }} wa-hide - Emitted when the dialog is requested to close. Calling
 *  `event.preventDefault()` will prevent the dialog from closing. You can inspect `event.detail.source` to see which
 *  element caused the dialog to close. If the source is the dialog element itself, the user has pressed [[Escape]] or
 *  the dialog has been closed programmatically. Avoid using this unless closing the dialog will result in destructive
 *  behavior such as data loss.
 * @event wa-after-hide - Emitted after the dialog closes and all animations are complete.
 *
 * @csspart dialog - The dialog's internal `<dialog>` element.
 * @csspart header - The dialog's header. This element wraps the title and header actions.
 * @csspart header-actions - Optional actions to add to the header. Works best with `<wa-button>`.
 * @csspart title - The dialog's title.
 * @csspart close-button - The close button, a `<wa-button>`.
 * @csspart close-button__base - The close button's exported `base` part.
 * @csspart body - The dialog's body.
 * @csspart footer - The dialog's footer.
 *
 * @cssproperty --spacing - The amount of space around and between the dialog's content.
 * @cssproperty --width - The preferred width of the dialog. Note that the dialog will shrink to accommodate smaller screens.
 * @cssproperty [--backdrop-filter=none] - A filter to apply to the backdrop behind the dialog.
 * @cssproperty [--show-duration=200ms] - The animation duration when showing the dialog.
 * @cssproperty [--hide-duration=200ms] - The animation duration when hiding the dialog.
 */
@customElement("wa-dialog")
export default class WaDialog extends WebAwesomeElement {
  static css = styles;

  private readonly localize = new LocalizeController(this);
  private readonly hasSlotController = new HasSlotController(
    this,
    "footer",
    "header-actions",
    "label",
  );
  private originalTrigger: HTMLElement | null;

  @query(".dialog") dialog: HTMLDialogElement;

  /** Indicates whether or not the dialog is open. Toggle this attribute to show and hide the dialog. */
  @property({ type: Boolean, reflect: true }) open = false;

  /**
   * The dialog's label as displayed in the header. You should always include a relevant label, as it is required for
   * proper accessibility. If you need to display HTML, use the `label` slot instead.
   */
  @property({ reflect: true }) label = "";

  /** Disables the header. This will also remove the default close button. */
  @property({ attribute: "without-header", type: Boolean, reflect: true }) withoutHeader = false;

  /** When enabled, the dialog will be closed when the user clicks outside of it. */
  @property({ attribute: "light-dismiss", type: Boolean }) lightDismiss = false;

  /**
   * Only required for SSR. Set to `true` if you're slotting in a `footer` element so the server-rendered markup
   * includes the footer before the component hydrates on the client.
   */
  @property({ attribute: "with-footer", type: Boolean }) withFooter = false;

  firstUpdated() {
    if (this.open) {
      this.addOpenListeners();
      this.dialog.showModal();
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
      animateWithClass(this.dialog, "pulse");
      return;
    }

    this.removeOpenListeners();

    await animateWithClass(this.dialog, "hide");

    this.open = false;
    this.dialog.close();
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
      !this.dialog.classList.contains("hide") &&
      event.target === this.dialog &&
      isTopDismissible(this)
    ) {
      this.requestClose(this.dialog);
    }
  }

  private handleDialogClick(event: MouseEvent) {
    const target = event.target as HTMLElement;
    const button = target.closest('[data-dialog="close"]');

    // Close when a button with [data-dialog="close"] is clicked
    if (button) {
      event.stopPropagation();
      this.requestClose(button);
    }
  }

  private async handleDialogPointerDown(event: PointerEvent) {
    // Detect when the backdrop is clicked
    if (event.target === this.dialog) {
      if (this.lightDismiss) {
        this.requestClose(this.dialog);
      } else {
        await animateWithClass(this.dialog, "pulse");
      }
    }
  }

  private handleDocumentKeyDown = (event: KeyboardEvent) => {
    if (event.key === "Escape" && this.open && isTopDismissible(this)) {
      event.preventDefault();
      event.stopPropagation();
      this.requestClose(this.dialog);
    }
  };

  @watch("open", { waitUntilFirstUpdate: true })
  handleOpenChange() {
    // Open or close the dialog
    if (this.open && !this.dialog.open) {
      this.show();
    } else if (!this.open && this.dialog.open) {
      this.open = true;
      this.requestClose(this.dialog);
    }
  }

  /** Shows the dialog. */
  private async show() {
    // Show
    const waShowEvent = new WaShowEvent();
    this.dispatchEvent(waShowEvent);
    if (waShowEvent.defaultPrevented) {
      this.open = false;
      return;
    }

    this.addOpenListeners();
    this.originalTrigger = document.activeElement as HTMLElement;
    this.open = true;
    this.dialog.showModal();

    lockBodyScrolling(this);

    // Set focus on autocomplete if it exists
    requestAnimationFrame(() => {
      const elementToFocus = this.querySelector<HTMLButtonElement>("[autofocus]");
      if (elementToFocus && typeof elementToFocus.focus === "function") {
        elementToFocus.focus();
      } else {
        this.dialog.focus();
      }
    });

    await animateWithClass(this.dialog, "show");

    this.dispatchEvent(new WaAfterShowEvent());
  }

  render() {
    const hasHeader = !this.withoutHeader;
    const hasFooter = this.hasUpdated ? this.hasSlotController.test("footer") : this.withFooter;

    return html`
      <dialog
        part="dialog"
        class=${classMap({
          dialog: true,
          open: this.open,
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

// Ugly, but it fixes light dismiss in Safari: https://bugs.webkit.org/show_bug.cgi?id=267688
if (!isServer) {
  //
  // Watch for data-dialog="open *" clicks
  //
  document.addEventListener("click", (event: MouseEvent) => {
    const dialogAttrEl = (event.target as Element).closest("[data-dialog]");

    if (dialogAttrEl instanceof Element) {
      const [command, id] = parseSpaceDelimitedTokens(
        dialogAttrEl.getAttribute("data-dialog") || "",
      );

      if (command === "open" && id?.length) {
        const doc = dialogAttrEl.getRootNode() as Document | ShadowRoot;
        const dialog = doc.getElementById(id) as WaDialog;

        if (dialog?.localName === "wa-dialog") {
          dialog.open = true;
        } else {
          console.warn(`A dialog with an ID of "${id}" could not be found in this document.`);
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
    "wa-dialog": WaDialog;
  }
}

`````
