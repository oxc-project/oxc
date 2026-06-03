{ inputs, ... }:
{
  perSystem =
    { system, ... }:
    let
      pkgs = import inputs.nixpkgs {
        inherit system;
        overlays = [ inputs.rust-overlay.overlays.default ];
      };

      rustToolchain = pkgs.rust-bin.fromRustupToolchainFile ../../rust-toolchain.toml;
    in
    {
      _module.args.oxcPkgs = pkgs;
      _module.args.oxcRustToolchain = rustToolchain;
      _module.args.oxcRustPlatform = pkgs.makeRustPlatform {
        cargo = rustToolchain;
        rustc = rustToolchain;
      };
    };
}
