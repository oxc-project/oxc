// Valid hex
<div>
  &#xC;
  &#x41;
  &#x123;
  &#x1234;
  &#x10000;
  &#x10FFFF;
</div>;

// Invalid hex
<div>
  &#x110000;
  &#xFFFFFF;
  &#xG;
</div>;

// Valid decimal (same characters as valid hex above)
<div>
  &#12;
  &#65;
  &#291;
  &#4660;
  &#65536;
  &#1114111;
</div>;

// Invalid decimal
<div>
  &#1114112;
  &#16777215;
  &#C;
</div>;
