# externals/prettier/markdown/blockquote/notext-end.md

## Option 2

`````json
{"printWidth":100,"proseWrap":"always"}
`````

### Diff

`````diff
===================================================================
--- prettier
+++ oxfmt
@@ -1,5 +1,6 @@
-> [!NOTE] `DOOM`
+> [!NOTE]
+> `DOOM`
 
 > _b_
 >
 > > `A` `B`

`````

### Actual (oxfmt)

`````md
> [!NOTE]
> `DOOM`

> _b_
>
> > `A` `B`

> _a_
>
> > # foo
> >
> > `a` > `b`

> This is a quote with an italic _across multuple lines which should just work_. So make sure there
> is no > if we set proseWrap to `never`

> This is a quote with a link [across multuple lines which should just work](<>). So make sure there
> is no > if we set proseWrap to `never`

`````

### Expected (prettier)

`````md
> [!NOTE] `DOOM`

> _b_
>
> > `A` `B`

> _a_
>
> > # foo
> >
> > `a` > `b`

> This is a quote with an italic _across multuple lines which should just work_. So make sure there
> is no > if we set proseWrap to `never`

> This is a quote with a link [across multuple lines which should just work](<>). So make sure there
> is no > if we set proseWrap to `never`

`````
