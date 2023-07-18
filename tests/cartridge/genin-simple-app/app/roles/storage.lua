local checks = require('checks')
local errors = require('errors')
local err_storage = errors.new_class("Storage error")

local function tuple_to_table(format, tuple)
    local map = {}
    for i, v in ipairs(format) do
        map[v.name] = tuple[i]
    end
    return map
end

local function complete_table(major, minor)
    for k, v in pairs(major) do
        if minor[k] == nil then
            minor[k] = v
        end
    end
end

local function init_space()
    local profile = box.schema.space.create(
        'profile',
        {
            format = {
                {'profile_id', 'unsigned'},
                {'bucket_id', 'unsigned'},
                {'first_name', 'string'},
                {'sur_name', 'string'},
                {'patronymic', 'string'},
                {'msgs_count', 'unsigned'},
                {'service_info', 'string'}
            },
            if_not_exists = true,
        }
    )
    profile:create_index('profile_id', {
        parts = {'profile_id'},
        if_not_exists = true,
    })
    profile:create_index('bucket_id', {
        parts = {'bucket_id'},
        unique = false,
        if_not_exists = true,
    })
end

local function profile_add(profile)
    checks('table')

    local exist = box.space.profile:get(profile.profile_id)
    if exist ~= nil then
        return {ok = false, error = err_storage:new("Profile already exist")}
    end

    box.space.profile:insert(box.space.profile:frommap(profile))

    return {ok = true, error = nil}
end

local function profile_update(id, changes)
    checks('number', 'table')
    
    local exists = box.space.profile:get(id)

    if exists == nil then
        return {profile = nil, error = err_storage:new("Profile not found")}
    end

    exists = tuple_to_table(box.space.profile:format(), exists)

    complete_table(exists, changes)
    box.space.profile:replace(box.space.profile:frommap(changes))

    changes.bucket_id = nil
    return {profile = changes, error = nil}
end

local function profile_get(id)
    checks('number')
    
    local profile = box.space.profile:get(id)
    if profile == nil then
        return {profile = nil, error = err_storage:new("Profile not found")}
    end

    profile = tuple_to_table(box.space.profile:format(), profile)
    
    profile.bucket_id = nil
    return {profile = profile, error = nil}
end

local function profile_delete(id)
    checks('number')
    
    local exists = box.space.profile:get(id)
    if exists == nil then
        return {ok = false, error = err_storage:new("Profile not found")}
    end
    exists = tuple_to_table(box.space.profile:format(), exists)

    box.space.profile:delete(id)
    return {ok = true, error = nil}
end

local function init(opts)
    if opts.is_master then
        init_space()

        box.schema.func.create('profile_add', {if_not_exists = true})
        box.schema.func.create('profile_get', {if_not_exists = true})
        box.schema.func.create('profile_update', {if_not_exists = true})
        box.schema.func.create('profile_delete', {if_not_exists = true})
    end

    rawset(_G, 'profile_add', profile_add)
    rawset(_G, 'profile_get', profile_get)
    rawset(_G, 'profile_update', profile_update)
    rawset(_G, 'profile_delete', profile_delete)

    return true
end

return {
    role_name = 'app.roles.storage',
    init = init,
    utils = {
        profile_add = profile_add,
        profile_update = profile_update,
        profile_get = profile_get,
        profile_delete = profile_delete,
    },
    dependencies = {
        'cartridge.roles.vshard-storage'
    }
}
