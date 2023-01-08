require "systems/message_api/registry"

function on_init()
    Registry.register_campaign {
        level = Level.load("levels/testing/testing_house"),
    }
    Log.info("content.lua ran")
end