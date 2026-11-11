# externals/ng-zorro-antd/components/time-picker/time-picker-panel.component.ts

## Option 1

`````json
{"printWidth":80}
`````

### Diff

`````diff
===================================================================
--- prettier
+++ oxfmt
@@ -297,9 +297,10 @@
     }
   }
 
   get nzDisabledSeconds():
-    undefined | ((hour: number, minute: number) => number[]) {
+    | undefined
+    | ((hour: number, minute: number) => number[]) {
     return this._disabledSeconds;
   }
 
   @Input()

`````

### Actual (oxfmt)

`````ts
/**
 * Use of this source code is governed by an MIT-style license that can be
 * found in the LICENSE file at https://github.com/NG-ZORRO/ng-zorro-antd/blob/master/LICENSE
 */

import { DecimalPipe, NgTemplateOutlet } from "@angular/common";
import {
  ChangeDetectionStrategy,
  ChangeDetectorRef,
  Component,
  DebugElement,
  ElementRef,
  EventEmitter,
  Input,
  NgZone,
  OnChanges,
  OnInit,
  Output,
  SimpleChanges,
  TemplateRef,
  ViewChild,
  ViewEncapsulation,
  booleanAttribute,
  forwardRef,
  numberAttribute,
  inject,
  DestroyRef,
} from "@angular/core";
import { takeUntilDestroyed } from "@angular/core/rxjs-interop";
import { ControlValueAccessor, NG_VALUE_ACCESSOR } from "@angular/forms";

import { NzButtonModule } from "ng-zorro-antd/button";
import { requestAnimationFrame } from "ng-zorro-antd/core/polyfill";
import { fromEventOutsideAngular, isNotNil } from "ng-zorro-antd/core/util";
import { DateHelperService, NzI18nModule } from "ng-zorro-antd/i18n";

import { TimeHolder } from "./time-holder";

function makeRange(
  length: number,
  step: number = 1,
  start: number = 0,
): number[] {
  return new Array(Math.ceil(length / step))
    .fill(0)
    .map((_, i) => (i + start) * step);
}

export type NzTimePickerUnit = "hour" | "minute" | "second" | "12-hour";

@Component({
  encapsulation: ViewEncapsulation.None,
  changeDetection: ChangeDetectionStrategy.OnPush,
  selector: "nz-time-picker-panel",
  exportAs: "nzTimePickerPanel",
  template: `
    @if (nzInDatePicker) {
      <div class="ant-picker-header">
        <div class="ant-picker-header-view">
          {{ dateHelper.format($any(time?.value), format) || "&nbsp;" }}
        </div>
      </div>
    }
    <div class="ant-picker-content">
      @if (hourEnabled) {
        <ul
          #hourListElement
          class="ant-picker-time-panel-column"
          style="position: relative;"
        >
          @for (hour of hourRange; track $index) {
            @if (!(nzHideDisabledOptions && hour.disabled)) {
              <li
                class="ant-picker-time-panel-cell"
                (click)="selectHour(hour)"
                [class.ant-picker-time-panel-cell-selected]="
                  isSelectedHour(hour)
                "
                [class.ant-picker-time-panel-cell-disabled]="hour.disabled"
              >
                <div class="ant-picker-time-panel-cell-inner">
                  {{ hour.index | number: "2.0-0" }}
                </div>
              </li>
            }
          }
        </ul>
      }
      @if (minuteEnabled) {
        <ul
          #minuteListElement
          class="ant-picker-time-panel-column"
          style="position: relative;"
        >
          @for (minute of minuteRange; track $index) {
            @if (!(nzHideDisabledOptions && minute.disabled)) {
              <li
                class="ant-picker-time-panel-cell"
                (click)="selectMinute(minute)"
                [class.ant-picker-time-panel-cell-selected]="
                  isSelectedMinute(minute)
                "
                [class.ant-picker-time-panel-cell-disabled]="minute.disabled"
              >
                <div class="ant-picker-time-panel-cell-inner">
                  {{ minute.index | number: "2.0-0" }}
                </div>
              </li>
            }
          }
        </ul>
      }
      @if (secondEnabled) {
        <ul
          #secondListElement
          class="ant-picker-time-panel-column"
          style="position: relative;"
        >
          @for (second of secondRange; track $index) {
            @if (!(nzHideDisabledOptions && second.disabled)) {
              <li
                class="ant-picker-time-panel-cell"
                (click)="selectSecond(second)"
                [class.ant-picker-time-panel-cell-selected]="
                  isSelectedSecond(second)
                "
                [class.ant-picker-time-panel-cell-disabled]="second.disabled"
              >
                <div class="ant-picker-time-panel-cell-inner">
                  {{ second.index | number: "2.0-0" }}
                </div>
              </li>
            }
          }
        </ul>
      }
      @if (nzUse12Hours) {
        <ul
          #use12HoursListElement
          class="ant-picker-time-panel-column"
          style="position: relative;"
        >
          @for (range of use12HoursRange; track range) {
            <li
              (click)="select12Hours(range)"
              class="ant-picker-time-panel-cell"
              [class.ant-picker-time-panel-cell-selected]="
                isSelected12Hours(range)
              "
            >
              <div class="ant-picker-time-panel-cell-inner">
                {{ range.value }}
              </div>
            </li>
          }
        </ul>
      }
    </div>
    @if (!nzInDatePicker) {
      <div class="ant-picker-footer">
        @if (nzAddOn) {
          <div class="ant-picker-footer-extra">
            <ng-template [ngTemplateOutlet]="nzAddOn" />
          </div>
        }
        <ul class="ant-picker-ranges">
          <li class="ant-picker-now">
            <a (click)="onClickNow()">
              {{ nzNowText || ("Calendar.lang.now" | nzI18n) }}
            </a>
          </li>
          <li class="ant-picker-ok">
            <button
              nz-button
              type="button"
              nzSize="small"
              nzType="primary"
              (click)="onClickOk()"
            >
              {{ nzOkText || ("Calendar.lang.ok" | nzI18n) }}
            </button>
          </li>
        </ul>
      </div>
    }
  `,
  host: {
    class: "ant-picker-time-panel",
    "[class.ant-picker-time-panel-column-0]": `enabledColumns === 0 && !nzInDatePicker`,
    "[class.ant-picker-time-panel-column-1]": `enabledColumns === 1 && !nzInDatePicker`,
    "[class.ant-picker-time-panel-column-2]": `enabledColumns === 2 && !nzInDatePicker`,
    "[class.ant-picker-time-panel-column-3]": `enabledColumns === 3 && !nzInDatePicker`,
    "[class.ant-picker-time-panel-narrow]": `enabledColumns < 3`,
    "[class.ant-picker-time-panel-placement-bottomLeft]": `!nzInDatePicker`,
  },
  providers: [
    {
      provide: NG_VALUE_ACCESSOR,
      useExisting: forwardRef(() => NzTimePickerPanelComponent),
      multi: true,
    },
  ],
  imports: [DecimalPipe, NgTemplateOutlet, NzI18nModule, NzButtonModule],
})
export class NzTimePickerPanelComponent
  implements ControlValueAccessor, OnInit, OnChanges
{
  dateHelper = inject(DateHelperService);
  private ngZone = inject(NgZone);
  private cdr = inject(ChangeDetectorRef);
  private elementRef = inject(ElementRef);
  private destroyRef = inject(DestroyRef);

  private _nzHourStep = 1;
  private _nzMinuteStep = 1;
  private _nzSecondStep = 1;
  private onChange?: (value: Date) => void;
  private onTouch?: () => void;
  private _format = "HH:mm:ss";
  private _disabledHours?: () => number[] = () => [];
  private _disabledMinutes?: (hour: number) => number[] = () => [];
  private _disabledSeconds?: (hour: number, minute: number) => number[] =
    () => [];
  private _allowEmpty = true;
  time = new TimeHolder();
  hourEnabled = true;
  minuteEnabled = true;
  secondEnabled = true;
  firstScrolled = false;
  enabledColumns = 3;
  hourRange!: ReadonlyArray<{ index: number; disabled: boolean }>;
  minuteRange!: ReadonlyArray<{ index: number; disabled: boolean }>;
  secondRange!: ReadonlyArray<{ index: number; disabled: boolean }>;
  use12HoursRange!: ReadonlyArray<{ index: number; value: string }>;

  @ViewChild("hourListElement", { static: false })
  hourListElement?: DebugElement;
  @ViewChild("minuteListElement", { static: false })
  minuteListElement?: DebugElement;
  @ViewChild("secondListElement", { static: false })
  secondListElement?: DebugElement;
  @ViewChild("use12HoursListElement", { static: false })
  use12HoursListElement?: DebugElement;

  @Input({ transform: booleanAttribute }) nzInDatePicker: boolean = false; // If inside a date-picker, more diff works need to be done
  @Input() nzAddOn?: TemplateRef<void>;
  @Input() nzHideDisabledOptions = false;
  @Input() nzClearText?: string;
  @Input() nzNowText?: string;
  @Input() nzOkText?: string;
  @Input() nzPlaceHolder?: string | null;
  @Input({ transform: booleanAttribute }) nzUse12Hours = false;
  @Input() nzDefaultOpenValue?: Date;

  @Output() readonly closePanel = new EventEmitter<void>();

  @Input({ transform: booleanAttribute })
  set nzAllowEmpty(value: boolean) {
    this._allowEmpty = value;
  }

  get nzAllowEmpty(): boolean {
    return this._allowEmpty;
  }

  @Input()
  set nzDisabledHours(value: undefined | (() => number[])) {
    this._disabledHours = value;
    if (this._disabledHours) {
      this.buildHours();
    }
  }

  get nzDisabledHours(): undefined | (() => number[]) {
    return this._disabledHours;
  }

  @Input()
  set nzDisabledMinutes(value: undefined | ((hour: number) => number[])) {
    if (isNotNil(value)) {
      this._disabledMinutes = value;
      this.buildMinutes();
    }
  }

  get nzDisabledMinutes(): undefined | ((hour: number) => number[]) {
    return this._disabledMinutes;
  }

  @Input()
  set nzDisabledSeconds(
    value: undefined | ((hour: number, minute: number) => number[]),
  ) {
    if (isNotNil(value)) {
      this._disabledSeconds = value;
      this.buildSeconds();
    }
  }

  get nzDisabledSeconds():
    | undefined
    | ((hour: number, minute: number) => number[]) {
    return this._disabledSeconds;
  }

  @Input()
  set format(value: string) {
    if (isNotNil(value)) {
      this._format = value;
      this.enabledColumns = 0;
      const charSet = new Set(value);
      this.hourEnabled = charSet.has("H") || charSet.has("h");
      this.minuteEnabled = charSet.has("m");
      this.secondEnabled = charSet.has("s");
      if (this.hourEnabled) {
        this.enabledColumns++;
      }
      if (this.minuteEnabled) {
        this.enabledColumns++;
      }
      if (this.secondEnabled) {
        this.enabledColumns++;
      }
      if (this.nzUse12Hours) {
        this.build12Hours();
      }
    }
  }

  get format(): string {
    return this._format;
  }

  @Input({ transform: numberAttribute })
  set nzHourStep(value: number) {
    this._nzHourStep = value || 1;
    this.buildHours();
  }

  get nzHourStep(): number {
    return this._nzHourStep;
  }

  @Input({ transform: numberAttribute })
  set nzMinuteStep(value: number) {
    this._nzMinuteStep = value || 1;
    this.buildMinutes();
  }

  get nzMinuteStep(): number {
    return this._nzMinuteStep;
  }

  @Input({ transform: numberAttribute })
  set nzSecondStep(value: number) {
    this._nzSecondStep = value || 1;
    this.buildSeconds();
  }

  get nzSecondStep(): number {
    return this._nzSecondStep;
  }

  buildHours(): void {
    let hourRanges = 24;
    let disabledHours = this.nzDisabledHours?.();
    let startIndex = 0;
    if (this.nzUse12Hours) {
      hourRanges = 12;
      if (disabledHours) {
        if (this.time.selected12Hours === "PM") {
          /**
           * Filter and transform hours which greater or equal to 12
           * [0, 1, 2, ..., 12, 13, 14, 15, ..., 23] => [12, 1, 2, 3, ..., 11]
           */
          disabledHours = disabledHours
            .filter((i) => i >= 12)
            .map((i) => (i > 12 ? i - 12 : i));
        } else {
          /**
           * Filter and transform hours which less than 12
           * [0, 1, 2, ..., 12, 13, 14, 15, ...23] => [12, 1, 2, 3, ..., 11]
           */
          disabledHours = disabledHours
            .filter((i) => i < 12 || i === 24)
            .map((i) => (i === 24 || i === 0 ? 12 : i));
        }
      }
      startIndex = 1;
    }
    this.hourRange = makeRange(hourRanges, this.nzHourStep, startIndex).map(
      (r) => ({
        index: r,
        disabled: !!disabledHours && disabledHours.indexOf(r) !== -1,
      }),
    );
    if (
      this.nzUse12Hours &&
      this.hourRange[this.hourRange.length - 1].index === 12
    ) {
      const temp = [...this.hourRange];
      temp.unshift(temp[temp.length - 1]);
      temp.splice(temp.length - 1, 1);
      this.hourRange = temp;
    }
  }

  buildMinutes(): void {
    this.minuteRange = makeRange(60, this.nzMinuteStep).map((r) => ({
      index: r,
      disabled:
        !!this.nzDisabledMinutes &&
        this.nzDisabledMinutes(this.time.hours!).indexOf(r) !== -1,
    }));
  }

  buildSeconds(): void {
    this.secondRange = makeRange(60, this.nzSecondStep).map((r) => ({
      index: r,
      disabled:
        !!this.nzDisabledSeconds &&
        this.nzDisabledSeconds(this.time.hours!, this.time.minutes!).indexOf(
          r,
        ) !== -1,
    }));
  }

  build12Hours(): void {
    const isUpperFormat = this._format.includes("A");
    this.use12HoursRange = [
      {
        index: 0,
        value: isUpperFormat ? "AM" : "am",
      },
      {
        index: 1,
        value: isUpperFormat ? "PM" : "pm",
      },
    ];
  }

  buildTimes(): void {
    this.buildHours();
    this.buildMinutes();
    this.buildSeconds();
    this.build12Hours();
  }

  scrollToTime(delay: number = 0): void {
    if (this.hourEnabled && this.hourListElement) {
      this.scrollToSelected(
        this.hourListElement.nativeElement,
        this.time.viewHours!,
        delay,
        "hour",
      );
    }
    if (this.minuteEnabled && this.minuteListElement) {
      this.scrollToSelected(
        this.minuteListElement.nativeElement,
        this.time.minutes!,
        delay,
        "minute",
      );
    }
    if (this.secondEnabled && this.secondListElement) {
      this.scrollToSelected(
        this.secondListElement.nativeElement,
        this.time.seconds!,
        delay,
        "second",
      );
    }
    if (this.nzUse12Hours && this.use12HoursListElement) {
      const selectedHours = this.time.selected12Hours;
      const index = selectedHours === "AM" ? 0 : 1;
      this.scrollToSelected(
        this.use12HoursListElement.nativeElement,
        index,
        delay,
        "12-hour",
      );
    }
  }

  selectHour(hour: { index: number; disabled: boolean }): void {
    this.time.setHours(hour.index, hour.disabled);
    if (this._disabledMinutes) {
      this.buildMinutes();
    }
    if (this._disabledSeconds || this._disabledMinutes) {
      this.buildSeconds();
    }
  }

  selectMinute(minute: { index: number; disabled: boolean }): void {
    this.time.setMinutes(minute.index, minute.disabled);
    if (this._disabledSeconds) {
      this.buildSeconds();
    }
  }

  selectSecond(second: { index: number; disabled: boolean }): void {
    this.time.setSeconds(second.index, second.disabled);
  }

  select12Hours(value: { index: number; value: string }): void {
    this.time.setSelected12Hours(value.value);
    if (this._disabledHours) {
      this.buildHours();
    }
    if (this._disabledMinutes) {
      this.buildMinutes();
    }
    if (this._disabledSeconds) {
      this.buildSeconds();
    }
  }

  scrollToSelected(
    instance: HTMLElement,
    index: number,
    duration: number = 0,
    unit: NzTimePickerUnit,
  ): void {
    if (!instance) {
      return;
    }
    const transIndex = this.translateIndex(index, unit);
    const currentOption = (instance.children[transIndex] ||
      instance.children[0]) as HTMLElement;
    this.scrollTo(instance, currentOption.offsetTop, duration);
  }

  translateIndex(index: number, unit: NzTimePickerUnit): number {
    if (unit === "hour") {
      return this.calcIndex(
        this.nzDisabledHours?.(),
        this.hourRange.map((item) => item.index).indexOf(index),
      );
    } else if (unit === "minute") {
      return this.calcIndex(
        this.nzDisabledMinutes?.(this.time.hours!),
        this.minuteRange.map((item) => item.index).indexOf(index),
      );
    } else if (unit === "second") {
      // second
      return this.calcIndex(
        this.nzDisabledSeconds?.(this.time.hours!, this.time.minutes!),
        this.secondRange.map((item) => item.index).indexOf(index),
      );
    } else {
      // 12-hour
      return this.calcIndex(
        [],
        this.use12HoursRange.map((item) => item.index).indexOf(index),
      );
    }
  }

  scrollTo(element: HTMLElement, to: number, duration: number): void {
    if (duration <= 0) {
      element.scrollTop = to;
      return;
    }
    const difference = to - element.scrollTop;
    const perTick = (difference / duration) * 10;

    this.ngZone.runOutsideAngular(() => {
      requestAnimationFrame(() => {
        element.scrollTop = element.scrollTop + perTick;
        if (element.scrollTop === to) {
          return;
        }
        this.scrollTo(element, to, duration - 10);
      });
    });
  }

  calcIndex(array: number[] | undefined, index: number): number {
    if (array?.length && this.nzHideDisabledOptions) {
      return (
        index - array.reduce((pre, value) => pre + (value < index ? 1 : 0), 0)
      );
    } else {
      return index;
    }
  }

  protected changed(): void {
    if (this.onChange) {
      this.onChange(this.time.value!);
    }
  }

  protected touched(): void {
    if (this.onTouch) {
      this.onTouch();
    }
  }

  timeDisabled(value: Date): boolean {
    const hour = value.getHours();
    const minute = value.getMinutes();
    const second = value.getSeconds();
    return (
      (this.nzDisabledHours?.().indexOf(hour) ?? -1) > -1 ||
      (this.nzDisabledMinutes?.(hour).indexOf(minute) ?? -1) > -1 ||
      (this.nzDisabledSeconds?.(hour, minute).indexOf(second) ?? -1) > -1
    );
  }

  onClickNow(): void {
    const now = new Date();
    if (this.timeDisabled(now)) {
      return;
    }
    this.time.setValue(now);
    this.changed();
    this.closePanel.emit();
  }

  onClickOk(): void {
    this.time.setValue(this.time.value, this.nzUse12Hours);
    this.changed();
    this.closePanel.emit();
  }

  isSelectedHour(hour: { index: number; disabled: boolean }): boolean {
    return hour.index === this.time.viewHours;
  }

  isSelectedMinute(minute: { index: number; disabled: boolean }): boolean {
    return minute.index === this.time.minutes;
  }

  isSelectedSecond(second: { index: number; disabled: boolean }): boolean {
    return second.index === this.time.seconds;
  }

  isSelected12Hours(value: { index: number; value: string }): boolean {
    return value.value.toUpperCase() === this.time.selected12Hours;
  }

  ngOnInit(): void {
    this.time.changes
      .pipe(takeUntilDestroyed(this.destroyRef))
      .subscribe(() => {
        this.changed();
        this.touched();
        this.scrollToTime(120);
      });
    this.buildTimes();

    this.ngZone.runOutsideAngular(() => {
      setTimeout(() => {
        this.scrollToTime();
        this.firstScrolled = true;
      });
    });

    fromEventOutsideAngular(this.elementRef.nativeElement, "mousedown")
      .pipe(takeUntilDestroyed(this.destroyRef))
      .subscribe((event) => {
        event.preventDefault();
      });
  }

  ngOnChanges(changes: SimpleChanges): void {
    const { nzUse12Hours, nzDefaultOpenValue } = changes;
    if (!nzUse12Hours?.previousValue && nzUse12Hours?.currentValue) {
      this.build12Hours();
      this.enabledColumns++;
    }
    if (nzDefaultOpenValue?.currentValue) {
      this.time.setDefaultOpenValue(this.nzDefaultOpenValue || new Date());
    }
  }

  writeValue(value: Date): void {
    this.time.setValue(value, this.nzUse12Hours);
    this.buildTimes();

    if (value && this.firstScrolled) {
      this.scrollToTime(120);
    }
    // Mark this component to be checked manually with internal properties changing (see: https://github.com/angular/angular/issues/10816)
    this.cdr.markForCheck();
  }

  registerOnChange(fn: (value: Date) => void): void {
    this.onChange = fn;
  }

  registerOnTouched(fn: () => void): void {
    this.onTouch = fn;
  }
}

`````

### Expected (prettier)

`````ts
/**
 * Use of this source code is governed by an MIT-style license that can be
 * found in the LICENSE file at https://github.com/NG-ZORRO/ng-zorro-antd/blob/master/LICENSE
 */

import { DecimalPipe, NgTemplateOutlet } from "@angular/common";
import {
  ChangeDetectionStrategy,
  ChangeDetectorRef,
  Component,
  DebugElement,
  ElementRef,
  EventEmitter,
  Input,
  NgZone,
  OnChanges,
  OnInit,
  Output,
  SimpleChanges,
  TemplateRef,
  ViewChild,
  ViewEncapsulation,
  booleanAttribute,
  forwardRef,
  numberAttribute,
  inject,
  DestroyRef,
} from "@angular/core";
import { takeUntilDestroyed } from "@angular/core/rxjs-interop";
import { ControlValueAccessor, NG_VALUE_ACCESSOR } from "@angular/forms";

import { NzButtonModule } from "ng-zorro-antd/button";
import { requestAnimationFrame } from "ng-zorro-antd/core/polyfill";
import { fromEventOutsideAngular, isNotNil } from "ng-zorro-antd/core/util";
import { DateHelperService, NzI18nModule } from "ng-zorro-antd/i18n";

import { TimeHolder } from "./time-holder";

function makeRange(
  length: number,
  step: number = 1,
  start: number = 0,
): number[] {
  return new Array(Math.ceil(length / step))
    .fill(0)
    .map((_, i) => (i + start) * step);
}

export type NzTimePickerUnit = "hour" | "minute" | "second" | "12-hour";

@Component({
  encapsulation: ViewEncapsulation.None,
  changeDetection: ChangeDetectionStrategy.OnPush,
  selector: "nz-time-picker-panel",
  exportAs: "nzTimePickerPanel",
  template: `
    @if (nzInDatePicker) {
      <div class="ant-picker-header">
        <div class="ant-picker-header-view">
          {{ dateHelper.format($any(time?.value), format) || "&nbsp;" }}
        </div>
      </div>
    }
    <div class="ant-picker-content">
      @if (hourEnabled) {
        <ul
          #hourListElement
          class="ant-picker-time-panel-column"
          style="position: relative;"
        >
          @for (hour of hourRange; track $index) {
            @if (!(nzHideDisabledOptions && hour.disabled)) {
              <li
                class="ant-picker-time-panel-cell"
                (click)="selectHour(hour)"
                [class.ant-picker-time-panel-cell-selected]="
                  isSelectedHour(hour)
                "
                [class.ant-picker-time-panel-cell-disabled]="hour.disabled"
              >
                <div class="ant-picker-time-panel-cell-inner">
                  {{ hour.index | number: "2.0-0" }}
                </div>
              </li>
            }
          }
        </ul>
      }
      @if (minuteEnabled) {
        <ul
          #minuteListElement
          class="ant-picker-time-panel-column"
          style="position: relative;"
        >
          @for (minute of minuteRange; track $index) {
            @if (!(nzHideDisabledOptions && minute.disabled)) {
              <li
                class="ant-picker-time-panel-cell"
                (click)="selectMinute(minute)"
                [class.ant-picker-time-panel-cell-selected]="
                  isSelectedMinute(minute)
                "
                [class.ant-picker-time-panel-cell-disabled]="minute.disabled"
              >
                <div class="ant-picker-time-panel-cell-inner">
                  {{ minute.index | number: "2.0-0" }}
                </div>
              </li>
            }
          }
        </ul>
      }
      @if (secondEnabled) {
        <ul
          #secondListElement
          class="ant-picker-time-panel-column"
          style="position: relative;"
        >
          @for (second of secondRange; track $index) {
            @if (!(nzHideDisabledOptions && second.disabled)) {
              <li
                class="ant-picker-time-panel-cell"
                (click)="selectSecond(second)"
                [class.ant-picker-time-panel-cell-selected]="
                  isSelectedSecond(second)
                "
                [class.ant-picker-time-panel-cell-disabled]="second.disabled"
              >
                <div class="ant-picker-time-panel-cell-inner">
                  {{ second.index | number: "2.0-0" }}
                </div>
              </li>
            }
          }
        </ul>
      }
      @if (nzUse12Hours) {
        <ul
          #use12HoursListElement
          class="ant-picker-time-panel-column"
          style="position: relative;"
        >
          @for (range of use12HoursRange; track range) {
            <li
              (click)="select12Hours(range)"
              class="ant-picker-time-panel-cell"
              [class.ant-picker-time-panel-cell-selected]="
                isSelected12Hours(range)
              "
            >
              <div class="ant-picker-time-panel-cell-inner">
                {{ range.value }}
              </div>
            </li>
          }
        </ul>
      }
    </div>
    @if (!nzInDatePicker) {
      <div class="ant-picker-footer">
        @if (nzAddOn) {
          <div class="ant-picker-footer-extra">
            <ng-template [ngTemplateOutlet]="nzAddOn" />
          </div>
        }
        <ul class="ant-picker-ranges">
          <li class="ant-picker-now">
            <a (click)="onClickNow()">
              {{ nzNowText || ("Calendar.lang.now" | nzI18n) }}
            </a>
          </li>
          <li class="ant-picker-ok">
            <button
              nz-button
              type="button"
              nzSize="small"
              nzType="primary"
              (click)="onClickOk()"
            >
              {{ nzOkText || ("Calendar.lang.ok" | nzI18n) }}
            </button>
          </li>
        </ul>
      </div>
    }
  `,
  host: {
    class: "ant-picker-time-panel",
    "[class.ant-picker-time-panel-column-0]": `enabledColumns === 0 && !nzInDatePicker`,
    "[class.ant-picker-time-panel-column-1]": `enabledColumns === 1 && !nzInDatePicker`,
    "[class.ant-picker-time-panel-column-2]": `enabledColumns === 2 && !nzInDatePicker`,
    "[class.ant-picker-time-panel-column-3]": `enabledColumns === 3 && !nzInDatePicker`,
    "[class.ant-picker-time-panel-narrow]": `enabledColumns < 3`,
    "[class.ant-picker-time-panel-placement-bottomLeft]": `!nzInDatePicker`,
  },
  providers: [
    {
      provide: NG_VALUE_ACCESSOR,
      useExisting: forwardRef(() => NzTimePickerPanelComponent),
      multi: true,
    },
  ],
  imports: [DecimalPipe, NgTemplateOutlet, NzI18nModule, NzButtonModule],
})
export class NzTimePickerPanelComponent
  implements ControlValueAccessor, OnInit, OnChanges
{
  dateHelper = inject(DateHelperService);
  private ngZone = inject(NgZone);
  private cdr = inject(ChangeDetectorRef);
  private elementRef = inject(ElementRef);
  private destroyRef = inject(DestroyRef);

  private _nzHourStep = 1;
  private _nzMinuteStep = 1;
  private _nzSecondStep = 1;
  private onChange?: (value: Date) => void;
  private onTouch?: () => void;
  private _format = "HH:mm:ss";
  private _disabledHours?: () => number[] = () => [];
  private _disabledMinutes?: (hour: number) => number[] = () => [];
  private _disabledSeconds?: (hour: number, minute: number) => number[] =
    () => [];
  private _allowEmpty = true;
  time = new TimeHolder();
  hourEnabled = true;
  minuteEnabled = true;
  secondEnabled = true;
  firstScrolled = false;
  enabledColumns = 3;
  hourRange!: ReadonlyArray<{ index: number; disabled: boolean }>;
  minuteRange!: ReadonlyArray<{ index: number; disabled: boolean }>;
  secondRange!: ReadonlyArray<{ index: number; disabled: boolean }>;
  use12HoursRange!: ReadonlyArray<{ index: number; value: string }>;

  @ViewChild("hourListElement", { static: false })
  hourListElement?: DebugElement;
  @ViewChild("minuteListElement", { static: false })
  minuteListElement?: DebugElement;
  @ViewChild("secondListElement", { static: false })
  secondListElement?: DebugElement;
  @ViewChild("use12HoursListElement", { static: false })
  use12HoursListElement?: DebugElement;

  @Input({ transform: booleanAttribute }) nzInDatePicker: boolean = false; // If inside a date-picker, more diff works need to be done
  @Input() nzAddOn?: TemplateRef<void>;
  @Input() nzHideDisabledOptions = false;
  @Input() nzClearText?: string;
  @Input() nzNowText?: string;
  @Input() nzOkText?: string;
  @Input() nzPlaceHolder?: string | null;
  @Input({ transform: booleanAttribute }) nzUse12Hours = false;
  @Input() nzDefaultOpenValue?: Date;

  @Output() readonly closePanel = new EventEmitter<void>();

  @Input({ transform: booleanAttribute })
  set nzAllowEmpty(value: boolean) {
    this._allowEmpty = value;
  }

  get nzAllowEmpty(): boolean {
    return this._allowEmpty;
  }

  @Input()
  set nzDisabledHours(value: undefined | (() => number[])) {
    this._disabledHours = value;
    if (this._disabledHours) {
      this.buildHours();
    }
  }

  get nzDisabledHours(): undefined | (() => number[]) {
    return this._disabledHours;
  }

  @Input()
  set nzDisabledMinutes(value: undefined | ((hour: number) => number[])) {
    if (isNotNil(value)) {
      this._disabledMinutes = value;
      this.buildMinutes();
    }
  }

  get nzDisabledMinutes(): undefined | ((hour: number) => number[]) {
    return this._disabledMinutes;
  }

  @Input()
  set nzDisabledSeconds(
    value: undefined | ((hour: number, minute: number) => number[]),
  ) {
    if (isNotNil(value)) {
      this._disabledSeconds = value;
      this.buildSeconds();
    }
  }

  get nzDisabledSeconds():
    undefined | ((hour: number, minute: number) => number[]) {
    return this._disabledSeconds;
  }

  @Input()
  set format(value: string) {
    if (isNotNil(value)) {
      this._format = value;
      this.enabledColumns = 0;
      const charSet = new Set(value);
      this.hourEnabled = charSet.has("H") || charSet.has("h");
      this.minuteEnabled = charSet.has("m");
      this.secondEnabled = charSet.has("s");
      if (this.hourEnabled) {
        this.enabledColumns++;
      }
      if (this.minuteEnabled) {
        this.enabledColumns++;
      }
      if (this.secondEnabled) {
        this.enabledColumns++;
      }
      if (this.nzUse12Hours) {
        this.build12Hours();
      }
    }
  }

  get format(): string {
    return this._format;
  }

  @Input({ transform: numberAttribute })
  set nzHourStep(value: number) {
    this._nzHourStep = value || 1;
    this.buildHours();
  }

  get nzHourStep(): number {
    return this._nzHourStep;
  }

  @Input({ transform: numberAttribute })
  set nzMinuteStep(value: number) {
    this._nzMinuteStep = value || 1;
    this.buildMinutes();
  }

  get nzMinuteStep(): number {
    return this._nzMinuteStep;
  }

  @Input({ transform: numberAttribute })
  set nzSecondStep(value: number) {
    this._nzSecondStep = value || 1;
    this.buildSeconds();
  }

  get nzSecondStep(): number {
    return this._nzSecondStep;
  }

  buildHours(): void {
    let hourRanges = 24;
    let disabledHours = this.nzDisabledHours?.();
    let startIndex = 0;
    if (this.nzUse12Hours) {
      hourRanges = 12;
      if (disabledHours) {
        if (this.time.selected12Hours === "PM") {
          /**
           * Filter and transform hours which greater or equal to 12
           * [0, 1, 2, ..., 12, 13, 14, 15, ..., 23] => [12, 1, 2, 3, ..., 11]
           */
          disabledHours = disabledHours
            .filter((i) => i >= 12)
            .map((i) => (i > 12 ? i - 12 : i));
        } else {
          /**
           * Filter and transform hours which less than 12
           * [0, 1, 2, ..., 12, 13, 14, 15, ...23] => [12, 1, 2, 3, ..., 11]
           */
          disabledHours = disabledHours
            .filter((i) => i < 12 || i === 24)
            .map((i) => (i === 24 || i === 0 ? 12 : i));
        }
      }
      startIndex = 1;
    }
    this.hourRange = makeRange(hourRanges, this.nzHourStep, startIndex).map(
      (r) => ({
        index: r,
        disabled: !!disabledHours && disabledHours.indexOf(r) !== -1,
      }),
    );
    if (
      this.nzUse12Hours &&
      this.hourRange[this.hourRange.length - 1].index === 12
    ) {
      const temp = [...this.hourRange];
      temp.unshift(temp[temp.length - 1]);
      temp.splice(temp.length - 1, 1);
      this.hourRange = temp;
    }
  }

  buildMinutes(): void {
    this.minuteRange = makeRange(60, this.nzMinuteStep).map((r) => ({
      index: r,
      disabled:
        !!this.nzDisabledMinutes &&
        this.nzDisabledMinutes(this.time.hours!).indexOf(r) !== -1,
    }));
  }

  buildSeconds(): void {
    this.secondRange = makeRange(60, this.nzSecondStep).map((r) => ({
      index: r,
      disabled:
        !!this.nzDisabledSeconds &&
        this.nzDisabledSeconds(this.time.hours!, this.time.minutes!).indexOf(
          r,
        ) !== -1,
    }));
  }

  build12Hours(): void {
    const isUpperFormat = this._format.includes("A");
    this.use12HoursRange = [
      {
        index: 0,
        value: isUpperFormat ? "AM" : "am",
      },
      {
        index: 1,
        value: isUpperFormat ? "PM" : "pm",
      },
    ];
  }

  buildTimes(): void {
    this.buildHours();
    this.buildMinutes();
    this.buildSeconds();
    this.build12Hours();
  }

  scrollToTime(delay: number = 0): void {
    if (this.hourEnabled && this.hourListElement) {
      this.scrollToSelected(
        this.hourListElement.nativeElement,
        this.time.viewHours!,
        delay,
        "hour",
      );
    }
    if (this.minuteEnabled && this.minuteListElement) {
      this.scrollToSelected(
        this.minuteListElement.nativeElement,
        this.time.minutes!,
        delay,
        "minute",
      );
    }
    if (this.secondEnabled && this.secondListElement) {
      this.scrollToSelected(
        this.secondListElement.nativeElement,
        this.time.seconds!,
        delay,
        "second",
      );
    }
    if (this.nzUse12Hours && this.use12HoursListElement) {
      const selectedHours = this.time.selected12Hours;
      const index = selectedHours === "AM" ? 0 : 1;
      this.scrollToSelected(
        this.use12HoursListElement.nativeElement,
        index,
        delay,
        "12-hour",
      );
    }
  }

  selectHour(hour: { index: number; disabled: boolean }): void {
    this.time.setHours(hour.index, hour.disabled);
    if (this._disabledMinutes) {
      this.buildMinutes();
    }
    if (this._disabledSeconds || this._disabledMinutes) {
      this.buildSeconds();
    }
  }

  selectMinute(minute: { index: number; disabled: boolean }): void {
    this.time.setMinutes(minute.index, minute.disabled);
    if (this._disabledSeconds) {
      this.buildSeconds();
    }
  }

  selectSecond(second: { index: number; disabled: boolean }): void {
    this.time.setSeconds(second.index, second.disabled);
  }

  select12Hours(value: { index: number; value: string }): void {
    this.time.setSelected12Hours(value.value);
    if (this._disabledHours) {
      this.buildHours();
    }
    if (this._disabledMinutes) {
      this.buildMinutes();
    }
    if (this._disabledSeconds) {
      this.buildSeconds();
    }
  }

  scrollToSelected(
    instance: HTMLElement,
    index: number,
    duration: number = 0,
    unit: NzTimePickerUnit,
  ): void {
    if (!instance) {
      return;
    }
    const transIndex = this.translateIndex(index, unit);
    const currentOption = (instance.children[transIndex] ||
      instance.children[0]) as HTMLElement;
    this.scrollTo(instance, currentOption.offsetTop, duration);
  }

  translateIndex(index: number, unit: NzTimePickerUnit): number {
    if (unit === "hour") {
      return this.calcIndex(
        this.nzDisabledHours?.(),
        this.hourRange.map((item) => item.index).indexOf(index),
      );
    } else if (unit === "minute") {
      return this.calcIndex(
        this.nzDisabledMinutes?.(this.time.hours!),
        this.minuteRange.map((item) => item.index).indexOf(index),
      );
    } else if (unit === "second") {
      // second
      return this.calcIndex(
        this.nzDisabledSeconds?.(this.time.hours!, this.time.minutes!),
        this.secondRange.map((item) => item.index).indexOf(index),
      );
    } else {
      // 12-hour
      return this.calcIndex(
        [],
        this.use12HoursRange.map((item) => item.index).indexOf(index),
      );
    }
  }

  scrollTo(element: HTMLElement, to: number, duration: number): void {
    if (duration <= 0) {
      element.scrollTop = to;
      return;
    }
    const difference = to - element.scrollTop;
    const perTick = (difference / duration) * 10;

    this.ngZone.runOutsideAngular(() => {
      requestAnimationFrame(() => {
        element.scrollTop = element.scrollTop + perTick;
        if (element.scrollTop === to) {
          return;
        }
        this.scrollTo(element, to, duration - 10);
      });
    });
  }

  calcIndex(array: number[] | undefined, index: number): number {
    if (array?.length && this.nzHideDisabledOptions) {
      return (
        index - array.reduce((pre, value) => pre + (value < index ? 1 : 0), 0)
      );
    } else {
      return index;
    }
  }

  protected changed(): void {
    if (this.onChange) {
      this.onChange(this.time.value!);
    }
  }

  protected touched(): void {
    if (this.onTouch) {
      this.onTouch();
    }
  }

  timeDisabled(value: Date): boolean {
    const hour = value.getHours();
    const minute = value.getMinutes();
    const second = value.getSeconds();
    return (
      (this.nzDisabledHours?.().indexOf(hour) ?? -1) > -1 ||
      (this.nzDisabledMinutes?.(hour).indexOf(minute) ?? -1) > -1 ||
      (this.nzDisabledSeconds?.(hour, minute).indexOf(second) ?? -1) > -1
    );
  }

  onClickNow(): void {
    const now = new Date();
    if (this.timeDisabled(now)) {
      return;
    }
    this.time.setValue(now);
    this.changed();
    this.closePanel.emit();
  }

  onClickOk(): void {
    this.time.setValue(this.time.value, this.nzUse12Hours);
    this.changed();
    this.closePanel.emit();
  }

  isSelectedHour(hour: { index: number; disabled: boolean }): boolean {
    return hour.index === this.time.viewHours;
  }

  isSelectedMinute(minute: { index: number; disabled: boolean }): boolean {
    return minute.index === this.time.minutes;
  }

  isSelectedSecond(second: { index: number; disabled: boolean }): boolean {
    return second.index === this.time.seconds;
  }

  isSelected12Hours(value: { index: number; value: string }): boolean {
    return value.value.toUpperCase() === this.time.selected12Hours;
  }

  ngOnInit(): void {
    this.time.changes
      .pipe(takeUntilDestroyed(this.destroyRef))
      .subscribe(() => {
        this.changed();
        this.touched();
        this.scrollToTime(120);
      });
    this.buildTimes();

    this.ngZone.runOutsideAngular(() => {
      setTimeout(() => {
        this.scrollToTime();
        this.firstScrolled = true;
      });
    });

    fromEventOutsideAngular(this.elementRef.nativeElement, "mousedown")
      .pipe(takeUntilDestroyed(this.destroyRef))
      .subscribe((event) => {
        event.preventDefault();
      });
  }

  ngOnChanges(changes: SimpleChanges): void {
    const { nzUse12Hours, nzDefaultOpenValue } = changes;
    if (!nzUse12Hours?.previousValue && nzUse12Hours?.currentValue) {
      this.build12Hours();
      this.enabledColumns++;
    }
    if (nzDefaultOpenValue?.currentValue) {
      this.time.setDefaultOpenValue(this.nzDefaultOpenValue || new Date());
    }
  }

  writeValue(value: Date): void {
    this.time.setValue(value, this.nzUse12Hours);
    this.buildTimes();

    if (value && this.firstScrolled) {
      this.scrollToTime(120);
    }
    // Mark this component to be checked manually with internal properties changing (see: https://github.com/angular/angular/issues/10816)
    this.cdr.markForCheck();
  }

  registerOnChange(fn: (value: Date) => void): void {
    this.onChange = fn;
  }

  registerOnTouched(fn: () => void): void {
    this.onTouch = fn;
  }
}

`````
