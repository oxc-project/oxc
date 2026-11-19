# Input

`````html
<div *ngIf="cond; els - e other"></div>
<li *ngFor="let a of b; 'trackBy': f"></li>

`````

# Output

## `{}`

`````html
<div *ngIf="cond; els - e: other"></div>
<li *ngFor="let a of b; 'trackBy': f"></li>

`````
