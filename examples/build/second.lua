local functions = {}
functions.__index = functions

--@param context table
--@return metatable
function functions.new(context)
	return setmetatable({ context = context }, functions)
end

--@param content string
--@param separator string?
--@return {string}
function functions:string_split(content, separator)
	local result = {}
	for v in string.gmatch(content, "([^" .. separator .. "]+)") do
		table.insert(result, v)
	end

	return result
end

--@param file string
--@return string
function functions:get_file_path(file)
	local result = {}
	for _, v in next, self:string_split(file, '/') do
		if v == ".." then
			table.remove(result)
		elseif v ~= '.' then
			table.insert(result, v)
		end
	end

	return table.concat(result, '/')
end

--@return string
function functions:get_caller_directory()
	local caller = self.context.file_path
	if not caller then return '' end

	local path = self:string_split(caller, '/')
	table.remove(path)

	local directory = table.concat(path, '/')
	if #directory == 0 then return '' end

	return directory .. '/'
end

--@param directory string
--@return string
function functions:remove_trailing_slash(directory)
	if string.sub(directory, 1, -1) == '/' then
		return string.sub(directory, 1, -2)
	end

	return directory
end

--@param module string
--@return (string?, string?)
function functions:get_module_from_global_path(module)
	for _, global in self.context.globals do
		local file = string.format("%s/%s", global, module)

		if self.context.files[file] then return module, file end
		if self.context.files[file .. "/init"] then
			return module, file .. "/init"
		end
	end
end

--@param path string
--@return (string?, string?)
function functions:get_module_from_relative_path(path)
	local file = self:get_file_path(self:get_caller_directory() .. path)
	file = self:remove_trailing_slash(file)

	if self.context.files[file] then return file, file end
	if self.context.files[file .. "/init"] then
		return file .. '/', file .. "/init"
	end
end

--@param module string?
--@return any?
function functions:require(module)
	if type(module) == "string" then
		local _name, file = self:get_module_from_relative_path(module)
		if not file then
			_name, file = self:get_module_from_global_path(module)
		end

		if not file then
			error(string.format("could not load module: %s, module was not found", module))
		end

		if not self.context.modules[file] then
			self.context.modules[file] = self.context.files[file](functions.new({
				file_path = file,
				files = self.context.files,
				globals = self.context.globals,
				modules = self.context.modules	
			}))
		end

		return self.context.modules[file]
	end
end

--@param functions table
--@return function
local function get_require(functions)
	return function(...)
		return functions:require(...)
	end
end

local files = {	
	["src/second"] = function(functions)
		local require, functions, get_require = get_require(functions), nil, nil
		
		print("second script")
	end,
}

functions.new({
    files = files,
    modules = {},
}):require("src/second")