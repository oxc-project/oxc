# externals/ng-zorro-antd/components/empty/empty.component.ts

## Option 1

`````json
{"printWidth":80}
`````

### Diff

`````diff
===================================================================
--- prettier
+++ oxfmt
@@ -25,9 +25,12 @@
 import { NzEmptySimpleComponent } from "./partial/simple";
 
 const NzEmptyDefaultImages = ["default", "simple"] as const;
 type NzEmptyNotFoundImageType =
-  (typeof NzEmptyDefaultImages)[number] | null | string | TemplateRef<void>;
+  | (typeof NzEmptyDefaultImages)[number]
+  | null
+  | string
+  | TemplateRef<void>;
 
 @Component({
   changeDetection: ChangeDetectionStrategy.OnPush,
   encapsulation: ViewEncapsulation.None,

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

import { NzOutletModule } from "ng-zorro-antd/core/outlet";
import { NzEmptyI18nInterface, NzI18nService } from "ng-zorro-antd/i18n";

import { NzEmptyDefaultComponent } from "./partial/default";
import { NzEmptySimpleComponent } from "./partial/simple";

const NzEmptyDefaultImages = ["default", "simple"] as const;
type NzEmptyNotFoundImageType =
  | (typeof NzEmptyDefaultImages)[number]
  | null
  | string
  | TemplateRef<void>;

@Component({
  changeDetection: ChangeDetectionStrategy.OnPush,
  encapsulation: ViewEncapsulation.None,
  selector: "nz-empty",
  exportAs: "nzEmpty",
  template: `
    <div class="ant-empty-image">
      @if (!isImageBuildIn) {
        <ng-container *nzStringTemplateOutlet="nzNotFoundImage">
          <img
            [src]="nzNotFoundImage"
            [alt]="isContentString ? nzNotFoundContent : 'empty'"
          />
        </ng-container>
      } @else {
        @if (nzNotFoundImage === "simple") {
          <nz-empty-simple />
        } @else {
          <nz-empty-default />
        }
      }
    </div>
    @if (nzNotFoundContent !== null) {
      <p class="ant-empty-description">
        <ng-container *nzStringTemplateOutlet="nzNotFoundContent">
          {{ isContentString ? nzNotFoundContent : locale["description"] }}
        </ng-container>
      </p>
    }

    @if (nzNotFoundFooter) {
      <div class="ant-empty-footer">
        <ng-container *nzStringTemplateOutlet="nzNotFoundFooter">
          {{ nzNotFoundFooter }}
        </ng-container>
      </div>
    }
  `,
  host: {
    class: "ant-empty",
  },
  imports: [NzOutletModule, NzEmptyDefaultComponent, NzEmptySimpleComponent],
})
export class NzEmptyComponent implements OnChanges, OnInit {
  private i18n = inject(NzI18nService);
  private cdr = inject(ChangeDetectorRef);
  private destroyRef = inject(DestroyRef);

  @Input() nzNotFoundImage: NzEmptyNotFoundImageType = "default";
  @Input() nzNotFoundContent?: string | TemplateRef<void> | null;
  @Input() nzNotFoundFooter?: string | TemplateRef<void>;

  isContentString = false;
  isImageBuildIn = true;
  locale!: NzEmptyI18nInterface;

  ngOnChanges(changes: SimpleChanges): void {
    const { nzNotFoundContent, nzNotFoundImage } = changes;

    if (nzNotFoundContent) {
      const content = nzNotFoundContent.currentValue;
      this.isContentString = typeof content === "string";
    }

    if (nzNotFoundImage) {
      const image = nzNotFoundImage.currentValue || "default";
      this.isImageBuildIn =
        NzEmptyDefaultImages.findIndex((i) => i === image) > -1;
    }
  }

  ngOnInit(): void {
    this.i18n.localeChange
      .pipe(takeUntilDestroyed(this.destroyRef))
      .subscribe(() => {
        this.locale = this.i18n.getLocaleData("Empty");
        this.cdr.markForCheck();
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

import {
  ChangeDetectionStrategy,
  ChangeDetectorRef,
  Component,
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

import { NzOutletModule } from "ng-zorro-antd/core/outlet";
import { NzEmptyI18nInterface, NzI18nService } from "ng-zorro-antd/i18n";

import { NzEmptyDefaultComponent } from "./partial/default";
import { NzEmptySimpleComponent } from "./partial/simple";

const NzEmptyDefaultImages = ["default", "simple"] as const;
type NzEmptyNotFoundImageType =
  (typeof NzEmptyDefaultImages)[number] | null | string | TemplateRef<void>;

@Component({
  changeDetection: ChangeDetectionStrategy.OnPush,
  encapsulation: ViewEncapsulation.None,
  selector: "nz-empty",
  exportAs: "nzEmpty",
  template: `
    <div class="ant-empty-image">
      @if (!isImageBuildIn) {
        <ng-container *nzStringTemplateOutlet="nzNotFoundImage">
          <img
            [src]="nzNotFoundImage"
            [alt]="isContentString ? nzNotFoundContent : 'empty'"
          />
        </ng-container>
      } @else {
        @if (nzNotFoundImage === "simple") {
          <nz-empty-simple />
        } @else {
          <nz-empty-default />
        }
      }
    </div>
    @if (nzNotFoundContent !== null) {
      <p class="ant-empty-description">
        <ng-container *nzStringTemplateOutlet="nzNotFoundContent">
          {{ isContentString ? nzNotFoundContent : locale["description"] }}
        </ng-container>
      </p>
    }

    @if (nzNotFoundFooter) {
      <div class="ant-empty-footer">
        <ng-container *nzStringTemplateOutlet="nzNotFoundFooter">
          {{ nzNotFoundFooter }}
        </ng-container>
      </div>
    }
  `,
  host: {
    class: "ant-empty",
  },
  imports: [NzOutletModule, NzEmptyDefaultComponent, NzEmptySimpleComponent],
})
export class NzEmptyComponent implements OnChanges, OnInit {
  private i18n = inject(NzI18nService);
  private cdr = inject(ChangeDetectorRef);
  private destroyRef = inject(DestroyRef);

  @Input() nzNotFoundImage: NzEmptyNotFoundImageType = "default";
  @Input() nzNotFoundContent?: string | TemplateRef<void> | null;
  @Input() nzNotFoundFooter?: string | TemplateRef<void>;

  isContentString = false;
  isImageBuildIn = true;
  locale!: NzEmptyI18nInterface;

  ngOnChanges(changes: SimpleChanges): void {
    const { nzNotFoundContent, nzNotFoundImage } = changes;

    if (nzNotFoundContent) {
      const content = nzNotFoundContent.currentValue;
      this.isContentString = typeof content === "string";
    }

    if (nzNotFoundImage) {
      const image = nzNotFoundImage.currentValue || "default";
      this.isImageBuildIn =
        NzEmptyDefaultImages.findIndex((i) => i === image) > -1;
    }
  }

  ngOnInit(): void {
    this.i18n.localeChange
      .pipe(takeUntilDestroyed(this.destroyRef))
      .subscribe(() => {
        this.locale = this.i18n.getLocaleData("Empty");
        this.cdr.markForCheck();
      });
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
@@ -25,9 +25,12 @@
 import { NzEmptySimpleComponent } from "./partial/simple";
 
 const NzEmptyDefaultImages = ["default", "simple"] as const;
 type NzEmptyNotFoundImageType =
-  (typeof NzEmptyDefaultImages)[number] | null | string | TemplateRef<void>;
+  | (typeof NzEmptyDefaultImages)[number]
+  | null
+  | string
+  | TemplateRef<void>;
 
 @Component({
   changeDetection: ChangeDetectionStrategy.OnPush,
   encapsulation: ViewEncapsulation.None,

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

import { NzOutletModule } from "ng-zorro-antd/core/outlet";
import { NzEmptyI18nInterface, NzI18nService } from "ng-zorro-antd/i18n";

import { NzEmptyDefaultComponent } from "./partial/default";
import { NzEmptySimpleComponent } from "./partial/simple";

const NzEmptyDefaultImages = ["default", "simple"] as const;
type NzEmptyNotFoundImageType =
  | (typeof NzEmptyDefaultImages)[number]
  | null
  | string
  | TemplateRef<void>;

@Component({
  changeDetection: ChangeDetectionStrategy.OnPush,
  encapsulation: ViewEncapsulation.None,
  selector: "nz-empty",
  exportAs: "nzEmpty",
  template: `
    <div class="ant-empty-image">
      @if (!isImageBuildIn) {
        <ng-container *nzStringTemplateOutlet="nzNotFoundImage">
          <img [src]="nzNotFoundImage" [alt]="isContentString ? nzNotFoundContent : 'empty'" />
        </ng-container>
      } @else {
        @if (nzNotFoundImage === "simple") {
          <nz-empty-simple />
        } @else {
          <nz-empty-default />
        }
      }
    </div>
    @if (nzNotFoundContent !== null) {
      <p class="ant-empty-description">
        <ng-container *nzStringTemplateOutlet="nzNotFoundContent">
          {{ isContentString ? nzNotFoundContent : locale["description"] }}
        </ng-container>
      </p>
    }

    @if (nzNotFoundFooter) {
      <div class="ant-empty-footer">
        <ng-container *nzStringTemplateOutlet="nzNotFoundFooter">
          {{ nzNotFoundFooter }}
        </ng-container>
      </div>
    }
  `,
  host: {
    class: "ant-empty",
  },
  imports: [NzOutletModule, NzEmptyDefaultComponent, NzEmptySimpleComponent],
})
export class NzEmptyComponent implements OnChanges, OnInit {
  private i18n = inject(NzI18nService);
  private cdr = inject(ChangeDetectorRef);
  private destroyRef = inject(DestroyRef);

  @Input() nzNotFoundImage: NzEmptyNotFoundImageType = "default";
  @Input() nzNotFoundContent?: string | TemplateRef<void> | null;
  @Input() nzNotFoundFooter?: string | TemplateRef<void>;

  isContentString = false;
  isImageBuildIn = true;
  locale!: NzEmptyI18nInterface;

  ngOnChanges(changes: SimpleChanges): void {
    const { nzNotFoundContent, nzNotFoundImage } = changes;

    if (nzNotFoundContent) {
      const content = nzNotFoundContent.currentValue;
      this.isContentString = typeof content === "string";
    }

    if (nzNotFoundImage) {
      const image = nzNotFoundImage.currentValue || "default";
      this.isImageBuildIn = NzEmptyDefaultImages.findIndex((i) => i === image) > -1;
    }
  }

  ngOnInit(): void {
    this.i18n.localeChange.pipe(takeUntilDestroyed(this.destroyRef)).subscribe(() => {
      this.locale = this.i18n.getLocaleData("Empty");
      this.cdr.markForCheck();
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

import {
  ChangeDetectionStrategy,
  ChangeDetectorRef,
  Component,
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

import { NzOutletModule } from "ng-zorro-antd/core/outlet";
import { NzEmptyI18nInterface, NzI18nService } from "ng-zorro-antd/i18n";

import { NzEmptyDefaultComponent } from "./partial/default";
import { NzEmptySimpleComponent } from "./partial/simple";

const NzEmptyDefaultImages = ["default", "simple"] as const;
type NzEmptyNotFoundImageType =
  (typeof NzEmptyDefaultImages)[number] | null | string | TemplateRef<void>;

@Component({
  changeDetection: ChangeDetectionStrategy.OnPush,
  encapsulation: ViewEncapsulation.None,
  selector: "nz-empty",
  exportAs: "nzEmpty",
  template: `
    <div class="ant-empty-image">
      @if (!isImageBuildIn) {
        <ng-container *nzStringTemplateOutlet="nzNotFoundImage">
          <img [src]="nzNotFoundImage" [alt]="isContentString ? nzNotFoundContent : 'empty'" />
        </ng-container>
      } @else {
        @if (nzNotFoundImage === "simple") {
          <nz-empty-simple />
        } @else {
          <nz-empty-default />
        }
      }
    </div>
    @if (nzNotFoundContent !== null) {
      <p class="ant-empty-description">
        <ng-container *nzStringTemplateOutlet="nzNotFoundContent">
          {{ isContentString ? nzNotFoundContent : locale["description"] }}
        </ng-container>
      </p>
    }

    @if (nzNotFoundFooter) {
      <div class="ant-empty-footer">
        <ng-container *nzStringTemplateOutlet="nzNotFoundFooter">
          {{ nzNotFoundFooter }}
        </ng-container>
      </div>
    }
  `,
  host: {
    class: "ant-empty",
  },
  imports: [NzOutletModule, NzEmptyDefaultComponent, NzEmptySimpleComponent],
})
export class NzEmptyComponent implements OnChanges, OnInit {
  private i18n = inject(NzI18nService);
  private cdr = inject(ChangeDetectorRef);
  private destroyRef = inject(DestroyRef);

  @Input() nzNotFoundImage: NzEmptyNotFoundImageType = "default";
  @Input() nzNotFoundContent?: string | TemplateRef<void> | null;
  @Input() nzNotFoundFooter?: string | TemplateRef<void>;

  isContentString = false;
  isImageBuildIn = true;
  locale!: NzEmptyI18nInterface;

  ngOnChanges(changes: SimpleChanges): void {
    const { nzNotFoundContent, nzNotFoundImage } = changes;

    if (nzNotFoundContent) {
      const content = nzNotFoundContent.currentValue;
      this.isContentString = typeof content === "string";
    }

    if (nzNotFoundImage) {
      const image = nzNotFoundImage.currentValue || "default";
      this.isImageBuildIn = NzEmptyDefaultImages.findIndex((i) => i === image) > -1;
    }
  }

  ngOnInit(): void {
    this.i18n.localeChange.pipe(takeUntilDestroyed(this.destroyRef)).subscribe(() => {
      this.locale = this.i18n.getLocaleData("Empty");
      this.cdr.markForCheck();
    });
  }
}

`````
