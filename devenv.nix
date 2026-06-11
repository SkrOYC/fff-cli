{ pkgs, lib, ... }:

{
  languages.rust = {
    enable = true;
    channel = "stable";
    components = [ "rustc" "cargo" "clippy" "rustfmt" "rust-analyzer" ];
  };

  packages = [
    pkgs.cargo-nextest
    pkgs.cargo-watch
    pkgs.hyperfine
    pkgs.ripgrep
    pkgs.fd
    pkgs.findutils
    pkgs.git
  ];

  scripts = {
    build.exec = "cargo build --workspace";
    test.exec = "cargo nextest run --workspace";
    lint.exec = "cargo clippy --workspace --all-targets -- -D warnings";
    fmt.exec = "cargo fmt --all";
    fmt-check.exec = "cargo fmt --all --check";
    bench.exec = "cargo bench --workspace";
    check-all.exec = "cargo fmt --all --check && cargo clippy --workspace --all-targets -- -D warnings && cargo nextest run --workspace";
  };

  enterShell = ''
    echo "ff development environment"
    echo "  rustc: $(rustc --version)"
    echo "  cargo: $(cargo --version)"
    echo ""
    echo "Available scripts: build, test, lint, fmt, fmt-check, bench, check-all"
  '';

  enterTest = ''
    echo "Running ff test suite"
    cargo fmt --all --check
    cargo clippy --workspace --all-targets -- -D warnings
    cargo nextest run --workspace
  '';
}
