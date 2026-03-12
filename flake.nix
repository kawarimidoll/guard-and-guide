{
  description = "CLI guard for AI coding agent hooks";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    git-hooks = {
      url = "github:cachix/git-hooks.nix";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs =
    {
      self,
      nixpkgs,
      git-hooks,
    }:
    let
      supportedSystems = [
        "x86_64-linux"
        "aarch64-linux"
        "x86_64-darwin"
        "aarch64-darwin"
      ];
      forAllSystems = nixpkgs.lib.genAttrs supportedSystems;
    in
    {
      packages = forAllSystems (
        system:
        let
          pkgs = nixpkgs.legacyPackages.${system};
          cargoToml = builtins.fromTOML (builtins.readFile ./Cargo.toml);
        in
        {
          default = pkgs.rustPlatform.buildRustPackage {
            pname = cargoToml.package.name;
            version = cargoToml.package.version;
            src = ./.;
            cargoLock = {
              lockFile = ./Cargo.lock;
            };

            meta = with pkgs.lib; {
              description = "CLI guard for AI coding agent hooks";
              homepage = "https://github.com/kawarimidoll/guard-and-guide";
              license = licenses.mit;
              maintainers = [ ];
              mainProgram = "guard-and-guide";
            };
          };
        }
      );

      checks = forAllSystems (
        system:
        let
          pkgs = nixpkgs.legacyPackages.${system};
        in
        {
          pre-commit-check = git-hooks.lib.${system}.run {
            src = ./.;
            hooks = {
              # Rust
              rustfmt.enable = true;
              clippy = {
                enable = true;
                stages = [ "pre-push" ];
                settings.denyWarnings = true;
              };

              # Nix
              nixfmt.enable = true;

              # Conventional Commits
              convco = {
                enable = true;
                entry =
                  let
                    script = pkgs.writeShellScript "convco-check" ''
                      msg=$(head -1 "$1")
                      re='^(fixup|squash|amend)! |^Revert "'
                      if [[ "$msg" =~ $re ]]; then
                        exit 0
                      fi
                      ${pkgs.lib.getExe pkgs.convco} check --from-stdin < "$1"
                    '';
                  in
                  builtins.toString script;
              };

              # Markdown / YAML
              dprint = {
                enable = true;
                name = "dprint";
                entry = "${pkgs.dprint}/bin/dprint fmt --diff";
                types = [
                  "markdown"
                  "yaml"
                ];
                pass_filenames = false;
              };

              # YAML (GitHub Actions)
              actionlint.enable = true;
              zizmor.enable = true;

              # Spell check
              typos.enable = true;

              # Security
              check-merge-conflicts.enable = true;
              detect-private-keys.enable = true;

              # File hygiene
              check-case-conflicts.enable = true;
              end-of-file-fixer.enable = true;
              trim-trailing-whitespace.enable = true;
            };
          };
        }
      );

      devShells = forAllSystems (
        system:
        let
          pkgs = nixpkgs.legacyPackages.${system};
        in
        {
          default = pkgs.mkShell {
            inherit (self.checks.${system}.pre-commit-check) shellHook;
            buildInputs = with pkgs; [
              cargo
              rustc
              rust-analyzer
              clippy
              rustfmt
              just
              nixfmt
              dprint
              actionlint
              zizmor
              convco
              typos
            ];
          };
        }
      );
    };
}
