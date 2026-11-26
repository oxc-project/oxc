# Input

`````html
<p [x]="f(x, {aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa: bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb | pppp})"></p>
<p>{{(birthday | date:'longDate') | uppercase}}</p>
<p [ngClass]="{'btn-success':(dialog$|async).level===1}"></p>
<p>{{[...a|p, ...(b|q)]}} {{f(...c|r)}} {{ {...d|s} }}</p>
<p [x]="(aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa | async)"></p>
<p>{{ f(a | p, (x) => x) }}</p>
<p>{{ someFunction(aaaaaaaaaaaaaaaaaaaaaa | async, (xxxxxxxxx) => xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx) }}</p>
<p [x]="conditionnnnnnnnnnnnnnnnnnnnnnnnnnnnn ? (bbbbbbbbbbbbbbbbbbbbbbbbbbb | async) : cccccccccccccccccccccccc"></p>
<p>{{ (aaaaaaaaaaaaaaa | async).bbbbbbbbbbbbbbbbb.cccccccccccccccccc().dddddddddddddddddd().eeeeeeeeeeeeeee() }}</p>
<p (click)="value = (aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa | p)"></p>
<p [x]="fn(aaaaaaaaaaaaaaaa | async, {kkkkkkkkkkkkkkkkkkkkk: vvvvvvvvvvvvvvvvvvvvvv, jjjjjjjjjjjjjjjjj: (w | p)})"></p>
<p [x]="(aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa | someVeryLongPipeName: argumentNumberOne : argumentNumberTwo).level"></p>
<p [x]="(aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa | someVeryLongPipeName: argumentNumberOne : argumentNumberTwo)[0]"></p>
<p>{{ "x" | i18n: (aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa | someVeryLongPipeName: argumentNumberOne : argumentNumberTwo) }}</p>
<p [x]="c ? (aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa | someVeryLongPipeName: argumentNumberOne : argumentNumberTwo) : d"></p>
<p [x]="f(c ? (aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa | someVeryLongPipeName: argumentNumberOne : argumentNumberTwo) : d)"></p>

`````

# Output

## `{}`

`````html
<p
  [x]="
    f(x, {
      aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa: bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb
        | pppp,
    })
  "
></p>
<p>{{ birthday | date: "longDate" | uppercase }}</p>
<p [ngClass]="{ 'btn-success': (dialog$ | async).level === 1 }"></p>
<p>{{ [...(a | p), ...(b | q)] }} {{ f(...(c | r)) }} {{ { ...(d | s) } }}</p>
<p
  [x]="
    aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa
      | async
  "
></p>
<p>{{ f(a | p, (x) => x) }}</p>
<p>
  {{
    someFunction(
      aaaaaaaaaaaaaaaaaaaaaa | async,
      (xxxxxxxxx) => xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx
    )
  }}
</p>
<p
  [x]="
    conditionnnnnnnnnnnnnnnnnnnnnnnnnnnnn
      ? (bbbbbbbbbbbbbbbbbbbbbbbbbbb | async)
      : cccccccccccccccccccccccc
  "
></p>
<p>
  {{
    (aaaaaaaaaaaaaaa | async).bbbbbbbbbbbbbbbbb
      .cccccccccccccccccc()
      .dddddddddddddddddd()
      .eeeeeeeeeeeeeee()
  }}
</p>
<p (click)="value = (aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa | p)"></p>
<p
  [x]="
    fn(aaaaaaaaaaaaaaaa | async, {
      kkkkkkkkkkkkkkkkkkkkk: vvvvvvvvvvvvvvvvvvvvvv,
      jjjjjjjjjjjjjjjjj: (w | p),
    })
  "
></p>
<p
  [x]="
    (
      aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa
      | someVeryLongPipeName: argumentNumberOne : argumentNumberTwo
    ).level
  "
></p>
<p
  [x]="
    (aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa
      | someVeryLongPipeName: argumentNumberOne : argumentNumberTwo)[0]
  "
></p>
<p>
  {{
    "x"
      | i18n
        : (aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa
            | someVeryLongPipeName: argumentNumberOne : argumentNumberTwo)
  }}
</p>
<p
  [x]="
    c
      ? (aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa
        | someVeryLongPipeName: argumentNumberOne : argumentNumberTwo)
      : d
  "
></p>
<p
  [x]="
    f(
      c
        ? (aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa
            | someVeryLongPipeName: argumentNumberOne : argumentNumberTwo)
        : d
    )
  "
></p>

`````
