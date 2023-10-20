use database_macros::Queryable;

#[derive(Queryable)]
struct TestModel {
    id: usize,
    comments: Option<String>,
    test_val: String
}
