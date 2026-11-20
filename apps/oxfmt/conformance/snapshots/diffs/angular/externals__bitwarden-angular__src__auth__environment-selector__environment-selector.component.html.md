# externals/bitwarden-angular/src/auth/environment-selector/environment-selector.component.html

## Option 1

`````json
{"printWidth":80}
`````

### Diff

`````diff
===================================================================
--- prettier
+++ oxfmt
@@ -1,8 +1,10 @@
 <ng-container
-  *ngIf="{
-    selectedRegion: selectedRegion$ | async,
-  } as data"
+  *ngIf="
+    {
+      selectedRegion: selectedRegion$ | async,
+    } as data
+  "
 >
   <div class="tw-mb-1">
     <bit-menu #environmentOptions>
       <button

`````

### Actual (oxfmt)

`````html
<ng-container
  *ngIf="
    {
      selectedRegion: selectedRegion$ | async,
    } as data
  "
>
  <div class="tw-mb-1">
    <bit-menu #environmentOptions>
      <button
        *ngFor="let region of availableRegions$ | async; let i = index"
        bitMenuItem
        type="button"
        [attr.aria-pressed]="data.selectedRegion === region ? 'true' : 'false'"
        (click)="toggle(region.key)"
      >
        <bit-icon
          name="bwi-check"
          class="bwi-fw bwi-sm"
          style="padding-bottom: 1px"
          [style.visibility]="
            data.selectedRegion === region ? 'visible' : 'hidden'
          "
        ></bit-icon>
        <span>{{ region.domain }}</span>
      </button>
      <button
        bitMenuItem
        type="button"
        [attr.aria-pressed]="data.selectedRegion ? 'false' : 'true'"
        (click)="toggle(ServerEnvironmentType.SelfHosted)"
      >
        <bit-icon
          name="bwi-check"
          class="bwi-fw bwi-sm"
          style="padding-bottom: 1px"
          [style.visibility]="data.selectedRegion ? 'hidden' : 'visible'"
        ></bit-icon>
        <span>{{ "selfHostedServer" | i18n }}</span>
      </button>
    </bit-menu>
    <div bitTypography="body2">
      {{ "accessing" | i18n }}:
      <button [bitMenuTriggerFor]="environmentOptions" bitLink type="button">
        <b class="tw-text-primary-600 tw-font-medium">{{
          data.selectedRegion?.domain || ("selfHostedServer" | i18n)
        }}</b>
        <bit-icon name="bwi-angle-down" class="bwi-fw bwi-sm"></bit-icon>
      </button>
    </div>
  </div>
</ng-container>

`````

### Expected (prettier)

`````html
<ng-container
  *ngIf="{
    selectedRegion: selectedRegion$ | async,
  } as data"
>
  <div class="tw-mb-1">
    <bit-menu #environmentOptions>
      <button
        *ngFor="let region of availableRegions$ | async; let i = index"
        bitMenuItem
        type="button"
        [attr.aria-pressed]="data.selectedRegion === region ? 'true' : 'false'"
        (click)="toggle(region.key)"
      >
        <bit-icon
          name="bwi-check"
          class="bwi-fw bwi-sm"
          style="padding-bottom: 1px"
          [style.visibility]="
            data.selectedRegion === region ? 'visible' : 'hidden'
          "
        ></bit-icon>
        <span>{{ region.domain }}</span>
      </button>
      <button
        bitMenuItem
        type="button"
        [attr.aria-pressed]="data.selectedRegion ? 'false' : 'true'"
        (click)="toggle(ServerEnvironmentType.SelfHosted)"
      >
        <bit-icon
          name="bwi-check"
          class="bwi-fw bwi-sm"
          style="padding-bottom: 1px"
          [style.visibility]="data.selectedRegion ? 'hidden' : 'visible'"
        ></bit-icon>
        <span>{{ "selfHostedServer" | i18n }}</span>
      </button>
    </bit-menu>
    <div bitTypography="body2">
      {{ "accessing" | i18n }}:
      <button [bitMenuTriggerFor]="environmentOptions" bitLink type="button">
        <b class="tw-text-primary-600 tw-font-medium">{{
          data.selectedRegion?.domain || ("selfHostedServer" | i18n)
        }}</b>
        <bit-icon name="bwi-angle-down" class="bwi-fw bwi-sm"></bit-icon>
      </button>
    </div>
  </div>
</ng-container>

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
@@ -1,8 +1,10 @@
 <ng-container
-  *ngIf="{
-    selectedRegion: selectedRegion$ | async,
-  } as data"
+  *ngIf="
+    {
+      selectedRegion: selectedRegion$ | async,
+    } as data
+  "
 >
   <div class="tw-mb-1">
     <bit-menu #environmentOptions>
       <button

`````

### Actual (oxfmt)

`````html
<ng-container
  *ngIf="
    {
      selectedRegion: selectedRegion$ | async,
    } as data
  "
>
  <div class="tw-mb-1">
    <bit-menu #environmentOptions>
      <button
        *ngFor="let region of availableRegions$ | async; let i = index"
        bitMenuItem
        type="button"
        [attr.aria-pressed]="data.selectedRegion === region ? 'true' : 'false'"
        (click)="toggle(region.key)"
      >
        <bit-icon
          name="bwi-check"
          class="bwi-fw bwi-sm"
          style="padding-bottom: 1px"
          [style.visibility]="data.selectedRegion === region ? 'visible' : 'hidden'"
        ></bit-icon>
        <span>{{ region.domain }}</span>
      </button>
      <button
        bitMenuItem
        type="button"
        [attr.aria-pressed]="data.selectedRegion ? 'false' : 'true'"
        (click)="toggle(ServerEnvironmentType.SelfHosted)"
      >
        <bit-icon
          name="bwi-check"
          class="bwi-fw bwi-sm"
          style="padding-bottom: 1px"
          [style.visibility]="data.selectedRegion ? 'hidden' : 'visible'"
        ></bit-icon>
        <span>{{ "selfHostedServer" | i18n }}</span>
      </button>
    </bit-menu>
    <div bitTypography="body2">
      {{ "accessing" | i18n }}:
      <button [bitMenuTriggerFor]="environmentOptions" bitLink type="button">
        <b class="tw-text-primary-600 tw-font-medium">
          {{ data.selectedRegion?.domain || ("selfHostedServer" | i18n) }}
        </b>
        <bit-icon name="bwi-angle-down" class="bwi-fw bwi-sm"></bit-icon>
      </button>
    </div>
  </div>
</ng-container>

`````

### Expected (prettier)

`````html
<ng-container
  *ngIf="{
    selectedRegion: selectedRegion$ | async,
  } as data"
>
  <div class="tw-mb-1">
    <bit-menu #environmentOptions>
      <button
        *ngFor="let region of availableRegions$ | async; let i = index"
        bitMenuItem
        type="button"
        [attr.aria-pressed]="data.selectedRegion === region ? 'true' : 'false'"
        (click)="toggle(region.key)"
      >
        <bit-icon
          name="bwi-check"
          class="bwi-fw bwi-sm"
          style="padding-bottom: 1px"
          [style.visibility]="data.selectedRegion === region ? 'visible' : 'hidden'"
        ></bit-icon>
        <span>{{ region.domain }}</span>
      </button>
      <button
        bitMenuItem
        type="button"
        [attr.aria-pressed]="data.selectedRegion ? 'false' : 'true'"
        (click)="toggle(ServerEnvironmentType.SelfHosted)"
      >
        <bit-icon
          name="bwi-check"
          class="bwi-fw bwi-sm"
          style="padding-bottom: 1px"
          [style.visibility]="data.selectedRegion ? 'hidden' : 'visible'"
        ></bit-icon>
        <span>{{ "selfHostedServer" | i18n }}</span>
      </button>
    </bit-menu>
    <div bitTypography="body2">
      {{ "accessing" | i18n }}:
      <button [bitMenuTriggerFor]="environmentOptions" bitLink type="button">
        <b class="tw-text-primary-600 tw-font-medium">
          {{ data.selectedRegion?.domain || ("selfHostedServer" | i18n) }}
        </b>
        <bit-icon name="bwi-angle-down" class="bwi-fw bwi-sm"></bit-icon>
      </button>
    </div>
  </div>
</ng-container>

`````
