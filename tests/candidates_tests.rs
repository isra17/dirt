extern crate dirt;
use dirt::{emu, rules};
use dirt::dirt_engine::{DirtEngine, TargetInfo};
mod bin_file;

#[test]
fn tests_all_candidates() {
    use std::fs;

    // List candidates in candidates folder.
    let paths = fs::read_dir("./candidates").unwrap();

    // Load and parse each candidates.
    let filepaths =
        paths.filter(|p| p.as_ref().unwrap().file_type().unwrap().is_file());
    for dir_entry in filepaths {
        let dir_path = dir_entry.unwrap().path();
        let path = dir_path.as_path();
        // Load and parse the candidate to extract the functions list to test.
        let candidate = bin_file::load(path).unwrap();

        let tests_iter = candidate.objects()
            .into_iter()
            .filter(|o| o.name.starts_with("test_"))
            .map(|o| candidate.read_str(o.value));

        // Create the emulation engine.
        let emu = emu::from_elf(path)
            .expect("Failed to create emulator from ELF");
        // Load the ruleset.
        let ruleset = rules::fixtures();
        // Create en DIRT engine.
        let dirt = DirtEngine::new(emu, ruleset);

        // Iterate through all test_ symbols and run the tested function
        // against the DIRT engine.
        for test in tests_iter {
            let fn_name = test.unwrap();
            let fva = candidate.get_symbol(fn_name.as_str()).unwrap().value;

            match dirt.identify_function(&TargetInfo {
                fva: fva,
                cc: dirt.default_cc(),
            }) {
                Ok(Some(func_info)) => assert_eq!(func_info.name, fn_name),
                Ok(None) => assert!(false, "Function not matched"),
                Err(e) => assert!(false, e),
            }
        }
    }
}
