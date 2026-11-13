# Exercise the generated unit, profile selection and Bubblewrap together.
{
  pkgs,
  module,
}: let
  settings = port: runtimeDirectory: {
    transports = {
      openai = false;
      http = true;
      http_bind = "127.0.0.1:${toString port}";
    };
    permissions = {
      default_allow = true;
      allow_shell = true;
      allow_read = ["/run/${runtimeDirectory}" "/run/credentials/${runtimeDirectory}.service"];
    };
  };
  probe = pkgs.writeText "dotlink-service-probe.py" ''
    import json
    import sys
    import urllib.request

    port, unit, filename = sys.argv[1:]
    url = f"http://127.0.0.1:{port}/mcp"
    headers = {"Content-Type": "application/json", "Accept": "application/json, text/event-stream"}

    def rpc(method, params, request_id=None):
        body = {"jsonrpc": "2.0", "method": method, "params": params}
        if request_id is not None:
            body["id"] = request_id
        request = urllib.request.Request(url, json.dumps(body).encode(), headers)
        with urllib.request.urlopen(request, timeout=30) as response:
            session = response.headers.get("Mcp-Session-Id")
            if session:
                headers["Mcp-Session-Id"] = session
            raw = response.read().decode()
            if not raw:
                return None
            if response.headers.get("Content-Type", "").startswith("text/event-stream"):
                raw = next(line[6:] for line in raw.splitlines() if line.startswith("data: "))
            result = json.loads(raw)
            assert "error" not in result, result
            return result["result"]

    rpc("initialize", {"protocolVersion": "2025-03-26", "capabilities": {},
                       "clientInfo": {"name": "nixos-test", "version": "1"}}, 1)
    headers["MCP-Protocol-Version"] = "2025-03-26"
    rpc("notifications/initialized", {})
    config = f"/run/{unit}/{filename}"
    credential = f"/run/credentials/{unit}.service/dotlink-api-key"
    for request_id, path in enumerate([config, credential], 2):
        result = rpc("tools/call", {"name": "read", "arguments": {"path": path}}, request_id)
        assert result.get("isError"), result
    command = (
        f"test ! -s {config} && test ! -s {credential} && "
        "test -z \"$DOTLINK_API_KEY_FILE\" && cat fixture.txt"
    )
    result = rpc("tools/call", {"name": "bash", "arguments": {"command": command}}, 4)
    assert not result.get("isError", False), result
    output = json.loads(next(item["text"] for item in result["content"] if item["type"] == "text"))
    assert output["exit_code"] == 0, output
    assert output["sandboxed"] is True, output
    assert output["stdout"] == "project-ok\n", output
  '';
  storeConfigs = pkgs.runCommand "dotlink-store-configs" {} ''
    mkdir -p "$out/odd'quote\$%"
    cp ${pkgs.writeText "default.jsonc" (builtins.toJSON ((settings 3000 "dotlink") // {version = 10;}))} "$out/odd'quote\$%/custom.jsonc"
    cp ${pkgs.writeText "named.jsonc" (builtins.toJSON ((settings 3001 "dotlink-named") // {version = 10;}))} "$out/odd'quote\$%/custom.named.jsonc"
  '';
  node = storeBacked: {
    imports = [module];
    virtualisation.memorySize = 2048;
    environment.systemPackages = [pkgs.python3];
    systemd.tmpfiles.rules = [
      "d /var/lib/dotlink/work 0700 dotlink dotlink -"
      "f /var/lib/dotlink/work/fixture.txt 0600 dotlink dotlink - project-ok\\n"
      "f /run/dotlink-test-key 0600 root root - test-key"
    ];
    services.dotlink = {
      enable = true;
      workingDirectory = "/var/lib/dotlink/work";
      settings =
        if storeBacked
        then null
        else settings 3000 "dotlink";
      configFile =
        if storeBacked
        then "${storeConfigs}/odd'quote$%/custom.jsonc"
        else null;
      credentials.runtimeApiKeyFile = "/run/dotlink-test-key";
      instances.named = {
        settings =
          if storeBacked
          then null
          else settings 3001 "dotlink-named";
        configFile =
          if storeBacked
          then "${storeConfigs}/odd'quote$%/custom.jsonc"
          else null;
        credentials.runtimeApiKeyFile = "/run/dotlink-test-key";
      };
    };
  };
in
  pkgs.testers.runNixOSTest {
    name = "dotlink-nixos-service";
    nodes = {
      machine = node false;
      store = node true;
    };
    testScript = ''
      start_all()
      for host in [machine, store]:
          for unit, port, filename in [
              ("dotlink", 3000, "config.jsonc"),
              ("dotlink-named", 3001, "config.named.jsonc"),
          ]:
              host.wait_for_unit(f"{unit}.service")
              host.wait_for_open_port(port)
              config = f"/run/{unit}/{filename}"
              host.succeed(f"test $(stat -c %a /run/{unit}) = 700")
              host.succeed(f"test $(stat -c %a {config}) = 400")
              host.succeed(f"test ! -L {config}")
              host.succeed(f"${pkgs.python3}/bin/python ${probe} {port} {unit} {filename}")
              # A fresh start must replace runtime edits with declarative settings.
              host.succeed(f"chmod 600 {config}; echo invalid > {config}")
              host.succeed(f"systemctl restart {unit}")
              host.wait_for_open_port(port)
              host.succeed(f"${pkgs.python3}/bin/python ${probe} {port} {unit} {filename}")
              host.succeed(f"systemctl stop {unit}; test ! -e /run/{unit}")
              host.succeed(f"test ! -e /run/credentials/{unit}.service")
    '';
  }
