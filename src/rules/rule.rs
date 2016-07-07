use emu::emu_engine::EmuEffects;
use emu::args::EmuArgs;

pub type Verifier = Fn(&EmuEffects) -> bool;

pub struct Rule {
    pub name: String,
    pub args: EmuArgs,
    pub verifier: Box<Verifier>,
}
