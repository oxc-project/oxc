# externals/webawesome/carousel/carousel.ts

## Option 1

`````json
{"printWidth":80}
`````

### Diff

`````diff
===================================================================
--- prettier
+++ oxfmt
@@ -688,86 +688,88 @@
         >
           <slot @slotchange=${() => this.requestUpdate()}></slot>
         </div>
 
-        ${
-          this.navigation
-            ? html`
-                <div part="navigation" class="navigation">
-                  <button
-                    part="navigation-button navigation-button-previous"
-                    class="${classMap({
+        ${this.navigation
+          ? html`
+              <div part="navigation" class="navigation">
+                <button
+                  part="navigation-button navigation-button-previous"
+                  class="${classMap({
                     "navigation-button": true,
                     "navigation-button-previous": true,
                     "navigation-button-disabled": !prevEnabled,
                   })}"
-                    aria-label="${this.localize.term("previousSlide")}"
-                    aria-controls="scroll-container"
-                    aria-disabled="${prevEnabled ? "false" : "true"}"
-                    @click=${prevEnabled ? () => this.previous() : null}
-                  >
-                    <slot name="previous-icon">
-                      <wa-icon
-                        library="system"
-                        name="${isRTL ? "chevron-right" : "chevron-left"}"
-                      ></wa-icon>
-                    </slot>
-                  </button>
+                  aria-label="${this.localize.term("previousSlide")}"
+                  aria-controls="scroll-container"
+                  aria-disabled="${prevEnabled ? "false" : "true"}"
+                  @click=${prevEnabled ? () => this.previous() : null}
+                >
+                  <slot name="previous-icon">
+                    <wa-icon
+                      library="system"
+                      name="${isRTL ? "chevron-right" : "chevron-left"}"
+                    ></wa-icon>
+                  </slot>
+                </button>
 
-                  <button
-                    part="navigation-button navigation-button-next"
-                    class=${classMap({
+                <button
+                  part="navigation-button navigation-button-next"
+                  class=${classMap({
                     "navigation-button": true,
                     "navigation-button-next": true,
                     "navigation-button-disabled": !nextEnabled,
                   })}
-                    aria-label="${this.localize.term("nextSlide")}"
-                    aria-controls="scroll-container"
-                    aria-disabled="${nextEnabled ? "false" : "true"}"
-                    @click=${nextEnabled ? () => this.next() : null}
-                  >
-                    <slot name="next-icon">
-                      <wa-icon
-                        library="system"
-                        name="${isRTL ? "chevron-left" : "chevron-right"}"
-                      ></wa-icon>
-                    </slot>
-                  </button>
-                </div>
-              `
-            : ""
-        }
-        ${
-          this.pagination
-            ? html`
-                <div
-                  part="pagination"
-                  role="tablist"
-                  class="pagination"
+                  aria-label="${this.localize.term("nextSlide")}"
                   aria-controls="scroll-container"
+                  aria-disabled="${nextEnabled ? "false" : "true"}"
+                  @click=${nextEnabled ? () => this.next() : null}
                 >
-                  ${map(range(pagesCount), (index) => {
+                  <slot name="next-icon">
+                    <wa-icon
+                      library="system"
+                      name="${isRTL ? "chevron-left" : "chevron-right"}"
+                    ></wa-icon>
+                  </slot>
+                </button>
+              </div>
+            `
+          : ""}
+        ${this.pagination
+          ? html`
+              <div
+                part="pagination"
+                role="tablist"
+                class="pagination"
+                aria-controls="scroll-container"
+              >
+                ${map(range(pagesCount), (index) => {
                   const isActive = index === currentPage;
                   return html`
                     <button
-                      part="pagination-item ${isActive ? "pagination-item-active" : ""}"
+                      part="pagination-item ${isActive
+                        ? "pagination-item-active"
+                        : ""}"
                       class="${classMap({
                         "pagination-item": true,
                         "pagination-item-active": isActive,
                       })}"
                       role="tab"
                       aria-selected="${isActive ? "true" : "false"}"
-                      aria-label="${this.localize.term("goToSlide", index + 1, pagesCount)}"
+                      aria-label="${this.localize.term(
+                        "goToSlide",
+                        index + 1,
+                        pagesCount,
+                      )}"
                       tabindex=${isActive ? "0" : "-1"}
                       @click=${() => this.goToSlide(index * slidesPerMove)}
                       @keydown=${this.handleKeyDown}
                     ></button>
                   `;
                 })}
-                </div>
-              `
-            : html``
-        }
+              </div>
+            `
+          : html``}
       </div>
     `;
   }
 }

`````

### Actual (oxfmt)

`````ts
import "../../internal/scrollend-polyfill.js";

import type { PropertyValueMap } from "lit";
import { html, isServer } from "lit";
import {
  customElement,
  eventOptions,
  property,
  query,
  state,
} from "lit/decorators.js";
import { classMap } from "lit/directives/class-map.js";
import { map } from "lit/directives/map.js";
import { range } from "lit/directives/range.js";
import { styleMap } from "lit/directives/style-map.js";
import { WaSlideChangeEvent } from "../../events/slide-change.js";
import { prefersReducedMotion } from "../../internal/animate.js";
import { waitForEvent } from "../../internal/event.js";
import { clamp } from "../../internal/math.js";
import { watch } from "../../internal/watch.js";
import WebAwesomeElement from "../../internal/webawesome-element.js";
import { LocalizeController } from "../../utilities/localize.js";
import type WaCarouselItem from "../carousel-item/carousel-item.js";
import "../icon/icon.js";
import { AutoplayController } from "./autoplay-controller.js";
import styles from "./carousel.styles.js";

/**
 * @summary Carousels display a series of content slides along a horizontal or vertical axis, one or more at a time.
 *  Users can navigate between slides with controls, pagination, or autoplay.
 *
 * @since 2.2
 * @status experimental
 *
 * @dependency wa-icon
 *
 * @event {{ index: number, slide: WaCarouselItem }} wa-slide-change - Emitted when the active slide changes.
 *
 * @slot - The carousel's main content, one or more `<wa-carousel-item>` elements.
 * @slot next-icon - Optional next icon to use instead of the default. Works best with `<wa-icon>`.
 * @slot previous-icon - Optional previous icon to use instead of the default. Works best with `<wa-icon>`.
 *
 * @csspart base - The carousel's internal wrapper.
 * @csspart scroll-container - The scroll container that wraps the slides.
 * @csspart pagination - The pagination indicators wrapper.
 * @csspart pagination-item - The pagination indicator.
 * @csspart pagination-item-active - Applied when the item is active.
 * @csspart navigation - The navigation wrapper.
 * @csspart navigation-button - The navigation button.
 * @csspart navigation-button-previous - Applied to the previous button.
 * @csspart navigation-button-next - Applied to the next button.
 *
 * @cssproperty [--aspect-ratio=16/9] - The aspect ratio of each slide.
 * @cssproperty --scroll-hint - The amount of padding to apply to the scroll area, allowing adjacent slides to become
 *  partially visible as a scroll hint.
 * @cssproperty [--slide-gap=var(--wa-space-m)] - The space between each slide.
 */
@customElement("wa-carousel")
export default class WaCarousel extends WebAwesomeElement {
  static css = styles;

  /** When set, allows the user to navigate the carousel in the same direction indefinitely. */
  @property({ type: Boolean, reflect: true }) loop = false;

  @property({ type: Number, reflect: true }) slides = 0;
  @property({ type: Number, reflect: true }) currentSlide = 0;

  /** When set, show the carousel's navigation. */
  @property({ type: Boolean, reflect: true }) navigation = false;

  /** When set, show the carousel's pagination indicators. */
  @property({ type: Boolean, reflect: true }) pagination = false;

  /** When set, the slides will scroll automatically when the user is not interacting with them.  */
  @property({ type: Boolean, reflect: true }) autoplay = false;

  /** Specifies the amount of time, in milliseconds, between each automatic scroll.  */
  @property({ type: Number, attribute: "autoplay-interval" }) autoplayInterval =
    3000;

  /** Specifies how many slides should be shown at a given time.  */
  @property({ type: Number, attribute: "slides-per-page" }) slidesPerPage = 1;

  /**
   * Specifies the number of slides the carousel will advance when scrolling, useful when specifying a `slides-per-page`
   * greater than one. It can't be higher than `slides-per-page`.
   */
  @property({ type: Number, attribute: "slides-per-move" }) slidesPerMove = 1;

  /** Specifies the orientation in which the carousel will lay out.  */
  @property() orientation: "horizontal" | "vertical" = "horizontal";

  /** When set, it is possible to scroll through the slides by dragging them with the mouse. */
  @property({ type: Boolean, reflect: true, attribute: "mouse-dragging" })
  mouseDragging = false;

  @query(".slides") scrollContainer: HTMLElement;
  @query(".pagination") paginationContainer: HTMLElement;

  // The index of the active slide
  @state() activeSlide = 0;

  @state() scrolling = false;

  @state() dragging = false;

  private autoplayController = new AutoplayController(this, () => this.next());
  private dragStartPosition: [number, number] = [-1, -1];
  private readonly localize = new LocalizeController(this);
  private mutationObserver: MutationObserver;
  private resizeObserver?: ResizeObserver;
  private pendingSlideChange = false;

  connectedCallback(): void {
    super.connectedCallback();

    // SSR guard: setAttribute is not available during server-side rendering
    if (!isServer) {
      this.setAttribute("role", "region");
      this.setAttribute("aria-label", this.localize.term("carousel"));
    }
  }

  disconnectedCallback(): void {
    super.disconnectedCallback();
    this.mutationObserver?.disconnect();
    this.resizeObserver?.disconnect();
  }

  protected firstUpdated(): void {
    this.initializeSlides();
    this.mutationObserver = new MutationObserver(this.handleSlotChange);
    this.mutationObserver.observe(this, {
      childList: true,
      subtree: true,
    });

    // When the carousel is placed inside a hidden container (e.g. an inactive tab panel),
    // initializeSlides() runs before the element has layout dimensions. The IntersectionObserver
    // inside synchronizeSlides() then reports all slides as non-intersecting and marks them
    // `inert`, making their contents unclickable until the user interacts with the carousel.
    // Re-run synchronizeSlides() once the carousel gains visible dimensions to correct this.
    this.resizeObserver = new ResizeObserver(() => {
      if (
        this.scrollContainer?.clientWidth ||
        this.scrollContainer?.clientHeight
      ) {
        this.synchronizeSlides();
        this.resizeObserver?.disconnect();
        this.resizeObserver = undefined;
      }
    });
    this.resizeObserver.observe(this);
  }

  protected willUpdate(
    changedProperties: PropertyValueMap<WaCarousel> | Map<PropertyKey, unknown>,
  ): void {
    // Ensure the slidesPerMove is never higher than the slidesPerPage
    if (
      changedProperties.has("slidesPerMove") ||
      changedProperties.has("slidesPerPage")
    ) {
      this.slidesPerMove = Math.min(this.slidesPerMove, this.slidesPerPage);
    }
  }

  private getPageCount() {
    const slidesCount = this.getSlides().length;
    const { slidesPerPage, slidesPerMove, loop } = this;

    const pages = loop
      ? slidesCount / slidesPerMove
      : (slidesCount - slidesPerPage) / slidesPerMove + 1;

    return Math.ceil(pages);
  }

  private getCurrentPage() {
    return Math.ceil(this.activeSlide / this.slidesPerMove);
  }

  private canScrollNext(): boolean {
    return this.loop || this.getCurrentPage() < this.getPageCount() - 1;
  }

  private canScrollPrev(): boolean {
    return this.loop || this.getCurrentPage() > 0;
  }

  /** @internal Gets all carousel items. */
  private getSlides({
    excludeClones = true,
  }: { excludeClones?: boolean } = {}) {
    return [...this.children].filter(
      (el: HTMLElement) =>
        this.isCarouselItem(el) &&
        (!excludeClones || !el.hasAttribute("data-clone")),
    ) as WaCarouselItem[];
  }

  private handleClick(event: MouseEvent) {
    if (
      this.dragging &&
      this.dragStartPosition[0] > 0 &&
      this.dragStartPosition[1] > 0
    ) {
      const deltaX = Math.abs(this.dragStartPosition[0] - event.clientX);
      const deltaY = Math.abs(this.dragStartPosition[1] - event.clientY);
      const delta = Math.sqrt(deltaX * deltaX + deltaY * deltaY);

      // Prevents clicks on interactive elements while dragging if the click is within a small range. This prevents
      // accidental drags from interfering with intentional clicks.
      if (delta >= 10) {
        event.preventDefault();
      }
    }
  }

  private handleKeyDown(event: KeyboardEvent) {
    if (
      [
        "ArrowLeft",
        "ArrowRight",
        "ArrowUp",
        "ArrowDown",
        "Home",
        "End",
      ].includes(event.key)
    ) {
      const target = event.target as HTMLElement;
      const isRtl = this.localize.dir() === "rtl";
      const isFocusInPagination =
        target.closest('[part~="pagination-item"]') !== null;
      const isNext =
        event.key === "ArrowDown" ||
        (!isRtl && event.key === "ArrowRight") ||
        (isRtl && event.key === "ArrowLeft");
      const isPrevious =
        event.key === "ArrowUp" ||
        (!isRtl && event.key === "ArrowLeft") ||
        (isRtl && event.key === "ArrowRight");

      event.preventDefault();

      if (isPrevious) {
        this.previous();
      }

      if (isNext) {
        this.next();
      }

      if (event.key === "Home") {
        this.goToSlide(0);
      }

      if (event.key === "End") {
        this.goToSlide(this.getSlides().length - 1);
      }

      if (isFocusInPagination) {
        this.updateComplete.then(() => {
          const activePaginationItem =
            this.shadowRoot?.querySelector<HTMLButtonElement>(
              '[part~="pagination-item-active"]',
            );

          if (activePaginationItem) {
            activePaginationItem.focus();
          }
        });
      }
    }
  }

  private handleMouseDragStart(event: PointerEvent) {
    const canDrag = this.mouseDragging && event.button === 0;
    if (canDrag) {
      event.preventDefault();

      document.addEventListener("pointermove", this.handleMouseDrag, {
        capture: true,
        passive: true,
      });
      document.addEventListener("pointerup", this.handleMouseDragEnd, {
        capture: true,
        once: true,
      });
    }
  }

  private handleMouseDrag = (event: PointerEvent) => {
    if (!this.dragging) {
      // Start dragging if it hasn't yet
      this.scrollContainer.style.setProperty("scroll-snap-type", "none");
      this.dragging = true;
      this.dragStartPosition = [event.clientX, event.clientY];
    }

    this.scrollContainer.scrollBy({
      left: -event.movementX,
      top: -event.movementY,
      behavior: "instant",
    });
  };

  private handleMouseDragEnd = () => {
    const scrollContainer = this.scrollContainer;

    document.removeEventListener("pointermove", this.handleMouseDrag, {
      capture: true,
    });

    // get the current scroll position
    const startLeft = scrollContainer.scrollLeft;
    const startTop = scrollContainer.scrollTop;

    // remove the scroll-snap-type property so that the browser will snap the slide to the correct position
    scrollContainer.style.removeProperty("scroll-snap-type");

    // fix(safari): forcing a style recalculation doesn't seem to immediately update the scroll
    // position in Safari. Setting "overflow" to "hidden" should force this behavior.
    scrollContainer.style.setProperty("overflow", "hidden");

    // get the final scroll position to the slide snapped by the browser
    const finalLeft = scrollContainer.scrollLeft;
    const finalTop = scrollContainer.scrollTop;

    // restore the scroll position to the original one, so that it can be smoothly animated if needed
    scrollContainer.style.removeProperty("overflow");
    scrollContainer.style.setProperty("scroll-snap-type", "none");
    scrollContainer.scrollTo({
      left: startLeft,
      top: startTop,
      behavior: "instant",
    });

    requestAnimationFrame(async () => {
      if (startLeft !== finalLeft || startTop !== finalTop) {
        scrollContainer.scrollTo({
          left: finalLeft,
          top: finalTop,
          behavior: prefersReducedMotion() ? "auto" : "smooth",
        });
        await waitForEvent(scrollContainer, "scrollend");
      }

      scrollContainer.style.removeProperty("scroll-snap-type");

      this.dragging = false;
      this.dragStartPosition = [-1, -1];
      this.handleScrollEnd();
    });
  };

  @eventOptions({ passive: true })
  private handleScroll() {
    this.scrolling = true;
    if (!this.pendingSlideChange) {
      this.synchronizeSlides();
    }
  }

  /** @internal Synchronizes the slides with the IntersectionObserver API. */
  private synchronizeSlides() {
    const io = new IntersectionObserver(
      (entries) => {
        io.disconnect();

        for (const entry of entries) {
          const slide = entry.target;
          slide.toggleAttribute("inert", !entry.isIntersecting);
          slide.classList.toggle("--in-view", entry.isIntersecting);
          slide.setAttribute(
            "aria-hidden",
            entry.isIntersecting ? "false" : "true",
          );
        }

        const firstIntersecting = entries.find((entry) => entry.isIntersecting);

        if (!firstIntersecting) {
          return;
        }

        const slidesWithClones = this.getSlides({ excludeClones: false });
        const slidesCount = this.getSlides().length;

        // Update the current index based on the first visible slide
        const slideIndex = slidesWithClones.indexOf(
          firstIntersecting.target as WaCarouselItem,
        );
        // Normalize the index to ignore clones
        const normalizedIndex = this.loop
          ? slideIndex - this.slidesPerPage
          : slideIndex;

        if (firstIntersecting) {
          // Set the index to the closest "snappable" slide
          this.activeSlide =
            (Math.ceil(normalizedIndex / this.slidesPerMove) *
              this.slidesPerMove +
              slidesCount) %
            slidesCount;

          if (!this.scrolling) {
            if (
              this.loop &&
              firstIntersecting.target.hasAttribute("data-clone")
            ) {
              const clonePosition = Number(
                firstIntersecting.target.getAttribute("data-clone"),
              );
              // Scrolls to the original slide without animating, so the user won't notice that the position has changed
              this.goToSlide(clonePosition, "instant");
            }
          }
        }
      },
      {
        root: this.scrollContainer,
        threshold: 0.6,
      },
    );

    this.getSlides({ excludeClones: false }).forEach((slide) => {
      io.observe(slide);
    });
  }

  private handleScrollEnd() {
    if (!this.scrolling || this.dragging) return;

    this.synchronizeSlides();

    this.scrolling = false;
    this.pendingSlideChange = false;
    this.synchronizeSlides();
  }

  private isCarouselItem(node: Node): node is WaCarouselItem {
    return (
      node instanceof Element &&
      node.tagName.toLowerCase() === "wa-carousel-item"
    );
  }

  private handleSlotChange = (mutations: MutationRecord[]) => {
    const needsInitialization = mutations.some((mutation) =>
      [...mutation.addedNodes, ...mutation.removedNodes].some(
        (el: HTMLElement) =>
          this.isCarouselItem(el) && !el.hasAttribute("data-clone"),
      ),
    );

    // Reinitialize the carousel if a carousel item has been added or removed
    if (needsInitialization) {
      this.initializeSlides();
    }

    this.requestUpdate();
  };

  @watch("loop", { waitUntilFirstUpdate: true })
  @watch("slidesPerPage", { waitUntilFirstUpdate: true })
  initializeSlides() {
    // Removes all the cloned elements from the carousel
    this.getSlides({ excludeClones: false }).forEach((slide, index) => {
      slide.classList.remove("--in-view");
      slide.classList.remove("--is-active");
      slide.setAttribute(
        "aria-label",
        this.localize.term("slideNum", index + 1),
      );

      if (slide.hasAttribute("data-clone")) {
        slide.remove();
      }
    });

    this.updateSlidesSnap();

    if (this.loop) {
      // Creates clones to be placed before and after the original elements to simulate infinite scrolling
      this.createClones();
    }

    // Because the DOM may be changed, restore the scroll position to the active slide
    this.goToSlide(this.activeSlide, "auto");

    this.synchronizeSlides();
  }

  private createClones() {
    const slides = this.getSlides();

    const slidesPerPage = this.slidesPerPage;
    const lastSlides = slides.slice(-slidesPerPage);
    const firstSlides = slides.slice(0, slidesPerPage);

    lastSlides.reverse().forEach((slide, i) => {
      const clone = slide.cloneNode(true) as HTMLElement;
      clone.setAttribute("data-clone", String(slides.length - i - 1));
      this.prepend(clone);
    });

    firstSlides.forEach((slide, i) => {
      const clone = slide.cloneNode(true) as HTMLElement;
      clone.setAttribute("data-clone", String(i));
      this.append(clone);
    });
  }

  @watch("activeSlide")
  handleSlideChange() {
    const slides = this.getSlides();
    slides.forEach((slide, i) => {
      slide.classList.toggle("--is-active", i === this.activeSlide);
    });

    // Do not emit an event on first render
    if (this.hasUpdated) {
      this.dispatchEvent(
        new WaSlideChangeEvent({
          index: this.activeSlide,
          slide: slides[this.activeSlide],
        }),
      );
    }
  }

  @watch("slidesPerMove")
  updateSlidesSnap() {
    const slides = this.getSlides();

    const slidesPerMove = this.slidesPerMove;
    slides.forEach((slide, i) => {
      const shouldSnap = (i + slidesPerMove) % slidesPerMove === 0;
      if (shouldSnap) {
        slide.style.removeProperty("scroll-snap-align");
      } else {
        slide.style.setProperty("scroll-snap-align", "none");
      }
    });
  }

  @watch("autoplay")
  handleAutoplayChange() {
    this.autoplayController.stop();
    if (this.autoplay) {
      this.autoplayController.start(this.autoplayInterval);
    }
  }

  /**
   * Move the carousel backward by `slides-per-move` slides.
   *
   * @param behavior - The behavior used for scrolling.
   */
  previous(behavior: ScrollBehavior = "smooth") {
    this.goToSlide(this.activeSlide - this.slidesPerMove, behavior);
  }

  /**
   * Move the carousel forward by `slides-per-move` slides.
   *
   * @param behavior - The behavior used for scrolling.
   */
  next(behavior: ScrollBehavior = "smooth") {
    this.goToSlide(this.activeSlide + this.slidesPerMove, behavior);
  }

  /**
   * Scrolls the carousel to the slide specified by `index`.
   *
   * @param index - The slide index.
   * @param behavior - The behavior used for scrolling.
   */
  goToSlide(index: number, behavior: ScrollBehavior = "smooth") {
    const { slidesPerPage, loop } = this;

    const slides = this.getSlides();
    const slidesWithClones = this.getSlides({ excludeClones: false });

    // No need to do anything in case there are no items in the carousel
    if (!slides.length) {
      return;
    }

    // Sets the next index without taking into account clones, if any.
    const newActiveSlide = loop
      ? (index + slides.length) % slides.length
      : clamp(index, 0, slides.length - slidesPerPage);
    this.activeSlide = newActiveSlide;

    const isRtl = this.localize.dir() === "rtl";

    // Get the index of the next slide. For looping carousel it adds `slidesPerPage`
    // to normalize the starting index in order to ignore the first nth clones.
    // For RTL it needs to scroll to the last slide of the page.
    const nextSlideIndex = clamp(
      index + (loop ? slidesPerPage : 0) + (isRtl ? slidesPerPage - 1 : 0),
      0,
      slidesWithClones.length - 1,
    );

    const nextSlide = slidesWithClones[nextSlideIndex];

    this.scrollToSlide(nextSlide, prefersReducedMotion() ? "auto" : behavior);
  }

  private scrollToSlide(
    slide: HTMLElement,
    behavior: ScrollBehavior = "smooth",
  ) {
    // Since the geometry doesn't happen until rAF, we don't know if we'll be scrolling or not...
    // It's best to assume that we will and cleanup in the else case below if we didn't need to
    this.pendingSlideChange = true;
    window.requestAnimationFrame(() => {
      // This can happen if goToSlide is called before the scroll container is rendered
      // We will have correctly set the activeSlide in goToSlide which will get picked up when initializeSlides is called.
      if (!this.scrollContainer) {
        return;
      }

      const scrollContainer = this.scrollContainer;
      const scrollContainerRect = scrollContainer.getBoundingClientRect();
      const nextSlideRect = slide.getBoundingClientRect();

      const nextLeft = nextSlideRect.left - scrollContainerRect.left;
      const nextTop = nextSlideRect.top - scrollContainerRect.top;

      if (nextLeft || nextTop) {
        // This is here just in case someone set it back to false
        // between rAF being requested and the callback actually running
        this.pendingSlideChange = true;
        scrollContainer.scrollTo({
          left: nextLeft + scrollContainer.scrollLeft,
          top: nextTop + scrollContainer.scrollTop,
          behavior,
        });
      } else {
        this.pendingSlideChange = false;
      }
    });
  }

  render() {
    const { slidesPerMove, scrolling } = this;

    let pagesCount = 0;
    let currentPage = 0;
    let prevEnabled = false;
    let nextEnabled = false;

    // @TODO: This is a super hacky way to get rid of hydration mismatch errors. The ideal solution is users being able to pass in `pagesCount` and `currentPage` and then on firstUpdated to we update the value for them.
    if (this.hasUpdated) {
      pagesCount = this.getPageCount();
      currentPage = this.getCurrentPage();
      prevEnabled = this.canScrollPrev();
      nextEnabled = this.canScrollNext();
    }

    // We can't rely on `this.matches()` on the server.
    const isRTL = isServer ? this.dir === "rtl" : this.localize.dir() === "rtl";

    return html`
      <div part="base" class="carousel">
        <div
          id="scroll-container"
          part="scroll-container"
          class="${classMap({
            slides: true,
            "slides-horizontal": this.orientation === "horizontal",
            "slides-vertical": this.orientation === "vertical",
            "slides-dragging": this.dragging,
          })}"
          style=${styleMap({ "--slides-per-page": this.slidesPerPage })}
          aria-busy="${scrolling ? "true" : "false"}"
          aria-atomic="true"
          tabindex="0"
          @keydown=${this.handleKeyDown}
          @mousedown="${this.handleMouseDragStart}"
          @scroll="${this.handleScroll}"
          @scrollend=${this.handleScrollEnd}
          @click=${this.handleClick}
        >
          <slot @slotchange=${() => this.requestUpdate()}></slot>
        </div>

        ${this.navigation
          ? html`
              <div part="navigation" class="navigation">
                <button
                  part="navigation-button navigation-button-previous"
                  class="${classMap({
                    "navigation-button": true,
                    "navigation-button-previous": true,
                    "navigation-button-disabled": !prevEnabled,
                  })}"
                  aria-label="${this.localize.term("previousSlide")}"
                  aria-controls="scroll-container"
                  aria-disabled="${prevEnabled ? "false" : "true"}"
                  @click=${prevEnabled ? () => this.previous() : null}
                >
                  <slot name="previous-icon">
                    <wa-icon
                      library="system"
                      name="${isRTL ? "chevron-right" : "chevron-left"}"
                    ></wa-icon>
                  </slot>
                </button>

                <button
                  part="navigation-button navigation-button-next"
                  class=${classMap({
                    "navigation-button": true,
                    "navigation-button-next": true,
                    "navigation-button-disabled": !nextEnabled,
                  })}
                  aria-label="${this.localize.term("nextSlide")}"
                  aria-controls="scroll-container"
                  aria-disabled="${nextEnabled ? "false" : "true"}"
                  @click=${nextEnabled ? () => this.next() : null}
                >
                  <slot name="next-icon">
                    <wa-icon
                      library="system"
                      name="${isRTL ? "chevron-left" : "chevron-right"}"
                    ></wa-icon>
                  </slot>
                </button>
              </div>
            `
          : ""}
        ${this.pagination
          ? html`
              <div
                part="pagination"
                role="tablist"
                class="pagination"
                aria-controls="scroll-container"
              >
                ${map(range(pagesCount), (index) => {
                  const isActive = index === currentPage;
                  return html`
                    <button
                      part="pagination-item ${isActive
                        ? "pagination-item-active"
                        : ""}"
                      class="${classMap({
                        "pagination-item": true,
                        "pagination-item-active": isActive,
                      })}"
                      role="tab"
                      aria-selected="${isActive ? "true" : "false"}"
                      aria-label="${this.localize.term(
                        "goToSlide",
                        index + 1,
                        pagesCount,
                      )}"
                      tabindex=${isActive ? "0" : "-1"}
                      @click=${() => this.goToSlide(index * slidesPerMove)}
                      @keydown=${this.handleKeyDown}
                    ></button>
                  `;
                })}
              </div>
            `
          : html``}
      </div>
    `;
  }
}

declare global {
  interface HTMLElementTagNameMap {
    "wa-carousel": WaCarousel;
  }
}

`````

### Expected (prettier)

`````ts
import "../../internal/scrollend-polyfill.js";

import type { PropertyValueMap } from "lit";
import { html, isServer } from "lit";
import {
  customElement,
  eventOptions,
  property,
  query,
  state,
} from "lit/decorators.js";
import { classMap } from "lit/directives/class-map.js";
import { map } from "lit/directives/map.js";
import { range } from "lit/directives/range.js";
import { styleMap } from "lit/directives/style-map.js";
import { WaSlideChangeEvent } from "../../events/slide-change.js";
import { prefersReducedMotion } from "../../internal/animate.js";
import { waitForEvent } from "../../internal/event.js";
import { clamp } from "../../internal/math.js";
import { watch } from "../../internal/watch.js";
import WebAwesomeElement from "../../internal/webawesome-element.js";
import { LocalizeController } from "../../utilities/localize.js";
import type WaCarouselItem from "../carousel-item/carousel-item.js";
import "../icon/icon.js";
import { AutoplayController } from "./autoplay-controller.js";
import styles from "./carousel.styles.js";

/**
 * @summary Carousels display a series of content slides along a horizontal or vertical axis, one or more at a time.
 *  Users can navigate between slides with controls, pagination, or autoplay.
 *
 * @since 2.2
 * @status experimental
 *
 * @dependency wa-icon
 *
 * @event {{ index: number, slide: WaCarouselItem }} wa-slide-change - Emitted when the active slide changes.
 *
 * @slot - The carousel's main content, one or more `<wa-carousel-item>` elements.
 * @slot next-icon - Optional next icon to use instead of the default. Works best with `<wa-icon>`.
 * @slot previous-icon - Optional previous icon to use instead of the default. Works best with `<wa-icon>`.
 *
 * @csspart base - The carousel's internal wrapper.
 * @csspart scroll-container - The scroll container that wraps the slides.
 * @csspart pagination - The pagination indicators wrapper.
 * @csspart pagination-item - The pagination indicator.
 * @csspart pagination-item-active - Applied when the item is active.
 * @csspart navigation - The navigation wrapper.
 * @csspart navigation-button - The navigation button.
 * @csspart navigation-button-previous - Applied to the previous button.
 * @csspart navigation-button-next - Applied to the next button.
 *
 * @cssproperty [--aspect-ratio=16/9] - The aspect ratio of each slide.
 * @cssproperty --scroll-hint - The amount of padding to apply to the scroll area, allowing adjacent slides to become
 *  partially visible as a scroll hint.
 * @cssproperty [--slide-gap=var(--wa-space-m)] - The space between each slide.
 */
@customElement("wa-carousel")
export default class WaCarousel extends WebAwesomeElement {
  static css = styles;

  /** When set, allows the user to navigate the carousel in the same direction indefinitely. */
  @property({ type: Boolean, reflect: true }) loop = false;

  @property({ type: Number, reflect: true }) slides = 0;
  @property({ type: Number, reflect: true }) currentSlide = 0;

  /** When set, show the carousel's navigation. */
  @property({ type: Boolean, reflect: true }) navigation = false;

  /** When set, show the carousel's pagination indicators. */
  @property({ type: Boolean, reflect: true }) pagination = false;

  /** When set, the slides will scroll automatically when the user is not interacting with them.  */
  @property({ type: Boolean, reflect: true }) autoplay = false;

  /** Specifies the amount of time, in milliseconds, between each automatic scroll.  */
  @property({ type: Number, attribute: "autoplay-interval" }) autoplayInterval =
    3000;

  /** Specifies how many slides should be shown at a given time.  */
  @property({ type: Number, attribute: "slides-per-page" }) slidesPerPage = 1;

  /**
   * Specifies the number of slides the carousel will advance when scrolling, useful when specifying a `slides-per-page`
   * greater than one. It can't be higher than `slides-per-page`.
   */
  @property({ type: Number, attribute: "slides-per-move" }) slidesPerMove = 1;

  /** Specifies the orientation in which the carousel will lay out.  */
  @property() orientation: "horizontal" | "vertical" = "horizontal";

  /** When set, it is possible to scroll through the slides by dragging them with the mouse. */
  @property({ type: Boolean, reflect: true, attribute: "mouse-dragging" })
  mouseDragging = false;

  @query(".slides") scrollContainer: HTMLElement;
  @query(".pagination") paginationContainer: HTMLElement;

  // The index of the active slide
  @state() activeSlide = 0;

  @state() scrolling = false;

  @state() dragging = false;

  private autoplayController = new AutoplayController(this, () => this.next());
  private dragStartPosition: [number, number] = [-1, -1];
  private readonly localize = new LocalizeController(this);
  private mutationObserver: MutationObserver;
  private resizeObserver?: ResizeObserver;
  private pendingSlideChange = false;

  connectedCallback(): void {
    super.connectedCallback();

    // SSR guard: setAttribute is not available during server-side rendering
    if (!isServer) {
      this.setAttribute("role", "region");
      this.setAttribute("aria-label", this.localize.term("carousel"));
    }
  }

  disconnectedCallback(): void {
    super.disconnectedCallback();
    this.mutationObserver?.disconnect();
    this.resizeObserver?.disconnect();
  }

  protected firstUpdated(): void {
    this.initializeSlides();
    this.mutationObserver = new MutationObserver(this.handleSlotChange);
    this.mutationObserver.observe(this, {
      childList: true,
      subtree: true,
    });

    // When the carousel is placed inside a hidden container (e.g. an inactive tab panel),
    // initializeSlides() runs before the element has layout dimensions. The IntersectionObserver
    // inside synchronizeSlides() then reports all slides as non-intersecting and marks them
    // `inert`, making their contents unclickable until the user interacts with the carousel.
    // Re-run synchronizeSlides() once the carousel gains visible dimensions to correct this.
    this.resizeObserver = new ResizeObserver(() => {
      if (
        this.scrollContainer?.clientWidth ||
        this.scrollContainer?.clientHeight
      ) {
        this.synchronizeSlides();
        this.resizeObserver?.disconnect();
        this.resizeObserver = undefined;
      }
    });
    this.resizeObserver.observe(this);
  }

  protected willUpdate(
    changedProperties: PropertyValueMap<WaCarousel> | Map<PropertyKey, unknown>,
  ): void {
    // Ensure the slidesPerMove is never higher than the slidesPerPage
    if (
      changedProperties.has("slidesPerMove") ||
      changedProperties.has("slidesPerPage")
    ) {
      this.slidesPerMove = Math.min(this.slidesPerMove, this.slidesPerPage);
    }
  }

  private getPageCount() {
    const slidesCount = this.getSlides().length;
    const { slidesPerPage, slidesPerMove, loop } = this;

    const pages = loop
      ? slidesCount / slidesPerMove
      : (slidesCount - slidesPerPage) / slidesPerMove + 1;

    return Math.ceil(pages);
  }

  private getCurrentPage() {
    return Math.ceil(this.activeSlide / this.slidesPerMove);
  }

  private canScrollNext(): boolean {
    return this.loop || this.getCurrentPage() < this.getPageCount() - 1;
  }

  private canScrollPrev(): boolean {
    return this.loop || this.getCurrentPage() > 0;
  }

  /** @internal Gets all carousel items. */
  private getSlides({
    excludeClones = true,
  }: { excludeClones?: boolean } = {}) {
    return [...this.children].filter(
      (el: HTMLElement) =>
        this.isCarouselItem(el) &&
        (!excludeClones || !el.hasAttribute("data-clone")),
    ) as WaCarouselItem[];
  }

  private handleClick(event: MouseEvent) {
    if (
      this.dragging &&
      this.dragStartPosition[0] > 0 &&
      this.dragStartPosition[1] > 0
    ) {
      const deltaX = Math.abs(this.dragStartPosition[0] - event.clientX);
      const deltaY = Math.abs(this.dragStartPosition[1] - event.clientY);
      const delta = Math.sqrt(deltaX * deltaX + deltaY * deltaY);

      // Prevents clicks on interactive elements while dragging if the click is within a small range. This prevents
      // accidental drags from interfering with intentional clicks.
      if (delta >= 10) {
        event.preventDefault();
      }
    }
  }

  private handleKeyDown(event: KeyboardEvent) {
    if (
      [
        "ArrowLeft",
        "ArrowRight",
        "ArrowUp",
        "ArrowDown",
        "Home",
        "End",
      ].includes(event.key)
    ) {
      const target = event.target as HTMLElement;
      const isRtl = this.localize.dir() === "rtl";
      const isFocusInPagination =
        target.closest('[part~="pagination-item"]') !== null;
      const isNext =
        event.key === "ArrowDown" ||
        (!isRtl && event.key === "ArrowRight") ||
        (isRtl && event.key === "ArrowLeft");
      const isPrevious =
        event.key === "ArrowUp" ||
        (!isRtl && event.key === "ArrowLeft") ||
        (isRtl && event.key === "ArrowRight");

      event.preventDefault();

      if (isPrevious) {
        this.previous();
      }

      if (isNext) {
        this.next();
      }

      if (event.key === "Home") {
        this.goToSlide(0);
      }

      if (event.key === "End") {
        this.goToSlide(this.getSlides().length - 1);
      }

      if (isFocusInPagination) {
        this.updateComplete.then(() => {
          const activePaginationItem =
            this.shadowRoot?.querySelector<HTMLButtonElement>(
              '[part~="pagination-item-active"]',
            );

          if (activePaginationItem) {
            activePaginationItem.focus();
          }
        });
      }
    }
  }

  private handleMouseDragStart(event: PointerEvent) {
    const canDrag = this.mouseDragging && event.button === 0;
    if (canDrag) {
      event.preventDefault();

      document.addEventListener("pointermove", this.handleMouseDrag, {
        capture: true,
        passive: true,
      });
      document.addEventListener("pointerup", this.handleMouseDragEnd, {
        capture: true,
        once: true,
      });
    }
  }

  private handleMouseDrag = (event: PointerEvent) => {
    if (!this.dragging) {
      // Start dragging if it hasn't yet
      this.scrollContainer.style.setProperty("scroll-snap-type", "none");
      this.dragging = true;
      this.dragStartPosition = [event.clientX, event.clientY];
    }

    this.scrollContainer.scrollBy({
      left: -event.movementX,
      top: -event.movementY,
      behavior: "instant",
    });
  };

  private handleMouseDragEnd = () => {
    const scrollContainer = this.scrollContainer;

    document.removeEventListener("pointermove", this.handleMouseDrag, {
      capture: true,
    });

    // get the current scroll position
    const startLeft = scrollContainer.scrollLeft;
    const startTop = scrollContainer.scrollTop;

    // remove the scroll-snap-type property so that the browser will snap the slide to the correct position
    scrollContainer.style.removeProperty("scroll-snap-type");

    // fix(safari): forcing a style recalculation doesn't seem to immediately update the scroll
    // position in Safari. Setting "overflow" to "hidden" should force this behavior.
    scrollContainer.style.setProperty("overflow", "hidden");

    // get the final scroll position to the slide snapped by the browser
    const finalLeft = scrollContainer.scrollLeft;
    const finalTop = scrollContainer.scrollTop;

    // restore the scroll position to the original one, so that it can be smoothly animated if needed
    scrollContainer.style.removeProperty("overflow");
    scrollContainer.style.setProperty("scroll-snap-type", "none");
    scrollContainer.scrollTo({
      left: startLeft,
      top: startTop,
      behavior: "instant",
    });

    requestAnimationFrame(async () => {
      if (startLeft !== finalLeft || startTop !== finalTop) {
        scrollContainer.scrollTo({
          left: finalLeft,
          top: finalTop,
          behavior: prefersReducedMotion() ? "auto" : "smooth",
        });
        await waitForEvent(scrollContainer, "scrollend");
      }

      scrollContainer.style.removeProperty("scroll-snap-type");

      this.dragging = false;
      this.dragStartPosition = [-1, -1];
      this.handleScrollEnd();
    });
  };

  @eventOptions({ passive: true })
  private handleScroll() {
    this.scrolling = true;
    if (!this.pendingSlideChange) {
      this.synchronizeSlides();
    }
  }

  /** @internal Synchronizes the slides with the IntersectionObserver API. */
  private synchronizeSlides() {
    const io = new IntersectionObserver(
      (entries) => {
        io.disconnect();

        for (const entry of entries) {
          const slide = entry.target;
          slide.toggleAttribute("inert", !entry.isIntersecting);
          slide.classList.toggle("--in-view", entry.isIntersecting);
          slide.setAttribute(
            "aria-hidden",
            entry.isIntersecting ? "false" : "true",
          );
        }

        const firstIntersecting = entries.find((entry) => entry.isIntersecting);

        if (!firstIntersecting) {
          return;
        }

        const slidesWithClones = this.getSlides({ excludeClones: false });
        const slidesCount = this.getSlides().length;

        // Update the current index based on the first visible slide
        const slideIndex = slidesWithClones.indexOf(
          firstIntersecting.target as WaCarouselItem,
        );
        // Normalize the index to ignore clones
        const normalizedIndex = this.loop
          ? slideIndex - this.slidesPerPage
          : slideIndex;

        if (firstIntersecting) {
          // Set the index to the closest "snappable" slide
          this.activeSlide =
            (Math.ceil(normalizedIndex / this.slidesPerMove) *
              this.slidesPerMove +
              slidesCount) %
            slidesCount;

          if (!this.scrolling) {
            if (
              this.loop &&
              firstIntersecting.target.hasAttribute("data-clone")
            ) {
              const clonePosition = Number(
                firstIntersecting.target.getAttribute("data-clone"),
              );
              // Scrolls to the original slide without animating, so the user won't notice that the position has changed
              this.goToSlide(clonePosition, "instant");
            }
          }
        }
      },
      {
        root: this.scrollContainer,
        threshold: 0.6,
      },
    );

    this.getSlides({ excludeClones: false }).forEach((slide) => {
      io.observe(slide);
    });
  }

  private handleScrollEnd() {
    if (!this.scrolling || this.dragging) return;

    this.synchronizeSlides();

    this.scrolling = false;
    this.pendingSlideChange = false;
    this.synchronizeSlides();
  }

  private isCarouselItem(node: Node): node is WaCarouselItem {
    return (
      node instanceof Element &&
      node.tagName.toLowerCase() === "wa-carousel-item"
    );
  }

  private handleSlotChange = (mutations: MutationRecord[]) => {
    const needsInitialization = mutations.some((mutation) =>
      [...mutation.addedNodes, ...mutation.removedNodes].some(
        (el: HTMLElement) =>
          this.isCarouselItem(el) && !el.hasAttribute("data-clone"),
      ),
    );

    // Reinitialize the carousel if a carousel item has been added or removed
    if (needsInitialization) {
      this.initializeSlides();
    }

    this.requestUpdate();
  };

  @watch("loop", { waitUntilFirstUpdate: true })
  @watch("slidesPerPage", { waitUntilFirstUpdate: true })
  initializeSlides() {
    // Removes all the cloned elements from the carousel
    this.getSlides({ excludeClones: false }).forEach((slide, index) => {
      slide.classList.remove("--in-view");
      slide.classList.remove("--is-active");
      slide.setAttribute(
        "aria-label",
        this.localize.term("slideNum", index + 1),
      );

      if (slide.hasAttribute("data-clone")) {
        slide.remove();
      }
    });

    this.updateSlidesSnap();

    if (this.loop) {
      // Creates clones to be placed before and after the original elements to simulate infinite scrolling
      this.createClones();
    }

    // Because the DOM may be changed, restore the scroll position to the active slide
    this.goToSlide(this.activeSlide, "auto");

    this.synchronizeSlides();
  }

  private createClones() {
    const slides = this.getSlides();

    const slidesPerPage = this.slidesPerPage;
    const lastSlides = slides.slice(-slidesPerPage);
    const firstSlides = slides.slice(0, slidesPerPage);

    lastSlides.reverse().forEach((slide, i) => {
      const clone = slide.cloneNode(true) as HTMLElement;
      clone.setAttribute("data-clone", String(slides.length - i - 1));
      this.prepend(clone);
    });

    firstSlides.forEach((slide, i) => {
      const clone = slide.cloneNode(true) as HTMLElement;
      clone.setAttribute("data-clone", String(i));
      this.append(clone);
    });
  }

  @watch("activeSlide")
  handleSlideChange() {
    const slides = this.getSlides();
    slides.forEach((slide, i) => {
      slide.classList.toggle("--is-active", i === this.activeSlide);
    });

    // Do not emit an event on first render
    if (this.hasUpdated) {
      this.dispatchEvent(
        new WaSlideChangeEvent({
          index: this.activeSlide,
          slide: slides[this.activeSlide],
        }),
      );
    }
  }

  @watch("slidesPerMove")
  updateSlidesSnap() {
    const slides = this.getSlides();

    const slidesPerMove = this.slidesPerMove;
    slides.forEach((slide, i) => {
      const shouldSnap = (i + slidesPerMove) % slidesPerMove === 0;
      if (shouldSnap) {
        slide.style.removeProperty("scroll-snap-align");
      } else {
        slide.style.setProperty("scroll-snap-align", "none");
      }
    });
  }

  @watch("autoplay")
  handleAutoplayChange() {
    this.autoplayController.stop();
    if (this.autoplay) {
      this.autoplayController.start(this.autoplayInterval);
    }
  }

  /**
   * Move the carousel backward by `slides-per-move` slides.
   *
   * @param behavior - The behavior used for scrolling.
   */
  previous(behavior: ScrollBehavior = "smooth") {
    this.goToSlide(this.activeSlide - this.slidesPerMove, behavior);
  }

  /**
   * Move the carousel forward by `slides-per-move` slides.
   *
   * @param behavior - The behavior used for scrolling.
   */
  next(behavior: ScrollBehavior = "smooth") {
    this.goToSlide(this.activeSlide + this.slidesPerMove, behavior);
  }

  /**
   * Scrolls the carousel to the slide specified by `index`.
   *
   * @param index - The slide index.
   * @param behavior - The behavior used for scrolling.
   */
  goToSlide(index: number, behavior: ScrollBehavior = "smooth") {
    const { slidesPerPage, loop } = this;

    const slides = this.getSlides();
    const slidesWithClones = this.getSlides({ excludeClones: false });

    // No need to do anything in case there are no items in the carousel
    if (!slides.length) {
      return;
    }

    // Sets the next index without taking into account clones, if any.
    const newActiveSlide = loop
      ? (index + slides.length) % slides.length
      : clamp(index, 0, slides.length - slidesPerPage);
    this.activeSlide = newActiveSlide;

    const isRtl = this.localize.dir() === "rtl";

    // Get the index of the next slide. For looping carousel it adds `slidesPerPage`
    // to normalize the starting index in order to ignore the first nth clones.
    // For RTL it needs to scroll to the last slide of the page.
    const nextSlideIndex = clamp(
      index + (loop ? slidesPerPage : 0) + (isRtl ? slidesPerPage - 1 : 0),
      0,
      slidesWithClones.length - 1,
    );

    const nextSlide = slidesWithClones[nextSlideIndex];

    this.scrollToSlide(nextSlide, prefersReducedMotion() ? "auto" : behavior);
  }

  private scrollToSlide(
    slide: HTMLElement,
    behavior: ScrollBehavior = "smooth",
  ) {
    // Since the geometry doesn't happen until rAF, we don't know if we'll be scrolling or not...
    // It's best to assume that we will and cleanup in the else case below if we didn't need to
    this.pendingSlideChange = true;
    window.requestAnimationFrame(() => {
      // This can happen if goToSlide is called before the scroll container is rendered
      // We will have correctly set the activeSlide in goToSlide which will get picked up when initializeSlides is called.
      if (!this.scrollContainer) {
        return;
      }

      const scrollContainer = this.scrollContainer;
      const scrollContainerRect = scrollContainer.getBoundingClientRect();
      const nextSlideRect = slide.getBoundingClientRect();

      const nextLeft = nextSlideRect.left - scrollContainerRect.left;
      const nextTop = nextSlideRect.top - scrollContainerRect.top;

      if (nextLeft || nextTop) {
        // This is here just in case someone set it back to false
        // between rAF being requested and the callback actually running
        this.pendingSlideChange = true;
        scrollContainer.scrollTo({
          left: nextLeft + scrollContainer.scrollLeft,
          top: nextTop + scrollContainer.scrollTop,
          behavior,
        });
      } else {
        this.pendingSlideChange = false;
      }
    });
  }

  render() {
    const { slidesPerMove, scrolling } = this;

    let pagesCount = 0;
    let currentPage = 0;
    let prevEnabled = false;
    let nextEnabled = false;

    // @TODO: This is a super hacky way to get rid of hydration mismatch errors. The ideal solution is users being able to pass in `pagesCount` and `currentPage` and then on firstUpdated to we update the value for them.
    if (this.hasUpdated) {
      pagesCount = this.getPageCount();
      currentPage = this.getCurrentPage();
      prevEnabled = this.canScrollPrev();
      nextEnabled = this.canScrollNext();
    }

    // We can't rely on `this.matches()` on the server.
    const isRTL = isServer ? this.dir === "rtl" : this.localize.dir() === "rtl";

    return html`
      <div part="base" class="carousel">
        <div
          id="scroll-container"
          part="scroll-container"
          class="${classMap({
            slides: true,
            "slides-horizontal": this.orientation === "horizontal",
            "slides-vertical": this.orientation === "vertical",
            "slides-dragging": this.dragging,
          })}"
          style=${styleMap({ "--slides-per-page": this.slidesPerPage })}
          aria-busy="${scrolling ? "true" : "false"}"
          aria-atomic="true"
          tabindex="0"
          @keydown=${this.handleKeyDown}
          @mousedown="${this.handleMouseDragStart}"
          @scroll="${this.handleScroll}"
          @scrollend=${this.handleScrollEnd}
          @click=${this.handleClick}
        >
          <slot @slotchange=${() => this.requestUpdate()}></slot>
        </div>

        ${
          this.navigation
            ? html`
                <div part="navigation" class="navigation">
                  <button
                    part="navigation-button navigation-button-previous"
                    class="${classMap({
                    "navigation-button": true,
                    "navigation-button-previous": true,
                    "navigation-button-disabled": !prevEnabled,
                  })}"
                    aria-label="${this.localize.term("previousSlide")}"
                    aria-controls="scroll-container"
                    aria-disabled="${prevEnabled ? "false" : "true"}"
                    @click=${prevEnabled ? () => this.previous() : null}
                  >
                    <slot name="previous-icon">
                      <wa-icon
                        library="system"
                        name="${isRTL ? "chevron-right" : "chevron-left"}"
                      ></wa-icon>
                    </slot>
                  </button>

                  <button
                    part="navigation-button navigation-button-next"
                    class=${classMap({
                    "navigation-button": true,
                    "navigation-button-next": true,
                    "navigation-button-disabled": !nextEnabled,
                  })}
                    aria-label="${this.localize.term("nextSlide")}"
                    aria-controls="scroll-container"
                    aria-disabled="${nextEnabled ? "false" : "true"}"
                    @click=${nextEnabled ? () => this.next() : null}
                  >
                    <slot name="next-icon">
                      <wa-icon
                        library="system"
                        name="${isRTL ? "chevron-left" : "chevron-right"}"
                      ></wa-icon>
                    </slot>
                  </button>
                </div>
              `
            : ""
        }
        ${
          this.pagination
            ? html`
                <div
                  part="pagination"
                  role="tablist"
                  class="pagination"
                  aria-controls="scroll-container"
                >
                  ${map(range(pagesCount), (index) => {
                  const isActive = index === currentPage;
                  return html`
                    <button
                      part="pagination-item ${isActive ? "pagination-item-active" : ""}"
                      class="${classMap({
                        "pagination-item": true,
                        "pagination-item-active": isActive,
                      })}"
                      role="tab"
                      aria-selected="${isActive ? "true" : "false"}"
                      aria-label="${this.localize.term("goToSlide", index + 1, pagesCount)}"
                      tabindex=${isActive ? "0" : "-1"}
                      @click=${() => this.goToSlide(index * slidesPerMove)}
                      @keydown=${this.handleKeyDown}
                    ></button>
                  `;
                })}
                </div>
              `
            : html``
        }
      </div>
    `;
  }
}

declare global {
  interface HTMLElementTagNameMap {
    "wa-carousel": WaCarousel;
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
@@ -626,65 +626,62 @@
         >
           <slot @slotchange=${() => this.requestUpdate()}></slot>
         </div>
 
-        ${
-          this.navigation
-            ? html`
-                <div part="navigation" class="navigation">
-                  <button
-                    part="navigation-button navigation-button-previous"
-                    class="${classMap({
+        ${this.navigation
+          ? html`
+              <div part="navigation" class="navigation">
+                <button
+                  part="navigation-button navigation-button-previous"
+                  class="${classMap({
                     "navigation-button": true,
                     "navigation-button-previous": true,
                     "navigation-button-disabled": !prevEnabled,
                   })}"
-                    aria-label="${this.localize.term("previousSlide")}"
-                    aria-controls="scroll-container"
-                    aria-disabled="${prevEnabled ? "false" : "true"}"
-                    @click=${prevEnabled ? () => this.previous() : null}
-                  >
-                    <slot name="previous-icon">
-                      <wa-icon
-                        library="system"
-                        name="${isRTL ? "chevron-right" : "chevron-left"}"
-                      ></wa-icon>
-                    </slot>
-                  </button>
+                  aria-label="${this.localize.term("previousSlide")}"
+                  aria-controls="scroll-container"
+                  aria-disabled="${prevEnabled ? "false" : "true"}"
+                  @click=${prevEnabled ? () => this.previous() : null}
+                >
+                  <slot name="previous-icon">
+                    <wa-icon
+                      library="system"
+                      name="${isRTL ? "chevron-right" : "chevron-left"}"
+                    ></wa-icon>
+                  </slot>
+                </button>
 
-                  <button
-                    part="navigation-button navigation-button-next"
-                    class=${classMap({
+                <button
+                  part="navigation-button navigation-button-next"
+                  class=${classMap({
                     "navigation-button": true,
                     "navigation-button-next": true,
                     "navigation-button-disabled": !nextEnabled,
                   })}
-                    aria-label="${this.localize.term("nextSlide")}"
-                    aria-controls="scroll-container"
-                    aria-disabled="${nextEnabled ? "false" : "true"}"
-                    @click=${nextEnabled ? () => this.next() : null}
-                  >
-                    <slot name="next-icon">
-                      <wa-icon
-                        library="system"
-                        name="${isRTL ? "chevron-left" : "chevron-right"}"
-                      ></wa-icon>
-                    </slot>
-                  </button>
-                </div>
-              `
-            : ""
-        }
-        ${
-          this.pagination
-            ? html`
-                <div
-                  part="pagination"
-                  role="tablist"
-                  class="pagination"
+                  aria-label="${this.localize.term("nextSlide")}"
                   aria-controls="scroll-container"
+                  aria-disabled="${nextEnabled ? "false" : "true"}"
+                  @click=${nextEnabled ? () => this.next() : null}
                 >
-                  ${map(range(pagesCount), (index) => {
+                  <slot name="next-icon">
+                    <wa-icon
+                      library="system"
+                      name="${isRTL ? "chevron-left" : "chevron-right"}"
+                    ></wa-icon>
+                  </slot>
+                </button>
+              </div>
+            `
+          : ""}
+        ${this.pagination
+          ? html`
+              <div
+                part="pagination"
+                role="tablist"
+                class="pagination"
+                aria-controls="scroll-container"
+              >
+                ${map(range(pagesCount), (index) => {
                   const isActive = index === currentPage;
                   return html`
                     <button
                       part="pagination-item ${isActive ? "pagination-item-active" : ""}"
@@ -700,12 +697,11 @@
                       @keydown=${this.handleKeyDown}
                     ></button>
                   `;
                 })}
-                </div>
-              `
-            : html``
-        }
+              </div>
+            `
+          : html``}
       </div>
     `;
   }
 }

`````

### Actual (oxfmt)

`````ts
import "../../internal/scrollend-polyfill.js";

import type { PropertyValueMap } from "lit";
import { html, isServer } from "lit";
import { customElement, eventOptions, property, query, state } from "lit/decorators.js";
import { classMap } from "lit/directives/class-map.js";
import { map } from "lit/directives/map.js";
import { range } from "lit/directives/range.js";
import { styleMap } from "lit/directives/style-map.js";
import { WaSlideChangeEvent } from "../../events/slide-change.js";
import { prefersReducedMotion } from "../../internal/animate.js";
import { waitForEvent } from "../../internal/event.js";
import { clamp } from "../../internal/math.js";
import { watch } from "../../internal/watch.js";
import WebAwesomeElement from "../../internal/webawesome-element.js";
import { LocalizeController } from "../../utilities/localize.js";
import type WaCarouselItem from "../carousel-item/carousel-item.js";
import "../icon/icon.js";
import { AutoplayController } from "./autoplay-controller.js";
import styles from "./carousel.styles.js";

/**
 * @summary Carousels display a series of content slides along a horizontal or vertical axis, one or more at a time.
 *  Users can navigate between slides with controls, pagination, or autoplay.
 *
 * @since 2.2
 * @status experimental
 *
 * @dependency wa-icon
 *
 * @event {{ index: number, slide: WaCarouselItem }} wa-slide-change - Emitted when the active slide changes.
 *
 * @slot - The carousel's main content, one or more `<wa-carousel-item>` elements.
 * @slot next-icon - Optional next icon to use instead of the default. Works best with `<wa-icon>`.
 * @slot previous-icon - Optional previous icon to use instead of the default. Works best with `<wa-icon>`.
 *
 * @csspart base - The carousel's internal wrapper.
 * @csspart scroll-container - The scroll container that wraps the slides.
 * @csspart pagination - The pagination indicators wrapper.
 * @csspart pagination-item - The pagination indicator.
 * @csspart pagination-item-active - Applied when the item is active.
 * @csspart navigation - The navigation wrapper.
 * @csspart navigation-button - The navigation button.
 * @csspart navigation-button-previous - Applied to the previous button.
 * @csspart navigation-button-next - Applied to the next button.
 *
 * @cssproperty [--aspect-ratio=16/9] - The aspect ratio of each slide.
 * @cssproperty --scroll-hint - The amount of padding to apply to the scroll area, allowing adjacent slides to become
 *  partially visible as a scroll hint.
 * @cssproperty [--slide-gap=var(--wa-space-m)] - The space between each slide.
 */
@customElement("wa-carousel")
export default class WaCarousel extends WebAwesomeElement {
  static css = styles;

  /** When set, allows the user to navigate the carousel in the same direction indefinitely. */
  @property({ type: Boolean, reflect: true }) loop = false;

  @property({ type: Number, reflect: true }) slides = 0;
  @property({ type: Number, reflect: true }) currentSlide = 0;

  /** When set, show the carousel's navigation. */
  @property({ type: Boolean, reflect: true }) navigation = false;

  /** When set, show the carousel's pagination indicators. */
  @property({ type: Boolean, reflect: true }) pagination = false;

  /** When set, the slides will scroll automatically when the user is not interacting with them.  */
  @property({ type: Boolean, reflect: true }) autoplay = false;

  /** Specifies the amount of time, in milliseconds, between each automatic scroll.  */
  @property({ type: Number, attribute: "autoplay-interval" }) autoplayInterval = 3000;

  /** Specifies how many slides should be shown at a given time.  */
  @property({ type: Number, attribute: "slides-per-page" }) slidesPerPage = 1;

  /**
   * Specifies the number of slides the carousel will advance when scrolling, useful when specifying a `slides-per-page`
   * greater than one. It can't be higher than `slides-per-page`.
   */
  @property({ type: Number, attribute: "slides-per-move" }) slidesPerMove = 1;

  /** Specifies the orientation in which the carousel will lay out.  */
  @property() orientation: "horizontal" | "vertical" = "horizontal";

  /** When set, it is possible to scroll through the slides by dragging them with the mouse. */
  @property({ type: Boolean, reflect: true, attribute: "mouse-dragging" }) mouseDragging = false;

  @query(".slides") scrollContainer: HTMLElement;
  @query(".pagination") paginationContainer: HTMLElement;

  // The index of the active slide
  @state() activeSlide = 0;

  @state() scrolling = false;

  @state() dragging = false;

  private autoplayController = new AutoplayController(this, () => this.next());
  private dragStartPosition: [number, number] = [-1, -1];
  private readonly localize = new LocalizeController(this);
  private mutationObserver: MutationObserver;
  private resizeObserver?: ResizeObserver;
  private pendingSlideChange = false;

  connectedCallback(): void {
    super.connectedCallback();

    // SSR guard: setAttribute is not available during server-side rendering
    if (!isServer) {
      this.setAttribute("role", "region");
      this.setAttribute("aria-label", this.localize.term("carousel"));
    }
  }

  disconnectedCallback(): void {
    super.disconnectedCallback();
    this.mutationObserver?.disconnect();
    this.resizeObserver?.disconnect();
  }

  protected firstUpdated(): void {
    this.initializeSlides();
    this.mutationObserver = new MutationObserver(this.handleSlotChange);
    this.mutationObserver.observe(this, {
      childList: true,
      subtree: true,
    });

    // When the carousel is placed inside a hidden container (e.g. an inactive tab panel),
    // initializeSlides() runs before the element has layout dimensions. The IntersectionObserver
    // inside synchronizeSlides() then reports all slides as non-intersecting and marks them
    // `inert`, making their contents unclickable until the user interacts with the carousel.
    // Re-run synchronizeSlides() once the carousel gains visible dimensions to correct this.
    this.resizeObserver = new ResizeObserver(() => {
      if (this.scrollContainer?.clientWidth || this.scrollContainer?.clientHeight) {
        this.synchronizeSlides();
        this.resizeObserver?.disconnect();
        this.resizeObserver = undefined;
      }
    });
    this.resizeObserver.observe(this);
  }

  protected willUpdate(
    changedProperties: PropertyValueMap<WaCarousel> | Map<PropertyKey, unknown>,
  ): void {
    // Ensure the slidesPerMove is never higher than the slidesPerPage
    if (changedProperties.has("slidesPerMove") || changedProperties.has("slidesPerPage")) {
      this.slidesPerMove = Math.min(this.slidesPerMove, this.slidesPerPage);
    }
  }

  private getPageCount() {
    const slidesCount = this.getSlides().length;
    const { slidesPerPage, slidesPerMove, loop } = this;

    const pages = loop
      ? slidesCount / slidesPerMove
      : (slidesCount - slidesPerPage) / slidesPerMove + 1;

    return Math.ceil(pages);
  }

  private getCurrentPage() {
    return Math.ceil(this.activeSlide / this.slidesPerMove);
  }

  private canScrollNext(): boolean {
    return this.loop || this.getCurrentPage() < this.getPageCount() - 1;
  }

  private canScrollPrev(): boolean {
    return this.loop || this.getCurrentPage() > 0;
  }

  /** @internal Gets all carousel items. */
  private getSlides({ excludeClones = true }: { excludeClones?: boolean } = {}) {
    return [...this.children].filter(
      (el: HTMLElement) =>
        this.isCarouselItem(el) && (!excludeClones || !el.hasAttribute("data-clone")),
    ) as WaCarouselItem[];
  }

  private handleClick(event: MouseEvent) {
    if (this.dragging && this.dragStartPosition[0] > 0 && this.dragStartPosition[1] > 0) {
      const deltaX = Math.abs(this.dragStartPosition[0] - event.clientX);
      const deltaY = Math.abs(this.dragStartPosition[1] - event.clientY);
      const delta = Math.sqrt(deltaX * deltaX + deltaY * deltaY);

      // Prevents clicks on interactive elements while dragging if the click is within a small range. This prevents
      // accidental drags from interfering with intentional clicks.
      if (delta >= 10) {
        event.preventDefault();
      }
    }
  }

  private handleKeyDown(event: KeyboardEvent) {
    if (["ArrowLeft", "ArrowRight", "ArrowUp", "ArrowDown", "Home", "End"].includes(event.key)) {
      const target = event.target as HTMLElement;
      const isRtl = this.localize.dir() === "rtl";
      const isFocusInPagination = target.closest('[part~="pagination-item"]') !== null;
      const isNext =
        event.key === "ArrowDown" ||
        (!isRtl && event.key === "ArrowRight") ||
        (isRtl && event.key === "ArrowLeft");
      const isPrevious =
        event.key === "ArrowUp" ||
        (!isRtl && event.key === "ArrowLeft") ||
        (isRtl && event.key === "ArrowRight");

      event.preventDefault();

      if (isPrevious) {
        this.previous();
      }

      if (isNext) {
        this.next();
      }

      if (event.key === "Home") {
        this.goToSlide(0);
      }

      if (event.key === "End") {
        this.goToSlide(this.getSlides().length - 1);
      }

      if (isFocusInPagination) {
        this.updateComplete.then(() => {
          const activePaginationItem = this.shadowRoot?.querySelector<HTMLButtonElement>(
            '[part~="pagination-item-active"]',
          );

          if (activePaginationItem) {
            activePaginationItem.focus();
          }
        });
      }
    }
  }

  private handleMouseDragStart(event: PointerEvent) {
    const canDrag = this.mouseDragging && event.button === 0;
    if (canDrag) {
      event.preventDefault();

      document.addEventListener("pointermove", this.handleMouseDrag, {
        capture: true,
        passive: true,
      });
      document.addEventListener("pointerup", this.handleMouseDragEnd, {
        capture: true,
        once: true,
      });
    }
  }

  private handleMouseDrag = (event: PointerEvent) => {
    if (!this.dragging) {
      // Start dragging if it hasn't yet
      this.scrollContainer.style.setProperty("scroll-snap-type", "none");
      this.dragging = true;
      this.dragStartPosition = [event.clientX, event.clientY];
    }

    this.scrollContainer.scrollBy({
      left: -event.movementX,
      top: -event.movementY,
      behavior: "instant",
    });
  };

  private handleMouseDragEnd = () => {
    const scrollContainer = this.scrollContainer;

    document.removeEventListener("pointermove", this.handleMouseDrag, { capture: true });

    // get the current scroll position
    const startLeft = scrollContainer.scrollLeft;
    const startTop = scrollContainer.scrollTop;

    // remove the scroll-snap-type property so that the browser will snap the slide to the correct position
    scrollContainer.style.removeProperty("scroll-snap-type");

    // fix(safari): forcing a style recalculation doesn't seem to immediately update the scroll
    // position in Safari. Setting "overflow" to "hidden" should force this behavior.
    scrollContainer.style.setProperty("overflow", "hidden");

    // get the final scroll position to the slide snapped by the browser
    const finalLeft = scrollContainer.scrollLeft;
    const finalTop = scrollContainer.scrollTop;

    // restore the scroll position to the original one, so that it can be smoothly animated if needed
    scrollContainer.style.removeProperty("overflow");
    scrollContainer.style.setProperty("scroll-snap-type", "none");
    scrollContainer.scrollTo({ left: startLeft, top: startTop, behavior: "instant" });

    requestAnimationFrame(async () => {
      if (startLeft !== finalLeft || startTop !== finalTop) {
        scrollContainer.scrollTo({
          left: finalLeft,
          top: finalTop,
          behavior: prefersReducedMotion() ? "auto" : "smooth",
        });
        await waitForEvent(scrollContainer, "scrollend");
      }

      scrollContainer.style.removeProperty("scroll-snap-type");

      this.dragging = false;
      this.dragStartPosition = [-1, -1];
      this.handleScrollEnd();
    });
  };

  @eventOptions({ passive: true })
  private handleScroll() {
    this.scrolling = true;
    if (!this.pendingSlideChange) {
      this.synchronizeSlides();
    }
  }

  /** @internal Synchronizes the slides with the IntersectionObserver API. */
  private synchronizeSlides() {
    const io = new IntersectionObserver(
      (entries) => {
        io.disconnect();

        for (const entry of entries) {
          const slide = entry.target;
          slide.toggleAttribute("inert", !entry.isIntersecting);
          slide.classList.toggle("--in-view", entry.isIntersecting);
          slide.setAttribute("aria-hidden", entry.isIntersecting ? "false" : "true");
        }

        const firstIntersecting = entries.find((entry) => entry.isIntersecting);

        if (!firstIntersecting) {
          return;
        }

        const slidesWithClones = this.getSlides({ excludeClones: false });
        const slidesCount = this.getSlides().length;

        // Update the current index based on the first visible slide
        const slideIndex = slidesWithClones.indexOf(firstIntersecting.target as WaCarouselItem);
        // Normalize the index to ignore clones
        const normalizedIndex = this.loop ? slideIndex - this.slidesPerPage : slideIndex;

        if (firstIntersecting) {
          // Set the index to the closest "snappable" slide
          this.activeSlide =
            (Math.ceil(normalizedIndex / this.slidesPerMove) * this.slidesPerMove + slidesCount) %
            slidesCount;

          if (!this.scrolling) {
            if (this.loop && firstIntersecting.target.hasAttribute("data-clone")) {
              const clonePosition = Number(firstIntersecting.target.getAttribute("data-clone"));
              // Scrolls to the original slide without animating, so the user won't notice that the position has changed
              this.goToSlide(clonePosition, "instant");
            }
          }
        }
      },
      {
        root: this.scrollContainer,
        threshold: 0.6,
      },
    );

    this.getSlides({ excludeClones: false }).forEach((slide) => {
      io.observe(slide);
    });
  }

  private handleScrollEnd() {
    if (!this.scrolling || this.dragging) return;

    this.synchronizeSlides();

    this.scrolling = false;
    this.pendingSlideChange = false;
    this.synchronizeSlides();
  }

  private isCarouselItem(node: Node): node is WaCarouselItem {
    return node instanceof Element && node.tagName.toLowerCase() === "wa-carousel-item";
  }

  private handleSlotChange = (mutations: MutationRecord[]) => {
    const needsInitialization = mutations.some((mutation) =>
      [...mutation.addedNodes, ...mutation.removedNodes].some(
        (el: HTMLElement) => this.isCarouselItem(el) && !el.hasAttribute("data-clone"),
      ),
    );

    // Reinitialize the carousel if a carousel item has been added or removed
    if (needsInitialization) {
      this.initializeSlides();
    }

    this.requestUpdate();
  };

  @watch("loop", { waitUntilFirstUpdate: true })
  @watch("slidesPerPage", { waitUntilFirstUpdate: true })
  initializeSlides() {
    // Removes all the cloned elements from the carousel
    this.getSlides({ excludeClones: false }).forEach((slide, index) => {
      slide.classList.remove("--in-view");
      slide.classList.remove("--is-active");
      slide.setAttribute("aria-label", this.localize.term("slideNum", index + 1));

      if (slide.hasAttribute("data-clone")) {
        slide.remove();
      }
    });

    this.updateSlidesSnap();

    if (this.loop) {
      // Creates clones to be placed before and after the original elements to simulate infinite scrolling
      this.createClones();
    }

    // Because the DOM may be changed, restore the scroll position to the active slide
    this.goToSlide(this.activeSlide, "auto");

    this.synchronizeSlides();
  }

  private createClones() {
    const slides = this.getSlides();

    const slidesPerPage = this.slidesPerPage;
    const lastSlides = slides.slice(-slidesPerPage);
    const firstSlides = slides.slice(0, slidesPerPage);

    lastSlides.reverse().forEach((slide, i) => {
      const clone = slide.cloneNode(true) as HTMLElement;
      clone.setAttribute("data-clone", String(slides.length - i - 1));
      this.prepend(clone);
    });

    firstSlides.forEach((slide, i) => {
      const clone = slide.cloneNode(true) as HTMLElement;
      clone.setAttribute("data-clone", String(i));
      this.append(clone);
    });
  }

  @watch("activeSlide")
  handleSlideChange() {
    const slides = this.getSlides();
    slides.forEach((slide, i) => {
      slide.classList.toggle("--is-active", i === this.activeSlide);
    });

    // Do not emit an event on first render
    if (this.hasUpdated) {
      this.dispatchEvent(
        new WaSlideChangeEvent({
          index: this.activeSlide,
          slide: slides[this.activeSlide],
        }),
      );
    }
  }

  @watch("slidesPerMove")
  updateSlidesSnap() {
    const slides = this.getSlides();

    const slidesPerMove = this.slidesPerMove;
    slides.forEach((slide, i) => {
      const shouldSnap = (i + slidesPerMove) % slidesPerMove === 0;
      if (shouldSnap) {
        slide.style.removeProperty("scroll-snap-align");
      } else {
        slide.style.setProperty("scroll-snap-align", "none");
      }
    });
  }

  @watch("autoplay")
  handleAutoplayChange() {
    this.autoplayController.stop();
    if (this.autoplay) {
      this.autoplayController.start(this.autoplayInterval);
    }
  }

  /**
   * Move the carousel backward by `slides-per-move` slides.
   *
   * @param behavior - The behavior used for scrolling.
   */
  previous(behavior: ScrollBehavior = "smooth") {
    this.goToSlide(this.activeSlide - this.slidesPerMove, behavior);
  }

  /**
   * Move the carousel forward by `slides-per-move` slides.
   *
   * @param behavior - The behavior used for scrolling.
   */
  next(behavior: ScrollBehavior = "smooth") {
    this.goToSlide(this.activeSlide + this.slidesPerMove, behavior);
  }

  /**
   * Scrolls the carousel to the slide specified by `index`.
   *
   * @param index - The slide index.
   * @param behavior - The behavior used for scrolling.
   */
  goToSlide(index: number, behavior: ScrollBehavior = "smooth") {
    const { slidesPerPage, loop } = this;

    const slides = this.getSlides();
    const slidesWithClones = this.getSlides({ excludeClones: false });

    // No need to do anything in case there are no items in the carousel
    if (!slides.length) {
      return;
    }

    // Sets the next index without taking into account clones, if any.
    const newActiveSlide = loop
      ? (index + slides.length) % slides.length
      : clamp(index, 0, slides.length - slidesPerPage);
    this.activeSlide = newActiveSlide;

    const isRtl = this.localize.dir() === "rtl";

    // Get the index of the next slide. For looping carousel it adds `slidesPerPage`
    // to normalize the starting index in order to ignore the first nth clones.
    // For RTL it needs to scroll to the last slide of the page.
    const nextSlideIndex = clamp(
      index + (loop ? slidesPerPage : 0) + (isRtl ? slidesPerPage - 1 : 0),
      0,
      slidesWithClones.length - 1,
    );

    const nextSlide = slidesWithClones[nextSlideIndex];

    this.scrollToSlide(nextSlide, prefersReducedMotion() ? "auto" : behavior);
  }

  private scrollToSlide(slide: HTMLElement, behavior: ScrollBehavior = "smooth") {
    // Since the geometry doesn't happen until rAF, we don't know if we'll be scrolling or not...
    // It's best to assume that we will and cleanup in the else case below if we didn't need to
    this.pendingSlideChange = true;
    window.requestAnimationFrame(() => {
      // This can happen if goToSlide is called before the scroll container is rendered
      // We will have correctly set the activeSlide in goToSlide which will get picked up when initializeSlides is called.
      if (!this.scrollContainer) {
        return;
      }

      const scrollContainer = this.scrollContainer;
      const scrollContainerRect = scrollContainer.getBoundingClientRect();
      const nextSlideRect = slide.getBoundingClientRect();

      const nextLeft = nextSlideRect.left - scrollContainerRect.left;
      const nextTop = nextSlideRect.top - scrollContainerRect.top;

      if (nextLeft || nextTop) {
        // This is here just in case someone set it back to false
        // between rAF being requested and the callback actually running
        this.pendingSlideChange = true;
        scrollContainer.scrollTo({
          left: nextLeft + scrollContainer.scrollLeft,
          top: nextTop + scrollContainer.scrollTop,
          behavior,
        });
      } else {
        this.pendingSlideChange = false;
      }
    });
  }

  render() {
    const { slidesPerMove, scrolling } = this;

    let pagesCount = 0;
    let currentPage = 0;
    let prevEnabled = false;
    let nextEnabled = false;

    // @TODO: This is a super hacky way to get rid of hydration mismatch errors. The ideal solution is users being able to pass in `pagesCount` and `currentPage` and then on firstUpdated to we update the value for them.
    if (this.hasUpdated) {
      pagesCount = this.getPageCount();
      currentPage = this.getCurrentPage();
      prevEnabled = this.canScrollPrev();
      nextEnabled = this.canScrollNext();
    }

    // We can't rely on `this.matches()` on the server.
    const isRTL = isServer ? this.dir === "rtl" : this.localize.dir() === "rtl";

    return html`
      <div part="base" class="carousel">
        <div
          id="scroll-container"
          part="scroll-container"
          class="${classMap({
            slides: true,
            "slides-horizontal": this.orientation === "horizontal",
            "slides-vertical": this.orientation === "vertical",
            "slides-dragging": this.dragging,
          })}"
          style=${styleMap({ "--slides-per-page": this.slidesPerPage })}
          aria-busy="${scrolling ? "true" : "false"}"
          aria-atomic="true"
          tabindex="0"
          @keydown=${this.handleKeyDown}
          @mousedown="${this.handleMouseDragStart}"
          @scroll="${this.handleScroll}"
          @scrollend=${this.handleScrollEnd}
          @click=${this.handleClick}
        >
          <slot @slotchange=${() => this.requestUpdate()}></slot>
        </div>

        ${this.navigation
          ? html`
              <div part="navigation" class="navigation">
                <button
                  part="navigation-button navigation-button-previous"
                  class="${classMap({
                    "navigation-button": true,
                    "navigation-button-previous": true,
                    "navigation-button-disabled": !prevEnabled,
                  })}"
                  aria-label="${this.localize.term("previousSlide")}"
                  aria-controls="scroll-container"
                  aria-disabled="${prevEnabled ? "false" : "true"}"
                  @click=${prevEnabled ? () => this.previous() : null}
                >
                  <slot name="previous-icon">
                    <wa-icon
                      library="system"
                      name="${isRTL ? "chevron-right" : "chevron-left"}"
                    ></wa-icon>
                  </slot>
                </button>

                <button
                  part="navigation-button navigation-button-next"
                  class=${classMap({
                    "navigation-button": true,
                    "navigation-button-next": true,
                    "navigation-button-disabled": !nextEnabled,
                  })}
                  aria-label="${this.localize.term("nextSlide")}"
                  aria-controls="scroll-container"
                  aria-disabled="${nextEnabled ? "false" : "true"}"
                  @click=${nextEnabled ? () => this.next() : null}
                >
                  <slot name="next-icon">
                    <wa-icon
                      library="system"
                      name="${isRTL ? "chevron-left" : "chevron-right"}"
                    ></wa-icon>
                  </slot>
                </button>
              </div>
            `
          : ""}
        ${this.pagination
          ? html`
              <div
                part="pagination"
                role="tablist"
                class="pagination"
                aria-controls="scroll-container"
              >
                ${map(range(pagesCount), (index) => {
                  const isActive = index === currentPage;
                  return html`
                    <button
                      part="pagination-item ${isActive ? "pagination-item-active" : ""}"
                      class="${classMap({
                        "pagination-item": true,
                        "pagination-item-active": isActive,
                      })}"
                      role="tab"
                      aria-selected="${isActive ? "true" : "false"}"
                      aria-label="${this.localize.term("goToSlide", index + 1, pagesCount)}"
                      tabindex=${isActive ? "0" : "-1"}
                      @click=${() => this.goToSlide(index * slidesPerMove)}
                      @keydown=${this.handleKeyDown}
                    ></button>
                  `;
                })}
              </div>
            `
          : html``}
      </div>
    `;
  }
}

declare global {
  interface HTMLElementTagNameMap {
    "wa-carousel": WaCarousel;
  }
}

`````

### Expected (prettier)

`````ts
import "../../internal/scrollend-polyfill.js";

import type { PropertyValueMap } from "lit";
import { html, isServer } from "lit";
import { customElement, eventOptions, property, query, state } from "lit/decorators.js";
import { classMap } from "lit/directives/class-map.js";
import { map } from "lit/directives/map.js";
import { range } from "lit/directives/range.js";
import { styleMap } from "lit/directives/style-map.js";
import { WaSlideChangeEvent } from "../../events/slide-change.js";
import { prefersReducedMotion } from "../../internal/animate.js";
import { waitForEvent } from "../../internal/event.js";
import { clamp } from "../../internal/math.js";
import { watch } from "../../internal/watch.js";
import WebAwesomeElement from "../../internal/webawesome-element.js";
import { LocalizeController } from "../../utilities/localize.js";
import type WaCarouselItem from "../carousel-item/carousel-item.js";
import "../icon/icon.js";
import { AutoplayController } from "./autoplay-controller.js";
import styles from "./carousel.styles.js";

/**
 * @summary Carousels display a series of content slides along a horizontal or vertical axis, one or more at a time.
 *  Users can navigate between slides with controls, pagination, or autoplay.
 *
 * @since 2.2
 * @status experimental
 *
 * @dependency wa-icon
 *
 * @event {{ index: number, slide: WaCarouselItem }} wa-slide-change - Emitted when the active slide changes.
 *
 * @slot - The carousel's main content, one or more `<wa-carousel-item>` elements.
 * @slot next-icon - Optional next icon to use instead of the default. Works best with `<wa-icon>`.
 * @slot previous-icon - Optional previous icon to use instead of the default. Works best with `<wa-icon>`.
 *
 * @csspart base - The carousel's internal wrapper.
 * @csspart scroll-container - The scroll container that wraps the slides.
 * @csspart pagination - The pagination indicators wrapper.
 * @csspart pagination-item - The pagination indicator.
 * @csspart pagination-item-active - Applied when the item is active.
 * @csspart navigation - The navigation wrapper.
 * @csspart navigation-button - The navigation button.
 * @csspart navigation-button-previous - Applied to the previous button.
 * @csspart navigation-button-next - Applied to the next button.
 *
 * @cssproperty [--aspect-ratio=16/9] - The aspect ratio of each slide.
 * @cssproperty --scroll-hint - The amount of padding to apply to the scroll area, allowing adjacent slides to become
 *  partially visible as a scroll hint.
 * @cssproperty [--slide-gap=var(--wa-space-m)] - The space between each slide.
 */
@customElement("wa-carousel")
export default class WaCarousel extends WebAwesomeElement {
  static css = styles;

  /** When set, allows the user to navigate the carousel in the same direction indefinitely. */
  @property({ type: Boolean, reflect: true }) loop = false;

  @property({ type: Number, reflect: true }) slides = 0;
  @property({ type: Number, reflect: true }) currentSlide = 0;

  /** When set, show the carousel's navigation. */
  @property({ type: Boolean, reflect: true }) navigation = false;

  /** When set, show the carousel's pagination indicators. */
  @property({ type: Boolean, reflect: true }) pagination = false;

  /** When set, the slides will scroll automatically when the user is not interacting with them.  */
  @property({ type: Boolean, reflect: true }) autoplay = false;

  /** Specifies the amount of time, in milliseconds, between each automatic scroll.  */
  @property({ type: Number, attribute: "autoplay-interval" }) autoplayInterval = 3000;

  /** Specifies how many slides should be shown at a given time.  */
  @property({ type: Number, attribute: "slides-per-page" }) slidesPerPage = 1;

  /**
   * Specifies the number of slides the carousel will advance when scrolling, useful when specifying a `slides-per-page`
   * greater than one. It can't be higher than `slides-per-page`.
   */
  @property({ type: Number, attribute: "slides-per-move" }) slidesPerMove = 1;

  /** Specifies the orientation in which the carousel will lay out.  */
  @property() orientation: "horizontal" | "vertical" = "horizontal";

  /** When set, it is possible to scroll through the slides by dragging them with the mouse. */
  @property({ type: Boolean, reflect: true, attribute: "mouse-dragging" }) mouseDragging = false;

  @query(".slides") scrollContainer: HTMLElement;
  @query(".pagination") paginationContainer: HTMLElement;

  // The index of the active slide
  @state() activeSlide = 0;

  @state() scrolling = false;

  @state() dragging = false;

  private autoplayController = new AutoplayController(this, () => this.next());
  private dragStartPosition: [number, number] = [-1, -1];
  private readonly localize = new LocalizeController(this);
  private mutationObserver: MutationObserver;
  private resizeObserver?: ResizeObserver;
  private pendingSlideChange = false;

  connectedCallback(): void {
    super.connectedCallback();

    // SSR guard: setAttribute is not available during server-side rendering
    if (!isServer) {
      this.setAttribute("role", "region");
      this.setAttribute("aria-label", this.localize.term("carousel"));
    }
  }

  disconnectedCallback(): void {
    super.disconnectedCallback();
    this.mutationObserver?.disconnect();
    this.resizeObserver?.disconnect();
  }

  protected firstUpdated(): void {
    this.initializeSlides();
    this.mutationObserver = new MutationObserver(this.handleSlotChange);
    this.mutationObserver.observe(this, {
      childList: true,
      subtree: true,
    });

    // When the carousel is placed inside a hidden container (e.g. an inactive tab panel),
    // initializeSlides() runs before the element has layout dimensions. The IntersectionObserver
    // inside synchronizeSlides() then reports all slides as non-intersecting and marks them
    // `inert`, making their contents unclickable until the user interacts with the carousel.
    // Re-run synchronizeSlides() once the carousel gains visible dimensions to correct this.
    this.resizeObserver = new ResizeObserver(() => {
      if (this.scrollContainer?.clientWidth || this.scrollContainer?.clientHeight) {
        this.synchronizeSlides();
        this.resizeObserver?.disconnect();
        this.resizeObserver = undefined;
      }
    });
    this.resizeObserver.observe(this);
  }

  protected willUpdate(
    changedProperties: PropertyValueMap<WaCarousel> | Map<PropertyKey, unknown>,
  ): void {
    // Ensure the slidesPerMove is never higher than the slidesPerPage
    if (changedProperties.has("slidesPerMove") || changedProperties.has("slidesPerPage")) {
      this.slidesPerMove = Math.min(this.slidesPerMove, this.slidesPerPage);
    }
  }

  private getPageCount() {
    const slidesCount = this.getSlides().length;
    const { slidesPerPage, slidesPerMove, loop } = this;

    const pages = loop
      ? slidesCount / slidesPerMove
      : (slidesCount - slidesPerPage) / slidesPerMove + 1;

    return Math.ceil(pages);
  }

  private getCurrentPage() {
    return Math.ceil(this.activeSlide / this.slidesPerMove);
  }

  private canScrollNext(): boolean {
    return this.loop || this.getCurrentPage() < this.getPageCount() - 1;
  }

  private canScrollPrev(): boolean {
    return this.loop || this.getCurrentPage() > 0;
  }

  /** @internal Gets all carousel items. */
  private getSlides({ excludeClones = true }: { excludeClones?: boolean } = {}) {
    return [...this.children].filter(
      (el: HTMLElement) =>
        this.isCarouselItem(el) && (!excludeClones || !el.hasAttribute("data-clone")),
    ) as WaCarouselItem[];
  }

  private handleClick(event: MouseEvent) {
    if (this.dragging && this.dragStartPosition[0] > 0 && this.dragStartPosition[1] > 0) {
      const deltaX = Math.abs(this.dragStartPosition[0] - event.clientX);
      const deltaY = Math.abs(this.dragStartPosition[1] - event.clientY);
      const delta = Math.sqrt(deltaX * deltaX + deltaY * deltaY);

      // Prevents clicks on interactive elements while dragging if the click is within a small range. This prevents
      // accidental drags from interfering with intentional clicks.
      if (delta >= 10) {
        event.preventDefault();
      }
    }
  }

  private handleKeyDown(event: KeyboardEvent) {
    if (["ArrowLeft", "ArrowRight", "ArrowUp", "ArrowDown", "Home", "End"].includes(event.key)) {
      const target = event.target as HTMLElement;
      const isRtl = this.localize.dir() === "rtl";
      const isFocusInPagination = target.closest('[part~="pagination-item"]') !== null;
      const isNext =
        event.key === "ArrowDown" ||
        (!isRtl && event.key === "ArrowRight") ||
        (isRtl && event.key === "ArrowLeft");
      const isPrevious =
        event.key === "ArrowUp" ||
        (!isRtl && event.key === "ArrowLeft") ||
        (isRtl && event.key === "ArrowRight");

      event.preventDefault();

      if (isPrevious) {
        this.previous();
      }

      if (isNext) {
        this.next();
      }

      if (event.key === "Home") {
        this.goToSlide(0);
      }

      if (event.key === "End") {
        this.goToSlide(this.getSlides().length - 1);
      }

      if (isFocusInPagination) {
        this.updateComplete.then(() => {
          const activePaginationItem = this.shadowRoot?.querySelector<HTMLButtonElement>(
            '[part~="pagination-item-active"]',
          );

          if (activePaginationItem) {
            activePaginationItem.focus();
          }
        });
      }
    }
  }

  private handleMouseDragStart(event: PointerEvent) {
    const canDrag = this.mouseDragging && event.button === 0;
    if (canDrag) {
      event.preventDefault();

      document.addEventListener("pointermove", this.handleMouseDrag, {
        capture: true,
        passive: true,
      });
      document.addEventListener("pointerup", this.handleMouseDragEnd, {
        capture: true,
        once: true,
      });
    }
  }

  private handleMouseDrag = (event: PointerEvent) => {
    if (!this.dragging) {
      // Start dragging if it hasn't yet
      this.scrollContainer.style.setProperty("scroll-snap-type", "none");
      this.dragging = true;
      this.dragStartPosition = [event.clientX, event.clientY];
    }

    this.scrollContainer.scrollBy({
      left: -event.movementX,
      top: -event.movementY,
      behavior: "instant",
    });
  };

  private handleMouseDragEnd = () => {
    const scrollContainer = this.scrollContainer;

    document.removeEventListener("pointermove", this.handleMouseDrag, { capture: true });

    // get the current scroll position
    const startLeft = scrollContainer.scrollLeft;
    const startTop = scrollContainer.scrollTop;

    // remove the scroll-snap-type property so that the browser will snap the slide to the correct position
    scrollContainer.style.removeProperty("scroll-snap-type");

    // fix(safari): forcing a style recalculation doesn't seem to immediately update the scroll
    // position in Safari. Setting "overflow" to "hidden" should force this behavior.
    scrollContainer.style.setProperty("overflow", "hidden");

    // get the final scroll position to the slide snapped by the browser
    const finalLeft = scrollContainer.scrollLeft;
    const finalTop = scrollContainer.scrollTop;

    // restore the scroll position to the original one, so that it can be smoothly animated if needed
    scrollContainer.style.removeProperty("overflow");
    scrollContainer.style.setProperty("scroll-snap-type", "none");
    scrollContainer.scrollTo({ left: startLeft, top: startTop, behavior: "instant" });

    requestAnimationFrame(async () => {
      if (startLeft !== finalLeft || startTop !== finalTop) {
        scrollContainer.scrollTo({
          left: finalLeft,
          top: finalTop,
          behavior: prefersReducedMotion() ? "auto" : "smooth",
        });
        await waitForEvent(scrollContainer, "scrollend");
      }

      scrollContainer.style.removeProperty("scroll-snap-type");

      this.dragging = false;
      this.dragStartPosition = [-1, -1];
      this.handleScrollEnd();
    });
  };

  @eventOptions({ passive: true })
  private handleScroll() {
    this.scrolling = true;
    if (!this.pendingSlideChange) {
      this.synchronizeSlides();
    }
  }

  /** @internal Synchronizes the slides with the IntersectionObserver API. */
  private synchronizeSlides() {
    const io = new IntersectionObserver(
      (entries) => {
        io.disconnect();

        for (const entry of entries) {
          const slide = entry.target;
          slide.toggleAttribute("inert", !entry.isIntersecting);
          slide.classList.toggle("--in-view", entry.isIntersecting);
          slide.setAttribute("aria-hidden", entry.isIntersecting ? "false" : "true");
        }

        const firstIntersecting = entries.find((entry) => entry.isIntersecting);

        if (!firstIntersecting) {
          return;
        }

        const slidesWithClones = this.getSlides({ excludeClones: false });
        const slidesCount = this.getSlides().length;

        // Update the current index based on the first visible slide
        const slideIndex = slidesWithClones.indexOf(firstIntersecting.target as WaCarouselItem);
        // Normalize the index to ignore clones
        const normalizedIndex = this.loop ? slideIndex - this.slidesPerPage : slideIndex;

        if (firstIntersecting) {
          // Set the index to the closest "snappable" slide
          this.activeSlide =
            (Math.ceil(normalizedIndex / this.slidesPerMove) * this.slidesPerMove + slidesCount) %
            slidesCount;

          if (!this.scrolling) {
            if (this.loop && firstIntersecting.target.hasAttribute("data-clone")) {
              const clonePosition = Number(firstIntersecting.target.getAttribute("data-clone"));
              // Scrolls to the original slide without animating, so the user won't notice that the position has changed
              this.goToSlide(clonePosition, "instant");
            }
          }
        }
      },
      {
        root: this.scrollContainer,
        threshold: 0.6,
      },
    );

    this.getSlides({ excludeClones: false }).forEach((slide) => {
      io.observe(slide);
    });
  }

  private handleScrollEnd() {
    if (!this.scrolling || this.dragging) return;

    this.synchronizeSlides();

    this.scrolling = false;
    this.pendingSlideChange = false;
    this.synchronizeSlides();
  }

  private isCarouselItem(node: Node): node is WaCarouselItem {
    return node instanceof Element && node.tagName.toLowerCase() === "wa-carousel-item";
  }

  private handleSlotChange = (mutations: MutationRecord[]) => {
    const needsInitialization = mutations.some((mutation) =>
      [...mutation.addedNodes, ...mutation.removedNodes].some(
        (el: HTMLElement) => this.isCarouselItem(el) && !el.hasAttribute("data-clone"),
      ),
    );

    // Reinitialize the carousel if a carousel item has been added or removed
    if (needsInitialization) {
      this.initializeSlides();
    }

    this.requestUpdate();
  };

  @watch("loop", { waitUntilFirstUpdate: true })
  @watch("slidesPerPage", { waitUntilFirstUpdate: true })
  initializeSlides() {
    // Removes all the cloned elements from the carousel
    this.getSlides({ excludeClones: false }).forEach((slide, index) => {
      slide.classList.remove("--in-view");
      slide.classList.remove("--is-active");
      slide.setAttribute("aria-label", this.localize.term("slideNum", index + 1));

      if (slide.hasAttribute("data-clone")) {
        slide.remove();
      }
    });

    this.updateSlidesSnap();

    if (this.loop) {
      // Creates clones to be placed before and after the original elements to simulate infinite scrolling
      this.createClones();
    }

    // Because the DOM may be changed, restore the scroll position to the active slide
    this.goToSlide(this.activeSlide, "auto");

    this.synchronizeSlides();
  }

  private createClones() {
    const slides = this.getSlides();

    const slidesPerPage = this.slidesPerPage;
    const lastSlides = slides.slice(-slidesPerPage);
    const firstSlides = slides.slice(0, slidesPerPage);

    lastSlides.reverse().forEach((slide, i) => {
      const clone = slide.cloneNode(true) as HTMLElement;
      clone.setAttribute("data-clone", String(slides.length - i - 1));
      this.prepend(clone);
    });

    firstSlides.forEach((slide, i) => {
      const clone = slide.cloneNode(true) as HTMLElement;
      clone.setAttribute("data-clone", String(i));
      this.append(clone);
    });
  }

  @watch("activeSlide")
  handleSlideChange() {
    const slides = this.getSlides();
    slides.forEach((slide, i) => {
      slide.classList.toggle("--is-active", i === this.activeSlide);
    });

    // Do not emit an event on first render
    if (this.hasUpdated) {
      this.dispatchEvent(
        new WaSlideChangeEvent({
          index: this.activeSlide,
          slide: slides[this.activeSlide],
        }),
      );
    }
  }

  @watch("slidesPerMove")
  updateSlidesSnap() {
    const slides = this.getSlides();

    const slidesPerMove = this.slidesPerMove;
    slides.forEach((slide, i) => {
      const shouldSnap = (i + slidesPerMove) % slidesPerMove === 0;
      if (shouldSnap) {
        slide.style.removeProperty("scroll-snap-align");
      } else {
        slide.style.setProperty("scroll-snap-align", "none");
      }
    });
  }

  @watch("autoplay")
  handleAutoplayChange() {
    this.autoplayController.stop();
    if (this.autoplay) {
      this.autoplayController.start(this.autoplayInterval);
    }
  }

  /**
   * Move the carousel backward by `slides-per-move` slides.
   *
   * @param behavior - The behavior used for scrolling.
   */
  previous(behavior: ScrollBehavior = "smooth") {
    this.goToSlide(this.activeSlide - this.slidesPerMove, behavior);
  }

  /**
   * Move the carousel forward by `slides-per-move` slides.
   *
   * @param behavior - The behavior used for scrolling.
   */
  next(behavior: ScrollBehavior = "smooth") {
    this.goToSlide(this.activeSlide + this.slidesPerMove, behavior);
  }

  /**
   * Scrolls the carousel to the slide specified by `index`.
   *
   * @param index - The slide index.
   * @param behavior - The behavior used for scrolling.
   */
  goToSlide(index: number, behavior: ScrollBehavior = "smooth") {
    const { slidesPerPage, loop } = this;

    const slides = this.getSlides();
    const slidesWithClones = this.getSlides({ excludeClones: false });

    // No need to do anything in case there are no items in the carousel
    if (!slides.length) {
      return;
    }

    // Sets the next index without taking into account clones, if any.
    const newActiveSlide = loop
      ? (index + slides.length) % slides.length
      : clamp(index, 0, slides.length - slidesPerPage);
    this.activeSlide = newActiveSlide;

    const isRtl = this.localize.dir() === "rtl";

    // Get the index of the next slide. For looping carousel it adds `slidesPerPage`
    // to normalize the starting index in order to ignore the first nth clones.
    // For RTL it needs to scroll to the last slide of the page.
    const nextSlideIndex = clamp(
      index + (loop ? slidesPerPage : 0) + (isRtl ? slidesPerPage - 1 : 0),
      0,
      slidesWithClones.length - 1,
    );

    const nextSlide = slidesWithClones[nextSlideIndex];

    this.scrollToSlide(nextSlide, prefersReducedMotion() ? "auto" : behavior);
  }

  private scrollToSlide(slide: HTMLElement, behavior: ScrollBehavior = "smooth") {
    // Since the geometry doesn't happen until rAF, we don't know if we'll be scrolling or not...
    // It's best to assume that we will and cleanup in the else case below if we didn't need to
    this.pendingSlideChange = true;
    window.requestAnimationFrame(() => {
      // This can happen if goToSlide is called before the scroll container is rendered
      // We will have correctly set the activeSlide in goToSlide which will get picked up when initializeSlides is called.
      if (!this.scrollContainer) {
        return;
      }

      const scrollContainer = this.scrollContainer;
      const scrollContainerRect = scrollContainer.getBoundingClientRect();
      const nextSlideRect = slide.getBoundingClientRect();

      const nextLeft = nextSlideRect.left - scrollContainerRect.left;
      const nextTop = nextSlideRect.top - scrollContainerRect.top;

      if (nextLeft || nextTop) {
        // This is here just in case someone set it back to false
        // between rAF being requested and the callback actually running
        this.pendingSlideChange = true;
        scrollContainer.scrollTo({
          left: nextLeft + scrollContainer.scrollLeft,
          top: nextTop + scrollContainer.scrollTop,
          behavior,
        });
      } else {
        this.pendingSlideChange = false;
      }
    });
  }

  render() {
    const { slidesPerMove, scrolling } = this;

    let pagesCount = 0;
    let currentPage = 0;
    let prevEnabled = false;
    let nextEnabled = false;

    // @TODO: This is a super hacky way to get rid of hydration mismatch errors. The ideal solution is users being able to pass in `pagesCount` and `currentPage` and then on firstUpdated to we update the value for them.
    if (this.hasUpdated) {
      pagesCount = this.getPageCount();
      currentPage = this.getCurrentPage();
      prevEnabled = this.canScrollPrev();
      nextEnabled = this.canScrollNext();
    }

    // We can't rely on `this.matches()` on the server.
    const isRTL = isServer ? this.dir === "rtl" : this.localize.dir() === "rtl";

    return html`
      <div part="base" class="carousel">
        <div
          id="scroll-container"
          part="scroll-container"
          class="${classMap({
            slides: true,
            "slides-horizontal": this.orientation === "horizontal",
            "slides-vertical": this.orientation === "vertical",
            "slides-dragging": this.dragging,
          })}"
          style=${styleMap({ "--slides-per-page": this.slidesPerPage })}
          aria-busy="${scrolling ? "true" : "false"}"
          aria-atomic="true"
          tabindex="0"
          @keydown=${this.handleKeyDown}
          @mousedown="${this.handleMouseDragStart}"
          @scroll="${this.handleScroll}"
          @scrollend=${this.handleScrollEnd}
          @click=${this.handleClick}
        >
          <slot @slotchange=${() => this.requestUpdate()}></slot>
        </div>

        ${
          this.navigation
            ? html`
                <div part="navigation" class="navigation">
                  <button
                    part="navigation-button navigation-button-previous"
                    class="${classMap({
                    "navigation-button": true,
                    "navigation-button-previous": true,
                    "navigation-button-disabled": !prevEnabled,
                  })}"
                    aria-label="${this.localize.term("previousSlide")}"
                    aria-controls="scroll-container"
                    aria-disabled="${prevEnabled ? "false" : "true"}"
                    @click=${prevEnabled ? () => this.previous() : null}
                  >
                    <slot name="previous-icon">
                      <wa-icon
                        library="system"
                        name="${isRTL ? "chevron-right" : "chevron-left"}"
                      ></wa-icon>
                    </slot>
                  </button>

                  <button
                    part="navigation-button navigation-button-next"
                    class=${classMap({
                    "navigation-button": true,
                    "navigation-button-next": true,
                    "navigation-button-disabled": !nextEnabled,
                  })}
                    aria-label="${this.localize.term("nextSlide")}"
                    aria-controls="scroll-container"
                    aria-disabled="${nextEnabled ? "false" : "true"}"
                    @click=${nextEnabled ? () => this.next() : null}
                  >
                    <slot name="next-icon">
                      <wa-icon
                        library="system"
                        name="${isRTL ? "chevron-left" : "chevron-right"}"
                      ></wa-icon>
                    </slot>
                  </button>
                </div>
              `
            : ""
        }
        ${
          this.pagination
            ? html`
                <div
                  part="pagination"
                  role="tablist"
                  class="pagination"
                  aria-controls="scroll-container"
                >
                  ${map(range(pagesCount), (index) => {
                  const isActive = index === currentPage;
                  return html`
                    <button
                      part="pagination-item ${isActive ? "pagination-item-active" : ""}"
                      class="${classMap({
                        "pagination-item": true,
                        "pagination-item-active": isActive,
                      })}"
                      role="tab"
                      aria-selected="${isActive ? "true" : "false"}"
                      aria-label="${this.localize.term("goToSlide", index + 1, pagesCount)}"
                      tabindex=${isActive ? "0" : "-1"}
                      @click=${() => this.goToSlide(index * slidesPerMove)}
                      @keydown=${this.handleKeyDown}
                    ></button>
                  `;
                })}
                </div>
              `
            : html``
        }
      </div>
    `;
  }
}

declare global {
  interface HTMLElementTagNameMap {
    "wa-carousel": WaCarousel;
  }
}

`````
