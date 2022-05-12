// Reference:
// https://github.com/GREsau/schemars/issues/128

use schemars::{
    schema::{RootSchema, Schema, SchemaObject, SingleOrVec},
    Map,
};

/// Trait used to recursively modify a constructed schema and its subschemas.
pub trait Visitor {
    type Error;

    /// Override this method to modify a [`RootSchema`] and (optionally) its
    /// subschemas.
    ///
    /// When overriding this method, you will usually want to call the
    /// [`visit_root_schema`] function to visit subschemas.
    fn visit_root_schema(&mut self, root: &mut RootSchema) -> Result<(), Self::Error> {
        visit_root_schema(self, root)
    }

    /// Override this method to modify a [`Schema`] and (optionally) its
    /// subschemas.
    ///
    /// When overriding this method, you will usually want to call the
    /// [`visit_schema`] function to visit subschemas.
    fn visit_schema(&mut self, schema: &mut Schema) -> Result<(), Self::Error> {
        visit_schema(self, schema)
    }

    /// Override this method to modify a [`SchemaObject`] and (optionally) its
    /// subschemas.
    ///
    /// When overriding this method, you will usually want to call the
    /// [`visit_schema_object`] function to visit subschemas.
    fn visit_schema_object(&mut self, schema: &mut SchemaObject) -> Result<(), Self::Error> {
        visit_schema_object(self, schema)
    }
}

/// Visits all subschemas of the [`RootSchema`].
pub fn visit_root_schema<V>(v: &mut V, root: &mut RootSchema) -> Result<(), V::Error>
where
    V: Visitor + ?Sized,
{
    v.visit_schema_object(&mut root.schema)?;
    visit_map_values(v, &mut root.definitions)?;

    Ok(())
}

/// Visits all subschemas of the [`Schema`].
pub fn visit_schema<V>(v: &mut V, schema: &mut Schema) -> Result<(), V::Error>
where
    V: Visitor + ?Sized,
{
    if let Schema::Object(schema) = schema {
        v.visit_schema_object(schema)?;
    }

    Ok(())
}

/// Visits all subschemas of the [`SchemaObject`].
pub fn visit_schema_object<V>(v: &mut V, schema: &mut SchemaObject) -> Result<(), V::Error>
where
    V: Visitor + ?Sized,
{
    if let Some(sub) = &mut schema.subschemas {
        visit_vec(v, &mut sub.all_of)?;
        visit_vec(v, &mut sub.any_of)?;
        visit_vec(v, &mut sub.one_of)?;
        visit_box(v, &mut sub.not)?;
        visit_box(v, &mut sub.if_schema)?;
        visit_box(v, &mut sub.then_schema)?;
        visit_box(v, &mut sub.else_schema)?;
    }

    if let Some(arr) = &mut schema.array {
        visit_single_or_vec(v, &mut arr.items)?;
        visit_box(v, &mut arr.additional_items)?;
        visit_box(v, &mut arr.contains)?;
    }

    if let Some(obj) = &mut schema.object {
        visit_map_values(v, &mut obj.properties)?;
        visit_map_values(v, &mut obj.pattern_properties)?;
        visit_box(v, &mut obj.additional_properties)?;
        visit_box(v, &mut obj.property_names)?;
    }

    Ok(())
}

pub fn visit_box<V>(v: &mut V, target: &mut Option<Box<Schema>>) -> Result<(), V::Error>
where
    V: Visitor + ?Sized,
{
    if let Some(s) = target {
        v.visit_schema(s)?;
    }

    Ok(())
}

pub fn visit_vec<V>(v: &mut V, target: &mut Option<Vec<Schema>>) -> Result<(), V::Error>
where
    V: Visitor + ?Sized,
{
    if let Some(vec) = target {
        for s in vec {
            v.visit_schema(s)?;
        }
    }

    Ok(())
}

pub fn visit_map_values<V>(v: &mut V, target: &mut Map<String, Schema>) -> Result<(), V::Error>
where
    V: Visitor + ?Sized,
{
    for s in target.values_mut() {
        v.visit_schema(s)?;
    }

    Ok(())
}

pub fn visit_single_or_vec<V>(
    v: &mut V,
    target: &mut Option<SingleOrVec<Schema>>,
) -> Result<(), V::Error>
where
    V: Visitor + ?Sized,
{
    match target {
        None => {}
        Some(SingleOrVec::Single(s)) => {
            v.visit_schema(s)?;
        }
        Some(SingleOrVec::Vec(vec)) => {
            for s in vec {
                v.visit_schema(s)?;
            }
        }
    }

    Ok(())
}
