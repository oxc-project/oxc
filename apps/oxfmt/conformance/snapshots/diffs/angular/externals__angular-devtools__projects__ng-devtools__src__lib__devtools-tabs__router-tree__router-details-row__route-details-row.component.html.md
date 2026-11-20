# externals/angular-devtools/projects/ng-devtools/src/lib/devtools-tabs/router-tree/router-details-row/route-details-row.component.html

## Option 1

`````json
{"printWidth":80}
`````

### Diff

`````diff
===================================================================
--- prettier
+++ oxfmt
@@ -9,9 +9,10 @@
     }
     @case ("list") {
       @for (provider of dataArray(); track $index) {
         <div class="value-container">
-          <span>{{ provider || "[empty string] " }}</span><ng-container
+          <span>{{ provider || "[empty string] " }}</span
+          ><ng-container
             [ngTemplateOutlet]="actionBtn"
             [ngTemplateOutletContext]="{ value: provider }"
           />
         </div>
@@ -36,9 +37,10 @@
           </ng-template>
         </ng-object-tree-explorer>
       } @else {
         <div class="value-container">
-          <span> {{ rowValue() || "[empty string] " }} </span><ng-container
+          <span> {{ rowValue() || "[empty string] " }} </span
+          ><ng-container
             [ngTemplateOutlet]="actionBtn"
             [ngTemplateOutletContext]="{ value: rowValue() }"
           />
         </div>

`````

### Actual (oxfmt)

`````html
<!-- TODO: Reconsider the existence of this component. -->
<dt>{{ label() }}</dt>
<dd>
  @switch (type()) {
    @case ("flag") {
      <span [class]="rowValue() ? 'tag-active' : 'tag-inactive'">
        {{ rowValue() || "false" }}
      </span>
    }
    @case ("list") {
      @for (provider of dataArray(); track $index) {
        <div class="value-container">
          <span>{{ provider || "[empty string] " }}</span
          ><ng-container
            [ngTemplateOutlet]="actionBtn"
            [ngTemplateOutletContext]="{ value: provider }"
          />
        </div>
      }
    }
    @default {
      @if (rowValue() && renderValueAsJson()) {
        <ng-object-tree-explorer
          [dataSource]="treeData()"
          [childrenAccessor]="treeDataChildrenAccessor"
          [omitRootNodesNames]="true"
        >
          <ng-template let-node>
            @if (!node.expandable) {
              <ng-container
                [ngTemplateOutlet]="actionBtn"
                [ngTemplateOutletContext]="{
                  value: node.prop.descriptor.preview,
                }"
              />
            }
          </ng-template>
        </ng-object-tree-explorer>
      } @else {
        <div class="value-container">
          <span> {{ rowValue() || "[empty string] " }} </span
          ><ng-container
            [ngTemplateOutlet]="actionBtn"
            [ngTemplateOutletContext]="{ value: rowValue() }"
          />
        </div>
      }
    }
  }
</dd>

<ng-template #actionBtn let-value="value">
  @if (actionBtnType() !== "none") {
    <button
      ng-button
      btnType="icon"
      (click)="actionBtnClick.emit(value)"
      [matTooltip]="!actionBtnDisabled() ? actionBtnTooltip() || value : null"
      [disabled]="actionBtnDisabled()"
    >
      @switch (actionBtnType()) {
        @case ("view-source") {
          <mat-icon class="view-source">code</mat-icon>
        }
        @case ("navigate") {
          <mat-icon class="navigate">output</mat-icon>
        }
      }
    </button>
  }
</ng-template>

`````

### Expected (prettier)

`````html
<!-- TODO: Reconsider the existence of this component. -->
<dt>{{ label() }}</dt>
<dd>
  @switch (type()) {
    @case ("flag") {
      <span [class]="rowValue() ? 'tag-active' : 'tag-inactive'">
        {{ rowValue() || "false" }}
      </span>
    }
    @case ("list") {
      @for (provider of dataArray(); track $index) {
        <div class="value-container">
          <span>{{ provider || "[empty string] " }}</span><ng-container
            [ngTemplateOutlet]="actionBtn"
            [ngTemplateOutletContext]="{ value: provider }"
          />
        </div>
      }
    }
    @default {
      @if (rowValue() && renderValueAsJson()) {
        <ng-object-tree-explorer
          [dataSource]="treeData()"
          [childrenAccessor]="treeDataChildrenAccessor"
          [omitRootNodesNames]="true"
        >
          <ng-template let-node>
            @if (!node.expandable) {
              <ng-container
                [ngTemplateOutlet]="actionBtn"
                [ngTemplateOutletContext]="{
                  value: node.prop.descriptor.preview,
                }"
              />
            }
          </ng-template>
        </ng-object-tree-explorer>
      } @else {
        <div class="value-container">
          <span> {{ rowValue() || "[empty string] " }} </span><ng-container
            [ngTemplateOutlet]="actionBtn"
            [ngTemplateOutletContext]="{ value: rowValue() }"
          />
        </div>
      }
    }
  }
</dd>

<ng-template #actionBtn let-value="value">
  @if (actionBtnType() !== "none") {
    <button
      ng-button
      btnType="icon"
      (click)="actionBtnClick.emit(value)"
      [matTooltip]="!actionBtnDisabled() ? actionBtnTooltip() || value : null"
      [disabled]="actionBtnDisabled()"
    >
      @switch (actionBtnType()) {
        @case ("view-source") {
          <mat-icon class="view-source">code</mat-icon>
        }
        @case ("navigate") {
          <mat-icon class="navigate">output</mat-icon>
        }
      }
    </button>
  }
</ng-template>

`````
