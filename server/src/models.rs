use database_macros::Queryable;

#[derive(Queryable, Debug)]
pub struct TestModel {
    pub id: usize,
    pub comments: Option<String>,
    pub test_val: String
}
