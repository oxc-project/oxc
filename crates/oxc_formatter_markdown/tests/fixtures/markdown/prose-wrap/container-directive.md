<!-- `:::` fence lines are kept as written; only the children are formatted.
     The opener takes any name-like line: remark `:::name[label]{attrs}`, VitePress `::: tip Title`, Pandoc `::: {.class}`.
     Nesting goes outer-longer, as in remark-directive: a fence of 3+ `:` alone closes the outermost open directive it fits -->

::::warning
Some **text** that is
long enough to wrap when the width is small and prose wrap is always on.

:::nested
inner
:::
::::

::: tip Custom Title
Some **text** that is
long enough to wrap when the width is small and prose wrap is always on.
:::

::: {.class}
Pandoc fenced div.
:::
