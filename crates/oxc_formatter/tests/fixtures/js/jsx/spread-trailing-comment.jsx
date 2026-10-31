// A comment before `}` stays inside the braces of a spread child or attribute.
<div>
  {
    ...a
    //comment
  }
</div>;

<div>
  {
    ...a
    /* comment */
  }
</div>;

<div
  {
    ...a
    //comment
  }
/>;
