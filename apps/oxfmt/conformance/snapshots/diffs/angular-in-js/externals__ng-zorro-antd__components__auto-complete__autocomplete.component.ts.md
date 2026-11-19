# externals/ng-zorro-antd/components/auto-complete/autocomplete.component.ts

## Option 1

`````json
{"printWidth":80}
`````

### Diff

`````diff
===================================================================
--- prettier
+++ oxfmt
@@ -309,13 +309,9 @@
   }
 
   getOptionIndex(value: NzSafeAny): number {
     return this.options.reduce(
-      (
-        result: number,
-        current: NzAutocompleteOptionComponent,
-        index: number,
-      ) =>
+      (result: number, current: NzAutocompleteOptionComponent, index: number) =>
         result === -1
           ? this.compareWith(value, current.nzValue)
             ? index
             : -1

`````

### Actual (oxfmt)

`````ts
/**
 * Use of this source code is governed by an MIT-style license that can be
 * found in the LICENSE file at https://github.com/NG-ZORRO/ng-zorro-antd/blob/master/LICENSE
 */

import { Direction, Directionality } from "@angular/cdk/bidi";
import { NgTemplateOutlet } from "@angular/common";
import {
  AfterContentInit,
  AfterViewInit,
  ChangeDetectionStrategy,
  ChangeDetectorRef,
  Component,
  ContentChildren,
  DestroyRef,
  ElementRef,
  EventEmitter,
  Input,
  OnChanges,
  OnInit,
  Output,
  QueryList,
  SimpleChanges,
  TemplateRef,
  ViewChild,
  ViewChildren,
  ViewEncapsulation,
  booleanAttribute,
  inject,
  type AnimationCallbackEvent,
} from "@angular/core";
import { takeUntilDestroyed } from "@angular/core/rxjs-interop";
import { Observable, Subscription, defer, merge } from "rxjs";
import { filter, switchMap } from "rxjs/operators";

import {
  NzNoAnimationDirective,
  isAnimationEnabled,
  slideAnimationEnter,
  slideAnimationLeave,
} from "ng-zorro-antd/core/animation";
import { NZ_AFTER_NEXT_RENDER$ } from "ng-zorro-antd/core/render";
import { CompareWith, NzSafeAny } from "ng-zorro-antd/core/types";
import { numberAttributeWithZeroFallback } from "ng-zorro-antd/core/util";

import {
  NzAutocompleteOptionComponent,
  NzOptionSelectionChange,
} from "./autocomplete-option.component";

export interface AutocompleteDataSourceItem {
  value: string;
  label: string;
}

export type AutocompleteDataSource = Array<
  AutocompleteDataSourceItem | string | number
>;

function normalizeDataSource(
  value: AutocompleteDataSource,
): AutocompleteDataSourceItem[] {
  return value?.map((item) => {
    if (typeof item === "number" || typeof item === "string") {
      return {
        label: item.toString(),
        value: item.toString(),
      };
    }
    return item;
  });
}

@Component({
  selector: "nz-autocomplete",
  exportAs: "nzAutocomplete",
  changeDetection: ChangeDetectionStrategy.OnPush,
  encapsulation: ViewEncapsulation.None,
  imports: [
    NgTemplateOutlet,
    NzAutocompleteOptionComponent,
    NzNoAnimationDirective,
  ],
  template: `
    <ng-template>
      <div
        #panel
        class="ant-select-dropdown ant-select-dropdown-placement-bottomLeft"
        [class.ant-select-dropdown-hidden]="!showPanel"
        [class.ant-select-dropdown-rtl]="dir === 'rtl'"
        [class]="nzOverlayClassName"
        [style]="nzOverlayStyle"
        [nzNoAnimation]="!animationEnabled()"
        [animate.enter]="autoCompleteAnimationEnter()"
        [animate.leave]="autoCompleteAnimationLeave()"
        (animate.leave)="onAnimationEvent($event)"
      >
        <div class="ant-select-dropdown-content-wrapper">
          <div class="ant-select-dropdown-content">
            <ng-template
              *ngTemplateOutlet="
                nzDataSource ? optionsTemplate : contentTemplate
              "
            />
          </div>
        </div>
      </div>
      <ng-template #contentTemplate>
        <ng-content />
      </ng-template>
      <ng-template #optionsTemplate>
        @for (option of normalizedDataSource; track option.value) {
          <nz-auto-option [nzValue]="option.value" [nzLabel]="option.label">
            {{ option.label }}
          </nz-auto-option>
        }
      </ng-template>
    </ng-template>
  `,
})
export class NzAutocompleteComponent
  implements AfterContentInit, AfterViewInit, OnInit, OnChanges
{
  private changeDetectorRef = inject(ChangeDetectorRef);
  private directionality = inject(Directionality);
  private destroyRef = inject(DestroyRef);

  @Input({ transform: numberAttributeWithZeroFallback }) nzWidth?: number;
  @Input() nzOverlayClassName = "";
  @Input() nzOverlayStyle: Record<string, string> = {};
  @Input({ transform: booleanAttribute }) nzDefaultActiveFirstOption = true;
  @Input({ transform: booleanAttribute }) nzBackfill = false;
  @Input() compareWith: CompareWith = (o1, o2) => o1 === o2;
  @Input() nzDataSource?: AutocompleteDataSource;
  @Output()
  readonly selectionChange: EventEmitter<NzAutocompleteOptionComponent> =
    new EventEmitter<NzAutocompleteOptionComponent>();

  showPanel: boolean = true;
  isOpen: boolean = false;
  activeItem: NzAutocompleteOptionComponent | null = null;
  dir: Direction = "ltr";
  normalizedDataSource: AutocompleteDataSourceItem[] = [];
  animationStateChange = new EventEmitter<AnimationCallbackEvent>();

  /**
   * Options accessor, its source may be content or dataSource
   */
  get options(): QueryList<NzAutocompleteOptionComponent> {
    // first dataSource
    if (this.nzDataSource) {
      return this.fromDataSourceOptions;
    } else {
      return this.fromContentOptions;
    }
  }

  /** Provided by content */
  @ContentChildren(NzAutocompleteOptionComponent, { descendants: true })
  fromContentOptions!: QueryList<NzAutocompleteOptionComponent>;
  /** Provided by dataSource */
  @ViewChildren(NzAutocompleteOptionComponent)
  fromDataSourceOptions!: QueryList<NzAutocompleteOptionComponent>;

  /** cdk-overlay */
  @ViewChild(TemplateRef, { static: false }) template?: TemplateRef<{}>;
  @ViewChild("panel", { static: false }) panel?: ElementRef;
  @ViewChild("content", { static: false }) content?: ElementRef;

  private activeItemIndex: number = -1;
  private selectionChangeSubscription: Subscription | null = Subscription.EMPTY;
  private optionMouseEnterSubscription: Subscription | null =
    Subscription.EMPTY;
  private dataSourceChangeSubscription: Subscription | null =
    Subscription.EMPTY;

  /** Options changes listener */
  private readonly optionSelectionChanges: Observable<NzOptionSelectionChange> =
    defer(() => {
      if (this.options) {
        return merge<NzOptionSelectionChange[]>(
          ...this.options.map((option) => option.selectionChange),
        );
      }

      return this.afterNextRender$.pipe(
        switchMap(() => this.optionSelectionChanges),
      );
    });

  private readonly optionMouseEnter: Observable<NzAutocompleteOptionComponent> =
    defer(() => {
      if (this.options) {
        return merge<NzAutocompleteOptionComponent[]>(
          ...this.options.map((option) => option.mouseEntered),
        );
      }

      return this.afterNextRender$.pipe(switchMap(() => this.optionMouseEnter));
    });

  private afterNextRender$ = inject(NZ_AFTER_NEXT_RENDER$);

  protected readonly autoCompleteAnimationEnter = slideAnimationEnter();
  protected readonly autoCompleteAnimationLeave = slideAnimationLeave();
  protected readonly animationEnabled = isAnimationEnabled(
    () => !this.noAnimation?.nzNoAnimation(),
  );

  noAnimation = inject(NzNoAnimationDirective, { host: true, optional: true });

  constructor() {
    this.destroyRef.onDestroy(() => {
      this.dataSourceChangeSubscription!.unsubscribe();
      this.selectionChangeSubscription!.unsubscribe();
      this.optionMouseEnterSubscription!.unsubscribe();
      // Caretaker note: we have to set these subscriptions to `null` since these will be closed subscriptions, but they
      // still keep references to destinations (which are `SafeSubscriber`s). Destinations keep referencing `next` functions,
      // which we pass, for instance, to `this.optionSelectionChanges.subscribe(...)`.
      this.dataSourceChangeSubscription =
        this.selectionChangeSubscription =
        this.optionMouseEnterSubscription =
          null;
    });
  }

  ngOnInit(): void {
    this.directionality.change
      ?.pipe(takeUntilDestroyed(this.destroyRef))
      .subscribe((direction: Direction) => {
        this.dir = direction;
        this.changeDetectorRef.detectChanges();
      });

    this.dir = this.directionality.value;
  }

  ngOnChanges(changes: SimpleChanges): void {
    const { nzDataSource } = changes;
    if (nzDataSource) {
      this.normalizedDataSource = normalizeDataSource(
        nzDataSource.currentValue,
      );
    }
  }

  onAnimationEvent(event: AnimationCallbackEvent): void {
    const element = event.target as HTMLElement;
    // If animations are disabled, complete immediately
    if (!this.animationEnabled()) {
      this.animationStateChange.emit(event);
      event.animationComplete();
      return;
    }
    const onAnimationEnd = (): void => {
      element.removeEventListener("animationend", onAnimationEnd);
      this.animationStateChange.emit(event);
      event.animationComplete();
    };
    element.addEventListener("animationend", onAnimationEnd);
  }

  ngAfterContentInit(): void {
    if (!this.nzDataSource) {
      this.optionsInit();
    }
  }

  ngAfterViewInit(): void {
    if (this.nzDataSource) {
      this.optionsInit();
    }
  }

  setVisibility(): void {
    this.showPanel = !!this.options.length;
    this.changeDetectorRef.markForCheck();
  }

  setActiveItem(index: number): void {
    const activeItem = this.options.get(index);
    if (activeItem && !activeItem.active) {
      this.activeItem = activeItem;
      this.activeItemIndex = index;
      this.clearSelectedOptions(this.activeItem);
      this.activeItem.setActiveStyles();
    } else {
      this.activeItem = null;
      this.activeItemIndex = -1;
      this.clearSelectedOptions();
    }
    this.changeDetectorRef.markForCheck();
  }

  setNextItemActive(): void {
    const nextIndex =
      this.activeItemIndex + 1 <= this.options.length - 1
        ? this.activeItemIndex + 1
        : 0;
    this.setActiveItem(nextIndex);
  }

  setPreviousItemActive(): void {
    const previousIndex =
      this.activeItemIndex - 1 < 0
        ? this.options.length - 1
        : this.activeItemIndex - 1;
    this.setActiveItem(previousIndex);
  }

  getOptionIndex(value: NzSafeAny): number {
    return this.options.reduce(
      (result: number, current: NzAutocompleteOptionComponent, index: number) =>
        result === -1
          ? this.compareWith(value, current.nzValue)
            ? index
            : -1
          : result,
      -1,
    )!;
  }

  getOption(value: NzSafeAny): NzAutocompleteOptionComponent | null {
    return (
      this.options.find((item) => this.compareWith(value, item.nzValue)) || null
    );
  }

  private optionsInit(): void {
    this.setVisibility();
    this.subscribeOptionChanges();
    const changes = this.nzDataSource
      ? this.fromDataSourceOptions.changes
      : this.fromContentOptions.changes;
    // async
    this.dataSourceChangeSubscription = changes.subscribe((e) => {
      if (!e.dirty && this.isOpen) {
        setTimeout(() => this.setVisibility());
      }
      this.subscribeOptionChanges();
    });
  }

  /**
   * Clear the status of options
   */
  clearSelectedOptions(
    skip?: NzAutocompleteOptionComponent | null,
    deselect: boolean = false,
  ): void {
    this.options.forEach((option) => {
      if (option !== skip) {
        if (deselect) {
          option.deselect();
        }
        option.setInactiveStyles();
      }
    });
  }

  private subscribeOptionChanges(): void {
    this.selectionChangeSubscription!.unsubscribe();
    this.selectionChangeSubscription = this.optionSelectionChanges
      .pipe(filter((event: NzOptionSelectionChange) => event.isUserInput))
      .subscribe((event: NzOptionSelectionChange) => {
        event.source.select();
        event.source.setActiveStyles();
        this.activeItem = event.source;
        this.activeItemIndex = this.getOptionIndex(this.activeItem.nzValue);
        this.clearSelectedOptions(event.source, true);
        this.selectionChange.emit(event.source);
      });

    this.optionMouseEnterSubscription!.unsubscribe();
    this.optionMouseEnterSubscription = this.optionMouseEnter.subscribe(
      (event: NzAutocompleteOptionComponent) => {
        event.setActiveStyles();
        this.activeItem = event;
        this.activeItemIndex = this.getOptionIndex(this.activeItem.nzValue);
        this.clearSelectedOptions(event);
      },
    );
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
import { NgTemplateOutlet } from "@angular/common";
import {
  AfterContentInit,
  AfterViewInit,
  ChangeDetectionStrategy,
  ChangeDetectorRef,
  Component,
  ContentChildren,
  DestroyRef,
  ElementRef,
  EventEmitter,
  Input,
  OnChanges,
  OnInit,
  Output,
  QueryList,
  SimpleChanges,
  TemplateRef,
  ViewChild,
  ViewChildren,
  ViewEncapsulation,
  booleanAttribute,
  inject,
  type AnimationCallbackEvent,
} from "@angular/core";
import { takeUntilDestroyed } from "@angular/core/rxjs-interop";
import { Observable, Subscription, defer, merge } from "rxjs";
import { filter, switchMap } from "rxjs/operators";

import {
  NzNoAnimationDirective,
  isAnimationEnabled,
  slideAnimationEnter,
  slideAnimationLeave,
} from "ng-zorro-antd/core/animation";
import { NZ_AFTER_NEXT_RENDER$ } from "ng-zorro-antd/core/render";
import { CompareWith, NzSafeAny } from "ng-zorro-antd/core/types";
import { numberAttributeWithZeroFallback } from "ng-zorro-antd/core/util";

import {
  NzAutocompleteOptionComponent,
  NzOptionSelectionChange,
} from "./autocomplete-option.component";

export interface AutocompleteDataSourceItem {
  value: string;
  label: string;
}

export type AutocompleteDataSource = Array<
  AutocompleteDataSourceItem | string | number
>;

function normalizeDataSource(
  value: AutocompleteDataSource,
): AutocompleteDataSourceItem[] {
  return value?.map((item) => {
    if (typeof item === "number" || typeof item === "string") {
      return {
        label: item.toString(),
        value: item.toString(),
      };
    }
    return item;
  });
}

@Component({
  selector: "nz-autocomplete",
  exportAs: "nzAutocomplete",
  changeDetection: ChangeDetectionStrategy.OnPush,
  encapsulation: ViewEncapsulation.None,
  imports: [
    NgTemplateOutlet,
    NzAutocompleteOptionComponent,
    NzNoAnimationDirective,
  ],
  template: `
    <ng-template>
      <div
        #panel
        class="ant-select-dropdown ant-select-dropdown-placement-bottomLeft"
        [class.ant-select-dropdown-hidden]="!showPanel"
        [class.ant-select-dropdown-rtl]="dir === 'rtl'"
        [class]="nzOverlayClassName"
        [style]="nzOverlayStyle"
        [nzNoAnimation]="!animationEnabled()"
        [animate.enter]="autoCompleteAnimationEnter()"
        [animate.leave]="autoCompleteAnimationLeave()"
        (animate.leave)="onAnimationEvent($event)"
      >
        <div class="ant-select-dropdown-content-wrapper">
          <div class="ant-select-dropdown-content">
            <ng-template
              *ngTemplateOutlet="
                nzDataSource ? optionsTemplate : contentTemplate
              "
            />
          </div>
        </div>
      </div>
      <ng-template #contentTemplate>
        <ng-content />
      </ng-template>
      <ng-template #optionsTemplate>
        @for (option of normalizedDataSource; track option.value) {
          <nz-auto-option [nzValue]="option.value" [nzLabel]="option.label">
            {{ option.label }}
          </nz-auto-option>
        }
      </ng-template>
    </ng-template>
  `,
})
export class NzAutocompleteComponent
  implements AfterContentInit, AfterViewInit, OnInit, OnChanges
{
  private changeDetectorRef = inject(ChangeDetectorRef);
  private directionality = inject(Directionality);
  private destroyRef = inject(DestroyRef);

  @Input({ transform: numberAttributeWithZeroFallback }) nzWidth?: number;
  @Input() nzOverlayClassName = "";
  @Input() nzOverlayStyle: Record<string, string> = {};
  @Input({ transform: booleanAttribute }) nzDefaultActiveFirstOption = true;
  @Input({ transform: booleanAttribute }) nzBackfill = false;
  @Input() compareWith: CompareWith = (o1, o2) => o1 === o2;
  @Input() nzDataSource?: AutocompleteDataSource;
  @Output()
  readonly selectionChange: EventEmitter<NzAutocompleteOptionComponent> =
    new EventEmitter<NzAutocompleteOptionComponent>();

  showPanel: boolean = true;
  isOpen: boolean = false;
  activeItem: NzAutocompleteOptionComponent | null = null;
  dir: Direction = "ltr";
  normalizedDataSource: AutocompleteDataSourceItem[] = [];
  animationStateChange = new EventEmitter<AnimationCallbackEvent>();

  /**
   * Options accessor, its source may be content or dataSource
   */
  get options(): QueryList<NzAutocompleteOptionComponent> {
    // first dataSource
    if (this.nzDataSource) {
      return this.fromDataSourceOptions;
    } else {
      return this.fromContentOptions;
    }
  }

  /** Provided by content */
  @ContentChildren(NzAutocompleteOptionComponent, { descendants: true })
  fromContentOptions!: QueryList<NzAutocompleteOptionComponent>;
  /** Provided by dataSource */
  @ViewChildren(NzAutocompleteOptionComponent)
  fromDataSourceOptions!: QueryList<NzAutocompleteOptionComponent>;

  /** cdk-overlay */
  @ViewChild(TemplateRef, { static: false }) template?: TemplateRef<{}>;
  @ViewChild("panel", { static: false }) panel?: ElementRef;
  @ViewChild("content", { static: false }) content?: ElementRef;

  private activeItemIndex: number = -1;
  private selectionChangeSubscription: Subscription | null = Subscription.EMPTY;
  private optionMouseEnterSubscription: Subscription | null =
    Subscription.EMPTY;
  private dataSourceChangeSubscription: Subscription | null =
    Subscription.EMPTY;

  /** Options changes listener */
  private readonly optionSelectionChanges: Observable<NzOptionSelectionChange> =
    defer(() => {
      if (this.options) {
        return merge<NzOptionSelectionChange[]>(
          ...this.options.map((option) => option.selectionChange),
        );
      }

      return this.afterNextRender$.pipe(
        switchMap(() => this.optionSelectionChanges),
      );
    });

  private readonly optionMouseEnter: Observable<NzAutocompleteOptionComponent> =
    defer(() => {
      if (this.options) {
        return merge<NzAutocompleteOptionComponent[]>(
          ...this.options.map((option) => option.mouseEntered),
        );
      }

      return this.afterNextRender$.pipe(switchMap(() => this.optionMouseEnter));
    });

  private afterNextRender$ = inject(NZ_AFTER_NEXT_RENDER$);

  protected readonly autoCompleteAnimationEnter = slideAnimationEnter();
  protected readonly autoCompleteAnimationLeave = slideAnimationLeave();
  protected readonly animationEnabled = isAnimationEnabled(
    () => !this.noAnimation?.nzNoAnimation(),
  );

  noAnimation = inject(NzNoAnimationDirective, { host: true, optional: true });

  constructor() {
    this.destroyRef.onDestroy(() => {
      this.dataSourceChangeSubscription!.unsubscribe();
      this.selectionChangeSubscription!.unsubscribe();
      this.optionMouseEnterSubscription!.unsubscribe();
      // Caretaker note: we have to set these subscriptions to `null` since these will be closed subscriptions, but they
      // still keep references to destinations (which are `SafeSubscriber`s). Destinations keep referencing `next` functions,
      // which we pass, for instance, to `this.optionSelectionChanges.subscribe(...)`.
      this.dataSourceChangeSubscription =
        this.selectionChangeSubscription =
        this.optionMouseEnterSubscription =
          null;
    });
  }

  ngOnInit(): void {
    this.directionality.change
      ?.pipe(takeUntilDestroyed(this.destroyRef))
      .subscribe((direction: Direction) => {
        this.dir = direction;
        this.changeDetectorRef.detectChanges();
      });

    this.dir = this.directionality.value;
  }

  ngOnChanges(changes: SimpleChanges): void {
    const { nzDataSource } = changes;
    if (nzDataSource) {
      this.normalizedDataSource = normalizeDataSource(
        nzDataSource.currentValue,
      );
    }
  }

  onAnimationEvent(event: AnimationCallbackEvent): void {
    const element = event.target as HTMLElement;
    // If animations are disabled, complete immediately
    if (!this.animationEnabled()) {
      this.animationStateChange.emit(event);
      event.animationComplete();
      return;
    }
    const onAnimationEnd = (): void => {
      element.removeEventListener("animationend", onAnimationEnd);
      this.animationStateChange.emit(event);
      event.animationComplete();
    };
    element.addEventListener("animationend", onAnimationEnd);
  }

  ngAfterContentInit(): void {
    if (!this.nzDataSource) {
      this.optionsInit();
    }
  }

  ngAfterViewInit(): void {
    if (this.nzDataSource) {
      this.optionsInit();
    }
  }

  setVisibility(): void {
    this.showPanel = !!this.options.length;
    this.changeDetectorRef.markForCheck();
  }

  setActiveItem(index: number): void {
    const activeItem = this.options.get(index);
    if (activeItem && !activeItem.active) {
      this.activeItem = activeItem;
      this.activeItemIndex = index;
      this.clearSelectedOptions(this.activeItem);
      this.activeItem.setActiveStyles();
    } else {
      this.activeItem = null;
      this.activeItemIndex = -1;
      this.clearSelectedOptions();
    }
    this.changeDetectorRef.markForCheck();
  }

  setNextItemActive(): void {
    const nextIndex =
      this.activeItemIndex + 1 <= this.options.length - 1
        ? this.activeItemIndex + 1
        : 0;
    this.setActiveItem(nextIndex);
  }

  setPreviousItemActive(): void {
    const previousIndex =
      this.activeItemIndex - 1 < 0
        ? this.options.length - 1
        : this.activeItemIndex - 1;
    this.setActiveItem(previousIndex);
  }

  getOptionIndex(value: NzSafeAny): number {
    return this.options.reduce(
      (
        result: number,
        current: NzAutocompleteOptionComponent,
        index: number,
      ) =>
        result === -1
          ? this.compareWith(value, current.nzValue)
            ? index
            : -1
          : result,
      -1,
    )!;
  }

  getOption(value: NzSafeAny): NzAutocompleteOptionComponent | null {
    return (
      this.options.find((item) => this.compareWith(value, item.nzValue)) || null
    );
  }

  private optionsInit(): void {
    this.setVisibility();
    this.subscribeOptionChanges();
    const changes = this.nzDataSource
      ? this.fromDataSourceOptions.changes
      : this.fromContentOptions.changes;
    // async
    this.dataSourceChangeSubscription = changes.subscribe((e) => {
      if (!e.dirty && this.isOpen) {
        setTimeout(() => this.setVisibility());
      }
      this.subscribeOptionChanges();
    });
  }

  /**
   * Clear the status of options
   */
  clearSelectedOptions(
    skip?: NzAutocompleteOptionComponent | null,
    deselect: boolean = false,
  ): void {
    this.options.forEach((option) => {
      if (option !== skip) {
        if (deselect) {
          option.deselect();
        }
        option.setInactiveStyles();
      }
    });
  }

  private subscribeOptionChanges(): void {
    this.selectionChangeSubscription!.unsubscribe();
    this.selectionChangeSubscription = this.optionSelectionChanges
      .pipe(filter((event: NzOptionSelectionChange) => event.isUserInput))
      .subscribe((event: NzOptionSelectionChange) => {
        event.source.select();
        event.source.setActiveStyles();
        this.activeItem = event.source;
        this.activeItemIndex = this.getOptionIndex(this.activeItem.nzValue);
        this.clearSelectedOptions(event.source, true);
        this.selectionChange.emit(event.source);
      });

    this.optionMouseEnterSubscription!.unsubscribe();
    this.optionMouseEnterSubscription = this.optionMouseEnter.subscribe(
      (event: NzAutocompleteOptionComponent) => {
        event.setActiveStyles();
        this.activeItem = event;
        this.activeItemIndex = this.getOptionIndex(this.activeItem.nzValue);
        this.clearSelectedOptions(event);
      },
    );
  }
}

`````
