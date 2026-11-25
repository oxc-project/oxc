# externals/bitwarden-license-web/pam/approvals/withdraw-approval-dialog/withdraw-approval-dialog.component.html

## Option 1

`````json
{"printWidth":80}
`````

### Diff

`````diff
===================================================================
--- prettier
+++ oxfmt
@@ -23,11 +23,10 @@
       [title]="'warning' | i18n"
       data-testid="withdraw-approval-dialog-warning"
     >
       {{ "pamWithdrawApprovalWarningOne" | i18n }}
-      <strong>{{ params.itemName }}</strong>{{
-        "pamWithdrawApprovalWarningTwo" | i18n
-      }}
+      <strong>{{ params.itemName }}</strong
+      >{{ "pamWithdrawApprovalWarningTwo" | i18n }}
       <strong>{{ params.requesterName }}</strong>
       {{ "pamWithdrawApprovalWarningThree" | i18n }}
     </bit-callout>
 

`````

### Actual (oxfmt)

`````html
<form
  [formGroup]="formGroup"
  [bitSubmit]="confirm"
  bit-dialog
  background="alt"
  dialogSize="large"
>
  <span bitDialogTitle>{{ "pamInboxWithdrawApproval" | i18n }}</span>

  <div bitDialogContent>
    <pam-request-summary
      data-testid="withdraw-approval-dialog-item"
      hideDetails
      [cipher]="params.cipher ?? null"
      [itemName]="params.itemName"
      [organizationName]="params.organizationName"
      [collectionName]="params.collectionName"
      [requesterName]="params.requesterName"
    />

    <bit-callout
      type="warning"
      [title]="'warning' | i18n"
      data-testid="withdraw-approval-dialog-warning"
    >
      {{ "pamWithdrawApprovalWarningOne" | i18n }}
      <strong>{{ params.itemName }}</strong
      >{{ "pamWithdrawApprovalWarningTwo" | i18n }}
      <strong>{{ params.requesterName }}</strong>
      {{ "pamWithdrawApprovalWarningThree" | i18n }}
    </bit-callout>

    <bit-form-field disableMargin>
      <bit-label>{{ "pamWithdrawApprovalReasonLabel" | i18n }}</bit-label>
      <textarea
        bitInput
        rows="3"
        formControlName="reason"
        id="pam-withdraw-approval-dialog_textarea_reason"
        [placeholder]="'pamWithdrawApprovalReasonPlaceholder' | i18n"
      ></textarea>
      <bit-hint>{{ "pamWithdrawApprovalReasonHint" | i18n }}</bit-hint>
    </bit-form-field>
  </div>

  <ng-container bitDialogFooter>
    <button
      type="submit"
      bitButton
      bitFormButton
      buttonType="danger"
      [disabled]="confirmDisabled()"
      id="pam-withdraw-approval-dialog_button_confirm"
    >
      {{ "pamInboxWithdrawApproval" | i18n }}
    </button>
    <button
      type="button"
      bitButton
      buttonType="secondary"
      bitDialogClose
      id="pam-withdraw-approval-dialog_button_cancel"
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
  [bitSubmit]="confirm"
  bit-dialog
  background="alt"
  dialogSize="large"
>
  <span bitDialogTitle>{{ "pamInboxWithdrawApproval" | i18n }}</span>

  <div bitDialogContent>
    <pam-request-summary
      data-testid="withdraw-approval-dialog-item"
      hideDetails
      [cipher]="params.cipher ?? null"
      [itemName]="params.itemName"
      [organizationName]="params.organizationName"
      [collectionName]="params.collectionName"
      [requesterName]="params.requesterName"
    />

    <bit-callout
      type="warning"
      [title]="'warning' | i18n"
      data-testid="withdraw-approval-dialog-warning"
    >
      {{ "pamWithdrawApprovalWarningOne" | i18n }}
      <strong>{{ params.itemName }}</strong>{{
        "pamWithdrawApprovalWarningTwo" | i18n
      }}
      <strong>{{ params.requesterName }}</strong>
      {{ "pamWithdrawApprovalWarningThree" | i18n }}
    </bit-callout>

    <bit-form-field disableMargin>
      <bit-label>{{ "pamWithdrawApprovalReasonLabel" | i18n }}</bit-label>
      <textarea
        bitInput
        rows="3"
        formControlName="reason"
        id="pam-withdraw-approval-dialog_textarea_reason"
        [placeholder]="'pamWithdrawApprovalReasonPlaceholder' | i18n"
      ></textarea>
      <bit-hint>{{ "pamWithdrawApprovalReasonHint" | i18n }}</bit-hint>
    </bit-form-field>
  </div>

  <ng-container bitDialogFooter>
    <button
      type="submit"
      bitButton
      bitFormButton
      buttonType="danger"
      [disabled]="confirmDisabled()"
      id="pam-withdraw-approval-dialog_button_confirm"
    >
      {{ "pamInboxWithdrawApproval" | i18n }}
    </button>
    <button
      type="button"
      bitButton
      buttonType="secondary"
      bitDialogClose
      id="pam-withdraw-approval-dialog_button_cancel"
    >
      {{ "cancel" | i18n }}
    </button>
  </ng-container>
</form>

`````
