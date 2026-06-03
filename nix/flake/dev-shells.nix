{ ... }:
{
  perSystem =
    {
      oxcPkgs,
      oxcRustToolchain,
      ...
    }:
    {
      devShells.default = oxcPkgs.mkShell {
        packages = [
          oxcRustToolchain
          oxcPkgs.just
          oxcPkgs.nodejs
          oxcPkgs.pnpm
          oxcPkgs.cmake
        ];
      };
    };
}
