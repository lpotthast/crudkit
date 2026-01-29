use crudkit_rs::crudkit_validation::violation::Severity;
use sea_orm::{DeriveActiveEnum, EnumIter};
use serde::{Deserialize, Serialize};

pub mod unified;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, EnumIter, DeriveActiveEnum)]
#[sea_orm(rs_type = "String", db_type = "String(Some(16))")]
pub enum PersistedViolationSeverity {
    #[sea_orm(string_value = "MAJOR")]
    Major,

    #[sea_orm(string_value = "CRITICAL")]
    Critical,
}

impl From<Severity> for PersistedViolationSeverity {
    fn from(value: Severity) -> Self {
        match value {
            Severity::Major => PersistedViolationSeverity::Major,
            Severity::Critical => PersistedViolationSeverity::Critical,
        }
    }
}
