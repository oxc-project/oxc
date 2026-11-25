# externals/bitwarden-license-web/pam/rotation/managed-credentials/rotation-history.component.html

## Option 1

`````json
{"printWidth":80}
`````

### Diff

`````diff
===================================================================
--- prettier
+++ oxfmt
@@ -7,11 +7,9 @@
     <table
       class="tw-w-full tw-table-auto tw-border-collapse tw-text-start tw-text-main"
     >
       <caption class="tw-sr-only">
-        {{
-          "pamRotationHistoryCaption" | i18n
-        }}
+        {{ "pamRotationHistoryCaption" | i18n }}
       </caption>
       <thead
         class="tw-border-0 tw-border-b-2 tw-border-solid tw-border-border-base tw-text-sm tw-font-medium tw-text-muted"
       >

`````

### Actual (oxfmt)

`````html
@if (jobViews().length === 0) {
  <p class="tw-m-0 tw-text-sm tw-text-muted">
    {{ "pamRotationHistoryEmpty" | i18n }}
  </p>
} @else {
  <div class="tw-overflow-x-auto">
    <table
      class="tw-w-full tw-table-auto tw-border-collapse tw-text-start tw-text-main"
    >
      <caption class="tw-sr-only">
        {{ "pamRotationHistoryCaption" | i18n }}
      </caption>
      <thead
        class="tw-border-0 tw-border-b-2 tw-border-solid tw-border-border-base tw-text-sm tw-font-medium tw-text-muted"
      >
        <tr data-testid="rotation-history-columns">
          <th bitCell scope="col" class="tw-whitespace-nowrap tw-text-start">
            {{ "pamRotationAttemptStarted" | i18n }}
          </th>
          @if (showCredential()) {
            <th bitCell scope="col" class="tw-text-start">
              {{ "pamRotationConfigColumnCredential" | i18n }}
            </th>
          }
          <th
            bitCell
            scope="col"
            class="tw-whitespace-nowrap tw-text-start"
            [class.tw-hidden]="detailsOpen()"
          >
            {{ "pamRotationHistoryColumnSource" | i18n }}
          </th>
          <th bitCell scope="col" class="tw-whitespace-nowrap tw-text-start">
            {{ "pamRotationHistoryColumnResult" | i18n }}
          </th>
          <th
            bitCell
            scope="col"
            class="tw-whitespace-nowrap tw-text-start"
            [class.tw-hidden]="detailsOpen()"
          >
            {{ "pamRotationHistoryDurationLabel" | i18n }}
          </th>
          <th
            bitCell
            scope="col"
            class="tw-whitespace-nowrap tw-text-start"
            [class.tw-hidden]="detailsOpen()"
          >
            {{ "pamRotationHistoryColumnAttempts" | i18n }}
          </th>
        </tr>
      </thead>
      <tbody>
        @for (job of jobViews(); track job.id) {
          <tr
            bitRow
            alignContent="top"
            class="tw-cursor-pointer"
            (click)="openJob(job, resultCell)"
            data-testid="rotation-history-job"
          >
            <th
              bitCell
              scope="row"
              class="tw-whitespace-nowrap tw-text-start tw-text-sm tw-font-normal"
            >
              {{ job.startedAt | date: "short" }}
            </th>

            @if (showCredential()) {
              <td
                bitCell
                class="tw-text-sm"
                data-testid="rotation-history-job-credential"
              >
                <span
                  class="tw-block tw-max-w-sm tw-break-words"
                  [class.tw-font-mono]="!job.credentialResolved"
                  [class.tw-text-muted]="!job.credentialResolved"
                  >{{ job.credentialName }}</span
                >
              </td>
            }

            <td
              bitCell
              class="tw-whitespace-nowrap tw-text-sm"
              [class.tw-hidden]="detailsOpen()"
            >
              {{ job.sourceLabelKey | i18n }}
            </td>

            <td
              bitCell
              role="button"
              tabindex="0"
              class="focus-visible:tw-outline-none focus-visible:tw-ring-2 focus-visible:tw-ring-inset focus-visible:tw-ring-border-focus"
              id="rotation-history_button_details-{{ job.id }}"
              [attr.aria-label]="
                (job.statusLabelKey | i18n) +
                (job.startedAt ? ' ' + (job.startedAt | date: 'medium') : '') +
                (job.sessionTerminationFailed
                  ? ' ' + ('pamRotationSessionTerminationTermFailed' | i18n)
                  : '')
              "
              (keydown.enter)="openJob(job, resultCell)"
              (keydown.space)="
                openJob(job, resultCell); $event.preventDefault()
              "
              data-testid="rotation-history-job-result"
              #resultCell
            >
              <span
                bitBadge
                [variant]="job.statusVariant"
                class="tw-whitespace-nowrap"
                >{{ job.statusLabelKey | i18n }}</span
              >
              @if (job.sessionTerminationFailed) {
                <p class="tw-m-0 tw-mt-1.5 tw-text-xs tw-text-warning">
                  {{ "pamRotationSessionTerminationTermFailed" | i18n }}
                </p>
              }
            </td>

            <td
              bitCell
              class="tw-whitespace-nowrap tw-text-sm"
              [class.tw-hidden]="detailsOpen()"
            >
              @if (job.duration) {
                <pam-rotation-duration [parts]="job.duration" />
              } @else if (job.running) {
                <span class="tw-text-muted">{{
                  "pamRotationAttemptInProgress" | i18n
                }}</span>
              }
            </td>

            <td
              bitCell
              class="tw-whitespace-nowrap tw-text-sm"
              [class.tw-hidden]="detailsOpen()"
              data-testid="rotation-history-job-attempts"
            >
              {{ job.attempts.length }}
            </td>
          </tr>
        }
      </tbody>
    </table>
  </div>
}

`````

### Expected (prettier)

`````html
@if (jobViews().length === 0) {
  <p class="tw-m-0 tw-text-sm tw-text-muted">
    {{ "pamRotationHistoryEmpty" | i18n }}
  </p>
} @else {
  <div class="tw-overflow-x-auto">
    <table
      class="tw-w-full tw-table-auto tw-border-collapse tw-text-start tw-text-main"
    >
      <caption class="tw-sr-only">
        {{
          "pamRotationHistoryCaption" | i18n
        }}
      </caption>
      <thead
        class="tw-border-0 tw-border-b-2 tw-border-solid tw-border-border-base tw-text-sm tw-font-medium tw-text-muted"
      >
        <tr data-testid="rotation-history-columns">
          <th bitCell scope="col" class="tw-whitespace-nowrap tw-text-start">
            {{ "pamRotationAttemptStarted" | i18n }}
          </th>
          @if (showCredential()) {
            <th bitCell scope="col" class="tw-text-start">
              {{ "pamRotationConfigColumnCredential" | i18n }}
            </th>
          }
          <th
            bitCell
            scope="col"
            class="tw-whitespace-nowrap tw-text-start"
            [class.tw-hidden]="detailsOpen()"
          >
            {{ "pamRotationHistoryColumnSource" | i18n }}
          </th>
          <th bitCell scope="col" class="tw-whitespace-nowrap tw-text-start">
            {{ "pamRotationHistoryColumnResult" | i18n }}
          </th>
          <th
            bitCell
            scope="col"
            class="tw-whitespace-nowrap tw-text-start"
            [class.tw-hidden]="detailsOpen()"
          >
            {{ "pamRotationHistoryDurationLabel" | i18n }}
          </th>
          <th
            bitCell
            scope="col"
            class="tw-whitespace-nowrap tw-text-start"
            [class.tw-hidden]="detailsOpen()"
          >
            {{ "pamRotationHistoryColumnAttempts" | i18n }}
          </th>
        </tr>
      </thead>
      <tbody>
        @for (job of jobViews(); track job.id) {
          <tr
            bitRow
            alignContent="top"
            class="tw-cursor-pointer"
            (click)="openJob(job, resultCell)"
            data-testid="rotation-history-job"
          >
            <th
              bitCell
              scope="row"
              class="tw-whitespace-nowrap tw-text-start tw-text-sm tw-font-normal"
            >
              {{ job.startedAt | date: "short" }}
            </th>

            @if (showCredential()) {
              <td
                bitCell
                class="tw-text-sm"
                data-testid="rotation-history-job-credential"
              >
                <span
                  class="tw-block tw-max-w-sm tw-break-words"
                  [class.tw-font-mono]="!job.credentialResolved"
                  [class.tw-text-muted]="!job.credentialResolved"
                  >{{ job.credentialName }}</span
                >
              </td>
            }

            <td
              bitCell
              class="tw-whitespace-nowrap tw-text-sm"
              [class.tw-hidden]="detailsOpen()"
            >
              {{ job.sourceLabelKey | i18n }}
            </td>

            <td
              bitCell
              role="button"
              tabindex="0"
              class="focus-visible:tw-outline-none focus-visible:tw-ring-2 focus-visible:tw-ring-inset focus-visible:tw-ring-border-focus"
              id="rotation-history_button_details-{{ job.id }}"
              [attr.aria-label]="
                (job.statusLabelKey | i18n) +
                (job.startedAt ? ' ' + (job.startedAt | date: 'medium') : '') +
                (job.sessionTerminationFailed
                  ? ' ' + ('pamRotationSessionTerminationTermFailed' | i18n)
                  : '')
              "
              (keydown.enter)="openJob(job, resultCell)"
              (keydown.space)="
                openJob(job, resultCell); $event.preventDefault()
              "
              data-testid="rotation-history-job-result"
              #resultCell
            >
              <span
                bitBadge
                [variant]="job.statusVariant"
                class="tw-whitespace-nowrap"
                >{{ job.statusLabelKey | i18n }}</span
              >
              @if (job.sessionTerminationFailed) {
                <p class="tw-m-0 tw-mt-1.5 tw-text-xs tw-text-warning">
                  {{ "pamRotationSessionTerminationTermFailed" | i18n }}
                </p>
              }
            </td>

            <td
              bitCell
              class="tw-whitespace-nowrap tw-text-sm"
              [class.tw-hidden]="detailsOpen()"
            >
              @if (job.duration) {
                <pam-rotation-duration [parts]="job.duration" />
              } @else if (job.running) {
                <span class="tw-text-muted">{{
                  "pamRotationAttemptInProgress" | i18n
                }}</span>
              }
            </td>

            <td
              bitCell
              class="tw-whitespace-nowrap tw-text-sm"
              [class.tw-hidden]="detailsOpen()"
              data-testid="rotation-history-job-attempts"
            >
              {{ job.attempts.length }}
            </td>
          </tr>
        }
      </tbody>
    </table>
  </div>
}

`````
