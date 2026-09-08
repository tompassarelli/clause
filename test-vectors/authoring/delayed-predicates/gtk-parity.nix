{ generated, nixpkgs }:
let
  pkgs = import nixpkgs { system = "x86_64-linux"; };
  lib = pkgs.lib;
  inspect = module: username: enabled: polarity: inner:
    let
      evaluated = lib.evalModules {
        specialArgs = { inherit pkgs; };
        modules = [
          {
            options.home-manager.users = lib.mkOption { type = lib.types.attrsOf lib.types.anything; default = {}; };
            options.myConfig.modules.users.username = lib.mkOption { type = lib.types.str; };
            options.stylix.polarity = lib.mkOption { type = lib.types.str; };
            config.myConfig.modules.users.username = username;
            config.stylix.polarity = polarity;
          }
          module
        ] ++ lib.optional (enabled != null) { myConfig.modules.gtk.enable = enabled; };
      };
      option = evaluated.options.myConfig.modules.gtk.enable;
    in {
      enabled = evaluated.config.myConfig.modules.gtk.enable;
      type = option.type.name; default = option.default; description = option.description;
      homes = lib.mapAttrs (_: home:
        let value = home { config.gtk = inner; pkgs = throw "inner packages must stay unused"; };
        in value // { home = value.home // { packages = map (p: { name = p.name; path = p.outPath; }) value.home.packages; }; }
      ) evaluated.config.home-manager.users;
    };
  actual = import generated;
  expected = import ./gtk-baseline.nix;
in map (username: map (enabled: map (polarity: map (inner:
  let a = inspect actual username enabled polarity inner; e = inspect expected username enabled polarity inner;
  in assert a == e; a
) [ { font = { name = "First Font"; size = 11; }; theme.name = "First Theme"; }
     { font = { name = "Other Font"; size = 16; }; theme.name = "Other Theme"; } ]) [ "dark" "light" ]) [ null false true ]) [ "tom" "gtk-test.user" ]
