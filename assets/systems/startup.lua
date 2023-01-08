Random.true_random_seed()

local level_handle = Level.load("levels/testing/testing_house")

level_handle:on_load(function(handle)
    local level = handle:get()
    level:spawn()
end)