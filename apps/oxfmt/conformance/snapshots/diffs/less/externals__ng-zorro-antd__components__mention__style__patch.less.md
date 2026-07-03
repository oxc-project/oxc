# externals/ng-zorro-antd/components/mention/style/patch.less

> Allowed (layout-only): wrapped :not() selector-arg indent (prettier/prettier#16165)

## Option 1

`````json
{"printWidth":80}
`````

### Diff

`````diff
===================================================================
--- prettier
+++ oxfmt
@@ -13,20 +13,20 @@
   }
 
   &&-status-error {
     &:not(.@{mention-prefix-cls}-disabled):not(
-        .@{mention-prefix-cls}-borderless
-      ).@{mention-prefix-cls} {
+      .@{mention-prefix-cls}-borderless
+    ).@{mention-prefix-cls} {
       &:focus-within {
         .active(@error-color, @error-color-hover, @error-color-outline);
       }
     }
   }
 
   &&-status-warning {
     &:not(.@{mention-prefix-cls}-disabled):not(
-        .@{mention-prefix-cls}-borderless
-      ).@{mention-prefix-cls} {
+      .@{mention-prefix-cls}-borderless
+    ).@{mention-prefix-cls} {
       &:focus-within {
         .active(@warning-color, @warning-color-hover, @warning-color-outline);
       }
     }

`````

### Actual (oxfmt)

`````less
.@{mention-prefix-cls} {
  &-dropdown {
    position: relative;
    top: 0;
    left: 12px;
    width: 100%;
    margin-top: 8px;
    margin-bottom: 4px;
  }

  &:focus-within {
    .active();
  }

  &&-status-error {
    &:not(.@{mention-prefix-cls}-disabled):not(
      .@{mention-prefix-cls}-borderless
    ).@{mention-prefix-cls} {
      &:focus-within {
        .active(@error-color, @error-color-hover, @error-color-outline);
      }
    }
  }

  &&-status-warning {
    &:not(.@{mention-prefix-cls}-disabled):not(
      .@{mention-prefix-cls}-borderless
    ).@{mention-prefix-cls} {
      &:focus-within {
        .active(@warning-color, @warning-color-hover, @warning-color-outline);
      }
    }
  }
}

`````

### Expected (prettier)

`````less
.@{mention-prefix-cls} {
  &-dropdown {
    position: relative;
    top: 0;
    left: 12px;
    width: 100%;
    margin-top: 8px;
    margin-bottom: 4px;
  }

  &:focus-within {
    .active();
  }

  &&-status-error {
    &:not(.@{mention-prefix-cls}-disabled):not(
        .@{mention-prefix-cls}-borderless
      ).@{mention-prefix-cls} {
      &:focus-within {
        .active(@error-color, @error-color-hover, @error-color-outline);
      }
    }
  }

  &&-status-warning {
    &:not(.@{mention-prefix-cls}-disabled):not(
        .@{mention-prefix-cls}-borderless
      ).@{mention-prefix-cls} {
      &:focus-within {
        .active(@warning-color, @warning-color-hover, @warning-color-outline);
      }
    }
  }
}

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
@@ -13,20 +13,20 @@
   }
 
   &&-status-error {
     &:not(.@{mention-prefix-cls}-disabled):not(
-        .@{mention-prefix-cls}-borderless
-      ).@{mention-prefix-cls} {
+      .@{mention-prefix-cls}-borderless
+    ).@{mention-prefix-cls} {
       &:focus-within {
         .active(@error-color, @error-color-hover, @error-color-outline);
       }
     }
   }
 
   &&-status-warning {
     &:not(.@{mention-prefix-cls}-disabled):not(
-        .@{mention-prefix-cls}-borderless
-      ).@{mention-prefix-cls} {
+      .@{mention-prefix-cls}-borderless
+    ).@{mention-prefix-cls} {
       &:focus-within {
         .active(@warning-color, @warning-color-hover, @warning-color-outline);
       }
     }

`````

### Actual (oxfmt)

`````less
.@{mention-prefix-cls} {
  &-dropdown {
    position: relative;
    top: 0;
    left: 12px;
    width: 100%;
    margin-top: 8px;
    margin-bottom: 4px;
  }

  &:focus-within {
    .active();
  }

  &&-status-error {
    &:not(.@{mention-prefix-cls}-disabled):not(
      .@{mention-prefix-cls}-borderless
    ).@{mention-prefix-cls} {
      &:focus-within {
        .active(@error-color, @error-color-hover, @error-color-outline);
      }
    }
  }

  &&-status-warning {
    &:not(.@{mention-prefix-cls}-disabled):not(
      .@{mention-prefix-cls}-borderless
    ).@{mention-prefix-cls} {
      &:focus-within {
        .active(@warning-color, @warning-color-hover, @warning-color-outline);
      }
    }
  }
}

`````

### Expected (prettier)

`````less
.@{mention-prefix-cls} {
  &-dropdown {
    position: relative;
    top: 0;
    left: 12px;
    width: 100%;
    margin-top: 8px;
    margin-bottom: 4px;
  }

  &:focus-within {
    .active();
  }

  &&-status-error {
    &:not(.@{mention-prefix-cls}-disabled):not(
        .@{mention-prefix-cls}-borderless
      ).@{mention-prefix-cls} {
      &:focus-within {
        .active(@error-color, @error-color-hover, @error-color-outline);
      }
    }
  }

  &&-status-warning {
    &:not(.@{mention-prefix-cls}-disabled):not(
        .@{mention-prefix-cls}-borderless
      ).@{mention-prefix-cls} {
      &:focus-within {
        .active(@warning-color, @warning-color-hover, @warning-color-outline);
      }
    }
  }
}

`````
