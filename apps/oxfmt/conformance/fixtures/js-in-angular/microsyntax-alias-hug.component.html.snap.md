# Input

`````html
<ng-container *ngIf="{aaaaaaaaaaaaaaaaaaaa: bbbbbbbbbbbbbbbbbbbb, cccccccccccccccccc: ddddddddddddddddd} as data"></ng-container>
<ng-container *ngIf="[aaaaaaaaaaaaaaaaaaaaaaaa, bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb, cccccccccccccccccccccc] as data"></ng-container>
<ng-container *appLet="{selectedRegion: selectedRegion$ | async, aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa: bbbbbbbbbbbbbbbbbbbbbbbbbbbbb} as vm"></ng-container>
<ng-container *ngIf="{a: b}   as   data"></ng-container>
<ng-container *ngIf="'aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa' as data"></ng-container>
<ng-container *ngIf="aaaaaaaaaaaaaaaaaaaaaaaaaaaaaa.bbbbbbbbbbbbbbbbbbbbbbbbbbbbbb.cccccccccccccccccccccccccccccc as data"></ng-container>
<ng-container *ngIf="aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa as data"></ng-container>
<ng-container *ngIf="{aaaaaaaaaaaaaaaaaaaa: bbbbbbbbbbbbbbbbbbbb, cccccccccccccccccc: ddddddddddddddddd} | async as data"></ng-container>
<ng-container *ngIf="({aaaaaaaaaaaaaaaaaaaa: bbbbbbbbbbbbbbbbbbbb, cccccccccccccccccc: ddddddddddddddddd} | async) as data"></ng-container>
<ng-container *ngIf="{aaaaaaaaaaaaaaaaaaaa: bbbbbbbbbbbbbbbbbbbb, cccccccccccccccccc: ddddddddddddddddd} as data; else loading"></ng-container>
<ng-container *ngIf="x as data"></ng-container>

`````

# Output

## `{}`

`````html
<ng-container
  *ngIf="{
    aaaaaaaaaaaaaaaaaaaa: bbbbbbbbbbbbbbbbbbbb,
    cccccccccccccccccc: ddddddddddddddddd,
  } as data"
></ng-container>
<ng-container
  *ngIf="[
    aaaaaaaaaaaaaaaaaaaaaaaa,
    bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb,
    cccccccccccccccccccccc,
  ] as data"
></ng-container>
<ng-container
  *appLet="{
    selectedRegion: selectedRegion$ | async,
    aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa: bbbbbbbbbbbbbbbbbbbbbbbbbbbbb,
  } as vm"
></ng-container>
<ng-container *ngIf="{ a: b } as data"></ng-container>
<ng-container
  *ngIf="'aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa' as data"
></ng-container>
<ng-container
  *ngIf="
    aaaaaaaaaaaaaaaaaaaaaaaaaaaaaa.bbbbbbbbbbbbbbbbbbbbbbbbbbbbbb
      .cccccccccccccccccccccccccccccc as data
  "
></ng-container>
<ng-container
  *ngIf="
    aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa as data
  "
></ng-container>
<ng-container
  *ngIf="
    { aaaaaaaaaaaaaaaaaaaa: bbbbbbbbbbbbbbbbbbbb, cccccccccccccccccc: ddddddddddddddddd }
      | async as data
  "
></ng-container>
<ng-container
  *ngIf="
    { aaaaaaaaaaaaaaaaaaaa: bbbbbbbbbbbbbbbbbbbb, cccccccccccccccccc: ddddddddddddddddd }
      | async as data
  "
></ng-container>
<ng-container
  *ngIf="
    { aaaaaaaaaaaaaaaaaaaa: bbbbbbbbbbbbbbbbbbbb, cccccccccccccccccc: ddddddddddddddddd } as data;
    else loading
  "
></ng-container>
<ng-container *ngIf="x as data"></ng-container>

`````
