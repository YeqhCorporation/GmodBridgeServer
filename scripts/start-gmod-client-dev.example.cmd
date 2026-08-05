@echo off
setlocal

rem Copy this file to start-gmod-client-dev.cmd and adjust GMOD_CLIENT_EXE.
set "GMOD_CLIENT_EXE=C:\Program Files (x86)\Steam\steamapps\common\GarrysMod\gmod.exe"
set "GMOD_SERVER_ADDRESS=127.0.0.1:27015"

"%GMOD_CLIENT_EXE%" -game garrysmod +connect %GMOD_SERVER_ADDRESS%
