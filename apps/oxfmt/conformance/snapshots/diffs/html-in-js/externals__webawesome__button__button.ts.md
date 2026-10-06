# externals/webawesome/button/button.ts

## Option 1

`````json
{"printWidth":80}
`````

### Diff

`````diff
===================================================================
--- prettier
+++ oxfmt
@@ -76,17 +76,32 @@
   @property() title = ""; // make reactive to pass through
 
   /** The button's theme variant. Defaults to `neutral` if not within another element with a variant. */
   @property({ reflect: true }) variant:
-    "neutral" | "brand" | "success" | "warning" | "danger" = "neutral";
+    | "neutral"
+    | "brand"
+    | "success"
+    | "warning"
+    | "danger" = "neutral";
 
   /** The button's visual appearance. */
   @property({ reflect: true }) appearance:
-    "accent" | "filled" | "outlined" | "filled-outlined" | "plain" = "accent";
+    | "accent"
+    | "filled"
+    | "outlined"
+    | "filled-outlined"
+    | "plain" = "accent";
 
   /** The button's size. */
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
@@ -157,9 +172,11 @@
 
   /** Used to override the form owner's `enctype` attribute.  */
   @property({ attribute: "formenctype" })
   formEnctype:
-    "application/x-www-form-urlencoded" | "multipart/form-data" | "text/plain";
+    | "application/x-www-form-urlencoded"
+    | "multipart/form-data"
+    | "text/plain";
 
   /** Used to override the form owner's `method` attribute.  */
   @property({ attribute: "formmethod" }) formMethod: "post" | "get";
 
@@ -168,9 +185,13 @@
   formNoValidate: boolean;
 
   /** Used to override the form owner's `target` attribute. */
   @property({ attribute: "formtarget" }) formTarget:
-    "_self" | "_blank" | "_parent" | "_top" | string;
+    | "_self"
+    | "_blank"
+    | "_parent"
+    | "_top"
+    | string;
 
   private constructLightDOMButton() {
     const button = document.createElement("button");
 

`````

### Actual (oxfmt)

`````ts
import { customElement, property, query, state } from "lit/decorators.js";
import { classMap } from "lit/directives/class-map.js";
import { ifDefined } from "lit/directives/if-defined.js";
import { html, literal } from "lit/static-html.js";
import { WaInvalidEvent } from "../../events/invalid.js";
import { warnDeprecatedSize } from "../../internal/size.js";
import { HasSlotController } from "../../internal/slot.js";
import { MirrorValidator } from "../../internal/validators/mirror-validator.js";
import { watch } from "../../internal/watch.js";
import { WebAwesomeFormAssociatedElement } from "../../internal/webawesome-form-associated-element.js";
import sizeStyles from "../../styles/component/size.styles.js";
import variantStyles from "../../styles/component/variants.styles.js";
import { LocalizeController } from "../../utilities/localize.js";
import "../icon/icon.js";
import type WaIcon from "../icon/icon.js";
import "../spinner/spinner.js";
import styles from "./button.styles.js";

/**
 * @summary Buttons represent actions the user can take, such as submitting a form, opening a dialog, or navigating to
 *  another page.
 * @documentation https://webawesome.com/docs/components/button
 * @status stable
 * @since 2.0
 *
 * @dependency wa-icon
 * @dependency wa-spinner
 *
 * @event blur - Emitted when the button loses focus.
 * @event focus - Emitted when the button gains focus.
 * @event wa-invalid - Emitted when the form control has been checked for validity and its constraints aren't satisfied.
 *
 * @slot - The button's label.
 * @slot start - An element, such as `<wa-icon>`, placed before the label.
 * @slot end - An element, such as `<wa-icon>`, placed after the label.
 *
 * @csspart base - The component's base wrapper.
 * @csspart start - The container that wraps the `start` slot.
 * @csspart label - The button's label.
 * @csspart end - The container that wraps the `end` slot.
 * @csspart caret - The button's caret icon, a `<wa-icon>` element.
 * @csspart spinner - The spinner that shows when the button is in the loading state.
 *
 * @cssstate disabled - Applied when the button is disabled.
 * @cssstate icon-button - Applied when the button contains only a `<wa-icon>` with no other content.
 * @cssstate link - Applied when the button is rendered as a link (i.e. `href` is set).
 * @cssstate loading - Applied when the button is in the loading state.
 */
@customElement("wa-button")
export default class WaButton extends WebAwesomeFormAssociatedElement {
  static shadowRootOptions = {
    ...WebAwesomeFormAssociatedElement.shadowRootOptions,
    delegatesFocus: true,
  };
  static css = [styles, variantStyles, sizeStyles];

  static get validators() {
    return [...super.validators, MirrorValidator()];
  }

  assumeInteractionOn = ["click"];
  private readonly hasSlotController = new HasSlotController(
    this,
    "[default]",
    "start",
    "end",
  );
  private readonly localize = new LocalizeController(this);

  @query(".button") button: HTMLButtonElement | HTMLLinkElement;
  @query("slot:not([name])") labelSlot: HTMLSlotElement;

  @state() invalid = false;
  @state() isIconButton = false;

  @property() title = ""; // make reactive to pass through

  /** The button's theme variant. Defaults to `neutral` if not within another element with a variant. */
  @property({ reflect: true }) variant:
    | "neutral"
    | "brand"
    | "success"
    | "warning"
    | "danger" = "neutral";

  /** The button's visual appearance. */
  @property({ reflect: true }) appearance:
    | "accent"
    | "filled"
    | "outlined"
    | "filled-outlined"
    | "plain" = "accent";

  /** The button's size. */
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

  /** Draws the button with a caret. Used to indicate that the button triggers a dropdown menu or similar behavior. */
  @property({ attribute: "with-caret", type: Boolean, reflect: true })
  withCaret = false;

  /**
   * Only required for SSR. Set to `true` if you're slotting in a `start` element so the server-rendered markup
   * includes the start slot before the component hydrates on the client.
   */
  @property({ attribute: "with-start", type: Boolean }) withStart = false;

  /**
   * Only required for SSR. Set to `true` if you're slotting in an `end` element so the server-rendered markup
   * includes the end slot before the component hydrates on the client.
   */
  @property({ attribute: "with-end", type: Boolean }) withEnd = false;

  /** Disables the button. */
  @property({ type: Boolean }) disabled = false;

  /** Draws the button in a loading state. */
  @property({ type: Boolean, reflect: true }) loading = false;

  /** Draws a pill-style button with rounded edges. */
  @property({ type: Boolean, reflect: true }) pill = false;

  /**
   * The type of button. Note that the default value is `button` instead of `submit`, which is opposite of how native
   * `<button>` elements behave. When the type is `submit`, the button will submit the surrounding form.
   */
  @property() type: "button" | "submit" | "reset" = "button";

  /**
   * The name of the button, submitted as a name/value pair with form data, but only when this button is the submitter.
   * This attribute is ignored when `href` is present.
   */
  @property({ reflect: true }) name: string;

  /**
   * The value of the button, submitted as a pair with the button's name as part of the form data, but only when this
   * button is the submitter. This attribute is ignored when `href` is present.
   */
  @property({ reflect: true }) value: string;

  /** When set, the underlying button will be rendered as an `<a>` with this `href` instead of a `<button>`. */
  @property({ reflect: true }) href: string;

  /** Tells the browser where to open the link. Only used when `href` is present. */
  @property() target: "_blank" | "_parent" | "_self" | "_top";

  /** When using `href`, this attribute will map to the underlying link's `rel` attribute. */
  @property() rel?: string;

  /** Tells the browser to download the linked file as this filename. Only used when `href` is present. */
  @property() download?: string;

  /**
   * The "form owner" to associate the button with. If omitted, the closest containing form will be used instead. The
   * value of this attribute must be an id of a form in the same document or shadow root as the button.
   */

  /** Used to override the form owner's `action` attribute. */
  @property({ attribute: "formaction" }) formAction: string;

  /** Used to override the form owner's `enctype` attribute.  */
  @property({ attribute: "formenctype" })
  formEnctype:
    | "application/x-www-form-urlencoded"
    | "multipart/form-data"
    | "text/plain";

  /** Used to override the form owner's `method` attribute.  */
  @property({ attribute: "formmethod" }) formMethod: "post" | "get";

  /** Used to override the form owner's `novalidate` attribute. */
  @property({ attribute: "formnovalidate", type: Boolean })
  formNoValidate: boolean;

  /** Used to override the form owner's `target` attribute. */
  @property({ attribute: "formtarget" }) formTarget:
    | "_self"
    | "_blank"
    | "_parent"
    | "_top"
    | string;

  private constructLightDOMButton() {
    const button = document.createElement("button");

    for (const attribute of this.attributes) {
      if (attribute.name === "style") {
        // Skip style attributes as they *shouldn't* be necessary
        continue;
      }
      button.setAttribute(attribute.name, attribute.value);
    }

    button.type = this.type;
    button.style.position = "absolute !important";
    button.style.width = "0 !important";
    button.style.height = "0 !important";
    button.style.clipPath = "inset(50%) !important";
    button.style.overflow = "hidden !important";
    button.style.whiteSpace = "nowrap !important";
    if (this.name) {
      button.name = this.name;
    }
    button.value = this.value || "";

    return button;
  }

  private handleClick(event: PointerEvent) {
    // Prevent disabled and loading buttons from being clicked
    if (this.disabled || this.loading) {
      event.preventDefault();
      event.stopImmediatePropagation();
      return;
    }

    // Only create a light dom button for submit / reset buttons.
    if (this.type !== "submit" && this.type !== "reset") {
      return;
    }

    const form = this.getForm();

    if (!form) return;

    const lightDOMButton = this.constructLightDOMButton();

    // form.append(lightDOMButton);
    this.parentElement?.append(lightDOMButton);
    lightDOMButton.click();
    lightDOMButton.remove();
  }

  private handleInvalid() {
    this.dispatchEvent(new WaInvalidEvent());
  }

  private handleLabelSlotChange() {
    const nodes = this.labelSlot.assignedNodes({ flatten: true });
    let hasIconLabel = false;
    let hasIcon = false;
    let hasText = false;
    let hasOtherElements = false;

    // Check all slotted nodes
    [...nodes].forEach((node) => {
      if (node.nodeType === Node.ELEMENT_NODE) {
        const element = node as HTMLElement;

        if (element.localName === "wa-icon") {
          hasIcon = true;
          if (!hasIconLabel)
            hasIconLabel = (element as WaIcon).label !== undefined;
        } else {
          // Any other element type means it's not an icon button
          hasOtherElements = true;
        }
      } else if (node.nodeType === Node.TEXT_NODE) {
        // Check if text node has actual content
        const text = node.textContent?.trim() || "";
        if (text.length > 0) {
          hasText = true;
        }
      }
    });

    // It's only an icon button if there's an icon and nothing else
    this.isIconButton = hasIcon && !hasText && !hasOtherElements;
    this.customStates.set("icon-button", this.isIconButton);

    if (this.isIconButton && !hasIconLabel) {
      console.warn(
        'Icon buttons must have a label for screen readers. Add <wa-icon label="..."> to remove this warning.',
        this,
      );
    }
  }

  private isButton() {
    return this.href ? false : true;
  }

  private isLink() {
    return this.href ? true : false;
  }

  @watch("disabled", { waitUntilFirstUpdate: true })
  handleDisabledChange() {
    this.customStates.set("disabled", this.disabled);
    this.updateValidity();
  }

  @watch("href")
  handleHrefChange() {
    this.customStates.set("link", this.isLink());
  }

  @watch("loading", { waitUntilFirstUpdate: true })
  handleLoadingChange() {
    this.customStates.set("loading", this.loading);
  }

  // eslint-disable-next-line
  setValue(..._args: Parameters<WebAwesomeFormAssociatedElement["setValue"]>) {
    // This is just a stub. We don't ever actually want to set a value on the form. That happens when the button is clicked and added
    // via the light dom button.
  }

  /** Simulates a click on the button. */
  click() {
    this.button.click();
  }

  /** Sets focus on the button. */
  focus(options?: FocusOptions) {
    this.button.focus(options);
  }

  /** Removes focus from the button. */
  blur() {
    this.button.blur();
  }

  render() {
    const isLink = this.isLink();
    const tag = isLink ? literal`a` : literal`button`;

    /* eslint-disable lit/no-invalid-html */
    /* eslint-disable lit/binding-positions */
    return html`
      <${tag}
        part="base"
        class=${classMap({
          button: true,
          caret: this.withCaret,
          disabled: this.disabled,
          loading: this.loading,
          rtl: this.localize.dir() === "rtl",
          "has-label": this.hasSlotController.test("[default]"),
          "has-start": this.hasUpdated
            ? this.hasSlotController.test("start")
            : this.withStart,
          "has-end": this.hasUpdated
            ? this.hasSlotController.test("end")
            : this.withEnd,
          "is-icon-button": this.isIconButton,
        })}
        ?disabled=${ifDefined(isLink ? undefined : this.disabled)}
        type=${ifDefined(isLink ? undefined : this.type)}
        title=${this.title /* An empty title prevents browser validation tooltips from appearing on hover */}
        name=${ifDefined(isLink ? undefined : this.name)}
        value=${ifDefined(isLink ? undefined : this.value)}
        href=${ifDefined(isLink ? this.href : undefined)}
        target=${ifDefined(isLink ? this.target : undefined)}
        download=${ifDefined(isLink ? this.download : undefined)}
        rel=${ifDefined(isLink && this.rel ? this.rel : undefined)}
        role=${ifDefined(isLink ? undefined : "button")}
        aria-disabled=${ifDefined(isLink && this.disabled ? "true" : undefined)}
        tabindex=${this.disabled ? "-1" : "0"}
        @invalid=${this.isButton() ? this.handleInvalid : null}
        @click=${this.handleClick}
      >
        <slot name="start" part="start" class="start"></slot>
        <slot part="label" class="label" @slotchange=${this.handleLabelSlotChange}></slot>
        <slot name="end" part="end" class="end"></slot>
        ${
          this.withCaret
            ? html`
                <wa-icon
                  part="caret"
                  class="caret"
                  library="system"
                  name="chevron-down"
                  variant="solid"
                ></wa-icon>
              `
            : ""
        }
        ${this.loading ? html`<wa-spinner part="spinner"></wa-spinner>` : ""}
      </${tag}>
    `;
  }
}

// The change-in-update warning is required for this component because the form-associated base class calls
// updateValidity() in firstUpdated(), which triggers requestUpdate('validity') to sync the validation state after the
// first render when the validation target is available. Additionally, HasSlotController triggers requestUpdate() on
// initial slotchange events, and the label slot's slotchange handler sets the isIconButton state property to detect
// icon-only buttons after slot content is available. See https://lit.dev/docs/tools/development/#development-build-runtime-warnings
WaButton.disableWarning?.("change-in-update");

declare global {
  interface HTMLElementTagNameMap {
    "wa-button": WaButton;
  }
}

`````

### Expected (prettier)

`````ts
import { customElement, property, query, state } from "lit/decorators.js";
import { classMap } from "lit/directives/class-map.js";
import { ifDefined } from "lit/directives/if-defined.js";
import { html, literal } from "lit/static-html.js";
import { WaInvalidEvent } from "../../events/invalid.js";
import { warnDeprecatedSize } from "../../internal/size.js";
import { HasSlotController } from "../../internal/slot.js";
import { MirrorValidator } from "../../internal/validators/mirror-validator.js";
import { watch } from "../../internal/watch.js";
import { WebAwesomeFormAssociatedElement } from "../../internal/webawesome-form-associated-element.js";
import sizeStyles from "../../styles/component/size.styles.js";
import variantStyles from "../../styles/component/variants.styles.js";
import { LocalizeController } from "../../utilities/localize.js";
import "../icon/icon.js";
import type WaIcon from "../icon/icon.js";
import "../spinner/spinner.js";
import styles from "./button.styles.js";

/**
 * @summary Buttons represent actions the user can take, such as submitting a form, opening a dialog, or navigating to
 *  another page.
 * @documentation https://webawesome.com/docs/components/button
 * @status stable
 * @since 2.0
 *
 * @dependency wa-icon
 * @dependency wa-spinner
 *
 * @event blur - Emitted when the button loses focus.
 * @event focus - Emitted when the button gains focus.
 * @event wa-invalid - Emitted when the form control has been checked for validity and its constraints aren't satisfied.
 *
 * @slot - The button's label.
 * @slot start - An element, such as `<wa-icon>`, placed before the label.
 * @slot end - An element, such as `<wa-icon>`, placed after the label.
 *
 * @csspart base - The component's base wrapper.
 * @csspart start - The container that wraps the `start` slot.
 * @csspart label - The button's label.
 * @csspart end - The container that wraps the `end` slot.
 * @csspart caret - The button's caret icon, a `<wa-icon>` element.
 * @csspart spinner - The spinner that shows when the button is in the loading state.
 *
 * @cssstate disabled - Applied when the button is disabled.
 * @cssstate icon-button - Applied when the button contains only a `<wa-icon>` with no other content.
 * @cssstate link - Applied when the button is rendered as a link (i.e. `href` is set).
 * @cssstate loading - Applied when the button is in the loading state.
 */
@customElement("wa-button")
export default class WaButton extends WebAwesomeFormAssociatedElement {
  static shadowRootOptions = {
    ...WebAwesomeFormAssociatedElement.shadowRootOptions,
    delegatesFocus: true,
  };
  static css = [styles, variantStyles, sizeStyles];

  static get validators() {
    return [...super.validators, MirrorValidator()];
  }

  assumeInteractionOn = ["click"];
  private readonly hasSlotController = new HasSlotController(
    this,
    "[default]",
    "start",
    "end",
  );
  private readonly localize = new LocalizeController(this);

  @query(".button") button: HTMLButtonElement | HTMLLinkElement;
  @query("slot:not([name])") labelSlot: HTMLSlotElement;

  @state() invalid = false;
  @state() isIconButton = false;

  @property() title = ""; // make reactive to pass through

  /** The button's theme variant. Defaults to `neutral` if not within another element with a variant. */
  @property({ reflect: true }) variant:
    "neutral" | "brand" | "success" | "warning" | "danger" = "neutral";

  /** The button's visual appearance. */
  @property({ reflect: true }) appearance:
    "accent" | "filled" | "outlined" | "filled-outlined" | "plain" = "accent";

  /** The button's size. */
  @property({ reflect: true }) size:
    "xs" | "s" | "m" | "l" | "xl" | "small" | "medium" | "large" = "m";

  @watch("size")
  handleSizeChange() {
    warnDeprecatedSize(this.localName, this.size);
  }

  /** Draws the button with a caret. Used to indicate that the button triggers a dropdown menu or similar behavior. */
  @property({ attribute: "with-caret", type: Boolean, reflect: true })
  withCaret = false;

  /**
   * Only required for SSR. Set to `true` if you're slotting in a `start` element so the server-rendered markup
   * includes the start slot before the component hydrates on the client.
   */
  @property({ attribute: "with-start", type: Boolean }) withStart = false;

  /**
   * Only required for SSR. Set to `true` if you're slotting in an `end` element so the server-rendered markup
   * includes the end slot before the component hydrates on the client.
   */
  @property({ attribute: "with-end", type: Boolean }) withEnd = false;

  /** Disables the button. */
  @property({ type: Boolean }) disabled = false;

  /** Draws the button in a loading state. */
  @property({ type: Boolean, reflect: true }) loading = false;

  /** Draws a pill-style button with rounded edges. */
  @property({ type: Boolean, reflect: true }) pill = false;

  /**
   * The type of button. Note that the default value is `button` instead of `submit`, which is opposite of how native
   * `<button>` elements behave. When the type is `submit`, the button will submit the surrounding form.
   */
  @property() type: "button" | "submit" | "reset" = "button";

  /**
   * The name of the button, submitted as a name/value pair with form data, but only when this button is the submitter.
   * This attribute is ignored when `href` is present.
   */
  @property({ reflect: true }) name: string;

  /**
   * The value of the button, submitted as a pair with the button's name as part of the form data, but only when this
   * button is the submitter. This attribute is ignored when `href` is present.
   */
  @property({ reflect: true }) value: string;

  /** When set, the underlying button will be rendered as an `<a>` with this `href` instead of a `<button>`. */
  @property({ reflect: true }) href: string;

  /** Tells the browser where to open the link. Only used when `href` is present. */
  @property() target: "_blank" | "_parent" | "_self" | "_top";

  /** When using `href`, this attribute will map to the underlying link's `rel` attribute. */
  @property() rel?: string;

  /** Tells the browser to download the linked file as this filename. Only used when `href` is present. */
  @property() download?: string;

  /**
   * The "form owner" to associate the button with. If omitted, the closest containing form will be used instead. The
   * value of this attribute must be an id of a form in the same document or shadow root as the button.
   */

  /** Used to override the form owner's `action` attribute. */
  @property({ attribute: "formaction" }) formAction: string;

  /** Used to override the form owner's `enctype` attribute.  */
  @property({ attribute: "formenctype" })
  formEnctype:
    "application/x-www-form-urlencoded" | "multipart/form-data" | "text/plain";

  /** Used to override the form owner's `method` attribute.  */
  @property({ attribute: "formmethod" }) formMethod: "post" | "get";

  /** Used to override the form owner's `novalidate` attribute. */
  @property({ attribute: "formnovalidate", type: Boolean })
  formNoValidate: boolean;

  /** Used to override the form owner's `target` attribute. */
  @property({ attribute: "formtarget" }) formTarget:
    "_self" | "_blank" | "_parent" | "_top" | string;

  private constructLightDOMButton() {
    const button = document.createElement("button");

    for (const attribute of this.attributes) {
      if (attribute.name === "style") {
        // Skip style attributes as they *shouldn't* be necessary
        continue;
      }
      button.setAttribute(attribute.name, attribute.value);
    }

    button.type = this.type;
    button.style.position = "absolute !important";
    button.style.width = "0 !important";
    button.style.height = "0 !important";
    button.style.clipPath = "inset(50%) !important";
    button.style.overflow = "hidden !important";
    button.style.whiteSpace = "nowrap !important";
    if (this.name) {
      button.name = this.name;
    }
    button.value = this.value || "";

    return button;
  }

  private handleClick(event: PointerEvent) {
    // Prevent disabled and loading buttons from being clicked
    if (this.disabled || this.loading) {
      event.preventDefault();
      event.stopImmediatePropagation();
      return;
    }

    // Only create a light dom button for submit / reset buttons.
    if (this.type !== "submit" && this.type !== "reset") {
      return;
    }

    const form = this.getForm();

    if (!form) return;

    const lightDOMButton = this.constructLightDOMButton();

    // form.append(lightDOMButton);
    this.parentElement?.append(lightDOMButton);
    lightDOMButton.click();
    lightDOMButton.remove();
  }

  private handleInvalid() {
    this.dispatchEvent(new WaInvalidEvent());
  }

  private handleLabelSlotChange() {
    const nodes = this.labelSlot.assignedNodes({ flatten: true });
    let hasIconLabel = false;
    let hasIcon = false;
    let hasText = false;
    let hasOtherElements = false;

    // Check all slotted nodes
    [...nodes].forEach((node) => {
      if (node.nodeType === Node.ELEMENT_NODE) {
        const element = node as HTMLElement;

        if (element.localName === "wa-icon") {
          hasIcon = true;
          if (!hasIconLabel)
            hasIconLabel = (element as WaIcon).label !== undefined;
        } else {
          // Any other element type means it's not an icon button
          hasOtherElements = true;
        }
      } else if (node.nodeType === Node.TEXT_NODE) {
        // Check if text node has actual content
        const text = node.textContent?.trim() || "";
        if (text.length > 0) {
          hasText = true;
        }
      }
    });

    // It's only an icon button if there's an icon and nothing else
    this.isIconButton = hasIcon && !hasText && !hasOtherElements;
    this.customStates.set("icon-button", this.isIconButton);

    if (this.isIconButton && !hasIconLabel) {
      console.warn(
        'Icon buttons must have a label for screen readers. Add <wa-icon label="..."> to remove this warning.',
        this,
      );
    }
  }

  private isButton() {
    return this.href ? false : true;
  }

  private isLink() {
    return this.href ? true : false;
  }

  @watch("disabled", { waitUntilFirstUpdate: true })
  handleDisabledChange() {
    this.customStates.set("disabled", this.disabled);
    this.updateValidity();
  }

  @watch("href")
  handleHrefChange() {
    this.customStates.set("link", this.isLink());
  }

  @watch("loading", { waitUntilFirstUpdate: true })
  handleLoadingChange() {
    this.customStates.set("loading", this.loading);
  }

  // eslint-disable-next-line
  setValue(..._args: Parameters<WebAwesomeFormAssociatedElement["setValue"]>) {
    // This is just a stub. We don't ever actually want to set a value on the form. That happens when the button is clicked and added
    // via the light dom button.
  }

  /** Simulates a click on the button. */
  click() {
    this.button.click();
  }

  /** Sets focus on the button. */
  focus(options?: FocusOptions) {
    this.button.focus(options);
  }

  /** Removes focus from the button. */
  blur() {
    this.button.blur();
  }

  render() {
    const isLink = this.isLink();
    const tag = isLink ? literal`a` : literal`button`;

    /* eslint-disable lit/no-invalid-html */
    /* eslint-disable lit/binding-positions */
    return html`
      <${tag}
        part="base"
        class=${classMap({
          button: true,
          caret: this.withCaret,
          disabled: this.disabled,
          loading: this.loading,
          rtl: this.localize.dir() === "rtl",
          "has-label": this.hasSlotController.test("[default]"),
          "has-start": this.hasUpdated
            ? this.hasSlotController.test("start")
            : this.withStart,
          "has-end": this.hasUpdated
            ? this.hasSlotController.test("end")
            : this.withEnd,
          "is-icon-button": this.isIconButton,
        })}
        ?disabled=${ifDefined(isLink ? undefined : this.disabled)}
        type=${ifDefined(isLink ? undefined : this.type)}
        title=${this.title /* An empty title prevents browser validation tooltips from appearing on hover */}
        name=${ifDefined(isLink ? undefined : this.name)}
        value=${ifDefined(isLink ? undefined : this.value)}
        href=${ifDefined(isLink ? this.href : undefined)}
        target=${ifDefined(isLink ? this.target : undefined)}
        download=${ifDefined(isLink ? this.download : undefined)}
        rel=${ifDefined(isLink && this.rel ? this.rel : undefined)}
        role=${ifDefined(isLink ? undefined : "button")}
        aria-disabled=${ifDefined(isLink && this.disabled ? "true" : undefined)}
        tabindex=${this.disabled ? "-1" : "0"}
        @invalid=${this.isButton() ? this.handleInvalid : null}
        @click=${this.handleClick}
      >
        <slot name="start" part="start" class="start"></slot>
        <slot part="label" class="label" @slotchange=${this.handleLabelSlotChange}></slot>
        <slot name="end" part="end" class="end"></slot>
        ${
          this.withCaret
            ? html`
                <wa-icon
                  part="caret"
                  class="caret"
                  library="system"
                  name="chevron-down"
                  variant="solid"
                ></wa-icon>
              `
            : ""
        }
        ${this.loading ? html`<wa-spinner part="spinner"></wa-spinner>` : ""}
      </${tag}>
    `;
  }
}

// The change-in-update warning is required for this component because the form-associated base class calls
// updateValidity() in firstUpdated(), which triggers requestUpdate('validity') to sync the validation state after the
// first render when the validation target is available. Additionally, HasSlotController triggers requestUpdate() on
// initial slotchange events, and the label slot's slotchange handler sets the isIconButton state property to detect
// icon-only buttons after slot content is available. See https://lit.dev/docs/tools/development/#development-build-runtime-warnings
WaButton.disableWarning?.("change-in-update");

declare global {
  interface HTMLElementTagNameMap {
    "wa-button": WaButton;
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
@@ -75,9 +75,13 @@
     "neutral";
 
   /** The button's visual appearance. */
   @property({ reflect: true }) appearance:
-    "accent" | "filled" | "outlined" | "filled-outlined" | "plain" = "accent";
+    | "accent"
+    | "filled"
+    | "outlined"
+    | "filled-outlined"
+    | "plain" = "accent";
 
   /** The button's size. */
   @property({ reflect: true }) size: "xs" | "s" | "m" | "l" | "xl" | "small" | "medium" | "large" =
     "m";
@@ -160,9 +164,13 @@
   @property({ attribute: "formnovalidate", type: Boolean }) formNoValidate: boolean;
 
   /** Used to override the form owner's `target` attribute. */
   @property({ attribute: "formtarget" }) formTarget:
-    "_self" | "_blank" | "_parent" | "_top" | string;
+    | "_self"
+    | "_blank"
+    | "_parent"
+    | "_top"
+    | string;
 
   private constructLightDOMButton() {
     const button = document.createElement("button");
 

`````

### Actual (oxfmt)

`````ts
import { customElement, property, query, state } from "lit/decorators.js";
import { classMap } from "lit/directives/class-map.js";
import { ifDefined } from "lit/directives/if-defined.js";
import { html, literal } from "lit/static-html.js";
import { WaInvalidEvent } from "../../events/invalid.js";
import { warnDeprecatedSize } from "../../internal/size.js";
import { HasSlotController } from "../../internal/slot.js";
import { MirrorValidator } from "../../internal/validators/mirror-validator.js";
import { watch } from "../../internal/watch.js";
import { WebAwesomeFormAssociatedElement } from "../../internal/webawesome-form-associated-element.js";
import sizeStyles from "../../styles/component/size.styles.js";
import variantStyles from "../../styles/component/variants.styles.js";
import { LocalizeController } from "../../utilities/localize.js";
import "../icon/icon.js";
import type WaIcon from "../icon/icon.js";
import "../spinner/spinner.js";
import styles from "./button.styles.js";

/**
 * @summary Buttons represent actions the user can take, such as submitting a form, opening a dialog, or navigating to
 *  another page.
 * @documentation https://webawesome.com/docs/components/button
 * @status stable
 * @since 2.0
 *
 * @dependency wa-icon
 * @dependency wa-spinner
 *
 * @event blur - Emitted when the button loses focus.
 * @event focus - Emitted when the button gains focus.
 * @event wa-invalid - Emitted when the form control has been checked for validity and its constraints aren't satisfied.
 *
 * @slot - The button's label.
 * @slot start - An element, such as `<wa-icon>`, placed before the label.
 * @slot end - An element, such as `<wa-icon>`, placed after the label.
 *
 * @csspart base - The component's base wrapper.
 * @csspart start - The container that wraps the `start` slot.
 * @csspart label - The button's label.
 * @csspart end - The container that wraps the `end` slot.
 * @csspart caret - The button's caret icon, a `<wa-icon>` element.
 * @csspart spinner - The spinner that shows when the button is in the loading state.
 *
 * @cssstate disabled - Applied when the button is disabled.
 * @cssstate icon-button - Applied when the button contains only a `<wa-icon>` with no other content.
 * @cssstate link - Applied when the button is rendered as a link (i.e. `href` is set).
 * @cssstate loading - Applied when the button is in the loading state.
 */
@customElement("wa-button")
export default class WaButton extends WebAwesomeFormAssociatedElement {
  static shadowRootOptions = {
    ...WebAwesomeFormAssociatedElement.shadowRootOptions,
    delegatesFocus: true,
  };
  static css = [styles, variantStyles, sizeStyles];

  static get validators() {
    return [...super.validators, MirrorValidator()];
  }

  assumeInteractionOn = ["click"];
  private readonly hasSlotController = new HasSlotController(this, "[default]", "start", "end");
  private readonly localize = new LocalizeController(this);

  @query(".button") button: HTMLButtonElement | HTMLLinkElement;
  @query("slot:not([name])") labelSlot: HTMLSlotElement;

  @state() invalid = false;
  @state() isIconButton = false;

  @property() title = ""; // make reactive to pass through

  /** The button's theme variant. Defaults to `neutral` if not within another element with a variant. */
  @property({ reflect: true }) variant: "neutral" | "brand" | "success" | "warning" | "danger" =
    "neutral";

  /** The button's visual appearance. */
  @property({ reflect: true }) appearance:
    | "accent"
    | "filled"
    | "outlined"
    | "filled-outlined"
    | "plain" = "accent";

  /** The button's size. */
  @property({ reflect: true }) size: "xs" | "s" | "m" | "l" | "xl" | "small" | "medium" | "large" =
    "m";

  @watch("size")
  handleSizeChange() {
    warnDeprecatedSize(this.localName, this.size);
  }

  /** Draws the button with a caret. Used to indicate that the button triggers a dropdown menu or similar behavior. */
  @property({ attribute: "with-caret", type: Boolean, reflect: true }) withCaret = false;

  /**
   * Only required for SSR. Set to `true` if you're slotting in a `start` element so the server-rendered markup
   * includes the start slot before the component hydrates on the client.
   */
  @property({ attribute: "with-start", type: Boolean }) withStart = false;

  /**
   * Only required for SSR. Set to `true` if you're slotting in an `end` element so the server-rendered markup
   * includes the end slot before the component hydrates on the client.
   */
  @property({ attribute: "with-end", type: Boolean }) withEnd = false;

  /** Disables the button. */
  @property({ type: Boolean }) disabled = false;

  /** Draws the button in a loading state. */
  @property({ type: Boolean, reflect: true }) loading = false;

  /** Draws a pill-style button with rounded edges. */
  @property({ type: Boolean, reflect: true }) pill = false;

  /**
   * The type of button. Note that the default value is `button` instead of `submit`, which is opposite of how native
   * `<button>` elements behave. When the type is `submit`, the button will submit the surrounding form.
   */
  @property() type: "button" | "submit" | "reset" = "button";

  /**
   * The name of the button, submitted as a name/value pair with form data, but only when this button is the submitter.
   * This attribute is ignored when `href` is present.
   */
  @property({ reflect: true }) name: string;

  /**
   * The value of the button, submitted as a pair with the button's name as part of the form data, but only when this
   * button is the submitter. This attribute is ignored when `href` is present.
   */
  @property({ reflect: true }) value: string;

  /** When set, the underlying button will be rendered as an `<a>` with this `href` instead of a `<button>`. */
  @property({ reflect: true }) href: string;

  /** Tells the browser where to open the link. Only used when `href` is present. */
  @property() target: "_blank" | "_parent" | "_self" | "_top";

  /** When using `href`, this attribute will map to the underlying link's `rel` attribute. */
  @property() rel?: string;

  /** Tells the browser to download the linked file as this filename. Only used when `href` is present. */
  @property() download?: string;

  /**
   * The "form owner" to associate the button with. If omitted, the closest containing form will be used instead. The
   * value of this attribute must be an id of a form in the same document or shadow root as the button.
   */

  /** Used to override the form owner's `action` attribute. */
  @property({ attribute: "formaction" }) formAction: string;

  /** Used to override the form owner's `enctype` attribute.  */
  @property({ attribute: "formenctype" })
  formEnctype: "application/x-www-form-urlencoded" | "multipart/form-data" | "text/plain";

  /** Used to override the form owner's `method` attribute.  */
  @property({ attribute: "formmethod" }) formMethod: "post" | "get";

  /** Used to override the form owner's `novalidate` attribute. */
  @property({ attribute: "formnovalidate", type: Boolean }) formNoValidate: boolean;

  /** Used to override the form owner's `target` attribute. */
  @property({ attribute: "formtarget" }) formTarget:
    | "_self"
    | "_blank"
    | "_parent"
    | "_top"
    | string;

  private constructLightDOMButton() {
    const button = document.createElement("button");

    for (const attribute of this.attributes) {
      if (attribute.name === "style") {
        // Skip style attributes as they *shouldn't* be necessary
        continue;
      }
      button.setAttribute(attribute.name, attribute.value);
    }

    button.type = this.type;
    button.style.position = "absolute !important";
    button.style.width = "0 !important";
    button.style.height = "0 !important";
    button.style.clipPath = "inset(50%) !important";
    button.style.overflow = "hidden !important";
    button.style.whiteSpace = "nowrap !important";
    if (this.name) {
      button.name = this.name;
    }
    button.value = this.value || "";

    return button;
  }

  private handleClick(event: PointerEvent) {
    // Prevent disabled and loading buttons from being clicked
    if (this.disabled || this.loading) {
      event.preventDefault();
      event.stopImmediatePropagation();
      return;
    }

    // Only create a light dom button for submit / reset buttons.
    if (this.type !== "submit" && this.type !== "reset") {
      return;
    }

    const form = this.getForm();

    if (!form) return;

    const lightDOMButton = this.constructLightDOMButton();

    // form.append(lightDOMButton);
    this.parentElement?.append(lightDOMButton);
    lightDOMButton.click();
    lightDOMButton.remove();
  }

  private handleInvalid() {
    this.dispatchEvent(new WaInvalidEvent());
  }

  private handleLabelSlotChange() {
    const nodes = this.labelSlot.assignedNodes({ flatten: true });
    let hasIconLabel = false;
    let hasIcon = false;
    let hasText = false;
    let hasOtherElements = false;

    // Check all slotted nodes
    [...nodes].forEach((node) => {
      if (node.nodeType === Node.ELEMENT_NODE) {
        const element = node as HTMLElement;

        if (element.localName === "wa-icon") {
          hasIcon = true;
          if (!hasIconLabel) hasIconLabel = (element as WaIcon).label !== undefined;
        } else {
          // Any other element type means it's not an icon button
          hasOtherElements = true;
        }
      } else if (node.nodeType === Node.TEXT_NODE) {
        // Check if text node has actual content
        const text = node.textContent?.trim() || "";
        if (text.length > 0) {
          hasText = true;
        }
      }
    });

    // It's only an icon button if there's an icon and nothing else
    this.isIconButton = hasIcon && !hasText && !hasOtherElements;
    this.customStates.set("icon-button", this.isIconButton);

    if (this.isIconButton && !hasIconLabel) {
      console.warn(
        'Icon buttons must have a label for screen readers. Add <wa-icon label="..."> to remove this warning.',
        this,
      );
    }
  }

  private isButton() {
    return this.href ? false : true;
  }

  private isLink() {
    return this.href ? true : false;
  }

  @watch("disabled", { waitUntilFirstUpdate: true })
  handleDisabledChange() {
    this.customStates.set("disabled", this.disabled);
    this.updateValidity();
  }

  @watch("href")
  handleHrefChange() {
    this.customStates.set("link", this.isLink());
  }

  @watch("loading", { waitUntilFirstUpdate: true })
  handleLoadingChange() {
    this.customStates.set("loading", this.loading);
  }

  // eslint-disable-next-line
  setValue(..._args: Parameters<WebAwesomeFormAssociatedElement["setValue"]>) {
    // This is just a stub. We don't ever actually want to set a value on the form. That happens when the button is clicked and added
    // via the light dom button.
  }

  /** Simulates a click on the button. */
  click() {
    this.button.click();
  }

  /** Sets focus on the button. */
  focus(options?: FocusOptions) {
    this.button.focus(options);
  }

  /** Removes focus from the button. */
  blur() {
    this.button.blur();
  }

  render() {
    const isLink = this.isLink();
    const tag = isLink ? literal`a` : literal`button`;

    /* eslint-disable lit/no-invalid-html */
    /* eslint-disable lit/binding-positions */
    return html`
      <${tag}
        part="base"
        class=${classMap({
          button: true,
          caret: this.withCaret,
          disabled: this.disabled,
          loading: this.loading,
          rtl: this.localize.dir() === "rtl",
          "has-label": this.hasSlotController.test("[default]"),
          "has-start": this.hasUpdated ? this.hasSlotController.test("start") : this.withStart,
          "has-end": this.hasUpdated ? this.hasSlotController.test("end") : this.withEnd,
          "is-icon-button": this.isIconButton,
        })}
        ?disabled=${ifDefined(isLink ? undefined : this.disabled)}
        type=${ifDefined(isLink ? undefined : this.type)}
        title=${this.title /* An empty title prevents browser validation tooltips from appearing on hover */}
        name=${ifDefined(isLink ? undefined : this.name)}
        value=${ifDefined(isLink ? undefined : this.value)}
        href=${ifDefined(isLink ? this.href : undefined)}
        target=${ifDefined(isLink ? this.target : undefined)}
        download=${ifDefined(isLink ? this.download : undefined)}
        rel=${ifDefined(isLink && this.rel ? this.rel : undefined)}
        role=${ifDefined(isLink ? undefined : "button")}
        aria-disabled=${ifDefined(isLink && this.disabled ? "true" : undefined)}
        tabindex=${this.disabled ? "-1" : "0"}
        @invalid=${this.isButton() ? this.handleInvalid : null}
        @click=${this.handleClick}
      >
        <slot name="start" part="start" class="start"></slot>
        <slot part="label" class="label" @slotchange=${this.handleLabelSlotChange}></slot>
        <slot name="end" part="end" class="end"></slot>
        ${
          this.withCaret
            ? html`
                <wa-icon
                  part="caret"
                  class="caret"
                  library="system"
                  name="chevron-down"
                  variant="solid"
                ></wa-icon>
              `
            : ""
        }
        ${
          this.loading
            ? html`
                <wa-spinner part="spinner"></wa-spinner>
              `
            : ""
        }
      </${tag}>
    `;
  }
}

// The change-in-update warning is required for this component because the form-associated base class calls
// updateValidity() in firstUpdated(), which triggers requestUpdate('validity') to sync the validation state after the
// first render when the validation target is available. Additionally, HasSlotController triggers requestUpdate() on
// initial slotchange events, and the label slot's slotchange handler sets the isIconButton state property to detect
// icon-only buttons after slot content is available. See https://lit.dev/docs/tools/development/#development-build-runtime-warnings
WaButton.disableWarning?.("change-in-update");

declare global {
  interface HTMLElementTagNameMap {
    "wa-button": WaButton;
  }
}

`````

### Expected (prettier)

`````ts
import { customElement, property, query, state } from "lit/decorators.js";
import { classMap } from "lit/directives/class-map.js";
import { ifDefined } from "lit/directives/if-defined.js";
import { html, literal } from "lit/static-html.js";
import { WaInvalidEvent } from "../../events/invalid.js";
import { warnDeprecatedSize } from "../../internal/size.js";
import { HasSlotController } from "../../internal/slot.js";
import { MirrorValidator } from "../../internal/validators/mirror-validator.js";
import { watch } from "../../internal/watch.js";
import { WebAwesomeFormAssociatedElement } from "../../internal/webawesome-form-associated-element.js";
import sizeStyles from "../../styles/component/size.styles.js";
import variantStyles from "../../styles/component/variants.styles.js";
import { LocalizeController } from "../../utilities/localize.js";
import "../icon/icon.js";
import type WaIcon from "../icon/icon.js";
import "../spinner/spinner.js";
import styles from "./button.styles.js";

/**
 * @summary Buttons represent actions the user can take, such as submitting a form, opening a dialog, or navigating to
 *  another page.
 * @documentation https://webawesome.com/docs/components/button
 * @status stable
 * @since 2.0
 *
 * @dependency wa-icon
 * @dependency wa-spinner
 *
 * @event blur - Emitted when the button loses focus.
 * @event focus - Emitted when the button gains focus.
 * @event wa-invalid - Emitted when the form control has been checked for validity and its constraints aren't satisfied.
 *
 * @slot - The button's label.
 * @slot start - An element, such as `<wa-icon>`, placed before the label.
 * @slot end - An element, such as `<wa-icon>`, placed after the label.
 *
 * @csspart base - The component's base wrapper.
 * @csspart start - The container that wraps the `start` slot.
 * @csspart label - The button's label.
 * @csspart end - The container that wraps the `end` slot.
 * @csspart caret - The button's caret icon, a `<wa-icon>` element.
 * @csspart spinner - The spinner that shows when the button is in the loading state.
 *
 * @cssstate disabled - Applied when the button is disabled.
 * @cssstate icon-button - Applied when the button contains only a `<wa-icon>` with no other content.
 * @cssstate link - Applied when the button is rendered as a link (i.e. `href` is set).
 * @cssstate loading - Applied when the button is in the loading state.
 */
@customElement("wa-button")
export default class WaButton extends WebAwesomeFormAssociatedElement {
  static shadowRootOptions = {
    ...WebAwesomeFormAssociatedElement.shadowRootOptions,
    delegatesFocus: true,
  };
  static css = [styles, variantStyles, sizeStyles];

  static get validators() {
    return [...super.validators, MirrorValidator()];
  }

  assumeInteractionOn = ["click"];
  private readonly hasSlotController = new HasSlotController(this, "[default]", "start", "end");
  private readonly localize = new LocalizeController(this);

  @query(".button") button: HTMLButtonElement | HTMLLinkElement;
  @query("slot:not([name])") labelSlot: HTMLSlotElement;

  @state() invalid = false;
  @state() isIconButton = false;

  @property() title = ""; // make reactive to pass through

  /** The button's theme variant. Defaults to `neutral` if not within another element with a variant. */
  @property({ reflect: true }) variant: "neutral" | "brand" | "success" | "warning" | "danger" =
    "neutral";

  /** The button's visual appearance. */
  @property({ reflect: true }) appearance:
    "accent" | "filled" | "outlined" | "filled-outlined" | "plain" = "accent";

  /** The button's size. */
  @property({ reflect: true }) size: "xs" | "s" | "m" | "l" | "xl" | "small" | "medium" | "large" =
    "m";

  @watch("size")
  handleSizeChange() {
    warnDeprecatedSize(this.localName, this.size);
  }

  /** Draws the button with a caret. Used to indicate that the button triggers a dropdown menu or similar behavior. */
  @property({ attribute: "with-caret", type: Boolean, reflect: true }) withCaret = false;

  /**
   * Only required for SSR. Set to `true` if you're slotting in a `start` element so the server-rendered markup
   * includes the start slot before the component hydrates on the client.
   */
  @property({ attribute: "with-start", type: Boolean }) withStart = false;

  /**
   * Only required for SSR. Set to `true` if you're slotting in an `end` element so the server-rendered markup
   * includes the end slot before the component hydrates on the client.
   */
  @property({ attribute: "with-end", type: Boolean }) withEnd = false;

  /** Disables the button. */
  @property({ type: Boolean }) disabled = false;

  /** Draws the button in a loading state. */
  @property({ type: Boolean, reflect: true }) loading = false;

  /** Draws a pill-style button with rounded edges. */
  @property({ type: Boolean, reflect: true }) pill = false;

  /**
   * The type of button. Note that the default value is `button` instead of `submit`, which is opposite of how native
   * `<button>` elements behave. When the type is `submit`, the button will submit the surrounding form.
   */
  @property() type: "button" | "submit" | "reset" = "button";

  /**
   * The name of the button, submitted as a name/value pair with form data, but only when this button is the submitter.
   * This attribute is ignored when `href` is present.
   */
  @property({ reflect: true }) name: string;

  /**
   * The value of the button, submitted as a pair with the button's name as part of the form data, but only when this
   * button is the submitter. This attribute is ignored when `href` is present.
   */
  @property({ reflect: true }) value: string;

  /** When set, the underlying button will be rendered as an `<a>` with this `href` instead of a `<button>`. */
  @property({ reflect: true }) href: string;

  /** Tells the browser where to open the link. Only used when `href` is present. */
  @property() target: "_blank" | "_parent" | "_self" | "_top";

  /** When using `href`, this attribute will map to the underlying link's `rel` attribute. */
  @property() rel?: string;

  /** Tells the browser to download the linked file as this filename. Only used when `href` is present. */
  @property() download?: string;

  /**
   * The "form owner" to associate the button with. If omitted, the closest containing form will be used instead. The
   * value of this attribute must be an id of a form in the same document or shadow root as the button.
   */

  /** Used to override the form owner's `action` attribute. */
  @property({ attribute: "formaction" }) formAction: string;

  /** Used to override the form owner's `enctype` attribute.  */
  @property({ attribute: "formenctype" })
  formEnctype: "application/x-www-form-urlencoded" | "multipart/form-data" | "text/plain";

  /** Used to override the form owner's `method` attribute.  */
  @property({ attribute: "formmethod" }) formMethod: "post" | "get";

  /** Used to override the form owner's `novalidate` attribute. */
  @property({ attribute: "formnovalidate", type: Boolean }) formNoValidate: boolean;

  /** Used to override the form owner's `target` attribute. */
  @property({ attribute: "formtarget" }) formTarget:
    "_self" | "_blank" | "_parent" | "_top" | string;

  private constructLightDOMButton() {
    const button = document.createElement("button");

    for (const attribute of this.attributes) {
      if (attribute.name === "style") {
        // Skip style attributes as they *shouldn't* be necessary
        continue;
      }
      button.setAttribute(attribute.name, attribute.value);
    }

    button.type = this.type;
    button.style.position = "absolute !important";
    button.style.width = "0 !important";
    button.style.height = "0 !important";
    button.style.clipPath = "inset(50%) !important";
    button.style.overflow = "hidden !important";
    button.style.whiteSpace = "nowrap !important";
    if (this.name) {
      button.name = this.name;
    }
    button.value = this.value || "";

    return button;
  }

  private handleClick(event: PointerEvent) {
    // Prevent disabled and loading buttons from being clicked
    if (this.disabled || this.loading) {
      event.preventDefault();
      event.stopImmediatePropagation();
      return;
    }

    // Only create a light dom button for submit / reset buttons.
    if (this.type !== "submit" && this.type !== "reset") {
      return;
    }

    const form = this.getForm();

    if (!form) return;

    const lightDOMButton = this.constructLightDOMButton();

    // form.append(lightDOMButton);
    this.parentElement?.append(lightDOMButton);
    lightDOMButton.click();
    lightDOMButton.remove();
  }

  private handleInvalid() {
    this.dispatchEvent(new WaInvalidEvent());
  }

  private handleLabelSlotChange() {
    const nodes = this.labelSlot.assignedNodes({ flatten: true });
    let hasIconLabel = false;
    let hasIcon = false;
    let hasText = false;
    let hasOtherElements = false;

    // Check all slotted nodes
    [...nodes].forEach((node) => {
      if (node.nodeType === Node.ELEMENT_NODE) {
        const element = node as HTMLElement;

        if (element.localName === "wa-icon") {
          hasIcon = true;
          if (!hasIconLabel) hasIconLabel = (element as WaIcon).label !== undefined;
        } else {
          // Any other element type means it's not an icon button
          hasOtherElements = true;
        }
      } else if (node.nodeType === Node.TEXT_NODE) {
        // Check if text node has actual content
        const text = node.textContent?.trim() || "";
        if (text.length > 0) {
          hasText = true;
        }
      }
    });

    // It's only an icon button if there's an icon and nothing else
    this.isIconButton = hasIcon && !hasText && !hasOtherElements;
    this.customStates.set("icon-button", this.isIconButton);

    if (this.isIconButton && !hasIconLabel) {
      console.warn(
        'Icon buttons must have a label for screen readers. Add <wa-icon label="..."> to remove this warning.',
        this,
      );
    }
  }

  private isButton() {
    return this.href ? false : true;
  }

  private isLink() {
    return this.href ? true : false;
  }

  @watch("disabled", { waitUntilFirstUpdate: true })
  handleDisabledChange() {
    this.customStates.set("disabled", this.disabled);
    this.updateValidity();
  }

  @watch("href")
  handleHrefChange() {
    this.customStates.set("link", this.isLink());
  }

  @watch("loading", { waitUntilFirstUpdate: true })
  handleLoadingChange() {
    this.customStates.set("loading", this.loading);
  }

  // eslint-disable-next-line
  setValue(..._args: Parameters<WebAwesomeFormAssociatedElement["setValue"]>) {
    // This is just a stub. We don't ever actually want to set a value on the form. That happens when the button is clicked and added
    // via the light dom button.
  }

  /** Simulates a click on the button. */
  click() {
    this.button.click();
  }

  /** Sets focus on the button. */
  focus(options?: FocusOptions) {
    this.button.focus(options);
  }

  /** Removes focus from the button. */
  blur() {
    this.button.blur();
  }

  render() {
    const isLink = this.isLink();
    const tag = isLink ? literal`a` : literal`button`;

    /* eslint-disable lit/no-invalid-html */
    /* eslint-disable lit/binding-positions */
    return html`
      <${tag}
        part="base"
        class=${classMap({
          button: true,
          caret: this.withCaret,
          disabled: this.disabled,
          loading: this.loading,
          rtl: this.localize.dir() === "rtl",
          "has-label": this.hasSlotController.test("[default]"),
          "has-start": this.hasUpdated ? this.hasSlotController.test("start") : this.withStart,
          "has-end": this.hasUpdated ? this.hasSlotController.test("end") : this.withEnd,
          "is-icon-button": this.isIconButton,
        })}
        ?disabled=${ifDefined(isLink ? undefined : this.disabled)}
        type=${ifDefined(isLink ? undefined : this.type)}
        title=${this.title /* An empty title prevents browser validation tooltips from appearing on hover */}
        name=${ifDefined(isLink ? undefined : this.name)}
        value=${ifDefined(isLink ? undefined : this.value)}
        href=${ifDefined(isLink ? this.href : undefined)}
        target=${ifDefined(isLink ? this.target : undefined)}
        download=${ifDefined(isLink ? this.download : undefined)}
        rel=${ifDefined(isLink && this.rel ? this.rel : undefined)}
        role=${ifDefined(isLink ? undefined : "button")}
        aria-disabled=${ifDefined(isLink && this.disabled ? "true" : undefined)}
        tabindex=${this.disabled ? "-1" : "0"}
        @invalid=${this.isButton() ? this.handleInvalid : null}
        @click=${this.handleClick}
      >
        <slot name="start" part="start" class="start"></slot>
        <slot part="label" class="label" @slotchange=${this.handleLabelSlotChange}></slot>
        <slot name="end" part="end" class="end"></slot>
        ${
          this.withCaret
            ? html`
                <wa-icon
                  part="caret"
                  class="caret"
                  library="system"
                  name="chevron-down"
                  variant="solid"
                ></wa-icon>
              `
            : ""
        }
        ${
          this.loading
            ? html`
                <wa-spinner part="spinner"></wa-spinner>
              `
            : ""
        }
      </${tag}>
    `;
  }
}

// The change-in-update warning is required for this component because the form-associated base class calls
// updateValidity() in firstUpdated(), which triggers requestUpdate('validity') to sync the validation state after the
// first render when the validation target is available. Additionally, HasSlotController triggers requestUpdate() on
// initial slotchange events, and the label slot's slotchange handler sets the isIconButton state property to detect
// icon-only buttons after slot content is available. See https://lit.dev/docs/tools/development/#development-build-runtime-warnings
WaButton.disableWarning?.("change-in-update");

declare global {
  interface HTMLElementTagNameMap {
    "wa-button": WaButton;
  }
}

`````
