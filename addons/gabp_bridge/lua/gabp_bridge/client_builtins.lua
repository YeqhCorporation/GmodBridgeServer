local allowMutations = nil

local function EnsureAllowMutations()
  if allowMutations == nil then
    allowMutations = CreateConVar("gabp_client_allow_mutations", "0", FCVAR_ARCHIVE, "Allow GABP mutating client development tools.")
  end

  if not allowMutations:GetBool() then
    error("Mutating client GABP tools are disabled. Set gabp_client_allow_mutations 1 in a private development client.")
  end
end

local function VectorToTable(value)
  if value == nil then return nil end
  return { x = value.x, y = value.y, z = value.z, string = tostring(value) }
end

local function AngleToTable(value)
  if value == nil then return nil end
  return { pitch = value.p, yaw = value.y, roll = value.r, string = tostring(value) }
end

local function SafeStringCall(fn, fallback)
  local ok, value = pcall(fn)
  if ok then return value end
  return fallback
end

local function IsAllowedClientCommand(command)
  local allowed = {
    connect = true,
    disconnect = true,
    retry = true,
    record = true,
    stop = true,
    jpeg = true,
    mat_reloadallmaterials = true
  }

  return allowed[command] == true
end

function GABPBridge.RegisterClientBuiltins()
  GABPBridge.RegisterTool("client/status", {
    description = "Read basic client status.",
    tags = { "read-only", "status", "observation", "client" },
    inputSchema = { type = "object", properties = {} },
    outputSchema = { type = "object" }
  }, function()
    local gamemodeName = nil
    if GM ~= nil then
      gamemodeName = GM.Name or GM.FolderName
    end

    return {
      map = game.GetMap(),
      frameTime = FrameTime(),
      realTime = RealTime(),
      screenWidth = ScrW(),
      screenHeight = ScrH(),
      gamemode = gamemodeName,
      isInGame = SafeStringCall(function() return IsInGame() end, nil),
      maxPlayers = game.MaxPlayers()
    }
  end)

  GABPBridge.RegisterTool("client/local_player", {
    description = "Read local player state.",
    tags = { "read-only", "observation", "client" },
    inputSchema = { type = "object", properties = {} },
    outputSchema = { type = "object" }
  }, function()
    local ply = LocalPlayer()
    if not IsValid(ply) then
      return { valid = false }
    end

    return {
      valid = true,
      name = ply:Nick(),
      steamId = SafeStringCall(function() return ply:SteamID() end, nil),
      steamId64 = SafeStringCall(function() return ply:SteamID64() end, nil),
      health = ply:Health(),
      armor = ply:Armor(),
      team = ply:Team(),
      alive = ply:Alive(),
      position = VectorToTable(ply:GetPos()),
      eyeAngles = AngleToTable(ply:EyeAngles()),
      inVehicle = ply:InVehicle()
    }
  end)

  GABPBridge.RegisterTool("client/convars/get", {
    description = "Read a client convar.",
    tags = { "read-only", "observation", "client" },
    inputSchema = {
      type = "object",
      properties = {
        name = { type = "string" }
      },
      required = { "name" }
    },
    outputSchema = { type = "object" }
  }, function(args)
    local name = tostring(args.name or "")
    if name == "" or name:find("[^%w_%.%-]") then
      error("Invalid convar name")
    end

    local convar = GetConVar(name)
    if convar == nil then
      return { name = name, exists = false }
    end

    return {
      name = name,
      exists = true,
      string = convar:GetString(),
      number = convar:GetFloat(),
      boolean = convar:GetBool(),
      default = SafeStringCall(function() return convar:GetDefault() end, nil)
    }
  end)

  GABPBridge.RegisterTool("client/entities", {
    description = "List client-visible entities.",
    tags = { "read-only", "observation", "client" },
    inputSchema = {
      type = "object",
      properties = {
        class = { type = "string" },
        limit = { type = "number" }
      }
    },
    outputSchema = { type = "object" }
  }, function(args)
    local entities = {}
    local limit = math.Clamp(tonumber(args.limit) or 100, 1, 500)
    local classFilter = tostring(args.class or "")

    for _, ent in ipairs(ents.GetAll()) do
      if classFilter == "" or ent:GetClass() == classFilter then
        table.insert(entities, {
          index = ent:EntIndex(),
          class = ent:GetClass(),
          model = ent:GetModel(),
          position = VectorToTable(ent:GetPos()),
          valid = IsValid(ent)
        })

        if #entities >= limit then break end
      end
    end

    return { entities = entities, count = #entities }
  end)

  GABPBridge.RegisterTool("client/connect_to_server", {
    description = "Connect the client to a server.",
    tags = { "development", "mutating", "client" },
    inputSchema = {
      type = "object",
      properties = {
        address = { type = "string" }
      },
      required = { "address" }
    },
    outputSchema = { type = "object" }
  }, function(args)
    EnsureAllowMutations()
    local address = tostring(args.address or "")
    if address == "" or address:find("[^%w_%.:%-]") then
      error("Invalid server address")
    end

    RunConsoleCommand("connect", address)
    return { connecting = true, address = address }
  end)

  GABPBridge.RegisterTool("client/disconnect", {
    description = "Disconnect the client from the current server.",
    tags = { "development", "mutating", "client" },
    inputSchema = { type = "object", properties = {} },
    outputSchema = { type = "object" }
  }, function()
    EnsureAllowMutations()
    RunConsoleCommand("disconnect")
    return { disconnecting = true }
  end)

  GABPBridge.RegisterTool("client/run_console_command", {
    description = "Run an allowlisted client console command.",
    tags = { "development", "mutating", "client" },
    inputSchema = {
      type = "object",
      properties = {
        command = { type = "string" },
        args = { type = "array" }
      },
      required = { "command" }
    },
    outputSchema = { type = "object" }
  }, function(args)
    EnsureAllowMutations()

    local command = tostring(args.command or "")
    if not IsAllowedClientCommand(command) then
      error("Client console command is not allowlisted: " .. command)
    end

    local commandArgs = {}
    for _, value in ipairs(args.args or {}) do
      table.insert(commandArgs, tostring(value))
    end

    RunConsoleCommand(command, unpack(commandArgs))
    return { ran = true, command = command, args = commandArgs }
  end)
end
