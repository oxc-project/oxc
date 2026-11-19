# externals/ng-zorro-antd/components/tabs/tab-body.component.ts

## Option 2

`````json
{"printWidth":100,"htmlWhitespaceSensitivity":"ignore"}
`````

### Diff

`````diff
===================================================================
--- prettier
+++ oxfmt
@@ -22,9 +22,14 @@
 import { requestAnimationFrame } from "ng-zorro-antd/core/polyfill";
 import { generateClassName } from "ng-zorro-antd/core/util";
 
 type AnimationState =
-  "enter-start" | "enter-active" | "leave-start" | "leave-active" | "void" | "hidden";
+  | "enter-start"
+  | "enter-active"
+  | "leave-start"
+  | "leave-active"
+  | "void"
+  | "hidden";
 
 const CLASS_NAME = "ant-tabs-tabpane";
 const ANIMATION_PREFIX = "ant-tabs-switch";
 const ANIMATION_CLASS_MAP: Record<AnimationState, string[]> = {

`````

### Actual (oxfmt)

`````ts
/**
 * Use of this source code is governed by an MIT-style license that can be
 * found in the LICENSE file at https://github.com/NG-ZORRO/ng-zorro-antd/blob/master/LICENSE
 */

import { NgTemplateOutlet } from "@angular/common";
import {
  ChangeDetectionStrategy,
  Component,
  computed,
  effect,
  ElementRef,
  inject,
  input,
  signal,
  TemplateRef,
  untracked,
  ViewEncapsulation,
} from "@angular/core";

import { isAnimationEnabled } from "ng-zorro-antd/core/animation";
import { requestAnimationFrame } from "ng-zorro-antd/core/polyfill";
import { generateClassName } from "ng-zorro-antd/core/util";

type AnimationState =
  | "enter-start"
  | "enter-active"
  | "leave-start"
  | "leave-active"
  | "void"
  | "hidden";

const CLASS_NAME = "ant-tabs-tabpane";
const ANIMATION_PREFIX = "ant-tabs-switch";
const ANIMATION_CLASS_MAP: Record<AnimationState, string[]> = {
  "enter-start": [
    generateClassName(ANIMATION_PREFIX, "enter"),
    generateClassName(ANIMATION_PREFIX, "enter-start"),
  ],
  "enter-active": [
    generateClassName(ANIMATION_PREFIX, "enter"),
    generateClassName(ANIMATION_PREFIX, "enter-active"),
  ],
  "leave-start": [
    generateClassName(ANIMATION_PREFIX, "leave"),
    generateClassName(ANIMATION_PREFIX, "leave-start"),
  ],
  "leave-active": [
    generateClassName(ANIMATION_PREFIX, "leave"),
    generateClassName(ANIMATION_PREFIX, "leave-active"),
  ],
  // If animation is enabled, we should hide the tabpane after the leave animation is done
  hidden: [generateClassName(CLASS_NAME, "hidden")],
  void: [],
};

@Component({
  selector: "[nz-tab-body]",
  exportAs: "nzTabBody",
  imports: [NgTemplateOutlet],
  template: `
    <ng-template [ngTemplateOutlet]="content()" />
  `,
  host: {
    "[class]": "class()",
    "[class.ant-tabs-tabpane-active]": "active()",
    "[attr.tabindex]": "active() ? 0 : -1",
    "[attr.aria-hidden]": "!active()",
    "(transitionend)": "_onTransitionEnd($event)",
  },
  encapsulation: ViewEncapsulation.None,
  changeDetection: ChangeDetectionStrategy.OnPush,
})
export class NzTabBodyComponent {
  private readonly elementRef = inject(ElementRef);

  readonly content = input<TemplateRef<void> | null>(null);
  readonly active = input(false);
  readonly animated = input(true);

  protected readonly _animationState = signal<AnimationState>("void");
  protected readonly _animationEnabled = isAnimationEnabled(() => this.animated());

  protected readonly class = computed(() => {
    const cls: string[] = [CLASS_NAME];
    if (this._animationEnabled()) {
      cls.push(...ANIMATION_CLASS_MAP[this._animationState()]);
    } else if (!this.active()) {
      cls.push(generateClassName(CLASS_NAME, "hidden"));
    }
    return cls;
  });

  constructor() {
    effect(() => {
      if (!this._animationEnabled()) {
        return;
      }

      if (!this.active()) {
        untracked(() => this._animationState.set("leave-start"));
        requestAnimationFrame(() => {
          this._animationState.set("leave-active");
        });
      } else {
        untracked(() => this._animationState.set("enter-start"));
        requestAnimationFrame(() => {
          this._animationState.set("enter-active");
        });
      }
    });
  }

  protected _onTransitionEnd(event: TransitionEvent): void {
    // avoid event bubbling from child elements
    if (event.target !== this.elementRef.nativeElement) {
      return;
    }

    const currentState = this._animationState();
    if (currentState === "enter-active") {
      this._animationState.set("void");
    } else if (currentState === "leave-active") {
      this._animationState.set("hidden");
    }
  }
}

`````

### Expected (prettier)

`````ts
/**
 * Use of this source code is governed by an MIT-style license that can be
 * found in the LICENSE file at https://github.com/NG-ZORRO/ng-zorro-antd/blob/master/LICENSE
 */

import { NgTemplateOutlet } from "@angular/common";
import {
  ChangeDetectionStrategy,
  Component,
  computed,
  effect,
  ElementRef,
  inject,
  input,
  signal,
  TemplateRef,
  untracked,
  ViewEncapsulation,
} from "@angular/core";

import { isAnimationEnabled } from "ng-zorro-antd/core/animation";
import { requestAnimationFrame } from "ng-zorro-antd/core/polyfill";
import { generateClassName } from "ng-zorro-antd/core/util";

type AnimationState =
  "enter-start" | "enter-active" | "leave-start" | "leave-active" | "void" | "hidden";

const CLASS_NAME = "ant-tabs-tabpane";
const ANIMATION_PREFIX = "ant-tabs-switch";
const ANIMATION_CLASS_MAP: Record<AnimationState, string[]> = {
  "enter-start": [
    generateClassName(ANIMATION_PREFIX, "enter"),
    generateClassName(ANIMATION_PREFIX, "enter-start"),
  ],
  "enter-active": [
    generateClassName(ANIMATION_PREFIX, "enter"),
    generateClassName(ANIMATION_PREFIX, "enter-active"),
  ],
  "leave-start": [
    generateClassName(ANIMATION_PREFIX, "leave"),
    generateClassName(ANIMATION_PREFIX, "leave-start"),
  ],
  "leave-active": [
    generateClassName(ANIMATION_PREFIX, "leave"),
    generateClassName(ANIMATION_PREFIX, "leave-active"),
  ],
  // If animation is enabled, we should hide the tabpane after the leave animation is done
  hidden: [generateClassName(CLASS_NAME, "hidden")],
  void: [],
};

@Component({
  selector: "[nz-tab-body]",
  exportAs: "nzTabBody",
  imports: [NgTemplateOutlet],
  template: `
    <ng-template [ngTemplateOutlet]="content()" />
  `,
  host: {
    "[class]": "class()",
    "[class.ant-tabs-tabpane-active]": "active()",
    "[attr.tabindex]": "active() ? 0 : -1",
    "[attr.aria-hidden]": "!active()",
    "(transitionend)": "_onTransitionEnd($event)",
  },
  encapsulation: ViewEncapsulation.None,
  changeDetection: ChangeDetectionStrategy.OnPush,
})
export class NzTabBodyComponent {
  private readonly elementRef = inject(ElementRef);

  readonly content = input<TemplateRef<void> | null>(null);
  readonly active = input(false);
  readonly animated = input(true);

  protected readonly _animationState = signal<AnimationState>("void");
  protected readonly _animationEnabled = isAnimationEnabled(() => this.animated());

  protected readonly class = computed(() => {
    const cls: string[] = [CLASS_NAME];
    if (this._animationEnabled()) {
      cls.push(...ANIMATION_CLASS_MAP[this._animationState()]);
    } else if (!this.active()) {
      cls.push(generateClassName(CLASS_NAME, "hidden"));
    }
    return cls;
  });

  constructor() {
    effect(() => {
      if (!this._animationEnabled()) {
        return;
      }

      if (!this.active()) {
        untracked(() => this._animationState.set("leave-start"));
        requestAnimationFrame(() => {
          this._animationState.set("leave-active");
        });
      } else {
        untracked(() => this._animationState.set("enter-start"));
        requestAnimationFrame(() => {
          this._animationState.set("enter-active");
        });
      }
    });
  }

  protected _onTransitionEnd(event: TransitionEvent): void {
    // avoid event bubbling from child elements
    if (event.target !== this.elementRef.nativeElement) {
      return;
    }

    const currentState = this._animationState();
    if (currentState === "enter-active") {
      this._animationState.set("void");
    } else if (currentState === "leave-active") {
      this._animationState.set("hidden");
    }
  }
}

`````
