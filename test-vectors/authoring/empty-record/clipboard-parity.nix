{ generated, nixpkgs }:
let
  pkgs = import nixpkgs { system = "x86_64-linux"; };
  lib = pkgs.lib;
  inspect = module: username: enabled: innerConfig:
    let
      evaluated = lib.evalModules {
        specialArgs = { inherit pkgs; };
        modules = [
          {
            options.environment.systemPackages = lib.mkOption { type = lib.types.listOf lib.types.package; default = []; };
            options.home-manager.users = lib.mkOption { type = lib.types.attrsOf lib.types.anything; default = {}; };
            options.myConfig.modules.users.username = lib.mkOption { type = lib.types.str; };
            config.myConfig.modules.users.username = username;
          }
          module
        ] ++ lib.optional (enabled != null) { myConfig.modules.clipboard-tools.enable = enabled; };
      };
      option = evaluated.options.myConfig.modules.clipboard-tools.enable;
    in {
      enabled = evaluated.config.myConfig.modules.clipboard-tools.enable;
      type = option.type.name; default = option.default; description = option.description;
      packages = map (p: { name = p.name; path = p.outPath; }) evaluated.config.environment.systemPackages;
      homes = lib.mapAttrs (_: home: home {
        config = innerConfig;
        pkgs = throw "inner package receiver must remain unused";
      }) evaluated.config.home-manager.users;
    };
  actual = import generated;
  expected = import ./clipboard-baseline.nix;
in map (username: map (enabled: map (innerConfig:
  let a = inspect actual username enabled innerConfig; e = inspect expected username enabled innerConfig;
  in assert a == e; a
) [ {} { unrelated = "inner configuration"; home.homeDirectory = "/invented/inner-home"; } ]) [ null false true ]) [ "tom" "clipboard-test.user" ]
