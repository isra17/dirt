extern crate dirt;

use dirt::{emu, rules};
use dirt::bin::bin_file;
use dirt::dirt_engine::{DirtEngine, TargetInfo};
use std::env;
use std::path::Path;

struct Candidate(String, u64);

#[test]
fn tests_all_candidates() {
    use std::fs;
    let mut any_failed = false;

    // List candidates in candidates folder.
    let paths = fs::read_dir("./candidates").unwrap();

    // Load and parse each candidates.
    let filepaths =
        paths.filter(|p| p.as_ref().unwrap().file_type().unwrap().is_file());
    for dir_entry in filepaths {
        let dir_path = dir_entry.unwrap().path();
        let path = dir_path.as_path();
        println!("Testing {:?}", path);
        // Load and parse the candidate to extract the functions list to test.
        let candidate = bin_file::load(path).unwrap();

        let tests_iter = candidate.objects()
            .into_iter()
            .filter(|o| {
                o.name.starts_with("test_") && !o.name.ends_with("_fn")
            })
            .map(|o| {
                let test_name = candidate.read_str(o.value).unwrap();
                Candidate(test_name.clone(),
                          candidate.get_symbol(&format!("{}{}",
                                                   &o.name[..o.name.len() -
                                                             6],
                                                   "fn"))
                              .unwrap_or_else(|| {
                                  candidate.get_symbol(&test_name).unwrap()
                              })
                              .value)
            });

        // Create the emulation engine.
        let emu = emu::from_elf(path)
            .expect("Failed to create emulator from ELF");
        // Load the ruleset.
        let ruleset = rules::load_all(Path::new("./rules"));
        // Create the DIRT engine.
        let mut dirt = DirtEngine::new(emu, ruleset);

        // Iterate through all test_ symbols and run the tested function
        // against the DIRT engine.
        let tests_iter: Vec<Candidate> = match env::var("FILTER") {
            Ok(name) => {
                tests_iter.filter(|&Candidate(ref fn_name, _)| fn_name == &name)
                    .collect()
            }
            _ => tests_iter.collect(),
        };
        let results: Vec<bool> = tests_iter.iter()
            .map(|&Candidate(ref fn_name, fva)| {
                let cc = dirt.default_cc();
                match dirt.identify_function(&TargetInfo { fva: fva, cc: cc }) {
                    Ok(matches) => {
                        if matches.len() == 0 {
                            println!("{}: No match", fn_name);
                            return false;
                        }
                        if matches.len() == 1 && &matches[0].name == fn_name {
                            println!("{}: Ok!", fn_name);
                            return true;
                        } else {
                            println!("{}: Overmatched by {:?}",
                                     fn_name,
                                     matches.iter()
                                         .filter(|m| &m.name != fn_name)
                                         .map(|m| m.name.as_str())
                                         .collect::<Vec<&str>>());
                            return false;
                        }
                    }
                    Err(e) => {
                        println!("{}: Err({:?})", fn_name, e);
                        return false;
                    }
                }
            })
            .collect();
        any_failed = any_failed || results.iter().any(|&x| !x);
        println!("{} Emulation Call Done", dirt.emu().emu_counter())
    }
    assert!(!any_failed, "One or more match failed.");
}
