# externals/ng-zorro-antd/components/modal/modal.component.ts

## Option 1

`````json
{"printWidth":80}
`````

### Diff

`````diff
===================================================================
--- prettier
+++ oxfmt
@@ -70,9 +70,12 @@
   @Input({ transform: booleanAttribute }) nzCentered = false;
   @Input({ transform: booleanAttribute }) nzDraggable = false;
   @Input() nzContent?: string | TemplateRef<{}> | Type<T>;
   @Input() nzFooter?:
-    string | TemplateRef<{}> | Array<ModalButtonOptions<T>> | null;
+    | string
+    | TemplateRef<{}>
+    | Array<ModalButtonOptions<T>>
+    | null;
   @Input({ transform: numberAttribute }) nzZIndex: number = 1000;
   @Input() nzWidth: number | string = 520;
   @Input() nzWrapClassName?: string;
   @Input() nzClassName?: string;

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
  ContentChild,
  DestroyRef,
  EventEmitter,
  Input,
  OnChanges,
  Output,
  SimpleChanges,
  TemplateRef,
  Type,
  ViewContainerRef,
  booleanAttribute,
  inject,
  numberAttribute,
} from "@angular/core";
import { takeUntilDestroyed } from "@angular/core/rxjs-interop";
import { Observable } from "rxjs";

import { NzButtonType } from "ng-zorro-antd/button";
import { NzSafeAny } from "ng-zorro-antd/core/types";

import { NzModalContentDirective } from "./modal-content.directive";
import { NzModalFooterDirective } from "./modal-footer.directive";
import { NzModalLegacyAPI } from "./modal-legacy-api";
import { NzModalRef } from "./modal-ref";
import { NzModalTitleDirective } from "./modal-title.directive";
import {
  ModalButtonOptions,
  ModalOptions,
  ModalTypes,
  OnClickCallback,
  StyleObjectLike,
} from "./modal-types";
import { NzModalService } from "./modal.service";
import { getConfigFromComponent } from "./utils";

@Component({
  selector: "nz-modal",
  exportAs: "nzModal",
  template: ``,
  changeDetection: ChangeDetectionStrategy.OnPush,
})
export class NzModalComponent<T extends ModalOptions = NzSafeAny, R = NzSafeAny>
  implements OnChanges, NzModalLegacyAPI<T, R>
{
  private cdr = inject(ChangeDetectorRef);
  private modal = inject(NzModalService);
  private viewContainerRef = inject(ViewContainerRef);
  private destroyRef = inject(DestroyRef);

  @Input({ transform: booleanAttribute }) nzMask?: boolean;
  @Input({ transform: booleanAttribute }) nzMaskClosable?: boolean;
  @Input({ transform: booleanAttribute }) nzCloseOnNavigation?: boolean;
  @Input({ transform: booleanAttribute }) nzVisible: boolean = false;
  @Input({ transform: booleanAttribute }) nzClosable: boolean = true;
  @Input({ transform: booleanAttribute }) nzOkLoading: boolean = false;
  @Input({ transform: booleanAttribute }) nzOkDisabled: boolean = false;
  @Input({ transform: booleanAttribute }) nzCancelDisabled: boolean = false;
  @Input({ transform: booleanAttribute }) nzCancelLoading: boolean = false;
  @Input({ transform: booleanAttribute }) nzKeyboard: boolean = true;
  @Input({ transform: booleanAttribute }) nzNoAnimation = false;
  @Input({ transform: booleanAttribute }) nzCentered = false;
  @Input({ transform: booleanAttribute }) nzDraggable = false;
  @Input() nzContent?: string | TemplateRef<{}> | Type<T>;
  @Input() nzFooter?:
    | string
    | TemplateRef<{}>
    | Array<ModalButtonOptions<T>>
    | null;
  @Input({ transform: numberAttribute }) nzZIndex: number = 1000;
  @Input() nzWidth: number | string = 520;
  @Input() nzWrapClassName?: string;
  @Input() nzClassName?: string;
  @Input() nzStyle?: object;
  @Input() nzTitle?: string | TemplateRef<{}>;
  @Input() nzCloseIcon: string | TemplateRef<void> = "close";
  @Input() nzMaskStyle?: StyleObjectLike;
  @Input() nzBodyStyle?: StyleObjectLike;
  @Input() nzOkText?: string | null;
  @Input() nzCancelText?: string | null;
  @Input() nzOkType: NzButtonType = "primary";
  @Input({ transform: booleanAttribute }) nzOkDanger: boolean = false;
  @Input() nzIconType: string = "question-circle"; // Confirm Modal ONLY
  @Input() nzModalType: ModalTypes = "default";
  @Input() nzAutofocus: "ok" | "cancel" | "auto" | null = "auto";

  /**
   * @note The input usage will be removed in v22.
   */
  @Input()
  @Output()
  readonly nzOnOk: EventEmitter<T> | OnClickCallback<T> | NzSafeAny =
    new EventEmitter<T>();

  /**
   * @note The input usage will be removed in v22.
   */
  @Input()
  @Output()
  readonly nzOnCancel: EventEmitter<T> | OnClickCallback<T> | NzSafeAny =
    new EventEmitter<T>();

  @Output() readonly nzAfterOpen = new EventEmitter<void>();
  @Output() readonly nzAfterClose = new EventEmitter<R>();
  @Output() readonly nzVisibleChange = new EventEmitter<boolean>();

  @ContentChild(NzModalTitleDirective, { static: true, read: TemplateRef })
  set modalTitle(value: TemplateRef<NzSafeAny>) {
    if (value) {
      this.setTitleWithTemplate(value);
    }
  }

  @ContentChild(NzModalContentDirective, { static: true, read: TemplateRef })
  contentFromContentChild!: TemplateRef<NzSafeAny>;

  @ContentChild(NzModalFooterDirective, { static: true, read: TemplateRef })
  set modalFooter(value: TemplateRef<NzSafeAny>) {
    if (value) {
      this.setFooterWithTemplate(value);
    }
  }

  private modalRef: NzModalRef | null = null;

  get afterOpen(): Observable<void> {
    // Observable alias for nzAfterOpen
    return this.nzAfterOpen.asObservable();
  }

  get afterClose(): Observable<R> {
    // Observable alias for nzAfterClose
    return this.nzAfterClose.asObservable();
  }

  constructor() {
    this.destroyRef.onDestroy(() => {
      this.modalRef?._finishDialogClose();
    });
  }

  open(): void {
    if (!this.nzVisible) {
      this.nzVisible = true;
      this.nzVisibleChange.emit(true);
    }

    if (!this.modalRef) {
      const config = this.getConfig();
      this.modalRef = this.modal.create(config);

      // When the modal is implicitly closed (e.g. closeAll) the nzVisible needs to be set to the correct value and emit.
      this.modalRef.afterClose
        .asObservable()
        .pipe(takeUntilDestroyed(this.destroyRef))
        .subscribe(() => {
          this.close();
        });
    }
  }

  close(result?: R): void {
    if (this.nzVisible) {
      this.nzVisible = false;
      this.nzVisibleChange.emit(false);
    }

    if (this.modalRef) {
      this.modalRef.close(result);
      this.modalRef = null;
    }
  }

  destroy(result?: R): void {
    this.close(result);
  }

  triggerOk(): void {
    this.modalRef?.triggerOk();
  }

  triggerCancel(): void {
    this.modalRef?.triggerCancel();
  }

  getContentComponent(): T | void {
    return this.modalRef?.getContentComponent();
  }

  getElement(): HTMLElement | void {
    return this.modalRef?.getElement();
  }

  getModalRef(): NzModalRef | null {
    return this.modalRef;
  }

  private setTitleWithTemplate(templateRef: TemplateRef<{}>): void {
    this.nzTitle = templateRef;
    if (this.modalRef) {
      // If modalRef already created, set the title in next tick
      Promise.resolve().then(() => {
        this.modalRef!.updateConfig({
          nzTitle: this.nzTitle,
        });
      });
    }
  }

  private setFooterWithTemplate(templateRef: TemplateRef<{}>): void {
    this.nzFooter = templateRef;
    if (this.modalRef) {
      // If modalRef already created, set the footer in next tick
      Promise.resolve().then(() => {
        this.modalRef!.updateConfig({
          nzFooter: this.nzFooter,
        });
      });
    }

    this.cdr.markForCheck();
  }

  private getConfig(): ModalOptions {
    const componentConfig = getConfigFromComponent(this);
    componentConfig.nzViewContainerRef = this.viewContainerRef;
    componentConfig.nzContent = this.nzContent || this.contentFromContentChild;
    return componentConfig;
  }

  ngOnChanges(changes: SimpleChanges): void {
    const { nzVisible, ...otherChanges } = changes;

    if (Object.keys(otherChanges).length && this.modalRef) {
      this.modalRef.updateConfig(getConfigFromComponent(this));
    }

    if (nzVisible) {
      if (this.nzVisible) {
        this.open();
      } else {
        this.close();
      }
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

import {
  ChangeDetectionStrategy,
  ChangeDetectorRef,
  Component,
  ContentChild,
  DestroyRef,
  EventEmitter,
  Input,
  OnChanges,
  Output,
  SimpleChanges,
  TemplateRef,
  Type,
  ViewContainerRef,
  booleanAttribute,
  inject,
  numberAttribute,
} from "@angular/core";
import { takeUntilDestroyed } from "@angular/core/rxjs-interop";
import { Observable } from "rxjs";

import { NzButtonType } from "ng-zorro-antd/button";
import { NzSafeAny } from "ng-zorro-antd/core/types";

import { NzModalContentDirective } from "./modal-content.directive";
import { NzModalFooterDirective } from "./modal-footer.directive";
import { NzModalLegacyAPI } from "./modal-legacy-api";
import { NzModalRef } from "./modal-ref";
import { NzModalTitleDirective } from "./modal-title.directive";
import {
  ModalButtonOptions,
  ModalOptions,
  ModalTypes,
  OnClickCallback,
  StyleObjectLike,
} from "./modal-types";
import { NzModalService } from "./modal.service";
import { getConfigFromComponent } from "./utils";

@Component({
  selector: "nz-modal",
  exportAs: "nzModal",
  template: ``,
  changeDetection: ChangeDetectionStrategy.OnPush,
})
export class NzModalComponent<T extends ModalOptions = NzSafeAny, R = NzSafeAny>
  implements OnChanges, NzModalLegacyAPI<T, R>
{
  private cdr = inject(ChangeDetectorRef);
  private modal = inject(NzModalService);
  private viewContainerRef = inject(ViewContainerRef);
  private destroyRef = inject(DestroyRef);

  @Input({ transform: booleanAttribute }) nzMask?: boolean;
  @Input({ transform: booleanAttribute }) nzMaskClosable?: boolean;
  @Input({ transform: booleanAttribute }) nzCloseOnNavigation?: boolean;
  @Input({ transform: booleanAttribute }) nzVisible: boolean = false;
  @Input({ transform: booleanAttribute }) nzClosable: boolean = true;
  @Input({ transform: booleanAttribute }) nzOkLoading: boolean = false;
  @Input({ transform: booleanAttribute }) nzOkDisabled: boolean = false;
  @Input({ transform: booleanAttribute }) nzCancelDisabled: boolean = false;
  @Input({ transform: booleanAttribute }) nzCancelLoading: boolean = false;
  @Input({ transform: booleanAttribute }) nzKeyboard: boolean = true;
  @Input({ transform: booleanAttribute }) nzNoAnimation = false;
  @Input({ transform: booleanAttribute }) nzCentered = false;
  @Input({ transform: booleanAttribute }) nzDraggable = false;
  @Input() nzContent?: string | TemplateRef<{}> | Type<T>;
  @Input() nzFooter?:
    string | TemplateRef<{}> | Array<ModalButtonOptions<T>> | null;
  @Input({ transform: numberAttribute }) nzZIndex: number = 1000;
  @Input() nzWidth: number | string = 520;
  @Input() nzWrapClassName?: string;
  @Input() nzClassName?: string;
  @Input() nzStyle?: object;
  @Input() nzTitle?: string | TemplateRef<{}>;
  @Input() nzCloseIcon: string | TemplateRef<void> = "close";
  @Input() nzMaskStyle?: StyleObjectLike;
  @Input() nzBodyStyle?: StyleObjectLike;
  @Input() nzOkText?: string | null;
  @Input() nzCancelText?: string | null;
  @Input() nzOkType: NzButtonType = "primary";
  @Input({ transform: booleanAttribute }) nzOkDanger: boolean = false;
  @Input() nzIconType: string = "question-circle"; // Confirm Modal ONLY
  @Input() nzModalType: ModalTypes = "default";
  @Input() nzAutofocus: "ok" | "cancel" | "auto" | null = "auto";

  /**
   * @note The input usage will be removed in v22.
   */
  @Input()
  @Output()
  readonly nzOnOk: EventEmitter<T> | OnClickCallback<T> | NzSafeAny =
    new EventEmitter<T>();

  /**
   * @note The input usage will be removed in v22.
   */
  @Input()
  @Output()
  readonly nzOnCancel: EventEmitter<T> | OnClickCallback<T> | NzSafeAny =
    new EventEmitter<T>();

  @Output() readonly nzAfterOpen = new EventEmitter<void>();
  @Output() readonly nzAfterClose = new EventEmitter<R>();
  @Output() readonly nzVisibleChange = new EventEmitter<boolean>();

  @ContentChild(NzModalTitleDirective, { static: true, read: TemplateRef })
  set modalTitle(value: TemplateRef<NzSafeAny>) {
    if (value) {
      this.setTitleWithTemplate(value);
    }
  }

  @ContentChild(NzModalContentDirective, { static: true, read: TemplateRef })
  contentFromContentChild!: TemplateRef<NzSafeAny>;

  @ContentChild(NzModalFooterDirective, { static: true, read: TemplateRef })
  set modalFooter(value: TemplateRef<NzSafeAny>) {
    if (value) {
      this.setFooterWithTemplate(value);
    }
  }

  private modalRef: NzModalRef | null = null;

  get afterOpen(): Observable<void> {
    // Observable alias for nzAfterOpen
    return this.nzAfterOpen.asObservable();
  }

  get afterClose(): Observable<R> {
    // Observable alias for nzAfterClose
    return this.nzAfterClose.asObservable();
  }

  constructor() {
    this.destroyRef.onDestroy(() => {
      this.modalRef?._finishDialogClose();
    });
  }

  open(): void {
    if (!this.nzVisible) {
      this.nzVisible = true;
      this.nzVisibleChange.emit(true);
    }

    if (!this.modalRef) {
      const config = this.getConfig();
      this.modalRef = this.modal.create(config);

      // When the modal is implicitly closed (e.g. closeAll) the nzVisible needs to be set to the correct value and emit.
      this.modalRef.afterClose
        .asObservable()
        .pipe(takeUntilDestroyed(this.destroyRef))
        .subscribe(() => {
          this.close();
        });
    }
  }

  close(result?: R): void {
    if (this.nzVisible) {
      this.nzVisible = false;
      this.nzVisibleChange.emit(false);
    }

    if (this.modalRef) {
      this.modalRef.close(result);
      this.modalRef = null;
    }
  }

  destroy(result?: R): void {
    this.close(result);
  }

  triggerOk(): void {
    this.modalRef?.triggerOk();
  }

  triggerCancel(): void {
    this.modalRef?.triggerCancel();
  }

  getContentComponent(): T | void {
    return this.modalRef?.getContentComponent();
  }

  getElement(): HTMLElement | void {
    return this.modalRef?.getElement();
  }

  getModalRef(): NzModalRef | null {
    return this.modalRef;
  }

  private setTitleWithTemplate(templateRef: TemplateRef<{}>): void {
    this.nzTitle = templateRef;
    if (this.modalRef) {
      // If modalRef already created, set the title in next tick
      Promise.resolve().then(() => {
        this.modalRef!.updateConfig({
          nzTitle: this.nzTitle,
        });
      });
    }
  }

  private setFooterWithTemplate(templateRef: TemplateRef<{}>): void {
    this.nzFooter = templateRef;
    if (this.modalRef) {
      // If modalRef already created, set the footer in next tick
      Promise.resolve().then(() => {
        this.modalRef!.updateConfig({
          nzFooter: this.nzFooter,
        });
      });
    }

    this.cdr.markForCheck();
  }

  private getConfig(): ModalOptions {
    const componentConfig = getConfigFromComponent(this);
    componentConfig.nzViewContainerRef = this.viewContainerRef;
    componentConfig.nzContent = this.nzContent || this.contentFromContentChild;
    return componentConfig;
  }

  ngOnChanges(changes: SimpleChanges): void {
    const { nzVisible, ...otherChanges } = changes;

    if (Object.keys(otherChanges).length && this.modalRef) {
      this.modalRef.updateConfig(getConfigFromComponent(this));
    }

    if (nzVisible) {
      if (this.nzVisible) {
        this.open();
      } else {
        this.close();
      }
    }
  }
}

`````
