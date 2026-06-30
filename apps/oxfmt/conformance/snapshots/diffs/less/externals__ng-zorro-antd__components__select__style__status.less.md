# externals/ng-zorro-antd/components/select/style/status.less

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
@@ -10,10 +10,10 @@
   @hoverBorderColor;
   @outlineColor;
 ) {
   &.@{select-prefix-cls}:not(.@{select-prefix-cls}-disabled):not(
-      .@{select-prefix-cls}-customize-input
-    ):not(.@{pagination-prefix-cls}-size-changer) {
+    .@{select-prefix-cls}-customize-input
+  ):not(.@{pagination-prefix-cls}-size-changer) {
     .@{select-prefix-cls}-selector {
       background-color: @background-color;
       border-color: @border-color !important;
     }

`````

### Actual (oxfmt)

`````less
@import "../../input/style/mixin";

@select-prefix-cls: ~"@{ant-prefix}-select";
@pagination-prefix-cls: ~"@{ant-prefix}-pagination";

.select-status-color(
  @text-color;
  @border-color;
  @background-color;
  @hoverBorderColor;
  @outlineColor;
) {
  &.@{select-prefix-cls}:not(.@{select-prefix-cls}-disabled):not(
    .@{select-prefix-cls}-customize-input
  ):not(.@{pagination-prefix-cls}-size-changer) {
    .@{select-prefix-cls}-selector {
      background-color: @background-color;
      border-color: @border-color !important;
    }
    &.@{select-prefix-cls}-open .@{select-prefix-cls}-selector,
    &.@{select-prefix-cls}-focused .@{select-prefix-cls}-selector {
      .active(@border-color, @hoverBorderColor, @outlineColor);
    }
  }
}

.@{select-prefix-cls} {
  &-status-error {
    .select-status-color(@error-color, @error-color, @select-background, @error-color-hover, @error-color-outline);
  }

  &-status-warning {
    .select-status-color(@warning-color, @warning-color, @input-bg, @warning-color-hover, @warning-color-outline);
  }

  &-status-error,
  &-status-warning,
  &-status-success,
  &-status-validating {
    &.@{select-prefix-cls}-has-feedback {
      //.@{prefix-cls}-arrow,
      .@{select-prefix-cls}-clear {
        inset-inline-end: 32px;
      }

      .@{select-prefix-cls}-selection-selected-value {
        padding-inline-end: 42px;
      }
    }
  }
}

`````

### Expected (prettier)

`````less
@import "../../input/style/mixin";

@select-prefix-cls: ~"@{ant-prefix}-select";
@pagination-prefix-cls: ~"@{ant-prefix}-pagination";

.select-status-color(
  @text-color;
  @border-color;
  @background-color;
  @hoverBorderColor;
  @outlineColor;
) {
  &.@{select-prefix-cls}:not(.@{select-prefix-cls}-disabled):not(
      .@{select-prefix-cls}-customize-input
    ):not(.@{pagination-prefix-cls}-size-changer) {
    .@{select-prefix-cls}-selector {
      background-color: @background-color;
      border-color: @border-color !important;
    }
    &.@{select-prefix-cls}-open .@{select-prefix-cls}-selector,
    &.@{select-prefix-cls}-focused .@{select-prefix-cls}-selector {
      .active(@border-color, @hoverBorderColor, @outlineColor);
    }
  }
}

.@{select-prefix-cls} {
  &-status-error {
    .select-status-color(@error-color, @error-color, @select-background, @error-color-hover, @error-color-outline);
  }

  &-status-warning {
    .select-status-color(@warning-color, @warning-color, @input-bg, @warning-color-hover, @warning-color-outline);
  }

  &-status-error,
  &-status-warning,
  &-status-success,
  &-status-validating {
    &.@{select-prefix-cls}-has-feedback {
      //.@{prefix-cls}-arrow,
      .@{select-prefix-cls}-clear {
        inset-inline-end: 32px;
      }

      .@{select-prefix-cls}-selection-selected-value {
        padding-inline-end: 42px;
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
@@ -10,10 +10,10 @@
   @hoverBorderColor;
   @outlineColor;
 ) {
   &.@{select-prefix-cls}:not(.@{select-prefix-cls}-disabled):not(
-      .@{select-prefix-cls}-customize-input
-    ):not(.@{pagination-prefix-cls}-size-changer) {
+    .@{select-prefix-cls}-customize-input
+  ):not(.@{pagination-prefix-cls}-size-changer) {
     .@{select-prefix-cls}-selector {
       background-color: @background-color;
       border-color: @border-color !important;
     }

`````

### Actual (oxfmt)

`````less
@import "../../input/style/mixin";

@select-prefix-cls: ~"@{ant-prefix}-select";
@pagination-prefix-cls: ~"@{ant-prefix}-pagination";

.select-status-color(
  @text-color;
  @border-color;
  @background-color;
  @hoverBorderColor;
  @outlineColor;
) {
  &.@{select-prefix-cls}:not(.@{select-prefix-cls}-disabled):not(
    .@{select-prefix-cls}-customize-input
  ):not(.@{pagination-prefix-cls}-size-changer) {
    .@{select-prefix-cls}-selector {
      background-color: @background-color;
      border-color: @border-color !important;
    }
    &.@{select-prefix-cls}-open .@{select-prefix-cls}-selector,
    &.@{select-prefix-cls}-focused .@{select-prefix-cls}-selector {
      .active(@border-color, @hoverBorderColor, @outlineColor);
    }
  }
}

.@{select-prefix-cls} {
  &-status-error {
    .select-status-color(@error-color, @error-color, @select-background, @error-color-hover, @error-color-outline);
  }

  &-status-warning {
    .select-status-color(@warning-color, @warning-color, @input-bg, @warning-color-hover, @warning-color-outline);
  }

  &-status-error,
  &-status-warning,
  &-status-success,
  &-status-validating {
    &.@{select-prefix-cls}-has-feedback {
      //.@{prefix-cls}-arrow,
      .@{select-prefix-cls}-clear {
        inset-inline-end: 32px;
      }

      .@{select-prefix-cls}-selection-selected-value {
        padding-inline-end: 42px;
      }
    }
  }
}

`````

### Expected (prettier)

`````less
@import "../../input/style/mixin";

@select-prefix-cls: ~"@{ant-prefix}-select";
@pagination-prefix-cls: ~"@{ant-prefix}-pagination";

.select-status-color(
  @text-color;
  @border-color;
  @background-color;
  @hoverBorderColor;
  @outlineColor;
) {
  &.@{select-prefix-cls}:not(.@{select-prefix-cls}-disabled):not(
      .@{select-prefix-cls}-customize-input
    ):not(.@{pagination-prefix-cls}-size-changer) {
    .@{select-prefix-cls}-selector {
      background-color: @background-color;
      border-color: @border-color !important;
    }
    &.@{select-prefix-cls}-open .@{select-prefix-cls}-selector,
    &.@{select-prefix-cls}-focused .@{select-prefix-cls}-selector {
      .active(@border-color, @hoverBorderColor, @outlineColor);
    }
  }
}

.@{select-prefix-cls} {
  &-status-error {
    .select-status-color(@error-color, @error-color, @select-background, @error-color-hover, @error-color-outline);
  }

  &-status-warning {
    .select-status-color(@warning-color, @warning-color, @input-bg, @warning-color-hover, @warning-color-outline);
  }

  &-status-error,
  &-status-warning,
  &-status-success,
  &-status-validating {
    &.@{select-prefix-cls}-has-feedback {
      //.@{prefix-cls}-arrow,
      .@{select-prefix-cls}-clear {
        inset-inline-end: 32px;
      }

      .@{select-prefix-cls}-selection-selected-value {
        padding-inline-end: 42px;
      }
    }
  }
}

`````
