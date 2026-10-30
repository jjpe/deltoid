{ pkgs }:
let
  tag = "[shell.nix]";
in
pkgs.mkShell {
  buildInputs = with pkgs; [
    # cargo-audit
    # cargo-bloat
    cargo-crev  # Review system for verifying security & quality of Cargo deps
    cargo-criterion  # Benchmarker
    cargo-edit
    cargo-expand  # Rust macro expansion utility
    # cargo-make  # build tool on top of cargo
    cargo-msrv  # Find the Minimum Supported Rust Version for a crate
    # cargo-ndk  # Android build support for Rust
    # cargo-nextest  # A new, faster test runner for Rust.
    cargo-outdated  # Check for outdated dependencies
    # cargo-watch  # Execute commands when Rust project files change
    # cargo-workspaces  # Optimizes the workflow around cargo workspaces
    # clang
    difftastic
    # jetbrains.rust-rover  # Quite large, but useful for its debugger GUI
    # openssl
    # pkg-config
    tokei
    (rust-bin.stable.latest.default.override {
      extensions = [
        "rustfmt"
        "rust-src"
        "rust-analyzer"
      ];
      targets = [
        "aarch64-apple-darwin"
        "aarch64-unknown-linux-gnu"
        "x86_64-unknown-linux-gnu"
        "wasm32-unknown-unknown"
      ];
    })
    # trunk  # Build, bundle & ship your Rust WASM app to the web
    # wasm-pack  # A tool to make WASM & Rust play nicely
    # zstd
  ];

  shellHook = ''
      export RUST_BACKTRACE=1

      # >&2 echo "${tag} Executing 'cargo clean'..."
      # cargo clean
  '';
}
