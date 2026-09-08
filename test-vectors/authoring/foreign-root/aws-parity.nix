{ generated, nixpkgs, pathRoot }:
let
  pkgs = import nixpkgs { system = "x86_64-linux"; };
  lib = pkgs.lib;
  inspect = module: fixture: enabled:
    let
      evaluated = lib.evalModules {
        specialArgs = { inherit pkgs; inherit (fixture) flakeRoot; };
        modules = [
          { options.environment.systemPackages = lib.mkOption { type = lib.types.listOf lib.types.package; default = []; };
            options.sops.secrets = lib.mkOption { type = lib.types.attrsOf lib.types.anything; default = {}; };
            options.sops.templates = lib.mkOption { type = lib.types.attrsOf lib.types.anything; default = {}; };
            options.sops.placeholder = lib.mkOption { type = lib.types.attrsOf lib.types.str; };
            options.myConfig.modules.users.username = lib.mkOption { type = lib.types.str; };
            config.myConfig.modules.users.username = fixture.username;
            config.sops.placeholder = { aws-access-key-id = "INVENTED-ACCESS-PLACEHOLDER"; aws-secret-access-key = "INVENTED-SECRET-PLACEHOLDER"; };
          }
          module
        ] ++ lib.optional (enabled != null) { myConfig.modules.awscli.enable = enabled; }
          ++ lib.optional (fixture.sopsFile != null) { myConfig.modules.awscli.sopsFile = fixture.sopsFile; };
      };
      option = evaluated.options.myConfig.modules.awscli.enable;
      sopsOption = evaluated.options.myConfig.modules.awscli.sopsFile;
    in {
      enabled = evaluated.config.myConfig.modules.awscli.enable;
      type = option.type.name; default = option.default; description = option.description;
      sopsOption = { type = sopsOption.type.name; default = sopsOption.default; description = sopsOption.description; value = evaluated.config.myConfig.modules.awscli.sopsFile; };
      packages = map (p: { packageName = p.name; packagePath = p.outPath; }) evaluated.config.environment.systemPackages;
      secrets = evaluated.config.sops.secrets;
      templates = evaluated.config.sops.templates;
    };
  actual = inspect (import (generated + "/awscli.nix"));
  expected = inspect (import ./awscli-baseline.nix);
  fixtures = [
    { username = "tom"; flakeRoot = "/invented/root"; sopsFile = null; }
    { username = "tom-test.user"; flakeRoot = pathRoot; sopsFile = null; }
    { username = "tom-test.user"; flakeRoot = "/invented/custom root"; sopsFile = "/invented/override.yaml"; }
  ];
in map (fixture: map (enabled: let a = actual fixture enabled; e = expected fixture enabled; in assert a == e; a) [ null false true ]) fixtures
