pub fn standardized_type_name<T: 'static>() -> String {
    process_type_name(std::any::type_name::<T>())
}

pub fn standardized_type_name_of<T: ?Sized>(_: &T) -> String {
    process_type_name(std::any::type_name::<T>())
}

fn process_type_name(type_name: &str) -> String {
    if type_name.starts_with('[') && type_name.contains(';') {
        if let (Some(semicolon_pos), Some(bracket_pos)) =
            (type_name.find(';'), type_name.rfind(']'))
        {
            let element_type = &type_name[1..semicolon_pos].trim();
            let size = &type_name[semicolon_pos..bracket_pos];
            return format!("[{}{}]", process_type_name(element_type), size);
        }
        return type_name.to_string();
    }

    if type_name.starts_with('&') {
        return format!("&{}", process_type_name(&type_name[1..]));
    }

    if type_name.starts_with("*const ") || type_name.starts_with("*mut ") {
        let (pointer_type, pointed_type) = type_name.split_at(6);
        return format!("{} {}", pointer_type, process_type_name(pointed_type));
    }

    if type_name.starts_with("dyn ") {
        return format!("dyn {}", process_base_type(&type_name[4..]));
    }

    if let (Some(generic_start), true) = (type_name.find('<'), type_name.ends_with('>')) {
        let base_type = &type_name[..generic_start];
        let generic_part = &type_name[generic_start + 1..type_name.len() - 1];

        return format!(
            "{}<{}>",
            process_base_type(base_type),
            parse_generics(generic_part)
        );
    }

    process_base_type(type_name)
}

fn parse_generics(generic_str: &str) -> String {
    let mut params = Vec::new();
    let mut bracket_depth = 0;
    let mut current_param_start = 0;

    for (i, c) in generic_str.chars().enumerate() {
        match c {
            '<' => bracket_depth += 1,
            '>' => bracket_depth -= 1,
            ',' if bracket_depth == 0 => {
                params.push(generic_str[current_param_start..i].trim());
                current_param_start = i + 1;
            }
            _ => {}
        }
    }

    let last_param = generic_str[current_param_start..].trim();
    if !last_param.is_empty() {
        params.push(last_param);
    }

    params
        .iter()
        .map(|param| process_type_name(param))
        .collect::<Vec<_>>()
        .join(", ")
}

fn process_base_type(base_type: &str) -> String {
    if base_type == "std::error::Error" {
        return "Error".to_string();
    }

    if base_type.contains("dyn ") {
        if let Some(trait_part) = base_type.split("dyn ").nth(1) {
            return format!("dyn {}", process_base_type(trait_part));
        }
    }

    match base_type {
        "core::fmt::Debug" => return "Debug".to_string(),
        "core::fmt::Display" => return "Display".to_string(),
        "core::any::Any" => return "Any".to_string(),
        _ => {}
    }

    if base_type.starts_with("std::")
        || base_type.starts_with("core::")
        || base_type.starts_with("alloc::")
    {
        if let Some(last_part) = base_type.split("::").last() {
            return last_part.to_string();
        }
    }

    base_type.to_string()
}

#[cfg(test)]
mod tests {

    use std::any::Any;
    use std::collections::{BTreeMap, HashMap};
    use std::fmt::Debug;
    use std::rc::Rc;
    use std::sync::{Arc, Mutex};

    use super::*;

    #[test]
    fn test_std_primitive_types() {
        assert_eq!(standardized_type_name::<i32>(), "i32");
        assert_eq!(standardized_type_name::<bool>(), "bool");
        assert_eq!(standardized_type_name::<f64>(), "f64");
        assert_eq!(standardized_type_name::<char>(), "char");
        assert_eq!(standardized_type_name::<()>(), "()");
    }

    #[test]
    fn test_std_string_types() {
        assert_eq!(standardized_type_name::<&str>(), "&str");
        assert_eq!(standardized_type_name::<String>(), "String");
    }

    #[test]
    fn test_std_collection_types() {
        assert_eq!(standardized_type_name::<Vec<i32>>(), "Vec<i32>");
        assert_eq!(
            standardized_type_name::<HashMap<String, i32>>(),
            "HashMap<String, i32>"
        );
        assert_eq!(standardized_type_name::<[i32; 5]>(), "[i32; 5]");
        assert_eq!(standardized_type_name::<&[i32]>(), "&[i32]");
    }

    #[test]
    fn test_std_option_result_types() {
        assert_eq!(standardized_type_name::<Option<i32>>(), "Option<i32>");
        assert_eq!(
            standardized_type_name::<Result<i32, String>>(),
            "Result<i32, String>"
        );
    }

    #[test]
    fn test_std_smart_pointers() {
        assert_eq!(standardized_type_name::<Box<i32>>(), "Box<i32>");
        assert_eq!(standardized_type_name::<Rc<String>>(), "Rc<String>");
        assert_eq!(standardized_type_name::<Arc<Vec<i32>>>(), "Arc<Vec<i32>>");
    }

    mod my_module {
        pub struct MyStruct<T> {
            pub _data: T,
        }

        pub mod nested {
            pub struct NestedType<T> {
                pub _value: T,
            }
        }
    }

    #[test]
    fn test_custom_types() {
        let custom_type = standardized_type_name::<my_module::MyStruct<i32>>();

        assert!(custom_type.contains("my_module::MyStruct<i32>"));

        let nested_type = standardized_type_name::<my_module::nested::NestedType<String>>();
        assert!(nested_type.contains("my_module::nested::NestedType<String>"));
    }

    #[test]
    fn test_complex_std_types() {
        assert_eq!(
            standardized_type_name::<HashMap<String, Vec<Option<i32>>>>(),
            "HashMap<String, Vec<Option<i32>>>"
        );

        assert_eq!(
            standardized_type_name::<Result<Vec<String>, Box<dyn std::error::Error>>>(),
            "Result<Vec<String>, Box<dyn Error>>"
        );
    }

    #[test]
    fn test_complex_nested_std_types() {
        assert_eq!(
            standardized_type_name::<BTreeMap<String, Vec<HashMap<String, String>>>>(),
            "BTreeMap<String, Vec<HashMap<String, String>>>"
        );

        assert_eq!(
            standardized_type_name::<Option<Result<Vec<HashMap<i32, String>>, Arc<Mutex<i32>>>>>(),
            "Option<Result<Vec<HashMap<i32, String>>, Arc<Mutex<i32>>>>"
        );

        assert_eq!(
            standardized_type_name::<HashMap<String, Vec<i32>>>(),
            "HashMap<String, Vec<i32>>"
        );
    }

    #[test]
    fn test_trait_objects() {
        assert_eq!(standardized_type_name::<&dyn Debug>(), "&dyn Debug");
        assert_eq!(standardized_type_name::<Box<dyn Any>>(), "Box<dyn Any>");
        assert_eq!(
            standardized_type_name::<Box<dyn std::error::Error>>(),
            "Box<dyn Error>"
        );
    }

    #[test]
    fn test_array_types() {
        assert_eq!(standardized_type_name::<[i32; 5]>(), "[i32; 5]");
        assert_eq!(standardized_type_name::<[bool; 10]>(), "[bool; 10]");
        assert_eq!(standardized_type_name::<[String; 3]>(), "[String; 3]");
    }

    #[test]
    fn test_type_format_of_values() {
        let value = 42i32;
        assert_eq!(standardized_type_name_of(&value), "i32");

        let string = String::from("hello");
        assert_eq!(standardized_type_name_of(&string), "String");

        let complex = HashMap::<String, Vec<i32>>::new();
        assert_eq!(
            standardized_type_name_of(&complex),
            "HashMap<String, Vec<i32>>"
        );
    }
}
