Registry = Registry or {}

Registry.campaigns = {}

-- Registers a campaign
-- The table `schema` supports the following properties:
--   level = handle<level> -- the handle to the level that acts as the entry point to the campaign
function register_campaign(schema)
    Log.info("registered campaign with level {}", schema.level)
    table.insert(Registry.campaigns, schema)
end

Log.info("registry.lua ran")