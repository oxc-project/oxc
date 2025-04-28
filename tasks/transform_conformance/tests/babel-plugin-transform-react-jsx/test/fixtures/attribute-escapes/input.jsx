// Valid escapes
<Foo bar="&Egrave; &euro; &quot;" />;
<Foo bar="&#xC; &#x41;" />;
<Foo bar="&#12; &#65;" />;

// Invalid escapes
<Foo bar="&donkey; &#x110000; &#xFFFFFF; &#xG; &#1114112; &#16777215; &#C;" />;

// Unterminated escapes
<Foo bar="&euro xxx" />;
<Foo bar="&#123 xxx" />;
<Foo bar="&#x123 xxx" />;
