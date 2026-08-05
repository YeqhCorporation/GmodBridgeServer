local allowMutations = CreateConVar("gabp_allow_mutations", "0", FCVAR_ARCHIVE, "Allow GABP mutating development tools.")

local function RequireMutations()
  if not allowMutations:GetBool() then
    error("Mutating GABP tools are disabled. Set gabp_allow_mutations 1 in a private development server.")
  end
end

local function FindPlayerByUserId(userId)
  userId = tonumber(userId)
  if userId == nil then return nil end

  for _, ply in ipairs(player.GetAll()) do
    if ply:UserID() == userId then return ply end
  end

  return nil
end

function GABPBridge.RegisterMutatingBuiltins()
  GABPBridge.RegisterTool("server/run_console_command", {
    description = "Run an allowlisted server console command.",
    tags = { "development", "mutating" },
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
    RequireMutations()

    local allowed = {
      changelevel = true,
      hostname = true,
      lua_openscript = true,
      map = true,
      sv_gravity = true
    }

    local command = tostring(args.command or "")
    if not allowed[command] then
      error("Console command is not allowlisted: " .. command)
    end

    local commandArgs = {}
    for _, value in ipairs(args.args or {}) do
      table.insert(commandArgs, tostring(value))
    end

    RunConsoleCommand(command, unpack(commandArgs))
    return { ran = true, command = command, args = commandArgs }
  end)

  GABPBridge.RegisterTool("server/changelevel", {
    description = "Change the active map.",
    tags = { "development", "mutating" },
    inputSchema = {
      type = "object",
      properties = { map = { type = "string" } },
      required = { "map" }
    },
    outputSchema = { type = "object" }
  }, function(args)
    RequireMutations()
    local mapName = tostring(args.map or "")
    if mapName == "" or mapName:find("[^%w_%-]") then
      error("Invalid map name")
    end

    RunConsoleCommand("changelevel", mapName)
    return { changing = true, map = mapName }
  end)

  GABPBridge.RegisterTool("player/kick", {
    description = "Kick a player by UserID.",
    tags = { "development", "mutating" },
    inputSchema = {
      type = "object",
      properties = {
        userId = { type = "number" },
        reason = { type = "string" }
      },
      required = { "userId" }
    },
    outputSchema = { type = "object" }
  }, function(args)
    RequireMutations()
    local ply = FindPlayerByUserId(args.userId)
    if not IsValid(ply) then error("Player not found") end

    ply:Kick(tostring(args.reason or "Kicked by GABP development bridge"))
    return { kicked = true, userId = args.userId }
  end)

  GABPBridge.RegisterTool("player/teleport", {
    description = "Teleport a player by UserID.",
    tags = { "development", "mutating" },
    inputSchema = {
      type = "object",
      properties = {
        userId = { type = "number" },
        x = { type = "number" },
        y = { type = "number" },
        z = { type = "number" }
      },
      required = { "userId", "x", "y", "z" }
    },
    outputSchema = { type = "object" }
  }, function(args)
    RequireMutations()
    local ply = FindPlayerByUserId(args.userId)
    if not IsValid(ply) then error("Player not found") end

    ply:SetPos(Vector(tonumber(args.x), tonumber(args.y), tonumber(args.z)))
    return { teleported = true, userId = args.userId, position = tostring(ply:GetPos()) }
  end)
end
