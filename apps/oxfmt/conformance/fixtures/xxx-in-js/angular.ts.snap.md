# Input

`````ts
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

`````

# Output

## `{}`

`````ts
// Angular @Component decorator - direct template and styles
// Uses Angular-specific syntax: interpolation, directives, bindings
@Component({
  selector: "app-root",
  template: `
    <h1>{{ title }}</h1>
    <div *ngIf="isVisible" [class.active]="isActive" (click)="onClick()">
      <span>{{ count }}</span>
    </div>
    <ul>
      <li *ngFor="let item of items">{{ item.name }}</li>
    </ul>
  `,
  styles: `
    h1 {
      color: blue;
    }
  `,
})
export class AppComponent1 {}

// Array form styles
@Component({
  selector: "app-test",
  template: `<ul>
    <li>test</li>
  </ul> `,
  styles: [
    `
      :host {
        color: red;
      }
      div {
        background: blue;
      }
    `,
  ],
})
class TestComponent {}

// Computed properties - should NOT be formatted
const styles = "foobar";
const template = "foobar";

@Component({
  selector: "app-computed",
  [template]: `<h1>{{       hello }}</h1>`,
  [styles]: `h1 { color: blue }`,
})
export class AppComponent2 {}

`````

## `{"embeddedLanguageFormatting":"off"}`

`````ts
// Angular @Component decorator - direct template and styles
// Uses Angular-specific syntax: interpolation, directives, bindings
@Component({
  selector: "app-root",
  template: `
        <h1>{{    title    }}</h1>
        <div *ngIf="isVisible"    [class.active]="isActive"     (click)="onClick()">
            <span>{{ count     }}</span>
        </div>
        <ul><li *ngFor="let item of items">{{item.name}}</li></ul>
    `,
  styles: `h1 { color: blue }`,
})
export class AppComponent1 {}

// Array form styles
@Component({
  selector: "app-test",
  template: `<ul>   <li>test</li>
  </ul>
  `,
  styles: [
    `

 :host {
   color: red;
 }
 div { background: blue
 }
`,
  ],
})
class TestComponent {}

// Computed properties - should NOT be formatted
const styles = "foobar";
const template = "foobar";

@Component({
  selector: "app-computed",
  [template]: `<h1>{{       hello }}</h1>`,
  [styles]: `h1 { color: blue }`,
})
export class AppComponent2 {}

`````

## `{"endOfLine":"crlf"}`

`````ts
// Angular @Component decorator - direct template and styles<CRLF>
// Uses Angular-specific syntax: interpolation, directives, bindings<CRLF>
@Component({<CRLF>
  selector: "app-root",<CRLF>
  template: `<CRLF>
    <h1>{{ title }}</h1><CRLF>
    <div *ngIf="isVisible" [class.active]="isActive" (click)="onClick()"><CRLF>
      <span>{{ count }}</span><CRLF>
    </div><CRLF>
    <ul><CRLF>
      <li *ngFor="let item of items">{{ item.name }}</li><CRLF>
    </ul><CRLF>
  `,<CRLF>
  styles: `<CRLF>
    h1 {<CRLF>
      color: blue;<CRLF>
    }<CRLF>
  `,<CRLF>
})<CRLF>
export class AppComponent1 {}<CRLF>
<CRLF>
// Array form styles<CRLF>
@Component({<CRLF>
  selector: "app-test",<CRLF>
  template: `<ul><CRLF>
    <li>test</li><CRLF>
  </ul> `,<CRLF>
  styles: [<CRLF>
    `<CRLF>
      :host {<CRLF>
        color: red;<CRLF>
      }<CRLF>
      div {<CRLF>
        background: blue;<CRLF>
      }<CRLF>
    `,<CRLF>
  ],<CRLF>
})<CRLF>
class TestComponent {}<CRLF>
<CRLF>
// Computed properties - should NOT be formatted<CRLF>
const styles = "foobar";<CRLF>
const template = "foobar";<CRLF>
<CRLF>
@Component({<CRLF>
  selector: "app-computed",<CRLF>
  [template]: `<h1>{{       hello }}</h1>`,<CRLF>
  [styles]: `h1 { color: blue }`,<CRLF>
})<CRLF>
export class AppComponent2 {}<CRLF>

`````
