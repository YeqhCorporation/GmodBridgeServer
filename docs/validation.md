# Validation and Troubleshooting

Use this checklist only on a private development system. It verifies the local
GABS connection, both bridge modules.

For the game registrations and local wrappers used below, see the
[GABS configuration guide](gabs-configuration.md). For the project overview,
return to the [README](../README.md).

## Before You Start

- Install the bridge addon and the server and client modules that match each
  process architecture.
- Configure the local `gmod-dev` server and `gmod-client-dev` client game IDs
  in GABS. These are example game IDs; they are separate from the MCP
  connection name.
- Keep the test server private and start with mutation convars at their default
  value of `0`: `gabp_allow_mutations` on the server and
  `gabp_client_allow_mutations` on the client.

The read-only checks below are enough for routine validation. The displayed
tool names are representative tool families, not an immutable exhaustive list.

## Launch and Connect

1. Start `gmod-dev` through GABS, then connect it with `games_connect`.
2. Start `gmod-client-dev` through GABS, then connect it with
   `games_connect`.
3. Join the local server from the GMod client.

If a start request reports `started_bridge_pending`, the process has started
but its bridge has not connected yet. Wait for the game to finish loading, then
run `games_connect` again; treat it as a wait-and-connect state rather than an
immediate failure.

## Verify the Server Bridge

Use `games_tool_names` after the server connects. Confirm that representative
server read-only tool families are available, including `server/status`,
`server/list_players`, `map/current`, and `entity/list`.

Call `server/status` to confirm the bridge can report server state. Then use
`map/current` to record the active map and `server/list_players` to confirm the
joined client or any test bot is visible.

## Verify the Client Bridge

Use `games_tool_names` after the client connects. Confirm that representative
client read-only tool families are available: `client/status`,
`client/local_player`, and `client/entities`.

Call `client/status`, then `client/local_player` after joining the server. The
result should identify the locally controlled player without requiring a
mutating tool call.

## Mutation Safety

Mutation convars default to `0`. Only enable a mutation convar temporarily on
a private test system for a narrowly scoped experiment, and restore it to `0`
as soon as the check is complete. Do not enable mutations for shared, public,
or unattended servers or clients.

## Stop Cleanly

Disconnect the client from the test server, then stop `gmod-client-dev` and
`gmod-dev` through GABS. Confirm that the configured `gmod.exe` and `srcds.exe`
processes have exited before starting another test run. If a process remains,
use the troubleshooting guidance below before relaunching.

## Troubleshooting

| Symptom | Likely cause | Safe response |
| --- | --- | --- |
| `require("gabp")` fails or the bridge never registers after launch | The module architecture does not match the running server or client process. | Rebuild or install the matching `win32` or `win64` module in that process's `garrysmod/lua/bin/` directory, then restart through GABS. |
| Server tools work but client tools are missing or unexpected | The client is using a stale bridge addon copy. | Replace the client addon from the current `addons/gabp_bridge` contents, confirm there is only one active copy, and restart the client. |
| `games_tool_names` lacks the expected server or client family | The game was not registered correctly, or GABS connected to the wrong game ID. | Check the local registration and wrapper settings in the [GABS configuration guide](gabs-configuration.md), then start and connect the intended game ID again. |
| `started_bridge_pending` persists after the game has finished loading | The bridge has not reached GABS, often because the wrapper, addon, module, port, or token handoff is incomplete. | Verify the game was launched through GABS, inspect the game console for module/addon errors, and retry `games_connect` after correcting the local setup. Keep tokens out of logs and committed files. |
| `gmod.exe` or `srcds.exe` survives after GABS reports shutdown | A child process did not exit cleanly. | Wait briefly, verify the process identity, then close only the confirmed local test child process before the next run. Review its console output if this recurs. |
