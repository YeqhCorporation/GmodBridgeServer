@echo off
setlocal

set "GMOD_SERVER_ROOT=C:\Games\GarrysModServer"

cd /d "%GMOD_SERVER_ROOT%"
srcds.exe -console -game garrysmod +maxplayers 4 +gamemode sandbox +map gm_construct
