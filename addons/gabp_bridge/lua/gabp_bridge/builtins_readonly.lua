function GABPBridge.RegisterReadOnlyBuiltins()
  GABPBridge.RegisterTool("server/status", {
    description = "Read basic server status.",
    tags = { "read-only", "status", "observation" },
    inputSchema = { type = "object", properties = {} },
    outputSchema = { type = "object" }
  }, function()
    return {
      hostname = GetHostName(),
      map = game.GetMap(),
      maxPlayers = game.MaxPlayers(),
      playerCount = #player.GetAll(),
      frameTime = FrameTime(),
      curTime = CurTime()
    }
  end)

  GABPBridge.RegisterTool("server/list_players", {
    description = "List connected players.",
    tags = { "read-only", "observation" },
    inputSchema = { type = "object", properties = {} },
    outputSchema = { type = "object" }
  }, function()
    local players = {}

    for _, ply in ipairs(player.GetAll()) do
      table.insert(players, {
        name = ply:Nick(),
        steamId = ply:SteamID(),
        userId = ply:UserID(),
        health = ply:Health(),
        alive = ply:Alive(),
        position = tostring(ply:GetPos())
      })
    end

    return { players = players }
  end)

  GABPBridge.RegisterTool("map/current", {
    description = "Read current map name.",
    tags = { "read-only", "status", "observation" },
    inputSchema = { type = "object", properties = {} },
    outputSchema = { type = "object" }
  }, function()
    return { map = game.GetMap() }
  end)

  GABPBridge.RegisterTool("entity/list", {
    description = "List entities with class, index, model, and position.",
    tags = { "read-only", "observation" },
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

    for _, ent in ipairs(ents.GetAll()) do
      if args.class == nil or args.class == "" or ent:GetClass() == args.class then
        table.insert(entities, {
          index = ent:EntIndex(),
          class = ent:GetClass(),
          model = ent:GetModel(),
          position = tostring(ent:GetPos())
        })

        if #entities >= limit then break end
      end
    end

    return { entities = entities, count = #entities }
  end)
end

function GABPBridge.RegisterBuiltins()
  GABPBridge.RegisterReadOnlyBuiltins()
  if GABPBridge.RegisterMutatingBuiltins then
    GABPBridge.RegisterMutatingBuiltins()
  end
end
