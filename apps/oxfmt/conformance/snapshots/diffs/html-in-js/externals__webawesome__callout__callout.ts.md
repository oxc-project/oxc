# externals/webawesome/callout/callout.ts

## Option 1

`````json
{"printWidth":80}
`````

### Diff

`````diff
===================================================================
--- prettier
+++ oxfmt
@@ -25,17 +25,32 @@
   static css = [styles, variantStyles, sizeStyles];
 
   /** The callout's theme variant. Defaults to `brand` if not within another element with a variant. */
   @property({ reflect: true }) variant:
-    "brand" | "neutral" | "success" | "warning" | "danger" = "brand";
+    | "brand"
+    | "neutral"
+    | "success"
+    | "warning"
+    | "danger" = "brand";
 
   /** The callout's visual appearance. */
   @property({ reflect: true }) appearance:
-    "accent" | "filled" | "outlined" | "plain" | "filled-outlined";
+    | "accent"
+    | "filled"
+    | "outlined"
+    | "plain"
+    | "filled-outlined";
 
   /** The callout's size. */
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
import { html } from "lit";
import { customElement, property } from "lit/decorators.js";
import { warnDeprecatedSize } from "../../internal/size.js";
import { watch } from "../../internal/watch.js";
import WebAwesomeElement from "../../internal/webawesome-element.js";
import sizeStyles from "../../styles/component/size.styles.js";
import variantStyles from "../../styles/component/variants.styles.js";
import styles from "./callout.styles.js";

/**
 * @summary Callouts display important messages inline with surrounding content. Use them to highlight tips, warnings,
 *  errors, or other information users should not miss.
 * @documentation https://webawesome.com/docs/components/callout
 * @status stable
 * @since 3.0
 *
 * @slot - The callout's main content.
 * @slot icon - An icon to show in the callout. Works best with `<wa-icon>`.
 *
 * @csspart icon - The container that wraps the optional icon.
 * @csspart message - The container that wraps the callout's main content.
 */
@customElement("wa-callout")
export default class WaCallout extends WebAwesomeElement {
  static css = [styles, variantStyles, sizeStyles];

  /** The callout's theme variant. Defaults to `brand` if not within another element with a variant. */
  @property({ reflect: true }) variant:
    | "brand"
    | "neutral"
    | "success"
    | "warning"
    | "danger" = "brand";

  /** The callout's visual appearance. */
  @property({ reflect: true }) appearance:
    | "accent"
    | "filled"
    | "outlined"
    | "plain"
    | "filled-outlined";

  /** The callout's size. */
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

  render() {
    return html`
      <div part="icon">
        <slot name="icon"></slot>
      </div>

      <div part="message">
        <slot></slot>
      </div>
    `;
  }
}

declare global {
  interface HTMLElementTagNameMap {
    "wa-callout": WaCallout;
  }
}

`````

### Expected (prettier)

`````ts
import { html } from "lit";
import { customElement, property } from "lit/decorators.js";
import { warnDeprecatedSize } from "../../internal/size.js";
import { watch } from "../../internal/watch.js";
import WebAwesomeElement from "../../internal/webawesome-element.js";
import sizeStyles from "../../styles/component/size.styles.js";
import variantStyles from "../../styles/component/variants.styles.js";
import styles from "./callout.styles.js";

/**
 * @summary Callouts display important messages inline with surrounding content. Use them to highlight tips, warnings,
 *  errors, or other information users should not miss.
 * @documentation https://webawesome.com/docs/components/callout
 * @status stable
 * @since 3.0
 *
 * @slot - The callout's main content.
 * @slot icon - An icon to show in the callout. Works best with `<wa-icon>`.
 *
 * @csspart icon - The container that wraps the optional icon.
 * @csspart message - The container that wraps the callout's main content.
 */
@customElement("wa-callout")
export default class WaCallout extends WebAwesomeElement {
  static css = [styles, variantStyles, sizeStyles];

  /** The callout's theme variant. Defaults to `brand` if not within another element with a variant. */
  @property({ reflect: true }) variant:
    "brand" | "neutral" | "success" | "warning" | "danger" = "brand";

  /** The callout's visual appearance. */
  @property({ reflect: true }) appearance:
    "accent" | "filled" | "outlined" | "plain" | "filled-outlined";

  /** The callout's size. */
  @property({ reflect: true }) size:
    "xs" | "s" | "m" | "l" | "xl" | "small" | "medium" | "large" = "m";

  @watch("size")
  handleSizeChange() {
    warnDeprecatedSize(this.localName, this.size);
  }

  render() {
    return html`
      <div part="icon">
        <slot name="icon"></slot>
      </div>

      <div part="message">
        <slot></slot>
      </div>
    `;
  }
}

declare global {
  interface HTMLElementTagNameMap {
    "wa-callout": WaCallout;
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
@@ -29,9 +29,13 @@
     "brand";
 
   /** The callout's visual appearance. */
   @property({ reflect: true }) appearance:
-    "accent" | "filled" | "outlined" | "plain" | "filled-outlined";
+    | "accent"
+    | "filled"
+    | "outlined"
+    | "plain"
+    | "filled-outlined";
 
   /** The callout's size. */
   @property({ reflect: true }) size: "xs" | "s" | "m" | "l" | "xl" | "small" | "medium" | "large" =
     "m";

`````

### Actual (oxfmt)

`````ts
import { html } from "lit";
import { customElement, property } from "lit/decorators.js";
import { warnDeprecatedSize } from "../../internal/size.js";
import { watch } from "../../internal/watch.js";
import WebAwesomeElement from "../../internal/webawesome-element.js";
import sizeStyles from "../../styles/component/size.styles.js";
import variantStyles from "../../styles/component/variants.styles.js";
import styles from "./callout.styles.js";

/**
 * @summary Callouts display important messages inline with surrounding content. Use them to highlight tips, warnings,
 *  errors, or other information users should not miss.
 * @documentation https://webawesome.com/docs/components/callout
 * @status stable
 * @since 3.0
 *
 * @slot - The callout's main content.
 * @slot icon - An icon to show in the callout. Works best with `<wa-icon>`.
 *
 * @csspart icon - The container that wraps the optional icon.
 * @csspart message - The container that wraps the callout's main content.
 */
@customElement("wa-callout")
export default class WaCallout extends WebAwesomeElement {
  static css = [styles, variantStyles, sizeStyles];

  /** The callout's theme variant. Defaults to `brand` if not within another element with a variant. */
  @property({ reflect: true }) variant: "brand" | "neutral" | "success" | "warning" | "danger" =
    "brand";

  /** The callout's visual appearance. */
  @property({ reflect: true }) appearance:
    | "accent"
    | "filled"
    | "outlined"
    | "plain"
    | "filled-outlined";

  /** The callout's size. */
  @property({ reflect: true }) size: "xs" | "s" | "m" | "l" | "xl" | "small" | "medium" | "large" =
    "m";

  @watch("size")
  handleSizeChange() {
    warnDeprecatedSize(this.localName, this.size);
  }

  render() {
    return html`
      <div part="icon">
        <slot name="icon"></slot>
      </div>

      <div part="message">
        <slot></slot>
      </div>
    `;
  }
}

declare global {
  interface HTMLElementTagNameMap {
    "wa-callout": WaCallout;
  }
}

`````

### Expected (prettier)

`````ts
import { html } from "lit";
import { customElement, property } from "lit/decorators.js";
import { warnDeprecatedSize } from "../../internal/size.js";
import { watch } from "../../internal/watch.js";
import WebAwesomeElement from "../../internal/webawesome-element.js";
import sizeStyles from "../../styles/component/size.styles.js";
import variantStyles from "../../styles/component/variants.styles.js";
import styles from "./callout.styles.js";

/**
 * @summary Callouts display important messages inline with surrounding content. Use them to highlight tips, warnings,
 *  errors, or other information users should not miss.
 * @documentation https://webawesome.com/docs/components/callout
 * @status stable
 * @since 3.0
 *
 * @slot - The callout's main content.
 * @slot icon - An icon to show in the callout. Works best with `<wa-icon>`.
 *
 * @csspart icon - The container that wraps the optional icon.
 * @csspart message - The container that wraps the callout's main content.
 */
@customElement("wa-callout")
export default class WaCallout extends WebAwesomeElement {
  static css = [styles, variantStyles, sizeStyles];

  /** The callout's theme variant. Defaults to `brand` if not within another element with a variant. */
  @property({ reflect: true }) variant: "brand" | "neutral" | "success" | "warning" | "danger" =
    "brand";

  /** The callout's visual appearance. */
  @property({ reflect: true }) appearance:
    "accent" | "filled" | "outlined" | "plain" | "filled-outlined";

  /** The callout's size. */
  @property({ reflect: true }) size: "xs" | "s" | "m" | "l" | "xl" | "small" | "medium" | "large" =
    "m";

  @watch("size")
  handleSizeChange() {
    warnDeprecatedSize(this.localName, this.size);
  }

  render() {
    return html`
      <div part="icon">
        <slot name="icon"></slot>
      </div>

      <div part="message">
        <slot></slot>
      </div>
    `;
  }
}

declare global {
  interface HTMLElementTagNameMap {
    "wa-callout": WaCallout;
  }
}

`````
