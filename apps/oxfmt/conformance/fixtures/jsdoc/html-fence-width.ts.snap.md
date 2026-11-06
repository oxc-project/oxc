# Input

`````ts
/**
 * ```html
 * <section class="wrapper-xl" data-role="banner" id="hero"><span>hello world text</span></section>
 * ```
 *
 * ```html
 * <section class="wrapper" data-role="banner" id="hero"><span>hello world text</span></section>
 * ```
 */
`````

# Output

## `{"jsdoc":{}}`

`````ts
/**
 * ```html
 * <section class="wrapper-xl" data-role="banner" id="hero">
 *   <span>hello world text</span>
 * </section>
 * ```
 *
 * ```html
 * <section class="wrapper" data-role="banner" id="hero"><span>hello world text</span></section>
 * ```
 */

`````
