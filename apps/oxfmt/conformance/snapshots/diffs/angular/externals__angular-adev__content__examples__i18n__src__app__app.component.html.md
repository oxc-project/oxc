# externals/angular-adev/content/examples/i18n/src/app/app.component.html

## Option 1

`````json
{"printWidth":80}
`````

### Diff

`````diff
===================================================================
--- prettier
+++ oxfmt
@@ -17,31 +17,26 @@
 <br />
 <button type="button" (click)="inc(1)">+</button>
 <button type="button" (click)="inc(-1)">-</button>
 <!--#docregion i18n-plural-->
-<span i18n
-  >Updated
+<span i18n>Updated
   {minutes, plural,
     =0 {just now}
     =1 {one minute ago}
     other {{{ minutes }} minutes ago}
-  }</span
->
+  }</span>
 <!--#enddocregion i18n-plural-->
 ({{ minutes }}) <br /><br />
 <button type="button" (click)="male()">&#9794;</button>
 <button type="button" (click)="female()">&#9792;</button>
 <button type="button" (click)="other()">&#9895;</button>
 <!--#docregion i18n-select-->
-<span i18n
-  >The author is
-  {gender, select, male {male} female {female} other {other}}</span
->
+<span i18n>The author is
+  {gender, select, male {male} female {female} other {other}}</span>
 <!--#enddocregion i18n-select-->
 <br /><br />
 <!--#docregion i18n-nested-->
-<span i18n
-  >Updated:
+<span i18n>Updated:
   {minutes, plural,
     =0 {just now}
     =1 {one minute ago}
     other {

`````

### Actual (oxfmt)

`````html
<!--#docregion-->
<h1
  i18n="User welcome|An introduction header for this sample@@introductionHeader"
>
  Hello i18n!
</h1>

<!--#docregion i18n-ng-container-->
<ng-container i18n>I don't output any element</ng-container>
<!--#enddocregion i18n-ng-container-->

<br />

<!--#docregion i18n-title-translate-->
<img [src]="logo" i18n-title title="Angular logo" alt="Angular logo" />
<!--#enddocregion i18n-title-translate-->
<br />
<button type="button" (click)="inc(1)">+</button>
<button type="button" (click)="inc(-1)">-</button>
<!--#docregion i18n-plural-->
<span i18n>Updated
  {minutes, plural,
    =0 {just now}
    =1 {one minute ago}
    other {{{ minutes }} minutes ago}
  }</span>
<!--#enddocregion i18n-plural-->
({{ minutes }}) <br /><br />
<button type="button" (click)="male()">&#9794;</button>
<button type="button" (click)="female()">&#9792;</button>
<button type="button" (click)="other()">&#9895;</button>
<!--#docregion i18n-select-->
<span i18n>The author is
  {gender, select, male {male} female {female} other {other}}</span>
<!--#enddocregion i18n-select-->
<br /><br />
<!--#docregion i18n-nested-->
<span i18n>Updated:
  {minutes, plural,
    =0 {just now}
    =1 {one minute ago}
    other {
      {{ minutes }} minutes ago by
      {gender, select, male {male} female {female} other {other}}
    }
  }
</span>
<!--#enddocregion i18n-nested-->
<br /><br />
<!--#docregion i18n-conditional-->
<button type="button" (click)="toggleDisplay()">Toggle</button>
<div i18n [attr.aria-label]="toggleAriaLabel()">{{ toggle() }}</div>
<!--#enddocregion i18n-conditional-->

`````

### Expected (prettier)

`````html
<!--#docregion-->
<h1
  i18n="User welcome|An introduction header for this sample@@introductionHeader"
>
  Hello i18n!
</h1>

<!--#docregion i18n-ng-container-->
<ng-container i18n>I don't output any element</ng-container>
<!--#enddocregion i18n-ng-container-->

<br />

<!--#docregion i18n-title-translate-->
<img [src]="logo" i18n-title title="Angular logo" alt="Angular logo" />
<!--#enddocregion i18n-title-translate-->
<br />
<button type="button" (click)="inc(1)">+</button>
<button type="button" (click)="inc(-1)">-</button>
<!--#docregion i18n-plural-->
<span i18n
  >Updated
  {minutes, plural,
    =0 {just now}
    =1 {one minute ago}
    other {{{ minutes }} minutes ago}
  }</span
>
<!--#enddocregion i18n-plural-->
({{ minutes }}) <br /><br />
<button type="button" (click)="male()">&#9794;</button>
<button type="button" (click)="female()">&#9792;</button>
<button type="button" (click)="other()">&#9895;</button>
<!--#docregion i18n-select-->
<span i18n
  >The author is
  {gender, select, male {male} female {female} other {other}}</span
>
<!--#enddocregion i18n-select-->
<br /><br />
<!--#docregion i18n-nested-->
<span i18n
  >Updated:
  {minutes, plural,
    =0 {just now}
    =1 {one minute ago}
    other {
      {{ minutes }} minutes ago by
      {gender, select, male {male} female {female} other {other}}
    }
  }
</span>
<!--#enddocregion i18n-nested-->
<br /><br />
<!--#docregion i18n-conditional-->
<button type="button" (click)="toggleDisplay()">Toggle</button>
<div i18n [attr.aria-label]="toggleAriaLabel()">{{ toggle() }}</div>
<!--#enddocregion i18n-conditional-->

`````
