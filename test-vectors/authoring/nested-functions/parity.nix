{ generated, nixpkgs }:
let
  pkgs = import nixpkgs { system = "x86_64-linux"; };
  lib = pkgs.lib;
  inspect = name: module: username: homeDirectory: enabled:
    let
      evaluated = lib.evalModules {
        specialArgs = { inherit pkgs; };
        modules = [
          {
            options.environment.systemPackages = lib.mkOption {
              type = lib.types.listOf lib.types.package;
              default = [];
            };
            options.home.homeDirectory = lib.mkOption {
              type = lib.types.str;
              default = "/outer/configuration/must-not-be-used";
            };
            options.lib = lib.mkOption {
              type = lib.types.raw;
              default.file.mkOutOfStoreSymlink = _: throw "outer symlink receiver used";
            };
            options.home-manager.users = lib.mkOption {
              type = lib.types.attrsOf (lib.types.submodule {
                options.home.homeDirectory = lib.mkOption {
                  type = lib.types.str;
                  default = homeDirectory;
                };
                options.lib = lib.mkOption {
                  type = lib.types.raw;
                  default.file.mkOutOfStoreSymlink = path: "captured-inner:" + path;
                };
                options.xdg.configFile = lib.mkOption {
                  type = lib.types.attrsOf (lib.types.submodule {
                    options.source = lib.mkOption { type = lib.types.str; };
                  });
                  default = {};
                };
              });
              default = {};
            };
            options.myConfig.modules.users.username = lib.mkOption {
              type = lib.types.str;
              default = username;
            };
          }
          module
        ] ++ lib.optional (enabled != null) { myConfig.modules.${name}.enable = enabled; };
      };
      option = evaluated.options.myConfig.modules.${name}.enable;
    in {
      enabled = evaluated.config.myConfig.modules.${name}.enable;
      type = option.type.name;
      default = option.default;
      description = option.description;
      packages = map (package: { name = lib.getName package; outPath = package.outPath; }) evaluated.config.environment.systemPackages;
      users = lib.mapAttrs (_: user: user.xdg.configFile) evaluated.config.home-manager.users;
    };
  compare = name: map (username: map (homeDirectory: map (enabled:
    let
      actual = inspect name (import (generated + "/${name}.nix")) username homeDirectory enabled;
      expected = inspect name (import (./. + "/${name}-baseline.nix")) username homeDirectory enabled;
      filename = if name == "fastfetch" then "config.jsonc" else "config.toml";
      source = "captured-inner:${homeDirectory}/code/nixos-config/dotfiles/${name}/${filename}";
    in assert actual == expected;
       assert actual.users == (if enabled == true then { ${username}.${"${name}/${filename}"}.source = source; } else {});
       { inherit username homeDirectory enabled actual; }
  ) [ null false true ]) [ "/home/inner-user" "/srv/homes/different.user" ]) [ "tom" "tom-test.user" ];
  interpolation = (import (generated + "/interpolation.nix")) { pkgs.name = "fixture-package"; };
in assert interpolation == "Driver = fixture-package/lib/driver.so\n";
   { fastfetch = compare "fastfetch"; tealdeer = compare "tealdeer"; inherit interpolation; }
