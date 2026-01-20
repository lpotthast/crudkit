use serde::{Deserialize, Serialize};
use std::borrow::Cow;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct ResourceName(Cow<'static, str>);

impl ResourceName {
    pub fn new(name: impl Into<Cow<'static, str>>) -> Self {
        Self(name.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl<T: Into<Cow<'static, str>>> From<T> for ResourceName {
    fn from(value: T) -> Self {
        Self(value.into())
    }
}

impl std::fmt::Display for ResourceName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[cfg(test)]
mod tests {
    use crate::ResourceName;
    use assertr::prelude::*;

    #[test]
    fn serialize_and_deserialize_serializable_id() {
        let resource_name = ResourceName::new("foo");

        let json = serde_json::to_string(&resource_name).unwrap();

        assert_that(&json).is_equal_to(r#""foo""#);

        let deserialized: ResourceName = serde_json::from_str(json.as_str()).unwrap();

        assert_that(deserialized).is_equal_to(resource_name);
    }
}
