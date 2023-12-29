use serde_json::Value;

/**
 * parse config for multiple style
 *
 * {
 *  "rules": {
 *    "jsx-a11y/rule-name": 2
 *  }
 * }
 * or
 * {
 *  "rules": {
 *     "jsx-a11y/rule-name": {
 *         "ignoreNonDOM": true
 *       }
 *    }
 * }
 * or
 *
 * {
 *  "rules": {
 *     "jsx-a11y/rule-name": [{
 *         "ignoreNonDOM": true
 *       }
 *    }]
 * }
 */
pub fn get_rule_config(config: &Value, key: &str) -> Option<Value> {
    if let Some(rule_config) = config.as_object() {
        if let Some(value) = rule_config.get(key) {
            return Some(value.clone());
        }
    }

    if let Some(rule_config_array) = config.as_array() {
        if let Some(obj) = rule_config_array.iter().find(|v| v.get(key).is_some()) {
            if let Some(object_value) = obj.get(key) {
                return Some(object_value.clone());
            }
        }
    }

    None
}
