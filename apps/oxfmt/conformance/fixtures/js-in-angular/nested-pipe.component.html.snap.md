# Input

`````html
<p [x]="f(x, {aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa: bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb | pppp})"></p>
<p>{{(birthday | date:'longDate') | uppercase}}</p>
<p [ngClass]="{'btn-success':(dialog$|async).level===1}"></p>
<p>{{[...a|p, ...(b|q)]}} {{f(...c|r)}} {{ {...d|s} }}</p>
<p [x]="(aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa | async)"></p>

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

`````
