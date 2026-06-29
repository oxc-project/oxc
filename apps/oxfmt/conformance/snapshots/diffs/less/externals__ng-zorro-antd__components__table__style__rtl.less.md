# externals/ng-zorro-antd/components/table/style/rtl.less

> Allowed (layout-only): wrapped :not() selector-arg indent (prettier/prettier#16165)
> Allowed (layout-only): nested Less math — Prettier's fill fit-check breaks inside the wide chunk, ours breaks the separator (biome fill). See crates/oxc_formatter_css/AGENTS.md

## Option 1

`````json
{"printWidth":80}
`````

### Diff

`````diff
===================================================================
--- prettier
+++ oxfmt
@@ -32,10 +32,10 @@
           }
         }
 
         &:not(:last-child):not(.@{table-prefix-cls}-selection-column):not(
-            .@{table-prefix-cls}-row-expand-icon-cell
-          ):not([colspan])::before {
+          .@{table-prefix-cls}-row-expand-icon-cell
+        ):not([colspan])::before {
           .@{table-wrapepr-rtl-cls} & {
             right: auto;
             left: 0;
           }
@@ -54,11 +54,10 @@
       // ========================= Nest Table ===========================
       .@{table-prefix-cls}-wrapper:only-child {
         .@{table-prefix-cls}.@{table-prefix-cls}-rtl {
           margin: -@table-padding-vertical
-            (
-              @table-padding-horizontal + ceil(@font-size-sm * 1.4)
-            ) -@table-padding-vertical -@table-padding-horizontal;
+            (@table-padding-horizontal + ceil(@font-size-sm * 1.4))
+            -@table-padding-vertical -@table-padding-horizontal;
         }
       }
     }
   }

`````

### Actual (oxfmt)

`````less
@import "../../style/themes/index";
@import "../../style/mixins/index";

@table-prefix-cls: ~"@{ant-prefix}-table";
@table-wrapepr-cls: ~"@{table-prefix-cls}-wrapper";
@table-wrapepr-rtl-cls: ~"@{table-prefix-cls}-wrapper-rtl";

.@{table-prefix-cls}-wrapper {
  &-rtl {
    direction: rtl;
  }
}

.@{table-prefix-cls} {
  &-rtl {
    direction: rtl;
  }

  table {
    .@{table-wrapepr-rtl-cls} & {
      text-align: right;
    }
  }

  // ============================ Header ============================
  &-thead {
    > tr {
      > th {
        &[colspan]:not([colspan="1"]) {
          .@{table-wrapepr-rtl-cls} & {
            text-align: center;
          }
        }

        &:not(:last-child):not(.@{table-prefix-cls}-selection-column):not(
          .@{table-prefix-cls}-row-expand-icon-cell
        ):not([colspan])::before {
          .@{table-wrapepr-rtl-cls} & {
            right: auto;
            left: 0;
          }
        }

        .@{table-wrapepr-rtl-cls} & {
          text-align: right;
        }
      }
    }
  }

  // ============================= Body =============================
  &-tbody {
    > tr {
      // ========================= Nest Table ===========================
      .@{table-prefix-cls}-wrapper:only-child {
        .@{table-prefix-cls}.@{table-prefix-cls}-rtl {
          margin: -@table-padding-vertical
            (@table-padding-horizontal + ceil(@font-size-sm * 1.4))
            -@table-padding-vertical -@table-padding-horizontal;
        }
      }
    }
  }

  // ========================== Pagination ==========================
  &-pagination {
    &-left {
      .@{table-wrapepr-cls}.@{table-wrapepr-rtl-cls} & {
        justify-content: flex-end;
      }
    }

    &-right {
      .@{table-wrapepr-cls}.@{table-wrapepr-rtl-cls} & {
        justify-content: flex-start;
      }
    }
  }

  // ================================================================
  // =                           Function                           =
  // ================================================================

  // ============================ Sorter ============================
  &-column-sorter {
    .@{table-wrapepr-rtl-cls} & {
      margin-right: 4px;
      margin-left: 0;
    }
  }

  // ============================ Filter ============================
  &-filter-column-title {
    .@{table-wrapepr-rtl-cls} & {
      padding: @table-padding-vertical @table-padding-horizontal
        @table-padding-vertical 2.3em;
    }
  }

  &-thead tr th.@{table-prefix-cls}-column-has-sorters {
    .@{table-prefix-cls}-filter-column-title {
      .@{table-prefix-cls}-rtl & {
        padding: 0 0 0 2.3em;
      }
    }
  }

  &-filter-trigger {
    .@{table-wrapepr-rtl-cls} & {
      margin: -4px 4px -4px (-@table-padding-horizontal / 2);
    }
  }

  // Dropdown
  &-filter-dropdown {
    // Checkbox
    &,
    &-submenu {
      .@{ant-prefix}-checkbox-wrapper + span {
        .@{ant-prefix}-dropdown-rtl &,
        .@{ant-prefix}-dropdown-menu-submenu-rtl& {
          padding-right: 8px;
          padding-left: 0;
        }
      }
    }
  }

  // ========================== Selections ==========================
  &-selection {
    .@{table-wrapepr-rtl-cls} & {
      text-align: center;
    }
  }

  // ========================== Expandable ==========================
  &-row-indent {
    .@{table-wrapepr-rtl-cls} & {
      float: right;
    }
  }

  &-row-expand-icon {
    .@{table-wrapepr-rtl-cls} & {
      float: right;
    }

    .@{table-prefix-cls}-row-indent + & {
      .@{table-wrapepr-rtl-cls} & {
        margin-right: 0;
        margin-left: @padding-xs;
      }
    }

    &::after {
      .@{table-wrapepr-rtl-cls} & {
        transform: rotate(-90deg);
      }
    }

    &-collapsed::before {
      .@{table-wrapepr-rtl-cls} & {
        transform: rotate(180deg);
      }
    }

    &-collapsed::after {
      .@{table-wrapepr-rtl-cls} & {
        transform: rotate(0deg);
      }
    }
  }
}

`````

### Expected (prettier)

`````less
@import "../../style/themes/index";
@import "../../style/mixins/index";

@table-prefix-cls: ~"@{ant-prefix}-table";
@table-wrapepr-cls: ~"@{table-prefix-cls}-wrapper";
@table-wrapepr-rtl-cls: ~"@{table-prefix-cls}-wrapper-rtl";

.@{table-prefix-cls}-wrapper {
  &-rtl {
    direction: rtl;
  }
}

.@{table-prefix-cls} {
  &-rtl {
    direction: rtl;
  }

  table {
    .@{table-wrapepr-rtl-cls} & {
      text-align: right;
    }
  }

  // ============================ Header ============================
  &-thead {
    > tr {
      > th {
        &[colspan]:not([colspan="1"]) {
          .@{table-wrapepr-rtl-cls} & {
            text-align: center;
          }
        }

        &:not(:last-child):not(.@{table-prefix-cls}-selection-column):not(
            .@{table-prefix-cls}-row-expand-icon-cell
          ):not([colspan])::before {
          .@{table-wrapepr-rtl-cls} & {
            right: auto;
            left: 0;
          }
        }

        .@{table-wrapepr-rtl-cls} & {
          text-align: right;
        }
      }
    }
  }

  // ============================= Body =============================
  &-tbody {
    > tr {
      // ========================= Nest Table ===========================
      .@{table-prefix-cls}-wrapper:only-child {
        .@{table-prefix-cls}.@{table-prefix-cls}-rtl {
          margin: -@table-padding-vertical
            (
              @table-padding-horizontal + ceil(@font-size-sm * 1.4)
            ) -@table-padding-vertical -@table-padding-horizontal;
        }
      }
    }
  }

  // ========================== Pagination ==========================
  &-pagination {
    &-left {
      .@{table-wrapepr-cls}.@{table-wrapepr-rtl-cls} & {
        justify-content: flex-end;
      }
    }

    &-right {
      .@{table-wrapepr-cls}.@{table-wrapepr-rtl-cls} & {
        justify-content: flex-start;
      }
    }
  }

  // ================================================================
  // =                           Function                           =
  // ================================================================

  // ============================ Sorter ============================
  &-column-sorter {
    .@{table-wrapepr-rtl-cls} & {
      margin-right: 4px;
      margin-left: 0;
    }
  }

  // ============================ Filter ============================
  &-filter-column-title {
    .@{table-wrapepr-rtl-cls} & {
      padding: @table-padding-vertical @table-padding-horizontal
        @table-padding-vertical 2.3em;
    }
  }

  &-thead tr th.@{table-prefix-cls}-column-has-sorters {
    .@{table-prefix-cls}-filter-column-title {
      .@{table-prefix-cls}-rtl & {
        padding: 0 0 0 2.3em;
      }
    }
  }

  &-filter-trigger {
    .@{table-wrapepr-rtl-cls} & {
      margin: -4px 4px -4px (-@table-padding-horizontal / 2);
    }
  }

  // Dropdown
  &-filter-dropdown {
    // Checkbox
    &,
    &-submenu {
      .@{ant-prefix}-checkbox-wrapper + span {
        .@{ant-prefix}-dropdown-rtl &,
        .@{ant-prefix}-dropdown-menu-submenu-rtl& {
          padding-right: 8px;
          padding-left: 0;
        }
      }
    }
  }

  // ========================== Selections ==========================
  &-selection {
    .@{table-wrapepr-rtl-cls} & {
      text-align: center;
    }
  }

  // ========================== Expandable ==========================
  &-row-indent {
    .@{table-wrapepr-rtl-cls} & {
      float: right;
    }
  }

  &-row-expand-icon {
    .@{table-wrapepr-rtl-cls} & {
      float: right;
    }

    .@{table-prefix-cls}-row-indent + & {
      .@{table-wrapepr-rtl-cls} & {
        margin-right: 0;
        margin-left: @padding-xs;
      }
    }

    &::after {
      .@{table-wrapepr-rtl-cls} & {
        transform: rotate(-90deg);
      }
    }

    &-collapsed::before {
      .@{table-wrapepr-rtl-cls} & {
        transform: rotate(180deg);
      }
    }

    &-collapsed::after {
      .@{table-wrapepr-rtl-cls} & {
        transform: rotate(0deg);
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
@@ -32,10 +32,10 @@
           }
         }
 
         &:not(:last-child):not(.@{table-prefix-cls}-selection-column):not(
-            .@{table-prefix-cls}-row-expand-icon-cell
-          ):not([colspan])::before {
+          .@{table-prefix-cls}-row-expand-icon-cell
+        ):not([colspan])::before {
           .@{table-wrapepr-rtl-cls} & {
             right: auto;
             left: 0;
           }
@@ -53,12 +53,10 @@
     > tr {
       // ========================= Nest Table ===========================
       .@{table-prefix-cls}-wrapper:only-child {
         .@{table-prefix-cls}.@{table-prefix-cls}-rtl {
-          margin: -@table-padding-vertical
-            (
-              @table-padding-horizontal + ceil(@font-size-sm * 1.4)
-            ) -@table-padding-vertical -@table-padding-horizontal;
+          margin: -@table-padding-vertical (@table-padding-horizontal + ceil(@font-size-sm * 1.4))
+            -@table-padding-vertical -@table-padding-horizontal;
         }
       }
     }
   }

`````

### Actual (oxfmt)

`````less
@import "../../style/themes/index";
@import "../../style/mixins/index";

@table-prefix-cls: ~"@{ant-prefix}-table";
@table-wrapepr-cls: ~"@{table-prefix-cls}-wrapper";
@table-wrapepr-rtl-cls: ~"@{table-prefix-cls}-wrapper-rtl";

.@{table-prefix-cls}-wrapper {
  &-rtl {
    direction: rtl;
  }
}

.@{table-prefix-cls} {
  &-rtl {
    direction: rtl;
  }

  table {
    .@{table-wrapepr-rtl-cls} & {
      text-align: right;
    }
  }

  // ============================ Header ============================
  &-thead {
    > tr {
      > th {
        &[colspan]:not([colspan="1"]) {
          .@{table-wrapepr-rtl-cls} & {
            text-align: center;
          }
        }

        &:not(:last-child):not(.@{table-prefix-cls}-selection-column):not(
          .@{table-prefix-cls}-row-expand-icon-cell
        ):not([colspan])::before {
          .@{table-wrapepr-rtl-cls} & {
            right: auto;
            left: 0;
          }
        }

        .@{table-wrapepr-rtl-cls} & {
          text-align: right;
        }
      }
    }
  }

  // ============================= Body =============================
  &-tbody {
    > tr {
      // ========================= Nest Table ===========================
      .@{table-prefix-cls}-wrapper:only-child {
        .@{table-prefix-cls}.@{table-prefix-cls}-rtl {
          margin: -@table-padding-vertical (@table-padding-horizontal + ceil(@font-size-sm * 1.4))
            -@table-padding-vertical -@table-padding-horizontal;
        }
      }
    }
  }

  // ========================== Pagination ==========================
  &-pagination {
    &-left {
      .@{table-wrapepr-cls}.@{table-wrapepr-rtl-cls} & {
        justify-content: flex-end;
      }
    }

    &-right {
      .@{table-wrapepr-cls}.@{table-wrapepr-rtl-cls} & {
        justify-content: flex-start;
      }
    }
  }

  // ================================================================
  // =                           Function                           =
  // ================================================================

  // ============================ Sorter ============================
  &-column-sorter {
    .@{table-wrapepr-rtl-cls} & {
      margin-right: 4px;
      margin-left: 0;
    }
  }

  // ============================ Filter ============================
  &-filter-column-title {
    .@{table-wrapepr-rtl-cls} & {
      padding: @table-padding-vertical @table-padding-horizontal @table-padding-vertical 2.3em;
    }
  }

  &-thead tr th.@{table-prefix-cls}-column-has-sorters {
    .@{table-prefix-cls}-filter-column-title {
      .@{table-prefix-cls}-rtl & {
        padding: 0 0 0 2.3em;
      }
    }
  }

  &-filter-trigger {
    .@{table-wrapepr-rtl-cls} & {
      margin: -4px 4px -4px (-@table-padding-horizontal / 2);
    }
  }

  // Dropdown
  &-filter-dropdown {
    // Checkbox
    &,
    &-submenu {
      .@{ant-prefix}-checkbox-wrapper + span {
        .@{ant-prefix}-dropdown-rtl &,
        .@{ant-prefix}-dropdown-menu-submenu-rtl& {
          padding-right: 8px;
          padding-left: 0;
        }
      }
    }
  }

  // ========================== Selections ==========================
  &-selection {
    .@{table-wrapepr-rtl-cls} & {
      text-align: center;
    }
  }

  // ========================== Expandable ==========================
  &-row-indent {
    .@{table-wrapepr-rtl-cls} & {
      float: right;
    }
  }

  &-row-expand-icon {
    .@{table-wrapepr-rtl-cls} & {
      float: right;
    }

    .@{table-prefix-cls}-row-indent + & {
      .@{table-wrapepr-rtl-cls} & {
        margin-right: 0;
        margin-left: @padding-xs;
      }
    }

    &::after {
      .@{table-wrapepr-rtl-cls} & {
        transform: rotate(-90deg);
      }
    }

    &-collapsed::before {
      .@{table-wrapepr-rtl-cls} & {
        transform: rotate(180deg);
      }
    }

    &-collapsed::after {
      .@{table-wrapepr-rtl-cls} & {
        transform: rotate(0deg);
      }
    }
  }
}

`````

### Expected (prettier)

`````less
@import "../../style/themes/index";
@import "../../style/mixins/index";

@table-prefix-cls: ~"@{ant-prefix}-table";
@table-wrapepr-cls: ~"@{table-prefix-cls}-wrapper";
@table-wrapepr-rtl-cls: ~"@{table-prefix-cls}-wrapper-rtl";

.@{table-prefix-cls}-wrapper {
  &-rtl {
    direction: rtl;
  }
}

.@{table-prefix-cls} {
  &-rtl {
    direction: rtl;
  }

  table {
    .@{table-wrapepr-rtl-cls} & {
      text-align: right;
    }
  }

  // ============================ Header ============================
  &-thead {
    > tr {
      > th {
        &[colspan]:not([colspan="1"]) {
          .@{table-wrapepr-rtl-cls} & {
            text-align: center;
          }
        }

        &:not(:last-child):not(.@{table-prefix-cls}-selection-column):not(
            .@{table-prefix-cls}-row-expand-icon-cell
          ):not([colspan])::before {
          .@{table-wrapepr-rtl-cls} & {
            right: auto;
            left: 0;
          }
        }

        .@{table-wrapepr-rtl-cls} & {
          text-align: right;
        }
      }
    }
  }

  // ============================= Body =============================
  &-tbody {
    > tr {
      // ========================= Nest Table ===========================
      .@{table-prefix-cls}-wrapper:only-child {
        .@{table-prefix-cls}.@{table-prefix-cls}-rtl {
          margin: -@table-padding-vertical
            (
              @table-padding-horizontal + ceil(@font-size-sm * 1.4)
            ) -@table-padding-vertical -@table-padding-horizontal;
        }
      }
    }
  }

  // ========================== Pagination ==========================
  &-pagination {
    &-left {
      .@{table-wrapepr-cls}.@{table-wrapepr-rtl-cls} & {
        justify-content: flex-end;
      }
    }

    &-right {
      .@{table-wrapepr-cls}.@{table-wrapepr-rtl-cls} & {
        justify-content: flex-start;
      }
    }
  }

  // ================================================================
  // =                           Function                           =
  // ================================================================

  // ============================ Sorter ============================
  &-column-sorter {
    .@{table-wrapepr-rtl-cls} & {
      margin-right: 4px;
      margin-left: 0;
    }
  }

  // ============================ Filter ============================
  &-filter-column-title {
    .@{table-wrapepr-rtl-cls} & {
      padding: @table-padding-vertical @table-padding-horizontal @table-padding-vertical 2.3em;
    }
  }

  &-thead tr th.@{table-prefix-cls}-column-has-sorters {
    .@{table-prefix-cls}-filter-column-title {
      .@{table-prefix-cls}-rtl & {
        padding: 0 0 0 2.3em;
      }
    }
  }

  &-filter-trigger {
    .@{table-wrapepr-rtl-cls} & {
      margin: -4px 4px -4px (-@table-padding-horizontal / 2);
    }
  }

  // Dropdown
  &-filter-dropdown {
    // Checkbox
    &,
    &-submenu {
      .@{ant-prefix}-checkbox-wrapper + span {
        .@{ant-prefix}-dropdown-rtl &,
        .@{ant-prefix}-dropdown-menu-submenu-rtl& {
          padding-right: 8px;
          padding-left: 0;
        }
      }
    }
  }

  // ========================== Selections ==========================
  &-selection {
    .@{table-wrapepr-rtl-cls} & {
      text-align: center;
    }
  }

  // ========================== Expandable ==========================
  &-row-indent {
    .@{table-wrapepr-rtl-cls} & {
      float: right;
    }
  }

  &-row-expand-icon {
    .@{table-wrapepr-rtl-cls} & {
      float: right;
    }

    .@{table-prefix-cls}-row-indent + & {
      .@{table-wrapepr-rtl-cls} & {
        margin-right: 0;
        margin-left: @padding-xs;
      }
    }

    &::after {
      .@{table-wrapepr-rtl-cls} & {
        transform: rotate(-90deg);
      }
    }

    &-collapsed::before {
      .@{table-wrapepr-rtl-cls} & {
        transform: rotate(180deg);
      }
    }

    &-collapsed::after {
      .@{table-wrapepr-rtl-cls} & {
        transform: rotate(0deg);
      }
    }
  }
}

`````
