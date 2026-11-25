# externals/bitwarden-web/billing/organizations/organization-plans.component.html

## Option 1

`````json
{"printWidth":80}
`````

### Diff

`````diff
===================================================================
--- prettier
+++ oxfmt
@@ -217,10 +217,9 @@
                     | i18n
                       : ((selectableProduct.isAnnual
                           ? pm.seatPrice / 12
                           : pm.seatPrice
-                        )
-                          | currency: "$")
+                        ) | currency: "$")
                 }}
                 /{{ "month" | i18n }}
               </span>
             }

`````

### Actual (oxfmt)

`````html
@let isLoading = loading;
@let passwordManagerPlans = this.passwordManagerPlans;
@let secretsManagerPlans = this.secretsManagerPlans;
@let upgradingFromPremium = this.canUpgradeFromPremium();

@if (isLoading) {
  <ng-container>
    <bit-icon
      name="bwi-spinner"
      class="bwi-spin tw-text-muted"
      title="{{ 'loading' | i18n }}"
      aria-hidden="true"
    ></bit-icon>
    <span class="tw-sr-only">{{ "loading" | i18n }}</span>
  </ng-container>
}
@if (selfHosted) {
  <ng-container>
    <organization-self-hosting-license-uploader
      (onLicenseFileUploaded)="onLicenseFileUploaded($event)"
    />
  </ng-container>
}
@if (!isLoading && !selfHosted && passwordManagerPlans && secretsManagerPlans) {
  <form
    [formGroup]="formGroup"
    [bitSubmit]="submit"
    [class.tw-pt-6]="!sponsoredPlanLayout()"
    [class.tw-pt-2]="sponsoredPlanLayout()"
  >
    <bit-section
      [class.tw-block]="sponsoredPlanLayout()"
      [class.tw-mb-6]="sponsoredPlanLayout()"
      [disableMargin]="sponsoredPlanLayout()"
    >
      <app-org-info
        (changedBusinessOwned)="changedOwnedBusiness()"
        [formGroup]="formGroup"
        [acceptingSponsorship]="acceptingSponsorship()"
      >
      </app-org-info>
    </bit-section>
    <bit-section
      [disableMargin]="sponsoredPlanLayout()"
      [class.tw-block]="sponsoredPlanLayout()"
      [class.tw-mb-6]="sponsoredPlanLayout()"
    >
      @if (acceptingSponsorship()) {
        <h2 bitTypography="h2">
          {{ "chooseYourPlan" | vfo1I18n: "sponsoredPlanDetails" }}
        </h2>
      } @else {
        <h2 bitTypography="h2">
          {{ "chooseYourPlan" | vfo1I18n: "choosePlan" }}
        </h2>
      }
      <bit-radio-group formControlName="productTier" [block]="true">
        @for (selectableProduct of selectableProducts(); track $index) {
          @let pm = selectableProduct.PasswordManager;
          @let tier = selectableProduct.productTier;
          @let hasSelfHost = selectableProduct.hasSelfHost;
          @let hasSso = selectableProduct.hasSso;
          @let hasPolicies = selectableProduct.hasPolicies;
          @let hasGroups = selectableProduct.hasGroups;
          @let hasApi = selectableProduct.hasApi;
          @let hasDirectory = selectableProduct.hasDirectory;
          @let usersGetPremium = selectableProduct.usersGetPremium;
          @let trialPeriodDays = selectableProduct.trialPeriodDays;
          @let effectiveTrial = trialLength() ?? trialPeriodDays;
          @let showTrialOffer =
            tier != productTypes.Free &&
            effectiveTrial &&
            !upgradingFromPremium;

          <div [class.tw-mb-3]="!sponsoredPlanLayout()">
            <bit-radio-button [value]="tier" (change)="changedProduct()">
              <bit-label>{{
                selectableProduct.nameLocalizationKey | i18n
              }}</bit-label>
              <bit-hint class="tw-text-sm"
                >{{ selectableProduct.descriptionLocalizationKey | i18n: "1" }}
                @if (tier === productTypes.Enterprise) {
                  <ul class="tw-pl-0 tw-list-inside tw-mb-0">
                    <li>{{ "includeAllTeamsFeatures" | i18n }}</li>
                    @if (hasSelfHost) {
                      <li>{{ "onPremHostingOptional" | i18n }}</li>
                    }
                    @if (hasSso) {
                      <li>{{ "includeSsoAuthentication" | i18n }}</li>
                    }
                    @if (hasPolicies) {
                      <li>{{ "includeEnterprisePolicies" | i18n }}</li>
                    }
                    @if (showTrialOffer) {
                      <li>{{ "xDayFreeTrial" | i18n: effectiveTrial }}</li>
                    }
                  </ul>
                } @else {
                  @if (
                    tier === productTypes.Teams && teamsStarterPlanIsAvailable()
                  ) {
                    <ul class="tw-pl-0 tw-list-inside tw-mb-0">
                      <li>{{ "includeAllTeamsStarterFeatures" | i18n }}</li>
                      <li>{{ "chooseMonthlyOrAnnualBilling" | i18n }}</li>
                      <li>{{ "abilityToAddMoreThanNMembers" | i18n: 10 }}</li>
                      @if (showTrialOffer) {
                        <li>
                          {{ "xDayFreeTrial" | i18n: effectiveTrial }}
                        </li>
                      }
                    </ul>
                  } @else {
                    <ul class="tw-pl-0 tw-list-inside tw-mb-0">
                      @if (tier == productTypes.Free) {
                        <li>{{ "limitedUsers" | i18n: pm.maxSeats }}</li>
                      }
                      @if (
                        tier != productTypes.Free &&
                        tier != productTypes.TeamsStarter &&
                        pm.maxSeats
                      ) {
                        <li>
                          {{ "addShareLimitedUsers" | i18n: pm.maxSeats }}
                        </li>
                      }
                      @if (!pm.maxSeats) {
                        <li>{{ "addShareUnlimitedUsers" | i18n }}</li>
                      }
                      @if (pm.maxCollections) {
                        <li>
                          {{
                            "limitedCollections"
                              | vfo1I18n
                                : "limitedSharedFolders"
                                : pm.maxCollections
                          }}
                        </li>
                      }
                      @if (pm.maxAdditionalSeats) {
                        <li>
                          {{
                            "addShareLimitedUsers" | i18n: pm.maxAdditionalSeats
                          }}
                        </li>
                      }
                      @if (!pm.maxCollections) {
                        <li>
                          {{
                            "createUnlimitedCollections"
                              | vfo1I18n: "createUnlimitedSharedFolders"
                          }}
                        </li>
                      }
                      @if (pm.baseStorageGb) {
                        <li>
                          {{
                            "gbEncryptedFileStorage"
                              | i18n: pm.baseStorageGb + " GB"
                          }}
                        </li>
                      }
                      @if (hasGroups) {
                        <li>{{ "controlAccessWithGroups" | i18n }}</li>
                      }
                      @if (hasApi) {
                        <li>{{ "trackAuditLogs" | i18n }}</li>
                      }
                      @if (hasDirectory) {
                        <li>{{ "syncUsersFromDirectory" | i18n }}</li>
                      }
                      @if (hasSelfHost) {
                        <li>{{ "onPremHostingOptional" | i18n }}</li>
                      }
                      @if (usersGetPremium) {
                        <li>{{ "usersGetPremium" | i18n }}</li>
                      }
                      @if (tier != productTypes.Free) {
                        <li>{{ "priorityCustomerSupport" | i18n }}</li>
                      }
                      @if (showTrialOffer) {
                        <li>{{ "xDayFreeTrial" | i18n: effectiveTrial }}</li>
                      }
                    </ul>
                  }
                }
              </bit-hint>
            </bit-radio-button>

            @if (tier != productTypes.Free && !sponsoredPlanLayout()) {
              <span class="tw-pl-4">
                @if (pm.basePrice && !acceptingSponsorship()) {
                  {{
                    (selectableProduct.isAnnual
                      ? pm.basePrice / 12
                      : pm.basePrice
                    ) | currency: "$"
                  }}
                  /{{ "month" | i18n }},
                  {{ "includesXUsers" | i18n: pm.baseSeats }}
                  @if (pm.hasAdditionalSeatsOption) {
                    {{ ("additionalUsers" | i18n).toLowerCase() }}
                    {{
                      (selectableProduct.isAnnual
                        ? pm.seatPrice / 12
                        : pm.seatPrice
                      ) | currency: "$"
                    }}
                    /{{ "month" | i18n }}
                  }
                }
              </span>
            }
            @if (!pm.basePrice && pm.hasAdditionalSeatsOption) {
              <span class="tw-pl-4">
                {{
                  "costPerUser"
                    | i18n
                      : ((selectableProduct.isAnnual
                          ? pm.seatPrice / 12
                          : pm.seatPrice
                        ) | currency: "$")
                }}
                /{{ "month" | i18n }}
              </span>
            }
            @if (tier == productTypes.Free) {
              <span class="tw-pl-4">{{ "freeForever" | i18n }}</span>
            }
          </div>
        }
      </bit-radio-group>
    </bit-section>
    @if (formGroup.value.productTier !== productTypes.Free) {
      @let plan = selectedPlan();
      <bit-section [disableMargin]="sponsoredPlanLayout()">
        @if (
          plan.PasswordManager.hasAdditionalSeatsOption &&
          !plan.PasswordManager.baseSeats &&
          !upgradingFromPremium
        ) {
          <bit-section
            [disableMargin]="sponsoredPlanLayout()"
            [class.tw-block]="sponsoredPlanLayout()"
            [class.tw-mb-6]="sponsoredPlanLayout()"
          >
            <h2 bitTypography="h2">{{ "users" | i18n }}</h2>
            <div class="tw-grid tw-grid-cols-12 tw-gap-4">
              <bit-form-field class="tw-col-span-6">
                <bit-label>{{ "userSeats" | i18n }}</bit-label>
                <input
                  bitInput
                  type="number"
                  formControlName="additionalSeats"
                  placeholder="{{ 'userSeatsDesc' | i18n }}"
                  required
                />
                <bit-hint class="tw-text-sm">{{
                  "userSeatsHowManyDesc" | i18n
                }}</bit-hint>
              </bit-form-field>
            </div>
          </bit-section>
        }
        @if (!upgradingFromPremium) {
          <bit-section
            [disableMargin]="sponsoredPlanLayout()"
            [class.tw-block]="sponsoredPlanLayout()"
            [class.tw-mb-6]="sponsoredPlanLayout()"
          >
            <h2 bitTypography="h2">{{ "addons" | i18n }}</h2>
            @if (
              plan.PasswordManager.hasAdditionalSeatsOption &&
              plan.PasswordManager.baseSeats
            ) {
              <div class="tw-grid tw-grid-cols-12 tw-gap-4">
                <bit-form-field class="tw-col-span-6">
                  <bit-label>{{ "additionalUserSeats" | i18n }}</bit-label>
                  <input
                    bitInput
                    type="number"
                    formControlName="additionalSeats"
                    placeholder="{{ 'userSeatsDesc' | i18n }}"
                  />
                  <bit-hint class="tw-text-sm"
                    >{{
                      "userSeatsAdditionalDesc"
                        | i18n
                          : plan.PasswordManager.baseSeats
                          : (seatPriceMonthly(plan) | currency: "$")
                    }}
                  </bit-hint>
                </bit-form-field>
              </div>
            }
            <div class="tw-grid tw-grid-cols-12 tw-gap-4">
              <bit-form-field class="tw-col-span-6">
                <bit-label>{{ "additionalStorageGb" | i18n }}</bit-label>
                <input
                  bitInput
                  type="number"
                  formControlName="additionalStorage"
                  step="1"
                  placeholder="{{ 'additionalStorageGbDesc' | i18n }}"
                />
                <bit-hint class="tw-text-sm">{{
                  "additionalStorageIntervalDesc"
                    | i18n
                      : `${plan.PasswordManager.baseStorageGb} GB`
                      : (additionalStoragePriceMonthly(plan) | currency: "$")
                      : ("month" | i18n)
                }}</bit-hint>
              </bit-form-field>
            </div>
          </bit-section>
        }
        <bit-section [disableMargin]="sponsoredPlanLayout()">
          @if (plan.PasswordManager.hasPremiumAccessOption) {
            <div class="tw-grid tw-grid-cols-12 tw-gap-4">
              <bit-form-control class="tw-col-span-6">
                <bit-label>{{ "premiumAccess" | i18n }}</bit-label>
                <input
                  type="checkbox"
                  bitCheckbox
                  formControlName="premiumAccessAddon"
                />
                <bit-hint class="tw-text-sm">{{
                  "premiumAccessDesc"
                    | i18n: (3.33 | currency: "$") : ("month" | i18n)
                }}</bit-hint>
              </bit-form-control>
            </div>
          }
        </bit-section>
        @if (!upgradingFromPremium) {
          <bit-section
            [disableMargin]="sponsoredPlanLayout()"
            [class.tw-block]="sponsoredPlanLayout()"
            [class.tw-mb-6]="sponsoredPlanLayout()"
          >
            <h2 bitTypography="h2">{{ "summary" | i18n }}</h2>
            <bit-radio-group formControlName="plan">
              @for (selectablePlan of selectablePlans(); track $index) {
                @let pm = selectablePlan.PasswordManager;
                @let isAnnual = selectablePlan.isAnnual;
                <bit-radio-button
                  type="radio"
                  id="interval{{ selectablePlan.type }}"
                  [value]="selectablePlan.type"
                >
                  <bit-label>{{
                    (isAnnual ? "annually" : "monthly") | i18n
                  }}</bit-label>
                  @if (isAnnual) {
                    <bit-hint>
                      @if (pm.basePrice) {
                        <p class="tw-mb-0" bitTypography="body2">
                          {{ "basePrice" | i18n }}:
                          {{
                            (isAnnual ? pm.basePrice / 12 : pm.basePrice)
                              | currency: "$"
                          }}
                          &times; 12
                          {{ "monthAbbr" | i18n }}
                          =
                          @if (acceptingSponsorship()) {
                            <span class="tw-line-through">{{
                              pm.basePrice | currency: "$"
                            }}</span>
                            {{ "freeWithSponsorship" | i18n }}
                          } @else {
                            {{ pm.basePrice | currency: "$" }}
                            /{{ "year" | i18n }}
                          }
                        </p>
                      }
                      @if (pm.hasAdditionalSeatsOption) {
                        <p class="tw-mb-0" bitTypography="body2">
                          @if (pm.baseSeats) {
                            <span>{{ "additionalUsers" | i18n }}:</span>
                          } @else {
                            <span>{{ "users" | i18n }}:</span>
                          }
                          {{ formGroup.controls.additionalSeats.value || 0 }}
                          &times;
                          {{
                            (isAnnual ? pm.seatPrice / 12 : pm.seatPrice)
                              | currency: "$"
                          }}
                          &times; 12 {{ "monthAbbr" | i18n }} =
                          {{
                            passwordManagerSeatTotal(
                              selectablePlan,
                              formGroup.value.additionalSeats
                            ) | currency: "$"
                          }}
                          /{{ "year" | i18n }}
                        </p>
                      }
                      @if (pm.hasAdditionalStorageOption) {
                        <p class="tw-mb-0" bitTypography="body2">
                          {{ "additionalStorageGb" | i18n }}:
                          {{ formGroup.controls.additionalStorage.value || 0 }}
                          &times;
                          {{
                            (isAnnual
                              ? pm.additionalStoragePricePerGb / 12
                              : pm.additionalStoragePricePerGb
                            ) | currency: "$"
                          }}
                          &times; 12 {{ "monthAbbr" | i18n }} =
                          {{
                            additionalStorageTotal(selectablePlan)
                              | currency: "$"
                          }}
                          /{{ "year" | i18n }}
                        </p>
                      }
                    </bit-hint>
                  } @else {
                    <bit-hint>
                      @if (pm.basePrice) {
                        <p class="tw-mb-0" bitTypography="body2">
                          {{ "basePrice" | i18n }}:
                          {{ pm.basePrice | currency: "$" }}
                          {{ "monthAbbr" | i18n }}
                          =
                          {{ pm.basePrice | currency: "$" }}
                          /{{ "month" | i18n }}
                        </p>
                      }
                      @if (pm.hasAdditionalSeatsOption) {
                        <p class="tw-mb-0" bitTypography="body2">
                          @if (pm.baseSeats) {
                            <span>{{ "additionalUsers" | i18n }}:</span>
                          } @else {
                            <span>{{ "users" | i18n }}:</span>
                          }
                          {{ formGroup.controls.additionalSeats.value || 0 }}
                          &times;
                          {{ pm.seatPrice | currency: "$" }}
                          {{ "monthAbbr" | i18n }} =
                          {{
                            passwordManagerSeatTotal(
                              selectablePlan,
                              formGroup.value.additionalSeats
                            ) | currency: "$"
                          }}
                          /{{ "month" | i18n }}
                        </p>
                      }
                      @if (pm.hasAdditionalStorageOption) {
                        <p class="tw-mb-0" bitTypography="body2">
                          {{ "additionalStorageGb" | i18n }}:
                          {{ formGroup.controls.additionalStorage.value || 0 }}
                          &times;
                          {{ pm.additionalStoragePricePerGb | currency: "$" }}
                          {{ "monthAbbr" | i18n }} =
                          {{
                            additionalStorageTotal(selectablePlan)
                              | currency: "$"
                          }}
                          /{{ "month" | i18n }}
                        </p>
                      }
                    </bit-hint>
                  }
                </bit-radio-button>
              }
            </bit-radio-group>
          </bit-section>
        }
      </bit-section>
    }
    <!-- Secrets Manager -->
    <bit-section [disableMargin]="sponsoredPlanLayout()">
      @if (planOffersSecretsManager() && !upgradingFromPremium) {
        <sm-subscribe
          [formGroup]="formGroup.controls.secretsManager"
          [selectedPlan]="selectedSecretsManagerPlan()"
        ></sm-subscribe>
      }
    </bit-section>

    <!-- Payment info -->
    @if (formGroup.value.productTier !== productTypes.Free) {
      <bit-section
        [disableMargin]="sponsoredPlanLayout()"
        [class.tw-block]="sponsoredPlanLayout()"
        [class.tw-mb-6]="sponsoredPlanLayout()"
      >
        <h2 bitTypography="h2">
          {{ "paymentInformation" | i18n }}
        </h2>
        <p
          class="tw-text-muted tw-italic tw-mb-3 tw-block"
          bitTypography="body2"
        >
          {{ paymentDesc }}
        </p>
        <app-enter-payment-method
          #enterPaymentMethodComponent
          [group]="billingFormGroup.controls.paymentMethod"
          [showBankAccount]="!upgradingFromPremium"
        ></app-enter-payment-method>
        <app-enter-billing-address
          [group]="billingFormGroup.controls.billingAddress"
          [scenario]="{ type: 'checkout', supportsTaxId: showTaxIdField() }"
          class="tw-my-4"
        >
        </app-enter-billing-address>
        <div id="price" class="tw-my-4">
          @if (previewFailed()) {
            <bit-callout type="danger" data-testid="invoice-preview-error">
              {{ previewErrorMessageKey() | i18n }}
            </bit-callout>
          }
          <billing-cart-summary
            [cart]="cart()"
            [showDiscountBadges]="true"
            [class.tw-hidden]="previewFailed()"
          />
        </div>
      </bit-section>
    }
    @if (singleOrgPolicyBlock) {
      <bit-section
        [disableMargin]="sponsoredPlanLayout()"
        [class.tw-block]="sponsoredPlanLayout()"
        [class.tw-mb-6]="sponsoredPlanLayout()"
      >
        <bit-callout type="danger" [title]="'error' | i18n">
          {{ "singleOrgBlockCreateMessage" | i18n }}
        </bit-callout>
      </bit-section>
    }
    <bit-section
      [disableMargin]="sponsoredPlanLayout()"
      [class.tw-block]="sponsoredPlanLayout()"
      [class.tw-mb-6]="sponsoredPlanLayout()"
    >
      <button
        type="submit"
        buttonType="primary"
        bitButton
        bitFormButton
        [disabled]="!formGroup.valid || previewFailed()"
      >
        {{ "submit" | i18n }}
      </button>
      <bit-error-summary [formGroup]="formGroup"></bit-error-summary>
    </bit-section>
  </form>
}

`````

### Expected (prettier)

`````html
@let isLoading = loading;
@let passwordManagerPlans = this.passwordManagerPlans;
@let secretsManagerPlans = this.secretsManagerPlans;
@let upgradingFromPremium = this.canUpgradeFromPremium();

@if (isLoading) {
  <ng-container>
    <bit-icon
      name="bwi-spinner"
      class="bwi-spin tw-text-muted"
      title="{{ 'loading' | i18n }}"
      aria-hidden="true"
    ></bit-icon>
    <span class="tw-sr-only">{{ "loading" | i18n }}</span>
  </ng-container>
}
@if (selfHosted) {
  <ng-container>
    <organization-self-hosting-license-uploader
      (onLicenseFileUploaded)="onLicenseFileUploaded($event)"
    />
  </ng-container>
}
@if (!isLoading && !selfHosted && passwordManagerPlans && secretsManagerPlans) {
  <form
    [formGroup]="formGroup"
    [bitSubmit]="submit"
    [class.tw-pt-6]="!sponsoredPlanLayout()"
    [class.tw-pt-2]="sponsoredPlanLayout()"
  >
    <bit-section
      [class.tw-block]="sponsoredPlanLayout()"
      [class.tw-mb-6]="sponsoredPlanLayout()"
      [disableMargin]="sponsoredPlanLayout()"
    >
      <app-org-info
        (changedBusinessOwned)="changedOwnedBusiness()"
        [formGroup]="formGroup"
        [acceptingSponsorship]="acceptingSponsorship()"
      >
      </app-org-info>
    </bit-section>
    <bit-section
      [disableMargin]="sponsoredPlanLayout()"
      [class.tw-block]="sponsoredPlanLayout()"
      [class.tw-mb-6]="sponsoredPlanLayout()"
    >
      @if (acceptingSponsorship()) {
        <h2 bitTypography="h2">
          {{ "chooseYourPlan" | vfo1I18n: "sponsoredPlanDetails" }}
        </h2>
      } @else {
        <h2 bitTypography="h2">
          {{ "chooseYourPlan" | vfo1I18n: "choosePlan" }}
        </h2>
      }
      <bit-radio-group formControlName="productTier" [block]="true">
        @for (selectableProduct of selectableProducts(); track $index) {
          @let pm = selectableProduct.PasswordManager;
          @let tier = selectableProduct.productTier;
          @let hasSelfHost = selectableProduct.hasSelfHost;
          @let hasSso = selectableProduct.hasSso;
          @let hasPolicies = selectableProduct.hasPolicies;
          @let hasGroups = selectableProduct.hasGroups;
          @let hasApi = selectableProduct.hasApi;
          @let hasDirectory = selectableProduct.hasDirectory;
          @let usersGetPremium = selectableProduct.usersGetPremium;
          @let trialPeriodDays = selectableProduct.trialPeriodDays;
          @let effectiveTrial = trialLength() ?? trialPeriodDays;
          @let showTrialOffer =
            tier != productTypes.Free &&
            effectiveTrial &&
            !upgradingFromPremium;

          <div [class.tw-mb-3]="!sponsoredPlanLayout()">
            <bit-radio-button [value]="tier" (change)="changedProduct()">
              <bit-label>{{
                selectableProduct.nameLocalizationKey | i18n
              }}</bit-label>
              <bit-hint class="tw-text-sm"
                >{{ selectableProduct.descriptionLocalizationKey | i18n: "1" }}
                @if (tier === productTypes.Enterprise) {
                  <ul class="tw-pl-0 tw-list-inside tw-mb-0">
                    <li>{{ "includeAllTeamsFeatures" | i18n }}</li>
                    @if (hasSelfHost) {
                      <li>{{ "onPremHostingOptional" | i18n }}</li>
                    }
                    @if (hasSso) {
                      <li>{{ "includeSsoAuthentication" | i18n }}</li>
                    }
                    @if (hasPolicies) {
                      <li>{{ "includeEnterprisePolicies" | i18n }}</li>
                    }
                    @if (showTrialOffer) {
                      <li>{{ "xDayFreeTrial" | i18n: effectiveTrial }}</li>
                    }
                  </ul>
                } @else {
                  @if (
                    tier === productTypes.Teams && teamsStarterPlanIsAvailable()
                  ) {
                    <ul class="tw-pl-0 tw-list-inside tw-mb-0">
                      <li>{{ "includeAllTeamsStarterFeatures" | i18n }}</li>
                      <li>{{ "chooseMonthlyOrAnnualBilling" | i18n }}</li>
                      <li>{{ "abilityToAddMoreThanNMembers" | i18n: 10 }}</li>
                      @if (showTrialOffer) {
                        <li>
                          {{ "xDayFreeTrial" | i18n: effectiveTrial }}
                        </li>
                      }
                    </ul>
                  } @else {
                    <ul class="tw-pl-0 tw-list-inside tw-mb-0">
                      @if (tier == productTypes.Free) {
                        <li>{{ "limitedUsers" | i18n: pm.maxSeats }}</li>
                      }
                      @if (
                        tier != productTypes.Free &&
                        tier != productTypes.TeamsStarter &&
                        pm.maxSeats
                      ) {
                        <li>
                          {{ "addShareLimitedUsers" | i18n: pm.maxSeats }}
                        </li>
                      }
                      @if (!pm.maxSeats) {
                        <li>{{ "addShareUnlimitedUsers" | i18n }}</li>
                      }
                      @if (pm.maxCollections) {
                        <li>
                          {{
                            "limitedCollections"
                              | vfo1I18n
                                : "limitedSharedFolders"
                                : pm.maxCollections
                          }}
                        </li>
                      }
                      @if (pm.maxAdditionalSeats) {
                        <li>
                          {{
                            "addShareLimitedUsers" | i18n: pm.maxAdditionalSeats
                          }}
                        </li>
                      }
                      @if (!pm.maxCollections) {
                        <li>
                          {{
                            "createUnlimitedCollections"
                              | vfo1I18n: "createUnlimitedSharedFolders"
                          }}
                        </li>
                      }
                      @if (pm.baseStorageGb) {
                        <li>
                          {{
                            "gbEncryptedFileStorage"
                              | i18n: pm.baseStorageGb + " GB"
                          }}
                        </li>
                      }
                      @if (hasGroups) {
                        <li>{{ "controlAccessWithGroups" | i18n }}</li>
                      }
                      @if (hasApi) {
                        <li>{{ "trackAuditLogs" | i18n }}</li>
                      }
                      @if (hasDirectory) {
                        <li>{{ "syncUsersFromDirectory" | i18n }}</li>
                      }
                      @if (hasSelfHost) {
                        <li>{{ "onPremHostingOptional" | i18n }}</li>
                      }
                      @if (usersGetPremium) {
                        <li>{{ "usersGetPremium" | i18n }}</li>
                      }
                      @if (tier != productTypes.Free) {
                        <li>{{ "priorityCustomerSupport" | i18n }}</li>
                      }
                      @if (showTrialOffer) {
                        <li>{{ "xDayFreeTrial" | i18n: effectiveTrial }}</li>
                      }
                    </ul>
                  }
                }
              </bit-hint>
            </bit-radio-button>

            @if (tier != productTypes.Free && !sponsoredPlanLayout()) {
              <span class="tw-pl-4">
                @if (pm.basePrice && !acceptingSponsorship()) {
                  {{
                    (selectableProduct.isAnnual
                      ? pm.basePrice / 12
                      : pm.basePrice
                    ) | currency: "$"
                  }}
                  /{{ "month" | i18n }},
                  {{ "includesXUsers" | i18n: pm.baseSeats }}
                  @if (pm.hasAdditionalSeatsOption) {
                    {{ ("additionalUsers" | i18n).toLowerCase() }}
                    {{
                      (selectableProduct.isAnnual
                        ? pm.seatPrice / 12
                        : pm.seatPrice
                      ) | currency: "$"
                    }}
                    /{{ "month" | i18n }}
                  }
                }
              </span>
            }
            @if (!pm.basePrice && pm.hasAdditionalSeatsOption) {
              <span class="tw-pl-4">
                {{
                  "costPerUser"
                    | i18n
                      : ((selectableProduct.isAnnual
                          ? pm.seatPrice / 12
                          : pm.seatPrice
                        )
                          | currency: "$")
                }}
                /{{ "month" | i18n }}
              </span>
            }
            @if (tier == productTypes.Free) {
              <span class="tw-pl-4">{{ "freeForever" | i18n }}</span>
            }
          </div>
        }
      </bit-radio-group>
    </bit-section>
    @if (formGroup.value.productTier !== productTypes.Free) {
      @let plan = selectedPlan();
      <bit-section [disableMargin]="sponsoredPlanLayout()">
        @if (
          plan.PasswordManager.hasAdditionalSeatsOption &&
          !plan.PasswordManager.baseSeats &&
          !upgradingFromPremium
        ) {
          <bit-section
            [disableMargin]="sponsoredPlanLayout()"
            [class.tw-block]="sponsoredPlanLayout()"
            [class.tw-mb-6]="sponsoredPlanLayout()"
          >
            <h2 bitTypography="h2">{{ "users" | i18n }}</h2>
            <div class="tw-grid tw-grid-cols-12 tw-gap-4">
              <bit-form-field class="tw-col-span-6">
                <bit-label>{{ "userSeats" | i18n }}</bit-label>
                <input
                  bitInput
                  type="number"
                  formControlName="additionalSeats"
                  placeholder="{{ 'userSeatsDesc' | i18n }}"
                  required
                />
                <bit-hint class="tw-text-sm">{{
                  "userSeatsHowManyDesc" | i18n
                }}</bit-hint>
              </bit-form-field>
            </div>
          </bit-section>
        }
        @if (!upgradingFromPremium) {
          <bit-section
            [disableMargin]="sponsoredPlanLayout()"
            [class.tw-block]="sponsoredPlanLayout()"
            [class.tw-mb-6]="sponsoredPlanLayout()"
          >
            <h2 bitTypography="h2">{{ "addons" | i18n }}</h2>
            @if (
              plan.PasswordManager.hasAdditionalSeatsOption &&
              plan.PasswordManager.baseSeats
            ) {
              <div class="tw-grid tw-grid-cols-12 tw-gap-4">
                <bit-form-field class="tw-col-span-6">
                  <bit-label>{{ "additionalUserSeats" | i18n }}</bit-label>
                  <input
                    bitInput
                    type="number"
                    formControlName="additionalSeats"
                    placeholder="{{ 'userSeatsDesc' | i18n }}"
                  />
                  <bit-hint class="tw-text-sm"
                    >{{
                      "userSeatsAdditionalDesc"
                        | i18n
                          : plan.PasswordManager.baseSeats
                          : (seatPriceMonthly(plan) | currency: "$")
                    }}
                  </bit-hint>
                </bit-form-field>
              </div>
            }
            <div class="tw-grid tw-grid-cols-12 tw-gap-4">
              <bit-form-field class="tw-col-span-6">
                <bit-label>{{ "additionalStorageGb" | i18n }}</bit-label>
                <input
                  bitInput
                  type="number"
                  formControlName="additionalStorage"
                  step="1"
                  placeholder="{{ 'additionalStorageGbDesc' | i18n }}"
                />
                <bit-hint class="tw-text-sm">{{
                  "additionalStorageIntervalDesc"
                    | i18n
                      : `${plan.PasswordManager.baseStorageGb} GB`
                      : (additionalStoragePriceMonthly(plan) | currency: "$")
                      : ("month" | i18n)
                }}</bit-hint>
              </bit-form-field>
            </div>
          </bit-section>
        }
        <bit-section [disableMargin]="sponsoredPlanLayout()">
          @if (plan.PasswordManager.hasPremiumAccessOption) {
            <div class="tw-grid tw-grid-cols-12 tw-gap-4">
              <bit-form-control class="tw-col-span-6">
                <bit-label>{{ "premiumAccess" | i18n }}</bit-label>
                <input
                  type="checkbox"
                  bitCheckbox
                  formControlName="premiumAccessAddon"
                />
                <bit-hint class="tw-text-sm">{{
                  "premiumAccessDesc"
                    | i18n: (3.33 | currency: "$") : ("month" | i18n)
                }}</bit-hint>
              </bit-form-control>
            </div>
          }
        </bit-section>
        @if (!upgradingFromPremium) {
          <bit-section
            [disableMargin]="sponsoredPlanLayout()"
            [class.tw-block]="sponsoredPlanLayout()"
            [class.tw-mb-6]="sponsoredPlanLayout()"
          >
            <h2 bitTypography="h2">{{ "summary" | i18n }}</h2>
            <bit-radio-group formControlName="plan">
              @for (selectablePlan of selectablePlans(); track $index) {
                @let pm = selectablePlan.PasswordManager;
                @let isAnnual = selectablePlan.isAnnual;
                <bit-radio-button
                  type="radio"
                  id="interval{{ selectablePlan.type }}"
                  [value]="selectablePlan.type"
                >
                  <bit-label>{{
                    (isAnnual ? "annually" : "monthly") | i18n
                  }}</bit-label>
                  @if (isAnnual) {
                    <bit-hint>
                      @if (pm.basePrice) {
                        <p class="tw-mb-0" bitTypography="body2">
                          {{ "basePrice" | i18n }}:
                          {{
                            (isAnnual ? pm.basePrice / 12 : pm.basePrice)
                              | currency: "$"
                          }}
                          &times; 12
                          {{ "monthAbbr" | i18n }}
                          =
                          @if (acceptingSponsorship()) {
                            <span class="tw-line-through">{{
                              pm.basePrice | currency: "$"
                            }}</span>
                            {{ "freeWithSponsorship" | i18n }}
                          } @else {
                            {{ pm.basePrice | currency: "$" }}
                            /{{ "year" | i18n }}
                          }
                        </p>
                      }
                      @if (pm.hasAdditionalSeatsOption) {
                        <p class="tw-mb-0" bitTypography="body2">
                          @if (pm.baseSeats) {
                            <span>{{ "additionalUsers" | i18n }}:</span>
                          } @else {
                            <span>{{ "users" | i18n }}:</span>
                          }
                          {{ formGroup.controls.additionalSeats.value || 0 }}
                          &times;
                          {{
                            (isAnnual ? pm.seatPrice / 12 : pm.seatPrice)
                              | currency: "$"
                          }}
                          &times; 12 {{ "monthAbbr" | i18n }} =
                          {{
                            passwordManagerSeatTotal(
                              selectablePlan,
                              formGroup.value.additionalSeats
                            ) | currency: "$"
                          }}
                          /{{ "year" | i18n }}
                        </p>
                      }
                      @if (pm.hasAdditionalStorageOption) {
                        <p class="tw-mb-0" bitTypography="body2">
                          {{ "additionalStorageGb" | i18n }}:
                          {{ formGroup.controls.additionalStorage.value || 0 }}
                          &times;
                          {{
                            (isAnnual
                              ? pm.additionalStoragePricePerGb / 12
                              : pm.additionalStoragePricePerGb
                            ) | currency: "$"
                          }}
                          &times; 12 {{ "monthAbbr" | i18n }} =
                          {{
                            additionalStorageTotal(selectablePlan)
                              | currency: "$"
                          }}
                          /{{ "year" | i18n }}
                        </p>
                      }
                    </bit-hint>
                  } @else {
                    <bit-hint>
                      @if (pm.basePrice) {
                        <p class="tw-mb-0" bitTypography="body2">
                          {{ "basePrice" | i18n }}:
                          {{ pm.basePrice | currency: "$" }}
                          {{ "monthAbbr" | i18n }}
                          =
                          {{ pm.basePrice | currency: "$" }}
                          /{{ "month" | i18n }}
                        </p>
                      }
                      @if (pm.hasAdditionalSeatsOption) {
                        <p class="tw-mb-0" bitTypography="body2">
                          @if (pm.baseSeats) {
                            <span>{{ "additionalUsers" | i18n }}:</span>
                          } @else {
                            <span>{{ "users" | i18n }}:</span>
                          }
                          {{ formGroup.controls.additionalSeats.value || 0 }}
                          &times;
                          {{ pm.seatPrice | currency: "$" }}
                          {{ "monthAbbr" | i18n }} =
                          {{
                            passwordManagerSeatTotal(
                              selectablePlan,
                              formGroup.value.additionalSeats
                            ) | currency: "$"
                          }}
                          /{{ "month" | i18n }}
                        </p>
                      }
                      @if (pm.hasAdditionalStorageOption) {
                        <p class="tw-mb-0" bitTypography="body2">
                          {{ "additionalStorageGb" | i18n }}:
                          {{ formGroup.controls.additionalStorage.value || 0 }}
                          &times;
                          {{ pm.additionalStoragePricePerGb | currency: "$" }}
                          {{ "monthAbbr" | i18n }} =
                          {{
                            additionalStorageTotal(selectablePlan)
                              | currency: "$"
                          }}
                          /{{ "month" | i18n }}
                        </p>
                      }
                    </bit-hint>
                  }
                </bit-radio-button>
              }
            </bit-radio-group>
          </bit-section>
        }
      </bit-section>
    }
    <!-- Secrets Manager -->
    <bit-section [disableMargin]="sponsoredPlanLayout()">
      @if (planOffersSecretsManager() && !upgradingFromPremium) {
        <sm-subscribe
          [formGroup]="formGroup.controls.secretsManager"
          [selectedPlan]="selectedSecretsManagerPlan()"
        ></sm-subscribe>
      }
    </bit-section>

    <!-- Payment info -->
    @if (formGroup.value.productTier !== productTypes.Free) {
      <bit-section
        [disableMargin]="sponsoredPlanLayout()"
        [class.tw-block]="sponsoredPlanLayout()"
        [class.tw-mb-6]="sponsoredPlanLayout()"
      >
        <h2 bitTypography="h2">
          {{ "paymentInformation" | i18n }}
        </h2>
        <p
          class="tw-text-muted tw-italic tw-mb-3 tw-block"
          bitTypography="body2"
        >
          {{ paymentDesc }}
        </p>
        <app-enter-payment-method
          #enterPaymentMethodComponent
          [group]="billingFormGroup.controls.paymentMethod"
          [showBankAccount]="!upgradingFromPremium"
        ></app-enter-payment-method>
        <app-enter-billing-address
          [group]="billingFormGroup.controls.billingAddress"
          [scenario]="{ type: 'checkout', supportsTaxId: showTaxIdField() }"
          class="tw-my-4"
        >
        </app-enter-billing-address>
        <div id="price" class="tw-my-4">
          @if (previewFailed()) {
            <bit-callout type="danger" data-testid="invoice-preview-error">
              {{ previewErrorMessageKey() | i18n }}
            </bit-callout>
          }
          <billing-cart-summary
            [cart]="cart()"
            [showDiscountBadges]="true"
            [class.tw-hidden]="previewFailed()"
          />
        </div>
      </bit-section>
    }
    @if (singleOrgPolicyBlock) {
      <bit-section
        [disableMargin]="sponsoredPlanLayout()"
        [class.tw-block]="sponsoredPlanLayout()"
        [class.tw-mb-6]="sponsoredPlanLayout()"
      >
        <bit-callout type="danger" [title]="'error' | i18n">
          {{ "singleOrgBlockCreateMessage" | i18n }}
        </bit-callout>
      </bit-section>
    }
    <bit-section
      [disableMargin]="sponsoredPlanLayout()"
      [class.tw-block]="sponsoredPlanLayout()"
      [class.tw-mb-6]="sponsoredPlanLayout()"
    >
      <button
        type="submit"
        buttonType="primary"
        bitButton
        bitFormButton
        [disabled]="!formGroup.valid || previewFailed()"
      >
        {{ "submit" | i18n }}
      </button>
      <bit-error-summary [formGroup]="formGroup"></bit-error-summary>
    </bit-section>
  </form>
}

`````
