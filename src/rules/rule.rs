use emu::emu_engine::EmuEffects;
use emu::args::EmuArgs;

pub trait Rule {
    fn name<'a>(&'a self) -> &'a str;
    fn args<'a>(&'a self) -> &'a EmuArgs;
    fn verify(&self, result: &EmuEffects) -> bool;
}
