# externals/ng-zorro-antd/components/drawer/drawer.component.ts

## Option 1

`````json
{"printWidth":80}
`````

### Diff

`````diff
===================================================================
--- prettier
+++ oxfmt
@@ -209,9 +209,10 @@
 
   readonly _nzModuleName: NzConfigKey = NZ_CONFIG_MODULE_NAME;
 
   @Input() nzContent!:
-    TemplateRef<{ $implicit: D; drawerRef: NzDrawerRef<R> }> | Type<T>;
+    | TemplateRef<{ $implicit: D; drawerRef: NzDrawerRef<R> }>
+    | Type<T>;
   @Input() nzCloseIcon: string | TemplateRef<void> = "close";
   @Input({ transform: booleanAttribute }) nzClosable: boolean = true;
   @Input({ transform: booleanAttribute })
   @WithConfig()

`````

### Actual (oxfmt)

`````ts
/**
 * Use of this source code is governed by an MIT-style license that can be
 * found in the LICENSE file at https://github.com/NG-ZORRO/ng-zorro-antd/blob/master/LICENSE
 */

import { FocusTrap, FocusTrapFactory } from "@angular/cdk/a11y";
import { Direction, Directionality } from "@angular/cdk/bidi";
import { ESCAPE } from "@angular/cdk/keycodes";
import {
  CdkScrollable,
  createBlockScrollStrategy,
  createGlobalPositionStrategy,
  createOverlayRef,
  OverlayKeyboardDispatcher,
  OverlayRef,
} from "@angular/cdk/overlay";
import {
  CdkPortalOutlet,
  ComponentPortal,
  PortalModule,
  TemplatePortal,
} from "@angular/cdk/portal";
import { NgTemplateOutlet } from "@angular/common";
import {
  AfterViewInit,
  booleanAttribute,
  ChangeDetectionStrategy,
  ChangeDetectorRef,
  Component,
  ComponentRef,
  ContentChild,
  DestroyRef,
  DOCUMENT,
  EventEmitter,
  inject,
  Injector,
  Input,
  OnChanges,
  OnInit,
  Output,
  Renderer2,
  SimpleChanges,
  TemplateRef,
  Type,
  ViewChild,
  ViewContainerRef,
} from "@angular/core";
import { takeUntilDestroyed } from "@angular/core/rxjs-interop";
import { Observable, Subject } from "rxjs";

import {
  NzNoAnimationDirective,
  withAnimationCheck,
} from "ng-zorro-antd/core/animation";
import { NzConfigKey, WithConfig } from "ng-zorro-antd/core/config";
import { NzOutletModule } from "ng-zorro-antd/core/outlet";
import { overlayZIndexSetter } from "ng-zorro-antd/core/overlay";
import { NgStyleInterface, NzSafeAny } from "ng-zorro-antd/core/types";
import { isTemplateRef, toCssPixel } from "ng-zorro-antd/core/util";
import { NzIconModule } from "ng-zorro-antd/icon";

import { NzDrawerContentDirective } from "./drawer-content.directive";
import {
  DRAWER_DEFAULT_SIZE,
  DRAWER_LARGE_SIZE,
  NZ_DRAWER_DATA,
  NzDrawerOptionsOfComponent,
  NzDrawerPlacement,
  NzDrawerSize,
} from "./drawer-options";
import { NzDrawerRef } from "./drawer-ref";

export const DRAWER_ANIMATE_DURATION = 300;

const NZ_CONFIG_MODULE_NAME: NzConfigKey = "drawer";

@Component({
  selector: "nz-drawer",
  exportAs: "nzDrawer",
  template: `
    <ng-template #drawerTemplate>
      <div
        class="ant-drawer"
        [nzNoAnimation]="nzNoAnimation"
        [class.ant-drawer-rtl]="dir === 'rtl'"
        [class.ant-drawer-open]="isOpen"
        [class.no-mask]="!nzMask"
        [class.ant-drawer-top]="nzPlacement === 'top'"
        [class.ant-drawer-bottom]="nzPlacement === 'bottom'"
        [class.ant-drawer-right]="nzPlacement === 'right'"
        [class.ant-drawer-left]="nzPlacement === 'left'"
        [style.transform]="offsetTransform"
        [style.transition]="placementChanging ? 'none' : null"
        [style.zIndex]="nzZIndex"
      >
        @if (nzMask && isOpen) {
          <div
            class="ant-drawer-mask"
            [animate.enter]="maskAnimationEnter()"
            [animate.leave]="maskAnimationLeave()"
            (click)="maskClick()"
            [style]="nzMaskStyle"
          ></div>
        }
        <div
          class="ant-drawer-content-wrapper {{ nzWrapClassName }}"
          [style.width]="width"
          [style.height]="height"
          [style.transform]="transform"
          [style.transition]="placementChanging ? 'none' : null"
        >
          <div class="ant-drawer-content">
            <div
              class="ant-drawer-wrapper-body"
              [style.height]="isLeftOrRight ? '100%' : null"
            >
              @if (nzTitle || nzClosable) {
                <div
                  class="ant-drawer-header"
                  [class.ant-drawer-header-close-only]="!nzTitle"
                >
                  <div class="ant-drawer-header-title">
                    @if (nzClosable) {
                      <button
                        (click)="closeClick()"
                        aria-label="Close"
                        class="ant-drawer-close"
                      >
                        <ng-container
                          *nzStringTemplateOutlet="nzCloseIcon; let closeIcon"
                        >
                          <nz-icon [nzType]="closeIcon" />
                        </ng-container>
                      </button>
                    }

                    @if (nzTitle) {
                      <div class="ant-drawer-title">
                        <ng-container *nzStringTemplateOutlet="nzTitle">{{
                          nzTitle
                        }}</ng-container>
                      </div>
                    }
                  </div>
                  @if (nzExtra) {
                    <div class="ant-drawer-extra">
                      <ng-container *nzStringTemplateOutlet="nzExtra">{{
                        nzExtra
                      }}</ng-container>
                    </div>
                  }
                </div>
              }
              <div class="ant-drawer-body" [style]="nzBodyStyle" cdkScrollable>
                <ng-template cdkPortalOutlet />
                @if (nzContent) {
                  @if (isTemplateRef(nzContent)) {
                    <ng-container
                      *ngTemplateOutlet="nzContent; context: templateContext"
                    />
                  }
                } @else if (
                  contentFromContentChild && (isOpen || inAnimation)
                ) {
                  <ng-template [ngTemplateOutlet]="contentFromContentChild" />
                }
              </div>
              @if (nzFooter) {
                <div class="ant-drawer-footer">
                  <ng-container *nzStringTemplateOutlet="nzFooter">{{
                    nzFooter
                  }}</ng-container>
                </div>
              }
            </div>
          </div>
        </div>
      </div>
    </ng-template>
  `,
  changeDetection: ChangeDetectionStrategy.OnPush,
  imports: [
    NzNoAnimationDirective,
    NzOutletModule,
    NzIconModule,
    PortalModule,
    NgTemplateOutlet,
    CdkScrollable,
  ],
})
export class NzDrawerComponent<
  T extends {} = NzSafeAny,
  R = NzSafeAny,
  D extends Partial<T> = NzSafeAny,
>
  extends NzDrawerRef<T, R>
  implements OnInit, AfterViewInit, OnChanges, NzDrawerOptionsOfComponent
{
  private cdr = inject(ChangeDetectorRef);
  private renderer = inject(Renderer2);
  private injector = inject(Injector);
  private changeDetectorRef = inject(ChangeDetectorRef);
  private focusTrapFactory = inject(FocusTrapFactory);
  private viewContainerRef = inject(ViewContainerRef);
  private overlayKeyboardDispatcher = inject(OverlayKeyboardDispatcher);
  private directionality = inject(Directionality);
  private destroyRef = inject(DestroyRef);
  private document = inject(DOCUMENT);

  readonly _nzModuleName: NzConfigKey = NZ_CONFIG_MODULE_NAME;

  @Input() nzContent!:
    | TemplateRef<{ $implicit: D; drawerRef: NzDrawerRef<R> }>
    | Type<T>;
  @Input() nzCloseIcon: string | TemplateRef<void> = "close";
  @Input({ transform: booleanAttribute }) nzClosable: boolean = true;
  @Input({ transform: booleanAttribute })
  @WithConfig()
  nzMaskClosable: boolean = true;
  @Input({ transform: booleanAttribute }) @WithConfig() nzMask: boolean = true;
  @Input({ transform: booleanAttribute })
  @WithConfig()
  nzCloseOnNavigation: boolean = true;
  @Input({ transform: booleanAttribute }) nzNoAnimation = false;
  @Input({ transform: booleanAttribute }) nzKeyboard: boolean = true;
  @Input() nzTitle?: string | TemplateRef<{}>;
  @Input() nzExtra?: string | TemplateRef<{}>;
  @Input() nzFooter?: string | TemplateRef<{}>;
  @Input() nzPlacement: NzDrawerPlacement = "right";
  @Input() nzSize: NzDrawerSize = "default";
  @Input() nzMaskStyle: NgStyleInterface = {};
  @Input() nzBodyStyle: NgStyleInterface = {};
  @Input() nzWrapClassName?: string;
  @Input() nzWidth?: number | string;
  @Input() nzHeight?: number | string;
  @Input() nzZIndex = 1000;
  @Input() nzOffsetX = 0;
  @Input() nzOffsetY = 0;
  private componentInstance: T | null = null;
  private componentRef: ComponentRef<T> | null = null;

  @Input({ transform: booleanAttribute })
  set nzVisible(value: boolean) {
    this.isOpen = value;
  }

  get nzVisible(): boolean {
    return this.isOpen;
  }

  @Output() readonly nzOnViewInit = new EventEmitter<void>();
  @Output() readonly nzOnClose = new EventEmitter<MouseEvent>();
  @Output() readonly nzVisibleChange = new EventEmitter<boolean>();

  @ViewChild("drawerTemplate", { static: true })
  drawerTemplate!: TemplateRef<void>;
  @ViewChild(CdkPortalOutlet, { static: false })
  bodyPortalOutlet?: CdkPortalOutlet;
  @ContentChild(NzDrawerContentDirective, { static: true, read: TemplateRef })
  contentFromContentChild?: TemplateRef<NzSafeAny>;

  previouslyFocusedElement?: HTMLElement;
  placementChanging = false;
  placementChangeTimeoutId?: ReturnType<typeof setTimeout>;
  nzContentParams?: NzSafeAny; // only service
  nzData?: D;
  overlayRef?: OverlayRef | null;
  portal?: TemplatePortal;
  focusTrap?: FocusTrap;
  isOpen = false;
  inAnimation = false;
  templateContext: { $implicit: D | undefined; drawerRef: NzDrawerRef<R> } = {
    $implicit: undefined,
    drawerRef: this as NzDrawerRef<R>,
  };
  protected isTemplateRef = isTemplateRef;
  protected readonly maskAnimationEnter = withAnimationCheck(
    () => "ant-drawer-mask-motion-enter",
  );
  protected readonly maskAnimationLeave = withAnimationCheck(
    () => "ant-drawer-mask-motion-leave",
  );

  get offsetTransform(): string | null {
    if (!this.isOpen || this.nzOffsetX + this.nzOffsetY === 0) {
      return null;
    }
    switch (this.nzPlacement) {
      case "left":
        return `translateX(${this.nzOffsetX}px)`;
      case "right":
        return `translateX(-${this.nzOffsetX}px)`;
      case "top":
        return `translateY(${this.nzOffsetY}px)`;
      case "bottom":
        return `translateY(-${this.nzOffsetY}px)`;
    }
  }

  get transform(): string | null {
    if (this.isOpen) {
      return null;
    }

    switch (this.nzPlacement) {
      case "left":
        return `translateX(-100%)`;
      case "right":
        return `translateX(100%)`;
      case "top":
        return `translateY(-100%)`;
      case "bottom":
        return `translateY(100%)`;
    }
  }

  get width(): string | null {
    if (this.isLeftOrRight) {
      const defaultWidth =
        this.nzSize === "large" ? DRAWER_LARGE_SIZE : DRAWER_DEFAULT_SIZE;
      return this.nzWidth === undefined
        ? toCssPixel(defaultWidth)
        : toCssPixel(this.nzWidth);
    }
    return null;
  }

  get height(): string | null {
    if (!this.isLeftOrRight) {
      const defaultHeight =
        this.nzSize === "large" ? DRAWER_LARGE_SIZE : DRAWER_DEFAULT_SIZE;
      return this.nzHeight === undefined
        ? toCssPixel(defaultHeight)
        : toCssPixel(this.nzHeight);
    }
    return null;
  }

  get isLeftOrRight(): boolean {
    return this.nzPlacement === "left" || this.nzPlacement === "right";
  }

  nzAfterOpen = new Subject<void>();
  nzAfterClose = new Subject<R | undefined>();

  get afterOpen(): Observable<void> {
    return this.nzAfterOpen.asObservable();
  }

  get afterClose(): Observable<R | undefined> {
    return this.nzAfterClose.asObservable();
  }

  get isNzContentTemplateRef(): boolean {
    return isTemplateRef(this.nzContent);
  }

  // from service config
  @WithConfig() nzDirection?: Direction = undefined;

  dir: Direction = "ltr";

  constructor() {
    super();
    this.destroyRef.onDestroy(() => {
      clearTimeout(this.placementChangeTimeoutId);
      this.disposeOverlay();
    });
  }

  ngOnInit(): void {
    this.directionality.change
      ?.pipe(takeUntilDestroyed(this.destroyRef))
      .subscribe((direction) => {
        this.dir = direction;
        this.cdr.detectChanges();
      });
    this.dir = this.nzDirection || this.directionality.value;

    this.attachOverlay();
    this.updateOverlayStyle();
    this.updateBodyOverflow();
    this.templateContext = {
      $implicit: this.nzData || this.nzContentParams,
      drawerRef: this as NzDrawerRef<R>,
    };
    this.changeDetectorRef.detectChanges();
  }

  ngAfterViewInit(): void {
    this.attachBodyContent();
    // The `setTimeout` triggers change detection. There's no sense to schedule the DOM timer if anyone is
    // listening to the `nzOnViewInit` event inside the template, for instance `<nz-drawer (nzOnViewInit)="...">`.
    if (this.nzOnViewInit.observers.length) {
      setTimeout(() => {
        this.nzOnViewInit.emit();
      });
    }
  }

  ngOnChanges(changes: SimpleChanges): void {
    const { nzPlacement, nzVisible } = changes;
    if (nzVisible) {
      const value = changes.nzVisible.currentValue;
      if (value) {
        this.open();
      } else {
        this.close();
      }
    }
    if (nzPlacement && !nzPlacement.isFirstChange()) {
      this.triggerPlacementChangeCycleOnce();
    }
  }

  private getAnimationDuration(): number {
    return this.nzNoAnimation ? 0 : DRAWER_ANIMATE_DURATION;
  }

  // Disable the transition animation temporarily when the placement changing
  private triggerPlacementChangeCycleOnce(): void {
    if (!this.nzNoAnimation) {
      this.placementChanging = true;
      this.changeDetectorRef.markForCheck();
      clearTimeout(this.placementChangeTimeoutId);
      this.placementChangeTimeoutId = setTimeout(() => {
        this.placementChanging = false;
        this.changeDetectorRef.markForCheck();
      }, this.getAnimationDuration());
    }
  }

  close(result?: R): void {
    this.isOpen = false;
    this.inAnimation = true;
    this.nzVisibleChange.emit(false);
    this.updateOverlayStyle();
    this.overlayKeyboardDispatcher.remove(this.overlayRef!);
    this.changeDetectorRef.detectChanges();
    setTimeout(() => {
      this.updateBodyOverflow();
      this.restoreFocus();
      this.inAnimation = false;
      this.nzAfterClose.next(result);
      this.nzAfterClose.complete();
      this.componentInstance = null;
      this.componentRef = null;
    }, this.getAnimationDuration());
  }

  open(): void {
    this.attachOverlay();
    this.isOpen = true;
    this.inAnimation = true;
    this.nzVisibleChange.emit(true);
    this.overlayKeyboardDispatcher.add(this.overlayRef!);
    this.updateOverlayStyle();
    this.updateBodyOverflow();
    this.savePreviouslyFocusedElement();
    this.trapFocus();
    this.changeDetectorRef.detectChanges();
    setTimeout(() => {
      this.inAnimation = false;
      this.changeDetectorRef.detectChanges();
      this.nzAfterOpen.next();
    }, this.getAnimationDuration());
  }

  getContentComponent(): T | null {
    return this.componentInstance;
  }

  override getContentComponentRef(): ComponentRef<T> | null {
    return this.componentRef;
  }

  closeClick(): void {
    this.nzOnClose.emit();
  }

  maskClick(): void {
    if (this.nzMaskClosable && this.nzMask) {
      this.nzOnClose.emit();
    }
  }

  private attachBodyContent(): void {
    this.bodyPortalOutlet!.dispose();

    if (this.nzContent instanceof Type) {
      const childInjector = Injector.create({
        parent: this.injector,
        providers: [
          { provide: NzDrawerRef, useValue: this },
          { provide: NZ_DRAWER_DATA, useValue: this.nzData },
        ],
      });
      const componentPortal = new ComponentPortal<T>(
        this.nzContent,
        null,
        childInjector,
      );
      this.componentRef =
        this.bodyPortalOutlet!.attachComponentPortal(componentPortal);

      this.componentInstance = this.componentRef.instance;
      /**TODO
       * When nzContentParam will be remove in the next major version, we have to remove the following line
       * **/
      Object.assign(
        this.componentRef.instance!,
        this.nzData || this.nzContentParams,
      );
      this.componentRef.changeDetectorRef.detectChanges();
    }
  }

  private attachOverlay(): void {
    if (!this.overlayRef) {
      this.portal = new TemplatePortal(
        this.drawerTemplate,
        this.viewContainerRef,
      );
      this.overlayRef = createOverlayRef(this.injector, {
        disposeOnNavigation: this.nzCloseOnNavigation,
        positionStrategy: createGlobalPositionStrategy(this.injector),
        scrollStrategy: createBlockScrollStrategy(this.injector),
      });

      overlayZIndexSetter(this.overlayRef, this.nzZIndex);
    }

    if (this.overlayRef && !this.overlayRef.hasAttached()) {
      this.overlayRef.attach(this.portal);
      this.overlayRef!.keydownEvents()
        .pipe(takeUntilDestroyed(this.destroyRef))
        .subscribe((event: KeyboardEvent) => {
          if (event.keyCode === ESCAPE && this.isOpen && this.nzKeyboard) {
            this.nzOnClose.emit();
          }
        });
      this.overlayRef
        .detachments()
        .pipe(takeUntilDestroyed(this.destroyRef))
        .subscribe(() => {
          this.close();
          this.disposeOverlay();
        });
    }
  }

  private disposeOverlay(): void {
    this.overlayRef?.dispose();
    this.overlayRef = null;
  }

  private updateOverlayStyle(): void {
    if (this.overlayRef && this.overlayRef.overlayElement) {
      this.renderer.setStyle(
        this.overlayRef.overlayElement,
        "pointer-events",
        this.isOpen ? "auto" : "none",
      );
    }
  }

  private updateBodyOverflow(): void {
    if (this.overlayRef) {
      if (this.isOpen) {
        this.overlayRef.getConfig().scrollStrategy!.enable();
      } else {
        this.overlayRef.getConfig().scrollStrategy!.disable();
      }
    }
  }

  savePreviouslyFocusedElement(): void {
    if (this.document && !this.previouslyFocusedElement) {
      this.previouslyFocusedElement = this.document
        .activeElement as HTMLElement;
      this.previouslyFocusedElement?.blur();
    }
  }

  private trapFocus(): void {
    if (!this.focusTrap && this.overlayRef && this.overlayRef.overlayElement) {
      this.focusTrap = this.focusTrapFactory.create(
        this.overlayRef!.overlayElement,
      );
      this.focusTrap.focusInitialElement();
    }
  }

  private restoreFocus(): void {
    if (this.previouslyFocusedElement) {
      this.previouslyFocusedElement.focus();
      this.previouslyFocusedElement = undefined;
    }
    if (this.focusTrap) {
      this.focusTrap.destroy();
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

import { FocusTrap, FocusTrapFactory } from "@angular/cdk/a11y";
import { Direction, Directionality } from "@angular/cdk/bidi";
import { ESCAPE } from "@angular/cdk/keycodes";
import {
  CdkScrollable,
  createBlockScrollStrategy,
  createGlobalPositionStrategy,
  createOverlayRef,
  OverlayKeyboardDispatcher,
  OverlayRef,
} from "@angular/cdk/overlay";
import {
  CdkPortalOutlet,
  ComponentPortal,
  PortalModule,
  TemplatePortal,
} from "@angular/cdk/portal";
import { NgTemplateOutlet } from "@angular/common";
import {
  AfterViewInit,
  booleanAttribute,
  ChangeDetectionStrategy,
  ChangeDetectorRef,
  Component,
  ComponentRef,
  ContentChild,
  DestroyRef,
  DOCUMENT,
  EventEmitter,
  inject,
  Injector,
  Input,
  OnChanges,
  OnInit,
  Output,
  Renderer2,
  SimpleChanges,
  TemplateRef,
  Type,
  ViewChild,
  ViewContainerRef,
} from "@angular/core";
import { takeUntilDestroyed } from "@angular/core/rxjs-interop";
import { Observable, Subject } from "rxjs";

import {
  NzNoAnimationDirective,
  withAnimationCheck,
} from "ng-zorro-antd/core/animation";
import { NzConfigKey, WithConfig } from "ng-zorro-antd/core/config";
import { NzOutletModule } from "ng-zorro-antd/core/outlet";
import { overlayZIndexSetter } from "ng-zorro-antd/core/overlay";
import { NgStyleInterface, NzSafeAny } from "ng-zorro-antd/core/types";
import { isTemplateRef, toCssPixel } from "ng-zorro-antd/core/util";
import { NzIconModule } from "ng-zorro-antd/icon";

import { NzDrawerContentDirective } from "./drawer-content.directive";
import {
  DRAWER_DEFAULT_SIZE,
  DRAWER_LARGE_SIZE,
  NZ_DRAWER_DATA,
  NzDrawerOptionsOfComponent,
  NzDrawerPlacement,
  NzDrawerSize,
} from "./drawer-options";
import { NzDrawerRef } from "./drawer-ref";

export const DRAWER_ANIMATE_DURATION = 300;

const NZ_CONFIG_MODULE_NAME: NzConfigKey = "drawer";

@Component({
  selector: "nz-drawer",
  exportAs: "nzDrawer",
  template: `
    <ng-template #drawerTemplate>
      <div
        class="ant-drawer"
        [nzNoAnimation]="nzNoAnimation"
        [class.ant-drawer-rtl]="dir === 'rtl'"
        [class.ant-drawer-open]="isOpen"
        [class.no-mask]="!nzMask"
        [class.ant-drawer-top]="nzPlacement === 'top'"
        [class.ant-drawer-bottom]="nzPlacement === 'bottom'"
        [class.ant-drawer-right]="nzPlacement === 'right'"
        [class.ant-drawer-left]="nzPlacement === 'left'"
        [style.transform]="offsetTransform"
        [style.transition]="placementChanging ? 'none' : null"
        [style.zIndex]="nzZIndex"
      >
        @if (nzMask && isOpen) {
          <div
            class="ant-drawer-mask"
            [animate.enter]="maskAnimationEnter()"
            [animate.leave]="maskAnimationLeave()"
            (click)="maskClick()"
            [style]="nzMaskStyle"
          ></div>
        }
        <div
          class="ant-drawer-content-wrapper {{ nzWrapClassName }}"
          [style.width]="width"
          [style.height]="height"
          [style.transform]="transform"
          [style.transition]="placementChanging ? 'none' : null"
        >
          <div class="ant-drawer-content">
            <div
              class="ant-drawer-wrapper-body"
              [style.height]="isLeftOrRight ? '100%' : null"
            >
              @if (nzTitle || nzClosable) {
                <div
                  class="ant-drawer-header"
                  [class.ant-drawer-header-close-only]="!nzTitle"
                >
                  <div class="ant-drawer-header-title">
                    @if (nzClosable) {
                      <button
                        (click)="closeClick()"
                        aria-label="Close"
                        class="ant-drawer-close"
                      >
                        <ng-container
                          *nzStringTemplateOutlet="nzCloseIcon; let closeIcon"
                        >
                          <nz-icon [nzType]="closeIcon" />
                        </ng-container>
                      </button>
                    }

                    @if (nzTitle) {
                      <div class="ant-drawer-title">
                        <ng-container *nzStringTemplateOutlet="nzTitle">{{
                          nzTitle
                        }}</ng-container>
                      </div>
                    }
                  </div>
                  @if (nzExtra) {
                    <div class="ant-drawer-extra">
                      <ng-container *nzStringTemplateOutlet="nzExtra">{{
                        nzExtra
                      }}</ng-container>
                    </div>
                  }
                </div>
              }
              <div class="ant-drawer-body" [style]="nzBodyStyle" cdkScrollable>
                <ng-template cdkPortalOutlet />
                @if (nzContent) {
                  @if (isTemplateRef(nzContent)) {
                    <ng-container
                      *ngTemplateOutlet="nzContent; context: templateContext"
                    />
                  }
                } @else if (
                  contentFromContentChild && (isOpen || inAnimation)
                ) {
                  <ng-template [ngTemplateOutlet]="contentFromContentChild" />
                }
              </div>
              @if (nzFooter) {
                <div class="ant-drawer-footer">
                  <ng-container *nzStringTemplateOutlet="nzFooter">{{
                    nzFooter
                  }}</ng-container>
                </div>
              }
            </div>
          </div>
        </div>
      </div>
    </ng-template>
  `,
  changeDetection: ChangeDetectionStrategy.OnPush,
  imports: [
    NzNoAnimationDirective,
    NzOutletModule,
    NzIconModule,
    PortalModule,
    NgTemplateOutlet,
    CdkScrollable,
  ],
})
export class NzDrawerComponent<
  T extends {} = NzSafeAny,
  R = NzSafeAny,
  D extends Partial<T> = NzSafeAny,
>
  extends NzDrawerRef<T, R>
  implements OnInit, AfterViewInit, OnChanges, NzDrawerOptionsOfComponent
{
  private cdr = inject(ChangeDetectorRef);
  private renderer = inject(Renderer2);
  private injector = inject(Injector);
  private changeDetectorRef = inject(ChangeDetectorRef);
  private focusTrapFactory = inject(FocusTrapFactory);
  private viewContainerRef = inject(ViewContainerRef);
  private overlayKeyboardDispatcher = inject(OverlayKeyboardDispatcher);
  private directionality = inject(Directionality);
  private destroyRef = inject(DestroyRef);
  private document = inject(DOCUMENT);

  readonly _nzModuleName: NzConfigKey = NZ_CONFIG_MODULE_NAME;

  @Input() nzContent!:
    TemplateRef<{ $implicit: D; drawerRef: NzDrawerRef<R> }> | Type<T>;
  @Input() nzCloseIcon: string | TemplateRef<void> = "close";
  @Input({ transform: booleanAttribute }) nzClosable: boolean = true;
  @Input({ transform: booleanAttribute })
  @WithConfig()
  nzMaskClosable: boolean = true;
  @Input({ transform: booleanAttribute }) @WithConfig() nzMask: boolean = true;
  @Input({ transform: booleanAttribute })
  @WithConfig()
  nzCloseOnNavigation: boolean = true;
  @Input({ transform: booleanAttribute }) nzNoAnimation = false;
  @Input({ transform: booleanAttribute }) nzKeyboard: boolean = true;
  @Input() nzTitle?: string | TemplateRef<{}>;
  @Input() nzExtra?: string | TemplateRef<{}>;
  @Input() nzFooter?: string | TemplateRef<{}>;
  @Input() nzPlacement: NzDrawerPlacement = "right";
  @Input() nzSize: NzDrawerSize = "default";
  @Input() nzMaskStyle: NgStyleInterface = {};
  @Input() nzBodyStyle: NgStyleInterface = {};
  @Input() nzWrapClassName?: string;
  @Input() nzWidth?: number | string;
  @Input() nzHeight?: number | string;
  @Input() nzZIndex = 1000;
  @Input() nzOffsetX = 0;
  @Input() nzOffsetY = 0;
  private componentInstance: T | null = null;
  private componentRef: ComponentRef<T> | null = null;

  @Input({ transform: booleanAttribute })
  set nzVisible(value: boolean) {
    this.isOpen = value;
  }

  get nzVisible(): boolean {
    return this.isOpen;
  }

  @Output() readonly nzOnViewInit = new EventEmitter<void>();
  @Output() readonly nzOnClose = new EventEmitter<MouseEvent>();
  @Output() readonly nzVisibleChange = new EventEmitter<boolean>();

  @ViewChild("drawerTemplate", { static: true })
  drawerTemplate!: TemplateRef<void>;
  @ViewChild(CdkPortalOutlet, { static: false })
  bodyPortalOutlet?: CdkPortalOutlet;
  @ContentChild(NzDrawerContentDirective, { static: true, read: TemplateRef })
  contentFromContentChild?: TemplateRef<NzSafeAny>;

  previouslyFocusedElement?: HTMLElement;
  placementChanging = false;
  placementChangeTimeoutId?: ReturnType<typeof setTimeout>;
  nzContentParams?: NzSafeAny; // only service
  nzData?: D;
  overlayRef?: OverlayRef | null;
  portal?: TemplatePortal;
  focusTrap?: FocusTrap;
  isOpen = false;
  inAnimation = false;
  templateContext: { $implicit: D | undefined; drawerRef: NzDrawerRef<R> } = {
    $implicit: undefined,
    drawerRef: this as NzDrawerRef<R>,
  };
  protected isTemplateRef = isTemplateRef;
  protected readonly maskAnimationEnter = withAnimationCheck(
    () => "ant-drawer-mask-motion-enter",
  );
  protected readonly maskAnimationLeave = withAnimationCheck(
    () => "ant-drawer-mask-motion-leave",
  );

  get offsetTransform(): string | null {
    if (!this.isOpen || this.nzOffsetX + this.nzOffsetY === 0) {
      return null;
    }
    switch (this.nzPlacement) {
      case "left":
        return `translateX(${this.nzOffsetX}px)`;
      case "right":
        return `translateX(-${this.nzOffsetX}px)`;
      case "top":
        return `translateY(${this.nzOffsetY}px)`;
      case "bottom":
        return `translateY(-${this.nzOffsetY}px)`;
    }
  }

  get transform(): string | null {
    if (this.isOpen) {
      return null;
    }

    switch (this.nzPlacement) {
      case "left":
        return `translateX(-100%)`;
      case "right":
        return `translateX(100%)`;
      case "top":
        return `translateY(-100%)`;
      case "bottom":
        return `translateY(100%)`;
    }
  }

  get width(): string | null {
    if (this.isLeftOrRight) {
      const defaultWidth =
        this.nzSize === "large" ? DRAWER_LARGE_SIZE : DRAWER_DEFAULT_SIZE;
      return this.nzWidth === undefined
        ? toCssPixel(defaultWidth)
        : toCssPixel(this.nzWidth);
    }
    return null;
  }

  get height(): string | null {
    if (!this.isLeftOrRight) {
      const defaultHeight =
        this.nzSize === "large" ? DRAWER_LARGE_SIZE : DRAWER_DEFAULT_SIZE;
      return this.nzHeight === undefined
        ? toCssPixel(defaultHeight)
        : toCssPixel(this.nzHeight);
    }
    return null;
  }

  get isLeftOrRight(): boolean {
    return this.nzPlacement === "left" || this.nzPlacement === "right";
  }

  nzAfterOpen = new Subject<void>();
  nzAfterClose = new Subject<R | undefined>();

  get afterOpen(): Observable<void> {
    return this.nzAfterOpen.asObservable();
  }

  get afterClose(): Observable<R | undefined> {
    return this.nzAfterClose.asObservable();
  }

  get isNzContentTemplateRef(): boolean {
    return isTemplateRef(this.nzContent);
  }

  // from service config
  @WithConfig() nzDirection?: Direction = undefined;

  dir: Direction = "ltr";

  constructor() {
    super();
    this.destroyRef.onDestroy(() => {
      clearTimeout(this.placementChangeTimeoutId);
      this.disposeOverlay();
    });
  }

  ngOnInit(): void {
    this.directionality.change
      ?.pipe(takeUntilDestroyed(this.destroyRef))
      .subscribe((direction) => {
        this.dir = direction;
        this.cdr.detectChanges();
      });
    this.dir = this.nzDirection || this.directionality.value;

    this.attachOverlay();
    this.updateOverlayStyle();
    this.updateBodyOverflow();
    this.templateContext = {
      $implicit: this.nzData || this.nzContentParams,
      drawerRef: this as NzDrawerRef<R>,
    };
    this.changeDetectorRef.detectChanges();
  }

  ngAfterViewInit(): void {
    this.attachBodyContent();
    // The `setTimeout` triggers change detection. There's no sense to schedule the DOM timer if anyone is
    // listening to the `nzOnViewInit` event inside the template, for instance `<nz-drawer (nzOnViewInit)="...">`.
    if (this.nzOnViewInit.observers.length) {
      setTimeout(() => {
        this.nzOnViewInit.emit();
      });
    }
  }

  ngOnChanges(changes: SimpleChanges): void {
    const { nzPlacement, nzVisible } = changes;
    if (nzVisible) {
      const value = changes.nzVisible.currentValue;
      if (value) {
        this.open();
      } else {
        this.close();
      }
    }
    if (nzPlacement && !nzPlacement.isFirstChange()) {
      this.triggerPlacementChangeCycleOnce();
    }
  }

  private getAnimationDuration(): number {
    return this.nzNoAnimation ? 0 : DRAWER_ANIMATE_DURATION;
  }

  // Disable the transition animation temporarily when the placement changing
  private triggerPlacementChangeCycleOnce(): void {
    if (!this.nzNoAnimation) {
      this.placementChanging = true;
      this.changeDetectorRef.markForCheck();
      clearTimeout(this.placementChangeTimeoutId);
      this.placementChangeTimeoutId = setTimeout(() => {
        this.placementChanging = false;
        this.changeDetectorRef.markForCheck();
      }, this.getAnimationDuration());
    }
  }

  close(result?: R): void {
    this.isOpen = false;
    this.inAnimation = true;
    this.nzVisibleChange.emit(false);
    this.updateOverlayStyle();
    this.overlayKeyboardDispatcher.remove(this.overlayRef!);
    this.changeDetectorRef.detectChanges();
    setTimeout(() => {
      this.updateBodyOverflow();
      this.restoreFocus();
      this.inAnimation = false;
      this.nzAfterClose.next(result);
      this.nzAfterClose.complete();
      this.componentInstance = null;
      this.componentRef = null;
    }, this.getAnimationDuration());
  }

  open(): void {
    this.attachOverlay();
    this.isOpen = true;
    this.inAnimation = true;
    this.nzVisibleChange.emit(true);
    this.overlayKeyboardDispatcher.add(this.overlayRef!);
    this.updateOverlayStyle();
    this.updateBodyOverflow();
    this.savePreviouslyFocusedElement();
    this.trapFocus();
    this.changeDetectorRef.detectChanges();
    setTimeout(() => {
      this.inAnimation = false;
      this.changeDetectorRef.detectChanges();
      this.nzAfterOpen.next();
    }, this.getAnimationDuration());
  }

  getContentComponent(): T | null {
    return this.componentInstance;
  }

  override getContentComponentRef(): ComponentRef<T> | null {
    return this.componentRef;
  }

  closeClick(): void {
    this.nzOnClose.emit();
  }

  maskClick(): void {
    if (this.nzMaskClosable && this.nzMask) {
      this.nzOnClose.emit();
    }
  }

  private attachBodyContent(): void {
    this.bodyPortalOutlet!.dispose();

    if (this.nzContent instanceof Type) {
      const childInjector = Injector.create({
        parent: this.injector,
        providers: [
          { provide: NzDrawerRef, useValue: this },
          { provide: NZ_DRAWER_DATA, useValue: this.nzData },
        ],
      });
      const componentPortal = new ComponentPortal<T>(
        this.nzContent,
        null,
        childInjector,
      );
      this.componentRef =
        this.bodyPortalOutlet!.attachComponentPortal(componentPortal);

      this.componentInstance = this.componentRef.instance;
      /**TODO
       * When nzContentParam will be remove in the next major version, we have to remove the following line
       * **/
      Object.assign(
        this.componentRef.instance!,
        this.nzData || this.nzContentParams,
      );
      this.componentRef.changeDetectorRef.detectChanges();
    }
  }

  private attachOverlay(): void {
    if (!this.overlayRef) {
      this.portal = new TemplatePortal(
        this.drawerTemplate,
        this.viewContainerRef,
      );
      this.overlayRef = createOverlayRef(this.injector, {
        disposeOnNavigation: this.nzCloseOnNavigation,
        positionStrategy: createGlobalPositionStrategy(this.injector),
        scrollStrategy: createBlockScrollStrategy(this.injector),
      });

      overlayZIndexSetter(this.overlayRef, this.nzZIndex);
    }

    if (this.overlayRef && !this.overlayRef.hasAttached()) {
      this.overlayRef.attach(this.portal);
      this.overlayRef!.keydownEvents()
        .pipe(takeUntilDestroyed(this.destroyRef))
        .subscribe((event: KeyboardEvent) => {
          if (event.keyCode === ESCAPE && this.isOpen && this.nzKeyboard) {
            this.nzOnClose.emit();
          }
        });
      this.overlayRef
        .detachments()
        .pipe(takeUntilDestroyed(this.destroyRef))
        .subscribe(() => {
          this.close();
          this.disposeOverlay();
        });
    }
  }

  private disposeOverlay(): void {
    this.overlayRef?.dispose();
    this.overlayRef = null;
  }

  private updateOverlayStyle(): void {
    if (this.overlayRef && this.overlayRef.overlayElement) {
      this.renderer.setStyle(
        this.overlayRef.overlayElement,
        "pointer-events",
        this.isOpen ? "auto" : "none",
      );
    }
  }

  private updateBodyOverflow(): void {
    if (this.overlayRef) {
      if (this.isOpen) {
        this.overlayRef.getConfig().scrollStrategy!.enable();
      } else {
        this.overlayRef.getConfig().scrollStrategy!.disable();
      }
    }
  }

  savePreviouslyFocusedElement(): void {
    if (this.document && !this.previouslyFocusedElement) {
      this.previouslyFocusedElement = this.document
        .activeElement as HTMLElement;
      this.previouslyFocusedElement?.blur();
    }
  }

  private trapFocus(): void {
    if (!this.focusTrap && this.overlayRef && this.overlayRef.overlayElement) {
      this.focusTrap = this.focusTrapFactory.create(
        this.overlayRef!.overlayElement,
      );
      this.focusTrap.focusInitialElement();
    }
  }

  private restoreFocus(): void {
    if (this.previouslyFocusedElement) {
      this.previouslyFocusedElement.focus();
      this.previouslyFocusedElement = undefined;
    }
    if (this.focusTrap) {
      this.focusTrap.destroy();
    }
  }
}

`````
