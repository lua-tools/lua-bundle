use std::{
    io::Write,
    path::{Path, PathBuf},
    process::{Command, Stdio},
};

use toml::Table;

const BUILD_FILE: &str = "build.toml";
const DEFAULT_REQUIRE_FUNCTION: &str = "require";

#[derive(Default, PartialEq, Eq)]
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
    let Some(build) = BuildFile::from_workspace() else {
        return;
    };

    for project in build.projects {
        project.build(&build.require_function);
    }
}

impl Project {
    fn build(&self, require_method: &str) {
        let mut output = include_str!("lua.lua").to_string();

        output.push_str("\nlocal files = {");
        for file in &self.files {
            let binding = path_without_extension(file);
            let name = binding.to_str().unwrap();

            let mut content = std::fs::read_to_string(file).unwrap();
            let extension = file.extension().unwrap().to_str().unwrap();

            if extension == "fnl" {
                content = compile_fennel_to_lua(&content);
            }

            output.push_str(insert_module(name, &content, require_method, 1).as_str());
        }
        output.push_str("\n}\n");

        output.push_str(
            insert_entry_point(
                path_without_extension(&self.entry_point)
                    .to_str()
                    .unwrap()
                    .into(),
            )
            .as_str(),
        );

        std::fs::create_dir_all(&self.output).unwrap();
        std::fs::write(self.output.join(&self.name), output).unwrap();
    }
}

impl BuildFile {
    fn from_workspace() -> Option<BuildFile> {
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
                eprintln!("error: missing [[project]] field in build.toml");
                return None;
            }
        };

        Some(BuildFile {
            projects,
            require_function: DEFAULT_REQUIRE_FUNCTION.into(),
        })
    }
}

fn compile_fennel_to_lua(source: &str) -> String {
    let mut fennel = Command::new("fennel")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .arg("--compile")
        .arg("-")
        .spawn()
        .expect("error: failed to launch fennel");

    write!(fennel.stdin.as_mut().unwrap(), "{}", source).unwrap();
    String::from_utf8_lossy(&fennel.wait_with_output().unwrap().stdout).to_string()
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

fn inject_require(code: &str, require: &str) -> String {
    format!(
        "local {require}, functions, get_require = get_require(functions), nil, nil

{code}"
    )
}

fn insert_module(file: &str, code: &str, require: &str, level: usize) -> String {
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
