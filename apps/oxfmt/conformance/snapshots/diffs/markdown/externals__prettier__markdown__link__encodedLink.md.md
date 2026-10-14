# externals/prettier/markdown/link/encodedLink.md

## Option 1

`````json
{"printWidth":80}
`````

### Diff

`````diff
===================================================================
--- prettier
+++ oxfmt
@@ -1,11 +1,11 @@
-[link](<https://www.google.fr/()foo-%3Ebar>)
+[link](<https://www.google.fr/()foo-\>bar>)
 [link](https://www.google.fr/foo->bar)
 [link](https://www.google.fr/foo-%3Ebar)
 [link](https://www.google.fr/foo-<bar)
 [link](https://www.google.fr/foo-%3Cbar)
 
-![link](<https://www.google.fr/()foo-%3Ebar>)
+![link](<https://www.google.fr/()foo-\>bar>)
 ![link](https://www.google.fr/foo->bar)
 ![link](https://www.google.fr/foo-%3Ebar)
 ![link](https://www.google.fr/foo-<bar)
 ![link](https://www.google.fr/foo-%3Cbar)

`````

### Actual (oxfmt)

`````md
[link](<https://www.google.fr/()foo-\>bar>)
[link](https://www.google.fr/foo->bar)
[link](https://www.google.fr/foo-%3Ebar)
[link](https://www.google.fr/foo-<bar)
[link](https://www.google.fr/foo-%3Cbar)

![link](<https://www.google.fr/()foo-\>bar>)
![link](https://www.google.fr/foo->bar)
![link](https://www.google.fr/foo-%3Ebar)
![link](https://www.google.fr/foo-<bar)
![link](https://www.google.fr/foo-%3Cbar)

[link]: https://www.google.fr/()foo->bar
[link]: https://www.google.fr/foo->bar
[link]: https://www.google.fr/foo-%3Ebar
[link]: https://www.google.fr/foo-<bar
[link]: https://www.google.fr/foo-%3Cbar

`````

### Expected (prettier)

`````md
[link](<https://www.google.fr/()foo-%3Ebar>)
[link](https://www.google.fr/foo->bar)
[link](https://www.google.fr/foo-%3Ebar)
[link](https://www.google.fr/foo-<bar)
[link](https://www.google.fr/foo-%3Cbar)

![link](<https://www.google.fr/()foo-%3Ebar>)
![link](https://www.google.fr/foo->bar)
![link](https://www.google.fr/foo-%3Ebar)
![link](https://www.google.fr/foo-<bar)
![link](https://www.google.fr/foo-%3Cbar)

[link]: https://www.google.fr/()foo->bar
[link]: https://www.google.fr/foo->bar
[link]: https://www.google.fr/foo-%3Ebar
[link]: https://www.google.fr/foo-<bar
[link]: https://www.google.fr/foo-%3Cbar

`````

## Option 2

`````json
{"printWidth":100,"proseWrap":"always"}
`````

### Diff

`````diff
===================================================================
--- prettier
+++ oxfmt
@@ -1,9 +1,9 @@
-[link](<https://www.google.fr/()foo-%3Ebar>) [link](https://www.google.fr/foo->bar)
+[link](<https://www.google.fr/()foo-\>bar>) [link](https://www.google.fr/foo->bar)
 [link](https://www.google.fr/foo-%3Ebar) [link](https://www.google.fr/foo-<bar)
 [link](https://www.google.fr/foo-%3Cbar)
 
-![link](<https://www.google.fr/()foo-%3Ebar>) ![link](https://www.google.fr/foo->bar)
+![link](<https://www.google.fr/()foo-\>bar>) ![link](https://www.google.fr/foo->bar)
 ![link](https://www.google.fr/foo-%3Ebar) ![link](https://www.google.fr/foo-<bar)
 ![link](https://www.google.fr/foo-%3Cbar)
 
 [link]: https://www.google.fr/()foo->bar

`````

### Actual (oxfmt)

`````md
[link](<https://www.google.fr/()foo-\>bar>) [link](https://www.google.fr/foo->bar)
[link](https://www.google.fr/foo-%3Ebar) [link](https://www.google.fr/foo-<bar)
[link](https://www.google.fr/foo-%3Cbar)

![link](<https://www.google.fr/()foo-\>bar>) ![link](https://www.google.fr/foo->bar)
![link](https://www.google.fr/foo-%3Ebar) ![link](https://www.google.fr/foo-<bar)
![link](https://www.google.fr/foo-%3Cbar)

[link]: https://www.google.fr/()foo->bar
[link]: https://www.google.fr/foo->bar
[link]: https://www.google.fr/foo-%3Ebar
[link]: https://www.google.fr/foo-<bar
[link]: https://www.google.fr/foo-%3Cbar

`````

### Expected (prettier)

`````md
[link](<https://www.google.fr/()foo-%3Ebar>) [link](https://www.google.fr/foo->bar)
[link](https://www.google.fr/foo-%3Ebar) [link](https://www.google.fr/foo-<bar)
[link](https://www.google.fr/foo-%3Cbar)

![link](<https://www.google.fr/()foo-%3Ebar>) ![link](https://www.google.fr/foo->bar)
![link](https://www.google.fr/foo-%3Ebar) ![link](https://www.google.fr/foo-<bar)
![link](https://www.google.fr/foo-%3Cbar)

[link]: https://www.google.fr/()foo->bar
[link]: https://www.google.fr/foo->bar
[link]: https://www.google.fr/foo-%3Ebar
[link]: https://www.google.fr/foo-<bar
[link]: https://www.google.fr/foo-%3Cbar

`````
