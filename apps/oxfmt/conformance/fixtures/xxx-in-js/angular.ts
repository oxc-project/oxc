// Angular @Component decorator - direct template and styles
// Uses Angular-specific syntax: interpolation, directives, bindings
@Component({
    selector: 'app-root',
    template: `
        <h1>{{    title    }}</h1>
        <div *ngIf="isVisible"    [class.active]="isActive"     (click)="onClick()">
            <span>{{ count     }}</span>
        </div>
        <ul><li *ngFor="let item of items">{{item.name}}</li></ul>
    `,
    styles: `h1 { color: blue }`
})
export class AppComponent1 {}

// Array form styles
@Component({
       selector: 'app-test',
  template: `<ul>   <li>test</li>
  </ul>
  `,
  styles: [   `

 :host {
   color: red;
 }
 div { background: blue
 }
`

]
})
class     TestComponent {}

// Computed properties - should NOT be formatted
const styles = "foobar";
const template = "foobar";

@Component({
    selector: 'app-computed',
    [template]: `<h1>{{       hello }}</h1>`,
    [styles]: `h1 { color: blue }`
})
export class AppComponent2 {}
