pub mod color;
pub mod compact;
pub mod table;

use crate::config::OutputMode;
use crate::query::QueryResult;

pub fn output_result(result: &QueryResult, mode: &OutputMode) {
    match mode {
        OutputMode::Compact => {
            compact::print_compact(result);
        }
        OutputMode::Table => {
            compact::print_compact(result);
            if !result.results.is_empty() || !result.domains.is_empty() {
                println!();
                table::print_table(result);
            }
        }
        OutputMode::Both => {
            compact::print_compact(result);
            if !result.results.is_empty() || !result.domains.is_empty() {
                println!();
                table::print_table(result);
            }
        }
    }
}

pub fn output_pipe(result: &QueryResult) {
    compact::print_compact(result);
}
