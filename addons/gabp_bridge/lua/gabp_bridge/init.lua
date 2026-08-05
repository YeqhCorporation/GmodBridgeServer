GABPBridge = GABPBridge or {}
GABPBridge.Tools = GABPBridge.Tools or {}

local function Encode(value)
  return util.TableToJSON(value or {}, false) or "{}"
end

local function Decode(value)
  if value == nil or value == "" then return {} end
  return util.JSONToTable(value) or {}
end

function GABPBridge.RegisterTool(name, descriptor, callback)
  if type(name) ~= "string" then error("tool name must be a string") end
  if type(descriptor) ~= "table" then error("descriptor must be a table") end
  if type(callback) ~= "function" then error("callback must be a function") end

  descriptor.description = descriptor.description or name
  descriptor.inputSchema = descriptor.inputSchema or { type = "object", properties = {} }

  local ok, err = gabp.register_tool_native(name, Encode(descriptor))
  if not ok then error("failed to register GABP tool " .. name .. ": " .. tostring(err)) end

  GABPBridge.Tools[name] = callback
end

function GABPBridge.Poll()
  while true do
    local callJson = gabp.poll_call_native()
    if callJson == nil or callJson == "" then return end

    local call = Decode(callJson)
    local callback = GABPBridge.Tools[call.toolName]

    if callback == nil then
      gabp.fail_call_native(call.requestId, -32400, "Tool not found in Lua registry", Encode({ tool = call.toolName }))
    else
      local ok, result = xpcall(function()
        return callback(call.arguments or {})
      end, debug.traceback)

      if ok then
        gabp.complete_call_native(call.requestId, Encode(result or {}))
      else
        gabp.fail_call_native(call.requestId, -32402, tostring(result), nil)
      end
    end
  end
end
