{
	description = "The Azure SQL Workload Identity Proxy (WIP) is a proxy analogous to the Google Cloud SQL Proxy";
	inputs.nixpkgs.url = "github:NixOS/nixpkgs";
	inputs.flake-utils.url = "github:numtide/flake-utils";
	inputs.rustOverlay.url = "github:oxalica/rust-overlay";
	inputs.crane = {
		url = "github:ipetkov/crane";
		inputs.nixpkgs.follows = "nixpkgs";
	};

	outputs = { self, nixpkgs, flake-utils, rustOverlay, crane }: 
		flake-utils.lib.eachDefaultSystem (system: let
			pkgs = import nixpkgs { 
				inherit system; 
				overlays = [ rustOverlay.overlays.default ];
			};
			rust = pkgs.rust-bin.stable."1.73.0".default;
			craneLib = (crane.mkLib pkgs).overrideToolchain rust;
			src = craneLib.cleanCargoSource (craneLib.path ./.);
			commonArgs = {
				inherit src;
				strictDeps = true;
			};
			cargoArtifacts = craneLib.buildDepsOnly commonArgs;
			wip = craneLib.buildPackage (commonArgs // {
				inherit cargoArtifacts;
			});
		in {
			checks = {
				inherit wip;
			};
			packages = rec {
				inherit wip;
				default = wip;
			};
			apps.default = flake-utils.lib.mkApp {
				drv = wip;
			};
			devShells.default = craneLib.devShell {
				checks = self.checks.${system};
			};
		});
}
