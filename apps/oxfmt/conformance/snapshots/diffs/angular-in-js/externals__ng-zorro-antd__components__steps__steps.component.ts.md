# externals/ng-zorro-antd/components/steps/steps.component.ts

## Option 1

`````json
{"printWidth":80}
`````

### Diff

`````diff
===================================================================
--- prettier
+++ oxfmt
@@ -60,9 +60,12 @@
   },
 })
 export class NzStepsComponent implements OnChanges, OnInit, AfterContentInit {
   static ngAcceptInputType_nzProgressDot:
-    BooleanInput | NzProgressDotTemplate | undefined | null;
+    | BooleanInput
+    | NzProgressDotTemplate
+    | undefined
+    | null;
 
   private cdr = inject(ChangeDetectorRef);
   private ngZone = inject(NgZone);
   private directionality = inject(Directionality);

`````

### Actual (oxfmt)

`````ts
/**
 * Use of this source code is governed by an MIT-style license that can be
 * found in the LICENSE file at https://github.com/NG-ZORRO/ng-zorro-antd/blob/master/LICENSE
 */

import { Direction, Directionality } from "@angular/cdk/bidi";
import {
  AfterContentInit,
  ChangeDetectionStrategy,
  ChangeDetectorRef,
  Component,
  ContentChildren,
  DestroyRef,
  EventEmitter,
  inject,
  Input,
  NgZone,
  OnChanges,
  OnInit,
  Output,
  QueryList,
  SimpleChanges,
  TemplateRef,
  ViewEncapsulation,
} from "@angular/core";
import { takeUntilDestroyed } from "@angular/core/rxjs-interop";
import { Subscription, merge } from "rxjs";
import { startWith } from "rxjs/operators";

import { BooleanInput, NzSizeDSType } from "ng-zorro-antd/core/types";
import { toBoolean } from "ng-zorro-antd/core/util";

import { NzStepComponent } from "./step.component";

export type NzDirectionType = "horizontal" | "vertical";
export type NzStatusType = "wait" | "process" | "finish" | "error";
export type NzProgressDotTemplate = TemplateRef<{
  $implicit: TemplateRef<void>;
  status: string;
  index: number;
}>;

@Component({
  changeDetection: ChangeDetectionStrategy.OnPush,
  encapsulation: ViewEncapsulation.None,
  selector: "nz-steps",
  exportAs: "nzSteps",
  template: `<ng-content />`,
  host: {
    class: "ant-steps",
    "[class.ant-steps-horizontal]": `nzDirection === 'horizontal'`,
    "[class.ant-steps-vertical]": `nzDirection === 'vertical'`,
    "[class.ant-steps-label-horizontal]": `nzDirection === 'horizontal'`,
    "[class.ant-steps-label-vertical]": `(showProcessDot || nzLabelPlacement === 'vertical') && nzDirection === 'horizontal'`,
    "[class.ant-steps-dot]": "showProcessDot",
    "[class.ant-steps-small]": `nzSize === 'small'`,
    "[class.ant-steps-navigation]": `nzType === 'navigation'`,
    "[class.ant-steps-rtl]": `dir === 'rtl'`,
    "[class.ant-steps-with-progress]": "showProgress",
  },
})
export class NzStepsComponent implements OnChanges, OnInit, AfterContentInit {
  static ngAcceptInputType_nzProgressDot:
    | BooleanInput
    | NzProgressDotTemplate
    | undefined
    | null;

  private cdr = inject(ChangeDetectorRef);
  private ngZone = inject(NgZone);
  private directionality = inject(Directionality);
  private destroyRef = inject(DestroyRef);

  @ContentChildren(NzStepComponent) steps!: QueryList<NzStepComponent>;

  @Input() nzCurrent = 0;
  @Input() nzDirection: NzDirectionType = "horizontal";
  @Input() nzLabelPlacement: "horizontal" | "vertical" = "horizontal";
  @Input() nzType: "default" | "navigation" = "default";
  @Input() nzSize: NzSizeDSType = "default";
  @Input() nzStartIndex = 0;
  @Input() nzStatus: NzStatusType = "process";

  @Input()
  set nzProgressDot(value: boolean | NzProgressDotTemplate | undefined | null) {
    if (value instanceof TemplateRef) {
      this.showProcessDot = true;
      this.customProcessDotTemplate = value;
    } else {
      this.showProcessDot = toBoolean(value);
    }
    this.updateChildrenSteps();
  }

  @Output() readonly nzIndexChange = new EventEmitter<number>();

  private indexChangeSubscription = Subscription.EMPTY;

  showProcessDot = false;
  showProgress = false;
  customProcessDotTemplate?: TemplateRef<{
    $implicit: TemplateRef<void>;
    status: string;
    index: number;
  }>;
  dir: Direction = "ltr";

  ngOnChanges(changes: SimpleChanges): void {
    const { nzStartIndex, nzDirection, nzStatus, nzCurrent, nzSize } = changes;
    if (nzStartIndex || nzDirection || nzStatus || nzCurrent || nzSize) {
      this.updateChildrenSteps();
    }
  }

  ngOnInit(): void {
    this.directionality.change
      ?.pipe(takeUntilDestroyed(this.destroyRef))
      .subscribe((direction) => {
        this.dir = direction;
        this.cdr.detectChanges();
      });

    this.dir = this.directionality.value;
    this.updateChildrenSteps();
  }

  ngAfterContentInit(): void {
    if (this.steps) {
      this.steps.changes
        .pipe(startWith(null), takeUntilDestroyed(this.destroyRef))
        .subscribe(() => {
          this.updateHostProgressClass();
          this.updateChildrenSteps();
        });
    }
  }

  private updateHostProgressClass(): void {
    if (this.steps && !this.showProcessDot) {
      this.showProgress = !!this.steps
        .toArray()
        .find((step) => step.nzPercentage !== null);
    }
  }

  private updateChildrenSteps(): void {
    if (this.steps) {
      const length = this.steps.length;
      this.steps.toArray().forEach((step, index) => {
        Promise.resolve().then(() => {
          step.nzSize = this.nzSize;
          step.outStatus = this.nzStatus;
          step.showProcessDot = this.showProcessDot;
          if (this.customProcessDotTemplate) {
            step.customProcessTemplate = this.customProcessDotTemplate;
          }
          step.clickable = this.nzIndexChange.observers.length > 0;
          step.direction = this.nzDirection;
          step.index = index + this.nzStartIndex;
          step.currentIndex = this.nzCurrent;
          step.last = length === index + 1;
          step.markForCheck();
        });
      });
      this.indexChangeSubscription.unsubscribe();
      this.indexChangeSubscription = merge(
        ...this.steps.map((step) => step.clickOutsideAngular$),
      )
        .pipe(takeUntilDestroyed(this.destroyRef))
        .subscribe((index) => {
          if (this.nzIndexChange.observers.length) {
            this.ngZone.run(() => this.nzIndexChange.emit(index));
          }
        });
    }
  }
}

`````

### Expected (prettier)

`````ts
/**
 * Use of this source code is governed by an MIT-style license that can be
 * found in the LICENSE file at https://github.com/NG-ZORRO/ng-zorro-antd/blob/master/LICENSE
 */

import { Direction, Directionality } from "@angular/cdk/bidi";
import {
  AfterContentInit,
  ChangeDetectionStrategy,
  ChangeDetectorRef,
  Component,
  ContentChildren,
  DestroyRef,
  EventEmitter,
  inject,
  Input,
  NgZone,
  OnChanges,
  OnInit,
  Output,
  QueryList,
  SimpleChanges,
  TemplateRef,
  ViewEncapsulation,
} from "@angular/core";
import { takeUntilDestroyed } from "@angular/core/rxjs-interop";
import { Subscription, merge } from "rxjs";
import { startWith } from "rxjs/operators";

import { BooleanInput, NzSizeDSType } from "ng-zorro-antd/core/types";
import { toBoolean } from "ng-zorro-antd/core/util";

import { NzStepComponent } from "./step.component";

export type NzDirectionType = "horizontal" | "vertical";
export type NzStatusType = "wait" | "process" | "finish" | "error";
export type NzProgressDotTemplate = TemplateRef<{
  $implicit: TemplateRef<void>;
  status: string;
  index: number;
}>;

@Component({
  changeDetection: ChangeDetectionStrategy.OnPush,
  encapsulation: ViewEncapsulation.None,
  selector: "nz-steps",
  exportAs: "nzSteps",
  template: `<ng-content />`,
  host: {
    class: "ant-steps",
    "[class.ant-steps-horizontal]": `nzDirection === 'horizontal'`,
    "[class.ant-steps-vertical]": `nzDirection === 'vertical'`,
    "[class.ant-steps-label-horizontal]": `nzDirection === 'horizontal'`,
    "[class.ant-steps-label-vertical]": `(showProcessDot || nzLabelPlacement === 'vertical') && nzDirection === 'horizontal'`,
    "[class.ant-steps-dot]": "showProcessDot",
    "[class.ant-steps-small]": `nzSize === 'small'`,
    "[class.ant-steps-navigation]": `nzType === 'navigation'`,
    "[class.ant-steps-rtl]": `dir === 'rtl'`,
    "[class.ant-steps-with-progress]": "showProgress",
  },
})
export class NzStepsComponent implements OnChanges, OnInit, AfterContentInit {
  static ngAcceptInputType_nzProgressDot:
    BooleanInput | NzProgressDotTemplate | undefined | null;

  private cdr = inject(ChangeDetectorRef);
  private ngZone = inject(NgZone);
  private directionality = inject(Directionality);
  private destroyRef = inject(DestroyRef);

  @ContentChildren(NzStepComponent) steps!: QueryList<NzStepComponent>;

  @Input() nzCurrent = 0;
  @Input() nzDirection: NzDirectionType = "horizontal";
  @Input() nzLabelPlacement: "horizontal" | "vertical" = "horizontal";
  @Input() nzType: "default" | "navigation" = "default";
  @Input() nzSize: NzSizeDSType = "default";
  @Input() nzStartIndex = 0;
  @Input() nzStatus: NzStatusType = "process";

  @Input()
  set nzProgressDot(value: boolean | NzProgressDotTemplate | undefined | null) {
    if (value instanceof TemplateRef) {
      this.showProcessDot = true;
      this.customProcessDotTemplate = value;
    } else {
      this.showProcessDot = toBoolean(value);
    }
    this.updateChildrenSteps();
  }

  @Output() readonly nzIndexChange = new EventEmitter<number>();

  private indexChangeSubscription = Subscription.EMPTY;

  showProcessDot = false;
  showProgress = false;
  customProcessDotTemplate?: TemplateRef<{
    $implicit: TemplateRef<void>;
    status: string;
    index: number;
  }>;
  dir: Direction = "ltr";

  ngOnChanges(changes: SimpleChanges): void {
    const { nzStartIndex, nzDirection, nzStatus, nzCurrent, nzSize } = changes;
    if (nzStartIndex || nzDirection || nzStatus || nzCurrent || nzSize) {
      this.updateChildrenSteps();
    }
  }

  ngOnInit(): void {
    this.directionality.change
      ?.pipe(takeUntilDestroyed(this.destroyRef))
      .subscribe((direction) => {
        this.dir = direction;
        this.cdr.detectChanges();
      });

    this.dir = this.directionality.value;
    this.updateChildrenSteps();
  }

  ngAfterContentInit(): void {
    if (this.steps) {
      this.steps.changes
        .pipe(startWith(null), takeUntilDestroyed(this.destroyRef))
        .subscribe(() => {
          this.updateHostProgressClass();
          this.updateChildrenSteps();
        });
    }
  }

  private updateHostProgressClass(): void {
    if (this.steps && !this.showProcessDot) {
      this.showProgress = !!this.steps
        .toArray()
        .find((step) => step.nzPercentage !== null);
    }
  }

  private updateChildrenSteps(): void {
    if (this.steps) {
      const length = this.steps.length;
      this.steps.toArray().forEach((step, index) => {
        Promise.resolve().then(() => {
          step.nzSize = this.nzSize;
          step.outStatus = this.nzStatus;
          step.showProcessDot = this.showProcessDot;
          if (this.customProcessDotTemplate) {
            step.customProcessTemplate = this.customProcessDotTemplate;
          }
          step.clickable = this.nzIndexChange.observers.length > 0;
          step.direction = this.nzDirection;
          step.index = index + this.nzStartIndex;
          step.currentIndex = this.nzCurrent;
          step.last = length === index + 1;
          step.markForCheck();
        });
      });
      this.indexChangeSubscription.unsubscribe();
      this.indexChangeSubscription = merge(
        ...this.steps.map((step) => step.clickOutsideAngular$),
      )
        .pipe(takeUntilDestroyed(this.destroyRef))
        .subscribe((index) => {
          if (this.nzIndexChange.observers.length) {
            this.ngZone.run(() => this.nzIndexChange.emit(index));
          }
        });
    }
  }
}

`````
