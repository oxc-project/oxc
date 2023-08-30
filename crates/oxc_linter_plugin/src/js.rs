use std::{collections::BTreeMap, sync::Arc};

use mini_v8::{MiniV8, Value, Values};
use trustfall::FieldValue;

fn to_v8(mv8: &MiniV8, fv: FieldValue) -> Value {
    match fv {
        FieldValue::Null => Value::Null,
        FieldValue::Boolean(b) => Value::Boolean(b),
        FieldValue::Int64(int) => {
            let as_i32: i32 =
                int.try_into().expect("for int64 number from trustfall to fit into i32 for js");
            Value::Number(as_i32.into())
        }
        FieldValue::Uint64(int) => {
            let as_u32: u32 =
                int.try_into().expect("for Uint64 number from trustfall to fit into u32 for js");
            Value::Number(as_u32.into())
        }
        FieldValue::Float64(f64) => Value::Number(f64),
        FieldValue::String(str) => Value::String(mv8.create_string(&str)),
        FieldValue::List(list) => {
            let arr = mv8.create_array();
            for el in &*list {
                arr.push(to_v8(mv8, el.clone()))
                    .expect("to be able to put elements from trustfall list into js list");
            }
            Value::Array(arr)
        }
        _ => unreachable!(),
    }
}

pub fn trustfall_results_to_js_arguments(
    mv8: &MiniV8,
    results: BTreeMap<Arc<str>, FieldValue>,
) -> Values {
    let data = mv8.create_object();

    for (k, v) in results {
        data.set(k.to_string(), to_v8(mv8, v)).unwrap();
    }

    Values::from_vec(vec![Value::Object(data)])
}
