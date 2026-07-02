# externals/webawesome/tag/tag.ts

## Option 1

`````json
{"printWidth":80}
`````

### Diff

`````diff
===================================================================
--- prettier
+++ oxfmt
@@ -35,17 +35,31 @@
   private readonly localize = new LocalizeController(this);
 
   /** The tag's theme variant. Defaults to `neutral` if not within another element with a variant. */
   @property({ reflect: true }) variant:
-    "brand" | "neutral" | "success" | "warning" | "danger" = "neutral";
+    | "brand"
+    | "neutral"
+    | "success"
+    | "warning"
+    | "danger" = "neutral";
 
   /** The tag's visual appearance. */
   @property({ reflect: true }) appearance:
-    "accent" | "filled" | "outlined" | "filled-outlined" = "filled-outlined";
+    | "accent"
+    | "filled"
+    | "outlined"
+    | "filled-outlined" = "filled-outlined";
 
   /** The tag's size. */
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
@@ -64,29 +78,27 @@
   render() {
     return html`
       <slot part="content" class="content"></slot>
 
-      ${
-        this.withRemove
-          ? html`
-              <wa-button
-                part="remove-button"
-                exportparts="base:remove-button__base"
-                class="remove"
-                appearance="plain"
-                @click=${this.handleRemoveClick}
-                tabindex="-1"
-              >
-                <wa-icon
-                  name="xmark"
-                  library="system"
-                  variant="solid"
-                  label=${this.localize.term("remove")}
-                ></wa-icon>
-              </wa-button>
-            `
-          : ""
-      }
+      ${this.withRemove
+        ? html`
+            <wa-button
+              part="remove-button"
+              exportparts="base:remove-button__base"
+              class="remove"
+              appearance="plain"
+              @click=${this.handleRemoveClick}
+              tabindex="-1"
+            >
+              <wa-icon
+                name="xmark"
+                library="system"
+                variant="solid"
+                label=${this.localize.term("remove")}
+              ></wa-icon>
+            </wa-button>
+          `
+        : ""}
     `;
   }
 }
 

`````

### Actual (oxfmt)

`````ts
import { html } from "lit";
import { customElement, property } from "lit/decorators.js";
import { WaRemoveEvent } from "../../events/remove.js";
import { warnDeprecatedSize } from "../../internal/size.js";
import { watch } from "../../internal/watch.js";
import WebAwesomeElement from "../../internal/webawesome-element.js";
import sizeStyles from "../../styles/component/size.styles.js";
import variantStyles from "../../styles/component/variants.styles.js";
import { LocalizeController } from "../../utilities/localize.js";
import "../button/button.js";
import styles from "./tag.styles.js";

/**
 * @summary Tags label, categorize, or represent selections with a compact visual marker. Use them for status
 *  indicators, filters, or removable chips.
 * @documentation https://webawesome.com/docs/components/tag
 * @status stable
 * @since 2.0
 *
 * @dependency wa-button
 *
 * @slot - The tag's content.
 *
 * @event wa-remove - Emitted when the remove button is activated.
 *
 * @csspart base - The component's base wrapper.
 * @csspart content - The tag's content.
 * @csspart remove-button - The tag's remove button, a `<wa-button>`.
 * @csspart remove-button__base - The remove button's exported `base` part.
 */
@customElement("wa-tag")
export default class WaTag extends WebAwesomeElement {
  static css = [styles, variantStyles, sizeStyles];

  private readonly localize = new LocalizeController(this);

  /** The tag's theme variant. Defaults to `neutral` if not within another element with a variant. */
  @property({ reflect: true }) variant:
    | "brand"
    | "neutral"
    | "success"
    | "warning"
    | "danger" = "neutral";

  /** The tag's visual appearance. */
  @property({ reflect: true }) appearance:
    | "accent"
    | "filled"
    | "outlined"
    | "filled-outlined" = "filled-outlined";

  /** The tag's size. */
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

  /** Draws a pill-style tag with rounded edges. */
  @property({ type: Boolean, reflect: true }) pill = false;

  /** Makes the tag removable and shows a remove button. */
  @property({ attribute: "with-remove", type: Boolean }) withRemove = false;

  private handleRemoveClick() {
    this.dispatchEvent(new WaRemoveEvent());
  }

  render() {
    return html`
      <slot part="content" class="content"></slot>

      ${this.withRemove
        ? html`
            <wa-button
              part="remove-button"
              exportparts="base:remove-button__base"
              class="remove"
              appearance="plain"
              @click=${this.handleRemoveClick}
              tabindex="-1"
            >
              <wa-icon
                name="xmark"
                library="system"
                variant="solid"
                label=${this.localize.term("remove")}
              ></wa-icon>
            </wa-button>
          `
        : ""}
    `;
  }
}

declare global {
  interface HTMLElementTagNameMap {
    "wa-tag": WaTag;
  }
}

`````

### Expected (prettier)

`````ts
import { html } from "lit";
import { customElement, property } from "lit/decorators.js";
import { WaRemoveEvent } from "../../events/remove.js";
import { warnDeprecatedSize } from "../../internal/size.js";
import { watch } from "../../internal/watch.js";
import WebAwesomeElement from "../../internal/webawesome-element.js";
import sizeStyles from "../../styles/component/size.styles.js";
import variantStyles from "../../styles/component/variants.styles.js";
import { LocalizeController } from "../../utilities/localize.js";
import "../button/button.js";
import styles from "./tag.styles.js";

/**
 * @summary Tags label, categorize, or represent selections with a compact visual marker. Use them for status
 *  indicators, filters, or removable chips.
 * @documentation https://webawesome.com/docs/components/tag
 * @status stable
 * @since 2.0
 *
 * @dependency wa-button
 *
 * @slot - The tag's content.
 *
 * @event wa-remove - Emitted when the remove button is activated.
 *
 * @csspart base - The component's base wrapper.
 * @csspart content - The tag's content.
 * @csspart remove-button - The tag's remove button, a `<wa-button>`.
 * @csspart remove-button__base - The remove button's exported `base` part.
 */
@customElement("wa-tag")
export default class WaTag extends WebAwesomeElement {
  static css = [styles, variantStyles, sizeStyles];

  private readonly localize = new LocalizeController(this);

  /** The tag's theme variant. Defaults to `neutral` if not within another element with a variant. */
  @property({ reflect: true }) variant:
    "brand" | "neutral" | "success" | "warning" | "danger" = "neutral";

  /** The tag's visual appearance. */
  @property({ reflect: true }) appearance:
    "accent" | "filled" | "outlined" | "filled-outlined" = "filled-outlined";

  /** The tag's size. */
  @property({ reflect: true }) size:
    "xs" | "s" | "m" | "l" | "xl" | "small" | "medium" | "large" = "m";

  @watch("size")
  handleSizeChange() {
    warnDeprecatedSize(this.localName, this.size);
  }

  /** Draws a pill-style tag with rounded edges. */
  @property({ type: Boolean, reflect: true }) pill = false;

  /** Makes the tag removable and shows a remove button. */
  @property({ attribute: "with-remove", type: Boolean }) withRemove = false;

  private handleRemoveClick() {
    this.dispatchEvent(new WaRemoveEvent());
  }

  render() {
    return html`
      <slot part="content" class="content"></slot>

      ${
        this.withRemove
          ? html`
              <wa-button
                part="remove-button"
                exportparts="base:remove-button__base"
                class="remove"
                appearance="plain"
                @click=${this.handleRemoveClick}
                tabindex="-1"
              >
                <wa-icon
                  name="xmark"
                  library="system"
                  variant="solid"
                  label=${this.localize.term("remove")}
                ></wa-icon>
              </wa-button>
            `
          : ""
      }
    `;
  }
}

declare global {
  interface HTMLElementTagNameMap {
    "wa-tag": WaTag;
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
@@ -64,29 +64,27 @@
   render() {
     return html`
       <slot part="content" class="content"></slot>
 
-      ${
-        this.withRemove
-          ? html`
-              <wa-button
-                part="remove-button"
-                exportparts="base:remove-button__base"
-                class="remove"
-                appearance="plain"
-                @click=${this.handleRemoveClick}
-                tabindex="-1"
-              >
-                <wa-icon
-                  name="xmark"
-                  library="system"
-                  variant="solid"
-                  label=${this.localize.term("remove")}
-                ></wa-icon>
-              </wa-button>
-            `
-          : ""
-      }
+      ${this.withRemove
+        ? html`
+            <wa-button
+              part="remove-button"
+              exportparts="base:remove-button__base"
+              class="remove"
+              appearance="plain"
+              @click=${this.handleRemoveClick}
+              tabindex="-1"
+            >
+              <wa-icon
+                name="xmark"
+                library="system"
+                variant="solid"
+                label=${this.localize.term("remove")}
+              ></wa-icon>
+            </wa-button>
+          `
+        : ""}
     `;
   }
 }
 

`````

### Actual (oxfmt)

`````ts
import { html } from "lit";
import { customElement, property } from "lit/decorators.js";
import { WaRemoveEvent } from "../../events/remove.js";
import { warnDeprecatedSize } from "../../internal/size.js";
import { watch } from "../../internal/watch.js";
import WebAwesomeElement from "../../internal/webawesome-element.js";
import sizeStyles from "../../styles/component/size.styles.js";
import variantStyles from "../../styles/component/variants.styles.js";
import { LocalizeController } from "../../utilities/localize.js";
import "../button/button.js";
import styles from "./tag.styles.js";

/**
 * @summary Tags label, categorize, or represent selections with a compact visual marker. Use them for status
 *  indicators, filters, or removable chips.
 * @documentation https://webawesome.com/docs/components/tag
 * @status stable
 * @since 2.0
 *
 * @dependency wa-button
 *
 * @slot - The tag's content.
 *
 * @event wa-remove - Emitted when the remove button is activated.
 *
 * @csspart base - The component's base wrapper.
 * @csspart content - The tag's content.
 * @csspart remove-button - The tag's remove button, a `<wa-button>`.
 * @csspart remove-button__base - The remove button's exported `base` part.
 */
@customElement("wa-tag")
export default class WaTag extends WebAwesomeElement {
  static css = [styles, variantStyles, sizeStyles];

  private readonly localize = new LocalizeController(this);

  /** The tag's theme variant. Defaults to `neutral` if not within another element with a variant. */
  @property({ reflect: true }) variant: "brand" | "neutral" | "success" | "warning" | "danger" =
    "neutral";

  /** The tag's visual appearance. */
  @property({ reflect: true }) appearance: "accent" | "filled" | "outlined" | "filled-outlined" =
    "filled-outlined";

  /** The tag's size. */
  @property({ reflect: true }) size: "xs" | "s" | "m" | "l" | "xl" | "small" | "medium" | "large" =
    "m";

  @watch("size")
  handleSizeChange() {
    warnDeprecatedSize(this.localName, this.size);
  }

  /** Draws a pill-style tag with rounded edges. */
  @property({ type: Boolean, reflect: true }) pill = false;

  /** Makes the tag removable and shows a remove button. */
  @property({ attribute: "with-remove", type: Boolean }) withRemove = false;

  private handleRemoveClick() {
    this.dispatchEvent(new WaRemoveEvent());
  }

  render() {
    return html`
      <slot part="content" class="content"></slot>

      ${this.withRemove
        ? html`
            <wa-button
              part="remove-button"
              exportparts="base:remove-button__base"
              class="remove"
              appearance="plain"
              @click=${this.handleRemoveClick}
              tabindex="-1"
            >
              <wa-icon
                name="xmark"
                library="system"
                variant="solid"
                label=${this.localize.term("remove")}
              ></wa-icon>
            </wa-button>
          `
        : ""}
    `;
  }
}

declare global {
  interface HTMLElementTagNameMap {
    "wa-tag": WaTag;
  }
}

`````

### Expected (prettier)

`````ts
import { html } from "lit";
import { customElement, property } from "lit/decorators.js";
import { WaRemoveEvent } from "../../events/remove.js";
import { warnDeprecatedSize } from "../../internal/size.js";
import { watch } from "../../internal/watch.js";
import WebAwesomeElement from "../../internal/webawesome-element.js";
import sizeStyles from "../../styles/component/size.styles.js";
import variantStyles from "../../styles/component/variants.styles.js";
import { LocalizeController } from "../../utilities/localize.js";
import "../button/button.js";
import styles from "./tag.styles.js";

/**
 * @summary Tags label, categorize, or represent selections with a compact visual marker. Use them for status
 *  indicators, filters, or removable chips.
 * @documentation https://webawesome.com/docs/components/tag
 * @status stable
 * @since 2.0
 *
 * @dependency wa-button
 *
 * @slot - The tag's content.
 *
 * @event wa-remove - Emitted when the remove button is activated.
 *
 * @csspart base - The component's base wrapper.
 * @csspart content - The tag's content.
 * @csspart remove-button - The tag's remove button, a `<wa-button>`.
 * @csspart remove-button__base - The remove button's exported `base` part.
 */
@customElement("wa-tag")
export default class WaTag extends WebAwesomeElement {
  static css = [styles, variantStyles, sizeStyles];

  private readonly localize = new LocalizeController(this);

  /** The tag's theme variant. Defaults to `neutral` if not within another element with a variant. */
  @property({ reflect: true }) variant: "brand" | "neutral" | "success" | "warning" | "danger" =
    "neutral";

  /** The tag's visual appearance. */
  @property({ reflect: true }) appearance: "accent" | "filled" | "outlined" | "filled-outlined" =
    "filled-outlined";

  /** The tag's size. */
  @property({ reflect: true }) size: "xs" | "s" | "m" | "l" | "xl" | "small" | "medium" | "large" =
    "m";

  @watch("size")
  handleSizeChange() {
    warnDeprecatedSize(this.localName, this.size);
  }

  /** Draws a pill-style tag with rounded edges. */
  @property({ type: Boolean, reflect: true }) pill = false;

  /** Makes the tag removable and shows a remove button. */
  @property({ attribute: "with-remove", type: Boolean }) withRemove = false;

  private handleRemoveClick() {
    this.dispatchEvent(new WaRemoveEvent());
  }

  render() {
    return html`
      <slot part="content" class="content"></slot>

      ${
        this.withRemove
          ? html`
              <wa-button
                part="remove-button"
                exportparts="base:remove-button__base"
                class="remove"
                appearance="plain"
                @click=${this.handleRemoveClick}
                tabindex="-1"
              >
                <wa-icon
                  name="xmark"
                  library="system"
                  variant="solid"
                  label=${this.localize.term("remove")}
                ></wa-icon>
              </wa-button>
            `
          : ""
      }
    `;
  }
}

declare global {
  interface HTMLElementTagNameMap {
    "wa-tag": WaTag;
  }
}

`````
