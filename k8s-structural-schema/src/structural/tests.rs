use schemars::schema::RootSchema;

use crate::{structural::StructuralSchemaVisitor, visit::Visitor};

fn check_structural_schema(schema: &[u8], expected: &[u8]) {
    let mut schema: RootSchema = serde_yaml::from_slice(schema).expect("valid schema");
    let expected: RootSchema = serde_yaml::from_slice(expected).expect("valid schema");

    StructuralSchemaVisitor.visit_root_schema(&mut schema).unwrap();
    schema = serde_json::from_value(serde_json::to_value(schema).unwrap()).unwrap();

    assert_eq!(
        schema.schema,
        expected.schema,
        r#"
left:
{},
right:
{}"#,
        serde_yaml::to_string(&schema).unwrap(),
        serde_yaml::to_string(&expected).unwrap()
    );
}

#[test]
fn test_examples() {
    // https://kubernetes.io/docs/tasks/extend-kubernetes/custom-resources/custom-resource-definitions/#specifying-a-structural-schema
    check_structural_schema(
        include_bytes!("./test-data/example-1.yaml"),
        include_bytes!("./test-data/example-1.structural.yaml"),
    );
    check_structural_schema(
        include_bytes!("./test-data/example-2.yaml"),
        include_bytes!("./test-data/example-2.structural.yaml"),
    );
    check_structural_schema(
        include_bytes!("./test-data/example-3.yaml"),
        include_bytes!("./test-data/example-3.structural.yaml"),
    );
    // https://github.com/kube-rs/kube-rs/pull/779#issue-1101981499
    check_structural_schema(
        include_bytes!("./test-data/schemars#84.yaml"),
        include_bytes!("./test-data/schemars#84.structural.yaml"),
    );
}

#[test]
fn test_nullable() {
    check_structural_schema(
        include_bytes!("./test-data/nullable-1.yaml"),
        include_bytes!("./test-data/nullable-1.structural.yaml"),
    );
    check_structural_schema(
        include_bytes!("./test-data/nullable-2.yaml"),
        include_bytes!("./test-data/nullable-2.structural.yaml"),
    );
}
