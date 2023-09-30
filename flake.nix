{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    systems.url = "github:nix-systems/default";
    devenv.url = "github:cachix/devenv";
  };

  nixConfig = {
    extra-trusted-public-keys = "devenv.cachix.org-1:w1cLUi8dv3hnoSPGAuibQv+f9TZLr6cv/Hm9XgU50cw=";
    extra-substituters = "https://devenv.cachix.org";
  };

  outputs = { self, nixpkgs, devenv, systems, ... } @ inputs:
    let
      forEachSystem = nixpkgs.lib.genAttrs (import systems);
    in
    {
      devShells = forEachSystem
        (system:
          let
            pkgs = nixpkgs.legacyPackages.${system};
          in
          {
            default = devenv.lib.mkShell {
              inherit inputs pkgs;
              modules = [
                {
                  # https://devenv.sh/reference/options/
                  packages = with pkgs; [ 
                    libxkbcommon
                    mesa
                    pango
                    cairo
                    udev
                    libinput
                    libGL
                    glib
                    # egl-wayland
                  ] ++ lib.optionals pkgs.stdenv.isDarwin (with pkgs.darwin.apple_sdk; [ frameworks.Security ]);
                  languages.rust = {
                    enable = true;
                    components = [ "rustc" "cargo" "clippy" "rustfmt" "rust-analyzer" ];
                    # channel = "nightly";
                  };

                  pre-commit.hooks = {
                    rustfmt.enable = true;
                    clippy.enable = true;
                  };

                  enterShell = ''
                    cargo --version
                  '';
                }
              ];
            };
          });
    };
}
