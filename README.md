# lua-bundle
A lua file concatter cli tool

# Usage
```sh
mkdir project
cd project

echo '
[[project]]
name = "project"
files = ["src"]
entry_point = "src/main.lua"' > build.toml

lua-bundle
```

# TO-DO
- [ ] `include_string(file_path)` - include a string from a file in lua code
- [ ] `lua-bundle new` - subcommand to start a simple project
- [ ] `file filtering` - filter out files with a blacklist, and have an extension whitelist 
- [ ] `lua-version file precedence` - prioritize file extensions from lua-version e.g. `luau(main.luau > main.lua)` 
- [ ] `tiny compression` - POTENTIAL: remove whitespace and rename variables to `r0..`

# Examples
[examples](https://github.com/lua-tools/lua-bundle/blob/master/tool-examples)
