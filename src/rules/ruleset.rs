use emu::emu_engine::{EmuArgs, EmuEffects};

pub trait RuleVerifier {
    fn verify(&self, effects: EmuEffects) -> bool;
}

pub struct TargetRules {
    pub inputs: Vec<EmuArgs>,
    verifier: Box<RuleVerifier>,
}

impl RuleVerifier for TargetRules {
    fn verify(&self, effects: EmuEffects) -> bool {
        return self.verifier.verify(effects);
    }
}

pub struct RuleSet {
    candidates_rules: Vec<TargetRules>,
}

impl IntoIterator for RuleSet {
    type Item = TargetRules;
    type IntoIter = ::std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        return self.candidates_rules.into_iter();
    }
}

impl<'a> IntoIterator for &'a RuleSet {
    type Item = &'a TargetRules;
    type IntoIter = ::std::slice::Iter<'a, TargetRules>;

    fn into_iter(self) -> Self::IntoIter {
        return self.candidates_rules.iter();
    }
}

impl RuleSet {
    pub fn new() -> RuleSet {
        return RuleSet { candidates_rules: vec![] };
    }
}

impl RuleSet {}
