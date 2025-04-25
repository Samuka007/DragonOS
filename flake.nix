{
  description = "DragonOS";

  inputs = {
    nixpkgs.url      = "github:NixOS/nixpkgs/nixos-unstable";
    rust-overlay.url = "github:oxalica/rust-overlay";
    flake-utils.url  = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, rust-overlay, flake-utils, ... }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        overlays = [ (import rust-overlay) ];
        pkgs = import nixpkgs {
          inherit system overlays;
        };
        lib = pkgs.lib;
        riscv64Toolchain = import nixpkgs {
          crossSystem = lib.systems.riscv64;
        };
        riscv64EmbeddedToolchain = import nixpkgs {
          crossSystem = lib.systems.riscv64-embedded;
        };
      in
      {
        devShells.default = with pkgs; mkShell {
          buildInputs = [
            openssl
            pkg-config
            eza
            fd
            curl
            wget
            cacert

            iptables
            bridge-utils
            dnsmasq

            util-linux
            multipath-tools
            dosfstools

            unzip
            gnupg
            gcc_multi
            gcc12

            qemu
            qemu_kvm
            (rust-bin.fromRustupToolchainFile ./kernel/rust-toolchain.toml)
          ];

          shellHook = ''
            alias ls=eza
            alias find=fd
          '';
        };
      }
    );
}
