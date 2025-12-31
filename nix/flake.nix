{
  inputs = {
    flake-utils.url = "github:numtide/flake-utils";
    rust-overlay.url = "github:oxalica/rust-overlay";
  };

  outputs = {
    self,
    nixpkgs,
    flake-utils,
    rust-overlay,
  }:
    flake-utils.lib.eachDefaultSystem (
      system: let
        pkgs = nixpkgs.legacyPackages.${system}.extend (import rust-overlay);
      in {
        devShells.default = let
          libPath = with pkgs;
            lib.makeLibraryPath [
            ];
        in
          pkgs.mkShell {
            nativeBuildInputs = with pkgs; [
              pkg-config
            ];
            buildInputs = with pkgs; [
              systemd
              clang
            ];

            LD_LIBRARY_PATH = libPath;
            LIBCLANG_PATH = "${pkgs.libclang.lib}/lib";

            packages = with pkgs; [
              rust-analyzer
              rust-bin.stable."1.88.0".default

              gdb
              sqlite
              # TODO: qemu
              cdrkit # genisoimage
              libguestfs-with-appliance # guestfish
            ];
          };
      }
    );
}
