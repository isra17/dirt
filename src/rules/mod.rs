pub mod rule;
pub mod lua;

pub use self::rule::Rule;
pub use self::lua::LuaRules as RuleSet;

use std::fs;
use std::path::Path;

pub fn load_all(_: &Path) -> RuleSet {
    let lua = RuleSet::new();
    let _: Vec<&Rule> = Vec::new();
    // List lua rules files in rules folder.
    let paths = fs::read_dir("./candidates").unwrap();
    // Load and parse each rules.
    let filepaths =
        paths.filter(|p| p.as_ref().unwrap().file_type().unwrap().is_file());
    for dir_entry in filepaths {
        let entry_path = dir_entry.unwrap().path();
        let path = entry_path.as_path();

        lua.load(path).expect("Failed to load rules");
    }

    return lua;
}
