{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";

    flake-utils.url = "github:numtide/flake-utils";
    flake-utils.inputs.nixpkgs.follows = "nixpkgs";

    rust-overlay.url = "github:oxalica/rust-overlay";
    rust-overlay.inputs.nixpkgs.follows = "nixpkgs";
  };

  outputs =
    {
      nixpkgs,
      flake-utils,
      rust-overlay,
      ...
    }:
    flake-utils.lib.eachDefaultSystemPassThrough (
      system:
      let
        overlays = [ rust-overlay.overlays.default ];
        pkgs = import nixpkgs {
          inherit system overlays;
        };

        lib = pkgs.lib;

        rustToolchain = pkgs.rust-bin.stable.latest.default.override {
          extensions = [ "rust-src" ];
        };

        nativeBuildInputs =
          [ rustToolchain ]
          ++ lib.optionals pkgs.stdenv.isLinux [
            pkgs.mold-wrapped
          ];

        buildInputs = [
          pkgs.xdg-desktop-portal-wlr
        ];

        nativeCheckInputs = [ pkgs.cargo-nextest ];
      in
      {
        formatter.${system} = pkgs.nixfmt-tree;

        devShells.${system}.default =
          let
            rustLinkerFlags =
              if pkgs.stdenv.isLinux then
                [
                  "-fuse-ld=mold"
                  "-Wl,--compress-debug-sections=zstd"
                ]
              else
                [ ];
          in
          pkgs.mkShell {
            packages =
              [
                pkgs.bacon
              ]
              ++ nativeBuildInputs
              ++ buildInputs
              ++ nativeCheckInputs;

            env = {
              RUSTFLAGS = lib.concatStringsSep " " (
                lib.concatMap (flag: [
                  "-C"
                  "link-arg=${flag}"
                ]) rustLinkerFlags
              );

              RUST_SRC_PATH = "${rustToolchain}/lib/rustlib/src/rust/library";
            };
          };
      }
    );
}
