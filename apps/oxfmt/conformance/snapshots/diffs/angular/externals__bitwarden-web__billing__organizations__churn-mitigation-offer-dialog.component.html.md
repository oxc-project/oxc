# externals/bitwarden-web/billing/organizations/churn-mitigation-offer-dialog.component.html

## Option 1

`````json
{"printWidth":80}
`````

### Diff

`````diff
===================================================================
--- prettier
+++ oxfmt
@@ -89,11 +89,10 @@
     <h3 bitDialogTitle>{{ "churnOfferTitle" | i18n }}</h3>
     <ng-container bitDialogContent>
       <p class="tw-text-muted" bitTypography="body1">
         {{ "churnOfferPreamble" | i18n }}
-        <strong>{{ params.accessEndDate | date: "longDate" }}</strong>{{
-          "churnOfferPreambleSuffix" | i18n
-        }}
+        <strong>{{ params.accessEndDate | date: "longDate" }}</strong
+        >{{ "churnOfferPreambleSuffix" | i18n }}
       </p>
       <bit-card
         class="tw-relative tw-overflow-hidden !tw-shadow-none !tw-border-none"
       >

`````

### Actual (oxfmt)

`````html
@if (offerRedeemed()) {
  <bit-dialog>
    <span bitDialogTitle>{{ "churnOfferSuccessTitle" | i18n }}</span>
    <ng-container bitDialogContent>
      <p bitTypography="body1" class="tw-text-muted">
        @if (isRecurringAmount) {
          {{
            "churnOfferSuccessBodyRecurringAmount"
              | i18n: discountLabel : billingIntervalUnit : durationDescription
          }}
        } @else {
          {{
            "churnOfferSuccessBody" | i18n: discountLabel : durationDescription
          }}
        }
      </p>
      <bit-card class="tw-bg-bg-secondary tw-shadow-none">
        <dl class="tw-grid tw-grid-cols-2 tw-gap-y-2 tw-m-0">
          <dt
            bitTypography="body1"
            class="tw-font-normal tw-text-fg-body-subtle"
          >
            {{ "plan" | i18n }}
          </dt>
          <dd bitTypography="body1" class="tw-font-medium tw-m-0 tw-text-right">
            {{ params.planName }}
          </dd>
          <dt
            bitTypography="body1"
            class="tw-font-normal tw-text-fg-body-subtle"
          >
            {{ "status" | i18n }}
          </dt>
          <dd bitTypography="body1" class="tw-font-medium tw-m-0 tw-text-right">
            {{ "active" | i18n }}
          </dd>
          <dt
            bitTypography="body1"
            class="tw-font-normal tw-text-fg-body-subtle"
          >
            {{ "discountApplied" | i18n }}
          </dt>
          <dd bitTypography="body1" class="tw-font-medium tw-m-0 tw-text-right">
            @if (isRecurringAmount) {
              {{
                "churnOfferDiscountSummaryRecurringAmount"
                  | i18n
                    : discountLabelPerInterval
                    : durationLength
                    : durationUnit
              }}
            } @else {
              {{
                "churnOfferDiscountSummary"
                  | i18n: discountLabel : durationLength : durationUnit
              }}
            }
          </dd>
          @if (params.nextChargeDate) {
            <dt
              bitTypography="body1"
              class="tw-font-normal tw-text-fg-body-subtle"
            >
              {{ "nextCharge" | i18n }}
            </dt>
            <dd
              bitTypography="body1"
              class="tw-font-medium tw-m-0 tw-text-right"
            >
              {{ params.nextChargeDate | date: "longDate" }}
            </dd>
          }
        </dl>
      </bit-card>
    </ng-container>
    <ng-container bitDialogFooter>
      <button
        type="button"
        bitButton
        buttonType="primary"
        (click)="closeAfterAccept()"
      >
        {{ "close" | i18n }}
      </button>
    </ng-container>
  </bit-dialog>
} @else {
  <bit-dialog dialogSize="large">
    <h3 bitDialogTitle>{{ "churnOfferTitle" | i18n }}</h3>
    <ng-container bitDialogContent>
      <p class="tw-text-muted" bitTypography="body1">
        {{ "churnOfferPreamble" | i18n }}
        <strong>{{ params.accessEndDate | date: "longDate" }}</strong
        >{{ "churnOfferPreambleSuffix" | i18n }}
      </p>
      <bit-card
        class="tw-relative tw-overflow-hidden !tw-shadow-none !tw-border-none"
      >
        <img
          src="../../../images/globe-bg.png"
          alt=""
          aria-hidden="true"
          class="tw-absolute tw-inset-0 tw-size-full tw-object-cover tw-pointer-events-none"
        />
        <div class="tw-relative">
          <div
            class="tw-inline-flex tw-items-center tw-justify-center tw-rounded-xl tw-bg-bg-brand-soft tw-p-3 tw-mb-4"
          >
            <i
              class="bwi bwi-gift tw-text-[36px] tw-text-primary-600"
              aria-hidden="true"
            ></i>
          </div>
          <p bitTypography="h3" class="tw-mb-1 !tw-text-fg-white">
            {{ "churnOfferDiscountHeading" | i18n: discountLabel }}
          </p>
          <span bitTypography="body1" class="tw-text-fg-white tw-font-normal">
            @if (isRecurringAmount) {
              {{
                "churnOfferDiscountBodyRecurringAmount"
                  | i18n
                    : discountLabel
                    : billingIntervalUnit
                    : durationDescription
              }}
            } @else {
              {{
                "churnOfferDiscountBody"
                  | i18n: discountLabel : durationDescription
              }}
            }
          </span>
        </div>
      </bit-card>
    </ng-container>
    <ng-container bitDialogFooter>
      <button
        type="button"
        bitButton
        buttonType="primary"
        [bitAction]="acceptOffer"
        [loading]="loading()"
      >
        {{ "churnOfferAccept" | i18n: discountLabel }}
      </button>
      <button
        type="button"
        bitButton
        buttonType="dangerGhost"
        (click)="decline()"
        [disabled]="loading()"
      >
        {{ "churnOfferDecline" | i18n }}
      </button>
    </ng-container>
  </bit-dialog>
}

`````

### Expected (prettier)

`````html
@if (offerRedeemed()) {
  <bit-dialog>
    <span bitDialogTitle>{{ "churnOfferSuccessTitle" | i18n }}</span>
    <ng-container bitDialogContent>
      <p bitTypography="body1" class="tw-text-muted">
        @if (isRecurringAmount) {
          {{
            "churnOfferSuccessBodyRecurringAmount"
              | i18n: discountLabel : billingIntervalUnit : durationDescription
          }}
        } @else {
          {{
            "churnOfferSuccessBody" | i18n: discountLabel : durationDescription
          }}
        }
      </p>
      <bit-card class="tw-bg-bg-secondary tw-shadow-none">
        <dl class="tw-grid tw-grid-cols-2 tw-gap-y-2 tw-m-0">
          <dt
            bitTypography="body1"
            class="tw-font-normal tw-text-fg-body-subtle"
          >
            {{ "plan" | i18n }}
          </dt>
          <dd bitTypography="body1" class="tw-font-medium tw-m-0 tw-text-right">
            {{ params.planName }}
          </dd>
          <dt
            bitTypography="body1"
            class="tw-font-normal tw-text-fg-body-subtle"
          >
            {{ "status" | i18n }}
          </dt>
          <dd bitTypography="body1" class="tw-font-medium tw-m-0 tw-text-right">
            {{ "active" | i18n }}
          </dd>
          <dt
            bitTypography="body1"
            class="tw-font-normal tw-text-fg-body-subtle"
          >
            {{ "discountApplied" | i18n }}
          </dt>
          <dd bitTypography="body1" class="tw-font-medium tw-m-0 tw-text-right">
            @if (isRecurringAmount) {
              {{
                "churnOfferDiscountSummaryRecurringAmount"
                  | i18n
                    : discountLabelPerInterval
                    : durationLength
                    : durationUnit
              }}
            } @else {
              {{
                "churnOfferDiscountSummary"
                  | i18n: discountLabel : durationLength : durationUnit
              }}
            }
          </dd>
          @if (params.nextChargeDate) {
            <dt
              bitTypography="body1"
              class="tw-font-normal tw-text-fg-body-subtle"
            >
              {{ "nextCharge" | i18n }}
            </dt>
            <dd
              bitTypography="body1"
              class="tw-font-medium tw-m-0 tw-text-right"
            >
              {{ params.nextChargeDate | date: "longDate" }}
            </dd>
          }
        </dl>
      </bit-card>
    </ng-container>
    <ng-container bitDialogFooter>
      <button
        type="button"
        bitButton
        buttonType="primary"
        (click)="closeAfterAccept()"
      >
        {{ "close" | i18n }}
      </button>
    </ng-container>
  </bit-dialog>
} @else {
  <bit-dialog dialogSize="large">
    <h3 bitDialogTitle>{{ "churnOfferTitle" | i18n }}</h3>
    <ng-container bitDialogContent>
      <p class="tw-text-muted" bitTypography="body1">
        {{ "churnOfferPreamble" | i18n }}
        <strong>{{ params.accessEndDate | date: "longDate" }}</strong>{{
          "churnOfferPreambleSuffix" | i18n
        }}
      </p>
      <bit-card
        class="tw-relative tw-overflow-hidden !tw-shadow-none !tw-border-none"
      >
        <img
          src="../../../images/globe-bg.png"
          alt=""
          aria-hidden="true"
          class="tw-absolute tw-inset-0 tw-size-full tw-object-cover tw-pointer-events-none"
        />
        <div class="tw-relative">
          <div
            class="tw-inline-flex tw-items-center tw-justify-center tw-rounded-xl tw-bg-bg-brand-soft tw-p-3 tw-mb-4"
          >
            <i
              class="bwi bwi-gift tw-text-[36px] tw-text-primary-600"
              aria-hidden="true"
            ></i>
          </div>
          <p bitTypography="h3" class="tw-mb-1 !tw-text-fg-white">
            {{ "churnOfferDiscountHeading" | i18n: discountLabel }}
          </p>
          <span bitTypography="body1" class="tw-text-fg-white tw-font-normal">
            @if (isRecurringAmount) {
              {{
                "churnOfferDiscountBodyRecurringAmount"
                  | i18n
                    : discountLabel
                    : billingIntervalUnit
                    : durationDescription
              }}
            } @else {
              {{
                "churnOfferDiscountBody"
                  | i18n: discountLabel : durationDescription
              }}
            }
          </span>
        </div>
      </bit-card>
    </ng-container>
    <ng-container bitDialogFooter>
      <button
        type="button"
        bitButton
        buttonType="primary"
        [bitAction]="acceptOffer"
        [loading]="loading()"
      >
        {{ "churnOfferAccept" | i18n: discountLabel }}
      </button>
      <button
        type="button"
        bitButton
        buttonType="dangerGhost"
        (click)="decline()"
        [disabled]="loading()"
      >
        {{ "churnOfferDecline" | i18n }}
      </button>
    </ng-container>
  </bit-dialog>
}

`````
