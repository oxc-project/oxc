# externals/mdn-css-guides/scroll-driven_animations/timeline_insets/index.md

## Option 1

`````json
{"printWidth":80,"bracketSameLine":true}
`````

### Diff

`````diff
===================================================================
--- prettier
+++ oxfmt
@@ -216,9 +216,10 @@
     transparent 59.5px,
     blue 59.5px 60.5px,
     transparent 60.5px 74.5px,
     red 74.5px 75.5px,
-    transparent 75.5px /* 174.5px,
+    transparent 75.5px
+      /* 174.5px,
     green 174.5px 154.5px,
     transparent 175.5px*/
   );
 }

`````

### Actual (oxfmt)

`````md
---
title: Understanding timeline insets
slug: Web/CSS/Guides/Scroll-driven_animations/Timeline_insets
page-type: guide
sidebar: cssref
---

By default, [view progress timelines](/en-US/docs/Web/CSS/Guides/Scroll-driven_animations/Timelines#view_progress_timelines) track elements across the entire [animation attachment range](/en-US/docs/Web/CSS/Guides/Scroll-driven_animations/Timeline_range_names#the_animation_attachment_range). The `0%` progress point is at the start of the range, while the `100%` progress point is at the end. The animation attachment range can be changed by setting a [timeline range name](/en-US/docs/Web/CSS/Guides/Scroll-driven_animations/Timeline_range_names), and the location of the `0%` and `100%` progress points along the range can be adjusted by setting length or percentage-based inset values.

This guide explains how to limit the animation timeline to a specific portion of the animation timeline range using length or percentage inset values.

## Animation timelines: a primer

[CSS animations](/en-US/docs/Web/CSS/Guides/Animations) are created by defining named {{cssxref("@keyframes")}} animations, which specify an animation's behavior, and then attaching the keyframe animation to an element using the animation's name.

The element's animation timeline, defined by the {{cssxref("animation-timeline")}} property, determines how and when the element progresses through those keyframes. By default, the timeline is time-based, using the document's default time-based {{domxref("DocumentTimeline")}}.

The [CSS scroll-driven animation](/en-US/docs/Web/CSS/Guides/Scroll-driven_animations) module defines scroll progress and view progress timelines, which are methods for animating property values along a scroll-based timeline rather than the default time-based document timeline. In this article, we'll only discuss view progress timelines, as [scroll progress timelines](/en-US/docs/Web/CSS/Guides/Scroll-driven_animations/Timelines#scroll_progress_timelines) are not relevant to timeline insets.

### View progress timelines

With [view progress timelines](/en-US/docs/Web/CSS/Guides/Scroll-driven_animations/Timelines#view_progress_timelines), the animation's timeline, or progress, is driven by element visibility instead of the passage of time, with keyframe progression tied to the subject element position and visibility within the scroll container. The animation advances and reverses as the element advances or reverses through the scrollport. The animation only occurs when at least part of the element is visible within its scrollport, pausing when scrolling pauses.

```css live-sample___svg_view
.animated_element {
  animation-name: nameOfAnimation;
  animation-timeline: view();
}
```

Setting an {{cssxref("animation-name")}} applies the animation to the selected element.

> [!NOTE]
> The `animation-timeline` property should always come after any `animation` shorthand declarations. While the shorthand property can't be used to set the `animation-timeline` property, it does reset the timeline to the default time-based document timeline.

> [!NOTE]
> In all the examples, the {{glossary("scroll container")}} is `250px` tall and we are using the default values for {{cssxref("animation-iteration-count")}} (`1`), {{cssxref("animation-delay")}} (`0s`), and {{cssxref("animation-direction")}} (`normal`). We set the {{cssxref("animation-timing-function")}} to `step-end` and the {{cssxref("animation-fill-mode")}} is to `forward` to make the it more apparent when the animation iteration has not yet started, when it is active, and when it is complete. See the [Using CSS animations guide](/en-US/docs/Web/CSS/Guides/Animations/Using) to learn more.

As you scroll up, the animation progresses. As you scroll down, the animation reverses.

{{EmbedLiveSample("initial", "100%", "400")}}

In this example, whenever any part of the subject element is visible in the scrollport, the animation occurs. By default, view progress animations begin just as the top edge of the subject element aligns with the bottom edge of the scroll container and end, reaching `100%` progress, when the end edge aligns with the start edge of the container, no matter the size of the subject element. By default, the animation is applied when any part of the subject is visible within the scrollport.

### Animation attachment ranges

In a [view progress timeline](/en-US/docs/Web/CSS/Guides/Scroll-driven_animations/Timelines#view_progress_timelines), when no animation range properties are defined, the `<timeline-range-name>` is `normal`, which defaults to `cover`. The animation is applied whenever any portion of the subject element is visible, meaning the default **animation attachment range** is the sum of the height of the scroll container and the height of the subject element, with that extra height being at the scroll end edge. In our example, as the scroll container is `250px` tall, and the subject is `50px`, `250px`, or `500px` tall, with the vertical animation attachment range being `300px`, `500px`, or `750px` respectively.

The `0%` progression occurs when the subject element's start edge intersects the scrollport at the end edge, reaching `100%` progress when the subject's end edge exits via the scrollport's start edge. These are the subject and scrollport's top and bottom edges when scrolling vertically, and the left and right or right and left edges when scrolling horizontally, depending on the writing mode.

The following diagram illustrates the position of the subject at the `0%` and `100%` progress points for the three subject sizes:

```html hidden live-sample___svg_view
<div>
  <svg viewBox="-1 -1 462 1252" xmlns="http://www.w3.org/2000/svg">
    <title>Default view progress timeline</title>
    <rect class="container" width="350" height="250" x="0" y="500" />
    <rect class="small end" width="100" height="50" x="10" y="450" />
    <rect class="medium end" width="100" height="250" x="125" y="250" />
    <rect class="large end" width="100" height="500" x="240" y="0" />
    <rect class="small start" width="100" height="50" x="10" y="750" />
    <rect class="medium start" width="100" height="250" x="125" y="750" />
    <rect class="large start" width="100" height="500" x="240" y="750" />
    <text y="520" x="360">100%</text>
    <line x1="0" x2="350" y1="500" y2="500" />
    <line x1="0" x2="350" y1="750" y2="750" />
    <text y="760" x="360">0%</text>
  </svg>
</div>
```

{{EmbedLiveSample("svg_view", "100%", "720")}}

The yellow subject elements represents the position of the element when the `from` keyframe is applied, which is the animation range's `0%` progress mark. The red represents the location of the animated element relative to the scrollport when the `to` keyframe is applied, which is the end of the animation, or the `100%` progress mark. The gray represents the scrollport.

By default, the element animates while it is "in view", but this default definition of "in view" may not fit your needs. Fortunately, we can control which edges define the edges of the animation attachment range and then offset the start and end of that range with the animation range properties.

### Animation range properties

The {{cssxref("animation-range")}} properties enable specifying a named timeline range, such as `contain` or `exit-crossing`, which changes the range used from the default `cover` range. You can also include a {{cssxref("length-percentage")}} value, which insets the attachment range from the start of the range. Percentages are relative to the named or default timeline range.

Named timeline ranges define the portions of a {{domxref("ViewTimeline")}} that define an animation's range, specifying the start and end of the animations's attachment range.

The `animation-range` property is a shorthand property, defining the {{cssxref("animation-range-start")}} and {{cssxref("animation-range-end")}} properties. The `animation-range-start` defines the position of the subject element when the animation starts. The `animation-range-end` defines the position of the subject element when the animation ends.

See the [timeline range names guide](/en-US/docs/Web/CSS/Guides/Scroll-driven_animations/Timeline_range_names) to learn about the different named timeline ranges. This guide focuses on how the {{cssxref("length-percentage")}} inset values work.

## Setting insets using lengths

The `animation-range-start` and `animation-range-end` properties each accept a named animation range, a {{cssxref("length-percentage")}} offset value, or both. Any length or percentage offset is measured from the _start_ of the animation attachment range.

When a {{cssxref("length")}} is set, the offset is fairly intuitive.
Here we use the `animation-range-start` and `animation-range-end` properties to inset the animation timeline. This defines a subsection of the element's full animation attachment range as the active interval, with the `<length>` values specifying distances from the start of the default `normal` animation attachment range.

```css live-sample___inset_length
.animated_element {
  animation-range-start: 1em;
  animation-range-end: 125px;
}
```

The start and end of the animation range are `1em` and `125px` from the start of the animation attachment range, respectively. Because the timeline range default is `normal`, which resolves to `cover`, the start of the animation attachment range is the block end edge of the container.

```css hidden live-sample___inset_length
:root {
  --start: 1em;
  --end: 125px;
}

article {
  background-image: linear-gradient(
    to top,
    transparent calc(var(--start) - 1px),
    #cccccc calc(var(--start) - 1px) calc(var(--start) + 1px),
    transparent calc(var(--start) + 1px) calc(var(--end) - 1px),
    #cccccc calc(var(--end) - 1px) calc(var(--end) + 1px),
    transparent calc(var(--end) + 1px)
  );
}
```

{{EmbedLiveSample("inset_length", "100%", "400")}}

We've added lines `1em` and `125px` from the block end edge of the scroll container. The animation starts when the block start edge of the subject element reaches the `1em` line and ends when it reaches the `125px` line.

In this case, as the animation attachment range resolves to `cover` for both the start and end inset values, the location of the insets is fairly straightforward.

### Effect of named ranges on length offsets

The offset distance is always from the start of the associated animation range. In this example, we set the `animation-range-start` to be `50px` from the start of the default `normal` range and set the `animation-range-end` to be `100px` from the start of the explicitly set `entry` range:

```css live-sample___different_length
.animated_element {
  animation-range-start: 50px;
  animation-range-end: entry 100px;
}
```

```html hidden live-sample___different_length live-sample___exit_length live-sample___exit_percent live-sample___center
<main>
  <article>
    <p>&nbsp;</p>
    <p>&nbsp;</p>
    <p>Scroll down ⇩</p>
    <p>&nbsp;</p>
    <p>&nbsp;</p>
    <section class="triple">
      <div>
        <i id="A" class="animated_element">50px</i>
        <i id="B" class="animated_element">250px</i>
        <i id="C" class="animated_element">500px</i>
      </div>
    </section>
    <p>&nbsp;</p>
    <p>&nbsp;</p>
    <p>&nbsp;</p>
    <p>&nbsp;</p>
    <p>Scroll up ⇧</p>
  </article>
</main>
```

{{EmbedLiveSample("different_length", "100%", "310")}}

As the start edge of both the `normal` and `entry` ranges is the container's end edge, the animation begins when the subject start edge is `50px` from the bottom of the scrollport and ends, reaching `100%` progress, when the subject start edge is `100px` from the bottom of the scrollport, irrespective of the subject size. While the size of the `entry` range is different for the three different subject sizes, in this case, the size of the underlying range didn't matter.

### Length offsets with varying ranges

The size of range matters when the range doesn't start at the element's end edge, as is the case with `exit` and `exit-crossing`, or if the offset is a percentage value. This fact, and the fact that you can mix and match animation range names, make view progress timeline offsets a bit more complicated to understand than non-offsetted [timeline range names](/en-US/docs/Web/CSS/Guides/Scroll-driven_animations/Timeline_range_names).

For example, when setting `exit` as the timeline range name, the subject size matters as it determines the location of the range's end edge.

```css live-sample___exit_length
.animated_element {
  animation-range-start: entry 60px;
  animation-range-end: exit 75px;
}
```

With both `entry` and `exit`, the range is the size of the subject, with the size clamped to the size of the scrollport. This means that the height of the `entry` and `exit` ranges are the height of the box in both the `50px` and `250px` examples, while in the `500px` example, the range is clamped to the height of the scrollport, which is `250px` tall.

{{EmbedLiveSample("exit_length", "100%", "310")}}

We added a few lines to facilitate the explanations that follow: the bottom blue line is `60px` from the end edge of the scrollport, and the top red line is `75px` from that same edge. These are where the animation range start and end respectively.

This example demonstrates several important features, which we'll explain in greater detail, including:

- Offsets are [measured from their respective named ranges](#measured_from_the_start_edge_of_the_range)
- Offsets can occur [beyond the edges of the scrollport](#beyond_the_scrollport_edges)
- [Ranges may be clamped](#effects_of_clamping) if the subject is larger than the scrollport

#### Measured from the start edge of the range

As the offset position is always relative to the start of the declaration's animation range, the start of the animation for all three elements occurs when the elements start edge crosses the point that is `60px` from the start of the `entry` range.

The `animation-range-end` value defines the position where the animation. The `exit 75px` value basically means "when `75px` of the subject has exited the start edge of the scrollport." This varies for each subject. For the `50px` subject, this only occurs `25px` after it has left the scrollport; when the element is not visible. The animation range end for both the `250px` and `500px` subjects occurs when their bottom end edge intersects the top blue line; 75 pixels from the end edge of the scrollport. Why are their end offsets the same? Because of [clamping](#effects_of_clamping)! The maximum size of the named animation range is clamped to the size of the scrollport. The `exit` range for both subjects is the same, so the range end offsets are the same.

#### Beyond the scrollport edges

For our `50px` tall subject, the `exit` range is 50px tall abutting the start edge of the scroll port. Setting `animation-range-end: exit 75px` for any element less than `75px` tall means the end of the range is outside the scrollport, as the point `75px` from the start of the `exit` range is past the start edge of the scrollport. In our example, the end of the animation range for the `50px` subject occurs when the subject's start edge is `75px` past the scrollport's start edge. The animation ends, reaching the `to` keyframe and the {{domxref("Element/animationend_event", "animationend")}} event, only when (and if) the element is scrolled `25px` out of view.

The animation ends even if the animation range end is outside of the scrollport, as long as there is room to scroll to that point. Had we set `animation-range-end: exit 250px`, the animation would have ended when the end edge of the medium and tall subjects exited the scrollport at the container's start edge.

With the end set to `exit 250px`, the small subject's animation might not end, as there might not be `450px` of content after the subject for the user to scroll to before the end point is reached.

#### Effects of clamping

With our `250px` tall container, when the subject is `250px` or `500px` tall, the `exit` range is the size of the container, with the start being the scroll container's end edge. With a `75px` offset, the end of the animation occurs when the end edge of the subject is `75px` from the end edge of the scroll container (denoted by the top red line).

As the offset position is always relative to the start of the named or default animation range, in our example, clamping impacts the large subject's `animation-range-end`. We set the end of the range to `exit 75px` which is `75px` from the start edge of the `exit` range. When the subject is the same size as the scrollport (our `250px` subject) or larger (our `500px` subject), the animation range end is `75px` from the end edge of the scrollport, which is `75px` from the start of the scrollport-clamped `exit` range.

```css hidden live-sample___exit_length
article {
  background-image: linear-gradient(
    to top,
    transparent 59.5px,
    blue 59.5px 60.5px,
    transparent 60.5px 74.5px,
    red 74.5px 75.5px,
    transparent 75.5px
      /* 174.5px,
    green 174.5px 154.5px,
    transparent 175.5px*/
  );
}
.animated_element {
  align-self: flex-end;
}
```

```css hidden live-sample___different_length live-sample___exit_length live-sample___exit_percent live-sample___center
@layer setup {
  #A {
    height: 50px;
  }
  #B {
    height: 250px;
  }
  #C {
    height: 500px;
  }
  div {
    display: flex;
    gap: 1em;
  }
  main {
    padding: 20px 0 0 20px;
    margin-bottom: 2em;
  }
  article {
    outline: 3px dashed;
    width: 475px;
    margin: auto;
    overflow: scroll;
    position: relative;
    height: 250px;
    box-sizing: content-box;
    background-image: linear-gradient(
      to top,
      transparent 49.5px,
      #666666 49.5px 50.5px,
      transparent 50.5px 99.5px,
      #666666 99.5px 100.5px,
      transparent 100.5px
    );
    background-origin: content-box;
  }

  p {
    padding: 10px;
    margin: 10px;
  }

  .animated_element {
    --clr: yellow;
    background-color: hsl(from var(--clr) h s calc(l * 1.4));
    display: block;
    animation: showAnim step-end 1 forwards;
    animation-timeline: view();
    flex: 1 0 auto;
  }

  i {
    font-family: sans-serif;
    font-size: 1.5rem;
  }

  @keyframes showAnim {
    from {
      --clr: green;
    }
    to {
      --clr: red;
    }
  }
  @layer no-support {
    @supports not (animation-timeline: view()) {
      body::before {
        content: "Your browser doesn't support view progress scrolling.";
        background-color: wheat;
        display: block;
        text-align: center;
      }
    }
  }
}
```

### Negative lengths

Up to this point, all the offsets have been greater than zero. It's important to note that negative lengths are valid. A negative offset on the `animation-range-start` makes the range longer while a negative offset on the `animation-range-end` makes the range shorter.

Let's compare the negative insets compared to the `0` values:

```css live-sample___exit_length_negative
#A {
  animation-range-start: contain -25px;
  animation-range-end: exit -25px;
}
#B {
  animation-range-start: contain 0;
  animation-range-end: exit 0;
}
```

{{EmbedLiveSample("exit_length_negative", "100%", "380")}}

The first animation range is offset by `25px` toward the container end edge.

```css hidden live-sample___exit_length_negative
fieldset.double {
  display: none;
}
#A::after {
  content: " (-25px)";
}
#B::after {
  content: " (0)";
}
```

## Setting insets using percentages

Like length values, percentage values define offsets from the _start_ of the animation attachment range. The percentage offsets are relative to the timeline range dimension, not relative to the scrollport. For this reason, percentage values are not as intuitive as length values for most people (realizing length values weren't that intuitive either).

Here, we use `animation-range-start` and `animation-range-end` to inset the animation timeline. While we are using the same properties, we set `<percentage>` values instead of `<length>` values:

```css live-sample___inset_percent
.animated_element {
  animation-range-start: 20%;
  animation-range-end: 60%;
}
```

```css hidden live-sample___inset_percent live-sample___inset_cover
i {
  background-image: linear-gradient(
    to bottom,
    transparent calc(20% - 1px),
    #33333333 calc(20% - 1px) calc(20% + 1px),
    transparent calc(20% + 1px) calc(60% - 1px),
    #33333333 calc(60% - 1px) calc(60% + 1px),
    transparent calc(60% + 1px)
  );
}
article {
  --total: calc(var(--animElHeight) + 250px);
  background-image:
    linear-gradient(
      to top,
      transparent 0 calc(var(--total) * 0.2 - 1px),
      green calc(var(--total) * 0.2 - 1px) calc((var(--total) * 0.2) + 1px),
      transparent calc(var(--total) * 0.2 + 1px)
    ),
    linear-gradient(
      to top,
      transparent 0 calc(var(--total) * 0.6 - 1px),
      red calc(var(--total) * 0.6 - 1px) calc((var(--total) * 0.6) + 1px),
      transparent calc(var(--total) * 0.6 + 1px)
    ),
    linear-gradient(
      to top,
      transparent 0 calc(var(--containerHeight) * 0.2 - 0.5px),
      #33333333 calc(var(--containerHeight) * 0.2 - 0.5px)
        calc(var(--containerHeight) * 0.2 + 0.5px),
      transparent calc(var(--containerHeight) * 0.2 + 0.5px)
        calc(var(--containerHeight) * 0.6 - 0.5px),
      #33333333 calc(var(--containerHeight) * 0.6 - 0.5px)
        calc(var(--containerHeight) * 0.6 + 0.5px),
      transparent 0 calc(var(--containerHeight) * 0.6 + 0.5px)
    );
  background-attachment: local, local, fixed;
}
```

This defines the active interval to begin `20%` into the default attachment range and end `60%` through that same range. The default `normal` animation attachment range, which behaves as [`cover`](/en-US/docs/Web/CSS/Guides/Scroll-driven_animations/Timeline_range_names#cover), is the height of the scroll container plus the height of the subject element, meaning the range will differ based on which radio button is selected.

{{EmbedLiveSample("inset_percent", "100%", "400")}}

For illustrative purposes, there are two dark lines crossing the container at the `20%` and `60%` points of the full animation range. The animation starts when the block-start edge reaches the `20%` point, which is the bottom green line. The animation ends when the start block edge is 60% of the way through the normal range, which is the top red line.

Only when the element is `50px` tall is the top of the subject still in the scrollport when the end of the animation is reached; there are no top red lines when `250px` or `500px` are selected, as the end of the animation range is outside of the scrollport.

Based on the height of our subjects, the `20%` mark is either `60px`, `100px`, or `150px` from the end edge of the scrollport (marked by the green line, which is always in the scrollport), and the `60%` mark is `180px`, `300px`, or `450px` from the same point (marked with a red line, but only visible for the `50px` subject).

For illustrative purposes, there are two light gray lines crossing the container `20%` and `60%` of the way through the scrollport, which are `50px` and `150px` from the bottom of the scrollport, respectively. As the `animation-range-*` percentages are relative to the timeline range, not the scrollport, these lines only show how the percentages **don't** align. We've also included two horizontal light gray lines going across each subject at their own `20%` and `60%` marks. These lines align with the scrollport's light gray lines when each subjects animation starts and ends.

The following image demonstrates where the subject elements are located when the animation starts (the `0%` keyframe) and ends (the `100% keyframe`).
This image includes the insets from the animation timeline in the previous demonstration and the timeline without insets for comparison.

```html hidden live-sample___svg_insets2
<div>
  <svg viewBox="-1 -1 482 1252" xmlns="http://www.w3.org/2000/svg">
    <title>Default view progress timeline with insets</title>
    <rect class="container" width="350" height="250" x="0" y="500" />
    <rect class="small end" width="100" height="50" x="10" y="571" />
    <rect class="medium end" width="100" height="250" x="120" y="450" />
    <rect class="large end" width="100" height="500" x="230" y="300" />
    <rect class="small start" width="100" height="50" x="10" y="689" />
    <rect class="medium start" width="100" height="250" x="120" y="649" />
    <rect class="large start" width="100" height="500" x="230" y="600" />
    <rect width="96" height="48" x="122" y="602" fill="url(#g)" />
    <rect width="96" height="198" x="232" y="527" fill="url(#g)" />
    <text y="610" x="385">60%</text>
    <line x1="0" x2="385" y1="600" y2="600" />
    <line x1="0" x2="385" y1="700" y2="700" />
    <text y="710" x="385">20%</text>
  </svg>
  <svg viewBox="-1 -1 482 1252" xmlns="http://www.w3.org/2000/svg">
    <title>Default view progress timeline</title>
    <rect class="container" width="350" height="250" x="0" y="500" />
    <rect class="small end" width="100" height="50" x="10" y="450" />
    <rect class="medium end" width="100" height="250" x="125" y="250" />
    <rect class="large end" width="100" height="500" x="240" y="0" />
    <rect class="small start" width="100" height="50" x="10" y="750" />
    <rect class="medium start" width="100" height="250" x="125" y="750" />
    <rect class="large start" width="100" height="500" x="240" y="750" />
    <text y="520" x="385">100%</text>
    <line x1="0" x2="385" y1="500" y2="500" />
    <line x1="0" x2="385" y1="750" y2="750" />
    <text y="760" x="390">0%</text>
  </svg>
</div>
```

{{EmbedLiveSample("svg_insets2", "100%", "710")}}

As before, the yellow represents the position of the element when the `from` keyframe is applied, the red represents the location when the `to` keyframe is applied, and the gray represents the scrollport. The striped areas are where the red and yellow element representations overlap. For illustrative purposes, we've added dashed black horizontal lines `20%` and `60%` way through the scrollport, starting from the bottom.

The animation only begins when the element reaches the `20%` mark along the animation attachment range. This point is `60px`, `100px`, or `150px` from the bottom edge of the scroll port, depending on the size of the element. The location of the subject element at this point, representing the position of the element when the `from` or `0%` keyframe is applied, is shown in yellow.

The red represents the location of the animated element relative to the scrollport when the `to` or `100%` keyframe is applied, which is the end of the animation. This point is either `180px`, `300px`, or `450px` from the bottom edge of the scrollport, depending on the subject size. The animation occurs when the element is between the `to` and the `from` positions.

You may have noticed something interesting about the dashed horizontal lines: when the animation starts, the line that is `20%` from the end edge of the viewport is `20%` from the _top_ of the subject element and the line that is `60%` from the end edge of the viewport is `60%` from the _top_ of the subject element when the animation ends. This is what was illustrated by the very light gray lines in the live demo for this example.

### Subject size matters

As we saw when we [set insets with lengths](#setting_insets_using_lengths), the size of the subject can make a difference. When setting animation ranges, percentage values are relative to the size of animation attachment range, not the scrollport. For most named ranges, the size of the attachment range depends partially on the subject size. As percentages are based on the size of the range, the named range impacts the resolved size of the insets. Depending on the name, the start position may also change, impacting the location of the range and therefore the location of progress points.

In this example, we define an active range that is `40%` of the size of the subject:

```css live-sample___exit_percent
.animated_element {
  animation-range-start: exit-crossing -20%;
  animation-range-end: exit-crossing 20%;
}
```

```css hidden live-sample___exit_percent
article {
  background-image: none;
}
body .animated_element {
  align-self: start;
}
```

{{EmbedLiveSample("exit_percent", "100%", "400")}}

The animation lasts `40%` of the animation-attachment range. As you scroll, note how the larger the subject, the longer the range. With exit-crossing, the animation range is not cropped; it is the size of the subject even if the subject is larger than the viewport, with the range abutting the start edge of the scrollport, and extending off the end edge if the subject is larger than the scrollport.

With the `-20%` and `20%` insets, the `50px` subject's will animation over `20px`: the animation starts when the subject's end is `-10px` from range start, or `60px` from exiting the screen, and ends when the subject's end is `40px` from exiting the screen. The middle subject will animate over `100px`: the animation starts when the subject end is `-50px` from range start, which is `50px` off of the scrollport's end edge, and ends when the subject's end is `50px` into the scrollport. The large subject animates over `200px`, starting when the bottom is `600px` from the container's start edge, with only `150px` in view, and ends when the bottom is 400px from that start edge, when `100px` have scrolled off the start edge.

### Percentages equal to the scrollport

When it comes to offsetting with percentages, the least complicated named timeline range is `contain`. With `contain`, the animation range is the size of the scrollport, meaning the start and end percentages are relative to the scrollport. For this reason, when using offsets, you may want to use `contain` instead of letting the range default and resolve to `cover`.

The `contain` range fully _contains_ the animation within the scrollport. It represents the range during which the principal box is either fully contained by, or fully covers, its view progress visibility range within the scrollport. With `contain`, if the subject is the same size or smaller than the scrollport, it can be fully visible. If the element is the same size as the container, however, the animation occurs over `0px`. This means that it runs, but it is not visible to the user.

In other words, without needing to know the size of the container or the subjects, we are able to limit our animation to the middle of scrollport, though the animation will happen over `0px` if the subject is the same size as the scrollport.

```css live-sample___center
.animated_element {
  animation-range-start: contain 25%;
  animation-range-end: contain 75%;
}
```

```css hidden live-sample___center
article {
  background-image: linear-gradient(
    transparent 25%,
    #ededed 25% 75%,
    transparent 75%
  );
}
body .animated_element {
  align-self: center;
}

.animated_element {
  background-image:
    linear-gradient(black, black), linear-gradient(black, black);
  background-size: 1px 1px;
  background-position:
    center 25%,
    center 75%;
  background-repeat: repeat-x;
```

{{EmbedLiveSample("center", "100%", "310")}}

The horizontal lines denote the middle half of the scrollport and the middle half of each subject.

```html hidden live-sample___svg_contain live-sample___svg_insets2 live-sample___svg_view
<svg class="gradient">
  <title>Striped repeating gradient</title>
  <defs>
    <linearGradient
      id="g"
      x1="0"
      y1="0"
      x2="20"
      y2="20"
      spreadMethod="repeat"
      gradientUnits="userSpaceOnUse">
      <stop offset="50%" stop-color="red" />
      <stop offset="50%" stop-color="yellow" />
    </linearGradient>
  </defs>
</svg>
```

```css hidden live-sample___svg_contain live-sample___svg_insets2 live-sample___svg_view
body::before {
  display: block;
  text-align: center;
  font-family: sans-serif;
  font-size: 1.5rem;
}
div {
  display: flex;
  gap: 20px;
}
svg {
  width: 260px;
}
rect {
  stroke: black;
  stroke-width: 3;
}
.start {
  fill: yellow;
}
.end {
  fill: red;
}
.container {
  fill: #dedede;
}
text {
  font: 40px monospace;
  fill: black;
}
line {
  stroke: black;
  stroke-width: 2;
  stroke-dasharray: 7;
}
.gradient {
  height: 1px;
  width: 1px;
  position: absolute;
  top: -100px;
}
```

```html hidden live-sample___initial live-sample___entry_exit live-sample___inset_percent live-sample___inset_length live-sample___inset_cover live-sample___inset_contain live-sample___cover_contain live-sample___exit_length_negative live-sample___entry_crossing live-sample___exit_crossing
<main>
  <article>
    <p>&nbsp;</p>
    <p>&nbsp;</p>
    <p>Scroll down ⇩</p>
    <p>&nbsp;</p>
    <p>&nbsp;</p>
    <section class="one animated_element">
      <div>
        <i>Animated Element</i>
        <span></span>
      </div>
    </section>
    <section class="double">
      <div>
        <i id="A" class="animated_element">A</i>
        <i id="B" class="animated_element">B</i>
      </div>
    </section>
    <p>&nbsp;</p>
    <p>&nbsp;</p>
    <p>&nbsp;</p>
    <p>&nbsp;</p>
    <p>Scroll up ⇧</p>
  </article>
</main>
```

```html hidden live-sample___initial live-sample___entry_exit live-sample___inset_percent live-sample___inset_length live-sample___inset_cover live-sample___inset_contain live-sample___cover_contain live-sample___entry_crossing live-sample___exit_crossing live-sample___exit_length_negative
<fieldset>
  <legend>Select the height of the animated element</legend>

  <label><input name="height" value="50" type="radio" checked /> 50px</label>
  <label><input name="height" value="250" type="radio" /> 250px</label>
  <label><input name="height" value="500" type="radio" /> 500px</label>
</fieldset>
<fieldset class="double">
  <legend>Select the animation range</legend>

  <label><input name="range" value="20" type="radio" checked />20% / 60%</label>
  <label><input name="range" value="0" type="radio" /> 0% / 100%</label>
</fieldset>
```

```css hidden live-sample___initial live-sample___entry_exit live-sample___inset_percent live-sample___inset_length live-sample___inset_cover live-sample___inset_contain live-sample___cover_contain live-sample___exit_length_negative live-sample___entry_crossing live-sample___exit_crossing
@layer {
  :root {
    --animElHeight: 50px;
    --animElHeightWord: "50px";
    --barColor: black;
    padding-top: 20px;
    --containerHeight: 250px;
  }
  body:has(input[value="250"]:checked) {
    --animElHeight: 250px;
    --animElHeightWord: "250px";
  }
  body:has(input[value="500"]:checked) {
    --animElHeight: 500px;
    --animElHeightWord: "500px";
  }
  main {
    padding: 20px 0 0 20px;
    margin-bottom: 2em;
  }
  article {
    outline: 3px dashed;
    width: 475px;
    margin: auto;
    overflow: scroll;
    position: relative;
    height: var(--containerHeight);
    box-sizing: content-box;
  }

  p {
    padding: 10px;
    margin: 10px;
  }

  section {
    --clr: yellow;
    --words: "Animation not started";
    position: relative;
    margin: 20px;
    text-align: center;
  }
  .one,
  .double i {
    animation: showAnim step-end 1 forwards;
    animation-timeline: view();
  }
  i,
  .animated_element {
    background-color: hsl(from var(--clr) h s calc(l * 1.4));
    display: block;
    height: var(--animElHeight);
    line-height: var(--animElHeight);
  }
  span {
    background-color: hsl(from var(--clr) h s 90%);
    border: 5px solid hsl(from var(--clr) h s 20%);
    min-width: 250px;
    height: 30px;
    line-height: 30px;
  }
  span,
  i {
    font-family: sans-serif;
    font-size: 1.5rem;
  }
  span::before {
    content: var(--words);
  }
  span {
    position: fixed;
    top: 10px;
    left: 10px;
    padding: 10px;
  }
  i::after {
    content: " ( " var(--animElHeightWord) " )";
  }
  label {
    padding-right: 2em;
  }
  legend {
    margin-top: 2em;
  }

  @keyframes showAnim {
    from {
      --clr: green;
      --words: "Currently animating";
    }
    to {
      --clr: red;
      --words: "Animation complete";
    }
  }
  body::before {
    display: block;
    text-align: center;
    font-family: sans-serif;
    font-size: 1.5rem;
  }

  @layer no-support {
    @supports not (animation-timeline: view()) {
      body::before {
        content: "Your browser doesn't support view progress scrolling.";
        background-color: wheat;
        display: block;
        text-align: center;
      }
    }
  }
}
```

```css hidden live-sample___initial live-sample___inset_percent live-sample___inset_length live-sample___inset_cover live-sample___inset_contain
.double {
  display: none;
}
```

```css hidden live-sample___cover_contain live-sample___exit_length_negative live-sample___entry_crossing live-sample___exit_crossing live-sample___entry_exit
.one {
  display: none;
}
.double div {
  display: flex;
  gap: 10px;
}
```

## See also

- {{cssxref("timeline-range-name")}} data type
- [Keyframe selectors](/en-US/docs/Web/CSS/Reference/Selectors/Keyframe_selectors)
- [Scroll-driven animation timelines](/en-US/docs/Web/CSS/Guides/Scroll-driven_animations/Timelines)
- [Scroll-driven animation](/en-US/docs/Web/CSS/Guides/Scroll-driven_animations) module
- [CSS animations](/en-US/docs/Web/CSS/Guides/Animations) module
- [Web Animations API](/en-US/docs/Web/API/Web_Animations_API)

`````

### Expected (prettier)

`````md
---
title: Understanding timeline insets
slug: Web/CSS/Guides/Scroll-driven_animations/Timeline_insets
page-type: guide
sidebar: cssref
---

By default, [view progress timelines](/en-US/docs/Web/CSS/Guides/Scroll-driven_animations/Timelines#view_progress_timelines) track elements across the entire [animation attachment range](/en-US/docs/Web/CSS/Guides/Scroll-driven_animations/Timeline_range_names#the_animation_attachment_range). The `0%` progress point is at the start of the range, while the `100%` progress point is at the end. The animation attachment range can be changed by setting a [timeline range name](/en-US/docs/Web/CSS/Guides/Scroll-driven_animations/Timeline_range_names), and the location of the `0%` and `100%` progress points along the range can be adjusted by setting length or percentage-based inset values.

This guide explains how to limit the animation timeline to a specific portion of the animation timeline range using length or percentage inset values.

## Animation timelines: a primer

[CSS animations](/en-US/docs/Web/CSS/Guides/Animations) are created by defining named {{cssxref("@keyframes")}} animations, which specify an animation's behavior, and then attaching the keyframe animation to an element using the animation's name.

The element's animation timeline, defined by the {{cssxref("animation-timeline")}} property, determines how and when the element progresses through those keyframes. By default, the timeline is time-based, using the document's default time-based {{domxref("DocumentTimeline")}}.

The [CSS scroll-driven animation](/en-US/docs/Web/CSS/Guides/Scroll-driven_animations) module defines scroll progress and view progress timelines, which are methods for animating property values along a scroll-based timeline rather than the default time-based document timeline. In this article, we'll only discuss view progress timelines, as [scroll progress timelines](/en-US/docs/Web/CSS/Guides/Scroll-driven_animations/Timelines#scroll_progress_timelines) are not relevant to timeline insets.

### View progress timelines

With [view progress timelines](/en-US/docs/Web/CSS/Guides/Scroll-driven_animations/Timelines#view_progress_timelines), the animation's timeline, or progress, is driven by element visibility instead of the passage of time, with keyframe progression tied to the subject element position and visibility within the scroll container. The animation advances and reverses as the element advances or reverses through the scrollport. The animation only occurs when at least part of the element is visible within its scrollport, pausing when scrolling pauses.

```css live-sample___svg_view
.animated_element {
  animation-name: nameOfAnimation;
  animation-timeline: view();
}
```

Setting an {{cssxref("animation-name")}} applies the animation to the selected element.

> [!NOTE]
> The `animation-timeline` property should always come after any `animation` shorthand declarations. While the shorthand property can't be used to set the `animation-timeline` property, it does reset the timeline to the default time-based document timeline.

> [!NOTE]
> In all the examples, the {{glossary("scroll container")}} is `250px` tall and we are using the default values for {{cssxref("animation-iteration-count")}} (`1`), {{cssxref("animation-delay")}} (`0s`), and {{cssxref("animation-direction")}} (`normal`). We set the {{cssxref("animation-timing-function")}} to `step-end` and the {{cssxref("animation-fill-mode")}} is to `forward` to make the it more apparent when the animation iteration has not yet started, when it is active, and when it is complete. See the [Using CSS animations guide](/en-US/docs/Web/CSS/Guides/Animations/Using) to learn more.

As you scroll up, the animation progresses. As you scroll down, the animation reverses.

{{EmbedLiveSample("initial", "100%", "400")}}

In this example, whenever any part of the subject element is visible in the scrollport, the animation occurs. By default, view progress animations begin just as the top edge of the subject element aligns with the bottom edge of the scroll container and end, reaching `100%` progress, when the end edge aligns with the start edge of the container, no matter the size of the subject element. By default, the animation is applied when any part of the subject is visible within the scrollport.

### Animation attachment ranges

In a [view progress timeline](/en-US/docs/Web/CSS/Guides/Scroll-driven_animations/Timelines#view_progress_timelines), when no animation range properties are defined, the `<timeline-range-name>` is `normal`, which defaults to `cover`. The animation is applied whenever any portion of the subject element is visible, meaning the default **animation attachment range** is the sum of the height of the scroll container and the height of the subject element, with that extra height being at the scroll end edge. In our example, as the scroll container is `250px` tall, and the subject is `50px`, `250px`, or `500px` tall, with the vertical animation attachment range being `300px`, `500px`, or `750px` respectively.

The `0%` progression occurs when the subject element's start edge intersects the scrollport at the end edge, reaching `100%` progress when the subject's end edge exits via the scrollport's start edge. These are the subject and scrollport's top and bottom edges when scrolling vertically, and the left and right or right and left edges when scrolling horizontally, depending on the writing mode.

The following diagram illustrates the position of the subject at the `0%` and `100%` progress points for the three subject sizes:

```html hidden live-sample___svg_view
<div>
  <svg viewBox="-1 -1 462 1252" xmlns="http://www.w3.org/2000/svg">
    <title>Default view progress timeline</title>
    <rect class="container" width="350" height="250" x="0" y="500" />
    <rect class="small end" width="100" height="50" x="10" y="450" />
    <rect class="medium end" width="100" height="250" x="125" y="250" />
    <rect class="large end" width="100" height="500" x="240" y="0" />
    <rect class="small start" width="100" height="50" x="10" y="750" />
    <rect class="medium start" width="100" height="250" x="125" y="750" />
    <rect class="large start" width="100" height="500" x="240" y="750" />
    <text y="520" x="360">100%</text>
    <line x1="0" x2="350" y1="500" y2="500" />
    <line x1="0" x2="350" y1="750" y2="750" />
    <text y="760" x="360">0%</text>
  </svg>
</div>
```

{{EmbedLiveSample("svg_view", "100%", "720")}}

The yellow subject elements represents the position of the element when the `from` keyframe is applied, which is the animation range's `0%` progress mark. The red represents the location of the animated element relative to the scrollport when the `to` keyframe is applied, which is the end of the animation, or the `100%` progress mark. The gray represents the scrollport.

By default, the element animates while it is "in view", but this default definition of "in view" may not fit your needs. Fortunately, we can control which edges define the edges of the animation attachment range and then offset the start and end of that range with the animation range properties.

### Animation range properties

The {{cssxref("animation-range")}} properties enable specifying a named timeline range, such as `contain` or `exit-crossing`, which changes the range used from the default `cover` range. You can also include a {{cssxref("length-percentage")}} value, which insets the attachment range from the start of the range. Percentages are relative to the named or default timeline range.

Named timeline ranges define the portions of a {{domxref("ViewTimeline")}} that define an animation's range, specifying the start and end of the animations's attachment range.

The `animation-range` property is a shorthand property, defining the {{cssxref("animation-range-start")}} and {{cssxref("animation-range-end")}} properties. The `animation-range-start` defines the position of the subject element when the animation starts. The `animation-range-end` defines the position of the subject element when the animation ends.

See the [timeline range names guide](/en-US/docs/Web/CSS/Guides/Scroll-driven_animations/Timeline_range_names) to learn about the different named timeline ranges. This guide focuses on how the {{cssxref("length-percentage")}} inset values work.

## Setting insets using lengths

The `animation-range-start` and `animation-range-end` properties each accept a named animation range, a {{cssxref("length-percentage")}} offset value, or both. Any length or percentage offset is measured from the _start_ of the animation attachment range.

When a {{cssxref("length")}} is set, the offset is fairly intuitive.
Here we use the `animation-range-start` and `animation-range-end` properties to inset the animation timeline. This defines a subsection of the element's full animation attachment range as the active interval, with the `<length>` values specifying distances from the start of the default `normal` animation attachment range.

```css live-sample___inset_length
.animated_element {
  animation-range-start: 1em;
  animation-range-end: 125px;
}
```

The start and end of the animation range are `1em` and `125px` from the start of the animation attachment range, respectively. Because the timeline range default is `normal`, which resolves to `cover`, the start of the animation attachment range is the block end edge of the container.

```css hidden live-sample___inset_length
:root {
  --start: 1em;
  --end: 125px;
}

article {
  background-image: linear-gradient(
    to top,
    transparent calc(var(--start) - 1px),
    #cccccc calc(var(--start) - 1px) calc(var(--start) + 1px),
    transparent calc(var(--start) + 1px) calc(var(--end) - 1px),
    #cccccc calc(var(--end) - 1px) calc(var(--end) + 1px),
    transparent calc(var(--end) + 1px)
  );
}
```

{{EmbedLiveSample("inset_length", "100%", "400")}}

We've added lines `1em` and `125px` from the block end edge of the scroll container. The animation starts when the block start edge of the subject element reaches the `1em` line and ends when it reaches the `125px` line.

In this case, as the animation attachment range resolves to `cover` for both the start and end inset values, the location of the insets is fairly straightforward.

### Effect of named ranges on length offsets

The offset distance is always from the start of the associated animation range. In this example, we set the `animation-range-start` to be `50px` from the start of the default `normal` range and set the `animation-range-end` to be `100px` from the start of the explicitly set `entry` range:

```css live-sample___different_length
.animated_element {
  animation-range-start: 50px;
  animation-range-end: entry 100px;
}
```

```html hidden live-sample___different_length live-sample___exit_length live-sample___exit_percent live-sample___center
<main>
  <article>
    <p>&nbsp;</p>
    <p>&nbsp;</p>
    <p>Scroll down ⇩</p>
    <p>&nbsp;</p>
    <p>&nbsp;</p>
    <section class="triple">
      <div>
        <i id="A" class="animated_element">50px</i>
        <i id="B" class="animated_element">250px</i>
        <i id="C" class="animated_element">500px</i>
      </div>
    </section>
    <p>&nbsp;</p>
    <p>&nbsp;</p>
    <p>&nbsp;</p>
    <p>&nbsp;</p>
    <p>Scroll up ⇧</p>
  </article>
</main>
```

{{EmbedLiveSample("different_length", "100%", "310")}}

As the start edge of both the `normal` and `entry` ranges is the container's end edge, the animation begins when the subject start edge is `50px` from the bottom of the scrollport and ends, reaching `100%` progress, when the subject start edge is `100px` from the bottom of the scrollport, irrespective of the subject size. While the size of the `entry` range is different for the three different subject sizes, in this case, the size of the underlying range didn't matter.

### Length offsets with varying ranges

The size of range matters when the range doesn't start at the element's end edge, as is the case with `exit` and `exit-crossing`, or if the offset is a percentage value. This fact, and the fact that you can mix and match animation range names, make view progress timeline offsets a bit more complicated to understand than non-offsetted [timeline range names](/en-US/docs/Web/CSS/Guides/Scroll-driven_animations/Timeline_range_names).

For example, when setting `exit` as the timeline range name, the subject size matters as it determines the location of the range's end edge.

```css live-sample___exit_length
.animated_element {
  animation-range-start: entry 60px;
  animation-range-end: exit 75px;
}
```

With both `entry` and `exit`, the range is the size of the subject, with the size clamped to the size of the scrollport. This means that the height of the `entry` and `exit` ranges are the height of the box in both the `50px` and `250px` examples, while in the `500px` example, the range is clamped to the height of the scrollport, which is `250px` tall.

{{EmbedLiveSample("exit_length", "100%", "310")}}

We added a few lines to facilitate the explanations that follow: the bottom blue line is `60px` from the end edge of the scrollport, and the top red line is `75px` from that same edge. These are where the animation range start and end respectively.

This example demonstrates several important features, which we'll explain in greater detail, including:

- Offsets are [measured from their respective named ranges](#measured_from_the_start_edge_of_the_range)
- Offsets can occur [beyond the edges of the scrollport](#beyond_the_scrollport_edges)
- [Ranges may be clamped](#effects_of_clamping) if the subject is larger than the scrollport

#### Measured from the start edge of the range

As the offset position is always relative to the start of the declaration's animation range, the start of the animation for all three elements occurs when the elements start edge crosses the point that is `60px` from the start of the `entry` range.

The `animation-range-end` value defines the position where the animation. The `exit 75px` value basically means "when `75px` of the subject has exited the start edge of the scrollport." This varies for each subject. For the `50px` subject, this only occurs `25px` after it has left the scrollport; when the element is not visible. The animation range end for both the `250px` and `500px` subjects occurs when their bottom end edge intersects the top blue line; 75 pixels from the end edge of the scrollport. Why are their end offsets the same? Because of [clamping](#effects_of_clamping)! The maximum size of the named animation range is clamped to the size of the scrollport. The `exit` range for both subjects is the same, so the range end offsets are the same.

#### Beyond the scrollport edges

For our `50px` tall subject, the `exit` range is 50px tall abutting the start edge of the scroll port. Setting `animation-range-end: exit 75px` for any element less than `75px` tall means the end of the range is outside the scrollport, as the point `75px` from the start of the `exit` range is past the start edge of the scrollport. In our example, the end of the animation range for the `50px` subject occurs when the subject's start edge is `75px` past the scrollport's start edge. The animation ends, reaching the `to` keyframe and the {{domxref("Element/animationend_event", "animationend")}} event, only when (and if) the element is scrolled `25px` out of view.

The animation ends even if the animation range end is outside of the scrollport, as long as there is room to scroll to that point. Had we set `animation-range-end: exit 250px`, the animation would have ended when the end edge of the medium and tall subjects exited the scrollport at the container's start edge.

With the end set to `exit 250px`, the small subject's animation might not end, as there might not be `450px` of content after the subject for the user to scroll to before the end point is reached.

#### Effects of clamping

With our `250px` tall container, when the subject is `250px` or `500px` tall, the `exit` range is the size of the container, with the start being the scroll container's end edge. With a `75px` offset, the end of the animation occurs when the end edge of the subject is `75px` from the end edge of the scroll container (denoted by the top red line).

As the offset position is always relative to the start of the named or default animation range, in our example, clamping impacts the large subject's `animation-range-end`. We set the end of the range to `exit 75px` which is `75px` from the start edge of the `exit` range. When the subject is the same size as the scrollport (our `250px` subject) or larger (our `500px` subject), the animation range end is `75px` from the end edge of the scrollport, which is `75px` from the start of the scrollport-clamped `exit` range.

```css hidden live-sample___exit_length
article {
  background-image: linear-gradient(
    to top,
    transparent 59.5px,
    blue 59.5px 60.5px,
    transparent 60.5px 74.5px,
    red 74.5px 75.5px,
    transparent 75.5px /* 174.5px,
    green 174.5px 154.5px,
    transparent 175.5px*/
  );
}
.animated_element {
  align-self: flex-end;
}
```

```css hidden live-sample___different_length live-sample___exit_length live-sample___exit_percent live-sample___center
@layer setup {
  #A {
    height: 50px;
  }
  #B {
    height: 250px;
  }
  #C {
    height: 500px;
  }
  div {
    display: flex;
    gap: 1em;
  }
  main {
    padding: 20px 0 0 20px;
    margin-bottom: 2em;
  }
  article {
    outline: 3px dashed;
    width: 475px;
    margin: auto;
    overflow: scroll;
    position: relative;
    height: 250px;
    box-sizing: content-box;
    background-image: linear-gradient(
      to top,
      transparent 49.5px,
      #666666 49.5px 50.5px,
      transparent 50.5px 99.5px,
      #666666 99.5px 100.5px,
      transparent 100.5px
    );
    background-origin: content-box;
  }

  p {
    padding: 10px;
    margin: 10px;
  }

  .animated_element {
    --clr: yellow;
    background-color: hsl(from var(--clr) h s calc(l * 1.4));
    display: block;
    animation: showAnim step-end 1 forwards;
    animation-timeline: view();
    flex: 1 0 auto;
  }

  i {
    font-family: sans-serif;
    font-size: 1.5rem;
  }

  @keyframes showAnim {
    from {
      --clr: green;
    }
    to {
      --clr: red;
    }
  }
  @layer no-support {
    @supports not (animation-timeline: view()) {
      body::before {
        content: "Your browser doesn't support view progress scrolling.";
        background-color: wheat;
        display: block;
        text-align: center;
      }
    }
  }
}
```

### Negative lengths

Up to this point, all the offsets have been greater than zero. It's important to note that negative lengths are valid. A negative offset on the `animation-range-start` makes the range longer while a negative offset on the `animation-range-end` makes the range shorter.

Let's compare the negative insets compared to the `0` values:

```css live-sample___exit_length_negative
#A {
  animation-range-start: contain -25px;
  animation-range-end: exit -25px;
}
#B {
  animation-range-start: contain 0;
  animation-range-end: exit 0;
}
```

{{EmbedLiveSample("exit_length_negative", "100%", "380")}}

The first animation range is offset by `25px` toward the container end edge.

```css hidden live-sample___exit_length_negative
fieldset.double {
  display: none;
}
#A::after {
  content: " (-25px)";
}
#B::after {
  content: " (0)";
}
```

## Setting insets using percentages

Like length values, percentage values define offsets from the _start_ of the animation attachment range. The percentage offsets are relative to the timeline range dimension, not relative to the scrollport. For this reason, percentage values are not as intuitive as length values for most people (realizing length values weren't that intuitive either).

Here, we use `animation-range-start` and `animation-range-end` to inset the animation timeline. While we are using the same properties, we set `<percentage>` values instead of `<length>` values:

```css live-sample___inset_percent
.animated_element {
  animation-range-start: 20%;
  animation-range-end: 60%;
}
```

```css hidden live-sample___inset_percent live-sample___inset_cover
i {
  background-image: linear-gradient(
    to bottom,
    transparent calc(20% - 1px),
    #33333333 calc(20% - 1px) calc(20% + 1px),
    transparent calc(20% + 1px) calc(60% - 1px),
    #33333333 calc(60% - 1px) calc(60% + 1px),
    transparent calc(60% + 1px)
  );
}
article {
  --total: calc(var(--animElHeight) + 250px);
  background-image:
    linear-gradient(
      to top,
      transparent 0 calc(var(--total) * 0.2 - 1px),
      green calc(var(--total) * 0.2 - 1px) calc((var(--total) * 0.2) + 1px),
      transparent calc(var(--total) * 0.2 + 1px)
    ),
    linear-gradient(
      to top,
      transparent 0 calc(var(--total) * 0.6 - 1px),
      red calc(var(--total) * 0.6 - 1px) calc((var(--total) * 0.6) + 1px),
      transparent calc(var(--total) * 0.6 + 1px)
    ),
    linear-gradient(
      to top,
      transparent 0 calc(var(--containerHeight) * 0.2 - 0.5px),
      #33333333 calc(var(--containerHeight) * 0.2 - 0.5px)
        calc(var(--containerHeight) * 0.2 + 0.5px),
      transparent calc(var(--containerHeight) * 0.2 + 0.5px)
        calc(var(--containerHeight) * 0.6 - 0.5px),
      #33333333 calc(var(--containerHeight) * 0.6 - 0.5px)
        calc(var(--containerHeight) * 0.6 + 0.5px),
      transparent 0 calc(var(--containerHeight) * 0.6 + 0.5px)
    );
  background-attachment: local, local, fixed;
}
```

This defines the active interval to begin `20%` into the default attachment range and end `60%` through that same range. The default `normal` animation attachment range, which behaves as [`cover`](/en-US/docs/Web/CSS/Guides/Scroll-driven_animations/Timeline_range_names#cover), is the height of the scroll container plus the height of the subject element, meaning the range will differ based on which radio button is selected.

{{EmbedLiveSample("inset_percent", "100%", "400")}}

For illustrative purposes, there are two dark lines crossing the container at the `20%` and `60%` points of the full animation range. The animation starts when the block-start edge reaches the `20%` point, which is the bottom green line. The animation ends when the start block edge is 60% of the way through the normal range, which is the top red line.

Only when the element is `50px` tall is the top of the subject still in the scrollport when the end of the animation is reached; there are no top red lines when `250px` or `500px` are selected, as the end of the animation range is outside of the scrollport.

Based on the height of our subjects, the `20%` mark is either `60px`, `100px`, or `150px` from the end edge of the scrollport (marked by the green line, which is always in the scrollport), and the `60%` mark is `180px`, `300px`, or `450px` from the same point (marked with a red line, but only visible for the `50px` subject).

For illustrative purposes, there are two light gray lines crossing the container `20%` and `60%` of the way through the scrollport, which are `50px` and `150px` from the bottom of the scrollport, respectively. As the `animation-range-*` percentages are relative to the timeline range, not the scrollport, these lines only show how the percentages **don't** align. We've also included two horizontal light gray lines going across each subject at their own `20%` and `60%` marks. These lines align with the scrollport's light gray lines when each subjects animation starts and ends.

The following image demonstrates where the subject elements are located when the animation starts (the `0%` keyframe) and ends (the `100% keyframe`).
This image includes the insets from the animation timeline in the previous demonstration and the timeline without insets for comparison.

```html hidden live-sample___svg_insets2
<div>
  <svg viewBox="-1 -1 482 1252" xmlns="http://www.w3.org/2000/svg">
    <title>Default view progress timeline with insets</title>
    <rect class="container" width="350" height="250" x="0" y="500" />
    <rect class="small end" width="100" height="50" x="10" y="571" />
    <rect class="medium end" width="100" height="250" x="120" y="450" />
    <rect class="large end" width="100" height="500" x="230" y="300" />
    <rect class="small start" width="100" height="50" x="10" y="689" />
    <rect class="medium start" width="100" height="250" x="120" y="649" />
    <rect class="large start" width="100" height="500" x="230" y="600" />
    <rect width="96" height="48" x="122" y="602" fill="url(#g)" />
    <rect width="96" height="198" x="232" y="527" fill="url(#g)" />
    <text y="610" x="385">60%</text>
    <line x1="0" x2="385" y1="600" y2="600" />
    <line x1="0" x2="385" y1="700" y2="700" />
    <text y="710" x="385">20%</text>
  </svg>
  <svg viewBox="-1 -1 482 1252" xmlns="http://www.w3.org/2000/svg">
    <title>Default view progress timeline</title>
    <rect class="container" width="350" height="250" x="0" y="500" />
    <rect class="small end" width="100" height="50" x="10" y="450" />
    <rect class="medium end" width="100" height="250" x="125" y="250" />
    <rect class="large end" width="100" height="500" x="240" y="0" />
    <rect class="small start" width="100" height="50" x="10" y="750" />
    <rect class="medium start" width="100" height="250" x="125" y="750" />
    <rect class="large start" width="100" height="500" x="240" y="750" />
    <text y="520" x="385">100%</text>
    <line x1="0" x2="385" y1="500" y2="500" />
    <line x1="0" x2="385" y1="750" y2="750" />
    <text y="760" x="390">0%</text>
  </svg>
</div>
```

{{EmbedLiveSample("svg_insets2", "100%", "710")}}

As before, the yellow represents the position of the element when the `from` keyframe is applied, the red represents the location when the `to` keyframe is applied, and the gray represents the scrollport. The striped areas are where the red and yellow element representations overlap. For illustrative purposes, we've added dashed black horizontal lines `20%` and `60%` way through the scrollport, starting from the bottom.

The animation only begins when the element reaches the `20%` mark along the animation attachment range. This point is `60px`, `100px`, or `150px` from the bottom edge of the scroll port, depending on the size of the element. The location of the subject element at this point, representing the position of the element when the `from` or `0%` keyframe is applied, is shown in yellow.

The red represents the location of the animated element relative to the scrollport when the `to` or `100%` keyframe is applied, which is the end of the animation. This point is either `180px`, `300px`, or `450px` from the bottom edge of the scrollport, depending on the subject size. The animation occurs when the element is between the `to` and the `from` positions.

You may have noticed something interesting about the dashed horizontal lines: when the animation starts, the line that is `20%` from the end edge of the viewport is `20%` from the _top_ of the subject element and the line that is `60%` from the end edge of the viewport is `60%` from the _top_ of the subject element when the animation ends. This is what was illustrated by the very light gray lines in the live demo for this example.

### Subject size matters

As we saw when we [set insets with lengths](#setting_insets_using_lengths), the size of the subject can make a difference. When setting animation ranges, percentage values are relative to the size of animation attachment range, not the scrollport. For most named ranges, the size of the attachment range depends partially on the subject size. As percentages are based on the size of the range, the named range impacts the resolved size of the insets. Depending on the name, the start position may also change, impacting the location of the range and therefore the location of progress points.

In this example, we define an active range that is `40%` of the size of the subject:

```css live-sample___exit_percent
.animated_element {
  animation-range-start: exit-crossing -20%;
  animation-range-end: exit-crossing 20%;
}
```

```css hidden live-sample___exit_percent
article {
  background-image: none;
}
body .animated_element {
  align-self: start;
}
```

{{EmbedLiveSample("exit_percent", "100%", "400")}}

The animation lasts `40%` of the animation-attachment range. As you scroll, note how the larger the subject, the longer the range. With exit-crossing, the animation range is not cropped; it is the size of the subject even if the subject is larger than the viewport, with the range abutting the start edge of the scrollport, and extending off the end edge if the subject is larger than the scrollport.

With the `-20%` and `20%` insets, the `50px` subject's will animation over `20px`: the animation starts when the subject's end is `-10px` from range start, or `60px` from exiting the screen, and ends when the subject's end is `40px` from exiting the screen. The middle subject will animate over `100px`: the animation starts when the subject end is `-50px` from range start, which is `50px` off of the scrollport's end edge, and ends when the subject's end is `50px` into the scrollport. The large subject animates over `200px`, starting when the bottom is `600px` from the container's start edge, with only `150px` in view, and ends when the bottom is 400px from that start edge, when `100px` have scrolled off the start edge.

### Percentages equal to the scrollport

When it comes to offsetting with percentages, the least complicated named timeline range is `contain`. With `contain`, the animation range is the size of the scrollport, meaning the start and end percentages are relative to the scrollport. For this reason, when using offsets, you may want to use `contain` instead of letting the range default and resolve to `cover`.

The `contain` range fully _contains_ the animation within the scrollport. It represents the range during which the principal box is either fully contained by, or fully covers, its view progress visibility range within the scrollport. With `contain`, if the subject is the same size or smaller than the scrollport, it can be fully visible. If the element is the same size as the container, however, the animation occurs over `0px`. This means that it runs, but it is not visible to the user.

In other words, without needing to know the size of the container or the subjects, we are able to limit our animation to the middle of scrollport, though the animation will happen over `0px` if the subject is the same size as the scrollport.

```css live-sample___center
.animated_element {
  animation-range-start: contain 25%;
  animation-range-end: contain 75%;
}
```

```css hidden live-sample___center
article {
  background-image: linear-gradient(
    transparent 25%,
    #ededed 25% 75%,
    transparent 75%
  );
}
body .animated_element {
  align-self: center;
}

.animated_element {
  background-image:
    linear-gradient(black, black), linear-gradient(black, black);
  background-size: 1px 1px;
  background-position:
    center 25%,
    center 75%;
  background-repeat: repeat-x;
```

{{EmbedLiveSample("center", "100%", "310")}}

The horizontal lines denote the middle half of the scrollport and the middle half of each subject.

```html hidden live-sample___svg_contain live-sample___svg_insets2 live-sample___svg_view
<svg class="gradient">
  <title>Striped repeating gradient</title>
  <defs>
    <linearGradient
      id="g"
      x1="0"
      y1="0"
      x2="20"
      y2="20"
      spreadMethod="repeat"
      gradientUnits="userSpaceOnUse">
      <stop offset="50%" stop-color="red" />
      <stop offset="50%" stop-color="yellow" />
    </linearGradient>
  </defs>
</svg>
```

```css hidden live-sample___svg_contain live-sample___svg_insets2 live-sample___svg_view
body::before {
  display: block;
  text-align: center;
  font-family: sans-serif;
  font-size: 1.5rem;
}
div {
  display: flex;
  gap: 20px;
}
svg {
  width: 260px;
}
rect {
  stroke: black;
  stroke-width: 3;
}
.start {
  fill: yellow;
}
.end {
  fill: red;
}
.container {
  fill: #dedede;
}
text {
  font: 40px monospace;
  fill: black;
}
line {
  stroke: black;
  stroke-width: 2;
  stroke-dasharray: 7;
}
.gradient {
  height: 1px;
  width: 1px;
  position: absolute;
  top: -100px;
}
```

```html hidden live-sample___initial live-sample___entry_exit live-sample___inset_percent live-sample___inset_length live-sample___inset_cover live-sample___inset_contain live-sample___cover_contain live-sample___exit_length_negative live-sample___entry_crossing live-sample___exit_crossing
<main>
  <article>
    <p>&nbsp;</p>
    <p>&nbsp;</p>
    <p>Scroll down ⇩</p>
    <p>&nbsp;</p>
    <p>&nbsp;</p>
    <section class="one animated_element">
      <div>
        <i>Animated Element</i>
        <span></span>
      </div>
    </section>
    <section class="double">
      <div>
        <i id="A" class="animated_element">A</i>
        <i id="B" class="animated_element">B</i>
      </div>
    </section>
    <p>&nbsp;</p>
    <p>&nbsp;</p>
    <p>&nbsp;</p>
    <p>&nbsp;</p>
    <p>Scroll up ⇧</p>
  </article>
</main>
```

```html hidden live-sample___initial live-sample___entry_exit live-sample___inset_percent live-sample___inset_length live-sample___inset_cover live-sample___inset_contain live-sample___cover_contain live-sample___entry_crossing live-sample___exit_crossing live-sample___exit_length_negative
<fieldset>
  <legend>Select the height of the animated element</legend>

  <label><input name="height" value="50" type="radio" checked /> 50px</label>
  <label><input name="height" value="250" type="radio" /> 250px</label>
  <label><input name="height" value="500" type="radio" /> 500px</label>
</fieldset>
<fieldset class="double">
  <legend>Select the animation range</legend>

  <label><input name="range" value="20" type="radio" checked />20% / 60%</label>
  <label><input name="range" value="0" type="radio" /> 0% / 100%</label>
</fieldset>
```

```css hidden live-sample___initial live-sample___entry_exit live-sample___inset_percent live-sample___inset_length live-sample___inset_cover live-sample___inset_contain live-sample___cover_contain live-sample___exit_length_negative live-sample___entry_crossing live-sample___exit_crossing
@layer {
  :root {
    --animElHeight: 50px;
    --animElHeightWord: "50px";
    --barColor: black;
    padding-top: 20px;
    --containerHeight: 250px;
  }
  body:has(input[value="250"]:checked) {
    --animElHeight: 250px;
    --animElHeightWord: "250px";
  }
  body:has(input[value="500"]:checked) {
    --animElHeight: 500px;
    --animElHeightWord: "500px";
  }
  main {
    padding: 20px 0 0 20px;
    margin-bottom: 2em;
  }
  article {
    outline: 3px dashed;
    width: 475px;
    margin: auto;
    overflow: scroll;
    position: relative;
    height: var(--containerHeight);
    box-sizing: content-box;
  }

  p {
    padding: 10px;
    margin: 10px;
  }

  section {
    --clr: yellow;
    --words: "Animation not started";
    position: relative;
    margin: 20px;
    text-align: center;
  }
  .one,
  .double i {
    animation: showAnim step-end 1 forwards;
    animation-timeline: view();
  }
  i,
  .animated_element {
    background-color: hsl(from var(--clr) h s calc(l * 1.4));
    display: block;
    height: var(--animElHeight);
    line-height: var(--animElHeight);
  }
  span {
    background-color: hsl(from var(--clr) h s 90%);
    border: 5px solid hsl(from var(--clr) h s 20%);
    min-width: 250px;
    height: 30px;
    line-height: 30px;
  }
  span,
  i {
    font-family: sans-serif;
    font-size: 1.5rem;
  }
  span::before {
    content: var(--words);
  }
  span {
    position: fixed;
    top: 10px;
    left: 10px;
    padding: 10px;
  }
  i::after {
    content: " ( " var(--animElHeightWord) " )";
  }
  label {
    padding-right: 2em;
  }
  legend {
    margin-top: 2em;
  }

  @keyframes showAnim {
    from {
      --clr: green;
      --words: "Currently animating";
    }
    to {
      --clr: red;
      --words: "Animation complete";
    }
  }
  body::before {
    display: block;
    text-align: center;
    font-family: sans-serif;
    font-size: 1.5rem;
  }

  @layer no-support {
    @supports not (animation-timeline: view()) {
      body::before {
        content: "Your browser doesn't support view progress scrolling.";
        background-color: wheat;
        display: block;
        text-align: center;
      }
    }
  }
}
```

```css hidden live-sample___initial live-sample___inset_percent live-sample___inset_length live-sample___inset_cover live-sample___inset_contain
.double {
  display: none;
}
```

```css hidden live-sample___cover_contain live-sample___exit_length_negative live-sample___entry_crossing live-sample___exit_crossing live-sample___entry_exit
.one {
  display: none;
}
.double div {
  display: flex;
  gap: 10px;
}
```

## See also

- {{cssxref("timeline-range-name")}} data type
- [Keyframe selectors](/en-US/docs/Web/CSS/Reference/Selectors/Keyframe_selectors)
- [Scroll-driven animation timelines](/en-US/docs/Web/CSS/Guides/Scroll-driven_animations/Timelines)
- [Scroll-driven animation](/en-US/docs/Web/CSS/Guides/Scroll-driven_animations) module
- [CSS animations](/en-US/docs/Web/CSS/Guides/Animations) module
- [Web Animations API](/en-US/docs/Web/API/Web_Animations_API)

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
@@ -216,9 +216,10 @@
     transparent 59.5px,
     blue 59.5px 60.5px,
     transparent 60.5px 74.5px,
     red 74.5px 75.5px,
-    transparent 75.5px /* 174.5px,
+    transparent 75.5px
+      /* 174.5px,
     green 174.5px 154.5px,
     transparent 175.5px*/
   );
 }

`````

### Actual (oxfmt)

`````md
---
title: Understanding timeline insets
slug: Web/CSS/Guides/Scroll-driven_animations/Timeline_insets
page-type: guide
sidebar: cssref
---

By default, [view progress timelines](/en-US/docs/Web/CSS/Guides/Scroll-driven_animations/Timelines#view_progress_timelines) track elements across the entire [animation attachment range](/en-US/docs/Web/CSS/Guides/Scroll-driven_animations/Timeline_range_names#the_animation_attachment_range). The `0%` progress point is at the start of the range, while the `100%` progress point is at the end. The animation attachment range can be changed by setting a [timeline range name](/en-US/docs/Web/CSS/Guides/Scroll-driven_animations/Timeline_range_names), and the location of the `0%` and `100%` progress points along the range can be adjusted by setting length or percentage-based inset values.

This guide explains how to limit the animation timeline to a specific portion of the animation timeline range using length or percentage inset values.

## Animation timelines: a primer

[CSS animations](/en-US/docs/Web/CSS/Guides/Animations) are created by defining named {{cssxref("@keyframes")}} animations, which specify an animation's behavior, and then attaching the keyframe animation to an element using the animation's name.

The element's animation timeline, defined by the {{cssxref("animation-timeline")}} property, determines how and when the element progresses through those keyframes. By default, the timeline is time-based, using the document's default time-based {{domxref("DocumentTimeline")}}.

The [CSS scroll-driven animation](/en-US/docs/Web/CSS/Guides/Scroll-driven_animations) module defines scroll progress and view progress timelines, which are methods for animating property values along a scroll-based timeline rather than the default time-based document timeline. In this article, we'll only discuss view progress timelines, as [scroll progress timelines](/en-US/docs/Web/CSS/Guides/Scroll-driven_animations/Timelines#scroll_progress_timelines) are not relevant to timeline insets.

### View progress timelines

With [view progress timelines](/en-US/docs/Web/CSS/Guides/Scroll-driven_animations/Timelines#view_progress_timelines), the animation's timeline, or progress, is driven by element visibility instead of the passage of time, with keyframe progression tied to the subject element position and visibility within the scroll container. The animation advances and reverses as the element advances or reverses through the scrollport. The animation only occurs when at least part of the element is visible within its scrollport, pausing when scrolling pauses.

```css live-sample___svg_view
.animated_element {
  animation-name: nameOfAnimation;
  animation-timeline: view();
}
```

Setting an {{cssxref("animation-name")}} applies the animation to the selected element.

> [!NOTE]
> The `animation-timeline` property should always come after any `animation` shorthand declarations. While the shorthand property can't be used to set the `animation-timeline` property, it does reset the timeline to the default time-based document timeline.

> [!NOTE]
> In all the examples, the {{glossary("scroll container")}} is `250px` tall and we are using the default values for {{cssxref("animation-iteration-count")}} (`1`), {{cssxref("animation-delay")}} (`0s`), and {{cssxref("animation-direction")}} (`normal`). We set the {{cssxref("animation-timing-function")}} to `step-end` and the {{cssxref("animation-fill-mode")}} is to `forward` to make the it more apparent when the animation iteration has not yet started, when it is active, and when it is complete. See the [Using CSS animations guide](/en-US/docs/Web/CSS/Guides/Animations/Using) to learn more.

As you scroll up, the animation progresses. As you scroll down, the animation reverses.

{{EmbedLiveSample("initial", "100%", "400")}}

In this example, whenever any part of the subject element is visible in the scrollport, the animation occurs. By default, view progress animations begin just as the top edge of the subject element aligns with the bottom edge of the scroll container and end, reaching `100%` progress, when the end edge aligns with the start edge of the container, no matter the size of the subject element. By default, the animation is applied when any part of the subject is visible within the scrollport.

### Animation attachment ranges

In a [view progress timeline](/en-US/docs/Web/CSS/Guides/Scroll-driven_animations/Timelines#view_progress_timelines), when no animation range properties are defined, the `<timeline-range-name>` is `normal`, which defaults to `cover`. The animation is applied whenever any portion of the subject element is visible, meaning the default **animation attachment range** is the sum of the height of the scroll container and the height of the subject element, with that extra height being at the scroll end edge. In our example, as the scroll container is `250px` tall, and the subject is `50px`, `250px`, or `500px` tall, with the vertical animation attachment range being `300px`, `500px`, or `750px` respectively.

The `0%` progression occurs when the subject element's start edge intersects the scrollport at the end edge, reaching `100%` progress when the subject's end edge exits via the scrollport's start edge. These are the subject and scrollport's top and bottom edges when scrolling vertically, and the left and right or right and left edges when scrolling horizontally, depending on the writing mode.

The following diagram illustrates the position of the subject at the `0%` and `100%` progress points for the three subject sizes:

```html hidden live-sample___svg_view
<div>
  <svg viewBox="-1 -1 462 1252" xmlns="http://www.w3.org/2000/svg">
    <title>Default view progress timeline</title>
    <rect class="container" width="350" height="250" x="0" y="500" />
    <rect class="small end" width="100" height="50" x="10" y="450" />
    <rect class="medium end" width="100" height="250" x="125" y="250" />
    <rect class="large end" width="100" height="500" x="240" y="0" />
    <rect class="small start" width="100" height="50" x="10" y="750" />
    <rect class="medium start" width="100" height="250" x="125" y="750" />
    <rect class="large start" width="100" height="500" x="240" y="750" />
    <text y="520" x="360">100%</text>
    <line x1="0" x2="350" y1="500" y2="500" />
    <line x1="0" x2="350" y1="750" y2="750" />
    <text y="760" x="360">0%</text>
  </svg>
</div>
```

{{EmbedLiveSample("svg_view", "100%", "720")}}

The yellow subject elements represents the position of the element when the `from` keyframe is applied, which is the animation range's `0%` progress mark. The red represents the location of the animated element relative to the scrollport when the `to` keyframe is applied, which is the end of the animation, or the `100%` progress mark. The gray represents the scrollport.

By default, the element animates while it is "in view", but this default definition of "in view" may not fit your needs. Fortunately, we can control which edges define the edges of the animation attachment range and then offset the start and end of that range with the animation range properties.

### Animation range properties

The {{cssxref("animation-range")}} properties enable specifying a named timeline range, such as `contain` or `exit-crossing`, which changes the range used from the default `cover` range. You can also include a {{cssxref("length-percentage")}} value, which insets the attachment range from the start of the range. Percentages are relative to the named or default timeline range.

Named timeline ranges define the portions of a {{domxref("ViewTimeline")}} that define an animation's range, specifying the start and end of the animations's attachment range.

The `animation-range` property is a shorthand property, defining the {{cssxref("animation-range-start")}} and {{cssxref("animation-range-end")}} properties. The `animation-range-start` defines the position of the subject element when the animation starts. The `animation-range-end` defines the position of the subject element when the animation ends.

See the [timeline range names guide](/en-US/docs/Web/CSS/Guides/Scroll-driven_animations/Timeline_range_names) to learn about the different named timeline ranges. This guide focuses on how the {{cssxref("length-percentage")}} inset values work.

## Setting insets using lengths

The `animation-range-start` and `animation-range-end` properties each accept a named animation range, a {{cssxref("length-percentage")}} offset value, or both. Any length or percentage offset is measured from the _start_ of the animation attachment range.

When a {{cssxref("length")}} is set, the offset is fairly intuitive.
Here we use the `animation-range-start` and `animation-range-end` properties to inset the animation timeline. This defines a subsection of the element's full animation attachment range as the active interval, with the `<length>` values specifying distances from the start of the default `normal` animation attachment range.

```css live-sample___inset_length
.animated_element {
  animation-range-start: 1em;
  animation-range-end: 125px;
}
```

The start and end of the animation range are `1em` and `125px` from the start of the animation attachment range, respectively. Because the timeline range default is `normal`, which resolves to `cover`, the start of the animation attachment range is the block end edge of the container.

```css hidden live-sample___inset_length
:root {
  --start: 1em;
  --end: 125px;
}

article {
  background-image: linear-gradient(
    to top,
    transparent calc(var(--start) - 1px),
    #cccccc calc(var(--start) - 1px) calc(var(--start) + 1px),
    transparent calc(var(--start) + 1px) calc(var(--end) - 1px),
    #cccccc calc(var(--end) - 1px) calc(var(--end) + 1px),
    transparent calc(var(--end) + 1px)
  );
}
```

{{EmbedLiveSample("inset_length", "100%", "400")}}

We've added lines `1em` and `125px` from the block end edge of the scroll container. The animation starts when the block start edge of the subject element reaches the `1em` line and ends when it reaches the `125px` line.

In this case, as the animation attachment range resolves to `cover` for both the start and end inset values, the location of the insets is fairly straightforward.

### Effect of named ranges on length offsets

The offset distance is always from the start of the associated animation range. In this example, we set the `animation-range-start` to be `50px` from the start of the default `normal` range and set the `animation-range-end` to be `100px` from the start of the explicitly set `entry` range:

```css live-sample___different_length
.animated_element {
  animation-range-start: 50px;
  animation-range-end: entry 100px;
}
```

```html hidden live-sample___different_length live-sample___exit_length live-sample___exit_percent live-sample___center
<main>
  <article>
    <p>&nbsp;</p>
    <p>&nbsp;</p>
    <p>Scroll down ⇩</p>
    <p>&nbsp;</p>
    <p>&nbsp;</p>
    <section class="triple">
      <div>
        <i id="A" class="animated_element">50px</i>
        <i id="B" class="animated_element">250px</i>
        <i id="C" class="animated_element">500px</i>
      </div>
    </section>
    <p>&nbsp;</p>
    <p>&nbsp;</p>
    <p>&nbsp;</p>
    <p>&nbsp;</p>
    <p>Scroll up ⇧</p>
  </article>
</main>
```

{{EmbedLiveSample("different_length", "100%", "310")}}

As the start edge of both the `normal` and `entry` ranges is the container's end edge, the animation begins when the subject start edge is `50px` from the bottom of the scrollport and ends, reaching `100%` progress, when the subject start edge is `100px` from the bottom of the scrollport, irrespective of the subject size. While the size of the `entry` range is different for the three different subject sizes, in this case, the size of the underlying range didn't matter.

### Length offsets with varying ranges

The size of range matters when the range doesn't start at the element's end edge, as is the case with `exit` and `exit-crossing`, or if the offset is a percentage value. This fact, and the fact that you can mix and match animation range names, make view progress timeline offsets a bit more complicated to understand than non-offsetted [timeline range names](/en-US/docs/Web/CSS/Guides/Scroll-driven_animations/Timeline_range_names).

For example, when setting `exit` as the timeline range name, the subject size matters as it determines the location of the range's end edge.

```css live-sample___exit_length
.animated_element {
  animation-range-start: entry 60px;
  animation-range-end: exit 75px;
}
```

With both `entry` and `exit`, the range is the size of the subject, with the size clamped to the size of the scrollport. This means that the height of the `entry` and `exit` ranges are the height of the box in both the `50px` and `250px` examples, while in the `500px` example, the range is clamped to the height of the scrollport, which is `250px` tall.

{{EmbedLiveSample("exit_length", "100%", "310")}}

We added a few lines to facilitate the explanations that follow: the bottom blue line is `60px` from the end edge of the scrollport, and the top red line is `75px` from that same edge. These are where the animation range start and end respectively.

This example demonstrates several important features, which we'll explain in greater detail, including:

- Offsets are [measured from their respective named ranges](#measured_from_the_start_edge_of_the_range)
- Offsets can occur [beyond the edges of the scrollport](#beyond_the_scrollport_edges)
- [Ranges may be clamped](#effects_of_clamping) if the subject is larger than the scrollport

#### Measured from the start edge of the range

As the offset position is always relative to the start of the declaration's animation range, the start of the animation for all three elements occurs when the elements start edge crosses the point that is `60px` from the start of the `entry` range.

The `animation-range-end` value defines the position where the animation. The `exit 75px` value basically means "when `75px` of the subject has exited the start edge of the scrollport." This varies for each subject. For the `50px` subject, this only occurs `25px` after it has left the scrollport; when the element is not visible. The animation range end for both the `250px` and `500px` subjects occurs when their bottom end edge intersects the top blue line; 75 pixels from the end edge of the scrollport. Why are their end offsets the same? Because of [clamping](#effects_of_clamping)! The maximum size of the named animation range is clamped to the size of the scrollport. The `exit` range for both subjects is the same, so the range end offsets are the same.

#### Beyond the scrollport edges

For our `50px` tall subject, the `exit` range is 50px tall abutting the start edge of the scroll port. Setting `animation-range-end: exit 75px` for any element less than `75px` tall means the end of the range is outside the scrollport, as the point `75px` from the start of the `exit` range is past the start edge of the scrollport. In our example, the end of the animation range for the `50px` subject occurs when the subject's start edge is `75px` past the scrollport's start edge. The animation ends, reaching the `to` keyframe and the {{domxref("Element/animationend_event", "animationend")}} event, only when (and if) the element is scrolled `25px` out of view.

The animation ends even if the animation range end is outside of the scrollport, as long as there is room to scroll to that point. Had we set `animation-range-end: exit 250px`, the animation would have ended when the end edge of the medium and tall subjects exited the scrollport at the container's start edge.

With the end set to `exit 250px`, the small subject's animation might not end, as there might not be `450px` of content after the subject for the user to scroll to before the end point is reached.

#### Effects of clamping

With our `250px` tall container, when the subject is `250px` or `500px` tall, the `exit` range is the size of the container, with the start being the scroll container's end edge. With a `75px` offset, the end of the animation occurs when the end edge of the subject is `75px` from the end edge of the scroll container (denoted by the top red line).

As the offset position is always relative to the start of the named or default animation range, in our example, clamping impacts the large subject's `animation-range-end`. We set the end of the range to `exit 75px` which is `75px` from the start edge of the `exit` range. When the subject is the same size as the scrollport (our `250px` subject) or larger (our `500px` subject), the animation range end is `75px` from the end edge of the scrollport, which is `75px` from the start of the scrollport-clamped `exit` range.

```css hidden live-sample___exit_length
article {
  background-image: linear-gradient(
    to top,
    transparent 59.5px,
    blue 59.5px 60.5px,
    transparent 60.5px 74.5px,
    red 74.5px 75.5px,
    transparent 75.5px
      /* 174.5px,
    green 174.5px 154.5px,
    transparent 175.5px*/
  );
}
.animated_element {
  align-self: flex-end;
}
```

```css hidden live-sample___different_length live-sample___exit_length live-sample___exit_percent live-sample___center
@layer setup {
  #A {
    height: 50px;
  }
  #B {
    height: 250px;
  }
  #C {
    height: 500px;
  }
  div {
    display: flex;
    gap: 1em;
  }
  main {
    padding: 20px 0 0 20px;
    margin-bottom: 2em;
  }
  article {
    outline: 3px dashed;
    width: 475px;
    margin: auto;
    overflow: scroll;
    position: relative;
    height: 250px;
    box-sizing: content-box;
    background-image: linear-gradient(
      to top,
      transparent 49.5px,
      #666666 49.5px 50.5px,
      transparent 50.5px 99.5px,
      #666666 99.5px 100.5px,
      transparent 100.5px
    );
    background-origin: content-box;
  }

  p {
    padding: 10px;
    margin: 10px;
  }

  .animated_element {
    --clr: yellow;
    background-color: hsl(from var(--clr) h s calc(l * 1.4));
    display: block;
    animation: showAnim step-end 1 forwards;
    animation-timeline: view();
    flex: 1 0 auto;
  }

  i {
    font-family: sans-serif;
    font-size: 1.5rem;
  }

  @keyframes showAnim {
    from {
      --clr: green;
    }
    to {
      --clr: red;
    }
  }
  @layer no-support {
    @supports not (animation-timeline: view()) {
      body::before {
        content: "Your browser doesn't support view progress scrolling.";
        background-color: wheat;
        display: block;
        text-align: center;
      }
    }
  }
}
```

### Negative lengths

Up to this point, all the offsets have been greater than zero. It's important to note that negative lengths are valid. A negative offset on the `animation-range-start` makes the range longer while a negative offset on the `animation-range-end` makes the range shorter.

Let's compare the negative insets compared to the `0` values:

```css live-sample___exit_length_negative
#A {
  animation-range-start: contain -25px;
  animation-range-end: exit -25px;
}
#B {
  animation-range-start: contain 0;
  animation-range-end: exit 0;
}
```

{{EmbedLiveSample("exit_length_negative", "100%", "380")}}

The first animation range is offset by `25px` toward the container end edge.

```css hidden live-sample___exit_length_negative
fieldset.double {
  display: none;
}
#A::after {
  content: " (-25px)";
}
#B::after {
  content: " (0)";
}
```

## Setting insets using percentages

Like length values, percentage values define offsets from the _start_ of the animation attachment range. The percentage offsets are relative to the timeline range dimension, not relative to the scrollport. For this reason, percentage values are not as intuitive as length values for most people (realizing length values weren't that intuitive either).

Here, we use `animation-range-start` and `animation-range-end` to inset the animation timeline. While we are using the same properties, we set `<percentage>` values instead of `<length>` values:

```css live-sample___inset_percent
.animated_element {
  animation-range-start: 20%;
  animation-range-end: 60%;
}
```

```css hidden live-sample___inset_percent live-sample___inset_cover
i {
  background-image: linear-gradient(
    to bottom,
    transparent calc(20% - 1px),
    #33333333 calc(20% - 1px) calc(20% + 1px),
    transparent calc(20% + 1px) calc(60% - 1px),
    #33333333 calc(60% - 1px) calc(60% + 1px),
    transparent calc(60% + 1px)
  );
}
article {
  --total: calc(var(--animElHeight) + 250px);
  background-image:
    linear-gradient(
      to top,
      transparent 0 calc(var(--total) * 0.2 - 1px),
      green calc(var(--total) * 0.2 - 1px) calc((var(--total) * 0.2) + 1px),
      transparent calc(var(--total) * 0.2 + 1px)
    ),
    linear-gradient(
      to top,
      transparent 0 calc(var(--total) * 0.6 - 1px),
      red calc(var(--total) * 0.6 - 1px) calc((var(--total) * 0.6) + 1px),
      transparent calc(var(--total) * 0.6 + 1px)
    ),
    linear-gradient(
      to top,
      transparent 0 calc(var(--containerHeight) * 0.2 - 0.5px),
      #33333333 calc(var(--containerHeight) * 0.2 - 0.5px)
        calc(var(--containerHeight) * 0.2 + 0.5px),
      transparent calc(var(--containerHeight) * 0.2 + 0.5px)
        calc(var(--containerHeight) * 0.6 - 0.5px),
      #33333333 calc(var(--containerHeight) * 0.6 - 0.5px)
        calc(var(--containerHeight) * 0.6 + 0.5px),
      transparent 0 calc(var(--containerHeight) * 0.6 + 0.5px)
    );
  background-attachment: local, local, fixed;
}
```

This defines the active interval to begin `20%` into the default attachment range and end `60%` through that same range. The default `normal` animation attachment range, which behaves as [`cover`](/en-US/docs/Web/CSS/Guides/Scroll-driven_animations/Timeline_range_names#cover), is the height of the scroll container plus the height of the subject element, meaning the range will differ based on which radio button is selected.

{{EmbedLiveSample("inset_percent", "100%", "400")}}

For illustrative purposes, there are two dark lines crossing the container at the `20%` and `60%` points of the full animation range. The animation starts when the block-start edge reaches the `20%` point, which is the bottom green line. The animation ends when the start block edge is 60% of the way through the normal range, which is the top red line.

Only when the element is `50px` tall is the top of the subject still in the scrollport when the end of the animation is reached; there are no top red lines when `250px` or `500px` are selected, as the end of the animation range is outside of the scrollport.

Based on the height of our subjects, the `20%` mark is either `60px`, `100px`, or `150px` from the end edge of the scrollport (marked by the green line, which is always in the scrollport), and the `60%` mark is `180px`, `300px`, or `450px` from the same point (marked with a red line, but only visible for the `50px` subject).

For illustrative purposes, there are two light gray lines crossing the container `20%` and `60%` of the way through the scrollport, which are `50px` and `150px` from the bottom of the scrollport, respectively. As the `animation-range-*` percentages are relative to the timeline range, not the scrollport, these lines only show how the percentages **don't** align. We've also included two horizontal light gray lines going across each subject at their own `20%` and `60%` marks. These lines align with the scrollport's light gray lines when each subjects animation starts and ends.

The following image demonstrates where the subject elements are located when the animation starts (the `0%` keyframe) and ends (the `100% keyframe`).
This image includes the insets from the animation timeline in the previous demonstration and the timeline without insets for comparison.

```html hidden live-sample___svg_insets2
<div>
  <svg viewBox="-1 -1 482 1252" xmlns="http://www.w3.org/2000/svg">
    <title>Default view progress timeline with insets</title>
    <rect class="container" width="350" height="250" x="0" y="500" />
    <rect class="small end" width="100" height="50" x="10" y="571" />
    <rect class="medium end" width="100" height="250" x="120" y="450" />
    <rect class="large end" width="100" height="500" x="230" y="300" />
    <rect class="small start" width="100" height="50" x="10" y="689" />
    <rect class="medium start" width="100" height="250" x="120" y="649" />
    <rect class="large start" width="100" height="500" x="230" y="600" />
    <rect width="96" height="48" x="122" y="602" fill="url(#g)" />
    <rect width="96" height="198" x="232" y="527" fill="url(#g)" />
    <text y="610" x="385">60%</text>
    <line x1="0" x2="385" y1="600" y2="600" />
    <line x1="0" x2="385" y1="700" y2="700" />
    <text y="710" x="385">20%</text>
  </svg>
  <svg viewBox="-1 -1 482 1252" xmlns="http://www.w3.org/2000/svg">
    <title>Default view progress timeline</title>
    <rect class="container" width="350" height="250" x="0" y="500" />
    <rect class="small end" width="100" height="50" x="10" y="450" />
    <rect class="medium end" width="100" height="250" x="125" y="250" />
    <rect class="large end" width="100" height="500" x="240" y="0" />
    <rect class="small start" width="100" height="50" x="10" y="750" />
    <rect class="medium start" width="100" height="250" x="125" y="750" />
    <rect class="large start" width="100" height="500" x="240" y="750" />
    <text y="520" x="385">100%</text>
    <line x1="0" x2="385" y1="500" y2="500" />
    <line x1="0" x2="385" y1="750" y2="750" />
    <text y="760" x="390">0%</text>
  </svg>
</div>
```

{{EmbedLiveSample("svg_insets2", "100%", "710")}}

As before, the yellow represents the position of the element when the `from` keyframe is applied, the red represents the location when the `to` keyframe is applied, and the gray represents the scrollport. The striped areas are where the red and yellow element representations overlap. For illustrative purposes, we've added dashed black horizontal lines `20%` and `60%` way through the scrollport, starting from the bottom.

The animation only begins when the element reaches the `20%` mark along the animation attachment range. This point is `60px`, `100px`, or `150px` from the bottom edge of the scroll port, depending on the size of the element. The location of the subject element at this point, representing the position of the element when the `from` or `0%` keyframe is applied, is shown in yellow.

The red represents the location of the animated element relative to the scrollport when the `to` or `100%` keyframe is applied, which is the end of the animation. This point is either `180px`, `300px`, or `450px` from the bottom edge of the scrollport, depending on the subject size. The animation occurs when the element is between the `to` and the `from` positions.

You may have noticed something interesting about the dashed horizontal lines: when the animation starts, the line that is `20%` from the end edge of the viewport is `20%` from the _top_ of the subject element and the line that is `60%` from the end edge of the viewport is `60%` from the _top_ of the subject element when the animation ends. This is what was illustrated by the very light gray lines in the live demo for this example.

### Subject size matters

As we saw when we [set insets with lengths](#setting_insets_using_lengths), the size of the subject can make a difference. When setting animation ranges, percentage values are relative to the size of animation attachment range, not the scrollport. For most named ranges, the size of the attachment range depends partially on the subject size. As percentages are based on the size of the range, the named range impacts the resolved size of the insets. Depending on the name, the start position may also change, impacting the location of the range and therefore the location of progress points.

In this example, we define an active range that is `40%` of the size of the subject:

```css live-sample___exit_percent
.animated_element {
  animation-range-start: exit-crossing -20%;
  animation-range-end: exit-crossing 20%;
}
```

```css hidden live-sample___exit_percent
article {
  background-image: none;
}
body .animated_element {
  align-self: start;
}
```

{{EmbedLiveSample("exit_percent", "100%", "400")}}

The animation lasts `40%` of the animation-attachment range. As you scroll, note how the larger the subject, the longer the range. With exit-crossing, the animation range is not cropped; it is the size of the subject even if the subject is larger than the viewport, with the range abutting the start edge of the scrollport, and extending off the end edge if the subject is larger than the scrollport.

With the `-20%` and `20%` insets, the `50px` subject's will animation over `20px`: the animation starts when the subject's end is `-10px` from range start, or `60px` from exiting the screen, and ends when the subject's end is `40px` from exiting the screen. The middle subject will animate over `100px`: the animation starts when the subject end is `-50px` from range start, which is `50px` off of the scrollport's end edge, and ends when the subject's end is `50px` into the scrollport. The large subject animates over `200px`, starting when the bottom is `600px` from the container's start edge, with only `150px` in view, and ends when the bottom is 400px from that start edge, when `100px` have scrolled off the start edge.

### Percentages equal to the scrollport

When it comes to offsetting with percentages, the least complicated named timeline range is `contain`. With `contain`, the animation range is the size of the scrollport, meaning the start and end percentages are relative to the scrollport. For this reason, when using offsets, you may want to use `contain` instead of letting the range default and resolve to `cover`.

The `contain` range fully _contains_ the animation within the scrollport. It represents the range during which the principal box is either fully contained by, or fully covers, its view progress visibility range within the scrollport. With `contain`, if the subject is the same size or smaller than the scrollport, it can be fully visible. If the element is the same size as the container, however, the animation occurs over `0px`. This means that it runs, but it is not visible to the user.

In other words, without needing to know the size of the container or the subjects, we are able to limit our animation to the middle of scrollport, though the animation will happen over `0px` if the subject is the same size as the scrollport.

```css live-sample___center
.animated_element {
  animation-range-start: contain 25%;
  animation-range-end: contain 75%;
}
```

```css hidden live-sample___center
article {
  background-image: linear-gradient(
    transparent 25%,
    #ededed 25% 75%,
    transparent 75%
  );
}
body .animated_element {
  align-self: center;
}

.animated_element {
  background-image:
    linear-gradient(black, black), linear-gradient(black, black);
  background-size: 1px 1px;
  background-position:
    center 25%,
    center 75%;
  background-repeat: repeat-x;
```

{{EmbedLiveSample("center", "100%", "310")}}

The horizontal lines denote the middle half of the scrollport and the middle half of each subject.

```html hidden live-sample___svg_contain live-sample___svg_insets2 live-sample___svg_view
<svg class="gradient">
  <title>Striped repeating gradient</title>
  <defs>
    <linearGradient
      id="g"
      x1="0"
      y1="0"
      x2="20"
      y2="20"
      spreadMethod="repeat"
      gradientUnits="userSpaceOnUse"
    >
      <stop offset="50%" stop-color="red" />
      <stop offset="50%" stop-color="yellow" />
    </linearGradient>
  </defs>
</svg>
```

```css hidden live-sample___svg_contain live-sample___svg_insets2 live-sample___svg_view
body::before {
  display: block;
  text-align: center;
  font-family: sans-serif;
  font-size: 1.5rem;
}
div {
  display: flex;
  gap: 20px;
}
svg {
  width: 260px;
}
rect {
  stroke: black;
  stroke-width: 3;
}
.start {
  fill: yellow;
}
.end {
  fill: red;
}
.container {
  fill: #dedede;
}
text {
  font: 40px monospace;
  fill: black;
}
line {
  stroke: black;
  stroke-width: 2;
  stroke-dasharray: 7;
}
.gradient {
  height: 1px;
  width: 1px;
  position: absolute;
  top: -100px;
}
```

```html hidden live-sample___initial live-sample___entry_exit live-sample___inset_percent live-sample___inset_length live-sample___inset_cover live-sample___inset_contain live-sample___cover_contain live-sample___exit_length_negative live-sample___entry_crossing live-sample___exit_crossing
<main>
  <article>
    <p>&nbsp;</p>
    <p>&nbsp;</p>
    <p>Scroll down ⇩</p>
    <p>&nbsp;</p>
    <p>&nbsp;</p>
    <section class="one animated_element">
      <div>
        <i>Animated Element</i>
        <span></span>
      </div>
    </section>
    <section class="double">
      <div>
        <i id="A" class="animated_element">A</i>
        <i id="B" class="animated_element">B</i>
      </div>
    </section>
    <p>&nbsp;</p>
    <p>&nbsp;</p>
    <p>&nbsp;</p>
    <p>&nbsp;</p>
    <p>Scroll up ⇧</p>
  </article>
</main>
```

```html hidden live-sample___initial live-sample___entry_exit live-sample___inset_percent live-sample___inset_length live-sample___inset_cover live-sample___inset_contain live-sample___cover_contain live-sample___entry_crossing live-sample___exit_crossing live-sample___exit_length_negative
<fieldset>
  <legend>Select the height of the animated element</legend>

  <label><input name="height" value="50" type="radio" checked /> 50px</label>
  <label><input name="height" value="250" type="radio" /> 250px</label>
  <label><input name="height" value="500" type="radio" /> 500px</label>
</fieldset>
<fieldset class="double">
  <legend>Select the animation range</legend>

  <label><input name="range" value="20" type="radio" checked />20% / 60%</label>
  <label><input name="range" value="0" type="radio" /> 0% / 100%</label>
</fieldset>
```

```css hidden live-sample___initial live-sample___entry_exit live-sample___inset_percent live-sample___inset_length live-sample___inset_cover live-sample___inset_contain live-sample___cover_contain live-sample___exit_length_negative live-sample___entry_crossing live-sample___exit_crossing
@layer {
  :root {
    --animElHeight: 50px;
    --animElHeightWord: "50px";
    --barColor: black;
    padding-top: 20px;
    --containerHeight: 250px;
  }
  body:has(input[value="250"]:checked) {
    --animElHeight: 250px;
    --animElHeightWord: "250px";
  }
  body:has(input[value="500"]:checked) {
    --animElHeight: 500px;
    --animElHeightWord: "500px";
  }
  main {
    padding: 20px 0 0 20px;
    margin-bottom: 2em;
  }
  article {
    outline: 3px dashed;
    width: 475px;
    margin: auto;
    overflow: scroll;
    position: relative;
    height: var(--containerHeight);
    box-sizing: content-box;
  }

  p {
    padding: 10px;
    margin: 10px;
  }

  section {
    --clr: yellow;
    --words: "Animation not started";
    position: relative;
    margin: 20px;
    text-align: center;
  }
  .one,
  .double i {
    animation: showAnim step-end 1 forwards;
    animation-timeline: view();
  }
  i,
  .animated_element {
    background-color: hsl(from var(--clr) h s calc(l * 1.4));
    display: block;
    height: var(--animElHeight);
    line-height: var(--animElHeight);
  }
  span {
    background-color: hsl(from var(--clr) h s 90%);
    border: 5px solid hsl(from var(--clr) h s 20%);
    min-width: 250px;
    height: 30px;
    line-height: 30px;
  }
  span,
  i {
    font-family: sans-serif;
    font-size: 1.5rem;
  }
  span::before {
    content: var(--words);
  }
  span {
    position: fixed;
    top: 10px;
    left: 10px;
    padding: 10px;
  }
  i::after {
    content: " ( " var(--animElHeightWord) " )";
  }
  label {
    padding-right: 2em;
  }
  legend {
    margin-top: 2em;
  }

  @keyframes showAnim {
    from {
      --clr: green;
      --words: "Currently animating";
    }
    to {
      --clr: red;
      --words: "Animation complete";
    }
  }
  body::before {
    display: block;
    text-align: center;
    font-family: sans-serif;
    font-size: 1.5rem;
  }

  @layer no-support {
    @supports not (animation-timeline: view()) {
      body::before {
        content: "Your browser doesn't support view progress scrolling.";
        background-color: wheat;
        display: block;
        text-align: center;
      }
    }
  }
}
```

```css hidden live-sample___initial live-sample___inset_percent live-sample___inset_length live-sample___inset_cover live-sample___inset_contain
.double {
  display: none;
}
```

```css hidden live-sample___cover_contain live-sample___exit_length_negative live-sample___entry_crossing live-sample___exit_crossing live-sample___entry_exit
.one {
  display: none;
}
.double div {
  display: flex;
  gap: 10px;
}
```

## See also

- {{cssxref("timeline-range-name")}} data type
- [Keyframe selectors](/en-US/docs/Web/CSS/Reference/Selectors/Keyframe_selectors)
- [Scroll-driven animation timelines](/en-US/docs/Web/CSS/Guides/Scroll-driven_animations/Timelines)
- [Scroll-driven animation](/en-US/docs/Web/CSS/Guides/Scroll-driven_animations) module
- [CSS animations](/en-US/docs/Web/CSS/Guides/Animations) module
- [Web Animations API](/en-US/docs/Web/API/Web_Animations_API)

`````

### Expected (prettier)

`````md
---
title: Understanding timeline insets
slug: Web/CSS/Guides/Scroll-driven_animations/Timeline_insets
page-type: guide
sidebar: cssref
---

By default, [view progress timelines](/en-US/docs/Web/CSS/Guides/Scroll-driven_animations/Timelines#view_progress_timelines) track elements across the entire [animation attachment range](/en-US/docs/Web/CSS/Guides/Scroll-driven_animations/Timeline_range_names#the_animation_attachment_range). The `0%` progress point is at the start of the range, while the `100%` progress point is at the end. The animation attachment range can be changed by setting a [timeline range name](/en-US/docs/Web/CSS/Guides/Scroll-driven_animations/Timeline_range_names), and the location of the `0%` and `100%` progress points along the range can be adjusted by setting length or percentage-based inset values.

This guide explains how to limit the animation timeline to a specific portion of the animation timeline range using length or percentage inset values.

## Animation timelines: a primer

[CSS animations](/en-US/docs/Web/CSS/Guides/Animations) are created by defining named {{cssxref("@keyframes")}} animations, which specify an animation's behavior, and then attaching the keyframe animation to an element using the animation's name.

The element's animation timeline, defined by the {{cssxref("animation-timeline")}} property, determines how and when the element progresses through those keyframes. By default, the timeline is time-based, using the document's default time-based {{domxref("DocumentTimeline")}}.

The [CSS scroll-driven animation](/en-US/docs/Web/CSS/Guides/Scroll-driven_animations) module defines scroll progress and view progress timelines, which are methods for animating property values along a scroll-based timeline rather than the default time-based document timeline. In this article, we'll only discuss view progress timelines, as [scroll progress timelines](/en-US/docs/Web/CSS/Guides/Scroll-driven_animations/Timelines#scroll_progress_timelines) are not relevant to timeline insets.

### View progress timelines

With [view progress timelines](/en-US/docs/Web/CSS/Guides/Scroll-driven_animations/Timelines#view_progress_timelines), the animation's timeline, or progress, is driven by element visibility instead of the passage of time, with keyframe progression tied to the subject element position and visibility within the scroll container. The animation advances and reverses as the element advances or reverses through the scrollport. The animation only occurs when at least part of the element is visible within its scrollport, pausing when scrolling pauses.

```css live-sample___svg_view
.animated_element {
  animation-name: nameOfAnimation;
  animation-timeline: view();
}
```

Setting an {{cssxref("animation-name")}} applies the animation to the selected element.

> [!NOTE]
> The `animation-timeline` property should always come after any `animation` shorthand declarations. While the shorthand property can't be used to set the `animation-timeline` property, it does reset the timeline to the default time-based document timeline.

> [!NOTE]
> In all the examples, the {{glossary("scroll container")}} is `250px` tall and we are using the default values for {{cssxref("animation-iteration-count")}} (`1`), {{cssxref("animation-delay")}} (`0s`), and {{cssxref("animation-direction")}} (`normal`). We set the {{cssxref("animation-timing-function")}} to `step-end` and the {{cssxref("animation-fill-mode")}} is to `forward` to make the it more apparent when the animation iteration has not yet started, when it is active, and when it is complete. See the [Using CSS animations guide](/en-US/docs/Web/CSS/Guides/Animations/Using) to learn more.

As you scroll up, the animation progresses. As you scroll down, the animation reverses.

{{EmbedLiveSample("initial", "100%", "400")}}

In this example, whenever any part of the subject element is visible in the scrollport, the animation occurs. By default, view progress animations begin just as the top edge of the subject element aligns with the bottom edge of the scroll container and end, reaching `100%` progress, when the end edge aligns with the start edge of the container, no matter the size of the subject element. By default, the animation is applied when any part of the subject is visible within the scrollport.

### Animation attachment ranges

In a [view progress timeline](/en-US/docs/Web/CSS/Guides/Scroll-driven_animations/Timelines#view_progress_timelines), when no animation range properties are defined, the `<timeline-range-name>` is `normal`, which defaults to `cover`. The animation is applied whenever any portion of the subject element is visible, meaning the default **animation attachment range** is the sum of the height of the scroll container and the height of the subject element, with that extra height being at the scroll end edge. In our example, as the scroll container is `250px` tall, and the subject is `50px`, `250px`, or `500px` tall, with the vertical animation attachment range being `300px`, `500px`, or `750px` respectively.

The `0%` progression occurs when the subject element's start edge intersects the scrollport at the end edge, reaching `100%` progress when the subject's end edge exits via the scrollport's start edge. These are the subject and scrollport's top and bottom edges when scrolling vertically, and the left and right or right and left edges when scrolling horizontally, depending on the writing mode.

The following diagram illustrates the position of the subject at the `0%` and `100%` progress points for the three subject sizes:

```html hidden live-sample___svg_view
<div>
  <svg viewBox="-1 -1 462 1252" xmlns="http://www.w3.org/2000/svg">
    <title>Default view progress timeline</title>
    <rect class="container" width="350" height="250" x="0" y="500" />
    <rect class="small end" width="100" height="50" x="10" y="450" />
    <rect class="medium end" width="100" height="250" x="125" y="250" />
    <rect class="large end" width="100" height="500" x="240" y="0" />
    <rect class="small start" width="100" height="50" x="10" y="750" />
    <rect class="medium start" width="100" height="250" x="125" y="750" />
    <rect class="large start" width="100" height="500" x="240" y="750" />
    <text y="520" x="360">100%</text>
    <line x1="0" x2="350" y1="500" y2="500" />
    <line x1="0" x2="350" y1="750" y2="750" />
    <text y="760" x="360">0%</text>
  </svg>
</div>
```

{{EmbedLiveSample("svg_view", "100%", "720")}}

The yellow subject elements represents the position of the element when the `from` keyframe is applied, which is the animation range's `0%` progress mark. The red represents the location of the animated element relative to the scrollport when the `to` keyframe is applied, which is the end of the animation, or the `100%` progress mark. The gray represents the scrollport.

By default, the element animates while it is "in view", but this default definition of "in view" may not fit your needs. Fortunately, we can control which edges define the edges of the animation attachment range and then offset the start and end of that range with the animation range properties.

### Animation range properties

The {{cssxref("animation-range")}} properties enable specifying a named timeline range, such as `contain` or `exit-crossing`, which changes the range used from the default `cover` range. You can also include a {{cssxref("length-percentage")}} value, which insets the attachment range from the start of the range. Percentages are relative to the named or default timeline range.

Named timeline ranges define the portions of a {{domxref("ViewTimeline")}} that define an animation's range, specifying the start and end of the animations's attachment range.

The `animation-range` property is a shorthand property, defining the {{cssxref("animation-range-start")}} and {{cssxref("animation-range-end")}} properties. The `animation-range-start` defines the position of the subject element when the animation starts. The `animation-range-end` defines the position of the subject element when the animation ends.

See the [timeline range names guide](/en-US/docs/Web/CSS/Guides/Scroll-driven_animations/Timeline_range_names) to learn about the different named timeline ranges. This guide focuses on how the {{cssxref("length-percentage")}} inset values work.

## Setting insets using lengths

The `animation-range-start` and `animation-range-end` properties each accept a named animation range, a {{cssxref("length-percentage")}} offset value, or both. Any length or percentage offset is measured from the _start_ of the animation attachment range.

When a {{cssxref("length")}} is set, the offset is fairly intuitive.
Here we use the `animation-range-start` and `animation-range-end` properties to inset the animation timeline. This defines a subsection of the element's full animation attachment range as the active interval, with the `<length>` values specifying distances from the start of the default `normal` animation attachment range.

```css live-sample___inset_length
.animated_element {
  animation-range-start: 1em;
  animation-range-end: 125px;
}
```

The start and end of the animation range are `1em` and `125px` from the start of the animation attachment range, respectively. Because the timeline range default is `normal`, which resolves to `cover`, the start of the animation attachment range is the block end edge of the container.

```css hidden live-sample___inset_length
:root {
  --start: 1em;
  --end: 125px;
}

article {
  background-image: linear-gradient(
    to top,
    transparent calc(var(--start) - 1px),
    #cccccc calc(var(--start) - 1px) calc(var(--start) + 1px),
    transparent calc(var(--start) + 1px) calc(var(--end) - 1px),
    #cccccc calc(var(--end) - 1px) calc(var(--end) + 1px),
    transparent calc(var(--end) + 1px)
  );
}
```

{{EmbedLiveSample("inset_length", "100%", "400")}}

We've added lines `1em` and `125px` from the block end edge of the scroll container. The animation starts when the block start edge of the subject element reaches the `1em` line and ends when it reaches the `125px` line.

In this case, as the animation attachment range resolves to `cover` for both the start and end inset values, the location of the insets is fairly straightforward.

### Effect of named ranges on length offsets

The offset distance is always from the start of the associated animation range. In this example, we set the `animation-range-start` to be `50px` from the start of the default `normal` range and set the `animation-range-end` to be `100px` from the start of the explicitly set `entry` range:

```css live-sample___different_length
.animated_element {
  animation-range-start: 50px;
  animation-range-end: entry 100px;
}
```

```html hidden live-sample___different_length live-sample___exit_length live-sample___exit_percent live-sample___center
<main>
  <article>
    <p>&nbsp;</p>
    <p>&nbsp;</p>
    <p>Scroll down ⇩</p>
    <p>&nbsp;</p>
    <p>&nbsp;</p>
    <section class="triple">
      <div>
        <i id="A" class="animated_element">50px</i>
        <i id="B" class="animated_element">250px</i>
        <i id="C" class="animated_element">500px</i>
      </div>
    </section>
    <p>&nbsp;</p>
    <p>&nbsp;</p>
    <p>&nbsp;</p>
    <p>&nbsp;</p>
    <p>Scroll up ⇧</p>
  </article>
</main>
```

{{EmbedLiveSample("different_length", "100%", "310")}}

As the start edge of both the `normal` and `entry` ranges is the container's end edge, the animation begins when the subject start edge is `50px` from the bottom of the scrollport and ends, reaching `100%` progress, when the subject start edge is `100px` from the bottom of the scrollport, irrespective of the subject size. While the size of the `entry` range is different for the three different subject sizes, in this case, the size of the underlying range didn't matter.

### Length offsets with varying ranges

The size of range matters when the range doesn't start at the element's end edge, as is the case with `exit` and `exit-crossing`, or if the offset is a percentage value. This fact, and the fact that you can mix and match animation range names, make view progress timeline offsets a bit more complicated to understand than non-offsetted [timeline range names](/en-US/docs/Web/CSS/Guides/Scroll-driven_animations/Timeline_range_names).

For example, when setting `exit` as the timeline range name, the subject size matters as it determines the location of the range's end edge.

```css live-sample___exit_length
.animated_element {
  animation-range-start: entry 60px;
  animation-range-end: exit 75px;
}
```

With both `entry` and `exit`, the range is the size of the subject, with the size clamped to the size of the scrollport. This means that the height of the `entry` and `exit` ranges are the height of the box in both the `50px` and `250px` examples, while in the `500px` example, the range is clamped to the height of the scrollport, which is `250px` tall.

{{EmbedLiveSample("exit_length", "100%", "310")}}

We added a few lines to facilitate the explanations that follow: the bottom blue line is `60px` from the end edge of the scrollport, and the top red line is `75px` from that same edge. These are where the animation range start and end respectively.

This example demonstrates several important features, which we'll explain in greater detail, including:

- Offsets are [measured from their respective named ranges](#measured_from_the_start_edge_of_the_range)
- Offsets can occur [beyond the edges of the scrollport](#beyond_the_scrollport_edges)
- [Ranges may be clamped](#effects_of_clamping) if the subject is larger than the scrollport

#### Measured from the start edge of the range

As the offset position is always relative to the start of the declaration's animation range, the start of the animation for all three elements occurs when the elements start edge crosses the point that is `60px` from the start of the `entry` range.

The `animation-range-end` value defines the position where the animation. The `exit 75px` value basically means "when `75px` of the subject has exited the start edge of the scrollport." This varies for each subject. For the `50px` subject, this only occurs `25px` after it has left the scrollport; when the element is not visible. The animation range end for both the `250px` and `500px` subjects occurs when their bottom end edge intersects the top blue line; 75 pixels from the end edge of the scrollport. Why are their end offsets the same? Because of [clamping](#effects_of_clamping)! The maximum size of the named animation range is clamped to the size of the scrollport. The `exit` range for both subjects is the same, so the range end offsets are the same.

#### Beyond the scrollport edges

For our `50px` tall subject, the `exit` range is 50px tall abutting the start edge of the scroll port. Setting `animation-range-end: exit 75px` for any element less than `75px` tall means the end of the range is outside the scrollport, as the point `75px` from the start of the `exit` range is past the start edge of the scrollport. In our example, the end of the animation range for the `50px` subject occurs when the subject's start edge is `75px` past the scrollport's start edge. The animation ends, reaching the `to` keyframe and the {{domxref("Element/animationend_event", "animationend")}} event, only when (and if) the element is scrolled `25px` out of view.

The animation ends even if the animation range end is outside of the scrollport, as long as there is room to scroll to that point. Had we set `animation-range-end: exit 250px`, the animation would have ended when the end edge of the medium and tall subjects exited the scrollport at the container's start edge.

With the end set to `exit 250px`, the small subject's animation might not end, as there might not be `450px` of content after the subject for the user to scroll to before the end point is reached.

#### Effects of clamping

With our `250px` tall container, when the subject is `250px` or `500px` tall, the `exit` range is the size of the container, with the start being the scroll container's end edge. With a `75px` offset, the end of the animation occurs when the end edge of the subject is `75px` from the end edge of the scroll container (denoted by the top red line).

As the offset position is always relative to the start of the named or default animation range, in our example, clamping impacts the large subject's `animation-range-end`. We set the end of the range to `exit 75px` which is `75px` from the start edge of the `exit` range. When the subject is the same size as the scrollport (our `250px` subject) or larger (our `500px` subject), the animation range end is `75px` from the end edge of the scrollport, which is `75px` from the start of the scrollport-clamped `exit` range.

```css hidden live-sample___exit_length
article {
  background-image: linear-gradient(
    to top,
    transparent 59.5px,
    blue 59.5px 60.5px,
    transparent 60.5px 74.5px,
    red 74.5px 75.5px,
    transparent 75.5px /* 174.5px,
    green 174.5px 154.5px,
    transparent 175.5px*/
  );
}
.animated_element {
  align-self: flex-end;
}
```

```css hidden live-sample___different_length live-sample___exit_length live-sample___exit_percent live-sample___center
@layer setup {
  #A {
    height: 50px;
  }
  #B {
    height: 250px;
  }
  #C {
    height: 500px;
  }
  div {
    display: flex;
    gap: 1em;
  }
  main {
    padding: 20px 0 0 20px;
    margin-bottom: 2em;
  }
  article {
    outline: 3px dashed;
    width: 475px;
    margin: auto;
    overflow: scroll;
    position: relative;
    height: 250px;
    box-sizing: content-box;
    background-image: linear-gradient(
      to top,
      transparent 49.5px,
      #666666 49.5px 50.5px,
      transparent 50.5px 99.5px,
      #666666 99.5px 100.5px,
      transparent 100.5px
    );
    background-origin: content-box;
  }

  p {
    padding: 10px;
    margin: 10px;
  }

  .animated_element {
    --clr: yellow;
    background-color: hsl(from var(--clr) h s calc(l * 1.4));
    display: block;
    animation: showAnim step-end 1 forwards;
    animation-timeline: view();
    flex: 1 0 auto;
  }

  i {
    font-family: sans-serif;
    font-size: 1.5rem;
  }

  @keyframes showAnim {
    from {
      --clr: green;
    }
    to {
      --clr: red;
    }
  }
  @layer no-support {
    @supports not (animation-timeline: view()) {
      body::before {
        content: "Your browser doesn't support view progress scrolling.";
        background-color: wheat;
        display: block;
        text-align: center;
      }
    }
  }
}
```

### Negative lengths

Up to this point, all the offsets have been greater than zero. It's important to note that negative lengths are valid. A negative offset on the `animation-range-start` makes the range longer while a negative offset on the `animation-range-end` makes the range shorter.

Let's compare the negative insets compared to the `0` values:

```css live-sample___exit_length_negative
#A {
  animation-range-start: contain -25px;
  animation-range-end: exit -25px;
}
#B {
  animation-range-start: contain 0;
  animation-range-end: exit 0;
}
```

{{EmbedLiveSample("exit_length_negative", "100%", "380")}}

The first animation range is offset by `25px` toward the container end edge.

```css hidden live-sample___exit_length_negative
fieldset.double {
  display: none;
}
#A::after {
  content: " (-25px)";
}
#B::after {
  content: " (0)";
}
```

## Setting insets using percentages

Like length values, percentage values define offsets from the _start_ of the animation attachment range. The percentage offsets are relative to the timeline range dimension, not relative to the scrollport. For this reason, percentage values are not as intuitive as length values for most people (realizing length values weren't that intuitive either).

Here, we use `animation-range-start` and `animation-range-end` to inset the animation timeline. While we are using the same properties, we set `<percentage>` values instead of `<length>` values:

```css live-sample___inset_percent
.animated_element {
  animation-range-start: 20%;
  animation-range-end: 60%;
}
```

```css hidden live-sample___inset_percent live-sample___inset_cover
i {
  background-image: linear-gradient(
    to bottom,
    transparent calc(20% - 1px),
    #33333333 calc(20% - 1px) calc(20% + 1px),
    transparent calc(20% + 1px) calc(60% - 1px),
    #33333333 calc(60% - 1px) calc(60% + 1px),
    transparent calc(60% + 1px)
  );
}
article {
  --total: calc(var(--animElHeight) + 250px);
  background-image:
    linear-gradient(
      to top,
      transparent 0 calc(var(--total) * 0.2 - 1px),
      green calc(var(--total) * 0.2 - 1px) calc((var(--total) * 0.2) + 1px),
      transparent calc(var(--total) * 0.2 + 1px)
    ),
    linear-gradient(
      to top,
      transparent 0 calc(var(--total) * 0.6 - 1px),
      red calc(var(--total) * 0.6 - 1px) calc((var(--total) * 0.6) + 1px),
      transparent calc(var(--total) * 0.6 + 1px)
    ),
    linear-gradient(
      to top,
      transparent 0 calc(var(--containerHeight) * 0.2 - 0.5px),
      #33333333 calc(var(--containerHeight) * 0.2 - 0.5px)
        calc(var(--containerHeight) * 0.2 + 0.5px),
      transparent calc(var(--containerHeight) * 0.2 + 0.5px)
        calc(var(--containerHeight) * 0.6 - 0.5px),
      #33333333 calc(var(--containerHeight) * 0.6 - 0.5px)
        calc(var(--containerHeight) * 0.6 + 0.5px),
      transparent 0 calc(var(--containerHeight) * 0.6 + 0.5px)
    );
  background-attachment: local, local, fixed;
}
```

This defines the active interval to begin `20%` into the default attachment range and end `60%` through that same range. The default `normal` animation attachment range, which behaves as [`cover`](/en-US/docs/Web/CSS/Guides/Scroll-driven_animations/Timeline_range_names#cover), is the height of the scroll container plus the height of the subject element, meaning the range will differ based on which radio button is selected.

{{EmbedLiveSample("inset_percent", "100%", "400")}}

For illustrative purposes, there are two dark lines crossing the container at the `20%` and `60%` points of the full animation range. The animation starts when the block-start edge reaches the `20%` point, which is the bottom green line. The animation ends when the start block edge is 60% of the way through the normal range, which is the top red line.

Only when the element is `50px` tall is the top of the subject still in the scrollport when the end of the animation is reached; there are no top red lines when `250px` or `500px` are selected, as the end of the animation range is outside of the scrollport.

Based on the height of our subjects, the `20%` mark is either `60px`, `100px`, or `150px` from the end edge of the scrollport (marked by the green line, which is always in the scrollport), and the `60%` mark is `180px`, `300px`, or `450px` from the same point (marked with a red line, but only visible for the `50px` subject).

For illustrative purposes, there are two light gray lines crossing the container `20%` and `60%` of the way through the scrollport, which are `50px` and `150px` from the bottom of the scrollport, respectively. As the `animation-range-*` percentages are relative to the timeline range, not the scrollport, these lines only show how the percentages **don't** align. We've also included two horizontal light gray lines going across each subject at their own `20%` and `60%` marks. These lines align with the scrollport's light gray lines when each subjects animation starts and ends.

The following image demonstrates where the subject elements are located when the animation starts (the `0%` keyframe) and ends (the `100% keyframe`).
This image includes the insets from the animation timeline in the previous demonstration and the timeline without insets for comparison.

```html hidden live-sample___svg_insets2
<div>
  <svg viewBox="-1 -1 482 1252" xmlns="http://www.w3.org/2000/svg">
    <title>Default view progress timeline with insets</title>
    <rect class="container" width="350" height="250" x="0" y="500" />
    <rect class="small end" width="100" height="50" x="10" y="571" />
    <rect class="medium end" width="100" height="250" x="120" y="450" />
    <rect class="large end" width="100" height="500" x="230" y="300" />
    <rect class="small start" width="100" height="50" x="10" y="689" />
    <rect class="medium start" width="100" height="250" x="120" y="649" />
    <rect class="large start" width="100" height="500" x="230" y="600" />
    <rect width="96" height="48" x="122" y="602" fill="url(#g)" />
    <rect width="96" height="198" x="232" y="527" fill="url(#g)" />
    <text y="610" x="385">60%</text>
    <line x1="0" x2="385" y1="600" y2="600" />
    <line x1="0" x2="385" y1="700" y2="700" />
    <text y="710" x="385">20%</text>
  </svg>
  <svg viewBox="-1 -1 482 1252" xmlns="http://www.w3.org/2000/svg">
    <title>Default view progress timeline</title>
    <rect class="container" width="350" height="250" x="0" y="500" />
    <rect class="small end" width="100" height="50" x="10" y="450" />
    <rect class="medium end" width="100" height="250" x="125" y="250" />
    <rect class="large end" width="100" height="500" x="240" y="0" />
    <rect class="small start" width="100" height="50" x="10" y="750" />
    <rect class="medium start" width="100" height="250" x="125" y="750" />
    <rect class="large start" width="100" height="500" x="240" y="750" />
    <text y="520" x="385">100%</text>
    <line x1="0" x2="385" y1="500" y2="500" />
    <line x1="0" x2="385" y1="750" y2="750" />
    <text y="760" x="390">0%</text>
  </svg>
</div>
```

{{EmbedLiveSample("svg_insets2", "100%", "710")}}

As before, the yellow represents the position of the element when the `from` keyframe is applied, the red represents the location when the `to` keyframe is applied, and the gray represents the scrollport. The striped areas are where the red and yellow element representations overlap. For illustrative purposes, we've added dashed black horizontal lines `20%` and `60%` way through the scrollport, starting from the bottom.

The animation only begins when the element reaches the `20%` mark along the animation attachment range. This point is `60px`, `100px`, or `150px` from the bottom edge of the scroll port, depending on the size of the element. The location of the subject element at this point, representing the position of the element when the `from` or `0%` keyframe is applied, is shown in yellow.

The red represents the location of the animated element relative to the scrollport when the `to` or `100%` keyframe is applied, which is the end of the animation. This point is either `180px`, `300px`, or `450px` from the bottom edge of the scrollport, depending on the subject size. The animation occurs when the element is between the `to` and the `from` positions.

You may have noticed something interesting about the dashed horizontal lines: when the animation starts, the line that is `20%` from the end edge of the viewport is `20%` from the _top_ of the subject element and the line that is `60%` from the end edge of the viewport is `60%` from the _top_ of the subject element when the animation ends. This is what was illustrated by the very light gray lines in the live demo for this example.

### Subject size matters

As we saw when we [set insets with lengths](#setting_insets_using_lengths), the size of the subject can make a difference. When setting animation ranges, percentage values are relative to the size of animation attachment range, not the scrollport. For most named ranges, the size of the attachment range depends partially on the subject size. As percentages are based on the size of the range, the named range impacts the resolved size of the insets. Depending on the name, the start position may also change, impacting the location of the range and therefore the location of progress points.

In this example, we define an active range that is `40%` of the size of the subject:

```css live-sample___exit_percent
.animated_element {
  animation-range-start: exit-crossing -20%;
  animation-range-end: exit-crossing 20%;
}
```

```css hidden live-sample___exit_percent
article {
  background-image: none;
}
body .animated_element {
  align-self: start;
}
```

{{EmbedLiveSample("exit_percent", "100%", "400")}}

The animation lasts `40%` of the animation-attachment range. As you scroll, note how the larger the subject, the longer the range. With exit-crossing, the animation range is not cropped; it is the size of the subject even if the subject is larger than the viewport, with the range abutting the start edge of the scrollport, and extending off the end edge if the subject is larger than the scrollport.

With the `-20%` and `20%` insets, the `50px` subject's will animation over `20px`: the animation starts when the subject's end is `-10px` from range start, or `60px` from exiting the screen, and ends when the subject's end is `40px` from exiting the screen. The middle subject will animate over `100px`: the animation starts when the subject end is `-50px` from range start, which is `50px` off of the scrollport's end edge, and ends when the subject's end is `50px` into the scrollport. The large subject animates over `200px`, starting when the bottom is `600px` from the container's start edge, with only `150px` in view, and ends when the bottom is 400px from that start edge, when `100px` have scrolled off the start edge.

### Percentages equal to the scrollport

When it comes to offsetting with percentages, the least complicated named timeline range is `contain`. With `contain`, the animation range is the size of the scrollport, meaning the start and end percentages are relative to the scrollport. For this reason, when using offsets, you may want to use `contain` instead of letting the range default and resolve to `cover`.

The `contain` range fully _contains_ the animation within the scrollport. It represents the range during which the principal box is either fully contained by, or fully covers, its view progress visibility range within the scrollport. With `contain`, if the subject is the same size or smaller than the scrollport, it can be fully visible. If the element is the same size as the container, however, the animation occurs over `0px`. This means that it runs, but it is not visible to the user.

In other words, without needing to know the size of the container or the subjects, we are able to limit our animation to the middle of scrollport, though the animation will happen over `0px` if the subject is the same size as the scrollport.

```css live-sample___center
.animated_element {
  animation-range-start: contain 25%;
  animation-range-end: contain 75%;
}
```

```css hidden live-sample___center
article {
  background-image: linear-gradient(
    transparent 25%,
    #ededed 25% 75%,
    transparent 75%
  );
}
body .animated_element {
  align-self: center;
}

.animated_element {
  background-image:
    linear-gradient(black, black), linear-gradient(black, black);
  background-size: 1px 1px;
  background-position:
    center 25%,
    center 75%;
  background-repeat: repeat-x;
```

{{EmbedLiveSample("center", "100%", "310")}}

The horizontal lines denote the middle half of the scrollport and the middle half of each subject.

```html hidden live-sample___svg_contain live-sample___svg_insets2 live-sample___svg_view
<svg class="gradient">
  <title>Striped repeating gradient</title>
  <defs>
    <linearGradient
      id="g"
      x1="0"
      y1="0"
      x2="20"
      y2="20"
      spreadMethod="repeat"
      gradientUnits="userSpaceOnUse"
    >
      <stop offset="50%" stop-color="red" />
      <stop offset="50%" stop-color="yellow" />
    </linearGradient>
  </defs>
</svg>
```

```css hidden live-sample___svg_contain live-sample___svg_insets2 live-sample___svg_view
body::before {
  display: block;
  text-align: center;
  font-family: sans-serif;
  font-size: 1.5rem;
}
div {
  display: flex;
  gap: 20px;
}
svg {
  width: 260px;
}
rect {
  stroke: black;
  stroke-width: 3;
}
.start {
  fill: yellow;
}
.end {
  fill: red;
}
.container {
  fill: #dedede;
}
text {
  font: 40px monospace;
  fill: black;
}
line {
  stroke: black;
  stroke-width: 2;
  stroke-dasharray: 7;
}
.gradient {
  height: 1px;
  width: 1px;
  position: absolute;
  top: -100px;
}
```

```html hidden live-sample___initial live-sample___entry_exit live-sample___inset_percent live-sample___inset_length live-sample___inset_cover live-sample___inset_contain live-sample___cover_contain live-sample___exit_length_negative live-sample___entry_crossing live-sample___exit_crossing
<main>
  <article>
    <p>&nbsp;</p>
    <p>&nbsp;</p>
    <p>Scroll down ⇩</p>
    <p>&nbsp;</p>
    <p>&nbsp;</p>
    <section class="one animated_element">
      <div>
        <i>Animated Element</i>
        <span></span>
      </div>
    </section>
    <section class="double">
      <div>
        <i id="A" class="animated_element">A</i>
        <i id="B" class="animated_element">B</i>
      </div>
    </section>
    <p>&nbsp;</p>
    <p>&nbsp;</p>
    <p>&nbsp;</p>
    <p>&nbsp;</p>
    <p>Scroll up ⇧</p>
  </article>
</main>
```

```html hidden live-sample___initial live-sample___entry_exit live-sample___inset_percent live-sample___inset_length live-sample___inset_cover live-sample___inset_contain live-sample___cover_contain live-sample___entry_crossing live-sample___exit_crossing live-sample___exit_length_negative
<fieldset>
  <legend>Select the height of the animated element</legend>

  <label><input name="height" value="50" type="radio" checked /> 50px</label>
  <label><input name="height" value="250" type="radio" /> 250px</label>
  <label><input name="height" value="500" type="radio" /> 500px</label>
</fieldset>
<fieldset class="double">
  <legend>Select the animation range</legend>

  <label><input name="range" value="20" type="radio" checked />20% / 60%</label>
  <label><input name="range" value="0" type="radio" /> 0% / 100%</label>
</fieldset>
```

```css hidden live-sample___initial live-sample___entry_exit live-sample___inset_percent live-sample___inset_length live-sample___inset_cover live-sample___inset_contain live-sample___cover_contain live-sample___exit_length_negative live-sample___entry_crossing live-sample___exit_crossing
@layer {
  :root {
    --animElHeight: 50px;
    --animElHeightWord: "50px";
    --barColor: black;
    padding-top: 20px;
    --containerHeight: 250px;
  }
  body:has(input[value="250"]:checked) {
    --animElHeight: 250px;
    --animElHeightWord: "250px";
  }
  body:has(input[value="500"]:checked) {
    --animElHeight: 500px;
    --animElHeightWord: "500px";
  }
  main {
    padding: 20px 0 0 20px;
    margin-bottom: 2em;
  }
  article {
    outline: 3px dashed;
    width: 475px;
    margin: auto;
    overflow: scroll;
    position: relative;
    height: var(--containerHeight);
    box-sizing: content-box;
  }

  p {
    padding: 10px;
    margin: 10px;
  }

  section {
    --clr: yellow;
    --words: "Animation not started";
    position: relative;
    margin: 20px;
    text-align: center;
  }
  .one,
  .double i {
    animation: showAnim step-end 1 forwards;
    animation-timeline: view();
  }
  i,
  .animated_element {
    background-color: hsl(from var(--clr) h s calc(l * 1.4));
    display: block;
    height: var(--animElHeight);
    line-height: var(--animElHeight);
  }
  span {
    background-color: hsl(from var(--clr) h s 90%);
    border: 5px solid hsl(from var(--clr) h s 20%);
    min-width: 250px;
    height: 30px;
    line-height: 30px;
  }
  span,
  i {
    font-family: sans-serif;
    font-size: 1.5rem;
  }
  span::before {
    content: var(--words);
  }
  span {
    position: fixed;
    top: 10px;
    left: 10px;
    padding: 10px;
  }
  i::after {
    content: " ( " var(--animElHeightWord) " )";
  }
  label {
    padding-right: 2em;
  }
  legend {
    margin-top: 2em;
  }

  @keyframes showAnim {
    from {
      --clr: green;
      --words: "Currently animating";
    }
    to {
      --clr: red;
      --words: "Animation complete";
    }
  }
  body::before {
    display: block;
    text-align: center;
    font-family: sans-serif;
    font-size: 1.5rem;
  }

  @layer no-support {
    @supports not (animation-timeline: view()) {
      body::before {
        content: "Your browser doesn't support view progress scrolling.";
        background-color: wheat;
        display: block;
        text-align: center;
      }
    }
  }
}
```

```css hidden live-sample___initial live-sample___inset_percent live-sample___inset_length live-sample___inset_cover live-sample___inset_contain
.double {
  display: none;
}
```

```css hidden live-sample___cover_contain live-sample___exit_length_negative live-sample___entry_crossing live-sample___exit_crossing live-sample___entry_exit
.one {
  display: none;
}
.double div {
  display: flex;
  gap: 10px;
}
```

## See also

- {{cssxref("timeline-range-name")}} data type
- [Keyframe selectors](/en-US/docs/Web/CSS/Reference/Selectors/Keyframe_selectors)
- [Scroll-driven animation timelines](/en-US/docs/Web/CSS/Guides/Scroll-driven_animations/Timelines)
- [Scroll-driven animation](/en-US/docs/Web/CSS/Guides/Scroll-driven_animations) module
- [CSS animations](/en-US/docs/Web/CSS/Guides/Animations) module
- [Web Animations API](/en-US/docs/Web/API/Web_Animations_API)

`````
