let
  lockFile = builtins.fromJSON (builtins.readFile ./flake.lock);
  flakeCompatNode = lockFile.nodes.${lockFile.nodes.root.inputs.flake-compat};
  flakeCompatLocked = flakeCompatNode.locked;
  flakeCompatUrl =
    flakeCompatLocked.url
      or "https://github.com/${flakeCompatLocked.owner}/${flakeCompatLocked.repo}/archive/${flakeCompatLocked.rev}.tar.gz";
  flakeCompat = builtins.fetchTarball {
    url = flakeCompatUrl;
    sha256 = flakeCompatLocked.narHash;
  };

  flake = import flakeCompat { src = ./.; };
in
flake.shellNix
