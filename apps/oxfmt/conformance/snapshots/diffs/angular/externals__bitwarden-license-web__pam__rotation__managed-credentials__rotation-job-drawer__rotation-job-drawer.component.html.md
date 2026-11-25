# externals/bitwarden-license-web/pam/rotation/managed-credentials/rotation-job-drawer/rotation-job-drawer.component.html

## Option 1

`````json
{"printWidth":80}
`````

### Diff

`````diff
===================================================================
--- prettier
+++ oxfmt
@@ -137,11 +137,9 @@
           <table
             class="tw-w-full tw-table-auto tw-border-collapse tw-text-start tw-text-sm"
           >
             <caption class="tw-sr-only">
-              {{
-                "pamRotationHistoryAttemptsCaption" | i18n
-              }}
+              {{ "pamRotationHistoryAttemptsCaption" | i18n }}
             </caption>
             <thead
               class="tw-border-0 tw-border-b tw-border-solid tw-border-border-base tw-text-xs tw-font-medium tw-text-muted"
             >

`````

### Actual (oxfmt)

`````html
<bit-dialog dialogSize="small">
  <ng-container bitDialogTitle>
    <span>{{ "pamRotationJobDrawerTitle" | i18n }}</span>
    <span
      bitBadge
      [variant]="job.statusVariant"
      class="tw-ms-2"
      data-testid="drawer-result"
      >{{ job.statusLabelKey | i18n }}</span
    >
  </ng-container>

  <ng-container bitDialogContent>
    <dl class="tw-m-0 tw-flex tw-flex-col tw-gap-4">
      @if (job.startedAt) {
        <div>
          <dt class="tw-text-sm tw-font-bold tw-text-muted">
            {{ "pamRotationAttemptStarted" | i18n }}
          </dt>
          <dd class="tw-m-0" data-testid="drawer-started">
            {{ job.startedAt | date: "long" }}
          </dd>
        </div>
      }

      @if (params.showCredential) {
        <div>
          <dt class="tw-text-sm tw-font-bold tw-text-muted">
            {{ "pamRotationConfigColumnCredential" | i18n }}
          </dt>
          <dd
            class="tw-m-0 tw-break-words"
            [class.tw-font-mono]="!job.credentialResolved"
            [class.tw-text-muted]="!job.credentialResolved"
            data-testid="drawer-credential"
          >
            {{ job.credentialName }}
          </dd>
        </div>
      }

      <div>
        <dt class="tw-text-sm tw-font-bold tw-text-muted">
          {{ "pamRotationHistoryColumnSource" | i18n }}
        </dt>
        <dd class="tw-m-0" data-testid="drawer-source">
          {{ job.sourceLabelKey | i18n }}
        </dd>
      </div>

      @if (job.duration || job.running) {
        <div>
          <dt class="tw-text-sm tw-font-bold tw-text-muted">
            {{ "pamRotationHistoryDurationLabel" | i18n }}
          </dt>
          <dd class="tw-m-0" data-testid="drawer-duration">
            @if (job.duration) {
              <pam-rotation-duration [parts]="job.duration" />
            } @else {
              <span class="tw-text-muted">{{
                "pamRotationAttemptInProgress" | i18n
              }}</span>
            }
          </dd>
        </div>
      }
    </dl>

    @if (job.sessionTerminationLabelKey) {
      <p
        class="tw-mb-0 tw-mt-4 tw-text-sm"
        [class]="
          job.sessionTerminationFailed ? 'tw-text-warning' : 'tw-text-muted'
        "
        data-testid="drawer-session-termination"
      >
        {{ job.sessionTerminationLabelKey | i18n }}
      </p>
    }

    @if (
      job.failed &&
      (job.causeLabelKey ||
        job.reportedReason ||
        job.syncStateLabelKey ||
        job.attemptsUniform)
    ) {
      <section
        class="tw-mt-4 tw-border-0 tw-border-t tw-border-solid tw-border-border-base tw-pt-4"
        data-testid="drawer-cause"
      >
        <h3 class="tw-m-0 tw-mb-2 tw-text-sm tw-font-bold tw-text-muted">
          {{ "pamRotationHistoryFailureHeading" | i18n }}
        </h3>
        @if (job.causeLabelKey) {
          <p class="tw-m-0 tw-text-main">{{ job.causeLabelKey | i18n }}</p>
        } @else if (job.reportedReason) {
          <p class="tw-m-0 tw-break-words tw-font-mono tw-text-sm tw-text-main">
            {{ job.reportedReason }}
          </p>
        }
        @if (job.syncStateLabelKey) {
          <p
            class="tw-m-0 tw-mt-2 tw-text-sm"
            [class]="
              job.syncStateIndeterminate ? 'tw-text-warning' : 'tw-text-muted'
            "
          >
            {{ job.syncStateLabelKey | i18n }}
          </p>
        }
        @if (job.attemptsUniform) {
          <p class="tw-m-0 tw-mt-2 tw-text-sm tw-text-muted">
            {{ "pamRotationHistoryAttemptsAllAlike" | i18n }}
          </p>
        }
        @if (job.causeLabelKey && job.reportedReason) {
          <p
            class="tw-m-0 tw-mt-2 tw-break-words tw-font-mono tw-text-xs tw-text-muted"
            data-testid="drawer-reported-reason"
          >
            {{ "pamRotationFailureReportedDetail" | i18n: job.reportedReason }}
          </p>
        }
      </section>
    }

    @if (job.attempts.length > 0) {
      <section
        class="tw-mt-4 tw-border-0 tw-border-t tw-border-solid tw-border-border-base tw-pt-4"
        data-testid="drawer-attempts"
      >
        <h3 class="tw-m-0 tw-mb-1 tw-text-sm tw-font-bold tw-text-muted">
          {{ "pamRotationHistoryColumnAttempts" | i18n }}
        </h3>
        <div class="tw-overflow-x-auto">
          <table
            class="tw-w-full tw-table-auto tw-border-collapse tw-text-start tw-text-sm"
          >
            <caption class="tw-sr-only">
              {{ "pamRotationHistoryAttemptsCaption" | i18n }}
            </caption>
            <thead
              class="tw-border-0 tw-border-b tw-border-solid tw-border-border-base tw-text-xs tw-font-medium tw-text-muted"
            >
              <tr>
                <th scope="col" class="tw-py-1.5 tw-pe-3 tw-text-start">
                  {{ "pamRotationHistoryColumnAttempt" | i18n }}
                </th>
                <th scope="col" class="tw-py-1.5 tw-pe-3 tw-text-start">
                  {{ "pamRotationAttemptStarted" | i18n }}
                </th>
                <th scope="col" class="tw-py-1.5 tw-pe-3 tw-text-start">
                  {{ "pamRotationHistoryDurationLabel" | i18n }}
                </th>
                <th scope="col" class="tw-py-1.5 tw-text-start">
                  {{ "pamRotationHistoryColumnResult" | i18n }}
                </th>
              </tr>
            </thead>
            <tbody>
              @for (attempt of job.attempts; track attempt.id) {
                <tr
                  class="tw-border-0 tw-border-b tw-border-solid tw-border-border-base tw-align-baseline last:tw-border-0"
                  data-testid="drawer-attempt-row"
                >
                  <th
                    scope="row"
                    class="tw-whitespace-nowrap tw-py-1.5 tw-pe-3 tw-text-start tw-font-normal tw-text-muted"
                  >
                    {{ attempt.ordinal }}
                  </th>
                  <td
                    class="tw-whitespace-nowrap tw-py-1.5 tw-pe-3 tw-text-muted"
                  >
                    {{ attempt.startedAt | date: "mediumTime" }}
                  </td>
                  <td
                    class="tw-whitespace-nowrap tw-py-1.5 tw-pe-3 tw-text-muted"
                  >
                    @if (attempt.duration) {
                      <pam-rotation-duration [parts]="attempt.duration" />
                    } @else if (attempt.running) {
                      {{ "pamRotationAttemptInProgress" | i18n }}
                    }
                  </td>
                  <td class="tw-py-1.5 tw-text-main">
                    {{ attempt.statusLabelKey | i18n }}
                    @if (attempt.divergentFailureReason; as reason) {
                      <span
                        class="tw-block tw-break-words tw-font-mono tw-text-xs tw-text-muted"
                      >
                        {{ reason }}
                      </span>
                    }
                  </td>
                </tr>
              }
            </tbody>
          </table>
        </div>
      </section>
    }
  </ng-container>
</bit-dialog>

`````

### Expected (prettier)

`````html
<bit-dialog dialogSize="small">
  <ng-container bitDialogTitle>
    <span>{{ "pamRotationJobDrawerTitle" | i18n }}</span>
    <span
      bitBadge
      [variant]="job.statusVariant"
      class="tw-ms-2"
      data-testid="drawer-result"
      >{{ job.statusLabelKey | i18n }}</span
    >
  </ng-container>

  <ng-container bitDialogContent>
    <dl class="tw-m-0 tw-flex tw-flex-col tw-gap-4">
      @if (job.startedAt) {
        <div>
          <dt class="tw-text-sm tw-font-bold tw-text-muted">
            {{ "pamRotationAttemptStarted" | i18n }}
          </dt>
          <dd class="tw-m-0" data-testid="drawer-started">
            {{ job.startedAt | date: "long" }}
          </dd>
        </div>
      }

      @if (params.showCredential) {
        <div>
          <dt class="tw-text-sm tw-font-bold tw-text-muted">
            {{ "pamRotationConfigColumnCredential" | i18n }}
          </dt>
          <dd
            class="tw-m-0 tw-break-words"
            [class.tw-font-mono]="!job.credentialResolved"
            [class.tw-text-muted]="!job.credentialResolved"
            data-testid="drawer-credential"
          >
            {{ job.credentialName }}
          </dd>
        </div>
      }

      <div>
        <dt class="tw-text-sm tw-font-bold tw-text-muted">
          {{ "pamRotationHistoryColumnSource" | i18n }}
        </dt>
        <dd class="tw-m-0" data-testid="drawer-source">
          {{ job.sourceLabelKey | i18n }}
        </dd>
      </div>

      @if (job.duration || job.running) {
        <div>
          <dt class="tw-text-sm tw-font-bold tw-text-muted">
            {{ "pamRotationHistoryDurationLabel" | i18n }}
          </dt>
          <dd class="tw-m-0" data-testid="drawer-duration">
            @if (job.duration) {
              <pam-rotation-duration [parts]="job.duration" />
            } @else {
              <span class="tw-text-muted">{{
                "pamRotationAttemptInProgress" | i18n
              }}</span>
            }
          </dd>
        </div>
      }
    </dl>

    @if (job.sessionTerminationLabelKey) {
      <p
        class="tw-mb-0 tw-mt-4 tw-text-sm"
        [class]="
          job.sessionTerminationFailed ? 'tw-text-warning' : 'tw-text-muted'
        "
        data-testid="drawer-session-termination"
      >
        {{ job.sessionTerminationLabelKey | i18n }}
      </p>
    }

    @if (
      job.failed &&
      (job.causeLabelKey ||
        job.reportedReason ||
        job.syncStateLabelKey ||
        job.attemptsUniform)
    ) {
      <section
        class="tw-mt-4 tw-border-0 tw-border-t tw-border-solid tw-border-border-base tw-pt-4"
        data-testid="drawer-cause"
      >
        <h3 class="tw-m-0 tw-mb-2 tw-text-sm tw-font-bold tw-text-muted">
          {{ "pamRotationHistoryFailureHeading" | i18n }}
        </h3>
        @if (job.causeLabelKey) {
          <p class="tw-m-0 tw-text-main">{{ job.causeLabelKey | i18n }}</p>
        } @else if (job.reportedReason) {
          <p class="tw-m-0 tw-break-words tw-font-mono tw-text-sm tw-text-main">
            {{ job.reportedReason }}
          </p>
        }
        @if (job.syncStateLabelKey) {
          <p
            class="tw-m-0 tw-mt-2 tw-text-sm"
            [class]="
              job.syncStateIndeterminate ? 'tw-text-warning' : 'tw-text-muted'
            "
          >
            {{ job.syncStateLabelKey | i18n }}
          </p>
        }
        @if (job.attemptsUniform) {
          <p class="tw-m-0 tw-mt-2 tw-text-sm tw-text-muted">
            {{ "pamRotationHistoryAttemptsAllAlike" | i18n }}
          </p>
        }
        @if (job.causeLabelKey && job.reportedReason) {
          <p
            class="tw-m-0 tw-mt-2 tw-break-words tw-font-mono tw-text-xs tw-text-muted"
            data-testid="drawer-reported-reason"
          >
            {{ "pamRotationFailureReportedDetail" | i18n: job.reportedReason }}
          </p>
        }
      </section>
    }

    @if (job.attempts.length > 0) {
      <section
        class="tw-mt-4 tw-border-0 tw-border-t tw-border-solid tw-border-border-base tw-pt-4"
        data-testid="drawer-attempts"
      >
        <h3 class="tw-m-0 tw-mb-1 tw-text-sm tw-font-bold tw-text-muted">
          {{ "pamRotationHistoryColumnAttempts" | i18n }}
        </h3>
        <div class="tw-overflow-x-auto">
          <table
            class="tw-w-full tw-table-auto tw-border-collapse tw-text-start tw-text-sm"
          >
            <caption class="tw-sr-only">
              {{
                "pamRotationHistoryAttemptsCaption" | i18n
              }}
            </caption>
            <thead
              class="tw-border-0 tw-border-b tw-border-solid tw-border-border-base tw-text-xs tw-font-medium tw-text-muted"
            >
              <tr>
                <th scope="col" class="tw-py-1.5 tw-pe-3 tw-text-start">
                  {{ "pamRotationHistoryColumnAttempt" | i18n }}
                </th>
                <th scope="col" class="tw-py-1.5 tw-pe-3 tw-text-start">
                  {{ "pamRotationAttemptStarted" | i18n }}
                </th>
                <th scope="col" class="tw-py-1.5 tw-pe-3 tw-text-start">
                  {{ "pamRotationHistoryDurationLabel" | i18n }}
                </th>
                <th scope="col" class="tw-py-1.5 tw-text-start">
                  {{ "pamRotationHistoryColumnResult" | i18n }}
                </th>
              </tr>
            </thead>
            <tbody>
              @for (attempt of job.attempts; track attempt.id) {
                <tr
                  class="tw-border-0 tw-border-b tw-border-solid tw-border-border-base tw-align-baseline last:tw-border-0"
                  data-testid="drawer-attempt-row"
                >
                  <th
                    scope="row"
                    class="tw-whitespace-nowrap tw-py-1.5 tw-pe-3 tw-text-start tw-font-normal tw-text-muted"
                  >
                    {{ attempt.ordinal }}
                  </th>
                  <td
                    class="tw-whitespace-nowrap tw-py-1.5 tw-pe-3 tw-text-muted"
                  >
                    {{ attempt.startedAt | date: "mediumTime" }}
                  </td>
                  <td
                    class="tw-whitespace-nowrap tw-py-1.5 tw-pe-3 tw-text-muted"
                  >
                    @if (attempt.duration) {
                      <pam-rotation-duration [parts]="attempt.duration" />
                    } @else if (attempt.running) {
                      {{ "pamRotationAttemptInProgress" | i18n }}
                    }
                  </td>
                  <td class="tw-py-1.5 tw-text-main">
                    {{ attempt.statusLabelKey | i18n }}
                    @if (attempt.divergentFailureReason; as reason) {
                      <span
                        class="tw-block tw-break-words tw-font-mono tw-text-xs tw-text-muted"
                      >
                        {{ reason }}
                      </span>
                    }
                  </td>
                </tr>
              }
            </tbody>
          </table>
        </div>
      </section>
    }
  </ng-container>
</bit-dialog>

`````
