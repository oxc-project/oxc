# Input

`````html
<p [title]="firstCondition && secondCondition && thirdCondition && fourthCondition && fifthConditionThatIsLong | json"></p>
<p [title]="firstCondition && secondCondition && thirdCondition && fourthCondition && fifthConditionThatIsLong"></p>

`````

# Output

## `{}`

`````html
<p
  [title]="
    firstCondition &&
    secondCondition &&
    thirdCondition &&
    fourthCondition &&
    fifthConditionThatIsLong | json
  "
></p>
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
