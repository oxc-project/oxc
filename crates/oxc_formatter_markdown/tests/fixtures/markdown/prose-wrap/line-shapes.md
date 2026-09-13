<!-- Dialect line shapes keep their line boundaries and are never re-wrapped:
     `<<<` snippet imports, component tags, `[!TYPE]` alert markers, stray `:::` lines -->

Intro text before a snippet import that is long enough to wrap when prose wrap is always.
<<< @/snippets/snippet-with-region.js#snippet{1,2 ts:line-numbers} [snippet with region]
<<< @/snippets/other.js
Text after the imports that is long enough to wrap when prose wrap is always on.

> [!TIP] Custom title
> Body of the alert, long enough to wrap when prose wrap is always and the width is small.

> [!NOTE]
> `DOOM`

Click <Badge type="tip" text="new" /> to see it, a sentence long enough to wrap around the tag.
<Badge type="tip" text="new" />
<my-element some-attribute="value">inline</my-element>
<span>plain html</span> keeps wrapping with the prose around it as usual.
:::)
plain text after an emoticon line that is long enough to wrap when prose wrap is always on.
