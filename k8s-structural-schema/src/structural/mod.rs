mod error;
#[cfg(test)]
mod tests;

use schemars::{
    schema::{
        ArrayValidation, InstanceType, ObjectValidation, RootSchema, Schema, SchemaObject,
        SingleOrVec,
    },
    Map,
};
use serde_json::Value;

use crate::{
    ext::NULLABLE,
    visit::{visit_box, visit_root_schema, visit_schema_object, visit_vec, Visitor},
};

pub use self::error::Error;

// TODO: add path info feature
// Reference:
//  - https://kubernetes.io/docs/tasks/extend-kubernetes/custom-resources/custom-resource-definitions/#specifying-a-structural-schema
//  - https://github.com/kubernetes/kubernetes/blob/5fdbfbcd4a750b8435d50d04b4cb8b1d9344eb7c/staging/src/k8s.io/apiextensions-apiserver/pkg/apis/apiextensions/validation/validation.go
//  - https://github.com/kubernetes/kubernetes/blob/5fdbfbcd4a750b8435d50d04b4cb8b1d9344eb7c/staging/src/k8s.io/apiextensions-apiserver/pkg/apiserver/schema/validation.go
//  - https://github.com/kubernetes/kubernetes/blob/5fdbfbcd4a750b8435d50d04b4cb8b1d9344eb7c/staging/src/k8s.io/apiextensions-apiserver/pkg/apiserver/schema/complete.go
#[derive(Clone, Debug)]
pub struct StructuralSchemaVisitor;

impl Visitor for StructuralSchemaVisitor {
    type Error = Error;

    fn visit_root_schema(&mut self, root: &mut RootSchema) -> Result<(), Self::Error> {
        // rule 4: if metadata is specified, then only restrictions on metadata.name and
        // metadata.generateName are allowed.
        if let Some(ref mut object) = root.schema.object {
            if let Some(Schema::Object(SchemaObject { object: Some(metadata_object), .. })) =
                object.properties.remove("metadata")
            {
                // must not specify anything other than name and generateName
                let properties = metadata_object
                    .properties
                    .into_iter()
                    .filter_map(|(key, mut value)| {
                        if key == "name" || key == "generateName" {
                            // default must not be set
                            if let Schema::Object(SchemaObject {
                                metadata: Some(ref mut metadata),
                                ..
                            }) = value
                            {
                                metadata.default = None;
                            }

                            Some((key, value))
                        } else {
                            None
                        }
                    })
                    .collect();

                object.properties.insert(
                    "metadata".to_string(),
                    Schema::Object(SchemaObject {
                        instance_type: Some(SingleOrVec::from(InstanceType::Object)),
                        object: Some(Box::new(ObjectValidation {
                            properties,
                            ..ObjectValidation::default()
                        })),
                        ..SchemaObject::default()
                    }),
                );
            }
        }

        visit_root_schema(self, root)?;

        Ok(())
    }

    fn visit_schema_object(&mut self, schema: &mut SchemaObject) -> Result<(), Error> {
        visit_schema_object(self, schema)?;

        if let Some(ref mut sub) = schema.subschemas {
            let mut subschema_visitor = SubschemaVisitor {
                parent_type: &mut schema.instance_type,
                parent_array: &mut schema.array,
                parent_object: &mut schema.object,
                parent_extensions: &mut schema.extensions,
            };
            visit_vec(&mut subschema_visitor, &mut sub.all_of)?;
            visit_vec(&mut subschema_visitor, &mut sub.any_of)?;
            visit_vec(&mut subschema_visitor, &mut sub.one_of)?;
            visit_box(&mut subschema_visitor, &mut sub.not)?;
        }

        Ok(())
    }
}

struct SubschemaVisitor<'a> {
    parent_type: &'a mut Option<SingleOrVec<InstanceType>>,
    parent_array: &'a mut Option<Box<ArrayValidation>>,
    parent_object: &'a mut Option<Box<ObjectValidation>>,
    parent_extensions: &'a mut Map<String, Value>,
}

impl<'a> SubschemaVisitor<'a> {
    fn new(schema: &'a mut SchemaObject) -> Self {
        Self {
            parent_type: &mut schema.instance_type,
            parent_array: &mut schema.array,
            parent_object: &mut schema.object,
            parent_extensions: &mut schema.extensions,
        }
    }
}

impl Visitor for SubschemaVisitor<'_> {
    type Error = Error;

    fn visit_schema_object(&mut self, schema: &mut SchemaObject) -> Result<(), Error> {
        // visit nested
        if let Some(ref mut sub) = schema.subschemas {
            visit_vec(self, &mut sub.all_of)?;
            visit_vec(self, &mut sub.any_of)?;
            visit_vec(self, &mut sub.one_of)?;
            visit_box(self, &mut sub.not)?;
        }

        // visit array items
        if let Some(ref mut array) = schema.array {
            match array.items {
                Some(SingleOrVec::Single(ref mut item)) => {
                    let parent_array_items = self
                        .parent_array
                        .get_or_insert_with(Box::default)
                        .items
                        .get_or_insert_with(|| {
                            SingleOrVec::from(Schema::Object(SchemaObject::default()))
                        });

                    if let SingleOrVec::Single(schema) = parent_array_items {
                        if let Schema::Object(parent) = schema.as_mut() {
                            SubschemaVisitor::new(parent).visit_schema(item)?;
                        }
                    } else {
                        return self::error::InvalidCustomResourceDefinitionSnafu {
                            reason: "`items` must be a schema object and not an array",
                        }
                        .fail();
                    }
                }
                Some(_) => {
                    return self::error::InvalidCustomResourceDefinitionSnafu {
                        reason: "`items` must be a schema object and not an array",
                    }
                    .fail();
                }
                _ => (),
            };
        }

        // visit object properties
        if let Some(ref mut object) = schema.object {
            let parent_object = self.parent_object.get_or_insert_with(Box::default);

            for (name, property) in &mut object.properties {
                let schema = parent_object
                    .properties
                    .entry(name.to_string())
                    .or_insert_with(|| Schema::Object(SchemaObject::default()));

                if let Schema::Object(ref mut parent) = schema {
                    SubschemaVisitor::new(parent).visit_schema(property)?;
                } else {
                    return self::error::InvalidCustomResourceDefinitionSnafu {
                        reason: "value in `properties` must be a schema object and not an bool",
                    }
                    .fail();
                }
            }
        }

        // move type to parent
        match (schema.instance_type.take(), &mut self.parent_type) {
            (Some(SingleOrVec::Vec(_)), _) | (_, Some(SingleOrVec::Vec(_))) => {
                return self::error::InvalidCustomResourceDefinitionSnafu {
                    reason: "`type` must be a type and not an array",
                }
                .fail();
            }
            (
                Some(SingleOrVec::Single(ref instance_type)),
                Some(SingleOrVec::Single(parent_type)),
            ) if instance_type != parent_type => {
                return self::error::InvalidCustomResourceDefinitionSnafu {
                    reason: "`type` must be same as parent",
                }
                .fail();
            }
            (Some(instance_type), parent_type @ None) => {
                **parent_type = Some(instance_type);
            }
            _ => (),
        }

        // set to parent if nullable
        if let Some((key, Value::Bool(true))) = schema.extensions.remove_entry(NULLABLE) {
            self.parent_extensions.insert(key, Value::Bool(true));
        }

        // TODO: move additional properties to parent

        // TODO: move to parent if this is the only subschema
        if let Some(ref mut metadata) = schema.metadata {
            metadata.default = None;
            metadata.title = None;
            metadata.description = None;
        }

        Ok(())
    }
}
