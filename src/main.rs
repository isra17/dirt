extern crate dirt;
extern crate pbr;

use pbr::ProgressBar;

use dirt::bin::cppfilt;
use dirt::bin::bin_file;
use dirt::emu;
use dirt::rules;
use dirt::dirt_engine::{DirtEngine, TargetInfo};
use std::path::Path;
use std::env;

pub fn main() {
    let target_opt = env::args().nth(1);
    if target_opt.is_none() {
        println!("Usage: dirt TARGET");
        return;
    }

    let target = target_opt.unwrap();
    let target = Path::new(&target);
    let bin = bin_file::load(target)
        .expect(&format!("Failed to load target: {}",
                         target.to_str().unwrap()));

    // Create the emulation engine.
    let emu = emu::from_elf(target)
        .expect("Failed to create emulator from ELF");
    // Load the ruleset.
    let ruleset = rules::load_all(Path::new("./rules"));
    // Create the DIRT engine.
    let mut dirt = DirtEngine::new(emu, ruleset);

    let funcs = bin.functions();
    println!("Identifying {} functions", funcs.len());
    let mut pb = ProgressBar::new(funcs.len() as u64);

    for func in &funcs {
        let cc = dirt.default_cc();
        match dirt.identify_function(&TargetInfo {
            fva: func.value,
            cc: cc,
        }) {
            Ok(matches) => {
                if matches.len() > 0 {
                    println!("\r\x1b[K{}: matched by {:?}",
                             cppfilt::unmangle(&func.name),
                             matches.iter()
                                 .map(|m| m.name.as_str())
                                 .collect::<Vec<&str>>());
                }
            }
            Err(e) => {
                println!("\r\x1b[K{}: Err({:?})", func.name, e);
            }
        }

        pb.inc();
    }

    pb.finish();
}
