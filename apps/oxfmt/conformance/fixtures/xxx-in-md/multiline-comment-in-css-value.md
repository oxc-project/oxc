A multi-line comment in a broken value breaks before it, as in its standalone file

```css
article {
  background-image: linear-gradient(
    to top,
    transparent 59.5px,
    red 74.5px 75.5px,
    transparent 75.5px /* 174.5px,
    green 174.5px 154.5px,
    transparent 175.5px*/
  );
}
```
