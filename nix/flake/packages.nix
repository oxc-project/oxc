{ ... }:
{
  perSystem =
    {
      config,
      oxcPkgs,
      oxcRustPlatform,
      ...
    }:
    let
      inherit (oxcPkgs) lib;

      mkFeatureFlags =
        {
          noDefaultFeatures ? false,
          features ? [ ],
        }:
        lib.optionals noDefaultFeatures [ "--no-default-features" ]
        ++ lib.optionals (features != [ ]) [
          "--features"
          (lib.concatStringsSep "," features)
        ];

      workspaceManifest = lib.importTOML ../../Cargo.toml;
      workspacePackage = workspaceManifest.workspace.package;

      mkOxcPackage =
        {
          cargoPackage,
          manifest,
          binary,
          noDefaultFeatures ? false,
          features ? [ ],
        }:
        let
          featureFlags = mkFeatureFlags { inherit noDefaultFeatures features; };
        in
        oxcRustPlatform.buildRustPackage {
          pname = binary;
          version = manifest.package.version;

          src = ../../.;
          cargoLock.lockFile = ../../Cargo.lock;

          cargoBuildFlags = [
            "--package"
            cargoPackage
          ]
          ++ featureFlags;

          doCheck = false;

          nativeBuildInputs = [ oxcPkgs.cmake ];

          buildInputs = lib.optionals oxcPkgs.stdenv.hostPlatform.isDarwin [
            oxcPkgs.apple-sdk_15
            oxcPkgs.libiconv
          ];

          meta = {
            inherit (workspacePackage) description homepage;
            license = lib.licenses.mit;
            mainProgram = binary;
          };
        };

      mkProgramAlias =
        {
          package,
          alias,
          binary,
        }:
        oxcPkgs.symlinkJoin {
          name = "${alias}-${package.version}";
          paths = [ package ];
          postBuild = ''
            ln -s "$out/bin/${binary}" "$out/bin/${alias}"
          '';
          meta = package.meta // {
            mainProgram = alias;
          };
        };

      mkApp = package: binary: {
        type = "app";
        program = "${package}/bin/${binary}";
      };

      oxfmtManifest = lib.importTOML ../../apps/oxfmt/Cargo.toml;
      oxlintManifest = lib.importTOML ../../apps/oxlint/Cargo.toml;
    in
    {
      packages = rec {
        oxfmt = mkOxcPackage {
          cargoPackage = "oxfmt";
          manifest = oxfmtManifest;
          binary = "oxfmt";
          noDefaultFeatures = true;
          features = [ "allocator" ];
        };

        oxlint = mkOxcPackage {
          cargoPackage = "oxlint";
          manifest = oxlintManifest;
          binary = "oxlint";
          features = [ "allocator" ];
        };

        oxcfmt = mkProgramAlias {
          package = oxfmt;
          alias = "oxcfmt";
          binary = "oxfmt";
        };

        oxclint = mkProgramAlias {
          package = oxlint;
          alias = "oxclint";
          binary = "oxlint";
        };

        default = oxlint;
      };

      apps = {
        oxfmt = mkApp config.packages.oxfmt "oxfmt";
        oxlint = mkApp config.packages.oxlint "oxlint";
        oxcfmt = mkApp config.packages.oxcfmt "oxcfmt";
        oxclint = mkApp config.packages.oxclint "oxclint";
        default = mkApp config.packages.default "oxlint";
      };

      checks = {
        inherit (config.packages) oxfmt oxlint;
      };
    };
}
