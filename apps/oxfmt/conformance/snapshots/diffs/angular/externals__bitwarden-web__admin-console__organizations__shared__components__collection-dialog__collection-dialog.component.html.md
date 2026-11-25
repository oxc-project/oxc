# externals/bitwarden-web/admin-console/organizations/shared/components/collection-dialog/collection-dialog.component.html

## Option 1

`````json
{"printWidth":80}
`````

### Diff

`````diff
===================================================================
--- prettier
+++ oxfmt
@@ -152,10 +152,10 @@
               @if (organization()?.allowAdminAccessToAllCollectionItems) {
                 <span>
                   {{
                     " " +
-                      ("adminCollectionAccess"
-                        | vfo1I18n: "adminSharedFolderAccess")
+                    ("adminCollectionAccess"
+                      | vfo1I18n: "adminSharedFolderAccess")
                   }}
                 </span>
               }
             }

`````

### Actual (oxfmt)

`````html
<form
  [formGroup]="formGroup"
  [bitSubmit]="submit"
  bit-dialog
  dialogSize="large"
>
  <span bitDialogTitle>
    @if (editMode) {
      {{
        (dialogReadonly ? "viewCollection" : "editCollection")
          | vfo1I18n: (dialogReadonly ? "viewSharedFolder" : "editSharedFolder")
      }}
      @if (!loading()) {
        <span class="tw-text-sm tw-normal-case tw-text-muted">{{
          collection()?.name
        }}</span>
      }
    }
    @if (!editMode) {
      {{ "addCollection" | vfo1I18n: "addSharedFolder" }}
    }
  </span>
  <div bitDialogContent>
    @if (editMode && accessRuleCallout) {
      <ng-container
        *ngComponentOutlet="
          accessRuleCallout;
          inputs: {
            organizationId: organizationSelected.value,
            collectionId: collectionId,
          }
        "
      />
    }
    @if (loading()) {
      <ng-container #spinner>
        <i class="bwi bwi-spinner bwi-lg bwi-spin" aria-hidden="true"></i>
      </ng-container>
    }
    @if (!loading()) {
      <bit-tab-group
        [selectedIndex]="tabIndex()"
        (selectedIndexChange)="tabIndex.set($event)"
      >
        <bit-tab label="{{ 'collectionInfo' | vfo1I18n: 'details' }}">
          <bit-form-field>
            <bit-label>{{ "name" | i18n }}</bit-label>
            <input bitInput appAutofocus formControlName="name" />
          </bit-form-field>
          @if (showOrgSelector()) {
            <bit-form-field>
              <bit-label>{{ "organization" | i18n }}</bit-label>
              <bit-select bitInput formControlName="selectedOrg">
                @for (org of organizations$ | async; track org) {
                  <bit-option
                    icon="bwi-business"
                    [value]="org.id"
                    [label]="org.name"
                  >
                  </bit-option>
                }
              </bit-select>
            </bit-form-field>
          }
          <bit-form-field>
            <bit-label>{{
              "nestCollectionUnder" | vfo1I18n: "nestSharedFolderUnder"
            }}</bit-label>
            <bit-select bitInput formControlName="parent">
              <bit-option
                [value]="undefined"
                [label]="'noCollection' | vfo1I18n: 'noSharedFolder'"
              >
              </bit-option>
              @if (deletedParentName()) {
                <bit-option
                  disabled
                  [icon]="'bwi-collection-shared' | vfo1Icon"
                  [value]="deletedParentName()"
                  label="{{ deletedParentName() }} ({{ 'deleted' | i18n }})"
                >
                </bit-option>
              }
              @for (collection of nestOptions(); track collection) {
                <bit-option
                  [icon]="'bwi-collection-shared' | vfo1Icon"
                  [value]="collection.name"
                  [label]="collection.name"
                >
                </bit-option>
              }
            </bit-select>
          </bit-form-field>
          @if (externalId(); as externalIdValue) {
            <bit-form-field>
              <bit-label>{{ "externalId" | i18n }}</bit-label>
              <input
                id="collection-dialog_input_external-id"
                bitInput
                type="text"
                readonly
                formControlName="externalId"
              />
              <button
                id="collection-dialog_button_copy-external-id"
                type="button"
                bitSuffix
                bitIconButton="bwi-clone"
                showToast
                [appCopyClick]="externalIdValue"
                [valueLabel]="'externalId' | i18n"
                [label]="'copyExternalId' | i18n"
              ></button>
              <bit-hint>{{ "externalIdDesc" | i18n }}</bit-hint>
            </bit-form-field>
          }
        </bit-tab>
        <bit-tab [label]="accessTabLabel">
          <div class="tw-mb-3">
            @if (dialogReadonly) {
              <span>{{
                "readOnlyCollectionAccess"
                  | vfo1I18n: "readOnlySharedFolderAccess"
              }}</span>
            }
            @if (!dialogReadonly) {
              @if (showAddAccessWarning()) {
                <bit-callout
                  title="{{
                    'grantManageCollectionWarningTitle'
                      | vfo1I18n: 'grantManagePermissionsWarningTitle'
                  }}"
                  type="warning"
                >
                  {{
                    "grantManageCollectionWarning"
                      | vfo1I18n: "grantManagePermissionsWarning"
                  }}
                </bit-callout>
              }
              @if (organization()?.useGroups) {
                <span>{{
                  "grantCollectionAccess" | vfo1I18n: "grantSharedFolderAccess"
                }}</span>
              }
              @if (!organization()?.useGroups) {
                <span>{{
                  "grantCollectionAccessMembersOnly"
                    | vfo1I18n: "grantSharedFolderAccessMembersOnly"
                }}</span>
              }
              @if (organization()?.allowAdminAccessToAllCollectionItems) {
                <span>
                  {{
                    " " +
                    ("adminCollectionAccess"
                      | vfo1I18n: "adminSharedFolderAccess")
                  }}
                </span>
              }
            }
          </div>
          @if (
            formGroup.controls.access.hasError("managePermissionRequired") &&
            !showAddAccessWarning()
          ) {
            <div class="tw-mb-3 tw-text-danger">
              <i class="bwi bwi-error"></i>
              {{ "managePermissionRequired" | i18n }}
            </div>
          }
          @if (organization()?.useGroups) {
            <bit-access-selector
              [permissionMode]="
                dialogReadonly ? PermissionMode.Readonly : PermissionMode.Edit
              "
              formControlName="access"
              [items]="accessItems$ | async"
              [columnHeader]="'groupSlashMemberColumnHeader' | i18n"
              [selectorLabelText]="'selectGroupsAndMembers' | i18n"
              [selectorHelpText]="'userPermissionOverrideHelperDesc' | i18n"
              [emptySelectionText]="'noMembersOrGroupsAdded' | i18n"
              [initialPermission]="initialPermission()"
            ></bit-access-selector>
          }
          @if (!organization()?.useGroups) {
            <bit-access-selector
              [permissionMode]="
                dialogReadonly ? PermissionMode.Readonly : PermissionMode.Edit
              "
              formControlName="access"
              [items]="accessItems$ | async"
              [columnHeader]="'memberColumnHeader' | i18n"
              [selectorLabelText]="'selectMembers' | i18n"
              [emptySelectionText]="'noMembersAdded' | i18n"
            ></bit-access-selector>
          }
        </bit-tab>
      </bit-tab-group>
    }
  </div>
  <ng-container bitDialogFooter>
    <button
      type="submit"
      bitButton
      bitFormButton
      buttonType="primary"
      [disabled]="loading() || dialogReadonly"
    >
      {{ buttonDisplayName$ | async | i18n }}
    </button>
    <button
      type="button"
      bitButton
      bitFormButton
      buttonType="secondary"
      (click)="cancel()"
      [disabled]="loading()"
    >
      {{ "cancel" | i18n }}
    </button>
    @if (showDeleteButton()) {
      <button
        type="button"
        bitIconButton="bwi-trash"
        buttonType="dangerGhost"
        class="tw-ml-auto"
        bitFormButton
        [label]="'delete' | i18n"
        [bitAction]="delete"
        [disabled]="loading()"
      ></button>
    }
  </ng-container>
</form>

`````

### Expected (prettier)

`````html
<form
  [formGroup]="formGroup"
  [bitSubmit]="submit"
  bit-dialog
  dialogSize="large"
>
  <span bitDialogTitle>
    @if (editMode) {
      {{
        (dialogReadonly ? "viewCollection" : "editCollection")
          | vfo1I18n: (dialogReadonly ? "viewSharedFolder" : "editSharedFolder")
      }}
      @if (!loading()) {
        <span class="tw-text-sm tw-normal-case tw-text-muted">{{
          collection()?.name
        }}</span>
      }
    }
    @if (!editMode) {
      {{ "addCollection" | vfo1I18n: "addSharedFolder" }}
    }
  </span>
  <div bitDialogContent>
    @if (editMode && accessRuleCallout) {
      <ng-container
        *ngComponentOutlet="
          accessRuleCallout;
          inputs: {
            organizationId: organizationSelected.value,
            collectionId: collectionId,
          }
        "
      />
    }
    @if (loading()) {
      <ng-container #spinner>
        <i class="bwi bwi-spinner bwi-lg bwi-spin" aria-hidden="true"></i>
      </ng-container>
    }
    @if (!loading()) {
      <bit-tab-group
        [selectedIndex]="tabIndex()"
        (selectedIndexChange)="tabIndex.set($event)"
      >
        <bit-tab label="{{ 'collectionInfo' | vfo1I18n: 'details' }}">
          <bit-form-field>
            <bit-label>{{ "name" | i18n }}</bit-label>
            <input bitInput appAutofocus formControlName="name" />
          </bit-form-field>
          @if (showOrgSelector()) {
            <bit-form-field>
              <bit-label>{{ "organization" | i18n }}</bit-label>
              <bit-select bitInput formControlName="selectedOrg">
                @for (org of organizations$ | async; track org) {
                  <bit-option
                    icon="bwi-business"
                    [value]="org.id"
                    [label]="org.name"
                  >
                  </bit-option>
                }
              </bit-select>
            </bit-form-field>
          }
          <bit-form-field>
            <bit-label>{{
              "nestCollectionUnder" | vfo1I18n: "nestSharedFolderUnder"
            }}</bit-label>
            <bit-select bitInput formControlName="parent">
              <bit-option
                [value]="undefined"
                [label]="'noCollection' | vfo1I18n: 'noSharedFolder'"
              >
              </bit-option>
              @if (deletedParentName()) {
                <bit-option
                  disabled
                  [icon]="'bwi-collection-shared' | vfo1Icon"
                  [value]="deletedParentName()"
                  label="{{ deletedParentName() }} ({{ 'deleted' | i18n }})"
                >
                </bit-option>
              }
              @for (collection of nestOptions(); track collection) {
                <bit-option
                  [icon]="'bwi-collection-shared' | vfo1Icon"
                  [value]="collection.name"
                  [label]="collection.name"
                >
                </bit-option>
              }
            </bit-select>
          </bit-form-field>
          @if (externalId(); as externalIdValue) {
            <bit-form-field>
              <bit-label>{{ "externalId" | i18n }}</bit-label>
              <input
                id="collection-dialog_input_external-id"
                bitInput
                type="text"
                readonly
                formControlName="externalId"
              />
              <button
                id="collection-dialog_button_copy-external-id"
                type="button"
                bitSuffix
                bitIconButton="bwi-clone"
                showToast
                [appCopyClick]="externalIdValue"
                [valueLabel]="'externalId' | i18n"
                [label]="'copyExternalId' | i18n"
              ></button>
              <bit-hint>{{ "externalIdDesc" | i18n }}</bit-hint>
            </bit-form-field>
          }
        </bit-tab>
        <bit-tab [label]="accessTabLabel">
          <div class="tw-mb-3">
            @if (dialogReadonly) {
              <span>{{
                "readOnlyCollectionAccess"
                  | vfo1I18n: "readOnlySharedFolderAccess"
              }}</span>
            }
            @if (!dialogReadonly) {
              @if (showAddAccessWarning()) {
                <bit-callout
                  title="{{
                    'grantManageCollectionWarningTitle'
                      | vfo1I18n: 'grantManagePermissionsWarningTitle'
                  }}"
                  type="warning"
                >
                  {{
                    "grantManageCollectionWarning"
                      | vfo1I18n: "grantManagePermissionsWarning"
                  }}
                </bit-callout>
              }
              @if (organization()?.useGroups) {
                <span>{{
                  "grantCollectionAccess" | vfo1I18n: "grantSharedFolderAccess"
                }}</span>
              }
              @if (!organization()?.useGroups) {
                <span>{{
                  "grantCollectionAccessMembersOnly"
                    | vfo1I18n: "grantSharedFolderAccessMembersOnly"
                }}</span>
              }
              @if (organization()?.allowAdminAccessToAllCollectionItems) {
                <span>
                  {{
                    " " +
                      ("adminCollectionAccess"
                        | vfo1I18n: "adminSharedFolderAccess")
                  }}
                </span>
              }
            }
          </div>
          @if (
            formGroup.controls.access.hasError("managePermissionRequired") &&
            !showAddAccessWarning()
          ) {
            <div class="tw-mb-3 tw-text-danger">
              <i class="bwi bwi-error"></i>
              {{ "managePermissionRequired" | i18n }}
            </div>
          }
          @if (organization()?.useGroups) {
            <bit-access-selector
              [permissionMode]="
                dialogReadonly ? PermissionMode.Readonly : PermissionMode.Edit
              "
              formControlName="access"
              [items]="accessItems$ | async"
              [columnHeader]="'groupSlashMemberColumnHeader' | i18n"
              [selectorLabelText]="'selectGroupsAndMembers' | i18n"
              [selectorHelpText]="'userPermissionOverrideHelperDesc' | i18n"
              [emptySelectionText]="'noMembersOrGroupsAdded' | i18n"
              [initialPermission]="initialPermission()"
            ></bit-access-selector>
          }
          @if (!organization()?.useGroups) {
            <bit-access-selector
              [permissionMode]="
                dialogReadonly ? PermissionMode.Readonly : PermissionMode.Edit
              "
              formControlName="access"
              [items]="accessItems$ | async"
              [columnHeader]="'memberColumnHeader' | i18n"
              [selectorLabelText]="'selectMembers' | i18n"
              [emptySelectionText]="'noMembersAdded' | i18n"
            ></bit-access-selector>
          }
        </bit-tab>
      </bit-tab-group>
    }
  </div>
  <ng-container bitDialogFooter>
    <button
      type="submit"
      bitButton
      bitFormButton
      buttonType="primary"
      [disabled]="loading() || dialogReadonly"
    >
      {{ buttonDisplayName$ | async | i18n }}
    </button>
    <button
      type="button"
      bitButton
      bitFormButton
      buttonType="secondary"
      (click)="cancel()"
      [disabled]="loading()"
    >
      {{ "cancel" | i18n }}
    </button>
    @if (showDeleteButton()) {
      <button
        type="button"
        bitIconButton="bwi-trash"
        buttonType="dangerGhost"
        class="tw-ml-auto"
        bitFormButton
        [label]="'delete' | i18n"
        [bitAction]="delete"
        [disabled]="loading()"
      ></button>
    }
  </ng-container>
</form>

`````
