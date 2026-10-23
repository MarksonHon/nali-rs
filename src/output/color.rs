use colored::Colorize;

pub fn print_error(msg: &str) {
    eprintln!("{} {}", "错误:".red().bold(), msg);
}

pub fn print_warning(msg: &str) {
    eprintln!("{} {}", "警告:".yellow().bold(), msg);
}

pub fn print_info(msg: &str) {
    println!("{} {}", "信息:".blue().bold(), msg);
}

pub fn print_success(msg: &str) {
    println!("{} {}", "成功:".green().bold(), msg);
}

pub fn print_title(msg: &str) {
    println!();
    println!("{}", msg.bright_cyan().bold());
    println!("{}", "-".repeat(40).bright_cyan());
}
