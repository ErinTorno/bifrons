Registry = Registry or {}

Registry.script_path = "systems/registry.lua"

Registry.register_campaign = function(schema)
    Message.new("register_campaign")
        :to_script(Registry.script_path)
        :attach(schema)
        :send()
end