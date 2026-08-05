# GABS and MCP Configuration

This guide connects a local GMod bridge to GABS, which exposes the bridge's
tools to an MCP client. Install the module and addon first with the
[installation guide](installation.md).

## Two Names, Two Roles

An MCP connection name identifies the GABS server to the AI client. For
example, `gmodtest` can be the connection name in the client's MCP settings.
It is not a game ID and does not need to match one.

GABS game IDs identify individual launched games. This guide uses `gmod-dev`
for the SRCDS server and `gmod-client-dev` for the GMod client. They are the
keys and `id` values in `~/.gabs/config.json`, and the names supplied to GABS
commands such as `games_start` and `games_connect`.

## Install and Expose GABS

Install GABS from the [upstream GABS project](https://github.com/pardeike/GABS)
and make its executable available on `PATH` (or use its absolute path in the
MCP client's local server configuration). Configure the MCP client to start
GABS in server mode; a generic connection can look like this:

```json
{
  "mcpServers": {
    "gmodtest": {
      "command": "gabs",
      "args": ["server"]
    }
  }
}
```

`gmodtest` is only an example connection name. Choose a local name that suits
your MCP client; it does not belong in the GABS game configuration.

## Create Local Launch Wrappers

Copy the repository's example commands to local wrapper files, then edit the
copies for the installed server and client locations:

```powershell
Copy-Item scripts\start-gmod-dev.example.cmd scripts\start-gmod-dev.cmd
Copy-Item scripts\start-gmod-client-dev.example.cmd scripts\start-gmod-client-dev.cmd
```

The local wrappers remain outside version control. Keep tokens and any
machine-specific installation paths outside version control as well. Use the
wrapper files as `CustomCommand` targets rather than placing an executable and
its arguments in one quoted target string.

When GABS launches either wrapper, it supplies `GABP_SERVER_PORT`,
`GABP_TOKEN`, and `GABS_GAME_ID` to the launched game. Do not add or copy a
token into a wrapper or committed configuration file.

## Register the Server and Client Games

Create or update `~/.gabs/config.json` with editable local paths. The example
below registers both games through their local wrappers. Replace
`D:\\GmodBridgeServer` with the directory containing this checkout.

```json
{
  "version": "1.0",
  "games": {
    "gmod-dev": {
      "id": "gmod-dev",
      "name": "Garry's Mod Development Server",
      "launchMode": "CustomCommand",
      "target": "D:\\GmodBridgeServer\\scripts\\start-gmod-dev.cmd",
      "workingDir": "D:\\GmodBridgeServer",
      "stopProcessName": "srcds.exe",
      "description": "Local GMod server with the GABP bridge"
    },
    "gmod-client-dev": {
      "id": "gmod-client-dev",
      "name": "Garry's Mod Development Client",
      "launchMode": "CustomCommand",
      "target": "D:\\GmodBridgeServer\\scripts\\start-gmod-client-dev.cmd",
      "workingDir": "D:\\GmodBridgeServer",
      "stopProcessName": "gmod.exe",
      "description": "Local GMod client with the GABP bridge"
    }
  }
}
```

Keep the `workingDir` at the repository root so the wrapper can resolve its
repository-relative resources. The configuration and wrappers are local setup;
do not commit them or replace the generic example paths with personal paths.

## Start and Connect

From the MCP client, start `gmod-dev`, then connect it with `games_connect`.
After the server bridge is connected, start `gmod-client-dev`, then connect it
with `games_connect`. Join the local server from the client after it connects.
Use `games_tool_names` to confirm the server and client tool sets are available
before calling them.

GABS manages the generated port and token for each launch, so start games
through GABS rather than invoking the wrapper directly when testing the bridge.

## Next Step

Follow the [validation guide](validation.md) for the private development test
sequence and troubleshooting. For the project overview, return to the
[README](../README.md).
