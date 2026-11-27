# externals/bitwarden-license-web/auth/sso/sso-manage.component.html

## Option 1

`````json
{"printWidth":80}
`````

### Diff

`````diff
===================================================================
--- prettier
+++ oxfmt
@@ -97,12 +97,14 @@
             @if (vfo1Enabled()) {
               {{ "memberDecryptionOptionTdeDescPart1" | i18n }}
               <a bitLink routerLink="../policies">{{
                 "memberDecryptionOptionTdeDescVfo1Link1" | i18n
-              }}</a>{{ "memberDecryptionOptionTdeDescVfo1Part2" | i18n }}
+              }}</a
+              >{{ "memberDecryptionOptionTdeDescVfo1Part2" | i18n }}
               <a bitLink routerLink="../policies">{{
                 "memberDecryptionOptionTdeDescVfo1Link2" | i18n
-              }}</a>{{ "memberDecryptionOptionTdeDescVfo1Part3" | i18n }}
+              }}</a
+              >{{ "memberDecryptionOptionTdeDescVfo1Part3" | i18n }}
               <a bitLink routerLink="../policies">{{
                 "memberDecryptionOptionTdeDescVfo1Link3" | i18n
               }}</a>
               {{ "memberDecryptionOptionTdeDescVfo1Part4" | i18n }}

`````

### Actual (oxfmt)

`````html
<app-header></app-header>

<bit-container>
  <ng-container *ngIf="isInitializing">
    <i
      class="bwi bwi-spinner bwi-spin tw-text-muted"
      title="{{ 'loading' | i18n }}"
      aria-hidden="true"
    ></i>
    <span class="tw-sr-only">{{ "loading" | i18n }}</span>
  </ng-container>

  <form
    [formGroup]="ssoConfigForm"
    [bitSubmit]="submit"
    *ngIf="!isInitializing"
  >
    <p>
      {{ "ssoPolicyHelpStart" | i18n }}
      <a bitLink routerLink="../policies">{{ "ssoPolicyHelpAnchor" | i18n }}</a>
      {{ "ssoPolicyHelpEnd" | i18n }}
      <br />
    </p>

    <!-- Root form -->
    <ng-container>
      <bit-form-control>
        <bit-label>{{ "allowSso" | i18n }}</bit-label>
        <input
          bitCheckbox
          type="checkbox"
          formControlName="enabled"
          id="enabled"
        />
        <bit-hint>{{ "allowSsoDesc" | i18n }}</bit-hint>
      </bit-form-control>

      <bit-form-field>
        <bit-label>{{ "ssoIdentifier" | i18n }}</bit-label>
        <input bitInput type="text" formControlName="ssoIdentifier" />
        <bit-hint>
          {{ "ssoIdentifierHint" | i18n }}
          <a bitLink routerLink="../domain-verification">{{
            "claimedDomainsLearnMore" | i18n
          }}</a>
        </bit-hint>
      </bit-form-field>

      <bit-radio-group formControlName="memberDecryptionType" [block]="true">
        <bit-label>{{ "memberDecryptionOption" | i18n }}</bit-label>

        <bit-radio-button
          class="tw-block"
          id="memberDecryptionPass"
          [value]="memberDecryptionType.MasterPassword"
        >
          <bit-label>{{ "masterPass" | i18n }}</bit-label>
        </bit-radio-button>

        <bit-radio-button
          class="tw-block"
          id="memberDecryptionKey"
          [value]="memberDecryptionType.KeyConnector"
          [disabled]="!organization.useKeyConnector || null"
          *ngIf="showKeyConnectorOptions"
        >
          <bit-label>
            {{ "keyConnector" | i18n }}
            <a
              target="_blank"
              rel="noreferrer"
              appA11yTitle="{{ 'learnMore' | i18n }}"
              href="https://bitwarden.com/help/about-key-connector/"
              slot="end"
              bitLink
              startIcon="bwi-question-circle"
            ></a>
          </bit-label>
          <bit-hint>
            {{ "memberDecryptionKeyConnectorDescStart" | i18n }}
            <a bitLink routerLink="../policies">{{
              "memberDecryptionKeyConnectorDescLink" | i18n
            }}</a>
            {{ "memberDecryptionKeyConnectorDescEnd" | i18n }}
          </bit-hint>
        </bit-radio-button>

        <bit-radio-button
          class="tw-block"
          id="memberDecryptionTde"
          [value]="memberDecryptionType.TrustedDeviceEncryption"
        >
          <bit-label>
            {{ "trustedDevices" | i18n }}
          </bit-label>
          <bit-hint>
            @if (vfo1Enabled()) {
              {{ "memberDecryptionOptionTdeDescPart1" | i18n }}
              <a bitLink routerLink="../policies">{{
                "memberDecryptionOptionTdeDescVfo1Link1" | i18n
              }}</a
              >{{ "memberDecryptionOptionTdeDescVfo1Part2" | i18n }}
              <a bitLink routerLink="../policies">{{
                "memberDecryptionOptionTdeDescVfo1Link2" | i18n
              }}</a
              >{{ "memberDecryptionOptionTdeDescVfo1Part3" | i18n }}
              <a bitLink routerLink="../policies">{{
                "memberDecryptionOptionTdeDescVfo1Link3" | i18n
              }}</a>
              {{ "memberDecryptionOptionTdeDescVfo1Part4" | i18n }}
            } @else {
              {{ "memberDecryptionOptionTdeDescPart1" | i18n }}
              <a bitLink routerLink="../policies">
                {{ "memberDecryptionOptionTdeDescLink1" | i18n }}
              </a>
              {{ "memberDecryptionOptionTdeDescPart2" | i18n }}
              <a bitLink routerLink="../policies">
                {{ "memberDecryptionOptionTdeDescLink2" | i18n }}
              </a>
              {{ "memberDecryptionOptionTdeDescPart3" | i18n }}
              <a bitLink routerLink="../policies">
                {{ "memberDecryptionOptionTdeDescLink3" | i18n }}
              </a>
              {{ "memberDecryptionOptionTdeDescPart4" | i18n }}
            }
          </bit-hint>
        </bit-radio-button>
      </bit-radio-group>

      <!-- Key Connector -->
      <ng-container
        *ngIf="
          ssoConfigForm.value.memberDecryptionType ===
            memberDecryptionType.KeyConnector && showKeyConnectorOptions
        "
      >
        <bit-callout type="warning">
          {{ "keyConnectorWarning" | i18n }}
        </bit-callout>

        <bit-form-field>
          <bit-label>{{ "keyConnectorUrl" | i18n }}</bit-label>
          <input
            bitInput
            type="text"
            required
            formControlName="keyConnectorUrl"
            appInputStripSpaces
            (input)="haveTestedKeyConnector = false"
          />
          <button
            bitSuffix
            bitLink
            [disabled]="!enableTestKeyConnector"
            type="button"
            (click)="validateKeyConnectorUrl()"
          >
            <i
              class="bwi bwi-spinner bwi-spin"
              title="{{ 'loading' | i18n }}"
              aria-hidden="true"
              *ngIf="keyConnectorUrlFormCtrl.pending"
            ></i>
            <span *ngIf="!keyConnectorUrlFormCtrl.pending">
              {{ "keyConnectorTest" | i18n }}
            </span>
          </button>
          <bit-hint
            aria-live="polite"
            *ngIf="
              haveTestedKeyConnector &&
              !keyConnectorUrlFormCtrl.hasError('invalidUrl')
            "
          >
            <small class="tw-text-success-600">
              <i class="bwi bwi-check-circle" aria-hidden="true"></i>
              {{ "keyConnectorTestSuccess" | i18n }}
            </small>
          </bit-hint>
        </bit-form-field>
      </ng-container>

      <hr class="tw-mb-4" />

      <bit-form-field>
        <bit-label>{{ "type" | i18n }}</bit-label>
        <select bitInput formControlName="configType">
          <option
            *ngFor="let o of ssoTypeOptions"
            [ngValue]="o.value"
            disabled="{{ o.disabled }}"
          >
            {{ o.name }}
          </option>
        </select>
      </bit-form-field>
    </ng-container>

    <!-- OIDC -->
    <div
      *ngIf="ssoConfigForm.get('configType').value === ssoType.OpenIdConnect"
      [formGroup]="openIdForm"
    >
      <div>
        <h2 bitTypography="h2">
          {{ "openIdConnectConfig" | i18n }}
        </h2>

        <bit-form-field>
          <bit-label>{{ "callbackPath" | i18n }}</bit-label>
          <input bitInput readonly [value]="callbackPath" />
          <button
            bitIconButton="bwi-clone"
            bitSuffix
            type="button"
            [appCopyClick]="callbackPath"
            [label]="'copyValue' | i18n"
          ></button>
        </bit-form-field>

        <bit-form-field>
          <bit-label>{{ "signedOutCallbackPath" | i18n }}</bit-label>
          <input bitInput readonly [value]="signedOutCallbackPath" />
          <button
            bitIconButton="bwi-clone"
            bitSuffix
            type="button"
            [appCopyClick]="signedOutCallbackPath"
            [label]="'copyValue' | i18n"
          ></button>
        </bit-form-field>

        <bit-form-field>
          <bit-label>{{ "authority" | i18n }}</bit-label>
          <input
            bitInput
            type="text"
            formControlName="authority"
            appInputStripSpaces
          />
        </bit-form-field>

        <bit-form-field>
          <bit-label>{{ "clientId" | i18n }}</bit-label>
          <input
            bitInput
            type="text"
            formControlName="clientId"
            appInputStripSpaces
          />
        </bit-form-field>

        <bit-form-field>
          <bit-label>{{ "clientSecret" | i18n }}</bit-label>
          <input
            bitInput
            type="password"
            formControlName="clientSecret"
            appInputStripSpaces
          />
          <button
            type="button"
            bitIconButton
            bitSuffix
            bitPasswordInputToggle
            [(toggled)]="showClientSecret"
          ></button>
        </bit-form-field>

        <bit-form-field>
          <bit-label>{{ "metadataAddress" | i18n }}</bit-label>
          <input
            bitInput
            type="text"
            formControlName="metadataAddress"
            appInputStripSpaces
          />
          <bit-hint>{{ "openIdAuthorityRequired" | i18n }}</bit-hint>
        </bit-form-field>

        <bit-form-field>
          <bit-label>{{ "oidcRedirectBehavior" | i18n }}</bit-label>
          <select bitInput formControlName="redirectBehavior">
            <option
              *ngFor="let o of connectRedirectOptions"
              [ngValue]="o.value"
              disabled="{{ o.disabled }}"
            >
              {{ o.name }}
            </option>
          </select>
        </bit-form-field>

        <bit-form-control>
          <bit-label>{{ "getClaimsFromUserInfoEndpoint" | i18n }}</bit-label>
          <input
            bitCheckbox
            type="checkbox"
            formControlName="getClaimsFromUserInfoEndpoint"
            id="getClaimsFromUserInfoEndpoint"
          />
        </bit-form-control>

        <!-- Optional customizations -->
        <div
          class="tw-flex tw-flex-row tw-items-center tw-my-3 tw-text-main hover:tw-cursor-pointer"
          (click)="toggleOpenIdCustomizations()"
        >
          <h3
            class="tw-mb-0 tw-mr-2 tw-font-normal tw-uppercase"
            id="customizations-header"
          >
            {{ "openIdOptionalCustomizations" | i18n }}
          </h3>
          <button
            bitLink
            linkType="primary"
            type="button"
            appStopClick
            role="button"
            aria-controls="customizations"
            [attr.aria-expanded]="showOpenIdCustomizations"
            aria-labelledby="customizations-header"
            [startIcon]="
              showOpenIdCustomizations ? 'bwi-angle-down' : 'bwi-angle-right'
            "
          ></button>
        </div>
        <div id="customizations" [hidden]="!showOpenIdCustomizations">
          <bit-form-field>
            <bit-label>{{ "additionalScopes" | i18n }}</bit-label>
            <input bitInput type="text" formControlName="additionalScopes" />
            <bit-hint>{{ "separateMultipleWithComma" | i18n }}</bit-hint>
          </bit-form-field>

          <bit-form-field>
            <bit-label>{{ "additionalUserIdClaimTypes" | i18n }}</bit-label>
            <input
              bitInput
              type="text"
              formControlName="additionalUserIdClaimTypes"
            />
            <bit-hint>{{ "separateMultipleWithComma" | i18n }}</bit-hint>
          </bit-form-field>

          <bit-form-field>
            <bit-label>{{ "additionalEmailClaimTypes" | i18n }}</bit-label>
            <input
              bitInput
              type="text"
              formControlName="additionalEmailClaimTypes"
            />
            <bit-hint>{{ "separateMultipleWithComma" | i18n }}</bit-hint>
          </bit-form-field>

          <bit-form-field>
            <bit-label>{{ "additionalNameClaimTypes" | i18n }}</bit-label>
            <input
              bitInput
              type="text"
              formControlName="additionalNameClaimTypes"
            />
            <bit-hint>{{ "separateMultipleWithComma" | i18n }}</bit-hint>
          </bit-form-field>

          <bit-form-field>
            <bit-label>{{ "acrValues" | i18n }}</bit-label>
            <input bitInput type="text" formControlName="acrValues" />
            <bit-hint>acr_values</bit-hint>
          </bit-form-field>

          <bit-form-field>
            <bit-label>{{ "expectedReturnAcrValue" | i18n }}</bit-label>
            <input
              bitInput
              type="text"
              formControlName="expectedReturnAcrValue"
            />
            <bit-hint>acr_validaton</bit-hint>
          </bit-form-field>
        </div>
      </div>
    </div>

    <!-- SAML2 SP -->
    <div
      *ngIf="ssoConfigForm.get('configType').value === ssoType.Saml2"
      [formGroup]="samlForm"
    >
      <!-- SAML2 SP -->
      <div>
        <h2 bitTypography="h2">
          {{ "samlSpConfig" | i18n }}
        </h2>

        <bit-form-control>
          <bit-label>{{ "spUniqueEntityId" | i18n }}</bit-label>
          <input
            bitCheckbox
            type="checkbox"
            formControlName="spUniqueEntityId"
            id="spUniqueEntityId"
          />
          <bit-hint>{{ "spUniqueEntityIdDesc" | i18n }}</bit-hint>
        </bit-form-control>

        <bit-form-field *ngIf="ssoConfigForm.value.saml.spUniqueEntityId">
          <bit-label>{{ "spEntityId" | i18n }}</bit-label>
          <input bitInput readonly [value]="spEntityId" />
          <button
            bitIconButton="bwi-clone"
            bitSuffix
            type="button"
            [appCopyClick]="spEntityId"
            [label]="'copyValue' | i18n"
          ></button>
        </bit-form-field>

        <bit-form-field *ngIf="!ssoConfigForm.value.saml.spUniqueEntityId">
          <bit-label>{{ "spEntityId" | i18n }}</bit-label>
          <input bitInput readonly [value]="spEntityIdStatic" />
          <button
            bitIconButton="bwi-clone"
            bitSuffix
            type="button"
            [appCopyClick]="spEntityIdStatic"
            [label]="'copyValue' | i18n"
          ></button>
        </bit-form-field>

        <bit-form-field>
          <bit-label>{{ "spMetadataUrl" | i18n }}</bit-label>
          <input bitInput readonly [value]="spMetadataUrl" />
          <button
            bitIconButton="bwi-external-link"
            bitSuffix
            type="button"
            [appLaunchClick]="spMetadataUrl"
            [label]="'launch' | i18n"
          ></button>
          <button
            bitIconButton="bwi-clone"
            bitSuffix
            type="button"
            [appCopyClick]="spMetadataUrl"
            [label]="'copyValue' | i18n"
          ></button>
        </bit-form-field>

        <bit-form-field>
          <bit-label>{{ "spAcsUrl" | i18n }}</bit-label>
          <input bitInput readonly [value]="spAcsUrl" />
          <button
            bitIconButton="bwi-clone"
            bitSuffix
            type="button"
            [appCopyClick]="spAcsUrl"
            [label]="'copyValue' | i18n"
          ></button>
        </bit-form-field>

        <bit-form-field>
          <bit-label>{{ "spNameIdFormat" | i18n }}</bit-label>
          <select bitInput formControlName="spNameIdFormat">
            <option
              *ngFor="let o of saml2NameIdFormatOptions"
              [ngValue]="o.value"
              disabled="{{ o.disabled }}"
            >
              {{ o.name }}
            </option>
          </select>
        </bit-form-field>

        <bit-form-field>
          <bit-label>{{ "spOutboundSigningAlgorithm" | i18n }}</bit-label>
          <select bitInput formControlName="spOutboundSigningAlgorithm">
            <option
              *ngFor="let o of samlSigningAlgorithmOptions"
              [ngValue]="o.value"
              disabled="{{ o.disabled }}"
            >
              {{ o.name }}
            </option>
          </select>
        </bit-form-field>

        <bit-form-field>
          <bit-label>{{ "spSigningBehavior" | i18n }}</bit-label>
          <select bitInput formControlName="spSigningBehavior">
            <option
              *ngFor="let o of saml2SigningBehaviourOptions"
              [ngValue]="o.value"
              disabled="{{ o.disabled }}"
            >
              {{ o.name }}
            </option>
          </select>
        </bit-form-field>

        <bit-form-field>
          <bit-label>{{ "spMinIncomingSigningAlgorithm" | i18n }}</bit-label>
          <select bitInput formControlName="spMinIncomingSigningAlgorithm">
            <option
              *ngFor="let o of samlSigningAlgorithmOptions"
              [ngValue]="o.value"
              disabled="{{ o.disabled }}"
            >
              {{ o.name }}
            </option>
          </select>
        </bit-form-field>

        <bit-form-control>
          <bit-label>{{ "spWantAssertionsSigned" | i18n }}</bit-label>
          <input
            bitCheckbox
            type="checkbox"
            formControlName="spWantAssertionsSigned"
            id="spWantAssertionsSigned"
          />
        </bit-form-control>

        <bit-form-control>
          <bit-label>{{ "spValidateCertificates" | i18n }}</bit-label>
          <input
            bitCheckbox
            type="checkbox"
            formControlName="spValidateCertificates"
            id="spValidateCertificates"
          />
        </bit-form-control>
      </div>

      <!-- SAML2 IDP -->
      <div>
        <h2 bitTypography="h2">
          {{ "samlIdpConfig" | i18n }}
        </h2>

        <bit-form-field>
          <bit-label>{{ "idpEntityId" | i18n }}</bit-label>
          <input bitInput type="text" formControlName="idpEntityId" />
        </bit-form-field>

        <bit-form-field>
          <bit-label>{{ "idpBindingType" | i18n }}</bit-label>
          <select bitInput formControlName="idpBindingType">
            <option
              *ngFor="let o of saml2BindingTypeOptions"
              [ngValue]="o.value"
              disabled="{{ o.disabled }}"
            >
              {{ o.name }}
            </option>
          </select>
        </bit-form-field>

        <bit-form-field>
          <bit-label>{{ "idpSingleSignOnServiceUrl" | i18n }}</bit-label>
          <input
            bitInput
            type="text"
            formControlName="idpSingleSignOnServiceUrl"
            appInputStripSpaces
          />
        </bit-form-field>

        <bit-form-field>
          <bit-label>{{ "idpSingleLogoutServiceUrl" | i18n }}</bit-label>
          <input
            bitInput
            type="text"
            formControlName="idpSingleLogoutServiceUrl"
            appInputStripSpaces
          />
        </bit-form-field>

        <bit-form-field>
          <bit-label>{{ "idpX509PublicCert" | i18n }}</bit-label>
          <textarea
            bitInput
            rows="6"
            formControlName="idpX509PublicCert"
          ></textarea>
        </bit-form-field>

        <bit-form-field>
          <bit-label>{{ "idpOutboundSigningAlgorithm" | i18n }}</bit-label>
          <select bitInput formControlName="idpOutboundSigningAlgorithm">
            <option
              *ngFor="let o of samlSigningAlgorithmOptions"
              [ngValue]="o.value"
              disabled="{{ o.disabled }}"
            >
              {{ o.name }}
            </option>
          </select>
        </bit-form-field>

        <!--TODO: Uncomment once Unsolicited IdP Response is supported-->
        <!-- <app-input-checkbox
          controlId="idpAllowUnsolicitedAuthnResponse"
          formControlName="idpAllowUnsolicitedAuthnResponse"
          [label]="'idpAllowUnsolicitedAuthnResponse' | i18n"
        ></app-input-checkbox> -->

        <bit-form-control>
          <bit-label>{{ "idpAllowOutboundLogoutRequests" | i18n }}</bit-label>
          <input
            bitCheckbox
            type="checkbox"
            formControlName="idpAllowOutboundLogoutRequests"
            id="idpAllowOutboundLogoutRequests"
          />
        </bit-form-control>

        <bit-form-control>
          <bit-label>{{ "idpSignAuthenticationRequests" | i18n }}</bit-label>
          <input
            bitCheckbox
            type="checkbox"
            formControlName="idpWantAuthnRequestsSigned"
            id="idpWantAuthnRequestsSigned"
          />
        </bit-form-control>
      </div>
    </div>

    <button type="submit" buttonType="primary" bitButton bitFormButton>
      {{ "save" | i18n }}
    </button>
    <bit-error-summary [formGroup]="ssoConfigForm"></bit-error-summary>
  </form>
</bit-container>

`````

### Expected (prettier)

`````html
<app-header></app-header>

<bit-container>
  <ng-container *ngIf="isInitializing">
    <i
      class="bwi bwi-spinner bwi-spin tw-text-muted"
      title="{{ 'loading' | i18n }}"
      aria-hidden="true"
    ></i>
    <span class="tw-sr-only">{{ "loading" | i18n }}</span>
  </ng-container>

  <form
    [formGroup]="ssoConfigForm"
    [bitSubmit]="submit"
    *ngIf="!isInitializing"
  >
    <p>
      {{ "ssoPolicyHelpStart" | i18n }}
      <a bitLink routerLink="../policies">{{ "ssoPolicyHelpAnchor" | i18n }}</a>
      {{ "ssoPolicyHelpEnd" | i18n }}
      <br />
    </p>

    <!-- Root form -->
    <ng-container>
      <bit-form-control>
        <bit-label>{{ "allowSso" | i18n }}</bit-label>
        <input
          bitCheckbox
          type="checkbox"
          formControlName="enabled"
          id="enabled"
        />
        <bit-hint>{{ "allowSsoDesc" | i18n }}</bit-hint>
      </bit-form-control>

      <bit-form-field>
        <bit-label>{{ "ssoIdentifier" | i18n }}</bit-label>
        <input bitInput type="text" formControlName="ssoIdentifier" />
        <bit-hint>
          {{ "ssoIdentifierHint" | i18n }}
          <a bitLink routerLink="../domain-verification">{{
            "claimedDomainsLearnMore" | i18n
          }}</a>
        </bit-hint>
      </bit-form-field>

      <bit-radio-group formControlName="memberDecryptionType" [block]="true">
        <bit-label>{{ "memberDecryptionOption" | i18n }}</bit-label>

        <bit-radio-button
          class="tw-block"
          id="memberDecryptionPass"
          [value]="memberDecryptionType.MasterPassword"
        >
          <bit-label>{{ "masterPass" | i18n }}</bit-label>
        </bit-radio-button>

        <bit-radio-button
          class="tw-block"
          id="memberDecryptionKey"
          [value]="memberDecryptionType.KeyConnector"
          [disabled]="!organization.useKeyConnector || null"
          *ngIf="showKeyConnectorOptions"
        >
          <bit-label>
            {{ "keyConnector" | i18n }}
            <a
              target="_blank"
              rel="noreferrer"
              appA11yTitle="{{ 'learnMore' | i18n }}"
              href="https://bitwarden.com/help/about-key-connector/"
              slot="end"
              bitLink
              startIcon="bwi-question-circle"
            ></a>
          </bit-label>
          <bit-hint>
            {{ "memberDecryptionKeyConnectorDescStart" | i18n }}
            <a bitLink routerLink="../policies">{{
              "memberDecryptionKeyConnectorDescLink" | i18n
            }}</a>
            {{ "memberDecryptionKeyConnectorDescEnd" | i18n }}
          </bit-hint>
        </bit-radio-button>

        <bit-radio-button
          class="tw-block"
          id="memberDecryptionTde"
          [value]="memberDecryptionType.TrustedDeviceEncryption"
        >
          <bit-label>
            {{ "trustedDevices" | i18n }}
          </bit-label>
          <bit-hint>
            @if (vfo1Enabled()) {
              {{ "memberDecryptionOptionTdeDescPart1" | i18n }}
              <a bitLink routerLink="../policies">{{
                "memberDecryptionOptionTdeDescVfo1Link1" | i18n
              }}</a>{{ "memberDecryptionOptionTdeDescVfo1Part2" | i18n }}
              <a bitLink routerLink="../policies">{{
                "memberDecryptionOptionTdeDescVfo1Link2" | i18n
              }}</a>{{ "memberDecryptionOptionTdeDescVfo1Part3" | i18n }}
              <a bitLink routerLink="../policies">{{
                "memberDecryptionOptionTdeDescVfo1Link3" | i18n
              }}</a>
              {{ "memberDecryptionOptionTdeDescVfo1Part4" | i18n }}
            } @else {
              {{ "memberDecryptionOptionTdeDescPart1" | i18n }}
              <a bitLink routerLink="../policies">
                {{ "memberDecryptionOptionTdeDescLink1" | i18n }}
              </a>
              {{ "memberDecryptionOptionTdeDescPart2" | i18n }}
              <a bitLink routerLink="../policies">
                {{ "memberDecryptionOptionTdeDescLink2" | i18n }}
              </a>
              {{ "memberDecryptionOptionTdeDescPart3" | i18n }}
              <a bitLink routerLink="../policies">
                {{ "memberDecryptionOptionTdeDescLink3" | i18n }}
              </a>
              {{ "memberDecryptionOptionTdeDescPart4" | i18n }}
            }
          </bit-hint>
        </bit-radio-button>
      </bit-radio-group>

      <!-- Key Connector -->
      <ng-container
        *ngIf="
          ssoConfigForm.value.memberDecryptionType ===
            memberDecryptionType.KeyConnector && showKeyConnectorOptions
        "
      >
        <bit-callout type="warning">
          {{ "keyConnectorWarning" | i18n }}
        </bit-callout>

        <bit-form-field>
          <bit-label>{{ "keyConnectorUrl" | i18n }}</bit-label>
          <input
            bitInput
            type="text"
            required
            formControlName="keyConnectorUrl"
            appInputStripSpaces
            (input)="haveTestedKeyConnector = false"
          />
          <button
            bitSuffix
            bitLink
            [disabled]="!enableTestKeyConnector"
            type="button"
            (click)="validateKeyConnectorUrl()"
          >
            <i
              class="bwi bwi-spinner bwi-spin"
              title="{{ 'loading' | i18n }}"
              aria-hidden="true"
              *ngIf="keyConnectorUrlFormCtrl.pending"
            ></i>
            <span *ngIf="!keyConnectorUrlFormCtrl.pending">
              {{ "keyConnectorTest" | i18n }}
            </span>
          </button>
          <bit-hint
            aria-live="polite"
            *ngIf="
              haveTestedKeyConnector &&
              !keyConnectorUrlFormCtrl.hasError('invalidUrl')
            "
          >
            <small class="tw-text-success-600">
              <i class="bwi bwi-check-circle" aria-hidden="true"></i>
              {{ "keyConnectorTestSuccess" | i18n }}
            </small>
          </bit-hint>
        </bit-form-field>
      </ng-container>

      <hr class="tw-mb-4" />

      <bit-form-field>
        <bit-label>{{ "type" | i18n }}</bit-label>
        <select bitInput formControlName="configType">
          <option
            *ngFor="let o of ssoTypeOptions"
            [ngValue]="o.value"
            disabled="{{ o.disabled }}"
          >
            {{ o.name }}
          </option>
        </select>
      </bit-form-field>
    </ng-container>

    <!-- OIDC -->
    <div
      *ngIf="ssoConfigForm.get('configType').value === ssoType.OpenIdConnect"
      [formGroup]="openIdForm"
    >
      <div>
        <h2 bitTypography="h2">
          {{ "openIdConnectConfig" | i18n }}
        </h2>

        <bit-form-field>
          <bit-label>{{ "callbackPath" | i18n }}</bit-label>
          <input bitInput readonly [value]="callbackPath" />
          <button
            bitIconButton="bwi-clone"
            bitSuffix
            type="button"
            [appCopyClick]="callbackPath"
            [label]="'copyValue' | i18n"
          ></button>
        </bit-form-field>

        <bit-form-field>
          <bit-label>{{ "signedOutCallbackPath" | i18n }}</bit-label>
          <input bitInput readonly [value]="signedOutCallbackPath" />
          <button
            bitIconButton="bwi-clone"
            bitSuffix
            type="button"
            [appCopyClick]="signedOutCallbackPath"
            [label]="'copyValue' | i18n"
          ></button>
        </bit-form-field>

        <bit-form-field>
          <bit-label>{{ "authority" | i18n }}</bit-label>
          <input
            bitInput
            type="text"
            formControlName="authority"
            appInputStripSpaces
          />
        </bit-form-field>

        <bit-form-field>
          <bit-label>{{ "clientId" | i18n }}</bit-label>
          <input
            bitInput
            type="text"
            formControlName="clientId"
            appInputStripSpaces
          />
        </bit-form-field>

        <bit-form-field>
          <bit-label>{{ "clientSecret" | i18n }}</bit-label>
          <input
            bitInput
            type="password"
            formControlName="clientSecret"
            appInputStripSpaces
          />
          <button
            type="button"
            bitIconButton
            bitSuffix
            bitPasswordInputToggle
            [(toggled)]="showClientSecret"
          ></button>
        </bit-form-field>

        <bit-form-field>
          <bit-label>{{ "metadataAddress" | i18n }}</bit-label>
          <input
            bitInput
            type="text"
            formControlName="metadataAddress"
            appInputStripSpaces
          />
          <bit-hint>{{ "openIdAuthorityRequired" | i18n }}</bit-hint>
        </bit-form-field>

        <bit-form-field>
          <bit-label>{{ "oidcRedirectBehavior" | i18n }}</bit-label>
          <select bitInput formControlName="redirectBehavior">
            <option
              *ngFor="let o of connectRedirectOptions"
              [ngValue]="o.value"
              disabled="{{ o.disabled }}"
            >
              {{ o.name }}
            </option>
          </select>
        </bit-form-field>

        <bit-form-control>
          <bit-label>{{ "getClaimsFromUserInfoEndpoint" | i18n }}</bit-label>
          <input
            bitCheckbox
            type="checkbox"
            formControlName="getClaimsFromUserInfoEndpoint"
            id="getClaimsFromUserInfoEndpoint"
          />
        </bit-form-control>

        <!-- Optional customizations -->
        <div
          class="tw-flex tw-flex-row tw-items-center tw-my-3 tw-text-main hover:tw-cursor-pointer"
          (click)="toggleOpenIdCustomizations()"
        >
          <h3
            class="tw-mb-0 tw-mr-2 tw-font-normal tw-uppercase"
            id="customizations-header"
          >
            {{ "openIdOptionalCustomizations" | i18n }}
          </h3>
          <button
            bitLink
            linkType="primary"
            type="button"
            appStopClick
            role="button"
            aria-controls="customizations"
            [attr.aria-expanded]="showOpenIdCustomizations"
            aria-labelledby="customizations-header"
            [startIcon]="
              showOpenIdCustomizations ? 'bwi-angle-down' : 'bwi-angle-right'
            "
          ></button>
        </div>
        <div id="customizations" [hidden]="!showOpenIdCustomizations">
          <bit-form-field>
            <bit-label>{{ "additionalScopes" | i18n }}</bit-label>
            <input bitInput type="text" formControlName="additionalScopes" />
            <bit-hint>{{ "separateMultipleWithComma" | i18n }}</bit-hint>
          </bit-form-field>

          <bit-form-field>
            <bit-label>{{ "additionalUserIdClaimTypes" | i18n }}</bit-label>
            <input
              bitInput
              type="text"
              formControlName="additionalUserIdClaimTypes"
            />
            <bit-hint>{{ "separateMultipleWithComma" | i18n }}</bit-hint>
          </bit-form-field>

          <bit-form-field>
            <bit-label>{{ "additionalEmailClaimTypes" | i18n }}</bit-label>
            <input
              bitInput
              type="text"
              formControlName="additionalEmailClaimTypes"
            />
            <bit-hint>{{ "separateMultipleWithComma" | i18n }}</bit-hint>
          </bit-form-field>

          <bit-form-field>
            <bit-label>{{ "additionalNameClaimTypes" | i18n }}</bit-label>
            <input
              bitInput
              type="text"
              formControlName="additionalNameClaimTypes"
            />
            <bit-hint>{{ "separateMultipleWithComma" | i18n }}</bit-hint>
          </bit-form-field>

          <bit-form-field>
            <bit-label>{{ "acrValues" | i18n }}</bit-label>
            <input bitInput type="text" formControlName="acrValues" />
            <bit-hint>acr_values</bit-hint>
          </bit-form-field>

          <bit-form-field>
            <bit-label>{{ "expectedReturnAcrValue" | i18n }}</bit-label>
            <input
              bitInput
              type="text"
              formControlName="expectedReturnAcrValue"
            />
            <bit-hint>acr_validaton</bit-hint>
          </bit-form-field>
        </div>
      </div>
    </div>

    <!-- SAML2 SP -->
    <div
      *ngIf="ssoConfigForm.get('configType').value === ssoType.Saml2"
      [formGroup]="samlForm"
    >
      <!-- SAML2 SP -->
      <div>
        <h2 bitTypography="h2">
          {{ "samlSpConfig" | i18n }}
        </h2>

        <bit-form-control>
          <bit-label>{{ "spUniqueEntityId" | i18n }}</bit-label>
          <input
            bitCheckbox
            type="checkbox"
            formControlName="spUniqueEntityId"
            id="spUniqueEntityId"
          />
          <bit-hint>{{ "spUniqueEntityIdDesc" | i18n }}</bit-hint>
        </bit-form-control>

        <bit-form-field *ngIf="ssoConfigForm.value.saml.spUniqueEntityId">
          <bit-label>{{ "spEntityId" | i18n }}</bit-label>
          <input bitInput readonly [value]="spEntityId" />
          <button
            bitIconButton="bwi-clone"
            bitSuffix
            type="button"
            [appCopyClick]="spEntityId"
            [label]="'copyValue' | i18n"
          ></button>
        </bit-form-field>

        <bit-form-field *ngIf="!ssoConfigForm.value.saml.spUniqueEntityId">
          <bit-label>{{ "spEntityId" | i18n }}</bit-label>
          <input bitInput readonly [value]="spEntityIdStatic" />
          <button
            bitIconButton="bwi-clone"
            bitSuffix
            type="button"
            [appCopyClick]="spEntityIdStatic"
            [label]="'copyValue' | i18n"
          ></button>
        </bit-form-field>

        <bit-form-field>
          <bit-label>{{ "spMetadataUrl" | i18n }}</bit-label>
          <input bitInput readonly [value]="spMetadataUrl" />
          <button
            bitIconButton="bwi-external-link"
            bitSuffix
            type="button"
            [appLaunchClick]="spMetadataUrl"
            [label]="'launch' | i18n"
          ></button>
          <button
            bitIconButton="bwi-clone"
            bitSuffix
            type="button"
            [appCopyClick]="spMetadataUrl"
            [label]="'copyValue' | i18n"
          ></button>
        </bit-form-field>

        <bit-form-field>
          <bit-label>{{ "spAcsUrl" | i18n }}</bit-label>
          <input bitInput readonly [value]="spAcsUrl" />
          <button
            bitIconButton="bwi-clone"
            bitSuffix
            type="button"
            [appCopyClick]="spAcsUrl"
            [label]="'copyValue' | i18n"
          ></button>
        </bit-form-field>

        <bit-form-field>
          <bit-label>{{ "spNameIdFormat" | i18n }}</bit-label>
          <select bitInput formControlName="spNameIdFormat">
            <option
              *ngFor="let o of saml2NameIdFormatOptions"
              [ngValue]="o.value"
              disabled="{{ o.disabled }}"
            >
              {{ o.name }}
            </option>
          </select>
        </bit-form-field>

        <bit-form-field>
          <bit-label>{{ "spOutboundSigningAlgorithm" | i18n }}</bit-label>
          <select bitInput formControlName="spOutboundSigningAlgorithm">
            <option
              *ngFor="let o of samlSigningAlgorithmOptions"
              [ngValue]="o.value"
              disabled="{{ o.disabled }}"
            >
              {{ o.name }}
            </option>
          </select>
        </bit-form-field>

        <bit-form-field>
          <bit-label>{{ "spSigningBehavior" | i18n }}</bit-label>
          <select bitInput formControlName="spSigningBehavior">
            <option
              *ngFor="let o of saml2SigningBehaviourOptions"
              [ngValue]="o.value"
              disabled="{{ o.disabled }}"
            >
              {{ o.name }}
            </option>
          </select>
        </bit-form-field>

        <bit-form-field>
          <bit-label>{{ "spMinIncomingSigningAlgorithm" | i18n }}</bit-label>
          <select bitInput formControlName="spMinIncomingSigningAlgorithm">
            <option
              *ngFor="let o of samlSigningAlgorithmOptions"
              [ngValue]="o.value"
              disabled="{{ o.disabled }}"
            >
              {{ o.name }}
            </option>
          </select>
        </bit-form-field>

        <bit-form-control>
          <bit-label>{{ "spWantAssertionsSigned" | i18n }}</bit-label>
          <input
            bitCheckbox
            type="checkbox"
            formControlName="spWantAssertionsSigned"
            id="spWantAssertionsSigned"
          />
        </bit-form-control>

        <bit-form-control>
          <bit-label>{{ "spValidateCertificates" | i18n }}</bit-label>
          <input
            bitCheckbox
            type="checkbox"
            formControlName="spValidateCertificates"
            id="spValidateCertificates"
          />
        </bit-form-control>
      </div>

      <!-- SAML2 IDP -->
      <div>
        <h2 bitTypography="h2">
          {{ "samlIdpConfig" | i18n }}
        </h2>

        <bit-form-field>
          <bit-label>{{ "idpEntityId" | i18n }}</bit-label>
          <input bitInput type="text" formControlName="idpEntityId" />
        </bit-form-field>

        <bit-form-field>
          <bit-label>{{ "idpBindingType" | i18n }}</bit-label>
          <select bitInput formControlName="idpBindingType">
            <option
              *ngFor="let o of saml2BindingTypeOptions"
              [ngValue]="o.value"
              disabled="{{ o.disabled }}"
            >
              {{ o.name }}
            </option>
          </select>
        </bit-form-field>

        <bit-form-field>
          <bit-label>{{ "idpSingleSignOnServiceUrl" | i18n }}</bit-label>
          <input
            bitInput
            type="text"
            formControlName="idpSingleSignOnServiceUrl"
            appInputStripSpaces
          />
        </bit-form-field>

        <bit-form-field>
          <bit-label>{{ "idpSingleLogoutServiceUrl" | i18n }}</bit-label>
          <input
            bitInput
            type="text"
            formControlName="idpSingleLogoutServiceUrl"
            appInputStripSpaces
          />
        </bit-form-field>

        <bit-form-field>
          <bit-label>{{ "idpX509PublicCert" | i18n }}</bit-label>
          <textarea
            bitInput
            rows="6"
            formControlName="idpX509PublicCert"
          ></textarea>
        </bit-form-field>

        <bit-form-field>
          <bit-label>{{ "idpOutboundSigningAlgorithm" | i18n }}</bit-label>
          <select bitInput formControlName="idpOutboundSigningAlgorithm">
            <option
              *ngFor="let o of samlSigningAlgorithmOptions"
              [ngValue]="o.value"
              disabled="{{ o.disabled }}"
            >
              {{ o.name }}
            </option>
          </select>
        </bit-form-field>

        <!--TODO: Uncomment once Unsolicited IdP Response is supported-->
        <!-- <app-input-checkbox
          controlId="idpAllowUnsolicitedAuthnResponse"
          formControlName="idpAllowUnsolicitedAuthnResponse"
          [label]="'idpAllowUnsolicitedAuthnResponse' | i18n"
        ></app-input-checkbox> -->

        <bit-form-control>
          <bit-label>{{ "idpAllowOutboundLogoutRequests" | i18n }}</bit-label>
          <input
            bitCheckbox
            type="checkbox"
            formControlName="idpAllowOutboundLogoutRequests"
            id="idpAllowOutboundLogoutRequests"
          />
        </bit-form-control>

        <bit-form-control>
          <bit-label>{{ "idpSignAuthenticationRequests" | i18n }}</bit-label>
          <input
            bitCheckbox
            type="checkbox"
            formControlName="idpWantAuthnRequestsSigned"
            id="idpWantAuthnRequestsSigned"
          />
        </bit-form-control>
      </div>
    </div>

    <button type="submit" buttonType="primary" bitButton bitFormButton>
      {{ "save" | i18n }}
    </button>
    <bit-error-summary [formGroup]="ssoConfigForm"></bit-error-summary>
  </form>
</bit-container>

`````
