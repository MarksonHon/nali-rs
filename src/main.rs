mod cli;
mod config;
mod db;
mod dirs;
mod error;
mod interactive;
mod io_text;
mod output;
mod query;

use std::io::{self, Write};

use anyhow::Result;
use clap::Parser;
use colored::Colorize;
use colored::control;

use cli::{Cli, Commands};
use config::Config;
use db::DatabaseManager;
use dirs::NalideDirs;

fn main() -> Result<()> {
    io_text::prepare_console();

    let cli = Cli::parse();

    init_logging(cli.verbose);

    let dirs = if cli.no_config {
        NalideDirs::new()?
    } else {
        NalideDirs::with_overrides(cli.config_dir, cli.data_dir, cli.cache_dir)?
    };

    if let Some(Commands::Dirs {
        open_config,
        open_data,
    }) = &cli.command
    {
        dirs.display();
        if *open_config {
            let _ = open::that(&dirs.config_dir);
        }
        if *open_data {
            let _ = open::that(&dirs.databases_dir);
        }
        return Ok(());
    }

    let config = if cli.no_config {
        Config::default()
    } else {
        Config::load(&dirs)?
    };

    let output_mode = crate::config::OutputMode::from(cli.output);

    let mut db_manager = DatabaseManager::new();

    for db_path in &cli.databases {
        let name = db_path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("unknown");

        let db_type = if db_path
            .extension()
            .and_then(|s| s.to_str())
            .map(|e| e == "mmdb")
            .unwrap_or(false)
        {
            "mmdb"
        } else {
            "v2ray"
        };

        if let Err(e) = db_manager.load_single(name, db_type, db_path) {
            eprintln!(
                "{} {}",
                "加载数据库失败:".red().bold(),
                e
            );
        }
    }

    if db_manager.databases().is_empty() {
        let failed = db_manager.load_all(&config, &dirs);

        if !failed.is_empty() {
            let need_download: Vec<_> = config
                .databases
                .iter()
                .filter(|db| db.enabled && failed.contains(&db.name))
                .cloned()
                .collect();

            if !need_download.is_empty() {
                println!(
                    "{}",
                    "发现缺失或损坏的数据库，正在自动下载...".yellow().bold()
                );

                if let Err(e) = auto_download_databases(&config, &dirs, &need_download) {
                    eprintln!(
                        "{} {}",
                        "自动下载数据库失败:".red().bold(),
                        e
                    );
                }

                db_manager.load_all(&config, &dirs);
            }
        }

        if db_manager.databases().is_empty() {
            eprintln!(
                "{}",
                "没有可用的数据库。请将数据库文件放入数据库目录，或使用 --db 参数指定。"
                    .red()
            );
            eprintln!(
                "数据库目录: {}",
                dirs.databases_dir.display()
            );
            return Ok(());
        }

        if atty_is_tty() {
            println!("{}", "数据库准备就绪！".green().bold());
        }
    }

    match &cli.command {
        Some(Commands::Update { database, force }) => {
            handle_update(&config, &dirs, database.as_deref(), *force)?;
        }
        Some(Commands::Status) => {
            handle_status(&config, &dirs);
        }
        Some(Commands::Dirs { .. }) => unreachable!(),
        None => {}
    }

    if cli.ips.is_empty() && cli.command.is_none() {
        if atty_is_tty() {
            interactive::run_interactive(&config, &db_manager);
        } else {
            process_stdin(&db_manager, &output_mode);
        }
    } else {
        for ip_input in &cli.ips {
            let result = query::query_direct(&db_manager, ip_input);
            output::output_result(&result, &output_mode);
        }
    }

    Ok(())
}

fn init_logging(verbose: bool) {
    let level = if verbose {
        log::LevelFilter::Debug
    } else {
        log::LevelFilter::Warn
    };

    env_logger::Builder::new().filter_level(level).init();
}

fn atty_is_tty() -> bool {
    atty::is(atty::Stream::Stdin)
}

fn process_stdin(db_manager: &DatabaseManager, _output_mode: &crate::config::OutputMode) {
    let decoded = match io_text::read_stdin_text() {
        Ok(decoded) => decoded,
        Err(e) => {
            eprintln!("{} {}", "读取标准输入错误:".red(), e);
            return;
        }
    };

    control::set_override(true);
    let mut stdout = io::stdout().lock();
    for line in decoded.text.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }

        let annotated = query::annotate_line(db_manager, line, true);
        let mut out = annotated.into_bytes();
        out.push(b'\n');
        if stdout.write_all(&out).is_err() || stdout.flush().is_err() {
            break;
        }
    }
}

fn handle_update(config: &Config, dirs: &NalideDirs, database: Option<&str>, _force: bool) -> Result<()> {
    println!("{}", "检查数据库更新...".blue().bold());

    let client = reqwest::blocking::Client::builder()
        .timeout(std::time::Duration::from_secs(30))
        .build()?;

    for db_config in &config.databases {
        if let Some(target_db) = database {
            if db_config.name != target_db {
                continue;
            }
        }

        if let Some(ref update_url) = db_config.update_url {
            println!(
                "{} {}",
                "更新数据库:".blue(),
                db_config.name
            );
            println!("  URL: {}", update_url);

            let db_path = config.resolve_db_path(db_config, dirs);

            match download_file(&client, update_url, &db_path) {
                Ok(_) => {
                    println!(
                        "  {} {}",
                        "✓".green().bold(),
                        "下载完成".green()
                    );
                }
                Err(e) => {
                    eprintln!(
                        "  {} {}",
                        "✗".red().bold(),
                        format!("下载失败: {}", e).red()
                    );
                }
            }
        } else {
            println!(
                "  {} {}",
                "跳过".yellow(),
                format!("{}: 没有配置更新 URL", db_config.name).yellow()
            );
        }
    }

    Ok(())
}

fn download_file(
    client: &reqwest::blocking::Client,
    url: &str,
    path: &std::path::Path,
) -> Result<()> {
    let response = client.get(url).send()?;

    if !response.status().is_success() {
        return Err(anyhow::anyhow!(
            "HTTP 请求失败: {}",
            response.status()
        ));
    }

    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    let mut file = std::fs::File::create(path)?;
    std::io::copy(&mut response.bytes()?.as_ref(), &mut file)?;

    Ok(())
}

fn handle_status(config: &Config, dirs: &NalideDirs) {
    println!("{}", "Nalide 状态信息".bright_cyan().bold());
    println!();

    dirs.display();

    println!();
    println!("{}", "已配置的数据库：".bright_yellow().bold());

    for db_config in &config.databases {
        let status = if db_config.enabled { "启用" } else { "禁用" };
        let status_color = if db_config.enabled {
            status.green()
        } else {
            status.red()
        };

        let path = config.resolve_db_path(db_config, dirs);
        let exists = path.exists();
        let exists_str = if exists { "存在" } else { "缺失" };
        let exists_color = if exists {
            exists_str.green()
        } else {
            exists_str.red()
        };

        println!(
            "  {} [{}] [{}] {}",
            db_config.name.bright_white(),
            status_color,
            exists_color,
            path.display().to_string().dimmed()
        );

        if let Some(ref url) = db_config.update_url {
            println!("    更新 URL: {}", url.dimmed());
        }
        println!("    更新周期: {} 秒", db_config.update_interval);
    }
}

fn auto_download_databases(
    config: &Config,
    dirs: &NalideDirs,
    databases: &[config::DatabaseConfig],
) -> Result<()> {
    dirs.ensure_dirs()?;

    let client = reqwest::blocking::Client::builder()
        .timeout(std::time::Duration::from_secs(120))
        .build()?;

    for db_config in databases {
        let db_path = config.resolve_db_path(db_config, dirs);

        if let Some(ref update_url) = db_config.update_url {
            println!(
                "  {} {}",
                "下载:".blue(),
                db_config.name
            );
            println!("    URL: {}", update_url);

            match download_file(&client, update_url, &db_path) {
                Ok(_) => {
                    println!(
                        "    {} {}",
                        "✓".green().bold(),
                        "下载完成".green()
                    );
                }
                Err(e) => {
                    eprintln!(
                        "    {} {}",
                        "✗".red().bold(),
                        format!("下载失败: {}", e).red()
                    );
                }
            }
        } else {
            println!(
                "  {} {}",
                "跳过".yellow(),
                format!("{}: 没有配置更新 URL", db_config.name).yellow()
            );
        }
    }

    Ok(())
}
