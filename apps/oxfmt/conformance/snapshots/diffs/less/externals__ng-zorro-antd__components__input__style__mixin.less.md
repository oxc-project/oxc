# externals/ng-zorro-antd/components/input/style/mixin.less

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
@@ -466,10 +466,10 @@
 .input-group-wrapper-compact(@inputClass) {
   // Fix the issue of using icons in Space Compact mode
   // https://github.com/ant-design/ant-design/issues/42122
   &:not(.@{inputClass}-compact-first-item):not(
-      .@{inputClass}-compact-last-item
-    ).@{inputClass}-compact-item {
+    .@{inputClass}-compact-last-item
+  ).@{inputClass}-compact-item {
     .@{inputClass},
     .@{inputClass}-group-addon {
       border-radius: 0;
     }

`````

### Actual (oxfmt)

`````less
@import "../../style/themes/index";
@import "../../style/mixins/index";

@input-affix-with-clear-btn-width: 38px;

// size mixins for input
.input-lg() {
  padding: @input-padding-vertical-lg @input-padding-horizontal-lg;
  font-size: @font-size-lg;
}

.input-sm() {
  padding: @input-padding-vertical-sm @input-padding-horizontal-sm;
}

// input status
// == when focus or active
.active(@borderColor: @primary-color; @hoverBorderColor: @primary-color-hover; @outlineColor: @primary-color-outline) {
  & when (@theme = dark) {
    border-color: @borderColor;
  }
  & when (not (@theme = dark) and not (@theme = variable)) {
    border-color: @hoverBorderColor;
  }
  & when not (@theme = variable) {
    box-shadow: @input-outline-offset @outline-blur-size @outline-width
      fade(@borderColor, @outline-fade);
  }
  & when (@theme = variable) {
    border-color: @hoverBorderColor;
    box-shadow: @input-outline-offset @outline-blur-size @outline-width
      @outlineColor;
  }
  border-right-width: @border-width-base;
  outline: 0;
}

// == when hover
.hover(@color: @input-hover-border-color) {
  border-color: @color;
  border-right-width: @border-width-base;
}

.disabled() {
  color: @input-disabled-color;
  background-color: @input-disabled-bg;
  border-color: @input-border-color;
  box-shadow: none;
  cursor: not-allowed;
  opacity: 1;

  &:hover {
    .hover(@input-border-color);
  }
}

// Basic style for input
.input() {
  position: relative;
  display: inline-block;
  width: 100%;
  min-width: 0;
  padding: @input-padding-vertical-base @input-padding-horizontal-base;
  color: @input-color;
  font-size: @font-size-base;
  line-height: @line-height-base;
  background-color: @input-bg;
  background-image: none;
  border: @border-width-base @border-style-base @input-border-color;
  border-radius: @control-border-radius;
  transition: all 0.3s;
  .placeholder(); // Reset placeholder

  &:hover {
    .hover();
  }

  &:focus,
  &-focused {
    .active();
  }

  &-disabled {
    .disabled();
  }

  &[disabled] {
    .disabled();
  }

  &&-borderless,
  &&-borderless:hover,
  &&-borderless:focus,
  &&-borderless&-focused,
  &&-borderless&-disabled,
  &&-borderless&[disabled] {
    background-color: transparent;
    border: none;
    box-shadow: none;
  }

  &&-filled {
    background-color: @input-variant-filled-bg;
    border-color: transparent;
    box-shadow: none;
  }

  &&-filled:hover {
    background-color: @input-variant-filled-hover-bg;
  }

  &&-filled:focus,
  &&-filled&-focused {
    .active();
    background-color: transparent;
    box-shadow: none;
  }

  &&-filled&-disabled,
  &&-filled&[disabled] {
    .disabled();
  }

  &&-underlined,
  &&-underlined:hover,
  &&-underlined:focus,
  &&-underlined&-focused,
  &&-underlined&-disabled,
  &&-underlined&[disabled] {
    background-color: transparent;
    border-width: 0 0 @border-width-base;
    border-radius: 0;
    box-shadow: none;
  }

  &&-underlined:hover:not(&-focused):not(:focus) {
    border-color: @input-border-color;
  }

  // Reset height for `textarea`s
  textarea& {
    max-width: 100%; // prevent textearea resize from coming out of its container
    height: auto;
    min-height: @input-height-base;
    line-height: @line-height-base;
    vertical-align: bottom;
    transition:
      all 0.3s,
      height 0s;
  }

  // Size
  &-lg {
    .input-lg();
  }

  &-sm {
    .input-sm();
  }
}

// label input
.input-group(@inputClass) {
  position: relative;
  display: table;
  width: 100%;
  border-collapse: separate;
  border-spacing: 0;

  // Undo padding and float of grid classes
  &[class*="col-"] {
    float: none;
    padding-right: 0;
    padding-left: 0;
  }

  > [class*="col-"] {
    padding-right: 8px;

    &:last-child {
      padding-right: 0;
    }
  }

  &-addon,
  &-wrap,
  > .@{inputClass} {
    display: table-cell;

    &:not(:first-child):not(:last-child) {
      border-radius: 0;
    }
  }

  &-addon,
  &-wrap {
    width: 1px; // To make addon/wrap as small as possible
    white-space: nowrap;
    vertical-align: middle;
  }

  &-wrap > * {
    display: block !important;
  }

  .@{inputClass} {
    float: left;
    width: 100%;
    margin-bottom: 0;
    text-align: inherit;

    &:focus {
      z-index: 1; // Fix https://gw.alipayobjects.com/zos/rmsportal/DHNpoqfMXSfrSnlZvhsJ.png
      border-right-width: 1px;
    }

    &:hover {
      z-index: 1;
      border-right-width: 1px;
      .@{ant-prefix}-input-search-with-button & {
        z-index: 0;
      }
    }
  }

  &-addon {
    position: relative;
    padding: 0 @input-padding-horizontal-base;
    color: @input-color;
    font-weight: normal;
    font-size: @font-size-base;
    text-align: center;
    background-color: @input-addon-bg;
    border: @border-width-base @border-style-base @input-border-color;
    border-radius: @control-border-radius;
    transition: all 0.3s;

    // Reset Select's style in addon
    .@{ant-prefix}-select {
      margin: -(@input-padding-vertical-base + 1px)
        (-@input-padding-horizontal-base);

      &.@{ant-prefix}-select-single:not(.@{ant-prefix}-select-customize-input)
        .@{ant-prefix}-select-selector {
        background-color: inherit;
        border: @border-width-base @border-style-base transparent;
        box-shadow: none;
      }

      &-open,
      &-focused {
        .@{ant-prefix}-select-selector {
          color: @primary-color;
        }
      }
    }

    // https://github.com/ant-design/ant-design/issues/31333
    .@{ant-prefix}-cascader-picker {
      margin: -9px (-@control-padding-horizontal);
      background-color: transparent;
      .@{ant-prefix}-cascader-input {
        text-align: left;
        border: 0;
        box-shadow: none;
      }
    }
  }

  // Reset rounded corners
  > .@{inputClass}:first-child,
  &-addon:first-child {
    border-top-right-radius: 0;
    border-bottom-right-radius: 0;

    // Reset Select's style in addon
    .@{ant-prefix}-select .@{ant-prefix}-select-selector {
      border-top-right-radius: 0;
      border-bottom-right-radius: 0;
    }
  }

  > .@{inputClass}-affix-wrapper {
    &:not(:first-child) .@{inputClass} {
      border-top-left-radius: 0;
      border-bottom-left-radius: 0;
    }

    &:not(:last-child) .@{inputClass} {
      border-top-right-radius: 0;
      border-bottom-right-radius: 0;
    }
  }

  &-addon:first-child {
    border-right: 0;
  }

  &-addon:last-child {
    border-left: 0;
  }

  > .@{inputClass}:last-child,
  &-addon:last-child {
    border-top-left-radius: 0;
    border-bottom-left-radius: 0;

    // Reset Select's style in addon
    .@{ant-prefix}-select .@{ant-prefix}-select-selector {
      border-top-left-radius: 0;
      border-bottom-left-radius: 0;
    }
  }

  // Sizing options
  &-lg .@{inputClass},
  &-lg > &-addon {
    .input-lg();
  }

  &-sm .@{inputClass},
  &-sm > &-addon {
    .input-sm();
  }

  // Fix https://github.com/ant-design/ant-design/issues/5754
  &-lg .@{ant-prefix}-select-single .@{ant-prefix}-select-selector {
    height: @input-height-lg;
  }

  &-sm .@{ant-prefix}-select-single .@{ant-prefix}-select-selector {
    height: @input-height-sm;
  }

  .@{inputClass}-affix-wrapper {
    &:not(:last-child) {
      border-top-right-radius: 0;
      border-bottom-right-radius: 0;
      .@{ant-prefix}-input-search & {
        border-top-left-radius: @control-border-radius;
        border-bottom-left-radius: @control-border-radius;
      }
    }

    &:not(:first-child),
    .@{ant-prefix}-input-search &:not(:first-child) {
      border-top-left-radius: 0;
      border-bottom-left-radius: 0;
    }
  }

  &&-compact {
    display: block;
    .clearfix();

    &-addon,
    &-wrap,
    > .@{inputClass} {
      &:not(:first-child):not(:last-child) {
        border-right-width: @border-width-base;

        &:hover {
          z-index: 1;
        }

        &:focus {
          z-index: 1;
        }
      }
    }

    & > * {
      display: inline-block;
      float: none;
      vertical-align: top; // https://github.com/ant-design/ant-design-pro/issues/139
      border-radius: 0;
    }

    & > .@{inputClass}-affix-wrapper,
    & > .@{inputClass}-number-affix-wrapper,
    & > .@{ant-prefix}-picker-range {
      display: inline-flex;
    }

    & > *:not(:last-child) {
      margin-right: -@border-width-base;
      border-right-width: @border-width-base;
    }

    // Undo float for .ant-input-group .ant-input
    .@{inputClass} {
      float: none;
    }

    // reset border for Select, DatePicker, AutoComplete, Cascader, Mention, TimePicker, Input
    & > .@{ant-prefix}-select > .@{ant-prefix}-select-selector,
    & > .@{ant-prefix}-select-auto-complete .@{ant-prefix}-input,
    & > .@{ant-prefix}-cascader-picker .@{ant-prefix}-input,
    & > .@{ant-prefix}-input-group-wrapper .@{ant-prefix}-input {
      border-right-width: @border-width-base;
      border-radius: 0;

      &:hover {
        z-index: 1;
      }

      &:focus {
        z-index: 1;
      }
    }

    & > .@{ant-prefix}-select-focused {
      z-index: 1;
    }

    // update z-index for arrow icon
    & > .@{ant-prefix}-select > .@{ant-prefix}-select-arrow {
      z-index: 1; // https://github.com/ant-design/ant-design/issues/20371
    }

    & > *:first-child,
    & > .@{ant-prefix}-select:first-child > .@{ant-prefix}-select-selector,
    & > .@{ant-prefix}-select-auto-complete:first-child .@{ant-prefix}-input,
    & > .@{ant-prefix}-cascader-picker:first-child .@{ant-prefix}-input {
      border-top-left-radius: @control-border-radius;
      border-bottom-left-radius: @control-border-radius;
    }

    & > *:last-child,
    & > .@{ant-prefix}-select:last-child > .@{ant-prefix}-select-selector,
    & > .@{ant-prefix}-cascader-picker:last-child .@{ant-prefix}-input,
    & > .@{ant-prefix}-cascader-picker-focused:last-child .@{ant-prefix}-input {
      border-right-width: @border-width-base;
      border-top-right-radius: @control-border-radius;
      border-bottom-right-radius: @control-border-radius;
    }

    // https://github.com/ant-design/ant-design/issues/12493
    & > .@{ant-prefix}-select-auto-complete .@{ant-prefix}-input {
      vertical-align: top;
    }

    .@{ant-prefix}-input-group-wrapper + .@{ant-prefix}-input-group-wrapper {
      margin-left: -1px;
      .@{ant-prefix}-input-affix-wrapper {
        border-radius: 0;
      }
    }

    .@{ant-prefix}-input-group-wrapper:not(:last-child) {
      &.@{ant-prefix}-input-search > .@{ant-prefix}-input-group {
        &
          > .@{ant-prefix}-input-group-addon
          > .@{ant-prefix}-input-search-button {
          border-radius: 0;
        }

        & > .@{ant-prefix}-input {
          border-radius: @control-border-radius 0 0 @control-border-radius;
        }
      }
    }
  }
}

.input-group-wrapper-compact(@inputClass) {
  // Fix the issue of using icons in Space Compact mode
  // https://github.com/ant-design/ant-design/issues/42122
  &:not(.@{inputClass}-compact-first-item):not(
    .@{inputClass}-compact-last-item
  ).@{inputClass}-compact-item {
    .@{inputClass},
    .@{inputClass}-group-addon {
      border-radius: 0;
    }
  }

  &:not(.@{inputClass}-compact-last-item).@{inputClass}-compact-first-item {
    .@{inputClass},
    .@{inputClass}-group-addon {
      border-start-end-radius: 0;
      border-end-end-radius: 0;
    }
  }

  &:not(.@{inputClass}-compact-first-item).@{inputClass}-compact-last-item {
    .@{inputClass},
    .@{inputClass}-group-addon {
      border-start-start-radius: 0;
      border-end-start-radius: 0;
    }
  }

  // Fix the issue of input use show-count param in space compact mode
  // https://github.com/ant-design/ant-design/issues/46872
  &:not(.@{inputClass}-compact-last-item).@{inputClass}-compact-item {
    .@{inputClass}-affix-wrapper {
      border-start-end-radius: 0;
      border-end-end-radius: 0;
    }
  }

  // Fix the issue of input use `addonAfter` param in space compact mode
  // https://github.com/ant-design/ant-design/issues/52483
  &:not(.@{inputClass}-compact-first-item).@{inputClass}-compact-item {
    .@{inputClass}-affix-wrapper {
      border-start-start-radius: 0;
      border-end-start-radius: 0;
    }
  }
}

.status-color(
  @prefix-cls: @input-prefix-cls;
  @text-color: @input-color;
  @border-color: @input-border-color;
  @background-color: @input-bg;
  @hoverBorderColor: @primary-color-hover;
  @outlineColor: @primary-color-outline;
) {
  &:not(.@{prefix-cls}-disabled):not(.@{prefix-cls}-borderless).@{prefix-cls} {
    &,
    &:hover {
      background: @background-color;
      border-color: @border-color;
    }

    &:focus,
    &-focused {
      .active(@text-color, @hoverBorderColor, @outlineColor);
    }
  }
}

.status-color-common(
  @prefix-cls: @input-prefix-cls;
  @text-color: @input-color;
  @border-color: @input-border-color;
  @background-color: @input-bg;
  @hoverBorderColor: @primary-color-hover;
  @outlineColor: @primary-color-outline;
) {
  .@{prefix-cls}-prefix {
    color: @text-color;
  }
}

.group-status-color(
  @prefix-cls: @input-prefix-cls;
  @text-color: @input-color;
  @border-color: @input-border-color;
) {
  .@{prefix-cls}-group-addon {
    color: @text-color;
    border-color: @border-color;
  }
}

`````

### Expected (prettier)

`````less
@import "../../style/themes/index";
@import "../../style/mixins/index";

@input-affix-with-clear-btn-width: 38px;

// size mixins for input
.input-lg() {
  padding: @input-padding-vertical-lg @input-padding-horizontal-lg;
  font-size: @font-size-lg;
}

.input-sm() {
  padding: @input-padding-vertical-sm @input-padding-horizontal-sm;
}

// input status
// == when focus or active
.active(@borderColor: @primary-color; @hoverBorderColor: @primary-color-hover; @outlineColor: @primary-color-outline) {
  & when (@theme = dark) {
    border-color: @borderColor;
  }
  & when (not (@theme = dark) and not (@theme = variable)) {
    border-color: @hoverBorderColor;
  }
  & when not (@theme = variable) {
    box-shadow: @input-outline-offset @outline-blur-size @outline-width
      fade(@borderColor, @outline-fade);
  }
  & when (@theme = variable) {
    border-color: @hoverBorderColor;
    box-shadow: @input-outline-offset @outline-blur-size @outline-width
      @outlineColor;
  }
  border-right-width: @border-width-base;
  outline: 0;
}

// == when hover
.hover(@color: @input-hover-border-color) {
  border-color: @color;
  border-right-width: @border-width-base;
}

.disabled() {
  color: @input-disabled-color;
  background-color: @input-disabled-bg;
  border-color: @input-border-color;
  box-shadow: none;
  cursor: not-allowed;
  opacity: 1;

  &:hover {
    .hover(@input-border-color);
  }
}

// Basic style for input
.input() {
  position: relative;
  display: inline-block;
  width: 100%;
  min-width: 0;
  padding: @input-padding-vertical-base @input-padding-horizontal-base;
  color: @input-color;
  font-size: @font-size-base;
  line-height: @line-height-base;
  background-color: @input-bg;
  background-image: none;
  border: @border-width-base @border-style-base @input-border-color;
  border-radius: @control-border-radius;
  transition: all 0.3s;
  .placeholder(); // Reset placeholder

  &:hover {
    .hover();
  }

  &:focus,
  &-focused {
    .active();
  }

  &-disabled {
    .disabled();
  }

  &[disabled] {
    .disabled();
  }

  &&-borderless,
  &&-borderless:hover,
  &&-borderless:focus,
  &&-borderless&-focused,
  &&-borderless&-disabled,
  &&-borderless&[disabled] {
    background-color: transparent;
    border: none;
    box-shadow: none;
  }

  &&-filled {
    background-color: @input-variant-filled-bg;
    border-color: transparent;
    box-shadow: none;
  }

  &&-filled:hover {
    background-color: @input-variant-filled-hover-bg;
  }

  &&-filled:focus,
  &&-filled&-focused {
    .active();
    background-color: transparent;
    box-shadow: none;
  }

  &&-filled&-disabled,
  &&-filled&[disabled] {
    .disabled();
  }

  &&-underlined,
  &&-underlined:hover,
  &&-underlined:focus,
  &&-underlined&-focused,
  &&-underlined&-disabled,
  &&-underlined&[disabled] {
    background-color: transparent;
    border-width: 0 0 @border-width-base;
    border-radius: 0;
    box-shadow: none;
  }

  &&-underlined:hover:not(&-focused):not(:focus) {
    border-color: @input-border-color;
  }

  // Reset height for `textarea`s
  textarea& {
    max-width: 100%; // prevent textearea resize from coming out of its container
    height: auto;
    min-height: @input-height-base;
    line-height: @line-height-base;
    vertical-align: bottom;
    transition:
      all 0.3s,
      height 0s;
  }

  // Size
  &-lg {
    .input-lg();
  }

  &-sm {
    .input-sm();
  }
}

// label input
.input-group(@inputClass) {
  position: relative;
  display: table;
  width: 100%;
  border-collapse: separate;
  border-spacing: 0;

  // Undo padding and float of grid classes
  &[class*="col-"] {
    float: none;
    padding-right: 0;
    padding-left: 0;
  }

  > [class*="col-"] {
    padding-right: 8px;

    &:last-child {
      padding-right: 0;
    }
  }

  &-addon,
  &-wrap,
  > .@{inputClass} {
    display: table-cell;

    &:not(:first-child):not(:last-child) {
      border-radius: 0;
    }
  }

  &-addon,
  &-wrap {
    width: 1px; // To make addon/wrap as small as possible
    white-space: nowrap;
    vertical-align: middle;
  }

  &-wrap > * {
    display: block !important;
  }

  .@{inputClass} {
    float: left;
    width: 100%;
    margin-bottom: 0;
    text-align: inherit;

    &:focus {
      z-index: 1; // Fix https://gw.alipayobjects.com/zos/rmsportal/DHNpoqfMXSfrSnlZvhsJ.png
      border-right-width: 1px;
    }

    &:hover {
      z-index: 1;
      border-right-width: 1px;
      .@{ant-prefix}-input-search-with-button & {
        z-index: 0;
      }
    }
  }

  &-addon {
    position: relative;
    padding: 0 @input-padding-horizontal-base;
    color: @input-color;
    font-weight: normal;
    font-size: @font-size-base;
    text-align: center;
    background-color: @input-addon-bg;
    border: @border-width-base @border-style-base @input-border-color;
    border-radius: @control-border-radius;
    transition: all 0.3s;

    // Reset Select's style in addon
    .@{ant-prefix}-select {
      margin: -(@input-padding-vertical-base + 1px)
        (-@input-padding-horizontal-base);

      &.@{ant-prefix}-select-single:not(.@{ant-prefix}-select-customize-input)
        .@{ant-prefix}-select-selector {
        background-color: inherit;
        border: @border-width-base @border-style-base transparent;
        box-shadow: none;
      }

      &-open,
      &-focused {
        .@{ant-prefix}-select-selector {
          color: @primary-color;
        }
      }
    }

    // https://github.com/ant-design/ant-design/issues/31333
    .@{ant-prefix}-cascader-picker {
      margin: -9px (-@control-padding-horizontal);
      background-color: transparent;
      .@{ant-prefix}-cascader-input {
        text-align: left;
        border: 0;
        box-shadow: none;
      }
    }
  }

  // Reset rounded corners
  > .@{inputClass}:first-child,
  &-addon:first-child {
    border-top-right-radius: 0;
    border-bottom-right-radius: 0;

    // Reset Select's style in addon
    .@{ant-prefix}-select .@{ant-prefix}-select-selector {
      border-top-right-radius: 0;
      border-bottom-right-radius: 0;
    }
  }

  > .@{inputClass}-affix-wrapper {
    &:not(:first-child) .@{inputClass} {
      border-top-left-radius: 0;
      border-bottom-left-radius: 0;
    }

    &:not(:last-child) .@{inputClass} {
      border-top-right-radius: 0;
      border-bottom-right-radius: 0;
    }
  }

  &-addon:first-child {
    border-right: 0;
  }

  &-addon:last-child {
    border-left: 0;
  }

  > .@{inputClass}:last-child,
  &-addon:last-child {
    border-top-left-radius: 0;
    border-bottom-left-radius: 0;

    // Reset Select's style in addon
    .@{ant-prefix}-select .@{ant-prefix}-select-selector {
      border-top-left-radius: 0;
      border-bottom-left-radius: 0;
    }
  }

  // Sizing options
  &-lg .@{inputClass},
  &-lg > &-addon {
    .input-lg();
  }

  &-sm .@{inputClass},
  &-sm > &-addon {
    .input-sm();
  }

  // Fix https://github.com/ant-design/ant-design/issues/5754
  &-lg .@{ant-prefix}-select-single .@{ant-prefix}-select-selector {
    height: @input-height-lg;
  }

  &-sm .@{ant-prefix}-select-single .@{ant-prefix}-select-selector {
    height: @input-height-sm;
  }

  .@{inputClass}-affix-wrapper {
    &:not(:last-child) {
      border-top-right-radius: 0;
      border-bottom-right-radius: 0;
      .@{ant-prefix}-input-search & {
        border-top-left-radius: @control-border-radius;
        border-bottom-left-radius: @control-border-radius;
      }
    }

    &:not(:first-child),
    .@{ant-prefix}-input-search &:not(:first-child) {
      border-top-left-radius: 0;
      border-bottom-left-radius: 0;
    }
  }

  &&-compact {
    display: block;
    .clearfix();

    &-addon,
    &-wrap,
    > .@{inputClass} {
      &:not(:first-child):not(:last-child) {
        border-right-width: @border-width-base;

        &:hover {
          z-index: 1;
        }

        &:focus {
          z-index: 1;
        }
      }
    }

    & > * {
      display: inline-block;
      float: none;
      vertical-align: top; // https://github.com/ant-design/ant-design-pro/issues/139
      border-radius: 0;
    }

    & > .@{inputClass}-affix-wrapper,
    & > .@{inputClass}-number-affix-wrapper,
    & > .@{ant-prefix}-picker-range {
      display: inline-flex;
    }

    & > *:not(:last-child) {
      margin-right: -@border-width-base;
      border-right-width: @border-width-base;
    }

    // Undo float for .ant-input-group .ant-input
    .@{inputClass} {
      float: none;
    }

    // reset border for Select, DatePicker, AutoComplete, Cascader, Mention, TimePicker, Input
    & > .@{ant-prefix}-select > .@{ant-prefix}-select-selector,
    & > .@{ant-prefix}-select-auto-complete .@{ant-prefix}-input,
    & > .@{ant-prefix}-cascader-picker .@{ant-prefix}-input,
    & > .@{ant-prefix}-input-group-wrapper .@{ant-prefix}-input {
      border-right-width: @border-width-base;
      border-radius: 0;

      &:hover {
        z-index: 1;
      }

      &:focus {
        z-index: 1;
      }
    }

    & > .@{ant-prefix}-select-focused {
      z-index: 1;
    }

    // update z-index for arrow icon
    & > .@{ant-prefix}-select > .@{ant-prefix}-select-arrow {
      z-index: 1; // https://github.com/ant-design/ant-design/issues/20371
    }

    & > *:first-child,
    & > .@{ant-prefix}-select:first-child > .@{ant-prefix}-select-selector,
    & > .@{ant-prefix}-select-auto-complete:first-child .@{ant-prefix}-input,
    & > .@{ant-prefix}-cascader-picker:first-child .@{ant-prefix}-input {
      border-top-left-radius: @control-border-radius;
      border-bottom-left-radius: @control-border-radius;
    }

    & > *:last-child,
    & > .@{ant-prefix}-select:last-child > .@{ant-prefix}-select-selector,
    & > .@{ant-prefix}-cascader-picker:last-child .@{ant-prefix}-input,
    & > .@{ant-prefix}-cascader-picker-focused:last-child .@{ant-prefix}-input {
      border-right-width: @border-width-base;
      border-top-right-radius: @control-border-radius;
      border-bottom-right-radius: @control-border-radius;
    }

    // https://github.com/ant-design/ant-design/issues/12493
    & > .@{ant-prefix}-select-auto-complete .@{ant-prefix}-input {
      vertical-align: top;
    }

    .@{ant-prefix}-input-group-wrapper + .@{ant-prefix}-input-group-wrapper {
      margin-left: -1px;
      .@{ant-prefix}-input-affix-wrapper {
        border-radius: 0;
      }
    }

    .@{ant-prefix}-input-group-wrapper:not(:last-child) {
      &.@{ant-prefix}-input-search > .@{ant-prefix}-input-group {
        &
          > .@{ant-prefix}-input-group-addon
          > .@{ant-prefix}-input-search-button {
          border-radius: 0;
        }

        & > .@{ant-prefix}-input {
          border-radius: @control-border-radius 0 0 @control-border-radius;
        }
      }
    }
  }
}

.input-group-wrapper-compact(@inputClass) {
  // Fix the issue of using icons in Space Compact mode
  // https://github.com/ant-design/ant-design/issues/42122
  &:not(.@{inputClass}-compact-first-item):not(
      .@{inputClass}-compact-last-item
    ).@{inputClass}-compact-item {
    .@{inputClass},
    .@{inputClass}-group-addon {
      border-radius: 0;
    }
  }

  &:not(.@{inputClass}-compact-last-item).@{inputClass}-compact-first-item {
    .@{inputClass},
    .@{inputClass}-group-addon {
      border-start-end-radius: 0;
      border-end-end-radius: 0;
    }
  }

  &:not(.@{inputClass}-compact-first-item).@{inputClass}-compact-last-item {
    .@{inputClass},
    .@{inputClass}-group-addon {
      border-start-start-radius: 0;
      border-end-start-radius: 0;
    }
  }

  // Fix the issue of input use show-count param in space compact mode
  // https://github.com/ant-design/ant-design/issues/46872
  &:not(.@{inputClass}-compact-last-item).@{inputClass}-compact-item {
    .@{inputClass}-affix-wrapper {
      border-start-end-radius: 0;
      border-end-end-radius: 0;
    }
  }

  // Fix the issue of input use `addonAfter` param in space compact mode
  // https://github.com/ant-design/ant-design/issues/52483
  &:not(.@{inputClass}-compact-first-item).@{inputClass}-compact-item {
    .@{inputClass}-affix-wrapper {
      border-start-start-radius: 0;
      border-end-start-radius: 0;
    }
  }
}

.status-color(
  @prefix-cls: @input-prefix-cls;
  @text-color: @input-color;
  @border-color: @input-border-color;
  @background-color: @input-bg;
  @hoverBorderColor: @primary-color-hover;
  @outlineColor: @primary-color-outline;
) {
  &:not(.@{prefix-cls}-disabled):not(.@{prefix-cls}-borderless).@{prefix-cls} {
    &,
    &:hover {
      background: @background-color;
      border-color: @border-color;
    }

    &:focus,
    &-focused {
      .active(@text-color, @hoverBorderColor, @outlineColor);
    }
  }
}

.status-color-common(
  @prefix-cls: @input-prefix-cls;
  @text-color: @input-color;
  @border-color: @input-border-color;
  @background-color: @input-bg;
  @hoverBorderColor: @primary-color-hover;
  @outlineColor: @primary-color-outline;
) {
  .@{prefix-cls}-prefix {
    color: @text-color;
  }
}

.group-status-color(
  @prefix-cls: @input-prefix-cls;
  @text-color: @input-color;
  @border-color: @input-border-color;
) {
  .@{prefix-cls}-group-addon {
    color: @text-color;
    border-color: @border-color;
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
@@ -462,10 +462,10 @@
 .input-group-wrapper-compact(@inputClass) {
   // Fix the issue of using icons in Space Compact mode
   // https://github.com/ant-design/ant-design/issues/42122
   &:not(.@{inputClass}-compact-first-item):not(
-      .@{inputClass}-compact-last-item
-    ).@{inputClass}-compact-item {
+    .@{inputClass}-compact-last-item
+  ).@{inputClass}-compact-item {
     .@{inputClass},
     .@{inputClass}-group-addon {
       border-radius: 0;
     }

`````

### Actual (oxfmt)

`````less
@import "../../style/themes/index";
@import "../../style/mixins/index";

@input-affix-with-clear-btn-width: 38px;

// size mixins for input
.input-lg() {
  padding: @input-padding-vertical-lg @input-padding-horizontal-lg;
  font-size: @font-size-lg;
}

.input-sm() {
  padding: @input-padding-vertical-sm @input-padding-horizontal-sm;
}

// input status
// == when focus or active
.active(@borderColor: @primary-color; @hoverBorderColor: @primary-color-hover; @outlineColor: @primary-color-outline) {
  & when (@theme = dark) {
    border-color: @borderColor;
  }
  & when (not (@theme = dark) and not (@theme = variable)) {
    border-color: @hoverBorderColor;
  }
  & when not (@theme = variable) {
    box-shadow: @input-outline-offset @outline-blur-size @outline-width
      fade(@borderColor, @outline-fade);
  }
  & when (@theme = variable) {
    border-color: @hoverBorderColor;
    box-shadow: @input-outline-offset @outline-blur-size @outline-width @outlineColor;
  }
  border-right-width: @border-width-base;
  outline: 0;
}

// == when hover
.hover(@color: @input-hover-border-color) {
  border-color: @color;
  border-right-width: @border-width-base;
}

.disabled() {
  color: @input-disabled-color;
  background-color: @input-disabled-bg;
  border-color: @input-border-color;
  box-shadow: none;
  cursor: not-allowed;
  opacity: 1;

  &:hover {
    .hover(@input-border-color);
  }
}

// Basic style for input
.input() {
  position: relative;
  display: inline-block;
  width: 100%;
  min-width: 0;
  padding: @input-padding-vertical-base @input-padding-horizontal-base;
  color: @input-color;
  font-size: @font-size-base;
  line-height: @line-height-base;
  background-color: @input-bg;
  background-image: none;
  border: @border-width-base @border-style-base @input-border-color;
  border-radius: @control-border-radius;
  transition: all 0.3s;
  .placeholder(); // Reset placeholder

  &:hover {
    .hover();
  }

  &:focus,
  &-focused {
    .active();
  }

  &-disabled {
    .disabled();
  }

  &[disabled] {
    .disabled();
  }

  &&-borderless,
  &&-borderless:hover,
  &&-borderless:focus,
  &&-borderless&-focused,
  &&-borderless&-disabled,
  &&-borderless&[disabled] {
    background-color: transparent;
    border: none;
    box-shadow: none;
  }

  &&-filled {
    background-color: @input-variant-filled-bg;
    border-color: transparent;
    box-shadow: none;
  }

  &&-filled:hover {
    background-color: @input-variant-filled-hover-bg;
  }

  &&-filled:focus,
  &&-filled&-focused {
    .active();
    background-color: transparent;
    box-shadow: none;
  }

  &&-filled&-disabled,
  &&-filled&[disabled] {
    .disabled();
  }

  &&-underlined,
  &&-underlined:hover,
  &&-underlined:focus,
  &&-underlined&-focused,
  &&-underlined&-disabled,
  &&-underlined&[disabled] {
    background-color: transparent;
    border-width: 0 0 @border-width-base;
    border-radius: 0;
    box-shadow: none;
  }

  &&-underlined:hover:not(&-focused):not(:focus) {
    border-color: @input-border-color;
  }

  // Reset height for `textarea`s
  textarea& {
    max-width: 100%; // prevent textearea resize from coming out of its container
    height: auto;
    min-height: @input-height-base;
    line-height: @line-height-base;
    vertical-align: bottom;
    transition:
      all 0.3s,
      height 0s;
  }

  // Size
  &-lg {
    .input-lg();
  }

  &-sm {
    .input-sm();
  }
}

// label input
.input-group(@inputClass) {
  position: relative;
  display: table;
  width: 100%;
  border-collapse: separate;
  border-spacing: 0;

  // Undo padding and float of grid classes
  &[class*="col-"] {
    float: none;
    padding-right: 0;
    padding-left: 0;
  }

  > [class*="col-"] {
    padding-right: 8px;

    &:last-child {
      padding-right: 0;
    }
  }

  &-addon,
  &-wrap,
  > .@{inputClass} {
    display: table-cell;

    &:not(:first-child):not(:last-child) {
      border-radius: 0;
    }
  }

  &-addon,
  &-wrap {
    width: 1px; // To make addon/wrap as small as possible
    white-space: nowrap;
    vertical-align: middle;
  }

  &-wrap > * {
    display: block !important;
  }

  .@{inputClass} {
    float: left;
    width: 100%;
    margin-bottom: 0;
    text-align: inherit;

    &:focus {
      z-index: 1; // Fix https://gw.alipayobjects.com/zos/rmsportal/DHNpoqfMXSfrSnlZvhsJ.png
      border-right-width: 1px;
    }

    &:hover {
      z-index: 1;
      border-right-width: 1px;
      .@{ant-prefix}-input-search-with-button & {
        z-index: 0;
      }
    }
  }

  &-addon {
    position: relative;
    padding: 0 @input-padding-horizontal-base;
    color: @input-color;
    font-weight: normal;
    font-size: @font-size-base;
    text-align: center;
    background-color: @input-addon-bg;
    border: @border-width-base @border-style-base @input-border-color;
    border-radius: @control-border-radius;
    transition: all 0.3s;

    // Reset Select's style in addon
    .@{ant-prefix}-select {
      margin: -(@input-padding-vertical-base + 1px) (-@input-padding-horizontal-base);

      &.@{ant-prefix}-select-single:not(.@{ant-prefix}-select-customize-input)
        .@{ant-prefix}-select-selector {
        background-color: inherit;
        border: @border-width-base @border-style-base transparent;
        box-shadow: none;
      }

      &-open,
      &-focused {
        .@{ant-prefix}-select-selector {
          color: @primary-color;
        }
      }
    }

    // https://github.com/ant-design/ant-design/issues/31333
    .@{ant-prefix}-cascader-picker {
      margin: -9px (-@control-padding-horizontal);
      background-color: transparent;
      .@{ant-prefix}-cascader-input {
        text-align: left;
        border: 0;
        box-shadow: none;
      }
    }
  }

  // Reset rounded corners
  > .@{inputClass}:first-child,
  &-addon:first-child {
    border-top-right-radius: 0;
    border-bottom-right-radius: 0;

    // Reset Select's style in addon
    .@{ant-prefix}-select .@{ant-prefix}-select-selector {
      border-top-right-radius: 0;
      border-bottom-right-radius: 0;
    }
  }

  > .@{inputClass}-affix-wrapper {
    &:not(:first-child) .@{inputClass} {
      border-top-left-radius: 0;
      border-bottom-left-radius: 0;
    }

    &:not(:last-child) .@{inputClass} {
      border-top-right-radius: 0;
      border-bottom-right-radius: 0;
    }
  }

  &-addon:first-child {
    border-right: 0;
  }

  &-addon:last-child {
    border-left: 0;
  }

  > .@{inputClass}:last-child,
  &-addon:last-child {
    border-top-left-radius: 0;
    border-bottom-left-radius: 0;

    // Reset Select's style in addon
    .@{ant-prefix}-select .@{ant-prefix}-select-selector {
      border-top-left-radius: 0;
      border-bottom-left-radius: 0;
    }
  }

  // Sizing options
  &-lg .@{inputClass},
  &-lg > &-addon {
    .input-lg();
  }

  &-sm .@{inputClass},
  &-sm > &-addon {
    .input-sm();
  }

  // Fix https://github.com/ant-design/ant-design/issues/5754
  &-lg .@{ant-prefix}-select-single .@{ant-prefix}-select-selector {
    height: @input-height-lg;
  }

  &-sm .@{ant-prefix}-select-single .@{ant-prefix}-select-selector {
    height: @input-height-sm;
  }

  .@{inputClass}-affix-wrapper {
    &:not(:last-child) {
      border-top-right-radius: 0;
      border-bottom-right-radius: 0;
      .@{ant-prefix}-input-search & {
        border-top-left-radius: @control-border-radius;
        border-bottom-left-radius: @control-border-radius;
      }
    }

    &:not(:first-child),
    .@{ant-prefix}-input-search &:not(:first-child) {
      border-top-left-radius: 0;
      border-bottom-left-radius: 0;
    }
  }

  &&-compact {
    display: block;
    .clearfix();

    &-addon,
    &-wrap,
    > .@{inputClass} {
      &:not(:first-child):not(:last-child) {
        border-right-width: @border-width-base;

        &:hover {
          z-index: 1;
        }

        &:focus {
          z-index: 1;
        }
      }
    }

    & > * {
      display: inline-block;
      float: none;
      vertical-align: top; // https://github.com/ant-design/ant-design-pro/issues/139
      border-radius: 0;
    }

    & > .@{inputClass}-affix-wrapper,
    & > .@{inputClass}-number-affix-wrapper,
    & > .@{ant-prefix}-picker-range {
      display: inline-flex;
    }

    & > *:not(:last-child) {
      margin-right: -@border-width-base;
      border-right-width: @border-width-base;
    }

    // Undo float for .ant-input-group .ant-input
    .@{inputClass} {
      float: none;
    }

    // reset border for Select, DatePicker, AutoComplete, Cascader, Mention, TimePicker, Input
    & > .@{ant-prefix}-select > .@{ant-prefix}-select-selector,
    & > .@{ant-prefix}-select-auto-complete .@{ant-prefix}-input,
    & > .@{ant-prefix}-cascader-picker .@{ant-prefix}-input,
    & > .@{ant-prefix}-input-group-wrapper .@{ant-prefix}-input {
      border-right-width: @border-width-base;
      border-radius: 0;

      &:hover {
        z-index: 1;
      }

      &:focus {
        z-index: 1;
      }
    }

    & > .@{ant-prefix}-select-focused {
      z-index: 1;
    }

    // update z-index for arrow icon
    & > .@{ant-prefix}-select > .@{ant-prefix}-select-arrow {
      z-index: 1; // https://github.com/ant-design/ant-design/issues/20371
    }

    & > *:first-child,
    & > .@{ant-prefix}-select:first-child > .@{ant-prefix}-select-selector,
    & > .@{ant-prefix}-select-auto-complete:first-child .@{ant-prefix}-input,
    & > .@{ant-prefix}-cascader-picker:first-child .@{ant-prefix}-input {
      border-top-left-radius: @control-border-radius;
      border-bottom-left-radius: @control-border-radius;
    }

    & > *:last-child,
    & > .@{ant-prefix}-select:last-child > .@{ant-prefix}-select-selector,
    & > .@{ant-prefix}-cascader-picker:last-child .@{ant-prefix}-input,
    & > .@{ant-prefix}-cascader-picker-focused:last-child .@{ant-prefix}-input {
      border-right-width: @border-width-base;
      border-top-right-radius: @control-border-radius;
      border-bottom-right-radius: @control-border-radius;
    }

    // https://github.com/ant-design/ant-design/issues/12493
    & > .@{ant-prefix}-select-auto-complete .@{ant-prefix}-input {
      vertical-align: top;
    }

    .@{ant-prefix}-input-group-wrapper + .@{ant-prefix}-input-group-wrapper {
      margin-left: -1px;
      .@{ant-prefix}-input-affix-wrapper {
        border-radius: 0;
      }
    }

    .@{ant-prefix}-input-group-wrapper:not(:last-child) {
      &.@{ant-prefix}-input-search > .@{ant-prefix}-input-group {
        & > .@{ant-prefix}-input-group-addon > .@{ant-prefix}-input-search-button {
          border-radius: 0;
        }

        & > .@{ant-prefix}-input {
          border-radius: @control-border-radius 0 0 @control-border-radius;
        }
      }
    }
  }
}

.input-group-wrapper-compact(@inputClass) {
  // Fix the issue of using icons in Space Compact mode
  // https://github.com/ant-design/ant-design/issues/42122
  &:not(.@{inputClass}-compact-first-item):not(
    .@{inputClass}-compact-last-item
  ).@{inputClass}-compact-item {
    .@{inputClass},
    .@{inputClass}-group-addon {
      border-radius: 0;
    }
  }

  &:not(.@{inputClass}-compact-last-item).@{inputClass}-compact-first-item {
    .@{inputClass},
    .@{inputClass}-group-addon {
      border-start-end-radius: 0;
      border-end-end-radius: 0;
    }
  }

  &:not(.@{inputClass}-compact-first-item).@{inputClass}-compact-last-item {
    .@{inputClass},
    .@{inputClass}-group-addon {
      border-start-start-radius: 0;
      border-end-start-radius: 0;
    }
  }

  // Fix the issue of input use show-count param in space compact mode
  // https://github.com/ant-design/ant-design/issues/46872
  &:not(.@{inputClass}-compact-last-item).@{inputClass}-compact-item {
    .@{inputClass}-affix-wrapper {
      border-start-end-radius: 0;
      border-end-end-radius: 0;
    }
  }

  // Fix the issue of input use `addonAfter` param in space compact mode
  // https://github.com/ant-design/ant-design/issues/52483
  &:not(.@{inputClass}-compact-first-item).@{inputClass}-compact-item {
    .@{inputClass}-affix-wrapper {
      border-start-start-radius: 0;
      border-end-start-radius: 0;
    }
  }
}

.status-color(
  @prefix-cls: @input-prefix-cls;
  @text-color: @input-color;
  @border-color: @input-border-color;
  @background-color: @input-bg;
  @hoverBorderColor: @primary-color-hover;
  @outlineColor: @primary-color-outline;
) {
  &:not(.@{prefix-cls}-disabled):not(.@{prefix-cls}-borderless).@{prefix-cls} {
    &,
    &:hover {
      background: @background-color;
      border-color: @border-color;
    }

    &:focus,
    &-focused {
      .active(@text-color, @hoverBorderColor, @outlineColor);
    }
  }
}

.status-color-common(
  @prefix-cls: @input-prefix-cls;
  @text-color: @input-color;
  @border-color: @input-border-color;
  @background-color: @input-bg;
  @hoverBorderColor: @primary-color-hover;
  @outlineColor: @primary-color-outline;
) {
  .@{prefix-cls}-prefix {
    color: @text-color;
  }
}

.group-status-color(
  @prefix-cls: @input-prefix-cls;
  @text-color: @input-color;
  @border-color: @input-border-color;
) {
  .@{prefix-cls}-group-addon {
    color: @text-color;
    border-color: @border-color;
  }
}

`````

### Expected (prettier)

`````less
@import "../../style/themes/index";
@import "../../style/mixins/index";

@input-affix-with-clear-btn-width: 38px;

// size mixins for input
.input-lg() {
  padding: @input-padding-vertical-lg @input-padding-horizontal-lg;
  font-size: @font-size-lg;
}

.input-sm() {
  padding: @input-padding-vertical-sm @input-padding-horizontal-sm;
}

// input status
// == when focus or active
.active(@borderColor: @primary-color; @hoverBorderColor: @primary-color-hover; @outlineColor: @primary-color-outline) {
  & when (@theme = dark) {
    border-color: @borderColor;
  }
  & when (not (@theme = dark) and not (@theme = variable)) {
    border-color: @hoverBorderColor;
  }
  & when not (@theme = variable) {
    box-shadow: @input-outline-offset @outline-blur-size @outline-width
      fade(@borderColor, @outline-fade);
  }
  & when (@theme = variable) {
    border-color: @hoverBorderColor;
    box-shadow: @input-outline-offset @outline-blur-size @outline-width @outlineColor;
  }
  border-right-width: @border-width-base;
  outline: 0;
}

// == when hover
.hover(@color: @input-hover-border-color) {
  border-color: @color;
  border-right-width: @border-width-base;
}

.disabled() {
  color: @input-disabled-color;
  background-color: @input-disabled-bg;
  border-color: @input-border-color;
  box-shadow: none;
  cursor: not-allowed;
  opacity: 1;

  &:hover {
    .hover(@input-border-color);
  }
}

// Basic style for input
.input() {
  position: relative;
  display: inline-block;
  width: 100%;
  min-width: 0;
  padding: @input-padding-vertical-base @input-padding-horizontal-base;
  color: @input-color;
  font-size: @font-size-base;
  line-height: @line-height-base;
  background-color: @input-bg;
  background-image: none;
  border: @border-width-base @border-style-base @input-border-color;
  border-radius: @control-border-radius;
  transition: all 0.3s;
  .placeholder(); // Reset placeholder

  &:hover {
    .hover();
  }

  &:focus,
  &-focused {
    .active();
  }

  &-disabled {
    .disabled();
  }

  &[disabled] {
    .disabled();
  }

  &&-borderless,
  &&-borderless:hover,
  &&-borderless:focus,
  &&-borderless&-focused,
  &&-borderless&-disabled,
  &&-borderless&[disabled] {
    background-color: transparent;
    border: none;
    box-shadow: none;
  }

  &&-filled {
    background-color: @input-variant-filled-bg;
    border-color: transparent;
    box-shadow: none;
  }

  &&-filled:hover {
    background-color: @input-variant-filled-hover-bg;
  }

  &&-filled:focus,
  &&-filled&-focused {
    .active();
    background-color: transparent;
    box-shadow: none;
  }

  &&-filled&-disabled,
  &&-filled&[disabled] {
    .disabled();
  }

  &&-underlined,
  &&-underlined:hover,
  &&-underlined:focus,
  &&-underlined&-focused,
  &&-underlined&-disabled,
  &&-underlined&[disabled] {
    background-color: transparent;
    border-width: 0 0 @border-width-base;
    border-radius: 0;
    box-shadow: none;
  }

  &&-underlined:hover:not(&-focused):not(:focus) {
    border-color: @input-border-color;
  }

  // Reset height for `textarea`s
  textarea& {
    max-width: 100%; // prevent textearea resize from coming out of its container
    height: auto;
    min-height: @input-height-base;
    line-height: @line-height-base;
    vertical-align: bottom;
    transition:
      all 0.3s,
      height 0s;
  }

  // Size
  &-lg {
    .input-lg();
  }

  &-sm {
    .input-sm();
  }
}

// label input
.input-group(@inputClass) {
  position: relative;
  display: table;
  width: 100%;
  border-collapse: separate;
  border-spacing: 0;

  // Undo padding and float of grid classes
  &[class*="col-"] {
    float: none;
    padding-right: 0;
    padding-left: 0;
  }

  > [class*="col-"] {
    padding-right: 8px;

    &:last-child {
      padding-right: 0;
    }
  }

  &-addon,
  &-wrap,
  > .@{inputClass} {
    display: table-cell;

    &:not(:first-child):not(:last-child) {
      border-radius: 0;
    }
  }

  &-addon,
  &-wrap {
    width: 1px; // To make addon/wrap as small as possible
    white-space: nowrap;
    vertical-align: middle;
  }

  &-wrap > * {
    display: block !important;
  }

  .@{inputClass} {
    float: left;
    width: 100%;
    margin-bottom: 0;
    text-align: inherit;

    &:focus {
      z-index: 1; // Fix https://gw.alipayobjects.com/zos/rmsportal/DHNpoqfMXSfrSnlZvhsJ.png
      border-right-width: 1px;
    }

    &:hover {
      z-index: 1;
      border-right-width: 1px;
      .@{ant-prefix}-input-search-with-button & {
        z-index: 0;
      }
    }
  }

  &-addon {
    position: relative;
    padding: 0 @input-padding-horizontal-base;
    color: @input-color;
    font-weight: normal;
    font-size: @font-size-base;
    text-align: center;
    background-color: @input-addon-bg;
    border: @border-width-base @border-style-base @input-border-color;
    border-radius: @control-border-radius;
    transition: all 0.3s;

    // Reset Select's style in addon
    .@{ant-prefix}-select {
      margin: -(@input-padding-vertical-base + 1px) (-@input-padding-horizontal-base);

      &.@{ant-prefix}-select-single:not(.@{ant-prefix}-select-customize-input)
        .@{ant-prefix}-select-selector {
        background-color: inherit;
        border: @border-width-base @border-style-base transparent;
        box-shadow: none;
      }

      &-open,
      &-focused {
        .@{ant-prefix}-select-selector {
          color: @primary-color;
        }
      }
    }

    // https://github.com/ant-design/ant-design/issues/31333
    .@{ant-prefix}-cascader-picker {
      margin: -9px (-@control-padding-horizontal);
      background-color: transparent;
      .@{ant-prefix}-cascader-input {
        text-align: left;
        border: 0;
        box-shadow: none;
      }
    }
  }

  // Reset rounded corners
  > .@{inputClass}:first-child,
  &-addon:first-child {
    border-top-right-radius: 0;
    border-bottom-right-radius: 0;

    // Reset Select's style in addon
    .@{ant-prefix}-select .@{ant-prefix}-select-selector {
      border-top-right-radius: 0;
      border-bottom-right-radius: 0;
    }
  }

  > .@{inputClass}-affix-wrapper {
    &:not(:first-child) .@{inputClass} {
      border-top-left-radius: 0;
      border-bottom-left-radius: 0;
    }

    &:not(:last-child) .@{inputClass} {
      border-top-right-radius: 0;
      border-bottom-right-radius: 0;
    }
  }

  &-addon:first-child {
    border-right: 0;
  }

  &-addon:last-child {
    border-left: 0;
  }

  > .@{inputClass}:last-child,
  &-addon:last-child {
    border-top-left-radius: 0;
    border-bottom-left-radius: 0;

    // Reset Select's style in addon
    .@{ant-prefix}-select .@{ant-prefix}-select-selector {
      border-top-left-radius: 0;
      border-bottom-left-radius: 0;
    }
  }

  // Sizing options
  &-lg .@{inputClass},
  &-lg > &-addon {
    .input-lg();
  }

  &-sm .@{inputClass},
  &-sm > &-addon {
    .input-sm();
  }

  // Fix https://github.com/ant-design/ant-design/issues/5754
  &-lg .@{ant-prefix}-select-single .@{ant-prefix}-select-selector {
    height: @input-height-lg;
  }

  &-sm .@{ant-prefix}-select-single .@{ant-prefix}-select-selector {
    height: @input-height-sm;
  }

  .@{inputClass}-affix-wrapper {
    &:not(:last-child) {
      border-top-right-radius: 0;
      border-bottom-right-radius: 0;
      .@{ant-prefix}-input-search & {
        border-top-left-radius: @control-border-radius;
        border-bottom-left-radius: @control-border-radius;
      }
    }

    &:not(:first-child),
    .@{ant-prefix}-input-search &:not(:first-child) {
      border-top-left-radius: 0;
      border-bottom-left-radius: 0;
    }
  }

  &&-compact {
    display: block;
    .clearfix();

    &-addon,
    &-wrap,
    > .@{inputClass} {
      &:not(:first-child):not(:last-child) {
        border-right-width: @border-width-base;

        &:hover {
          z-index: 1;
        }

        &:focus {
          z-index: 1;
        }
      }
    }

    & > * {
      display: inline-block;
      float: none;
      vertical-align: top; // https://github.com/ant-design/ant-design-pro/issues/139
      border-radius: 0;
    }

    & > .@{inputClass}-affix-wrapper,
    & > .@{inputClass}-number-affix-wrapper,
    & > .@{ant-prefix}-picker-range {
      display: inline-flex;
    }

    & > *:not(:last-child) {
      margin-right: -@border-width-base;
      border-right-width: @border-width-base;
    }

    // Undo float for .ant-input-group .ant-input
    .@{inputClass} {
      float: none;
    }

    // reset border for Select, DatePicker, AutoComplete, Cascader, Mention, TimePicker, Input
    & > .@{ant-prefix}-select > .@{ant-prefix}-select-selector,
    & > .@{ant-prefix}-select-auto-complete .@{ant-prefix}-input,
    & > .@{ant-prefix}-cascader-picker .@{ant-prefix}-input,
    & > .@{ant-prefix}-input-group-wrapper .@{ant-prefix}-input {
      border-right-width: @border-width-base;
      border-radius: 0;

      &:hover {
        z-index: 1;
      }

      &:focus {
        z-index: 1;
      }
    }

    & > .@{ant-prefix}-select-focused {
      z-index: 1;
    }

    // update z-index for arrow icon
    & > .@{ant-prefix}-select > .@{ant-prefix}-select-arrow {
      z-index: 1; // https://github.com/ant-design/ant-design/issues/20371
    }

    & > *:first-child,
    & > .@{ant-prefix}-select:first-child > .@{ant-prefix}-select-selector,
    & > .@{ant-prefix}-select-auto-complete:first-child .@{ant-prefix}-input,
    & > .@{ant-prefix}-cascader-picker:first-child .@{ant-prefix}-input {
      border-top-left-radius: @control-border-radius;
      border-bottom-left-radius: @control-border-radius;
    }

    & > *:last-child,
    & > .@{ant-prefix}-select:last-child > .@{ant-prefix}-select-selector,
    & > .@{ant-prefix}-cascader-picker:last-child .@{ant-prefix}-input,
    & > .@{ant-prefix}-cascader-picker-focused:last-child .@{ant-prefix}-input {
      border-right-width: @border-width-base;
      border-top-right-radius: @control-border-radius;
      border-bottom-right-radius: @control-border-radius;
    }

    // https://github.com/ant-design/ant-design/issues/12493
    & > .@{ant-prefix}-select-auto-complete .@{ant-prefix}-input {
      vertical-align: top;
    }

    .@{ant-prefix}-input-group-wrapper + .@{ant-prefix}-input-group-wrapper {
      margin-left: -1px;
      .@{ant-prefix}-input-affix-wrapper {
        border-radius: 0;
      }
    }

    .@{ant-prefix}-input-group-wrapper:not(:last-child) {
      &.@{ant-prefix}-input-search > .@{ant-prefix}-input-group {
        & > .@{ant-prefix}-input-group-addon > .@{ant-prefix}-input-search-button {
          border-radius: 0;
        }

        & > .@{ant-prefix}-input {
          border-radius: @control-border-radius 0 0 @control-border-radius;
        }
      }
    }
  }
}

.input-group-wrapper-compact(@inputClass) {
  // Fix the issue of using icons in Space Compact mode
  // https://github.com/ant-design/ant-design/issues/42122
  &:not(.@{inputClass}-compact-first-item):not(
      .@{inputClass}-compact-last-item
    ).@{inputClass}-compact-item {
    .@{inputClass},
    .@{inputClass}-group-addon {
      border-radius: 0;
    }
  }

  &:not(.@{inputClass}-compact-last-item).@{inputClass}-compact-first-item {
    .@{inputClass},
    .@{inputClass}-group-addon {
      border-start-end-radius: 0;
      border-end-end-radius: 0;
    }
  }

  &:not(.@{inputClass}-compact-first-item).@{inputClass}-compact-last-item {
    .@{inputClass},
    .@{inputClass}-group-addon {
      border-start-start-radius: 0;
      border-end-start-radius: 0;
    }
  }

  // Fix the issue of input use show-count param in space compact mode
  // https://github.com/ant-design/ant-design/issues/46872
  &:not(.@{inputClass}-compact-last-item).@{inputClass}-compact-item {
    .@{inputClass}-affix-wrapper {
      border-start-end-radius: 0;
      border-end-end-radius: 0;
    }
  }

  // Fix the issue of input use `addonAfter` param in space compact mode
  // https://github.com/ant-design/ant-design/issues/52483
  &:not(.@{inputClass}-compact-first-item).@{inputClass}-compact-item {
    .@{inputClass}-affix-wrapper {
      border-start-start-radius: 0;
      border-end-start-radius: 0;
    }
  }
}

.status-color(
  @prefix-cls: @input-prefix-cls;
  @text-color: @input-color;
  @border-color: @input-border-color;
  @background-color: @input-bg;
  @hoverBorderColor: @primary-color-hover;
  @outlineColor: @primary-color-outline;
) {
  &:not(.@{prefix-cls}-disabled):not(.@{prefix-cls}-borderless).@{prefix-cls} {
    &,
    &:hover {
      background: @background-color;
      border-color: @border-color;
    }

    &:focus,
    &-focused {
      .active(@text-color, @hoverBorderColor, @outlineColor);
    }
  }
}

.status-color-common(
  @prefix-cls: @input-prefix-cls;
  @text-color: @input-color;
  @border-color: @input-border-color;
  @background-color: @input-bg;
  @hoverBorderColor: @primary-color-hover;
  @outlineColor: @primary-color-outline;
) {
  .@{prefix-cls}-prefix {
    color: @text-color;
  }
}

.group-status-color(
  @prefix-cls: @input-prefix-cls;
  @text-color: @input-color;
  @border-color: @input-border-color;
) {
  .@{prefix-cls}-group-addon {
    color: @text-color;
    border-color: @border-color;
  }
}

`````
