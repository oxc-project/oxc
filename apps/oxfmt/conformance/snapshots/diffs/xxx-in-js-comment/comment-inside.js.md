# comment-inside.js

> html embed expressions not yet implemented

## Option 1

`````json
{"printWidth":80}
`````

### Diff

`````diff
===================================================================
--- prettier
+++ oxfmt
@@ -12,12 +12,12 @@
   foo
   /* comment */
 }`;
 html`
-  ${
-    foo
-    /* comment */
-  }
+${
+  foo
+  /* comment */
+}
 `;
 
 graphql`
   ${
@@ -61,7 +61,6 @@
   <div>
     ${x(
       foo, // fg
       bar,
-    )}
-  </div>
+    )}</div>
 `;

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
    )}</div>
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
