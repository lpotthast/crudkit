use serde::{Deserialize, Serialize};
use std::ops::Deref;
use std::vec;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Severity {
    // TODO: Information?
    Major,
    Critical,
}

#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize)]
pub enum Violation {
    // TODO: Information(String)?
    Major(String),
    Critical(String),
}

impl Violation {
    pub fn major(violation: impl Into<String>) -> Violation {
        Violation::Major(violation.into())
    }

    pub fn critical(violation: impl Into<String>) -> Violation {
        Violation::Critical(violation.into())
    }

    pub fn severity(&self) -> Severity {
        match self {
            Violation::Major(_) => Severity::Major,
            Violation::Critical(_) => Severity::Critical,
        }
    }

    pub fn is_of_severity(&self, severity: Severity) -> bool {
        self.severity() == severity
    }

    pub fn message(&self) -> &str {
        match self {
            Violation::Major(msg) => msg.as_str(),
            Violation::Critical(msg) => msg.as_str(),
        }
    }
    pub fn into_message(self) -> String {
        match self {
            Violation::Major(msg) => msg,
            Violation::Critical(msg) => msg,
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize)]
pub struct Violations {
    pub violations: Vec<Violation>,
}

impl Violations {
    pub fn empty() -> Self {
        Self {
            violations: Vec::new(),
        }
    }

    pub fn len(&self) -> usize {
        self.violations.len()
    }

    pub fn is_empty(&self) -> bool {
        self.violations.is_empty()
    }

    pub fn push(&mut self, violation: Violation) {
        self.violations.push(violation);
    }

    pub fn reserve(&mut self, additional: usize) {
        self.violations.reserve(additional);
    }

    pub fn extend(&mut self, violation: Violations) {
        self.violations.extend(violation);
    }

    pub fn drop_critical(&mut self) {
        self.violations
            .retain(|violation| violation.severity() != Severity::Critical);
    }

    pub fn has_any_violations_of(&self, severity: Severity) -> bool {
        for violation in &self.violations {
            if violation.is_of_severity(severity) {
                return true;
            }
        }
        false
    }

    pub fn has_critical_violations(&self) -> bool {
        self.has_any_violations_of(Severity::Critical)
    }
}

impl Default for Violations {
    fn default() -> Self {
        Self::empty()
    }
}

impl Deref for Violations {
    type Target = [Violation];

    fn deref(&self) -> &Self::Target {
        self.violations.as_slice()
    }
}

impl IntoIterator for Violations {
    type Item = Violation;
    type IntoIter = vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.violations.into_iter()
    }
}
