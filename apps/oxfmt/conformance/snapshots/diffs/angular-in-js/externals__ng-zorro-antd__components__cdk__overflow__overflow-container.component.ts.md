# externals/ng-zorro-antd/components/cdk/overflow/overflow-container.component.ts

## Option 1

`````json
{"printWidth":80}
`````

### Diff

`````diff
===================================================================
--- prettier
+++ oxfmt
@@ -60,9 +60,10 @@
   overflowItems: QueryList<NzOverflowItemDirective> | undefined = undefined;
   @ContentChild(NzOverflowSuffixDirective)
   overflowSuffix: NzOverflowSuffixDirective | undefined = undefined;
   @ContentChild(NzOverflowRestDirective) overflowRest:
-    NzOverflowRestDirective | undefined = undefined;
+    | NzOverflowRestDirective
+    | undefined = undefined;
   overflowItems$ = new ReplaySubject<QueryList<NzOverflowItemDirective>>(1);
 
   containerWidth$ = this.nzResizeObserver
     .observe(this.elementRef.nativeElement)

`````

### Actual (oxfmt)

`````ts
/**
 * Use of this source code is governed by an MIT-style license that can be
 * found in the LICENSE file at https://github.com/NG-ZORRO/ng-zorro-antd/blob/master/LICENSE
 */

import {
  Component,
  ChangeDetectionStrategy,
  ContentChildren,
  QueryList,
  ElementRef,
  OnInit,
  AfterContentInit,
  ContentChild,
  ChangeDetectorRef,
  inject,
  DestroyRef,
} from "@angular/core";
import { takeUntilDestroyed } from "@angular/core/rxjs-interop";
import {
  BehaviorSubject,
  combineLatest,
  Observable,
  ReplaySubject,
  Subject,
} from "rxjs";
import {
  filter,
  map,
  pairwise,
  startWith,
  switchMap,
  withLatestFrom,
} from "rxjs/operators";

import { NzResizeObserver } from "ng-zorro-antd/cdk/resize-observer";

import { NzOverflowItemDirective } from "./overflow-item.directive";
import { NzOverflowRestDirective } from "./overflow-rest.directive";
import { NzOverflowSuffixDirective } from "./overflow-suffix.directive";

@Component({
  selector: "nz-overflow-container",
  template: `
    <ng-content />
    <ng-content select="[appOverflowRest]" />
    <ng-content select="[appOverflowSuffix]" />
  `,
  providers: [NzResizeObserver],
  changeDetection: ChangeDetectionStrategy.OnPush,
})
export class NzOverflowContainerComponent implements OnInit, AfterContentInit {
  private nzResizeObserver = inject(NzResizeObserver);
  private elementRef = inject(ElementRef);
  private cdr = inject(ChangeDetectorRef);
  private destroyRef = inject(DestroyRef);

  contentInit$ = new Subject<void>();
  @ContentChildren(NzOverflowItemDirective)
  overflowItems: QueryList<NzOverflowItemDirective> | undefined = undefined;
  @ContentChild(NzOverflowSuffixDirective)
  overflowSuffix: NzOverflowSuffixDirective | undefined = undefined;
  @ContentChild(NzOverflowRestDirective) overflowRest:
    | NzOverflowRestDirective
    | undefined = undefined;
  overflowItems$ = new ReplaySubject<QueryList<NzOverflowItemDirective>>(1);

  containerWidth$ = this.nzResizeObserver
    .observe(this.elementRef.nativeElement)
    .pipe(map(([item]) => item.target.clientWidth || 0));
  restWidth$ = new BehaviorSubject<number>(0);
  suffixWidth$ = new BehaviorSubject<number>(0);
  suffixFixedStart$ = new BehaviorSubject<number | null>(null);
  displayCount$ = new BehaviorSubject<number>(Number.MAX_SAFE_INTEGER);
  restReady$ = new BehaviorSubject<boolean>(false);
  maxRestWith$ = this.restWidth$.pipe(
    pairwise(),
    map(([prevRestWidth, restWidth]) => Math.max(prevRestWidth, restWidth)),
  );
  omittedItems$ = combineLatest([this.overflowItems$, this.displayCount$]).pipe(
    withLatestFrom(this.contentInit$),
    map(([[overflowItems, displayCount]]) =>
      overflowItems.toArray().slice(displayCount + 1),
    ),
  );
  displayRest$ = combineLatest([this.restReady$, this.omittedItems$]).pipe(
    map(([restReady, omittedItems]) => restReady && !!omittedItems.length),
  );

  updateDisplayCount(count: number, notReady?: boolean): void {
    this.displayCount$.next(count);
    if (this.overflowItems && !notReady) {
      this.restReady$.next(count < this.overflowItems.length - 1);
    }
  }

  ngOnInit(): void {
    const overflowItemsWidth$ = this.overflowItems$.pipe(
      switchMap((items) => combineLatest(items.map((item) => item.itemWidth$))),
    ) as Observable<number[]>;
    this.overflowItems$
      .pipe(takeUntilDestroyed(this.destroyRef))
      .subscribe((overflowItems) => {
        if (!overflowItems.length) {
          this.displayCount$.next(0);
          this.suffixFixedStart$.next(null);
        }
      });
    combineLatest([
      overflowItemsWidth$,
      this.containerWidth$,
      this.maxRestWith$,
      this.restWidth$,
      this.suffixWidth$,
    ])
      .pipe(
        filter(
          ([, containerWidth, maxRestWith]) =>
            !!(containerWidth && maxRestWith),
        ),
        takeUntilDestroyed(this.destroyRef),
      )
      .subscribe(
        ([
          overflowItemsWidth,
          containerWidth,
          maxRestWith,
          restWidth,
          suffixWidth,
        ]) => {
          let totalWidth = suffixWidth;
          const len = overflowItemsWidth.length;
          const lastIndex = len - 1;
          for (let i = 0; i < len; i += 1) {
            const currentItemWidth = overflowItemsWidth[i];
            // Break since data not ready
            if (currentItemWidth === undefined) {
              this.updateDisplayCount(i - 1, true);
              break;
            } else {
              // Find best match
              totalWidth += currentItemWidth;

              if (
                // Only one means `totalWidth` is the final width
                (lastIndex === 0 && totalWidth <= containerWidth) ||
                // Last two width will be the final width
                (i === lastIndex - 1 &&
                  overflowItemsWidth[lastIndex] !== undefined &&
                  totalWidth + overflowItemsWidth[lastIndex]! <= containerWidth)
              ) {
                // Additional check if match the end
                this.updateDisplayCount(lastIndex);
                this.suffixFixedStart$.next(null);
                break;
              } else if (totalWidth + maxRestWith > containerWidth) {
                // Can not hold all the content to show rest
                this.updateDisplayCount(i - 1);
                this.suffixFixedStart$.next(
                  totalWidth - currentItemWidth - suffixWidth + restWidth,
                );
                break;
              }
              this.cdr.detectChanges();
            }
          }
          if (
            this.overflowSuffix &&
            overflowItemsWidth[0] !== undefined &&
            overflowItemsWidth[0] + suffixWidth > containerWidth
          ) {
            this.suffixFixedStart$.next(null);
          }

          this.cdr.detectChanges();
        },
      );
    combineLatest([this.suffixFixedStart$, this.displayCount$])
      .pipe(takeUntilDestroyed(this.destroyRef))
      .subscribe(([suffixFixedStart, displayCount]) => {
        this.overflowSuffix?.setSuffixStyle(suffixFixedStart, displayCount);
      });
    combineLatest([this.displayCount$, this.overflowItems$])
      .pipe(takeUntilDestroyed(this.destroyRef))
      .subscribe(([displayCount, overflowItems]) =>
        overflowItems.forEach((item, index) =>
          item.setItemStyle(index <= displayCount, index),
        ),
      );
    combineLatest([this.displayRest$, this.displayCount$])
      .pipe(takeUntilDestroyed(this.destroyRef))
      .subscribe(([displayRest, displayCount]) => {
        this.overflowRest?.setRestStyle(
          displayRest,
          displayRest ? displayCount : Number.MAX_SAFE_INTEGER,
        );
      });
  }
  ngAfterContentInit(): void {
    this.overflowItems?.changes
      .pipe(startWith(this.overflowItems))
      .subscribe(this.overflowItems$);
    this.overflowSuffix?.suffixWidth$
      .pipe(takeUntilDestroyed(this.destroyRef))
      .subscribe(this.suffixWidth$);
    this.overflowRest?.restWidth$
      .pipe(takeUntilDestroyed(this.destroyRef))
      .subscribe(this.restWidth$);
    this.contentInit$.next();
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
  Component,
  ChangeDetectionStrategy,
  ContentChildren,
  QueryList,
  ElementRef,
  OnInit,
  AfterContentInit,
  ContentChild,
  ChangeDetectorRef,
  inject,
  DestroyRef,
} from "@angular/core";
import { takeUntilDestroyed } from "@angular/core/rxjs-interop";
import {
  BehaviorSubject,
  combineLatest,
  Observable,
  ReplaySubject,
  Subject,
} from "rxjs";
import {
  filter,
  map,
  pairwise,
  startWith,
  switchMap,
  withLatestFrom,
} from "rxjs/operators";

import { NzResizeObserver } from "ng-zorro-antd/cdk/resize-observer";

import { NzOverflowItemDirective } from "./overflow-item.directive";
import { NzOverflowRestDirective } from "./overflow-rest.directive";
import { NzOverflowSuffixDirective } from "./overflow-suffix.directive";

@Component({
  selector: "nz-overflow-container",
  template: `
    <ng-content />
    <ng-content select="[appOverflowRest]" />
    <ng-content select="[appOverflowSuffix]" />
  `,
  providers: [NzResizeObserver],
  changeDetection: ChangeDetectionStrategy.OnPush,
})
export class NzOverflowContainerComponent implements OnInit, AfterContentInit {
  private nzResizeObserver = inject(NzResizeObserver);
  private elementRef = inject(ElementRef);
  private cdr = inject(ChangeDetectorRef);
  private destroyRef = inject(DestroyRef);

  contentInit$ = new Subject<void>();
  @ContentChildren(NzOverflowItemDirective)
  overflowItems: QueryList<NzOverflowItemDirective> | undefined = undefined;
  @ContentChild(NzOverflowSuffixDirective)
  overflowSuffix: NzOverflowSuffixDirective | undefined = undefined;
  @ContentChild(NzOverflowRestDirective) overflowRest:
    NzOverflowRestDirective | undefined = undefined;
  overflowItems$ = new ReplaySubject<QueryList<NzOverflowItemDirective>>(1);

  containerWidth$ = this.nzResizeObserver
    .observe(this.elementRef.nativeElement)
    .pipe(map(([item]) => item.target.clientWidth || 0));
  restWidth$ = new BehaviorSubject<number>(0);
  suffixWidth$ = new BehaviorSubject<number>(0);
  suffixFixedStart$ = new BehaviorSubject<number | null>(null);
  displayCount$ = new BehaviorSubject<number>(Number.MAX_SAFE_INTEGER);
  restReady$ = new BehaviorSubject<boolean>(false);
  maxRestWith$ = this.restWidth$.pipe(
    pairwise(),
    map(([prevRestWidth, restWidth]) => Math.max(prevRestWidth, restWidth)),
  );
  omittedItems$ = combineLatest([this.overflowItems$, this.displayCount$]).pipe(
    withLatestFrom(this.contentInit$),
    map(([[overflowItems, displayCount]]) =>
      overflowItems.toArray().slice(displayCount + 1),
    ),
  );
  displayRest$ = combineLatest([this.restReady$, this.omittedItems$]).pipe(
    map(([restReady, omittedItems]) => restReady && !!omittedItems.length),
  );

  updateDisplayCount(count: number, notReady?: boolean): void {
    this.displayCount$.next(count);
    if (this.overflowItems && !notReady) {
      this.restReady$.next(count < this.overflowItems.length - 1);
    }
  }

  ngOnInit(): void {
    const overflowItemsWidth$ = this.overflowItems$.pipe(
      switchMap((items) => combineLatest(items.map((item) => item.itemWidth$))),
    ) as Observable<number[]>;
    this.overflowItems$
      .pipe(takeUntilDestroyed(this.destroyRef))
      .subscribe((overflowItems) => {
        if (!overflowItems.length) {
          this.displayCount$.next(0);
          this.suffixFixedStart$.next(null);
        }
      });
    combineLatest([
      overflowItemsWidth$,
      this.containerWidth$,
      this.maxRestWith$,
      this.restWidth$,
      this.suffixWidth$,
    ])
      .pipe(
        filter(
          ([, containerWidth, maxRestWith]) =>
            !!(containerWidth && maxRestWith),
        ),
        takeUntilDestroyed(this.destroyRef),
      )
      .subscribe(
        ([
          overflowItemsWidth,
          containerWidth,
          maxRestWith,
          restWidth,
          suffixWidth,
        ]) => {
          let totalWidth = suffixWidth;
          const len = overflowItemsWidth.length;
          const lastIndex = len - 1;
          for (let i = 0; i < len; i += 1) {
            const currentItemWidth = overflowItemsWidth[i];
            // Break since data not ready
            if (currentItemWidth === undefined) {
              this.updateDisplayCount(i - 1, true);
              break;
            } else {
              // Find best match
              totalWidth += currentItemWidth;

              if (
                // Only one means `totalWidth` is the final width
                (lastIndex === 0 && totalWidth <= containerWidth) ||
                // Last two width will be the final width
                (i === lastIndex - 1 &&
                  overflowItemsWidth[lastIndex] !== undefined &&
                  totalWidth + overflowItemsWidth[lastIndex]! <= containerWidth)
              ) {
                // Additional check if match the end
                this.updateDisplayCount(lastIndex);
                this.suffixFixedStart$.next(null);
                break;
              } else if (totalWidth + maxRestWith > containerWidth) {
                // Can not hold all the content to show rest
                this.updateDisplayCount(i - 1);
                this.suffixFixedStart$.next(
                  totalWidth - currentItemWidth - suffixWidth + restWidth,
                );
                break;
              }
              this.cdr.detectChanges();
            }
          }
          if (
            this.overflowSuffix &&
            overflowItemsWidth[0] !== undefined &&
            overflowItemsWidth[0] + suffixWidth > containerWidth
          ) {
            this.suffixFixedStart$.next(null);
          }

          this.cdr.detectChanges();
        },
      );
    combineLatest([this.suffixFixedStart$, this.displayCount$])
      .pipe(takeUntilDestroyed(this.destroyRef))
      .subscribe(([suffixFixedStart, displayCount]) => {
        this.overflowSuffix?.setSuffixStyle(suffixFixedStart, displayCount);
      });
    combineLatest([this.displayCount$, this.overflowItems$])
      .pipe(takeUntilDestroyed(this.destroyRef))
      .subscribe(([displayCount, overflowItems]) =>
        overflowItems.forEach((item, index) =>
          item.setItemStyle(index <= displayCount, index),
        ),
      );
    combineLatest([this.displayRest$, this.displayCount$])
      .pipe(takeUntilDestroyed(this.destroyRef))
      .subscribe(([displayRest, displayCount]) => {
        this.overflowRest?.setRestStyle(
          displayRest,
          displayRest ? displayCount : Number.MAX_SAFE_INTEGER,
        );
      });
  }
  ngAfterContentInit(): void {
    this.overflowItems?.changes
      .pipe(startWith(this.overflowItems))
      .subscribe(this.overflowItems$);
    this.overflowSuffix?.suffixWidth$
      .pipe(takeUntilDestroyed(this.destroyRef))
      .subscribe(this.suffixWidth$);
    this.overflowRest?.restWidth$
      .pipe(takeUntilDestroyed(this.destroyRef))
      .subscribe(this.restWidth$);
    this.contentInit$.next();
  }
}

`````
