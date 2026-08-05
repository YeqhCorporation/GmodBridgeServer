# Installation

This guide installs the native GABP bridge module and its companion addon for
Garry's Mod or SRCDS. For an overview of the project and its development
shape, see the [README](../README.md).

## Prerequisites

- A Garry's Mod client and/or an SRCDS/Garry's Mod server installation.
- The Rust toolchain for Windows MSVC builds.
- The `i686-pc-windows-msvc` and `x86_64-pc-windows-msvc` Rust targets.
- PowerShell, run from the repository root.

## Build

Install the two Windows targets, test the module, and build both architectures:

```powershell
rustup target add i686-pc-windows-msvc
rustup target add x86_64-pc-windows-msvc
cargo test -p gmod-gabp
cargo build -p gmod-gabp --target i686-pc-windows-msvc
cargo build -p gmod-gabp --target x86_64-pc-windows-msvc
```

Each build produces `gmod_gabp.dll` under its target directory:

- `target/i686-pc-windows-msvc/debug/gmod_gabp.dll`
- `target/x86_64-pc-windows-msvc/debug/gmod_gabp.dll`

## Architecture Selection

Build and install both architectures when practical. The architecture of the
running GMod or SRCDS process determines which module is loaded; the Steam
launcher alone does not determine it. A 32-bit process loads a `win32` module,
while an x86-64 process loads its `win64` counterpart.

## Server Installation

Set a generic, editable server root. The `garrysmod` folder is the directory
that contains `lua` and `addons` for the server process.

```powershell
$serverGmod = 'C:\Games\GarrysModServer\garrysmod'

New-Item -ItemType Directory -Force "$serverGmod\lua\bin" | Out-Null
Copy-Item 'target\i686-pc-windows-msvc\debug\gmod_gabp.dll' "$serverGmod\lua\bin\gmsv_gabp_win32.dll" -Force
Copy-Item 'target\x86_64-pc-windows-msvc\debug\gmod_gabp.dll' "$serverGmod\lua\bin\gmsv_gabp_win64.dll" -Force

New-Item -ItemType Directory -Force "$serverGmod\addons\gabp_bridge" | Out-Null
Copy-Item 'addons\gabp_bridge\*' "$serverGmod\addons\gabp_bridge" -Recurse -Force
```

The server module names are `gmsv_gabp_win32.dll` and
`gmsv_gabp_win64.dll`.

## Client Installation

Set a separate, generic client root. It is intentionally independent from the
server root so a local client and a dedicated server can be installed or
updated separately.

```powershell
$clientGmod = 'C:\Games\GarrysModClient\garrysmod'

New-Item -ItemType Directory -Force "$clientGmod\lua\bin" | Out-Null
Copy-Item 'target\i686-pc-windows-msvc\debug\gmod_gabp.dll' "$clientGmod\lua\bin\gmcl_gabp_win32.dll" -Force
Copy-Item 'target\x86_64-pc-windows-msvc\debug\gmod_gabp.dll' "$clientGmod\lua\bin\gmcl_gabp_win64.dll" -Force

New-Item -ItemType Directory -Force "$clientGmod\addons\gabp_bridge" | Out-Null
Copy-Item 'addons\gabp_bridge\*' "$clientGmod\addons\gabp_bridge" -Recurse -Force
```

The client module names are `gmcl_gabp_win32.dll` and
`gmcl_gabp_win64.dll`.

## Addon Updates

Create the destination addon directory before copying the addon contents.
Use `Copy-Item 'addons\gabp_bridge\*' <destination> -Recurse -Force`, with
`<destination>` replaced by the relevant `addons\gabp_bridge` directory as in
the examples above. Copying the contents, rather than the source directory
itself, prevents an accidental nested `gabp_bridge\gabp_bridge` directory.

Repeat the corresponding module-copy commands after each new build, then
restart the affected GMod or SRCDS process so it loads the updated DLL.

## Deployment Layouts

- **Dedicated server only:** install the server modules and addon under the
  server root.
- **Client only:** install the client modules and addon under the client root.
- **Separate local client and server:** keep `$serverGmod` and `$clientGmod`
  pointing to different installations, and install the appropriate modules and
  addon in each one.

The Lua addon loads the matching module with `require("gabp")`; keep the addon
and the architecture-compatible module together beneath the same root.

## Next Step

Continue with [GABS configuration](gabs-configuration.md) after the modules
and addon are in place.
