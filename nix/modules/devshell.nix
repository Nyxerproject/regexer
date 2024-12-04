{inputs, ...}: {
  perSystem = {
    config,
    self',
    pkgs,
    lib,
    ...
  }: {
    devShells.default = pkgs.mkShell {
      name = "regexer-shell";
      inputsFrom = [
        self'.devShells.rust
        config.treefmt.build.devShell
      ];
      packages = with pkgs; [
        just
        devenv
        nixd # Nix language server
        cargo-watch
        config.process-compose.cargo-doc-live.outputs.package
      ];
    };
  };
}
