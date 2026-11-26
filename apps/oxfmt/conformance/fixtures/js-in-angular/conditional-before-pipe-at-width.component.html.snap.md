# Input

`````html
<p>{{ (hasBillingSyncTokenXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXX() ? "viewBillingToken" : "setUpBillingSync") | i18n }}</p>

`````

# Output

## `{}`

`````html
<p>
  {{
    (hasBillingSyncTokenXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXX()
      ? "viewBillingToken"
      : "setUpBillingSync"
    ) | i18n
  }}
</p>

`````
