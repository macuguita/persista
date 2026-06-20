{
  config,
  lib,
  pkgs,
  ...
}:

with lib;

let
  cfg = config.services.persista;
in
{
  options.services.persista = {
    enable = mkEnableOption "Persista supporter API";

    package = mkOption {
      type = types.package;
      description = "The Persista package to use.";
    };

    port = mkOption {
      type = types.port;
      default = 8080;
      description = "Port Persista listens on.";
    };

    database = {
      enable = mkEnableOption "Enables the PostgreSQL service";
      host = mkOption {
        type = types.str;
        default = "localhost";
        description = "PostgreSQL host.";
      };
      port = mkOption {
        type = types.port;
        default = 5432;
        description = "PostgreSQL port.";
      };
      name = mkOption {
        type = types.str;
        default = "persista";
        description = "PostgreSQL database name.";
      };
      user = mkOption {
        type = types.str;
        default = "persista";
        description = "PostgreSQL user.";
      };
    };

    jwtSecretFile = mkOption {
      type = types.path;
      description = "File containing the JWT secret.";
    };

    dbPasswordFile = mkOption {
      type = types.path;
      description = "File containing the DB password.";
    };

    adminSecretFile = mkOption {
      type = types.path;
      description = "File containing the admin password.";
    };

    openFirewall = mkOption {
      type = types.bool;
      default = false;
      description = "Whether to open the firewall for the Persista port.";
    };

    nginx = {
      enable = mkOption {
        type = types.bool;
        default = false;
        description = "Whether to enable nginx reverse proxy.";
      };

      domain = mkOption {
        type = types.str;
        default = "";
        description = "Domain name for the nginx virtual host.";
        example = "persista.example.com";
      };

      enableACME = mkOption {
        type = types.bool;
        default = true;
        description = "Whether to enable ACME/Let's Encrypt SSL certificates.";
      };

      acmeEmail = mkOption {
        type = types.str;
        default = "";
        description = "Email address for ACME registration.";
        example = "admin@example.com";
      };

      forceSSL = mkOption {
        type = types.bool;
        default = true;
        description = "Whether to force SSL.";
      };

      extraConfig = mkOption {
        type = types.str;
        default = "";
        description = "Extra nginx configuration for the virtual host.";
        example = "access_log off;";
      };
    };
  };

  config = mkIf cfg.enable {
    assertions = [
      {
        assertion = !cfg.nginx.enable || cfg.nginx.domain != "";
        message = "services.persista.nginx.domain must be set when nginx is enabled";
      }
      {
        assertion = !cfg.nginx.enable || !cfg.nginx.enableACME || cfg.nginx.acmeEmail != "";
        message = "services.persista.nginx.acmeEmail must be set when ACME is enabled";
      }
    ];

    services.postgresql = mkIf (cfg.database.enable && cfg.database.host == "localhost") {
      enable = true;
      ensureDatabases = [ cfg.database.name ];
      ensureUsers = [
        {
          name = cfg.database.user;
          ensureDBOwnership = true;
        }
      ];
    };

    networking.firewall = lib.mkIf cfg.openFirewall {
      allowedTCPPorts = mkMerge [
        (mkIf cfg.openFirewall [ cfg.port ])
        (mkIf cfg.nginx.enable [
          80
          443
        ])
      ];
    };

    systemd.services.persista = {
      description = "Persista supporter API";
      after = [
        "network.target"
      ]
      ++ lib.optional (cfg.database.host == "localhost") "postgresql.service";
      requires = lib.optional (cfg.database.host == "localhost") "postgresql.service";
      wantedBy = [ "multi-user.target" ];

      serviceConfig = {
        Restart = "on-failure";
        RestartSec = "10s";
        DynamicUser = true;
        NoNewPrivileges = true;
        PrivateTmp = true;
        ProtectSystem = "strict";
        ProtectHome = true;
        LoadCredential = [
          "jwt_secret:${cfg.jwtSecretFile}"
          "db_password:${cfg.dbPasswordFile}"
          "admin_secret:${cfg.adminSecretFile}"
        ];
      };

      script = ''
        export PORT=${toString cfg.port}
        export DB_URL="postgresql://${cfg.database.user}:$(cat $CREDENTIALS_DIRECTORY/db_password)@${cfg.database.host}:${toString cfg.database.port}/${cfg.database.name}"
        export JWT_SECRET="$(cat $CREDENTIALS_DIRECTORY/jwt_secret)"
        export ADMIN_SECRET="$(cat $CREDENTIALS_DIRECTORY/admin_secret)"
        exec ${lib.getExe cfg.package}
      '';
    };

    services.nginx = mkIf cfg.nginx.enable {
      enable = true;
      virtualHosts.${cfg.nginx.domain} = {
        enableACME = cfg.nginx.enableACME;
        forceSSL = cfg.nginx.forceSSL;
        locations."/" = {
          proxyPass = "http://127.0.0.1:${toString cfg.port}";
          extraConfig = ''
            proxy_set_header Host $host;
            proxy_set_header X-Real-IP $remote_addr;
            proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
            proxy_set_header X-Forwarded-Proto $scheme;
            proxy_set_header CF-Connecting-IP $http_cf_connecting_ip;
          '';
        };
        extraConfig = cfg.nginx.extraConfig;
      };
    };

    security.acme = mkIf (cfg.nginx.enable && cfg.nginx.enableACME) {
      acceptTerms = true;
      defaults.email = cfg.nginx.acmeEmail;
    };
  };
}
