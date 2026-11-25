const a = cn(x.includes(`a  b`) && "p-4  flex");
const b = cn(x.includes(`${"a  b"}  c`));
const C = <div className={x.split(`a  b`).length ? "p-4  flex" : ""} />;
const D = <div className={foo(`p-4
  flex`)} />;
