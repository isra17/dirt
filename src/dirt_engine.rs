use emu;
use emu::emu_engine::EmuEngine;
use rules::ruleset::RuleSet;
use rules::target_rules::RuleVerifier;

#[derive(Debug)]
pub enum Error {
    EmuError(emu::Error),
}

pub enum CallingConvention {
    Stdcall,
}

/// DirtEngine is the glue code between the rules, emulation and function list
/// to identify. It is the entry point of DIRT.
pub struct DirtEngine {
    /// Emulation engine initialized for the target binary.
    emu: EmuEngine,
    /// Rules loaded in the current context.
    ruleset: RuleSet,
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
    pub fn new(emu: EmuEngine, ruleset: RuleSet) -> DirtEngine {
        return DirtEngine {
            emu: emu,
            ruleset: ruleset,
        };
    }

    /// Identify a single function.
    pub fn identify_function(&self,
                             target: &TargetInfo)
                             -> Result<Option<FunctionInfo>, Error> {
        // Iterate through each candidate's rules.
        for candidate_rule in &self.ruleset {
            // For each target rules, get a list of the input argument to be
            // emulated and run the unknown function. Check with the rule if the
            // result match its conditions.
            let call_result: Result<Vec<()>, CallError> = candidate_rule.inputs
                .iter()
                .map(|input_args| {
                    return match self.emu.call(target, input_args) {
                        Ok(call_effects) => {
                            if candidate_rule.verify(&call_effects,
                                                     &self.emu.vmstate) {
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
                        name: candidate_rule.name.clone(),
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
        CallingConvention::Stdcall
    }
}
