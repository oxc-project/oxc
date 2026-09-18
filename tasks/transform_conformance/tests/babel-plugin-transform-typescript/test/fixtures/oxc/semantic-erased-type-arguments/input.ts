const value = 1;
type Hidden = NS.Generic<{
  [(() => { const local = value; return local; })()]: typeof value;
  method(param: NS.Other): void;
}>;
interface HiddenInterface extends NS.Base<{ [key]: typeof missing }> {}
declare class Ambient extends Base<NS.Other> { [field]: NS.Other; }
use(value);
