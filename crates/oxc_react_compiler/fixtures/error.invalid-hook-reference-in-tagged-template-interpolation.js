import {useState} from 'react';

function tag(strings, value) {
  return value;
}

function Component() {
  return tag`${useState}`;
}
