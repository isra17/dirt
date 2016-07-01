use emu::emu_engine::EmuEngine;
use rules::ruleset::RuleSet;

#[derive(Debug)]
pub enum DirtError {
}

pub enum CallingConvention {
  Stdcall,
}

/// DirtEngine is the glue code between the rules, emulation and function list to identify. It
/// is the entry point of DIRT.
pub struct DirtEngine {
  /// Emulation engine initialized for the target binary.
  emu: EmuEngine,
  /// Rules loaded in the current context.
  ruleset: RuleSet,
}

/// TargetInfo contains the information about a function to be sent and identified by DIRT.
pub struct TargetInfo {
  pub fva: u64,
  pub cc: CallingConvention,
}

/// Function info contains the information of a succesful identification.
pub struct FunctionInfo {
  pub name: String,
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
                           target: TargetInfo)
                           -> Result<FunctionInfo, DirtError> {
    return Ok(FunctionInfo { name: String::new() });
  }

  /// Helper function, returns the default calling convention for the target plateform.
  pub fn default_cc(&self) -> CallingConvention {
    CallingConvention::Stdcall
  }
}