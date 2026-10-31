# externals/mdn-learn/css_layout/positioning/index.md

## Option 1

`````json
{"printWidth":80,"bracketSameLine":true}
`````

### Diff

`````diff
===================================================================
--- prettier
+++ oxfmt
@@ -40,9 +40,9 @@
 
 We'd like you to do the following exercises on your local computer. To get started, create a new HTML file on your local system and add the following content to it:
 
 ```html
-<!DOCTYPE html>
+<!doctype html>
 <html lang="en-US">
   <head>
     <meta charset="utf-8" />
     <meta name="viewport" content="width=device-width" />

`````

### Actual (oxfmt)

`````md
---
title: Positioning
slug: Learn_web_development/Core/CSS_layout/Positioning
page-type: learn-module-chapter
sidebar: learnsidebar
---

{{PreviousMenuNext("Learn_web_development/Core/CSS_layout/Test_your_skills/Floats", "Learn_web_development/Core/CSS_layout/Test_your_skills/Position", "Learn_web_development/Core/CSS_layout")}}

Positioning allows you to take elements out of normal document flow and make them behave differently, for example, sitting on top of one another or always remaining in the same place inside the browser viewport. This article explains the different {{cssxref("position")}} values and how to use them.

<table>
  <tbody>
    <tr>
      <th scope="row">Prerequisites:</th>
      <td>
        <a href="/en-US/docs/Learn_web_development/Core/Structuring_content"
          >Structuring content with HTML</a
        >,
        <a href="/en-US/docs/Learn_web_development/Core/Styling_basics">CSS Styling basics</a>,
        <a href="/en-US/docs/Learn_web_development/Core/Text_styling/Fundamentals">Fundamental text and font styling</a>,
        familiarity with <a href="/en-US/docs/Learn_web_development/Core/CSS_layout/Introduction">CSS layout fundamental concepts</a>.
      </td>
    </tr>
    <tr>
      <th scope="row">Learning outcomes:</th>
      <td>
        <ul>
          <li><code>static</code> positioning is the default way elements are positioned on the page.</li>
          <li>Relatively positioned elements remain in the normal flow, but absolute (and fixed/sticky) positioning takes elements completely out of the normal flow to sit in a separate layer.</li>
          <li>Final layout position can be modified using the <code>top</code>, <code>bottom</code>, <code>left</code>, and <code>right</code> properties, but these have different effects depending on the set <code>position</code> value.</li>
          <li>Setting the positioning context of a positioned element by positioning an ancestor element.</li>
        </ul>
      </td>
    </tr>
  </tbody>
</table>

## Working through the exercises

We'd like you to do the following exercises on your local computer. To get started, create a new HTML file on your local system and add the following content to it:

```html
<!doctype html>
<html lang="en-US">
  <head>
    <meta charset="utf-8" />
    <meta name="viewport" content="width=device-width" />
    <title>Positioning example</title>

    <style>
      body {
        width: 500px;
        margin: 0 auto;
      }

      p {
        background: aqua;
        border: 3px solid blue;
        padding: 10px;
        margin: 10px;
      }

      span {
        background: red;
        border: 1px solid black;
      }
    </style>
  </head>
  <body>
    <h1>Basic document flow</h1>

    <p>
      I am a basic block level element. My adjacent block level elements sit on
      new lines below me.
    </p>

    <p>
      By default we span 100% of the width of our parent element, and our height
      is as tall as our child content. Our total width and height is our content
      + padding + border width/height.
    </p>

    <p>
      We are separated by our margins. Because of margin collapsing, we are
      separated by the width of one of our margins, not both.
    </p>

    <p>
      inline elements <span>like this one</span> and <span>this one</span> sit
      on the same line as one another, and adjacent text nodes, if there is
      space on the same line. Overflowing inline elements
      <span
        >wrap onto a new line if possible — like this one containing text</span
      >, or just go on to a new line if not, much like this image will do:
      <img
        src="https://mdn.github.io/shared-assets/images/examples/long.jpg"
        alt="a wide but short section of a photo of several fabrics" />
    </p>
  </body>
</html>
```

## Introducing positioning

Positioning allows us to produce interesting results by overriding normal document flow. What if you want to slightly alter the position of some boxes from their default flow position to give a slightly quirky, distressed feel? Positioning is your tool. Or what if you want to create a UI element that floats over the top of other parts of the page and/or always sits in the same place inside the browser window no matter how much the page is scrolled? Positioning makes such layout work possible.

There are a number of different types of positioning that you can put into effect on HTML elements. To make a specific type of positioning active on an element, we use the {{cssxref("position")}} property.

## Static positioning

Static positioning is the default that every element gets. It just means "put the element into its default position in the normal flow — nothing special to see here."

To see this (and get your example set up for future sections) first add a `class` of `positioned` to the second {{htmlelement("p")}} in the HTML:

```html
<p class="positioned">…</p>
```

Now add the following rule to the bottom of your CSS:

```html hidden live-sample___static
<h1>Static positioning</h1>

<p>
  I am a basic block level element. My adjacent block level elements sit on new
  lines below me.
</p>

<p class="positioned">
  By default we span 100% of the width of our parent element, and our are as
  tall as our child content. Our total width and height is our content + padding
  + border width/height.
</p>

<p>
  We are separated by our margins. Because of margin collapsing, we are
  separated by the width of one of our margins, not both.
</p>

<p>
  inline elements <span>like this one</span> and <span>this one</span> sit on
  the same line as one another, and adjacent text nodes, if there is space on
  the same line. Overflowing inline elements
  <span>wrap onto a new line if possible — like this one containing text</span>,
  or just go on to a new line if not, much like this image will do:
  <img
    src="https://mdn.github.io/shared-assets/images/examples/long.jpg"
    alt="a wide but short section of a photo of several fabrics" />
</p>
```

```css hidden live-sample___static live-sample___relative live-sample___absolute
body {
  width: 500px;
  margin: 0 auto;
}

p {
  background: aqua;
  border: 3px solid blue;
  padding: 10px;
  margin: 10px;
}

span {
  background: red;
  border: 1px solid black;
}
```

```css live-sample___static
.positioned {
  position: static;
  background: yellow;
}
```

If you save and refresh, you'll see no difference at all, except for the updated background color of the second paragraph. The example should look like so:

{{embedlivesample("static", "100%", 500)}}

This is fine — as we said before, static positioning is the default behavior!

## Relative positioning

Relative positioning is the first position type we'll take a look at. This is very similar to static positioning, except that once the positioned element has taken its place in the normal flow, you can then modify its final position, including making it overlap other elements on the page. Go ahead and update the `position` declaration in your code:

```css
.positioned {
  position: relative;
  background: yellow;
}
```

If you save and refresh at this stage, you won't see a change in the result at all. So how do you modify the element's position? You need to use the {{cssxref("top")}}, {{cssxref("bottom")}}, {{cssxref("left")}}, and {{cssxref("right")}} properties, which we'll explain in the next section.

### Introducing top, bottom, left, and right

{{cssxref("top")}}, {{cssxref("bottom")}}, {{cssxref("left")}}, and {{cssxref("right")}} are used alongside {{cssxref("position")}} to specify exactly where to move the positioned element to. To try this out, add the following declarations to the `.positioned` rule in your CSS:

```css live-sample___relative
.positioned {
  position: relative;
  background: yellow;
  top: 30px;
  left: 30px;
}
```

> [!NOTE]
> The values of these properties can take any [units](/en-US/docs/Learn_web_development/Core/Styling_basics/Values_and_units) you'd reasonably expect: pixels, mm, rems, %, etc.

If you now save and refresh, you'll get a result something like this:

```html hidden live-sample___relative
<h1>Relative positioning</h1>

<p>
  I am a basic block level element. My adjacent block level elements sit on new
  lines below me.
</p>

<p class="positioned">
  By default we span 100% of the width of our parent element, and we are as tall
  as our child content. Our total width and height is our content + padding +
  border width/height.
</p>

<p>
  We are separated by our margins. Because of margin collapsing, we are
  separated by the width of one of our margins, not both.
</p>

<p>
  Inline elements <span>like this one</span> and <span>this one</span> sit on
  the same line as one another, and adjacent text nodes, if there is space on
  the same line. Overflowing inline elements
  <span>wrap onto a new line if possible — like this one containing text</span>,
  or just go on to a new line if not, much like this image will do:
  <img
    src="https://mdn.github.io/shared-assets/images/examples/long.jpg"
    alt="snippet of cloth" />
</p>
```

{{embedlivesample("relative", "100%", 500)}}

Cool, huh? Ok, so this probably wasn't what you were expecting. Why has it moved to the bottom and to the right if we specified _top_ and _left_? This may seem counterintuitive. You need to think of it as if there's an invisible force that pushes the specified side of the positioned box, moving it in the opposite direction. So, for example, if you specify `top: 30px;`, it's as if a force will push the top of the box, causing it to move downwards by `30px`.

## Absolute positioning

Absolute positioning brings very different results.

### Setting position: absolute

Let's try changing the position declaration in your code as follows:

```css live-sample___absolute
.positioned {
  position: absolute;
  background: yellow;
  top: 30px;
  left: 30px;
}
```

If you now save and refresh, you should see something like so:

```html hidden live-sample___absolute
<h1>Absolute positioning</h1>

<p>
  I am a basic block level element. My adjacent block level elements sit on new
  lines below me.
</p>

<p class="positioned">
  By default we span 100% of the width of our parent element, and we are as tall
  as our child content. Our total width and height is our content + padding +
  border width/height.
</p>

<p>
  We are separated by our margins. Because of margin collapsing, we are
  separated by the width of one of our margins, not both.
</p>

<p>
  inline elements <span>like this one</span> and <span>this one</span> sit on
  the same line as one another, and adjacent text nodes, if there is space on
  the same line. Overflowing inline elements
  <span>wrap onto a new line if possible — like this one containing text</span>,
  or just go on to a new line if not, much like this image will do:
  <img
    src="https://mdn.github.io/shared-assets/images/examples/long.jpg"
    alt="snippet of cloth" />
</p>
```

{{embedlivesample("absolute", "100%", 420)}}

First of all, note that the gap where the positioned element should be in the document flow is no longer there — the first and third elements have closed together like it no longer exists! Well, in a way, this is true. An absolutely positioned element no longer exists in the normal document flow. Instead, it sits on its own layer separate from everything else. This is very useful: it means that we can create isolated UI features that don't interfere with the layout of other elements on the page. For example, popup information boxes, control menus, rollover panels, UI features that can be dragged and dropped anywhere on the page, and so on.

Second, notice that the position of the element has changed. This is because {{cssxref("top")}}, {{cssxref("bottom")}}, {{cssxref("left")}}, and {{cssxref("right")}} behave in a different way with absolute positioning. Rather than positioning the element based on its relative position within the normal document flow, they specify the distance the element should be from each of the containing element's sides. In this case, we are saying that the absolutely positioned element should sit 30px from the top of the **containing element** (the **initial containing block**, in this case, see below) and 30px from the left.

> [!NOTE]
> You can use {{cssxref("top")}}, {{cssxref("bottom")}}, {{cssxref("left")}}, and {{cssxref("right")}} to resize elements if you need to. Try setting `top: 0; bottom: 0; left: 0; right: 0;` and `margin: 0;` on your positioned elements and see what happens! Put it back again afterwards…

> [!NOTE]
> Yes, margins still affect positioned elements. Margin collapsing doesn't, however.

### Positioning contexts

Which element is the "containing element" of an absolutely positioned element? This is very much dependent on the `position` property value of the ancestors of the positioned element.

If no ancestor elements have their position property explicitly defined, then by default all ancestor elements will have a static position. The result of this is the absolutely positioned element will be contained in the **initial containing block**. The initial containing block has the dimensions of the viewport and is also the block that contains the {{htmlelement("html")}} element. In other words, the absolutely positioned element will be displayed outside of the {{htmlelement("html")}} element and be positioned relative to the initial viewport.

The positioned element is nested inside the {{htmlelement("body")}} in the HTML source, but in the final layout it's 30px away from the top and the left edges of the page.

We can change the **positioning context**, that is, which element the absolutely positioned element is positioned relative to. This is done by setting positioning on one of the element's ancestors (the elements it's nested inside; you can't position it relative to an element it isn't nested inside). To see this, update your `body` rule to set `position: relative` on it:

```css
body {
  width: 500px;
  margin: 0 auto;
  position: relative;
}
```

This should give the following result:

```html hidden live-sample___contexts
<h1>Positioning context</h1>

<p>
  I am a basic block level element. My adjacent block level elements sit on new
  lines below me.
</p>

<p class="positioned">
  Now I'm absolutely positioned relative to the
  <code>&lt;body&gt;</code> element, not the <code>&lt;html&gt;</code> element!
</p>

<p>
  We are separated by our margins. Because of margin collapsing, we are
  separated by the width of one of our margins, not both.
</p>

<p>
  inline elements <span>like this one</span> and <span>this one</span> sit on
  the same line as one another, and adjacent text nodes, if there is space on
  the same line. Overflowing inline elements
  <span>wrap onto a new line if possible — like this one containing text</span>,
  or just go on to a new line if not, much like this image will do:
  <img
    src="https://mdn.github.io/shared-assets/images/examples/long.jpg"
    alt="snippet of cloth" />
</p>
```

```css hidden live-sample___contexts live-sample___z-index
body {
  width: 500px;
  margin: 0 auto;
  position: relative;
}

p {
  background: aqua;
  border: 3px solid blue;
  padding: 10px;
  margin: 10px;
}

span {
  background: red;
  border: 1px solid black;
}

.positioned {
  position: absolute;
  background: yellow;
  top: 30px;
  left: 30px;
}
```

{{embedlivesample("contexts", "100%", 420)}}

The positioned element now sits relative to the {{htmlelement("body")}} element.

### Introducing z-index

All this absolute positioning is good fun, but there's another feature we haven't considered yet. When elements start to overlap, what determines which elements appear over others and which elements appear under others? In the example we've seen so far, we only have one positioned element in the positioning context, and it appears on the top since positioned elements win over non-positioned elements. What about when we have more than one?

Try adding the following to your CSS to make the first paragraph absolutely positioned too:

```css
p:nth-of-type(1) {
  position: absolute;
  background: lime;
  top: 10px;
  right: 30px;
}
```

At this point you'll see the first paragraph colored lime, moved out of the document flow, and positioned a bit above from where it originally was. It's also stacked below the original `.positioned` paragraph where the two overlap. This is because the `.positioned` paragraph is the second paragraph in the source order, and positioned elements later in the source order win over positioned elements earlier in the source order.

Can you change the stacking order? Yes, you can, by using the {{cssxref("z-index")}} property. "z-index" is a reference to the z-axis. You may recall from previous points in the course where we discussed web pages using horizontal (x-axis) and vertical (y-axis) coordinates to work out positioning for things like background images and drop shadow offsets. For languages that run left to right, (0,0) is at the top left of the page (or element), and the x- and y-axes run across to the right and down the page.

Web pages also have a z-axis: an imaginary line that runs from the surface of your screen towards your face (or whatever else you like to have in front of the screen). {{cssxref("z-index")}} values affect where positioned elements sit on that axis; positive values move them higher up the stack, negative values move them lower down the stack. By default, positioned elements all have a `z-index` of `auto`, which is effectively 0.

To change the stacking order, try adding the `z-index: 1` declaration to your `p:nth-of-type(1)` rule:

```css live-sample___z-index
p:nth-of-type(1) {
  position: absolute;
  background: lime;
  top: 10px;
  right: 30px;
  z-index: 1;
}
```

You should now see the lime paragraph on top:

```html hidden live-sample___z-index
<h1>z-index</h1>

<p>
  I am a basic block level element. My adjacent block level elements sit on new
  lines below me.
</p>

<p class="positioned">
  Now I'm absolutely positioned relative to the
  <code>&lt;body&gt;</code> element, not the <code>&lt;html&gt;</code> element!
</p>

<p>
  We are separated by our margins. Because of margin collapsing, we are
  separated by the width of one of our margins, not both.
</p>

<p>
  inline elements <span>like this one</span> and <span>this one</span> sit on
  the same line as one another, and adjacent text nodes, if there is space on
  the same line. Overflowing inline elements
  <span>wrap onto a new line if possible — like this one containing text</span>,
  or just go on to a new line if not, much like this image will do:
  <img
    src="https://mdn.github.io/shared-assets/images/examples/long.jpg"
    alt="snippet of cloth" />
</p>
```

{{embedlivesample("z-index", "100%", 350)}}

Note that `z-index` only accepts unitless index values; you can't specify that you want one element to be 23 pixels up the Z-axis — it doesn't work like that. Higher values will go above lower values and it's up to you what values you use. Using values of 2 and 3 will give the same effect as values of 300 and 40000.

## Fixed positioning

Let's now look at fixed positioning. This works in exactly the same way as absolute positioning, with one key difference: whereas absolute positioning fixes an element in place relative to its nearest positioned ancestor (the initial containing block if there isn't one), **fixed positioning** fixes an element in place relative to the visible portion of the viewport. This means that you can create useful UI items that are fixed in place, like persistent navigation menus that are always visible no matter how much the page scrolls.

Let's put together a simple example to show what we mean. First of all, delete the existing `p:nth-of-type(1)` and `.positioned` rules from your CSS.

Now update the `body` rule to remove the `position: relative;` declaration and add a fixed height, like so:

```css
body {
  width: 500px;
  height: 1400px;
  margin: 0 auto;
}
```

Now we're going to give the {{htmlelement("Heading_Elements", "&lt;h1>")}} element a `position: fixed;` declaration and position it at the top of the viewport. Add the following rule to your CSS:

```css
h1 {
  position: fixed;
  top: 0;
  width: 500px;
  margin-top: 0;
  background: white;
  padding: 10px;
}
```

The `top: 0;` is required to make it stick to the top of the screen. We give the heading the same width as the content column, and a white background and some padding and margin so the content won't be visible underneath it.

If you save and refresh, you'll see a fun little effect of the heading staying fixed — the content appears to scroll up and disappear underneath it. But notice how some of the content is initially clipped under the heading. This is because the positioned heading no longer appears in the document flow, so the rest of the content moves up to the top.

We can improve this by moving the paragraphs all down a bit. Set some top margin on the first paragraph, like so:

```css
p:nth-of-type(1) {
  margin-top: 60px;
}
```

You should now see the following example rendered:

```html hidden live-sample___fixed
<h1>Fixed positioning</h1>

<p>
  I am a basic block level element. My adjacent block level elements sit on new
  lines below me.
</p>

<p class="positioned">I'm not positioned any more.</p>

<p>
  We are separated by our margins. Because of margin collapsing, we are
  separated by the width of one of our margins, not both.
</p>

<p>
  Inline elements <span>like this one</span> and <span>this one</span> sit on
  the same line as one another, and adjacent text nodes, if there is space on
  the same line. Overflowing inline elements
  <span>wrap onto a new line if possible — like this one containing text</span>,
  or just go on to a new line if not, much like this image will do:
  <img
    src="https://mdn.github.io/shared-assets/images/examples/long.jpg"
    alt="snippet of cloth" />
</p>
```

```css hidden live-sample___fixed
body {
  width: 500px;
  height: 1400px;
  margin: 0 auto;
}

p {
  background: aqua;
  border: 3px solid blue;
  padding: 10px;
  margin: 10px;
}

span {
  background: red;
  border: 1px solid black;
}

h1 {
  position: fixed;
  top: 0px;
  width: 500px;
  margin-top: 0;
  background: white;
  padding: 10px;
}

p:nth-of-type(1) {
  margin-top: 60px;
}
```

{{ EmbedLiveSample('fixed', '100%', 400) }}

## Sticky positioning

There is another position value available called `position: sticky`, which is somewhat newer than the others. This is basically a hybrid between relative and fixed position. It allows a positioned element to act like it's relatively positioned until it scrolls to a certain threshold (e.g., 10px from the top of the viewport), after which it becomes fixed.

### Basic example

Sticky positioning can be used, for example, to cause a navigation bar to scroll with the page until a certain point and then stick to the top of the page.

```html hidden live-sample___basic-sticky
<h1>Sticky positioning</h1>

<p>
  Lorem ipsum dolor sit amet, consectetur adipiscing elit. Nulla luctus aliquam
  dolor, eu lacinia lorem placerat vulputate. Duis felis orci, pulvinar id metus
  ut, rutrum luctus orci. Cras porttitor imperdiet nunc, at ultricies tellus
  laoreet sit amet. Sed auctor cursus massa at porta. Integer ligula ipsum,
  tristique sit amet orci vel, viverra egestas ligula. Curabitur vehicula tellus
  neque, ac ornare ex malesuada et. In vitae convallis lacus. Aliquam erat
  volutpat. Suspendisse ac imperdiet turpis. Aenean finibus sollicitudin eros
  pharetra congue. Duis ornare egestas augue ut luctus. Proin blandit quam nec
  lacus varius commodo et a urna. Ut id ornare felis, eget fermentum sapien.
</p>

<div class="positioned">Sticky</div>

<p>
  Nam vulputate diam nec tempor bibendum. Donec luctus augue eget malesuada
  ultrices. Phasellus turpis est, posuere sit amet dapibus ut, facilisis sed
  est. Nam id risus quis ante semper consectetur eget aliquam lorem. Vivamus
  tristique elit dolor, sed pretium metus suscipit vel. Mauris ultricies lectus
  sed lobortis finibus. Vivamus eu urna eget velit cursus viverra quis
  vestibulum sem. Aliquam tincidunt eget purus in interdum. Cum sociis natoque
  penatibus et magnis dis parturient montes, nascetur ridiculus mus.
</p>

<p>
  Lorem ipsum dolor sit amet, consectetur adipiscing elit. Nulla luctus aliquam
  dolor, eu lacinia lorem placerat vulputate. Duis felis orci, pulvinar id metus
  ut, rutrum luctus orci. Cras porttitor imperdiet nunc, at ultricies tellus
  laoreet sit amet. Sed auctor cursus massa at porta. Integer ligula ipsum,
  tristique sit amet orci vel, viverra egestas ligula. Curabitur vehicula tellus
  neque, ac ornare ex malesuada et. In vitae convallis lacus. Aliquam erat
  volutpat. Suspendisse ac imperdiet turpis. Aenean finibus sollicitudin eros
  pharetra congue. Duis ornare egestas augue ut luctus. Proin blandit quam nec
  lacus varius commodo et a urna. Ut id ornare felis, eget fermentum sapien.
</p>
```

```css hidden live-sample___basic-sticky
body {
  width: 500px;
  margin: 0 auto;
}

.positioned {
  background: rgb(255 84 104 / 30%);
  border: 2px solid rgb(255 84 104);
  padding: 10px;
  margin: 10px;
  border-radius: 5px;
}
```

```css live-sample___basic-sticky
.positioned {
  position: sticky;
  top: 30px;
  left: 30px;
}
```

{{ EmbedLiveSample('basic-sticky', '100%', 200) }}

### Scrolling index

An interesting and common use of `position: sticky` is to create a scrolling index page where different headings stick to the top of the page as they reach it. The markup for such an example might look like so:

```html live-sample___sticky-scrolling-index
<h1>Sticky scrolling index</h1>

<dl>
  <dt>A</dt>
  <dd>Apple</dd>
  <dd>Ant</dd>
  <dd>Altimeter</dd>
  <dd>Airplane</dd>
  <dt>B</dt>
  <dd>Bird</dd>
  <dd>Buzzard</dd>
  <dd>Bee</dd>
  <dd>Banana</dd>
  <dd>Beanstalk</dd>
  <dt>C</dt>
  <dd>Calculator</dd>
  <dd>Cane</dd>
  <dd>Camera</dd>
  <dd>Camel</dd>
  <dt>D</dt>
  <dd>Duck</dd>
  <dd>Dime</dd>
  <dd>Dipstick</dd>
  <dd>Drone</dd>
  <dt>E</dt>
  <dd>Egg</dd>
  <dd>Elephant</dd>
  <dd>Egret</dd>
</dl>
```

The CSS would look as follows. In normal flow the {{htmlelement("dt")}} elements will scroll with the content. When we add `position: sticky` to the {{htmlelement("dt")}} element, along with a {{cssxref("top")}} value of `0`, the headings will stick to the top of the viewport as they reach it. Each subsequent header will then replace the previous one as it scrolls up to that position.

```css live-sample___sticky-scrolling-index
dt {
  background-color: black;
  color: white;
  padding: 10px;
  position: sticky;
  top: 0;
  left: 0;
  margin: 1em 0;
}
```

```css hidden live-sample___sticky-scrolling-index
body {
  width: 500px;
  height: 880px;
  margin: 0 auto;
}
```

{{ EmbedLiveSample('sticky-scrolling-index', '100%', 200) }}

Sticky elements are "sticky" relative to the nearest ancestor with a "scrolling mechanism", which is determined by its ancestors' [overflow](/en-US/docs/Web/CSS/Reference/Properties/overflow) property.

## Summary

I'm sure you had fun playing with basic positioning. While it's not an ideal method to use for entire layouts, there are many specific objectives it's suited for.

In the next article, we'll give you some tests that you can use to check how well you've understood and retained all this information.

## See also

- The {{cssxref("position")}} property reference.
- [Practical positioning examples](/en-US/docs/Learn_web_development/Core/CSS_layout/Practical_positioning_examples), for some more useful ideas.

{{PreviousMenuNext("Learn_web_development/Core/CSS_layout/Test_your_skills/Floats", "Learn_web_development/Core/CSS_layout/Test_your_skills/Position", "Learn_web_development/Core/CSS_layout")}}

`````

### Expected (prettier)

`````md
---
title: Positioning
slug: Learn_web_development/Core/CSS_layout/Positioning
page-type: learn-module-chapter
sidebar: learnsidebar
---

{{PreviousMenuNext("Learn_web_development/Core/CSS_layout/Test_your_skills/Floats", "Learn_web_development/Core/CSS_layout/Test_your_skills/Position", "Learn_web_development/Core/CSS_layout")}}

Positioning allows you to take elements out of normal document flow and make them behave differently, for example, sitting on top of one another or always remaining in the same place inside the browser viewport. This article explains the different {{cssxref("position")}} values and how to use them.

<table>
  <tbody>
    <tr>
      <th scope="row">Prerequisites:</th>
      <td>
        <a href="/en-US/docs/Learn_web_development/Core/Structuring_content"
          >Structuring content with HTML</a
        >,
        <a href="/en-US/docs/Learn_web_development/Core/Styling_basics">CSS Styling basics</a>,
        <a href="/en-US/docs/Learn_web_development/Core/Text_styling/Fundamentals">Fundamental text and font styling</a>,
        familiarity with <a href="/en-US/docs/Learn_web_development/Core/CSS_layout/Introduction">CSS layout fundamental concepts</a>.
      </td>
    </tr>
    <tr>
      <th scope="row">Learning outcomes:</th>
      <td>
        <ul>
          <li><code>static</code> positioning is the default way elements are positioned on the page.</li>
          <li>Relatively positioned elements remain in the normal flow, but absolute (and fixed/sticky) positioning takes elements completely out of the normal flow to sit in a separate layer.</li>
          <li>Final layout position can be modified using the <code>top</code>, <code>bottom</code>, <code>left</code>, and <code>right</code> properties, but these have different effects depending on the set <code>position</code> value.</li>
          <li>Setting the positioning context of a positioned element by positioning an ancestor element.</li>
        </ul>
      </td>
    </tr>
  </tbody>
</table>

## Working through the exercises

We'd like you to do the following exercises on your local computer. To get started, create a new HTML file on your local system and add the following content to it:

```html
<!DOCTYPE html>
<html lang="en-US">
  <head>
    <meta charset="utf-8" />
    <meta name="viewport" content="width=device-width" />
    <title>Positioning example</title>

    <style>
      body {
        width: 500px;
        margin: 0 auto;
      }

      p {
        background: aqua;
        border: 3px solid blue;
        padding: 10px;
        margin: 10px;
      }

      span {
        background: red;
        border: 1px solid black;
      }
    </style>
  </head>
  <body>
    <h1>Basic document flow</h1>

    <p>
      I am a basic block level element. My adjacent block level elements sit on
      new lines below me.
    </p>

    <p>
      By default we span 100% of the width of our parent element, and our height
      is as tall as our child content. Our total width and height is our content
      + padding + border width/height.
    </p>

    <p>
      We are separated by our margins. Because of margin collapsing, we are
      separated by the width of one of our margins, not both.
    </p>

    <p>
      inline elements <span>like this one</span> and <span>this one</span> sit
      on the same line as one another, and adjacent text nodes, if there is
      space on the same line. Overflowing inline elements
      <span
        >wrap onto a new line if possible — like this one containing text</span
      >, or just go on to a new line if not, much like this image will do:
      <img
        src="https://mdn.github.io/shared-assets/images/examples/long.jpg"
        alt="a wide but short section of a photo of several fabrics" />
    </p>
  </body>
</html>
```

## Introducing positioning

Positioning allows us to produce interesting results by overriding normal document flow. What if you want to slightly alter the position of some boxes from their default flow position to give a slightly quirky, distressed feel? Positioning is your tool. Or what if you want to create a UI element that floats over the top of other parts of the page and/or always sits in the same place inside the browser window no matter how much the page is scrolled? Positioning makes such layout work possible.

There are a number of different types of positioning that you can put into effect on HTML elements. To make a specific type of positioning active on an element, we use the {{cssxref("position")}} property.

## Static positioning

Static positioning is the default that every element gets. It just means "put the element into its default position in the normal flow — nothing special to see here."

To see this (and get your example set up for future sections) first add a `class` of `positioned` to the second {{htmlelement("p")}} in the HTML:

```html
<p class="positioned">…</p>
```

Now add the following rule to the bottom of your CSS:

```html hidden live-sample___static
<h1>Static positioning</h1>

<p>
  I am a basic block level element. My adjacent block level elements sit on new
  lines below me.
</p>

<p class="positioned">
  By default we span 100% of the width of our parent element, and our are as
  tall as our child content. Our total width and height is our content + padding
  + border width/height.
</p>

<p>
  We are separated by our margins. Because of margin collapsing, we are
  separated by the width of one of our margins, not both.
</p>

<p>
  inline elements <span>like this one</span> and <span>this one</span> sit on
  the same line as one another, and adjacent text nodes, if there is space on
  the same line. Overflowing inline elements
  <span>wrap onto a new line if possible — like this one containing text</span>,
  or just go on to a new line if not, much like this image will do:
  <img
    src="https://mdn.github.io/shared-assets/images/examples/long.jpg"
    alt="a wide but short section of a photo of several fabrics" />
</p>
```

```css hidden live-sample___static live-sample___relative live-sample___absolute
body {
  width: 500px;
  margin: 0 auto;
}

p {
  background: aqua;
  border: 3px solid blue;
  padding: 10px;
  margin: 10px;
}

span {
  background: red;
  border: 1px solid black;
}
```

```css live-sample___static
.positioned {
  position: static;
  background: yellow;
}
```

If you save and refresh, you'll see no difference at all, except for the updated background color of the second paragraph. The example should look like so:

{{embedlivesample("static", "100%", 500)}}

This is fine — as we said before, static positioning is the default behavior!

## Relative positioning

Relative positioning is the first position type we'll take a look at. This is very similar to static positioning, except that once the positioned element has taken its place in the normal flow, you can then modify its final position, including making it overlap other elements on the page. Go ahead and update the `position` declaration in your code:

```css
.positioned {
  position: relative;
  background: yellow;
}
```

If you save and refresh at this stage, you won't see a change in the result at all. So how do you modify the element's position? You need to use the {{cssxref("top")}}, {{cssxref("bottom")}}, {{cssxref("left")}}, and {{cssxref("right")}} properties, which we'll explain in the next section.

### Introducing top, bottom, left, and right

{{cssxref("top")}}, {{cssxref("bottom")}}, {{cssxref("left")}}, and {{cssxref("right")}} are used alongside {{cssxref("position")}} to specify exactly where to move the positioned element to. To try this out, add the following declarations to the `.positioned` rule in your CSS:

```css live-sample___relative
.positioned {
  position: relative;
  background: yellow;
  top: 30px;
  left: 30px;
}
```

> [!NOTE]
> The values of these properties can take any [units](/en-US/docs/Learn_web_development/Core/Styling_basics/Values_and_units) you'd reasonably expect: pixels, mm, rems, %, etc.

If you now save and refresh, you'll get a result something like this:

```html hidden live-sample___relative
<h1>Relative positioning</h1>

<p>
  I am a basic block level element. My adjacent block level elements sit on new
  lines below me.
</p>

<p class="positioned">
  By default we span 100% of the width of our parent element, and we are as tall
  as our child content. Our total width and height is our content + padding +
  border width/height.
</p>

<p>
  We are separated by our margins. Because of margin collapsing, we are
  separated by the width of one of our margins, not both.
</p>

<p>
  Inline elements <span>like this one</span> and <span>this one</span> sit on
  the same line as one another, and adjacent text nodes, if there is space on
  the same line. Overflowing inline elements
  <span>wrap onto a new line if possible — like this one containing text</span>,
  or just go on to a new line if not, much like this image will do:
  <img
    src="https://mdn.github.io/shared-assets/images/examples/long.jpg"
    alt="snippet of cloth" />
</p>
```

{{embedlivesample("relative", "100%", 500)}}

Cool, huh? Ok, so this probably wasn't what you were expecting. Why has it moved to the bottom and to the right if we specified _top_ and _left_? This may seem counterintuitive. You need to think of it as if there's an invisible force that pushes the specified side of the positioned box, moving it in the opposite direction. So, for example, if you specify `top: 30px;`, it's as if a force will push the top of the box, causing it to move downwards by `30px`.

## Absolute positioning

Absolute positioning brings very different results.

### Setting position: absolute

Let's try changing the position declaration in your code as follows:

```css live-sample___absolute
.positioned {
  position: absolute;
  background: yellow;
  top: 30px;
  left: 30px;
}
```

If you now save and refresh, you should see something like so:

```html hidden live-sample___absolute
<h1>Absolute positioning</h1>

<p>
  I am a basic block level element. My adjacent block level elements sit on new
  lines below me.
</p>

<p class="positioned">
  By default we span 100% of the width of our parent element, and we are as tall
  as our child content. Our total width and height is our content + padding +
  border width/height.
</p>

<p>
  We are separated by our margins. Because of margin collapsing, we are
  separated by the width of one of our margins, not both.
</p>

<p>
  inline elements <span>like this one</span> and <span>this one</span> sit on
  the same line as one another, and adjacent text nodes, if there is space on
  the same line. Overflowing inline elements
  <span>wrap onto a new line if possible — like this one containing text</span>,
  or just go on to a new line if not, much like this image will do:
  <img
    src="https://mdn.github.io/shared-assets/images/examples/long.jpg"
    alt="snippet of cloth" />
</p>
```

{{embedlivesample("absolute", "100%", 420)}}

First of all, note that the gap where the positioned element should be in the document flow is no longer there — the first and third elements have closed together like it no longer exists! Well, in a way, this is true. An absolutely positioned element no longer exists in the normal document flow. Instead, it sits on its own layer separate from everything else. This is very useful: it means that we can create isolated UI features that don't interfere with the layout of other elements on the page. For example, popup information boxes, control menus, rollover panels, UI features that can be dragged and dropped anywhere on the page, and so on.

Second, notice that the position of the element has changed. This is because {{cssxref("top")}}, {{cssxref("bottom")}}, {{cssxref("left")}}, and {{cssxref("right")}} behave in a different way with absolute positioning. Rather than positioning the element based on its relative position within the normal document flow, they specify the distance the element should be from each of the containing element's sides. In this case, we are saying that the absolutely positioned element should sit 30px from the top of the **containing element** (the **initial containing block**, in this case, see below) and 30px from the left.

> [!NOTE]
> You can use {{cssxref("top")}}, {{cssxref("bottom")}}, {{cssxref("left")}}, and {{cssxref("right")}} to resize elements if you need to. Try setting `top: 0; bottom: 0; left: 0; right: 0;` and `margin: 0;` on your positioned elements and see what happens! Put it back again afterwards…

> [!NOTE]
> Yes, margins still affect positioned elements. Margin collapsing doesn't, however.

### Positioning contexts

Which element is the "containing element" of an absolutely positioned element? This is very much dependent on the `position` property value of the ancestors of the positioned element.

If no ancestor elements have their position property explicitly defined, then by default all ancestor elements will have a static position. The result of this is the absolutely positioned element will be contained in the **initial containing block**. The initial containing block has the dimensions of the viewport and is also the block that contains the {{htmlelement("html")}} element. In other words, the absolutely positioned element will be displayed outside of the {{htmlelement("html")}} element and be positioned relative to the initial viewport.

The positioned element is nested inside the {{htmlelement("body")}} in the HTML source, but in the final layout it's 30px away from the top and the left edges of the page.

We can change the **positioning context**, that is, which element the absolutely positioned element is positioned relative to. This is done by setting positioning on one of the element's ancestors (the elements it's nested inside; you can't position it relative to an element it isn't nested inside). To see this, update your `body` rule to set `position: relative` on it:

```css
body {
  width: 500px;
  margin: 0 auto;
  position: relative;
}
```

This should give the following result:

```html hidden live-sample___contexts
<h1>Positioning context</h1>

<p>
  I am a basic block level element. My adjacent block level elements sit on new
  lines below me.
</p>

<p class="positioned">
  Now I'm absolutely positioned relative to the
  <code>&lt;body&gt;</code> element, not the <code>&lt;html&gt;</code> element!
</p>

<p>
  We are separated by our margins. Because of margin collapsing, we are
  separated by the width of one of our margins, not both.
</p>

<p>
  inline elements <span>like this one</span> and <span>this one</span> sit on
  the same line as one another, and adjacent text nodes, if there is space on
  the same line. Overflowing inline elements
  <span>wrap onto a new line if possible — like this one containing text</span>,
  or just go on to a new line if not, much like this image will do:
  <img
    src="https://mdn.github.io/shared-assets/images/examples/long.jpg"
    alt="snippet of cloth" />
</p>
```

```css hidden live-sample___contexts live-sample___z-index
body {
  width: 500px;
  margin: 0 auto;
  position: relative;
}

p {
  background: aqua;
  border: 3px solid blue;
  padding: 10px;
  margin: 10px;
}

span {
  background: red;
  border: 1px solid black;
}

.positioned {
  position: absolute;
  background: yellow;
  top: 30px;
  left: 30px;
}
```

{{embedlivesample("contexts", "100%", 420)}}

The positioned element now sits relative to the {{htmlelement("body")}} element.

### Introducing z-index

All this absolute positioning is good fun, but there's another feature we haven't considered yet. When elements start to overlap, what determines which elements appear over others and which elements appear under others? In the example we've seen so far, we only have one positioned element in the positioning context, and it appears on the top since positioned elements win over non-positioned elements. What about when we have more than one?

Try adding the following to your CSS to make the first paragraph absolutely positioned too:

```css
p:nth-of-type(1) {
  position: absolute;
  background: lime;
  top: 10px;
  right: 30px;
}
```

At this point you'll see the first paragraph colored lime, moved out of the document flow, and positioned a bit above from where it originally was. It's also stacked below the original `.positioned` paragraph where the two overlap. This is because the `.positioned` paragraph is the second paragraph in the source order, and positioned elements later in the source order win over positioned elements earlier in the source order.

Can you change the stacking order? Yes, you can, by using the {{cssxref("z-index")}} property. "z-index" is a reference to the z-axis. You may recall from previous points in the course where we discussed web pages using horizontal (x-axis) and vertical (y-axis) coordinates to work out positioning for things like background images and drop shadow offsets. For languages that run left to right, (0,0) is at the top left of the page (or element), and the x- and y-axes run across to the right and down the page.

Web pages also have a z-axis: an imaginary line that runs from the surface of your screen towards your face (or whatever else you like to have in front of the screen). {{cssxref("z-index")}} values affect where positioned elements sit on that axis; positive values move them higher up the stack, negative values move them lower down the stack. By default, positioned elements all have a `z-index` of `auto`, which is effectively 0.

To change the stacking order, try adding the `z-index: 1` declaration to your `p:nth-of-type(1)` rule:

```css live-sample___z-index
p:nth-of-type(1) {
  position: absolute;
  background: lime;
  top: 10px;
  right: 30px;
  z-index: 1;
}
```

You should now see the lime paragraph on top:

```html hidden live-sample___z-index
<h1>z-index</h1>

<p>
  I am a basic block level element. My adjacent block level elements sit on new
  lines below me.
</p>

<p class="positioned">
  Now I'm absolutely positioned relative to the
  <code>&lt;body&gt;</code> element, not the <code>&lt;html&gt;</code> element!
</p>

<p>
  We are separated by our margins. Because of margin collapsing, we are
  separated by the width of one of our margins, not both.
</p>

<p>
  inline elements <span>like this one</span> and <span>this one</span> sit on
  the same line as one another, and adjacent text nodes, if there is space on
  the same line. Overflowing inline elements
  <span>wrap onto a new line if possible — like this one containing text</span>,
  or just go on to a new line if not, much like this image will do:
  <img
    src="https://mdn.github.io/shared-assets/images/examples/long.jpg"
    alt="snippet of cloth" />
</p>
```

{{embedlivesample("z-index", "100%", 350)}}

Note that `z-index` only accepts unitless index values; you can't specify that you want one element to be 23 pixels up the Z-axis — it doesn't work like that. Higher values will go above lower values and it's up to you what values you use. Using values of 2 and 3 will give the same effect as values of 300 and 40000.

## Fixed positioning

Let's now look at fixed positioning. This works in exactly the same way as absolute positioning, with one key difference: whereas absolute positioning fixes an element in place relative to its nearest positioned ancestor (the initial containing block if there isn't one), **fixed positioning** fixes an element in place relative to the visible portion of the viewport. This means that you can create useful UI items that are fixed in place, like persistent navigation menus that are always visible no matter how much the page scrolls.

Let's put together a simple example to show what we mean. First of all, delete the existing `p:nth-of-type(1)` and `.positioned` rules from your CSS.

Now update the `body` rule to remove the `position: relative;` declaration and add a fixed height, like so:

```css
body {
  width: 500px;
  height: 1400px;
  margin: 0 auto;
}
```

Now we're going to give the {{htmlelement("Heading_Elements", "&lt;h1>")}} element a `position: fixed;` declaration and position it at the top of the viewport. Add the following rule to your CSS:

```css
h1 {
  position: fixed;
  top: 0;
  width: 500px;
  margin-top: 0;
  background: white;
  padding: 10px;
}
```

The `top: 0;` is required to make it stick to the top of the screen. We give the heading the same width as the content column, and a white background and some padding and margin so the content won't be visible underneath it.

If you save and refresh, you'll see a fun little effect of the heading staying fixed — the content appears to scroll up and disappear underneath it. But notice how some of the content is initially clipped under the heading. This is because the positioned heading no longer appears in the document flow, so the rest of the content moves up to the top.

We can improve this by moving the paragraphs all down a bit. Set some top margin on the first paragraph, like so:

```css
p:nth-of-type(1) {
  margin-top: 60px;
}
```

You should now see the following example rendered:

```html hidden live-sample___fixed
<h1>Fixed positioning</h1>

<p>
  I am a basic block level element. My adjacent block level elements sit on new
  lines below me.
</p>

<p class="positioned">I'm not positioned any more.</p>

<p>
  We are separated by our margins. Because of margin collapsing, we are
  separated by the width of one of our margins, not both.
</p>

<p>
  Inline elements <span>like this one</span> and <span>this one</span> sit on
  the same line as one another, and adjacent text nodes, if there is space on
  the same line. Overflowing inline elements
  <span>wrap onto a new line if possible — like this one containing text</span>,
  or just go on to a new line if not, much like this image will do:
  <img
    src="https://mdn.github.io/shared-assets/images/examples/long.jpg"
    alt="snippet of cloth" />
</p>
```

```css hidden live-sample___fixed
body {
  width: 500px;
  height: 1400px;
  margin: 0 auto;
}

p {
  background: aqua;
  border: 3px solid blue;
  padding: 10px;
  margin: 10px;
}

span {
  background: red;
  border: 1px solid black;
}

h1 {
  position: fixed;
  top: 0px;
  width: 500px;
  margin-top: 0;
  background: white;
  padding: 10px;
}

p:nth-of-type(1) {
  margin-top: 60px;
}
```

{{ EmbedLiveSample('fixed', '100%', 400) }}

## Sticky positioning

There is another position value available called `position: sticky`, which is somewhat newer than the others. This is basically a hybrid between relative and fixed position. It allows a positioned element to act like it's relatively positioned until it scrolls to a certain threshold (e.g., 10px from the top of the viewport), after which it becomes fixed.

### Basic example

Sticky positioning can be used, for example, to cause a navigation bar to scroll with the page until a certain point and then stick to the top of the page.

```html hidden live-sample___basic-sticky
<h1>Sticky positioning</h1>

<p>
  Lorem ipsum dolor sit amet, consectetur adipiscing elit. Nulla luctus aliquam
  dolor, eu lacinia lorem placerat vulputate. Duis felis orci, pulvinar id metus
  ut, rutrum luctus orci. Cras porttitor imperdiet nunc, at ultricies tellus
  laoreet sit amet. Sed auctor cursus massa at porta. Integer ligula ipsum,
  tristique sit amet orci vel, viverra egestas ligula. Curabitur vehicula tellus
  neque, ac ornare ex malesuada et. In vitae convallis lacus. Aliquam erat
  volutpat. Suspendisse ac imperdiet turpis. Aenean finibus sollicitudin eros
  pharetra congue. Duis ornare egestas augue ut luctus. Proin blandit quam nec
  lacus varius commodo et a urna. Ut id ornare felis, eget fermentum sapien.
</p>

<div class="positioned">Sticky</div>

<p>
  Nam vulputate diam nec tempor bibendum. Donec luctus augue eget malesuada
  ultrices. Phasellus turpis est, posuere sit amet dapibus ut, facilisis sed
  est. Nam id risus quis ante semper consectetur eget aliquam lorem. Vivamus
  tristique elit dolor, sed pretium metus suscipit vel. Mauris ultricies lectus
  sed lobortis finibus. Vivamus eu urna eget velit cursus viverra quis
  vestibulum sem. Aliquam tincidunt eget purus in interdum. Cum sociis natoque
  penatibus et magnis dis parturient montes, nascetur ridiculus mus.
</p>

<p>
  Lorem ipsum dolor sit amet, consectetur adipiscing elit. Nulla luctus aliquam
  dolor, eu lacinia lorem placerat vulputate. Duis felis orci, pulvinar id metus
  ut, rutrum luctus orci. Cras porttitor imperdiet nunc, at ultricies tellus
  laoreet sit amet. Sed auctor cursus massa at porta. Integer ligula ipsum,
  tristique sit amet orci vel, viverra egestas ligula. Curabitur vehicula tellus
  neque, ac ornare ex malesuada et. In vitae convallis lacus. Aliquam erat
  volutpat. Suspendisse ac imperdiet turpis. Aenean finibus sollicitudin eros
  pharetra congue. Duis ornare egestas augue ut luctus. Proin blandit quam nec
  lacus varius commodo et a urna. Ut id ornare felis, eget fermentum sapien.
</p>
```

```css hidden live-sample___basic-sticky
body {
  width: 500px;
  margin: 0 auto;
}

.positioned {
  background: rgb(255 84 104 / 30%);
  border: 2px solid rgb(255 84 104);
  padding: 10px;
  margin: 10px;
  border-radius: 5px;
}
```

```css live-sample___basic-sticky
.positioned {
  position: sticky;
  top: 30px;
  left: 30px;
}
```

{{ EmbedLiveSample('basic-sticky', '100%', 200) }}

### Scrolling index

An interesting and common use of `position: sticky` is to create a scrolling index page where different headings stick to the top of the page as they reach it. The markup for such an example might look like so:

```html live-sample___sticky-scrolling-index
<h1>Sticky scrolling index</h1>

<dl>
  <dt>A</dt>
  <dd>Apple</dd>
  <dd>Ant</dd>
  <dd>Altimeter</dd>
  <dd>Airplane</dd>
  <dt>B</dt>
  <dd>Bird</dd>
  <dd>Buzzard</dd>
  <dd>Bee</dd>
  <dd>Banana</dd>
  <dd>Beanstalk</dd>
  <dt>C</dt>
  <dd>Calculator</dd>
  <dd>Cane</dd>
  <dd>Camera</dd>
  <dd>Camel</dd>
  <dt>D</dt>
  <dd>Duck</dd>
  <dd>Dime</dd>
  <dd>Dipstick</dd>
  <dd>Drone</dd>
  <dt>E</dt>
  <dd>Egg</dd>
  <dd>Elephant</dd>
  <dd>Egret</dd>
</dl>
```

The CSS would look as follows. In normal flow the {{htmlelement("dt")}} elements will scroll with the content. When we add `position: sticky` to the {{htmlelement("dt")}} element, along with a {{cssxref("top")}} value of `0`, the headings will stick to the top of the viewport as they reach it. Each subsequent header will then replace the previous one as it scrolls up to that position.

```css live-sample___sticky-scrolling-index
dt {
  background-color: black;
  color: white;
  padding: 10px;
  position: sticky;
  top: 0;
  left: 0;
  margin: 1em 0;
}
```

```css hidden live-sample___sticky-scrolling-index
body {
  width: 500px;
  height: 880px;
  margin: 0 auto;
}
```

{{ EmbedLiveSample('sticky-scrolling-index', '100%', 200) }}

Sticky elements are "sticky" relative to the nearest ancestor with a "scrolling mechanism", which is determined by its ancestors' [overflow](/en-US/docs/Web/CSS/Reference/Properties/overflow) property.

## Summary

I'm sure you had fun playing with basic positioning. While it's not an ideal method to use for entire layouts, there are many specific objectives it's suited for.

In the next article, we'll give you some tests that you can use to check how well you've understood and retained all this information.

## See also

- The {{cssxref("position")}} property reference.
- [Practical positioning examples](/en-US/docs/Learn_web_development/Core/CSS_layout/Practical_positioning_examples), for some more useful ideas.

{{PreviousMenuNext("Learn_web_development/Core/CSS_layout/Test_your_skills/Floats", "Learn_web_development/Core/CSS_layout/Test_your_skills/Position", "Learn_web_development/Core/CSS_layout")}}

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
@@ -40,9 +40,9 @@
 
 We'd like you to do the following exercises on your local computer. To get started, create a new HTML file on your local system and add the following content to it:
 
 ```html
-<!DOCTYPE html>
+<!doctype html>
 <html lang="en-US">
   <head>
     <meta charset="utf-8" />
     <meta name="viewport" content="width=device-width" />

`````

### Actual (oxfmt)

`````md
---
title: Positioning
slug: Learn_web_development/Core/CSS_layout/Positioning
page-type: learn-module-chapter
sidebar: learnsidebar
---

{{PreviousMenuNext("Learn_web_development/Core/CSS_layout/Test_your_skills/Floats", "Learn_web_development/Core/CSS_layout/Test_your_skills/Position", "Learn_web_development/Core/CSS_layout")}}

Positioning allows you to take elements out of normal document flow and make them behave differently, for example, sitting on top of one another or always remaining in the same place inside the browser viewport. This article explains the different {{cssxref("position")}} values and how to use them.

<table>
  <tbody>
    <tr>
      <th scope="row">Prerequisites:</th>
      <td>
        <a href="/en-US/docs/Learn_web_development/Core/Structuring_content"
          >Structuring content with HTML</a
        >,
        <a href="/en-US/docs/Learn_web_development/Core/Styling_basics">CSS Styling basics</a>,
        <a href="/en-US/docs/Learn_web_development/Core/Text_styling/Fundamentals">Fundamental text and font styling</a>,
        familiarity with <a href="/en-US/docs/Learn_web_development/Core/CSS_layout/Introduction">CSS layout fundamental concepts</a>.
      </td>
    </tr>
    <tr>
      <th scope="row">Learning outcomes:</th>
      <td>
        <ul>
          <li><code>static</code> positioning is the default way elements are positioned on the page.</li>
          <li>Relatively positioned elements remain in the normal flow, but absolute (and fixed/sticky) positioning takes elements completely out of the normal flow to sit in a separate layer.</li>
          <li>Final layout position can be modified using the <code>top</code>, <code>bottom</code>, <code>left</code>, and <code>right</code> properties, but these have different effects depending on the set <code>position</code> value.</li>
          <li>Setting the positioning context of a positioned element by positioning an ancestor element.</li>
        </ul>
      </td>
    </tr>
  </tbody>
</table>

## Working through the exercises

We'd like you to do the following exercises on your local computer. To get started, create a new HTML file on your local system and add the following content to it:

```html
<!doctype html>
<html lang="en-US">
  <head>
    <meta charset="utf-8" />
    <meta name="viewport" content="width=device-width" />
    <title>Positioning example</title>

    <style>
      body {
        width: 500px;
        margin: 0 auto;
      }

      p {
        background: aqua;
        border: 3px solid blue;
        padding: 10px;
        margin: 10px;
      }

      span {
        background: red;
        border: 1px solid black;
      }
    </style>
  </head>
  <body>
    <h1>Basic document flow</h1>

    <p>
      I am a basic block level element. My adjacent block level elements sit on new lines below me.
    </p>

    <p>
      By default we span 100% of the width of our parent element, and our height is as tall as our
      child content. Our total width and height is our content + padding + border width/height.
    </p>

    <p>
      We are separated by our margins. Because of margin collapsing, we are separated by the width
      of one of our margins, not both.
    </p>

    <p>
      inline elements <span>like this one</span> and <span>this one</span> sit on the same line as
      one another, and adjacent text nodes, if there is space on the same line. Overflowing inline
      elements <span>wrap onto a new line if possible — like this one containing text</span>, or
      just go on to a new line if not, much like this image will do:
      <img
        src="https://mdn.github.io/shared-assets/images/examples/long.jpg"
        alt="a wide but short section of a photo of several fabrics"
      />
    </p>
  </body>
</html>
```

## Introducing positioning

Positioning allows us to produce interesting results by overriding normal document flow. What if you want to slightly alter the position of some boxes from their default flow position to give a slightly quirky, distressed feel? Positioning is your tool. Or what if you want to create a UI element that floats over the top of other parts of the page and/or always sits in the same place inside the browser window no matter how much the page is scrolled? Positioning makes such layout work possible.

There are a number of different types of positioning that you can put into effect on HTML elements. To make a specific type of positioning active on an element, we use the {{cssxref("position")}} property.

## Static positioning

Static positioning is the default that every element gets. It just means "put the element into its default position in the normal flow — nothing special to see here."

To see this (and get your example set up for future sections) first add a `class` of `positioned` to the second {{htmlelement("p")}} in the HTML:

```html
<p class="positioned">…</p>
```

Now add the following rule to the bottom of your CSS:

```html hidden live-sample___static
<h1>Static positioning</h1>

<p>I am a basic block level element. My adjacent block level elements sit on new lines below me.</p>

<p class="positioned">
  By default we span 100% of the width of our parent element, and our are as tall as our child
  content. Our total width and height is our content + padding + border width/height.
</p>

<p>
  We are separated by our margins. Because of margin collapsing, we are separated by the width of
  one of our margins, not both.
</p>

<p>
  inline elements <span>like this one</span> and <span>this one</span> sit on the same line as one
  another, and adjacent text nodes, if there is space on the same line. Overflowing inline elements
  <span>wrap onto a new line if possible — like this one containing text</span>, or just go on to a
  new line if not, much like this image will do:
  <img
    src="https://mdn.github.io/shared-assets/images/examples/long.jpg"
    alt="a wide but short section of a photo of several fabrics"
  />
</p>
```

```css hidden live-sample___static live-sample___relative live-sample___absolute
body {
  width: 500px;
  margin: 0 auto;
}

p {
  background: aqua;
  border: 3px solid blue;
  padding: 10px;
  margin: 10px;
}

span {
  background: red;
  border: 1px solid black;
}
```

```css live-sample___static
.positioned {
  position: static;
  background: yellow;
}
```

If you save and refresh, you'll see no difference at all, except for the updated background color of the second paragraph. The example should look like so:

{{embedlivesample("static", "100%", 500)}}

This is fine — as we said before, static positioning is the default behavior!

## Relative positioning

Relative positioning is the first position type we'll take a look at. This is very similar to static positioning, except that once the positioned element has taken its place in the normal flow, you can then modify its final position, including making it overlap other elements on the page. Go ahead and update the `position` declaration in your code:

```css
.positioned {
  position: relative;
  background: yellow;
}
```

If you save and refresh at this stage, you won't see a change in the result at all. So how do you modify the element's position? You need to use the {{cssxref("top")}}, {{cssxref("bottom")}}, {{cssxref("left")}}, and {{cssxref("right")}} properties, which we'll explain in the next section.

### Introducing top, bottom, left, and right

{{cssxref("top")}}, {{cssxref("bottom")}}, {{cssxref("left")}}, and {{cssxref("right")}} are used alongside {{cssxref("position")}} to specify exactly where to move the positioned element to. To try this out, add the following declarations to the `.positioned` rule in your CSS:

```css live-sample___relative
.positioned {
  position: relative;
  background: yellow;
  top: 30px;
  left: 30px;
}
```

> [!NOTE]
> The values of these properties can take any [units](/en-US/docs/Learn_web_development/Core/Styling_basics/Values_and_units) you'd reasonably expect: pixels, mm, rems, %, etc.

If you now save and refresh, you'll get a result something like this:

```html hidden live-sample___relative
<h1>Relative positioning</h1>

<p>I am a basic block level element. My adjacent block level elements sit on new lines below me.</p>

<p class="positioned">
  By default we span 100% of the width of our parent element, and we are as tall as our child
  content. Our total width and height is our content + padding + border width/height.
</p>

<p>
  We are separated by our margins. Because of margin collapsing, we are separated by the width of
  one of our margins, not both.
</p>

<p>
  Inline elements <span>like this one</span> and <span>this one</span> sit on the same line as one
  another, and adjacent text nodes, if there is space on the same line. Overflowing inline elements
  <span>wrap onto a new line if possible — like this one containing text</span>, or just go on to a
  new line if not, much like this image will do:
  <img src="https://mdn.github.io/shared-assets/images/examples/long.jpg" alt="snippet of cloth" />
</p>
```

{{embedlivesample("relative", "100%", 500)}}

Cool, huh? Ok, so this probably wasn't what you were expecting. Why has it moved to the bottom and to the right if we specified _top_ and _left_? This may seem counterintuitive. You need to think of it as if there's an invisible force that pushes the specified side of the positioned box, moving it in the opposite direction. So, for example, if you specify `top: 30px;`, it's as if a force will push the top of the box, causing it to move downwards by `30px`.

## Absolute positioning

Absolute positioning brings very different results.

### Setting position: absolute

Let's try changing the position declaration in your code as follows:

```css live-sample___absolute
.positioned {
  position: absolute;
  background: yellow;
  top: 30px;
  left: 30px;
}
```

If you now save and refresh, you should see something like so:

```html hidden live-sample___absolute
<h1>Absolute positioning</h1>

<p>I am a basic block level element. My adjacent block level elements sit on new lines below me.</p>

<p class="positioned">
  By default we span 100% of the width of our parent element, and we are as tall as our child
  content. Our total width and height is our content + padding + border width/height.
</p>

<p>
  We are separated by our margins. Because of margin collapsing, we are separated by the width of
  one of our margins, not both.
</p>

<p>
  inline elements <span>like this one</span> and <span>this one</span> sit on the same line as one
  another, and adjacent text nodes, if there is space on the same line. Overflowing inline elements
  <span>wrap onto a new line if possible — like this one containing text</span>, or just go on to a
  new line if not, much like this image will do:
  <img src="https://mdn.github.io/shared-assets/images/examples/long.jpg" alt="snippet of cloth" />
</p>
```

{{embedlivesample("absolute", "100%", 420)}}

First of all, note that the gap where the positioned element should be in the document flow is no longer there — the first and third elements have closed together like it no longer exists! Well, in a way, this is true. An absolutely positioned element no longer exists in the normal document flow. Instead, it sits on its own layer separate from everything else. This is very useful: it means that we can create isolated UI features that don't interfere with the layout of other elements on the page. For example, popup information boxes, control menus, rollover panels, UI features that can be dragged and dropped anywhere on the page, and so on.

Second, notice that the position of the element has changed. This is because {{cssxref("top")}}, {{cssxref("bottom")}}, {{cssxref("left")}}, and {{cssxref("right")}} behave in a different way with absolute positioning. Rather than positioning the element based on its relative position within the normal document flow, they specify the distance the element should be from each of the containing element's sides. In this case, we are saying that the absolutely positioned element should sit 30px from the top of the **containing element** (the **initial containing block**, in this case, see below) and 30px from the left.

> [!NOTE]
> You can use {{cssxref("top")}}, {{cssxref("bottom")}}, {{cssxref("left")}}, and {{cssxref("right")}} to resize elements if you need to. Try setting `top: 0; bottom: 0; left: 0; right: 0;` and `margin: 0;` on your positioned elements and see what happens! Put it back again afterwards…

> [!NOTE]
> Yes, margins still affect positioned elements. Margin collapsing doesn't, however.

### Positioning contexts

Which element is the "containing element" of an absolutely positioned element? This is very much dependent on the `position` property value of the ancestors of the positioned element.

If no ancestor elements have their position property explicitly defined, then by default all ancestor elements will have a static position. The result of this is the absolutely positioned element will be contained in the **initial containing block**. The initial containing block has the dimensions of the viewport and is also the block that contains the {{htmlelement("html")}} element. In other words, the absolutely positioned element will be displayed outside of the {{htmlelement("html")}} element and be positioned relative to the initial viewport.

The positioned element is nested inside the {{htmlelement("body")}} in the HTML source, but in the final layout it's 30px away from the top and the left edges of the page.

We can change the **positioning context**, that is, which element the absolutely positioned element is positioned relative to. This is done by setting positioning on one of the element's ancestors (the elements it's nested inside; you can't position it relative to an element it isn't nested inside). To see this, update your `body` rule to set `position: relative` on it:

```css
body {
  width: 500px;
  margin: 0 auto;
  position: relative;
}
```

This should give the following result:

```html hidden live-sample___contexts
<h1>Positioning context</h1>

<p>I am a basic block level element. My adjacent block level elements sit on new lines below me.</p>

<p class="positioned">
  Now I'm absolutely positioned relative to the
  <code>&lt;body&gt;</code> element, not the <code>&lt;html&gt;</code> element!
</p>

<p>
  We are separated by our margins. Because of margin collapsing, we are separated by the width of
  one of our margins, not both.
</p>

<p>
  inline elements <span>like this one</span> and <span>this one</span> sit on the same line as one
  another, and adjacent text nodes, if there is space on the same line. Overflowing inline elements
  <span>wrap onto a new line if possible — like this one containing text</span>, or just go on to a
  new line if not, much like this image will do:
  <img src="https://mdn.github.io/shared-assets/images/examples/long.jpg" alt="snippet of cloth" />
</p>
```

```css hidden live-sample___contexts live-sample___z-index
body {
  width: 500px;
  margin: 0 auto;
  position: relative;
}

p {
  background: aqua;
  border: 3px solid blue;
  padding: 10px;
  margin: 10px;
}

span {
  background: red;
  border: 1px solid black;
}

.positioned {
  position: absolute;
  background: yellow;
  top: 30px;
  left: 30px;
}
```

{{embedlivesample("contexts", "100%", 420)}}

The positioned element now sits relative to the {{htmlelement("body")}} element.

### Introducing z-index

All this absolute positioning is good fun, but there's another feature we haven't considered yet. When elements start to overlap, what determines which elements appear over others and which elements appear under others? In the example we've seen so far, we only have one positioned element in the positioning context, and it appears on the top since positioned elements win over non-positioned elements. What about when we have more than one?

Try adding the following to your CSS to make the first paragraph absolutely positioned too:

```css
p:nth-of-type(1) {
  position: absolute;
  background: lime;
  top: 10px;
  right: 30px;
}
```

At this point you'll see the first paragraph colored lime, moved out of the document flow, and positioned a bit above from where it originally was. It's also stacked below the original `.positioned` paragraph where the two overlap. This is because the `.positioned` paragraph is the second paragraph in the source order, and positioned elements later in the source order win over positioned elements earlier in the source order.

Can you change the stacking order? Yes, you can, by using the {{cssxref("z-index")}} property. "z-index" is a reference to the z-axis. You may recall from previous points in the course where we discussed web pages using horizontal (x-axis) and vertical (y-axis) coordinates to work out positioning for things like background images and drop shadow offsets. For languages that run left to right, (0,0) is at the top left of the page (or element), and the x- and y-axes run across to the right and down the page.

Web pages also have a z-axis: an imaginary line that runs from the surface of your screen towards your face (or whatever else you like to have in front of the screen). {{cssxref("z-index")}} values affect where positioned elements sit on that axis; positive values move them higher up the stack, negative values move them lower down the stack. By default, positioned elements all have a `z-index` of `auto`, which is effectively 0.

To change the stacking order, try adding the `z-index: 1` declaration to your `p:nth-of-type(1)` rule:

```css live-sample___z-index
p:nth-of-type(1) {
  position: absolute;
  background: lime;
  top: 10px;
  right: 30px;
  z-index: 1;
}
```

You should now see the lime paragraph on top:

```html hidden live-sample___z-index
<h1>z-index</h1>

<p>I am a basic block level element. My adjacent block level elements sit on new lines below me.</p>

<p class="positioned">
  Now I'm absolutely positioned relative to the
  <code>&lt;body&gt;</code> element, not the <code>&lt;html&gt;</code> element!
</p>

<p>
  We are separated by our margins. Because of margin collapsing, we are separated by the width of
  one of our margins, not both.
</p>

<p>
  inline elements <span>like this one</span> and <span>this one</span> sit on the same line as one
  another, and adjacent text nodes, if there is space on the same line. Overflowing inline elements
  <span>wrap onto a new line if possible — like this one containing text</span>, or just go on to a
  new line if not, much like this image will do:
  <img src="https://mdn.github.io/shared-assets/images/examples/long.jpg" alt="snippet of cloth" />
</p>
```

{{embedlivesample("z-index", "100%", 350)}}

Note that `z-index` only accepts unitless index values; you can't specify that you want one element to be 23 pixels up the Z-axis — it doesn't work like that. Higher values will go above lower values and it's up to you what values you use. Using values of 2 and 3 will give the same effect as values of 300 and 40000.

## Fixed positioning

Let's now look at fixed positioning. This works in exactly the same way as absolute positioning, with one key difference: whereas absolute positioning fixes an element in place relative to its nearest positioned ancestor (the initial containing block if there isn't one), **fixed positioning** fixes an element in place relative to the visible portion of the viewport. This means that you can create useful UI items that are fixed in place, like persistent navigation menus that are always visible no matter how much the page scrolls.

Let's put together a simple example to show what we mean. First of all, delete the existing `p:nth-of-type(1)` and `.positioned` rules from your CSS.

Now update the `body` rule to remove the `position: relative;` declaration and add a fixed height, like so:

```css
body {
  width: 500px;
  height: 1400px;
  margin: 0 auto;
}
```

Now we're going to give the {{htmlelement("Heading_Elements", "&lt;h1>")}} element a `position: fixed;` declaration and position it at the top of the viewport. Add the following rule to your CSS:

```css
h1 {
  position: fixed;
  top: 0;
  width: 500px;
  margin-top: 0;
  background: white;
  padding: 10px;
}
```

The `top: 0;` is required to make it stick to the top of the screen. We give the heading the same width as the content column, and a white background and some padding and margin so the content won't be visible underneath it.

If you save and refresh, you'll see a fun little effect of the heading staying fixed — the content appears to scroll up and disappear underneath it. But notice how some of the content is initially clipped under the heading. This is because the positioned heading no longer appears in the document flow, so the rest of the content moves up to the top.

We can improve this by moving the paragraphs all down a bit. Set some top margin on the first paragraph, like so:

```css
p:nth-of-type(1) {
  margin-top: 60px;
}
```

You should now see the following example rendered:

```html hidden live-sample___fixed
<h1>Fixed positioning</h1>

<p>I am a basic block level element. My adjacent block level elements sit on new lines below me.</p>

<p class="positioned">I'm not positioned any more.</p>

<p>
  We are separated by our margins. Because of margin collapsing, we are separated by the width of
  one of our margins, not both.
</p>

<p>
  Inline elements <span>like this one</span> and <span>this one</span> sit on the same line as one
  another, and adjacent text nodes, if there is space on the same line. Overflowing inline elements
  <span>wrap onto a new line if possible — like this one containing text</span>, or just go on to a
  new line if not, much like this image will do:
  <img src="https://mdn.github.io/shared-assets/images/examples/long.jpg" alt="snippet of cloth" />
</p>
```

```css hidden live-sample___fixed
body {
  width: 500px;
  height: 1400px;
  margin: 0 auto;
}

p {
  background: aqua;
  border: 3px solid blue;
  padding: 10px;
  margin: 10px;
}

span {
  background: red;
  border: 1px solid black;
}

h1 {
  position: fixed;
  top: 0px;
  width: 500px;
  margin-top: 0;
  background: white;
  padding: 10px;
}

p:nth-of-type(1) {
  margin-top: 60px;
}
```

{{ EmbedLiveSample('fixed', '100%', 400) }}

## Sticky positioning

There is another position value available called `position: sticky`, which is somewhat newer than the others. This is basically a hybrid between relative and fixed position. It allows a positioned element to act like it's relatively positioned until it scrolls to a certain threshold (e.g., 10px from the top of the viewport), after which it becomes fixed.

### Basic example

Sticky positioning can be used, for example, to cause a navigation bar to scroll with the page until a certain point and then stick to the top of the page.

```html hidden live-sample___basic-sticky
<h1>Sticky positioning</h1>

<p>
  Lorem ipsum dolor sit amet, consectetur adipiscing elit. Nulla luctus aliquam dolor, eu lacinia
  lorem placerat vulputate. Duis felis orci, pulvinar id metus ut, rutrum luctus orci. Cras
  porttitor imperdiet nunc, at ultricies tellus laoreet sit amet. Sed auctor cursus massa at porta.
  Integer ligula ipsum, tristique sit amet orci vel, viverra egestas ligula. Curabitur vehicula
  tellus neque, ac ornare ex malesuada et. In vitae convallis lacus. Aliquam erat volutpat.
  Suspendisse ac imperdiet turpis. Aenean finibus sollicitudin eros pharetra congue. Duis ornare
  egestas augue ut luctus. Proin blandit quam nec lacus varius commodo et a urna. Ut id ornare
  felis, eget fermentum sapien.
</p>

<div class="positioned">Sticky</div>

<p>
  Nam vulputate diam nec tempor bibendum. Donec luctus augue eget malesuada ultrices. Phasellus
  turpis est, posuere sit amet dapibus ut, facilisis sed est. Nam id risus quis ante semper
  consectetur eget aliquam lorem. Vivamus tristique elit dolor, sed pretium metus suscipit vel.
  Mauris ultricies lectus sed lobortis finibus. Vivamus eu urna eget velit cursus viverra quis
  vestibulum sem. Aliquam tincidunt eget purus in interdum. Cum sociis natoque penatibus et magnis
  dis parturient montes, nascetur ridiculus mus.
</p>

<p>
  Lorem ipsum dolor sit amet, consectetur adipiscing elit. Nulla luctus aliquam dolor, eu lacinia
  lorem placerat vulputate. Duis felis orci, pulvinar id metus ut, rutrum luctus orci. Cras
  porttitor imperdiet nunc, at ultricies tellus laoreet sit amet. Sed auctor cursus massa at porta.
  Integer ligula ipsum, tristique sit amet orci vel, viverra egestas ligula. Curabitur vehicula
  tellus neque, ac ornare ex malesuada et. In vitae convallis lacus. Aliquam erat volutpat.
  Suspendisse ac imperdiet turpis. Aenean finibus sollicitudin eros pharetra congue. Duis ornare
  egestas augue ut luctus. Proin blandit quam nec lacus varius commodo et a urna. Ut id ornare
  felis, eget fermentum sapien.
</p>
```

```css hidden live-sample___basic-sticky
body {
  width: 500px;
  margin: 0 auto;
}

.positioned {
  background: rgb(255 84 104 / 30%);
  border: 2px solid rgb(255 84 104);
  padding: 10px;
  margin: 10px;
  border-radius: 5px;
}
```

```css live-sample___basic-sticky
.positioned {
  position: sticky;
  top: 30px;
  left: 30px;
}
```

{{ EmbedLiveSample('basic-sticky', '100%', 200) }}

### Scrolling index

An interesting and common use of `position: sticky` is to create a scrolling index page where different headings stick to the top of the page as they reach it. The markup for such an example might look like so:

```html live-sample___sticky-scrolling-index
<h1>Sticky scrolling index</h1>

<dl>
  <dt>A</dt>
  <dd>Apple</dd>
  <dd>Ant</dd>
  <dd>Altimeter</dd>
  <dd>Airplane</dd>
  <dt>B</dt>
  <dd>Bird</dd>
  <dd>Buzzard</dd>
  <dd>Bee</dd>
  <dd>Banana</dd>
  <dd>Beanstalk</dd>
  <dt>C</dt>
  <dd>Calculator</dd>
  <dd>Cane</dd>
  <dd>Camera</dd>
  <dd>Camel</dd>
  <dt>D</dt>
  <dd>Duck</dd>
  <dd>Dime</dd>
  <dd>Dipstick</dd>
  <dd>Drone</dd>
  <dt>E</dt>
  <dd>Egg</dd>
  <dd>Elephant</dd>
  <dd>Egret</dd>
</dl>
```

The CSS would look as follows. In normal flow the {{htmlelement("dt")}} elements will scroll with the content. When we add `position: sticky` to the {{htmlelement("dt")}} element, along with a {{cssxref("top")}} value of `0`, the headings will stick to the top of the viewport as they reach it. Each subsequent header will then replace the previous one as it scrolls up to that position.

```css live-sample___sticky-scrolling-index
dt {
  background-color: black;
  color: white;
  padding: 10px;
  position: sticky;
  top: 0;
  left: 0;
  margin: 1em 0;
}
```

```css hidden live-sample___sticky-scrolling-index
body {
  width: 500px;
  height: 880px;
  margin: 0 auto;
}
```

{{ EmbedLiveSample('sticky-scrolling-index', '100%', 200) }}

Sticky elements are "sticky" relative to the nearest ancestor with a "scrolling mechanism", which is determined by its ancestors' [overflow](/en-US/docs/Web/CSS/Reference/Properties/overflow) property.

## Summary

I'm sure you had fun playing with basic positioning. While it's not an ideal method to use for entire layouts, there are many specific objectives it's suited for.

In the next article, we'll give you some tests that you can use to check how well you've understood and retained all this information.

## See also

- The {{cssxref("position")}} property reference.
- [Practical positioning examples](/en-US/docs/Learn_web_development/Core/CSS_layout/Practical_positioning_examples), for some more useful ideas.

{{PreviousMenuNext("Learn_web_development/Core/CSS_layout/Test_your_skills/Floats", "Learn_web_development/Core/CSS_layout/Test_your_skills/Position", "Learn_web_development/Core/CSS_layout")}}

`````

### Expected (prettier)

`````md
---
title: Positioning
slug: Learn_web_development/Core/CSS_layout/Positioning
page-type: learn-module-chapter
sidebar: learnsidebar
---

{{PreviousMenuNext("Learn_web_development/Core/CSS_layout/Test_your_skills/Floats", "Learn_web_development/Core/CSS_layout/Test_your_skills/Position", "Learn_web_development/Core/CSS_layout")}}

Positioning allows you to take elements out of normal document flow and make them behave differently, for example, sitting on top of one another or always remaining in the same place inside the browser viewport. This article explains the different {{cssxref("position")}} values and how to use them.

<table>
  <tbody>
    <tr>
      <th scope="row">Prerequisites:</th>
      <td>
        <a href="/en-US/docs/Learn_web_development/Core/Structuring_content"
          >Structuring content with HTML</a
        >,
        <a href="/en-US/docs/Learn_web_development/Core/Styling_basics">CSS Styling basics</a>,
        <a href="/en-US/docs/Learn_web_development/Core/Text_styling/Fundamentals">Fundamental text and font styling</a>,
        familiarity with <a href="/en-US/docs/Learn_web_development/Core/CSS_layout/Introduction">CSS layout fundamental concepts</a>.
      </td>
    </tr>
    <tr>
      <th scope="row">Learning outcomes:</th>
      <td>
        <ul>
          <li><code>static</code> positioning is the default way elements are positioned on the page.</li>
          <li>Relatively positioned elements remain in the normal flow, but absolute (and fixed/sticky) positioning takes elements completely out of the normal flow to sit in a separate layer.</li>
          <li>Final layout position can be modified using the <code>top</code>, <code>bottom</code>, <code>left</code>, and <code>right</code> properties, but these have different effects depending on the set <code>position</code> value.</li>
          <li>Setting the positioning context of a positioned element by positioning an ancestor element.</li>
        </ul>
      </td>
    </tr>
  </tbody>
</table>

## Working through the exercises

We'd like you to do the following exercises on your local computer. To get started, create a new HTML file on your local system and add the following content to it:

```html
<!DOCTYPE html>
<html lang="en-US">
  <head>
    <meta charset="utf-8" />
    <meta name="viewport" content="width=device-width" />
    <title>Positioning example</title>

    <style>
      body {
        width: 500px;
        margin: 0 auto;
      }

      p {
        background: aqua;
        border: 3px solid blue;
        padding: 10px;
        margin: 10px;
      }

      span {
        background: red;
        border: 1px solid black;
      }
    </style>
  </head>
  <body>
    <h1>Basic document flow</h1>

    <p>
      I am a basic block level element. My adjacent block level elements sit on new lines below me.
    </p>

    <p>
      By default we span 100% of the width of our parent element, and our height is as tall as our
      child content. Our total width and height is our content + padding + border width/height.
    </p>

    <p>
      We are separated by our margins. Because of margin collapsing, we are separated by the width
      of one of our margins, not both.
    </p>

    <p>
      inline elements <span>like this one</span> and <span>this one</span> sit on the same line as
      one another, and adjacent text nodes, if there is space on the same line. Overflowing inline
      elements <span>wrap onto a new line if possible — like this one containing text</span>, or
      just go on to a new line if not, much like this image will do:
      <img
        src="https://mdn.github.io/shared-assets/images/examples/long.jpg"
        alt="a wide but short section of a photo of several fabrics"
      />
    </p>
  </body>
</html>
```

## Introducing positioning

Positioning allows us to produce interesting results by overriding normal document flow. What if you want to slightly alter the position of some boxes from their default flow position to give a slightly quirky, distressed feel? Positioning is your tool. Or what if you want to create a UI element that floats over the top of other parts of the page and/or always sits in the same place inside the browser window no matter how much the page is scrolled? Positioning makes such layout work possible.

There are a number of different types of positioning that you can put into effect on HTML elements. To make a specific type of positioning active on an element, we use the {{cssxref("position")}} property.

## Static positioning

Static positioning is the default that every element gets. It just means "put the element into its default position in the normal flow — nothing special to see here."

To see this (and get your example set up for future sections) first add a `class` of `positioned` to the second {{htmlelement("p")}} in the HTML:

```html
<p class="positioned">…</p>
```

Now add the following rule to the bottom of your CSS:

```html hidden live-sample___static
<h1>Static positioning</h1>

<p>I am a basic block level element. My adjacent block level elements sit on new lines below me.</p>

<p class="positioned">
  By default we span 100% of the width of our parent element, and our are as tall as our child
  content. Our total width and height is our content + padding + border width/height.
</p>

<p>
  We are separated by our margins. Because of margin collapsing, we are separated by the width of
  one of our margins, not both.
</p>

<p>
  inline elements <span>like this one</span> and <span>this one</span> sit on the same line as one
  another, and adjacent text nodes, if there is space on the same line. Overflowing inline elements
  <span>wrap onto a new line if possible — like this one containing text</span>, or just go on to a
  new line if not, much like this image will do:
  <img
    src="https://mdn.github.io/shared-assets/images/examples/long.jpg"
    alt="a wide but short section of a photo of several fabrics"
  />
</p>
```

```css hidden live-sample___static live-sample___relative live-sample___absolute
body {
  width: 500px;
  margin: 0 auto;
}

p {
  background: aqua;
  border: 3px solid blue;
  padding: 10px;
  margin: 10px;
}

span {
  background: red;
  border: 1px solid black;
}
```

```css live-sample___static
.positioned {
  position: static;
  background: yellow;
}
```

If you save and refresh, you'll see no difference at all, except for the updated background color of the second paragraph. The example should look like so:

{{embedlivesample("static", "100%", 500)}}

This is fine — as we said before, static positioning is the default behavior!

## Relative positioning

Relative positioning is the first position type we'll take a look at. This is very similar to static positioning, except that once the positioned element has taken its place in the normal flow, you can then modify its final position, including making it overlap other elements on the page. Go ahead and update the `position` declaration in your code:

```css
.positioned {
  position: relative;
  background: yellow;
}
```

If you save and refresh at this stage, you won't see a change in the result at all. So how do you modify the element's position? You need to use the {{cssxref("top")}}, {{cssxref("bottom")}}, {{cssxref("left")}}, and {{cssxref("right")}} properties, which we'll explain in the next section.

### Introducing top, bottom, left, and right

{{cssxref("top")}}, {{cssxref("bottom")}}, {{cssxref("left")}}, and {{cssxref("right")}} are used alongside {{cssxref("position")}} to specify exactly where to move the positioned element to. To try this out, add the following declarations to the `.positioned` rule in your CSS:

```css live-sample___relative
.positioned {
  position: relative;
  background: yellow;
  top: 30px;
  left: 30px;
}
```

> [!NOTE]
> The values of these properties can take any [units](/en-US/docs/Learn_web_development/Core/Styling_basics/Values_and_units) you'd reasonably expect: pixels, mm, rems, %, etc.

If you now save and refresh, you'll get a result something like this:

```html hidden live-sample___relative
<h1>Relative positioning</h1>

<p>I am a basic block level element. My adjacent block level elements sit on new lines below me.</p>

<p class="positioned">
  By default we span 100% of the width of our parent element, and we are as tall as our child
  content. Our total width and height is our content + padding + border width/height.
</p>

<p>
  We are separated by our margins. Because of margin collapsing, we are separated by the width of
  one of our margins, not both.
</p>

<p>
  Inline elements <span>like this one</span> and <span>this one</span> sit on the same line as one
  another, and adjacent text nodes, if there is space on the same line. Overflowing inline elements
  <span>wrap onto a new line if possible — like this one containing text</span>, or just go on to a
  new line if not, much like this image will do:
  <img src="https://mdn.github.io/shared-assets/images/examples/long.jpg" alt="snippet of cloth" />
</p>
```

{{embedlivesample("relative", "100%", 500)}}

Cool, huh? Ok, so this probably wasn't what you were expecting. Why has it moved to the bottom and to the right if we specified _top_ and _left_? This may seem counterintuitive. You need to think of it as if there's an invisible force that pushes the specified side of the positioned box, moving it in the opposite direction. So, for example, if you specify `top: 30px;`, it's as if a force will push the top of the box, causing it to move downwards by `30px`.

## Absolute positioning

Absolute positioning brings very different results.

### Setting position: absolute

Let's try changing the position declaration in your code as follows:

```css live-sample___absolute
.positioned {
  position: absolute;
  background: yellow;
  top: 30px;
  left: 30px;
}
```

If you now save and refresh, you should see something like so:

```html hidden live-sample___absolute
<h1>Absolute positioning</h1>

<p>I am a basic block level element. My adjacent block level elements sit on new lines below me.</p>

<p class="positioned">
  By default we span 100% of the width of our parent element, and we are as tall as our child
  content. Our total width and height is our content + padding + border width/height.
</p>

<p>
  We are separated by our margins. Because of margin collapsing, we are separated by the width of
  one of our margins, not both.
</p>

<p>
  inline elements <span>like this one</span> and <span>this one</span> sit on the same line as one
  another, and adjacent text nodes, if there is space on the same line. Overflowing inline elements
  <span>wrap onto a new line if possible — like this one containing text</span>, or just go on to a
  new line if not, much like this image will do:
  <img src="https://mdn.github.io/shared-assets/images/examples/long.jpg" alt="snippet of cloth" />
</p>
```

{{embedlivesample("absolute", "100%", 420)}}

First of all, note that the gap where the positioned element should be in the document flow is no longer there — the first and third elements have closed together like it no longer exists! Well, in a way, this is true. An absolutely positioned element no longer exists in the normal document flow. Instead, it sits on its own layer separate from everything else. This is very useful: it means that we can create isolated UI features that don't interfere with the layout of other elements on the page. For example, popup information boxes, control menus, rollover panels, UI features that can be dragged and dropped anywhere on the page, and so on.

Second, notice that the position of the element has changed. This is because {{cssxref("top")}}, {{cssxref("bottom")}}, {{cssxref("left")}}, and {{cssxref("right")}} behave in a different way with absolute positioning. Rather than positioning the element based on its relative position within the normal document flow, they specify the distance the element should be from each of the containing element's sides. In this case, we are saying that the absolutely positioned element should sit 30px from the top of the **containing element** (the **initial containing block**, in this case, see below) and 30px from the left.

> [!NOTE]
> You can use {{cssxref("top")}}, {{cssxref("bottom")}}, {{cssxref("left")}}, and {{cssxref("right")}} to resize elements if you need to. Try setting `top: 0; bottom: 0; left: 0; right: 0;` and `margin: 0;` on your positioned elements and see what happens! Put it back again afterwards…

> [!NOTE]
> Yes, margins still affect positioned elements. Margin collapsing doesn't, however.

### Positioning contexts

Which element is the "containing element" of an absolutely positioned element? This is very much dependent on the `position` property value of the ancestors of the positioned element.

If no ancestor elements have their position property explicitly defined, then by default all ancestor elements will have a static position. The result of this is the absolutely positioned element will be contained in the **initial containing block**. The initial containing block has the dimensions of the viewport and is also the block that contains the {{htmlelement("html")}} element. In other words, the absolutely positioned element will be displayed outside of the {{htmlelement("html")}} element and be positioned relative to the initial viewport.

The positioned element is nested inside the {{htmlelement("body")}} in the HTML source, but in the final layout it's 30px away from the top and the left edges of the page.

We can change the **positioning context**, that is, which element the absolutely positioned element is positioned relative to. This is done by setting positioning on one of the element's ancestors (the elements it's nested inside; you can't position it relative to an element it isn't nested inside). To see this, update your `body` rule to set `position: relative` on it:

```css
body {
  width: 500px;
  margin: 0 auto;
  position: relative;
}
```

This should give the following result:

```html hidden live-sample___contexts
<h1>Positioning context</h1>

<p>I am a basic block level element. My adjacent block level elements sit on new lines below me.</p>

<p class="positioned">
  Now I'm absolutely positioned relative to the
  <code>&lt;body&gt;</code> element, not the <code>&lt;html&gt;</code> element!
</p>

<p>
  We are separated by our margins. Because of margin collapsing, we are separated by the width of
  one of our margins, not both.
</p>

<p>
  inline elements <span>like this one</span> and <span>this one</span> sit on the same line as one
  another, and adjacent text nodes, if there is space on the same line. Overflowing inline elements
  <span>wrap onto a new line if possible — like this one containing text</span>, or just go on to a
  new line if not, much like this image will do:
  <img src="https://mdn.github.io/shared-assets/images/examples/long.jpg" alt="snippet of cloth" />
</p>
```

```css hidden live-sample___contexts live-sample___z-index
body {
  width: 500px;
  margin: 0 auto;
  position: relative;
}

p {
  background: aqua;
  border: 3px solid blue;
  padding: 10px;
  margin: 10px;
}

span {
  background: red;
  border: 1px solid black;
}

.positioned {
  position: absolute;
  background: yellow;
  top: 30px;
  left: 30px;
}
```

{{embedlivesample("contexts", "100%", 420)}}

The positioned element now sits relative to the {{htmlelement("body")}} element.

### Introducing z-index

All this absolute positioning is good fun, but there's another feature we haven't considered yet. When elements start to overlap, what determines which elements appear over others and which elements appear under others? In the example we've seen so far, we only have one positioned element in the positioning context, and it appears on the top since positioned elements win over non-positioned elements. What about when we have more than one?

Try adding the following to your CSS to make the first paragraph absolutely positioned too:

```css
p:nth-of-type(1) {
  position: absolute;
  background: lime;
  top: 10px;
  right: 30px;
}
```

At this point you'll see the first paragraph colored lime, moved out of the document flow, and positioned a bit above from where it originally was. It's also stacked below the original `.positioned` paragraph where the two overlap. This is because the `.positioned` paragraph is the second paragraph in the source order, and positioned elements later in the source order win over positioned elements earlier in the source order.

Can you change the stacking order? Yes, you can, by using the {{cssxref("z-index")}} property. "z-index" is a reference to the z-axis. You may recall from previous points in the course where we discussed web pages using horizontal (x-axis) and vertical (y-axis) coordinates to work out positioning for things like background images and drop shadow offsets. For languages that run left to right, (0,0) is at the top left of the page (or element), and the x- and y-axes run across to the right and down the page.

Web pages also have a z-axis: an imaginary line that runs from the surface of your screen towards your face (or whatever else you like to have in front of the screen). {{cssxref("z-index")}} values affect where positioned elements sit on that axis; positive values move them higher up the stack, negative values move them lower down the stack. By default, positioned elements all have a `z-index` of `auto`, which is effectively 0.

To change the stacking order, try adding the `z-index: 1` declaration to your `p:nth-of-type(1)` rule:

```css live-sample___z-index
p:nth-of-type(1) {
  position: absolute;
  background: lime;
  top: 10px;
  right: 30px;
  z-index: 1;
}
```

You should now see the lime paragraph on top:

```html hidden live-sample___z-index
<h1>z-index</h1>

<p>I am a basic block level element. My adjacent block level elements sit on new lines below me.</p>

<p class="positioned">
  Now I'm absolutely positioned relative to the
  <code>&lt;body&gt;</code> element, not the <code>&lt;html&gt;</code> element!
</p>

<p>
  We are separated by our margins. Because of margin collapsing, we are separated by the width of
  one of our margins, not both.
</p>

<p>
  inline elements <span>like this one</span> and <span>this one</span> sit on the same line as one
  another, and adjacent text nodes, if there is space on the same line. Overflowing inline elements
  <span>wrap onto a new line if possible — like this one containing text</span>, or just go on to a
  new line if not, much like this image will do:
  <img src="https://mdn.github.io/shared-assets/images/examples/long.jpg" alt="snippet of cloth" />
</p>
```

{{embedlivesample("z-index", "100%", 350)}}

Note that `z-index` only accepts unitless index values; you can't specify that you want one element to be 23 pixels up the Z-axis — it doesn't work like that. Higher values will go above lower values and it's up to you what values you use. Using values of 2 and 3 will give the same effect as values of 300 and 40000.

## Fixed positioning

Let's now look at fixed positioning. This works in exactly the same way as absolute positioning, with one key difference: whereas absolute positioning fixes an element in place relative to its nearest positioned ancestor (the initial containing block if there isn't one), **fixed positioning** fixes an element in place relative to the visible portion of the viewport. This means that you can create useful UI items that are fixed in place, like persistent navigation menus that are always visible no matter how much the page scrolls.

Let's put together a simple example to show what we mean. First of all, delete the existing `p:nth-of-type(1)` and `.positioned` rules from your CSS.

Now update the `body` rule to remove the `position: relative;` declaration and add a fixed height, like so:

```css
body {
  width: 500px;
  height: 1400px;
  margin: 0 auto;
}
```

Now we're going to give the {{htmlelement("Heading_Elements", "&lt;h1>")}} element a `position: fixed;` declaration and position it at the top of the viewport. Add the following rule to your CSS:

```css
h1 {
  position: fixed;
  top: 0;
  width: 500px;
  margin-top: 0;
  background: white;
  padding: 10px;
}
```

The `top: 0;` is required to make it stick to the top of the screen. We give the heading the same width as the content column, and a white background and some padding and margin so the content won't be visible underneath it.

If you save and refresh, you'll see a fun little effect of the heading staying fixed — the content appears to scroll up and disappear underneath it. But notice how some of the content is initially clipped under the heading. This is because the positioned heading no longer appears in the document flow, so the rest of the content moves up to the top.

We can improve this by moving the paragraphs all down a bit. Set some top margin on the first paragraph, like so:

```css
p:nth-of-type(1) {
  margin-top: 60px;
}
```

You should now see the following example rendered:

```html hidden live-sample___fixed
<h1>Fixed positioning</h1>

<p>I am a basic block level element. My adjacent block level elements sit on new lines below me.</p>

<p class="positioned">I'm not positioned any more.</p>

<p>
  We are separated by our margins. Because of margin collapsing, we are separated by the width of
  one of our margins, not both.
</p>

<p>
  Inline elements <span>like this one</span> and <span>this one</span> sit on the same line as one
  another, and adjacent text nodes, if there is space on the same line. Overflowing inline elements
  <span>wrap onto a new line if possible — like this one containing text</span>, or just go on to a
  new line if not, much like this image will do:
  <img src="https://mdn.github.io/shared-assets/images/examples/long.jpg" alt="snippet of cloth" />
</p>
```

```css hidden live-sample___fixed
body {
  width: 500px;
  height: 1400px;
  margin: 0 auto;
}

p {
  background: aqua;
  border: 3px solid blue;
  padding: 10px;
  margin: 10px;
}

span {
  background: red;
  border: 1px solid black;
}

h1 {
  position: fixed;
  top: 0px;
  width: 500px;
  margin-top: 0;
  background: white;
  padding: 10px;
}

p:nth-of-type(1) {
  margin-top: 60px;
}
```

{{ EmbedLiveSample('fixed', '100%', 400) }}

## Sticky positioning

There is another position value available called `position: sticky`, which is somewhat newer than the others. This is basically a hybrid between relative and fixed position. It allows a positioned element to act like it's relatively positioned until it scrolls to a certain threshold (e.g., 10px from the top of the viewport), after which it becomes fixed.

### Basic example

Sticky positioning can be used, for example, to cause a navigation bar to scroll with the page until a certain point and then stick to the top of the page.

```html hidden live-sample___basic-sticky
<h1>Sticky positioning</h1>

<p>
  Lorem ipsum dolor sit amet, consectetur adipiscing elit. Nulla luctus aliquam dolor, eu lacinia
  lorem placerat vulputate. Duis felis orci, pulvinar id metus ut, rutrum luctus orci. Cras
  porttitor imperdiet nunc, at ultricies tellus laoreet sit amet. Sed auctor cursus massa at porta.
  Integer ligula ipsum, tristique sit amet orci vel, viverra egestas ligula. Curabitur vehicula
  tellus neque, ac ornare ex malesuada et. In vitae convallis lacus. Aliquam erat volutpat.
  Suspendisse ac imperdiet turpis. Aenean finibus sollicitudin eros pharetra congue. Duis ornare
  egestas augue ut luctus. Proin blandit quam nec lacus varius commodo et a urna. Ut id ornare
  felis, eget fermentum sapien.
</p>

<div class="positioned">Sticky</div>

<p>
  Nam vulputate diam nec tempor bibendum. Donec luctus augue eget malesuada ultrices. Phasellus
  turpis est, posuere sit amet dapibus ut, facilisis sed est. Nam id risus quis ante semper
  consectetur eget aliquam lorem. Vivamus tristique elit dolor, sed pretium metus suscipit vel.
  Mauris ultricies lectus sed lobortis finibus. Vivamus eu urna eget velit cursus viverra quis
  vestibulum sem. Aliquam tincidunt eget purus in interdum. Cum sociis natoque penatibus et magnis
  dis parturient montes, nascetur ridiculus mus.
</p>

<p>
  Lorem ipsum dolor sit amet, consectetur adipiscing elit. Nulla luctus aliquam dolor, eu lacinia
  lorem placerat vulputate. Duis felis orci, pulvinar id metus ut, rutrum luctus orci. Cras
  porttitor imperdiet nunc, at ultricies tellus laoreet sit amet. Sed auctor cursus massa at porta.
  Integer ligula ipsum, tristique sit amet orci vel, viverra egestas ligula. Curabitur vehicula
  tellus neque, ac ornare ex malesuada et. In vitae convallis lacus. Aliquam erat volutpat.
  Suspendisse ac imperdiet turpis. Aenean finibus sollicitudin eros pharetra congue. Duis ornare
  egestas augue ut luctus. Proin blandit quam nec lacus varius commodo et a urna. Ut id ornare
  felis, eget fermentum sapien.
</p>
```

```css hidden live-sample___basic-sticky
body {
  width: 500px;
  margin: 0 auto;
}

.positioned {
  background: rgb(255 84 104 / 30%);
  border: 2px solid rgb(255 84 104);
  padding: 10px;
  margin: 10px;
  border-radius: 5px;
}
```

```css live-sample___basic-sticky
.positioned {
  position: sticky;
  top: 30px;
  left: 30px;
}
```

{{ EmbedLiveSample('basic-sticky', '100%', 200) }}

### Scrolling index

An interesting and common use of `position: sticky` is to create a scrolling index page where different headings stick to the top of the page as they reach it. The markup for such an example might look like so:

```html live-sample___sticky-scrolling-index
<h1>Sticky scrolling index</h1>

<dl>
  <dt>A</dt>
  <dd>Apple</dd>
  <dd>Ant</dd>
  <dd>Altimeter</dd>
  <dd>Airplane</dd>
  <dt>B</dt>
  <dd>Bird</dd>
  <dd>Buzzard</dd>
  <dd>Bee</dd>
  <dd>Banana</dd>
  <dd>Beanstalk</dd>
  <dt>C</dt>
  <dd>Calculator</dd>
  <dd>Cane</dd>
  <dd>Camera</dd>
  <dd>Camel</dd>
  <dt>D</dt>
  <dd>Duck</dd>
  <dd>Dime</dd>
  <dd>Dipstick</dd>
  <dd>Drone</dd>
  <dt>E</dt>
  <dd>Egg</dd>
  <dd>Elephant</dd>
  <dd>Egret</dd>
</dl>
```

The CSS would look as follows. In normal flow the {{htmlelement("dt")}} elements will scroll with the content. When we add `position: sticky` to the {{htmlelement("dt")}} element, along with a {{cssxref("top")}} value of `0`, the headings will stick to the top of the viewport as they reach it. Each subsequent header will then replace the previous one as it scrolls up to that position.

```css live-sample___sticky-scrolling-index
dt {
  background-color: black;
  color: white;
  padding: 10px;
  position: sticky;
  top: 0;
  left: 0;
  margin: 1em 0;
}
```

```css hidden live-sample___sticky-scrolling-index
body {
  width: 500px;
  height: 880px;
  margin: 0 auto;
}
```

{{ EmbedLiveSample('sticky-scrolling-index', '100%', 200) }}

Sticky elements are "sticky" relative to the nearest ancestor with a "scrolling mechanism", which is determined by its ancestors' [overflow](/en-US/docs/Web/CSS/Reference/Properties/overflow) property.

## Summary

I'm sure you had fun playing with basic positioning. While it's not an ideal method to use for entire layouts, there are many specific objectives it's suited for.

In the next article, we'll give you some tests that you can use to check how well you've understood and retained all this information.

## See also

- The {{cssxref("position")}} property reference.
- [Practical positioning examples](/en-US/docs/Learn_web_development/Core/CSS_layout/Practical_positioning_examples), for some more useful ideas.

{{PreviousMenuNext("Learn_web_development/Core/CSS_layout/Test_your_skills/Floats", "Learn_web_development/Core/CSS_layout/Test_your_skills/Position", "Learn_web_development/Core/CSS_layout")}}

`````
