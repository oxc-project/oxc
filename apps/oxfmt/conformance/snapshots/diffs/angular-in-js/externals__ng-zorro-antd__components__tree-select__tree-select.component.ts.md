# externals/ng-zorro-antd/components/tree-select/tree-select.component.ts

## Option 1

`````json
{"printWidth":80}
`````

### Diff

`````diff
===================================================================
--- prettier
+++ oxfmt
@@ -106,9 +106,13 @@
 
 import { NzTreeSelectService } from "./tree-select.service";
 
 export type NzPlacementType =
-  "bottomLeft" | "bottomRight" | "topLeft" | "topRight" | "";
+  | "bottomLeft"
+  | "bottomRight"
+  | "topLeft"
+  | "topRight"
+  | "";
 const NZ_CONFIG_MODULE_NAME: NzConfigKey = "treeSelect";
 const TREE_SELECT_DEFAULT_CLASS =
   "ant-select-dropdown ant-select-tree-dropdown";
 

`````

### Actual (oxfmt)

`````ts
/**
 * Use of this source code is governed by an MIT-style license that can be
 * found in the LICENSE file at https://github.com/NG-ZORRO/ng-zorro-antd/blob/master/LICENSE
 */

import { FocusMonitor } from "@angular/cdk/a11y";
import { Directionality } from "@angular/cdk/bidi";
import { BACKSPACE, ESCAPE, TAB } from "@angular/cdk/keycodes";
import {
  CdkConnectedOverlay,
  CdkOverlayOrigin,
  ConnectedOverlayPositionChange,
  ConnectionPositionPair,
} from "@angular/cdk/overlay";
import { Platform, _getEventTarget } from "@angular/cdk/platform";
import { SlicePipe } from "@angular/common";
import {
  ChangeDetectorRef,
  Component,
  ContentChild,
  DestroyRef,
  ElementRef,
  EventEmitter,
  Input,
  OnChanges,
  OnInit,
  Output,
  Renderer2,
  SimpleChanges,
  TemplateRef,
  ViewChild,
  booleanAttribute,
  computed,
  forwardRef,
  inject,
  numberAttribute,
  signal,
} from "@angular/core";
import { takeUntilDestroyed } from "@angular/core/rxjs-interop";
import { ControlValueAccessor, NG_VALUE_ACCESSOR } from "@angular/forms";
import { Subject, combineLatest, merge, of as observableOf } from "rxjs";
import {
  distinctUntilChanged,
  filter,
  map,
  startWith,
  tap,
  withLatestFrom,
} from "rxjs/operators";

import {
  NzNoAnimationDirective,
  slideAnimationEnter,
  slideAnimationLeave,
} from "ng-zorro-antd/core/animation";
import {
  NzConfigKey,
  WithConfig,
  onConfigChangeEventForComponent,
} from "ng-zorro-antd/core/config";
import {
  NZ_FORM_SIZE,
  NZ_FORM_VARIANT,
  NzFormItemFeedbackIconComponent,
  NzFormNoStatusService,
  NzFormStatusService,
} from "ng-zorro-antd/core/form";
import { NzStringTemplateOutletDirective } from "ng-zorro-antd/core/outlet";
import {
  NzOverlayModule,
  POSITION_MAP,
  POSITION_TYPE,
  getPlacementName,
} from "ng-zorro-antd/core/overlay";
import {
  cancelAnimationFrame,
  requestAnimationFrame,
} from "ng-zorro-antd/core/polyfill";
import {
  NzFormatEmitEvent,
  NzTreeBase,
  NzTreeHigherOrderServiceToken,
  NzTreeNode,
  NzTreeNodeOptions,
} from "ng-zorro-antd/core/tree";
import {
  NgClassInterface,
  NgStyleInterface,
  NzSafeAny,
  NzSizeLDSType,
  NzStatus,
  NzValidateStatus,
  NzVariant,
  OnChangeType,
  OnTouchedType,
} from "ng-zorro-antd/core/types";
import { getStatusClassNames, isNotNil } from "ng-zorro-antd/core/util";
import { NzEmptyModule } from "ng-zorro-antd/empty";
import { NzSelectModule, NzSelectSearchComponent } from "ng-zorro-antd/select";
import {
  NZ_SPACE_COMPACT_ITEM_TYPE,
  NZ_SPACE_COMPACT_SIZE,
  NzSpaceCompactItemDirective,
} from "ng-zorro-antd/space";
import { NzTreeComponent, NzTreeModule } from "ng-zorro-antd/tree";

import { NzTreeSelectService } from "./tree-select.service";

export type NzPlacementType =
  | "bottomLeft"
  | "bottomRight"
  | "topLeft"
  | "topRight"
  | "";
const NZ_CONFIG_MODULE_NAME: NzConfigKey = "treeSelect";
const TREE_SELECT_DEFAULT_CLASS =
  "ant-select-dropdown ant-select-tree-dropdown";

@Component({
  selector: "nz-tree-select",
  exportAs: "nzTreeSelect",
  imports: [
    NzOverlayModule,
    CdkConnectedOverlay,
    NzNoAnimationDirective,
    NzTreeModule,
    NzEmptyModule,
    CdkOverlayOrigin,
    SlicePipe,
    NzSelectModule,
    NzFormItemFeedbackIconComponent,
    NzStringTemplateOutletDirective,
  ],
  template: `
    <ng-template
      cdkConnectedOverlay
      nzConnectedOverlay
      [cdkConnectedOverlayOffsetY]="placement().startsWith('top') ? -4 : 4"
      [cdkConnectedOverlayHasBackdrop]="nzBackdrop"
      [cdkConnectedOverlayOrigin]="cdkOverlayOrigin"
      [cdkConnectedOverlayPositions]="nzPlacement ? positions : []"
      [cdkConnectedOverlayOpen]="nzOpen"
      cdkConnectedOverlayTransformOriginOn=".ant-select-tree-dropdown"
      [cdkConnectedOverlayMinWidth]="
        $any(nzDropdownMatchSelectWidth ? null : triggerWidth)
      "
      [cdkConnectedOverlayWidth]="
        $any(nzDropdownMatchSelectWidth ? triggerWidth : null)
      "
      (overlayOutsideClick)="onClickOutside($event)"
      (detach)="closeDropdown()"
      (positionChange)="onPositionChange($event)"
    >
      <div
        [class]="dropdownClassName"
        [nzNoAnimation]="!!noAnimation?.nzNoAnimation?.()"
        [animate.enter]="slideAnimationEnter()"
        [animate.leave]="slideAnimationLeave()"
        [class.ant-select-dropdown-placement-bottomLeft]="
          placement() === 'bottomLeft'
        "
        [class.ant-select-dropdown-placement-topLeft]="
          placement() === 'topLeft'
        "
        [class.ant-select-dropdown-placement-bottomRight]="
          placement() === 'bottomRight'
        "
        [class.ant-select-dropdown-placement-topRight]="
          placement() === 'topRight'
        "
        [class.ant-tree-select-dropdown-rtl]="dir() === 'rtl'"
        [dir]="dir()"
        [style]="nzDropdownStyle"
      >
        <nz-tree
          #treeRef
          [hidden]="isNotFound"
          nzNoAnimation
          nzSelectMode
          nzBlockNode
          [nzData]="nzNodes"
          [nzMultiple]="nzMultiple"
          [nzSearchValue]="inputValue"
          [nzHideUnMatched]="nzHideUnMatched"
          [nzShowIcon]="nzShowIcon"
          [nzCheckable]="nzCheckable"
          [nzAsyncData]="nzAsyncData"
          [nzShowExpand]="nzShowExpand"
          [nzShowLine]="nzShowLine"
          [nzExpandedIcon]="nzExpandedIcon"
          [nzExpandAll]="nzDefaultExpandAll"
          [nzExpandedKeys]="expandedKeys"
          [nzCheckedKeys]="nzCheckable ? value : []"
          [nzSelectedKeys]="!nzCheckable ? value : []"
          [nzTreeTemplate]="treeTemplate"
          [nzCheckStrictly]="nzCheckStrictly"
          [nzVirtualItemSize]="nzVirtualItemSize"
          [nzVirtualMaxBufferPx]="nzVirtualMaxBufferPx"
          [nzVirtualMinBufferPx]="nzVirtualMinBufferPx"
          [nzVirtualHeight]="nzVirtualHeight"
          (nzExpandChange)="onExpandedKeysChange($event)"
          (nzClick)="nzTreeClick.emit($event)"
          (nzCheckedKeysChange)="updateSelectedNodes()"
          (nzSelectedKeysChange)="updateSelectedNodes()"
          (nzCheckboxChange)="nzTreeCheckboxChange.emit($event)"
          (nzSearchValueChange)="setSearchValues($event)"
        />
        @if (nzNodes.length === 0 || isNotFound) {
          <span class="ant-select-not-found">
            <nz-embed-empty
              nzComponentName="tree-select"
              [specificContent]="nzNotFoundContent"
            />
          </span>
        }
      </div>
    </ng-template>

    <div cdkOverlayOrigin class="ant-select-selector">
      @if (nzPrefix; as prefix) {
        <div class="ant-select-prefix">
          <ng-container *nzStringTemplateOutlet="prefix">{{
            prefix
          }}</ng-container>
        </div>
      }

      <span class="ant-select-selection-wrap">
        @if (isMultiple) {
          <div class="ant-select-selection-overflow">
            @for (
              node of selectedNodes | slice: 0 : nzMaxTagCount;
              track node.key
            ) {
              <div class="ant-select-selection-overflow-item">
                <nz-select-item
                  deletable
                  [disabled]="node.isDisabled || nzDisabled"
                  [label]="nzDisplayWith(node)"
                  displayLabelInHtml
                  (delete)="removeSelected(node, true)"
                />
              </div>
            }
            @if (selectedNodes.length > nzMaxTagCount) {
              <div class="ant-select-selection-overflow-item">
                <nz-select-item
                  [contentTemplateOutlet]="nzMaxTagPlaceholder"
                  [contentTemplateOutletContext]="
                    selectedNodes | slice: nzMaxTagCount
                  "
                  [label]="
                    '+ ' + (selectedNodes.length - nzMaxTagCount) + ' ...'
                  "
                />
              </div>
            }
            <div
              class="ant-select-selection-overflow-item ant-select-selection-overflow-item-suffix"
            >
              <nz-select-search
                [nzId]="nzId"
                [showInput]="nzShowSearch"
                (keydown)="onKeyDownInput($event)"
                (isComposingChange)="isComposingChange($event)"
                (valueChange)="setInputValue($event)"
                [value]="inputValue"
                [mirrorSync]="true"
                [disabled]="nzDisabled"
                [focusTrigger]="nzOpen"
              />
            </div>
          </div>
        } @else {
          <nz-select-search
            [nzId]="nzId"
            [showInput]="nzShowSearch"
            (keydown)="onKeyDownInput($event)"
            (isComposingChange)="isComposingChange($event)"
            (valueChange)="setInputValue($event)"
            [value]="inputValue"
            [mirrorSync]="false"
            [disabled]="nzDisabled"
            [focusTrigger]="nzOpen"
          />
          @if (
            selectedNodes.length === 1 && !isComposing && inputValue === ""
          ) {
            <nz-select-item
              [label]="nzDisplayWith(selectedNodes[0])"
              displayLabelInHtml
            />
          }
        }

        @if (nzPlaceHolder && selectedNodes.length === 0) {
          <nz-select-placeholder
            [placeholder]="nzPlaceHolder"
            [style.display]="placeHolderDisplay"
          />
        }
      </span>

      <nz-select-arrow
        [showArrow]="true"
        [search]="nzOpen && nzShowSearch"
        [suffixIcon]="nzSuffixIcon"
        [feedbackIcon]="feedbackIconTpl"
      >
        <ng-template #feedbackIconTpl>
          @if (hasFeedback && !!status) {
            <nz-form-item-feedback-icon [status]="status" />
          }
        </ng-template>
      </nz-select-arrow>

      @if (nzAllowClear && !nzDisabled && selectedNodes.length) {
        <nz-select-clear (clear)="onClearSelection()" />
      }
    </div>
  `,
  providers: [
    NzTreeSelectService,
    { provide: NZ_SPACE_COMPACT_ITEM_TYPE, useValue: "select" },
    {
      provide: NzTreeHigherOrderServiceToken,
      useExisting: NzTreeSelectService,
    },
    {
      provide: NG_VALUE_ACCESSOR,
      useExisting: forwardRef(() => NzTreeSelectComponent),
      multi: true,
    },
  ],
  host: {
    class: "ant-select ant-tree-select",
    "[class.ant-select-in-form-item]": "!!nzFormStatusService",
    "[class.ant-select-rtl]": 'dir()==="rtl"',
    "[class.ant-select-lg]": 'finalSize() === "large"',
    "[class.ant-select-sm]": 'finalSize() === "small"',
    "[class.ant-select-disabled]": "nzDisabled",
    "[class.ant-select-single]": "!isMultiple",
    "[class.ant-select-show-arrow]": "!isMultiple",
    "[class.ant-select-show-search]": "!isMultiple",
    "[class.ant-select-outlined]": 'finalVariant() === "outlined"',
    "[class.ant-select-borderless]": 'finalVariant() === "borderless"',
    "[class.ant-select-filled]": 'finalVariant() === "filled"',
    "[class.ant-select-underlined]": 'finalVariant() === "underlined"',
    "[class.ant-select-multiple]": "isMultiple",
    "[class.ant-select-allow-clear]": "nzAllowClear",
    "[class.ant-select-open]": "nzOpen",
    "[class.ant-select-focused]": "nzOpen || focused",
    "(click)": "trigger()",
    "(keydown)": "onKeydown($event)",
  },
  hostDirectives: [NzSpaceCompactItemDirective],
})
export class NzTreeSelectComponent
  extends NzTreeBase
  implements ControlValueAccessor, OnInit, OnChanges
{
  readonly _nzModuleName: NzConfigKey = NZ_CONFIG_MODULE_NAME;

  private readonly renderer = inject(Renderer2);
  private readonly cdr = inject(ChangeDetectorRef);
  private readonly elementRef = inject(ElementRef);
  private readonly focusMonitor = inject(FocusMonitor);
  private readonly destroyRef = inject(DestroyRef);
  private readonly platform = inject(Platform);
  private requestId: number = -1;

  protected readonly slideAnimationEnter = slideAnimationEnter(() =>
    this.placement().startsWith("top") ? "down" : "up",
  );
  protected readonly slideAnimationLeave = slideAnimationLeave(() =>
    this.placement().startsWith("top") ? "down" : "up",
  );

  @Input() nzId: string | null = null;
  @Input({ transform: booleanAttribute }) nzAllowClear: boolean = true;
  @Input({ transform: booleanAttribute }) nzShowExpand: boolean = true;
  @Input({ transform: booleanAttribute }) nzShowLine: boolean = false;
  @Input({ transform: booleanAttribute })
  @WithConfig()
  nzDropdownMatchSelectWidth: boolean = true;
  @Input({ transform: booleanAttribute }) nzCheckable: boolean = false;
  @Input({ transform: booleanAttribute })
  @WithConfig()
  nzHideUnMatched: boolean = false;
  @Input({ transform: booleanAttribute }) @WithConfig() nzShowIcon: boolean =
    false;
  @Input({ transform: booleanAttribute }) nzShowSearch: boolean = false;
  @Input({ transform: booleanAttribute }) nzDisabled = false;
  @Input({ transform: booleanAttribute }) nzAsyncData = false;
  @Input({ transform: booleanAttribute }) nzMultiple = false;
  @Input({ transform: booleanAttribute }) nzDefaultExpandAll = false;
  @Input({ transform: booleanAttribute }) nzCheckStrictly = false;
  @Input() nzVirtualItemSize = 28;
  @Input() nzVirtualMaxBufferPx = 500;
  @Input() nzVirtualMinBufferPx = 28;
  @Input() nzVirtualHeight: string | null = null;
  @Input() nzExpandedIcon?: TemplateRef<{
    $implicit: NzTreeNode;
    origin: NzTreeNodeOptions;
  }>;
  @Input() nzNotFoundContent?: string | TemplateRef<void>;
  @Input() nzNodes: NzTreeNodeOptions[] | NzTreeNode[] = [];
  @Input() nzOpen = false;
  @Input() @WithConfig() nzSize: NzSizeLDSType = "default";
  @Input() @WithConfig() nzVariant: NzVariant | undefined = undefined;
  @Input() nzPlaceHolder = "";
  @Input() nzDropdownStyle: NgStyleInterface | null = null;
  @Input() nzDropdownClassName?: string;
  @Input() @WithConfig() nzBackdrop = false;
  @Input() nzStatus: NzStatus = "";
  @Input() nzPlacement: NzPlacementType = "";
  @Input()
  set nzExpandedKeys(value: string[]) {
    this.expandedKeys = value;
  }
  get nzExpandedKeys(): string[] {
    return this.expandedKeys;
  }

  @Input() nzPrefix: TemplateRef<NzSafeAny> | string | null = null;
  @Input() nzSuffixIcon: TemplateRef<NzSafeAny> | string | null = null;
  @Input() nzDisplayWith: (node: NzTreeNode) => string | undefined = (
    node: NzTreeNode,
  ) => node.title;
  @Input({ transform: numberAttribute }) nzMaxTagCount!: number;
  @Input() nzMaxTagPlaceholder: TemplateRef<{
    $implicit: NzTreeNode[];
  }> | null = null;
  @Output() readonly nzOpenChange = new EventEmitter<boolean>();
  @Output() readonly nzCleared = new EventEmitter<void>();
  @Output() readonly nzRemoved = new EventEmitter<NzTreeNode>();
  @Output() readonly nzExpandChange = new EventEmitter<NzFormatEmitEvent>();
  @Output() readonly nzTreeClick = new EventEmitter<NzFormatEmitEvent>();
  @Output() readonly nzTreeCheckboxChange =
    new EventEmitter<NzFormatEmitEvent>();

  @ViewChild(NzSelectSearchComponent, { static: false })
  nzSelectSearchComponent!: NzSelectSearchComponent;
  @ViewChild("treeRef", { static: false }) treeRef!: NzTreeComponent;
  @ViewChild(CdkOverlayOrigin, { static: true })
  cdkOverlayOrigin!: CdkOverlayOrigin;
  @ViewChild(CdkConnectedOverlay, { static: false })
  cdkConnectedOverlay!: CdkConnectedOverlay;

  @Input() nzTreeTemplate!: TemplateRef<{
    $implicit: NzTreeNode;
    origin: NzTreeNodeOptions;
  }>;
  @ContentChild("nzTreeTemplate", { static: true })
  nzTreeTemplateChild!: TemplateRef<{
    $implicit: NzTreeNode;
    origin: NzTreeNodeOptions;
  }>;
  get treeTemplate(): TemplateRef<{
    $implicit: NzTreeNode;
    origin: NzTreeNodeOptions;
  }> {
    return this.nzTreeTemplate || this.nzTreeTemplateChild;
  }

  prefixCls: string = "ant-select";
  statusCls: NgClassInterface = {};
  status: NzValidateStatus = "";
  hasFeedback: boolean = false;

  dropdownClassName = TREE_SELECT_DEFAULT_CLASS;
  triggerWidth?: number;
  isComposing = false;
  isNotFound = false;
  focused = false;
  inputValue = "";
  selectedNodes: NzTreeNode[] = [];
  expandedKeys: string[] = [];
  value: string[] = [];
  protected readonly dir = inject(Directionality).valueSignal;
  positions: ConnectionPositionPair[] = [];

  protected finalSize = computed(() => {
    if (this.formSize?.()) {
      return this.formSize();
    }
    if (this.compactSize) {
      return this.compactSize();
    }
    return this.size();
  });

  protected readonly finalVariant = computed(
    () => this.variant() || this.formVariant?.() || "outlined",
  );
  protected readonly placement = signal<NzPlacementType>("bottomLeft");

  private readonly size = signal<NzSizeLDSType>(this.nzSize);
  private readonly variant = signal<NzVariant | undefined>(this.nzVariant);

  private readonly formSize = inject(NZ_FORM_SIZE, { optional: true });
  private readonly formVariant = inject(NZ_FORM_VARIANT, { optional: true });

  private compactSize = inject(NZ_SPACE_COMPACT_SIZE, { optional: true });
  private isNzDisableFirstChange: boolean = true;
  private isComposingChange$ = new Subject<boolean>();
  private searchValueChange$ = new Subject<string>();

  onChange: OnChangeType = (_value) => {};
  onTouched: OnTouchedType = () => {};

  get placeHolderDisplay(): string {
    return this.inputValue || this.isComposing || this.selectedNodes.length
      ? "none"
      : "block";
  }

  get isMultiple(): boolean {
    return this.nzMultiple || this.nzCheckable;
  }

  noAnimation = inject(NzNoAnimationDirective, { host: true, optional: true });
  nzFormStatusService = inject(NzFormStatusService, { optional: true });
  private nzFormNoStatusService = inject(NzFormNoStatusService, {
    optional: true,
  });

  constructor() {
    super(inject(NzTreeSelectService));

    this.destroyRef.onDestroy(() => {
      this.closeDropdown();
    });

    onConfigChangeEventForComponent(NZ_CONFIG_MODULE_NAME, () => {
      this.size.set(this.nzSize);
      this.cdr.markForCheck();
    });
  }

  ngOnInit(): void {
    this.size.set(this.nzSize);

    this.nzFormStatusService?.formStatusChanges
      .pipe(
        distinctUntilChanged((pre, cur) => {
          return (
            pre.status === cur.status && pre.hasFeedback === cur.hasFeedback
          );
        }),
        withLatestFrom(
          this.nzFormNoStatusService
            ? this.nzFormNoStatusService.noFormStatus
            : observableOf(false),
        ),
        map(([{ status, hasFeedback }, noStatus]) => ({
          status: noStatus ? "" : status,
          hasFeedback,
        })),
        takeUntilDestroyed(this.destroyRef),
      )
      .subscribe(({ status, hasFeedback }) => {
        this.setStatusStyles(status, hasFeedback);
      });

    this.subscribeSelectionChange();

    this.focusMonitor
      .monitor(this.elementRef, true)
      .pipe(takeUntilDestroyed(this.destroyRef))
      .subscribe((focusOrigin) => {
        if (!focusOrigin) {
          this.focused = false;
          this.cdr.markForCheck();
          Promise.resolve().then(() => {
            this.onTouched();
          });
        } else {
          this.focused = true;
          this.cdr.markForCheck();
        }
      });

    // setInputValue method executed earlier than isComposingChange
    combineLatest([
      this.searchValueChange$,
      this.isComposingChange$.pipe(startWith(false)),
    ])
      .pipe(takeUntilDestroyed(this.destroyRef))
      .subscribe(([searchValue, isComposing]) => {
        this.isComposing = isComposing;
        if (!isComposing) {
          this.inputValue = searchValue;
          this.updatePosition();
        }
      });
  }

  isComposingChange(isComposing: boolean): void {
    this.isComposingChange$.next(isComposing);
  }

  setDisabledState(isDisabled: boolean): void {
    this.nzDisabled =
      (this.isNzDisableFirstChange && this.nzDisabled) || isDisabled;
    this.closeDropdown();
    this.isNzDisableFirstChange = false;
  }

  private setStatusStyles(
    status: NzValidateStatus,
    hasFeedback: boolean,
  ): void {
    // set inner status
    this.status = status;
    this.hasFeedback = hasFeedback;
    this.cdr.markForCheck();
    // render status if nzStatus is set
    this.statusCls = getStatusClassNames(this.prefixCls, status, hasFeedback);
    Object.keys(this.statusCls).forEach((status) => {
      if (this.statusCls[status]) {
        this.renderer.addClass(this.elementRef.nativeElement, status);
      } else {
        this.renderer.removeClass(this.elementRef.nativeElement, status);
      }
    });
  }

  ngOnChanges({
    nzNodes,
    nzDropdownClassName,
    nzStatus,
    nzPlacement,
    nzSize,
    nzVariant,
  }: SimpleChanges): void {
    if (nzNodes) {
      this.updateSelectedNodes(true);
    }
    if (nzDropdownClassName) {
      const className =
        this.nzDropdownClassName && this.nzDropdownClassName.trim();
      this.dropdownClassName = className
        ? `${TREE_SELECT_DEFAULT_CLASS} ${className}`
        : TREE_SELECT_DEFAULT_CLASS;
    }
    if (nzStatus) {
      this.setStatusStyles(this.nzStatus, this.hasFeedback);
    }

    if (nzPlacement) {
      const { currentValue } = nzPlacement;
      this.placement.set(currentValue);
      const listOfPlacement = [
        "bottomLeft",
        "topLeft",
        "bottomRight",
        "topRight",
      ];
      if (currentValue && listOfPlacement.includes(currentValue)) {
        this.positions = [POSITION_MAP[currentValue as POSITION_TYPE]];
      } else {
        this.positions = listOfPlacement.map(
          (e) => POSITION_MAP[e as POSITION_TYPE],
        );
      }
    }
    if (nzSize) {
      this.size.set(nzSize.currentValue);
    }
    if (nzVariant) {
      this.variant.set(nzVariant.currentValue);
    }
  }

  writeValue(value: string[] | string): void {
    if (isNotNil(value)) {
      if (this.isMultiple && Array.isArray(value)) {
        this.value = value;
      } else {
        this.value = [value as string];
      }
      // need clear selected nodes when user set value before updating
      this.clearSelectedNodes();
      this.updateSelectedNodes(true);
    } else {
      this.value = [];
      this.clearSelectedNodes();
      this.selectedNodes = [];
    }
    this.cdr.markForCheck();
  }

  registerOnChange(fn: (_: string[] | string | null) => void): void {
    this.onChange = fn;
  }

  registerOnTouched(fn: () => void): void {
    this.onTouched = fn;
  }

  onKeydown(event: KeyboardEvent): void {
    if (this.nzDisabled) {
      return;
    }
    switch (event.keyCode) {
      case ESCAPE:
        /**
         * Skip the ESCAPE processing, it will be handled in {@link onOverlayKeyDown}.
         */
        break;
      case TAB:
        this.closeDropdown();
        break;
      default:
        if (!this.nzOpen) {
          this.openDropdown();
        }
    }
  }

  trigger(): void {
    if (this.nzDisabled || (!this.nzDisabled && this.nzOpen)) {
      this.closeDropdown();
    } else {
      this.openDropdown();
    }
  }

  openDropdown(): void {
    if (!this.nzDisabled) {
      this.nzOpen = true;
      this.nzOpenChange.emit(this.nzOpen);
      this.updateCdkConnectedOverlayStatus();
      if (this.nzShowSearch || this.isMultiple) {
        this.focusOnInput();
      }
    }
  }

  closeDropdown(): void {
    Promise.resolve().then(() => this.onTouched());
    this.nzOpen = false;
    this.inputValue = "";
    this.isNotFound = false;
    this.nzOpenChange.emit(this.nzOpen);
    this.cdr.markForCheck();
  }

  onKeyDownInput(e: KeyboardEvent): void {
    const keyCode = e.keyCode;
    const eventTarget = e.target as HTMLInputElement;
    if (this.isMultiple && !eventTarget.value && keyCode === BACKSPACE) {
      e.preventDefault();
      if (this.selectedNodes.length) {
        const removeNode = this.selectedNodes[this.selectedNodes.length - 1];
        if (removeNode && !removeNode.isDisabled) {
          this.removeSelected(removeNode);
        }
      }
    }
  }

  onExpandedKeysChange(value: NzFormatEmitEvent): void {
    this.nzExpandChange.emit(value);
    this.expandedKeys = [...value.keys!];
  }

  setInputValue(value: string): void {
    this.searchValueChange$.next(value);
  }

  removeSelected(node: NzTreeNode, emit: boolean = true): void {
    node.isSelected = false;
    node.isChecked = false;
    if (this.nzCheckable) {
      this.nzTreeService.conduct(node, this.nzCheckStrictly);
    } else {
      this.nzTreeService.setSelectedNodeList(node, this.nzMultiple);
    }

    if (emit) {
      this.nzRemoved.emit(node);
    }
  }

  focusOnInput(): void {
    if (this.nzSelectSearchComponent) {
      this.nzSelectSearchComponent.focus();
    }
  }

  subscribeSelectionChange(): void {
    merge(
      this.nzTreeClick.pipe(
        tap((event: NzFormatEmitEvent) => {
          const node = event.node!;
          if (this.nzCheckable && !node.isDisabled && !node.isDisableCheckbox) {
            node.isChecked = !node.isChecked;
            node.isHalfChecked = false;
            if (!this.nzCheckStrictly) {
              this.nzTreeService.conduct(node);
            }
          }
          if (this.nzCheckable) {
            node.isSelected = false;
          }
        }),
        filter((event: NzFormatEmitEvent) => {
          const node = event.node!;
          return this.nzCheckable
            ? !node.isDisabled && !node.isDisableCheckbox
            : !node.isDisabled && node.isSelectable;
        }),
      ),
      this.nzCheckable
        ? this.nzTreeCheckboxChange.asObservable()
        : observableOf(),
      this.nzCleared,
      this.nzRemoved,
    )
      .pipe(takeUntilDestroyed(this.destroyRef))
      .subscribe(() => {
        this.updateSelectedNodes();
        const value = this.selectedNodes.map((node) => node.key!);
        this.value = [...value];
        if (this.nzShowSearch || this.isMultiple) {
          this.inputValue = "";
          this.isNotFound = false;
        }
        if (this.isMultiple) {
          this.onChange(value);
          this.focusOnInput();
          this.updatePosition();
        } else {
          this.closeDropdown();
          this.onChange(value.length ? value[0] : null);
        }
      });
  }

  updateSelectedNodes(init: boolean = false): void {
    if (init) {
      const nodes = this.coerceTreeNodes(this.nzNodes);
      this.nzTreeService.isMultiple = this.isMultiple;
      this.nzTreeService.isCheckStrictly = this.nzCheckStrictly;
      this.nzTreeService.initTree(nodes);
      if (this.nzCheckable) {
        this.nzTreeService.conductCheck(this.value, this.nzCheckStrictly);
      } else {
        this.nzTreeService.conductSelectedKeys(this.value, this.isMultiple);
      }
    }

    this.selectedNodes = [
      ...(this.nzCheckable
        ? this.getCheckedNodeList()
        : this.getSelectedNodeList()),
    ].sort((a, b) => {
      const indexA = this.value.indexOf(a.key);
      const indexB = this.value.indexOf(b.key);
      if (indexA !== -1 && indexB !== -1) {
        return indexA - indexB;
      }
      if (indexA !== -1) {
        return -1;
      }
      if (indexB !== -1) {
        return 1;
      }
      return 0;
    });
  }

  updatePosition(): void {
    requestAnimationFrame(() => {
      this.cdkConnectedOverlay?.overlayRef?.updatePosition();
    });
  }

  onPositionChange(position: ConnectedOverlayPositionChange): void {
    const placement = getPlacementName(position);
    this.placement.set(placement as NzPlacementType);
  }

  onClearSelection(): void {
    this.selectedNodes.forEach((node) => {
      this.removeSelected(node, false);
    });
    this.nzCleared.emit();
  }

  onClickOutside(event: MouseEvent): void {
    const target = _getEventTarget(event);
    if (!this.elementRef.nativeElement.contains(target)) {
      this.closeDropdown();
    }
  }

  setSearchValues($event: NzFormatEmitEvent): void {
    Promise.resolve().then(() => {
      this.isNotFound =
        (this.nzShowSearch || this.isMultiple) &&
        !!this.inputValue &&
        $event.matchedKeys!.length === 0;
    });
  }

  updateCdkConnectedOverlayStatus(): void {
    if (
      this.platform.isBrowser &&
      this.cdkOverlayOrigin.elementRef.nativeElement
    ) {
      const triggerWidth = this.triggerWidth;
      cancelAnimationFrame(this.requestId);
      this.requestId = requestAnimationFrame(() => {
        // Blink triggers style and layout pipelines anytime the `getBoundingClientRect()` is called, which may cause a
        // frame drop. That's why it's scheduled through the `requestAnimationFrame` to unload the composite thread.
        this.triggerWidth =
          this.cdkOverlayOrigin.elementRef.nativeElement.getBoundingClientRect().width;
        if (triggerWidth !== this.triggerWidth) {
          // The `requestAnimationFrame` will trigger change detection, but we're inside an `OnPush` component which won't have
          // the `ChecksEnabled` state. Calling `markForCheck()` will allow Angular to run the change detection from the root component
          // down to the `nz-select`. But we'll trigger only local change detection if the `triggerWidth` has been changed.
          this.cdr.detectChanges();
        }
      });
    }
  }

  clearSelectedNodes(): void {
    this.selectedNodes.forEach((node) => {
      this.removeSelected(node, false);
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

import { FocusMonitor } from "@angular/cdk/a11y";
import { Directionality } from "@angular/cdk/bidi";
import { BACKSPACE, ESCAPE, TAB } from "@angular/cdk/keycodes";
import {
  CdkConnectedOverlay,
  CdkOverlayOrigin,
  ConnectedOverlayPositionChange,
  ConnectionPositionPair,
} from "@angular/cdk/overlay";
import { Platform, _getEventTarget } from "@angular/cdk/platform";
import { SlicePipe } from "@angular/common";
import {
  ChangeDetectorRef,
  Component,
  ContentChild,
  DestroyRef,
  ElementRef,
  EventEmitter,
  Input,
  OnChanges,
  OnInit,
  Output,
  Renderer2,
  SimpleChanges,
  TemplateRef,
  ViewChild,
  booleanAttribute,
  computed,
  forwardRef,
  inject,
  numberAttribute,
  signal,
} from "@angular/core";
import { takeUntilDestroyed } from "@angular/core/rxjs-interop";
import { ControlValueAccessor, NG_VALUE_ACCESSOR } from "@angular/forms";
import { Subject, combineLatest, merge, of as observableOf } from "rxjs";
import {
  distinctUntilChanged,
  filter,
  map,
  startWith,
  tap,
  withLatestFrom,
} from "rxjs/operators";

import {
  NzNoAnimationDirective,
  slideAnimationEnter,
  slideAnimationLeave,
} from "ng-zorro-antd/core/animation";
import {
  NzConfigKey,
  WithConfig,
  onConfigChangeEventForComponent,
} from "ng-zorro-antd/core/config";
import {
  NZ_FORM_SIZE,
  NZ_FORM_VARIANT,
  NzFormItemFeedbackIconComponent,
  NzFormNoStatusService,
  NzFormStatusService,
} from "ng-zorro-antd/core/form";
import { NzStringTemplateOutletDirective } from "ng-zorro-antd/core/outlet";
import {
  NzOverlayModule,
  POSITION_MAP,
  POSITION_TYPE,
  getPlacementName,
} from "ng-zorro-antd/core/overlay";
import {
  cancelAnimationFrame,
  requestAnimationFrame,
} from "ng-zorro-antd/core/polyfill";
import {
  NzFormatEmitEvent,
  NzTreeBase,
  NzTreeHigherOrderServiceToken,
  NzTreeNode,
  NzTreeNodeOptions,
} from "ng-zorro-antd/core/tree";
import {
  NgClassInterface,
  NgStyleInterface,
  NzSafeAny,
  NzSizeLDSType,
  NzStatus,
  NzValidateStatus,
  NzVariant,
  OnChangeType,
  OnTouchedType,
} from "ng-zorro-antd/core/types";
import { getStatusClassNames, isNotNil } from "ng-zorro-antd/core/util";
import { NzEmptyModule } from "ng-zorro-antd/empty";
import { NzSelectModule, NzSelectSearchComponent } from "ng-zorro-antd/select";
import {
  NZ_SPACE_COMPACT_ITEM_TYPE,
  NZ_SPACE_COMPACT_SIZE,
  NzSpaceCompactItemDirective,
} from "ng-zorro-antd/space";
import { NzTreeComponent, NzTreeModule } from "ng-zorro-antd/tree";

import { NzTreeSelectService } from "./tree-select.service";

export type NzPlacementType =
  "bottomLeft" | "bottomRight" | "topLeft" | "topRight" | "";
const NZ_CONFIG_MODULE_NAME: NzConfigKey = "treeSelect";
const TREE_SELECT_DEFAULT_CLASS =
  "ant-select-dropdown ant-select-tree-dropdown";

@Component({
  selector: "nz-tree-select",
  exportAs: "nzTreeSelect",
  imports: [
    NzOverlayModule,
    CdkConnectedOverlay,
    NzNoAnimationDirective,
    NzTreeModule,
    NzEmptyModule,
    CdkOverlayOrigin,
    SlicePipe,
    NzSelectModule,
    NzFormItemFeedbackIconComponent,
    NzStringTemplateOutletDirective,
  ],
  template: `
    <ng-template
      cdkConnectedOverlay
      nzConnectedOverlay
      [cdkConnectedOverlayOffsetY]="placement().startsWith('top') ? -4 : 4"
      [cdkConnectedOverlayHasBackdrop]="nzBackdrop"
      [cdkConnectedOverlayOrigin]="cdkOverlayOrigin"
      [cdkConnectedOverlayPositions]="nzPlacement ? positions : []"
      [cdkConnectedOverlayOpen]="nzOpen"
      cdkConnectedOverlayTransformOriginOn=".ant-select-tree-dropdown"
      [cdkConnectedOverlayMinWidth]="
        $any(nzDropdownMatchSelectWidth ? null : triggerWidth)
      "
      [cdkConnectedOverlayWidth]="
        $any(nzDropdownMatchSelectWidth ? triggerWidth : null)
      "
      (overlayOutsideClick)="onClickOutside($event)"
      (detach)="closeDropdown()"
      (positionChange)="onPositionChange($event)"
    >
      <div
        [class]="dropdownClassName"
        [nzNoAnimation]="!!noAnimation?.nzNoAnimation?.()"
        [animate.enter]="slideAnimationEnter()"
        [animate.leave]="slideAnimationLeave()"
        [class.ant-select-dropdown-placement-bottomLeft]="
          placement() === 'bottomLeft'
        "
        [class.ant-select-dropdown-placement-topLeft]="
          placement() === 'topLeft'
        "
        [class.ant-select-dropdown-placement-bottomRight]="
          placement() === 'bottomRight'
        "
        [class.ant-select-dropdown-placement-topRight]="
          placement() === 'topRight'
        "
        [class.ant-tree-select-dropdown-rtl]="dir() === 'rtl'"
        [dir]="dir()"
        [style]="nzDropdownStyle"
      >
        <nz-tree
          #treeRef
          [hidden]="isNotFound"
          nzNoAnimation
          nzSelectMode
          nzBlockNode
          [nzData]="nzNodes"
          [nzMultiple]="nzMultiple"
          [nzSearchValue]="inputValue"
          [nzHideUnMatched]="nzHideUnMatched"
          [nzShowIcon]="nzShowIcon"
          [nzCheckable]="nzCheckable"
          [nzAsyncData]="nzAsyncData"
          [nzShowExpand]="nzShowExpand"
          [nzShowLine]="nzShowLine"
          [nzExpandedIcon]="nzExpandedIcon"
          [nzExpandAll]="nzDefaultExpandAll"
          [nzExpandedKeys]="expandedKeys"
          [nzCheckedKeys]="nzCheckable ? value : []"
          [nzSelectedKeys]="!nzCheckable ? value : []"
          [nzTreeTemplate]="treeTemplate"
          [nzCheckStrictly]="nzCheckStrictly"
          [nzVirtualItemSize]="nzVirtualItemSize"
          [nzVirtualMaxBufferPx]="nzVirtualMaxBufferPx"
          [nzVirtualMinBufferPx]="nzVirtualMinBufferPx"
          [nzVirtualHeight]="nzVirtualHeight"
          (nzExpandChange)="onExpandedKeysChange($event)"
          (nzClick)="nzTreeClick.emit($event)"
          (nzCheckedKeysChange)="updateSelectedNodes()"
          (nzSelectedKeysChange)="updateSelectedNodes()"
          (nzCheckboxChange)="nzTreeCheckboxChange.emit($event)"
          (nzSearchValueChange)="setSearchValues($event)"
        />
        @if (nzNodes.length === 0 || isNotFound) {
          <span class="ant-select-not-found">
            <nz-embed-empty
              nzComponentName="tree-select"
              [specificContent]="nzNotFoundContent"
            />
          </span>
        }
      </div>
    </ng-template>

    <div cdkOverlayOrigin class="ant-select-selector">
      @if (nzPrefix; as prefix) {
        <div class="ant-select-prefix">
          <ng-container *nzStringTemplateOutlet="prefix">{{
            prefix
          }}</ng-container>
        </div>
      }

      <span class="ant-select-selection-wrap">
        @if (isMultiple) {
          <div class="ant-select-selection-overflow">
            @for (
              node of selectedNodes | slice: 0 : nzMaxTagCount;
              track node.key
            ) {
              <div class="ant-select-selection-overflow-item">
                <nz-select-item
                  deletable
                  [disabled]="node.isDisabled || nzDisabled"
                  [label]="nzDisplayWith(node)"
                  displayLabelInHtml
                  (delete)="removeSelected(node, true)"
                />
              </div>
            }
            @if (selectedNodes.length > nzMaxTagCount) {
              <div class="ant-select-selection-overflow-item">
                <nz-select-item
                  [contentTemplateOutlet]="nzMaxTagPlaceholder"
                  [contentTemplateOutletContext]="
                    selectedNodes | slice: nzMaxTagCount
                  "
                  [label]="
                    '+ ' + (selectedNodes.length - nzMaxTagCount) + ' ...'
                  "
                />
              </div>
            }
            <div
              class="ant-select-selection-overflow-item ant-select-selection-overflow-item-suffix"
            >
              <nz-select-search
                [nzId]="nzId"
                [showInput]="nzShowSearch"
                (keydown)="onKeyDownInput($event)"
                (isComposingChange)="isComposingChange($event)"
                (valueChange)="setInputValue($event)"
                [value]="inputValue"
                [mirrorSync]="true"
                [disabled]="nzDisabled"
                [focusTrigger]="nzOpen"
              />
            </div>
          </div>
        } @else {
          <nz-select-search
            [nzId]="nzId"
            [showInput]="nzShowSearch"
            (keydown)="onKeyDownInput($event)"
            (isComposingChange)="isComposingChange($event)"
            (valueChange)="setInputValue($event)"
            [value]="inputValue"
            [mirrorSync]="false"
            [disabled]="nzDisabled"
            [focusTrigger]="nzOpen"
          />
          @if (
            selectedNodes.length === 1 && !isComposing && inputValue === ""
          ) {
            <nz-select-item
              [label]="nzDisplayWith(selectedNodes[0])"
              displayLabelInHtml
            />
          }
        }

        @if (nzPlaceHolder && selectedNodes.length === 0) {
          <nz-select-placeholder
            [placeholder]="nzPlaceHolder"
            [style.display]="placeHolderDisplay"
          />
        }
      </span>

      <nz-select-arrow
        [showArrow]="true"
        [search]="nzOpen && nzShowSearch"
        [suffixIcon]="nzSuffixIcon"
        [feedbackIcon]="feedbackIconTpl"
      >
        <ng-template #feedbackIconTpl>
          @if (hasFeedback && !!status) {
            <nz-form-item-feedback-icon [status]="status" />
          }
        </ng-template>
      </nz-select-arrow>

      @if (nzAllowClear && !nzDisabled && selectedNodes.length) {
        <nz-select-clear (clear)="onClearSelection()" />
      }
    </div>
  `,
  providers: [
    NzTreeSelectService,
    { provide: NZ_SPACE_COMPACT_ITEM_TYPE, useValue: "select" },
    {
      provide: NzTreeHigherOrderServiceToken,
      useExisting: NzTreeSelectService,
    },
    {
      provide: NG_VALUE_ACCESSOR,
      useExisting: forwardRef(() => NzTreeSelectComponent),
      multi: true,
    },
  ],
  host: {
    class: "ant-select ant-tree-select",
    "[class.ant-select-in-form-item]": "!!nzFormStatusService",
    "[class.ant-select-rtl]": 'dir()==="rtl"',
    "[class.ant-select-lg]": 'finalSize() === "large"',
    "[class.ant-select-sm]": 'finalSize() === "small"',
    "[class.ant-select-disabled]": "nzDisabled",
    "[class.ant-select-single]": "!isMultiple",
    "[class.ant-select-show-arrow]": "!isMultiple",
    "[class.ant-select-show-search]": "!isMultiple",
    "[class.ant-select-outlined]": 'finalVariant() === "outlined"',
    "[class.ant-select-borderless]": 'finalVariant() === "borderless"',
    "[class.ant-select-filled]": 'finalVariant() === "filled"',
    "[class.ant-select-underlined]": 'finalVariant() === "underlined"',
    "[class.ant-select-multiple]": "isMultiple",
    "[class.ant-select-allow-clear]": "nzAllowClear",
    "[class.ant-select-open]": "nzOpen",
    "[class.ant-select-focused]": "nzOpen || focused",
    "(click)": "trigger()",
    "(keydown)": "onKeydown($event)",
  },
  hostDirectives: [NzSpaceCompactItemDirective],
})
export class NzTreeSelectComponent
  extends NzTreeBase
  implements ControlValueAccessor, OnInit, OnChanges
{
  readonly _nzModuleName: NzConfigKey = NZ_CONFIG_MODULE_NAME;

  private readonly renderer = inject(Renderer2);
  private readonly cdr = inject(ChangeDetectorRef);
  private readonly elementRef = inject(ElementRef);
  private readonly focusMonitor = inject(FocusMonitor);
  private readonly destroyRef = inject(DestroyRef);
  private readonly platform = inject(Platform);
  private requestId: number = -1;

  protected readonly slideAnimationEnter = slideAnimationEnter(() =>
    this.placement().startsWith("top") ? "down" : "up",
  );
  protected readonly slideAnimationLeave = slideAnimationLeave(() =>
    this.placement().startsWith("top") ? "down" : "up",
  );

  @Input() nzId: string | null = null;
  @Input({ transform: booleanAttribute }) nzAllowClear: boolean = true;
  @Input({ transform: booleanAttribute }) nzShowExpand: boolean = true;
  @Input({ transform: booleanAttribute }) nzShowLine: boolean = false;
  @Input({ transform: booleanAttribute })
  @WithConfig()
  nzDropdownMatchSelectWidth: boolean = true;
  @Input({ transform: booleanAttribute }) nzCheckable: boolean = false;
  @Input({ transform: booleanAttribute })
  @WithConfig()
  nzHideUnMatched: boolean = false;
  @Input({ transform: booleanAttribute }) @WithConfig() nzShowIcon: boolean =
    false;
  @Input({ transform: booleanAttribute }) nzShowSearch: boolean = false;
  @Input({ transform: booleanAttribute }) nzDisabled = false;
  @Input({ transform: booleanAttribute }) nzAsyncData = false;
  @Input({ transform: booleanAttribute }) nzMultiple = false;
  @Input({ transform: booleanAttribute }) nzDefaultExpandAll = false;
  @Input({ transform: booleanAttribute }) nzCheckStrictly = false;
  @Input() nzVirtualItemSize = 28;
  @Input() nzVirtualMaxBufferPx = 500;
  @Input() nzVirtualMinBufferPx = 28;
  @Input() nzVirtualHeight: string | null = null;
  @Input() nzExpandedIcon?: TemplateRef<{
    $implicit: NzTreeNode;
    origin: NzTreeNodeOptions;
  }>;
  @Input() nzNotFoundContent?: string | TemplateRef<void>;
  @Input() nzNodes: NzTreeNodeOptions[] | NzTreeNode[] = [];
  @Input() nzOpen = false;
  @Input() @WithConfig() nzSize: NzSizeLDSType = "default";
  @Input() @WithConfig() nzVariant: NzVariant | undefined = undefined;
  @Input() nzPlaceHolder = "";
  @Input() nzDropdownStyle: NgStyleInterface | null = null;
  @Input() nzDropdownClassName?: string;
  @Input() @WithConfig() nzBackdrop = false;
  @Input() nzStatus: NzStatus = "";
  @Input() nzPlacement: NzPlacementType = "";
  @Input()
  set nzExpandedKeys(value: string[]) {
    this.expandedKeys = value;
  }
  get nzExpandedKeys(): string[] {
    return this.expandedKeys;
  }

  @Input() nzPrefix: TemplateRef<NzSafeAny> | string | null = null;
  @Input() nzSuffixIcon: TemplateRef<NzSafeAny> | string | null = null;
  @Input() nzDisplayWith: (node: NzTreeNode) => string | undefined = (
    node: NzTreeNode,
  ) => node.title;
  @Input({ transform: numberAttribute }) nzMaxTagCount!: number;
  @Input() nzMaxTagPlaceholder: TemplateRef<{
    $implicit: NzTreeNode[];
  }> | null = null;
  @Output() readonly nzOpenChange = new EventEmitter<boolean>();
  @Output() readonly nzCleared = new EventEmitter<void>();
  @Output() readonly nzRemoved = new EventEmitter<NzTreeNode>();
  @Output() readonly nzExpandChange = new EventEmitter<NzFormatEmitEvent>();
  @Output() readonly nzTreeClick = new EventEmitter<NzFormatEmitEvent>();
  @Output() readonly nzTreeCheckboxChange =
    new EventEmitter<NzFormatEmitEvent>();

  @ViewChild(NzSelectSearchComponent, { static: false })
  nzSelectSearchComponent!: NzSelectSearchComponent;
  @ViewChild("treeRef", { static: false }) treeRef!: NzTreeComponent;
  @ViewChild(CdkOverlayOrigin, { static: true })
  cdkOverlayOrigin!: CdkOverlayOrigin;
  @ViewChild(CdkConnectedOverlay, { static: false })
  cdkConnectedOverlay!: CdkConnectedOverlay;

  @Input() nzTreeTemplate!: TemplateRef<{
    $implicit: NzTreeNode;
    origin: NzTreeNodeOptions;
  }>;
  @ContentChild("nzTreeTemplate", { static: true })
  nzTreeTemplateChild!: TemplateRef<{
    $implicit: NzTreeNode;
    origin: NzTreeNodeOptions;
  }>;
  get treeTemplate(): TemplateRef<{
    $implicit: NzTreeNode;
    origin: NzTreeNodeOptions;
  }> {
    return this.nzTreeTemplate || this.nzTreeTemplateChild;
  }

  prefixCls: string = "ant-select";
  statusCls: NgClassInterface = {};
  status: NzValidateStatus = "";
  hasFeedback: boolean = false;

  dropdownClassName = TREE_SELECT_DEFAULT_CLASS;
  triggerWidth?: number;
  isComposing = false;
  isNotFound = false;
  focused = false;
  inputValue = "";
  selectedNodes: NzTreeNode[] = [];
  expandedKeys: string[] = [];
  value: string[] = [];
  protected readonly dir = inject(Directionality).valueSignal;
  positions: ConnectionPositionPair[] = [];

  protected finalSize = computed(() => {
    if (this.formSize?.()) {
      return this.formSize();
    }
    if (this.compactSize) {
      return this.compactSize();
    }
    return this.size();
  });

  protected readonly finalVariant = computed(
    () => this.variant() || this.formVariant?.() || "outlined",
  );
  protected readonly placement = signal<NzPlacementType>("bottomLeft");

  private readonly size = signal<NzSizeLDSType>(this.nzSize);
  private readonly variant = signal<NzVariant | undefined>(this.nzVariant);

  private readonly formSize = inject(NZ_FORM_SIZE, { optional: true });
  private readonly formVariant = inject(NZ_FORM_VARIANT, { optional: true });

  private compactSize = inject(NZ_SPACE_COMPACT_SIZE, { optional: true });
  private isNzDisableFirstChange: boolean = true;
  private isComposingChange$ = new Subject<boolean>();
  private searchValueChange$ = new Subject<string>();

  onChange: OnChangeType = (_value) => {};
  onTouched: OnTouchedType = () => {};

  get placeHolderDisplay(): string {
    return this.inputValue || this.isComposing || this.selectedNodes.length
      ? "none"
      : "block";
  }

  get isMultiple(): boolean {
    return this.nzMultiple || this.nzCheckable;
  }

  noAnimation = inject(NzNoAnimationDirective, { host: true, optional: true });
  nzFormStatusService = inject(NzFormStatusService, { optional: true });
  private nzFormNoStatusService = inject(NzFormNoStatusService, {
    optional: true,
  });

  constructor() {
    super(inject(NzTreeSelectService));

    this.destroyRef.onDestroy(() => {
      this.closeDropdown();
    });

    onConfigChangeEventForComponent(NZ_CONFIG_MODULE_NAME, () => {
      this.size.set(this.nzSize);
      this.cdr.markForCheck();
    });
  }

  ngOnInit(): void {
    this.size.set(this.nzSize);

    this.nzFormStatusService?.formStatusChanges
      .pipe(
        distinctUntilChanged((pre, cur) => {
          return (
            pre.status === cur.status && pre.hasFeedback === cur.hasFeedback
          );
        }),
        withLatestFrom(
          this.nzFormNoStatusService
            ? this.nzFormNoStatusService.noFormStatus
            : observableOf(false),
        ),
        map(([{ status, hasFeedback }, noStatus]) => ({
          status: noStatus ? "" : status,
          hasFeedback,
        })),
        takeUntilDestroyed(this.destroyRef),
      )
      .subscribe(({ status, hasFeedback }) => {
        this.setStatusStyles(status, hasFeedback);
      });

    this.subscribeSelectionChange();

    this.focusMonitor
      .monitor(this.elementRef, true)
      .pipe(takeUntilDestroyed(this.destroyRef))
      .subscribe((focusOrigin) => {
        if (!focusOrigin) {
          this.focused = false;
          this.cdr.markForCheck();
          Promise.resolve().then(() => {
            this.onTouched();
          });
        } else {
          this.focused = true;
          this.cdr.markForCheck();
        }
      });

    // setInputValue method executed earlier than isComposingChange
    combineLatest([
      this.searchValueChange$,
      this.isComposingChange$.pipe(startWith(false)),
    ])
      .pipe(takeUntilDestroyed(this.destroyRef))
      .subscribe(([searchValue, isComposing]) => {
        this.isComposing = isComposing;
        if (!isComposing) {
          this.inputValue = searchValue;
          this.updatePosition();
        }
      });
  }

  isComposingChange(isComposing: boolean): void {
    this.isComposingChange$.next(isComposing);
  }

  setDisabledState(isDisabled: boolean): void {
    this.nzDisabled =
      (this.isNzDisableFirstChange && this.nzDisabled) || isDisabled;
    this.closeDropdown();
    this.isNzDisableFirstChange = false;
  }

  private setStatusStyles(
    status: NzValidateStatus,
    hasFeedback: boolean,
  ): void {
    // set inner status
    this.status = status;
    this.hasFeedback = hasFeedback;
    this.cdr.markForCheck();
    // render status if nzStatus is set
    this.statusCls = getStatusClassNames(this.prefixCls, status, hasFeedback);
    Object.keys(this.statusCls).forEach((status) => {
      if (this.statusCls[status]) {
        this.renderer.addClass(this.elementRef.nativeElement, status);
      } else {
        this.renderer.removeClass(this.elementRef.nativeElement, status);
      }
    });
  }

  ngOnChanges({
    nzNodes,
    nzDropdownClassName,
    nzStatus,
    nzPlacement,
    nzSize,
    nzVariant,
  }: SimpleChanges): void {
    if (nzNodes) {
      this.updateSelectedNodes(true);
    }
    if (nzDropdownClassName) {
      const className =
        this.nzDropdownClassName && this.nzDropdownClassName.trim();
      this.dropdownClassName = className
        ? `${TREE_SELECT_DEFAULT_CLASS} ${className}`
        : TREE_SELECT_DEFAULT_CLASS;
    }
    if (nzStatus) {
      this.setStatusStyles(this.nzStatus, this.hasFeedback);
    }

    if (nzPlacement) {
      const { currentValue } = nzPlacement;
      this.placement.set(currentValue);
      const listOfPlacement = [
        "bottomLeft",
        "topLeft",
        "bottomRight",
        "topRight",
      ];
      if (currentValue && listOfPlacement.includes(currentValue)) {
        this.positions = [POSITION_MAP[currentValue as POSITION_TYPE]];
      } else {
        this.positions = listOfPlacement.map(
          (e) => POSITION_MAP[e as POSITION_TYPE],
        );
      }
    }
    if (nzSize) {
      this.size.set(nzSize.currentValue);
    }
    if (nzVariant) {
      this.variant.set(nzVariant.currentValue);
    }
  }

  writeValue(value: string[] | string): void {
    if (isNotNil(value)) {
      if (this.isMultiple && Array.isArray(value)) {
        this.value = value;
      } else {
        this.value = [value as string];
      }
      // need clear selected nodes when user set value before updating
      this.clearSelectedNodes();
      this.updateSelectedNodes(true);
    } else {
      this.value = [];
      this.clearSelectedNodes();
      this.selectedNodes = [];
    }
    this.cdr.markForCheck();
  }

  registerOnChange(fn: (_: string[] | string | null) => void): void {
    this.onChange = fn;
  }

  registerOnTouched(fn: () => void): void {
    this.onTouched = fn;
  }

  onKeydown(event: KeyboardEvent): void {
    if (this.nzDisabled) {
      return;
    }
    switch (event.keyCode) {
      case ESCAPE:
        /**
         * Skip the ESCAPE processing, it will be handled in {@link onOverlayKeyDown}.
         */
        break;
      case TAB:
        this.closeDropdown();
        break;
      default:
        if (!this.nzOpen) {
          this.openDropdown();
        }
    }
  }

  trigger(): void {
    if (this.nzDisabled || (!this.nzDisabled && this.nzOpen)) {
      this.closeDropdown();
    } else {
      this.openDropdown();
    }
  }

  openDropdown(): void {
    if (!this.nzDisabled) {
      this.nzOpen = true;
      this.nzOpenChange.emit(this.nzOpen);
      this.updateCdkConnectedOverlayStatus();
      if (this.nzShowSearch || this.isMultiple) {
        this.focusOnInput();
      }
    }
  }

  closeDropdown(): void {
    Promise.resolve().then(() => this.onTouched());
    this.nzOpen = false;
    this.inputValue = "";
    this.isNotFound = false;
    this.nzOpenChange.emit(this.nzOpen);
    this.cdr.markForCheck();
  }

  onKeyDownInput(e: KeyboardEvent): void {
    const keyCode = e.keyCode;
    const eventTarget = e.target as HTMLInputElement;
    if (this.isMultiple && !eventTarget.value && keyCode === BACKSPACE) {
      e.preventDefault();
      if (this.selectedNodes.length) {
        const removeNode = this.selectedNodes[this.selectedNodes.length - 1];
        if (removeNode && !removeNode.isDisabled) {
          this.removeSelected(removeNode);
        }
      }
    }
  }

  onExpandedKeysChange(value: NzFormatEmitEvent): void {
    this.nzExpandChange.emit(value);
    this.expandedKeys = [...value.keys!];
  }

  setInputValue(value: string): void {
    this.searchValueChange$.next(value);
  }

  removeSelected(node: NzTreeNode, emit: boolean = true): void {
    node.isSelected = false;
    node.isChecked = false;
    if (this.nzCheckable) {
      this.nzTreeService.conduct(node, this.nzCheckStrictly);
    } else {
      this.nzTreeService.setSelectedNodeList(node, this.nzMultiple);
    }

    if (emit) {
      this.nzRemoved.emit(node);
    }
  }

  focusOnInput(): void {
    if (this.nzSelectSearchComponent) {
      this.nzSelectSearchComponent.focus();
    }
  }

  subscribeSelectionChange(): void {
    merge(
      this.nzTreeClick.pipe(
        tap((event: NzFormatEmitEvent) => {
          const node = event.node!;
          if (this.nzCheckable && !node.isDisabled && !node.isDisableCheckbox) {
            node.isChecked = !node.isChecked;
            node.isHalfChecked = false;
            if (!this.nzCheckStrictly) {
              this.nzTreeService.conduct(node);
            }
          }
          if (this.nzCheckable) {
            node.isSelected = false;
          }
        }),
        filter((event: NzFormatEmitEvent) => {
          const node = event.node!;
          return this.nzCheckable
            ? !node.isDisabled && !node.isDisableCheckbox
            : !node.isDisabled && node.isSelectable;
        }),
      ),
      this.nzCheckable
        ? this.nzTreeCheckboxChange.asObservable()
        : observableOf(),
      this.nzCleared,
      this.nzRemoved,
    )
      .pipe(takeUntilDestroyed(this.destroyRef))
      .subscribe(() => {
        this.updateSelectedNodes();
        const value = this.selectedNodes.map((node) => node.key!);
        this.value = [...value];
        if (this.nzShowSearch || this.isMultiple) {
          this.inputValue = "";
          this.isNotFound = false;
        }
        if (this.isMultiple) {
          this.onChange(value);
          this.focusOnInput();
          this.updatePosition();
        } else {
          this.closeDropdown();
          this.onChange(value.length ? value[0] : null);
        }
      });
  }

  updateSelectedNodes(init: boolean = false): void {
    if (init) {
      const nodes = this.coerceTreeNodes(this.nzNodes);
      this.nzTreeService.isMultiple = this.isMultiple;
      this.nzTreeService.isCheckStrictly = this.nzCheckStrictly;
      this.nzTreeService.initTree(nodes);
      if (this.nzCheckable) {
        this.nzTreeService.conductCheck(this.value, this.nzCheckStrictly);
      } else {
        this.nzTreeService.conductSelectedKeys(this.value, this.isMultiple);
      }
    }

    this.selectedNodes = [
      ...(this.nzCheckable
        ? this.getCheckedNodeList()
        : this.getSelectedNodeList()),
    ].sort((a, b) => {
      const indexA = this.value.indexOf(a.key);
      const indexB = this.value.indexOf(b.key);
      if (indexA !== -1 && indexB !== -1) {
        return indexA - indexB;
      }
      if (indexA !== -1) {
        return -1;
      }
      if (indexB !== -1) {
        return 1;
      }
      return 0;
    });
  }

  updatePosition(): void {
    requestAnimationFrame(() => {
      this.cdkConnectedOverlay?.overlayRef?.updatePosition();
    });
  }

  onPositionChange(position: ConnectedOverlayPositionChange): void {
    const placement = getPlacementName(position);
    this.placement.set(placement as NzPlacementType);
  }

  onClearSelection(): void {
    this.selectedNodes.forEach((node) => {
      this.removeSelected(node, false);
    });
    this.nzCleared.emit();
  }

  onClickOutside(event: MouseEvent): void {
    const target = _getEventTarget(event);
    if (!this.elementRef.nativeElement.contains(target)) {
      this.closeDropdown();
    }
  }

  setSearchValues($event: NzFormatEmitEvent): void {
    Promise.resolve().then(() => {
      this.isNotFound =
        (this.nzShowSearch || this.isMultiple) &&
        !!this.inputValue &&
        $event.matchedKeys!.length === 0;
    });
  }

  updateCdkConnectedOverlayStatus(): void {
    if (
      this.platform.isBrowser &&
      this.cdkOverlayOrigin.elementRef.nativeElement
    ) {
      const triggerWidth = this.triggerWidth;
      cancelAnimationFrame(this.requestId);
      this.requestId = requestAnimationFrame(() => {
        // Blink triggers style and layout pipelines anytime the `getBoundingClientRect()` is called, which may cause a
        // frame drop. That's why it's scheduled through the `requestAnimationFrame` to unload the composite thread.
        this.triggerWidth =
          this.cdkOverlayOrigin.elementRef.nativeElement.getBoundingClientRect().width;
        if (triggerWidth !== this.triggerWidth) {
          // The `requestAnimationFrame` will trigger change detection, but we're inside an `OnPush` component which won't have
          // the `ChecksEnabled` state. Calling `markForCheck()` will allow Angular to run the change detection from the root component
          // down to the `nz-select`. But we'll trigger only local change detection if the `triggerWidth` has been changed.
          this.cdr.detectChanges();
        }
      });
    }
  }

  clearSelectedNodes(): void {
    this.selectedNodes.forEach((node) => {
      this.removeSelected(node, false);
    });
  }
}

`````
