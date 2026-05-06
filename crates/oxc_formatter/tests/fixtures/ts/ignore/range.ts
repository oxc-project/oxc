type Before={  formatted:true};

/* prettier-ignore-start */
type B1 = string;
type B2 = string
/* prettier-ignore-end */

type Between={  formatted:true};

// oxfmt-ignore-start
interface Kept {
  value   :string
}
type Values=[  string,number]
// oxfmt-ignore-end

type After={  formatted:true};
