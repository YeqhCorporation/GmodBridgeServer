if not SERVER then return end

AddCSLuaFile("autorun/client/gabp_bridge_client.lua")
AddCSLuaFile("gabp_bridge/init.lua")
AddCSLuaFile("gabp_bridge/client_builtins.lua")

local ok, err = pcall(require, "gabp")
if not ok then
  MsgC(Color(255, 80, 80), "[gabp_bridge] failed to load native module: " .. tostring(err) .. "\n")
  return
end

include("gabp_bridge/init.lua")
include("gabp_bridge/builtins_readonly.lua")
include("gabp_bridge/builtins_mutating.lua")

local started, startErr = gabp.start()
if not started then
  MsgC(Color(255, 180, 80), "[gabp_bridge] bridge not started: " .. tostring(startErr) .. "\n")
  return
end

GABPBridge.RegisterBuiltins()

hook.Add("Think", "GABPBridge.Poll", function()
  GABPBridge.Poll()
end)
