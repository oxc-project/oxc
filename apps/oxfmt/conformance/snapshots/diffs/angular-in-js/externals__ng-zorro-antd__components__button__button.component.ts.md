# externals/ng-zorro-antd/components/button/button.component.ts

## Option 1

`````json
{"printWidth":80}
`````

### Diff

`````diff
===================================================================
--- prettier
+++ oxfmt
@@ -46,9 +46,14 @@
   NzSpaceCompactItemDirective,
 } from "ng-zorro-antd/space";
 
 export type NzButtonType =
-  "primary" | "default" | "dashed" | "link" | "text" | null;
+  | "primary"
+  | "default"
+  | "dashed"
+  | "link"
+  | "text"
+  | null;
 export type NzButtonShape = "circle" | "round" | null;
 export type NzButtonSize = NzSizeLDSType;
 
 const NZ_CONFIG_MODULE_NAME: NzConfigKey = "button";

`````

### Actual (oxfmt)

`````ts
/**
 * Use of this source code is governed by an MIT-style license that can be
 * found in the LICENSE file at https://github.com/NG-ZORRO/ng-zorro-antd/blob/master/LICENSE
 */

import { Directionality } from "@angular/cdk/bidi";
import {
  AfterContentInit,
  afterEveryRender,
  AfterViewInit,
  booleanAttribute,
  ChangeDetectionStrategy,
  ChangeDetectorRef,
  Component,
  computed,
  contentChild,
  ContentChild,
  DestroyRef,
  ElementRef,
  inject,
  Input,
  OnChanges,
  OnInit,
  Renderer2,
  signal,
  SimpleChanges,
  viewChild,
  ViewEncapsulation,
} from "@angular/core";
import { takeUntilDestroyed } from "@angular/core/rxjs-interop";
import { Subject } from "rxjs";
import { filter, startWith } from "rxjs/operators";

import {
  NzConfigKey,
  onConfigChangeEventForComponent,
  WithConfig,
} from "ng-zorro-antd/core/config";
import { NZ_FORM_SIZE } from "ng-zorro-antd/core/form";
import { NzSizeLDSType } from "ng-zorro-antd/core/types";
import { fromEventOutsideAngular } from "ng-zorro-antd/core/util";
import { NzIconDirective, NzIconModule } from "ng-zorro-antd/icon";
import {
  NZ_SPACE_COMPACT_ITEM_TYPE,
  NZ_SPACE_COMPACT_SIZE,
  NzSpaceCompactItemDirective,
} from "ng-zorro-antd/space";

export type NzButtonType =
  | "primary"
  | "default"
  | "dashed"
  | "link"
  | "text"
  | null;
export type NzButtonShape = "circle" | "round" | null;
export type NzButtonSize = NzSizeLDSType;

const NZ_CONFIG_MODULE_NAME: NzConfigKey = "button";

@Component({
  selector: "button[nz-button], a[nz-button]",
  exportAs: "nzButton",
  imports: [NzIconModule],
  changeDetection: ChangeDetectionStrategy.OnPush,
  encapsulation: ViewEncapsulation.None,
  template: `
    @if (nzLoading) {
      <span class="ant-btn-icon ant-btn-loading-icon">
        <nz-icon nzType="loading" />
      </span>
    }
    <ng-content />
  `,
  host: {
    class: "ant-btn",
    "[class.ant-btn-default]": `nzType === 'default'`,
    "[class.ant-btn-primary]": `nzType === 'primary'`,
    "[class.ant-btn-dashed]": `nzType === 'dashed'`,
    "[class.ant-btn-link]": `nzType === 'link'`,
    "[class.ant-btn-text]": `nzType === 'text'`,
    "[class.ant-btn-circle]": `nzShape === 'circle'`,
    "[class.ant-btn-round]": `nzShape === 'round'`,
    "[class.ant-btn-lg]": `finalSize() === 'large'`,
    "[class.ant-btn-sm]": `finalSize() === 'small'`,
    "[class.ant-btn-dangerous]": `nzDanger`,
    "[class.ant-btn-loading]": `nzLoading`,
    "[class.ant-btn-background-ghost]": `nzGhost`,
    "[class.ant-btn-block]": `nzBlock`,
    "[class.ant-input-search-button]": `nzSearch`,
    "[class.ant-btn-rtl]": `dir() === 'rtl'`,
    "[class.ant-btn-icon-only]": `iconOnly()`,
    "[attr.tabindex]": "disabled ? -1 : (tabIndex === null ? null : tabIndex)",
    "[attr.disabled]": "disabled || null",
  },
  hostDirectives: [NzSpaceCompactItemDirective],
  providers: [{ provide: NZ_SPACE_COMPACT_ITEM_TYPE, useValue: "btn" }],
})
export class NzButtonComponent
  implements OnChanges, AfterViewInit, AfterContentInit, OnInit
{
  private elementRef: ElementRef<HTMLButtonElement | HTMLAnchorElement> =
    inject(ElementRef);
  private cdr = inject(ChangeDetectorRef);
  private renderer = inject(Renderer2);
  private destroyRef = inject(DestroyRef);
  readonly _nzModuleName: NzConfigKey = NZ_CONFIG_MODULE_NAME;

  @ContentChild(NzIconDirective, { read: ElementRef })
  nzIconDirectiveElement!: ElementRef;
  @Input({ transform: booleanAttribute }) nzBlock: boolean = false;
  @Input({ transform: booleanAttribute }) nzGhost: boolean = false;
  /**
   * @deprecated Will be removed in v22.0.0. Please use `nz-input-search` instead.
   */
  @Input({ transform: booleanAttribute }) nzSearch: boolean = false;
  @Input({ transform: booleanAttribute }) nzLoading: boolean = false;
  @Input({ transform: booleanAttribute }) nzDanger: boolean = false;
  @Input({ transform: booleanAttribute }) disabled: boolean = false;
  @Input() tabIndex: number | string | null = null;
  @Input() nzType: NzButtonType = null;
  @Input() nzShape: NzButtonShape = null;
  @Input() @WithConfig() nzSize: NzButtonSize = "default";
  protected readonly dir = inject(Directionality).valueSignal;

  private readonly elementOnly = signal(false);
  private readonly size = signal<NzSizeLDSType>(this.nzSize);

  private readonly formSize = inject(NZ_FORM_SIZE, { optional: true });

  private readonly compactSize = inject(NZ_SPACE_COMPACT_SIZE, {
    optional: true,
  });
  private readonly loading$ = new Subject<boolean>();

  protected readonly finalSize = computed(() => {
    if (this.formSize?.()) {
      return this.formSize();
    }
    if (this.compactSize) {
      return this.compactSize();
    }
    return this.size();
  });

  readonly iconDir = contentChild(NzIconDirective);
  readonly loadingIconDir = viewChild(NzIconDirective);

  readonly iconOnly = computed(
    () => this.elementOnly() && (!!this.iconDir() || !!this.loadingIconDir()),
  );

  constructor() {
    onConfigChangeEventForComponent(NZ_CONFIG_MODULE_NAME, () => {
      this.size.set(this.nzSize);
      this.cdr.markForCheck();
    });

    afterEveryRender({
      read: () => {
        const { children } = this.elementRef.nativeElement;
        const visibleElement = Array.from(children).filter(
          (element) => (element as HTMLElement).style.display !== "none",
        );
        this.elementOnly.set(visibleElement.length === 1);
      },
    });
  }

  ngOnInit(): void {
    this.size.set(this.nzSize);

    // Caretaker note: this event listener could've been added through `host.click` or `HostListener`.
    // The compiler generates the `ɵɵlistener` instruction which wraps the actual listener internally into the
    // function, which runs `markDirty()` before running the actual listener (the decorated class method).
    // Since we're preventing the default behavior and stopping event propagation this doesn't require Angular to run the change detection.
    fromEventOutsideAngular<MouseEvent>(
      this.elementRef.nativeElement,
      "click",
      { capture: true },
    )
      .pipe(takeUntilDestroyed(this.destroyRef))
      .subscribe((event) => {
        if (
          (this.disabled && (event.target as HTMLElement)?.tagName === "A") ||
          this.nzLoading
        ) {
          event.preventDefault();
          event.stopImmediatePropagation();
        }
      });
  }

  ngOnChanges({ nzLoading, nzSize }: SimpleChanges): void {
    if (nzLoading) {
      this.loading$.next(this.nzLoading);
    }
    if (nzSize) {
      this.size.set(nzSize.currentValue);
    }
  }

  ngAfterViewInit(): void {
    this.insertSpan();
  }

  ngAfterContentInit(): void {
    this.loading$
      .pipe(
        startWith(this.nzLoading),
        filter(() => !!this.nzIconDirectiveElement),
        takeUntilDestroyed(this.destroyRef),
      )
      .subscribe((loading) => {
        const nativeElement = this.nzIconDirectiveElement.nativeElement;
        if (loading) {
          this.renderer.setStyle(nativeElement, "display", "none");
        } else {
          this.renderer.removeStyle(nativeElement, "display");
        }
      });
  }

  insertSpan(): void {
    this.elementRef.nativeElement.childNodes.forEach((node) => {
      if (
        node.nodeType === Node.TEXT_NODE &&
        node.textContent!.trim().length > 0
      ) {
        const span = this.renderer.createElement("span");
        const parent = this.renderer.parentNode(node);
        this.renderer.insertBefore(parent, span, node);
        this.renderer.appendChild(span, node);
      }
    });
  }
}

`````

### Expected (prettier)

`````ts
/**
 * Use of this source code is governed by an MIT-style license that can be
 * found in the LICENSE file at https://github.com/NG-ZORRO/ng-zorro-antd/blob/master/LICENSE
 */

import { Directionality } from "@angular/cdk/bidi";
import {
  AfterContentInit,
  afterEveryRender,
  AfterViewInit,
  booleanAttribute,
  ChangeDetectionStrategy,
  ChangeDetectorRef,
  Component,
  computed,
  contentChild,
  ContentChild,
  DestroyRef,
  ElementRef,
  inject,
  Input,
  OnChanges,
  OnInit,
  Renderer2,
  signal,
  SimpleChanges,
  viewChild,
  ViewEncapsulation,
} from "@angular/core";
import { takeUntilDestroyed } from "@angular/core/rxjs-interop";
import { Subject } from "rxjs";
import { filter, startWith } from "rxjs/operators";

import {
  NzConfigKey,
  onConfigChangeEventForComponent,
  WithConfig,
} from "ng-zorro-antd/core/config";
import { NZ_FORM_SIZE } from "ng-zorro-antd/core/form";
import { NzSizeLDSType } from "ng-zorro-antd/core/types";
import { fromEventOutsideAngular } from "ng-zorro-antd/core/util";
import { NzIconDirective, NzIconModule } from "ng-zorro-antd/icon";
import {
  NZ_SPACE_COMPACT_ITEM_TYPE,
  NZ_SPACE_COMPACT_SIZE,
  NzSpaceCompactItemDirective,
} from "ng-zorro-antd/space";

export type NzButtonType =
  "primary" | "default" | "dashed" | "link" | "text" | null;
export type NzButtonShape = "circle" | "round" | null;
export type NzButtonSize = NzSizeLDSType;

const NZ_CONFIG_MODULE_NAME: NzConfigKey = "button";

@Component({
  selector: "button[nz-button], a[nz-button]",
  exportAs: "nzButton",
  imports: [NzIconModule],
  changeDetection: ChangeDetectionStrategy.OnPush,
  encapsulation: ViewEncapsulation.None,
  template: `
    @if (nzLoading) {
      <span class="ant-btn-icon ant-btn-loading-icon">
        <nz-icon nzType="loading" />
      </span>
    }
    <ng-content />
  `,
  host: {
    class: "ant-btn",
    "[class.ant-btn-default]": `nzType === 'default'`,
    "[class.ant-btn-primary]": `nzType === 'primary'`,
    "[class.ant-btn-dashed]": `nzType === 'dashed'`,
    "[class.ant-btn-link]": `nzType === 'link'`,
    "[class.ant-btn-text]": `nzType === 'text'`,
    "[class.ant-btn-circle]": `nzShape === 'circle'`,
    "[class.ant-btn-round]": `nzShape === 'round'`,
    "[class.ant-btn-lg]": `finalSize() === 'large'`,
    "[class.ant-btn-sm]": `finalSize() === 'small'`,
    "[class.ant-btn-dangerous]": `nzDanger`,
    "[class.ant-btn-loading]": `nzLoading`,
    "[class.ant-btn-background-ghost]": `nzGhost`,
    "[class.ant-btn-block]": `nzBlock`,
    "[class.ant-input-search-button]": `nzSearch`,
    "[class.ant-btn-rtl]": `dir() === 'rtl'`,
    "[class.ant-btn-icon-only]": `iconOnly()`,
    "[attr.tabindex]": "disabled ? -1 : (tabIndex === null ? null : tabIndex)",
    "[attr.disabled]": "disabled || null",
  },
  hostDirectives: [NzSpaceCompactItemDirective],
  providers: [{ provide: NZ_SPACE_COMPACT_ITEM_TYPE, useValue: "btn" }],
})
export class NzButtonComponent
  implements OnChanges, AfterViewInit, AfterContentInit, OnInit
{
  private elementRef: ElementRef<HTMLButtonElement | HTMLAnchorElement> =
    inject(ElementRef);
  private cdr = inject(ChangeDetectorRef);
  private renderer = inject(Renderer2);
  private destroyRef = inject(DestroyRef);
  readonly _nzModuleName: NzConfigKey = NZ_CONFIG_MODULE_NAME;

  @ContentChild(NzIconDirective, { read: ElementRef })
  nzIconDirectiveElement!: ElementRef;
  @Input({ transform: booleanAttribute }) nzBlock: boolean = false;
  @Input({ transform: booleanAttribute }) nzGhost: boolean = false;
  /**
   * @deprecated Will be removed in v22.0.0. Please use `nz-input-search` instead.
   */
  @Input({ transform: booleanAttribute }) nzSearch: boolean = false;
  @Input({ transform: booleanAttribute }) nzLoading: boolean = false;
  @Input({ transform: booleanAttribute }) nzDanger: boolean = false;
  @Input({ transform: booleanAttribute }) disabled: boolean = false;
  @Input() tabIndex: number | string | null = null;
  @Input() nzType: NzButtonType = null;
  @Input() nzShape: NzButtonShape = null;
  @Input() @WithConfig() nzSize: NzButtonSize = "default";
  protected readonly dir = inject(Directionality).valueSignal;

  private readonly elementOnly = signal(false);
  private readonly size = signal<NzSizeLDSType>(this.nzSize);

  private readonly formSize = inject(NZ_FORM_SIZE, { optional: true });

  private readonly compactSize = inject(NZ_SPACE_COMPACT_SIZE, {
    optional: true,
  });
  private readonly loading$ = new Subject<boolean>();

  protected readonly finalSize = computed(() => {
    if (this.formSize?.()) {
      return this.formSize();
    }
    if (this.compactSize) {
      return this.compactSize();
    }
    return this.size();
  });

  readonly iconDir = contentChild(NzIconDirective);
  readonly loadingIconDir = viewChild(NzIconDirective);

  readonly iconOnly = computed(
    () => this.elementOnly() && (!!this.iconDir() || !!this.loadingIconDir()),
  );

  constructor() {
    onConfigChangeEventForComponent(NZ_CONFIG_MODULE_NAME, () => {
      this.size.set(this.nzSize);
      this.cdr.markForCheck();
    });

    afterEveryRender({
      read: () => {
        const { children } = this.elementRef.nativeElement;
        const visibleElement = Array.from(children).filter(
          (element) => (element as HTMLElement).style.display !== "none",
        );
        this.elementOnly.set(visibleElement.length === 1);
      },
    });
  }

  ngOnInit(): void {
    this.size.set(this.nzSize);

    // Caretaker note: this event listener could've been added through `host.click` or `HostListener`.
    // The compiler generates the `ɵɵlistener` instruction which wraps the actual listener internally into the
    // function, which runs `markDirty()` before running the actual listener (the decorated class method).
    // Since we're preventing the default behavior and stopping event propagation this doesn't require Angular to run the change detection.
    fromEventOutsideAngular<MouseEvent>(
      this.elementRef.nativeElement,
      "click",
      { capture: true },
    )
      .pipe(takeUntilDestroyed(this.destroyRef))
      .subscribe((event) => {
        if (
          (this.disabled && (event.target as HTMLElement)?.tagName === "A") ||
          this.nzLoading
        ) {
          event.preventDefault();
          event.stopImmediatePropagation();
        }
      });
  }

  ngOnChanges({ nzLoading, nzSize }: SimpleChanges): void {
    if (nzLoading) {
      this.loading$.next(this.nzLoading);
    }
    if (nzSize) {
      this.size.set(nzSize.currentValue);
    }
  }

  ngAfterViewInit(): void {
    this.insertSpan();
  }

  ngAfterContentInit(): void {
    this.loading$
      .pipe(
        startWith(this.nzLoading),
        filter(() => !!this.nzIconDirectiveElement),
        takeUntilDestroyed(this.destroyRef),
      )
      .subscribe((loading) => {
        const nativeElement = this.nzIconDirectiveElement.nativeElement;
        if (loading) {
          this.renderer.setStyle(nativeElement, "display", "none");
        } else {
          this.renderer.removeStyle(nativeElement, "display");
        }
      });
  }

  insertSpan(): void {
    this.elementRef.nativeElement.childNodes.forEach((node) => {
      if (
        node.nodeType === Node.TEXT_NODE &&
        node.textContent!.trim().length > 0
      ) {
        const span = this.renderer.createElement("span");
        const parent = this.renderer.parentNode(node);
        this.renderer.insertBefore(parent, span, node);
        this.renderer.appendChild(span, node);
      }
    });
  }
}

`````
