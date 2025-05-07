use std::{error::Error, fmt::Display};
pub mod type_name;

use serde::{Deserialize, Serialize};
use type_name::standardized_type_name_of;

#[derive(Debug, Serialize, Deserialize, Clone, valuable::Valuable)]
#[serde(rename_all = "camelCase")]
pub struct AnyError {
    #[serde(rename = "$type")]
    pub r#type: String,
    pub context: AnyErrorContext,
}
impl<E: Error + Sized> From<E> for AnyError {
    fn from(value: E) -> Self {
        let r#type = standardized_type_name_of(&value);
        let message = format!("{value}");
        let inner_error = value.source().map(|e| Box::new(AnyError::from(e)));

        Self {
            r#type,
            context: AnyErrorContext {
                message,
                inner_error,
            },
        }
    }
}
impl Display for AnyError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {}", self.r#type, self.context.message)?;
        if let Some(inner_error) = self.context.inner_error.as_ref() {
            write!(f, "({})", inner_error)?;
        }

        Ok(())
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, valuable::Valuable)]
#[serde(rename_all = "camelCase")]
pub struct AnyErrorContext {
    message: String,
    inner_error: Option<Box<AnyError>>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json;
    use std::error::Error as StdError;
    use std::fmt;
    use std::io;

    #[derive(Debug)]
    struct SimpleError {
        message: String,
    }

    impl fmt::Display for SimpleError {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            write!(f, "{}", self.message)
        }
    }

    impl StdError for SimpleError {}

    #[derive(Debug)]
    struct NestedError {
        message: String,
        source: SimpleError,
    }

    impl fmt::Display for NestedError {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            write!(f, "{}", self.message)
        }
    }

    impl StdError for NestedError {
        fn source(&self) -> Option<&(dyn StdError + 'static)> {
            Some(&self.source)
        }
    }

    #[derive(Debug)]
    struct DeepNestedError {
        message: String,
        source: NestedError,
    }

    impl fmt::Display for DeepNestedError {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            write!(f, "{}", self.message)
        }
    }

    impl StdError for DeepNestedError {
        fn source(&self) -> Option<&(dyn StdError + 'static)> {
            Some(&self.source)
        }
    }

    #[test]
    fn test_from_simple_error() {
        let simple_error = SimpleError {
            message: "This is a simple error".to_string(),
        };

        let any_error = AnyError::from(simple_error);

        assert!(
            any_error.r#type.ends_with("SimpleError") || any_error.r#type.contains("::SimpleError")
        );

        assert_eq!(any_error.context.message, "This is a simple error");

        assert!(any_error.context.inner_error.is_none());
    }

    #[test]
    fn test_from_standard_io_error() {
        let io_error = io::Error::new(io::ErrorKind::NotFound, "File not found");

        let any_error = AnyError::from(io_error);

        assert!(any_error.r#type.contains("Error"));

        assert_eq!(any_error.context.message, "File not found");
    }

    #[test]
    fn test_from_nested_error() {
        let inner = SimpleError {
            message: "Inner error".to_string(),
        };

        let nested = NestedError {
            message: "Outer error".to_string(),
            source: inner,
        };

        let any_error = AnyError::from(nested);

        assert!(
            any_error.r#type.ends_with("NestedError") || any_error.r#type.contains("::NestedError")
        );

        assert_eq!(any_error.context.message, "Outer error");

        assert!(any_error.context.inner_error.is_some());

        let inner_error = any_error.context.inner_error.unwrap();

        assert!(
            inner_error.r#type.ends_with("SimpleError")
                || inner_error.r#type.contains("::SimpleError")
                || inner_error.r#type.contains("dyn Error")
                || inner_error.r#type.contains("AnyError")
        );

        assert_eq!(inner_error.context.message, "Inner error");
        assert!(inner_error.context.inner_error.is_none());
    }

    #[test]
    fn test_from_deep_nested_error() {
        let level1 = SimpleError {
            message: "Level 1 error".to_string(),
        };

        let level2 = NestedError {
            message: "Level 2 error".to_string(),
            source: level1,
        };

        let level3 = DeepNestedError {
            message: "Level 3 error".to_string(),
            source: level2,
        };

        let any_error = AnyError::from(level3);

        assert!(
            any_error.r#type.ends_with("DeepNestedError")
                || any_error.r#type.contains("::DeepNestedError")
        );
        assert_eq!(any_error.context.message, "Level 3 error");

        assert!(any_error.context.inner_error.is_some());
        let level2_error = any_error.context.inner_error.as_ref().unwrap();

        assert!(
            level2_error.r#type.ends_with("NestedError")
                || level2_error.r#type.contains("::NestedError")
                || level2_error.r#type.contains("dyn Error")
                || level2_error.r#type.contains("AnyError")
        );
        assert_eq!(level2_error.context.message, "Level 2 error");

        assert!(level2_error.context.inner_error.is_some());
        let level1_error = level2_error.context.inner_error.as_ref().unwrap();

        assert!(
            level1_error.r#type.ends_with("SimpleError")
                || level1_error.r#type.contains("::SimpleError")
                || level1_error.r#type.contains("dyn Error")
                || level1_error.r#type.contains("AnyError")
        );
        assert_eq!(level1_error.context.message, "Level 1 error");
        assert!(level1_error.context.inner_error.is_none());
    }

    #[test]
    fn test_display_simple_error() {
        let simple_error = SimpleError {
            message: "Display test".to_string(),
        };

        let any_error = AnyError::from(simple_error);

        let display_string = format!("{}", any_error);

        assert!(
            display_string.contains("SimpleError")
                || display_string.contains(any_error.r#type.as_str())
        );
        assert!(display_string.contains("Display test"));
        assert!(!display_string.contains("("));
    }

    #[test]
    fn test_display_nested_error() {
        let inner = SimpleError {
            message: "Inner display".to_string(),
        };

        let nested = NestedError {
            message: "Outer display".to_string(),
            source: inner,
        };

        let any_error = AnyError::from(nested);

        let display_string = format!("{}", any_error);

        assert!(
            display_string.contains("NestedError")
                || display_string.contains(any_error.r#type.as_str())
        );

        assert!(display_string.contains("Outer display"));
        assert!(display_string.contains("("));

        assert!(display_string.contains("Inner display"));

        if let Some(inner) = &any_error.context.inner_error {
            assert!(
                display_string.contains(inner.r#type.as_str())
                    || display_string.contains("SimpleError")
                    || display_string.contains("AnyError")
            );
        }
    }

    #[test]
    fn test_serde_serialization() {
        let simple_error = SimpleError {
            message: "Serialization test".to_string(),
        };

        let any_error = AnyError::from(simple_error);

        let json = serde_json::to_string(&any_error).expect("Serialization failed");

        assert!(json.contains("\"$type\""));

        let type_json_fragment = format!("\"$type\":\"{}\"", any_error.r#type);
        assert!(json.contains(&type_json_fragment));

        assert!(json.contains("Serialization test"));
        assert!(json.contains("\"innerError\":null"));
    }

    #[test]
    fn test_serde_deserialization() {
        let json = r#"{
            "$type": "TestError",
            "context": {
                "message": "Test message",
                "innerError": null
            }
        }"#;

        let deserialized: AnyError = serde_json::from_str(json).expect("Deserialization failed");

        assert_eq!(deserialized.r#type, "TestError");
        assert_eq!(deserialized.context.message, "Test message");
        assert!(deserialized.context.inner_error.is_none());
    }

    #[test]
    fn test_serde_nested_deserialization() {
        let json = r#"{
            "$type": "OuterError",
            "context": {
                "message": "Outer message",
                "innerError": {
                    "$type": "InnerError",
                    "context": {
                        "message": "Inner message",
                        "innerError": null
                    }
                }
            }
        }"#;

        let deserialized: AnyError = serde_json::from_str(json).expect("Deserialization failed");

        assert_eq!(deserialized.r#type, "OuterError");
        assert_eq!(deserialized.context.message, "Outer message");

        let inner = deserialized.context.inner_error.unwrap();
        assert_eq!(inner.r#type, "InnerError");
        assert_eq!(inner.context.message, "Inner message");
        assert!(inner.context.inner_error.is_none());
    }

    #[test]
    fn test_clone() {
        let simple_error = SimpleError {
            message: "Clone test".to_string(),
        };

        let any_error = AnyError::from(simple_error);

        let cloned = any_error.clone();

        assert_eq!(cloned.r#type, any_error.r#type);
        assert_eq!(cloned.context.message, any_error.context.message);
        assert!(cloned.context.inner_error.is_none());
    }

    #[test]
    fn test_valuable_trait() {
        let simple_error = SimpleError {
            message: "Valuable test".to_string(),
        };

        let any_error = AnyError::from(simple_error);

        let _ = valuable::Valuable::as_value(&any_error);
    }
}
