{
  description = "Development environment for command-line-rust";

  inputs = {
    flake-parts.url = "github:hercules-ci/flake-parts";
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-25.11";
  };

  outputs = inputs@{ flake-parts, ... }:
    flake-parts.lib.mkFlake { inherit inputs; } {
      systems = [ "x86_64-linux" ];
      perSystem = { config, self', inputs', pkgs, system, ... }: {
        devShells.default = pkgs.mkShell {
          name = "command-line-rust-dev-shell";
          meta.description = "Command Line Rust development environment";
          packages = with pkgs; [
            cargo
            rustc
            rust-analyzer
            clippy
            rustfmt

            # rustic - rust docs in org-mode
            rustup
            pandoc
            (writeShellScriptBin "cargo-makedocs" ''
              exec cargo makedocs "$@"
            '')
            fd
          ];
        };
      };
    };
}
