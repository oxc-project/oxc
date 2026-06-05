use schemars::{
    r#gen::SchemaGenerator,
    schema::{InstanceType, ObjectValidation, Schema, SchemaObject, SubschemaValidation},
};

/// Generates a JSON schema for an object where the keys are strings and the values are either strings or null.
/// TS type equivalent: `Record<string, string | null>`
pub fn object_with_nullable_string_schema(_gen: &mut SchemaGenerator) -> Schema {
    let value_schema = SchemaObject {
        subschemas: Some(Box::new(SubschemaValidation {
            any_of: Some(vec![
                SchemaObject {
                    instance_type: Some(InstanceType::String.into()),
                    ..Default::default()
                }
                .into(),
                SchemaObject {
                    instance_type: Some(InstanceType::Null.into()),
                    ..Default::default()
                }
                .into(),
            ]),
            ..Default::default()
        })),
        ..Default::default()
    };

    SchemaObject {
        instance_type: Some(InstanceType::Object.into()),
        object: Some(Box::new(ObjectValidation {
            additional_properties: Some(Box::new(value_schema.into())),
            ..Default::default()
        })),
        ..Default::default()
    }
    .into()
}
