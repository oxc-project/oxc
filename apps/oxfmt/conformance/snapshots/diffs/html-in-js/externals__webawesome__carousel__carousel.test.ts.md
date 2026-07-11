# externals/webawesome/carousel/carousel.test.ts

## Option 1

`````json
{"printWidth":80}
`````

### Diff

`````diff
===================================================================
--- prettier
+++ oxfmt
@@ -256,9 +256,12 @@
                     slides-per-page="${slidesPerPage}"
                     slides-per-move="${slidesPerMove}"
                     ?loop=${loop}
                   >
-                    ${map(range(slides), (i) => html`<wa-carousel-item>${i}</wa-carousel-item>`)}
+                    ${map(
+                      range(slides),
+                      (i) => html`<wa-carousel-item>${i}</wa-carousel-item>`,
+                    )}
                   </wa-carousel>
                 `);
 
                 const paginationItems =

`````

### Actual (oxfmt)

`````ts
import {
  aTimeout,
  expect,
  nextFrame,
  oneEvent,
  waitUntil,
} from "@open-wc/testing";
import { resetMouse } from "@web/test-runner-commands";
import { html } from "lit";
import { map } from "lit/directives/map.js";
import { range } from "lit/directives/range.js";
import type { SinonStub } from "sinon";
import sinon from "sinon";
import { clientFixture } from "../../internal/test/fixture.js";
import {
  clickOnElement,
  moveMouseOnElement,
} from "../../internal/test/pointer-utilities.js";
import type WaCarousel from "./carousel.js";

describe("<wa-carousel>", () => {
  // @TODO: Fix hydrated fixture.
  for (const fixture of [clientFixture]) {
    describe(`with "${fixture.type}" rendering`, () => {
      const sandbox = sinon.createSandbox();
      const ioCallbacks = new Map<IntersectionObserver, SinonStub>();
      const intersectionObserverCallbacks = () => {
        const callbacks = [...ioCallbacks.values()];
        return waitUntil(() => callbacks.every((callback) => callback.called));
      };
      const OriginalIntersectionObserver = globalThis.IntersectionObserver;

      beforeEach(() => {
        globalThis.IntersectionObserver = class IntersectionObserverMock extends (
          OriginalIntersectionObserver
        ) {
          constructor(
            callback: IntersectionObserverCallback,
            options?: IntersectionObserverInit,
          ) {
            const stubCallback = sandbox.stub().callsFake(callback);

            super(stubCallback, options);

            ioCallbacks.set(this, stubCallback);
          }
        };
      });

      afterEach(async () => {
        await resetMouse();
        sandbox.restore();
        globalThis.IntersectionObserver = OriginalIntersectionObserver;
        ioCallbacks.clear();
      });

      describe("accessibility", () => {
        it("should render with correct ARIA attributes", async () => {
          const el = await fixture(html`
            <wa-carousel>
              <wa-carousel-item>Node 1</wa-carousel-item>
              <wa-carousel-item>Node 2</wa-carousel-item>
              <wa-carousel-item>Node 3</wa-carousel-item>
            </wa-carousel>
          `);

          expect(el).to.exist;
          expect(el).to.have.attribute("role", "region");
          expect(el).to.have.attribute("aria-label", "Carousel");
        });

        it("should pass accessibility tests", async () => {
          const el = await fixture<WaCarousel>(html`
            <wa-carousel navigation pagination>
              <wa-carousel-item>Node 1</wa-carousel-item>
              <wa-carousel-item>Node 2</wa-carousel-item>
              <wa-carousel-item>Node 3</wa-carousel-item>
            </wa-carousel>
          `);
          const pagination = el.shadowRoot!.querySelector(".pagination")!;
          const navigation = el.shadowRoot!.querySelector(".navigation")!;
          await el.updateComplete;

          expect(el.scrollContainer).to.have.attribute("aria-busy", "false");
          expect(el.scrollContainer).to.have.attribute("aria-atomic", "true");

          expect(pagination).to.have.attribute("role", "tablist");
          expect(pagination).to.have.attribute(
            "aria-controls",
            el.scrollContainer.id,
          );
          for (const paginationItem of pagination.querySelectorAll(
            ".pagination-item",
          )) {
            expect(paginationItem).to.have.attribute("role", "tab");
            expect(paginationItem).to.have.attribute("aria-selected");
            expect(paginationItem).to.have.attribute("aria-label");
          }

          for (const navigationItem of navigation.querySelectorAll(
            ".navigation-item",
          )) {
            expect(navigationItem).to.have.attribute(
              "aria-controls",
              el.scrollContainer.id,
            );
            expect(navigationItem).to.have.attribute("aria-disabled");
            expect(navigationItem).to.have.attribute("aria-label");
          }

          await expect(el).to.be.accessible();
        });

        it("should update aria-busy attribute when scrolling", async () => {
          const el = await fixture<WaCarousel>(html`
            <wa-carousel autoplay>
              <wa-carousel-item>Node 1</wa-carousel-item>
              <wa-carousel-item>Node 2</wa-carousel-item>
              <wa-carousel-item>Node 3</wa-carousel-item>
            </wa-carousel>
          `);

          await el.updateComplete;

          el.goToSlide(2, "smooth");
          await oneEvent(el.scrollContainer, "scroll");
          await el.updateComplete;

          expect(el.scrollContainer).to.have.attribute("aria-busy", "true");

          await oneEvent(el.scrollContainer, "scrollend");
          await el.updateComplete;
          expect(el.scrollContainer).to.have.attribute("aria-busy", "false");
        });
      });

      describe("properties", () => {
        it("should not show navigation or pagination by default", async () => {
          const el = await fixture(html`
            <wa-carousel>
              <wa-carousel-item>Node 1</wa-carousel-item>
              <wa-carousel-item>Node 2</wa-carousel-item>
              <wa-carousel-item>Node 3</wa-carousel-item>
            </wa-carousel>
          `);

          expect(el.shadowRoot!.querySelector(".navigation")).not.to.exist;
          expect(el.shadowRoot!.querySelector(".pagination")).not.to.exist;
        });

        describe("navigation", () => {
          it("should render navigation controls when navigation is set", async () => {
            const el = await fixture(html`
              <wa-carousel navigation>
                <wa-carousel-item>Node 1</wa-carousel-item>
                <wa-carousel-item>Node 2</wa-carousel-item>
                <wa-carousel-item>Node 3</wa-carousel-item>
              </wa-carousel>
            `);

            expect(el.shadowRoot!.querySelector(".navigation")).to.exist;
            expect(el.shadowRoot!.querySelector(".pagination")).not.to.exist;
          });
        });

        describe("pagination", () => {
          it("should render pagination controls when pagination is set", async () => {
            const el = await fixture(html`
              <wa-carousel pagination>
                <wa-carousel-item>Node 1</wa-carousel-item>
                <wa-carousel-item>Node 2</wa-carousel-item>
                <wa-carousel-item>Node 3</wa-carousel-item>
              </wa-carousel>
            `);

            expect(el.shadowRoot!.querySelector(".navigation")).not.to.exist;
            expect(el.shadowRoot!.querySelector(".pagination")).to.exist;
          });
        });

        describe("loop", () => {
          it("should create clones of the first and last slides", async () => {
            const el = await fixture<WaCarousel>(html`
              <wa-carousel loop>
                <wa-carousel-item>Node 1</wa-carousel-item>
                <wa-carousel-item>Node 2</wa-carousel-item>
                <wa-carousel-item>Node 3</wa-carousel-item>
              </wa-carousel>
            `);

            await el.updateComplete;

            expect(el.firstElementChild).to.have.attribute("data-clone", "2");
            expect(el.lastElementChild).to.have.attribute("data-clone", "0");
          });

          it("should create multiple clones when slides-per-page is set", async () => {
            const el = await fixture<WaCarousel>(html`
              <wa-carousel loop slides-per-page="2">
                <wa-carousel-item>Node 1</wa-carousel-item>
                <wa-carousel-item>Node 2</wa-carousel-item>
                <wa-carousel-item>Node 3</wa-carousel-item>
              </wa-carousel>
            `);

            await el.updateComplete;
            const clones = [...el.children].filter((child) =>
              child.hasAttribute("data-clone"),
            );

            expect(clones).to.have.lengthOf(4);
          });
        });

        describe("slides-per-page", () => {
          it("should show multiple slides at a given time", async () => {
            const el = await fixture<WaCarousel>(html`
              <wa-carousel slides-per-page="2">
                <wa-carousel-item>Node 1</wa-carousel-item>
                <wa-carousel-item>Node 2</wa-carousel-item>
                <wa-carousel-item>Node 3</wa-carousel-item>
              </wa-carousel>
            `);

            await el.updateComplete;

            expect(
              el.scrollContainer.style
                .getPropertyValue("--slides-per-page")
                .trim(),
            ).to.be.equal("2");
          });

          [
            [7, 2, 1, false, 6],
            [5, 3, 3, false, 2],
            [10, 2, 2, false, 5],
            [7, 2, 1, true, 7],
            [5, 3, 3, true, 2],
            [10, 2, 2, true, 5],
          ].forEach(
            ([slides, slidesPerPage, slidesPerMove, loop, expected]: [
              number,
              number,
              number,
              boolean,
              number,
            ]) => {
              it(`should display ${expected} pages for ${slides} slides grouped by ${slidesPerPage} and scrolled by ${slidesPerMove}${
                loop ? " (loop)" : ""
              }`, async () => {
                const el = await fixture<WaCarousel>(html`
                  <wa-carousel
                    pagination
                    navigation
                    slides-per-page="${slidesPerPage}"
                    slides-per-move="${slidesPerMove}"
                    ?loop=${loop}
                  >
                    ${map(
                      range(slides),
                      (i) => html`<wa-carousel-item>${i}</wa-carousel-item>`,
                    )}
                  </wa-carousel>
                `);

                const paginationItems =
                  el.shadowRoot!.querySelectorAll(".pagination-item");
                expect(paginationItems.length).to.equal(expected);
              });
            },
          );
        });

        describe("slides-per-move", () => {
          it("should set the granularity of snapping", async () => {
            const expectedSnapGranularity = 2;
            const el = await fixture<WaCarousel>(html`
              <wa-carousel
                slides-per-page="${expectedSnapGranularity}"
                slides-per-move="${expectedSnapGranularity}"
              >
                <wa-carousel-item>Node 1</wa-carousel-item>
                <wa-carousel-item>Node 2</wa-carousel-item>
                <wa-carousel-item>Node 3</wa-carousel-item>
                <wa-carousel-item>Node 4</wa-carousel-item>
              </wa-carousel>
            `);

            await el.updateComplete;

            for (let i = 0; i < el.children.length; i++) {
              const child = el.children[i] as HTMLElement;

              if (i % expectedSnapGranularity === 0) {
                expect(
                  child.style.getPropertyValue("scroll-snap-align"),
                ).to.be.equal("");
              } else {
                expect(
                  child.style.getPropertyValue("scroll-snap-align"),
                ).to.be.equal("none");
              }
            }
          });

          it("should be possible to move by the given number of slides at a time", async () => {
            const el = await fixture<WaCarousel>(html`
              <wa-carousel navigation slides-per-move="2" slides-per-page="2">
                <wa-carousel-item>Node 1</wa-carousel-item>
                <wa-carousel-item>Node 2</wa-carousel-item>
                <wa-carousel-item class="expected">Node 3</wa-carousel-item>
                <wa-carousel-item class="expected">Node 4</wa-carousel-item>
                <wa-carousel-item>Node 5</wa-carousel-item>
                <wa-carousel-item>Node 6</wa-carousel-item>
              </wa-carousel>
            `);
            const expectedSlides = el.querySelectorAll(".expected");
            const nextButton: HTMLElement = el.shadowRoot!.querySelector(
              ".navigation-button-next",
            )!;

            await clickOnElement(nextButton);

            await oneEvent(el.scrollContainer, "scrollend");
            await intersectionObserverCallbacks();
            await el.updateComplete;

            for (const expectedSlide of expectedSlides) {
              expect(expectedSlide).to.have.class("--in-view");
              expect(expectedSlide).to.be.visible;
            }
          });

          it("should be possible to move by a number less than the displayed number", async () => {
            const el = await fixture<WaCarousel>(html`
              <wa-carousel navigation slides-per-move="1" slides-per-page="2">
                <wa-carousel-item>Node 1</wa-carousel-item>
                <wa-carousel-item>Node 2</wa-carousel-item>
                <wa-carousel-item>Node 3</wa-carousel-item>
                <wa-carousel-item>Node 4</wa-carousel-item>
                <wa-carousel-item class="expected">Node 5</wa-carousel-item>
                <wa-carousel-item class="expected">Node 6</wa-carousel-item>
              </wa-carousel>
            `);
            const expectedSlides = el.querySelectorAll(".expected");
            const nextButton: HTMLElement = el.shadowRoot!.querySelector(
              ".navigation-button-next",
            )!;

            await clickOnElement(nextButton);
            await aTimeout(50);
            await clickOnElement(nextButton);
            await aTimeout(50);
            await clickOnElement(nextButton);
            await aTimeout(50);
            await clickOnElement(nextButton);
            await aTimeout(50);
            await clickOnElement(nextButton);
            await aTimeout(50);
            await clickOnElement(nextButton);

            await oneEvent(el.scrollContainer, "scrollend");
            await intersectionObserverCallbacks();
            await el.updateComplete;

            for (const expectedSlide of expectedSlides) {
              expect(expectedSlide).to.have.class("--in-view");
              expect(expectedSlide).to.be.visible;
            }
          });

          it("should not allow slides-per-move to be greater than slides-per-page", async () => {
            const expectedSlidesPerMove = 2;
            const el = await fixture<WaCarousel>(html`
              <wa-carousel slides-per-page="${expectedSlidesPerMove}">
                <wa-carousel-item>Node 1</wa-carousel-item>
                <wa-carousel-item>Node 2</wa-carousel-item>
                <wa-carousel-item>Node 3</wa-carousel-item>
                <wa-carousel-item>Node 4</wa-carousel-item>
                <wa-carousel-item>Node 5</wa-carousel-item>
                <wa-carousel-item>Node 6</wa-carousel-item>
              </wa-carousel>
            `);

            el.slidesPerMove = 3;
            await el.updateComplete;

            expect(el.slidesPerMove).to.be.equal(expectedSlidesPerMove);
          });
        });

        describe("orientation", () => {
          it("should make the content scrollable along the y-axis when vertical", async () => {
            const el = await fixture<WaCarousel>(html`
              <wa-carousel orientation="vertical" style="height: 100px">
                <wa-carousel-item>Node 1</wa-carousel-item>
                <wa-carousel-item>Node 2</wa-carousel-item>
              </wa-carousel>
            `);

            await el.updateComplete;

            expect(el.scrollContainer.scrollWidth).to.be.equal(
              el.scrollContainer.clientWidth,
            );
            expect(el.scrollContainer.scrollHeight).to.be.greaterThan(
              el.scrollContainer.clientHeight,
            );
          });

          it("should make the content scrollable along the x-axis when horizontal", async () => {
            const el = await fixture<WaCarousel>(html`
              <wa-carousel orientation="horizontal" style="height: 100px">
                <wa-carousel-item>Node 1</wa-carousel-item>
                <wa-carousel-item>Node 2</wa-carousel-item>
              </wa-carousel>
            `);

            await el.updateComplete;

            expect(el.scrollContainer.scrollWidth).to.be.greaterThan(
              el.scrollContainer.clientWidth,
            );
            expect(el.scrollContainer.scrollHeight).to.be.equal(
              el.scrollContainer.clientHeight,
            );
          });
        });

        describe("autoplay", () => {
          let clock: sinon.SinonFakeTimers;

          beforeEach(() => {
            clock = sandbox.useFakeTimers({
              now: new Date(),
            });
          });

          it("should pause the autoplay while the user is interacting", async () => {
            const el = await fixture<WaCarousel>(html`
              <wa-carousel autoplay autoplay-interval="10">
                <wa-carousel-item>Node 1</wa-carousel-item>
                <wa-carousel-item>Node 2</wa-carousel-item>
                <wa-carousel-item>Node 3</wa-carousel-item>
              </wa-carousel>
            `);
            sandbox.stub(el, "next");

            await el.updateComplete;

            el.dispatchEvent(new Event("mouseenter"));
            await el.updateComplete;
            clock.next();
            clock.next();

            expect(el.next).not.to.have.been.called;
          });

          it("should not resume if the user is still interacting", async () => {
            const el = await fixture<WaCarousel>(html`
              <wa-carousel autoplay autoplay-interval="10">
                <wa-carousel-item>Node 1</wa-carousel-item>
                <wa-carousel-item>Node 2</wa-carousel-item>
                <wa-carousel-item>Node 3</wa-carousel-item>
              </wa-carousel>
            `);
            sandbox.stub(el, "next");

            await el.updateComplete;

            el.dispatchEvent(new Event("mouseenter"));
            el.dispatchEvent(new Event("focusin"));
            await el.updateComplete;

            el.dispatchEvent(new Event("mouseleave"));
            await el.updateComplete;

            clock.next();
            clock.next();

            expect(el.next).not.to.have.been.called;
          });
        });

        describe("mouse-dragging", () => {
          it("should be possible to interact with clickable elements", async () => {
            const el = await fixture<WaCarousel>(html`
              <wa-carousel mouse-dragging>
                <wa-carousel-item><button>click me</button></wa-carousel-item>
                <wa-carousel-item>Node 2</wa-carousel-item>
                <wa-carousel-item>Node 3</wa-carousel-item>
              </wa-carousel>
            `);
            const button = el.querySelector("button")!;

            const clickSpy = sinon.spy();
            button.addEventListener("click", clickSpy);

            await moveMouseOnElement(button);
            await clickOnElement(button);

            expect(clickSpy).to.have.been.called;
          });
        });
      });

      describe("events", () => {
        // wa-slide-change is tested implicitly through navigation tests
      });

      describe("navigation controls", () => {
        describe("next button", () => {
          it("should scroll to the next slide", async () => {
            const el = await fixture<WaCarousel>(html`
              <wa-carousel navigation>
                <wa-carousel-item>Node 1</wa-carousel-item>
                <wa-carousel-item>Node 2</wa-carousel-item>
                <wa-carousel-item>Node 3</wa-carousel-item>
              </wa-carousel>
            `);
            const nextButton: HTMLElement = el.shadowRoot!.querySelector(
              ".navigation-button-next",
            )!;
            sandbox.stub(el, "next");

            await el.updateComplete;

            await clickOnElement(nextButton);
            await el.updateComplete;

            expect(el.next).to.have.been.calledOnce;
          });

          it("should not scroll when on the last slide", async () => {
            const el = await fixture<WaCarousel>(html`
              <wa-carousel navigation>
                <wa-carousel-item>Node 1</wa-carousel-item>
                <wa-carousel-item>Node 2</wa-carousel-item>
                <wa-carousel-item>Node 3</wa-carousel-item>
              </wa-carousel>
            `);
            const nextButton: HTMLElement = el.shadowRoot!.querySelector(
              ".navigation-button-next",
            )!;
            sandbox.stub(el, "next");

            el.goToSlide(2, "auto");
            await oneEvent(el.scrollContainer, "scrollend");
            await intersectionObserverCallbacks();
            await el.updateComplete;

            await clickOnElement(nextButton);
            await el.updateComplete;

            expect(nextButton).to.have.attribute("aria-disabled", "true");
            expect(el.next).not.to.have.been.called;
          });

          it("should scroll to the first slide when loop is enabled and on the last slide", async () => {
            const el = await fixture<WaCarousel>(html`
              <wa-carousel navigation loop>
                <wa-carousel-item>Node 1</wa-carousel-item>
                <wa-carousel-item>Node 2</wa-carousel-item>
                <wa-carousel-item>Node 3</wa-carousel-item>
              </wa-carousel>
            `);
            const nextButton: HTMLElement = el.shadowRoot!.querySelector(
              ".navigation-button-next",
            )!;

            el.goToSlide(2, "auto");
            await oneEvent(el.scrollContainer, "scrollend");
            await el.updateComplete;

            await clickOnElement(nextButton);

            // wait first scroll to clone
            await oneEvent(el.scrollContainer, "scrollend");
            // wait scroll to actual item
            await oneEvent(el.scrollContainer, "scrollend");

            await intersectionObserverCallbacks();
            await el.updateComplete;

            expect(nextButton).to.have.attribute("aria-disabled", "false");
            expect(el.activeSlide).to.be.equal(0);
          });
        });

        describe("previous button", () => {
          it("should scroll to the previous slide", async () => {
            const el = await fixture<WaCarousel>(html`
              <wa-carousel navigation>
                <wa-carousel-item>Node 1</wa-carousel-item>
                <wa-carousel-item>Node 2</wa-carousel-item>
                <wa-carousel-item>Node 3</wa-carousel-item>
              </wa-carousel>
            `);

            // Go to the second slide so the previous button is enabled
            el.goToSlide(1, "auto");
            await oneEvent(el.scrollContainer, "scrollend");
            await el.updateComplete;

            const previousButton: HTMLElement = el.shadowRoot!.querySelector(
              ".navigation-button-previous",
            )!;
            sandbox.stub(el, "previous");

            await el.updateComplete;

            await clickOnElement(previousButton);
            await el.updateComplete;

            expect(el.previous).to.have.been.calledOnce;
          });

          it("should not scroll when on the first slide", async () => {
            const el = await fixture<WaCarousel>(html`
              <wa-carousel navigation>
                <wa-carousel-item>Node 1</wa-carousel-item>
                <wa-carousel-item>Node 2</wa-carousel-item>
                <wa-carousel-item>Node 3</wa-carousel-item>
              </wa-carousel>
            `);

            const previousButton: HTMLElement = el.shadowRoot!.querySelector(
              ".navigation-button-previous",
            )!;
            sandbox.stub(el, "previous");
            await el.updateComplete;

            await clickOnElement(previousButton);
            await el.updateComplete;

            expect(previousButton).to.have.attribute("aria-disabled", "true");
            expect(el.previous).not.to.have.been.called;
          });

          it("should scroll to the last slide when loop is enabled and on the first slide", async () => {
            const el = await fixture<WaCarousel>(html`
              <wa-carousel navigation loop>
                <wa-carousel-item>Node 1</wa-carousel-item>
                <wa-carousel-item>Node 2</wa-carousel-item>
                <wa-carousel-item>Node 3</wa-carousel-item>
              </wa-carousel>
            `);

            const previousButton: HTMLElement = el.shadowRoot!.querySelector(
              ".navigation-button-previous",
            )!;
            await el.updateComplete;

            await clickOnElement(previousButton);

            // wait first scroll to clone
            await oneEvent(el.scrollContainer, "scrollend");
            // wait scroll to actual item
            await oneEvent(el.scrollContainer, "scrollend");

            await intersectionObserverCallbacks();

            expect(previousButton).to.have.attribute("aria-disabled", "false");
            expect(el.activeSlide).to.be.equal(2);
          });
        });
      });

      describe("API methods", () => {
        describe("#next", () => {
          it("should scroll the carousel to the next slide", async () => {
            const el = await fixture<WaCarousel>(html`
              <wa-carousel>
                <wa-carousel-item>Node 1</wa-carousel-item>
                <wa-carousel-item>Node 2</wa-carousel-item>
                <wa-carousel-item>Node 3</wa-carousel-item>
              </wa-carousel>
            `);
            sandbox.spy(el, "goToSlide");
            const expectedCarouselItem: HTMLElement = el.querySelector(
              "wa-carousel-item:nth-child(2)",
            )!;

            el.next();
            await oneEvent(el.scrollContainer, "scrollend");
            await el.updateComplete;

            const containerRect = el.scrollContainer.getBoundingClientRect();
            const itemRect = expectedCarouselItem.getBoundingClientRect();

            expect(el.goToSlide).to.have.been.calledWith(1);
            expect(itemRect.top).to.be.equal(containerRect.top);
            expect(itemRect.left).to.be.equal(containerRect.left);
          });
        });

        describe("#previous", () => {
          it("should scroll the carousel to the previous slide", async () => {
            const el = await fixture<WaCarousel>(html`
              <wa-carousel>
                <wa-carousel-item>Node 1</wa-carousel-item>
                <wa-carousel-item>Node 2</wa-carousel-item>
                <wa-carousel-item>Node 3</wa-carousel-item>
              </wa-carousel>
            `);
            const expectedCarouselItem: HTMLElement = el.querySelector(
              "wa-carousel-item:nth-child(1)",
            )!;

            el.goToSlide(1);

            await oneEvent(el.scrollContainer, "scrollend");
            await intersectionObserverCallbacks();
            await nextFrame();

            sandbox.spy(el, "goToSlide");

            el.previous();
            await oneEvent(el.scrollContainer, "scrollend");
            await intersectionObserverCallbacks();

            const containerRect = el.scrollContainer.getBoundingClientRect();
            const itemRect = expectedCarouselItem.getBoundingClientRect();

            expect(el.goToSlide).to.have.been.calledWith(0);
            expect(itemRect.top).to.be.equal(containerRect.top);
            expect(itemRect.left).to.be.equal(containerRect.left);
          });
        });

        describe("#goToSlide", () => {
          it("should scroll the carousel to the nth slide", async () => {
            const el = await fixture<WaCarousel>(html`
              <wa-carousel>
                <wa-carousel-item>Node 1</wa-carousel-item>
                <wa-carousel-item>Node 2</wa-carousel-item>
                <wa-carousel-item>Node 3</wa-carousel-item>
              </wa-carousel>
            `);
            await el.updateComplete;

            el.goToSlide(2);
            await oneEvent(el.scrollContainer, "scrollend");
            await intersectionObserverCallbacks();
            await el.updateComplete;

            expect(el.activeSlide).to.be.equal(2);
          });
        });
      });

      describe("when inside a hidden container", () => {
        it("should not leave slides inert when the container becomes visible", async () => {
          const container = document.createElement("div");
          container.style.display = "none";
          document.body.appendChild(container);

          container.innerHTML = `
            <wa-carousel>
              <wa-carousel-item>Node 1</wa-carousel-item>
              <wa-carousel-item>Node 2</wa-carousel-item>
              <wa-carousel-item>Node 3</wa-carousel-item>
            </wa-carousel>
          `;

          const el = container.querySelector<WaCarousel>("wa-carousel")!;
          await el.updateComplete;

          container.style.display = "block";

          // Allow ResizeObserver callback and IntersectionObserver to settle
          await aTimeout(50);
          await intersectionObserverCallbacks();
          await el.updateComplete;

          const slides = [...el.querySelectorAll("wa-carousel-item")];
          expect(slides[0]).to.not.have.attribute("inert");

          document.body.removeChild(container);
        });
      });
    });
  }
});

`````

### Expected (prettier)

`````ts
import {
  aTimeout,
  expect,
  nextFrame,
  oneEvent,
  waitUntil,
} from "@open-wc/testing";
import { resetMouse } from "@web/test-runner-commands";
import { html } from "lit";
import { map } from "lit/directives/map.js";
import { range } from "lit/directives/range.js";
import type { SinonStub } from "sinon";
import sinon from "sinon";
import { clientFixture } from "../../internal/test/fixture.js";
import {
  clickOnElement,
  moveMouseOnElement,
} from "../../internal/test/pointer-utilities.js";
import type WaCarousel from "./carousel.js";

describe("<wa-carousel>", () => {
  // @TODO: Fix hydrated fixture.
  for (const fixture of [clientFixture]) {
    describe(`with "${fixture.type}" rendering`, () => {
      const sandbox = sinon.createSandbox();
      const ioCallbacks = new Map<IntersectionObserver, SinonStub>();
      const intersectionObserverCallbacks = () => {
        const callbacks = [...ioCallbacks.values()];
        return waitUntil(() => callbacks.every((callback) => callback.called));
      };
      const OriginalIntersectionObserver = globalThis.IntersectionObserver;

      beforeEach(() => {
        globalThis.IntersectionObserver = class IntersectionObserverMock extends (
          OriginalIntersectionObserver
        ) {
          constructor(
            callback: IntersectionObserverCallback,
            options?: IntersectionObserverInit,
          ) {
            const stubCallback = sandbox.stub().callsFake(callback);

            super(stubCallback, options);

            ioCallbacks.set(this, stubCallback);
          }
        };
      });

      afterEach(async () => {
        await resetMouse();
        sandbox.restore();
        globalThis.IntersectionObserver = OriginalIntersectionObserver;
        ioCallbacks.clear();
      });

      describe("accessibility", () => {
        it("should render with correct ARIA attributes", async () => {
          const el = await fixture(html`
            <wa-carousel>
              <wa-carousel-item>Node 1</wa-carousel-item>
              <wa-carousel-item>Node 2</wa-carousel-item>
              <wa-carousel-item>Node 3</wa-carousel-item>
            </wa-carousel>
          `);

          expect(el).to.exist;
          expect(el).to.have.attribute("role", "region");
          expect(el).to.have.attribute("aria-label", "Carousel");
        });

        it("should pass accessibility tests", async () => {
          const el = await fixture<WaCarousel>(html`
            <wa-carousel navigation pagination>
              <wa-carousel-item>Node 1</wa-carousel-item>
              <wa-carousel-item>Node 2</wa-carousel-item>
              <wa-carousel-item>Node 3</wa-carousel-item>
            </wa-carousel>
          `);
          const pagination = el.shadowRoot!.querySelector(".pagination")!;
          const navigation = el.shadowRoot!.querySelector(".navigation")!;
          await el.updateComplete;

          expect(el.scrollContainer).to.have.attribute("aria-busy", "false");
          expect(el.scrollContainer).to.have.attribute("aria-atomic", "true");

          expect(pagination).to.have.attribute("role", "tablist");
          expect(pagination).to.have.attribute(
            "aria-controls",
            el.scrollContainer.id,
          );
          for (const paginationItem of pagination.querySelectorAll(
            ".pagination-item",
          )) {
            expect(paginationItem).to.have.attribute("role", "tab");
            expect(paginationItem).to.have.attribute("aria-selected");
            expect(paginationItem).to.have.attribute("aria-label");
          }

          for (const navigationItem of navigation.querySelectorAll(
            ".navigation-item",
          )) {
            expect(navigationItem).to.have.attribute(
              "aria-controls",
              el.scrollContainer.id,
            );
            expect(navigationItem).to.have.attribute("aria-disabled");
            expect(navigationItem).to.have.attribute("aria-label");
          }

          await expect(el).to.be.accessible();
        });

        it("should update aria-busy attribute when scrolling", async () => {
          const el = await fixture<WaCarousel>(html`
            <wa-carousel autoplay>
              <wa-carousel-item>Node 1</wa-carousel-item>
              <wa-carousel-item>Node 2</wa-carousel-item>
              <wa-carousel-item>Node 3</wa-carousel-item>
            </wa-carousel>
          `);

          await el.updateComplete;

          el.goToSlide(2, "smooth");
          await oneEvent(el.scrollContainer, "scroll");
          await el.updateComplete;

          expect(el.scrollContainer).to.have.attribute("aria-busy", "true");

          await oneEvent(el.scrollContainer, "scrollend");
          await el.updateComplete;
          expect(el.scrollContainer).to.have.attribute("aria-busy", "false");
        });
      });

      describe("properties", () => {
        it("should not show navigation or pagination by default", async () => {
          const el = await fixture(html`
            <wa-carousel>
              <wa-carousel-item>Node 1</wa-carousel-item>
              <wa-carousel-item>Node 2</wa-carousel-item>
              <wa-carousel-item>Node 3</wa-carousel-item>
            </wa-carousel>
          `);

          expect(el.shadowRoot!.querySelector(".navigation")).not.to.exist;
          expect(el.shadowRoot!.querySelector(".pagination")).not.to.exist;
        });

        describe("navigation", () => {
          it("should render navigation controls when navigation is set", async () => {
            const el = await fixture(html`
              <wa-carousel navigation>
                <wa-carousel-item>Node 1</wa-carousel-item>
                <wa-carousel-item>Node 2</wa-carousel-item>
                <wa-carousel-item>Node 3</wa-carousel-item>
              </wa-carousel>
            `);

            expect(el.shadowRoot!.querySelector(".navigation")).to.exist;
            expect(el.shadowRoot!.querySelector(".pagination")).not.to.exist;
          });
        });

        describe("pagination", () => {
          it("should render pagination controls when pagination is set", async () => {
            const el = await fixture(html`
              <wa-carousel pagination>
                <wa-carousel-item>Node 1</wa-carousel-item>
                <wa-carousel-item>Node 2</wa-carousel-item>
                <wa-carousel-item>Node 3</wa-carousel-item>
              </wa-carousel>
            `);

            expect(el.shadowRoot!.querySelector(".navigation")).not.to.exist;
            expect(el.shadowRoot!.querySelector(".pagination")).to.exist;
          });
        });

        describe("loop", () => {
          it("should create clones of the first and last slides", async () => {
            const el = await fixture<WaCarousel>(html`
              <wa-carousel loop>
                <wa-carousel-item>Node 1</wa-carousel-item>
                <wa-carousel-item>Node 2</wa-carousel-item>
                <wa-carousel-item>Node 3</wa-carousel-item>
              </wa-carousel>
            `);

            await el.updateComplete;

            expect(el.firstElementChild).to.have.attribute("data-clone", "2");
            expect(el.lastElementChild).to.have.attribute("data-clone", "0");
          });

          it("should create multiple clones when slides-per-page is set", async () => {
            const el = await fixture<WaCarousel>(html`
              <wa-carousel loop slides-per-page="2">
                <wa-carousel-item>Node 1</wa-carousel-item>
                <wa-carousel-item>Node 2</wa-carousel-item>
                <wa-carousel-item>Node 3</wa-carousel-item>
              </wa-carousel>
            `);

            await el.updateComplete;
            const clones = [...el.children].filter((child) =>
              child.hasAttribute("data-clone"),
            );

            expect(clones).to.have.lengthOf(4);
          });
        });

        describe("slides-per-page", () => {
          it("should show multiple slides at a given time", async () => {
            const el = await fixture<WaCarousel>(html`
              <wa-carousel slides-per-page="2">
                <wa-carousel-item>Node 1</wa-carousel-item>
                <wa-carousel-item>Node 2</wa-carousel-item>
                <wa-carousel-item>Node 3</wa-carousel-item>
              </wa-carousel>
            `);

            await el.updateComplete;

            expect(
              el.scrollContainer.style
                .getPropertyValue("--slides-per-page")
                .trim(),
            ).to.be.equal("2");
          });

          [
            [7, 2, 1, false, 6],
            [5, 3, 3, false, 2],
            [10, 2, 2, false, 5],
            [7, 2, 1, true, 7],
            [5, 3, 3, true, 2],
            [10, 2, 2, true, 5],
          ].forEach(
            ([slides, slidesPerPage, slidesPerMove, loop, expected]: [
              number,
              number,
              number,
              boolean,
              number,
            ]) => {
              it(`should display ${expected} pages for ${slides} slides grouped by ${slidesPerPage} and scrolled by ${slidesPerMove}${
                loop ? " (loop)" : ""
              }`, async () => {
                const el = await fixture<WaCarousel>(html`
                  <wa-carousel
                    pagination
                    navigation
                    slides-per-page="${slidesPerPage}"
                    slides-per-move="${slidesPerMove}"
                    ?loop=${loop}
                  >
                    ${map(range(slides), (i) => html`<wa-carousel-item>${i}</wa-carousel-item>`)}
                  </wa-carousel>
                `);

                const paginationItems =
                  el.shadowRoot!.querySelectorAll(".pagination-item");
                expect(paginationItems.length).to.equal(expected);
              });
            },
          );
        });

        describe("slides-per-move", () => {
          it("should set the granularity of snapping", async () => {
            const expectedSnapGranularity = 2;
            const el = await fixture<WaCarousel>(html`
              <wa-carousel
                slides-per-page="${expectedSnapGranularity}"
                slides-per-move="${expectedSnapGranularity}"
              >
                <wa-carousel-item>Node 1</wa-carousel-item>
                <wa-carousel-item>Node 2</wa-carousel-item>
                <wa-carousel-item>Node 3</wa-carousel-item>
                <wa-carousel-item>Node 4</wa-carousel-item>
              </wa-carousel>
            `);

            await el.updateComplete;

            for (let i = 0; i < el.children.length; i++) {
              const child = el.children[i] as HTMLElement;

              if (i % expectedSnapGranularity === 0) {
                expect(
                  child.style.getPropertyValue("scroll-snap-align"),
                ).to.be.equal("");
              } else {
                expect(
                  child.style.getPropertyValue("scroll-snap-align"),
                ).to.be.equal("none");
              }
            }
          });

          it("should be possible to move by the given number of slides at a time", async () => {
            const el = await fixture<WaCarousel>(html`
              <wa-carousel navigation slides-per-move="2" slides-per-page="2">
                <wa-carousel-item>Node 1</wa-carousel-item>
                <wa-carousel-item>Node 2</wa-carousel-item>
                <wa-carousel-item class="expected">Node 3</wa-carousel-item>
                <wa-carousel-item class="expected">Node 4</wa-carousel-item>
                <wa-carousel-item>Node 5</wa-carousel-item>
                <wa-carousel-item>Node 6</wa-carousel-item>
              </wa-carousel>
            `);
            const expectedSlides = el.querySelectorAll(".expected");
            const nextButton: HTMLElement = el.shadowRoot!.querySelector(
              ".navigation-button-next",
            )!;

            await clickOnElement(nextButton);

            await oneEvent(el.scrollContainer, "scrollend");
            await intersectionObserverCallbacks();
            await el.updateComplete;

            for (const expectedSlide of expectedSlides) {
              expect(expectedSlide).to.have.class("--in-view");
              expect(expectedSlide).to.be.visible;
            }
          });

          it("should be possible to move by a number less than the displayed number", async () => {
            const el = await fixture<WaCarousel>(html`
              <wa-carousel navigation slides-per-move="1" slides-per-page="2">
                <wa-carousel-item>Node 1</wa-carousel-item>
                <wa-carousel-item>Node 2</wa-carousel-item>
                <wa-carousel-item>Node 3</wa-carousel-item>
                <wa-carousel-item>Node 4</wa-carousel-item>
                <wa-carousel-item class="expected">Node 5</wa-carousel-item>
                <wa-carousel-item class="expected">Node 6</wa-carousel-item>
              </wa-carousel>
            `);
            const expectedSlides = el.querySelectorAll(".expected");
            const nextButton: HTMLElement = el.shadowRoot!.querySelector(
              ".navigation-button-next",
            )!;

            await clickOnElement(nextButton);
            await aTimeout(50);
            await clickOnElement(nextButton);
            await aTimeout(50);
            await clickOnElement(nextButton);
            await aTimeout(50);
            await clickOnElement(nextButton);
            await aTimeout(50);
            await clickOnElement(nextButton);
            await aTimeout(50);
            await clickOnElement(nextButton);

            await oneEvent(el.scrollContainer, "scrollend");
            await intersectionObserverCallbacks();
            await el.updateComplete;

            for (const expectedSlide of expectedSlides) {
              expect(expectedSlide).to.have.class("--in-view");
              expect(expectedSlide).to.be.visible;
            }
          });

          it("should not allow slides-per-move to be greater than slides-per-page", async () => {
            const expectedSlidesPerMove = 2;
            const el = await fixture<WaCarousel>(html`
              <wa-carousel slides-per-page="${expectedSlidesPerMove}">
                <wa-carousel-item>Node 1</wa-carousel-item>
                <wa-carousel-item>Node 2</wa-carousel-item>
                <wa-carousel-item>Node 3</wa-carousel-item>
                <wa-carousel-item>Node 4</wa-carousel-item>
                <wa-carousel-item>Node 5</wa-carousel-item>
                <wa-carousel-item>Node 6</wa-carousel-item>
              </wa-carousel>
            `);

            el.slidesPerMove = 3;
            await el.updateComplete;

            expect(el.slidesPerMove).to.be.equal(expectedSlidesPerMove);
          });
        });

        describe("orientation", () => {
          it("should make the content scrollable along the y-axis when vertical", async () => {
            const el = await fixture<WaCarousel>(html`
              <wa-carousel orientation="vertical" style="height: 100px">
                <wa-carousel-item>Node 1</wa-carousel-item>
                <wa-carousel-item>Node 2</wa-carousel-item>
              </wa-carousel>
            `);

            await el.updateComplete;

            expect(el.scrollContainer.scrollWidth).to.be.equal(
              el.scrollContainer.clientWidth,
            );
            expect(el.scrollContainer.scrollHeight).to.be.greaterThan(
              el.scrollContainer.clientHeight,
            );
          });

          it("should make the content scrollable along the x-axis when horizontal", async () => {
            const el = await fixture<WaCarousel>(html`
              <wa-carousel orientation="horizontal" style="height: 100px">
                <wa-carousel-item>Node 1</wa-carousel-item>
                <wa-carousel-item>Node 2</wa-carousel-item>
              </wa-carousel>
            `);

            await el.updateComplete;

            expect(el.scrollContainer.scrollWidth).to.be.greaterThan(
              el.scrollContainer.clientWidth,
            );
            expect(el.scrollContainer.scrollHeight).to.be.equal(
              el.scrollContainer.clientHeight,
            );
          });
        });

        describe("autoplay", () => {
          let clock: sinon.SinonFakeTimers;

          beforeEach(() => {
            clock = sandbox.useFakeTimers({
              now: new Date(),
            });
          });

          it("should pause the autoplay while the user is interacting", async () => {
            const el = await fixture<WaCarousel>(html`
              <wa-carousel autoplay autoplay-interval="10">
                <wa-carousel-item>Node 1</wa-carousel-item>
                <wa-carousel-item>Node 2</wa-carousel-item>
                <wa-carousel-item>Node 3</wa-carousel-item>
              </wa-carousel>
            `);
            sandbox.stub(el, "next");

            await el.updateComplete;

            el.dispatchEvent(new Event("mouseenter"));
            await el.updateComplete;
            clock.next();
            clock.next();

            expect(el.next).not.to.have.been.called;
          });

          it("should not resume if the user is still interacting", async () => {
            const el = await fixture<WaCarousel>(html`
              <wa-carousel autoplay autoplay-interval="10">
                <wa-carousel-item>Node 1</wa-carousel-item>
                <wa-carousel-item>Node 2</wa-carousel-item>
                <wa-carousel-item>Node 3</wa-carousel-item>
              </wa-carousel>
            `);
            sandbox.stub(el, "next");

            await el.updateComplete;

            el.dispatchEvent(new Event("mouseenter"));
            el.dispatchEvent(new Event("focusin"));
            await el.updateComplete;

            el.dispatchEvent(new Event("mouseleave"));
            await el.updateComplete;

            clock.next();
            clock.next();

            expect(el.next).not.to.have.been.called;
          });
        });

        describe("mouse-dragging", () => {
          it("should be possible to interact with clickable elements", async () => {
            const el = await fixture<WaCarousel>(html`
              <wa-carousel mouse-dragging>
                <wa-carousel-item><button>click me</button></wa-carousel-item>
                <wa-carousel-item>Node 2</wa-carousel-item>
                <wa-carousel-item>Node 3</wa-carousel-item>
              </wa-carousel>
            `);
            const button = el.querySelector("button")!;

            const clickSpy = sinon.spy();
            button.addEventListener("click", clickSpy);

            await moveMouseOnElement(button);
            await clickOnElement(button);

            expect(clickSpy).to.have.been.called;
          });
        });
      });

      describe("events", () => {
        // wa-slide-change is tested implicitly through navigation tests
      });

      describe("navigation controls", () => {
        describe("next button", () => {
          it("should scroll to the next slide", async () => {
            const el = await fixture<WaCarousel>(html`
              <wa-carousel navigation>
                <wa-carousel-item>Node 1</wa-carousel-item>
                <wa-carousel-item>Node 2</wa-carousel-item>
                <wa-carousel-item>Node 3</wa-carousel-item>
              </wa-carousel>
            `);
            const nextButton: HTMLElement = el.shadowRoot!.querySelector(
              ".navigation-button-next",
            )!;
            sandbox.stub(el, "next");

            await el.updateComplete;

            await clickOnElement(nextButton);
            await el.updateComplete;

            expect(el.next).to.have.been.calledOnce;
          });

          it("should not scroll when on the last slide", async () => {
            const el = await fixture<WaCarousel>(html`
              <wa-carousel navigation>
                <wa-carousel-item>Node 1</wa-carousel-item>
                <wa-carousel-item>Node 2</wa-carousel-item>
                <wa-carousel-item>Node 3</wa-carousel-item>
              </wa-carousel>
            `);
            const nextButton: HTMLElement = el.shadowRoot!.querySelector(
              ".navigation-button-next",
            )!;
            sandbox.stub(el, "next");

            el.goToSlide(2, "auto");
            await oneEvent(el.scrollContainer, "scrollend");
            await intersectionObserverCallbacks();
            await el.updateComplete;

            await clickOnElement(nextButton);
            await el.updateComplete;

            expect(nextButton).to.have.attribute("aria-disabled", "true");
            expect(el.next).not.to.have.been.called;
          });

          it("should scroll to the first slide when loop is enabled and on the last slide", async () => {
            const el = await fixture<WaCarousel>(html`
              <wa-carousel navigation loop>
                <wa-carousel-item>Node 1</wa-carousel-item>
                <wa-carousel-item>Node 2</wa-carousel-item>
                <wa-carousel-item>Node 3</wa-carousel-item>
              </wa-carousel>
            `);
            const nextButton: HTMLElement = el.shadowRoot!.querySelector(
              ".navigation-button-next",
            )!;

            el.goToSlide(2, "auto");
            await oneEvent(el.scrollContainer, "scrollend");
            await el.updateComplete;

            await clickOnElement(nextButton);

            // wait first scroll to clone
            await oneEvent(el.scrollContainer, "scrollend");
            // wait scroll to actual item
            await oneEvent(el.scrollContainer, "scrollend");

            await intersectionObserverCallbacks();
            await el.updateComplete;

            expect(nextButton).to.have.attribute("aria-disabled", "false");
            expect(el.activeSlide).to.be.equal(0);
          });
        });

        describe("previous button", () => {
          it("should scroll to the previous slide", async () => {
            const el = await fixture<WaCarousel>(html`
              <wa-carousel navigation>
                <wa-carousel-item>Node 1</wa-carousel-item>
                <wa-carousel-item>Node 2</wa-carousel-item>
                <wa-carousel-item>Node 3</wa-carousel-item>
              </wa-carousel>
            `);

            // Go to the second slide so the previous button is enabled
            el.goToSlide(1, "auto");
            await oneEvent(el.scrollContainer, "scrollend");
            await el.updateComplete;

            const previousButton: HTMLElement = el.shadowRoot!.querySelector(
              ".navigation-button-previous",
            )!;
            sandbox.stub(el, "previous");

            await el.updateComplete;

            await clickOnElement(previousButton);
            await el.updateComplete;

            expect(el.previous).to.have.been.calledOnce;
          });

          it("should not scroll when on the first slide", async () => {
            const el = await fixture<WaCarousel>(html`
              <wa-carousel navigation>
                <wa-carousel-item>Node 1</wa-carousel-item>
                <wa-carousel-item>Node 2</wa-carousel-item>
                <wa-carousel-item>Node 3</wa-carousel-item>
              </wa-carousel>
            `);

            const previousButton: HTMLElement = el.shadowRoot!.querySelector(
              ".navigation-button-previous",
            )!;
            sandbox.stub(el, "previous");
            await el.updateComplete;

            await clickOnElement(previousButton);
            await el.updateComplete;

            expect(previousButton).to.have.attribute("aria-disabled", "true");
            expect(el.previous).not.to.have.been.called;
          });

          it("should scroll to the last slide when loop is enabled and on the first slide", async () => {
            const el = await fixture<WaCarousel>(html`
              <wa-carousel navigation loop>
                <wa-carousel-item>Node 1</wa-carousel-item>
                <wa-carousel-item>Node 2</wa-carousel-item>
                <wa-carousel-item>Node 3</wa-carousel-item>
              </wa-carousel>
            `);

            const previousButton: HTMLElement = el.shadowRoot!.querySelector(
              ".navigation-button-previous",
            )!;
            await el.updateComplete;

            await clickOnElement(previousButton);

            // wait first scroll to clone
            await oneEvent(el.scrollContainer, "scrollend");
            // wait scroll to actual item
            await oneEvent(el.scrollContainer, "scrollend");

            await intersectionObserverCallbacks();

            expect(previousButton).to.have.attribute("aria-disabled", "false");
            expect(el.activeSlide).to.be.equal(2);
          });
        });
      });

      describe("API methods", () => {
        describe("#next", () => {
          it("should scroll the carousel to the next slide", async () => {
            const el = await fixture<WaCarousel>(html`
              <wa-carousel>
                <wa-carousel-item>Node 1</wa-carousel-item>
                <wa-carousel-item>Node 2</wa-carousel-item>
                <wa-carousel-item>Node 3</wa-carousel-item>
              </wa-carousel>
            `);
            sandbox.spy(el, "goToSlide");
            const expectedCarouselItem: HTMLElement = el.querySelector(
              "wa-carousel-item:nth-child(2)",
            )!;

            el.next();
            await oneEvent(el.scrollContainer, "scrollend");
            await el.updateComplete;

            const containerRect = el.scrollContainer.getBoundingClientRect();
            const itemRect = expectedCarouselItem.getBoundingClientRect();

            expect(el.goToSlide).to.have.been.calledWith(1);
            expect(itemRect.top).to.be.equal(containerRect.top);
            expect(itemRect.left).to.be.equal(containerRect.left);
          });
        });

        describe("#previous", () => {
          it("should scroll the carousel to the previous slide", async () => {
            const el = await fixture<WaCarousel>(html`
              <wa-carousel>
                <wa-carousel-item>Node 1</wa-carousel-item>
                <wa-carousel-item>Node 2</wa-carousel-item>
                <wa-carousel-item>Node 3</wa-carousel-item>
              </wa-carousel>
            `);
            const expectedCarouselItem: HTMLElement = el.querySelector(
              "wa-carousel-item:nth-child(1)",
            )!;

            el.goToSlide(1);

            await oneEvent(el.scrollContainer, "scrollend");
            await intersectionObserverCallbacks();
            await nextFrame();

            sandbox.spy(el, "goToSlide");

            el.previous();
            await oneEvent(el.scrollContainer, "scrollend");
            await intersectionObserverCallbacks();

            const containerRect = el.scrollContainer.getBoundingClientRect();
            const itemRect = expectedCarouselItem.getBoundingClientRect();

            expect(el.goToSlide).to.have.been.calledWith(0);
            expect(itemRect.top).to.be.equal(containerRect.top);
            expect(itemRect.left).to.be.equal(containerRect.left);
          });
        });

        describe("#goToSlide", () => {
          it("should scroll the carousel to the nth slide", async () => {
            const el = await fixture<WaCarousel>(html`
              <wa-carousel>
                <wa-carousel-item>Node 1</wa-carousel-item>
                <wa-carousel-item>Node 2</wa-carousel-item>
                <wa-carousel-item>Node 3</wa-carousel-item>
              </wa-carousel>
            `);
            await el.updateComplete;

            el.goToSlide(2);
            await oneEvent(el.scrollContainer, "scrollend");
            await intersectionObserverCallbacks();
            await el.updateComplete;

            expect(el.activeSlide).to.be.equal(2);
          });
        });
      });

      describe("when inside a hidden container", () => {
        it("should not leave slides inert when the container becomes visible", async () => {
          const container = document.createElement("div");
          container.style.display = "none";
          document.body.appendChild(container);

          container.innerHTML = `
            <wa-carousel>
              <wa-carousel-item>Node 1</wa-carousel-item>
              <wa-carousel-item>Node 2</wa-carousel-item>
              <wa-carousel-item>Node 3</wa-carousel-item>
            </wa-carousel>
          `;

          const el = container.querySelector<WaCarousel>("wa-carousel")!;
          await el.updateComplete;

          container.style.display = "block";

          // Allow ResizeObserver callback and IntersectionObserver to settle
          await aTimeout(50);
          await intersectionObserverCallbacks();
          await el.updateComplete;

          const slides = [...el.querySelectorAll("wa-carousel-item")];
          expect(slides[0]).to.not.have.attribute("inert");

          document.body.removeChild(container);
        });
      });
    });
  }
});

`````
