# externals/prettier/js/multiparser-comments/comment-inside.js

## Option 1

`````json
{"printWidth":80}
`````

### Diff

`````diff
===================================================================
--- prettier
+++ oxfmt
@@ -13,11 +13,11 @@
   /* comment */
 }`;
 html`
   ${
-  foo
-  /* comment */
-}
+    foo
+    /* comment */
+  }
 `;
 
 graphql`
   ${
@@ -26,11 +26,11 @@
   }
 `;
 graphql`
   ${
-  foo
-  /* comment */
-}
+    foo
+    /* comment */
+  }
 `;
 
 css`
   ${
@@ -39,11 +39,11 @@
   }
 `;
 css`
   ${
-  foo
-  /* comment */
-}
+    foo
+    /* comment */
+  }
 `;
 
 markdown`${
   foo

`````

### Actual (oxfmt)

`````js
// #9274
html`
  <div>
    ${
      this.set && this.set.artist
      /* avoid console errors if `this.set` is undefined */
    }
  </div>
`;

html`${
  foo
  /* comment */
}`;
html`
  ${
    foo
    /* comment */
  }
`;

graphql`
  ${
    foo
    /* comment */
  }
`;
graphql`
  ${
    foo
    /* comment */
  }
`;

css`
  ${
    foo
    /* comment */
  }
`;
css`
  ${
    foo
    /* comment */
  }
`;

markdown`${
  foo
  /* comment */
}`;
markdown`
${
  foo
  /* comment */
}
`;

// https://github.com/prettier/prettier/pull/9278#issuecomment-700589195
expr1 = html`
  <div>
    ${x(
      foo, // fg
      bar,
    )}
  </div>
`;

`````

### Expected (prettier)

`````js
// #9274
html`
  <div>
    ${
      this.set && this.set.artist
      /* avoid console errors if `this.set` is undefined */
    }
  </div>
`;

html`${
  foo
  /* comment */
}`;
html`
  ${
  foo
  /* comment */
}
`;

graphql`
  ${
    foo
    /* comment */
  }
`;
graphql`
  ${
  foo
  /* comment */
}
`;

css`
  ${
    foo
    /* comment */
  }
`;
css`
  ${
  foo
  /* comment */
}
`;

markdown`${
  foo
  /* comment */
}`;
markdown`
${
  foo
  /* comment */
}
`;

// https://github.com/prettier/prettier/pull/9278#issuecomment-700589195
expr1 = html`
  <div>
    ${x(
      foo, // fg
      bar,
    )}
  </div>
`;

`````

## Option 2

`````json
{"printWith":100}
`````

### Diff

`````diff
===================================================================
--- prettier
+++ oxfmt
@@ -13,11 +13,11 @@
   /* comment */
 }`;
 html`
   ${
-  foo
-  /* comment */
-}
+    foo
+    /* comment */
+  }
 `;
 
 graphql`
   ${
@@ -26,11 +26,11 @@
   }
 `;
 graphql`
   ${
-  foo
-  /* comment */
-}
+    foo
+    /* comment */
+  }
 `;
 
 css`
   ${
@@ -39,11 +39,11 @@
   }
 `;
 css`
   ${
-  foo
-  /* comment */
-}
+    foo
+    /* comment */
+  }
 `;
 
 markdown`${
   foo

`````

### Actual (oxfmt)

`````js
// #9274
html`
  <div>
    ${
      this.set && this.set.artist
      /* avoid console errors if `this.set` is undefined */
    }
  </div>
`;

html`${
  foo
  /* comment */
}`;
html`
  ${
    foo
    /* comment */
  }
`;

graphql`
  ${
    foo
    /* comment */
  }
`;
graphql`
  ${
    foo
    /* comment */
  }
`;

css`
  ${
    foo
    /* comment */
  }
`;
css`
  ${
    foo
    /* comment */
  }
`;

markdown`${
  foo
  /* comment */
}`;
markdown`
${
  foo
  /* comment */
}
`;

// https://github.com/prettier/prettier/pull/9278#issuecomment-700589195
expr1 = html`
  <div>
    ${x(
      foo, // fg
      bar,
    )}
  </div>
`;

`````

### Expected (prettier)

`````js
// #9274
html`
  <div>
    ${
      this.set && this.set.artist
      /* avoid console errors if `this.set` is undefined */
    }
  </div>
`;

html`${
  foo
  /* comment */
}`;
html`
  ${
  foo
  /* comment */
}
`;

graphql`
  ${
    foo
    /* comment */
  }
`;
graphql`
  ${
  foo
  /* comment */
}
`;

css`
  ${
    foo
    /* comment */
  }
`;
css`
  ${
  foo
  /* comment */
}
`;

markdown`${
  foo
  /* comment */
}`;
markdown`
${
  foo
  /* comment */
}
`;

// https://github.com/prettier/prettier/pull/9278#issuecomment-700589195
expr1 = html`
  <div>
    ${x(
      foo, // fg
      bar,
    )}
  </div>
`;

`````
