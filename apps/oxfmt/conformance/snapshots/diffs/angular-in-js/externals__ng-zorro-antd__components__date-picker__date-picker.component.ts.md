# externals/ng-zorro-antd/components/date-picker/date-picker.component.ts

## Option 1

`````json
{"printWidth":80}
`````

### Diff

`````diff
===================================================================
--- prettier
+++ oxfmt
@@ -333,9 +333,12 @@
 
   readonly _nzModuleName: NzConfigKey = NZ_CONFIG_MODULE_NAME;
 
   static ngAcceptInputType_nzShowTime:
-    BooleanInput | SupportTimeOptions | null | undefined;
+    | BooleanInput
+    | SupportTimeOptions
+    | null
+    | undefined;
   static ngAcceptInputType_nzMode: NzDateMode | string;
 
   isRange = !!inject(NzRangePickerComponent, { self: true, optional: true }); // Indicate whether the value is a range value
   extraFooter?: TemplateRef<NzSafeAny> | string;
@@ -366,9 +369,11 @@
   @Input() nzStatus: NzStatus = "";
   @Input() nzFormat!: string;
   @Input() @WithConfig() nzVariant: NzVariant | undefined = undefined;
   @Input() nzDateRender?:
-    TemplateRef<NzSafeAny> | string | FunctionProp<TemplateRef<Date> | string>;
+    | TemplateRef<NzSafeAny>
+    | string
+    | FunctionProp<TemplateRef<Date> | string>;
   @Input() nzDisabledTime?: DisabledTimeFn;
   @Input() nzRenderExtraFooter?:
     | TemplateRef<NzSafeAny>
     | string

`````

### Actual (oxfmt)

`````ts
/**
 * Use of this source code is governed by an MIT-style license that can be
 * found in the LICENSE file at https://github.com/NG-ZORRO/ng-zorro-antd/blob/master/LICENSE
 */

import { Directionality } from "@angular/cdk/bidi";
import { ESCAPE } from "@angular/cdk/keycodes";
import {
  CdkConnectedOverlay,
  ConnectedOverlayPositionChange,
  ConnectionPositionPair,
  OriginConnectionPosition,
} from "@angular/cdk/overlay";
import { Platform } from "@angular/cdk/platform";
import { NgTemplateOutlet } from "@angular/common";
import {
  AfterViewInit,
  booleanAttribute,
  ChangeDetectionStrategy,
  ChangeDetectorRef,
  Component,
  computed,
  DestroyRef,
  DOCUMENT,
  ElementRef,
  EventEmitter,
  forwardRef,
  inject,
  input,
  Input,
  OnChanges,
  OnInit,
  Output,
  QueryList,
  Renderer2,
  signal,
  SimpleChanges,
  TemplateRef,
  ViewChild,
  ViewChildren,
  ViewEncapsulation,
} from "@angular/core";
import { takeUntilDestroyed } from "@angular/core/rxjs-interop";
import {
  ControlValueAccessor,
  FormsModule,
  NG_VALUE_ACCESSOR,
} from "@angular/forms";
import { of } from "rxjs";
import { distinctUntilChanged, map, withLatestFrom } from "rxjs/operators";

import { NzResizeObserver } from "ng-zorro-antd/cdk/resize-observer";
import {
  slideAnimationEnter,
  slideAnimationLeave,
} from "ng-zorro-antd/core/animation";
import {
  NzConfigKey,
  NzConfigService,
  WithConfig,
} from "ng-zorro-antd/core/config";
import {
  NZ_FORM_SIZE,
  NZ_FORM_VARIANT,
  NzFormItemFeedbackIconComponent,
  NzFormNoStatusService,
  NzFormStatusService,
} from "ng-zorro-antd/core/form";
import { NzOutletModule } from "ng-zorro-antd/core/outlet";
import {
  DATE_PICKER_POSITION_MAP,
  DEFAULT_DATE_PICKER_POSITIONS,
  NzOverlayModule,
} from "ng-zorro-antd/core/overlay";
import {
  CandyDate,
  cloneDate,
  CompatibleValue,
  wrongSortOrder,
} from "ng-zorro-antd/core/time";
import {
  BooleanInput,
  FunctionProp,
  NgClassInterface,
  NzSafeAny,
  NzSizeLDSType,
  NzStatus,
  NzValidateStatus,
  NzVariant,
  OnChangeType,
  OnTouchedType,
  type NzPlacement,
} from "ng-zorro-antd/core/types";
import {
  fromEventOutsideAngular,
  generateClassName,
  getStatusClassNames,
  toBoolean,
  valueFunctionProp,
} from "ng-zorro-antd/core/util";
import {
  DateHelperService,
  NzDatePickerI18nInterface,
  NzDatePickerLangI18nInterface,
  NzI18nService,
} from "ng-zorro-antd/i18n";
import { NzIconModule } from "ng-zorro-antd/icon";
import {
  NZ_SPACE_COMPACT_ITEM_TYPE,
  NZ_SPACE_COMPACT_SIZE,
  NzSpaceCompactItemDirective,
} from "ng-zorro-antd/space";

import { DatePickerService } from "./date-picker.service";
import { DateRangePopupComponent } from "./date-range-popup.component";
import { NzRangePickerComponent } from "./range-picker.component";
import {
  CompatibleDate,
  DisabledTimeFn,
  NzDateMode,
  NzPanelChangeType,
  PresetRanges,
  RangePartType,
  SupportTimeOptions,
} from "./standard-types";
import { PREFIX_CLASS } from "./util";

const POPUP_STYLE_PATCH = { position: "relative" }; // Aim to override antd's style to support overlay's position strategy (position:absolute will cause it not working because the overlay can't get the height/width of it's content)
const NZ_CONFIG_MODULE_NAME: NzConfigKey = "datePicker";

export type NzDatePickerSizeType = "large" | "default" | "small";

/**
 * The base picker for all common APIs
 */
@Component({
  encapsulation: ViewEncapsulation.None,
  changeDetection: ChangeDetectionStrategy.OnPush,
  selector:
    "nz-date-picker,nz-week-picker,nz-month-picker,nz-quarter-picker,nz-year-picker,nz-range-picker",
  exportAs: "nzDatePicker",
  template: `
    @if (!nzInline()) {
      @if (!isRange) {
        <div class="{{ prefixCls }}-input">
          <input
            #pickerInput
            [attr.id]="nzId"
            [class.ant-input-disabled]="nzDisabled"
            [disabled]="nzDisabled"
            [readOnly]="nzInputReadOnly"
            [(ngModel)]="inputValue"
            placeholder="{{ getPlaceholder() }}"
            [size]="inputSize"
            autocomplete="off"
            (focus)="onFocus($event)"
            (focusout)="onFocusout($event)"
            (ngModelChange)="onInputChange($event)"
            (keyup.enter)="onKeyupEnter($event)"
          />
          <ng-container *ngTemplateOutlet="tplRightRest" />
        </div>
      } @else {
        <div class="{{ prefixCls }}-input">
          <ng-container
            *ngTemplateOutlet="tplRangeInput; context: { partType: 'left' }"
          />
        </div>
        <div #separatorElement class="{{ prefixCls }}-range-separator">
          <span class="{{ prefixCls }}-separator">
            <ng-container *nzStringTemplateOutlet="nzSeparator; let separator">
              @if (nzSeparator) {
                {{ nzSeparator }}
              } @else {
                <nz-icon nzType="swap-right" nzTheme="outline" />
              }
            </ng-container>
          </span>
        </div>
        <div class="{{ prefixCls }}-input">
          <ng-container
            *ngTemplateOutlet="tplRangeInput; context: { partType: 'right' }"
          />
        </div>
        <ng-container *ngTemplateOutlet="tplRightRest" />
      }
    } @else {
      <ng-template [ngTemplateOutlet]="inlineMode" />
    }
    <!-- Input for Range ONLY -->
    <ng-template #tplRangeInput let-partType="partType">
      <input
        #rangePickerInput
        [attr.id]="nzId"
        [disabled]="nzDisabled"
        [readOnly]="nzInputReadOnly"
        [size]="inputSize"
        autocomplete="off"
        (click)="onClickInputBox($event)"
        (focusout)="onFocusout($event)"
        (focus)="onFocus($event, partType)"
        (keyup.enter)="onKeyupEnter($event)"
        [(ngModel)]="inputValue[datePickerService.getActiveIndex(partType)]"
        (ngModelChange)="onInputChange($event)"
        placeholder="{{ getPlaceholder(partType) }}"
      />
    </ng-template>

    <!-- Right operator icons -->
    <ng-template #tplRightRest>
      <div class="{{ prefixCls }}-active-bar" [style]="activeBarStyle"></div>
      @if (showClear) {
        <span class="{{ prefixCls }}-clear" (click)="onClickClear($event)">
          <nz-icon nzType="close-circle" nzTheme="fill" />
        </span>
      }

      <span class="{{ prefixCls }}-suffix">
        <ng-container *nzStringTemplateOutlet="nzSuffixIcon; let suffixIcon">
          <nz-icon [nzType]="suffixIcon" />
        </ng-container>
        @if (hasFeedback && !!status) {
          <nz-form-item-feedback-icon [status]="status" />
        }
      </span>
    </ng-template>

    <ng-template #inlineMode>
      <div
        [class]="dropdownClass()"
        [class.ant-picker-dropdown-range]="!nzInline() && isRange"
        [class.ant-picker-active-left]="
          datePickerService.activeInput === 'left'
        "
        [class.ant-picker-active-right]="
          datePickerService.activeInput === 'right'
        "
        [style]="nzPopupStyle"
        [animate.enter]="$any(!nzInline() && datepickerAnimationEnter())"
        [animate.leave]="$any(!nzInline() && datepickerAnimationLeave())"
      >
        <date-range-popup
          [isRange]="isRange"
          [inline]="nzInline()"
          [defaultPickerValue]="nzDefaultPickerValue"
          [showWeek]="nzShowWeekNumber || nzMode === 'week'"
          [panelMode]="panelMode"
          (panelModeChange)="onPanelModeChange($event)"
          (calendarChange)="onCalendarChange($event)"
          [locale]="nzLocale?.lang!"
          [showToday]="
            nzMode === 'date' && nzShowToday && !isRange && !nzShowTime
          "
          [showNow]="nzMode === 'date' && nzShowNow && !isRange && !!nzShowTime"
          [showTime]="nzShowTime"
          [dateRender]="nzDateRender"
          [disabledDate]="nzDisabledDate"
          [disabledTime]="nzDisabledTime"
          [extraFooter]="extraFooter"
          [ranges]="nzRanges"
          [dir]="dir()"
          [format]="nzFormat"
          (resultOk)="onResultOk()"
        />
      </div>
    </ng-template>

    <!-- Overlay -->
    <ng-template
      cdkConnectedOverlay
      nzConnectedOverlay
      cdkConnectedOverlayTransformOriginOn=".{{ prefixCls }}-dropdown"
      [cdkConnectedOverlayHasBackdrop]="nzBackdrop"
      [cdkConnectedOverlayOrigin]="origin"
      [cdkConnectedOverlayOpen]="realOpenState"
      [cdkConnectedOverlayPositions]="overlayPositions"
      (positionChange)="onPositionChange($event)"
      (detach)="close()"
      (overlayKeydown)="onOverlayKeydown($event)"
    >
      <ng-container *ngTemplateOutlet="inlineMode" />
    </ng-template>
  `,
  host: {
    "[class.ant-picker]": `true`,
    "[class.ant-picker-range]": `isRange`,
    "[class.ant-picker-large]": `finalSize() === 'large'`,
    "[class.ant-picker-small]": `finalSize() === 'small'`,
    "[class.ant-picker-disabled]": `nzDisabled`,
    "[class.ant-picker-rtl]": `dir() === 'rtl'`,
    "[class.ant-picker-outlined]": `finalVariant() === 'outlined'`,
    "[class.ant-picker-borderless]": `finalVariant() === 'borderless'`,
    "[class.ant-picker-filled]": `finalVariant() === 'filled'`,
    "[class.ant-picker-underlined]": `finalVariant() === 'underlined'`,
    "[class.ant-picker-inline]": `nzInline()`,
    "(click)": "onClickInputBox($event)",
  },
  hostDirectives: [NzSpaceCompactItemDirective],
  providers: [
    DatePickerService,
    { provide: NZ_SPACE_COMPACT_ITEM_TYPE, useValue: "picker" },
    {
      provide: NG_VALUE_ACCESSOR,
      multi: true,
      useExisting: forwardRef(() => NzDatePickerComponent),
    },
  ],
  imports: [
    FormsModule,
    NgTemplateOutlet,
    NzOutletModule,
    NzIconModule,
    NzFormItemFeedbackIconComponent,
    DateRangePopupComponent,
    CdkConnectedOverlay,
    NzOverlayModule,
  ],
})
export class NzDatePickerComponent
  implements OnInit, OnChanges, AfterViewInit, ControlValueAccessor
{
  public nzConfigService = inject(NzConfigService);
  public datePickerService = inject(DatePickerService);
  protected i18n = inject(NzI18nService);
  protected cdr = inject(ChangeDetectorRef);
  private renderer = inject(Renderer2);
  private elementRef = inject(ElementRef<HTMLElement>);
  private dateHelper = inject(DateHelperService);
  private nzResizeObserver = inject(NzResizeObserver);
  private platform = inject(Platform);
  private destroyRef = inject(DestroyRef);
  protected readonly dir = inject(Directionality).valueSignal;

  readonly _nzModuleName: NzConfigKey = NZ_CONFIG_MODULE_NAME;

  static ngAcceptInputType_nzShowTime:
    | BooleanInput
    | SupportTimeOptions
    | null
    | undefined;
  static ngAcceptInputType_nzMode: NzDateMode | string;

  isRange = !!inject(NzRangePickerComponent, { self: true, optional: true }); // Indicate whether the value is a range value
  extraFooter?: TemplateRef<NzSafeAny> | string;

  // status
  statusCls: NgClassInterface = {};
  status: NzValidateStatus = "";
  hasFeedback: boolean = false;

  public panelMode: NzDateMode | NzDateMode[] = "date";
  private isCustomPlaceHolder: boolean = false;
  private isCustomFormat: boolean = false;
  private showTime: SupportTimeOptions | boolean = false;
  private isNzDisableFirstChange: boolean = true;
  // --- Common API
  readonly nzInline = input(false, { transform: booleanAttribute });
  @Input({ transform: booleanAttribute }) nzAllowClear: boolean = true;
  @Input({ transform: booleanAttribute }) nzAutoFocus: boolean = false;
  @Input({ transform: booleanAttribute }) nzDisabled: boolean = false;
  @Input({ transform: booleanAttribute }) nzInputReadOnly: boolean = false;
  @Input({ transform: booleanAttribute }) nzOpen?: boolean;
  @Input() nzDisabledDate?: (d: Date) => boolean;
  @Input() nzLocale!: NzDatePickerI18nInterface;
  @Input() nzPlaceHolder: string | string[] = "";
  @Input() nzPopupStyle: object = POPUP_STYLE_PATCH;
  readonly nzDropdownClassName = input<string>();
  @Input() nzSize: NzDatePickerSizeType = "default";
  @Input() nzStatus: NzStatus = "";
  @Input() nzFormat!: string;
  @Input() @WithConfig() nzVariant: NzVariant | undefined = undefined;
  @Input() nzDateRender?:
    | TemplateRef<NzSafeAny>
    | string
    | FunctionProp<TemplateRef<Date> | string>;
  @Input() nzDisabledTime?: DisabledTimeFn;
  @Input() nzRenderExtraFooter?:
    | TemplateRef<NzSafeAny>
    | string
    | FunctionProp<TemplateRef<NzSafeAny> | string>;
  @Input({ transform: booleanAttribute }) nzShowToday: boolean = true;
  @Input() nzMode: NzDateMode = "date";
  @Input({ transform: booleanAttribute }) nzShowNow: boolean = true;
  @Input() nzRanges?: PresetRanges;
  @Input() nzDefaultPickerValue: CompatibleDate | null = null;
  @Input() @WithConfig() nzSeparator?: string | TemplateRef<NzSafeAny> =
    undefined;
  @Input() @WithConfig() nzSuffixIcon: string | TemplateRef<NzSafeAny> =
    "calendar";
  @Input() @WithConfig() nzBackdrop = false;
  @Input() nzId: string | null = null;
  @Input() nzPlacement: NzPlacement = "bottomLeft";
  @Input({ transform: booleanAttribute }) nzShowWeekNumber: boolean = false;

  @Output() readonly nzOnPanelChange = new EventEmitter<NzPanelChangeType>();
  @Output() readonly nzOnCalendarChange = new EventEmitter<
    Array<Date | null>
  >();
  @Output() readonly nzOnOk = new EventEmitter<CompatibleDate | null>();
  @Output() readonly nzOnOpenChange = new EventEmitter<boolean>();

  @Input() get nzShowTime(): SupportTimeOptions | boolean {
    return this.showTime;
  }

  set nzShowTime(value: SupportTimeOptions | boolean) {
    this.showTime = typeof value === "object" ? value : toBoolean(value);
  }

  // ------------------------------------------------------------------------
  // Input API Start
  // ------------------------------------------------------------------------
  @ViewChild(CdkConnectedOverlay, { static: false })
  cdkConnectedOverlay?: CdkConnectedOverlay;
  @ViewChild(DateRangePopupComponent, { static: false })
  panel!: DateRangePopupComponent;
  @ViewChild("separatorElement", { static: false })
  separatorElement?: ElementRef;
  @ViewChild("pickerInput", { static: false })
  pickerInput?: ElementRef<HTMLInputElement>;
  @ViewChildren("rangePickerInput") rangePickerInputs?: QueryList<
    ElementRef<HTMLInputElement>
  >;

  get origin(): ElementRef {
    return this.elementRef;
  }

  inputSize: number = 12;
  inputWidth?: number;
  prefixCls = PREFIX_CLASS;
  inputValue!: NzSafeAny;
  activeBarStyle: object = {};
  overlayOpen: boolean = false; // Available when "nzOpen" = undefined
  overlayPositions: ConnectionPositionPair[] = [
    ...DEFAULT_DATE_PICKER_POSITIONS,
  ];

  private readonly currentPosition = signal<OriginConnectionPosition>({
    originX: "start",
    originY: "bottom",
  });

  protected readonly datepickerAnimationEnter = slideAnimationEnter();
  protected readonly datepickerAnimationLeave = slideAnimationLeave();

  protected readonly dropdownClass = computed(() => {
    const cls: string[] = [];
    if (!this.nzInline()) {
      cls.push(this.generateClass("dropdown"));

      const customCls = this.nzDropdownClassName();
      if (customCls) {
        cls.push(customCls);
      }

      const { originX, originY } = this.currentPosition();

      if (originX === "start" && originY === "bottom") {
        cls.push(this.generateClass("dropdown-placement-bottomLeft"));
      } else if (originX === "start" && originY === "top") {
        cls.push(this.generateClass("dropdown-placement-topLeft"));
      } else if (originX === "end" && originY === "bottom") {
        cls.push(this.generateClass("dropdown-placement-bottomRight"));
      } else if (originX === "end" && originY === "top") {
        cls.push(this.generateClass("dropdown-placement-topRight"));
      }

      if (this.dir() === "rtl") {
        cls.push(this.generateClass("dropdown-rtl"));
      }
    }
    return cls;
  });

  get realOpenState(): boolean {
    // The value that really decide the open state of overlay
    return this.isOpenHandledByUser() ? !!this.nzOpen : this.overlayOpen;
  }

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

  private size = signal<NzSizeLDSType>(this.nzSize);
  private variant = signal<NzVariant | undefined>(this.nzVariant);

  private readonly formSize = inject(NZ_FORM_SIZE, { optional: true });
  private readonly formVariant = inject(NZ_FORM_VARIANT, { optional: true });

  private compactSize = inject(NZ_SPACE_COMPACT_SIZE, { optional: true });
  private document: Document = inject(DOCUMENT);

  constructor() {
    this.datePickerService.isRange = this.isRange;
    this.datePickerService.initValue(true);
  }

  ngAfterViewInit(): void {
    if (this.nzAutoFocus) {
      this.focus();
    }

    if (this.isRange && this.platform.isBrowser) {
      this.nzResizeObserver
        .observe(this.elementRef)
        .pipe(takeUntilDestroyed(this.destroyRef))
        .subscribe(() => {
          this.updateInputWidthAndArrowLeft();
        });
    }

    this.datePickerService.inputPartChange$
      .pipe(takeUntilDestroyed(this.destroyRef))
      .subscribe((partType) => {
        if (partType) {
          this.datePickerService.activeInput = partType;
        }
        this.focus();
        this.updateInputWidthAndArrowLeft();
      });

    if (this.platform.isBrowser) {
      // prevent mousedown event to trigger focusout event when click in date picker
      // see: https://github.com/NG-ZORRO/ng-zorro-antd/issues/7450
      fromEventOutsideAngular(this.elementRef.nativeElement, "mousedown")
        .pipe(takeUntilDestroyed(this.destroyRef))
        .subscribe((event) => {
          if (
            (event.target as HTMLInputElement).tagName.toLowerCase() !== "input"
          ) {
            event.preventDefault();
          }
        });
    }
  }

  updateInputWidthAndArrowLeft(): void {
    this.inputWidth =
      this.rangePickerInputs?.first?.nativeElement.offsetWidth || 0;

    const baseStyle = { position: "absolute", width: `${this.inputWidth}px` };
    this.datePickerService.arrowLeft =
      this.datePickerService.activeInput === "left"
        ? 0
        : this.inputWidth + this.separatorElement?.nativeElement.offsetWidth ||
          0;

    if (this.dir() === "rtl") {
      this.activeBarStyle = {
        ...baseStyle,
        right: `${this.datePickerService.arrowLeft}px`,
      };
    } else {
      this.activeBarStyle = {
        ...baseStyle,
        left: `${this.datePickerService.arrowLeft}px`,
      };
    }

    this.cdr.markForCheck();
  }

  getInput(partType?: RangePartType): HTMLInputElement | undefined {
    if (this.nzInline()) {
      return undefined;
    }
    return this.isRange
      ? partType === "left"
        ? this.rangePickerInputs?.first.nativeElement
        : this.rangePickerInputs?.last.nativeElement
      : this.pickerInput!.nativeElement;
  }

  focus(): void {
    const activeInputElement = this.getInput(
      this.datePickerService.activeInput,
    );
    if (this.document.activeElement !== activeInputElement) {
      activeInputElement?.focus();
    }
  }

  onFocus(event: FocusEvent, partType?: RangePartType): void {
    event.preventDefault();
    if (partType) {
      this.datePickerService.inputPartChange$.next(partType);
    }
    this.renderClass(true);
  }

  // blur event has not the relatedTarget in IE11, use focusout instead.
  onFocusout(event: FocusEvent): void {
    event.preventDefault();
    this.onTouchedFn();
    if (!this.elementRef.nativeElement.contains(event.relatedTarget as Node)) {
      this.checkAndClose();
    }
    this.renderClass(false);
  }

  // Show overlay content
  open(): void {
    if (this.nzInline()) {
      return;
    }
    if (!this.realOpenState && !this.nzDisabled) {
      this.updateInputWidthAndArrowLeft();
      this.overlayOpen = true;
      this.nzOnOpenChange.emit(true);
      this.focus();
      this.cdr.markForCheck();
    }
  }

  close(): void {
    if (this.nzInline()) {
      return;
    }
    if (this.realOpenState) {
      this.overlayOpen = false;
      this.nzOnOpenChange.emit(false);
    }
  }

  get showClear(): boolean {
    return (
      !this.nzDisabled &&
      !this.isEmptyValue(this.datePickerService.value) &&
      this.nzAllowClear
    );
  }

  checkAndClose(): void {
    if (!this.realOpenState) {
      return;
    }

    if (this.panel.isAllowed(this.datePickerService.value!, true)) {
      if (
        Array.isArray(this.datePickerService.value) &&
        wrongSortOrder(this.datePickerService.value)
      ) {
        const index = this.datePickerService.getActiveIndex();
        const value = this.datePickerService.value[index];
        this.panel.changeValueFromSelect(value!, true);
        return;
      }
      this.updateInputValue();
      this.datePickerService.emitValue$.next();
    } else {
      this.datePickerService.setValue(this.datePickerService.initialValue!);
      this.close();
    }
  }

  onClickInputBox(event: MouseEvent): void {
    event.stopPropagation();
    this.focus();
    if (!this.isOpenHandledByUser()) {
      this.open();
    }
  }

  onOverlayKeydown(event: KeyboardEvent): void {
    if (event.keyCode === ESCAPE) {
      this.datePickerService.initValue();
    }
  }

  onPositionChange(position: ConnectedOverlayPositionChange): void {
    this.currentPosition.set(position.connectionPair);
  }

  onClickClear(event: MouseEvent): void {
    event.preventDefault();
    event.stopPropagation();

    this.datePickerService.initValue(true);
    this.datePickerService.emitValue$.next();
  }

  updateInputValue(): void {
    const newValue = this.datePickerService.value;
    if (this.isRange) {
      this.inputValue = newValue
        ? (newValue as CandyDate[]).map((v) => this.formatValue(v))
        : ["", ""];
    } else {
      this.inputValue = this.formatValue(newValue as CandyDate);
    }
    this.cdr.markForCheck();
  }

  formatValue(value: CandyDate): string {
    return this.dateHelper.format(
      value && (value as CandyDate).nativeDate,
      this.nzFormat,
    );
  }

  onInputChange(value: string, isEnter: boolean = false): void {
    /**
     * in IE11 focus/blur will trigger ngModelChange if placeholder changes,
     * so we forbid IE11 to open panel through input change
     */
    if (
      !this.platform.TRIDENT &&
      this.document.activeElement ===
        this.getInput(this.datePickerService.activeInput) &&
      !this.realOpenState
    ) {
      this.open();
      return;
    }

    const date = this.checkValidDate(value);
    // Can only change date when it's open
    if (date && this.realOpenState) {
      this.panel.changeValueFromSelect(date, isEnter);
    }
  }

  onKeyupEnter(event: Event): void {
    this.onInputChange((event.target as HTMLInputElement).value, true);
  }

  private checkValidDate(value: string): CandyDate | null {
    const date = new CandyDate(this.dateHelper.parseDate(value, this.nzFormat));

    if (
      !date.isValid() ||
      value !== this.dateHelper.format(date.nativeDate, this.nzFormat)
    ) {
      return null;
    }

    return date;
  }

  getPlaceholder(partType?: RangePartType): string {
    return this.isRange
      ? this.nzPlaceHolder[this.datePickerService.getActiveIndex(partType!)]
      : (this.nzPlaceHolder as string);
  }

  isEmptyValue(value: CompatibleValue): boolean {
    if (value === null) {
      return true;
    } else if (this.isRange) {
      return !value || !Array.isArray(value) || value.every((val) => !val);
    } else {
      return !value;
    }
  }

  // Whether open state is permanently controlled by user himself
  isOpenHandledByUser(): boolean {
    return this.nzOpen !== undefined;
  }

  private nzFormStatusService = inject(NzFormStatusService, { optional: true });
  private nzFormNoStatusService = inject(NzFormNoStatusService, {
    optional: true,
  });

  // ------------------------------------------------------------------------
  // Input API End
  // ------------------------------------------------------------------------

  ngOnInit(): void {
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
            : of(false),
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

    // Subscribe the every locale change if the nzLocale is not handled by user
    if (!this.nzLocale) {
      this.i18n.localeChange
        .pipe(takeUntilDestroyed(this.destroyRef))
        .subscribe(() => this.setLocale());
    }

    this.datePickerService.emitValue$
      .pipe(takeUntilDestroyed(this.destroyRef))
      .subscribe(() => {
        const granularityComparison = this.showTime ? "second" : "day";
        const value = this.datePickerService.value;
        const datePickerPreviousValue = this.datePickerService.initialValue;

        // Check if the value has change for a simple datepicker, let us avoid notify the control for nothing
        if (
          !this.isRange &&
          (value as CandyDate)?.isSame(
            (datePickerPreviousValue as CandyDate)?.nativeDate,
            granularityComparison,
          )
        ) {
          this.onTouchedFn();
          return this.close();
        }

        // check if the value has change for a range picker, let us avoid notify the control for nothing
        if (this.isRange) {
          const [previousStartDate, previousEndDate] =
            datePickerPreviousValue as CandyDate[];
          const [currentStartDate, currentEndDate] = value as CandyDate[];
          if (
            previousStartDate?.isSame(
              currentStartDate?.nativeDate,
              granularityComparison,
            ) &&
            previousEndDate?.isSame(
              currentEndDate?.nativeDate,
              granularityComparison,
            )
          ) {
            this.onTouchedFn();
            return this.close();
          }
        }

        this.datePickerService.initialValue = cloneDate(value);
        if (this.isRange) {
          const vAsRange = value as CandyDate[];
          if (vAsRange.length) {
            this.onChangeFn([
              vAsRange[0]?.nativeDate ?? null,
              vAsRange[1]?.nativeDate ?? null,
            ]);
          } else {
            this.onChangeFn([]);
          }
        } else {
          if (value) {
            this.onChangeFn((value as CandyDate).nativeDate);
          } else {
            this.onChangeFn(null);
          }
        }
        this.onTouchedFn();
        // When value emitted, overlay will be closed
        this.close();
      });

    this.inputValue = this.isRange ? ["", ""] : "";
    this.setModeAndFormat();

    this.datePickerService.valueChange$
      .pipe(takeUntilDestroyed(this.destroyRef))
      .subscribe(() => {
        this.updateInputValue();
      });
  }

  ngOnChanges({
    nzStatus,
    nzPlacement,
    nzPopupStyle,
    nzPlaceHolder,
    nzLocale,
    nzFormat,
    nzRenderExtraFooter,
    nzMode,
    nzSize,
    nzVariant,
  }: SimpleChanges): void {
    if (nzPopupStyle) {
      // Always assign the popup style patch
      this.nzPopupStyle = this.nzPopupStyle
        ? { ...this.nzPopupStyle, ...POPUP_STYLE_PATCH }
        : POPUP_STYLE_PATCH;
    }

    // Mark as customized placeholder by user once nzPlaceHolder assigned at the first time
    if (nzPlaceHolder?.currentValue) {
      this.isCustomPlaceHolder = true;
    }

    if (nzFormat?.currentValue) {
      this.isCustomFormat = true;
      this.updateInputValue();
    }

    if (nzLocale) {
      // The nzLocale is currently handled by user
      this.setDefaultPlaceHolder();
    }

    if (nzRenderExtraFooter) {
      this.extraFooter = valueFunctionProp(this.nzRenderExtraFooter!);
    }

    if (nzMode) {
      this.setDefaultPlaceHolder();
      this.setModeAndFormat();
    }

    if (nzStatus) {
      this.setStatusStyles(this.nzStatus, this.hasFeedback);
    }

    if (nzPlacement) {
      this.setPlacement(this.nzPlacement);
    }

    if (nzSize) {
      this.size.set(nzSize.currentValue);
    }

    if (nzVariant) {
      this.variant.set(nzVariant.currentValue);
    }
  }

  setModeAndFormat(): void {
    const inputFormats: Partial<Record<NzDateMode, string>> = {
      year: "yyyy",
      quarter: "yyyy-[Q]Q",
      month: "yyyy-MM",
      week: "YYYY-ww",
      date: this.nzShowTime ? "yyyy-MM-dd HH:mm:ss" : "yyyy-MM-dd",
    };

    if (!this.nzMode) {
      this.nzMode = "date";
    }

    this.panelMode = this.isRange ? [this.nzMode, this.nzMode] : this.nzMode;

    // Default format when it's empty
    if (!this.isCustomFormat) {
      this.nzFormat = inputFormats[this.nzMode as NzDateMode]!;
    }

    this.inputSize = Math.max(10, this.nzFormat.length) + 2;
    this.updateInputValue();
  }

  /**
   * Triggered when overlayOpen changes (different with realOpenState)
   *
   * @param open The overlayOpen in picker component
   */
  onOpenChange(open: boolean): void {
    this.nzOnOpenChange.emit(open);
  }

  // ------------------------------------------------------------------------
  // | Control value accessor implements
  // ------------------------------------------------------------------------

  // NOTE: onChangeFn/onTouchedFn will not be assigned if user not use as ngModel
  onChangeFn: OnChangeType = () => void 0;
  onTouchedFn: OnTouchedType = () => void 0;

  writeValue(value: CompatibleDate): void {
    this.setValue(value);
    this.cdr.markForCheck();
  }

  registerOnChange(fn: OnChangeType): void {
    this.onChangeFn = fn;
  }

  registerOnTouched(fn: OnTouchedType): void {
    this.onTouchedFn = fn;
  }

  setDisabledState(isDisabled: boolean): void {
    this.nzDisabled =
      (this.isNzDisableFirstChange && this.nzDisabled) || isDisabled;
    this.cdr.markForCheck();
    this.isNzDisableFirstChange = false;
  }

  // ------------------------------------------------------------------------
  // | Internal methods
  // ------------------------------------------------------------------------

  // Reload locale from i18n with side effects
  private setLocale(): void {
    this.nzLocale = this.i18n.getLocaleData("DatePicker", {});
    this.setDefaultPlaceHolder();
    this.cdr.markForCheck();
  }

  private setDefaultPlaceHolder(): void {
    if (!this.isCustomPlaceHolder && this.nzLocale) {
      const defaultPlaceholder: Partial<Record<NzDateMode, string>> = {
        year: this.getPropertyOfLocale("yearPlaceholder"),
        quarter: this.getPropertyOfLocale("quarterPlaceholder"),
        month: this.getPropertyOfLocale("monthPlaceholder"),
        week: this.getPropertyOfLocale("weekPlaceholder"),
        date: this.getPropertyOfLocale("placeholder"),
      };

      const defaultRangePlaceholder: Partial<Record<NzDateMode, string[]>> = {
        year: this.getPropertyOfLocale("rangeYearPlaceholder"),
        quarter: this.getPropertyOfLocale("rangeQuarterPlaceholder"),
        month: this.getPropertyOfLocale("rangeMonthPlaceholder"),
        week: this.getPropertyOfLocale("rangeWeekPlaceholder"),
        date: this.getPropertyOfLocale("rangePlaceholder"),
      };

      this.nzPlaceHolder = this.isRange
        ? defaultRangePlaceholder[this.nzMode as NzDateMode]!
        : defaultPlaceholder[this.nzMode as NzDateMode]!;
    }
  }

  private getPropertyOfLocale<T extends keyof NzDatePickerLangI18nInterface>(
    type: T,
  ): NzDatePickerLangI18nInterface[T] {
    return (
      this.nzLocale.lang[type] ||
      this.i18n.getLocaleData(`DatePicker.lang.${type}`)
    );
  }

  // Safe way of setting value with default
  private setValue(value: CompatibleDate): void {
    const newValue = this.datePickerService.makeValue(value);
    this.datePickerService.setValue(newValue);
    this.datePickerService.initialValue = cloneDate(newValue);
    this.cdr.detectChanges();
  }

  renderClass(value: boolean): void {
    // TODO: avoid autoFocus cause change after checked error
    if (value) {
      this.renderer.addClass(
        this.elementRef.nativeElement,
        "ant-picker-focused",
      );
    } else {
      this.renderer.removeClass(
        this.elementRef.nativeElement,
        "ant-picker-focused",
      );
    }
  }

  onPanelModeChange(panelChange: NzPanelChangeType): void {
    this.nzOnPanelChange.emit(panelChange);
  }

  // Emit nzOnCalendarChange when select date by nz-range-picker
  onCalendarChange(value: CompatibleValue): void {
    if (this.isRange && Array.isArray(value)) {
      const rangeValue = value
        .filter((x) => x instanceof CandyDate)
        .map((x) => x!.nativeDate);
      this.nzOnCalendarChange.emit(rangeValue);
    }
  }

  onResultOk(): void {
    if (this.isRange) {
      const value = this.datePickerService.value as CandyDate[];
      if (value.length) {
        this.nzOnOk.emit([
          value[0]?.nativeDate || null,
          value[1]?.nativeDate || null,
        ]);
      } else {
        this.nzOnOk.emit([]);
      }
    } else {
      if (this.datePickerService.value) {
        this.nzOnOk.emit(
          (this.datePickerService.value as CandyDate).nativeDate,
        );
      } else {
        this.nzOnOk.emit(null);
      }
    }
  }

  // status
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

  private setPlacement(placement: NzPlacement): void {
    const position: ConnectionPositionPair =
      DATE_PICKER_POSITION_MAP[placement];
    this.overlayPositions = [position, ...DEFAULT_DATE_PICKER_POSITIONS];
    this.currentPosition.set(position);
  }

  private generateClass(suffix: string): string {
    return generateClassName(this.prefixCls, suffix);
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
import { ESCAPE } from "@angular/cdk/keycodes";
import {
  CdkConnectedOverlay,
  ConnectedOverlayPositionChange,
  ConnectionPositionPair,
  OriginConnectionPosition,
} from "@angular/cdk/overlay";
import { Platform } from "@angular/cdk/platform";
import { NgTemplateOutlet } from "@angular/common";
import {
  AfterViewInit,
  booleanAttribute,
  ChangeDetectionStrategy,
  ChangeDetectorRef,
  Component,
  computed,
  DestroyRef,
  DOCUMENT,
  ElementRef,
  EventEmitter,
  forwardRef,
  inject,
  input,
  Input,
  OnChanges,
  OnInit,
  Output,
  QueryList,
  Renderer2,
  signal,
  SimpleChanges,
  TemplateRef,
  ViewChild,
  ViewChildren,
  ViewEncapsulation,
} from "@angular/core";
import { takeUntilDestroyed } from "@angular/core/rxjs-interop";
import {
  ControlValueAccessor,
  FormsModule,
  NG_VALUE_ACCESSOR,
} from "@angular/forms";
import { of } from "rxjs";
import { distinctUntilChanged, map, withLatestFrom } from "rxjs/operators";

import { NzResizeObserver } from "ng-zorro-antd/cdk/resize-observer";
import {
  slideAnimationEnter,
  slideAnimationLeave,
} from "ng-zorro-antd/core/animation";
import {
  NzConfigKey,
  NzConfigService,
  WithConfig,
} from "ng-zorro-antd/core/config";
import {
  NZ_FORM_SIZE,
  NZ_FORM_VARIANT,
  NzFormItemFeedbackIconComponent,
  NzFormNoStatusService,
  NzFormStatusService,
} from "ng-zorro-antd/core/form";
import { NzOutletModule } from "ng-zorro-antd/core/outlet";
import {
  DATE_PICKER_POSITION_MAP,
  DEFAULT_DATE_PICKER_POSITIONS,
  NzOverlayModule,
} from "ng-zorro-antd/core/overlay";
import {
  CandyDate,
  cloneDate,
  CompatibleValue,
  wrongSortOrder,
} from "ng-zorro-antd/core/time";
import {
  BooleanInput,
  FunctionProp,
  NgClassInterface,
  NzSafeAny,
  NzSizeLDSType,
  NzStatus,
  NzValidateStatus,
  NzVariant,
  OnChangeType,
  OnTouchedType,
  type NzPlacement,
} from "ng-zorro-antd/core/types";
import {
  fromEventOutsideAngular,
  generateClassName,
  getStatusClassNames,
  toBoolean,
  valueFunctionProp,
} from "ng-zorro-antd/core/util";
import {
  DateHelperService,
  NzDatePickerI18nInterface,
  NzDatePickerLangI18nInterface,
  NzI18nService,
} from "ng-zorro-antd/i18n";
import { NzIconModule } from "ng-zorro-antd/icon";
import {
  NZ_SPACE_COMPACT_ITEM_TYPE,
  NZ_SPACE_COMPACT_SIZE,
  NzSpaceCompactItemDirective,
} from "ng-zorro-antd/space";

import { DatePickerService } from "./date-picker.service";
import { DateRangePopupComponent } from "./date-range-popup.component";
import { NzRangePickerComponent } from "./range-picker.component";
import {
  CompatibleDate,
  DisabledTimeFn,
  NzDateMode,
  NzPanelChangeType,
  PresetRanges,
  RangePartType,
  SupportTimeOptions,
} from "./standard-types";
import { PREFIX_CLASS } from "./util";

const POPUP_STYLE_PATCH = { position: "relative" }; // Aim to override antd's style to support overlay's position strategy (position:absolute will cause it not working because the overlay can't get the height/width of it's content)
const NZ_CONFIG_MODULE_NAME: NzConfigKey = "datePicker";

export type NzDatePickerSizeType = "large" | "default" | "small";

/**
 * The base picker for all common APIs
 */
@Component({
  encapsulation: ViewEncapsulation.None,
  changeDetection: ChangeDetectionStrategy.OnPush,
  selector:
    "nz-date-picker,nz-week-picker,nz-month-picker,nz-quarter-picker,nz-year-picker,nz-range-picker",
  exportAs: "nzDatePicker",
  template: `
    @if (!nzInline()) {
      @if (!isRange) {
        <div class="{{ prefixCls }}-input">
          <input
            #pickerInput
            [attr.id]="nzId"
            [class.ant-input-disabled]="nzDisabled"
            [disabled]="nzDisabled"
            [readOnly]="nzInputReadOnly"
            [(ngModel)]="inputValue"
            placeholder="{{ getPlaceholder() }}"
            [size]="inputSize"
            autocomplete="off"
            (focus)="onFocus($event)"
            (focusout)="onFocusout($event)"
            (ngModelChange)="onInputChange($event)"
            (keyup.enter)="onKeyupEnter($event)"
          />
          <ng-container *ngTemplateOutlet="tplRightRest" />
        </div>
      } @else {
        <div class="{{ prefixCls }}-input">
          <ng-container
            *ngTemplateOutlet="tplRangeInput; context: { partType: 'left' }"
          />
        </div>
        <div #separatorElement class="{{ prefixCls }}-range-separator">
          <span class="{{ prefixCls }}-separator">
            <ng-container *nzStringTemplateOutlet="nzSeparator; let separator">
              @if (nzSeparator) {
                {{ nzSeparator }}
              } @else {
                <nz-icon nzType="swap-right" nzTheme="outline" />
              }
            </ng-container>
          </span>
        </div>
        <div class="{{ prefixCls }}-input">
          <ng-container
            *ngTemplateOutlet="tplRangeInput; context: { partType: 'right' }"
          />
        </div>
        <ng-container *ngTemplateOutlet="tplRightRest" />
      }
    } @else {
      <ng-template [ngTemplateOutlet]="inlineMode" />
    }
    <!-- Input for Range ONLY -->
    <ng-template #tplRangeInput let-partType="partType">
      <input
        #rangePickerInput
        [attr.id]="nzId"
        [disabled]="nzDisabled"
        [readOnly]="nzInputReadOnly"
        [size]="inputSize"
        autocomplete="off"
        (click)="onClickInputBox($event)"
        (focusout)="onFocusout($event)"
        (focus)="onFocus($event, partType)"
        (keyup.enter)="onKeyupEnter($event)"
        [(ngModel)]="inputValue[datePickerService.getActiveIndex(partType)]"
        (ngModelChange)="onInputChange($event)"
        placeholder="{{ getPlaceholder(partType) }}"
      />
    </ng-template>

    <!-- Right operator icons -->
    <ng-template #tplRightRest>
      <div class="{{ prefixCls }}-active-bar" [style]="activeBarStyle"></div>
      @if (showClear) {
        <span class="{{ prefixCls }}-clear" (click)="onClickClear($event)">
          <nz-icon nzType="close-circle" nzTheme="fill" />
        </span>
      }

      <span class="{{ prefixCls }}-suffix">
        <ng-container *nzStringTemplateOutlet="nzSuffixIcon; let suffixIcon">
          <nz-icon [nzType]="suffixIcon" />
        </ng-container>
        @if (hasFeedback && !!status) {
          <nz-form-item-feedback-icon [status]="status" />
        }
      </span>
    </ng-template>

    <ng-template #inlineMode>
      <div
        [class]="dropdownClass()"
        [class.ant-picker-dropdown-range]="!nzInline() && isRange"
        [class.ant-picker-active-left]="
          datePickerService.activeInput === 'left'
        "
        [class.ant-picker-active-right]="
          datePickerService.activeInput === 'right'
        "
        [style]="nzPopupStyle"
        [animate.enter]="$any(!nzInline() && datepickerAnimationEnter())"
        [animate.leave]="$any(!nzInline() && datepickerAnimationLeave())"
      >
        <date-range-popup
          [isRange]="isRange"
          [inline]="nzInline()"
          [defaultPickerValue]="nzDefaultPickerValue"
          [showWeek]="nzShowWeekNumber || nzMode === 'week'"
          [panelMode]="panelMode"
          (panelModeChange)="onPanelModeChange($event)"
          (calendarChange)="onCalendarChange($event)"
          [locale]="nzLocale?.lang!"
          [showToday]="
            nzMode === 'date' && nzShowToday && !isRange && !nzShowTime
          "
          [showNow]="nzMode === 'date' && nzShowNow && !isRange && !!nzShowTime"
          [showTime]="nzShowTime"
          [dateRender]="nzDateRender"
          [disabledDate]="nzDisabledDate"
          [disabledTime]="nzDisabledTime"
          [extraFooter]="extraFooter"
          [ranges]="nzRanges"
          [dir]="dir()"
          [format]="nzFormat"
          (resultOk)="onResultOk()"
        />
      </div>
    </ng-template>

    <!-- Overlay -->
    <ng-template
      cdkConnectedOverlay
      nzConnectedOverlay
      cdkConnectedOverlayTransformOriginOn=".{{ prefixCls }}-dropdown"
      [cdkConnectedOverlayHasBackdrop]="nzBackdrop"
      [cdkConnectedOverlayOrigin]="origin"
      [cdkConnectedOverlayOpen]="realOpenState"
      [cdkConnectedOverlayPositions]="overlayPositions"
      (positionChange)="onPositionChange($event)"
      (detach)="close()"
      (overlayKeydown)="onOverlayKeydown($event)"
    >
      <ng-container *ngTemplateOutlet="inlineMode" />
    </ng-template>
  `,
  host: {
    "[class.ant-picker]": `true`,
    "[class.ant-picker-range]": `isRange`,
    "[class.ant-picker-large]": `finalSize() === 'large'`,
    "[class.ant-picker-small]": `finalSize() === 'small'`,
    "[class.ant-picker-disabled]": `nzDisabled`,
    "[class.ant-picker-rtl]": `dir() === 'rtl'`,
    "[class.ant-picker-outlined]": `finalVariant() === 'outlined'`,
    "[class.ant-picker-borderless]": `finalVariant() === 'borderless'`,
    "[class.ant-picker-filled]": `finalVariant() === 'filled'`,
    "[class.ant-picker-underlined]": `finalVariant() === 'underlined'`,
    "[class.ant-picker-inline]": `nzInline()`,
    "(click)": "onClickInputBox($event)",
  },
  hostDirectives: [NzSpaceCompactItemDirective],
  providers: [
    DatePickerService,
    { provide: NZ_SPACE_COMPACT_ITEM_TYPE, useValue: "picker" },
    {
      provide: NG_VALUE_ACCESSOR,
      multi: true,
      useExisting: forwardRef(() => NzDatePickerComponent),
    },
  ],
  imports: [
    FormsModule,
    NgTemplateOutlet,
    NzOutletModule,
    NzIconModule,
    NzFormItemFeedbackIconComponent,
    DateRangePopupComponent,
    CdkConnectedOverlay,
    NzOverlayModule,
  ],
})
export class NzDatePickerComponent
  implements OnInit, OnChanges, AfterViewInit, ControlValueAccessor
{
  public nzConfigService = inject(NzConfigService);
  public datePickerService = inject(DatePickerService);
  protected i18n = inject(NzI18nService);
  protected cdr = inject(ChangeDetectorRef);
  private renderer = inject(Renderer2);
  private elementRef = inject(ElementRef<HTMLElement>);
  private dateHelper = inject(DateHelperService);
  private nzResizeObserver = inject(NzResizeObserver);
  private platform = inject(Platform);
  private destroyRef = inject(DestroyRef);
  protected readonly dir = inject(Directionality).valueSignal;

  readonly _nzModuleName: NzConfigKey = NZ_CONFIG_MODULE_NAME;

  static ngAcceptInputType_nzShowTime:
    BooleanInput | SupportTimeOptions | null | undefined;
  static ngAcceptInputType_nzMode: NzDateMode | string;

  isRange = !!inject(NzRangePickerComponent, { self: true, optional: true }); // Indicate whether the value is a range value
  extraFooter?: TemplateRef<NzSafeAny> | string;

  // status
  statusCls: NgClassInterface = {};
  status: NzValidateStatus = "";
  hasFeedback: boolean = false;

  public panelMode: NzDateMode | NzDateMode[] = "date";
  private isCustomPlaceHolder: boolean = false;
  private isCustomFormat: boolean = false;
  private showTime: SupportTimeOptions | boolean = false;
  private isNzDisableFirstChange: boolean = true;
  // --- Common API
  readonly nzInline = input(false, { transform: booleanAttribute });
  @Input({ transform: booleanAttribute }) nzAllowClear: boolean = true;
  @Input({ transform: booleanAttribute }) nzAutoFocus: boolean = false;
  @Input({ transform: booleanAttribute }) nzDisabled: boolean = false;
  @Input({ transform: booleanAttribute }) nzInputReadOnly: boolean = false;
  @Input({ transform: booleanAttribute }) nzOpen?: boolean;
  @Input() nzDisabledDate?: (d: Date) => boolean;
  @Input() nzLocale!: NzDatePickerI18nInterface;
  @Input() nzPlaceHolder: string | string[] = "";
  @Input() nzPopupStyle: object = POPUP_STYLE_PATCH;
  readonly nzDropdownClassName = input<string>();
  @Input() nzSize: NzDatePickerSizeType = "default";
  @Input() nzStatus: NzStatus = "";
  @Input() nzFormat!: string;
  @Input() @WithConfig() nzVariant: NzVariant | undefined = undefined;
  @Input() nzDateRender?:
    TemplateRef<NzSafeAny> | string | FunctionProp<TemplateRef<Date> | string>;
  @Input() nzDisabledTime?: DisabledTimeFn;
  @Input() nzRenderExtraFooter?:
    | TemplateRef<NzSafeAny>
    | string
    | FunctionProp<TemplateRef<NzSafeAny> | string>;
  @Input({ transform: booleanAttribute }) nzShowToday: boolean = true;
  @Input() nzMode: NzDateMode = "date";
  @Input({ transform: booleanAttribute }) nzShowNow: boolean = true;
  @Input() nzRanges?: PresetRanges;
  @Input() nzDefaultPickerValue: CompatibleDate | null = null;
  @Input() @WithConfig() nzSeparator?: string | TemplateRef<NzSafeAny> =
    undefined;
  @Input() @WithConfig() nzSuffixIcon: string | TemplateRef<NzSafeAny> =
    "calendar";
  @Input() @WithConfig() nzBackdrop = false;
  @Input() nzId: string | null = null;
  @Input() nzPlacement: NzPlacement = "bottomLeft";
  @Input({ transform: booleanAttribute }) nzShowWeekNumber: boolean = false;

  @Output() readonly nzOnPanelChange = new EventEmitter<NzPanelChangeType>();
  @Output() readonly nzOnCalendarChange = new EventEmitter<
    Array<Date | null>
  >();
  @Output() readonly nzOnOk = new EventEmitter<CompatibleDate | null>();
  @Output() readonly nzOnOpenChange = new EventEmitter<boolean>();

  @Input() get nzShowTime(): SupportTimeOptions | boolean {
    return this.showTime;
  }

  set nzShowTime(value: SupportTimeOptions | boolean) {
    this.showTime = typeof value === "object" ? value : toBoolean(value);
  }

  // ------------------------------------------------------------------------
  // Input API Start
  // ------------------------------------------------------------------------
  @ViewChild(CdkConnectedOverlay, { static: false })
  cdkConnectedOverlay?: CdkConnectedOverlay;
  @ViewChild(DateRangePopupComponent, { static: false })
  panel!: DateRangePopupComponent;
  @ViewChild("separatorElement", { static: false })
  separatorElement?: ElementRef;
  @ViewChild("pickerInput", { static: false })
  pickerInput?: ElementRef<HTMLInputElement>;
  @ViewChildren("rangePickerInput") rangePickerInputs?: QueryList<
    ElementRef<HTMLInputElement>
  >;

  get origin(): ElementRef {
    return this.elementRef;
  }

  inputSize: number = 12;
  inputWidth?: number;
  prefixCls = PREFIX_CLASS;
  inputValue!: NzSafeAny;
  activeBarStyle: object = {};
  overlayOpen: boolean = false; // Available when "nzOpen" = undefined
  overlayPositions: ConnectionPositionPair[] = [
    ...DEFAULT_DATE_PICKER_POSITIONS,
  ];

  private readonly currentPosition = signal<OriginConnectionPosition>({
    originX: "start",
    originY: "bottom",
  });

  protected readonly datepickerAnimationEnter = slideAnimationEnter();
  protected readonly datepickerAnimationLeave = slideAnimationLeave();

  protected readonly dropdownClass = computed(() => {
    const cls: string[] = [];
    if (!this.nzInline()) {
      cls.push(this.generateClass("dropdown"));

      const customCls = this.nzDropdownClassName();
      if (customCls) {
        cls.push(customCls);
      }

      const { originX, originY } = this.currentPosition();

      if (originX === "start" && originY === "bottom") {
        cls.push(this.generateClass("dropdown-placement-bottomLeft"));
      } else if (originX === "start" && originY === "top") {
        cls.push(this.generateClass("dropdown-placement-topLeft"));
      } else if (originX === "end" && originY === "bottom") {
        cls.push(this.generateClass("dropdown-placement-bottomRight"));
      } else if (originX === "end" && originY === "top") {
        cls.push(this.generateClass("dropdown-placement-topRight"));
      }

      if (this.dir() === "rtl") {
        cls.push(this.generateClass("dropdown-rtl"));
      }
    }
    return cls;
  });

  get realOpenState(): boolean {
    // The value that really decide the open state of overlay
    return this.isOpenHandledByUser() ? !!this.nzOpen : this.overlayOpen;
  }

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

  private size = signal<NzSizeLDSType>(this.nzSize);
  private variant = signal<NzVariant | undefined>(this.nzVariant);

  private readonly formSize = inject(NZ_FORM_SIZE, { optional: true });
  private readonly formVariant = inject(NZ_FORM_VARIANT, { optional: true });

  private compactSize = inject(NZ_SPACE_COMPACT_SIZE, { optional: true });
  private document: Document = inject(DOCUMENT);

  constructor() {
    this.datePickerService.isRange = this.isRange;
    this.datePickerService.initValue(true);
  }

  ngAfterViewInit(): void {
    if (this.nzAutoFocus) {
      this.focus();
    }

    if (this.isRange && this.platform.isBrowser) {
      this.nzResizeObserver
        .observe(this.elementRef)
        .pipe(takeUntilDestroyed(this.destroyRef))
        .subscribe(() => {
          this.updateInputWidthAndArrowLeft();
        });
    }

    this.datePickerService.inputPartChange$
      .pipe(takeUntilDestroyed(this.destroyRef))
      .subscribe((partType) => {
        if (partType) {
          this.datePickerService.activeInput = partType;
        }
        this.focus();
        this.updateInputWidthAndArrowLeft();
      });

    if (this.platform.isBrowser) {
      // prevent mousedown event to trigger focusout event when click in date picker
      // see: https://github.com/NG-ZORRO/ng-zorro-antd/issues/7450
      fromEventOutsideAngular(this.elementRef.nativeElement, "mousedown")
        .pipe(takeUntilDestroyed(this.destroyRef))
        .subscribe((event) => {
          if (
            (event.target as HTMLInputElement).tagName.toLowerCase() !== "input"
          ) {
            event.preventDefault();
          }
        });
    }
  }

  updateInputWidthAndArrowLeft(): void {
    this.inputWidth =
      this.rangePickerInputs?.first?.nativeElement.offsetWidth || 0;

    const baseStyle = { position: "absolute", width: `${this.inputWidth}px` };
    this.datePickerService.arrowLeft =
      this.datePickerService.activeInput === "left"
        ? 0
        : this.inputWidth + this.separatorElement?.nativeElement.offsetWidth ||
          0;

    if (this.dir() === "rtl") {
      this.activeBarStyle = {
        ...baseStyle,
        right: `${this.datePickerService.arrowLeft}px`,
      };
    } else {
      this.activeBarStyle = {
        ...baseStyle,
        left: `${this.datePickerService.arrowLeft}px`,
      };
    }

    this.cdr.markForCheck();
  }

  getInput(partType?: RangePartType): HTMLInputElement | undefined {
    if (this.nzInline()) {
      return undefined;
    }
    return this.isRange
      ? partType === "left"
        ? this.rangePickerInputs?.first.nativeElement
        : this.rangePickerInputs?.last.nativeElement
      : this.pickerInput!.nativeElement;
  }

  focus(): void {
    const activeInputElement = this.getInput(
      this.datePickerService.activeInput,
    );
    if (this.document.activeElement !== activeInputElement) {
      activeInputElement?.focus();
    }
  }

  onFocus(event: FocusEvent, partType?: RangePartType): void {
    event.preventDefault();
    if (partType) {
      this.datePickerService.inputPartChange$.next(partType);
    }
    this.renderClass(true);
  }

  // blur event has not the relatedTarget in IE11, use focusout instead.
  onFocusout(event: FocusEvent): void {
    event.preventDefault();
    this.onTouchedFn();
    if (!this.elementRef.nativeElement.contains(event.relatedTarget as Node)) {
      this.checkAndClose();
    }
    this.renderClass(false);
  }

  // Show overlay content
  open(): void {
    if (this.nzInline()) {
      return;
    }
    if (!this.realOpenState && !this.nzDisabled) {
      this.updateInputWidthAndArrowLeft();
      this.overlayOpen = true;
      this.nzOnOpenChange.emit(true);
      this.focus();
      this.cdr.markForCheck();
    }
  }

  close(): void {
    if (this.nzInline()) {
      return;
    }
    if (this.realOpenState) {
      this.overlayOpen = false;
      this.nzOnOpenChange.emit(false);
    }
  }

  get showClear(): boolean {
    return (
      !this.nzDisabled &&
      !this.isEmptyValue(this.datePickerService.value) &&
      this.nzAllowClear
    );
  }

  checkAndClose(): void {
    if (!this.realOpenState) {
      return;
    }

    if (this.panel.isAllowed(this.datePickerService.value!, true)) {
      if (
        Array.isArray(this.datePickerService.value) &&
        wrongSortOrder(this.datePickerService.value)
      ) {
        const index = this.datePickerService.getActiveIndex();
        const value = this.datePickerService.value[index];
        this.panel.changeValueFromSelect(value!, true);
        return;
      }
      this.updateInputValue();
      this.datePickerService.emitValue$.next();
    } else {
      this.datePickerService.setValue(this.datePickerService.initialValue!);
      this.close();
    }
  }

  onClickInputBox(event: MouseEvent): void {
    event.stopPropagation();
    this.focus();
    if (!this.isOpenHandledByUser()) {
      this.open();
    }
  }

  onOverlayKeydown(event: KeyboardEvent): void {
    if (event.keyCode === ESCAPE) {
      this.datePickerService.initValue();
    }
  }

  onPositionChange(position: ConnectedOverlayPositionChange): void {
    this.currentPosition.set(position.connectionPair);
  }

  onClickClear(event: MouseEvent): void {
    event.preventDefault();
    event.stopPropagation();

    this.datePickerService.initValue(true);
    this.datePickerService.emitValue$.next();
  }

  updateInputValue(): void {
    const newValue = this.datePickerService.value;
    if (this.isRange) {
      this.inputValue = newValue
        ? (newValue as CandyDate[]).map((v) => this.formatValue(v))
        : ["", ""];
    } else {
      this.inputValue = this.formatValue(newValue as CandyDate);
    }
    this.cdr.markForCheck();
  }

  formatValue(value: CandyDate): string {
    return this.dateHelper.format(
      value && (value as CandyDate).nativeDate,
      this.nzFormat,
    );
  }

  onInputChange(value: string, isEnter: boolean = false): void {
    /**
     * in IE11 focus/blur will trigger ngModelChange if placeholder changes,
     * so we forbid IE11 to open panel through input change
     */
    if (
      !this.platform.TRIDENT &&
      this.document.activeElement ===
        this.getInput(this.datePickerService.activeInput) &&
      !this.realOpenState
    ) {
      this.open();
      return;
    }

    const date = this.checkValidDate(value);
    // Can only change date when it's open
    if (date && this.realOpenState) {
      this.panel.changeValueFromSelect(date, isEnter);
    }
  }

  onKeyupEnter(event: Event): void {
    this.onInputChange((event.target as HTMLInputElement).value, true);
  }

  private checkValidDate(value: string): CandyDate | null {
    const date = new CandyDate(this.dateHelper.parseDate(value, this.nzFormat));

    if (
      !date.isValid() ||
      value !== this.dateHelper.format(date.nativeDate, this.nzFormat)
    ) {
      return null;
    }

    return date;
  }

  getPlaceholder(partType?: RangePartType): string {
    return this.isRange
      ? this.nzPlaceHolder[this.datePickerService.getActiveIndex(partType!)]
      : (this.nzPlaceHolder as string);
  }

  isEmptyValue(value: CompatibleValue): boolean {
    if (value === null) {
      return true;
    } else if (this.isRange) {
      return !value || !Array.isArray(value) || value.every((val) => !val);
    } else {
      return !value;
    }
  }

  // Whether open state is permanently controlled by user himself
  isOpenHandledByUser(): boolean {
    return this.nzOpen !== undefined;
  }

  private nzFormStatusService = inject(NzFormStatusService, { optional: true });
  private nzFormNoStatusService = inject(NzFormNoStatusService, {
    optional: true,
  });

  // ------------------------------------------------------------------------
  // Input API End
  // ------------------------------------------------------------------------

  ngOnInit(): void {
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
            : of(false),
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

    // Subscribe the every locale change if the nzLocale is not handled by user
    if (!this.nzLocale) {
      this.i18n.localeChange
        .pipe(takeUntilDestroyed(this.destroyRef))
        .subscribe(() => this.setLocale());
    }

    this.datePickerService.emitValue$
      .pipe(takeUntilDestroyed(this.destroyRef))
      .subscribe(() => {
        const granularityComparison = this.showTime ? "second" : "day";
        const value = this.datePickerService.value;
        const datePickerPreviousValue = this.datePickerService.initialValue;

        // Check if the value has change for a simple datepicker, let us avoid notify the control for nothing
        if (
          !this.isRange &&
          (value as CandyDate)?.isSame(
            (datePickerPreviousValue as CandyDate)?.nativeDate,
            granularityComparison,
          )
        ) {
          this.onTouchedFn();
          return this.close();
        }

        // check if the value has change for a range picker, let us avoid notify the control for nothing
        if (this.isRange) {
          const [previousStartDate, previousEndDate] =
            datePickerPreviousValue as CandyDate[];
          const [currentStartDate, currentEndDate] = value as CandyDate[];
          if (
            previousStartDate?.isSame(
              currentStartDate?.nativeDate,
              granularityComparison,
            ) &&
            previousEndDate?.isSame(
              currentEndDate?.nativeDate,
              granularityComparison,
            )
          ) {
            this.onTouchedFn();
            return this.close();
          }
        }

        this.datePickerService.initialValue = cloneDate(value);
        if (this.isRange) {
          const vAsRange = value as CandyDate[];
          if (vAsRange.length) {
            this.onChangeFn([
              vAsRange[0]?.nativeDate ?? null,
              vAsRange[1]?.nativeDate ?? null,
            ]);
          } else {
            this.onChangeFn([]);
          }
        } else {
          if (value) {
            this.onChangeFn((value as CandyDate).nativeDate);
          } else {
            this.onChangeFn(null);
          }
        }
        this.onTouchedFn();
        // When value emitted, overlay will be closed
        this.close();
      });

    this.inputValue = this.isRange ? ["", ""] : "";
    this.setModeAndFormat();

    this.datePickerService.valueChange$
      .pipe(takeUntilDestroyed(this.destroyRef))
      .subscribe(() => {
        this.updateInputValue();
      });
  }

  ngOnChanges({
    nzStatus,
    nzPlacement,
    nzPopupStyle,
    nzPlaceHolder,
    nzLocale,
    nzFormat,
    nzRenderExtraFooter,
    nzMode,
    nzSize,
    nzVariant,
  }: SimpleChanges): void {
    if (nzPopupStyle) {
      // Always assign the popup style patch
      this.nzPopupStyle = this.nzPopupStyle
        ? { ...this.nzPopupStyle, ...POPUP_STYLE_PATCH }
        : POPUP_STYLE_PATCH;
    }

    // Mark as customized placeholder by user once nzPlaceHolder assigned at the first time
    if (nzPlaceHolder?.currentValue) {
      this.isCustomPlaceHolder = true;
    }

    if (nzFormat?.currentValue) {
      this.isCustomFormat = true;
      this.updateInputValue();
    }

    if (nzLocale) {
      // The nzLocale is currently handled by user
      this.setDefaultPlaceHolder();
    }

    if (nzRenderExtraFooter) {
      this.extraFooter = valueFunctionProp(this.nzRenderExtraFooter!);
    }

    if (nzMode) {
      this.setDefaultPlaceHolder();
      this.setModeAndFormat();
    }

    if (nzStatus) {
      this.setStatusStyles(this.nzStatus, this.hasFeedback);
    }

    if (nzPlacement) {
      this.setPlacement(this.nzPlacement);
    }

    if (nzSize) {
      this.size.set(nzSize.currentValue);
    }

    if (nzVariant) {
      this.variant.set(nzVariant.currentValue);
    }
  }

  setModeAndFormat(): void {
    const inputFormats: Partial<Record<NzDateMode, string>> = {
      year: "yyyy",
      quarter: "yyyy-[Q]Q",
      month: "yyyy-MM",
      week: "YYYY-ww",
      date: this.nzShowTime ? "yyyy-MM-dd HH:mm:ss" : "yyyy-MM-dd",
    };

    if (!this.nzMode) {
      this.nzMode = "date";
    }

    this.panelMode = this.isRange ? [this.nzMode, this.nzMode] : this.nzMode;

    // Default format when it's empty
    if (!this.isCustomFormat) {
      this.nzFormat = inputFormats[this.nzMode as NzDateMode]!;
    }

    this.inputSize = Math.max(10, this.nzFormat.length) + 2;
    this.updateInputValue();
  }

  /**
   * Triggered when overlayOpen changes (different with realOpenState)
   *
   * @param open The overlayOpen in picker component
   */
  onOpenChange(open: boolean): void {
    this.nzOnOpenChange.emit(open);
  }

  // ------------------------------------------------------------------------
  // | Control value accessor implements
  // ------------------------------------------------------------------------

  // NOTE: onChangeFn/onTouchedFn will not be assigned if user not use as ngModel
  onChangeFn: OnChangeType = () => void 0;
  onTouchedFn: OnTouchedType = () => void 0;

  writeValue(value: CompatibleDate): void {
    this.setValue(value);
    this.cdr.markForCheck();
  }

  registerOnChange(fn: OnChangeType): void {
    this.onChangeFn = fn;
  }

  registerOnTouched(fn: OnTouchedType): void {
    this.onTouchedFn = fn;
  }

  setDisabledState(isDisabled: boolean): void {
    this.nzDisabled =
      (this.isNzDisableFirstChange && this.nzDisabled) || isDisabled;
    this.cdr.markForCheck();
    this.isNzDisableFirstChange = false;
  }

  // ------------------------------------------------------------------------
  // | Internal methods
  // ------------------------------------------------------------------------

  // Reload locale from i18n with side effects
  private setLocale(): void {
    this.nzLocale = this.i18n.getLocaleData("DatePicker", {});
    this.setDefaultPlaceHolder();
    this.cdr.markForCheck();
  }

  private setDefaultPlaceHolder(): void {
    if (!this.isCustomPlaceHolder && this.nzLocale) {
      const defaultPlaceholder: Partial<Record<NzDateMode, string>> = {
        year: this.getPropertyOfLocale("yearPlaceholder"),
        quarter: this.getPropertyOfLocale("quarterPlaceholder"),
        month: this.getPropertyOfLocale("monthPlaceholder"),
        week: this.getPropertyOfLocale("weekPlaceholder"),
        date: this.getPropertyOfLocale("placeholder"),
      };

      const defaultRangePlaceholder: Partial<Record<NzDateMode, string[]>> = {
        year: this.getPropertyOfLocale("rangeYearPlaceholder"),
        quarter: this.getPropertyOfLocale("rangeQuarterPlaceholder"),
        month: this.getPropertyOfLocale("rangeMonthPlaceholder"),
        week: this.getPropertyOfLocale("rangeWeekPlaceholder"),
        date: this.getPropertyOfLocale("rangePlaceholder"),
      };

      this.nzPlaceHolder = this.isRange
        ? defaultRangePlaceholder[this.nzMode as NzDateMode]!
        : defaultPlaceholder[this.nzMode as NzDateMode]!;
    }
  }

  private getPropertyOfLocale<T extends keyof NzDatePickerLangI18nInterface>(
    type: T,
  ): NzDatePickerLangI18nInterface[T] {
    return (
      this.nzLocale.lang[type] ||
      this.i18n.getLocaleData(`DatePicker.lang.${type}`)
    );
  }

  // Safe way of setting value with default
  private setValue(value: CompatibleDate): void {
    const newValue = this.datePickerService.makeValue(value);
    this.datePickerService.setValue(newValue);
    this.datePickerService.initialValue = cloneDate(newValue);
    this.cdr.detectChanges();
  }

  renderClass(value: boolean): void {
    // TODO: avoid autoFocus cause change after checked error
    if (value) {
      this.renderer.addClass(
        this.elementRef.nativeElement,
        "ant-picker-focused",
      );
    } else {
      this.renderer.removeClass(
        this.elementRef.nativeElement,
        "ant-picker-focused",
      );
    }
  }

  onPanelModeChange(panelChange: NzPanelChangeType): void {
    this.nzOnPanelChange.emit(panelChange);
  }

  // Emit nzOnCalendarChange when select date by nz-range-picker
  onCalendarChange(value: CompatibleValue): void {
    if (this.isRange && Array.isArray(value)) {
      const rangeValue = value
        .filter((x) => x instanceof CandyDate)
        .map((x) => x!.nativeDate);
      this.nzOnCalendarChange.emit(rangeValue);
    }
  }

  onResultOk(): void {
    if (this.isRange) {
      const value = this.datePickerService.value as CandyDate[];
      if (value.length) {
        this.nzOnOk.emit([
          value[0]?.nativeDate || null,
          value[1]?.nativeDate || null,
        ]);
      } else {
        this.nzOnOk.emit([]);
      }
    } else {
      if (this.datePickerService.value) {
        this.nzOnOk.emit(
          (this.datePickerService.value as CandyDate).nativeDate,
        );
      } else {
        this.nzOnOk.emit(null);
      }
    }
  }

  // status
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

  private setPlacement(placement: NzPlacement): void {
    const position: ConnectionPositionPair =
      DATE_PICKER_POSITION_MAP[placement];
    this.overlayPositions = [position, ...DEFAULT_DATE_PICKER_POSITIONS];
    this.currentPosition.set(position);
  }

  private generateClass(suffix: string): string {
    return generateClassName(this.prefixCls, suffix);
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
@@ -339,12 +339,16 @@
   @Input() nzStatus: NzStatus = "";
   @Input() nzFormat!: string;
   @Input() @WithConfig() nzVariant: NzVariant | undefined = undefined;
   @Input() nzDateRender?:
-    TemplateRef<NzSafeAny> | string | FunctionProp<TemplateRef<Date> | string>;
+    | TemplateRef<NzSafeAny>
+    | string
+    | FunctionProp<TemplateRef<Date> | string>;
   @Input() nzDisabledTime?: DisabledTimeFn;
   @Input() nzRenderExtraFooter?:
-    TemplateRef<NzSafeAny> | string | FunctionProp<TemplateRef<NzSafeAny> | string>;
+    | TemplateRef<NzSafeAny>
+    | string
+    | FunctionProp<TemplateRef<NzSafeAny> | string>;
   @Input({ transform: booleanAttribute }) nzShowToday: boolean = true;
   @Input() nzMode: NzDateMode = "date";
   @Input({ transform: booleanAttribute }) nzShowNow: boolean = true;
   @Input() nzRanges?: PresetRanges;

`````

### Actual (oxfmt)

`````ts
/**
 * Use of this source code is governed by an MIT-style license that can be
 * found in the LICENSE file at https://github.com/NG-ZORRO/ng-zorro-antd/blob/master/LICENSE
 */

import { Directionality } from "@angular/cdk/bidi";
import { ESCAPE } from "@angular/cdk/keycodes";
import {
  CdkConnectedOverlay,
  ConnectedOverlayPositionChange,
  ConnectionPositionPair,
  OriginConnectionPosition,
} from "@angular/cdk/overlay";
import { Platform } from "@angular/cdk/platform";
import { NgTemplateOutlet } from "@angular/common";
import {
  AfterViewInit,
  booleanAttribute,
  ChangeDetectionStrategy,
  ChangeDetectorRef,
  Component,
  computed,
  DestroyRef,
  DOCUMENT,
  ElementRef,
  EventEmitter,
  forwardRef,
  inject,
  input,
  Input,
  OnChanges,
  OnInit,
  Output,
  QueryList,
  Renderer2,
  signal,
  SimpleChanges,
  TemplateRef,
  ViewChild,
  ViewChildren,
  ViewEncapsulation,
} from "@angular/core";
import { takeUntilDestroyed } from "@angular/core/rxjs-interop";
import { ControlValueAccessor, FormsModule, NG_VALUE_ACCESSOR } from "@angular/forms";
import { of } from "rxjs";
import { distinctUntilChanged, map, withLatestFrom } from "rxjs/operators";

import { NzResizeObserver } from "ng-zorro-antd/cdk/resize-observer";
import { slideAnimationEnter, slideAnimationLeave } from "ng-zorro-antd/core/animation";
import { NzConfigKey, NzConfigService, WithConfig } from "ng-zorro-antd/core/config";
import {
  NZ_FORM_SIZE,
  NZ_FORM_VARIANT,
  NzFormItemFeedbackIconComponent,
  NzFormNoStatusService,
  NzFormStatusService,
} from "ng-zorro-antd/core/form";
import { NzOutletModule } from "ng-zorro-antd/core/outlet";
import {
  DATE_PICKER_POSITION_MAP,
  DEFAULT_DATE_PICKER_POSITIONS,
  NzOverlayModule,
} from "ng-zorro-antd/core/overlay";
import { CandyDate, cloneDate, CompatibleValue, wrongSortOrder } from "ng-zorro-antd/core/time";
import {
  BooleanInput,
  FunctionProp,
  NgClassInterface,
  NzSafeAny,
  NzSizeLDSType,
  NzStatus,
  NzValidateStatus,
  NzVariant,
  OnChangeType,
  OnTouchedType,
  type NzPlacement,
} from "ng-zorro-antd/core/types";
import {
  fromEventOutsideAngular,
  generateClassName,
  getStatusClassNames,
  toBoolean,
  valueFunctionProp,
} from "ng-zorro-antd/core/util";
import {
  DateHelperService,
  NzDatePickerI18nInterface,
  NzDatePickerLangI18nInterface,
  NzI18nService,
} from "ng-zorro-antd/i18n";
import { NzIconModule } from "ng-zorro-antd/icon";
import {
  NZ_SPACE_COMPACT_ITEM_TYPE,
  NZ_SPACE_COMPACT_SIZE,
  NzSpaceCompactItemDirective,
} from "ng-zorro-antd/space";

import { DatePickerService } from "./date-picker.service";
import { DateRangePopupComponent } from "./date-range-popup.component";
import { NzRangePickerComponent } from "./range-picker.component";
import {
  CompatibleDate,
  DisabledTimeFn,
  NzDateMode,
  NzPanelChangeType,
  PresetRanges,
  RangePartType,
  SupportTimeOptions,
} from "./standard-types";
import { PREFIX_CLASS } from "./util";

const POPUP_STYLE_PATCH = { position: "relative" }; // Aim to override antd's style to support overlay's position strategy (position:absolute will cause it not working because the overlay can't get the height/width of it's content)
const NZ_CONFIG_MODULE_NAME: NzConfigKey = "datePicker";

export type NzDatePickerSizeType = "large" | "default" | "small";

/**
 * The base picker for all common APIs
 */
@Component({
  encapsulation: ViewEncapsulation.None,
  changeDetection: ChangeDetectionStrategy.OnPush,
  selector:
    "nz-date-picker,nz-week-picker,nz-month-picker,nz-quarter-picker,nz-year-picker,nz-range-picker",
  exportAs: "nzDatePicker",
  template: `
    @if (!nzInline()) {
      @if (!isRange) {
        <div class="{{ prefixCls }}-input">
          <input
            #pickerInput
            [attr.id]="nzId"
            [class.ant-input-disabled]="nzDisabled"
            [disabled]="nzDisabled"
            [readOnly]="nzInputReadOnly"
            [(ngModel)]="inputValue"
            placeholder="{{ getPlaceholder() }}"
            [size]="inputSize"
            autocomplete="off"
            (focus)="onFocus($event)"
            (focusout)="onFocusout($event)"
            (ngModelChange)="onInputChange($event)"
            (keyup.enter)="onKeyupEnter($event)"
          />
          <ng-container *ngTemplateOutlet="tplRightRest" />
        </div>
      } @else {
        <div class="{{ prefixCls }}-input">
          <ng-container *ngTemplateOutlet="tplRangeInput; context: { partType: 'left' }" />
        </div>
        <div #separatorElement class="{{ prefixCls }}-range-separator">
          <span class="{{ prefixCls }}-separator">
            <ng-container *nzStringTemplateOutlet="nzSeparator; let separator">
              @if (nzSeparator) {
                {{ nzSeparator }}
              } @else {
                <nz-icon nzType="swap-right" nzTheme="outline" />
              }
            </ng-container>
          </span>
        </div>
        <div class="{{ prefixCls }}-input">
          <ng-container *ngTemplateOutlet="tplRangeInput; context: { partType: 'right' }" />
        </div>
        <ng-container *ngTemplateOutlet="tplRightRest" />
      }
    } @else {
      <ng-template [ngTemplateOutlet]="inlineMode" />
    }
    <!-- Input for Range ONLY -->
    <ng-template #tplRangeInput let-partType="partType">
      <input
        #rangePickerInput
        [attr.id]="nzId"
        [disabled]="nzDisabled"
        [readOnly]="nzInputReadOnly"
        [size]="inputSize"
        autocomplete="off"
        (click)="onClickInputBox($event)"
        (focusout)="onFocusout($event)"
        (focus)="onFocus($event, partType)"
        (keyup.enter)="onKeyupEnter($event)"
        [(ngModel)]="inputValue[datePickerService.getActiveIndex(partType)]"
        (ngModelChange)="onInputChange($event)"
        placeholder="{{ getPlaceholder(partType) }}"
      />
    </ng-template>

    <!-- Right operator icons -->
    <ng-template #tplRightRest>
      <div class="{{ prefixCls }}-active-bar" [style]="activeBarStyle"></div>
      @if (showClear) {
        <span class="{{ prefixCls }}-clear" (click)="onClickClear($event)">
          <nz-icon nzType="close-circle" nzTheme="fill" />
        </span>
      }

      <span class="{{ prefixCls }}-suffix">
        <ng-container *nzStringTemplateOutlet="nzSuffixIcon; let suffixIcon">
          <nz-icon [nzType]="suffixIcon" />
        </ng-container>
        @if (hasFeedback && !!status) {
          <nz-form-item-feedback-icon [status]="status" />
        }
      </span>
    </ng-template>

    <ng-template #inlineMode>
      <div
        [class]="dropdownClass()"
        [class.ant-picker-dropdown-range]="!nzInline() && isRange"
        [class.ant-picker-active-left]="datePickerService.activeInput === 'left'"
        [class.ant-picker-active-right]="datePickerService.activeInput === 'right'"
        [style]="nzPopupStyle"
        [animate.enter]="$any(!nzInline() && datepickerAnimationEnter())"
        [animate.leave]="$any(!nzInline() && datepickerAnimationLeave())"
      >
        <date-range-popup
          [isRange]="isRange"
          [inline]="nzInline()"
          [defaultPickerValue]="nzDefaultPickerValue"
          [showWeek]="nzShowWeekNumber || nzMode === 'week'"
          [panelMode]="panelMode"
          (panelModeChange)="onPanelModeChange($event)"
          (calendarChange)="onCalendarChange($event)"
          [locale]="nzLocale?.lang!"
          [showToday]="nzMode === 'date' && nzShowToday && !isRange && !nzShowTime"
          [showNow]="nzMode === 'date' && nzShowNow && !isRange && !!nzShowTime"
          [showTime]="nzShowTime"
          [dateRender]="nzDateRender"
          [disabledDate]="nzDisabledDate"
          [disabledTime]="nzDisabledTime"
          [extraFooter]="extraFooter"
          [ranges]="nzRanges"
          [dir]="dir()"
          [format]="nzFormat"
          (resultOk)="onResultOk()"
        />
      </div>
    </ng-template>

    <!-- Overlay -->
    <ng-template
      cdkConnectedOverlay
      nzConnectedOverlay
      cdkConnectedOverlayTransformOriginOn=".{{ prefixCls }}-dropdown"
      [cdkConnectedOverlayHasBackdrop]="nzBackdrop"
      [cdkConnectedOverlayOrigin]="origin"
      [cdkConnectedOverlayOpen]="realOpenState"
      [cdkConnectedOverlayPositions]="overlayPositions"
      (positionChange)="onPositionChange($event)"
      (detach)="close()"
      (overlayKeydown)="onOverlayKeydown($event)"
    >
      <ng-container *ngTemplateOutlet="inlineMode" />
    </ng-template>
  `,
  host: {
    "[class.ant-picker]": `true`,
    "[class.ant-picker-range]": `isRange`,
    "[class.ant-picker-large]": `finalSize() === 'large'`,
    "[class.ant-picker-small]": `finalSize() === 'small'`,
    "[class.ant-picker-disabled]": `nzDisabled`,
    "[class.ant-picker-rtl]": `dir() === 'rtl'`,
    "[class.ant-picker-outlined]": `finalVariant() === 'outlined'`,
    "[class.ant-picker-borderless]": `finalVariant() === 'borderless'`,
    "[class.ant-picker-filled]": `finalVariant() === 'filled'`,
    "[class.ant-picker-underlined]": `finalVariant() === 'underlined'`,
    "[class.ant-picker-inline]": `nzInline()`,
    "(click)": "onClickInputBox($event)",
  },
  hostDirectives: [NzSpaceCompactItemDirective],
  providers: [
    DatePickerService,
    { provide: NZ_SPACE_COMPACT_ITEM_TYPE, useValue: "picker" },
    {
      provide: NG_VALUE_ACCESSOR,
      multi: true,
      useExisting: forwardRef(() => NzDatePickerComponent),
    },
  ],
  imports: [
    FormsModule,
    NgTemplateOutlet,
    NzOutletModule,
    NzIconModule,
    NzFormItemFeedbackIconComponent,
    DateRangePopupComponent,
    CdkConnectedOverlay,
    NzOverlayModule,
  ],
})
export class NzDatePickerComponent
  implements OnInit, OnChanges, AfterViewInit, ControlValueAccessor
{
  public nzConfigService = inject(NzConfigService);
  public datePickerService = inject(DatePickerService);
  protected i18n = inject(NzI18nService);
  protected cdr = inject(ChangeDetectorRef);
  private renderer = inject(Renderer2);
  private elementRef = inject(ElementRef<HTMLElement>);
  private dateHelper = inject(DateHelperService);
  private nzResizeObserver = inject(NzResizeObserver);
  private platform = inject(Platform);
  private destroyRef = inject(DestroyRef);
  protected readonly dir = inject(Directionality).valueSignal;

  readonly _nzModuleName: NzConfigKey = NZ_CONFIG_MODULE_NAME;

  static ngAcceptInputType_nzShowTime: BooleanInput | SupportTimeOptions | null | undefined;
  static ngAcceptInputType_nzMode: NzDateMode | string;

  isRange = !!inject(NzRangePickerComponent, { self: true, optional: true }); // Indicate whether the value is a range value
  extraFooter?: TemplateRef<NzSafeAny> | string;

  // status
  statusCls: NgClassInterface = {};
  status: NzValidateStatus = "";
  hasFeedback: boolean = false;

  public panelMode: NzDateMode | NzDateMode[] = "date";
  private isCustomPlaceHolder: boolean = false;
  private isCustomFormat: boolean = false;
  private showTime: SupportTimeOptions | boolean = false;
  private isNzDisableFirstChange: boolean = true;
  // --- Common API
  readonly nzInline = input(false, { transform: booleanAttribute });
  @Input({ transform: booleanAttribute }) nzAllowClear: boolean = true;
  @Input({ transform: booleanAttribute }) nzAutoFocus: boolean = false;
  @Input({ transform: booleanAttribute }) nzDisabled: boolean = false;
  @Input({ transform: booleanAttribute }) nzInputReadOnly: boolean = false;
  @Input({ transform: booleanAttribute }) nzOpen?: boolean;
  @Input() nzDisabledDate?: (d: Date) => boolean;
  @Input() nzLocale!: NzDatePickerI18nInterface;
  @Input() nzPlaceHolder: string | string[] = "";
  @Input() nzPopupStyle: object = POPUP_STYLE_PATCH;
  readonly nzDropdownClassName = input<string>();
  @Input() nzSize: NzDatePickerSizeType = "default";
  @Input() nzStatus: NzStatus = "";
  @Input() nzFormat!: string;
  @Input() @WithConfig() nzVariant: NzVariant | undefined = undefined;
  @Input() nzDateRender?:
    | TemplateRef<NzSafeAny>
    | string
    | FunctionProp<TemplateRef<Date> | string>;
  @Input() nzDisabledTime?: DisabledTimeFn;
  @Input() nzRenderExtraFooter?:
    | TemplateRef<NzSafeAny>
    | string
    | FunctionProp<TemplateRef<NzSafeAny> | string>;
  @Input({ transform: booleanAttribute }) nzShowToday: boolean = true;
  @Input() nzMode: NzDateMode = "date";
  @Input({ transform: booleanAttribute }) nzShowNow: boolean = true;
  @Input() nzRanges?: PresetRanges;
  @Input() nzDefaultPickerValue: CompatibleDate | null = null;
  @Input() @WithConfig() nzSeparator?: string | TemplateRef<NzSafeAny> = undefined;
  @Input() @WithConfig() nzSuffixIcon: string | TemplateRef<NzSafeAny> = "calendar";
  @Input() @WithConfig() nzBackdrop = false;
  @Input() nzId: string | null = null;
  @Input() nzPlacement: NzPlacement = "bottomLeft";
  @Input({ transform: booleanAttribute }) nzShowWeekNumber: boolean = false;

  @Output() readonly nzOnPanelChange = new EventEmitter<NzPanelChangeType>();
  @Output() readonly nzOnCalendarChange = new EventEmitter<Array<Date | null>>();
  @Output() readonly nzOnOk = new EventEmitter<CompatibleDate | null>();
  @Output() readonly nzOnOpenChange = new EventEmitter<boolean>();

  @Input() get nzShowTime(): SupportTimeOptions | boolean {
    return this.showTime;
  }

  set nzShowTime(value: SupportTimeOptions | boolean) {
    this.showTime = typeof value === "object" ? value : toBoolean(value);
  }

  // ------------------------------------------------------------------------
  // Input API Start
  // ------------------------------------------------------------------------
  @ViewChild(CdkConnectedOverlay, { static: false }) cdkConnectedOverlay?: CdkConnectedOverlay;
  @ViewChild(DateRangePopupComponent, { static: false }) panel!: DateRangePopupComponent;
  @ViewChild("separatorElement", { static: false }) separatorElement?: ElementRef;
  @ViewChild("pickerInput", { static: false }) pickerInput?: ElementRef<HTMLInputElement>;
  @ViewChildren("rangePickerInput") rangePickerInputs?: QueryList<ElementRef<HTMLInputElement>>;

  get origin(): ElementRef {
    return this.elementRef;
  }

  inputSize: number = 12;
  inputWidth?: number;
  prefixCls = PREFIX_CLASS;
  inputValue!: NzSafeAny;
  activeBarStyle: object = {};
  overlayOpen: boolean = false; // Available when "nzOpen" = undefined
  overlayPositions: ConnectionPositionPair[] = [...DEFAULT_DATE_PICKER_POSITIONS];

  private readonly currentPosition = signal<OriginConnectionPosition>({
    originX: "start",
    originY: "bottom",
  });

  protected readonly datepickerAnimationEnter = slideAnimationEnter();
  protected readonly datepickerAnimationLeave = slideAnimationLeave();

  protected readonly dropdownClass = computed(() => {
    const cls: string[] = [];
    if (!this.nzInline()) {
      cls.push(this.generateClass("dropdown"));

      const customCls = this.nzDropdownClassName();
      if (customCls) {
        cls.push(customCls);
      }

      const { originX, originY } = this.currentPosition();

      if (originX === "start" && originY === "bottom") {
        cls.push(this.generateClass("dropdown-placement-bottomLeft"));
      } else if (originX === "start" && originY === "top") {
        cls.push(this.generateClass("dropdown-placement-topLeft"));
      } else if (originX === "end" && originY === "bottom") {
        cls.push(this.generateClass("dropdown-placement-bottomRight"));
      } else if (originX === "end" && originY === "top") {
        cls.push(this.generateClass("dropdown-placement-topRight"));
      }

      if (this.dir() === "rtl") {
        cls.push(this.generateClass("dropdown-rtl"));
      }
    }
    return cls;
  });

  get realOpenState(): boolean {
    // The value that really decide the open state of overlay
    return this.isOpenHandledByUser() ? !!this.nzOpen : this.overlayOpen;
  }

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

  private size = signal<NzSizeLDSType>(this.nzSize);
  private variant = signal<NzVariant | undefined>(this.nzVariant);

  private readonly formSize = inject(NZ_FORM_SIZE, { optional: true });
  private readonly formVariant = inject(NZ_FORM_VARIANT, { optional: true });

  private compactSize = inject(NZ_SPACE_COMPACT_SIZE, { optional: true });
  private document: Document = inject(DOCUMENT);

  constructor() {
    this.datePickerService.isRange = this.isRange;
    this.datePickerService.initValue(true);
  }

  ngAfterViewInit(): void {
    if (this.nzAutoFocus) {
      this.focus();
    }

    if (this.isRange && this.platform.isBrowser) {
      this.nzResizeObserver
        .observe(this.elementRef)
        .pipe(takeUntilDestroyed(this.destroyRef))
        .subscribe(() => {
          this.updateInputWidthAndArrowLeft();
        });
    }

    this.datePickerService.inputPartChange$
      .pipe(takeUntilDestroyed(this.destroyRef))
      .subscribe((partType) => {
        if (partType) {
          this.datePickerService.activeInput = partType;
        }
        this.focus();
        this.updateInputWidthAndArrowLeft();
      });

    if (this.platform.isBrowser) {
      // prevent mousedown event to trigger focusout event when click in date picker
      // see: https://github.com/NG-ZORRO/ng-zorro-antd/issues/7450
      fromEventOutsideAngular(this.elementRef.nativeElement, "mousedown")
        .pipe(takeUntilDestroyed(this.destroyRef))
        .subscribe((event) => {
          if ((event.target as HTMLInputElement).tagName.toLowerCase() !== "input") {
            event.preventDefault();
          }
        });
    }
  }

  updateInputWidthAndArrowLeft(): void {
    this.inputWidth = this.rangePickerInputs?.first?.nativeElement.offsetWidth || 0;

    const baseStyle = { position: "absolute", width: `${this.inputWidth}px` };
    this.datePickerService.arrowLeft =
      this.datePickerService.activeInput === "left"
        ? 0
        : this.inputWidth + this.separatorElement?.nativeElement.offsetWidth || 0;

    if (this.dir() === "rtl") {
      this.activeBarStyle = { ...baseStyle, right: `${this.datePickerService.arrowLeft}px` };
    } else {
      this.activeBarStyle = { ...baseStyle, left: `${this.datePickerService.arrowLeft}px` };
    }

    this.cdr.markForCheck();
  }

  getInput(partType?: RangePartType): HTMLInputElement | undefined {
    if (this.nzInline()) {
      return undefined;
    }
    return this.isRange
      ? partType === "left"
        ? this.rangePickerInputs?.first.nativeElement
        : this.rangePickerInputs?.last.nativeElement
      : this.pickerInput!.nativeElement;
  }

  focus(): void {
    const activeInputElement = this.getInput(this.datePickerService.activeInput);
    if (this.document.activeElement !== activeInputElement) {
      activeInputElement?.focus();
    }
  }

  onFocus(event: FocusEvent, partType?: RangePartType): void {
    event.preventDefault();
    if (partType) {
      this.datePickerService.inputPartChange$.next(partType);
    }
    this.renderClass(true);
  }

  // blur event has not the relatedTarget in IE11, use focusout instead.
  onFocusout(event: FocusEvent): void {
    event.preventDefault();
    this.onTouchedFn();
    if (!this.elementRef.nativeElement.contains(event.relatedTarget as Node)) {
      this.checkAndClose();
    }
    this.renderClass(false);
  }

  // Show overlay content
  open(): void {
    if (this.nzInline()) {
      return;
    }
    if (!this.realOpenState && !this.nzDisabled) {
      this.updateInputWidthAndArrowLeft();
      this.overlayOpen = true;
      this.nzOnOpenChange.emit(true);
      this.focus();
      this.cdr.markForCheck();
    }
  }

  close(): void {
    if (this.nzInline()) {
      return;
    }
    if (this.realOpenState) {
      this.overlayOpen = false;
      this.nzOnOpenChange.emit(false);
    }
  }

  get showClear(): boolean {
    return (
      !this.nzDisabled && !this.isEmptyValue(this.datePickerService.value) && this.nzAllowClear
    );
  }

  checkAndClose(): void {
    if (!this.realOpenState) {
      return;
    }

    if (this.panel.isAllowed(this.datePickerService.value!, true)) {
      if (
        Array.isArray(this.datePickerService.value) &&
        wrongSortOrder(this.datePickerService.value)
      ) {
        const index = this.datePickerService.getActiveIndex();
        const value = this.datePickerService.value[index];
        this.panel.changeValueFromSelect(value!, true);
        return;
      }
      this.updateInputValue();
      this.datePickerService.emitValue$.next();
    } else {
      this.datePickerService.setValue(this.datePickerService.initialValue!);
      this.close();
    }
  }

  onClickInputBox(event: MouseEvent): void {
    event.stopPropagation();
    this.focus();
    if (!this.isOpenHandledByUser()) {
      this.open();
    }
  }

  onOverlayKeydown(event: KeyboardEvent): void {
    if (event.keyCode === ESCAPE) {
      this.datePickerService.initValue();
    }
  }

  onPositionChange(position: ConnectedOverlayPositionChange): void {
    this.currentPosition.set(position.connectionPair);
  }

  onClickClear(event: MouseEvent): void {
    event.preventDefault();
    event.stopPropagation();

    this.datePickerService.initValue(true);
    this.datePickerService.emitValue$.next();
  }

  updateInputValue(): void {
    const newValue = this.datePickerService.value;
    if (this.isRange) {
      this.inputValue = newValue
        ? (newValue as CandyDate[]).map((v) => this.formatValue(v))
        : ["", ""];
    } else {
      this.inputValue = this.formatValue(newValue as CandyDate);
    }
    this.cdr.markForCheck();
  }

  formatValue(value: CandyDate): string {
    return this.dateHelper.format(value && (value as CandyDate).nativeDate, this.nzFormat);
  }

  onInputChange(value: string, isEnter: boolean = false): void {
    /**
     * in IE11 focus/blur will trigger ngModelChange if placeholder changes,
     * so we forbid IE11 to open panel through input change
     */
    if (
      !this.platform.TRIDENT &&
      this.document.activeElement === this.getInput(this.datePickerService.activeInput) &&
      !this.realOpenState
    ) {
      this.open();
      return;
    }

    const date = this.checkValidDate(value);
    // Can only change date when it's open
    if (date && this.realOpenState) {
      this.panel.changeValueFromSelect(date, isEnter);
    }
  }

  onKeyupEnter(event: Event): void {
    this.onInputChange((event.target as HTMLInputElement).value, true);
  }

  private checkValidDate(value: string): CandyDate | null {
    const date = new CandyDate(this.dateHelper.parseDate(value, this.nzFormat));

    if (!date.isValid() || value !== this.dateHelper.format(date.nativeDate, this.nzFormat)) {
      return null;
    }

    return date;
  }

  getPlaceholder(partType?: RangePartType): string {
    return this.isRange
      ? this.nzPlaceHolder[this.datePickerService.getActiveIndex(partType!)]
      : (this.nzPlaceHolder as string);
  }

  isEmptyValue(value: CompatibleValue): boolean {
    if (value === null) {
      return true;
    } else if (this.isRange) {
      return !value || !Array.isArray(value) || value.every((val) => !val);
    } else {
      return !value;
    }
  }

  // Whether open state is permanently controlled by user himself
  isOpenHandledByUser(): boolean {
    return this.nzOpen !== undefined;
  }

  private nzFormStatusService = inject(NzFormStatusService, { optional: true });
  private nzFormNoStatusService = inject(NzFormNoStatusService, { optional: true });

  // ------------------------------------------------------------------------
  // Input API End
  // ------------------------------------------------------------------------

  ngOnInit(): void {
    this.nzFormStatusService?.formStatusChanges
      .pipe(
        distinctUntilChanged((pre, cur) => {
          return pre.status === cur.status && pre.hasFeedback === cur.hasFeedback;
        }),
        withLatestFrom(
          this.nzFormNoStatusService ? this.nzFormNoStatusService.noFormStatus : of(false),
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

    // Subscribe the every locale change if the nzLocale is not handled by user
    if (!this.nzLocale) {
      this.i18n.localeChange
        .pipe(takeUntilDestroyed(this.destroyRef))
        .subscribe(() => this.setLocale());
    }

    this.datePickerService.emitValue$.pipe(takeUntilDestroyed(this.destroyRef)).subscribe(() => {
      const granularityComparison = this.showTime ? "second" : "day";
      const value = this.datePickerService.value;
      const datePickerPreviousValue = this.datePickerService.initialValue;

      // Check if the value has change for a simple datepicker, let us avoid notify the control for nothing
      if (
        !this.isRange &&
        (value as CandyDate)?.isSame(
          (datePickerPreviousValue as CandyDate)?.nativeDate,
          granularityComparison,
        )
      ) {
        this.onTouchedFn();
        return this.close();
      }

      // check if the value has change for a range picker, let us avoid notify the control for nothing
      if (this.isRange) {
        const [previousStartDate, previousEndDate] = datePickerPreviousValue as CandyDate[];
        const [currentStartDate, currentEndDate] = value as CandyDate[];
        if (
          previousStartDate?.isSame(currentStartDate?.nativeDate, granularityComparison) &&
          previousEndDate?.isSame(currentEndDate?.nativeDate, granularityComparison)
        ) {
          this.onTouchedFn();
          return this.close();
        }
      }

      this.datePickerService.initialValue = cloneDate(value);
      if (this.isRange) {
        const vAsRange = value as CandyDate[];
        if (vAsRange.length) {
          this.onChangeFn([vAsRange[0]?.nativeDate ?? null, vAsRange[1]?.nativeDate ?? null]);
        } else {
          this.onChangeFn([]);
        }
      } else {
        if (value) {
          this.onChangeFn((value as CandyDate).nativeDate);
        } else {
          this.onChangeFn(null);
        }
      }
      this.onTouchedFn();
      // When value emitted, overlay will be closed
      this.close();
    });

    this.inputValue = this.isRange ? ["", ""] : "";
    this.setModeAndFormat();

    this.datePickerService.valueChange$.pipe(takeUntilDestroyed(this.destroyRef)).subscribe(() => {
      this.updateInputValue();
    });
  }

  ngOnChanges({
    nzStatus,
    nzPlacement,
    nzPopupStyle,
    nzPlaceHolder,
    nzLocale,
    nzFormat,
    nzRenderExtraFooter,
    nzMode,
    nzSize,
    nzVariant,
  }: SimpleChanges): void {
    if (nzPopupStyle) {
      // Always assign the popup style patch
      this.nzPopupStyle = this.nzPopupStyle
        ? { ...this.nzPopupStyle, ...POPUP_STYLE_PATCH }
        : POPUP_STYLE_PATCH;
    }

    // Mark as customized placeholder by user once nzPlaceHolder assigned at the first time
    if (nzPlaceHolder?.currentValue) {
      this.isCustomPlaceHolder = true;
    }

    if (nzFormat?.currentValue) {
      this.isCustomFormat = true;
      this.updateInputValue();
    }

    if (nzLocale) {
      // The nzLocale is currently handled by user
      this.setDefaultPlaceHolder();
    }

    if (nzRenderExtraFooter) {
      this.extraFooter = valueFunctionProp(this.nzRenderExtraFooter!);
    }

    if (nzMode) {
      this.setDefaultPlaceHolder();
      this.setModeAndFormat();
    }

    if (nzStatus) {
      this.setStatusStyles(this.nzStatus, this.hasFeedback);
    }

    if (nzPlacement) {
      this.setPlacement(this.nzPlacement);
    }

    if (nzSize) {
      this.size.set(nzSize.currentValue);
    }

    if (nzVariant) {
      this.variant.set(nzVariant.currentValue);
    }
  }

  setModeAndFormat(): void {
    const inputFormats: Partial<Record<NzDateMode, string>> = {
      year: "yyyy",
      quarter: "yyyy-[Q]Q",
      month: "yyyy-MM",
      week: "YYYY-ww",
      date: this.nzShowTime ? "yyyy-MM-dd HH:mm:ss" : "yyyy-MM-dd",
    };

    if (!this.nzMode) {
      this.nzMode = "date";
    }

    this.panelMode = this.isRange ? [this.nzMode, this.nzMode] : this.nzMode;

    // Default format when it's empty
    if (!this.isCustomFormat) {
      this.nzFormat = inputFormats[this.nzMode as NzDateMode]!;
    }

    this.inputSize = Math.max(10, this.nzFormat.length) + 2;
    this.updateInputValue();
  }

  /**
   * Triggered when overlayOpen changes (different with realOpenState)
   *
   * @param open The overlayOpen in picker component
   */
  onOpenChange(open: boolean): void {
    this.nzOnOpenChange.emit(open);
  }

  // ------------------------------------------------------------------------
  // | Control value accessor implements
  // ------------------------------------------------------------------------

  // NOTE: onChangeFn/onTouchedFn will not be assigned if user not use as ngModel
  onChangeFn: OnChangeType = () => void 0;
  onTouchedFn: OnTouchedType = () => void 0;

  writeValue(value: CompatibleDate): void {
    this.setValue(value);
    this.cdr.markForCheck();
  }

  registerOnChange(fn: OnChangeType): void {
    this.onChangeFn = fn;
  }

  registerOnTouched(fn: OnTouchedType): void {
    this.onTouchedFn = fn;
  }

  setDisabledState(isDisabled: boolean): void {
    this.nzDisabled = (this.isNzDisableFirstChange && this.nzDisabled) || isDisabled;
    this.cdr.markForCheck();
    this.isNzDisableFirstChange = false;
  }

  // ------------------------------------------------------------------------
  // | Internal methods
  // ------------------------------------------------------------------------

  // Reload locale from i18n with side effects
  private setLocale(): void {
    this.nzLocale = this.i18n.getLocaleData("DatePicker", {});
    this.setDefaultPlaceHolder();
    this.cdr.markForCheck();
  }

  private setDefaultPlaceHolder(): void {
    if (!this.isCustomPlaceHolder && this.nzLocale) {
      const defaultPlaceholder: Partial<Record<NzDateMode, string>> = {
        year: this.getPropertyOfLocale("yearPlaceholder"),
        quarter: this.getPropertyOfLocale("quarterPlaceholder"),
        month: this.getPropertyOfLocale("monthPlaceholder"),
        week: this.getPropertyOfLocale("weekPlaceholder"),
        date: this.getPropertyOfLocale("placeholder"),
      };

      const defaultRangePlaceholder: Partial<Record<NzDateMode, string[]>> = {
        year: this.getPropertyOfLocale("rangeYearPlaceholder"),
        quarter: this.getPropertyOfLocale("rangeQuarterPlaceholder"),
        month: this.getPropertyOfLocale("rangeMonthPlaceholder"),
        week: this.getPropertyOfLocale("rangeWeekPlaceholder"),
        date: this.getPropertyOfLocale("rangePlaceholder"),
      };

      this.nzPlaceHolder = this.isRange
        ? defaultRangePlaceholder[this.nzMode as NzDateMode]!
        : defaultPlaceholder[this.nzMode as NzDateMode]!;
    }
  }

  private getPropertyOfLocale<T extends keyof NzDatePickerLangI18nInterface>(
    type: T,
  ): NzDatePickerLangI18nInterface[T] {
    return this.nzLocale.lang[type] || this.i18n.getLocaleData(`DatePicker.lang.${type}`);
  }

  // Safe way of setting value with default
  private setValue(value: CompatibleDate): void {
    const newValue = this.datePickerService.makeValue(value);
    this.datePickerService.setValue(newValue);
    this.datePickerService.initialValue = cloneDate(newValue);
    this.cdr.detectChanges();
  }

  renderClass(value: boolean): void {
    // TODO: avoid autoFocus cause change after checked error
    if (value) {
      this.renderer.addClass(this.elementRef.nativeElement, "ant-picker-focused");
    } else {
      this.renderer.removeClass(this.elementRef.nativeElement, "ant-picker-focused");
    }
  }

  onPanelModeChange(panelChange: NzPanelChangeType): void {
    this.nzOnPanelChange.emit(panelChange);
  }

  // Emit nzOnCalendarChange when select date by nz-range-picker
  onCalendarChange(value: CompatibleValue): void {
    if (this.isRange && Array.isArray(value)) {
      const rangeValue = value.filter((x) => x instanceof CandyDate).map((x) => x!.nativeDate);
      this.nzOnCalendarChange.emit(rangeValue);
    }
  }

  onResultOk(): void {
    if (this.isRange) {
      const value = this.datePickerService.value as CandyDate[];
      if (value.length) {
        this.nzOnOk.emit([value[0]?.nativeDate || null, value[1]?.nativeDate || null]);
      } else {
        this.nzOnOk.emit([]);
      }
    } else {
      if (this.datePickerService.value) {
        this.nzOnOk.emit((this.datePickerService.value as CandyDate).nativeDate);
      } else {
        this.nzOnOk.emit(null);
      }
    }
  }

  // status
  private setStatusStyles(status: NzValidateStatus, hasFeedback: boolean): void {
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

  private setPlacement(placement: NzPlacement): void {
    const position: ConnectionPositionPair = DATE_PICKER_POSITION_MAP[placement];
    this.overlayPositions = [position, ...DEFAULT_DATE_PICKER_POSITIONS];
    this.currentPosition.set(position);
  }

  private generateClass(suffix: string): string {
    return generateClassName(this.prefixCls, suffix);
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
import { ESCAPE } from "@angular/cdk/keycodes";
import {
  CdkConnectedOverlay,
  ConnectedOverlayPositionChange,
  ConnectionPositionPair,
  OriginConnectionPosition,
} from "@angular/cdk/overlay";
import { Platform } from "@angular/cdk/platform";
import { NgTemplateOutlet } from "@angular/common";
import {
  AfterViewInit,
  booleanAttribute,
  ChangeDetectionStrategy,
  ChangeDetectorRef,
  Component,
  computed,
  DestroyRef,
  DOCUMENT,
  ElementRef,
  EventEmitter,
  forwardRef,
  inject,
  input,
  Input,
  OnChanges,
  OnInit,
  Output,
  QueryList,
  Renderer2,
  signal,
  SimpleChanges,
  TemplateRef,
  ViewChild,
  ViewChildren,
  ViewEncapsulation,
} from "@angular/core";
import { takeUntilDestroyed } from "@angular/core/rxjs-interop";
import { ControlValueAccessor, FormsModule, NG_VALUE_ACCESSOR } from "@angular/forms";
import { of } from "rxjs";
import { distinctUntilChanged, map, withLatestFrom } from "rxjs/operators";

import { NzResizeObserver } from "ng-zorro-antd/cdk/resize-observer";
import { slideAnimationEnter, slideAnimationLeave } from "ng-zorro-antd/core/animation";
import { NzConfigKey, NzConfigService, WithConfig } from "ng-zorro-antd/core/config";
import {
  NZ_FORM_SIZE,
  NZ_FORM_VARIANT,
  NzFormItemFeedbackIconComponent,
  NzFormNoStatusService,
  NzFormStatusService,
} from "ng-zorro-antd/core/form";
import { NzOutletModule } from "ng-zorro-antd/core/outlet";
import {
  DATE_PICKER_POSITION_MAP,
  DEFAULT_DATE_PICKER_POSITIONS,
  NzOverlayModule,
} from "ng-zorro-antd/core/overlay";
import { CandyDate, cloneDate, CompatibleValue, wrongSortOrder } from "ng-zorro-antd/core/time";
import {
  BooleanInput,
  FunctionProp,
  NgClassInterface,
  NzSafeAny,
  NzSizeLDSType,
  NzStatus,
  NzValidateStatus,
  NzVariant,
  OnChangeType,
  OnTouchedType,
  type NzPlacement,
} from "ng-zorro-antd/core/types";
import {
  fromEventOutsideAngular,
  generateClassName,
  getStatusClassNames,
  toBoolean,
  valueFunctionProp,
} from "ng-zorro-antd/core/util";
import {
  DateHelperService,
  NzDatePickerI18nInterface,
  NzDatePickerLangI18nInterface,
  NzI18nService,
} from "ng-zorro-antd/i18n";
import { NzIconModule } from "ng-zorro-antd/icon";
import {
  NZ_SPACE_COMPACT_ITEM_TYPE,
  NZ_SPACE_COMPACT_SIZE,
  NzSpaceCompactItemDirective,
} from "ng-zorro-antd/space";

import { DatePickerService } from "./date-picker.service";
import { DateRangePopupComponent } from "./date-range-popup.component";
import { NzRangePickerComponent } from "./range-picker.component";
import {
  CompatibleDate,
  DisabledTimeFn,
  NzDateMode,
  NzPanelChangeType,
  PresetRanges,
  RangePartType,
  SupportTimeOptions,
} from "./standard-types";
import { PREFIX_CLASS } from "./util";

const POPUP_STYLE_PATCH = { position: "relative" }; // Aim to override antd's style to support overlay's position strategy (position:absolute will cause it not working because the overlay can't get the height/width of it's content)
const NZ_CONFIG_MODULE_NAME: NzConfigKey = "datePicker";

export type NzDatePickerSizeType = "large" | "default" | "small";

/**
 * The base picker for all common APIs
 */
@Component({
  encapsulation: ViewEncapsulation.None,
  changeDetection: ChangeDetectionStrategy.OnPush,
  selector:
    "nz-date-picker,nz-week-picker,nz-month-picker,nz-quarter-picker,nz-year-picker,nz-range-picker",
  exportAs: "nzDatePicker",
  template: `
    @if (!nzInline()) {
      @if (!isRange) {
        <div class="{{ prefixCls }}-input">
          <input
            #pickerInput
            [attr.id]="nzId"
            [class.ant-input-disabled]="nzDisabled"
            [disabled]="nzDisabled"
            [readOnly]="nzInputReadOnly"
            [(ngModel)]="inputValue"
            placeholder="{{ getPlaceholder() }}"
            [size]="inputSize"
            autocomplete="off"
            (focus)="onFocus($event)"
            (focusout)="onFocusout($event)"
            (ngModelChange)="onInputChange($event)"
            (keyup.enter)="onKeyupEnter($event)"
          />
          <ng-container *ngTemplateOutlet="tplRightRest" />
        </div>
      } @else {
        <div class="{{ prefixCls }}-input">
          <ng-container *ngTemplateOutlet="tplRangeInput; context: { partType: 'left' }" />
        </div>
        <div #separatorElement class="{{ prefixCls }}-range-separator">
          <span class="{{ prefixCls }}-separator">
            <ng-container *nzStringTemplateOutlet="nzSeparator; let separator">
              @if (nzSeparator) {
                {{ nzSeparator }}
              } @else {
                <nz-icon nzType="swap-right" nzTheme="outline" />
              }
            </ng-container>
          </span>
        </div>
        <div class="{{ prefixCls }}-input">
          <ng-container *ngTemplateOutlet="tplRangeInput; context: { partType: 'right' }" />
        </div>
        <ng-container *ngTemplateOutlet="tplRightRest" />
      }
    } @else {
      <ng-template [ngTemplateOutlet]="inlineMode" />
    }
    <!-- Input for Range ONLY -->
    <ng-template #tplRangeInput let-partType="partType">
      <input
        #rangePickerInput
        [attr.id]="nzId"
        [disabled]="nzDisabled"
        [readOnly]="nzInputReadOnly"
        [size]="inputSize"
        autocomplete="off"
        (click)="onClickInputBox($event)"
        (focusout)="onFocusout($event)"
        (focus)="onFocus($event, partType)"
        (keyup.enter)="onKeyupEnter($event)"
        [(ngModel)]="inputValue[datePickerService.getActiveIndex(partType)]"
        (ngModelChange)="onInputChange($event)"
        placeholder="{{ getPlaceholder(partType) }}"
      />
    </ng-template>

    <!-- Right operator icons -->
    <ng-template #tplRightRest>
      <div class="{{ prefixCls }}-active-bar" [style]="activeBarStyle"></div>
      @if (showClear) {
        <span class="{{ prefixCls }}-clear" (click)="onClickClear($event)">
          <nz-icon nzType="close-circle" nzTheme="fill" />
        </span>
      }

      <span class="{{ prefixCls }}-suffix">
        <ng-container *nzStringTemplateOutlet="nzSuffixIcon; let suffixIcon">
          <nz-icon [nzType]="suffixIcon" />
        </ng-container>
        @if (hasFeedback && !!status) {
          <nz-form-item-feedback-icon [status]="status" />
        }
      </span>
    </ng-template>

    <ng-template #inlineMode>
      <div
        [class]="dropdownClass()"
        [class.ant-picker-dropdown-range]="!nzInline() && isRange"
        [class.ant-picker-active-left]="datePickerService.activeInput === 'left'"
        [class.ant-picker-active-right]="datePickerService.activeInput === 'right'"
        [style]="nzPopupStyle"
        [animate.enter]="$any(!nzInline() && datepickerAnimationEnter())"
        [animate.leave]="$any(!nzInline() && datepickerAnimationLeave())"
      >
        <date-range-popup
          [isRange]="isRange"
          [inline]="nzInline()"
          [defaultPickerValue]="nzDefaultPickerValue"
          [showWeek]="nzShowWeekNumber || nzMode === 'week'"
          [panelMode]="panelMode"
          (panelModeChange)="onPanelModeChange($event)"
          (calendarChange)="onCalendarChange($event)"
          [locale]="nzLocale?.lang!"
          [showToday]="nzMode === 'date' && nzShowToday && !isRange && !nzShowTime"
          [showNow]="nzMode === 'date' && nzShowNow && !isRange && !!nzShowTime"
          [showTime]="nzShowTime"
          [dateRender]="nzDateRender"
          [disabledDate]="nzDisabledDate"
          [disabledTime]="nzDisabledTime"
          [extraFooter]="extraFooter"
          [ranges]="nzRanges"
          [dir]="dir()"
          [format]="nzFormat"
          (resultOk)="onResultOk()"
        />
      </div>
    </ng-template>

    <!-- Overlay -->
    <ng-template
      cdkConnectedOverlay
      nzConnectedOverlay
      cdkConnectedOverlayTransformOriginOn=".{{ prefixCls }}-dropdown"
      [cdkConnectedOverlayHasBackdrop]="nzBackdrop"
      [cdkConnectedOverlayOrigin]="origin"
      [cdkConnectedOverlayOpen]="realOpenState"
      [cdkConnectedOverlayPositions]="overlayPositions"
      (positionChange)="onPositionChange($event)"
      (detach)="close()"
      (overlayKeydown)="onOverlayKeydown($event)"
    >
      <ng-container *ngTemplateOutlet="inlineMode" />
    </ng-template>
  `,
  host: {
    "[class.ant-picker]": `true`,
    "[class.ant-picker-range]": `isRange`,
    "[class.ant-picker-large]": `finalSize() === 'large'`,
    "[class.ant-picker-small]": `finalSize() === 'small'`,
    "[class.ant-picker-disabled]": `nzDisabled`,
    "[class.ant-picker-rtl]": `dir() === 'rtl'`,
    "[class.ant-picker-outlined]": `finalVariant() === 'outlined'`,
    "[class.ant-picker-borderless]": `finalVariant() === 'borderless'`,
    "[class.ant-picker-filled]": `finalVariant() === 'filled'`,
    "[class.ant-picker-underlined]": `finalVariant() === 'underlined'`,
    "[class.ant-picker-inline]": `nzInline()`,
    "(click)": "onClickInputBox($event)",
  },
  hostDirectives: [NzSpaceCompactItemDirective],
  providers: [
    DatePickerService,
    { provide: NZ_SPACE_COMPACT_ITEM_TYPE, useValue: "picker" },
    {
      provide: NG_VALUE_ACCESSOR,
      multi: true,
      useExisting: forwardRef(() => NzDatePickerComponent),
    },
  ],
  imports: [
    FormsModule,
    NgTemplateOutlet,
    NzOutletModule,
    NzIconModule,
    NzFormItemFeedbackIconComponent,
    DateRangePopupComponent,
    CdkConnectedOverlay,
    NzOverlayModule,
  ],
})
export class NzDatePickerComponent
  implements OnInit, OnChanges, AfterViewInit, ControlValueAccessor
{
  public nzConfigService = inject(NzConfigService);
  public datePickerService = inject(DatePickerService);
  protected i18n = inject(NzI18nService);
  protected cdr = inject(ChangeDetectorRef);
  private renderer = inject(Renderer2);
  private elementRef = inject(ElementRef<HTMLElement>);
  private dateHelper = inject(DateHelperService);
  private nzResizeObserver = inject(NzResizeObserver);
  private platform = inject(Platform);
  private destroyRef = inject(DestroyRef);
  protected readonly dir = inject(Directionality).valueSignal;

  readonly _nzModuleName: NzConfigKey = NZ_CONFIG_MODULE_NAME;

  static ngAcceptInputType_nzShowTime: BooleanInput | SupportTimeOptions | null | undefined;
  static ngAcceptInputType_nzMode: NzDateMode | string;

  isRange = !!inject(NzRangePickerComponent, { self: true, optional: true }); // Indicate whether the value is a range value
  extraFooter?: TemplateRef<NzSafeAny> | string;

  // status
  statusCls: NgClassInterface = {};
  status: NzValidateStatus = "";
  hasFeedback: boolean = false;

  public panelMode: NzDateMode | NzDateMode[] = "date";
  private isCustomPlaceHolder: boolean = false;
  private isCustomFormat: boolean = false;
  private showTime: SupportTimeOptions | boolean = false;
  private isNzDisableFirstChange: boolean = true;
  // --- Common API
  readonly nzInline = input(false, { transform: booleanAttribute });
  @Input({ transform: booleanAttribute }) nzAllowClear: boolean = true;
  @Input({ transform: booleanAttribute }) nzAutoFocus: boolean = false;
  @Input({ transform: booleanAttribute }) nzDisabled: boolean = false;
  @Input({ transform: booleanAttribute }) nzInputReadOnly: boolean = false;
  @Input({ transform: booleanAttribute }) nzOpen?: boolean;
  @Input() nzDisabledDate?: (d: Date) => boolean;
  @Input() nzLocale!: NzDatePickerI18nInterface;
  @Input() nzPlaceHolder: string | string[] = "";
  @Input() nzPopupStyle: object = POPUP_STYLE_PATCH;
  readonly nzDropdownClassName = input<string>();
  @Input() nzSize: NzDatePickerSizeType = "default";
  @Input() nzStatus: NzStatus = "";
  @Input() nzFormat!: string;
  @Input() @WithConfig() nzVariant: NzVariant | undefined = undefined;
  @Input() nzDateRender?:
    TemplateRef<NzSafeAny> | string | FunctionProp<TemplateRef<Date> | string>;
  @Input() nzDisabledTime?: DisabledTimeFn;
  @Input() nzRenderExtraFooter?:
    TemplateRef<NzSafeAny> | string | FunctionProp<TemplateRef<NzSafeAny> | string>;
  @Input({ transform: booleanAttribute }) nzShowToday: boolean = true;
  @Input() nzMode: NzDateMode = "date";
  @Input({ transform: booleanAttribute }) nzShowNow: boolean = true;
  @Input() nzRanges?: PresetRanges;
  @Input() nzDefaultPickerValue: CompatibleDate | null = null;
  @Input() @WithConfig() nzSeparator?: string | TemplateRef<NzSafeAny> = undefined;
  @Input() @WithConfig() nzSuffixIcon: string | TemplateRef<NzSafeAny> = "calendar";
  @Input() @WithConfig() nzBackdrop = false;
  @Input() nzId: string | null = null;
  @Input() nzPlacement: NzPlacement = "bottomLeft";
  @Input({ transform: booleanAttribute }) nzShowWeekNumber: boolean = false;

  @Output() readonly nzOnPanelChange = new EventEmitter<NzPanelChangeType>();
  @Output() readonly nzOnCalendarChange = new EventEmitter<Array<Date | null>>();
  @Output() readonly nzOnOk = new EventEmitter<CompatibleDate | null>();
  @Output() readonly nzOnOpenChange = new EventEmitter<boolean>();

  @Input() get nzShowTime(): SupportTimeOptions | boolean {
    return this.showTime;
  }

  set nzShowTime(value: SupportTimeOptions | boolean) {
    this.showTime = typeof value === "object" ? value : toBoolean(value);
  }

  // ------------------------------------------------------------------------
  // Input API Start
  // ------------------------------------------------------------------------
  @ViewChild(CdkConnectedOverlay, { static: false }) cdkConnectedOverlay?: CdkConnectedOverlay;
  @ViewChild(DateRangePopupComponent, { static: false }) panel!: DateRangePopupComponent;
  @ViewChild("separatorElement", { static: false }) separatorElement?: ElementRef;
  @ViewChild("pickerInput", { static: false }) pickerInput?: ElementRef<HTMLInputElement>;
  @ViewChildren("rangePickerInput") rangePickerInputs?: QueryList<ElementRef<HTMLInputElement>>;

  get origin(): ElementRef {
    return this.elementRef;
  }

  inputSize: number = 12;
  inputWidth?: number;
  prefixCls = PREFIX_CLASS;
  inputValue!: NzSafeAny;
  activeBarStyle: object = {};
  overlayOpen: boolean = false; // Available when "nzOpen" = undefined
  overlayPositions: ConnectionPositionPair[] = [...DEFAULT_DATE_PICKER_POSITIONS];

  private readonly currentPosition = signal<OriginConnectionPosition>({
    originX: "start",
    originY: "bottom",
  });

  protected readonly datepickerAnimationEnter = slideAnimationEnter();
  protected readonly datepickerAnimationLeave = slideAnimationLeave();

  protected readonly dropdownClass = computed(() => {
    const cls: string[] = [];
    if (!this.nzInline()) {
      cls.push(this.generateClass("dropdown"));

      const customCls = this.nzDropdownClassName();
      if (customCls) {
        cls.push(customCls);
      }

      const { originX, originY } = this.currentPosition();

      if (originX === "start" && originY === "bottom") {
        cls.push(this.generateClass("dropdown-placement-bottomLeft"));
      } else if (originX === "start" && originY === "top") {
        cls.push(this.generateClass("dropdown-placement-topLeft"));
      } else if (originX === "end" && originY === "bottom") {
        cls.push(this.generateClass("dropdown-placement-bottomRight"));
      } else if (originX === "end" && originY === "top") {
        cls.push(this.generateClass("dropdown-placement-topRight"));
      }

      if (this.dir() === "rtl") {
        cls.push(this.generateClass("dropdown-rtl"));
      }
    }
    return cls;
  });

  get realOpenState(): boolean {
    // The value that really decide the open state of overlay
    return this.isOpenHandledByUser() ? !!this.nzOpen : this.overlayOpen;
  }

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

  private size = signal<NzSizeLDSType>(this.nzSize);
  private variant = signal<NzVariant | undefined>(this.nzVariant);

  private readonly formSize = inject(NZ_FORM_SIZE, { optional: true });
  private readonly formVariant = inject(NZ_FORM_VARIANT, { optional: true });

  private compactSize = inject(NZ_SPACE_COMPACT_SIZE, { optional: true });
  private document: Document = inject(DOCUMENT);

  constructor() {
    this.datePickerService.isRange = this.isRange;
    this.datePickerService.initValue(true);
  }

  ngAfterViewInit(): void {
    if (this.nzAutoFocus) {
      this.focus();
    }

    if (this.isRange && this.platform.isBrowser) {
      this.nzResizeObserver
        .observe(this.elementRef)
        .pipe(takeUntilDestroyed(this.destroyRef))
        .subscribe(() => {
          this.updateInputWidthAndArrowLeft();
        });
    }

    this.datePickerService.inputPartChange$
      .pipe(takeUntilDestroyed(this.destroyRef))
      .subscribe((partType) => {
        if (partType) {
          this.datePickerService.activeInput = partType;
        }
        this.focus();
        this.updateInputWidthAndArrowLeft();
      });

    if (this.platform.isBrowser) {
      // prevent mousedown event to trigger focusout event when click in date picker
      // see: https://github.com/NG-ZORRO/ng-zorro-antd/issues/7450
      fromEventOutsideAngular(this.elementRef.nativeElement, "mousedown")
        .pipe(takeUntilDestroyed(this.destroyRef))
        .subscribe((event) => {
          if ((event.target as HTMLInputElement).tagName.toLowerCase() !== "input") {
            event.preventDefault();
          }
        });
    }
  }

  updateInputWidthAndArrowLeft(): void {
    this.inputWidth = this.rangePickerInputs?.first?.nativeElement.offsetWidth || 0;

    const baseStyle = { position: "absolute", width: `${this.inputWidth}px` };
    this.datePickerService.arrowLeft =
      this.datePickerService.activeInput === "left"
        ? 0
        : this.inputWidth + this.separatorElement?.nativeElement.offsetWidth || 0;

    if (this.dir() === "rtl") {
      this.activeBarStyle = { ...baseStyle, right: `${this.datePickerService.arrowLeft}px` };
    } else {
      this.activeBarStyle = { ...baseStyle, left: `${this.datePickerService.arrowLeft}px` };
    }

    this.cdr.markForCheck();
  }

  getInput(partType?: RangePartType): HTMLInputElement | undefined {
    if (this.nzInline()) {
      return undefined;
    }
    return this.isRange
      ? partType === "left"
        ? this.rangePickerInputs?.first.nativeElement
        : this.rangePickerInputs?.last.nativeElement
      : this.pickerInput!.nativeElement;
  }

  focus(): void {
    const activeInputElement = this.getInput(this.datePickerService.activeInput);
    if (this.document.activeElement !== activeInputElement) {
      activeInputElement?.focus();
    }
  }

  onFocus(event: FocusEvent, partType?: RangePartType): void {
    event.preventDefault();
    if (partType) {
      this.datePickerService.inputPartChange$.next(partType);
    }
    this.renderClass(true);
  }

  // blur event has not the relatedTarget in IE11, use focusout instead.
  onFocusout(event: FocusEvent): void {
    event.preventDefault();
    this.onTouchedFn();
    if (!this.elementRef.nativeElement.contains(event.relatedTarget as Node)) {
      this.checkAndClose();
    }
    this.renderClass(false);
  }

  // Show overlay content
  open(): void {
    if (this.nzInline()) {
      return;
    }
    if (!this.realOpenState && !this.nzDisabled) {
      this.updateInputWidthAndArrowLeft();
      this.overlayOpen = true;
      this.nzOnOpenChange.emit(true);
      this.focus();
      this.cdr.markForCheck();
    }
  }

  close(): void {
    if (this.nzInline()) {
      return;
    }
    if (this.realOpenState) {
      this.overlayOpen = false;
      this.nzOnOpenChange.emit(false);
    }
  }

  get showClear(): boolean {
    return (
      !this.nzDisabled && !this.isEmptyValue(this.datePickerService.value) && this.nzAllowClear
    );
  }

  checkAndClose(): void {
    if (!this.realOpenState) {
      return;
    }

    if (this.panel.isAllowed(this.datePickerService.value!, true)) {
      if (
        Array.isArray(this.datePickerService.value) &&
        wrongSortOrder(this.datePickerService.value)
      ) {
        const index = this.datePickerService.getActiveIndex();
        const value = this.datePickerService.value[index];
        this.panel.changeValueFromSelect(value!, true);
        return;
      }
      this.updateInputValue();
      this.datePickerService.emitValue$.next();
    } else {
      this.datePickerService.setValue(this.datePickerService.initialValue!);
      this.close();
    }
  }

  onClickInputBox(event: MouseEvent): void {
    event.stopPropagation();
    this.focus();
    if (!this.isOpenHandledByUser()) {
      this.open();
    }
  }

  onOverlayKeydown(event: KeyboardEvent): void {
    if (event.keyCode === ESCAPE) {
      this.datePickerService.initValue();
    }
  }

  onPositionChange(position: ConnectedOverlayPositionChange): void {
    this.currentPosition.set(position.connectionPair);
  }

  onClickClear(event: MouseEvent): void {
    event.preventDefault();
    event.stopPropagation();

    this.datePickerService.initValue(true);
    this.datePickerService.emitValue$.next();
  }

  updateInputValue(): void {
    const newValue = this.datePickerService.value;
    if (this.isRange) {
      this.inputValue = newValue
        ? (newValue as CandyDate[]).map((v) => this.formatValue(v))
        : ["", ""];
    } else {
      this.inputValue = this.formatValue(newValue as CandyDate);
    }
    this.cdr.markForCheck();
  }

  formatValue(value: CandyDate): string {
    return this.dateHelper.format(value && (value as CandyDate).nativeDate, this.nzFormat);
  }

  onInputChange(value: string, isEnter: boolean = false): void {
    /**
     * in IE11 focus/blur will trigger ngModelChange if placeholder changes,
     * so we forbid IE11 to open panel through input change
     */
    if (
      !this.platform.TRIDENT &&
      this.document.activeElement === this.getInput(this.datePickerService.activeInput) &&
      !this.realOpenState
    ) {
      this.open();
      return;
    }

    const date = this.checkValidDate(value);
    // Can only change date when it's open
    if (date && this.realOpenState) {
      this.panel.changeValueFromSelect(date, isEnter);
    }
  }

  onKeyupEnter(event: Event): void {
    this.onInputChange((event.target as HTMLInputElement).value, true);
  }

  private checkValidDate(value: string): CandyDate | null {
    const date = new CandyDate(this.dateHelper.parseDate(value, this.nzFormat));

    if (!date.isValid() || value !== this.dateHelper.format(date.nativeDate, this.nzFormat)) {
      return null;
    }

    return date;
  }

  getPlaceholder(partType?: RangePartType): string {
    return this.isRange
      ? this.nzPlaceHolder[this.datePickerService.getActiveIndex(partType!)]
      : (this.nzPlaceHolder as string);
  }

  isEmptyValue(value: CompatibleValue): boolean {
    if (value === null) {
      return true;
    } else if (this.isRange) {
      return !value || !Array.isArray(value) || value.every((val) => !val);
    } else {
      return !value;
    }
  }

  // Whether open state is permanently controlled by user himself
  isOpenHandledByUser(): boolean {
    return this.nzOpen !== undefined;
  }

  private nzFormStatusService = inject(NzFormStatusService, { optional: true });
  private nzFormNoStatusService = inject(NzFormNoStatusService, { optional: true });

  // ------------------------------------------------------------------------
  // Input API End
  // ------------------------------------------------------------------------

  ngOnInit(): void {
    this.nzFormStatusService?.formStatusChanges
      .pipe(
        distinctUntilChanged((pre, cur) => {
          return pre.status === cur.status && pre.hasFeedback === cur.hasFeedback;
        }),
        withLatestFrom(
          this.nzFormNoStatusService ? this.nzFormNoStatusService.noFormStatus : of(false),
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

    // Subscribe the every locale change if the nzLocale is not handled by user
    if (!this.nzLocale) {
      this.i18n.localeChange
        .pipe(takeUntilDestroyed(this.destroyRef))
        .subscribe(() => this.setLocale());
    }

    this.datePickerService.emitValue$.pipe(takeUntilDestroyed(this.destroyRef)).subscribe(() => {
      const granularityComparison = this.showTime ? "second" : "day";
      const value = this.datePickerService.value;
      const datePickerPreviousValue = this.datePickerService.initialValue;

      // Check if the value has change for a simple datepicker, let us avoid notify the control for nothing
      if (
        !this.isRange &&
        (value as CandyDate)?.isSame(
          (datePickerPreviousValue as CandyDate)?.nativeDate,
          granularityComparison,
        )
      ) {
        this.onTouchedFn();
        return this.close();
      }

      // check if the value has change for a range picker, let us avoid notify the control for nothing
      if (this.isRange) {
        const [previousStartDate, previousEndDate] = datePickerPreviousValue as CandyDate[];
        const [currentStartDate, currentEndDate] = value as CandyDate[];
        if (
          previousStartDate?.isSame(currentStartDate?.nativeDate, granularityComparison) &&
          previousEndDate?.isSame(currentEndDate?.nativeDate, granularityComparison)
        ) {
          this.onTouchedFn();
          return this.close();
        }
      }

      this.datePickerService.initialValue = cloneDate(value);
      if (this.isRange) {
        const vAsRange = value as CandyDate[];
        if (vAsRange.length) {
          this.onChangeFn([vAsRange[0]?.nativeDate ?? null, vAsRange[1]?.nativeDate ?? null]);
        } else {
          this.onChangeFn([]);
        }
      } else {
        if (value) {
          this.onChangeFn((value as CandyDate).nativeDate);
        } else {
          this.onChangeFn(null);
        }
      }
      this.onTouchedFn();
      // When value emitted, overlay will be closed
      this.close();
    });

    this.inputValue = this.isRange ? ["", ""] : "";
    this.setModeAndFormat();

    this.datePickerService.valueChange$.pipe(takeUntilDestroyed(this.destroyRef)).subscribe(() => {
      this.updateInputValue();
    });
  }

  ngOnChanges({
    nzStatus,
    nzPlacement,
    nzPopupStyle,
    nzPlaceHolder,
    nzLocale,
    nzFormat,
    nzRenderExtraFooter,
    nzMode,
    nzSize,
    nzVariant,
  }: SimpleChanges): void {
    if (nzPopupStyle) {
      // Always assign the popup style patch
      this.nzPopupStyle = this.nzPopupStyle
        ? { ...this.nzPopupStyle, ...POPUP_STYLE_PATCH }
        : POPUP_STYLE_PATCH;
    }

    // Mark as customized placeholder by user once nzPlaceHolder assigned at the first time
    if (nzPlaceHolder?.currentValue) {
      this.isCustomPlaceHolder = true;
    }

    if (nzFormat?.currentValue) {
      this.isCustomFormat = true;
      this.updateInputValue();
    }

    if (nzLocale) {
      // The nzLocale is currently handled by user
      this.setDefaultPlaceHolder();
    }

    if (nzRenderExtraFooter) {
      this.extraFooter = valueFunctionProp(this.nzRenderExtraFooter!);
    }

    if (nzMode) {
      this.setDefaultPlaceHolder();
      this.setModeAndFormat();
    }

    if (nzStatus) {
      this.setStatusStyles(this.nzStatus, this.hasFeedback);
    }

    if (nzPlacement) {
      this.setPlacement(this.nzPlacement);
    }

    if (nzSize) {
      this.size.set(nzSize.currentValue);
    }

    if (nzVariant) {
      this.variant.set(nzVariant.currentValue);
    }
  }

  setModeAndFormat(): void {
    const inputFormats: Partial<Record<NzDateMode, string>> = {
      year: "yyyy",
      quarter: "yyyy-[Q]Q",
      month: "yyyy-MM",
      week: "YYYY-ww",
      date: this.nzShowTime ? "yyyy-MM-dd HH:mm:ss" : "yyyy-MM-dd",
    };

    if (!this.nzMode) {
      this.nzMode = "date";
    }

    this.panelMode = this.isRange ? [this.nzMode, this.nzMode] : this.nzMode;

    // Default format when it's empty
    if (!this.isCustomFormat) {
      this.nzFormat = inputFormats[this.nzMode as NzDateMode]!;
    }

    this.inputSize = Math.max(10, this.nzFormat.length) + 2;
    this.updateInputValue();
  }

  /**
   * Triggered when overlayOpen changes (different with realOpenState)
   *
   * @param open The overlayOpen in picker component
   */
  onOpenChange(open: boolean): void {
    this.nzOnOpenChange.emit(open);
  }

  // ------------------------------------------------------------------------
  // | Control value accessor implements
  // ------------------------------------------------------------------------

  // NOTE: onChangeFn/onTouchedFn will not be assigned if user not use as ngModel
  onChangeFn: OnChangeType = () => void 0;
  onTouchedFn: OnTouchedType = () => void 0;

  writeValue(value: CompatibleDate): void {
    this.setValue(value);
    this.cdr.markForCheck();
  }

  registerOnChange(fn: OnChangeType): void {
    this.onChangeFn = fn;
  }

  registerOnTouched(fn: OnTouchedType): void {
    this.onTouchedFn = fn;
  }

  setDisabledState(isDisabled: boolean): void {
    this.nzDisabled = (this.isNzDisableFirstChange && this.nzDisabled) || isDisabled;
    this.cdr.markForCheck();
    this.isNzDisableFirstChange = false;
  }

  // ------------------------------------------------------------------------
  // | Internal methods
  // ------------------------------------------------------------------------

  // Reload locale from i18n with side effects
  private setLocale(): void {
    this.nzLocale = this.i18n.getLocaleData("DatePicker", {});
    this.setDefaultPlaceHolder();
    this.cdr.markForCheck();
  }

  private setDefaultPlaceHolder(): void {
    if (!this.isCustomPlaceHolder && this.nzLocale) {
      const defaultPlaceholder: Partial<Record<NzDateMode, string>> = {
        year: this.getPropertyOfLocale("yearPlaceholder"),
        quarter: this.getPropertyOfLocale("quarterPlaceholder"),
        month: this.getPropertyOfLocale("monthPlaceholder"),
        week: this.getPropertyOfLocale("weekPlaceholder"),
        date: this.getPropertyOfLocale("placeholder"),
      };

      const defaultRangePlaceholder: Partial<Record<NzDateMode, string[]>> = {
        year: this.getPropertyOfLocale("rangeYearPlaceholder"),
        quarter: this.getPropertyOfLocale("rangeQuarterPlaceholder"),
        month: this.getPropertyOfLocale("rangeMonthPlaceholder"),
        week: this.getPropertyOfLocale("rangeWeekPlaceholder"),
        date: this.getPropertyOfLocale("rangePlaceholder"),
      };

      this.nzPlaceHolder = this.isRange
        ? defaultRangePlaceholder[this.nzMode as NzDateMode]!
        : defaultPlaceholder[this.nzMode as NzDateMode]!;
    }
  }

  private getPropertyOfLocale<T extends keyof NzDatePickerLangI18nInterface>(
    type: T,
  ): NzDatePickerLangI18nInterface[T] {
    return this.nzLocale.lang[type] || this.i18n.getLocaleData(`DatePicker.lang.${type}`);
  }

  // Safe way of setting value with default
  private setValue(value: CompatibleDate): void {
    const newValue = this.datePickerService.makeValue(value);
    this.datePickerService.setValue(newValue);
    this.datePickerService.initialValue = cloneDate(newValue);
    this.cdr.detectChanges();
  }

  renderClass(value: boolean): void {
    // TODO: avoid autoFocus cause change after checked error
    if (value) {
      this.renderer.addClass(this.elementRef.nativeElement, "ant-picker-focused");
    } else {
      this.renderer.removeClass(this.elementRef.nativeElement, "ant-picker-focused");
    }
  }

  onPanelModeChange(panelChange: NzPanelChangeType): void {
    this.nzOnPanelChange.emit(panelChange);
  }

  // Emit nzOnCalendarChange when select date by nz-range-picker
  onCalendarChange(value: CompatibleValue): void {
    if (this.isRange && Array.isArray(value)) {
      const rangeValue = value.filter((x) => x instanceof CandyDate).map((x) => x!.nativeDate);
      this.nzOnCalendarChange.emit(rangeValue);
    }
  }

  onResultOk(): void {
    if (this.isRange) {
      const value = this.datePickerService.value as CandyDate[];
      if (value.length) {
        this.nzOnOk.emit([value[0]?.nativeDate || null, value[1]?.nativeDate || null]);
      } else {
        this.nzOnOk.emit([]);
      }
    } else {
      if (this.datePickerService.value) {
        this.nzOnOk.emit((this.datePickerService.value as CandyDate).nativeDate);
      } else {
        this.nzOnOk.emit(null);
      }
    }
  }

  // status
  private setStatusStyles(status: NzValidateStatus, hasFeedback: boolean): void {
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

  private setPlacement(placement: NzPlacement): void {
    const position: ConnectionPositionPair = DATE_PICKER_POSITION_MAP[placement];
    this.overlayPositions = [position, ...DEFAULT_DATE_PICKER_POSITIONS];
    this.currentPosition.set(position);
  }

  private generateClass(suffix: string): string {
    return generateClassName(this.prefixCls, suffix);
  }
}

`````
