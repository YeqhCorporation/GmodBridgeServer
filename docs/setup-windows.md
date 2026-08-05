# Windows Setup

## Requirements

- Garry's Mod 32-bit or x86-64 branch.
- Rust toolchain with `i686-pc-windows-msvc` and `x86_64-pc-windows-msvc`.
- GABS binary available on PATH or through an absolute path.

## Build The Module

```powershell
rustup target add i686-pc-windows-msvc
rustup target add x86_64-pc-windows-msvc
cargo test -p gmod-gabp
cargo build -p gmod-gabp --target i686-pc-windows-msvc
cargo build -p gmod-gabp --target x86_64-pc-windows-msvc
```

## Install Into GMod

```powershell
$serverGmod = "C:\Games\GarrysModServer\garrysmod"
$clientGmod = "C:\Program Files (x86)\Steam\steamapps\common\GarrysMod\garrysmod"

New-Item -ItemType Directory -Force "$serverGmod\lua\bin" | Out-Null
New-Item -ItemType Directory -Force "$clientGmod\lua\bin" | Out-Null

Copy-Item target\i686-pc-windows-msvc\debug\gmod_gabp.dll "$serverGmod\lua\bin\gmsv_gabp_win32.dll"
Copy-Item target\x86_64-pc-windows-msvc\debug\gmod_gabp.dll "$serverGmod\lua\bin\gmsv_gabp_win64.dll"
Copy-Item target\i686-pc-windows-msvc\debug\gmod_gabp.dll "$clientGmod\lua\bin\gmcl_gabp_win32.dll"
Copy-Item target\x86_64-pc-windows-msvc\debug\gmod_gabp.dll "$clientGmod\lua\bin\gmcl_gabp_win64.dll"

Copy-Item -Recurse -Force addons\gabp_bridge "$serverGmod\addons\gabp_bridge"
Copy-Item -Recurse -Force addons\gabp_bridge "$clientGmod\addons\gabp_bridge"
```

If the server logs `Module not found!` while `gmsv_gabp_win64.dll` exists, the
server is probably running 32-bit and needs `gmsv_gabp_win32.dll`.

## Configure GABS

Create a local wrapper from the example:

```powershell
Copy-Item scripts\start-gmod-dev.example.cmd scripts\start-gmod-dev.cmd
notepad scripts\start-gmod-dev.cmd
```

Set `GMOD_SERVER_ROOT` to your SRCDS install directory. Then configure GABS with
`CustomCommand` and use the wrapper as the target:

```text
Target:
C:\Users\thiag\Documents\GitHub\GmodBridgeServer\scripts\start-gmod-dev.cmd

Working directory:
C:\Users\thiag\Documents\GitHub\GmodBridgeServer
```

Do not put the SRCDS executable and arguments together in one quoted `target`
string. GABS v1.0.8 treats `target` as the executable path for `CustomCommand`.

Create a local client wrapper from the example:

```powershell
Copy-Item scripts\start-gmod-client-dev.example.cmd scripts\start-gmod-client-dev.cmd
notepad scripts\start-gmod-client-dev.cmd
```

Set `GMOD_CLIENT_EXE` to your installed Garry's Mod client executable, usually
`gmod.exe` in the Garry's Mod Steam install directory.

Launching through GABS lets it pass `GABP_SERVER_PORT`, `GABP_TOKEN`, and
`GABS_GAME_ID` into the process.

## Validate

1. Start the game through GABS.
2. Run `games_connect` for the configured game id.
3. Run `games_tool_names`.
4. Call `server/status`.
5. Keep `gabp_allow_mutations 0` unless using a private development server.

For the client:

1. Start `gmod-client-dev` through GABS after `gmod-dev` is running.
2. Run `games_connect` for `gmod-client-dev`.
3. Run `games_tool_names` and confirm `client/status` appears.
4. Call `client/status`.
5. Keep `gabp_client_allow_mutations 0` unless using a private development client.
