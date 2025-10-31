use crate::error::invalid_input;
use crate::key::StatePath;
use greentic_types::GResult;
use serde_json::{Map, Value};

/// Retrieves a nested value at the provided `StatePath`.
pub fn get_at_path<'a>(value: &'a Value, path: &StatePath) -> Option<&'a Value> {
    if path.segments.is_empty() {
        return Some(value);
    }

    let mut current = value;
    for segment in &path.segments {
        match current {
            Value::Object(map) => {
                current = map.get(segment)?;
            }
            Value::Array(items) => {
                let index = parse_index(segment)?;
                current = items.get(index)?;
            }
            _ => return None,
        }
    }
    Some(current)
}

/// Upserts `new_value` at the provided `StatePath`.
pub fn set_at_path(target: &mut Value, path: &StatePath, new_value: Value) -> GResult<()> {
    if path.segments.is_empty() {
        *target = new_value;
        return Ok(());
    }

    let mut current = target;
    let last = path.segments.len() - 1;
    for (idx, segment) in path.segments.iter().enumerate() {
        if matches!(current, Value::Null) {
            *current = container_for_segment(segment);
        }

        match current {
            Value::Object(map) => {
                if idx == last {
                    map.insert(segment.clone(), new_value);
                    return Ok(());
                }
                let next = map.entry(segment.clone()).or_insert_with(|| Value::Null);
                current = next;
            }
            Value::Array(items) => {
                let index = parse_index(segment).ok_or_else(|| {
                    invalid_input(format!("array index expected for segment `{segment}`"))
                })?;
                if idx == last {
                    ensure_len(items, index);
                    items[index] = new_value;
                    return Ok(());
                }
                ensure_len(items, index);
                current = &mut items[index];
            }
            _ => {
                return Err(invalid_input(format!(
                    "segment `{segment}` cannot be applied to non-container value"
                )))
            }
        }
    }

    Ok(())
}

fn parse_index(segment: &str) -> Option<usize> {
    segment.parse::<usize>().ok()
}

fn ensure_len(items: &mut Vec<Value>, index: usize) {
    if index >= items.len() {
        items.resize_with(index + 1, || Value::Null);
    }
}

fn container_for_segment(segment: &str) -> Value {
    if parse_index(segment).is_some() {
        Value::Array(Vec::new())
    } else {
        Value::Object(Map::new())
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::expect_used, clippy::unwrap_used)]
    use super::*;
    use serde_json::json;

    #[test]
    fn root_retrieves_original_value() {
        let value = json!({"a": 1});
        let path = StatePath::default();
        let extracted = get_at_path(&value, &path).unwrap();
        assert_eq!(extracted, &json!({"a": 1}));
    }

    #[test]
    fn nested_insert_and_get() {
        let mut value = Value::Null;
        let path = StatePath::from_pointer("/a/0/b");
        set_at_path(&mut value, &path, json!("leaf")).expect("set");
        let extracted = get_at_path(&value, &path).unwrap();
        assert_eq!(extracted, &json!("leaf"));
    }

    #[test]
    fn invalid_array_index_errors() {
        let mut value = Value::Array(Vec::new());
        let path = StatePath::from_pointer("/foo");
        let err = set_at_path(&mut value, &path, json!("leaf")).unwrap_err();
        assert_eq!(err.code, greentic_types::ErrorCode::InvalidInput);
    }
}
