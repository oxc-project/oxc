# externals/prettier/markdown/blockquote/ignore-code.md

## Option 2

`````json
{"printWidth":100,"proseWrap":"always"}
`````

### Diff

`````diff
===================================================================
--- prettier
+++ oxfmt
@@ -41,9 +41,8 @@
 > <!-- prettier-ignore -->
 > - This is a long long
 >   long long long long
 >   long long paragraph.
->
 
 > ```js
 > // prettier-ignore
 > const x = 1,

`````

### Actual (oxfmt)

`````md
> ````md
> <!-- prettier-ignore -->
> ```js
> ugly   ( code ) ;
> ```
> ````

> ```md
> <!-- prettier-ignore -->
> - This is a long long
>   long long long long
>   long long paragraph.
> ```

> - test
>   ```md
>   <!-- prettier-ignore -->
>   - This is a long long
>     long long long long
>     long long paragraph.
>   ```

````md
> ```md
> <!-- prettier-ignore -->
> - This is a long long
>   long long long long
>   long long paragraph.
> ```
````

> ````md
> > ```md
> > <!-- prettier-ignore -->
> > - This is a long long
> >   long long long long
> >   long long paragraph.
> > ```
> ````

> <!-- prettier-ignore -->
> - This is a long long
>   long long long long
>   long long paragraph.

> ```js
> // prettier-ignore
> const x = 1,
> b = 2;
> ```

`````

### Expected (prettier)

`````md
> ````md
> <!-- prettier-ignore -->
> ```js
> ugly   ( code ) ;
> ```
> ````

> ```md
> <!-- prettier-ignore -->
> - This is a long long
>   long long long long
>   long long paragraph.
> ```

> - test
>   ```md
>   <!-- prettier-ignore -->
>   - This is a long long
>     long long long long
>     long long paragraph.
>   ```

````md
> ```md
> <!-- prettier-ignore -->
> - This is a long long
>   long long long long
>   long long paragraph.
> ```
````

> ````md
> > ```md
> > <!-- prettier-ignore -->
> > - This is a long long
> >   long long long long
> >   long long paragraph.
> > ```
> ````

> <!-- prettier-ignore -->
> - This is a long long
>   long long long long
>   long long paragraph.
>

> ```js
> // prettier-ignore
> const x = 1,
> b = 2;
> ```

`````
