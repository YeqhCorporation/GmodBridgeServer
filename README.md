# GmodBridgeServer

Native Garry's Mod bridge for GABS/GABP. The Windows module can be installed
server-side as `gmsv_gabp_win32.dll` or `gmsv_gabp_win64.dll`, and client-side as
`gmcl_gabp_win32.dll` or `gmcl_gabp_win64.dll`, loaded from GLua with
`require("gabp")`.

## Development Shape

GABS is the MCP server. This project is the GMod-side GABP server:

```text
AI client -> MCP -> GABS -> gmod-dev        -> SRCDS + gmsv_gabp + server tools
AI client -> MCP -> GABS -> gmod-client-dev -> GMod client + gmcl_gabp + client tools
```

## Quick Start

1. [Build the bridge](#build) for the Windows architectures you need.
2. [Install the modules and addon](docs/installation.md) in the local server and
   client installations.
3. [Configure GABS and MCP](docs/gabs-configuration.md) to launch and connect
   the bridge.

## Build

```powershell
rustup target add x86_64-pc-windows-msvc
rustup target add i686-pc-windows-msvc
cargo test -p gmod-gabp
cargo build -p gmod-gabp --target i686-pc-windows-msvc
cargo build -p gmod-gabp --target x86_64-pc-windows-msvc
```

The produced DLLs are installed under these four module names:

- `target/i686-pc-windows-msvc/debug/gmod_gabp.dll` -> `gmsv_gabp_win32.dll`
- `target/x86_64-pc-windows-msvc/debug/gmod_gabp.dll` -> `gmsv_gabp_win64.dll`
- `target/i686-pc-windows-msvc/debug/gmod_gabp.dll` -> `gmcl_gabp_win32.dll`
- `target/x86_64-pc-windows-msvc/debug/gmod_gabp.dll` -> `gmcl_gabp_win64.dll`

## Releases

Pushing a version tag publishes a GitHub release automatically:

```powershell
git tag v0.1.1
git push origin v0.1.1
```

The release workflow builds Windows and Linux modules, uploads individual DLLs,
publishes a modules-only zip, and publishes an installable zip containing
`addons/gabp_bridge` plus `lua/bin` with the ready-to-use module names.

## Documentation

| Guide | Purpose |
| --- | --- |
| [Installation](docs/installation.md) | Install the server/client modules and bridge addon in local Garry's Mod installations. |
| [GABS and MCP configuration](docs/gabs-configuration.md) | Configure GABS, MCP, and separate local server/client launch entries. |
| [Windows setup](docs/setup-windows.md) | Retained Windows setup reference for the original server-oriented workflow. |

## Reference Material

- [GABS](https://github.com/pardeike/GABS)
- [GABP](https://github.com/pardeike/GABP)
- [RimBridgeServer](https://github.com/pardeike/RimBridgeServer)
