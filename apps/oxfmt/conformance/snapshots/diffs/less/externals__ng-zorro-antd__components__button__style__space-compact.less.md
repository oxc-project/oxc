# externals/ng-zorro-antd/components/button/style/space-compact.less

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
@@ -32,32 +32,32 @@
 
   // ----------RTL----------
   &-compact-item-rtl {
     &.@{btn-prefix-cls}-compact-first-item&:not(
-        .@{btn-prefix-cls}-compact-last-item
-      ) {
+      .@{btn-prefix-cls}-compact-last-item
+    ) {
       border-top-left-radius: 0;
       border-bottom-left-radius: 0;
     }
 
     &.@{btn-prefix-cls}-compact-last-item&:not(
-        .@{btn-prefix-cls}-compact-first-item
-      ) {
+      .@{btn-prefix-cls}-compact-first-item
+    ) {
       border-top-right-radius: 0;
       border-bottom-right-radius: 0;
     }
 
     &.@{btn-prefix-cls}-sm {
       &.@{btn-prefix-cls}-compact-first-item&:not(
-          .@{btn-prefix-cls}-compact-last-item
-        ) {
+        .@{btn-prefix-cls}-compact-last-item
+      ) {
         border-top-left-radius: 0;
         border-bottom-left-radius: 0;
       }
 
       &.@{btn-prefix-cls}-compact-last-item&:not(
-          .@{btn-prefix-cls}-compact-first-item
-        ) {
+        .@{btn-prefix-cls}-compact-first-item
+      ) {
         border-top-right-radius: 0;
         border-bottom-right-radius: 0;
       }
     }

`````

### Actual (oxfmt)

`````less
@import "../../style/mixins/index";

@btn-prefix-cls: ~"@{ant-prefix}-btn";

// Button in Space.Compact
.@{btn-prefix-cls} {
  .compact-item(@btn-prefix-cls);

  // make `btn-icon-only` not too narrow
  &-icon-only&-compact-item {
    flex: none;
  }

  // Special styles for Primary Button
  &-compact-item.@{btn-prefix-cls}-primary {
    &:not([disabled])
      + &:not([disabled]):not([ant-click-animating-without-extra-node="true"]) {
      position: relative;

      &::after {
        position: absolute;
        top: -@border-width-base;
        left: -@border-width-base;
        display: inline-block;
        width: @border-width-base;
        height: calc(100% + @border-width-base * 2);
        background-color: @btn-group-border;
        content: " ";
      }
    }
  }

  // ----------RTL----------
  &-compact-item-rtl {
    &.@{btn-prefix-cls}-compact-first-item&:not(
      .@{btn-prefix-cls}-compact-last-item
    ) {
      border-top-left-radius: 0;
      border-bottom-left-radius: 0;
    }

    &.@{btn-prefix-cls}-compact-last-item&:not(
      .@{btn-prefix-cls}-compact-first-item
    ) {
      border-top-right-radius: 0;
      border-bottom-right-radius: 0;
    }

    &.@{btn-prefix-cls}-sm {
      &.@{btn-prefix-cls}-compact-first-item&:not(
        .@{btn-prefix-cls}-compact-last-item
      ) {
        border-top-left-radius: 0;
        border-bottom-left-radius: 0;
      }

      &.@{btn-prefix-cls}-compact-last-item&:not(
        .@{btn-prefix-cls}-compact-first-item
      ) {
        border-top-right-radius: 0;
        border-bottom-right-radius: 0;
      }
    }

    // ----------RTL Special styles for Primary Button----------
    &.@{btn-prefix-cls}-primary {
      &:not([disabled]) + &:not([disabled]) {
        &::after {
          right: -@border-width-base;
        }
      }
    }
  }

  // Button in Space.Compact when direction=vertical
  .compact-item-vertical(@btn-prefix-cls);

  // Special styles for Primary Button
  &-compact-vertical-item {
    &.@{btn-prefix-cls}-primary {
      &:not([disabled])
        + &:not([disabled]):not(
          [ant-click-animating-without-extra-node="true"]
        ) {
        position: relative;

        &::after {
          position: absolute;
          top: -@border-width-base;
          left: -@border-width-base;
          display: inline-block;
          width: calc(100% + @border-width-base * 2);
          height: @border-width-base;
          background-color: @btn-group-border;
          content: " ";
        }
      }
    }
  }
}

`````

### Expected (prettier)

`````less
@import "../../style/mixins/index";

@btn-prefix-cls: ~"@{ant-prefix}-btn";

// Button in Space.Compact
.@{btn-prefix-cls} {
  .compact-item(@btn-prefix-cls);

  // make `btn-icon-only` not too narrow
  &-icon-only&-compact-item {
    flex: none;
  }

  // Special styles for Primary Button
  &-compact-item.@{btn-prefix-cls}-primary {
    &:not([disabled])
      + &:not([disabled]):not([ant-click-animating-without-extra-node="true"]) {
      position: relative;

      &::after {
        position: absolute;
        top: -@border-width-base;
        left: -@border-width-base;
        display: inline-block;
        width: @border-width-base;
        height: calc(100% + @border-width-base * 2);
        background-color: @btn-group-border;
        content: " ";
      }
    }
  }

  // ----------RTL----------
  &-compact-item-rtl {
    &.@{btn-prefix-cls}-compact-first-item&:not(
        .@{btn-prefix-cls}-compact-last-item
      ) {
      border-top-left-radius: 0;
      border-bottom-left-radius: 0;
    }

    &.@{btn-prefix-cls}-compact-last-item&:not(
        .@{btn-prefix-cls}-compact-first-item
      ) {
      border-top-right-radius: 0;
      border-bottom-right-radius: 0;
    }

    &.@{btn-prefix-cls}-sm {
      &.@{btn-prefix-cls}-compact-first-item&:not(
          .@{btn-prefix-cls}-compact-last-item
        ) {
        border-top-left-radius: 0;
        border-bottom-left-radius: 0;
      }

      &.@{btn-prefix-cls}-compact-last-item&:not(
          .@{btn-prefix-cls}-compact-first-item
        ) {
        border-top-right-radius: 0;
        border-bottom-right-radius: 0;
      }
    }

    // ----------RTL Special styles for Primary Button----------
    &.@{btn-prefix-cls}-primary {
      &:not([disabled]) + &:not([disabled]) {
        &::after {
          right: -@border-width-base;
        }
      }
    }
  }

  // Button in Space.Compact when direction=vertical
  .compact-item-vertical(@btn-prefix-cls);

  // Special styles for Primary Button
  &-compact-vertical-item {
    &.@{btn-prefix-cls}-primary {
      &:not([disabled])
        + &:not([disabled]):not(
          [ant-click-animating-without-extra-node="true"]
        ) {
        position: relative;

        &::after {
          position: absolute;
          top: -@border-width-base;
          left: -@border-width-base;
          display: inline-block;
          width: calc(100% + @border-width-base * 2);
          height: @border-width-base;
          background-color: @btn-group-border;
          content: " ";
        }
      }
    }
  }
}

`````
