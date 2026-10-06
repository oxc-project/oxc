# externals/mdn-css-guides/custom_functions_and_mixins/using_custom_functions/index.md

## Option 1

`````json
{"printWidth":80,"bracketSameLine":true}
`````

### Diff

`````diff
===================================================================
--- prettier
+++ oxfmt
@@ -247,9 +247,9 @@
 We could rewrite the `--narrow-wide()` function to use an `if()` function instead:
 
 ```css
 @function --narrow-wide(--narrow, --wide) {
-  result: if(media(width < 700px): var(--narrow) ; else: var(--wide));
+  result: if(media(width < 700px): var(--narrow); else: var(--wide));
 }
 ```
 
 ## Writing complex syntax once, then reusing it

`````

### Actual (oxfmt)

`````md
---
title: Using CSS custom functions
slug: Web/CSS/Guides/Custom_functions_and_mixins/Using_custom_functions
page-type: guide
sidebar: cssref
---

CSS custom functions enable you to create reusable blocks of CSS code that can accept arguments, contain complex logic (defined using features such as CSS {{cssxref("if()")}} functions and {{cssxref("@media")}} at-rules), and return values based on that logic. They work similarly to [CSS custom properties](/en-US/docs/Web/CSS/Reference/Properties/--*), but provide more flexibility.

In this article, we'll show you how to use them and present some real-world examples.

## Function basics

A basic CSS custom function definition looks like this:

```css
@function --half-opacity() {
  result: 0.5;
}
```

After the `@function` syntax, we define a name for the function: `--half-opacity`. This must be a {{cssxref("&lt;dashed-ident>")}} type — it must start with a double-dash, and is case-sensitive. The function name is immediately followed by a set of parentheses (`()`) and a set of curly braces (`{}`).

> [!NOTE]
> If multiple CSS functions are given the same name, the function in the stronger cascade {{cssxref("@layer")}} wins. If all of them are in the same layer, the function defined last in the source order wins.

Inside the curly braces is the function **body**, which is where the function logic is defined. This can contain multiple declarations, including custom properties (which will be locally scoped to the function body), at-rules such as {{cssxref("@media")}}, and the [`result`](/en-US/docs/Web/CSS/Reference/At-rules/@function#result_2) descriptor. The value of the `result` descriptor is evaluated to determine the value returned by the function.

Here, we are setting `result` to the value `0.5`: the `--half-opacity()` function will always return `0.5`.

### Why "result" and not "return"?

The `result` descriptor sounds similar in functionality to the JavaScript function [`return`](/en-US/docs/Web/JavaScript/Reference/Statements/return) statement. However, `return` is not used in CSS functions. This is because, unlike JavaScript `return` statements, CSS functions do not return a value as soon as a `result` declaration is reached.

The body of a CSS function is evaluated from start to finish. If there are multiple `result` declarations included in the body, the last one in the source order overrides the earlier ones.

### Calling a CSS function

A CSS function can be called in place of any suitable property value using the {{cssxref("&lt;dashed-function>")}} syntax, which consists of the function name followed by parentheses containing the arguments to pass to the function, if any. For example, we can call our `--half-opacity()` function like this:

```css
h2 {
  opacity: --half-opacity();
}
```

Since this function always returns the value `0.5`, the previous declaration is the equivalent of `opacity: 0.5`. This isn't very useful. You might as well just use a custom property, or the literal value `0.5`.

Let's move on to look at how we can use CSS functions.

## Feature detecting CSS functions

One practical use of CSS functions with no parameters is in feature detection. In all of [the examples we'll be looking at](https://github.com/mdn/dom-examples/tree/main/css-custom-functions) in this article, we define a `--supports()` function that looks like this:

```css
@function --supports() {
  result: none;
}
```

You can then define a "feature not supported" banner, and set its {{cssxref("display")}} property to `--supports()`:

```html
<p class="support">
  ⚠️ Your browser doesn't currently support CSS custom functions.
</p>
```

```css
.support {
  /* ... */
  display: --supports();
}
```

In browsers that support custom functions, `display` will be set to `none`, and the support banner will be hidden. In non-supporting browsers, the `display: --supports()` declaration will be invalid and therefore ignored; therefore, the banner will be displayed.

## Specifying function parameters

CSS function parameters are specified as comma-separated custom properties inside the parentheses following the function name. For example:

```css
@function --transparent(--color, --alpha) {
  result: oklch(from var(--color) l c h / var(--alpha));
}
```

This function has a name of `--transparent` and takes two custom properties as parameters, `--color` and `--alpha`, which can be used locally inside the function body. The body contains a `result` descriptor, which uses [CSS relative color syntax](/en-US/docs/Web/CSS/Guides/Colors/Using_relative_colors) to convert the input `--color` value into an [`oklch()`](/en-US/docs/Web/CSS/Reference/Values/color_value/oklch) color with an alpha channel value as specified in the input `--alpha` value.

You can then call this function anywhere you want to produce a semi-transparent version of an existing color.

For example:

```css
section {
  --base-color: #faa6ff;
  background-color: --transparent(var(--base-color), 0.8);
}
```

## Specifying data types

It is possible to specify permitted data types for the function parameters and return value. When you don't specify these, the function will accept any type for these values.

Let's modify our previous function to provide data types:

```css
@function --transparent(--color type(<color>), --alpha type(<number>)) returns
  type(<color>) {
  result: oklch(from var(--color) l c h / var(--alpha));
}
```

The data type of each parameter is specified after the parameter name, and the data type of the `result` is specified just before the opening curly brace, preceded by the `returns` keyword. The {{cssxref("type()")}} function is used to specify a data type.

Note that, in cases where you are only specifying a single data type, you can omit the `type()` syntax and just write the type as a shorthand:

```css
@function --transparent(--color <color>, --alpha <number>) returns <color> {
  result: oklch(from var(--color) l c h / var(--alpha));
}
```

Now the function will only produce a valid value if the input arguments are a {{cssxref("&lt;color>")}} and a {{cssxref("&lt;number>")}}, respectively, and the `result` is a {{cssxref("&lt;color>")}}. If not, for example:

```css
section {
  --base-color: #faa6ff;
  background-color: --transparent(var(--base-color), 50%);
}
```

then the value will become invalid at computed-value time (since `50%` is not a `<number>` but a `<percentage>`) and the `background-color` will end up being set to `transparent`.

### Specifying multiple permitted types

You can specify multiple accepted data types using the `|` symbol as a separator, for example:

```css
@function --transparent(--color <color>, --alpha type(<number> | <percentage>))
  returns <color> {
  result: oklch(from var(--color) l c h / var(--alpha));
}
```

In such cases, the full `type()` syntax must be used.

With this adjustment, the `--transparent(var(--base-color), 50%)` function call is now valid.

## Specifying default values

You can also specify default values for parameters, after a colon at the end of their definition. For example:

```css
@function --transparent(--color <color>, --alpha <number>: 0.8) returns <color> {
  result: oklch(from var(--color) l c h / var(--alpha));
}
```

The `--alpha` parameter's default value is now `0.8`. If you want to use this value, you can omit the second argument when calling the function:

```css
section {
  --base-color: #faa6ff;
  background-color: --transparent(var(--base-color));
}
```

> [!NOTE]
> If an invalid value is passed in as a function argument and a default value is specified in that parameter definition, the invalid value will be ignored, and the default value will be used instead.

### Color adjust functions example

You can see the `--transparent()` function in action in our [color-adjust-functions](https://mdn.github.io/dom-examples/css-custom-functions/color-adjust-functions/) example (see the [source code](https://github.com/mdn/dom-examples/tree/main/css-custom-functions/color-adjust-functions)).

This example also includes functions called `--lighter()` and `--darker()` that work similarly to `--transparent()`, but return lighter and darker variants of a color, respectively:

```css
@function --transparent(--color <color>, --alpha <number>: 0.8) returns <color> {
  result: oklch(from var(--color) l c h / var(--alpha));
}

@function --lighter(--color <color>, --lightness-adjust <number>: 0.2) returns
  <color> {
  result: oklch(from var(--color) calc(l + var(--lightness-adjust)) c h);
}

@function --darker(--color <color>, --lightness-adjust <number>: 0.2) returns
  <color> {
  result: oklch(from var(--color) calc(l - var(--lightness-adjust)) c h);
}
```

A library of functions like these can become very useful for defining color schemes based on a single color:

```css
:root {
  --base-color: #faa6ff;
}

section {
  background-color: --transparent(var(--base-color));
  border: 3px solid --lighter(var(--base-color), 0.1);
  color: --darker(var(--base-color), 0.55);
}
```

## Including complex logic

You can include more complex logic in functions using constructs such as {{cssxref("@media")}} at-rules and {{cssxref("if()")}} functions.

Our [responsive-narrow-wide](https://mdn.github.io/dom-examples/css-custom-functions/responsive-narrow-wide/) example (see the [source code](https://github.com/mdn/dom-examples/tree/main/css-custom-functions/responsive-narrow-wide)) features a function called `--narrow-wide()`, which can be used to provide two value options for any property. One will be set if the viewport is below a specific breakpoint, and the other will be set if it is above.

The `--narrow-wide()` function accepts two parameters, `--narrow` and `--wide`. The returned `result` is the `--wide` property, unless the viewport is less than `700px` wide, in which case `--narrow` is returned.

```css
@function --narrow-wide(--narrow, --wide) {
  result: var(--wide);
  @media (width < 700px) {
    result: var(--narrow);
  }
}
```

This function can be used to provide responsive value options in multiple contexts:

```css
body {
  display: grid;
  grid-template-columns: repeat(--narrow-wide(1, 3), 1fr);
  gap: --narrow-wide(0, 20px);
  padding: 0 20px;
}

h2 {
  font-size: --narrow-wide(2.5rem, 2rem);
}

p {
  font-size: --narrow-wide(1.4rem, 1rem);
  line-height: 1.5;
}
```

### Using an `if()` function

We could rewrite the `--narrow-wide()` function to use an `if()` function instead:

```css
@function --narrow-wide(--narrow, --wide) {
  result: if(media(width < 700px): var(--narrow); else: var(--wide));
}
```

## Writing complex syntax once, then reusing it

A key use case for CSS functions is to define a complex section of syntax once, and be able to reuse it multiple times with a much simpler function call.

Our [gradient-function](https://mdn.github.io/dom-examples/css-custom-functions/gradient-function/) example (see the [source code](https://github.com/mdn/dom-examples/tree/main/css-custom-functions/gradient-function)) provides an example of this. It features a function called `--shippo-pattern()` that accepts length and color arguments, and returns a complex {{cssxref("background")}} value featuring multiple {{cssxref("radial-gradient()")}} backgrounds:

```css
@function --shippo-pattern(--size <length>, --tint <color>) {
  result:
    radial-gradient(closest-side, transparent 98%, rgb(0 0 0 / 0.3) 99%) 0 0 /
      var(--size) var(--size),
    radial-gradient(closest-side, transparent 98%, rgb(0 0 0 / 0.3) 99%)
      calc(var(--size) / 2) calc(var(--size) / 2) / var(--size) var(--size)
      var(--tint);
}
```

With this function defined, we can now create variants of this background value with different color tints and circle sizes:

```css
#one {
  background: --shippo-pattern(100px, #ddeeff);
}

#two {
  background: --shippo-pattern(3.5rem, lime);
}

#three {
  background: --shippo-pattern(10vw, purple);
}
```

## See also

- [CSS custom properties](/en-US/docs/Web/CSS/Reference/Properties/--*)
- [CSS custom functions and mixins](/en-US/docs/Web/CSS/Guides/Custom_functions_and_mixins) module
- [Custom CSS Functions in the Browser](https://www.oddbird.net/2025/04/11/custom-functions/) by Miriam Suzanne (2025)
- [CSS @function + CSS if()](https://www.bram.us/2025/02/18/css-at-function-and-css-if/) by Bramus (2025)
- [5 Useful CSS functions using the new @function rule](https://una.im/5-css-functions/) by Una Kravets (2025)

`````

### Expected (prettier)

`````md
---
title: Using CSS custom functions
slug: Web/CSS/Guides/Custom_functions_and_mixins/Using_custom_functions
page-type: guide
sidebar: cssref
---

CSS custom functions enable you to create reusable blocks of CSS code that can accept arguments, contain complex logic (defined using features such as CSS {{cssxref("if()")}} functions and {{cssxref("@media")}} at-rules), and return values based on that logic. They work similarly to [CSS custom properties](/en-US/docs/Web/CSS/Reference/Properties/--*), but provide more flexibility.

In this article, we'll show you how to use them and present some real-world examples.

## Function basics

A basic CSS custom function definition looks like this:

```css
@function --half-opacity() {
  result: 0.5;
}
```

After the `@function` syntax, we define a name for the function: `--half-opacity`. This must be a {{cssxref("&lt;dashed-ident>")}} type — it must start with a double-dash, and is case-sensitive. The function name is immediately followed by a set of parentheses (`()`) and a set of curly braces (`{}`).

> [!NOTE]
> If multiple CSS functions are given the same name, the function in the stronger cascade {{cssxref("@layer")}} wins. If all of them are in the same layer, the function defined last in the source order wins.

Inside the curly braces is the function **body**, which is where the function logic is defined. This can contain multiple declarations, including custom properties (which will be locally scoped to the function body), at-rules such as {{cssxref("@media")}}, and the [`result`](/en-US/docs/Web/CSS/Reference/At-rules/@function#result_2) descriptor. The value of the `result` descriptor is evaluated to determine the value returned by the function.

Here, we are setting `result` to the value `0.5`: the `--half-opacity()` function will always return `0.5`.

### Why "result" and not "return"?

The `result` descriptor sounds similar in functionality to the JavaScript function [`return`](/en-US/docs/Web/JavaScript/Reference/Statements/return) statement. However, `return` is not used in CSS functions. This is because, unlike JavaScript `return` statements, CSS functions do not return a value as soon as a `result` declaration is reached.

The body of a CSS function is evaluated from start to finish. If there are multiple `result` declarations included in the body, the last one in the source order overrides the earlier ones.

### Calling a CSS function

A CSS function can be called in place of any suitable property value using the {{cssxref("&lt;dashed-function>")}} syntax, which consists of the function name followed by parentheses containing the arguments to pass to the function, if any. For example, we can call our `--half-opacity()` function like this:

```css
h2 {
  opacity: --half-opacity();
}
```

Since this function always returns the value `0.5`, the previous declaration is the equivalent of `opacity: 0.5`. This isn't very useful. You might as well just use a custom property, or the literal value `0.5`.

Let's move on to look at how we can use CSS functions.

## Feature detecting CSS functions

One practical use of CSS functions with no parameters is in feature detection. In all of [the examples we'll be looking at](https://github.com/mdn/dom-examples/tree/main/css-custom-functions) in this article, we define a `--supports()` function that looks like this:

```css
@function --supports() {
  result: none;
}
```

You can then define a "feature not supported" banner, and set its {{cssxref("display")}} property to `--supports()`:

```html
<p class="support">
  ⚠️ Your browser doesn't currently support CSS custom functions.
</p>
```

```css
.support {
  /* ... */
  display: --supports();
}
```

In browsers that support custom functions, `display` will be set to `none`, and the support banner will be hidden. In non-supporting browsers, the `display: --supports()` declaration will be invalid and therefore ignored; therefore, the banner will be displayed.

## Specifying function parameters

CSS function parameters are specified as comma-separated custom properties inside the parentheses following the function name. For example:

```css
@function --transparent(--color, --alpha) {
  result: oklch(from var(--color) l c h / var(--alpha));
}
```

This function has a name of `--transparent` and takes two custom properties as parameters, `--color` and `--alpha`, which can be used locally inside the function body. The body contains a `result` descriptor, which uses [CSS relative color syntax](/en-US/docs/Web/CSS/Guides/Colors/Using_relative_colors) to convert the input `--color` value into an [`oklch()`](/en-US/docs/Web/CSS/Reference/Values/color_value/oklch) color with an alpha channel value as specified in the input `--alpha` value.

You can then call this function anywhere you want to produce a semi-transparent version of an existing color.

For example:

```css
section {
  --base-color: #faa6ff;
  background-color: --transparent(var(--base-color), 0.8);
}
```

## Specifying data types

It is possible to specify permitted data types for the function parameters and return value. When you don't specify these, the function will accept any type for these values.

Let's modify our previous function to provide data types:

```css
@function --transparent(--color type(<color>), --alpha type(<number>)) returns
  type(<color>) {
  result: oklch(from var(--color) l c h / var(--alpha));
}
```

The data type of each parameter is specified after the parameter name, and the data type of the `result` is specified just before the opening curly brace, preceded by the `returns` keyword. The {{cssxref("type()")}} function is used to specify a data type.

Note that, in cases where you are only specifying a single data type, you can omit the `type()` syntax and just write the type as a shorthand:

```css
@function --transparent(--color <color>, --alpha <number>) returns <color> {
  result: oklch(from var(--color) l c h / var(--alpha));
}
```

Now the function will only produce a valid value if the input arguments are a {{cssxref("&lt;color>")}} and a {{cssxref("&lt;number>")}}, respectively, and the `result` is a {{cssxref("&lt;color>")}}. If not, for example:

```css
section {
  --base-color: #faa6ff;
  background-color: --transparent(var(--base-color), 50%);
}
```

then the value will become invalid at computed-value time (since `50%` is not a `<number>` but a `<percentage>`) and the `background-color` will end up being set to `transparent`.

### Specifying multiple permitted types

You can specify multiple accepted data types using the `|` symbol as a separator, for example:

```css
@function --transparent(--color <color>, --alpha type(<number> | <percentage>))
  returns <color> {
  result: oklch(from var(--color) l c h / var(--alpha));
}
```

In such cases, the full `type()` syntax must be used.

With this adjustment, the `--transparent(var(--base-color), 50%)` function call is now valid.

## Specifying default values

You can also specify default values for parameters, after a colon at the end of their definition. For example:

```css
@function --transparent(--color <color>, --alpha <number>: 0.8) returns <color> {
  result: oklch(from var(--color) l c h / var(--alpha));
}
```

The `--alpha` parameter's default value is now `0.8`. If you want to use this value, you can omit the second argument when calling the function:

```css
section {
  --base-color: #faa6ff;
  background-color: --transparent(var(--base-color));
}
```

> [!NOTE]
> If an invalid value is passed in as a function argument and a default value is specified in that parameter definition, the invalid value will be ignored, and the default value will be used instead.

### Color adjust functions example

You can see the `--transparent()` function in action in our [color-adjust-functions](https://mdn.github.io/dom-examples/css-custom-functions/color-adjust-functions/) example (see the [source code](https://github.com/mdn/dom-examples/tree/main/css-custom-functions/color-adjust-functions)).

This example also includes functions called `--lighter()` and `--darker()` that work similarly to `--transparent()`, but return lighter and darker variants of a color, respectively:

```css
@function --transparent(--color <color>, --alpha <number>: 0.8) returns <color> {
  result: oklch(from var(--color) l c h / var(--alpha));
}

@function --lighter(--color <color>, --lightness-adjust <number>: 0.2) returns
  <color> {
  result: oklch(from var(--color) calc(l + var(--lightness-adjust)) c h);
}

@function --darker(--color <color>, --lightness-adjust <number>: 0.2) returns
  <color> {
  result: oklch(from var(--color) calc(l - var(--lightness-adjust)) c h);
}
```

A library of functions like these can become very useful for defining color schemes based on a single color:

```css
:root {
  --base-color: #faa6ff;
}

section {
  background-color: --transparent(var(--base-color));
  border: 3px solid --lighter(var(--base-color), 0.1);
  color: --darker(var(--base-color), 0.55);
}
```

## Including complex logic

You can include more complex logic in functions using constructs such as {{cssxref("@media")}} at-rules and {{cssxref("if()")}} functions.

Our [responsive-narrow-wide](https://mdn.github.io/dom-examples/css-custom-functions/responsive-narrow-wide/) example (see the [source code](https://github.com/mdn/dom-examples/tree/main/css-custom-functions/responsive-narrow-wide)) features a function called `--narrow-wide()`, which can be used to provide two value options for any property. One will be set if the viewport is below a specific breakpoint, and the other will be set if it is above.

The `--narrow-wide()` function accepts two parameters, `--narrow` and `--wide`. The returned `result` is the `--wide` property, unless the viewport is less than `700px` wide, in which case `--narrow` is returned.

```css
@function --narrow-wide(--narrow, --wide) {
  result: var(--wide);
  @media (width < 700px) {
    result: var(--narrow);
  }
}
```

This function can be used to provide responsive value options in multiple contexts:

```css
body {
  display: grid;
  grid-template-columns: repeat(--narrow-wide(1, 3), 1fr);
  gap: --narrow-wide(0, 20px);
  padding: 0 20px;
}

h2 {
  font-size: --narrow-wide(2.5rem, 2rem);
}

p {
  font-size: --narrow-wide(1.4rem, 1rem);
  line-height: 1.5;
}
```

### Using an `if()` function

We could rewrite the `--narrow-wide()` function to use an `if()` function instead:

```css
@function --narrow-wide(--narrow, --wide) {
  result: if(media(width < 700px): var(--narrow) ; else: var(--wide));
}
```

## Writing complex syntax once, then reusing it

A key use case for CSS functions is to define a complex section of syntax once, and be able to reuse it multiple times with a much simpler function call.

Our [gradient-function](https://mdn.github.io/dom-examples/css-custom-functions/gradient-function/) example (see the [source code](https://github.com/mdn/dom-examples/tree/main/css-custom-functions/gradient-function)) provides an example of this. It features a function called `--shippo-pattern()` that accepts length and color arguments, and returns a complex {{cssxref("background")}} value featuring multiple {{cssxref("radial-gradient()")}} backgrounds:

```css
@function --shippo-pattern(--size <length>, --tint <color>) {
  result:
    radial-gradient(closest-side, transparent 98%, rgb(0 0 0 / 0.3) 99%) 0 0 /
      var(--size) var(--size),
    radial-gradient(closest-side, transparent 98%, rgb(0 0 0 / 0.3) 99%)
      calc(var(--size) / 2) calc(var(--size) / 2) / var(--size) var(--size)
      var(--tint);
}
```

With this function defined, we can now create variants of this background value with different color tints and circle sizes:

```css
#one {
  background: --shippo-pattern(100px, #ddeeff);
}

#two {
  background: --shippo-pattern(3.5rem, lime);
}

#three {
  background: --shippo-pattern(10vw, purple);
}
```

## See also

- [CSS custom properties](/en-US/docs/Web/CSS/Reference/Properties/--*)
- [CSS custom functions and mixins](/en-US/docs/Web/CSS/Guides/Custom_functions_and_mixins) module
- [Custom CSS Functions in the Browser](https://www.oddbird.net/2025/04/11/custom-functions/) by Miriam Suzanne (2025)
- [CSS @function + CSS if()](https://www.bram.us/2025/02/18/css-at-function-and-css-if/) by Bramus (2025)
- [5 Useful CSS functions using the new @function rule](https://una.im/5-css-functions/) by Una Kravets (2025)

`````

## Option 2

`````json
{"printWidth":100}
`````

### Diff

`````diff
===================================================================
--- prettier
+++ oxfmt
@@ -241,9 +241,9 @@
 We could rewrite the `--narrow-wide()` function to use an `if()` function instead:
 
 ```css
 @function --narrow-wide(--narrow, --wide) {
-  result: if(media(width < 700px): var(--narrow) ; else: var(--wide));
+  result: if(media(width < 700px): var(--narrow); else: var(--wide));
 }
 ```
 
 ## Writing complex syntax once, then reusing it

`````

### Actual (oxfmt)

`````md
---
title: Using CSS custom functions
slug: Web/CSS/Guides/Custom_functions_and_mixins/Using_custom_functions
page-type: guide
sidebar: cssref
---

CSS custom functions enable you to create reusable blocks of CSS code that can accept arguments, contain complex logic (defined using features such as CSS {{cssxref("if()")}} functions and {{cssxref("@media")}} at-rules), and return values based on that logic. They work similarly to [CSS custom properties](/en-US/docs/Web/CSS/Reference/Properties/--*), but provide more flexibility.

In this article, we'll show you how to use them and present some real-world examples.

## Function basics

A basic CSS custom function definition looks like this:

```css
@function --half-opacity() {
  result: 0.5;
}
```

After the `@function` syntax, we define a name for the function: `--half-opacity`. This must be a {{cssxref("&lt;dashed-ident>")}} type — it must start with a double-dash, and is case-sensitive. The function name is immediately followed by a set of parentheses (`()`) and a set of curly braces (`{}`).

> [!NOTE]
> If multiple CSS functions are given the same name, the function in the stronger cascade {{cssxref("@layer")}} wins. If all of them are in the same layer, the function defined last in the source order wins.

Inside the curly braces is the function **body**, which is where the function logic is defined. This can contain multiple declarations, including custom properties (which will be locally scoped to the function body), at-rules such as {{cssxref("@media")}}, and the [`result`](/en-US/docs/Web/CSS/Reference/At-rules/@function#result_2) descriptor. The value of the `result` descriptor is evaluated to determine the value returned by the function.

Here, we are setting `result` to the value `0.5`: the `--half-opacity()` function will always return `0.5`.

### Why "result" and not "return"?

The `result` descriptor sounds similar in functionality to the JavaScript function [`return`](/en-US/docs/Web/JavaScript/Reference/Statements/return) statement. However, `return` is not used in CSS functions. This is because, unlike JavaScript `return` statements, CSS functions do not return a value as soon as a `result` declaration is reached.

The body of a CSS function is evaluated from start to finish. If there are multiple `result` declarations included in the body, the last one in the source order overrides the earlier ones.

### Calling a CSS function

A CSS function can be called in place of any suitable property value using the {{cssxref("&lt;dashed-function>")}} syntax, which consists of the function name followed by parentheses containing the arguments to pass to the function, if any. For example, we can call our `--half-opacity()` function like this:

```css
h2 {
  opacity: --half-opacity();
}
```

Since this function always returns the value `0.5`, the previous declaration is the equivalent of `opacity: 0.5`. This isn't very useful. You might as well just use a custom property, or the literal value `0.5`.

Let's move on to look at how we can use CSS functions.

## Feature detecting CSS functions

One practical use of CSS functions with no parameters is in feature detection. In all of [the examples we'll be looking at](https://github.com/mdn/dom-examples/tree/main/css-custom-functions) in this article, we define a `--supports()` function that looks like this:

```css
@function --supports() {
  result: none;
}
```

You can then define a "feature not supported" banner, and set its {{cssxref("display")}} property to `--supports()`:

```html
<p class="support">⚠️ Your browser doesn't currently support CSS custom functions.</p>
```

```css
.support {
  /* ... */
  display: --supports();
}
```

In browsers that support custom functions, `display` will be set to `none`, and the support banner will be hidden. In non-supporting browsers, the `display: --supports()` declaration will be invalid and therefore ignored; therefore, the banner will be displayed.

## Specifying function parameters

CSS function parameters are specified as comma-separated custom properties inside the parentheses following the function name. For example:

```css
@function --transparent(--color, --alpha) {
  result: oklch(from var(--color) l c h / var(--alpha));
}
```

This function has a name of `--transparent` and takes two custom properties as parameters, `--color` and `--alpha`, which can be used locally inside the function body. The body contains a `result` descriptor, which uses [CSS relative color syntax](/en-US/docs/Web/CSS/Guides/Colors/Using_relative_colors) to convert the input `--color` value into an [`oklch()`](/en-US/docs/Web/CSS/Reference/Values/color_value/oklch) color with an alpha channel value as specified in the input `--alpha` value.

You can then call this function anywhere you want to produce a semi-transparent version of an existing color.

For example:

```css
section {
  --base-color: #faa6ff;
  background-color: --transparent(var(--base-color), 0.8);
}
```

## Specifying data types

It is possible to specify permitted data types for the function parameters and return value. When you don't specify these, the function will accept any type for these values.

Let's modify our previous function to provide data types:

```css
@function --transparent(--color type(<color>), --alpha type(<number>)) returns type(<color>) {
  result: oklch(from var(--color) l c h / var(--alpha));
}
```

The data type of each parameter is specified after the parameter name, and the data type of the `result` is specified just before the opening curly brace, preceded by the `returns` keyword. The {{cssxref("type()")}} function is used to specify a data type.

Note that, in cases where you are only specifying a single data type, you can omit the `type()` syntax and just write the type as a shorthand:

```css
@function --transparent(--color <color>, --alpha <number>) returns <color> {
  result: oklch(from var(--color) l c h / var(--alpha));
}
```

Now the function will only produce a valid value if the input arguments are a {{cssxref("&lt;color>")}} and a {{cssxref("&lt;number>")}}, respectively, and the `result` is a {{cssxref("&lt;color>")}}. If not, for example:

```css
section {
  --base-color: #faa6ff;
  background-color: --transparent(var(--base-color), 50%);
}
```

then the value will become invalid at computed-value time (since `50%` is not a `<number>` but a `<percentage>`) and the `background-color` will end up being set to `transparent`.

### Specifying multiple permitted types

You can specify multiple accepted data types using the `|` symbol as a separator, for example:

```css
@function --transparent(--color <color>, --alpha type(<number> | <percentage>)) returns <color> {
  result: oklch(from var(--color) l c h / var(--alpha));
}
```

In such cases, the full `type()` syntax must be used.

With this adjustment, the `--transparent(var(--base-color), 50%)` function call is now valid.

## Specifying default values

You can also specify default values for parameters, after a colon at the end of their definition. For example:

```css
@function --transparent(--color <color>, --alpha <number>: 0.8) returns <color> {
  result: oklch(from var(--color) l c h / var(--alpha));
}
```

The `--alpha` parameter's default value is now `0.8`. If you want to use this value, you can omit the second argument when calling the function:

```css
section {
  --base-color: #faa6ff;
  background-color: --transparent(var(--base-color));
}
```

> [!NOTE]
> If an invalid value is passed in as a function argument and a default value is specified in that parameter definition, the invalid value will be ignored, and the default value will be used instead.

### Color adjust functions example

You can see the `--transparent()` function in action in our [color-adjust-functions](https://mdn.github.io/dom-examples/css-custom-functions/color-adjust-functions/) example (see the [source code](https://github.com/mdn/dom-examples/tree/main/css-custom-functions/color-adjust-functions)).

This example also includes functions called `--lighter()` and `--darker()` that work similarly to `--transparent()`, but return lighter and darker variants of a color, respectively:

```css
@function --transparent(--color <color>, --alpha <number>: 0.8) returns <color> {
  result: oklch(from var(--color) l c h / var(--alpha));
}

@function --lighter(--color <color>, --lightness-adjust <number>: 0.2) returns <color> {
  result: oklch(from var(--color) calc(l + var(--lightness-adjust)) c h);
}

@function --darker(--color <color>, --lightness-adjust <number>: 0.2) returns <color> {
  result: oklch(from var(--color) calc(l - var(--lightness-adjust)) c h);
}
```

A library of functions like these can become very useful for defining color schemes based on a single color:

```css
:root {
  --base-color: #faa6ff;
}

section {
  background-color: --transparent(var(--base-color));
  border: 3px solid --lighter(var(--base-color), 0.1);
  color: --darker(var(--base-color), 0.55);
}
```

## Including complex logic

You can include more complex logic in functions using constructs such as {{cssxref("@media")}} at-rules and {{cssxref("if()")}} functions.

Our [responsive-narrow-wide](https://mdn.github.io/dom-examples/css-custom-functions/responsive-narrow-wide/) example (see the [source code](https://github.com/mdn/dom-examples/tree/main/css-custom-functions/responsive-narrow-wide)) features a function called `--narrow-wide()`, which can be used to provide two value options for any property. One will be set if the viewport is below a specific breakpoint, and the other will be set if it is above.

The `--narrow-wide()` function accepts two parameters, `--narrow` and `--wide`. The returned `result` is the `--wide` property, unless the viewport is less than `700px` wide, in which case `--narrow` is returned.

```css
@function --narrow-wide(--narrow, --wide) {
  result: var(--wide);
  @media (width < 700px) {
    result: var(--narrow);
  }
}
```

This function can be used to provide responsive value options in multiple contexts:

```css
body {
  display: grid;
  grid-template-columns: repeat(--narrow-wide(1, 3), 1fr);
  gap: --narrow-wide(0, 20px);
  padding: 0 20px;
}

h2 {
  font-size: --narrow-wide(2.5rem, 2rem);
}

p {
  font-size: --narrow-wide(1.4rem, 1rem);
  line-height: 1.5;
}
```

### Using an `if()` function

We could rewrite the `--narrow-wide()` function to use an `if()` function instead:

```css
@function --narrow-wide(--narrow, --wide) {
  result: if(media(width < 700px): var(--narrow); else: var(--wide));
}
```

## Writing complex syntax once, then reusing it

A key use case for CSS functions is to define a complex section of syntax once, and be able to reuse it multiple times with a much simpler function call.

Our [gradient-function](https://mdn.github.io/dom-examples/css-custom-functions/gradient-function/) example (see the [source code](https://github.com/mdn/dom-examples/tree/main/css-custom-functions/gradient-function)) provides an example of this. It features a function called `--shippo-pattern()` that accepts length and color arguments, and returns a complex {{cssxref("background")}} value featuring multiple {{cssxref("radial-gradient()")}} backgrounds:

```css
@function --shippo-pattern(--size <length>, --tint <color>) {
  result:
    radial-gradient(closest-side, transparent 98%, rgb(0 0 0 / 0.3) 99%) 0 0 / var(--size)
      var(--size),
    radial-gradient(closest-side, transparent 98%, rgb(0 0 0 / 0.3) 99%) calc(var(--size) / 2)
      calc(var(--size) / 2) / var(--size) var(--size) var(--tint);
}
```

With this function defined, we can now create variants of this background value with different color tints and circle sizes:

```css
#one {
  background: --shippo-pattern(100px, #ddeeff);
}

#two {
  background: --shippo-pattern(3.5rem, lime);
}

#three {
  background: --shippo-pattern(10vw, purple);
}
```

## See also

- [CSS custom properties](/en-US/docs/Web/CSS/Reference/Properties/--*)
- [CSS custom functions and mixins](/en-US/docs/Web/CSS/Guides/Custom_functions_and_mixins) module
- [Custom CSS Functions in the Browser](https://www.oddbird.net/2025/04/11/custom-functions/) by Miriam Suzanne (2025)
- [CSS @function + CSS if()](https://www.bram.us/2025/02/18/css-at-function-and-css-if/) by Bramus (2025)
- [5 Useful CSS functions using the new @function rule](https://una.im/5-css-functions/) by Una Kravets (2025)

`````

### Expected (prettier)

`````md
---
title: Using CSS custom functions
slug: Web/CSS/Guides/Custom_functions_and_mixins/Using_custom_functions
page-type: guide
sidebar: cssref
---

CSS custom functions enable you to create reusable blocks of CSS code that can accept arguments, contain complex logic (defined using features such as CSS {{cssxref("if()")}} functions and {{cssxref("@media")}} at-rules), and return values based on that logic. They work similarly to [CSS custom properties](/en-US/docs/Web/CSS/Reference/Properties/--*), but provide more flexibility.

In this article, we'll show you how to use them and present some real-world examples.

## Function basics

A basic CSS custom function definition looks like this:

```css
@function --half-opacity() {
  result: 0.5;
}
```

After the `@function` syntax, we define a name for the function: `--half-opacity`. This must be a {{cssxref("&lt;dashed-ident>")}} type — it must start with a double-dash, and is case-sensitive. The function name is immediately followed by a set of parentheses (`()`) and a set of curly braces (`{}`).

> [!NOTE]
> If multiple CSS functions are given the same name, the function in the stronger cascade {{cssxref("@layer")}} wins. If all of them are in the same layer, the function defined last in the source order wins.

Inside the curly braces is the function **body**, which is where the function logic is defined. This can contain multiple declarations, including custom properties (which will be locally scoped to the function body), at-rules such as {{cssxref("@media")}}, and the [`result`](/en-US/docs/Web/CSS/Reference/At-rules/@function#result_2) descriptor. The value of the `result` descriptor is evaluated to determine the value returned by the function.

Here, we are setting `result` to the value `0.5`: the `--half-opacity()` function will always return `0.5`.

### Why "result" and not "return"?

The `result` descriptor sounds similar in functionality to the JavaScript function [`return`](/en-US/docs/Web/JavaScript/Reference/Statements/return) statement. However, `return` is not used in CSS functions. This is because, unlike JavaScript `return` statements, CSS functions do not return a value as soon as a `result` declaration is reached.

The body of a CSS function is evaluated from start to finish. If there are multiple `result` declarations included in the body, the last one in the source order overrides the earlier ones.

### Calling a CSS function

A CSS function can be called in place of any suitable property value using the {{cssxref("&lt;dashed-function>")}} syntax, which consists of the function name followed by parentheses containing the arguments to pass to the function, if any. For example, we can call our `--half-opacity()` function like this:

```css
h2 {
  opacity: --half-opacity();
}
```

Since this function always returns the value `0.5`, the previous declaration is the equivalent of `opacity: 0.5`. This isn't very useful. You might as well just use a custom property, or the literal value `0.5`.

Let's move on to look at how we can use CSS functions.

## Feature detecting CSS functions

One practical use of CSS functions with no parameters is in feature detection. In all of [the examples we'll be looking at](https://github.com/mdn/dom-examples/tree/main/css-custom-functions) in this article, we define a `--supports()` function that looks like this:

```css
@function --supports() {
  result: none;
}
```

You can then define a "feature not supported" banner, and set its {{cssxref("display")}} property to `--supports()`:

```html
<p class="support">⚠️ Your browser doesn't currently support CSS custom functions.</p>
```

```css
.support {
  /* ... */
  display: --supports();
}
```

In browsers that support custom functions, `display` will be set to `none`, and the support banner will be hidden. In non-supporting browsers, the `display: --supports()` declaration will be invalid and therefore ignored; therefore, the banner will be displayed.

## Specifying function parameters

CSS function parameters are specified as comma-separated custom properties inside the parentheses following the function name. For example:

```css
@function --transparent(--color, --alpha) {
  result: oklch(from var(--color) l c h / var(--alpha));
}
```

This function has a name of `--transparent` and takes two custom properties as parameters, `--color` and `--alpha`, which can be used locally inside the function body. The body contains a `result` descriptor, which uses [CSS relative color syntax](/en-US/docs/Web/CSS/Guides/Colors/Using_relative_colors) to convert the input `--color` value into an [`oklch()`](/en-US/docs/Web/CSS/Reference/Values/color_value/oklch) color with an alpha channel value as specified in the input `--alpha` value.

You can then call this function anywhere you want to produce a semi-transparent version of an existing color.

For example:

```css
section {
  --base-color: #faa6ff;
  background-color: --transparent(var(--base-color), 0.8);
}
```

## Specifying data types

It is possible to specify permitted data types for the function parameters and return value. When you don't specify these, the function will accept any type for these values.

Let's modify our previous function to provide data types:

```css
@function --transparent(--color type(<color>), --alpha type(<number>)) returns type(<color>) {
  result: oklch(from var(--color) l c h / var(--alpha));
}
```

The data type of each parameter is specified after the parameter name, and the data type of the `result` is specified just before the opening curly brace, preceded by the `returns` keyword. The {{cssxref("type()")}} function is used to specify a data type.

Note that, in cases where you are only specifying a single data type, you can omit the `type()` syntax and just write the type as a shorthand:

```css
@function --transparent(--color <color>, --alpha <number>) returns <color> {
  result: oklch(from var(--color) l c h / var(--alpha));
}
```

Now the function will only produce a valid value if the input arguments are a {{cssxref("&lt;color>")}} and a {{cssxref("&lt;number>")}}, respectively, and the `result` is a {{cssxref("&lt;color>")}}. If not, for example:

```css
section {
  --base-color: #faa6ff;
  background-color: --transparent(var(--base-color), 50%);
}
```

then the value will become invalid at computed-value time (since `50%` is not a `<number>` but a `<percentage>`) and the `background-color` will end up being set to `transparent`.

### Specifying multiple permitted types

You can specify multiple accepted data types using the `|` symbol as a separator, for example:

```css
@function --transparent(--color <color>, --alpha type(<number> | <percentage>)) returns <color> {
  result: oklch(from var(--color) l c h / var(--alpha));
}
```

In such cases, the full `type()` syntax must be used.

With this adjustment, the `--transparent(var(--base-color), 50%)` function call is now valid.

## Specifying default values

You can also specify default values for parameters, after a colon at the end of their definition. For example:

```css
@function --transparent(--color <color>, --alpha <number>: 0.8) returns <color> {
  result: oklch(from var(--color) l c h / var(--alpha));
}
```

The `--alpha` parameter's default value is now `0.8`. If you want to use this value, you can omit the second argument when calling the function:

```css
section {
  --base-color: #faa6ff;
  background-color: --transparent(var(--base-color));
}
```

> [!NOTE]
> If an invalid value is passed in as a function argument and a default value is specified in that parameter definition, the invalid value will be ignored, and the default value will be used instead.

### Color adjust functions example

You can see the `--transparent()` function in action in our [color-adjust-functions](https://mdn.github.io/dom-examples/css-custom-functions/color-adjust-functions/) example (see the [source code](https://github.com/mdn/dom-examples/tree/main/css-custom-functions/color-adjust-functions)).

This example also includes functions called `--lighter()` and `--darker()` that work similarly to `--transparent()`, but return lighter and darker variants of a color, respectively:

```css
@function --transparent(--color <color>, --alpha <number>: 0.8) returns <color> {
  result: oklch(from var(--color) l c h / var(--alpha));
}

@function --lighter(--color <color>, --lightness-adjust <number>: 0.2) returns <color> {
  result: oklch(from var(--color) calc(l + var(--lightness-adjust)) c h);
}

@function --darker(--color <color>, --lightness-adjust <number>: 0.2) returns <color> {
  result: oklch(from var(--color) calc(l - var(--lightness-adjust)) c h);
}
```

A library of functions like these can become very useful for defining color schemes based on a single color:

```css
:root {
  --base-color: #faa6ff;
}

section {
  background-color: --transparent(var(--base-color));
  border: 3px solid --lighter(var(--base-color), 0.1);
  color: --darker(var(--base-color), 0.55);
}
```

## Including complex logic

You can include more complex logic in functions using constructs such as {{cssxref("@media")}} at-rules and {{cssxref("if()")}} functions.

Our [responsive-narrow-wide](https://mdn.github.io/dom-examples/css-custom-functions/responsive-narrow-wide/) example (see the [source code](https://github.com/mdn/dom-examples/tree/main/css-custom-functions/responsive-narrow-wide)) features a function called `--narrow-wide()`, which can be used to provide two value options for any property. One will be set if the viewport is below a specific breakpoint, and the other will be set if it is above.

The `--narrow-wide()` function accepts two parameters, `--narrow` and `--wide`. The returned `result` is the `--wide` property, unless the viewport is less than `700px` wide, in which case `--narrow` is returned.

```css
@function --narrow-wide(--narrow, --wide) {
  result: var(--wide);
  @media (width < 700px) {
    result: var(--narrow);
  }
}
```

This function can be used to provide responsive value options in multiple contexts:

```css
body {
  display: grid;
  grid-template-columns: repeat(--narrow-wide(1, 3), 1fr);
  gap: --narrow-wide(0, 20px);
  padding: 0 20px;
}

h2 {
  font-size: --narrow-wide(2.5rem, 2rem);
}

p {
  font-size: --narrow-wide(1.4rem, 1rem);
  line-height: 1.5;
}
```

### Using an `if()` function

We could rewrite the `--narrow-wide()` function to use an `if()` function instead:

```css
@function --narrow-wide(--narrow, --wide) {
  result: if(media(width < 700px): var(--narrow) ; else: var(--wide));
}
```

## Writing complex syntax once, then reusing it

A key use case for CSS functions is to define a complex section of syntax once, and be able to reuse it multiple times with a much simpler function call.

Our [gradient-function](https://mdn.github.io/dom-examples/css-custom-functions/gradient-function/) example (see the [source code](https://github.com/mdn/dom-examples/tree/main/css-custom-functions/gradient-function)) provides an example of this. It features a function called `--shippo-pattern()` that accepts length and color arguments, and returns a complex {{cssxref("background")}} value featuring multiple {{cssxref("radial-gradient()")}} backgrounds:

```css
@function --shippo-pattern(--size <length>, --tint <color>) {
  result:
    radial-gradient(closest-side, transparent 98%, rgb(0 0 0 / 0.3) 99%) 0 0 / var(--size)
      var(--size),
    radial-gradient(closest-side, transparent 98%, rgb(0 0 0 / 0.3) 99%) calc(var(--size) / 2)
      calc(var(--size) / 2) / var(--size) var(--size) var(--tint);
}
```

With this function defined, we can now create variants of this background value with different color tints and circle sizes:

```css
#one {
  background: --shippo-pattern(100px, #ddeeff);
}

#two {
  background: --shippo-pattern(3.5rem, lime);
}

#three {
  background: --shippo-pattern(10vw, purple);
}
```

## See also

- [CSS custom properties](/en-US/docs/Web/CSS/Reference/Properties/--*)
- [CSS custom functions and mixins](/en-US/docs/Web/CSS/Guides/Custom_functions_and_mixins) module
- [Custom CSS Functions in the Browser](https://www.oddbird.net/2025/04/11/custom-functions/) by Miriam Suzanne (2025)
- [CSS @function + CSS if()](https://www.bram.us/2025/02/18/css-at-function-and-css-if/) by Bramus (2025)
- [5 Useful CSS functions using the new @function rule](https://una.im/5-css-functions/) by Una Kravets (2025)

`````
