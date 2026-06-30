# externals/webawesome/page/page.styles.ts

> Layout-only: Prettier's fill fit-check breaks inside `::slotted()` after a long `:not(...)`; ours breaks inside `:not(...)`. See crates/oxc_formatter_css/AGENTS.md

## Option 1

`````json
{"printWidth":80}
`````

### Diff

`````diff
===================================================================
--- prettier
+++ oxfmt
@@ -20,11 +20,12 @@
     --header-top: var(--header-height);
     --subheader-top: var(--subheader-height);
   }
 
-  slot[name]:not([name="skip-to-content"], [name="navigation-toggle"])::slotted(
-      *
-    ) {
+  slot[name]:not(
+    [name="skip-to-content"],
+    [name="navigation-toggle"]
+  )::slotted(*) {
     display: flex;
     background-color: var(--wa-color-surface-default);
   }
 

`````

### Actual (oxfmt)

`````ts
import { css } from "lit";

export default css`
  :host {
    display: block;
    background-color: var(--wa-color-surface-default);
    box-sizing: border-box;
    min-height: 100%;
    --menu-width: auto;
    --main-width: 1fr;
    --aside-width: auto;
    --banner-height: 0px;
    --header-height: 0px;
    --subheader-height: 0px;
    --scroll-margin-top: calc(
      var(--header-height, 0px) + var(--subheader-height, 0px) + 0.5em
    );

    --banner-top: var(--banner-height);
    --header-top: var(--header-height);
    --subheader-top: var(--subheader-height);
  }

  slot[name]:not(
    [name="skip-to-content"],
    [name="navigation-toggle"]
  )::slotted(*) {
    display: flex;
    background-color: var(--wa-color-surface-default);
  }

  ::slotted([slot="banner"]) {
    align-items: center;
    justify-content: center;
    gap: var(--wa-space-m);
    padding: var(--wa-space-xs) var(--wa-space-m);
  }

  ::slotted([slot="header"]) {
    align-items: center;
    justify-content: space-between;
    flex-wrap: wrap;
    gap: var(--wa-space-m);
    padding: var(--wa-space-m);
    flex: auto;
  }

  ::slotted([slot="subheader"]) {
    align-items: center;
    justify-content: space-between;
    flex-wrap: wrap;
    gap: var(--wa-space-m);
    padding: var(--wa-space-xs) var(--wa-space-m);
  }

  ::slotted([slot*="navigation"]),
  ::slotted([slot="menu"]),
  ::slotted([slot="aside"]) {
    flex-direction: column;
    gap: var(--wa-space-m);
    padding: var(--wa-space-m);
  }

  ::slotted([slot="main-header"]) {
    align-items: center;
    justify-content: space-between;
    flex-wrap: wrap;
    gap: var(--wa-space-m);
    padding: var(--wa-space-m) var(--wa-space-3xl);
  }

  slot:not([name]) {
    /* See #331 */
    &::slotted(main),
    &::slotted(section) {
      padding: var(--wa-space-3xl);
    }
  }

  ::slotted([slot="main-footer"]),
  ::slotted([slot="footer"]) {
    align-items: start;
    justify-content: space-between;
    flex-wrap: wrap;
    gap: var(--wa-space-m);
    padding: var(--wa-space-3xl);
  }

  :host([disable-sticky~="banner"]) {
    --banner-top: 0px;
  }
  :host([disable-sticky~="header"]) {
    --header-top: 0px;
  }
  :host([disable-sticky~="subheader"]) {
    --subheader-top: 0px;
  }

  /* Nothing else depends on subheader-height. */
  :host([disable-sticky~="subheader"]) {
  }
  :host([disable-sticky~="aside"]) [part~="aside"],
  :host([disable-sticky~="menu"]) [part~="menu"] {
    height: unset;
    max-height: unset;
  }

  :host([disable-sticky~="banner"]) [part~="banner"],
  :host([disable-sticky~="header"]) [part~="header"],
  :host([disable-sticky~="subheader"]) [part~="subheader"],
  :host([disable-sticky~="aside"]) [part~="aside"],
  :host([disable-sticky~="menu"]) [part~="menu"] {
    position: static;
    overflow: unset;
    z-index: unset;
  }

  :host([disable-sticky~="aside"]) [part~="aside"],
  :host([disable-sticky~="menu"]) [part~="menu"] {
    height: auto;
    max-height: auto;
  }

  [part~="base"] {
    min-height: 100dvh;
    display: grid;
    grid-template-rows: repeat(3, minmax(0, auto)) minmax(0, 1fr) minmax(
        0,
        auto
      );
    grid-template-columns: 100%;
    width: 100%;
    grid-template-areas:
      "banner"
      "header"
      "subheader"
      "body"
      "footer";
  }

  /* Grid areas */
  [part~="banner"] {
    grid-area: banner;
  }
  [part~="header"] {
    grid-area: header;
  }
  [part~="subheader"] {
    grid-area: subheader;
  }
  [part~="menu"] {
    grid-area: menu;
  }
  [part~="body"] {
    grid-area: body;
  }
  [part~="main"] {
    grid-area: main;
  }
  [part~="aside"] {
    grid-area: aside;
  }
  [part~="footer"] {
    grid-area: footer;
  }

  /* Z-indexes */
  [part~="banner"],
  [part~="header"],
  [part~="subheader"] {
    position: sticky;
    z-index: 5;
  }
  [part~="banner"] {
    top: 0px;
  }
  [part~="header"] {
    top: var(--banner-top);

    /** Make the header flex so that you don't unexpectedly have the default toggle button appearing above a slotted div because block elements are fun. */
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    justify-content: space-between;
  }
  [part~="subheader"] {
    top: calc(var(--header-top) + var(--banner-top));
  }
  [part~="body"] {
    display: grid;
    min-height: 100%;
    align-items: flex-start;
    grid-template-columns: minmax(0, var(--menu-width)) minmax(
        0,
        var(--main-width)
      ) minmax(0, var(--aside-width));
    grid-template-rows: minmax(0, 1fr);
    grid-template-areas: "menu main aside";
  }
  [part~="main"] {
    display: grid;
    min-height: 100%;
    grid-template-columns: minmax(0, 1fr);
    grid-template-rows: minmax(0, auto) minmax(0, 1fr) minmax(0, auto);
    grid-template-areas:
      "main-header"
      "main-content"
      "main-footer";
  }
  [part~="main-header"] {
    grid-area: main-header;
  }
  [part~="main-content"] {
    grid-area: main-content;
  }
  [part~="main-footer"] {
    grid-area: main-footer;
  }

  .skip-to-content {
    position: absolute;
    top: var(--wa-space-m);
    left: var(--wa-space-m);
    z-index: 6;
    border-radius: var(--wa-corners-1x);
    background-color: var(--wa-color-surface-default);
    color: var(--wa-color-text-link);
    text-decoration: none;
    padding: var(--wa-space-s) var(--wa-space-m);
    box-shadow: var(--wa-shadow-l);
    outline: var(--wa-focus-ring);
    outline-offset: var(--wa-focus-ring-offset);
  }

  [part~="menu"],
  [part~="aside"] {
    position: sticky;
    top: calc(var(--banner-top) + var(--header-top) + var(--subheader-top));
    z-index: 4;
    height: min(
      var(--main-height),
      calc(
        100dvh - var(--header-top) - var(--banner-top) - var(--subheader-top)
      )
    );
    max-height: min(
      var(--main-height),
      calc(
        100dvh - var(--header-top) - var(--banner-top) - var(--subheader-top)
      )
    );
    overflow: auto;
  }

  [part~="navigation"] {
    height: 100%;
    display: grid;
    grid-template-columns: minmax(0, 1fr);
    grid-template-rows: minmax(0, auto) minmax(0, 1fr) minmax(0, auto);
  }

  [part~="drawer"]::part(dialog) {
    background-color: var(--wa-color-surface-default);
  }

  /* Set these on the slot because we don't always control the navigation-toggle since that may be slotted. */
  slot[name~="navigation-toggle"],
  :host([disable-navigation-toggle]) slot[name~="navigation-toggle"] {
    display: none;
  }

  /* Sometimes the media query in the viewport is stubborn in iframes. This is an extra check to make it behave properly. */
  :host(:not([disable-navigation-toggle])[view="mobile"])
    slot[name~="navigation-toggle"] {
    display: contents;
  }

  [part~="navigation-toggle"] {
    /* Use only a margin-inline-start because the slotted header is expected to have default padding
        so it looks really awkward if this sets a margin-inline-end and the slotted header has a padding-inline-start. */
    margin-inline-start: var(--wa-space-m);
  }
`;

`````

### Expected (prettier)

`````ts
import { css } from "lit";

export default css`
  :host {
    display: block;
    background-color: var(--wa-color-surface-default);
    box-sizing: border-box;
    min-height: 100%;
    --menu-width: auto;
    --main-width: 1fr;
    --aside-width: auto;
    --banner-height: 0px;
    --header-height: 0px;
    --subheader-height: 0px;
    --scroll-margin-top: calc(
      var(--header-height, 0px) + var(--subheader-height, 0px) + 0.5em
    );

    --banner-top: var(--banner-height);
    --header-top: var(--header-height);
    --subheader-top: var(--subheader-height);
  }

  slot[name]:not([name="skip-to-content"], [name="navigation-toggle"])::slotted(
      *
    ) {
    display: flex;
    background-color: var(--wa-color-surface-default);
  }

  ::slotted([slot="banner"]) {
    align-items: center;
    justify-content: center;
    gap: var(--wa-space-m);
    padding: var(--wa-space-xs) var(--wa-space-m);
  }

  ::slotted([slot="header"]) {
    align-items: center;
    justify-content: space-between;
    flex-wrap: wrap;
    gap: var(--wa-space-m);
    padding: var(--wa-space-m);
    flex: auto;
  }

  ::slotted([slot="subheader"]) {
    align-items: center;
    justify-content: space-between;
    flex-wrap: wrap;
    gap: var(--wa-space-m);
    padding: var(--wa-space-xs) var(--wa-space-m);
  }

  ::slotted([slot*="navigation"]),
  ::slotted([slot="menu"]),
  ::slotted([slot="aside"]) {
    flex-direction: column;
    gap: var(--wa-space-m);
    padding: var(--wa-space-m);
  }

  ::slotted([slot="main-header"]) {
    align-items: center;
    justify-content: space-between;
    flex-wrap: wrap;
    gap: var(--wa-space-m);
    padding: var(--wa-space-m) var(--wa-space-3xl);
  }

  slot:not([name]) {
    /* See #331 */
    &::slotted(main),
    &::slotted(section) {
      padding: var(--wa-space-3xl);
    }
  }

  ::slotted([slot="main-footer"]),
  ::slotted([slot="footer"]) {
    align-items: start;
    justify-content: space-between;
    flex-wrap: wrap;
    gap: var(--wa-space-m);
    padding: var(--wa-space-3xl);
  }

  :host([disable-sticky~="banner"]) {
    --banner-top: 0px;
  }
  :host([disable-sticky~="header"]) {
    --header-top: 0px;
  }
  :host([disable-sticky~="subheader"]) {
    --subheader-top: 0px;
  }

  /* Nothing else depends on subheader-height. */
  :host([disable-sticky~="subheader"]) {
  }
  :host([disable-sticky~="aside"]) [part~="aside"],
  :host([disable-sticky~="menu"]) [part~="menu"] {
    height: unset;
    max-height: unset;
  }

  :host([disable-sticky~="banner"]) [part~="banner"],
  :host([disable-sticky~="header"]) [part~="header"],
  :host([disable-sticky~="subheader"]) [part~="subheader"],
  :host([disable-sticky~="aside"]) [part~="aside"],
  :host([disable-sticky~="menu"]) [part~="menu"] {
    position: static;
    overflow: unset;
    z-index: unset;
  }

  :host([disable-sticky~="aside"]) [part~="aside"],
  :host([disable-sticky~="menu"]) [part~="menu"] {
    height: auto;
    max-height: auto;
  }

  [part~="base"] {
    min-height: 100dvh;
    display: grid;
    grid-template-rows: repeat(3, minmax(0, auto)) minmax(0, 1fr) minmax(
        0,
        auto
      );
    grid-template-columns: 100%;
    width: 100%;
    grid-template-areas:
      "banner"
      "header"
      "subheader"
      "body"
      "footer";
  }

  /* Grid areas */
  [part~="banner"] {
    grid-area: banner;
  }
  [part~="header"] {
    grid-area: header;
  }
  [part~="subheader"] {
    grid-area: subheader;
  }
  [part~="menu"] {
    grid-area: menu;
  }
  [part~="body"] {
    grid-area: body;
  }
  [part~="main"] {
    grid-area: main;
  }
  [part~="aside"] {
    grid-area: aside;
  }
  [part~="footer"] {
    grid-area: footer;
  }

  /* Z-indexes */
  [part~="banner"],
  [part~="header"],
  [part~="subheader"] {
    position: sticky;
    z-index: 5;
  }
  [part~="banner"] {
    top: 0px;
  }
  [part~="header"] {
    top: var(--banner-top);

    /** Make the header flex so that you don't unexpectedly have the default toggle button appearing above a slotted div because block elements are fun. */
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    justify-content: space-between;
  }
  [part~="subheader"] {
    top: calc(var(--header-top) + var(--banner-top));
  }
  [part~="body"] {
    display: grid;
    min-height: 100%;
    align-items: flex-start;
    grid-template-columns: minmax(0, var(--menu-width)) minmax(
        0,
        var(--main-width)
      ) minmax(0, var(--aside-width));
    grid-template-rows: minmax(0, 1fr);
    grid-template-areas: "menu main aside";
  }
  [part~="main"] {
    display: grid;
    min-height: 100%;
    grid-template-columns: minmax(0, 1fr);
    grid-template-rows: minmax(0, auto) minmax(0, 1fr) minmax(0, auto);
    grid-template-areas:
      "main-header"
      "main-content"
      "main-footer";
  }
  [part~="main-header"] {
    grid-area: main-header;
  }
  [part~="main-content"] {
    grid-area: main-content;
  }
  [part~="main-footer"] {
    grid-area: main-footer;
  }

  .skip-to-content {
    position: absolute;
    top: var(--wa-space-m);
    left: var(--wa-space-m);
    z-index: 6;
    border-radius: var(--wa-corners-1x);
    background-color: var(--wa-color-surface-default);
    color: var(--wa-color-text-link);
    text-decoration: none;
    padding: var(--wa-space-s) var(--wa-space-m);
    box-shadow: var(--wa-shadow-l);
    outline: var(--wa-focus-ring);
    outline-offset: var(--wa-focus-ring-offset);
  }

  [part~="menu"],
  [part~="aside"] {
    position: sticky;
    top: calc(var(--banner-top) + var(--header-top) + var(--subheader-top));
    z-index: 4;
    height: min(
      var(--main-height),
      calc(
        100dvh - var(--header-top) - var(--banner-top) - var(--subheader-top)
      )
    );
    max-height: min(
      var(--main-height),
      calc(
        100dvh - var(--header-top) - var(--banner-top) - var(--subheader-top)
      )
    );
    overflow: auto;
  }

  [part~="navigation"] {
    height: 100%;
    display: grid;
    grid-template-columns: minmax(0, 1fr);
    grid-template-rows: minmax(0, auto) minmax(0, 1fr) minmax(0, auto);
  }

  [part~="drawer"]::part(dialog) {
    background-color: var(--wa-color-surface-default);
  }

  /* Set these on the slot because we don't always control the navigation-toggle since that may be slotted. */
  slot[name~="navigation-toggle"],
  :host([disable-navigation-toggle]) slot[name~="navigation-toggle"] {
    display: none;
  }

  /* Sometimes the media query in the viewport is stubborn in iframes. This is an extra check to make it behave properly. */
  :host(:not([disable-navigation-toggle])[view="mobile"])
    slot[name~="navigation-toggle"] {
    display: contents;
  }

  [part~="navigation-toggle"] {
    /* Use only a margin-inline-start because the slotted header is expected to have default padding
        so it looks really awkward if this sets a margin-inline-end and the slotted header has a padding-inline-start. */
    margin-inline-start: var(--wa-space-m);
  }
`;

`````
