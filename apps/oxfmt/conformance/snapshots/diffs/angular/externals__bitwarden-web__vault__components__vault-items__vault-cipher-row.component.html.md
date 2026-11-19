# externals/bitwarden-web/vault/components/vault-items/vault-cipher-row.component.html

## Option 1

`````json
{"printWidth":80}
`````

### Diff

`````diff
===================================================================
--- prettier
+++ oxfmt
@@ -188,12 +188,10 @@
             <i class="bwi bwi-fw bwi-external-link" aria-hidden="true"></i>
             {{ "launch" | i18n }}
           </button>
         }
-        @if (
-          accessActions != null &&
-          (accessActions.cancelableRequest$(cipher) | async)
-        ) {
+        @if (accessActions != null &&
+          (accessActions.cancelableRequest$(cipher) | async)) {
           <button
             type="button"
             bitMenuItem
             (click)="accessActions.cancelRequest(cipher)"

`````

### Actual (oxfmt)

`````html
<td bitCell [ngClass]="RowHeightClass" class="tw-min-w-fit">
  @if (showCheckbox) {
    <input
      type="checkbox"
      bitCheckbox
      appStopProp
      [disabled]="disabled || decryptionFailure || isPartial"
      [checked]="checked"
      (change)="$event ? this.checkedToggled.next() : null"
      [attr.aria-label]="'vaultItemSelect' | i18n"
    />
  }
</td>
<td
  bitCell
  [ngClass]="RowHeightClass"
  class="tw-min-w-fit [&>*]:tw-flex tw-flex"
>
  <app-vault-icon [cipher]="cipher"></app-vault-icon>
</td>
<td bitCell [ngClass]="RowHeightClass" class="tw-truncate tw-pr-7 lg:tw-pr-0">
  <div class="tw-inline-flex tw-w-full">
    <!-- Opt out of router focus manager via [state] input, since the dialog will handle focus -->
    <button
      bitLink
      class="tw-overflow-hidden tw-text-ellipsis tw-text-start tw-leading-snug"
      [disabled]="disabled"
      [routerLink]="[]"
      [queryParams]="{ itemId: cipher.id, action: clickAction }"
      queryParamsHandling="merge"
      [replaceUrl]="true"
      title="{{ 'editItemWithName' | i18n: cipher.name }}"
      type="button"
      appStopProp
      aria-haspopup="dialog"
      id="cipher-btn-{{ cipher.id }}"
      [state]="{
        focusAfterNav: false,
      }"
    >
      {{ cipher.name }}
    </button>
    @if (hasAttachments) {
      <ng-container>
        <i
          class="bwi bwi-paperclip tw-ml-2 tw-leading-normal tw-hidden lg:tw-table-cell"
          appStopProp
          title="{{ 'attachments' | i18n }}"
          aria-hidden="true"
        ></i>
        <span class="tw-sr-only">{{ "attachments" | i18n }}</span>
        @if (showFixOldAttachments) {
          <ng-container>
            <i
              class="bwi bwi-exclamation-triangle tw-ml-2 tw-leading-normal tw-text-warning"
              appStopProp
              title="{{ 'attachmentsNeedFix' | i18n }}"
              aria-hidden="true"
            ></i>
            <span class="tw-sr-only">{{ "attachmentsNeedFix" | i18n }}</span>
          </ng-container>
        }
      </ng-container>
    }
  </div>
  <br />
  <span class="tw-text-sm tw-text-muted" appStopProp>{{ subtitle }}</span>
</td>
@if (showOwner) {
  <td bitCell [ngClass]="RowHeightClass" class="tw-hidden lg:tw-table-cell">
    <app-org-badge
      [disabled]="disabled"
      [organizationId]="cipher.organizationId"
      [organizationName]="cipher.organizationId | orgNameFromId: organizations"
      appStopProp
    >
    </app-org-badge>
  </td>
}
@if (showCollections) {
  <td
    bitCell
    [ngClass]="RowHeightClass"
    class="tw-hidden tw-pr-7 xl:tw-table-cell"
  >
    @if (cipher.collectionIds) {
      <app-collection-badge
        [collectionIds]="cipher.collectionIds"
        [collections]="collections"
      ></app-collection-badge>
    }
  </td>
}
@if (showGroups) {
  <td
    bitCell
    [ngClass]="RowHeightClass"
    class="tw-hidden tw-pr-7 xl:tw-table-cell tw-text-sm"
  ></td>
}
@if (viewingOrgVault) {
  <td
    bitCell
    [ngClass]="RowHeightClass"
    class="tw-hidden tw-pr-7 lg:tw-table-cell"
  >
    <p
      class="tw-mb-0 tw-truncate tw-text-sm tw-text-muted"
      [bitTooltip]="permissionTooltip"
      [title]="permissionText"
    >
      {{ permissionText }}
    </p>
  </td>
}
@if (showControlledAccess) {
  <td bitCell [ngClass]="RowHeightClass" class="tw-hidden lg:tw-table-cell">
    @if (leaseBadge) {
      <ng-container
        *ngComponentOutlet="leaseBadge; inputs: { cipher: cipher }"
      />
    }
  </td>
}
<td bitCell [ngClass]="RowHeightClass" class="tw-text-right">
  @if (decryptionFailure) {
    <button
      [disabled]="disabled || !canManageCollection"
      [bitMenuTriggerFor]="corruptedCipherOptions"
      size="small"
      bitIconButton="bwi-ellipsis-v"
      type="button"
      label="{{ 'options' | i18n }}"
      appStopProp
    ></button>
  }
  <bit-menu #corruptedCipherOptions>
    @if (canDeleteCipher) {
      <button
        bitMenuItem
        variant="danger"
        (click)="deleteCipher()"
        type="button"
      >
        <bit-icon fixedWidth name="bwi-trash" slot="start" />
        {{ (isDeleted ? "permanentlyDelete" : "delete") | i18n }}
      </button>
    }
  </bit-menu>
  @if (!decryptionFailure) {
    @let cancelableRequest =
      (accessActions?.cancelableRequest$(cipher) | async) ?? false;
    <span class="tw-relative tw-inline-flex tw-items-center">
      <div
        class="tw-absolute tw-right-full tw-inset-y-0 tw-flex tw-items-center tw-gap-2 tw-pr-2 tw-opacity-0 tw-pointer-events-none tw-transition-opacity group-hover/cipher-row:tw-opacity-100 group-hover/cipher-row:tw-pointer-events-auto group-has-[:focus-visible]/cipher-row:tw-opacity-100 group-has-[:focus-visible]/cipher-row:tw-pointer-events-auto [&:has([aria-expanded='true'])]:tw-opacity-100 [&:has([aria-expanded='true'])]:tw-pointer-events-auto"
      >
        @if (canLaunch) {
          <button
            bitIconButton="bwi-external-link"
            bitSuffix
            type="button"
            (click)="openUri(launchUri)"
            data-testid="launch-website"
            [label]="'launchWebsite' | i18n"
          ></button>
        }

        <vault-item-copy-actions
          [cipher]="cipher"
          [showQuickCopyActions]="showQuickCopyActions()"
        ></vault-item-copy-actions>
      </div>

      @if (!isPartial || showLaunchInMenu || cancelableRequest) {
        <button
          [bitMenuTriggerFor]="cipherOptions"
          [disabled]="disabled"
          size="small"
          bitIconButton="bwi-ellipsis-v"
          type="button"
          appStopProp
          label="{{ 'moreOptions' | i18n }}"
        ></button>
      }
      <bit-menu #cipherOptions>
        @if (showLaunchInMenu) {
          <button type="button" bitMenuItem (click)="openUri(launchUri)">
            <i class="bwi bwi-fw bwi-external-link" aria-hidden="true"></i>
            {{ "launch" | i18n }}
          </button>
        }
        @if (accessActions != null &&
          (accessActions.cancelableRequest$(cipher) | async)) {
          <button
            type="button"
            bitMenuItem
            (click)="accessActions.cancelRequest(cipher)"
          >
            <i class="bwi bwi-fw bwi-close" aria-hidden="true"></i>
            {{ "pendingStateCancelRequest" | i18n }}
          </button>
        }
        @if (showFavorite && !isPartial) {
          <button bitMenuItem type="button" (click)="toggleFavorite()">
            <i class="bwi bwi-fw bwi-star" aria-hidden="true"></i>
            {{ (cipher.favorite ? "unfavorite" : "favorite") | i18n }}
          </button>
        }
        @if (!isDeleted && canEditCipher && !isPartial) {
          <button bitMenuItem type="button" (click)="editCipher()">
            <i class="bwi bwi-fw bwi-pencil-square" aria-hidden="true"></i>
            {{ "edit" | i18n }}
          </button>
        }
        @if (showAttachments && !isPartial) {
          <button bitMenuItem type="button" (click)="attachments()">
            <i class="bwi bwi-fw bwi-paperclip" aria-hidden="true"></i>
            {{ "attachments" | i18n }}
          </button>
        }
        @if (showClone && !isPartial) {
          <button bitMenuItem type="button" (click)="clone()">
            <i class="bwi bwi-fw bwi-files" aria-hidden="true"></i>
            {{ "clone" | i18n }}
          </button>
        }
        @if (showAssignToCollections && !isPartial) {
          <button bitMenuItem type="button" (click)="assignToCollections()">
            <bit-icon
              fixedWidth
              [name]="'bwi-collection-shared' | vfo1Icon"
              slot="start"
            />
            {{ "assignToCollections" | vfo1I18n: "addToSharedFolder" }}
          </button>
        }
        <button
          bitMenuItem
          type="button"
          *appShareLinkMenuItem="cipher; let share = share"
          (click)="share()"
        >
          <bit-icon name="bwi-share-link" fixedWidth />
          {{ "shareViaLink" | i18n }}
        </button>
        @if (showEventLogs) {
          <button bitMenuItem type="button" (click)="events()">
            <i class="bwi bwi-fw bwi-file-text" aria-hidden="true"></i>
            {{ "eventLogs" | i18n }}
          </button>
        }
        @if (showArchiveButton && !isPartial) {
          @if (userCanArchive) {
            <button bitMenuItem (click)="archive()" type="button">
              <i class="bwi bwi-fw bwi-archive" aria-hidden="true"></i>
              {{ "archiveVerb" | i18n }}
            </button>
          }
          @if (!userCanArchive) {
            <button
              bitMenuItem
              (click)="badge.promptForPremium($event)"
              type="button"
            >
              <i class="bwi bwi-fw bwi-archive" aria-hidden="true"></i>
              {{ "archiveVerb" | i18n }}
              <!-- Hide app-premium badge from accessibility tools as it results in a button within a button -->
              <div slot="end" class="-tw-mt-0.5" aria-hidden>
                <app-premium-badge #badge></app-premium-badge>
              </div>
            </button>
          }
        }

        @if (showUnArchiveButton && !isPartial) {
          <button bitMenuItem (click)="unarchive()" type="button">
            <i class="bwi bwi-fw bwi-unarchive" aria-hidden="true"></i>
            {{ "unArchive" | i18n }}
          </button>
        }

        @if (isDeleted && canRestoreCipher && !isPartial) {
          <button bitMenuItem (click)="restore()" type="button">
            <i class="bwi bwi-fw bwi-undo" aria-hidden="true"></i>
            {{ "restore" | i18n }}
          </button>
        }
        @if (canDeleteCipher && !isPartial) {
          <button
            bitMenuItem
            variant="danger"
            (click)="deleteCipher()"
            type="button"
          >
            <bit-icon fixedWidth name="bwi-trash" slot="start" />
            {{ (isDeleted ? "permanentlyDelete" : "delete") | i18n }}
          </button>
        }
      </bit-menu>
    </span>
  }
</td>

`````

### Expected (prettier)

`````html
<td bitCell [ngClass]="RowHeightClass" class="tw-min-w-fit">
  @if (showCheckbox) {
    <input
      type="checkbox"
      bitCheckbox
      appStopProp
      [disabled]="disabled || decryptionFailure || isPartial"
      [checked]="checked"
      (change)="$event ? this.checkedToggled.next() : null"
      [attr.aria-label]="'vaultItemSelect' | i18n"
    />
  }
</td>
<td
  bitCell
  [ngClass]="RowHeightClass"
  class="tw-min-w-fit [&>*]:tw-flex tw-flex"
>
  <app-vault-icon [cipher]="cipher"></app-vault-icon>
</td>
<td bitCell [ngClass]="RowHeightClass" class="tw-truncate tw-pr-7 lg:tw-pr-0">
  <div class="tw-inline-flex tw-w-full">
    <!-- Opt out of router focus manager via [state] input, since the dialog will handle focus -->
    <button
      bitLink
      class="tw-overflow-hidden tw-text-ellipsis tw-text-start tw-leading-snug"
      [disabled]="disabled"
      [routerLink]="[]"
      [queryParams]="{ itemId: cipher.id, action: clickAction }"
      queryParamsHandling="merge"
      [replaceUrl]="true"
      title="{{ 'editItemWithName' | i18n: cipher.name }}"
      type="button"
      appStopProp
      aria-haspopup="dialog"
      id="cipher-btn-{{ cipher.id }}"
      [state]="{
        focusAfterNav: false,
      }"
    >
      {{ cipher.name }}
    </button>
    @if (hasAttachments) {
      <ng-container>
        <i
          class="bwi bwi-paperclip tw-ml-2 tw-leading-normal tw-hidden lg:tw-table-cell"
          appStopProp
          title="{{ 'attachments' | i18n }}"
          aria-hidden="true"
        ></i>
        <span class="tw-sr-only">{{ "attachments" | i18n }}</span>
        @if (showFixOldAttachments) {
          <ng-container>
            <i
              class="bwi bwi-exclamation-triangle tw-ml-2 tw-leading-normal tw-text-warning"
              appStopProp
              title="{{ 'attachmentsNeedFix' | i18n }}"
              aria-hidden="true"
            ></i>
            <span class="tw-sr-only">{{ "attachmentsNeedFix" | i18n }}</span>
          </ng-container>
        }
      </ng-container>
    }
  </div>
  <br />
  <span class="tw-text-sm tw-text-muted" appStopProp>{{ subtitle }}</span>
</td>
@if (showOwner) {
  <td bitCell [ngClass]="RowHeightClass" class="tw-hidden lg:tw-table-cell">
    <app-org-badge
      [disabled]="disabled"
      [organizationId]="cipher.organizationId"
      [organizationName]="cipher.organizationId | orgNameFromId: organizations"
      appStopProp
    >
    </app-org-badge>
  </td>
}
@if (showCollections) {
  <td
    bitCell
    [ngClass]="RowHeightClass"
    class="tw-hidden tw-pr-7 xl:tw-table-cell"
  >
    @if (cipher.collectionIds) {
      <app-collection-badge
        [collectionIds]="cipher.collectionIds"
        [collections]="collections"
      ></app-collection-badge>
    }
  </td>
}
@if (showGroups) {
  <td
    bitCell
    [ngClass]="RowHeightClass"
    class="tw-hidden tw-pr-7 xl:tw-table-cell tw-text-sm"
  ></td>
}
@if (viewingOrgVault) {
  <td
    bitCell
    [ngClass]="RowHeightClass"
    class="tw-hidden tw-pr-7 lg:tw-table-cell"
  >
    <p
      class="tw-mb-0 tw-truncate tw-text-sm tw-text-muted"
      [bitTooltip]="permissionTooltip"
      [title]="permissionText"
    >
      {{ permissionText }}
    </p>
  </td>
}
@if (showControlledAccess) {
  <td bitCell [ngClass]="RowHeightClass" class="tw-hidden lg:tw-table-cell">
    @if (leaseBadge) {
      <ng-container
        *ngComponentOutlet="leaseBadge; inputs: { cipher: cipher }"
      />
    }
  </td>
}
<td bitCell [ngClass]="RowHeightClass" class="tw-text-right">
  @if (decryptionFailure) {
    <button
      [disabled]="disabled || !canManageCollection"
      [bitMenuTriggerFor]="corruptedCipherOptions"
      size="small"
      bitIconButton="bwi-ellipsis-v"
      type="button"
      label="{{ 'options' | i18n }}"
      appStopProp
    ></button>
  }
  <bit-menu #corruptedCipherOptions>
    @if (canDeleteCipher) {
      <button
        bitMenuItem
        variant="danger"
        (click)="deleteCipher()"
        type="button"
      >
        <bit-icon fixedWidth name="bwi-trash" slot="start" />
        {{ (isDeleted ? "permanentlyDelete" : "delete") | i18n }}
      </button>
    }
  </bit-menu>
  @if (!decryptionFailure) {
    @let cancelableRequest =
      (accessActions?.cancelableRequest$(cipher) | async) ?? false;
    <span class="tw-relative tw-inline-flex tw-items-center">
      <div
        class="tw-absolute tw-right-full tw-inset-y-0 tw-flex tw-items-center tw-gap-2 tw-pr-2 tw-opacity-0 tw-pointer-events-none tw-transition-opacity group-hover/cipher-row:tw-opacity-100 group-hover/cipher-row:tw-pointer-events-auto group-has-[:focus-visible]/cipher-row:tw-opacity-100 group-has-[:focus-visible]/cipher-row:tw-pointer-events-auto [&:has([aria-expanded='true'])]:tw-opacity-100 [&:has([aria-expanded='true'])]:tw-pointer-events-auto"
      >
        @if (canLaunch) {
          <button
            bitIconButton="bwi-external-link"
            bitSuffix
            type="button"
            (click)="openUri(launchUri)"
            data-testid="launch-website"
            [label]="'launchWebsite' | i18n"
          ></button>
        }

        <vault-item-copy-actions
          [cipher]="cipher"
          [showQuickCopyActions]="showQuickCopyActions()"
        ></vault-item-copy-actions>
      </div>

      @if (!isPartial || showLaunchInMenu || cancelableRequest) {
        <button
          [bitMenuTriggerFor]="cipherOptions"
          [disabled]="disabled"
          size="small"
          bitIconButton="bwi-ellipsis-v"
          type="button"
          appStopProp
          label="{{ 'moreOptions' | i18n }}"
        ></button>
      }
      <bit-menu #cipherOptions>
        @if (showLaunchInMenu) {
          <button type="button" bitMenuItem (click)="openUri(launchUri)">
            <i class="bwi bwi-fw bwi-external-link" aria-hidden="true"></i>
            {{ "launch" | i18n }}
          </button>
        }
        @if (
          accessActions != null &&
          (accessActions.cancelableRequest$(cipher) | async)
        ) {
          <button
            type="button"
            bitMenuItem
            (click)="accessActions.cancelRequest(cipher)"
          >
            <i class="bwi bwi-fw bwi-close" aria-hidden="true"></i>
            {{ "pendingStateCancelRequest" | i18n }}
          </button>
        }
        @if (showFavorite && !isPartial) {
          <button bitMenuItem type="button" (click)="toggleFavorite()">
            <i class="bwi bwi-fw bwi-star" aria-hidden="true"></i>
            {{ (cipher.favorite ? "unfavorite" : "favorite") | i18n }}
          </button>
        }
        @if (!isDeleted && canEditCipher && !isPartial) {
          <button bitMenuItem type="button" (click)="editCipher()">
            <i class="bwi bwi-fw bwi-pencil-square" aria-hidden="true"></i>
            {{ "edit" | i18n }}
          </button>
        }
        @if (showAttachments && !isPartial) {
          <button bitMenuItem type="button" (click)="attachments()">
            <i class="bwi bwi-fw bwi-paperclip" aria-hidden="true"></i>
            {{ "attachments" | i18n }}
          </button>
        }
        @if (showClone && !isPartial) {
          <button bitMenuItem type="button" (click)="clone()">
            <i class="bwi bwi-fw bwi-files" aria-hidden="true"></i>
            {{ "clone" | i18n }}
          </button>
        }
        @if (showAssignToCollections && !isPartial) {
          <button bitMenuItem type="button" (click)="assignToCollections()">
            <bit-icon
              fixedWidth
              [name]="'bwi-collection-shared' | vfo1Icon"
              slot="start"
            />
            {{ "assignToCollections" | vfo1I18n: "addToSharedFolder" }}
          </button>
        }
        <button
          bitMenuItem
          type="button"
          *appShareLinkMenuItem="cipher; let share = share"
          (click)="share()"
        >
          <bit-icon name="bwi-share-link" fixedWidth />
          {{ "shareViaLink" | i18n }}
        </button>
        @if (showEventLogs) {
          <button bitMenuItem type="button" (click)="events()">
            <i class="bwi bwi-fw bwi-file-text" aria-hidden="true"></i>
            {{ "eventLogs" | i18n }}
          </button>
        }
        @if (showArchiveButton && !isPartial) {
          @if (userCanArchive) {
            <button bitMenuItem (click)="archive()" type="button">
              <i class="bwi bwi-fw bwi-archive" aria-hidden="true"></i>
              {{ "archiveVerb" | i18n }}
            </button>
          }
          @if (!userCanArchive) {
            <button
              bitMenuItem
              (click)="badge.promptForPremium($event)"
              type="button"
            >
              <i class="bwi bwi-fw bwi-archive" aria-hidden="true"></i>
              {{ "archiveVerb" | i18n }}
              <!-- Hide app-premium badge from accessibility tools as it results in a button within a button -->
              <div slot="end" class="-tw-mt-0.5" aria-hidden>
                <app-premium-badge #badge></app-premium-badge>
              </div>
            </button>
          }
        }

        @if (showUnArchiveButton && !isPartial) {
          <button bitMenuItem (click)="unarchive()" type="button">
            <i class="bwi bwi-fw bwi-unarchive" aria-hidden="true"></i>
            {{ "unArchive" | i18n }}
          </button>
        }

        @if (isDeleted && canRestoreCipher && !isPartial) {
          <button bitMenuItem (click)="restore()" type="button">
            <i class="bwi bwi-fw bwi-undo" aria-hidden="true"></i>
            {{ "restore" | i18n }}
          </button>
        }
        @if (canDeleteCipher && !isPartial) {
          <button
            bitMenuItem
            variant="danger"
            (click)="deleteCipher()"
            type="button"
          >
            <bit-icon fixedWidth name="bwi-trash" slot="start" />
            {{ (isDeleted ? "permanentlyDelete" : "delete") | i18n }}
          </button>
        }
      </bit-menu>
    </span>
  }
</td>

`````
