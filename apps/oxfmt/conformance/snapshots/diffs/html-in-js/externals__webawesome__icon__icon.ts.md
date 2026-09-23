# externals/webawesome/icon/icon.ts

## Option 2

`````json
{"printWidth":100,"htmlWhitespaceSensitivity":"ignore"}
`````

### Diff

`````diff
===================================================================
--- prettier
+++ oxfmt
@@ -18,9 +18,12 @@
 
 const CACHEABLE_ERROR = Symbol();
 const RETRYABLE_ERROR = Symbol();
 type SVGResult =
-  HTMLTemplateResult | SVGSVGElement | typeof RETRYABLE_ERROR | typeof CACHEABLE_ERROR;
+  | HTMLTemplateResult
+  | SVGSVGElement
+  | typeof RETRYABLE_ERROR
+  | typeof CACHEABLE_ERROR;
 
 let parser: DOMParser;
 const iconCache = new Map<string, Promise<SVGResult>>();
 

`````

### Actual (oxfmt)

`````ts
import { html } from "lit";
import { customElement, property, state } from "lit/decorators.js";
import { isTemplateResult } from "lit/directive-helpers.js";
import { WaErrorEvent } from "../../events/error.js";
import { WaLoadEvent } from "../../events/load.js";
import { watch } from "../../internal/watch.js";
import WebAwesomeElement from "../../internal/webawesome-element.js";
import styles from "./icon.styles.js";
import {
  getDefaultIconFamily,
  getIconLibrary,
  unwatchIcon,
  watchIcon,
  type IconLibrary,
} from "./library.js";

import type { HTMLTemplateResult, PropertyValues } from "lit";

const CACHEABLE_ERROR = Symbol();
const RETRYABLE_ERROR = Symbol();
type SVGResult =
  | HTMLTemplateResult
  | SVGSVGElement
  | typeof RETRYABLE_ERROR
  | typeof CACHEABLE_ERROR;

let parser: DOMParser;
const iconCache = new Map<string, Promise<SVGResult>>();

interface IconSource {
  url?: string;
  fromLibrary: boolean;
}

export type IconAnimation =
  | "beat"
  | "fade"
  | "beat-fade"
  | "bounce"
  | "flip"
  | "shake"
  | "spin"
  | "spin-pulse"
  | "spin-reverse";

/**
 * @summary Icons are scalable vector symbols that represent actions, content, or status throughout your application.
 *  They support Font Awesome and custom icon libraries with animation presets.
 * @documentation https://webawesome.com/docs/components/icon
 * @status stable
 * @since 2.0
 *
 * @event wa-load - Emitted when the icon has loaded. When using `spriteSheet: true` this will not emit.
 * @event wa-error - Emitted when the icon fails to load due to an error. When using `spriteSheet: true` this will not emit.
 *
 * @csspart svg - The internal SVG element.
 * @csspart use - The `<use>` element generated when using `spriteSheet: true`
 *
 * @cssproperty [--animation-delay=0] Sets when the animation will start.
 * @cssproperty [--animation-direction=normal] Defines whether or not the animation should play in reverse on alternate cycles.
 * @cssproperty [--animation-duration=1s] Defines the length of time that an animation takes to complete one cycle.
 * @cssproperty [--animation-iteration-count=infinite] Defines the number of times an animation cycle is played.
 * @cssproperty [--animation-timing] Describes how the animation will progress over one cycle of its duration.
 * @cssproperty [--beat-fade-opacity] Set lowest opacity value an icon with `beat-fade` animation will fade to and from.
 * @cssproperty [--beat-fade-scale] Set max value that an icon with `beat-fade` animation will scale.
 * @cssproperty [--beat-scale] Set max value that an icon with `beat` animation will scale.
 * @cssproperty [--bounce-height] Set the max height an icon with `bounce` animation will jump to when bouncing.
 * @cssproperty [--bounce-jump-scale-x] Set the icon’s horizontal distortion (“squish”) at the top of the jump.
 * @cssproperty [--bounce-jump-scale-y] Set the icon’s vertical distortion (“squish”) at the top of the jump.
 * @cssproperty [--bounce-land-scale-x] Set the icon’s horizontal distortion (“squish”) when landing after the jump.
 * @cssproperty [--bounce-land-scale-y] Set the icon’s vertical distortion (“squish”) when landing after the jump.
 * @cssproperty [--bounce-rebound] Set the amount of rebound an icon with `bounce` animation has when landing after the jump.
 * @cssproperty [--bounce-start-scale-x] Set the icon’s horizontal distortion (“squish”) when starting to bounce.
 * @cssproperty [--bounce-start-scale-y] Set the icon’s vertical distortion (“squish”) when starting to bounce.
 * @cssproperty [--fade-opacity] Set lowest opacity value an icon with `fade` animation will fade to and from.
 * @cssproperty [--flip-angle] Set rotation angle of flip for an icon with `flip` animation. A positive angle denotes a clockwise rotation, a negative angle a counter-clockwise one.
 * @cssproperty [--flip-x] Set x-coordinate of the vector denoting the axis of rotation (between 0 and 1) for an icon with `flip` animation.
 * @cssproperty [--flip-y] Set y-coordinate of the vector denoting the axis of rotation (between 0 and 1) for an icon with `flip` animation.
 * @cssproperty [--flip-z] Set z-coordinate of the vector denoting the axis of rotation (between 0 and 1) for an icon with `flip` animation.
 * @cssproperty [--primary-color=currentColor] - Sets a duotone icon's primary color.
 * @cssproperty [--primary-opacity=1] - Sets a duotone icon's primary opacity.
 * @cssproperty [--secondary-color=currentColor] - Sets a duotone icon's secondary color.
 * @cssproperty [--secondary-opacity=0.4] - Sets a duotone icon's secondary opacity.
 */
@customElement("wa-icon")
export default class WaIcon extends WebAwesomeElement {
  static css = styles;

  @state() private svg: SVGElement | HTMLTemplateResult | null = null;

  /** The name of the icon to draw. Available names depend on the icon library being used. */
  @property({ reflect: true }) name?: string;

  /**
   * The family of icons to choose from. For Font Awesome Free, valid options include `classic` and `brands`. For
   * Font Awesome Pro subscribers, valid options include, `classic`, `sharp`, `duotone`, `sharp-duotone`, and `brands`.
   * A valid kit code must be present to show pro icons via CDN. You can set `<html data-fa-kit-code="...">` to provide
   * one.
   */
  @property({ reflect: true }) family: string;

  /**
   * The name of the icon's variant. For Font Awesome, valid options include `thin`, `light`, `regular`, and `solid` for
   * the `classic` and `sharp` families. Some variants require a Font Awesome Pro subscription. Custom icon libraries
   * may or may not use this property.
   */
  @property({ reflect: true }) variant: string;

  /** Sets the width of the icon to match the cropped SVG viewBox. This operates like the Font `fa-width-auto` class. */
  @property({ attribute: "auto-width", type: Boolean, reflect: true }) autoWidth = false;

  /** Swaps the opacity of duotone icons. */
  @property({ attribute: "swap-opacity", type: Boolean, reflect: true }) swapOpacity = false;

  /**
   * An external URL of an SVG file. Be sure you trust the content you are including, as it will be executed as code and
   * can result in XSS attacks.
   */
  @property() src?: string;

  /**
   * An alternate description to use for assistive devices. If omitted, the icon will be considered presentational and
   * ignored by assistive devices.
   */
  @property() label = "";

  /** The name of a registered custom icon library. */
  @property({ reflect: true }) library = "default";

  /** Sets the rotation degree of the icon */
  @property({ type: Number, reflect: true }) rotate = 0;

  /** Sets the flip direction of the icon along the 'x' (horizontal), 'y' (vertical), or 'both' axes. */
  @property({ type: String, reflect: true }) flip?: "x" | "y" | "both";

  /** Sets the animation for the icon */
  @property({ type: String, reflect: true }) animation?: IconAnimation;

  connectedCallback() {
    super.connectedCallback();

    watchIcon(this);
  }

  firstUpdated(changedProperties: PropertyValues<this>) {
    super.firstUpdated(changedProperties);
    // Set initial rotate angle if rotate attribute is present
    if (this.hasAttribute("rotate")) {
      this.style.setProperty("--rotate-angle", `${this.rotate}deg`);
    }
    this.setIcon();
  }

  disconnectedCallback() {
    super.disconnectedCallback();
    unwatchIcon(this);
  }

  private async getIconSource(): Promise<IconSource> {
    const library = getIconLibrary(this.library);
    const family = this.family || getDefaultIconFamily();

    if (this.name && library) {
      let url: string | undefined;
      try {
        url = await library.resolver(this.name, family, this.variant, this.autoWidth);
      } catch {
        url = undefined;
      }
      return { url, fromLibrary: true };
    }

    return {
      url: this.src,
      fromLibrary: false,
    };
  }

  /** Given a URL, this function returns the resulting SVG element or an appropriate error symbol. */
  private resolveIcon = async (url: string, library?: IconLibrary): Promise<SVGResult> => {
    let fileData: Response;

    if (library?.spriteSheet) {
      // So this looks weird, but because of how SSR works, we need to first wait for the first update to complete.
      // After the first update, *then* we can set `this.svg` so we can then query + mutate the element.
      if (!this.hasUpdated) {
        await this.updateComplete;
      }

      this.svg = html`
        <svg part="svg">
          <use part="use" href="${url}"></use>
        </svg>
      `;

      // Using a templateResult requires the SVG to be written to the DOM first before we can grab the SVGElement
      // to be passed to the library's mutator function.
      await this.updateComplete;

      const svg = this.shadowRoot!.querySelector<SVGElement>("[part='svg']")!;

      if (typeof library.mutator === "function") {
        library.mutator(svg, this);
      }

      return this.svg;
    }

    try {
      fileData = await fetch(url, { mode: "cors" });
      if (!fileData.ok) return fileData.status === 410 ? CACHEABLE_ERROR : RETRYABLE_ERROR;
    } catch {
      return RETRYABLE_ERROR;
    }

    try {
      const div = document.createElement("div");
      div.innerHTML = await fileData.text();

      const svg = div.firstElementChild;
      if (svg?.tagName?.toLowerCase() !== "svg") return CACHEABLE_ERROR;

      if (!parser) parser = new DOMParser();
      const doc = parser.parseFromString(svg.outerHTML, "text/html");

      const svgEl = doc.body.querySelector("svg");
      if (!svgEl) return CACHEABLE_ERROR;

      svgEl.part.add("svg");
      return document.adoptNode(svgEl);
    } catch {
      return CACHEABLE_ERROR;
    }
  };

  @watch("label")
  handleLabelChange() {
    const hasLabel = typeof this.label === "string" && this.label.length > 0;

    if (hasLabel) {
      this.setAttribute("role", "img");
      this.setAttribute("aria-label", this.label);
      this.removeAttribute("aria-hidden");
    } else {
      this.removeAttribute("role");
      this.removeAttribute("aria-label");
      this.setAttribute("aria-hidden", "true");
    }
  }

  @watch(["family", "name", "library", "variant", "src", "autoWidth", "swapOpacity"], {
    waitUntilFirstUpdate: true,
  })
  async setIcon() {
    const { url, fromLibrary } = await this.getIconSource();
    const library = fromLibrary ? getIconLibrary(this.library) : undefined;

    if (!url) {
      this.svg = null;
      return;
    }

    let iconResolver = iconCache.get(url);
    if (!iconResolver) {
      iconResolver = this.resolveIcon(url, library);
      iconCache.set(url, iconResolver);
    }

    const svg = await iconResolver;

    if (svg === RETRYABLE_ERROR) {
      iconCache.delete(url);
    }

    const sourceAfterFetch = await this.getIconSource();
    if (url !== sourceAfterFetch.url) {
      // If the url has changed while fetching the icon, ignore this request
      return;
    }

    if (isTemplateResult(svg)) {
      this.svg = svg;
      return;
    }

    switch (svg) {
      case RETRYABLE_ERROR:
      case CACHEABLE_ERROR:
        this.svg = null;
        this.dispatchEvent(new WaErrorEvent());
        break;
      default:
        this.svg = svg.cloneNode(true) as SVGElement;
        library?.mutator?.(this.svg, this);
        this.dispatchEvent(new WaLoadEvent());
    }
  }

  updated(changedProperties: PropertyValues<this>) {
    super.updated(changedProperties);
    // Sometimes (like with SSR -> hydration) mutators don't get applied due to race conditions. This ensures mutators get re-applied.
    const library = getIconLibrary(this.library);
    // Set rotate angle whenever rotate attribute is present (not just on change)
    if (this.hasAttribute("rotate")) {
      this.style.setProperty("--rotate-angle", `${this.rotate}deg`);
    }
    const svg = this.shadowRoot?.querySelector("svg");
    if (svg) {
      library?.mutator?.(svg, this);
    }
  }

  render() {
    if (this.hasUpdated) {
      return this.svg;
    }
    return html`
      <svg part="svg" width="16" height="16"></svg>
    `;
  }
}

declare global {
  interface HTMLElementTagNameMap {
    "wa-icon": WaIcon;
  }
}

`````

### Expected (prettier)

`````ts
import { html } from "lit";
import { customElement, property, state } from "lit/decorators.js";
import { isTemplateResult } from "lit/directive-helpers.js";
import { WaErrorEvent } from "../../events/error.js";
import { WaLoadEvent } from "../../events/load.js";
import { watch } from "../../internal/watch.js";
import WebAwesomeElement from "../../internal/webawesome-element.js";
import styles from "./icon.styles.js";
import {
  getDefaultIconFamily,
  getIconLibrary,
  unwatchIcon,
  watchIcon,
  type IconLibrary,
} from "./library.js";

import type { HTMLTemplateResult, PropertyValues } from "lit";

const CACHEABLE_ERROR = Symbol();
const RETRYABLE_ERROR = Symbol();
type SVGResult =
  HTMLTemplateResult | SVGSVGElement | typeof RETRYABLE_ERROR | typeof CACHEABLE_ERROR;

let parser: DOMParser;
const iconCache = new Map<string, Promise<SVGResult>>();

interface IconSource {
  url?: string;
  fromLibrary: boolean;
}

export type IconAnimation =
  | "beat"
  | "fade"
  | "beat-fade"
  | "bounce"
  | "flip"
  | "shake"
  | "spin"
  | "spin-pulse"
  | "spin-reverse";

/**
 * @summary Icons are scalable vector symbols that represent actions, content, or status throughout your application.
 *  They support Font Awesome and custom icon libraries with animation presets.
 * @documentation https://webawesome.com/docs/components/icon
 * @status stable
 * @since 2.0
 *
 * @event wa-load - Emitted when the icon has loaded. When using `spriteSheet: true` this will not emit.
 * @event wa-error - Emitted when the icon fails to load due to an error. When using `spriteSheet: true` this will not emit.
 *
 * @csspart svg - The internal SVG element.
 * @csspart use - The `<use>` element generated when using `spriteSheet: true`
 *
 * @cssproperty [--animation-delay=0] Sets when the animation will start.
 * @cssproperty [--animation-direction=normal] Defines whether or not the animation should play in reverse on alternate cycles.
 * @cssproperty [--animation-duration=1s] Defines the length of time that an animation takes to complete one cycle.
 * @cssproperty [--animation-iteration-count=infinite] Defines the number of times an animation cycle is played.
 * @cssproperty [--animation-timing] Describes how the animation will progress over one cycle of its duration.
 * @cssproperty [--beat-fade-opacity] Set lowest opacity value an icon with `beat-fade` animation will fade to and from.
 * @cssproperty [--beat-fade-scale] Set max value that an icon with `beat-fade` animation will scale.
 * @cssproperty [--beat-scale] Set max value that an icon with `beat` animation will scale.
 * @cssproperty [--bounce-height] Set the max height an icon with `bounce` animation will jump to when bouncing.
 * @cssproperty [--bounce-jump-scale-x] Set the icon’s horizontal distortion (“squish”) at the top of the jump.
 * @cssproperty [--bounce-jump-scale-y] Set the icon’s vertical distortion (“squish”) at the top of the jump.
 * @cssproperty [--bounce-land-scale-x] Set the icon’s horizontal distortion (“squish”) when landing after the jump.
 * @cssproperty [--bounce-land-scale-y] Set the icon’s vertical distortion (“squish”) when landing after the jump.
 * @cssproperty [--bounce-rebound] Set the amount of rebound an icon with `bounce` animation has when landing after the jump.
 * @cssproperty [--bounce-start-scale-x] Set the icon’s horizontal distortion (“squish”) when starting to bounce.
 * @cssproperty [--bounce-start-scale-y] Set the icon’s vertical distortion (“squish”) when starting to bounce.
 * @cssproperty [--fade-opacity] Set lowest opacity value an icon with `fade` animation will fade to and from.
 * @cssproperty [--flip-angle] Set rotation angle of flip for an icon with `flip` animation. A positive angle denotes a clockwise rotation, a negative angle a counter-clockwise one.
 * @cssproperty [--flip-x] Set x-coordinate of the vector denoting the axis of rotation (between 0 and 1) for an icon with `flip` animation.
 * @cssproperty [--flip-y] Set y-coordinate of the vector denoting the axis of rotation (between 0 and 1) for an icon with `flip` animation.
 * @cssproperty [--flip-z] Set z-coordinate of the vector denoting the axis of rotation (between 0 and 1) for an icon with `flip` animation.
 * @cssproperty [--primary-color=currentColor] - Sets a duotone icon's primary color.
 * @cssproperty [--primary-opacity=1] - Sets a duotone icon's primary opacity.
 * @cssproperty [--secondary-color=currentColor] - Sets a duotone icon's secondary color.
 * @cssproperty [--secondary-opacity=0.4] - Sets a duotone icon's secondary opacity.
 */
@customElement("wa-icon")
export default class WaIcon extends WebAwesomeElement {
  static css = styles;

  @state() private svg: SVGElement | HTMLTemplateResult | null = null;

  /** The name of the icon to draw. Available names depend on the icon library being used. */
  @property({ reflect: true }) name?: string;

  /**
   * The family of icons to choose from. For Font Awesome Free, valid options include `classic` and `brands`. For
   * Font Awesome Pro subscribers, valid options include, `classic`, `sharp`, `duotone`, `sharp-duotone`, and `brands`.
   * A valid kit code must be present to show pro icons via CDN. You can set `<html data-fa-kit-code="...">` to provide
   * one.
   */
  @property({ reflect: true }) family: string;

  /**
   * The name of the icon's variant. For Font Awesome, valid options include `thin`, `light`, `regular`, and `solid` for
   * the `classic` and `sharp` families. Some variants require a Font Awesome Pro subscription. Custom icon libraries
   * may or may not use this property.
   */
  @property({ reflect: true }) variant: string;

  /** Sets the width of the icon to match the cropped SVG viewBox. This operates like the Font `fa-width-auto` class. */
  @property({ attribute: "auto-width", type: Boolean, reflect: true }) autoWidth = false;

  /** Swaps the opacity of duotone icons. */
  @property({ attribute: "swap-opacity", type: Boolean, reflect: true }) swapOpacity = false;

  /**
   * An external URL of an SVG file. Be sure you trust the content you are including, as it will be executed as code and
   * can result in XSS attacks.
   */
  @property() src?: string;

  /**
   * An alternate description to use for assistive devices. If omitted, the icon will be considered presentational and
   * ignored by assistive devices.
   */
  @property() label = "";

  /** The name of a registered custom icon library. */
  @property({ reflect: true }) library = "default";

  /** Sets the rotation degree of the icon */
  @property({ type: Number, reflect: true }) rotate = 0;

  /** Sets the flip direction of the icon along the 'x' (horizontal), 'y' (vertical), or 'both' axes. */
  @property({ type: String, reflect: true }) flip?: "x" | "y" | "both";

  /** Sets the animation for the icon */
  @property({ type: String, reflect: true }) animation?: IconAnimation;

  connectedCallback() {
    super.connectedCallback();

    watchIcon(this);
  }

  firstUpdated(changedProperties: PropertyValues<this>) {
    super.firstUpdated(changedProperties);
    // Set initial rotate angle if rotate attribute is present
    if (this.hasAttribute("rotate")) {
      this.style.setProperty("--rotate-angle", `${this.rotate}deg`);
    }
    this.setIcon();
  }

  disconnectedCallback() {
    super.disconnectedCallback();
    unwatchIcon(this);
  }

  private async getIconSource(): Promise<IconSource> {
    const library = getIconLibrary(this.library);
    const family = this.family || getDefaultIconFamily();

    if (this.name && library) {
      let url: string | undefined;
      try {
        url = await library.resolver(this.name, family, this.variant, this.autoWidth);
      } catch {
        url = undefined;
      }
      return { url, fromLibrary: true };
    }

    return {
      url: this.src,
      fromLibrary: false,
    };
  }

  /** Given a URL, this function returns the resulting SVG element or an appropriate error symbol. */
  private resolveIcon = async (url: string, library?: IconLibrary): Promise<SVGResult> => {
    let fileData: Response;

    if (library?.spriteSheet) {
      // So this looks weird, but because of how SSR works, we need to first wait for the first update to complete.
      // After the first update, *then* we can set `this.svg` so we can then query + mutate the element.
      if (!this.hasUpdated) {
        await this.updateComplete;
      }

      this.svg = html`
        <svg part="svg">
          <use part="use" href="${url}"></use>
        </svg>
      `;

      // Using a templateResult requires the SVG to be written to the DOM first before we can grab the SVGElement
      // to be passed to the library's mutator function.
      await this.updateComplete;

      const svg = this.shadowRoot!.querySelector<SVGElement>("[part='svg']")!;

      if (typeof library.mutator === "function") {
        library.mutator(svg, this);
      }

      return this.svg;
    }

    try {
      fileData = await fetch(url, { mode: "cors" });
      if (!fileData.ok) return fileData.status === 410 ? CACHEABLE_ERROR : RETRYABLE_ERROR;
    } catch {
      return RETRYABLE_ERROR;
    }

    try {
      const div = document.createElement("div");
      div.innerHTML = await fileData.text();

      const svg = div.firstElementChild;
      if (svg?.tagName?.toLowerCase() !== "svg") return CACHEABLE_ERROR;

      if (!parser) parser = new DOMParser();
      const doc = parser.parseFromString(svg.outerHTML, "text/html");

      const svgEl = doc.body.querySelector("svg");
      if (!svgEl) return CACHEABLE_ERROR;

      svgEl.part.add("svg");
      return document.adoptNode(svgEl);
    } catch {
      return CACHEABLE_ERROR;
    }
  };

  @watch("label")
  handleLabelChange() {
    const hasLabel = typeof this.label === "string" && this.label.length > 0;

    if (hasLabel) {
      this.setAttribute("role", "img");
      this.setAttribute("aria-label", this.label);
      this.removeAttribute("aria-hidden");
    } else {
      this.removeAttribute("role");
      this.removeAttribute("aria-label");
      this.setAttribute("aria-hidden", "true");
    }
  }

  @watch(["family", "name", "library", "variant", "src", "autoWidth", "swapOpacity"], {
    waitUntilFirstUpdate: true,
  })
  async setIcon() {
    const { url, fromLibrary } = await this.getIconSource();
    const library = fromLibrary ? getIconLibrary(this.library) : undefined;

    if (!url) {
      this.svg = null;
      return;
    }

    let iconResolver = iconCache.get(url);
    if (!iconResolver) {
      iconResolver = this.resolveIcon(url, library);
      iconCache.set(url, iconResolver);
    }

    const svg = await iconResolver;

    if (svg === RETRYABLE_ERROR) {
      iconCache.delete(url);
    }

    const sourceAfterFetch = await this.getIconSource();
    if (url !== sourceAfterFetch.url) {
      // If the url has changed while fetching the icon, ignore this request
      return;
    }

    if (isTemplateResult(svg)) {
      this.svg = svg;
      return;
    }

    switch (svg) {
      case RETRYABLE_ERROR:
      case CACHEABLE_ERROR:
        this.svg = null;
        this.dispatchEvent(new WaErrorEvent());
        break;
      default:
        this.svg = svg.cloneNode(true) as SVGElement;
        library?.mutator?.(this.svg, this);
        this.dispatchEvent(new WaLoadEvent());
    }
  }

  updated(changedProperties: PropertyValues<this>) {
    super.updated(changedProperties);
    // Sometimes (like with SSR -> hydration) mutators don't get applied due to race conditions. This ensures mutators get re-applied.
    const library = getIconLibrary(this.library);
    // Set rotate angle whenever rotate attribute is present (not just on change)
    if (this.hasAttribute("rotate")) {
      this.style.setProperty("--rotate-angle", `${this.rotate}deg`);
    }
    const svg = this.shadowRoot?.querySelector("svg");
    if (svg) {
      library?.mutator?.(svg, this);
    }
  }

  render() {
    if (this.hasUpdated) {
      return this.svg;
    }
    return html`
      <svg part="svg" width="16" height="16"></svg>
    `;
  }
}

declare global {
  interface HTMLElementTagNameMap {
    "wa-icon": WaIcon;
  }
}

`````
