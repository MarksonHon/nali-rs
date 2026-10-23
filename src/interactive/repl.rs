use colored::Colorize;
use rustyline::DefaultEditor;
use rustyline::error::ReadlineError;

use crate::config::Config;
use crate::db::DatabaseManager;
use crate::output;
use crate::query;

pub fn run_repl(config: &Config, db_manager: &DatabaseManager) {
    println!();
    println!(
        "{}",
        "Nalide 交互式查询模式".bright_cyan().bold()
    );
    println!("{}", "输入 IP 地址进行查询，输入 quit/exit 退出".dimmed());
    println!();

    let mut rl = DefaultEditor::new().expect("Failed to initialize readline");

    loop {
        let prompt = ">> ";
        match rl.readline(&prompt) {
            Ok(line) => {
                let line = line.trim();
                if line.is_empty() {
                    continue;
                }

                if line == "quit" || line == "exit" || line == "q" {
                    println!("{}", "再见!".bright_cyan());
                    break;
                }

                if line == "help" || line == "h" {
                    print_help();
                    continue;
                }

                let input = line
                    .trim_start_matches("--all ")
                    .trim_start_matches("-a ");
                let result = query::query_direct(db_manager, input);
                output::output_result(&result, &config.output_mode);
            }
            Err(ReadlineError::Interrupted) => {
                println!("{}", "Ctrl-C，输入 quit 退出".yellow());
            }
            Err(ReadlineError::Eof) => {
                println!("{}", "再见!".bright_cyan());
                break;
            }
            Err(err) => {
                eprintln!("{} {}", "读取错误:".red(), err);
                break;
            }
        }
    }
}

fn print_help() {
    println!();
    println!("{}", "可用命令：".bright_yellow().bold());
    println!("  {} - 查询 IP（GeoLite2 与 geoip 一起显示）", "<IP>".green());
    println!("  {} - 查询域名（geosite 集合）", "<域名>".green());
    println!("  {} - 显示本帮助信息", "help/h".green());
    println!("  {} - 退出程序", "quit/exit/q".green());
    println!();
}
