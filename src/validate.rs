use sea_orm::ActiveValue;

pub fn validate_required<T: Into<sea_orm::Value>>(
    name: &'static str,
    val: &ActiveValue<T>,
    result: &mut Vec<String>,
) {
    match val {
        ActiveValue::NotSet => result.push(format!("Field {name} not set but required!")),
        _ => {}
    }
}

pub fn validate_length(
    name: &'static str,
    val: &ActiveValue<String>,
    max_len: usize,
    result: &mut Vec<String>,
) {
    match val {
        ActiveValue::Set(v) | ActiveValue::Unchanged(v) => {
            if v.len() > max_len {
                result.push(format!("Field {name} with value \"{v}\" does not meet length requirement: length is {}, max_length: {max_len}!", v.len()));
            }
        }
        _ => {}
    }
}
