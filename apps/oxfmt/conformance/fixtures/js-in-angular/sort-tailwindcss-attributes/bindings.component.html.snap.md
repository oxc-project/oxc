# Input

`````html
<div class="p-4 flex" [class]="'p-4 flex'" [ngClass]="{ 'p-4 flex': a }" foo="p-4 flex" [bar]="'p-4 flex'" bind-class="'p-4 flex'"></div>
<div [(class)]="'p-4 flex'" [class.x]="'p-4 flex'" ngClass="p-4 flex" class="p-4 flex {{ a }}"></div>
<div [class]="cn('p-4 flex')" [ngClass]="('p-4 flex' | async)"></div>

`````

# Output

## `{"sortTailwindcss":{"attributes":["foo","[bar]"]}}`

`````html
<div
  class="flex p-4"
  [class]="'flex p-4'"
  [ngClass]="{ 'flex p-4': a }"
  foo="flex p-4"
  [bar]="'flex p-4'"
  bind-class="'flex p-4'"
></div>
<div
  [(class)]="'p-4 flex'"
  [class.x]="'p-4 flex'"
  ngClass="p-4 flex"
  class="p-4 flex {{ a }}"
></div>
<div [class]="cn('p-4 flex')" [ngClass]="'flex p-4' | async"></div>

`````
