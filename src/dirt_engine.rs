use emu;
use emu::emu_engine::EmuEngine;
use emu::debugger::Debugger;
use rules::Rule;
use rules::RuleSet;

#[derive(Debug)]
pub enum Error {
    EmuError(emu::Error),
}

impl ::std::convert::From<emu::Error> for Error {
    fn from(e: emu::Error) -> Error {
        return Error::EmuError(e);
    }
}

pub enum CallingConvention {
    Stdcall,
    SystemV,
}

/// DirtEngine is the glue code between the rules, emulation and function list
/// to identify. It is the entry point of DIRT.
pub struct DirtEngine {
    /// Emulation engine initialized for the target binary.
    emu: EmuEngine,
    /// Rules loaded in the current context.
    ruleset: Box<RuleSet>,
    debugger: Debugger,
}

/// TargetInfo contains the information about a function to be sent and
/// identified by DIRT.
pub struct TargetInfo {
    pub fva: u64,
    pub cc: CallingConvention,
}

/// Function info contains the information of a succesful identification.
pub struct FunctionInfo {
    pub name: String,
}

enum CallError {
    /// Error that happened while emulating an unknown function. Expected to
    /// happen often and likely to be a lead for negative.
    NotMatched,
    /// Error that happened while setting up the emulation engine. Must be
    /// handled.
    EmuError(emu::Error),
}

impl DirtEngine {
    /// Create a new DirtEngine given an emulation engine and ruleset.
    pub fn new(emu: EmuEngine, ruleset: Box<RuleSet>) -> DirtEngine {
        let debugger_engine = emu.vmstate.engine.clone();
        return DirtEngine {
            emu: emu,
            ruleset: ruleset,
            debugger: Debugger::new(debugger_engine),
        };
    }

    /// Identify a single function.
    pub fn identify_function(&mut self,
                             target: &TargetInfo)
                             -> Result<Option<FunctionInfo>, Error> {
        let debugger = &mut self.debugger;
        let emu = &self.emu;
        // Iterate through each candidate's rules.
        for (candidate_name, rules) in self.ruleset.candidates() {
            // For each target rules, get a list of the input argument to be
            // emulated and run the unknown function. Check with the rule if the
            // result match its conditions.
            let call_result: Result<Vec<()>, CallError> = rules.iter()
                .map(|rule| {
                    println!("Calling {}({:?})", rule.name, rule.args);
                    debugger.detach().expect("Failed to detach debugger");
                    if rule.name == "std::string::string(char*)" && false {
                        debugger.attach().expect("Failed to attach debugger");
                    }
                    return match emu.call(target, &rule.args) {
                        Ok(call_effects) => {
                            if rule.verify(&call_effects) {
                                Ok(())
                            } else {
                                Err(CallError::NotMatched)
                            }
                        }
                        Err(emu::Error::ExecError(e)) => {
                            println!("ExecError: {:?}", e);
                            Err(CallError::NotMatched)
                        }
                        Err(e) => Err(CallError::EmuError(e)),
                    };
                })
                .collect();

            match call_result {
                Ok(_) => {
                    return Ok(Some(FunctionInfo {
                        name: candidate_name.clone(),
                    }))
                }
                Err(CallError::NotMatched) => (),
                Err(CallError::EmuError(e)) => return Err(Error::EmuError(e)),
            };
        }
        return Ok(None);
    }

    /// Helper function, returns the default calling convention for the target
    /// plateform.
    pub fn default_cc(&self) -> CallingConvention {
        CallingConvention::SystemV
    }
}
