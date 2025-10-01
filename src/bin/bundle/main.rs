use std::path::{Path, PathBuf};

use toml::Table;

const BUILD_FILE: &str = "build.toml";
const DEFAULT_REQUIRE_FUNCTION: &str = "require";

#[derive(Default)]
enum LuaVersion {
    #[default]
    Default,
    Lua51,
    Luau,
    Fennel,
}

struct Project {
    name: String,
    output: PathBuf,
    entry_point: PathBuf,
    files: Vec<PathBuf>,
    lua_version: LuaVersion,
}

struct BuildFile {
    projects: Vec<Project>,
    require_function: String,
}

fn main() {
    let Some(build) = parse_build_file() else {
        return;
    };

    let require_method = build.require_function;

    for project in build.projects {
        let mut output = include_str!("lua.lua").to_string();

        output.push_str("\nlocal files = {");
        for file in project.files {
            let name = path_without_extension(&file).to_str().unwrap().into();
            let content = std::fs::read_to_string(file).unwrap();
            output.push_str(insert_module(name, content, require_method.clone(), 1).as_str());
        }
        output.push_str("\n}\n");

        output.push_str(
            insert_entry_point(
                path_without_extension(&project.entry_point)
                    .to_str()
                    .unwrap()
                    .into(),
            )
            .as_str(),
        );

        std::fs::create_dir_all(&project.output).unwrap();
        std::fs::write(project.output.join(&project.name), output).unwrap();
    }
}

fn parse_build_file() -> Option<BuildFile> {
    if !std::fs::exists(BUILD_FILE).unwrap() {
        eprintln!("error: could not find `build.toml` file");
        return None;
    }

    let build = std::fs::read_to_string(BUILD_FILE).unwrap();
    let table = build.as_str().parse::<Table>().unwrap();

    let projects = match table.get("project") {
        Some(value) => {
            let mut projects = Vec::new();
            let array = value.as_array().unwrap();

            for value in array {
                let table = value.as_table().unwrap();
                let Some(project) = parse_project(table) else {
                    continue;
                };

                projects.push(project);
            }

            projects
        }
        None => {
            eprintln!("error: missing [[project]] filed in build.toml");
            return None;
        }
    };

    Some(BuildFile {
        projects,
        require_function: DEFAULT_REQUIRE_FUNCTION.into(),
    })
}

fn parse_project(table: &Table) -> Option<Project> {
    let name = format!(
        "{}.lua",
        match table.get("name") {
            Some(value) => value.as_str().unwrap(),
            None => "a",
        }
    );

    let output = match table.get("output") {
        Some(value) => value.as_str().unwrap(),
        None => "build",
    }
    .into();

    let entry_point = match table.get("entry_point") {
        Some(value) => {
            let entry = PathBuf::from(value.as_str().unwrap());
            if !entry.exists() {
                eprintln!("error: a project entry contains an invalid file in the `entry_point`");
                return None;
            }
            entry
        }

        None => {
            eprintln!("error: a project entry is missing a `entry_point` file");
            return None;
        }
    };

    let lua_version = match table.get("lua_version") {
        Some(value) => LuaVersion::from(value.as_str().unwrap()),
        None => LuaVersion::default(),
    };

    let files = match table.get("files") {
        Some(value) => {
            let mut files = Vec::new();
            let array = value.as_array().unwrap();
            for value in array {
                let entry = PathBuf::from(value.as_str().unwrap());
                if !entry.exists() {
                    eprintln!(
                        "error: a project entry contains an invalid file in the `files` list"
                    );
                    return None;
                }

                files.extend(files_from_path(&entry));
            }

            files
        }

        None => {
            eprintln!("error: a project entry is missing a `files` list");
            return None;
        }
    };

    Some(Project {
        name,
        output,
        entry_point,
        files,
        lua_version,
    })
}

fn files_from_path(path: &Path) -> Vec<PathBuf> {
    let mut files = Vec::new();

    if path.is_file() {
        files.push(path.into());
    }

    if path.is_dir() {
        for entry in path.read_dir().unwrap() {
            let entry = entry.unwrap().path();
            files.extend(files_from_path(&entry));
        }
    }

    files
}

fn path_without_extension(path: &Path) -> PathBuf {
    let mut path = path.to_path_buf();
    // TODO: use file_prefix
    if let Some(stem) = path.clone().file_stem() {
        path.set_file_name(stem);
    }

    path
}

fn indent_block(block: String, level: usize) -> String {
    let lines: Vec<String> = block
        .lines()
        .map(|line| {
            let indentation = "\t".repeat(level);
            format!("{indentation}{line}")
        })
        .collect();

    lines.join("\n")
}

fn inject_require(code: String, require: String) -> String {
    format!(
        "local {require}, functions, get_require = get_require(functions), nil, nil

{code}"
    )
}

fn insert_module(file: String, code: String, require: String, level: usize) -> String {
    let code = indent_block(inject_require(code, require), 1);
    indent_block(
        format!(
            "
[\"{file}\"] = function(functions)
{code}
end,
"
        ),
        level,
    )
}

fn insert_entry_point(entry_point: String) -> String {
    format!(
        "
functions.new({{
    files = files,
    modules = {{}},
}}):require(\"{entry_point}\")"
    )
}

impl From<&str> for LuaVersion {
    fn from(string: &str) -> Self {
        match string {
            "Lua51" => Self::Lua51,
            "Luau" => Self::Luau,
            "Fennel" => Self::Fennel,
            _ => Self::Default,
        }
    }
}
