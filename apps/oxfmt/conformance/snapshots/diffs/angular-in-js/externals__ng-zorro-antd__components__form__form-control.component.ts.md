# externals/ng-zorro-antd/components/form/form-control.component.ts

## Option 1

`````json
{"printWidth":80}
`````

### Diff

`````diff
===================================================================
--- prettier
+++ oxfmt
@@ -128,21 +128,27 @@
 
   status: NzFormControlStatusType = "";
   validateControl: AbstractControl | NgModel | null = null;
   innerTip:
-    string | TemplateRef<{ $implicit: AbstractControl | NgModel }> | null =
-    null;
+    | string
+    | TemplateRef<{ $implicit: AbstractControl | NgModel }>
+    | null = null;
 
   @ContentChild(NgControl, { static: false }) defaultValidateControl?:
-    FormControlName | FormControlDirective;
+    | FormControlName
+    | FormControlDirective;
   @Input() nzSuccessTip?:
-    string | TemplateRef<{ $implicit: AbstractControl | NgModel }>;
+    | string
+    | TemplateRef<{ $implicit: AbstractControl | NgModel }>;
   @Input() nzWarningTip?:
-    string | TemplateRef<{ $implicit: AbstractControl | NgModel }>;
+    | string
+    | TemplateRef<{ $implicit: AbstractControl | NgModel }>;
   @Input() nzErrorTip?:
-    string | TemplateRef<{ $implicit: AbstractControl | NgModel }>;
+    | string
+    | TemplateRef<{ $implicit: AbstractControl | NgModel }>;
   @Input() nzValidatingTip?:
-    string | TemplateRef<{ $implicit: AbstractControl | NgModel }>;
+    | string
+    | TemplateRef<{ $implicit: AbstractControl | NgModel }>;
   @Input() nzExtra?: string | TemplateRef<void>;
   @Input() nzAutoTips: Record<string, Record<string, string>> = {};
   @Input({ transform: booleanAttribute }) nzDisableAutoTips?: boolean;
 

`````

### Actual (oxfmt)

`````ts
/**
 * Use of this source code is governed by an MIT-style license that can be
 * found in the LICENSE file at https://github.com/NG-ZORRO/ng-zorro-antd/blob/master/LICENSE
 */

import {
  AfterContentInit,
  AnimationCallbackEvent,
  booleanAttribute,
  ChangeDetectionStrategy,
  ChangeDetectorRef,
  Component,
  ContentChild,
  DestroyRef,
  inject,
  Input,
  OnChanges,
  OnInit,
  SimpleChanges,
  TemplateRef,
  ViewEncapsulation,
} from "@angular/core";
import { takeUntilDestroyed } from "@angular/core/rxjs-interop";
import {
  AbstractControl,
  FormControlDirective,
  FormControlName,
  NgControl,
  NgModel,
} from "@angular/forms";
import { Observable, Subscription } from "rxjs";
import { filter, startWith, tap } from "rxjs/operators";

import {
  isAnimationEnabled,
  NzNoAnimationDirective,
  withAnimationCheck,
} from "ng-zorro-antd/core/animation";
import { NzFormStatusService } from "ng-zorro-antd/core/form";
import { NzOutletModule } from "ng-zorro-antd/core/outlet";
import { NzSafeAny } from "ng-zorro-antd/core/types";
import { toBoolean } from "ng-zorro-antd/core/util";
import { NzI18nService } from "ng-zorro-antd/i18n";

import {
  NzFormControlStatusType,
  NzFormItemComponent,
} from "./form-item.component";
import { NzFormDirective } from "./form.directive";

@Component({
  selector: "nz-form-control",
  exportAs: "nzFormControl",
  encapsulation: ViewEncapsulation.None,
  changeDetection: ChangeDetectionStrategy.OnPush,
  template: `
    <div class="ant-form-item-control-input">
      <div class="ant-form-item-control-input-content">
        <ng-content />
      </div>
    </div>
    @if (innerTip) {
      <div
        [animate.enter]="nzValidateAnimationEnter()"
        [animate.leave]="nzValidateAnimationLeave()"
        (animate.leave)="onAnimationLeave($event)"
        class="ant-form-item-explain ant-form-item-explain-connected"
      >
        <div role="alert" [class]="['ant-form-item-explain-' + status]">
          <ng-container
            *nzStringTemplateOutlet="
              innerTip;
              context: { $implicit: validateControl }
            "
            >{{ innerTip }}</ng-container
          >
        </div>
      </div>
    }

    @if (nzExtra) {
      <div class="ant-form-item-extra">
        <ng-container *nzStringTemplateOutlet="nzExtra">{{
          nzExtra
        }}</ng-container>
      </div>
    }
  `,
  providers: [NzFormStatusService],
  host: {
    class: "ant-form-item-control",
  },
  imports: [NzOutletModule],
})
export class NzFormControlComponent
  implements OnChanges, OnInit, AfterContentInit
{
  private cdr = inject(ChangeDetectorRef);
  public i18n = inject(NzI18nService);
  private nzFormStatusService = inject(NzFormStatusService);
  private destroyRef = inject(DestroyRef);

  private _hasFeedback = false;
  private validateChanges: Subscription = Subscription.EMPTY;
  private validateString: string | null = null;
  private localeId!: string;
  private autoErrorTip?: string;

  private get disableAutoTips(): boolean {
    return this.nzDisableAutoTips !== undefined
      ? toBoolean(this.nzDisableAutoTips)
      : !!this.nzFormDirective?.nzDisableAutoTips;
  }

  private readonly noAnimation = inject(NzNoAnimationDirective, {
    optional: true,
    host: true,
  });
  private readonly animationEnabled = isAnimationEnabled(
    () => !this.noAnimation?.nzNoAnimation(),
  );
  protected readonly nzValidateAnimationEnter = withAnimationCheck(
    () => "ant-form-validate_animation-enter",
  );
  protected readonly nzValidateAnimationLeave = withAnimationCheck(
    () => "ant-form-validate_animation-leave",
  );

  status: NzFormControlStatusType = "";
  validateControl: AbstractControl | NgModel | null = null;
  innerTip:
    | string
    | TemplateRef<{ $implicit: AbstractControl | NgModel }>
    | null = null;

  @ContentChild(NgControl, { static: false }) defaultValidateControl?:
    | FormControlName
    | FormControlDirective;
  @Input() nzSuccessTip?:
    | string
    | TemplateRef<{ $implicit: AbstractControl | NgModel }>;
  @Input() nzWarningTip?:
    | string
    | TemplateRef<{ $implicit: AbstractControl | NgModel }>;
  @Input() nzErrorTip?:
    | string
    | TemplateRef<{ $implicit: AbstractControl | NgModel }>;
  @Input() nzValidatingTip?:
    | string
    | TemplateRef<{ $implicit: AbstractControl | NgModel }>;
  @Input() nzExtra?: string | TemplateRef<void>;
  @Input() nzAutoTips: Record<string, Record<string, string>> = {};
  @Input({ transform: booleanAttribute }) nzDisableAutoTips?: boolean;

  @Input({ transform: booleanAttribute })
  set nzHasFeedback(value: boolean) {
    this._hasFeedback = value;
    this.nzFormStatusService.formStatusChanges.next({
      status: this.status,
      hasFeedback: this._hasFeedback,
    });
    if (this.nzFormItemComponent) {
      this.nzFormItemComponent.setHasFeedback(this._hasFeedback);
    }
  }

  get nzHasFeedback(): boolean {
    return this._hasFeedback;
  }

  @Input()
  set nzValidateStatus(
    value: string | AbstractControl | FormControlName | NgModel,
  ) {
    if (value instanceof AbstractControl || value instanceof NgModel) {
      this.validateControl = value;
      this.validateString = null;
      this.watchControl();
    } else if (value instanceof FormControlName) {
      this.validateControl = value.control;
      this.validateString = null;
      this.watchControl();
    } else {
      this.validateString = value;
      this.validateControl = null;
      this.setStatus();
    }
  }

  private watchControl(): void {
    this.validateChanges.unsubscribe();
    /** miss detect https://github.com/angular/angular/issues/10887 **/
    if (this.validateControl && this.validateControl.statusChanges) {
      this.validateChanges = (
        this.validateControl.statusChanges as Observable<NzSafeAny>
      )
        .pipe(startWith(null), takeUntilDestroyed(this.destroyRef))
        .subscribe(() => {
          if (!this.disableAutoTips) {
            this.updateAutoErrorTip();
          }
          this.setStatus();
          this.cdr.markForCheck();
        });
    }
  }

  private setStatus(): void {
    this.status = this.getControlStatus(this.validateString);
    this.innerTip = this.getInnerTip(this.status);
    this.nzFormStatusService.formStatusChanges.next({
      status: this.status,
      hasFeedback: this.nzHasFeedback,
    });
    if (this.nzFormItemComponent) {
      this.nzFormItemComponent.setWithHelpViaTips(!!this.innerTip);
      this.nzFormItemComponent.setStatus(this.status);
    }
  }

  private getControlStatus(
    validateString: string | null,
  ): NzFormControlStatusType {
    let status: NzFormControlStatusType;

    if (
      validateString === "warning" ||
      this.validateControlStatus("INVALID", "warning")
    ) {
      status = "warning";
    } else if (
      validateString === "error" ||
      this.validateControlStatus("INVALID")
    ) {
      status = "error";
    } else if (
      validateString === "validating" ||
      validateString === "pending" ||
      this.validateControlStatus("PENDING")
    ) {
      status = "validating";
    } else if (
      validateString === "success" ||
      this.validateControlStatus("VALID")
    ) {
      status = "success";
    } else {
      status = "";
    }

    return status;
  }

  private validateControlStatus(
    validStatus: string,
    statusType?: NzFormControlStatusType,
  ): boolean {
    if (!this.validateControl) {
      return false;
    } else {
      const { dirty, touched, status } = this.validateControl;
      return (
        (!!dirty || !!touched) &&
        (statusType
          ? this.validateControl.hasError(statusType)
          : status === validStatus)
      );
    }
  }

  private getInnerTip(
    status: NzFormControlStatusType,
  ): string | TemplateRef<{ $implicit: AbstractControl | NgModel }> | null {
    switch (status) {
      case "error":
        return (
          (!this.disableAutoTips && this.autoErrorTip) ||
          this.nzErrorTip ||
          null
        );
      case "validating":
        return this.nzValidatingTip || null;
      case "success":
        return this.nzSuccessTip || null;
      case "warning":
        return this.nzWarningTip || null;
      default:
        return null;
    }
  }

  private updateAutoErrorTip(): void {
    if (this.validateControl) {
      const errors = this.validateControl.errors || {};
      let autoErrorTip = "";
      for (const key in errors) {
        if (errors.hasOwnProperty(key)) {
          autoErrorTip =
            errors[key]?.[this.localeId] ??
            this.nzAutoTips?.[this.localeId]?.[key] ??
            this.nzAutoTips.default?.[key] ??
            this.nzFormDirective?.nzAutoTips?.[this.localeId]?.[key] ??
            this.nzFormDirective?.nzAutoTips.default?.[key];
        }
        if (autoErrorTip) {
          break;
        }
      }
      this.autoErrorTip = autoErrorTip;
    }
  }

  private subscribeAutoTips(observable?: Observable<NzSafeAny>): void {
    observable?.pipe(takeUntilDestroyed(this.destroyRef)).subscribe(() => {
      if (!this.disableAutoTips) {
        this.updateAutoErrorTip();
        this.setStatus();
        this.cdr.markForCheck();
      }
    });
  }

  private nzFormItemComponent = inject(NzFormItemComponent, {
    host: true,
    optional: true,
  });
  private nzFormDirective = inject(NzFormDirective, { optional: true });

  constructor() {
    this.subscribeAutoTips(
      this.i18n.localeChange.pipe(
        tap((locale) => (this.localeId = locale.locale)),
      ),
    );
    this.subscribeAutoTips(
      this.nzFormDirective?.getInputObservable("nzAutoTips"),
    );
    this.subscribeAutoTips(
      this.nzFormDirective
        ?.getInputObservable("nzDisableAutoTips")
        .pipe(filter(() => this.nzDisableAutoTips === undefined)),
    );
  }

  ngOnChanges(changes: SimpleChanges): void {
    const {
      nzDisableAutoTips,
      nzAutoTips,
      nzSuccessTip,
      nzWarningTip,
      nzErrorTip,
      nzValidatingTip,
    } = changes;

    if (nzDisableAutoTips || nzAutoTips) {
      this.updateAutoErrorTip();
      this.setStatus();
    } else if (nzSuccessTip || nzWarningTip || nzErrorTip || nzValidatingTip) {
      this.setStatus();
    }
  }

  ngOnInit(): void {
    this.setStatus();
  }

  ngAfterContentInit(): void {
    if (!this.validateControl && !this.validateString) {
      if (this.defaultValidateControl instanceof FormControlDirective) {
        this.nzValidateStatus = this.defaultValidateControl.control;
      } else {
        this.nzValidateStatus = this.defaultValidateControl!;
      }
    }
  }

  protected onAnimationLeave(event: AnimationCallbackEvent): void {
    if (!this.animationEnabled()) {
      return event.animationComplete();
    }

    const element = event.target as HTMLElement;
    const onTransitionEnd = (): void => {
      element.removeEventListener("transitionend", onTransitionEnd);
      event.animationComplete();
    };
    element.addEventListener("transitionend", onTransitionEnd);
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
  AfterContentInit,
  AnimationCallbackEvent,
  booleanAttribute,
  ChangeDetectionStrategy,
  ChangeDetectorRef,
  Component,
  ContentChild,
  DestroyRef,
  inject,
  Input,
  OnChanges,
  OnInit,
  SimpleChanges,
  TemplateRef,
  ViewEncapsulation,
} from "@angular/core";
import { takeUntilDestroyed } from "@angular/core/rxjs-interop";
import {
  AbstractControl,
  FormControlDirective,
  FormControlName,
  NgControl,
  NgModel,
} from "@angular/forms";
import { Observable, Subscription } from "rxjs";
import { filter, startWith, tap } from "rxjs/operators";

import {
  isAnimationEnabled,
  NzNoAnimationDirective,
  withAnimationCheck,
} from "ng-zorro-antd/core/animation";
import { NzFormStatusService } from "ng-zorro-antd/core/form";
import { NzOutletModule } from "ng-zorro-antd/core/outlet";
import { NzSafeAny } from "ng-zorro-antd/core/types";
import { toBoolean } from "ng-zorro-antd/core/util";
import { NzI18nService } from "ng-zorro-antd/i18n";

import {
  NzFormControlStatusType,
  NzFormItemComponent,
} from "./form-item.component";
import { NzFormDirective } from "./form.directive";

@Component({
  selector: "nz-form-control",
  exportAs: "nzFormControl",
  encapsulation: ViewEncapsulation.None,
  changeDetection: ChangeDetectionStrategy.OnPush,
  template: `
    <div class="ant-form-item-control-input">
      <div class="ant-form-item-control-input-content">
        <ng-content />
      </div>
    </div>
    @if (innerTip) {
      <div
        [animate.enter]="nzValidateAnimationEnter()"
        [animate.leave]="nzValidateAnimationLeave()"
        (animate.leave)="onAnimationLeave($event)"
        class="ant-form-item-explain ant-form-item-explain-connected"
      >
        <div role="alert" [class]="['ant-form-item-explain-' + status]">
          <ng-container
            *nzStringTemplateOutlet="
              innerTip;
              context: { $implicit: validateControl }
            "
            >{{ innerTip }}</ng-container
          >
        </div>
      </div>
    }

    @if (nzExtra) {
      <div class="ant-form-item-extra">
        <ng-container *nzStringTemplateOutlet="nzExtra">{{
          nzExtra
        }}</ng-container>
      </div>
    }
  `,
  providers: [NzFormStatusService],
  host: {
    class: "ant-form-item-control",
  },
  imports: [NzOutletModule],
})
export class NzFormControlComponent
  implements OnChanges, OnInit, AfterContentInit
{
  private cdr = inject(ChangeDetectorRef);
  public i18n = inject(NzI18nService);
  private nzFormStatusService = inject(NzFormStatusService);
  private destroyRef = inject(DestroyRef);

  private _hasFeedback = false;
  private validateChanges: Subscription = Subscription.EMPTY;
  private validateString: string | null = null;
  private localeId!: string;
  private autoErrorTip?: string;

  private get disableAutoTips(): boolean {
    return this.nzDisableAutoTips !== undefined
      ? toBoolean(this.nzDisableAutoTips)
      : !!this.nzFormDirective?.nzDisableAutoTips;
  }

  private readonly noAnimation = inject(NzNoAnimationDirective, {
    optional: true,
    host: true,
  });
  private readonly animationEnabled = isAnimationEnabled(
    () => !this.noAnimation?.nzNoAnimation(),
  );
  protected readonly nzValidateAnimationEnter = withAnimationCheck(
    () => "ant-form-validate_animation-enter",
  );
  protected readonly nzValidateAnimationLeave = withAnimationCheck(
    () => "ant-form-validate_animation-leave",
  );

  status: NzFormControlStatusType = "";
  validateControl: AbstractControl | NgModel | null = null;
  innerTip:
    string | TemplateRef<{ $implicit: AbstractControl | NgModel }> | null =
    null;

  @ContentChild(NgControl, { static: false }) defaultValidateControl?:
    FormControlName | FormControlDirective;
  @Input() nzSuccessTip?:
    string | TemplateRef<{ $implicit: AbstractControl | NgModel }>;
  @Input() nzWarningTip?:
    string | TemplateRef<{ $implicit: AbstractControl | NgModel }>;
  @Input() nzErrorTip?:
    string | TemplateRef<{ $implicit: AbstractControl | NgModel }>;
  @Input() nzValidatingTip?:
    string | TemplateRef<{ $implicit: AbstractControl | NgModel }>;
  @Input() nzExtra?: string | TemplateRef<void>;
  @Input() nzAutoTips: Record<string, Record<string, string>> = {};
  @Input({ transform: booleanAttribute }) nzDisableAutoTips?: boolean;

  @Input({ transform: booleanAttribute })
  set nzHasFeedback(value: boolean) {
    this._hasFeedback = value;
    this.nzFormStatusService.formStatusChanges.next({
      status: this.status,
      hasFeedback: this._hasFeedback,
    });
    if (this.nzFormItemComponent) {
      this.nzFormItemComponent.setHasFeedback(this._hasFeedback);
    }
  }

  get nzHasFeedback(): boolean {
    return this._hasFeedback;
  }

  @Input()
  set nzValidateStatus(
    value: string | AbstractControl | FormControlName | NgModel,
  ) {
    if (value instanceof AbstractControl || value instanceof NgModel) {
      this.validateControl = value;
      this.validateString = null;
      this.watchControl();
    } else if (value instanceof FormControlName) {
      this.validateControl = value.control;
      this.validateString = null;
      this.watchControl();
    } else {
      this.validateString = value;
      this.validateControl = null;
      this.setStatus();
    }
  }

  private watchControl(): void {
    this.validateChanges.unsubscribe();
    /** miss detect https://github.com/angular/angular/issues/10887 **/
    if (this.validateControl && this.validateControl.statusChanges) {
      this.validateChanges = (
        this.validateControl.statusChanges as Observable<NzSafeAny>
      )
        .pipe(startWith(null), takeUntilDestroyed(this.destroyRef))
        .subscribe(() => {
          if (!this.disableAutoTips) {
            this.updateAutoErrorTip();
          }
          this.setStatus();
          this.cdr.markForCheck();
        });
    }
  }

  private setStatus(): void {
    this.status = this.getControlStatus(this.validateString);
    this.innerTip = this.getInnerTip(this.status);
    this.nzFormStatusService.formStatusChanges.next({
      status: this.status,
      hasFeedback: this.nzHasFeedback,
    });
    if (this.nzFormItemComponent) {
      this.nzFormItemComponent.setWithHelpViaTips(!!this.innerTip);
      this.nzFormItemComponent.setStatus(this.status);
    }
  }

  private getControlStatus(
    validateString: string | null,
  ): NzFormControlStatusType {
    let status: NzFormControlStatusType;

    if (
      validateString === "warning" ||
      this.validateControlStatus("INVALID", "warning")
    ) {
      status = "warning";
    } else if (
      validateString === "error" ||
      this.validateControlStatus("INVALID")
    ) {
      status = "error";
    } else if (
      validateString === "validating" ||
      validateString === "pending" ||
      this.validateControlStatus("PENDING")
    ) {
      status = "validating";
    } else if (
      validateString === "success" ||
      this.validateControlStatus("VALID")
    ) {
      status = "success";
    } else {
      status = "";
    }

    return status;
  }

  private validateControlStatus(
    validStatus: string,
    statusType?: NzFormControlStatusType,
  ): boolean {
    if (!this.validateControl) {
      return false;
    } else {
      const { dirty, touched, status } = this.validateControl;
      return (
        (!!dirty || !!touched) &&
        (statusType
          ? this.validateControl.hasError(statusType)
          : status === validStatus)
      );
    }
  }

  private getInnerTip(
    status: NzFormControlStatusType,
  ): string | TemplateRef<{ $implicit: AbstractControl | NgModel }> | null {
    switch (status) {
      case "error":
        return (
          (!this.disableAutoTips && this.autoErrorTip) ||
          this.nzErrorTip ||
          null
        );
      case "validating":
        return this.nzValidatingTip || null;
      case "success":
        return this.nzSuccessTip || null;
      case "warning":
        return this.nzWarningTip || null;
      default:
        return null;
    }
  }

  private updateAutoErrorTip(): void {
    if (this.validateControl) {
      const errors = this.validateControl.errors || {};
      let autoErrorTip = "";
      for (const key in errors) {
        if (errors.hasOwnProperty(key)) {
          autoErrorTip =
            errors[key]?.[this.localeId] ??
            this.nzAutoTips?.[this.localeId]?.[key] ??
            this.nzAutoTips.default?.[key] ??
            this.nzFormDirective?.nzAutoTips?.[this.localeId]?.[key] ??
            this.nzFormDirective?.nzAutoTips.default?.[key];
        }
        if (autoErrorTip) {
          break;
        }
      }
      this.autoErrorTip = autoErrorTip;
    }
  }

  private subscribeAutoTips(observable?: Observable<NzSafeAny>): void {
    observable?.pipe(takeUntilDestroyed(this.destroyRef)).subscribe(() => {
      if (!this.disableAutoTips) {
        this.updateAutoErrorTip();
        this.setStatus();
        this.cdr.markForCheck();
      }
    });
  }

  private nzFormItemComponent = inject(NzFormItemComponent, {
    host: true,
    optional: true,
  });
  private nzFormDirective = inject(NzFormDirective, { optional: true });

  constructor() {
    this.subscribeAutoTips(
      this.i18n.localeChange.pipe(
        tap((locale) => (this.localeId = locale.locale)),
      ),
    );
    this.subscribeAutoTips(
      this.nzFormDirective?.getInputObservable("nzAutoTips"),
    );
    this.subscribeAutoTips(
      this.nzFormDirective
        ?.getInputObservable("nzDisableAutoTips")
        .pipe(filter(() => this.nzDisableAutoTips === undefined)),
    );
  }

  ngOnChanges(changes: SimpleChanges): void {
    const {
      nzDisableAutoTips,
      nzAutoTips,
      nzSuccessTip,
      nzWarningTip,
      nzErrorTip,
      nzValidatingTip,
    } = changes;

    if (nzDisableAutoTips || nzAutoTips) {
      this.updateAutoErrorTip();
      this.setStatus();
    } else if (nzSuccessTip || nzWarningTip || nzErrorTip || nzValidatingTip) {
      this.setStatus();
    }
  }

  ngOnInit(): void {
    this.setStatus();
  }

  ngAfterContentInit(): void {
    if (!this.validateControl && !this.validateString) {
      if (this.defaultValidateControl instanceof FormControlDirective) {
        this.nzValidateStatus = this.defaultValidateControl.control;
      } else {
        this.nzValidateStatus = this.defaultValidateControl!;
      }
    }
  }

  protected onAnimationLeave(event: AnimationCallbackEvent): void {
    if (!this.animationEnabled()) {
      return event.animationComplete();
    }

    const element = event.target as HTMLElement;
    const onTransitionEnd = (): void => {
      element.removeEventListener("transitionend", onTransitionEnd);
      event.animationComplete();
    };
    element.addEventListener("transitionend", onTransitionEnd);
  }
}

`````

## Option 2

`````json
{"printWidth":100,"htmlWhitespaceSensitivity":"ignore"}
`````

### Diff

`````diff
===================================================================
--- prettier
+++ oxfmt
@@ -114,9 +114,10 @@
   validateControl: AbstractControl | NgModel | null = null;
   innerTip: string | TemplateRef<{ $implicit: AbstractControl | NgModel }> | null = null;
 
   @ContentChild(NgControl, { static: false }) defaultValidateControl?:
-    FormControlName | FormControlDirective;
+    | FormControlName
+    | FormControlDirective;
   @Input() nzSuccessTip?: string | TemplateRef<{ $implicit: AbstractControl | NgModel }>;
   @Input() nzWarningTip?: string | TemplateRef<{ $implicit: AbstractControl | NgModel }>;
   @Input() nzErrorTip?: string | TemplateRef<{ $implicit: AbstractControl | NgModel }>;
   @Input() nzValidatingTip?: string | TemplateRef<{ $implicit: AbstractControl | NgModel }>;

`````

### Actual (oxfmt)

`````ts
/**
 * Use of this source code is governed by an MIT-style license that can be
 * found in the LICENSE file at https://github.com/NG-ZORRO/ng-zorro-antd/blob/master/LICENSE
 */

import {
  AfterContentInit,
  AnimationCallbackEvent,
  booleanAttribute,
  ChangeDetectionStrategy,
  ChangeDetectorRef,
  Component,
  ContentChild,
  DestroyRef,
  inject,
  Input,
  OnChanges,
  OnInit,
  SimpleChanges,
  TemplateRef,
  ViewEncapsulation,
} from "@angular/core";
import { takeUntilDestroyed } from "@angular/core/rxjs-interop";
import {
  AbstractControl,
  FormControlDirective,
  FormControlName,
  NgControl,
  NgModel,
} from "@angular/forms";
import { Observable, Subscription } from "rxjs";
import { filter, startWith, tap } from "rxjs/operators";

import {
  isAnimationEnabled,
  NzNoAnimationDirective,
  withAnimationCheck,
} from "ng-zorro-antd/core/animation";
import { NzFormStatusService } from "ng-zorro-antd/core/form";
import { NzOutletModule } from "ng-zorro-antd/core/outlet";
import { NzSafeAny } from "ng-zorro-antd/core/types";
import { toBoolean } from "ng-zorro-antd/core/util";
import { NzI18nService } from "ng-zorro-antd/i18n";

import { NzFormControlStatusType, NzFormItemComponent } from "./form-item.component";
import { NzFormDirective } from "./form.directive";

@Component({
  selector: "nz-form-control",
  exportAs: "nzFormControl",
  encapsulation: ViewEncapsulation.None,
  changeDetection: ChangeDetectionStrategy.OnPush,
  template: `
    <div class="ant-form-item-control-input">
      <div class="ant-form-item-control-input-content">
        <ng-content />
      </div>
    </div>
    @if (innerTip) {
      <div
        [animate.enter]="nzValidateAnimationEnter()"
        [animate.leave]="nzValidateAnimationLeave()"
        (animate.leave)="onAnimationLeave($event)"
        class="ant-form-item-explain ant-form-item-explain-connected"
      >
        <div role="alert" [class]="['ant-form-item-explain-' + status]">
          <ng-container *nzStringTemplateOutlet="innerTip; context: { $implicit: validateControl }">
            {{ innerTip }}
          </ng-container>
        </div>
      </div>
    }

    @if (nzExtra) {
      <div class="ant-form-item-extra">
        <ng-container *nzStringTemplateOutlet="nzExtra">{{ nzExtra }}</ng-container>
      </div>
    }
  `,
  providers: [NzFormStatusService],
  host: {
    class: "ant-form-item-control",
  },
  imports: [NzOutletModule],
})
export class NzFormControlComponent implements OnChanges, OnInit, AfterContentInit {
  private cdr = inject(ChangeDetectorRef);
  public i18n = inject(NzI18nService);
  private nzFormStatusService = inject(NzFormStatusService);
  private destroyRef = inject(DestroyRef);

  private _hasFeedback = false;
  private validateChanges: Subscription = Subscription.EMPTY;
  private validateString: string | null = null;
  private localeId!: string;
  private autoErrorTip?: string;

  private get disableAutoTips(): boolean {
    return this.nzDisableAutoTips !== undefined
      ? toBoolean(this.nzDisableAutoTips)
      : !!this.nzFormDirective?.nzDisableAutoTips;
  }

  private readonly noAnimation = inject(NzNoAnimationDirective, { optional: true, host: true });
  private readonly animationEnabled = isAnimationEnabled(() => !this.noAnimation?.nzNoAnimation());
  protected readonly nzValidateAnimationEnter = withAnimationCheck(
    () => "ant-form-validate_animation-enter",
  );
  protected readonly nzValidateAnimationLeave = withAnimationCheck(
    () => "ant-form-validate_animation-leave",
  );

  status: NzFormControlStatusType = "";
  validateControl: AbstractControl | NgModel | null = null;
  innerTip: string | TemplateRef<{ $implicit: AbstractControl | NgModel }> | null = null;

  @ContentChild(NgControl, { static: false }) defaultValidateControl?:
    | FormControlName
    | FormControlDirective;
  @Input() nzSuccessTip?: string | TemplateRef<{ $implicit: AbstractControl | NgModel }>;
  @Input() nzWarningTip?: string | TemplateRef<{ $implicit: AbstractControl | NgModel }>;
  @Input() nzErrorTip?: string | TemplateRef<{ $implicit: AbstractControl | NgModel }>;
  @Input() nzValidatingTip?: string | TemplateRef<{ $implicit: AbstractControl | NgModel }>;
  @Input() nzExtra?: string | TemplateRef<void>;
  @Input() nzAutoTips: Record<string, Record<string, string>> = {};
  @Input({ transform: booleanAttribute }) nzDisableAutoTips?: boolean;

  @Input({ transform: booleanAttribute })
  set nzHasFeedback(value: boolean) {
    this._hasFeedback = value;
    this.nzFormStatusService.formStatusChanges.next({
      status: this.status,
      hasFeedback: this._hasFeedback,
    });
    if (this.nzFormItemComponent) {
      this.nzFormItemComponent.setHasFeedback(this._hasFeedback);
    }
  }

  get nzHasFeedback(): boolean {
    return this._hasFeedback;
  }

  @Input()
  set nzValidateStatus(value: string | AbstractControl | FormControlName | NgModel) {
    if (value instanceof AbstractControl || value instanceof NgModel) {
      this.validateControl = value;
      this.validateString = null;
      this.watchControl();
    } else if (value instanceof FormControlName) {
      this.validateControl = value.control;
      this.validateString = null;
      this.watchControl();
    } else {
      this.validateString = value;
      this.validateControl = null;
      this.setStatus();
    }
  }

  private watchControl(): void {
    this.validateChanges.unsubscribe();
    /** miss detect https://github.com/angular/angular/issues/10887 **/
    if (this.validateControl && this.validateControl.statusChanges) {
      this.validateChanges = (this.validateControl.statusChanges as Observable<NzSafeAny>)
        .pipe(startWith(null), takeUntilDestroyed(this.destroyRef))
        .subscribe(() => {
          if (!this.disableAutoTips) {
            this.updateAutoErrorTip();
          }
          this.setStatus();
          this.cdr.markForCheck();
        });
    }
  }

  private setStatus(): void {
    this.status = this.getControlStatus(this.validateString);
    this.innerTip = this.getInnerTip(this.status);
    this.nzFormStatusService.formStatusChanges.next({
      status: this.status,
      hasFeedback: this.nzHasFeedback,
    });
    if (this.nzFormItemComponent) {
      this.nzFormItemComponent.setWithHelpViaTips(!!this.innerTip);
      this.nzFormItemComponent.setStatus(this.status);
    }
  }

  private getControlStatus(validateString: string | null): NzFormControlStatusType {
    let status: NzFormControlStatusType;

    if (validateString === "warning" || this.validateControlStatus("INVALID", "warning")) {
      status = "warning";
    } else if (validateString === "error" || this.validateControlStatus("INVALID")) {
      status = "error";
    } else if (
      validateString === "validating" ||
      validateString === "pending" ||
      this.validateControlStatus("PENDING")
    ) {
      status = "validating";
    } else if (validateString === "success" || this.validateControlStatus("VALID")) {
      status = "success";
    } else {
      status = "";
    }

    return status;
  }

  private validateControlStatus(
    validStatus: string,
    statusType?: NzFormControlStatusType,
  ): boolean {
    if (!this.validateControl) {
      return false;
    } else {
      const { dirty, touched, status } = this.validateControl;
      return (
        (!!dirty || !!touched) &&
        (statusType ? this.validateControl.hasError(statusType) : status === validStatus)
      );
    }
  }

  private getInnerTip(
    status: NzFormControlStatusType,
  ): string | TemplateRef<{ $implicit: AbstractControl | NgModel }> | null {
    switch (status) {
      case "error":
        return (!this.disableAutoTips && this.autoErrorTip) || this.nzErrorTip || null;
      case "validating":
        return this.nzValidatingTip || null;
      case "success":
        return this.nzSuccessTip || null;
      case "warning":
        return this.nzWarningTip || null;
      default:
        return null;
    }
  }

  private updateAutoErrorTip(): void {
    if (this.validateControl) {
      const errors = this.validateControl.errors || {};
      let autoErrorTip = "";
      for (const key in errors) {
        if (errors.hasOwnProperty(key)) {
          autoErrorTip =
            errors[key]?.[this.localeId] ??
            this.nzAutoTips?.[this.localeId]?.[key] ??
            this.nzAutoTips.default?.[key] ??
            this.nzFormDirective?.nzAutoTips?.[this.localeId]?.[key] ??
            this.nzFormDirective?.nzAutoTips.default?.[key];
        }
        if (autoErrorTip) {
          break;
        }
      }
      this.autoErrorTip = autoErrorTip;
    }
  }

  private subscribeAutoTips(observable?: Observable<NzSafeAny>): void {
    observable?.pipe(takeUntilDestroyed(this.destroyRef)).subscribe(() => {
      if (!this.disableAutoTips) {
        this.updateAutoErrorTip();
        this.setStatus();
        this.cdr.markForCheck();
      }
    });
  }

  private nzFormItemComponent = inject(NzFormItemComponent, { host: true, optional: true });
  private nzFormDirective = inject(NzFormDirective, { optional: true });

  constructor() {
    this.subscribeAutoTips(
      this.i18n.localeChange.pipe(tap((locale) => (this.localeId = locale.locale))),
    );
    this.subscribeAutoTips(this.nzFormDirective?.getInputObservable("nzAutoTips"));
    this.subscribeAutoTips(
      this.nzFormDirective
        ?.getInputObservable("nzDisableAutoTips")
        .pipe(filter(() => this.nzDisableAutoTips === undefined)),
    );
  }

  ngOnChanges(changes: SimpleChanges): void {
    const {
      nzDisableAutoTips,
      nzAutoTips,
      nzSuccessTip,
      nzWarningTip,
      nzErrorTip,
      nzValidatingTip,
    } = changes;

    if (nzDisableAutoTips || nzAutoTips) {
      this.updateAutoErrorTip();
      this.setStatus();
    } else if (nzSuccessTip || nzWarningTip || nzErrorTip || nzValidatingTip) {
      this.setStatus();
    }
  }

  ngOnInit(): void {
    this.setStatus();
  }

  ngAfterContentInit(): void {
    if (!this.validateControl && !this.validateString) {
      if (this.defaultValidateControl instanceof FormControlDirective) {
        this.nzValidateStatus = this.defaultValidateControl.control;
      } else {
        this.nzValidateStatus = this.defaultValidateControl!;
      }
    }
  }

  protected onAnimationLeave(event: AnimationCallbackEvent): void {
    if (!this.animationEnabled()) {
      return event.animationComplete();
    }

    const element = event.target as HTMLElement;
    const onTransitionEnd = (): void => {
      element.removeEventListener("transitionend", onTransitionEnd);
      event.animationComplete();
    };
    element.addEventListener("transitionend", onTransitionEnd);
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
  AfterContentInit,
  AnimationCallbackEvent,
  booleanAttribute,
  ChangeDetectionStrategy,
  ChangeDetectorRef,
  Component,
  ContentChild,
  DestroyRef,
  inject,
  Input,
  OnChanges,
  OnInit,
  SimpleChanges,
  TemplateRef,
  ViewEncapsulation,
} from "@angular/core";
import { takeUntilDestroyed } from "@angular/core/rxjs-interop";
import {
  AbstractControl,
  FormControlDirective,
  FormControlName,
  NgControl,
  NgModel,
} from "@angular/forms";
import { Observable, Subscription } from "rxjs";
import { filter, startWith, tap } from "rxjs/operators";

import {
  isAnimationEnabled,
  NzNoAnimationDirective,
  withAnimationCheck,
} from "ng-zorro-antd/core/animation";
import { NzFormStatusService } from "ng-zorro-antd/core/form";
import { NzOutletModule } from "ng-zorro-antd/core/outlet";
import { NzSafeAny } from "ng-zorro-antd/core/types";
import { toBoolean } from "ng-zorro-antd/core/util";
import { NzI18nService } from "ng-zorro-antd/i18n";

import { NzFormControlStatusType, NzFormItemComponent } from "./form-item.component";
import { NzFormDirective } from "./form.directive";

@Component({
  selector: "nz-form-control",
  exportAs: "nzFormControl",
  encapsulation: ViewEncapsulation.None,
  changeDetection: ChangeDetectionStrategy.OnPush,
  template: `
    <div class="ant-form-item-control-input">
      <div class="ant-form-item-control-input-content">
        <ng-content />
      </div>
    </div>
    @if (innerTip) {
      <div
        [animate.enter]="nzValidateAnimationEnter()"
        [animate.leave]="nzValidateAnimationLeave()"
        (animate.leave)="onAnimationLeave($event)"
        class="ant-form-item-explain ant-form-item-explain-connected"
      >
        <div role="alert" [class]="['ant-form-item-explain-' + status]">
          <ng-container *nzStringTemplateOutlet="innerTip; context: { $implicit: validateControl }">
            {{ innerTip }}
          </ng-container>
        </div>
      </div>
    }

    @if (nzExtra) {
      <div class="ant-form-item-extra">
        <ng-container *nzStringTemplateOutlet="nzExtra">{{ nzExtra }}</ng-container>
      </div>
    }
  `,
  providers: [NzFormStatusService],
  host: {
    class: "ant-form-item-control",
  },
  imports: [NzOutletModule],
})
export class NzFormControlComponent implements OnChanges, OnInit, AfterContentInit {
  private cdr = inject(ChangeDetectorRef);
  public i18n = inject(NzI18nService);
  private nzFormStatusService = inject(NzFormStatusService);
  private destroyRef = inject(DestroyRef);

  private _hasFeedback = false;
  private validateChanges: Subscription = Subscription.EMPTY;
  private validateString: string | null = null;
  private localeId!: string;
  private autoErrorTip?: string;

  private get disableAutoTips(): boolean {
    return this.nzDisableAutoTips !== undefined
      ? toBoolean(this.nzDisableAutoTips)
      : !!this.nzFormDirective?.nzDisableAutoTips;
  }

  private readonly noAnimation = inject(NzNoAnimationDirective, { optional: true, host: true });
  private readonly animationEnabled = isAnimationEnabled(() => !this.noAnimation?.nzNoAnimation());
  protected readonly nzValidateAnimationEnter = withAnimationCheck(
    () => "ant-form-validate_animation-enter",
  );
  protected readonly nzValidateAnimationLeave = withAnimationCheck(
    () => "ant-form-validate_animation-leave",
  );

  status: NzFormControlStatusType = "";
  validateControl: AbstractControl | NgModel | null = null;
  innerTip: string | TemplateRef<{ $implicit: AbstractControl | NgModel }> | null = null;

  @ContentChild(NgControl, { static: false }) defaultValidateControl?:
    FormControlName | FormControlDirective;
  @Input() nzSuccessTip?: string | TemplateRef<{ $implicit: AbstractControl | NgModel }>;
  @Input() nzWarningTip?: string | TemplateRef<{ $implicit: AbstractControl | NgModel }>;
  @Input() nzErrorTip?: string | TemplateRef<{ $implicit: AbstractControl | NgModel }>;
  @Input() nzValidatingTip?: string | TemplateRef<{ $implicit: AbstractControl | NgModel }>;
  @Input() nzExtra?: string | TemplateRef<void>;
  @Input() nzAutoTips: Record<string, Record<string, string>> = {};
  @Input({ transform: booleanAttribute }) nzDisableAutoTips?: boolean;

  @Input({ transform: booleanAttribute })
  set nzHasFeedback(value: boolean) {
    this._hasFeedback = value;
    this.nzFormStatusService.formStatusChanges.next({
      status: this.status,
      hasFeedback: this._hasFeedback,
    });
    if (this.nzFormItemComponent) {
      this.nzFormItemComponent.setHasFeedback(this._hasFeedback);
    }
  }

  get nzHasFeedback(): boolean {
    return this._hasFeedback;
  }

  @Input()
  set nzValidateStatus(value: string | AbstractControl | FormControlName | NgModel) {
    if (value instanceof AbstractControl || value instanceof NgModel) {
      this.validateControl = value;
      this.validateString = null;
      this.watchControl();
    } else if (value instanceof FormControlName) {
      this.validateControl = value.control;
      this.validateString = null;
      this.watchControl();
    } else {
      this.validateString = value;
      this.validateControl = null;
      this.setStatus();
    }
  }

  private watchControl(): void {
    this.validateChanges.unsubscribe();
    /** miss detect https://github.com/angular/angular/issues/10887 **/
    if (this.validateControl && this.validateControl.statusChanges) {
      this.validateChanges = (this.validateControl.statusChanges as Observable<NzSafeAny>)
        .pipe(startWith(null), takeUntilDestroyed(this.destroyRef))
        .subscribe(() => {
          if (!this.disableAutoTips) {
            this.updateAutoErrorTip();
          }
          this.setStatus();
          this.cdr.markForCheck();
        });
    }
  }

  private setStatus(): void {
    this.status = this.getControlStatus(this.validateString);
    this.innerTip = this.getInnerTip(this.status);
    this.nzFormStatusService.formStatusChanges.next({
      status: this.status,
      hasFeedback: this.nzHasFeedback,
    });
    if (this.nzFormItemComponent) {
      this.nzFormItemComponent.setWithHelpViaTips(!!this.innerTip);
      this.nzFormItemComponent.setStatus(this.status);
    }
  }

  private getControlStatus(validateString: string | null): NzFormControlStatusType {
    let status: NzFormControlStatusType;

    if (validateString === "warning" || this.validateControlStatus("INVALID", "warning")) {
      status = "warning";
    } else if (validateString === "error" || this.validateControlStatus("INVALID")) {
      status = "error";
    } else if (
      validateString === "validating" ||
      validateString === "pending" ||
      this.validateControlStatus("PENDING")
    ) {
      status = "validating";
    } else if (validateString === "success" || this.validateControlStatus("VALID")) {
      status = "success";
    } else {
      status = "";
    }

    return status;
  }

  private validateControlStatus(
    validStatus: string,
    statusType?: NzFormControlStatusType,
  ): boolean {
    if (!this.validateControl) {
      return false;
    } else {
      const { dirty, touched, status } = this.validateControl;
      return (
        (!!dirty || !!touched) &&
        (statusType ? this.validateControl.hasError(statusType) : status === validStatus)
      );
    }
  }

  private getInnerTip(
    status: NzFormControlStatusType,
  ): string | TemplateRef<{ $implicit: AbstractControl | NgModel }> | null {
    switch (status) {
      case "error":
        return (!this.disableAutoTips && this.autoErrorTip) || this.nzErrorTip || null;
      case "validating":
        return this.nzValidatingTip || null;
      case "success":
        return this.nzSuccessTip || null;
      case "warning":
        return this.nzWarningTip || null;
      default:
        return null;
    }
  }

  private updateAutoErrorTip(): void {
    if (this.validateControl) {
      const errors = this.validateControl.errors || {};
      let autoErrorTip = "";
      for (const key in errors) {
        if (errors.hasOwnProperty(key)) {
          autoErrorTip =
            errors[key]?.[this.localeId] ??
            this.nzAutoTips?.[this.localeId]?.[key] ??
            this.nzAutoTips.default?.[key] ??
            this.nzFormDirective?.nzAutoTips?.[this.localeId]?.[key] ??
            this.nzFormDirective?.nzAutoTips.default?.[key];
        }
        if (autoErrorTip) {
          break;
        }
      }
      this.autoErrorTip = autoErrorTip;
    }
  }

  private subscribeAutoTips(observable?: Observable<NzSafeAny>): void {
    observable?.pipe(takeUntilDestroyed(this.destroyRef)).subscribe(() => {
      if (!this.disableAutoTips) {
        this.updateAutoErrorTip();
        this.setStatus();
        this.cdr.markForCheck();
      }
    });
  }

  private nzFormItemComponent = inject(NzFormItemComponent, { host: true, optional: true });
  private nzFormDirective = inject(NzFormDirective, { optional: true });

  constructor() {
    this.subscribeAutoTips(
      this.i18n.localeChange.pipe(tap((locale) => (this.localeId = locale.locale))),
    );
    this.subscribeAutoTips(this.nzFormDirective?.getInputObservable("nzAutoTips"));
    this.subscribeAutoTips(
      this.nzFormDirective
        ?.getInputObservable("nzDisableAutoTips")
        .pipe(filter(() => this.nzDisableAutoTips === undefined)),
    );
  }

  ngOnChanges(changes: SimpleChanges): void {
    const {
      nzDisableAutoTips,
      nzAutoTips,
      nzSuccessTip,
      nzWarningTip,
      nzErrorTip,
      nzValidatingTip,
    } = changes;

    if (nzDisableAutoTips || nzAutoTips) {
      this.updateAutoErrorTip();
      this.setStatus();
    } else if (nzSuccessTip || nzWarningTip || nzErrorTip || nzValidatingTip) {
      this.setStatus();
    }
  }

  ngOnInit(): void {
    this.setStatus();
  }

  ngAfterContentInit(): void {
    if (!this.validateControl && !this.validateString) {
      if (this.defaultValidateControl instanceof FormControlDirective) {
        this.nzValidateStatus = this.defaultValidateControl.control;
      } else {
        this.nzValidateStatus = this.defaultValidateControl!;
      }
    }
  }

  protected onAnimationLeave(event: AnimationCallbackEvent): void {
    if (!this.animationEnabled()) {
      return event.animationComplete();
    }

    const element = event.target as HTMLElement;
    const onTransitionEnd = (): void => {
      element.removeEventListener("transitionend", onTransitionEnd);
      event.animationComplete();
    };
    element.addEventListener("transitionend", onTransitionEnd);
  }
}

`````
