"use strict";
let value;
[value = eval] = [];
[value = () => eval] = [];
({ value = eval } = {});
({ [eval]: value } = {});
function readArguments() {
  [value = arguments] = [];
  ({ value = arguments } = {});
  ({ [arguments]: value } = {});
}
