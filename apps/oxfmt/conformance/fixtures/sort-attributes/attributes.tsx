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
