# externals/ng-zorro-antd/components/form/form-item.component.ts

## Option 1

`````json
{"printWidth":80}
`````

### Diff

`````diff
===================================================================
--- prettier
+++ oxfmt
@@ -14,9 +14,13 @@
 
 import type { NzFormItemLayout } from "./types";
 
 export type NzFormControlStatusType =
-  "success" | "error" | "warning" | "validating" | "";
+  | "success"
+  | "error"
+  | "warning"
+  | "validating"
+  | "";
 
 /** should add nz-row directive to host, track https://github.com/angular/angular/issues/8785 **/
 @Component({
   selector: "nz-form-item",

`````

### Actual (oxfmt)

`````ts
/**
 * Use of this source code is governed by an MIT-style license that can be
 * found in the LICENSE file at https://github.com/NG-ZORRO/ng-zorro-antd/blob/master/LICENSE
 */

import {
  ChangeDetectionStrategy,
  ChangeDetectorRef,
  Component,
  inject,
  input,
  ViewEncapsulation,
} from "@angular/core";

import type { NzFormItemLayout } from "./types";

export type NzFormControlStatusType =
  | "success"
  | "error"
  | "warning"
  | "validating"
  | "";

/** should add nz-row directive to host, track https://github.com/angular/angular/issues/8785 **/
@Component({
  selector: "nz-form-item",
  exportAs: "nzFormItem",
  changeDetection: ChangeDetectionStrategy.OnPush,
  encapsulation: ViewEncapsulation.None,
  host: {
    class: "ant-form-item",
    "[class.ant-form-item-has-success]": 'status === "success"',
    "[class.ant-form-item-has-warning]": 'status === "warning"',
    "[class.ant-form-item-has-error]": 'status === "error"',
    "[class.ant-form-item-is-validating]": 'status === "validating"',
    "[class.ant-form-item-has-feedback]": "hasFeedback && status",
    "[class.ant-form-item-with-help]": "withHelpClass",
    "[class.ant-form-item-horizontal]": 'nzLayout() === "horizontal"',
    "[class.ant-form-item-vertical]": 'nzLayout() === "vertical"',
  },
  template: `<ng-content />`,
})
export class NzFormItemComponent {
  private cdr = inject(ChangeDetectorRef);

  readonly nzLayout = input<NzFormItemLayout>();

  status: NzFormControlStatusType = "";
  hasFeedback = false;
  withHelpClass = false;

  setWithHelpViaTips(value: boolean): void {
    this.withHelpClass = value;
    this.cdr.markForCheck();
  }

  setStatus(status: NzFormControlStatusType): void {
    this.status = status;
    this.cdr.markForCheck();
  }

  setHasFeedback(hasFeedback: boolean): void {
    this.hasFeedback = hasFeedback;
    this.cdr.markForCheck();
  }
}

`````

### Expected (prettier)

`````ts
/**
 * Use of this source code is governed by an MIT-style license that can be
 * found in the LICENSE file at https://github.com/NG-ZORRO/ng-zorro-antd/blob/master/LICENSE
 */

import {
  ChangeDetectionStrategy,
  ChangeDetectorRef,
  Component,
  inject,
  input,
  ViewEncapsulation,
} from "@angular/core";

import type { NzFormItemLayout } from "./types";

export type NzFormControlStatusType =
  "success" | "error" | "warning" | "validating" | "";

/** should add nz-row directive to host, track https://github.com/angular/angular/issues/8785 **/
@Component({
  selector: "nz-form-item",
  exportAs: "nzFormItem",
  changeDetection: ChangeDetectionStrategy.OnPush,
  encapsulation: ViewEncapsulation.None,
  host: {
    class: "ant-form-item",
    "[class.ant-form-item-has-success]": 'status === "success"',
    "[class.ant-form-item-has-warning]": 'status === "warning"',
    "[class.ant-form-item-has-error]": 'status === "error"',
    "[class.ant-form-item-is-validating]": 'status === "validating"',
    "[class.ant-form-item-has-feedback]": "hasFeedback && status",
    "[class.ant-form-item-with-help]": "withHelpClass",
    "[class.ant-form-item-horizontal]": 'nzLayout() === "horizontal"',
    "[class.ant-form-item-vertical]": 'nzLayout() === "vertical"',
  },
  template: `<ng-content />`,
})
export class NzFormItemComponent {
  private cdr = inject(ChangeDetectorRef);

  readonly nzLayout = input<NzFormItemLayout>();

  status: NzFormControlStatusType = "";
  hasFeedback = false;
  withHelpClass = false;

  setWithHelpViaTips(value: boolean): void {
    this.withHelpClass = value;
    this.cdr.markForCheck();
  }

  setStatus(status: NzFormControlStatusType): void {
    this.status = status;
    this.cdr.markForCheck();
  }

  setHasFeedback(hasFeedback: boolean): void {
    this.hasFeedback = hasFeedback;
    this.cdr.markForCheck();
  }
}

`````
