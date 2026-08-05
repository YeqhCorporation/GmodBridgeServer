if not CLIENT then return end

local ok, err = pcall(require, "gabp")
if not ok then
  MsgC(Color(255, 80, 80), "[gabp_bridge_client] failed to load native module: " .. tostring(err) .. "\n")
  return
end

include("gabp_bridge/init.lua")
include("gabp_bridge/client_builtins.lua")

local started, startErr = gabp.start()
if not started then
  MsgC(Color(255, 180, 80), "[gabp_bridge_client] bridge not started: " .. tostring(startErr) .. "\n")
  return
end

GABPBridge.RegisterClientBuiltins()

hook.Add("Think", "GABPBridge.ClientPoll", function()
  GABPBridge.Poll()
end)
