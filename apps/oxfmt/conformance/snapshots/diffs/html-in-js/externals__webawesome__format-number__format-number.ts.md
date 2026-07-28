# externals/webawesome/format-number/format-number.ts

## Option 1

`````json
{"printWidth":80}
`````

### Diff

`````diff
===================================================================
--- prettier
+++ oxfmt
@@ -31,9 +31,12 @@
   @property() currency = "USD";
 
   /** How to display the currency. */
   @property({ attribute: "currency-display" }) currencyDisplay:
-    "symbol" | "narrowSymbol" | "code" | "name" = "symbol";
+    | "symbol"
+    | "narrowSymbol"
+    | "code"
+    | "name" = "symbol";
 
   /** The minimum number of integer digits to use. Possible values are 1-21. */
   @property({ attribute: "minimum-integer-digits", type: Number })
   minimumIntegerDigits: number;

`````

### Actual (oxfmt)

`````ts
import { customElement, property } from "lit/decorators.js";
import WebAwesomeElement from "../../internal/webawesome-element.js";
import { LocalizeController } from "../../utilities/localize.js";

/**
 * @summary Formats a number for display using the specified locale and options, including currency, percent, and unit
 *  styles. Powered by the Intl.NumberFormat API.
 * @documentation https://webawesome.com/docs/components/format-number
 * @status stable
 * @since 2.0
 */
@customElement("wa-format-number")
export default class WaFormatNumber extends WebAwesomeElement {
  static get styles() {
    return [];
  }

  private readonly localize = new LocalizeController(this);

  /** The number to format. */
  @property({ type: Number }) value = 0;

  /** The formatting style to use. */
  @property() type: "currency" | "decimal" | "percent" = "decimal";

  /** Turns off grouping separators. */
  @property({ attribute: "without-grouping", type: Boolean }) withoutGrouping =
    false;

  /** The [ISO 4217](https://en.wikipedia.org/wiki/ISO_4217) currency code to use when formatting. */
  @property() currency = "USD";

  /** How to display the currency. */
  @property({ attribute: "currency-display" }) currencyDisplay:
    | "symbol"
    | "narrowSymbol"
    | "code"
    | "name" = "symbol";

  /** The minimum number of integer digits to use. Possible values are 1-21. */
  @property({ attribute: "minimum-integer-digits", type: Number })
  minimumIntegerDigits: number;

  /** The minimum number of fraction digits to use. Possible values are 0-100. */
  @property({ attribute: "minimum-fraction-digits", type: Number })
  minimumFractionDigits: number;

  /** The maximum number of fraction digits to use. Possible values are 0-100. */
  @property({ attribute: "maximum-fraction-digits", type: Number })
  maximumFractionDigits: number;

  /** The minimum number of significant digits to use. Possible values are 1-21. */
  @property({ attribute: "minimum-significant-digits", type: Number })
  minimumSignificantDigits: number;

  /** The maximum number of significant digits to use,. Possible values are 1-21. */
  @property({ attribute: "maximum-significant-digits", type: Number })
  maximumSignificantDigits: number;

  render() {
    if (isNaN(this.value)) {
      return "";
    }

    return this.localize.number(this.value, {
      style: this.type,
      currency: this.currency,
      currencyDisplay: this.currencyDisplay,
      useGrouping: !this.withoutGrouping,
      minimumIntegerDigits: this.minimumIntegerDigits,
      minimumFractionDigits: this.minimumFractionDigits,
      maximumFractionDigits: this.maximumFractionDigits,
      minimumSignificantDigits: this.minimumSignificantDigits,
      maximumSignificantDigits: this.maximumSignificantDigits,
    });
  }
}

declare global {
  interface HTMLElementTagNameMap {
    "wa-format-number": WaFormatNumber;
  }
}

`````

### Expected (prettier)

`````ts
import { customElement, property } from "lit/decorators.js";
import WebAwesomeElement from "../../internal/webawesome-element.js";
import { LocalizeController } from "../../utilities/localize.js";

/**
 * @summary Formats a number for display using the specified locale and options, including currency, percent, and unit
 *  styles. Powered by the Intl.NumberFormat API.
 * @documentation https://webawesome.com/docs/components/format-number
 * @status stable
 * @since 2.0
 */
@customElement("wa-format-number")
export default class WaFormatNumber extends WebAwesomeElement {
  static get styles() {
    return [];
  }

  private readonly localize = new LocalizeController(this);

  /** The number to format. */
  @property({ type: Number }) value = 0;

  /** The formatting style to use. */
  @property() type: "currency" | "decimal" | "percent" = "decimal";

  /** Turns off grouping separators. */
  @property({ attribute: "without-grouping", type: Boolean }) withoutGrouping =
    false;

  /** The [ISO 4217](https://en.wikipedia.org/wiki/ISO_4217) currency code to use when formatting. */
  @property() currency = "USD";

  /** How to display the currency. */
  @property({ attribute: "currency-display" }) currencyDisplay:
    "symbol" | "narrowSymbol" | "code" | "name" = "symbol";

  /** The minimum number of integer digits to use. Possible values are 1-21. */
  @property({ attribute: "minimum-integer-digits", type: Number })
  minimumIntegerDigits: number;

  /** The minimum number of fraction digits to use. Possible values are 0-100. */
  @property({ attribute: "minimum-fraction-digits", type: Number })
  minimumFractionDigits: number;

  /** The maximum number of fraction digits to use. Possible values are 0-100. */
  @property({ attribute: "maximum-fraction-digits", type: Number })
  maximumFractionDigits: number;

  /** The minimum number of significant digits to use. Possible values are 1-21. */
  @property({ attribute: "minimum-significant-digits", type: Number })
  minimumSignificantDigits: number;

  /** The maximum number of significant digits to use,. Possible values are 1-21. */
  @property({ attribute: "maximum-significant-digits", type: Number })
  maximumSignificantDigits: number;

  render() {
    if (isNaN(this.value)) {
      return "";
    }

    return this.localize.number(this.value, {
      style: this.type,
      currency: this.currency,
      currencyDisplay: this.currencyDisplay,
      useGrouping: !this.withoutGrouping,
      minimumIntegerDigits: this.minimumIntegerDigits,
      minimumFractionDigits: this.minimumFractionDigits,
      maximumFractionDigits: this.maximumFractionDigits,
      minimumSignificantDigits: this.minimumSignificantDigits,
      maximumSignificantDigits: this.maximumSignificantDigits,
    });
  }
}

declare global {
  interface HTMLElementTagNameMap {
    "wa-format-number": WaFormatNumber;
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
@@ -30,9 +30,12 @@
   @property() currency = "USD";
 
   /** How to display the currency. */
   @property({ attribute: "currency-display" }) currencyDisplay:
-    "symbol" | "narrowSymbol" | "code" | "name" = "symbol";
+    | "symbol"
+    | "narrowSymbol"
+    | "code"
+    | "name" = "symbol";
 
   /** The minimum number of integer digits to use. Possible values are 1-21. */
   @property({ attribute: "minimum-integer-digits", type: Number }) minimumIntegerDigits: number;
 

`````

### Actual (oxfmt)

`````ts
import { customElement, property } from "lit/decorators.js";
import WebAwesomeElement from "../../internal/webawesome-element.js";
import { LocalizeController } from "../../utilities/localize.js";

/**
 * @summary Formats a number for display using the specified locale and options, including currency, percent, and unit
 *  styles. Powered by the Intl.NumberFormat API.
 * @documentation https://webawesome.com/docs/components/format-number
 * @status stable
 * @since 2.0
 */
@customElement("wa-format-number")
export default class WaFormatNumber extends WebAwesomeElement {
  static get styles() {
    return [];
  }

  private readonly localize = new LocalizeController(this);

  /** The number to format. */
  @property({ type: Number }) value = 0;

  /** The formatting style to use. */
  @property() type: "currency" | "decimal" | "percent" = "decimal";

  /** Turns off grouping separators. */
  @property({ attribute: "without-grouping", type: Boolean }) withoutGrouping = false;

  /** The [ISO 4217](https://en.wikipedia.org/wiki/ISO_4217) currency code to use when formatting. */
  @property() currency = "USD";

  /** How to display the currency. */
  @property({ attribute: "currency-display" }) currencyDisplay:
    | "symbol"
    | "narrowSymbol"
    | "code"
    | "name" = "symbol";

  /** The minimum number of integer digits to use. Possible values are 1-21. */
  @property({ attribute: "minimum-integer-digits", type: Number }) minimumIntegerDigits: number;

  /** The minimum number of fraction digits to use. Possible values are 0-100. */
  @property({ attribute: "minimum-fraction-digits", type: Number }) minimumFractionDigits: number;

  /** The maximum number of fraction digits to use. Possible values are 0-100. */
  @property({ attribute: "maximum-fraction-digits", type: Number }) maximumFractionDigits: number;

  /** The minimum number of significant digits to use. Possible values are 1-21. */
  @property({ attribute: "minimum-significant-digits", type: Number })
  minimumSignificantDigits: number;

  /** The maximum number of significant digits to use,. Possible values are 1-21. */
  @property({ attribute: "maximum-significant-digits", type: Number })
  maximumSignificantDigits: number;

  render() {
    if (isNaN(this.value)) {
      return "";
    }

    return this.localize.number(this.value, {
      style: this.type,
      currency: this.currency,
      currencyDisplay: this.currencyDisplay,
      useGrouping: !this.withoutGrouping,
      minimumIntegerDigits: this.minimumIntegerDigits,
      minimumFractionDigits: this.minimumFractionDigits,
      maximumFractionDigits: this.maximumFractionDigits,
      minimumSignificantDigits: this.minimumSignificantDigits,
      maximumSignificantDigits: this.maximumSignificantDigits,
    });
  }
}

declare global {
  interface HTMLElementTagNameMap {
    "wa-format-number": WaFormatNumber;
  }
}

`````

### Expected (prettier)

`````ts
import { customElement, property } from "lit/decorators.js";
import WebAwesomeElement from "../../internal/webawesome-element.js";
import { LocalizeController } from "../../utilities/localize.js";

/**
 * @summary Formats a number for display using the specified locale and options, including currency, percent, and unit
 *  styles. Powered by the Intl.NumberFormat API.
 * @documentation https://webawesome.com/docs/components/format-number
 * @status stable
 * @since 2.0
 */
@customElement("wa-format-number")
export default class WaFormatNumber extends WebAwesomeElement {
  static get styles() {
    return [];
  }

  private readonly localize = new LocalizeController(this);

  /** The number to format. */
  @property({ type: Number }) value = 0;

  /** The formatting style to use. */
  @property() type: "currency" | "decimal" | "percent" = "decimal";

  /** Turns off grouping separators. */
  @property({ attribute: "without-grouping", type: Boolean }) withoutGrouping = false;

  /** The [ISO 4217](https://en.wikipedia.org/wiki/ISO_4217) currency code to use when formatting. */
  @property() currency = "USD";

  /** How to display the currency. */
  @property({ attribute: "currency-display" }) currencyDisplay:
    "symbol" | "narrowSymbol" | "code" | "name" = "symbol";

  /** The minimum number of integer digits to use. Possible values are 1-21. */
  @property({ attribute: "minimum-integer-digits", type: Number }) minimumIntegerDigits: number;

  /** The minimum number of fraction digits to use. Possible values are 0-100. */
  @property({ attribute: "minimum-fraction-digits", type: Number }) minimumFractionDigits: number;

  /** The maximum number of fraction digits to use. Possible values are 0-100. */
  @property({ attribute: "maximum-fraction-digits", type: Number }) maximumFractionDigits: number;

  /** The minimum number of significant digits to use. Possible values are 1-21. */
  @property({ attribute: "minimum-significant-digits", type: Number })
  minimumSignificantDigits: number;

  /** The maximum number of significant digits to use,. Possible values are 1-21. */
  @property({ attribute: "maximum-significant-digits", type: Number })
  maximumSignificantDigits: number;

  render() {
    if (isNaN(this.value)) {
      return "";
    }

    return this.localize.number(this.value, {
      style: this.type,
      currency: this.currency,
      currencyDisplay: this.currencyDisplay,
      useGrouping: !this.withoutGrouping,
      minimumIntegerDigits: this.minimumIntegerDigits,
      minimumFractionDigits: this.minimumFractionDigits,
      maximumFractionDigits: this.maximumFractionDigits,
      minimumSignificantDigits: this.minimumSignificantDigits,
      maximumSignificantDigits: this.maximumSignificantDigits,
    });
  }
}

declare global {
  interface HTMLElementTagNameMap {
    "wa-format-number": WaFormatNumber;
  }
}

`````
