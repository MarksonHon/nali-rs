pub mod repl;

use crate::config::Config;
use crate::db::DatabaseManager;

pub fn run_interactive(config: &Config, db_manager: &DatabaseManager) {
    repl::run_repl(config, db_manager);
}
