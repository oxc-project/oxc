# externals/ng-zorro-antd/components/radio/style/rtl.less

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
@@ -40,10 +40,10 @@
       border-right: @border-width-base @border-style-base @border-color-base;
       border-radius: 0 @border-radius-base @border-radius-base 0;
     }
     .@{radio-prefix-cls-button-wrapper}-checked:not(
-        [class*=~"' @{radio-prefix-cls}-button-wrapper-disabled'"]
-      )& {
+      [class*=~"' @{radio-prefix-cls}-button-wrapper-disabled'"]
+    )& {
       border-right-color: @radio-button-hover-color;
     }
   }
 

`````

### Actual (oxfmt)

`````less
@import "../../style/themes/index";
@import "../../style/mixins/index";

@radio-prefix-cls: ~"@{ant-prefix}-radio";
@radio-group-prefix-cls: ~"@{radio-prefix-cls}-group";
@radio-prefix-cls-button-wrapper: ~"@{radio-prefix-cls}-button-wrapper";

.@{radio-group-prefix-cls} {
  &&-rtl {
    direction: rtl;
  }
}

// 一般状态
.@{radio-prefix-cls}-wrapper {
  &&-rtl {
    margin-right: 0;
    margin-left: @radio-wrapper-margin-right;
    direction: rtl;
  }
}

.@{radio-prefix-cls-button-wrapper} {
  &&-rtl {
    border-right-width: 0;
    border-left-width: @border-width-base;
  }

  &:not(:first-child) {
    &::before {
      .@{radio-prefix-cls-button-wrapper}.@{radio-prefix-cls-button-wrapper}-rtl& {
        right: -1px;
        left: 0;
      }
    }
  }

  &:first-child {
    .@{radio-prefix-cls-button-wrapper}.@{radio-prefix-cls-button-wrapper}-rtl& {
      border-right: @border-width-base @border-style-base @border-color-base;
      border-radius: 0 @border-radius-base @border-radius-base 0;
    }
    .@{radio-prefix-cls-button-wrapper}-checked:not(
      [class*=~"' @{radio-prefix-cls}-button-wrapper-disabled'"]
    )& {
      border-right-color: @radio-button-hover-color;
    }
  }

  &:last-child {
    .@{radio-prefix-cls-button-wrapper}.@{radio-prefix-cls-button-wrapper}-rtl& {
      border-radius: @border-radius-base 0 0 @border-radius-base;
    }
  }

  &-disabled {
    &:first-child {
      .@{radio-prefix-cls-button-wrapper}.@{radio-prefix-cls-button-wrapper}-rtl& {
        border-right-color: @border-color-base;
      }
    }
  }
}

`````

### Expected (prettier)

`````less
@import "../../style/themes/index";
@import "../../style/mixins/index";

@radio-prefix-cls: ~"@{ant-prefix}-radio";
@radio-group-prefix-cls: ~"@{radio-prefix-cls}-group";
@radio-prefix-cls-button-wrapper: ~"@{radio-prefix-cls}-button-wrapper";

.@{radio-group-prefix-cls} {
  &&-rtl {
    direction: rtl;
  }
}

// 一般状态
.@{radio-prefix-cls}-wrapper {
  &&-rtl {
    margin-right: 0;
    margin-left: @radio-wrapper-margin-right;
    direction: rtl;
  }
}

.@{radio-prefix-cls-button-wrapper} {
  &&-rtl {
    border-right-width: 0;
    border-left-width: @border-width-base;
  }

  &:not(:first-child) {
    &::before {
      .@{radio-prefix-cls-button-wrapper}.@{radio-prefix-cls-button-wrapper}-rtl& {
        right: -1px;
        left: 0;
      }
    }
  }

  &:first-child {
    .@{radio-prefix-cls-button-wrapper}.@{radio-prefix-cls-button-wrapper}-rtl& {
      border-right: @border-width-base @border-style-base @border-color-base;
      border-radius: 0 @border-radius-base @border-radius-base 0;
    }
    .@{radio-prefix-cls-button-wrapper}-checked:not(
        [class*=~"' @{radio-prefix-cls}-button-wrapper-disabled'"]
      )& {
      border-right-color: @radio-button-hover-color;
    }
  }

  &:last-child {
    .@{radio-prefix-cls-button-wrapper}.@{radio-prefix-cls-button-wrapper}-rtl& {
      border-radius: @border-radius-base 0 0 @border-radius-base;
    }
  }

  &-disabled {
    &:first-child {
      .@{radio-prefix-cls-button-wrapper}.@{radio-prefix-cls-button-wrapper}-rtl& {
        border-right-color: @border-color-base;
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
@@ -40,10 +40,10 @@
       border-right: @border-width-base @border-style-base @border-color-base;
       border-radius: 0 @border-radius-base @border-radius-base 0;
     }
     .@{radio-prefix-cls-button-wrapper}-checked:not(
-        [class*=~"' @{radio-prefix-cls}-button-wrapper-disabled'"]
-      )& {
+      [class*=~"' @{radio-prefix-cls}-button-wrapper-disabled'"]
+    )& {
       border-right-color: @radio-button-hover-color;
     }
   }
 

`````

### Actual (oxfmt)

`````less
@import "../../style/themes/index";
@import "../../style/mixins/index";

@radio-prefix-cls: ~"@{ant-prefix}-radio";
@radio-group-prefix-cls: ~"@{radio-prefix-cls}-group";
@radio-prefix-cls-button-wrapper: ~"@{radio-prefix-cls}-button-wrapper";

.@{radio-group-prefix-cls} {
  &&-rtl {
    direction: rtl;
  }
}

// 一般状态
.@{radio-prefix-cls}-wrapper {
  &&-rtl {
    margin-right: 0;
    margin-left: @radio-wrapper-margin-right;
    direction: rtl;
  }
}

.@{radio-prefix-cls-button-wrapper} {
  &&-rtl {
    border-right-width: 0;
    border-left-width: @border-width-base;
  }

  &:not(:first-child) {
    &::before {
      .@{radio-prefix-cls-button-wrapper}.@{radio-prefix-cls-button-wrapper}-rtl& {
        right: -1px;
        left: 0;
      }
    }
  }

  &:first-child {
    .@{radio-prefix-cls-button-wrapper}.@{radio-prefix-cls-button-wrapper}-rtl& {
      border-right: @border-width-base @border-style-base @border-color-base;
      border-radius: 0 @border-radius-base @border-radius-base 0;
    }
    .@{radio-prefix-cls-button-wrapper}-checked:not(
      [class*=~"' @{radio-prefix-cls}-button-wrapper-disabled'"]
    )& {
      border-right-color: @radio-button-hover-color;
    }
  }

  &:last-child {
    .@{radio-prefix-cls-button-wrapper}.@{radio-prefix-cls-button-wrapper}-rtl& {
      border-radius: @border-radius-base 0 0 @border-radius-base;
    }
  }

  &-disabled {
    &:first-child {
      .@{radio-prefix-cls-button-wrapper}.@{radio-prefix-cls-button-wrapper}-rtl& {
        border-right-color: @border-color-base;
      }
    }
  }
}

`````

### Expected (prettier)

`````less
@import "../../style/themes/index";
@import "../../style/mixins/index";

@radio-prefix-cls: ~"@{ant-prefix}-radio";
@radio-group-prefix-cls: ~"@{radio-prefix-cls}-group";
@radio-prefix-cls-button-wrapper: ~"@{radio-prefix-cls}-button-wrapper";

.@{radio-group-prefix-cls} {
  &&-rtl {
    direction: rtl;
  }
}

// 一般状态
.@{radio-prefix-cls}-wrapper {
  &&-rtl {
    margin-right: 0;
    margin-left: @radio-wrapper-margin-right;
    direction: rtl;
  }
}

.@{radio-prefix-cls-button-wrapper} {
  &&-rtl {
    border-right-width: 0;
    border-left-width: @border-width-base;
  }

  &:not(:first-child) {
    &::before {
      .@{radio-prefix-cls-button-wrapper}.@{radio-prefix-cls-button-wrapper}-rtl& {
        right: -1px;
        left: 0;
      }
    }
  }

  &:first-child {
    .@{radio-prefix-cls-button-wrapper}.@{radio-prefix-cls-button-wrapper}-rtl& {
      border-right: @border-width-base @border-style-base @border-color-base;
      border-radius: 0 @border-radius-base @border-radius-base 0;
    }
    .@{radio-prefix-cls-button-wrapper}-checked:not(
        [class*=~"' @{radio-prefix-cls}-button-wrapper-disabled'"]
      )& {
      border-right-color: @radio-button-hover-color;
    }
  }

  &:last-child {
    .@{radio-prefix-cls-button-wrapper}.@{radio-prefix-cls-button-wrapper}-rtl& {
      border-radius: @border-radius-base 0 0 @border-radius-base;
    }
  }

  &-disabled {
    &:first-child {
      .@{radio-prefix-cls-button-wrapper}.@{radio-prefix-cls-button-wrapper}-rtl& {
        border-right-color: @border-color-base;
      }
    }
  }
}

`````
