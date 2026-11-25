# Input

`````html
<p>{{ firstCondition && secondCondition && thirdCondition && fourthCondition && fifthConditionThatIsLong }}</p>
<button (click)="firstCondition && secondCondition && thirdCondition && fourthCondition && fifthConditionThatIsLong"></button>
<p [title]="firstCondition && secondCondition && thirdCondition && fourthCondition && fifthConditionThatIsLong"></p>

`````

# Output

## `{}`

`````html
<p>
  {{
    firstCondition &&
    secondCondition &&
    thirdCondition &&
    fourthCondition &&
    fifthConditionThatIsLong
  }}
</p>
<button
  (click)="
    firstCondition &&
    secondCondition &&
    thirdCondition &&
    fourthCondition &&
    fifthConditionThatIsLong
  "
></button>
<p
  [title]="
    firstCondition &&
    secondCondition &&
    thirdCondition &&
    fourthCondition &&
    fifthConditionThatIsLong
  "
></p>

`````
