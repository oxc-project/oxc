# Input

`````tsx
const element = <Widget z="last" data-z="z" id="example" aria-label="Label" className="classes" data-a="a" a="first" disabled />;
const spread = <Widget z="before" a="before" {...props} z="after" a="after" {...more} y="end" b="end" />;
const effects = <Widget z={z} a={a} effect={run()} y="after" b="after" member={object.value} x="last" c="last" />;
const handlers = <Widget z={true} onClick={() => run()} id={id} a={null} />;
const duplicate = <Widget z="last" a="first" a="second" />;
const namespace = <svg z="last" xml : lang="en" a="first" xlink:href="#icon" />;
const comments = <Widget z="last" /* keep before a */ a="first" />;
const inside = <Widget z="last" fn={() => { /* keep */ run(); }} a="first" />;
// prettier-ignore
const ignored = <Widget z="last"   a="first" />;
const identifiers = <Widget z={z} a={a} />;
const reverseIdentifiers = <Widget a={a} z={z} />;
const identifierBoundary = <Widget z="before" a="before" getter={value} y="after" b="after" />;

`````

# Output

## `{"sortAttributes":true}`

`````tsx
const element = (
  <Widget
    a="first"
    aria-label="Label"
    className="classes"
    data-a="a"
    data-z="z"
    disabled
    id="example"
    z="last"
  />
);
const spread = (
  <Widget a="before" z="before" {...props} a="after" z="after" {...more} b="end" y="end" />
);
const effects = (
  <Widget z={z} a={a} effect={run()} b="after" y="after" member={object.value} c="last" x="last" />
);
const handlers = <Widget onClick={() => run()} z={true} id={id} a={null} />;
const duplicate = <Widget a="first" a="second" z="last" />;
const namespace = <svg a="first" xlink:href="#icon" xml:lang="en" z="last" />;
const comments = <Widget z="last" /* keep before a */ a="first" />;
const inside = (
  <Widget
    z="last"
    fn={() => {
      /* keep */ run();
    }}
    a="first"
  />
);
// prettier-ignore
const ignored = <Widget z="last"   a="first" />;
const identifiers = <Widget z={z} a={a} />;
const reverseIdentifiers = <Widget a={a} z={z} />;
const identifierBoundary = <Widget a="before" z="before" getter={value} b="after" y="after" />;

`````

## `{"sortAttributes":{"order":"desc"}}`

`````tsx
const element = (
  <Widget
    z="last"
    id="example"
    disabled
    data-z="z"
    data-a="a"
    className="classes"
    aria-label="Label"
    a="first"
  />
);
const spread = (
  <Widget z="before" a="before" {...props} z="after" a="after" {...more} y="end" b="end" />
);
const effects = (
  <Widget z={z} a={a} effect={run()} y="after" b="after" member={object.value} x="last" c="last" />
);
const handlers = <Widget z={true} onClick={() => run()} id={id} a={null} />;
const duplicate = <Widget z="last" a="first" a="second" />;
const namespace = <svg z="last" xml:lang="en" xlink:href="#icon" a="first" />;
const comments = <Widget z="last" /* keep before a */ a="first" />;
const inside = (
  <Widget
    z="last"
    fn={() => {
      /* keep */ run();
    }}
    a="first"
  />
);
// prettier-ignore
const ignored = <Widget z="last"   a="first" />;
const identifiers = <Widget z={z} a={a} />;
const reverseIdentifiers = <Widget a={a} z={z} />;
const identifierBoundary = <Widget z="before" a="before" getter={value} y="after" b="after" />;

`````

## `{"sortAttributes":{"groups":["^id$","^class(?:Name)?$","$DEFAULT","^data-","^aria-"],"order":"asc"}}`

`````tsx
const element = (
  <Widget
    id="example"
    className="classes"
    a="first"
    disabled
    z="last"
    data-a="a"
    data-z="z"
    aria-label="Label"
  />
);
const spread = (
  <Widget a="before" z="before" {...props} a="after" z="after" {...more} b="end" y="end" />
);
const effects = (
  <Widget z={z} a={a} effect={run()} b="after" y="after" member={object.value} c="last" x="last" />
);
const handlers = <Widget onClick={() => run()} z={true} id={id} a={null} />;
const duplicate = <Widget a="first" a="second" z="last" />;
const namespace = <svg a="first" xlink:href="#icon" xml:lang="en" z="last" />;
const comments = <Widget z="last" /* keep before a */ a="first" />;
const inside = (
  <Widget
    z="last"
    fn={() => {
      /* keep */ run();
    }}
    a="first"
  />
);
// prettier-ignore
const ignored = <Widget z="last"   a="first" />;
const identifiers = <Widget z={z} a={a} />;
const reverseIdentifiers = <Widget a={a} z={z} />;
const identifierBoundary = <Widget a="before" z="before" getter={value} b="after" y="after" />;

`````

## `{"sortAttributes":{"groups":["^data-","^id$"]}}`

`````tsx
const element = (
  <Widget
    data-a="a"
    data-z="z"
    id="example"
    a="first"
    aria-label="Label"
    className="classes"
    disabled
    z="last"
  />
);
const spread = (
  <Widget a="before" z="before" {...props} a="after" z="after" {...more} b="end" y="end" />
);
const effects = (
  <Widget z={z} a={a} effect={run()} b="after" y="after" member={object.value} c="last" x="last" />
);
const handlers = <Widget onClick={() => run()} z={true} id={id} a={null} />;
const duplicate = <Widget a="first" a="second" z="last" />;
const namespace = <svg a="first" xlink:href="#icon" xml:lang="en" z="last" />;
const comments = <Widget z="last" /* keep before a */ a="first" />;
const inside = (
  <Widget
    z="last"
    fn={() => {
      /* keep */ run();
    }}
    a="first"
  />
);
// prettier-ignore
const ignored = <Widget z="last"   a="first" />;
const identifiers = <Widget z={z} a={a} />;
const reverseIdentifiers = <Widget a={a} z={z} />;
const identifierBoundary = <Widget a="before" z="before" getter={value} b="after" y="after" />;

`````

## `{"sortAttributes":true,"singleAttributePerLine":true,"bracketSameLine":true}`

`````tsx
const element = (
  <Widget
    a="first"
    aria-label="Label"
    className="classes"
    data-a="a"
    data-z="z"
    disabled
    id="example"
    z="last"
  />
);
const spread = (
  <Widget
    a="before"
    z="before"
    {...props}
    a="after"
    z="after"
    {...more}
    b="end"
    y="end"
  />
);
const effects = (
  <Widget
    z={z}
    a={a}
    effect={run()}
    b="after"
    y="after"
    member={object.value}
    c="last"
    x="last"
  />
);
const handlers = (
  <Widget
    onClick={() => run()}
    z={true}
    id={id}
    a={null}
  />
);
const duplicate = (
  <Widget
    a="first"
    a="second"
    z="last"
  />
);
const namespace = (
  <svg
    a="first"
    xlink:href="#icon"
    xml:lang="en"
    z="last"
  />
);
const comments = (
  <Widget
    z="last"
    /* keep before a */ a="first"
  />
);
const inside = (
  <Widget
    z="last"
    fn={() => {
      /* keep */ run();
    }}
    a="first"
  />
);
// prettier-ignore
const ignored = <Widget z="last"   a="first" />;
const identifiers = (
  <Widget
    z={z}
    a={a}
  />
);
const reverseIdentifiers = (
  <Widget
    a={a}
    z={z}
  />
);
const identifierBoundary = (
  <Widget
    a="before"
    z="before"
    getter={value}
    b="after"
    y="after"
  />
);

`````
