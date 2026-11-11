# Input

`````vue
<template>
  <button
    @click="count++; if (a) { b(); c() }"
    @input="handler;"
    @change="if (x) y();"
    @blur="a = 1; [b, c] = [c, b]"
    @focus="; /* c */"
    @keyup="function () { return 1 }"
    @dblclick="(function () { a(); b() }).call(this)"
    @mousedown="({ a() { b(); c() } }).a()"
    @mouseup="(class { x = 1 }).name"
    @mouseenter="({ a } = { b() { c(); d() } })"
    @mouseleave="(let)[0] = { a() { b(); c() } }"
    @select="#!a
b(); c()"
    @dragstart="a();"
    @dragend="if (a) b()"
    @drop="function () { a(); }"
    @contextmenu="a() /* c */; b()"
  ></button>
</template>

`````

# Output

## `{"printWidth":80}`

`````vue
<template>
  <button
    @click="
      count++;
      if (a) {
        b();
        c();
      }
    "
    @input="handler;"
    @change="if (x) y();"
    @blur="
      a = 1;
      [b, c] = [c, b];
    "
    @focus="; /* c */"
    @keyup="
      (function () {
        return 1;
      })
    "
    @dblclick="
      (function () {
        a();
        b();
      }).call(this)
    "
    @mousedown="
      ({
        a() {
          b();
          c();
        },
      }).a()
    "
    @mouseup="
      (class {
        x = 1;
      }).name
    "
    @mouseenter="
      ({ a } = {
        b() {
          c();
          d();
        },
      })
    "
    @mouseleave="
      (let)[0] = {
        a() {
          b();
          c();
        },
      }
    "
    @select="
      #!a
      b();
      c();
    "
    @dragstart="a()"
    @dragend="if (a) b()"
    @drop="function () { a(); }"
    @contextmenu="
      a(); /* c */
      b();
    "
  ></button>
</template>

`````

## `{"printWidth":100,"vueIndentScriptAndStyle":true,"singleQuote":true,"semi":false}`

`````vue
<template>
  <button
    @click="
      count++;
      if (a) {
        b()
        c()
      }
    "
    @input="handler;"
    @change="if (x) y();"
    @blur="
      a = 1;
      [b, c] = [c, b]
    "
    @focus="; /* c */"
    @keyup="
      (function () {
        return 1
      })
    "
    @dblclick="
      (function () {
        a()
        b()
      }).call(this)
    "
    @mousedown="
      ({
        a() {
          b()
          c()
        },
      }).a()
    "
    @mouseup="
      (class {
        x = 1
      }).name
    "
    @mouseenter="
      ({ a } = {
        b() {
          c()
          d()
        },
      })
    "
    @mouseleave="
      (let)[0] = {
        a() {
          b()
          c()
        },
      }
    "
    @select="
      #!a
      b();
      c()
    "
    @dragstart="a()"
    @dragend="if (a) b()"
    @drop="function () { a(); }"
    @contextmenu="
      a(); /* c */
      b()
    "
  ></button>
</template>

`````
