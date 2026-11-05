# externals/mdn-learn/css_layout/multiple-column_layout/index.md

## Option 1

`````json
{"printWidth":80,"bracketSameLine":true}
`````

### Diff

`````diff
===================================================================
--- prettier
+++ oxfmt
@@ -33,9 +33,9 @@
 
 Let's explore how to use multiple-column layout — often referred to as _multicol_ — by building up an example step by step. To follow along, create a new HTML on your local system and add the following content into it:
 
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
title: Multiple-column layout
slug: Learn_web_development/Core/CSS_layout/Multiple-column_Layout
page-type: learn-module-chapter
sidebar: learnsidebar
---

The multiple-column layout specification provides you with a method for laying content out in columns, as you might see in a newspaper. This article explains how to use this feature.

<table>
  <tbody>
    <tr>
      <th scope="row">Prerequisites:</th>
      <td>
        HTML basics (study
        <a href="/en-US/docs/Learn_web_development/Core/Structuring_content"
          >Structuring content with HTML</a
        >), and an idea of How CSS works (study
        <a href="/en-US/docs/Learn_web_development/Core/Styling_basics">CSS Styling basics</a>.)
      </td>
    </tr>
    <tr>
      <th scope="row">Objective:</th>
      <td>
        To learn how to create multiple-column layout on webpages, such as you
        might find in a newspaper.
      </td>
    </tr>
  </tbody>
</table>

## A basic example

Let's explore how to use multiple-column layout — often referred to as _multicol_ — by building up an example step by step. To follow along, create a new HTML on your local system and add the following content into it:

```html
<!doctype html>
<html lang="en-US">
  <head>
    <meta charset="utf-8" />
    <meta name="viewport" content="width=device-width" />
    <title>Multicol example</title>
    <style>
      body {
        width: 90%;
        max-width: 900px;
        margin: 2em auto;
        font:
          0.9em/1.2 "Arial",
          "Helvetica",
          sans-serif;
      }
    </style>
  </head>

  <body>
    <div class="container">
      <h1>Simple multicol example</h1>

      <p>
        Lorem ipsum dolor sit amet, consectetur adipiscing elit. Nulla luctus
        aliquam dolor, eu lacinia lorem placerat vulputate. Duis felis orci,
        pulvinar id metus ut, rutrum luctus orci. Cras porttitor imperdiet nunc,
        at ultricies tellus laoreet sit amet. Sed auctor cursus massa at porta.
        Integer ligula ipsum, tristique sit amet orci vel, viverra egestas
        ligula. Curabitur vehicula tellus neque, ac ornare ex malesuada et. In
        vitae convallis lacus. Aliquam erat volutpat. Suspendisse ac imperdiet
        turpis. Aenean finibus sollicitudin eros pharetra congue. Duis ornare
        egestas augue ut luctus. Proin blandit quam nec lacus varius commodo et
        a urna. Ut id ornare felis, eget fermentum sapien.
      </p>

      <p>
        Nam vulputate diam nec tempor bibendum. Donec luctus augue eget
        malesuada ultrices. Phasellus turpis est, posuere sit amet dapibus ut,
        facilisis sed est. Nam id risus quis ante semper consectetur eget
        aliquam lorem. Vivamus tristique elit dolor, sed pretium metus suscipit
        vel. Mauris ultricies lectus sed lobortis finibus. Vivamus eu urna eget
        velit cursus viverra quis vestibulum sem. Aliquam tincidunt eget purus
        in interdum. Cum sociis natoque penatibus et magnis dis parturient
        montes, nascetur ridiculus mus.
      </p>
    </div>
  </body>
</html>
```

You'll see various live examples below showing you what the rendered output should look like at each stage.

### A three-column layout

Our starting point file contains some very simple HTML: a wrapper with a class of `container`, inside of which is a heading and some paragraphs.

The {{htmlelement("div")}} with a class of container will become our multicol container. We enable multicol by using one of two properties: {{cssxref("column-count")}} or {{cssxref("column-width")}}. The `column-count` property takes a number as its value and creates that number of columns. If you add the following CSS to your stylesheet and reload the page, you'll get three columns:

```css live-sample___column-count
.container {
  column-count: 3;
}
```

The columns that you create have flexible widths — the browser works out how much space to assign each column.

```css hidden live-sample___column-count live-sample___column-width live-sample___column-styling live-sample___column-spanning
body {
  width: 90%;
  max-width: 900px;
  margin: 2em auto;
  font:
    0.9em/1.2 "Helvetica",
    "Arial",
    sans-serif;
}
```

```html hidden live-sample___column-count live-sample___column-width live-sample___column-styling
<div class="container">
  <h1>Simple multicol example</h1>

  <p>
    Lorem ipsum dolor sit amet, consectetur adipiscing elit. Nulla luctus
    aliquam dolor, eu lacinia lorem placerat vulputate. Duis felis orci,
    pulvinar id metus ut, rutrum luctus orci. Cras porttitor imperdiet nunc, at
    ultricies tellus laoreet sit amet. Sed auctor cursus massa at porta. Integer
    ligula ipsum, tristique sit amet orci vel, viverra egestas ligula. Curabitur
    vehicula tellus neque, ac ornare ex malesuada et. In vitae convallis lacus.
    Aliquam erat volutpat. Suspendisse ac imperdiet turpis. Aenean finibus
    sollicitudin eros pharetra congue. Duis ornare egestas augue ut luctus.
    Proin blandit quam nec lacus varius commodo et a urna. Ut id ornare felis,
    eget fermentum sapien.
  </p>

  <p>
    Nam vulputate diam nec tempor bibendum. Donec luctus augue eget malesuada
    ultrices. Phasellus turpis est, posuere sit amet dapibus ut, facilisis sed
    est. Nam id risus quis ante semper consectetur eget aliquam lorem. Vivamus
    tristique elit dolor, sed pretium metus suscipit vel. Mauris ultricies
    lectus sed lobortis finibus. Vivamus eu urna eget velit cursus viverra quis
    vestibulum sem. Aliquam tincidunt eget purus in interdum. Cum sociis natoque
    penatibus et magnis dis parturient montes, nascetur ridiculus mus.
  </p>
</div>
```

{{ EmbedLiveSample('column-count', '100%', 400) }}

### Setting column-width

Change your CSS to use `column-width` as follows:

```css live-sample___column-width
.container {
  column-width: 200px;
}
```

The browser will now give you as many columns as it can of the size that you specify; any remaining space is then shared between the existing columns. This means that you won't get exactly the width that you specify unless your container is exactly divisible by that width.

{{ EmbedLiveSample('column-width', '100%', 400) }}

## Styling the columns

The columns created by multicol cannot be styled individually. There's no way to make one column bigger than other columns or to change the background or text color of a single column. You have two opportunities to change the way that columns display:

- Changing the size of the gap between columns using the {{cssxref("column-gap")}}.
- Adding a rule between columns with {{cssxref("column-rule")}}.

Using your example above, change the size of the gap by adding a `column-gap` property. You can play around with different values — the property accepts any length unit.

Now add a rule between the columns with `column-rule`. In a similar way to the {{cssxref("border")}} property that you encountered in previous lessons, `column-rule` is a shorthand for {{cssxref("column-rule-color")}}, {{cssxref("column-rule-style")}}, and {{cssxref("column-rule-width")}}, and accepts the same values as `border`.

```css live-sample___column-styling live-sample___column-spanning
.container {
  column-count: 3;
  column-gap: 20px;
  column-rule: 4px dotted rgb(79 185 227);
}
```

Try adding rules of different styles and colors.

{{ EmbedLiveSample('column-styling', '100%', 400) }}

Something to take note of is that the rule doesn't take up any width of its own. It lies across the gap you created with `column-gap`. To make more space on either side of the rule, you'll need to increase the `column-gap` size.

## Spanning columns

You can cause an element to span across all the columns. In this case, the content breaks where the spanning element's introduced and then continues below the element, creating a new set of columns. To cause an element to span all the columns, specify the value of `all` for the {{cssxref("column-span")}} property.

> [!NOTE]
> It isn't possible to cause an element to span just _some_ columns. The property can only have the values of `none` (which is the default) or `all`.

Add the following rule to your CSS, below the previous ones:

```css live-sample___column-spanning
h2 {
  column-span: all;
  background-color: rgb(79 185 227);
  color: white;
  padding: 0.5em;
}
```

Now add a second-level heading between the first and second paragraphs:

```html
<h2>Spanning subhead</h2>
```

```html hidden live-sample___column-spanning
<div class="container">
  <h1>Simple multicol example</h1>

  <p>
    Lorem ipsum dolor sit amet, consectetur adipiscing elit. Nulla luctus
    aliquam dolor, eu lacinia lorem placerat vulputate. Duis felis orci,
    pulvinar id metus ut, rutrum luctus orci. Cras porttitor imperdiet nunc, at
    ultricies tellus laoreet sit amet. Sed auctor cursus massa at porta. Integer
    ligula ipsum, tristique sit amet orci vel, viverra egestas ligula.
  </p>

  <h2>Spanning subhead</h2>

  <p>
    Curabitur vehicula tellus neque, ac ornare ex malesuada et. In vitae
    convallis lacus. Aliquam erat volutpat. Suspendisse ac imperdiet turpis.
    Aenean finibus sollicitudin eros pharetra congue. Duis ornare egestas augue
    ut luctus. Proin blandit quam nec lacus varius commodo et a urna. Ut id
    ornare felis, eget fermentum sapien.
  </p>

  <p>
    Nam vulputate diam nec tempor bibendum. Donec luctus augue eget malesuada
    ultrices. Phasellus turpis est, posuere sit amet dapibus ut, facilisis sed
    est. Nam id risus quis ante semper consectetur eget aliquam lorem. Vivamus
    tristique elit dolor, sed pretium metus suscipit vel. Mauris ultricies
    lectus sed lobortis finibus. Vivamus eu urna eget velit cursus viverra quis
    vestibulum sem. Aliquam tincidunt eget purus in interdum. Cum sociis natoque
    penatibus et magnis dis parturient montes, nascetur ridiculus mus.
  </p>
</div>
```

You rendered code should now look like this:

{{ EmbedLiveSample('column-spanning', '100%', 550) }}

## Columns and fragmentation

The content of a multi-column layout is fragmented. It essentially behaves the same way as content behaves in paged media, such as when you print a webpage. When you turn your content into a multicol container, it fragments into columns. In order for the content to do this, it must _break_.

### Fragmented boxes

Sometimes, this breaking will happen in places that lead to a poor reading experience. In the example below, I have used multicol to lay out a series of boxes, each of which has a heading and some text inside. The heading becomes separated from the text if the columns fragment between the two.

```css hidden live-sample___fragmented-boxes live-sample___fragmented-boxes-fixed
body {
  width: 90%;
  max-width: 900px;
  margin: 2em auto;
  font:
    0.9em/1.2 "Helvetica",
    "Arial",
    sans-serif;
}
```

```html live-sample___fragmented-boxes live-sample___fragmented-boxes-fixed
<div class="container">
  <div class="card">
    <h2>I am the heading</h2>
    <p>
      Lorem ipsum dolor sit amet, consectetur adipiscing elit. Nulla luctus
      aliquam dolor, eu lacinia lorem placerat vulputate. Duis felis orci,
      pulvinar id metus ut, rutrum luctus orci. Cras porttitor imperdiet nunc,
      at ultricies tellus laoreet sit amet. Sed auctor cursus massa at porta.
      Integer ligula ipsum, tristique sit amet orci vel, viverra egestas ligula.
    </p>
  </div>

  <div class="card">
    <h2>I am the heading</h2>
    <p>
      Lorem ipsum dolor sit amet, consectetur adipiscing elit. Nulla luctus
      aliquam dolor, eu lacinia lorem placerat vulputate. Duis felis orci,
      pulvinar id metus ut, rutrum luctus orci. Cras porttitor imperdiet nunc,
      at ultricies tellus laoreet sit amet. Sed auctor cursus massa at porta.
      Integer ligula ipsum, tristique sit amet orci vel, viverra egestas ligula.
    </p>
  </div>

  <div class="card">
    <h2>I am the heading</h2>
    <p>
      Lorem ipsum dolor sit amet, consectetur adipiscing elit. Nulla luctus
      aliquam dolor, eu lacinia lorem placerat vulputate. Duis felis orci,
      pulvinar id metus ut, rutrum luctus orci. Cras porttitor imperdiet nunc,
      at ultricies tellus laoreet sit amet. Sed auctor cursus massa at porta.
      Integer ligula ipsum, tristique sit amet orci vel, viverra egestas ligula.
    </p>
  </div>
  <div class="card">
    <h2>I am the heading</h2>
    <p>
      Lorem ipsum dolor sit amet, consectetur adipiscing elit. Nulla luctus
      aliquam dolor, eu lacinia lorem placerat vulputate. Duis felis orci,
      pulvinar id metus ut, rutrum luctus orci. Cras porttitor imperdiet nunc,
      at ultricies tellus laoreet sit amet. Sed auctor cursus massa at porta.
      Integer ligula ipsum, tristique sit amet orci vel, viverra egestas ligula.
    </p>
  </div>

  <div class="card">
    <h2>I am the heading</h2>
    <p>
      Lorem ipsum dolor sit amet, consectetur adipiscing elit. Nulla luctus
      aliquam dolor, eu lacinia lorem placerat vulputate. Duis felis orci,
      pulvinar id metus ut, rutrum luctus orci. Cras porttitor imperdiet nunc,
      at ultricies tellus laoreet sit amet. Sed auctor cursus massa at porta.
      Integer ligula ipsum, tristique sit amet orci vel, viverra egestas ligula.
    </p>
  </div>

  <div class="card">
    <h2>I am the heading</h2>
    <p>
      Lorem ipsum dolor sit amet, consectetur adipiscing elit. Nulla luctus
      aliquam dolor, eu lacinia lorem placerat vulputate. Duis felis orci,
      pulvinar id metus ut, rutrum luctus orci. Cras porttitor imperdiet nunc,
      at ultricies tellus laoreet sit amet. Sed auctor cursus massa at porta.
      Integer ligula ipsum, tristique sit amet orci vel, viverra egestas ligula.
    </p>
  </div>

  <div class="card">
    <h2>I am the heading</h2>
    <p>
      Lorem ipsum dolor sit amet, consectetur adipiscing elit. Nulla luctus
      aliquam dolor, eu lacinia lorem placerat vulputate. Duis felis orci,
      pulvinar id metus ut, rutrum luctus orci. Cras porttitor imperdiet nunc,
      at ultricies tellus laoreet sit amet. Sed auctor cursus massa at porta.
      Integer ligula ipsum, tristique sit amet orci vel, viverra egestas ligula.
    </p>
  </div>
</div>
```

```css live-sample___fragmented-boxes live-sample___fragmented-boxes-fixed
.container {
  column-width: 250px;
  column-gap: 1em;
}

.card {
  background-color: rgb(207 232 220);
  border: 2px solid rgb(79 185 227);
  padding: 10px;
  margin-bottom: 1em;
}
```

{{ EmbedLiveSample('fragmented-boxes', '100%', 1000) }}

### Setting break-inside

To control this behavior, we can use properties from the [CSS Fragmentation](/en-US/docs/Web/CSS/Guides/Fragmentation) specification. This specification gives us properties to control the breaking of content in multicol and in paged media. For example, by adding the property {{cssxref("break-inside")}} with a value of `avoid` to the rules for `.card`. This is the container of the heading and text, so we don't want it fragmented.

```css live-sample___fragmented-boxes-fixed
.card {
  break-inside: avoid;
  background-color: rgb(207 232 220);
  border: 2px solid rgb(79 185 227);
  padding: 10px;
  margin-bottom: 1em;
}
```

The addition of this property causes the boxes to stay in one piece—they now do not _fragment_ across the columns.

{{ EmbedLiveSample('fragmented-boxes-fixed', '100%', 1100) }}

## Summary

You now know how to use the basic features of multiple-column layout, another tool at your disposal when choosing a layout method for the designs you're building.

## See also

- [CSS Fragmentation](/en-US/docs/Web/CSS/Guides/Fragmentation)
- [Using multi-column layouts](/en-US/docs/Web/CSS/Guides/Multicol_layout/Using)

`````

### Expected (prettier)

`````md
---
title: Multiple-column layout
slug: Learn_web_development/Core/CSS_layout/Multiple-column_Layout
page-type: learn-module-chapter
sidebar: learnsidebar
---

The multiple-column layout specification provides you with a method for laying content out in columns, as you might see in a newspaper. This article explains how to use this feature.

<table>
  <tbody>
    <tr>
      <th scope="row">Prerequisites:</th>
      <td>
        HTML basics (study
        <a href="/en-US/docs/Learn_web_development/Core/Structuring_content"
          >Structuring content with HTML</a
        >), and an idea of How CSS works (study
        <a href="/en-US/docs/Learn_web_development/Core/Styling_basics">CSS Styling basics</a>.)
      </td>
    </tr>
    <tr>
      <th scope="row">Objective:</th>
      <td>
        To learn how to create multiple-column layout on webpages, such as you
        might find in a newspaper.
      </td>
    </tr>
  </tbody>
</table>

## A basic example

Let's explore how to use multiple-column layout — often referred to as _multicol_ — by building up an example step by step. To follow along, create a new HTML on your local system and add the following content into it:

```html
<!DOCTYPE html>
<html lang="en-US">
  <head>
    <meta charset="utf-8" />
    <meta name="viewport" content="width=device-width" />
    <title>Multicol example</title>
    <style>
      body {
        width: 90%;
        max-width: 900px;
        margin: 2em auto;
        font:
          0.9em/1.2 "Arial",
          "Helvetica",
          sans-serif;
      }
    </style>
  </head>

  <body>
    <div class="container">
      <h1>Simple multicol example</h1>

      <p>
        Lorem ipsum dolor sit amet, consectetur adipiscing elit. Nulla luctus
        aliquam dolor, eu lacinia lorem placerat vulputate. Duis felis orci,
        pulvinar id metus ut, rutrum luctus orci. Cras porttitor imperdiet nunc,
        at ultricies tellus laoreet sit amet. Sed auctor cursus massa at porta.
        Integer ligula ipsum, tristique sit amet orci vel, viverra egestas
        ligula. Curabitur vehicula tellus neque, ac ornare ex malesuada et. In
        vitae convallis lacus. Aliquam erat volutpat. Suspendisse ac imperdiet
        turpis. Aenean finibus sollicitudin eros pharetra congue. Duis ornare
        egestas augue ut luctus. Proin blandit quam nec lacus varius commodo et
        a urna. Ut id ornare felis, eget fermentum sapien.
      </p>

      <p>
        Nam vulputate diam nec tempor bibendum. Donec luctus augue eget
        malesuada ultrices. Phasellus turpis est, posuere sit amet dapibus ut,
        facilisis sed est. Nam id risus quis ante semper consectetur eget
        aliquam lorem. Vivamus tristique elit dolor, sed pretium metus suscipit
        vel. Mauris ultricies lectus sed lobortis finibus. Vivamus eu urna eget
        velit cursus viverra quis vestibulum sem. Aliquam tincidunt eget purus
        in interdum. Cum sociis natoque penatibus et magnis dis parturient
        montes, nascetur ridiculus mus.
      </p>
    </div>
  </body>
</html>
```

You'll see various live examples below showing you what the rendered output should look like at each stage.

### A three-column layout

Our starting point file contains some very simple HTML: a wrapper with a class of `container`, inside of which is a heading and some paragraphs.

The {{htmlelement("div")}} with a class of container will become our multicol container. We enable multicol by using one of two properties: {{cssxref("column-count")}} or {{cssxref("column-width")}}. The `column-count` property takes a number as its value and creates that number of columns. If you add the following CSS to your stylesheet and reload the page, you'll get three columns:

```css live-sample___column-count
.container {
  column-count: 3;
}
```

The columns that you create have flexible widths — the browser works out how much space to assign each column.

```css hidden live-sample___column-count live-sample___column-width live-sample___column-styling live-sample___column-spanning
body {
  width: 90%;
  max-width: 900px;
  margin: 2em auto;
  font:
    0.9em/1.2 "Helvetica",
    "Arial",
    sans-serif;
}
```

```html hidden live-sample___column-count live-sample___column-width live-sample___column-styling
<div class="container">
  <h1>Simple multicol example</h1>

  <p>
    Lorem ipsum dolor sit amet, consectetur adipiscing elit. Nulla luctus
    aliquam dolor, eu lacinia lorem placerat vulputate. Duis felis orci,
    pulvinar id metus ut, rutrum luctus orci. Cras porttitor imperdiet nunc, at
    ultricies tellus laoreet sit amet. Sed auctor cursus massa at porta. Integer
    ligula ipsum, tristique sit amet orci vel, viverra egestas ligula. Curabitur
    vehicula tellus neque, ac ornare ex malesuada et. In vitae convallis lacus.
    Aliquam erat volutpat. Suspendisse ac imperdiet turpis. Aenean finibus
    sollicitudin eros pharetra congue. Duis ornare egestas augue ut luctus.
    Proin blandit quam nec lacus varius commodo et a urna. Ut id ornare felis,
    eget fermentum sapien.
  </p>

  <p>
    Nam vulputate diam nec tempor bibendum. Donec luctus augue eget malesuada
    ultrices. Phasellus turpis est, posuere sit amet dapibus ut, facilisis sed
    est. Nam id risus quis ante semper consectetur eget aliquam lorem. Vivamus
    tristique elit dolor, sed pretium metus suscipit vel. Mauris ultricies
    lectus sed lobortis finibus. Vivamus eu urna eget velit cursus viverra quis
    vestibulum sem. Aliquam tincidunt eget purus in interdum. Cum sociis natoque
    penatibus et magnis dis parturient montes, nascetur ridiculus mus.
  </p>
</div>
```

{{ EmbedLiveSample('column-count', '100%', 400) }}

### Setting column-width

Change your CSS to use `column-width` as follows:

```css live-sample___column-width
.container {
  column-width: 200px;
}
```

The browser will now give you as many columns as it can of the size that you specify; any remaining space is then shared between the existing columns. This means that you won't get exactly the width that you specify unless your container is exactly divisible by that width.

{{ EmbedLiveSample('column-width', '100%', 400) }}

## Styling the columns

The columns created by multicol cannot be styled individually. There's no way to make one column bigger than other columns or to change the background or text color of a single column. You have two opportunities to change the way that columns display:

- Changing the size of the gap between columns using the {{cssxref("column-gap")}}.
- Adding a rule between columns with {{cssxref("column-rule")}}.

Using your example above, change the size of the gap by adding a `column-gap` property. You can play around with different values — the property accepts any length unit.

Now add a rule between the columns with `column-rule`. In a similar way to the {{cssxref("border")}} property that you encountered in previous lessons, `column-rule` is a shorthand for {{cssxref("column-rule-color")}}, {{cssxref("column-rule-style")}}, and {{cssxref("column-rule-width")}}, and accepts the same values as `border`.

```css live-sample___column-styling live-sample___column-spanning
.container {
  column-count: 3;
  column-gap: 20px;
  column-rule: 4px dotted rgb(79 185 227);
}
```

Try adding rules of different styles and colors.

{{ EmbedLiveSample('column-styling', '100%', 400) }}

Something to take note of is that the rule doesn't take up any width of its own. It lies across the gap you created with `column-gap`. To make more space on either side of the rule, you'll need to increase the `column-gap` size.

## Spanning columns

You can cause an element to span across all the columns. In this case, the content breaks where the spanning element's introduced and then continues below the element, creating a new set of columns. To cause an element to span all the columns, specify the value of `all` for the {{cssxref("column-span")}} property.

> [!NOTE]
> It isn't possible to cause an element to span just _some_ columns. The property can only have the values of `none` (which is the default) or `all`.

Add the following rule to your CSS, below the previous ones:

```css live-sample___column-spanning
h2 {
  column-span: all;
  background-color: rgb(79 185 227);
  color: white;
  padding: 0.5em;
}
```

Now add a second-level heading between the first and second paragraphs:

```html
<h2>Spanning subhead</h2>
```

```html hidden live-sample___column-spanning
<div class="container">
  <h1>Simple multicol example</h1>

  <p>
    Lorem ipsum dolor sit amet, consectetur adipiscing elit. Nulla luctus
    aliquam dolor, eu lacinia lorem placerat vulputate. Duis felis orci,
    pulvinar id metus ut, rutrum luctus orci. Cras porttitor imperdiet nunc, at
    ultricies tellus laoreet sit amet. Sed auctor cursus massa at porta. Integer
    ligula ipsum, tristique sit amet orci vel, viverra egestas ligula.
  </p>

  <h2>Spanning subhead</h2>

  <p>
    Curabitur vehicula tellus neque, ac ornare ex malesuada et. In vitae
    convallis lacus. Aliquam erat volutpat. Suspendisse ac imperdiet turpis.
    Aenean finibus sollicitudin eros pharetra congue. Duis ornare egestas augue
    ut luctus. Proin blandit quam nec lacus varius commodo et a urna. Ut id
    ornare felis, eget fermentum sapien.
  </p>

  <p>
    Nam vulputate diam nec tempor bibendum. Donec luctus augue eget malesuada
    ultrices. Phasellus turpis est, posuere sit amet dapibus ut, facilisis sed
    est. Nam id risus quis ante semper consectetur eget aliquam lorem. Vivamus
    tristique elit dolor, sed pretium metus suscipit vel. Mauris ultricies
    lectus sed lobortis finibus. Vivamus eu urna eget velit cursus viverra quis
    vestibulum sem. Aliquam tincidunt eget purus in interdum. Cum sociis natoque
    penatibus et magnis dis parturient montes, nascetur ridiculus mus.
  </p>
</div>
```

You rendered code should now look like this:

{{ EmbedLiveSample('column-spanning', '100%', 550) }}

## Columns and fragmentation

The content of a multi-column layout is fragmented. It essentially behaves the same way as content behaves in paged media, such as when you print a webpage. When you turn your content into a multicol container, it fragments into columns. In order for the content to do this, it must _break_.

### Fragmented boxes

Sometimes, this breaking will happen in places that lead to a poor reading experience. In the example below, I have used multicol to lay out a series of boxes, each of which has a heading and some text inside. The heading becomes separated from the text if the columns fragment between the two.

```css hidden live-sample___fragmented-boxes live-sample___fragmented-boxes-fixed
body {
  width: 90%;
  max-width: 900px;
  margin: 2em auto;
  font:
    0.9em/1.2 "Helvetica",
    "Arial",
    sans-serif;
}
```

```html live-sample___fragmented-boxes live-sample___fragmented-boxes-fixed
<div class="container">
  <div class="card">
    <h2>I am the heading</h2>
    <p>
      Lorem ipsum dolor sit amet, consectetur adipiscing elit. Nulla luctus
      aliquam dolor, eu lacinia lorem placerat vulputate. Duis felis orci,
      pulvinar id metus ut, rutrum luctus orci. Cras porttitor imperdiet nunc,
      at ultricies tellus laoreet sit amet. Sed auctor cursus massa at porta.
      Integer ligula ipsum, tristique sit amet orci vel, viverra egestas ligula.
    </p>
  </div>

  <div class="card">
    <h2>I am the heading</h2>
    <p>
      Lorem ipsum dolor sit amet, consectetur adipiscing elit. Nulla luctus
      aliquam dolor, eu lacinia lorem placerat vulputate. Duis felis orci,
      pulvinar id metus ut, rutrum luctus orci. Cras porttitor imperdiet nunc,
      at ultricies tellus laoreet sit amet. Sed auctor cursus massa at porta.
      Integer ligula ipsum, tristique sit amet orci vel, viverra egestas ligula.
    </p>
  </div>

  <div class="card">
    <h2>I am the heading</h2>
    <p>
      Lorem ipsum dolor sit amet, consectetur adipiscing elit. Nulla luctus
      aliquam dolor, eu lacinia lorem placerat vulputate. Duis felis orci,
      pulvinar id metus ut, rutrum luctus orci. Cras porttitor imperdiet nunc,
      at ultricies tellus laoreet sit amet. Sed auctor cursus massa at porta.
      Integer ligula ipsum, tristique sit amet orci vel, viverra egestas ligula.
    </p>
  </div>
  <div class="card">
    <h2>I am the heading</h2>
    <p>
      Lorem ipsum dolor sit amet, consectetur adipiscing elit. Nulla luctus
      aliquam dolor, eu lacinia lorem placerat vulputate. Duis felis orci,
      pulvinar id metus ut, rutrum luctus orci. Cras porttitor imperdiet nunc,
      at ultricies tellus laoreet sit amet. Sed auctor cursus massa at porta.
      Integer ligula ipsum, tristique sit amet orci vel, viverra egestas ligula.
    </p>
  </div>

  <div class="card">
    <h2>I am the heading</h2>
    <p>
      Lorem ipsum dolor sit amet, consectetur adipiscing elit. Nulla luctus
      aliquam dolor, eu lacinia lorem placerat vulputate. Duis felis orci,
      pulvinar id metus ut, rutrum luctus orci. Cras porttitor imperdiet nunc,
      at ultricies tellus laoreet sit amet. Sed auctor cursus massa at porta.
      Integer ligula ipsum, tristique sit amet orci vel, viverra egestas ligula.
    </p>
  </div>

  <div class="card">
    <h2>I am the heading</h2>
    <p>
      Lorem ipsum dolor sit amet, consectetur adipiscing elit. Nulla luctus
      aliquam dolor, eu lacinia lorem placerat vulputate. Duis felis orci,
      pulvinar id metus ut, rutrum luctus orci. Cras porttitor imperdiet nunc,
      at ultricies tellus laoreet sit amet. Sed auctor cursus massa at porta.
      Integer ligula ipsum, tristique sit amet orci vel, viverra egestas ligula.
    </p>
  </div>

  <div class="card">
    <h2>I am the heading</h2>
    <p>
      Lorem ipsum dolor sit amet, consectetur adipiscing elit. Nulla luctus
      aliquam dolor, eu lacinia lorem placerat vulputate. Duis felis orci,
      pulvinar id metus ut, rutrum luctus orci. Cras porttitor imperdiet nunc,
      at ultricies tellus laoreet sit amet. Sed auctor cursus massa at porta.
      Integer ligula ipsum, tristique sit amet orci vel, viverra egestas ligula.
    </p>
  </div>
</div>
```

```css live-sample___fragmented-boxes live-sample___fragmented-boxes-fixed
.container {
  column-width: 250px;
  column-gap: 1em;
}

.card {
  background-color: rgb(207 232 220);
  border: 2px solid rgb(79 185 227);
  padding: 10px;
  margin-bottom: 1em;
}
```

{{ EmbedLiveSample('fragmented-boxes', '100%', 1000) }}

### Setting break-inside

To control this behavior, we can use properties from the [CSS Fragmentation](/en-US/docs/Web/CSS/Guides/Fragmentation) specification. This specification gives us properties to control the breaking of content in multicol and in paged media. For example, by adding the property {{cssxref("break-inside")}} with a value of `avoid` to the rules for `.card`. This is the container of the heading and text, so we don't want it fragmented.

```css live-sample___fragmented-boxes-fixed
.card {
  break-inside: avoid;
  background-color: rgb(207 232 220);
  border: 2px solid rgb(79 185 227);
  padding: 10px;
  margin-bottom: 1em;
}
```

The addition of this property causes the boxes to stay in one piece—they now do not _fragment_ across the columns.

{{ EmbedLiveSample('fragmented-boxes-fixed', '100%', 1100) }}

## Summary

You now know how to use the basic features of multiple-column layout, another tool at your disposal when choosing a layout method for the designs you're building.

## See also

- [CSS Fragmentation](/en-US/docs/Web/CSS/Guides/Fragmentation)
- [Using multi-column layouts](/en-US/docs/Web/CSS/Guides/Multicol_layout/Using)

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
@@ -33,9 +33,9 @@
 
 Let's explore how to use multiple-column layout — often referred to as _multicol_ — by building up an example step by step. To follow along, create a new HTML on your local system and add the following content into it:
 
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
title: Multiple-column layout
slug: Learn_web_development/Core/CSS_layout/Multiple-column_Layout
page-type: learn-module-chapter
sidebar: learnsidebar
---

The multiple-column layout specification provides you with a method for laying content out in columns, as you might see in a newspaper. This article explains how to use this feature.

<table>
  <tbody>
    <tr>
      <th scope="row">Prerequisites:</th>
      <td>
        HTML basics (study
        <a href="/en-US/docs/Learn_web_development/Core/Structuring_content"
          >Structuring content with HTML</a
        >), and an idea of How CSS works (study
        <a href="/en-US/docs/Learn_web_development/Core/Styling_basics">CSS Styling basics</a>.)
      </td>
    </tr>
    <tr>
      <th scope="row">Objective:</th>
      <td>
        To learn how to create multiple-column layout on webpages, such as you
        might find in a newspaper.
      </td>
    </tr>
  </tbody>
</table>

## A basic example

Let's explore how to use multiple-column layout — often referred to as _multicol_ — by building up an example step by step. To follow along, create a new HTML on your local system and add the following content into it:

```html
<!doctype html>
<html lang="en-US">
  <head>
    <meta charset="utf-8" />
    <meta name="viewport" content="width=device-width" />
    <title>Multicol example</title>
    <style>
      body {
        width: 90%;
        max-width: 900px;
        margin: 2em auto;
        font:
          0.9em/1.2 "Arial",
          "Helvetica",
          sans-serif;
      }
    </style>
  </head>

  <body>
    <div class="container">
      <h1>Simple multicol example</h1>

      <p>
        Lorem ipsum dolor sit amet, consectetur adipiscing elit. Nulla luctus aliquam dolor, eu
        lacinia lorem placerat vulputate. Duis felis orci, pulvinar id metus ut, rutrum luctus orci.
        Cras porttitor imperdiet nunc, at ultricies tellus laoreet sit amet. Sed auctor cursus massa
        at porta. Integer ligula ipsum, tristique sit amet orci vel, viverra egestas ligula.
        Curabitur vehicula tellus neque, ac ornare ex malesuada et. In vitae convallis lacus.
        Aliquam erat volutpat. Suspendisse ac imperdiet turpis. Aenean finibus sollicitudin eros
        pharetra congue. Duis ornare egestas augue ut luctus. Proin blandit quam nec lacus varius
        commodo et a urna. Ut id ornare felis, eget fermentum sapien.
      </p>

      <p>
        Nam vulputate diam nec tempor bibendum. Donec luctus augue eget malesuada ultrices.
        Phasellus turpis est, posuere sit amet dapibus ut, facilisis sed est. Nam id risus quis ante
        semper consectetur eget aliquam lorem. Vivamus tristique elit dolor, sed pretium metus
        suscipit vel. Mauris ultricies lectus sed lobortis finibus. Vivamus eu urna eget velit
        cursus viverra quis vestibulum sem. Aliquam tincidunt eget purus in interdum. Cum sociis
        natoque penatibus et magnis dis parturient montes, nascetur ridiculus mus.
      </p>
    </div>
  </body>
</html>
```

You'll see various live examples below showing you what the rendered output should look like at each stage.

### A three-column layout

Our starting point file contains some very simple HTML: a wrapper with a class of `container`, inside of which is a heading and some paragraphs.

The {{htmlelement("div")}} with a class of container will become our multicol container. We enable multicol by using one of two properties: {{cssxref("column-count")}} or {{cssxref("column-width")}}. The `column-count` property takes a number as its value and creates that number of columns. If you add the following CSS to your stylesheet and reload the page, you'll get three columns:

```css live-sample___column-count
.container {
  column-count: 3;
}
```

The columns that you create have flexible widths — the browser works out how much space to assign each column.

```css hidden live-sample___column-count live-sample___column-width live-sample___column-styling live-sample___column-spanning
body {
  width: 90%;
  max-width: 900px;
  margin: 2em auto;
  font:
    0.9em/1.2 "Helvetica",
    "Arial",
    sans-serif;
}
```

```html hidden live-sample___column-count live-sample___column-width live-sample___column-styling
<div class="container">
  <h1>Simple multicol example</h1>

  <p>
    Lorem ipsum dolor sit amet, consectetur adipiscing elit. Nulla luctus aliquam dolor, eu lacinia
    lorem placerat vulputate. Duis felis orci, pulvinar id metus ut, rutrum luctus orci. Cras
    porttitor imperdiet nunc, at ultricies tellus laoreet sit amet. Sed auctor cursus massa at
    porta. Integer ligula ipsum, tristique sit amet orci vel, viverra egestas ligula. Curabitur
    vehicula tellus neque, ac ornare ex malesuada et. In vitae convallis lacus. Aliquam erat
    volutpat. Suspendisse ac imperdiet turpis. Aenean finibus sollicitudin eros pharetra congue.
    Duis ornare egestas augue ut luctus. Proin blandit quam nec lacus varius commodo et a urna. Ut
    id ornare felis, eget fermentum sapien.
  </p>

  <p>
    Nam vulputate diam nec tempor bibendum. Donec luctus augue eget malesuada ultrices. Phasellus
    turpis est, posuere sit amet dapibus ut, facilisis sed est. Nam id risus quis ante semper
    consectetur eget aliquam lorem. Vivamus tristique elit dolor, sed pretium metus suscipit vel.
    Mauris ultricies lectus sed lobortis finibus. Vivamus eu urna eget velit cursus viverra quis
    vestibulum sem. Aliquam tincidunt eget purus in interdum. Cum sociis natoque penatibus et magnis
    dis parturient montes, nascetur ridiculus mus.
  </p>
</div>
```

{{ EmbedLiveSample('column-count', '100%', 400) }}

### Setting column-width

Change your CSS to use `column-width` as follows:

```css live-sample___column-width
.container {
  column-width: 200px;
}
```

The browser will now give you as many columns as it can of the size that you specify; any remaining space is then shared between the existing columns. This means that you won't get exactly the width that you specify unless your container is exactly divisible by that width.

{{ EmbedLiveSample('column-width', '100%', 400) }}

## Styling the columns

The columns created by multicol cannot be styled individually. There's no way to make one column bigger than other columns or to change the background or text color of a single column. You have two opportunities to change the way that columns display:

- Changing the size of the gap between columns using the {{cssxref("column-gap")}}.
- Adding a rule between columns with {{cssxref("column-rule")}}.

Using your example above, change the size of the gap by adding a `column-gap` property. You can play around with different values — the property accepts any length unit.

Now add a rule between the columns with `column-rule`. In a similar way to the {{cssxref("border")}} property that you encountered in previous lessons, `column-rule` is a shorthand for {{cssxref("column-rule-color")}}, {{cssxref("column-rule-style")}}, and {{cssxref("column-rule-width")}}, and accepts the same values as `border`.

```css live-sample___column-styling live-sample___column-spanning
.container {
  column-count: 3;
  column-gap: 20px;
  column-rule: 4px dotted rgb(79 185 227);
}
```

Try adding rules of different styles and colors.

{{ EmbedLiveSample('column-styling', '100%', 400) }}

Something to take note of is that the rule doesn't take up any width of its own. It lies across the gap you created with `column-gap`. To make more space on either side of the rule, you'll need to increase the `column-gap` size.

## Spanning columns

You can cause an element to span across all the columns. In this case, the content breaks where the spanning element's introduced and then continues below the element, creating a new set of columns. To cause an element to span all the columns, specify the value of `all` for the {{cssxref("column-span")}} property.

> [!NOTE]
> It isn't possible to cause an element to span just _some_ columns. The property can only have the values of `none` (which is the default) or `all`.

Add the following rule to your CSS, below the previous ones:

```css live-sample___column-spanning
h2 {
  column-span: all;
  background-color: rgb(79 185 227);
  color: white;
  padding: 0.5em;
}
```

Now add a second-level heading between the first and second paragraphs:

```html
<h2>Spanning subhead</h2>
```

```html hidden live-sample___column-spanning
<div class="container">
  <h1>Simple multicol example</h1>

  <p>
    Lorem ipsum dolor sit amet, consectetur adipiscing elit. Nulla luctus aliquam dolor, eu lacinia
    lorem placerat vulputate. Duis felis orci, pulvinar id metus ut, rutrum luctus orci. Cras
    porttitor imperdiet nunc, at ultricies tellus laoreet sit amet. Sed auctor cursus massa at
    porta. Integer ligula ipsum, tristique sit amet orci vel, viverra egestas ligula.
  </p>

  <h2>Spanning subhead</h2>

  <p>
    Curabitur vehicula tellus neque, ac ornare ex malesuada et. In vitae convallis lacus. Aliquam
    erat volutpat. Suspendisse ac imperdiet turpis. Aenean finibus sollicitudin eros pharetra
    congue. Duis ornare egestas augue ut luctus. Proin blandit quam nec lacus varius commodo et a
    urna. Ut id ornare felis, eget fermentum sapien.
  </p>

  <p>
    Nam vulputate diam nec tempor bibendum. Donec luctus augue eget malesuada ultrices. Phasellus
    turpis est, posuere sit amet dapibus ut, facilisis sed est. Nam id risus quis ante semper
    consectetur eget aliquam lorem. Vivamus tristique elit dolor, sed pretium metus suscipit vel.
    Mauris ultricies lectus sed lobortis finibus. Vivamus eu urna eget velit cursus viverra quis
    vestibulum sem. Aliquam tincidunt eget purus in interdum. Cum sociis natoque penatibus et magnis
    dis parturient montes, nascetur ridiculus mus.
  </p>
</div>
```

You rendered code should now look like this:

{{ EmbedLiveSample('column-spanning', '100%', 550) }}

## Columns and fragmentation

The content of a multi-column layout is fragmented. It essentially behaves the same way as content behaves in paged media, such as when you print a webpage. When you turn your content into a multicol container, it fragments into columns. In order for the content to do this, it must _break_.

### Fragmented boxes

Sometimes, this breaking will happen in places that lead to a poor reading experience. In the example below, I have used multicol to lay out a series of boxes, each of which has a heading and some text inside. The heading becomes separated from the text if the columns fragment between the two.

```css hidden live-sample___fragmented-boxes live-sample___fragmented-boxes-fixed
body {
  width: 90%;
  max-width: 900px;
  margin: 2em auto;
  font:
    0.9em/1.2 "Helvetica",
    "Arial",
    sans-serif;
}
```

```html live-sample___fragmented-boxes live-sample___fragmented-boxes-fixed
<div class="container">
  <div class="card">
    <h2>I am the heading</h2>
    <p>
      Lorem ipsum dolor sit amet, consectetur adipiscing elit. Nulla luctus aliquam dolor, eu
      lacinia lorem placerat vulputate. Duis felis orci, pulvinar id metus ut, rutrum luctus orci.
      Cras porttitor imperdiet nunc, at ultricies tellus laoreet sit amet. Sed auctor cursus massa
      at porta. Integer ligula ipsum, tristique sit amet orci vel, viverra egestas ligula.
    </p>
  </div>

  <div class="card">
    <h2>I am the heading</h2>
    <p>
      Lorem ipsum dolor sit amet, consectetur adipiscing elit. Nulla luctus aliquam dolor, eu
      lacinia lorem placerat vulputate. Duis felis orci, pulvinar id metus ut, rutrum luctus orci.
      Cras porttitor imperdiet nunc, at ultricies tellus laoreet sit amet. Sed auctor cursus massa
      at porta. Integer ligula ipsum, tristique sit amet orci vel, viverra egestas ligula.
    </p>
  </div>

  <div class="card">
    <h2>I am the heading</h2>
    <p>
      Lorem ipsum dolor sit amet, consectetur adipiscing elit. Nulla luctus aliquam dolor, eu
      lacinia lorem placerat vulputate. Duis felis orci, pulvinar id metus ut, rutrum luctus orci.
      Cras porttitor imperdiet nunc, at ultricies tellus laoreet sit amet. Sed auctor cursus massa
      at porta. Integer ligula ipsum, tristique sit amet orci vel, viverra egestas ligula.
    </p>
  </div>
  <div class="card">
    <h2>I am the heading</h2>
    <p>
      Lorem ipsum dolor sit amet, consectetur adipiscing elit. Nulla luctus aliquam dolor, eu
      lacinia lorem placerat vulputate. Duis felis orci, pulvinar id metus ut, rutrum luctus orci.
      Cras porttitor imperdiet nunc, at ultricies tellus laoreet sit amet. Sed auctor cursus massa
      at porta. Integer ligula ipsum, tristique sit amet orci vel, viverra egestas ligula.
    </p>
  </div>

  <div class="card">
    <h2>I am the heading</h2>
    <p>
      Lorem ipsum dolor sit amet, consectetur adipiscing elit. Nulla luctus aliquam dolor, eu
      lacinia lorem placerat vulputate. Duis felis orci, pulvinar id metus ut, rutrum luctus orci.
      Cras porttitor imperdiet nunc, at ultricies tellus laoreet sit amet. Sed auctor cursus massa
      at porta. Integer ligula ipsum, tristique sit amet orci vel, viverra egestas ligula.
    </p>
  </div>

  <div class="card">
    <h2>I am the heading</h2>
    <p>
      Lorem ipsum dolor sit amet, consectetur adipiscing elit. Nulla luctus aliquam dolor, eu
      lacinia lorem placerat vulputate. Duis felis orci, pulvinar id metus ut, rutrum luctus orci.
      Cras porttitor imperdiet nunc, at ultricies tellus laoreet sit amet. Sed auctor cursus massa
      at porta. Integer ligula ipsum, tristique sit amet orci vel, viverra egestas ligula.
    </p>
  </div>

  <div class="card">
    <h2>I am the heading</h2>
    <p>
      Lorem ipsum dolor sit amet, consectetur adipiscing elit. Nulla luctus aliquam dolor, eu
      lacinia lorem placerat vulputate. Duis felis orci, pulvinar id metus ut, rutrum luctus orci.
      Cras porttitor imperdiet nunc, at ultricies tellus laoreet sit amet. Sed auctor cursus massa
      at porta. Integer ligula ipsum, tristique sit amet orci vel, viverra egestas ligula.
    </p>
  </div>
</div>
```

```css live-sample___fragmented-boxes live-sample___fragmented-boxes-fixed
.container {
  column-width: 250px;
  column-gap: 1em;
}

.card {
  background-color: rgb(207 232 220);
  border: 2px solid rgb(79 185 227);
  padding: 10px;
  margin-bottom: 1em;
}
```

{{ EmbedLiveSample('fragmented-boxes', '100%', 1000) }}

### Setting break-inside

To control this behavior, we can use properties from the [CSS Fragmentation](/en-US/docs/Web/CSS/Guides/Fragmentation) specification. This specification gives us properties to control the breaking of content in multicol and in paged media. For example, by adding the property {{cssxref("break-inside")}} with a value of `avoid` to the rules for `.card`. This is the container of the heading and text, so we don't want it fragmented.

```css live-sample___fragmented-boxes-fixed
.card {
  break-inside: avoid;
  background-color: rgb(207 232 220);
  border: 2px solid rgb(79 185 227);
  padding: 10px;
  margin-bottom: 1em;
}
```

The addition of this property causes the boxes to stay in one piece—they now do not _fragment_ across the columns.

{{ EmbedLiveSample('fragmented-boxes-fixed', '100%', 1100) }}

## Summary

You now know how to use the basic features of multiple-column layout, another tool at your disposal when choosing a layout method for the designs you're building.

## See also

- [CSS Fragmentation](/en-US/docs/Web/CSS/Guides/Fragmentation)
- [Using multi-column layouts](/en-US/docs/Web/CSS/Guides/Multicol_layout/Using)

`````

### Expected (prettier)

`````md
---
title: Multiple-column layout
slug: Learn_web_development/Core/CSS_layout/Multiple-column_Layout
page-type: learn-module-chapter
sidebar: learnsidebar
---

The multiple-column layout specification provides you with a method for laying content out in columns, as you might see in a newspaper. This article explains how to use this feature.

<table>
  <tbody>
    <tr>
      <th scope="row">Prerequisites:</th>
      <td>
        HTML basics (study
        <a href="/en-US/docs/Learn_web_development/Core/Structuring_content"
          >Structuring content with HTML</a
        >), and an idea of How CSS works (study
        <a href="/en-US/docs/Learn_web_development/Core/Styling_basics">CSS Styling basics</a>.)
      </td>
    </tr>
    <tr>
      <th scope="row">Objective:</th>
      <td>
        To learn how to create multiple-column layout on webpages, such as you
        might find in a newspaper.
      </td>
    </tr>
  </tbody>
</table>

## A basic example

Let's explore how to use multiple-column layout — often referred to as _multicol_ — by building up an example step by step. To follow along, create a new HTML on your local system and add the following content into it:

```html
<!DOCTYPE html>
<html lang="en-US">
  <head>
    <meta charset="utf-8" />
    <meta name="viewport" content="width=device-width" />
    <title>Multicol example</title>
    <style>
      body {
        width: 90%;
        max-width: 900px;
        margin: 2em auto;
        font:
          0.9em/1.2 "Arial",
          "Helvetica",
          sans-serif;
      }
    </style>
  </head>

  <body>
    <div class="container">
      <h1>Simple multicol example</h1>

      <p>
        Lorem ipsum dolor sit amet, consectetur adipiscing elit. Nulla luctus aliquam dolor, eu
        lacinia lorem placerat vulputate. Duis felis orci, pulvinar id metus ut, rutrum luctus orci.
        Cras porttitor imperdiet nunc, at ultricies tellus laoreet sit amet. Sed auctor cursus massa
        at porta. Integer ligula ipsum, tristique sit amet orci vel, viverra egestas ligula.
        Curabitur vehicula tellus neque, ac ornare ex malesuada et. In vitae convallis lacus.
        Aliquam erat volutpat. Suspendisse ac imperdiet turpis. Aenean finibus sollicitudin eros
        pharetra congue. Duis ornare egestas augue ut luctus. Proin blandit quam nec lacus varius
        commodo et a urna. Ut id ornare felis, eget fermentum sapien.
      </p>

      <p>
        Nam vulputate diam nec tempor bibendum. Donec luctus augue eget malesuada ultrices.
        Phasellus turpis est, posuere sit amet dapibus ut, facilisis sed est. Nam id risus quis ante
        semper consectetur eget aliquam lorem. Vivamus tristique elit dolor, sed pretium metus
        suscipit vel. Mauris ultricies lectus sed lobortis finibus. Vivamus eu urna eget velit
        cursus viverra quis vestibulum sem. Aliquam tincidunt eget purus in interdum. Cum sociis
        natoque penatibus et magnis dis parturient montes, nascetur ridiculus mus.
      </p>
    </div>
  </body>
</html>
```

You'll see various live examples below showing you what the rendered output should look like at each stage.

### A three-column layout

Our starting point file contains some very simple HTML: a wrapper with a class of `container`, inside of which is a heading and some paragraphs.

The {{htmlelement("div")}} with a class of container will become our multicol container. We enable multicol by using one of two properties: {{cssxref("column-count")}} or {{cssxref("column-width")}}. The `column-count` property takes a number as its value and creates that number of columns. If you add the following CSS to your stylesheet and reload the page, you'll get three columns:

```css live-sample___column-count
.container {
  column-count: 3;
}
```

The columns that you create have flexible widths — the browser works out how much space to assign each column.

```css hidden live-sample___column-count live-sample___column-width live-sample___column-styling live-sample___column-spanning
body {
  width: 90%;
  max-width: 900px;
  margin: 2em auto;
  font:
    0.9em/1.2 "Helvetica",
    "Arial",
    sans-serif;
}
```

```html hidden live-sample___column-count live-sample___column-width live-sample___column-styling
<div class="container">
  <h1>Simple multicol example</h1>

  <p>
    Lorem ipsum dolor sit amet, consectetur adipiscing elit. Nulla luctus aliquam dolor, eu lacinia
    lorem placerat vulputate. Duis felis orci, pulvinar id metus ut, rutrum luctus orci. Cras
    porttitor imperdiet nunc, at ultricies tellus laoreet sit amet. Sed auctor cursus massa at
    porta. Integer ligula ipsum, tristique sit amet orci vel, viverra egestas ligula. Curabitur
    vehicula tellus neque, ac ornare ex malesuada et. In vitae convallis lacus. Aliquam erat
    volutpat. Suspendisse ac imperdiet turpis. Aenean finibus sollicitudin eros pharetra congue.
    Duis ornare egestas augue ut luctus. Proin blandit quam nec lacus varius commodo et a urna. Ut
    id ornare felis, eget fermentum sapien.
  </p>

  <p>
    Nam vulputate diam nec tempor bibendum. Donec luctus augue eget malesuada ultrices. Phasellus
    turpis est, posuere sit amet dapibus ut, facilisis sed est. Nam id risus quis ante semper
    consectetur eget aliquam lorem. Vivamus tristique elit dolor, sed pretium metus suscipit vel.
    Mauris ultricies lectus sed lobortis finibus. Vivamus eu urna eget velit cursus viverra quis
    vestibulum sem. Aliquam tincidunt eget purus in interdum. Cum sociis natoque penatibus et magnis
    dis parturient montes, nascetur ridiculus mus.
  </p>
</div>
```

{{ EmbedLiveSample('column-count', '100%', 400) }}

### Setting column-width

Change your CSS to use `column-width` as follows:

```css live-sample___column-width
.container {
  column-width: 200px;
}
```

The browser will now give you as many columns as it can of the size that you specify; any remaining space is then shared between the existing columns. This means that you won't get exactly the width that you specify unless your container is exactly divisible by that width.

{{ EmbedLiveSample('column-width', '100%', 400) }}

## Styling the columns

The columns created by multicol cannot be styled individually. There's no way to make one column bigger than other columns or to change the background or text color of a single column. You have two opportunities to change the way that columns display:

- Changing the size of the gap between columns using the {{cssxref("column-gap")}}.
- Adding a rule between columns with {{cssxref("column-rule")}}.

Using your example above, change the size of the gap by adding a `column-gap` property. You can play around with different values — the property accepts any length unit.

Now add a rule between the columns with `column-rule`. In a similar way to the {{cssxref("border")}} property that you encountered in previous lessons, `column-rule` is a shorthand for {{cssxref("column-rule-color")}}, {{cssxref("column-rule-style")}}, and {{cssxref("column-rule-width")}}, and accepts the same values as `border`.

```css live-sample___column-styling live-sample___column-spanning
.container {
  column-count: 3;
  column-gap: 20px;
  column-rule: 4px dotted rgb(79 185 227);
}
```

Try adding rules of different styles and colors.

{{ EmbedLiveSample('column-styling', '100%', 400) }}

Something to take note of is that the rule doesn't take up any width of its own. It lies across the gap you created with `column-gap`. To make more space on either side of the rule, you'll need to increase the `column-gap` size.

## Spanning columns

You can cause an element to span across all the columns. In this case, the content breaks where the spanning element's introduced and then continues below the element, creating a new set of columns. To cause an element to span all the columns, specify the value of `all` for the {{cssxref("column-span")}} property.

> [!NOTE]
> It isn't possible to cause an element to span just _some_ columns. The property can only have the values of `none` (which is the default) or `all`.

Add the following rule to your CSS, below the previous ones:

```css live-sample___column-spanning
h2 {
  column-span: all;
  background-color: rgb(79 185 227);
  color: white;
  padding: 0.5em;
}
```

Now add a second-level heading between the first and second paragraphs:

```html
<h2>Spanning subhead</h2>
```

```html hidden live-sample___column-spanning
<div class="container">
  <h1>Simple multicol example</h1>

  <p>
    Lorem ipsum dolor sit amet, consectetur adipiscing elit. Nulla luctus aliquam dolor, eu lacinia
    lorem placerat vulputate. Duis felis orci, pulvinar id metus ut, rutrum luctus orci. Cras
    porttitor imperdiet nunc, at ultricies tellus laoreet sit amet. Sed auctor cursus massa at
    porta. Integer ligula ipsum, tristique sit amet orci vel, viverra egestas ligula.
  </p>

  <h2>Spanning subhead</h2>

  <p>
    Curabitur vehicula tellus neque, ac ornare ex malesuada et. In vitae convallis lacus. Aliquam
    erat volutpat. Suspendisse ac imperdiet turpis. Aenean finibus sollicitudin eros pharetra
    congue. Duis ornare egestas augue ut luctus. Proin blandit quam nec lacus varius commodo et a
    urna. Ut id ornare felis, eget fermentum sapien.
  </p>

  <p>
    Nam vulputate diam nec tempor bibendum. Donec luctus augue eget malesuada ultrices. Phasellus
    turpis est, posuere sit amet dapibus ut, facilisis sed est. Nam id risus quis ante semper
    consectetur eget aliquam lorem. Vivamus tristique elit dolor, sed pretium metus suscipit vel.
    Mauris ultricies lectus sed lobortis finibus. Vivamus eu urna eget velit cursus viverra quis
    vestibulum sem. Aliquam tincidunt eget purus in interdum. Cum sociis natoque penatibus et magnis
    dis parturient montes, nascetur ridiculus mus.
  </p>
</div>
```

You rendered code should now look like this:

{{ EmbedLiveSample('column-spanning', '100%', 550) }}

## Columns and fragmentation

The content of a multi-column layout is fragmented. It essentially behaves the same way as content behaves in paged media, such as when you print a webpage. When you turn your content into a multicol container, it fragments into columns. In order for the content to do this, it must _break_.

### Fragmented boxes

Sometimes, this breaking will happen in places that lead to a poor reading experience. In the example below, I have used multicol to lay out a series of boxes, each of which has a heading and some text inside. The heading becomes separated from the text if the columns fragment between the two.

```css hidden live-sample___fragmented-boxes live-sample___fragmented-boxes-fixed
body {
  width: 90%;
  max-width: 900px;
  margin: 2em auto;
  font:
    0.9em/1.2 "Helvetica",
    "Arial",
    sans-serif;
}
```

```html live-sample___fragmented-boxes live-sample___fragmented-boxes-fixed
<div class="container">
  <div class="card">
    <h2>I am the heading</h2>
    <p>
      Lorem ipsum dolor sit amet, consectetur adipiscing elit. Nulla luctus aliquam dolor, eu
      lacinia lorem placerat vulputate. Duis felis orci, pulvinar id metus ut, rutrum luctus orci.
      Cras porttitor imperdiet nunc, at ultricies tellus laoreet sit amet. Sed auctor cursus massa
      at porta. Integer ligula ipsum, tristique sit amet orci vel, viverra egestas ligula.
    </p>
  </div>

  <div class="card">
    <h2>I am the heading</h2>
    <p>
      Lorem ipsum dolor sit amet, consectetur adipiscing elit. Nulla luctus aliquam dolor, eu
      lacinia lorem placerat vulputate. Duis felis orci, pulvinar id metus ut, rutrum luctus orci.
      Cras porttitor imperdiet nunc, at ultricies tellus laoreet sit amet. Sed auctor cursus massa
      at porta. Integer ligula ipsum, tristique sit amet orci vel, viverra egestas ligula.
    </p>
  </div>

  <div class="card">
    <h2>I am the heading</h2>
    <p>
      Lorem ipsum dolor sit amet, consectetur adipiscing elit. Nulla luctus aliquam dolor, eu
      lacinia lorem placerat vulputate. Duis felis orci, pulvinar id metus ut, rutrum luctus orci.
      Cras porttitor imperdiet nunc, at ultricies tellus laoreet sit amet. Sed auctor cursus massa
      at porta. Integer ligula ipsum, tristique sit amet orci vel, viverra egestas ligula.
    </p>
  </div>
  <div class="card">
    <h2>I am the heading</h2>
    <p>
      Lorem ipsum dolor sit amet, consectetur adipiscing elit. Nulla luctus aliquam dolor, eu
      lacinia lorem placerat vulputate. Duis felis orci, pulvinar id metus ut, rutrum luctus orci.
      Cras porttitor imperdiet nunc, at ultricies tellus laoreet sit amet. Sed auctor cursus massa
      at porta. Integer ligula ipsum, tristique sit amet orci vel, viverra egestas ligula.
    </p>
  </div>

  <div class="card">
    <h2>I am the heading</h2>
    <p>
      Lorem ipsum dolor sit amet, consectetur adipiscing elit. Nulla luctus aliquam dolor, eu
      lacinia lorem placerat vulputate. Duis felis orci, pulvinar id metus ut, rutrum luctus orci.
      Cras porttitor imperdiet nunc, at ultricies tellus laoreet sit amet. Sed auctor cursus massa
      at porta. Integer ligula ipsum, tristique sit amet orci vel, viverra egestas ligula.
    </p>
  </div>

  <div class="card">
    <h2>I am the heading</h2>
    <p>
      Lorem ipsum dolor sit amet, consectetur adipiscing elit. Nulla luctus aliquam dolor, eu
      lacinia lorem placerat vulputate. Duis felis orci, pulvinar id metus ut, rutrum luctus orci.
      Cras porttitor imperdiet nunc, at ultricies tellus laoreet sit amet. Sed auctor cursus massa
      at porta. Integer ligula ipsum, tristique sit amet orci vel, viverra egestas ligula.
    </p>
  </div>

  <div class="card">
    <h2>I am the heading</h2>
    <p>
      Lorem ipsum dolor sit amet, consectetur adipiscing elit. Nulla luctus aliquam dolor, eu
      lacinia lorem placerat vulputate. Duis felis orci, pulvinar id metus ut, rutrum luctus orci.
      Cras porttitor imperdiet nunc, at ultricies tellus laoreet sit amet. Sed auctor cursus massa
      at porta. Integer ligula ipsum, tristique sit amet orci vel, viverra egestas ligula.
    </p>
  </div>
</div>
```

```css live-sample___fragmented-boxes live-sample___fragmented-boxes-fixed
.container {
  column-width: 250px;
  column-gap: 1em;
}

.card {
  background-color: rgb(207 232 220);
  border: 2px solid rgb(79 185 227);
  padding: 10px;
  margin-bottom: 1em;
}
```

{{ EmbedLiveSample('fragmented-boxes', '100%', 1000) }}

### Setting break-inside

To control this behavior, we can use properties from the [CSS Fragmentation](/en-US/docs/Web/CSS/Guides/Fragmentation) specification. This specification gives us properties to control the breaking of content in multicol and in paged media. For example, by adding the property {{cssxref("break-inside")}} with a value of `avoid` to the rules for `.card`. This is the container of the heading and text, so we don't want it fragmented.

```css live-sample___fragmented-boxes-fixed
.card {
  break-inside: avoid;
  background-color: rgb(207 232 220);
  border: 2px solid rgb(79 185 227);
  padding: 10px;
  margin-bottom: 1em;
}
```

The addition of this property causes the boxes to stay in one piece—they now do not _fragment_ across the columns.

{{ EmbedLiveSample('fragmented-boxes-fixed', '100%', 1100) }}

## Summary

You now know how to use the basic features of multiple-column layout, another tool at your disposal when choosing a layout method for the designs you're building.

## See also

- [CSS Fragmentation](/en-US/docs/Web/CSS/Guides/Fragmentation)
- [Using multi-column layouts](/en-US/docs/Web/CSS/Guides/Multicol_layout/Using)

`````
