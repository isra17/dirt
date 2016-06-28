mod bin_file;

#[test]
fn tests_all_candidates() {
    use std::fs;

    // List candidates in candidates folder.
    let paths = fs::read_dir("./candidates").unwrap();

    // Load and parse each candidates.
    let filepaths = paths.filter(|p| p.as_ref().unwrap().file_type().unwrap().is_file());
    for path in filepaths {
        let candidate = bin_file::load(path.unwrap().path().as_path()).unwrap();

        let tests_iter = candidate.objects()
            .into_iter()
            .filter(|o| o.name.starts_with("test_"))
            .map(|o| candidate.read_str(o.value));

        // Iterate through all test_ symbols and run the tested function
        // against the DIRT engine.
        for test in tests_iter {
            let fn_name = test.unwrap();
            println!("{}: 0x{:x}",
                     fn_name,
                     candidate.get_symbol(fn_name.as_str()).unwrap().value);
        }
    }
}
