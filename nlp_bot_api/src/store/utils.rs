use sqlx::{query_builder::QueryBuilder, Sqlite};

pub fn build_in_clause(
    query_builder: &mut QueryBuilder<'_, Sqlite>,
    column: &str,
    values: &[String],
) {
    query_builder.push(" ");
    query_builder.push(column);
    query_builder.push(" IN ");

    query_builder.push("(");
    for (index, value) in values.iter().enumerate() {
        query_builder.push_bind(value.clone());

        if index != values.len() - 1 {
            query_builder.push(",");
        }
    }
    query_builder.push(") ");
}
