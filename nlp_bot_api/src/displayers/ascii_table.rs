use crate::processor::ngram::NgramForByCountCommand;
use ascii_table::AsciiTable;

pub fn display_ngram_list(ngrams: &[NgramForByCountCommand]) -> String {
    let mut table = AsciiTable::default();

    table.column(0).set_header("N-gram");
    table.column(1).set_header("Count");

    let data: Vec<Vec<String>> = ngrams
        .iter()
        .map(|ngram: &NgramForByCountCommand| {
            vec![ngram.content.clone(), ngram.occurrence_count.to_string()]
        })
        .collect();

    table.format(data)
}
