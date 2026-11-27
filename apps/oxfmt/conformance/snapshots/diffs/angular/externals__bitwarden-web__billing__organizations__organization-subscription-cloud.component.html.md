# externals/bitwarden-web/billing/organizations/organization-subscription-cloud.component.html

## Option 1

`````json
{"printWidth":80}
`````

### Diff

`````diff
===================================================================
--- prettier
+++ oxfmt
@@ -340,9 +340,10 @@
               org.providerId,
               'billing',
               'subscription',
             ]"
-            >{{ "providerPortal" | i18n }}</a>.
+            >{{ "providerPortal" | i18n }}</a
+          >.
         </p>
       } @else {
         <p bitTypography="body1">
           {{ "billingManagedByProvider" | i18n: org.providerName }}.

`````

### Actual (oxfmt)

`````html
@let subscription =
  subscriptionPreview.hasValue() ? subscriptionPreview.value() : null;
@let org = organization();
@let orgSub = organizationSubscription();
@let billingSubscription = orgSub?.subscription;
@let showSubscription = access()?.showSubscription;
@let showSubscriptionStatus = access()?.showSubscriptionStatus;
@let showSelfHost = access()?.showSelfHost;
@let canUseBillingSync = access()?.canUseBillingSync;
@let showProviderManagedBilling = access()?.showProviderManagedBilling;

<app-header [title]="'subscription' | i18n"></app-header>

<bit-container>
  @if (showSubscription) {
    @if (canEditSubscription()) {
      @if (isFreeOrg()) {
        <strong
          class="tw-flex tw-items-center tw-gap-3 tw-border-0 tw-border-b tw-border-solid tw-border-secondary-300 tw-pb-2"
        >
          {{ "details" | i18n }}
        </strong>
        <bit-table>
          <ng-template body>
            @if (org?.usePasswordManager) {
              <tr bitRow>
                <td bitCell>
                  {{ "passwordManager" | i18n }} -
                  {{ "freeOrganization" | i18n }}
                </td>
                <td bitCell class="tw-text-right">{{ "free" | i18n }}</td>
              </tr>
            }
            @if (org?.useSecretsManager) {
              <tr bitRow>
                <td bitCell>
                  {{ "secretsManager" | i18n }} -
                  {{ "freeOrganization" | i18n }}
                </td>
                <td bitCell class="tw-text-right">{{ "free" | i18n }}</td>
              </tr>
            }
          </ng-template>
        </bit-table>
      } @else {
        @if (
          subscriptionPreview.error() ||
          organizationSubscriptionResource.error()
        ) {
          <billing-error-card
            [title]="'subscriptionDetailsNotLoading' | i18n"
            [description]="'subscriptionDetailsNotLoadingDesc' | i18n"
            [buttonText]="'refresh' | i18n"
            icon="bwi-clear"
            [loading]="
              subscriptionPreview.isLoading() ||
              organizationSubscriptionResource.isLoading()
            "
            (actionClicked)="refreshAll()"
          />
        } @else if (subscriptionPreview.isLoading() || !subscription) {
          <div class="tw-text-center">
            <bit-spinner></bit-spinner>
          </div>
        } @else {
          <billing-subscription-card
            [title]="cardTitle()"
            [subscription]="subscription"
            (callToActionClicked)="handleCardAction($event)"
          >
            <app-organization-scheduled-price-increase-warning
              [organization]="org"
            />
            @if (pendingAnnualUpgrade()) {
              <bit-callout type="info">
                {{
                  "pendingAnnualUpgradeNotice"
                    | i18n
                      : (pendingAnnualUpgrade()?.effectiveDate
                          | date: "mediumDate")
                }}
              </bit-callout>
            }
          </billing-subscription-card>
        }
      }

      @if (showChangePlanButton()) {
        <div class="tw-mt-5">
          <button
            bitButton
            buttonType="secondary"
            type="button"
            (click)="changePlan()"
          >
            {{ "changeBillingPlan" | i18n }}
          </button>
        </div>
      }

      @if (showSecretsManagerSubscribe() && orgSub) {
        <div class="tw-mt-7">
          <sm-subscribe-standalone
            [plan]="orgSub.plan"
            [organization]="org"
            [customerDiscount]="customerDiscount()"
            (onSubscribe)="subscriptionAdjusted()"
          ></sm-subscribe-standalone>
        </div>
      }

      @if (showPrivilegedControlsContactSalesToAdd()) {
        <div class="tw-mt-7" id="privileged-controls-contact-sales-to-add">
          <h2 bitTypography="h2">{{ "privilegedControls" | i18n }}</h2>
          <p bitTypography="body1">
            {{ "privilegedControlsAddThroughSales" | i18n }}
          </p>
          <a
            bitButton
            buttonType="secondary"
            href="https://bitwarden.com/contact/"
            target="_blank"
            rel="noreferrer"
            id="organization-subscription-cloud_button_privileged-controls-contact-sales-add"
          >
            {{ "contactSales" | i18n }}
          </a>
        </div>
      }

      <h2 bitTypography="h2" class="tw-mt-7">
        {{ "manageSubscription" | i18n }}
      </h2>

      @if (customerDiscount()?.id === "sm-standalone") {
        <p bitTypography="body1">
          {{ "smStandaloneTrialSeatCountUpdateMessageFragment1" | i18n }}
          {{ "passwordManager" | i18n }}
          {{ "smStandaloneTrialSeatCountUpdateMessageFragment2" | i18n }}
          <a
            href="https://bitwarden.com/contact/"
            target="_blank"
            rel="noreferrer"
          >
            {{ "contactSupportShort" | i18n }}.
          </a>
        </p>
      }

      <p bitTypography="body1">{{ subscriptionDesc() }}</p>

      @if (canAdjustPasswordManagerSeats()) {
        <h3 bitTypography="h3" class="tw-mt-7">
          {{ "passwordManager" | i18n }}
        </h3>
        <app-adjust-subscription
          [seatPrice]="seatPrice()"
          [organizationId]="organizationId"
          [interval]="billingInterval()"
          [currentSeatCount]="seats()"
          [maxAutoscaleSeats]="maxAutoscaleSeats()"
          (onAdjusted)="subscriptionAdjusted()"
        ></app-adjust-subscription>
      }

      @if (showSelfHost) {
        <h2 bitTypography="h2" class="tw-mt-7">
          {{ "selfHostingTitleProper" | i18n }}
        </h2>
        <p bitTypography="body1">
          {{ "toHostBitwardenOnYourOwnServer" | i18n }}
        </p>
        <div class="tw-flex tw-space-x-2">
          <button
            bitButton
            buttonType="secondary"
            type="button"
            (click)="downloadLicense()"
          >
            {{ "downloadLicense" | i18n }}
          </button>
          @if (canUseBillingSync) {
            <button
              bitButton
              buttonType="secondary"
              type="button"
              (click)="manageBillingSync()"
            >
              {{
                (hasBillingSyncToken()
                  ? "viewBillingToken"
                  : "setUpBillingSync"
                ) | i18n
              }}
            </button>
          }
        </div>
      }

      @if (isSponsoredSubscription()) {
        <button
          bitButton
          buttonType="danger"
          type="button"
          [bitAction]="removeSponsorship"
        >
          {{ "removeSponsorship" | i18n }}
        </button>
      }

      @if (showStorage()) {
        <h4 bitTypography="h4" class="tw-mt-9">{{ "storage" | i18n }}</h4>
        <p bitTypography="body1">
          {{ "subscriptionStorage" | i18n: maxStorageGb() : storageName() }}
        </p>
        <bit-progress-bar [value]="storagePercentage()" variant="success">
          <bit-hint slot="start">{{ storagePercentage() }}%</bit-hint>
        </bit-progress-bar>
        @if (
          billingSubscription && !isSubscriptionCanceledOrPendingCancellation()
        ) {
          <div class="tw-mt-3">
            <div class="tw-flex tw-space-x-2">
              <button
                bitButton
                buttonType="secondary"
                type="button"
                [bitAction]="adjustStorage(true)"
              >
                {{ "addStorage" | i18n }}
              </button>
              <button
                bitButton
                buttonType="secondary"
                type="button"
                [bitAction]="adjustStorage(false)"
              >
                {{ "removeStorage" | i18n }}
              </button>
            </div>
          </div>
        }
      }

      @if (showAdjustSecretsManager()) {
        <h3 bitTypography="h3" class="tw-mt-9">
          {{ "secretsManager" | i18n }}
        </h3>
        <app-sm-adjust-subscription
          [organizationId]="organizationId"
          [options]="smOptions()"
          (onAdjusted)="subscriptionAdjusted()"
        ></app-sm-adjust-subscription>
      }

      @if (
        showAdjustPrivilegedControls() && privilegedControlsOptions();
        as pamOptions
      ) {
        <h3 bitTypography="h3" class="tw-mt-9">
          {{ "privilegedControls" | i18n }}
        </h3>
        <app-privileged-controls-adjust-subscription
          [organizationId]="typedOrganizationId"
          [options]="pamOptions"
          (adjusted)="subscriptionAdjusted()"
        ></app-privileged-controls-adjust-subscription>
      }

      @if (showPrivilegedControlsContactSalesToChange()) {
        <div id="privileged-controls-contact-sales-to-change">
          <h3 bitTypography="h3" class="tw-mt-9">
            {{ "privilegedControls" | i18n }}
          </h3>
          @if (orgSub?.occupiedPamSeats != null && orgSub?.pamSeats != null) {
            <p bitTypography="body1" id="privileged-controls-read-only-seats">
              {{
                "privilegedControlsSeatsInUse"
                  | i18n: orgSub.occupiedPamSeats : orgSub.pamSeats
              }}
            </p>
          }
          <p bitTypography="body1">
            {{ "privilegedControlsChangeThroughSales" | i18n }}
          </p>
          <a
            bitButton
            buttonType="secondary"
            href="https://bitwarden.com/contact/"
            target="_blank"
            rel="noreferrer"
            id="organization-subscription-cloud_button_privileged-controls-contact-sales-change"
          >
            {{ "contactSales" | i18n }}
          </a>
        </div>
      }

      <h2 bitTypography="h2" class="tw-mt-7">
        {{ "additionalOptions" | i18n }}
      </h2>
      <p bitTypography="body1">{{ "additionalOptionsDesc" | i18n }}</p>

      @if (
        billingSubscription && !isSubscriptionCanceledOrPendingCancellation()
      ) {
        <div class="tw-flex tw-space-x-2">
          <button
            bitButton
            buttonType="danger"
            type="button"
            (click)="cancelSubscription()"
          >
            {{ "cancelSubscription" | i18n }}
          </button>
        </div>
      }
    }
  } @else {
    @if (showSubscriptionStatus && orgSub) {
      <dl class="tw-grid tw-grid-flow-col tw-grid-rows-2">
        <dt>{{ "billingPlan" | i18n }}</dt>
        <dd>{{ statusPlanName() }}</dd>
        <dt>{{ "status" | i18n }}</dt>
        <dd>{{ statusLabel() }}</dd>
        <dt>{{ "nextCharge" | i18n }}</dt>
        <dd>{{ nextChargeDate() | date: "mediumDate" }}</dd>
      </dl>
    }

    <h2 bitTypography="h2">{{ "manageSubscription" | i18n }}</h2>

    @if (showProviderManagedBilling && org) {
      @if (org.isProviderUser) {
        <p bitTypography="body1">
          {{ "manageSubscriptionFromThe" | i18n }}
          <a
            [routerLink]="[
              '/providers',
              org.providerId,
              'billing',
              'subscription',
            ]"
            >{{ "providerPortal" | i18n }}</a
          >.
        </p>
      } @else {
        <p bitTypography="body1">
          {{ "billingManagedByProvider" | i18n: org.providerName }}.
          {{ "billingContactProviderForAssistance" | i18n }}.
        </p>
      }
    } @else if (org && !showSubscriptionStatus && !showProviderManagedBilling) {
      <div class="tw-flex tw-flex-col tw-items-center tw-text-info">
        <div class="tw-size-56 tw-content-center">
          <bit-svg [content]="gearIcon" aria-hidden="true"></bit-svg>
        </div>
        <p class="tw-font-medium">
          {{ "billingManagedByProvider" | i18n: org.providerName }}
        </p>
        <p>{{ "billingContactProviderForAssistance" | i18n }}</p>
      </div>
    }

    @if (
      (showSubscriptionStatus && orgSub) || (showProviderManagedBilling && org)
    ) {
      @if (org.hasReseller && resellerSeatsRemaining() != null) {
        <p bitTypography="body1">
          {{
            "seatsRemaining"
              | i18n: resellerSeatsRemaining() ?? 0 : org?.seats ?? 0
          }}
        </p>
      }

      @if (showSelfHost) {
        <h2 bitTypography="h2" class="tw-mt-7">
          {{ "selfHostingTitleProper" | i18n }}
        </h2>
        <p bitTypography="body1">
          {{ "toHostBitwardenOnYourOwnServer" | i18n }}
        </p>
        <div class="tw-flex tw-space-x-2">
          <button
            bitButton
            buttonType="secondary"
            type="button"
            (click)="downloadLicense()"
          >
            {{ "downloadLicense" | i18n }}
          </button>

          @if (canUseBillingSync) {
            <button
              bitButton
              buttonType="secondary"
              type="button"
              (click)="manageBillingSync()"
            >
              {{
                (hasBillingSyncToken()
                  ? "viewBillingToken"
                  : "setUpBillingSync"
                ) | i18n
              }}
            </button>
          }
        </div>
      }
    }
  }
</bit-container>

`````

### Expected (prettier)

`````html
@let subscription =
  subscriptionPreview.hasValue() ? subscriptionPreview.value() : null;
@let org = organization();
@let orgSub = organizationSubscription();
@let billingSubscription = orgSub?.subscription;
@let showSubscription = access()?.showSubscription;
@let showSubscriptionStatus = access()?.showSubscriptionStatus;
@let showSelfHost = access()?.showSelfHost;
@let canUseBillingSync = access()?.canUseBillingSync;
@let showProviderManagedBilling = access()?.showProviderManagedBilling;

<app-header [title]="'subscription' | i18n"></app-header>

<bit-container>
  @if (showSubscription) {
    @if (canEditSubscription()) {
      @if (isFreeOrg()) {
        <strong
          class="tw-flex tw-items-center tw-gap-3 tw-border-0 tw-border-b tw-border-solid tw-border-secondary-300 tw-pb-2"
        >
          {{ "details" | i18n }}
        </strong>
        <bit-table>
          <ng-template body>
            @if (org?.usePasswordManager) {
              <tr bitRow>
                <td bitCell>
                  {{ "passwordManager" | i18n }} -
                  {{ "freeOrganization" | i18n }}
                </td>
                <td bitCell class="tw-text-right">{{ "free" | i18n }}</td>
              </tr>
            }
            @if (org?.useSecretsManager) {
              <tr bitRow>
                <td bitCell>
                  {{ "secretsManager" | i18n }} -
                  {{ "freeOrganization" | i18n }}
                </td>
                <td bitCell class="tw-text-right">{{ "free" | i18n }}</td>
              </tr>
            }
          </ng-template>
        </bit-table>
      } @else {
        @if (
          subscriptionPreview.error() ||
          organizationSubscriptionResource.error()
        ) {
          <billing-error-card
            [title]="'subscriptionDetailsNotLoading' | i18n"
            [description]="'subscriptionDetailsNotLoadingDesc' | i18n"
            [buttonText]="'refresh' | i18n"
            icon="bwi-clear"
            [loading]="
              subscriptionPreview.isLoading() ||
              organizationSubscriptionResource.isLoading()
            "
            (actionClicked)="refreshAll()"
          />
        } @else if (subscriptionPreview.isLoading() || !subscription) {
          <div class="tw-text-center">
            <bit-spinner></bit-spinner>
          </div>
        } @else {
          <billing-subscription-card
            [title]="cardTitle()"
            [subscription]="subscription"
            (callToActionClicked)="handleCardAction($event)"
          >
            <app-organization-scheduled-price-increase-warning
              [organization]="org"
            />
            @if (pendingAnnualUpgrade()) {
              <bit-callout type="info">
                {{
                  "pendingAnnualUpgradeNotice"
                    | i18n
                      : (pendingAnnualUpgrade()?.effectiveDate
                          | date: "mediumDate")
                }}
              </bit-callout>
            }
          </billing-subscription-card>
        }
      }

      @if (showChangePlanButton()) {
        <div class="tw-mt-5">
          <button
            bitButton
            buttonType="secondary"
            type="button"
            (click)="changePlan()"
          >
            {{ "changeBillingPlan" | i18n }}
          </button>
        </div>
      }

      @if (showSecretsManagerSubscribe() && orgSub) {
        <div class="tw-mt-7">
          <sm-subscribe-standalone
            [plan]="orgSub.plan"
            [organization]="org"
            [customerDiscount]="customerDiscount()"
            (onSubscribe)="subscriptionAdjusted()"
          ></sm-subscribe-standalone>
        </div>
      }

      @if (showPrivilegedControlsContactSalesToAdd()) {
        <div class="tw-mt-7" id="privileged-controls-contact-sales-to-add">
          <h2 bitTypography="h2">{{ "privilegedControls" | i18n }}</h2>
          <p bitTypography="body1">
            {{ "privilegedControlsAddThroughSales" | i18n }}
          </p>
          <a
            bitButton
            buttonType="secondary"
            href="https://bitwarden.com/contact/"
            target="_blank"
            rel="noreferrer"
            id="organization-subscription-cloud_button_privileged-controls-contact-sales-add"
          >
            {{ "contactSales" | i18n }}
          </a>
        </div>
      }

      <h2 bitTypography="h2" class="tw-mt-7">
        {{ "manageSubscription" | i18n }}
      </h2>

      @if (customerDiscount()?.id === "sm-standalone") {
        <p bitTypography="body1">
          {{ "smStandaloneTrialSeatCountUpdateMessageFragment1" | i18n }}
          {{ "passwordManager" | i18n }}
          {{ "smStandaloneTrialSeatCountUpdateMessageFragment2" | i18n }}
          <a
            href="https://bitwarden.com/contact/"
            target="_blank"
            rel="noreferrer"
          >
            {{ "contactSupportShort" | i18n }}.
          </a>
        </p>
      }

      <p bitTypography="body1">{{ subscriptionDesc() }}</p>

      @if (canAdjustPasswordManagerSeats()) {
        <h3 bitTypography="h3" class="tw-mt-7">
          {{ "passwordManager" | i18n }}
        </h3>
        <app-adjust-subscription
          [seatPrice]="seatPrice()"
          [organizationId]="organizationId"
          [interval]="billingInterval()"
          [currentSeatCount]="seats()"
          [maxAutoscaleSeats]="maxAutoscaleSeats()"
          (onAdjusted)="subscriptionAdjusted()"
        ></app-adjust-subscription>
      }

      @if (showSelfHost) {
        <h2 bitTypography="h2" class="tw-mt-7">
          {{ "selfHostingTitleProper" | i18n }}
        </h2>
        <p bitTypography="body1">
          {{ "toHostBitwardenOnYourOwnServer" | i18n }}
        </p>
        <div class="tw-flex tw-space-x-2">
          <button
            bitButton
            buttonType="secondary"
            type="button"
            (click)="downloadLicense()"
          >
            {{ "downloadLicense" | i18n }}
          </button>
          @if (canUseBillingSync) {
            <button
              bitButton
              buttonType="secondary"
              type="button"
              (click)="manageBillingSync()"
            >
              {{
                (hasBillingSyncToken()
                  ? "viewBillingToken"
                  : "setUpBillingSync"
                ) | i18n
              }}
            </button>
          }
        </div>
      }

      @if (isSponsoredSubscription()) {
        <button
          bitButton
          buttonType="danger"
          type="button"
          [bitAction]="removeSponsorship"
        >
          {{ "removeSponsorship" | i18n }}
        </button>
      }

      @if (showStorage()) {
        <h4 bitTypography="h4" class="tw-mt-9">{{ "storage" | i18n }}</h4>
        <p bitTypography="body1">
          {{ "subscriptionStorage" | i18n: maxStorageGb() : storageName() }}
        </p>
        <bit-progress-bar [value]="storagePercentage()" variant="success">
          <bit-hint slot="start">{{ storagePercentage() }}%</bit-hint>
        </bit-progress-bar>
        @if (
          billingSubscription && !isSubscriptionCanceledOrPendingCancellation()
        ) {
          <div class="tw-mt-3">
            <div class="tw-flex tw-space-x-2">
              <button
                bitButton
                buttonType="secondary"
                type="button"
                [bitAction]="adjustStorage(true)"
              >
                {{ "addStorage" | i18n }}
              </button>
              <button
                bitButton
                buttonType="secondary"
                type="button"
                [bitAction]="adjustStorage(false)"
              >
                {{ "removeStorage" | i18n }}
              </button>
            </div>
          </div>
        }
      }

      @if (showAdjustSecretsManager()) {
        <h3 bitTypography="h3" class="tw-mt-9">
          {{ "secretsManager" | i18n }}
        </h3>
        <app-sm-adjust-subscription
          [organizationId]="organizationId"
          [options]="smOptions()"
          (onAdjusted)="subscriptionAdjusted()"
        ></app-sm-adjust-subscription>
      }

      @if (
        showAdjustPrivilegedControls() && privilegedControlsOptions();
        as pamOptions
      ) {
        <h3 bitTypography="h3" class="tw-mt-9">
          {{ "privilegedControls" | i18n }}
        </h3>
        <app-privileged-controls-adjust-subscription
          [organizationId]="typedOrganizationId"
          [options]="pamOptions"
          (adjusted)="subscriptionAdjusted()"
        ></app-privileged-controls-adjust-subscription>
      }

      @if (showPrivilegedControlsContactSalesToChange()) {
        <div id="privileged-controls-contact-sales-to-change">
          <h3 bitTypography="h3" class="tw-mt-9">
            {{ "privilegedControls" | i18n }}
          </h3>
          @if (orgSub?.occupiedPamSeats != null && orgSub?.pamSeats != null) {
            <p bitTypography="body1" id="privileged-controls-read-only-seats">
              {{
                "privilegedControlsSeatsInUse"
                  | i18n: orgSub.occupiedPamSeats : orgSub.pamSeats
              }}
            </p>
          }
          <p bitTypography="body1">
            {{ "privilegedControlsChangeThroughSales" | i18n }}
          </p>
          <a
            bitButton
            buttonType="secondary"
            href="https://bitwarden.com/contact/"
            target="_blank"
            rel="noreferrer"
            id="organization-subscription-cloud_button_privileged-controls-contact-sales-change"
          >
            {{ "contactSales" | i18n }}
          </a>
        </div>
      }

      <h2 bitTypography="h2" class="tw-mt-7">
        {{ "additionalOptions" | i18n }}
      </h2>
      <p bitTypography="body1">{{ "additionalOptionsDesc" | i18n }}</p>

      @if (
        billingSubscription && !isSubscriptionCanceledOrPendingCancellation()
      ) {
        <div class="tw-flex tw-space-x-2">
          <button
            bitButton
            buttonType="danger"
            type="button"
            (click)="cancelSubscription()"
          >
            {{ "cancelSubscription" | i18n }}
          </button>
        </div>
      }
    }
  } @else {
    @if (showSubscriptionStatus && orgSub) {
      <dl class="tw-grid tw-grid-flow-col tw-grid-rows-2">
        <dt>{{ "billingPlan" | i18n }}</dt>
        <dd>{{ statusPlanName() }}</dd>
        <dt>{{ "status" | i18n }}</dt>
        <dd>{{ statusLabel() }}</dd>
        <dt>{{ "nextCharge" | i18n }}</dt>
        <dd>{{ nextChargeDate() | date: "mediumDate" }}</dd>
      </dl>
    }

    <h2 bitTypography="h2">{{ "manageSubscription" | i18n }}</h2>

    @if (showProviderManagedBilling && org) {
      @if (org.isProviderUser) {
        <p bitTypography="body1">
          {{ "manageSubscriptionFromThe" | i18n }}
          <a
            [routerLink]="[
              '/providers',
              org.providerId,
              'billing',
              'subscription',
            ]"
            >{{ "providerPortal" | i18n }}</a>.
        </p>
      } @else {
        <p bitTypography="body1">
          {{ "billingManagedByProvider" | i18n: org.providerName }}.
          {{ "billingContactProviderForAssistance" | i18n }}.
        </p>
      }
    } @else if (org && !showSubscriptionStatus && !showProviderManagedBilling) {
      <div class="tw-flex tw-flex-col tw-items-center tw-text-info">
        <div class="tw-size-56 tw-content-center">
          <bit-svg [content]="gearIcon" aria-hidden="true"></bit-svg>
        </div>
        <p class="tw-font-medium">
          {{ "billingManagedByProvider" | i18n: org.providerName }}
        </p>
        <p>{{ "billingContactProviderForAssistance" | i18n }}</p>
      </div>
    }

    @if (
      (showSubscriptionStatus && orgSub) || (showProviderManagedBilling && org)
    ) {
      @if (org.hasReseller && resellerSeatsRemaining() != null) {
        <p bitTypography="body1">
          {{
            "seatsRemaining"
              | i18n: resellerSeatsRemaining() ?? 0 : org?.seats ?? 0
          }}
        </p>
      }

      @if (showSelfHost) {
        <h2 bitTypography="h2" class="tw-mt-7">
          {{ "selfHostingTitleProper" | i18n }}
        </h2>
        <p bitTypography="body1">
          {{ "toHostBitwardenOnYourOwnServer" | i18n }}
        </p>
        <div class="tw-flex tw-space-x-2">
          <button
            bitButton
            buttonType="secondary"
            type="button"
            (click)="downloadLicense()"
          >
            {{ "downloadLicense" | i18n }}
          </button>

          @if (canUseBillingSync) {
            <button
              bitButton
              buttonType="secondary"
              type="button"
              (click)="manageBillingSync()"
            >
              {{
                (hasBillingSyncToken()
                  ? "viewBillingToken"
                  : "setUpBillingSync"
                ) | i18n
              }}
            </button>
          }
        </div>
      }
    }
  }
</bit-container>

`````
