local cartridge = require('cartridge')
local log = require('log')

local function init(opts) -- luacheck: no unused args
    local httpd = assert(cartridge.service_get('httpd'), "Failed to get httpd service")
    httpd:route({method = 'GET', path = '/api/v1/engine'}, function()
        return {body = 'Engine started'}
    end)

    return true
end

local function stop()
    return true
end

local function calculate_something(plan) -- luacheck: no unused args
    checks('table')
    log.info("calculate:\n" .. tostring(plan))
    return true
end

local function get_result() -- luacheck: no unused args
    log.info("calculation result")
    return true
end

return {
    role_name = 'app.roles.engine',
    init = init,
    stop = stop,
    calculate_something = calculate_something,
    get_result = get_result,
}
