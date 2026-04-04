# lua-bundle
A lua file concatter cli tool

# Usage
```sh
lua-bundle new project

cd project
lua-bundle build
```

# TO-DO
- [ ] `include_string(file_path)` - include a string from a file in lua code
- [x] `lua-bundle new` - subcommand to start a simple project
- [ ] `file filtering` - filter out files with a blacklist, and have an extension whitelist 
- [ ] `lua-version file precedence` - prioritize file extensions from lua-version e.g. `luau(main.luau > main.lua)` 

# Examples
[examples](https://github.com/lua-tools/lua-bundle/blob/master/tool-examples)
