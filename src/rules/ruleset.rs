use rules::target_rules::TargetRules;

pub struct RuleSet {
    pub candidates_rules: Vec<TargetRules>,
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
