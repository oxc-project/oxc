{ ... }:
{
  perSystem =
    { oxcPkgs, ... }:
    {
      formatter = oxcPkgs.nixfmt;
    };
}
