# externals/mdn-css-guides/values_and_units/textual_data_types/index.md

## Option 1

`````json
{"printWidth":80,"bracketSameLine":true}
`````

### Diff

`````diff
===================================================================
--- prettier
+++ oxfmt
@@ -16,9 +16,9 @@
 ```css
 @keyframes validIdent {
   /* keyframes go here */
 }
-@keyframes 'validString' {
+@keyframes "validString" {
   /* keyframes go here */
 }
 ```
 

`````

### Actual (oxfmt)

`````md
---
title: Textual data types
slug: Web/CSS/Guides/Values_and_units/Textual_data_types
page-type: guide
sidebar: cssref
---

Every CSS declaration consists of a property/value pair. The value can include various data types depending on the property, such as a single keyword, integer, function, or a combination of different types; some values have units, while others do not. This guide provides an overview of the textual data types. Refer to the page for each value type for more detailed information.

Text data types are either {{cssxref("&lt;string&gt;")}}, a quoted series of characters, an {{cssxref("&lt;ident&gt;")}}, a "CSS Identifier" which is an unquoted string, or an optionally quoted {{cssxref("url_value", "&lt;url&gt;")}}. A `<string>` is quoted with either single or double quotes. CSS identifiers, listed in the specifications as `<ident>` or {{cssxref("&lt;custom-ident&gt;")}}, must be unquoted.

In CSS specifications, values that can be defined by the web developer — such as keyframe animation names, font-family names, or grid areas — are listed as a {{cssxref("&lt;custom-ident&gt;")}}, {{cssxref("&lt;string&gt;")}}, or both.

When both quoted and unquoted user-defined text values are permitted, the specification will list `<custom-ident> | <string>`, meaning quotes are optional, such as is the case with [keyframe animation names]:

```css
@keyframes validIdent {
  /* keyframes go here */
}
@keyframes "validString" {
  /* keyframes go here */
}
```

Some text values are not valid if encompassed in quotes. For example, the value of {{cssxref("grid-area")}} can be a `<custom-ident>`, so if we had a grid area named `content` we would use it without quotes:

```css
.item {
  grid-area: content;
}
```

In comparison, a data type that is a {{cssxref("&lt;string&gt;")}}, such as a string value of the {{cssxref("content")}} property, must be quoted:

```css
.item::after {
  content: "This is my content.";
}
```

While you can generally create any name you want, including using emojis, the identifier can't be `none`, `unset`, `initial`, or `inherit`, start with a digit or two dashes, and generally you don't want it to be any other pre-defined CSS keyword. See the {{cssxref("&lt;custom-ident&gt;")}} and {{cssxref("&lt;string&gt;")}} reference pages for more details.

## Pre-defined keyword values

Pre-defined keywords are text values defined by the specification for that property. These keywords are also CSS identifiers, and are therefore used without quotes.

When viewing CSS property value syntax in a CSS specification or the MDN property page, allowable keywords will be listed in the following form. The following {{glossary("enumerated")}} values are the pre-defined keyword values allowed for {{cssxref("float")}}.

```plain
left | right | none | inline-start | inline-end
```

Such values are used without quotes:

```css
.box {
  float: left;
}
```

## CSS-wide keywords

In addition to the pre-defined keywords that are part of the specification for a property, all CSS properties accept the CSS-wide, or "global", property values {{cssxref("initial")}}, {{cssxref("inherit")}}, {{cssxref("unset")}}, {{cssxref("revert")}}, and {{cssxref("revert-layer")}}, which explicitly specify defaulting behaviors.

- {{cssxref("initial")}}
  - : Represents the value specified as the property's initial value.
- {{cssxref("inherit")}}
  - : Represents the computed value of the property on the element's parent, provided it is inherited.
- {{cssxref("unset")}}
  - : Acts as either `inherit` or `initial`, depending on whether the property is inherited or not.
- {{cssxref("revert")}}
  - : Resets the property to its inherited value if it inherits from its parent or to the default value established by the user agent's stylesheet (or by user styles if any exist).
- {{cssxref("revert-layer")}}
  - : Rolls back the value of a property in a [cascade layer](/en-US/docs/Web/CSS/Reference/At-rules/@layer) to the value of the property in a CSS rule matching the element in a previous cascade layer. The value of the property with this keyword is recalculated as if no rules were specified on the target element in the current cascade layer.

## URLs

A {{cssxref("url_value", "&lt;url&gt;")}} type uses functional notation, which accepts a `<string>` that is a URL. This may be an absolute URL or a relative URL. For example, if you wanted to include a background image, you might use either of the following:

```css
.box {
  background-image: url("images/my-background.png");
}

.box {
  background-image: url("https://www.exammple.com/images/my-background.png");
}
```

The parameter for `url()` can be either quoted or unquoted. If unquoted, it is parsed as a `<url-token>`, which has extra requirements including escaping certain characters. See {{cssxref("url_value", "&lt;url&gt;")}} for more information.

## See also

- [Numeric data types](/en-US/docs/Web/CSS/Guides/Values_and_units/Numeric_data_types)
- [CSS data types](/en-US/docs/Web/CSS/Reference/Values/Data_types)
- [CSS values and units](/en-US/docs/Web/CSS/Guides/Values_and_units) module
- [Learn: Values and units](/en-US/docs/Learn_web_development/Core/Styling_basics/Values_and_units)
- [CSS cascade and inheritance](/en-US/docs/Web/CSS/Guides/Cascade) module

`````

### Expected (prettier)

`````md
---
title: Textual data types
slug: Web/CSS/Guides/Values_and_units/Textual_data_types
page-type: guide
sidebar: cssref
---

Every CSS declaration consists of a property/value pair. The value can include various data types depending on the property, such as a single keyword, integer, function, or a combination of different types; some values have units, while others do not. This guide provides an overview of the textual data types. Refer to the page for each value type for more detailed information.

Text data types are either {{cssxref("&lt;string&gt;")}}, a quoted series of characters, an {{cssxref("&lt;ident&gt;")}}, a "CSS Identifier" which is an unquoted string, or an optionally quoted {{cssxref("url_value", "&lt;url&gt;")}}. A `<string>` is quoted with either single or double quotes. CSS identifiers, listed in the specifications as `<ident>` or {{cssxref("&lt;custom-ident&gt;")}}, must be unquoted.

In CSS specifications, values that can be defined by the web developer — such as keyframe animation names, font-family names, or grid areas — are listed as a {{cssxref("&lt;custom-ident&gt;")}}, {{cssxref("&lt;string&gt;")}}, or both.

When both quoted and unquoted user-defined text values are permitted, the specification will list `<custom-ident> | <string>`, meaning quotes are optional, such as is the case with [keyframe animation names]:

```css
@keyframes validIdent {
  /* keyframes go here */
}
@keyframes 'validString' {
  /* keyframes go here */
}
```

Some text values are not valid if encompassed in quotes. For example, the value of {{cssxref("grid-area")}} can be a `<custom-ident>`, so if we had a grid area named `content` we would use it without quotes:

```css
.item {
  grid-area: content;
}
```

In comparison, a data type that is a {{cssxref("&lt;string&gt;")}}, such as a string value of the {{cssxref("content")}} property, must be quoted:

```css
.item::after {
  content: "This is my content.";
}
```

While you can generally create any name you want, including using emojis, the identifier can't be `none`, `unset`, `initial`, or `inherit`, start with a digit or two dashes, and generally you don't want it to be any other pre-defined CSS keyword. See the {{cssxref("&lt;custom-ident&gt;")}} and {{cssxref("&lt;string&gt;")}} reference pages for more details.

## Pre-defined keyword values

Pre-defined keywords are text values defined by the specification for that property. These keywords are also CSS identifiers, and are therefore used without quotes.

When viewing CSS property value syntax in a CSS specification or the MDN property page, allowable keywords will be listed in the following form. The following {{glossary("enumerated")}} values are the pre-defined keyword values allowed for {{cssxref("float")}}.

```plain
left | right | none | inline-start | inline-end
```

Such values are used without quotes:

```css
.box {
  float: left;
}
```

## CSS-wide keywords

In addition to the pre-defined keywords that are part of the specification for a property, all CSS properties accept the CSS-wide, or "global", property values {{cssxref("initial")}}, {{cssxref("inherit")}}, {{cssxref("unset")}}, {{cssxref("revert")}}, and {{cssxref("revert-layer")}}, which explicitly specify defaulting behaviors.

- {{cssxref("initial")}}
  - : Represents the value specified as the property's initial value.
- {{cssxref("inherit")}}
  - : Represents the computed value of the property on the element's parent, provided it is inherited.
- {{cssxref("unset")}}
  - : Acts as either `inherit` or `initial`, depending on whether the property is inherited or not.
- {{cssxref("revert")}}
  - : Resets the property to its inherited value if it inherits from its parent or to the default value established by the user agent's stylesheet (or by user styles if any exist).
- {{cssxref("revert-layer")}}
  - : Rolls back the value of a property in a [cascade layer](/en-US/docs/Web/CSS/Reference/At-rules/@layer) to the value of the property in a CSS rule matching the element in a previous cascade layer. The value of the property with this keyword is recalculated as if no rules were specified on the target element in the current cascade layer.

## URLs

A {{cssxref("url_value", "&lt;url&gt;")}} type uses functional notation, which accepts a `<string>` that is a URL. This may be an absolute URL or a relative URL. For example, if you wanted to include a background image, you might use either of the following:

```css
.box {
  background-image: url("images/my-background.png");
}

.box {
  background-image: url("https://www.exammple.com/images/my-background.png");
}
```

The parameter for `url()` can be either quoted or unquoted. If unquoted, it is parsed as a `<url-token>`, which has extra requirements including escaping certain characters. See {{cssxref("url_value", "&lt;url&gt;")}} for more information.

## See also

- [Numeric data types](/en-US/docs/Web/CSS/Guides/Values_and_units/Numeric_data_types)
- [CSS data types](/en-US/docs/Web/CSS/Reference/Values/Data_types)
- [CSS values and units](/en-US/docs/Web/CSS/Guides/Values_and_units) module
- [Learn: Values and units](/en-US/docs/Learn_web_development/Core/Styling_basics/Values_and_units)
- [CSS cascade and inheritance](/en-US/docs/Web/CSS/Guides/Cascade) module

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
@@ -16,9 +16,9 @@
 ```css
 @keyframes validIdent {
   /* keyframes go here */
 }
-@keyframes 'validString' {
+@keyframes "validString" {
   /* keyframes go here */
 }
 ```
 

`````

### Actual (oxfmt)

`````md
---
title: Textual data types
slug: Web/CSS/Guides/Values_and_units/Textual_data_types
page-type: guide
sidebar: cssref
---

Every CSS declaration consists of a property/value pair. The value can include various data types depending on the property, such as a single keyword, integer, function, or a combination of different types; some values have units, while others do not. This guide provides an overview of the textual data types. Refer to the page for each value type for more detailed information.

Text data types are either {{cssxref("&lt;string&gt;")}}, a quoted series of characters, an {{cssxref("&lt;ident&gt;")}}, a "CSS Identifier" which is an unquoted string, or an optionally quoted {{cssxref("url_value", "&lt;url&gt;")}}. A `<string>` is quoted with either single or double quotes. CSS identifiers, listed in the specifications as `<ident>` or {{cssxref("&lt;custom-ident&gt;")}}, must be unquoted.

In CSS specifications, values that can be defined by the web developer — such as keyframe animation names, font-family names, or grid areas — are listed as a {{cssxref("&lt;custom-ident&gt;")}}, {{cssxref("&lt;string&gt;")}}, or both.

When both quoted and unquoted user-defined text values are permitted, the specification will list `<custom-ident> | <string>`, meaning quotes are optional, such as is the case with [keyframe animation names]:

```css
@keyframes validIdent {
  /* keyframes go here */
}
@keyframes "validString" {
  /* keyframes go here */
}
```

Some text values are not valid if encompassed in quotes. For example, the value of {{cssxref("grid-area")}} can be a `<custom-ident>`, so if we had a grid area named `content` we would use it without quotes:

```css
.item {
  grid-area: content;
}
```

In comparison, a data type that is a {{cssxref("&lt;string&gt;")}}, such as a string value of the {{cssxref("content")}} property, must be quoted:

```css
.item::after {
  content: "This is my content.";
}
```

While you can generally create any name you want, including using emojis, the identifier can't be `none`, `unset`, `initial`, or `inherit`, start with a digit or two dashes, and generally you don't want it to be any other pre-defined CSS keyword. See the {{cssxref("&lt;custom-ident&gt;")}} and {{cssxref("&lt;string&gt;")}} reference pages for more details.

## Pre-defined keyword values

Pre-defined keywords are text values defined by the specification for that property. These keywords are also CSS identifiers, and are therefore used without quotes.

When viewing CSS property value syntax in a CSS specification or the MDN property page, allowable keywords will be listed in the following form. The following {{glossary("enumerated")}} values are the pre-defined keyword values allowed for {{cssxref("float")}}.

```plain
left | right | none | inline-start | inline-end
```

Such values are used without quotes:

```css
.box {
  float: left;
}
```

## CSS-wide keywords

In addition to the pre-defined keywords that are part of the specification for a property, all CSS properties accept the CSS-wide, or "global", property values {{cssxref("initial")}}, {{cssxref("inherit")}}, {{cssxref("unset")}}, {{cssxref("revert")}}, and {{cssxref("revert-layer")}}, which explicitly specify defaulting behaviors.

- {{cssxref("initial")}}
  - : Represents the value specified as the property's initial value.
- {{cssxref("inherit")}}
  - : Represents the computed value of the property on the element's parent, provided it is inherited.
- {{cssxref("unset")}}
  - : Acts as either `inherit` or `initial`, depending on whether the property is inherited or not.
- {{cssxref("revert")}}
  - : Resets the property to its inherited value if it inherits from its parent or to the default value established by the user agent's stylesheet (or by user styles if any exist).
- {{cssxref("revert-layer")}}
  - : Rolls back the value of a property in a [cascade layer](/en-US/docs/Web/CSS/Reference/At-rules/@layer) to the value of the property in a CSS rule matching the element in a previous cascade layer. The value of the property with this keyword is recalculated as if no rules were specified on the target element in the current cascade layer.

## URLs

A {{cssxref("url_value", "&lt;url&gt;")}} type uses functional notation, which accepts a `<string>` that is a URL. This may be an absolute URL or a relative URL. For example, if you wanted to include a background image, you might use either of the following:

```css
.box {
  background-image: url("images/my-background.png");
}

.box {
  background-image: url("https://www.exammple.com/images/my-background.png");
}
```

The parameter for `url()` can be either quoted or unquoted. If unquoted, it is parsed as a `<url-token>`, which has extra requirements including escaping certain characters. See {{cssxref("url_value", "&lt;url&gt;")}} for more information.

## See also

- [Numeric data types](/en-US/docs/Web/CSS/Guides/Values_and_units/Numeric_data_types)
- [CSS data types](/en-US/docs/Web/CSS/Reference/Values/Data_types)
- [CSS values and units](/en-US/docs/Web/CSS/Guides/Values_and_units) module
- [Learn: Values and units](/en-US/docs/Learn_web_development/Core/Styling_basics/Values_and_units)
- [CSS cascade and inheritance](/en-US/docs/Web/CSS/Guides/Cascade) module

`````

### Expected (prettier)

`````md
---
title: Textual data types
slug: Web/CSS/Guides/Values_and_units/Textual_data_types
page-type: guide
sidebar: cssref
---

Every CSS declaration consists of a property/value pair. The value can include various data types depending on the property, such as a single keyword, integer, function, or a combination of different types; some values have units, while others do not. This guide provides an overview of the textual data types. Refer to the page for each value type for more detailed information.

Text data types are either {{cssxref("&lt;string&gt;")}}, a quoted series of characters, an {{cssxref("&lt;ident&gt;")}}, a "CSS Identifier" which is an unquoted string, or an optionally quoted {{cssxref("url_value", "&lt;url&gt;")}}. A `<string>` is quoted with either single or double quotes. CSS identifiers, listed in the specifications as `<ident>` or {{cssxref("&lt;custom-ident&gt;")}}, must be unquoted.

In CSS specifications, values that can be defined by the web developer — such as keyframe animation names, font-family names, or grid areas — are listed as a {{cssxref("&lt;custom-ident&gt;")}}, {{cssxref("&lt;string&gt;")}}, or both.

When both quoted and unquoted user-defined text values are permitted, the specification will list `<custom-ident> | <string>`, meaning quotes are optional, such as is the case with [keyframe animation names]:

```css
@keyframes validIdent {
  /* keyframes go here */
}
@keyframes 'validString' {
  /* keyframes go here */
}
```

Some text values are not valid if encompassed in quotes. For example, the value of {{cssxref("grid-area")}} can be a `<custom-ident>`, so if we had a grid area named `content` we would use it without quotes:

```css
.item {
  grid-area: content;
}
```

In comparison, a data type that is a {{cssxref("&lt;string&gt;")}}, such as a string value of the {{cssxref("content")}} property, must be quoted:

```css
.item::after {
  content: "This is my content.";
}
```

While you can generally create any name you want, including using emojis, the identifier can't be `none`, `unset`, `initial`, or `inherit`, start with a digit or two dashes, and generally you don't want it to be any other pre-defined CSS keyword. See the {{cssxref("&lt;custom-ident&gt;")}} and {{cssxref("&lt;string&gt;")}} reference pages for more details.

## Pre-defined keyword values

Pre-defined keywords are text values defined by the specification for that property. These keywords are also CSS identifiers, and are therefore used without quotes.

When viewing CSS property value syntax in a CSS specification or the MDN property page, allowable keywords will be listed in the following form. The following {{glossary("enumerated")}} values are the pre-defined keyword values allowed for {{cssxref("float")}}.

```plain
left | right | none | inline-start | inline-end
```

Such values are used without quotes:

```css
.box {
  float: left;
}
```

## CSS-wide keywords

In addition to the pre-defined keywords that are part of the specification for a property, all CSS properties accept the CSS-wide, or "global", property values {{cssxref("initial")}}, {{cssxref("inherit")}}, {{cssxref("unset")}}, {{cssxref("revert")}}, and {{cssxref("revert-layer")}}, which explicitly specify defaulting behaviors.

- {{cssxref("initial")}}
  - : Represents the value specified as the property's initial value.
- {{cssxref("inherit")}}
  - : Represents the computed value of the property on the element's parent, provided it is inherited.
- {{cssxref("unset")}}
  - : Acts as either `inherit` or `initial`, depending on whether the property is inherited or not.
- {{cssxref("revert")}}
  - : Resets the property to its inherited value if it inherits from its parent or to the default value established by the user agent's stylesheet (or by user styles if any exist).
- {{cssxref("revert-layer")}}
  - : Rolls back the value of a property in a [cascade layer](/en-US/docs/Web/CSS/Reference/At-rules/@layer) to the value of the property in a CSS rule matching the element in a previous cascade layer. The value of the property with this keyword is recalculated as if no rules were specified on the target element in the current cascade layer.

## URLs

A {{cssxref("url_value", "&lt;url&gt;")}} type uses functional notation, which accepts a `<string>` that is a URL. This may be an absolute URL or a relative URL. For example, if you wanted to include a background image, you might use either of the following:

```css
.box {
  background-image: url("images/my-background.png");
}

.box {
  background-image: url("https://www.exammple.com/images/my-background.png");
}
```

The parameter for `url()` can be either quoted or unquoted. If unquoted, it is parsed as a `<url-token>`, which has extra requirements including escaping certain characters. See {{cssxref("url_value", "&lt;url&gt;")}} for more information.

## See also

- [Numeric data types](/en-US/docs/Web/CSS/Guides/Values_and_units/Numeric_data_types)
- [CSS data types](/en-US/docs/Web/CSS/Reference/Values/Data_types)
- [CSS values and units](/en-US/docs/Web/CSS/Guides/Values_and_units) module
- [Learn: Values and units](/en-US/docs/Learn_web_development/Core/Styling_basics/Values_and_units)
- [CSS cascade and inheritance](/en-US/docs/Web/CSS/Guides/Cascade) module

`````
