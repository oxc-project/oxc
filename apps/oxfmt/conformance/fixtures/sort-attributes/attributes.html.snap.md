# Input

`````html
<div z="last" data-z="z" id="example" aria-label="Label" class="a  b" data-a="a" a="first" disabled></div>
<svg><use xlink:href="#example" id="icon" aria-hidden="true" /></svg>
<div Z="uppercase" ID="example" CLASS="classes" a="first"></div>
<div z="last" ID="first" id="second" a="first"></div>
<!-- prettier-ignore -->
<div z="last"   a="first"></div>
<!-- prettier-ignore-attribute -->
<div z="last"   a='first'></div>
<!-- oxfmt-ignore-attribute z -->
<div z = 'last' a="first"></div>

`````

# Output

## `{"sortAttributes":true}`

`````html
<div
  a="first"
  aria-label="Label"
  class="a b"
  data-a="a"
  data-z="z"
  disabled
  id="example"
  z="last"
></div>
<svg><use aria-hidden="true" id="icon" xlink:href="#example" /></svg>
<div a="first" class="classes" id="example" Z="uppercase"></div>
<div z="last" id="first" id="second" a="first"></div>
<!-- prettier-ignore -->
<div z="last"   a="first"></div>
<!-- prettier-ignore-attribute -->
<div z="last" a='first'></div>
<!-- oxfmt-ignore-attribute z -->
<div z = 'last' a="first"></div>

`````

## `{"sortAttributes":{"order":"desc"}}`

`````html
<div
  z="last"
  id="example"
  disabled
  data-z="z"
  data-a="a"
  class="a b"
  aria-label="Label"
  a="first"
></div>
<svg><use xlink:href="#example" id="icon" aria-hidden="true" /></svg>
<div Z="uppercase" id="example" class="classes" a="first"></div>
<div z="last" id="first" id="second" a="first"></div>
<!-- prettier-ignore -->
<div z="last"   a="first"></div>
<!-- prettier-ignore-attribute -->
<div z="last" a='first'></div>
<!-- oxfmt-ignore-attribute z -->
<div z = 'last' a="first"></div>

`````

## `{"sortAttributes":{"groups":["^id$","^class(?:Name)?$","$DEFAULT","^data-","^aria-"],"order":"asc"}}`

`````html
<div
  id="example"
  class="a b"
  a="first"
  disabled
  z="last"
  data-a="a"
  data-z="z"
  aria-label="Label"
></div>
<svg><use id="icon" xlink:href="#example" aria-hidden="true" /></svg>
<div id="example" class="classes" a="first" Z="uppercase"></div>
<div z="last" id="first" id="second" a="first"></div>
<!-- prettier-ignore -->
<div z="last"   a="first"></div>
<!-- prettier-ignore-attribute -->
<div z="last" a='first'></div>
<!-- oxfmt-ignore-attribute z -->
<div z = 'last' a="first"></div>

`````

## `{"sortAttributes":{"groups":["^data-","^id$"]}}`

`````html
<div
  data-a="a"
  data-z="z"
  id="example"
  a="first"
  aria-label="Label"
  class="a b"
  disabled
  z="last"
></div>
<svg><use id="icon" aria-hidden="true" xlink:href="#example" /></svg>
<div id="example" a="first" class="classes" Z="uppercase"></div>
<div z="last" id="first" id="second" a="first"></div>
<!-- prettier-ignore -->
<div z="last"   a="first"></div>
<!-- prettier-ignore-attribute -->
<div z="last" a='first'></div>
<!-- oxfmt-ignore-attribute z -->
<div z = 'last' a="first"></div>

`````

## `{"sortAttributes":true,"singleAttributePerLine":true,"bracketSameLine":true}`

`````html
<div
  a="first"
  aria-label="Label"
  class="a b"
  data-a="a"
  data-z="z"
  disabled
  id="example"
  z="last"></div>
<svg>
  <use
    aria-hidden="true"
    id="icon"
    xlink:href="#example" />
</svg>
<div
  a="first"
  class="classes"
  id="example"
  Z="uppercase"></div>
<div
  z="last"
  id="first"
  id="second"
  a="first"></div>
<!-- prettier-ignore -->
<div z="last"   a="first"></div>
<!-- prettier-ignore-attribute -->
<div
  z="last"
  a='first'></div>
<!-- oxfmt-ignore-attribute z -->
<div
  z = 'last'
  a="first"></div>

`````
