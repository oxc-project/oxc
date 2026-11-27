# externals/bitwarden-web/billing/organizations/change-plan-dialog.component.html

## Option 1

`````json
{"printWidth":80}
`````

### Diff

`````diff
===================================================================
--- prettier
+++ oxfmt
@@ -150,10 +150,9 @@
                         : (((this.sub.useSecretsManager
                             ? selectableProduct.SecretsManager.seatPrice
                             : 0) +
                             selectableProduct.PasswordManager.seatPrice) /
-                            (selectableProduct.isAnnual ? 12 : 1)
-                            | currency: "$")
+                          (selectableProduct.isAnnual ? 12 : 1) | currency: "$")
                   }}
                 </b>
                 <span class="tw-text-xs tw-px-0">
                   /{{ "monthPerMember" | i18n }}</span
@@ -255,9 +254,9 @@
           <ng-template #nonEnterprisePlans>
             <ng-container
               *ngIf="
                 selectableProduct.productTier === productTypes.Teams &&
-                  teamsStarterPlanIsAvailable;
+                teamsStarterPlanIsAvailable;
                 else fullFeatureList
               "
             >
               <ul class="tw-px-2 tw-pb-2 tw-list-inside tw-mb-0 tw-text-xs">

`````

### Actual (oxfmt)

`````html
<form
  [formGroup]="formGroup"
  [bitSubmit]="submit"
  bit-dialog
  dialogSize="large"
  [loading]="loading"
>
  <span bitDialogTitle class="tw-font-medium">
    {{ dialogHeaderName }}
  </span>
  <div bitDialogContent>
    <p>{{ "upgradePlans" | i18n }}</p>
    <div class="tw-mb-3 tw-flex tw-justify-between">
      <span
        [hidden]="isSubscriptionCanceled"
        class="tw-text-lg tw-pr-1 tw-font-medium"
        >{{ "selectAPlan" | i18n }}</span
      >
      <!-- Discount Badge -->
      <div class="tw-flex tw-items-center tw-gap-2">
        <!-- Plan Interval Toggle -->
        <div class="tw-inline-block" *ngIf="!isSubscriptionCanceled">
          <bit-toggle-group
            [selected]="selectedInterval"
            (selectedChange)="updateInterval($event)"
            [label]="'planBillingInterval' | i18n"
          >
            <bit-toggle
              *ngFor="
                let planInterval of getPlanIntervals();
                trackBy: optimizedNgForRender;
                let i = index
              "
              [value]="planInterval.value"
            >
              {{ planInterval.name }}
            </bit-toggle>
          </bit-toggle-group>
        </div>
      </div>
    </div>
    <!-- Plan Selection Cards -->
    <ng-container *ngIf="!loading && !selfHosted && this.passwordManagerPlans">
      <div
        class="tw-grid tw-grid-flow-col tw-gap-4 tw-mb-4"
        [class]="'tw-grid-cols-' + selectableProducts.length"
      >
        <bit-card
          *ngFor="
            let selectableProduct of selectableProducts;
            trackBy: manageSelectableProduct;
            let i = index
          "
          [ngClass]="getPlanCardContainerClasses(selectableProduct, i)"
          (click)="selectPlan(selectableProduct)"
          [attr.tabindex]="focusedIndex !== i || isCardDisabled(i) ? '-1' : '0'"
          (keyup)="onKeydown($event, i)"
          (focus)="onFocus(i)"
          [attr.aria-disabled]="isCardDisabled(i)"
          [id]="i + 'a_plan_card'"
        >
          <div class="tw-relative">
            <div
              *ngIf="
                selectableProduct.productTier === productTypes.Enterprise &&
                !isSubscriptionCanceled
              "
              class="tw-bg-secondary-100 tw-text-center !tw-border-0 tw-text-sm tw-font-medium tw-py-1"
              [ngClass]="{
                'tw-bg-primary-700 !tw-text-contrast':
                  selectableProduct === selectedPlan,
                'tw-bg-secondary-100': !(selectableProduct === selectedPlan),
              }"
            >
              {{ "recommended" | i18n }}
            </div>
            <div
              class="tw-px-2 tw-pb-[4px]"
              [ngClass]="{
                'tw-py-1': !(selectableProduct === selectedPlan),
                'tw-py-0': selectableProduct === selectedPlan,
              }"
            >
              <h3
                class="tw-text-[1.25rem] tw-mt-[6px] tw-font-medium tw-mb-0 tw-leading-[2rem] tw-flex tw-items-center"
              >
                <span class="tw-capitalize tw-whitespace-nowrap">{{
                  selectableProduct.nameLocalizationKey | i18n
                }}</span>
                <span
                  bitBadge
                  variant="secondary"
                  *ngIf="selectableProduct === currentPlan"
                  class="tw-ml-2 tw-align-middle"
                >
                  {{ "current" | i18n }}</span
                >
              </h3>
              <span *ngIf="selectableProduct.productTier != productTypes.Free">
                <ng-container
                  *ngIf="
                    selectableProduct.PasswordManager.basePrice &&
                    !acceptingSponsorship
                  "
                >
                  <b class="tw-text-lg tw-font-medium">
                    {{
                      (selectableProduct.isAnnual
                        ? selectableProduct.PasswordManager.basePrice / 12
                        : selectableProduct.PasswordManager.basePrice
                      ) | currency: "$"
                    }}
                  </b>
                  <span class="tw-text-xs tw-px-0">
                    /{{
                      selectableProduct.productTier === productTypes.Families
                        ? "month"
                        : ("monthPerMember" | i18n)
                    }}</span
                  >
                  <b class="tw-text-sm tw-font-medium">
                    <ng-container
                      *ngIf="
                        selectableProduct.PasswordManager
                          .hasAdditionalSeatsOption
                      "
                    >
                      {{ ("additionalUsers" | i18n).toLowerCase() }}
                      {{
                        (selectableProduct.isAnnual
                          ? selectableProduct.PasswordManager.seatPrice / 12
                          : selectableProduct.PasswordManager.seatPrice
                        ) | currency: "$"
                      }}
                      /{{ selectedPlanInterval | i18n }}
                    </ng-container>
                  </b>
                </ng-container>
              </span>
              <span
                *ngIf="
                  !selectableProduct.PasswordManager.basePrice &&
                  selectableProduct.PasswordManager.hasAdditionalSeatsOption
                "
              >
                <b class="tw-text-lg tw-font-medium"
                  >{{
                    "costPerMember"
                      | i18n
                        : (((this.sub.useSecretsManager
                            ? selectableProduct.SecretsManager.seatPrice
                            : 0) +
                            selectableProduct.PasswordManager.seatPrice) /
                          (selectableProduct.isAnnual ? 12 : 1) | currency: "$")
                  }}
                </b>
                <span class="tw-text-xs tw-px-0">
                  /{{ "monthPerMember" | i18n }}</span
                >
              </span>
              <span *ngIf="selectableProduct.productTier == productTypes.Free"
                >{{ "freeForever" | i18n }}
              </span>
            </div>
          </div>

          <ng-container
            *ngIf="
              selectableProduct.productTier === productTypes.Enterprise;
              else nonEnterprisePlans
            "
          >
            <p
              class="tw-text-xs tw-px-2 tw-font-medium tw-mb-1"
              *ngIf="organization.useSecretsManager"
            >
              {{ "bitwardenPasswordManager" | i18n }}
            </p>
            <p class="tw-text-xs tw-px-2 tw-mb-1">
              {{ "enterprisePlanUpgradeMessage" | i18n }}
            </p>

            <ul class="bwi-ul tw-text-xs">
              <li>
                <i
                  class="bwi bwi-check tw-text-success bwi-li"
                  aria-hidden="true"
                ></i>
                {{ "includeEnterprisePolicies" | i18n }}
              </li>
              <li>
                <i
                  class="bwi bwi-check tw-text-success bwi-li"
                  aria-hidden="true"
                ></i>
                {{ "passwordLessSso" | i18n }}
              </li>
              <li>
                <i
                  class="bwi bwi-check tw-text-success bwi-li"
                  aria-hidden="true"
                ></i>
                {{ "accountRecovery" | i18n }}
              </li>
              <li>
                <i
                  class="bwi bwi-check tw-text-success bwi-li"
                  aria-hidden="true"
                ></i>
                {{ "customRoles" | i18n }}
              </li>
            </ul>

            <p
              class="tw-text-xs tw-px-2 tw-font-medium tw-mb-1"
              *ngIf="organization.useSecretsManager"
            >
              {{ "bitwardenSecretsManager" | i18n }}
            </p>
            <ul
              class="bwi-ul tw-text-xs"
              *ngIf="organization.useSecretsManager"
            >
              <li>
                <i
                  class="bwi bwi-check tw-text-success bwi-li"
                  aria-hidden="true"
                ></i>
                {{ "unlimitedSecretsStorage" | i18n }}
              </li>
              <li>
                <i
                  class="bwi bwi-check tw-text-success bwi-li"
                  aria-hidden="true"
                ></i>
                {{ "unlimitedUsers" | i18n }}
              </li>
              <li>
                <i
                  class="bwi bwi-check tw-text-success bwi-li"
                  aria-hidden="true"
                ></i>
                {{ "unlimitedProjects" | i18n }}
              </li>
              <li>
                <i
                  class="bwi bwi-check tw-text-success bwi-li"
                  aria-hidden="true"
                ></i>
                {{ "UpTo50MachineAccounts" | i18n }}
              </li>
            </ul>
          </ng-container>
          <ng-template #nonEnterprisePlans>
            <ng-container
              *ngIf="
                selectableProduct.productTier === productTypes.Teams &&
                teamsStarterPlanIsAvailable;
                else fullFeatureList
              "
            >
              <ul class="tw-px-2 tw-pb-2 tw-list-inside tw-mb-0 tw-text-xs">
                <li>{{ "includeAllTeamsStarterFeatures" | i18n }}</li>
                <li>{{ "chooseMonthlyOrAnnualBilling" | i18n }}</li>
                <li>{{ "abilityToAddMoreThanNMembers" | i18n: 10 }}</li>
              </ul>
            </ng-container>
            <ng-template #fullFeatureList>
              <p
                class="tw-text-xs tw-px-2 tw-font-medium tw-mb-1"
                *ngIf="organization.useSecretsManager"
              >
                {{ "bitwardenPasswordManager" | i18n }}
              </p>
              <p
                *ngIf="selectableProduct.productTier === productTypes.Teams"
                class="tw-text-xs tw-px-2 tw-mb-1"
              >
                {{ "teamsPlanUpgradeMessage" | i18n }}
              </p>
              <p
                *ngIf="selectableProduct.productTier === productTypes.Families"
                class="tw-text-xs tw-px-2 tw-mb-1"
              >
                {{ "familyPlanUpgradeMessage" | i18n }}
              </p>
              <ul
                class="bwi-ul tw-text-xs tw-mb-1"
                *ngIf="selectableProduct.productTier == productTypes.Families"
              >
                <li>
                  <i
                    class="bwi bwi-check tw-text-success bwi-li"
                    aria-hidden="true"
                  ></i>
                  {{ "premiumAccounts" | i18n }}
                </li>
                <li>
                  <i
                    class="bwi bwi-check tw-text-success bwi-li"
                    aria-hidden="true"
                  ></i>
                  {{ "unlimitedSharing" | i18n }}
                </li>
                <li>
                  <i
                    class="bwi bwi-check tw-text-success bwi-li"
                    aria-hidden="true"
                  ></i>
                  {{
                    "unlimitedCollections" | vfo1I18n: "unlimitedSharedFolders"
                  }}
                </li>
              </ul>
              <ul
                class="bwi-ul tw-text-xs"
                *ngIf="selectableProduct.productTier == productTypes.Teams"
              >
                <li>
                  <i
                    class="bwi bwi-check tw-text-success bwi-li"
                    aria-hidden="true"
                  ></i>
                  {{ "secureDataSharing" | i18n }}
                </li>
                <li>
                  <i
                    class="bwi bwi-check tw-text-success bwi-li"
                    aria-hidden="true"
                  ></i>
                  {{ "eventLogMonitoring" | i18n }}
                </li>
                <li>
                  <i
                    class="bwi bwi-check tw-text-success bwi-li"
                    aria-hidden="true"
                  ></i>
                  {{ "directoryIntegration" | i18n }}
                </li>
              </ul>
              <p
                class="tw-text-xs tw-px-2 tw-font-medium tw-mb-1"
                *ngIf="
                  organization.useSecretsManager &&
                  selectableProduct.productTier !== productTypes.Families
                "
              >
                {{ "bitwardenSecretsManager" | i18n }}
              </p>
              <ul
                class="bwi-ul tw-text-xs"
                *ngIf="
                  organization.useSecretsManager &&
                  selectableProduct.productTier == productTypes.Teams
                "
              >
                <li>
                  <i
                    class="bwi bwi-check tw-text-success bwi-li"
                    aria-hidden="true"
                  ></i>
                  {{ "unlimitedSecretsStorage" | i18n }}
                </li>
                <li>
                  <i
                    class="bwi bwi-check tw-text-success bwi-li"
                    aria-hidden="true"
                  ></i>
                  {{ "unlimitedProjects" | i18n }}
                </li>
                <li>
                  <i
                    class="bwi bwi-check tw-text-success bwi-li"
                    aria-hidden="true"
                  ></i>
                  {{ "UpTo20MachineAccounts" | i18n }}
                </li>
              </ul>
            </ng-template>
          </ng-template>
        </bit-card>
      </div>
      <br />
      <bit-callout
        *ngIf="organization.useSecretsManager && !isSecretsManagerTrial()"
        type="info"
        title="SECRETS MANAGER SUBSCRIPTION"
      >
        {{ "secretsManagerSubscriptionInfo" | i18n }}
      </bit-callout>
      <bit-callout
        *ngIf="organization.useSecretsManager && isSecretsManagerTrial()"
        type="info"
        title="PASSWORD MANAGER SUBSCRIPTION"
      >
        {{ "secretsManagerComplimentaryPasswordManager" | i18n }}
      </bit-callout>
      <br />
    </ng-container>
    <!-- Payment info -->
    <ng-container
      *ngIf="
        formGroup.value.productTier !== productTypes.Free ||
        isSubscriptionCanceled
      "
    >
      <h2 bitTypography="h4">{{ "paymentMethod" | i18n }}</h2>
      <p *ngIf="!showPayment && !!paymentMethod && !isSubscriptionCanceled">
        @switch (paymentMethod.type) {
          @case ("bankAccount") {
            <i class="bwi bwi-fw bwi-billing"></i>
            {{ paymentMethod.bankName }}, *{{ paymentMethod.last4 }}
            @if (paymentMethod.hostedVerificationUrl) {
              <span>- {{ "unverified" | i18n }}</span>
            }
            <span
              class="tw-ml-2 tw-text-primary-600 tw-cursor-pointer"
              (click)="toggleShowPayment()"
            >
              {{ "changePaymentMethod" | i18n }}
            </span>
          }
          @case ("card") {
            <p class="tw-flex tw-items-center tw-gap-2">
              @let cardBrandIcon = getCardBrandIcon();
              @if (cardBrandIcon !== null) {
                <i class="bwi bwi-fw credit-card-icon {{ cardBrandIcon }}"></i>
              } @else {
                <i class="bwi bwi-fw bwi-credit-card"></i>
              }
              {{ paymentMethod.brand | titlecase }}, *{{ paymentMethod.last4 }},
              {{ paymentMethod.expiration }}
              <span
                class="tw-ml-2 tw-text-primary-600 tw-cursor-pointer"
                (click)="toggleShowPayment()"
              >
                {{ "changePaymentMethod" | i18n }}
              </span>
            </p>
          }
          @case ("payPal") {
            <i class="bwi bwi-fw bwi-paypal tw-text-primary-600"></i>
            {{ paymentMethod.email }}
            <span
              class="tw-ml-2 tw-text-primary-600 tw-cursor-pointer"
              (click)="toggleShowPayment()"
            >
              {{ "changePaymentMethod" | i18n }}
            </span>
          }
        }
        <!-- FIXME: Empty anchor element needs content or should be removed -->
        <!-- eslint-disable-next-line @angular-eslint/template/elements-content -->
        <a></a>
      </p>
      <ng-container *ngIf="canUpdatePaymentInformation()">
        <app-enter-payment-method
          [group]="billingFormGroup.controls.paymentMethod"
        >
        </app-enter-payment-method>
        <app-enter-billing-address
          [group]="billingFormGroup.controls.billingAddress"
          [scenario]="{ type: 'checkout', supportsTaxId }"
        >
        </app-enter-billing-address>
      </ng-container>
      @if (planChangeCart.isLoading()) {
        <div class="tw-mt-4 tw-text-center">
          <bit-spinner></bit-spinner>
        </div>
      } @else if (planChangeCart.error() != null) {
        <bit-callout type="danger" class="tw-mt-4 tw-block">
          {{ previewErrorMessageKey | i18n }}
        </bit-callout>
      } @else if (planChangeCart.hasValue()) {
        <div class="tw-mt-4">
          <billing-cart-summary [cart]="planChangeCart.value()" />
          @if (isFreeUpgradeWithTrial) {
            <bit-callout
              type="info"
              [title]="null"
              [accessibleName]="'trial' | i18n"
              class="tw-mt-4 tw-block"
            >
              {{ "planChangeTrialIncluded" | i18n: trialLengthDays }}
            </bit-callout>
          } @else if (isTrialing && remainingTrialDays > 0) {
            <bit-callout
              type="info"
              [title]="null"
              [accessibleName]="'trial' | i18n"
              class="tw-mt-4 tw-block"
            >
              {{
                trialRemainingMessageKey
                  | i18n
                    : remainingTrialDays
                    : (trialEndDate | date: "MMM. d, y")
              }}
            </bit-callout>
          }
        </div>
      }
    </ng-container>
  </div>
  <ng-container bitDialogFooter>
    <button
      bitButton
      bitFormButton
      buttonType="primary"
      type="submit"
      [disabled]="isSubmitDisabled"
    >
      {{ submitButtonLabel }}
    </button>
    <button
      bitButton
      buttonType="secondary"
      type="button"
      [bitDialogClose]="ResultType.Closed"
    >
      {{ "cancel" | i18n }}
    </button>
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
  [loading]="loading"
>
  <span bitDialogTitle class="tw-font-medium">
    {{ dialogHeaderName }}
  </span>
  <div bitDialogContent>
    <p>{{ "upgradePlans" | i18n }}</p>
    <div class="tw-mb-3 tw-flex tw-justify-between">
      <span
        [hidden]="isSubscriptionCanceled"
        class="tw-text-lg tw-pr-1 tw-font-medium"
        >{{ "selectAPlan" | i18n }}</span
      >
      <!-- Discount Badge -->
      <div class="tw-flex tw-items-center tw-gap-2">
        <!-- Plan Interval Toggle -->
        <div class="tw-inline-block" *ngIf="!isSubscriptionCanceled">
          <bit-toggle-group
            [selected]="selectedInterval"
            (selectedChange)="updateInterval($event)"
            [label]="'planBillingInterval' | i18n"
          >
            <bit-toggle
              *ngFor="
                let planInterval of getPlanIntervals();
                trackBy: optimizedNgForRender;
                let i = index
              "
              [value]="planInterval.value"
            >
              {{ planInterval.name }}
            </bit-toggle>
          </bit-toggle-group>
        </div>
      </div>
    </div>
    <!-- Plan Selection Cards -->
    <ng-container *ngIf="!loading && !selfHosted && this.passwordManagerPlans">
      <div
        class="tw-grid tw-grid-flow-col tw-gap-4 tw-mb-4"
        [class]="'tw-grid-cols-' + selectableProducts.length"
      >
        <bit-card
          *ngFor="
            let selectableProduct of selectableProducts;
            trackBy: manageSelectableProduct;
            let i = index
          "
          [ngClass]="getPlanCardContainerClasses(selectableProduct, i)"
          (click)="selectPlan(selectableProduct)"
          [attr.tabindex]="focusedIndex !== i || isCardDisabled(i) ? '-1' : '0'"
          (keyup)="onKeydown($event, i)"
          (focus)="onFocus(i)"
          [attr.aria-disabled]="isCardDisabled(i)"
          [id]="i + 'a_plan_card'"
        >
          <div class="tw-relative">
            <div
              *ngIf="
                selectableProduct.productTier === productTypes.Enterprise &&
                !isSubscriptionCanceled
              "
              class="tw-bg-secondary-100 tw-text-center !tw-border-0 tw-text-sm tw-font-medium tw-py-1"
              [ngClass]="{
                'tw-bg-primary-700 !tw-text-contrast':
                  selectableProduct === selectedPlan,
                'tw-bg-secondary-100': !(selectableProduct === selectedPlan),
              }"
            >
              {{ "recommended" | i18n }}
            </div>
            <div
              class="tw-px-2 tw-pb-[4px]"
              [ngClass]="{
                'tw-py-1': !(selectableProduct === selectedPlan),
                'tw-py-0': selectableProduct === selectedPlan,
              }"
            >
              <h3
                class="tw-text-[1.25rem] tw-mt-[6px] tw-font-medium tw-mb-0 tw-leading-[2rem] tw-flex tw-items-center"
              >
                <span class="tw-capitalize tw-whitespace-nowrap">{{
                  selectableProduct.nameLocalizationKey | i18n
                }}</span>
                <span
                  bitBadge
                  variant="secondary"
                  *ngIf="selectableProduct === currentPlan"
                  class="tw-ml-2 tw-align-middle"
                >
                  {{ "current" | i18n }}</span
                >
              </h3>
              <span *ngIf="selectableProduct.productTier != productTypes.Free">
                <ng-container
                  *ngIf="
                    selectableProduct.PasswordManager.basePrice &&
                    !acceptingSponsorship
                  "
                >
                  <b class="tw-text-lg tw-font-medium">
                    {{
                      (selectableProduct.isAnnual
                        ? selectableProduct.PasswordManager.basePrice / 12
                        : selectableProduct.PasswordManager.basePrice
                      ) | currency: "$"
                    }}
                  </b>
                  <span class="tw-text-xs tw-px-0">
                    /{{
                      selectableProduct.productTier === productTypes.Families
                        ? "month"
                        : ("monthPerMember" | i18n)
                    }}</span
                  >
                  <b class="tw-text-sm tw-font-medium">
                    <ng-container
                      *ngIf="
                        selectableProduct.PasswordManager
                          .hasAdditionalSeatsOption
                      "
                    >
                      {{ ("additionalUsers" | i18n).toLowerCase() }}
                      {{
                        (selectableProduct.isAnnual
                          ? selectableProduct.PasswordManager.seatPrice / 12
                          : selectableProduct.PasswordManager.seatPrice
                        ) | currency: "$"
                      }}
                      /{{ selectedPlanInterval | i18n }}
                    </ng-container>
                  </b>
                </ng-container>
              </span>
              <span
                *ngIf="
                  !selectableProduct.PasswordManager.basePrice &&
                  selectableProduct.PasswordManager.hasAdditionalSeatsOption
                "
              >
                <b class="tw-text-lg tw-font-medium"
                  >{{
                    "costPerMember"
                      | i18n
                        : (((this.sub.useSecretsManager
                            ? selectableProduct.SecretsManager.seatPrice
                            : 0) +
                            selectableProduct.PasswordManager.seatPrice) /
                            (selectableProduct.isAnnual ? 12 : 1)
                            | currency: "$")
                  }}
                </b>
                <span class="tw-text-xs tw-px-0">
                  /{{ "monthPerMember" | i18n }}</span
                >
              </span>
              <span *ngIf="selectableProduct.productTier == productTypes.Free"
                >{{ "freeForever" | i18n }}
              </span>
            </div>
          </div>

          <ng-container
            *ngIf="
              selectableProduct.productTier === productTypes.Enterprise;
              else nonEnterprisePlans
            "
          >
            <p
              class="tw-text-xs tw-px-2 tw-font-medium tw-mb-1"
              *ngIf="organization.useSecretsManager"
            >
              {{ "bitwardenPasswordManager" | i18n }}
            </p>
            <p class="tw-text-xs tw-px-2 tw-mb-1">
              {{ "enterprisePlanUpgradeMessage" | i18n }}
            </p>

            <ul class="bwi-ul tw-text-xs">
              <li>
                <i
                  class="bwi bwi-check tw-text-success bwi-li"
                  aria-hidden="true"
                ></i>
                {{ "includeEnterprisePolicies" | i18n }}
              </li>
              <li>
                <i
                  class="bwi bwi-check tw-text-success bwi-li"
                  aria-hidden="true"
                ></i>
                {{ "passwordLessSso" | i18n }}
              </li>
              <li>
                <i
                  class="bwi bwi-check tw-text-success bwi-li"
                  aria-hidden="true"
                ></i>
                {{ "accountRecovery" | i18n }}
              </li>
              <li>
                <i
                  class="bwi bwi-check tw-text-success bwi-li"
                  aria-hidden="true"
                ></i>
                {{ "customRoles" | i18n }}
              </li>
            </ul>

            <p
              class="tw-text-xs tw-px-2 tw-font-medium tw-mb-1"
              *ngIf="organization.useSecretsManager"
            >
              {{ "bitwardenSecretsManager" | i18n }}
            </p>
            <ul
              class="bwi-ul tw-text-xs"
              *ngIf="organization.useSecretsManager"
            >
              <li>
                <i
                  class="bwi bwi-check tw-text-success bwi-li"
                  aria-hidden="true"
                ></i>
                {{ "unlimitedSecretsStorage" | i18n }}
              </li>
              <li>
                <i
                  class="bwi bwi-check tw-text-success bwi-li"
                  aria-hidden="true"
                ></i>
                {{ "unlimitedUsers" | i18n }}
              </li>
              <li>
                <i
                  class="bwi bwi-check tw-text-success bwi-li"
                  aria-hidden="true"
                ></i>
                {{ "unlimitedProjects" | i18n }}
              </li>
              <li>
                <i
                  class="bwi bwi-check tw-text-success bwi-li"
                  aria-hidden="true"
                ></i>
                {{ "UpTo50MachineAccounts" | i18n }}
              </li>
            </ul>
          </ng-container>
          <ng-template #nonEnterprisePlans>
            <ng-container
              *ngIf="
                selectableProduct.productTier === productTypes.Teams &&
                  teamsStarterPlanIsAvailable;
                else fullFeatureList
              "
            >
              <ul class="tw-px-2 tw-pb-2 tw-list-inside tw-mb-0 tw-text-xs">
                <li>{{ "includeAllTeamsStarterFeatures" | i18n }}</li>
                <li>{{ "chooseMonthlyOrAnnualBilling" | i18n }}</li>
                <li>{{ "abilityToAddMoreThanNMembers" | i18n: 10 }}</li>
              </ul>
            </ng-container>
            <ng-template #fullFeatureList>
              <p
                class="tw-text-xs tw-px-2 tw-font-medium tw-mb-1"
                *ngIf="organization.useSecretsManager"
              >
                {{ "bitwardenPasswordManager" | i18n }}
              </p>
              <p
                *ngIf="selectableProduct.productTier === productTypes.Teams"
                class="tw-text-xs tw-px-2 tw-mb-1"
              >
                {{ "teamsPlanUpgradeMessage" | i18n }}
              </p>
              <p
                *ngIf="selectableProduct.productTier === productTypes.Families"
                class="tw-text-xs tw-px-2 tw-mb-1"
              >
                {{ "familyPlanUpgradeMessage" | i18n }}
              </p>
              <ul
                class="bwi-ul tw-text-xs tw-mb-1"
                *ngIf="selectableProduct.productTier == productTypes.Families"
              >
                <li>
                  <i
                    class="bwi bwi-check tw-text-success bwi-li"
                    aria-hidden="true"
                  ></i>
                  {{ "premiumAccounts" | i18n }}
                </li>
                <li>
                  <i
                    class="bwi bwi-check tw-text-success bwi-li"
                    aria-hidden="true"
                  ></i>
                  {{ "unlimitedSharing" | i18n }}
                </li>
                <li>
                  <i
                    class="bwi bwi-check tw-text-success bwi-li"
                    aria-hidden="true"
                  ></i>
                  {{
                    "unlimitedCollections" | vfo1I18n: "unlimitedSharedFolders"
                  }}
                </li>
              </ul>
              <ul
                class="bwi-ul tw-text-xs"
                *ngIf="selectableProduct.productTier == productTypes.Teams"
              >
                <li>
                  <i
                    class="bwi bwi-check tw-text-success bwi-li"
                    aria-hidden="true"
                  ></i>
                  {{ "secureDataSharing" | i18n }}
                </li>
                <li>
                  <i
                    class="bwi bwi-check tw-text-success bwi-li"
                    aria-hidden="true"
                  ></i>
                  {{ "eventLogMonitoring" | i18n }}
                </li>
                <li>
                  <i
                    class="bwi bwi-check tw-text-success bwi-li"
                    aria-hidden="true"
                  ></i>
                  {{ "directoryIntegration" | i18n }}
                </li>
              </ul>
              <p
                class="tw-text-xs tw-px-2 tw-font-medium tw-mb-1"
                *ngIf="
                  organization.useSecretsManager &&
                  selectableProduct.productTier !== productTypes.Families
                "
              >
                {{ "bitwardenSecretsManager" | i18n }}
              </p>
              <ul
                class="bwi-ul tw-text-xs"
                *ngIf="
                  organization.useSecretsManager &&
                  selectableProduct.productTier == productTypes.Teams
                "
              >
                <li>
                  <i
                    class="bwi bwi-check tw-text-success bwi-li"
                    aria-hidden="true"
                  ></i>
                  {{ "unlimitedSecretsStorage" | i18n }}
                </li>
                <li>
                  <i
                    class="bwi bwi-check tw-text-success bwi-li"
                    aria-hidden="true"
                  ></i>
                  {{ "unlimitedProjects" | i18n }}
                </li>
                <li>
                  <i
                    class="bwi bwi-check tw-text-success bwi-li"
                    aria-hidden="true"
                  ></i>
                  {{ "UpTo20MachineAccounts" | i18n }}
                </li>
              </ul>
            </ng-template>
          </ng-template>
        </bit-card>
      </div>
      <br />
      <bit-callout
        *ngIf="organization.useSecretsManager && !isSecretsManagerTrial()"
        type="info"
        title="SECRETS MANAGER SUBSCRIPTION"
      >
        {{ "secretsManagerSubscriptionInfo" | i18n }}
      </bit-callout>
      <bit-callout
        *ngIf="organization.useSecretsManager && isSecretsManagerTrial()"
        type="info"
        title="PASSWORD MANAGER SUBSCRIPTION"
      >
        {{ "secretsManagerComplimentaryPasswordManager" | i18n }}
      </bit-callout>
      <br />
    </ng-container>
    <!-- Payment info -->
    <ng-container
      *ngIf="
        formGroup.value.productTier !== productTypes.Free ||
        isSubscriptionCanceled
      "
    >
      <h2 bitTypography="h4">{{ "paymentMethod" | i18n }}</h2>
      <p *ngIf="!showPayment && !!paymentMethod && !isSubscriptionCanceled">
        @switch (paymentMethod.type) {
          @case ("bankAccount") {
            <i class="bwi bwi-fw bwi-billing"></i>
            {{ paymentMethod.bankName }}, *{{ paymentMethod.last4 }}
            @if (paymentMethod.hostedVerificationUrl) {
              <span>- {{ "unverified" | i18n }}</span>
            }
            <span
              class="tw-ml-2 tw-text-primary-600 tw-cursor-pointer"
              (click)="toggleShowPayment()"
            >
              {{ "changePaymentMethod" | i18n }}
            </span>
          }
          @case ("card") {
            <p class="tw-flex tw-items-center tw-gap-2">
              @let cardBrandIcon = getCardBrandIcon();
              @if (cardBrandIcon !== null) {
                <i class="bwi bwi-fw credit-card-icon {{ cardBrandIcon }}"></i>
              } @else {
                <i class="bwi bwi-fw bwi-credit-card"></i>
              }
              {{ paymentMethod.brand | titlecase }}, *{{ paymentMethod.last4 }},
              {{ paymentMethod.expiration }}
              <span
                class="tw-ml-2 tw-text-primary-600 tw-cursor-pointer"
                (click)="toggleShowPayment()"
              >
                {{ "changePaymentMethod" | i18n }}
              </span>
            </p>
          }
          @case ("payPal") {
            <i class="bwi bwi-fw bwi-paypal tw-text-primary-600"></i>
            {{ paymentMethod.email }}
            <span
              class="tw-ml-2 tw-text-primary-600 tw-cursor-pointer"
              (click)="toggleShowPayment()"
            >
              {{ "changePaymentMethod" | i18n }}
            </span>
          }
        }
        <!-- FIXME: Empty anchor element needs content or should be removed -->
        <!-- eslint-disable-next-line @angular-eslint/template/elements-content -->
        <a></a>
      </p>
      <ng-container *ngIf="canUpdatePaymentInformation()">
        <app-enter-payment-method
          [group]="billingFormGroup.controls.paymentMethod"
        >
        </app-enter-payment-method>
        <app-enter-billing-address
          [group]="billingFormGroup.controls.billingAddress"
          [scenario]="{ type: 'checkout', supportsTaxId }"
        >
        </app-enter-billing-address>
      </ng-container>
      @if (planChangeCart.isLoading()) {
        <div class="tw-mt-4 tw-text-center">
          <bit-spinner></bit-spinner>
        </div>
      } @else if (planChangeCart.error() != null) {
        <bit-callout type="danger" class="tw-mt-4 tw-block">
          {{ previewErrorMessageKey | i18n }}
        </bit-callout>
      } @else if (planChangeCart.hasValue()) {
        <div class="tw-mt-4">
          <billing-cart-summary [cart]="planChangeCart.value()" />
          @if (isFreeUpgradeWithTrial) {
            <bit-callout
              type="info"
              [title]="null"
              [accessibleName]="'trial' | i18n"
              class="tw-mt-4 tw-block"
            >
              {{ "planChangeTrialIncluded" | i18n: trialLengthDays }}
            </bit-callout>
          } @else if (isTrialing && remainingTrialDays > 0) {
            <bit-callout
              type="info"
              [title]="null"
              [accessibleName]="'trial' | i18n"
              class="tw-mt-4 tw-block"
            >
              {{
                trialRemainingMessageKey
                  | i18n
                    : remainingTrialDays
                    : (trialEndDate | date: "MMM. d, y")
              }}
            </bit-callout>
          }
        </div>
      }
    </ng-container>
  </div>
  <ng-container bitDialogFooter>
    <button
      bitButton
      bitFormButton
      buttonType="primary"
      type="submit"
      [disabled]="isSubmitDisabled"
    >
      {{ submitButtonLabel }}
    </button>
    <button
      bitButton
      buttonType="secondary"
      type="button"
      [bitDialogClose]="ResultType.Closed"
    >
      {{ "cancel" | i18n }}
    </button>
  </ng-container>
</form>

`````

## Option 2

`````json
{"printWidth":100,"htmlWhitespaceSensitivity":"ignore"}
`````

### Diff

`````diff
===================================================================
--- prettier
+++ oxfmt
@@ -140,10 +140,9 @@
                         : (((this.sub.useSecretsManager
                             ? selectableProduct.SecretsManager.seatPrice
                             : 0) +
                             selectableProduct.PasswordManager.seatPrice) /
-                            (selectableProduct.isAnnual ? 12 : 1)
-                            | currency: "$")
+                          (selectableProduct.isAnnual ? 12 : 1) | currency: "$")
                   }}
                 </b>
                 <span class="tw-text-xs tw-px-0">/{{ "monthPerMember" | i18n }}</span>
               </span>

`````

### Actual (oxfmt)

`````html
<form
  [formGroup]="formGroup"
  [bitSubmit]="submit"
  bit-dialog
  dialogSize="large"
  [loading]="loading"
>
  <span bitDialogTitle class="tw-font-medium">
    {{ dialogHeaderName }}
  </span>
  <div bitDialogContent>
    <p>{{ "upgradePlans" | i18n }}</p>
    <div class="tw-mb-3 tw-flex tw-justify-between">
      <span [hidden]="isSubscriptionCanceled" class="tw-text-lg tw-pr-1 tw-font-medium">
        {{ "selectAPlan" | i18n }}
      </span>
      <!-- Discount Badge -->
      <div class="tw-flex tw-items-center tw-gap-2">
        <!-- Plan Interval Toggle -->
        <div class="tw-inline-block" *ngIf="!isSubscriptionCanceled">
          <bit-toggle-group
            [selected]="selectedInterval"
            (selectedChange)="updateInterval($event)"
            [label]="'planBillingInterval' | i18n"
          >
            <bit-toggle
              *ngFor="
                let planInterval of getPlanIntervals();
                trackBy: optimizedNgForRender;
                let i = index
              "
              [value]="planInterval.value"
            >
              {{ planInterval.name }}
            </bit-toggle>
          </bit-toggle-group>
        </div>
      </div>
    </div>
    <!-- Plan Selection Cards -->
    <ng-container *ngIf="!loading && !selfHosted && this.passwordManagerPlans">
      <div
        class="tw-grid tw-grid-flow-col tw-gap-4 tw-mb-4"
        [class]="'tw-grid-cols-' + selectableProducts.length"
      >
        <bit-card
          *ngFor="
            let selectableProduct of selectableProducts;
            trackBy: manageSelectableProduct;
            let i = index
          "
          [ngClass]="getPlanCardContainerClasses(selectableProduct, i)"
          (click)="selectPlan(selectableProduct)"
          [attr.tabindex]="focusedIndex !== i || isCardDisabled(i) ? '-1' : '0'"
          (keyup)="onKeydown($event, i)"
          (focus)="onFocus(i)"
          [attr.aria-disabled]="isCardDisabled(i)"
          [id]="i + 'a_plan_card'"
        >
          <div class="tw-relative">
            <div
              *ngIf="
                selectableProduct.productTier === productTypes.Enterprise && !isSubscriptionCanceled
              "
              class="tw-bg-secondary-100 tw-text-center !tw-border-0 tw-text-sm tw-font-medium tw-py-1"
              [ngClass]="{
                'tw-bg-primary-700 !tw-text-contrast': selectableProduct === selectedPlan,
                'tw-bg-secondary-100': !(selectableProduct === selectedPlan),
              }"
            >
              {{ "recommended" | i18n }}
            </div>
            <div
              class="tw-px-2 tw-pb-[4px]"
              [ngClass]="{
                'tw-py-1': !(selectableProduct === selectedPlan),
                'tw-py-0': selectableProduct === selectedPlan,
              }"
            >
              <h3
                class="tw-text-[1.25rem] tw-mt-[6px] tw-font-medium tw-mb-0 tw-leading-[2rem] tw-flex tw-items-center"
              >
                <span class="tw-capitalize tw-whitespace-nowrap">
                  {{ selectableProduct.nameLocalizationKey | i18n }}
                </span>
                <span
                  bitBadge
                  variant="secondary"
                  *ngIf="selectableProduct === currentPlan"
                  class="tw-ml-2 tw-align-middle"
                >
                  {{ "current" | i18n }}
                </span>
              </h3>
              <span *ngIf="selectableProduct.productTier != productTypes.Free">
                <ng-container
                  *ngIf="selectableProduct.PasswordManager.basePrice && !acceptingSponsorship"
                >
                  <b class="tw-text-lg tw-font-medium">
                    {{
                      (selectableProduct.isAnnual
                        ? selectableProduct.PasswordManager.basePrice / 12
                        : selectableProduct.PasswordManager.basePrice
                      ) | currency: "$"
                    }}
                  </b>
                  <span class="tw-text-xs tw-px-0">
                    /{{
                      selectableProduct.productTier === productTypes.Families
                        ? "month"
                        : ("monthPerMember" | i18n)
                    }}
                  </span>
                  <b class="tw-text-sm tw-font-medium">
                    <ng-container
                      *ngIf="selectableProduct.PasswordManager.hasAdditionalSeatsOption"
                    >
                      {{ ("additionalUsers" | i18n).toLowerCase() }}
                      {{
                        (selectableProduct.isAnnual
                          ? selectableProduct.PasswordManager.seatPrice / 12
                          : selectableProduct.PasswordManager.seatPrice
                        ) | currency: "$"
                      }}
                      /{{ selectedPlanInterval | i18n }}
                    </ng-container>
                  </b>
                </ng-container>
              </span>
              <span
                *ngIf="
                  !selectableProduct.PasswordManager.basePrice &&
                  selectableProduct.PasswordManager.hasAdditionalSeatsOption
                "
              >
                <b class="tw-text-lg tw-font-medium">
                  {{
                    "costPerMember"
                      | i18n
                        : (((this.sub.useSecretsManager
                            ? selectableProduct.SecretsManager.seatPrice
                            : 0) +
                            selectableProduct.PasswordManager.seatPrice) /
                          (selectableProduct.isAnnual ? 12 : 1) | currency: "$")
                  }}
                </b>
                <span class="tw-text-xs tw-px-0">/{{ "monthPerMember" | i18n }}</span>
              </span>
              <span *ngIf="selectableProduct.productTier == productTypes.Free">
                {{ "freeForever" | i18n }}
              </span>
            </div>
          </div>

          <ng-container
            *ngIf="
              selectableProduct.productTier === productTypes.Enterprise;
              else nonEnterprisePlans
            "
          >
            <p
              class="tw-text-xs tw-px-2 tw-font-medium tw-mb-1"
              *ngIf="organization.useSecretsManager"
            >
              {{ "bitwardenPasswordManager" | i18n }}
            </p>
            <p class="tw-text-xs tw-px-2 tw-mb-1">{{ "enterprisePlanUpgradeMessage" | i18n }}</p>

            <ul class="bwi-ul tw-text-xs">
              <li>
                <i class="bwi bwi-check tw-text-success bwi-li" aria-hidden="true"></i>
                {{ "includeEnterprisePolicies" | i18n }}
              </li>
              <li>
                <i class="bwi bwi-check tw-text-success bwi-li" aria-hidden="true"></i>
                {{ "passwordLessSso" | i18n }}
              </li>
              <li>
                <i class="bwi bwi-check tw-text-success bwi-li" aria-hidden="true"></i>
                {{ "accountRecovery" | i18n }}
              </li>
              <li>
                <i class="bwi bwi-check tw-text-success bwi-li" aria-hidden="true"></i>
                {{ "customRoles" | i18n }}
              </li>
            </ul>

            <p
              class="tw-text-xs tw-px-2 tw-font-medium tw-mb-1"
              *ngIf="organization.useSecretsManager"
            >
              {{ "bitwardenSecretsManager" | i18n }}
            </p>
            <ul class="bwi-ul tw-text-xs" *ngIf="organization.useSecretsManager">
              <li>
                <i class="bwi bwi-check tw-text-success bwi-li" aria-hidden="true"></i>
                {{ "unlimitedSecretsStorage" | i18n }}
              </li>
              <li>
                <i class="bwi bwi-check tw-text-success bwi-li" aria-hidden="true"></i>
                {{ "unlimitedUsers" | i18n }}
              </li>
              <li>
                <i class="bwi bwi-check tw-text-success bwi-li" aria-hidden="true"></i>
                {{ "unlimitedProjects" | i18n }}
              </li>
              <li>
                <i class="bwi bwi-check tw-text-success bwi-li" aria-hidden="true"></i>
                {{ "UpTo50MachineAccounts" | i18n }}
              </li>
            </ul>
          </ng-container>
          <ng-template #nonEnterprisePlans>
            <ng-container
              *ngIf="
                selectableProduct.productTier === productTypes.Teams && teamsStarterPlanIsAvailable;
                else fullFeatureList
              "
            >
              <ul class="tw-px-2 tw-pb-2 tw-list-inside tw-mb-0 tw-text-xs">
                <li>{{ "includeAllTeamsStarterFeatures" | i18n }}</li>
                <li>{{ "chooseMonthlyOrAnnualBilling" | i18n }}</li>
                <li>{{ "abilityToAddMoreThanNMembers" | i18n: 10 }}</li>
              </ul>
            </ng-container>
            <ng-template #fullFeatureList>
              <p
                class="tw-text-xs tw-px-2 tw-font-medium tw-mb-1"
                *ngIf="organization.useSecretsManager"
              >
                {{ "bitwardenPasswordManager" | i18n }}
              </p>
              <p
                *ngIf="selectableProduct.productTier === productTypes.Teams"
                class="tw-text-xs tw-px-2 tw-mb-1"
              >
                {{ "teamsPlanUpgradeMessage" | i18n }}
              </p>
              <p
                *ngIf="selectableProduct.productTier === productTypes.Families"
                class="tw-text-xs tw-px-2 tw-mb-1"
              >
                {{ "familyPlanUpgradeMessage" | i18n }}
              </p>
              <ul
                class="bwi-ul tw-text-xs tw-mb-1"
                *ngIf="selectableProduct.productTier == productTypes.Families"
              >
                <li>
                  <i class="bwi bwi-check tw-text-success bwi-li" aria-hidden="true"></i>
                  {{ "premiumAccounts" | i18n }}
                </li>
                <li>
                  <i class="bwi bwi-check tw-text-success bwi-li" aria-hidden="true"></i>
                  {{ "unlimitedSharing" | i18n }}
                </li>
                <li>
                  <i class="bwi bwi-check tw-text-success bwi-li" aria-hidden="true"></i>
                  {{ "unlimitedCollections" | vfo1I18n: "unlimitedSharedFolders" }}
                </li>
              </ul>
              <ul
                class="bwi-ul tw-text-xs"
                *ngIf="selectableProduct.productTier == productTypes.Teams"
              >
                <li>
                  <i class="bwi bwi-check tw-text-success bwi-li" aria-hidden="true"></i>
                  {{ "secureDataSharing" | i18n }}
                </li>
                <li>
                  <i class="bwi bwi-check tw-text-success bwi-li" aria-hidden="true"></i>
                  {{ "eventLogMonitoring" | i18n }}
                </li>
                <li>
                  <i class="bwi bwi-check tw-text-success bwi-li" aria-hidden="true"></i>
                  {{ "directoryIntegration" | i18n }}
                </li>
              </ul>
              <p
                class="tw-text-xs tw-px-2 tw-font-medium tw-mb-1"
                *ngIf="
                  organization.useSecretsManager &&
                  selectableProduct.productTier !== productTypes.Families
                "
              >
                {{ "bitwardenSecretsManager" | i18n }}
              </p>
              <ul
                class="bwi-ul tw-text-xs"
                *ngIf="
                  organization.useSecretsManager &&
                  selectableProduct.productTier == productTypes.Teams
                "
              >
                <li>
                  <i class="bwi bwi-check tw-text-success bwi-li" aria-hidden="true"></i>
                  {{ "unlimitedSecretsStorage" | i18n }}
                </li>
                <li>
                  <i class="bwi bwi-check tw-text-success bwi-li" aria-hidden="true"></i>
                  {{ "unlimitedProjects" | i18n }}
                </li>
                <li>
                  <i class="bwi bwi-check tw-text-success bwi-li" aria-hidden="true"></i>
                  {{ "UpTo20MachineAccounts" | i18n }}
                </li>
              </ul>
            </ng-template>
          </ng-template>
        </bit-card>
      </div>
      <br />
      <bit-callout
        *ngIf="organization.useSecretsManager && !isSecretsManagerTrial()"
        type="info"
        title="SECRETS MANAGER SUBSCRIPTION"
      >
        {{ "secretsManagerSubscriptionInfo" | i18n }}
      </bit-callout>
      <bit-callout
        *ngIf="organization.useSecretsManager && isSecretsManagerTrial()"
        type="info"
        title="PASSWORD MANAGER SUBSCRIPTION"
      >
        {{ "secretsManagerComplimentaryPasswordManager" | i18n }}
      </bit-callout>
      <br />
    </ng-container>
    <!-- Payment info -->
    <ng-container
      *ngIf="formGroup.value.productTier !== productTypes.Free || isSubscriptionCanceled"
    >
      <h2 bitTypography="h4">{{ "paymentMethod" | i18n }}</h2>
      <p *ngIf="!showPayment && !!paymentMethod && !isSubscriptionCanceled">
        @switch (paymentMethod.type) {
          @case ("bankAccount") {
            <i class="bwi bwi-fw bwi-billing"></i>
            {{ paymentMethod.bankName }}, *{{ paymentMethod.last4 }}
            @if (paymentMethod.hostedVerificationUrl) {
              <span>- {{ "unverified" | i18n }}</span>
            }
            <span
              class="tw-ml-2 tw-text-primary-600 tw-cursor-pointer"
              (click)="toggleShowPayment()"
            >
              {{ "changePaymentMethod" | i18n }}
            </span>
          }
          @case ("card") {
            <p class="tw-flex tw-items-center tw-gap-2">
              @let cardBrandIcon = getCardBrandIcon();
              @if (cardBrandIcon !== null) {
                <i class="bwi bwi-fw credit-card-icon {{ cardBrandIcon }}"></i>
              } @else {
                <i class="bwi bwi-fw bwi-credit-card"></i>
              }
              {{ paymentMethod.brand | titlecase }}, *{{ paymentMethod.last4 }},
              {{ paymentMethod.expiration }}
              <span
                class="tw-ml-2 tw-text-primary-600 tw-cursor-pointer"
                (click)="toggleShowPayment()"
              >
                {{ "changePaymentMethod" | i18n }}
              </span>
            </p>
          }
          @case ("payPal") {
            <i class="bwi bwi-fw bwi-paypal tw-text-primary-600"></i>
            {{ paymentMethod.email }}
            <span
              class="tw-ml-2 tw-text-primary-600 tw-cursor-pointer"
              (click)="toggleShowPayment()"
            >
              {{ "changePaymentMethod" | i18n }}
            </span>
          }
        }
        <!-- FIXME: Empty anchor element needs content or should be removed -->
        <!-- eslint-disable-next-line @angular-eslint/template/elements-content -->
        <a></a>
      </p>
      <ng-container *ngIf="canUpdatePaymentInformation()">
        <app-enter-payment-method
          [group]="billingFormGroup.controls.paymentMethod"
        ></app-enter-payment-method>
        <app-enter-billing-address
          [group]="billingFormGroup.controls.billingAddress"
          [scenario]="{ type: 'checkout', supportsTaxId }"
        ></app-enter-billing-address>
      </ng-container>
      @if (planChangeCart.isLoading()) {
        <div class="tw-mt-4 tw-text-center">
          <bit-spinner></bit-spinner>
        </div>
      } @else if (planChangeCart.error() != null) {
        <bit-callout type="danger" class="tw-mt-4 tw-block">
          {{ previewErrorMessageKey | i18n }}
        </bit-callout>
      } @else if (planChangeCart.hasValue()) {
        <div class="tw-mt-4">
          <billing-cart-summary [cart]="planChangeCart.value()" />
          @if (isFreeUpgradeWithTrial) {
            <bit-callout
              type="info"
              [title]="null"
              [accessibleName]="'trial' | i18n"
              class="tw-mt-4 tw-block"
            >
              {{ "planChangeTrialIncluded" | i18n: trialLengthDays }}
            </bit-callout>
          } @else if (isTrialing && remainingTrialDays > 0) {
            <bit-callout
              type="info"
              [title]="null"
              [accessibleName]="'trial' | i18n"
              class="tw-mt-4 tw-block"
            >
              {{
                trialRemainingMessageKey
                  | i18n: remainingTrialDays : (trialEndDate | date: "MMM. d, y")
              }}
            </bit-callout>
          }
        </div>
      }
    </ng-container>
  </div>
  <ng-container bitDialogFooter>
    <button
      bitButton
      bitFormButton
      buttonType="primary"
      type="submit"
      [disabled]="isSubmitDisabled"
    >
      {{ submitButtonLabel }}
    </button>
    <button bitButton buttonType="secondary" type="button" [bitDialogClose]="ResultType.Closed">
      {{ "cancel" | i18n }}
    </button>
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
  [loading]="loading"
>
  <span bitDialogTitle class="tw-font-medium">
    {{ dialogHeaderName }}
  </span>
  <div bitDialogContent>
    <p>{{ "upgradePlans" | i18n }}</p>
    <div class="tw-mb-3 tw-flex tw-justify-between">
      <span [hidden]="isSubscriptionCanceled" class="tw-text-lg tw-pr-1 tw-font-medium">
        {{ "selectAPlan" | i18n }}
      </span>
      <!-- Discount Badge -->
      <div class="tw-flex tw-items-center tw-gap-2">
        <!-- Plan Interval Toggle -->
        <div class="tw-inline-block" *ngIf="!isSubscriptionCanceled">
          <bit-toggle-group
            [selected]="selectedInterval"
            (selectedChange)="updateInterval($event)"
            [label]="'planBillingInterval' | i18n"
          >
            <bit-toggle
              *ngFor="
                let planInterval of getPlanIntervals();
                trackBy: optimizedNgForRender;
                let i = index
              "
              [value]="planInterval.value"
            >
              {{ planInterval.name }}
            </bit-toggle>
          </bit-toggle-group>
        </div>
      </div>
    </div>
    <!-- Plan Selection Cards -->
    <ng-container *ngIf="!loading && !selfHosted && this.passwordManagerPlans">
      <div
        class="tw-grid tw-grid-flow-col tw-gap-4 tw-mb-4"
        [class]="'tw-grid-cols-' + selectableProducts.length"
      >
        <bit-card
          *ngFor="
            let selectableProduct of selectableProducts;
            trackBy: manageSelectableProduct;
            let i = index
          "
          [ngClass]="getPlanCardContainerClasses(selectableProduct, i)"
          (click)="selectPlan(selectableProduct)"
          [attr.tabindex]="focusedIndex !== i || isCardDisabled(i) ? '-1' : '0'"
          (keyup)="onKeydown($event, i)"
          (focus)="onFocus(i)"
          [attr.aria-disabled]="isCardDisabled(i)"
          [id]="i + 'a_plan_card'"
        >
          <div class="tw-relative">
            <div
              *ngIf="
                selectableProduct.productTier === productTypes.Enterprise && !isSubscriptionCanceled
              "
              class="tw-bg-secondary-100 tw-text-center !tw-border-0 tw-text-sm tw-font-medium tw-py-1"
              [ngClass]="{
                'tw-bg-primary-700 !tw-text-contrast': selectableProduct === selectedPlan,
                'tw-bg-secondary-100': !(selectableProduct === selectedPlan),
              }"
            >
              {{ "recommended" | i18n }}
            </div>
            <div
              class="tw-px-2 tw-pb-[4px]"
              [ngClass]="{
                'tw-py-1': !(selectableProduct === selectedPlan),
                'tw-py-0': selectableProduct === selectedPlan,
              }"
            >
              <h3
                class="tw-text-[1.25rem] tw-mt-[6px] tw-font-medium tw-mb-0 tw-leading-[2rem] tw-flex tw-items-center"
              >
                <span class="tw-capitalize tw-whitespace-nowrap">
                  {{ selectableProduct.nameLocalizationKey | i18n }}
                </span>
                <span
                  bitBadge
                  variant="secondary"
                  *ngIf="selectableProduct === currentPlan"
                  class="tw-ml-2 tw-align-middle"
                >
                  {{ "current" | i18n }}
                </span>
              </h3>
              <span *ngIf="selectableProduct.productTier != productTypes.Free">
                <ng-container
                  *ngIf="selectableProduct.PasswordManager.basePrice && !acceptingSponsorship"
                >
                  <b class="tw-text-lg tw-font-medium">
                    {{
                      (selectableProduct.isAnnual
                        ? selectableProduct.PasswordManager.basePrice / 12
                        : selectableProduct.PasswordManager.basePrice
                      ) | currency: "$"
                    }}
                  </b>
                  <span class="tw-text-xs tw-px-0">
                    /{{
                      selectableProduct.productTier === productTypes.Families
                        ? "month"
                        : ("monthPerMember" | i18n)
                    }}
                  </span>
                  <b class="tw-text-sm tw-font-medium">
                    <ng-container
                      *ngIf="selectableProduct.PasswordManager.hasAdditionalSeatsOption"
                    >
                      {{ ("additionalUsers" | i18n).toLowerCase() }}
                      {{
                        (selectableProduct.isAnnual
                          ? selectableProduct.PasswordManager.seatPrice / 12
                          : selectableProduct.PasswordManager.seatPrice
                        ) | currency: "$"
                      }}
                      /{{ selectedPlanInterval | i18n }}
                    </ng-container>
                  </b>
                </ng-container>
              </span>
              <span
                *ngIf="
                  !selectableProduct.PasswordManager.basePrice &&
                  selectableProduct.PasswordManager.hasAdditionalSeatsOption
                "
              >
                <b class="tw-text-lg tw-font-medium">
                  {{
                    "costPerMember"
                      | i18n
                        : (((this.sub.useSecretsManager
                            ? selectableProduct.SecretsManager.seatPrice
                            : 0) +
                            selectableProduct.PasswordManager.seatPrice) /
                            (selectableProduct.isAnnual ? 12 : 1)
                            | currency: "$")
                  }}
                </b>
                <span class="tw-text-xs tw-px-0">/{{ "monthPerMember" | i18n }}</span>
              </span>
              <span *ngIf="selectableProduct.productTier == productTypes.Free">
                {{ "freeForever" | i18n }}
              </span>
            </div>
          </div>

          <ng-container
            *ngIf="
              selectableProduct.productTier === productTypes.Enterprise;
              else nonEnterprisePlans
            "
          >
            <p
              class="tw-text-xs tw-px-2 tw-font-medium tw-mb-1"
              *ngIf="organization.useSecretsManager"
            >
              {{ "bitwardenPasswordManager" | i18n }}
            </p>
            <p class="tw-text-xs tw-px-2 tw-mb-1">{{ "enterprisePlanUpgradeMessage" | i18n }}</p>

            <ul class="bwi-ul tw-text-xs">
              <li>
                <i class="bwi bwi-check tw-text-success bwi-li" aria-hidden="true"></i>
                {{ "includeEnterprisePolicies" | i18n }}
              </li>
              <li>
                <i class="bwi bwi-check tw-text-success bwi-li" aria-hidden="true"></i>
                {{ "passwordLessSso" | i18n }}
              </li>
              <li>
                <i class="bwi bwi-check tw-text-success bwi-li" aria-hidden="true"></i>
                {{ "accountRecovery" | i18n }}
              </li>
              <li>
                <i class="bwi bwi-check tw-text-success bwi-li" aria-hidden="true"></i>
                {{ "customRoles" | i18n }}
              </li>
            </ul>

            <p
              class="tw-text-xs tw-px-2 tw-font-medium tw-mb-1"
              *ngIf="organization.useSecretsManager"
            >
              {{ "bitwardenSecretsManager" | i18n }}
            </p>
            <ul class="bwi-ul tw-text-xs" *ngIf="organization.useSecretsManager">
              <li>
                <i class="bwi bwi-check tw-text-success bwi-li" aria-hidden="true"></i>
                {{ "unlimitedSecretsStorage" | i18n }}
              </li>
              <li>
                <i class="bwi bwi-check tw-text-success bwi-li" aria-hidden="true"></i>
                {{ "unlimitedUsers" | i18n }}
              </li>
              <li>
                <i class="bwi bwi-check tw-text-success bwi-li" aria-hidden="true"></i>
                {{ "unlimitedProjects" | i18n }}
              </li>
              <li>
                <i class="bwi bwi-check tw-text-success bwi-li" aria-hidden="true"></i>
                {{ "UpTo50MachineAccounts" | i18n }}
              </li>
            </ul>
          </ng-container>
          <ng-template #nonEnterprisePlans>
            <ng-container
              *ngIf="
                selectableProduct.productTier === productTypes.Teams && teamsStarterPlanIsAvailable;
                else fullFeatureList
              "
            >
              <ul class="tw-px-2 tw-pb-2 tw-list-inside tw-mb-0 tw-text-xs">
                <li>{{ "includeAllTeamsStarterFeatures" | i18n }}</li>
                <li>{{ "chooseMonthlyOrAnnualBilling" | i18n }}</li>
                <li>{{ "abilityToAddMoreThanNMembers" | i18n: 10 }}</li>
              </ul>
            </ng-container>
            <ng-template #fullFeatureList>
              <p
                class="tw-text-xs tw-px-2 tw-font-medium tw-mb-1"
                *ngIf="organization.useSecretsManager"
              >
                {{ "bitwardenPasswordManager" | i18n }}
              </p>
              <p
                *ngIf="selectableProduct.productTier === productTypes.Teams"
                class="tw-text-xs tw-px-2 tw-mb-1"
              >
                {{ "teamsPlanUpgradeMessage" | i18n }}
              </p>
              <p
                *ngIf="selectableProduct.productTier === productTypes.Families"
                class="tw-text-xs tw-px-2 tw-mb-1"
              >
                {{ "familyPlanUpgradeMessage" | i18n }}
              </p>
              <ul
                class="bwi-ul tw-text-xs tw-mb-1"
                *ngIf="selectableProduct.productTier == productTypes.Families"
              >
                <li>
                  <i class="bwi bwi-check tw-text-success bwi-li" aria-hidden="true"></i>
                  {{ "premiumAccounts" | i18n }}
                </li>
                <li>
                  <i class="bwi bwi-check tw-text-success bwi-li" aria-hidden="true"></i>
                  {{ "unlimitedSharing" | i18n }}
                </li>
                <li>
                  <i class="bwi bwi-check tw-text-success bwi-li" aria-hidden="true"></i>
                  {{ "unlimitedCollections" | vfo1I18n: "unlimitedSharedFolders" }}
                </li>
              </ul>
              <ul
                class="bwi-ul tw-text-xs"
                *ngIf="selectableProduct.productTier == productTypes.Teams"
              >
                <li>
                  <i class="bwi bwi-check tw-text-success bwi-li" aria-hidden="true"></i>
                  {{ "secureDataSharing" | i18n }}
                </li>
                <li>
                  <i class="bwi bwi-check tw-text-success bwi-li" aria-hidden="true"></i>
                  {{ "eventLogMonitoring" | i18n }}
                </li>
                <li>
                  <i class="bwi bwi-check tw-text-success bwi-li" aria-hidden="true"></i>
                  {{ "directoryIntegration" | i18n }}
                </li>
              </ul>
              <p
                class="tw-text-xs tw-px-2 tw-font-medium tw-mb-1"
                *ngIf="
                  organization.useSecretsManager &&
                  selectableProduct.productTier !== productTypes.Families
                "
              >
                {{ "bitwardenSecretsManager" | i18n }}
              </p>
              <ul
                class="bwi-ul tw-text-xs"
                *ngIf="
                  organization.useSecretsManager &&
                  selectableProduct.productTier == productTypes.Teams
                "
              >
                <li>
                  <i class="bwi bwi-check tw-text-success bwi-li" aria-hidden="true"></i>
                  {{ "unlimitedSecretsStorage" | i18n }}
                </li>
                <li>
                  <i class="bwi bwi-check tw-text-success bwi-li" aria-hidden="true"></i>
                  {{ "unlimitedProjects" | i18n }}
                </li>
                <li>
                  <i class="bwi bwi-check tw-text-success bwi-li" aria-hidden="true"></i>
                  {{ "UpTo20MachineAccounts" | i18n }}
                </li>
              </ul>
            </ng-template>
          </ng-template>
        </bit-card>
      </div>
      <br />
      <bit-callout
        *ngIf="organization.useSecretsManager && !isSecretsManagerTrial()"
        type="info"
        title="SECRETS MANAGER SUBSCRIPTION"
      >
        {{ "secretsManagerSubscriptionInfo" | i18n }}
      </bit-callout>
      <bit-callout
        *ngIf="organization.useSecretsManager && isSecretsManagerTrial()"
        type="info"
        title="PASSWORD MANAGER SUBSCRIPTION"
      >
        {{ "secretsManagerComplimentaryPasswordManager" | i18n }}
      </bit-callout>
      <br />
    </ng-container>
    <!-- Payment info -->
    <ng-container
      *ngIf="formGroup.value.productTier !== productTypes.Free || isSubscriptionCanceled"
    >
      <h2 bitTypography="h4">{{ "paymentMethod" | i18n }}</h2>
      <p *ngIf="!showPayment && !!paymentMethod && !isSubscriptionCanceled">
        @switch (paymentMethod.type) {
          @case ("bankAccount") {
            <i class="bwi bwi-fw bwi-billing"></i>
            {{ paymentMethod.bankName }}, *{{ paymentMethod.last4 }}
            @if (paymentMethod.hostedVerificationUrl) {
              <span>- {{ "unverified" | i18n }}</span>
            }
            <span
              class="tw-ml-2 tw-text-primary-600 tw-cursor-pointer"
              (click)="toggleShowPayment()"
            >
              {{ "changePaymentMethod" | i18n }}
            </span>
          }
          @case ("card") {
            <p class="tw-flex tw-items-center tw-gap-2">
              @let cardBrandIcon = getCardBrandIcon();
              @if (cardBrandIcon !== null) {
                <i class="bwi bwi-fw credit-card-icon {{ cardBrandIcon }}"></i>
              } @else {
                <i class="bwi bwi-fw bwi-credit-card"></i>
              }
              {{ paymentMethod.brand | titlecase }}, *{{ paymentMethod.last4 }},
              {{ paymentMethod.expiration }}
              <span
                class="tw-ml-2 tw-text-primary-600 tw-cursor-pointer"
                (click)="toggleShowPayment()"
              >
                {{ "changePaymentMethod" | i18n }}
              </span>
            </p>
          }
          @case ("payPal") {
            <i class="bwi bwi-fw bwi-paypal tw-text-primary-600"></i>
            {{ paymentMethod.email }}
            <span
              class="tw-ml-2 tw-text-primary-600 tw-cursor-pointer"
              (click)="toggleShowPayment()"
            >
              {{ "changePaymentMethod" | i18n }}
            </span>
          }
        }
        <!-- FIXME: Empty anchor element needs content or should be removed -->
        <!-- eslint-disable-next-line @angular-eslint/template/elements-content -->
        <a></a>
      </p>
      <ng-container *ngIf="canUpdatePaymentInformation()">
        <app-enter-payment-method
          [group]="billingFormGroup.controls.paymentMethod"
        ></app-enter-payment-method>
        <app-enter-billing-address
          [group]="billingFormGroup.controls.billingAddress"
          [scenario]="{ type: 'checkout', supportsTaxId }"
        ></app-enter-billing-address>
      </ng-container>
      @if (planChangeCart.isLoading()) {
        <div class="tw-mt-4 tw-text-center">
          <bit-spinner></bit-spinner>
        </div>
      } @else if (planChangeCart.error() != null) {
        <bit-callout type="danger" class="tw-mt-4 tw-block">
          {{ previewErrorMessageKey | i18n }}
        </bit-callout>
      } @else if (planChangeCart.hasValue()) {
        <div class="tw-mt-4">
          <billing-cart-summary [cart]="planChangeCart.value()" />
          @if (isFreeUpgradeWithTrial) {
            <bit-callout
              type="info"
              [title]="null"
              [accessibleName]="'trial' | i18n"
              class="tw-mt-4 tw-block"
            >
              {{ "planChangeTrialIncluded" | i18n: trialLengthDays }}
            </bit-callout>
          } @else if (isTrialing && remainingTrialDays > 0) {
            <bit-callout
              type="info"
              [title]="null"
              [accessibleName]="'trial' | i18n"
              class="tw-mt-4 tw-block"
            >
              {{
                trialRemainingMessageKey
                  | i18n: remainingTrialDays : (trialEndDate | date: "MMM. d, y")
              }}
            </bit-callout>
          }
        </div>
      }
    </ng-container>
  </div>
  <ng-container bitDialogFooter>
    <button
      bitButton
      bitFormButton
      buttonType="primary"
      type="submit"
      [disabled]="isSubmitDisabled"
    >
      {{ submitButtonLabel }}
    </button>
    <button bitButton buttonType="secondary" type="button" [bitDialogClose]="ResultType.Closed">
      {{ "cancel" | i18n }}
    </button>
  </ng-container>
</form>

`````
