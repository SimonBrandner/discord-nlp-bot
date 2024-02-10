fn wrap_in_code_block(content: &str) -> String {
    format!("```\n{}\n```", content)
}

pub fn format_table(table: &str, heading: &str) -> String {
    format!("{}\n{}", heading, wrap_in_code_block(table))
}
