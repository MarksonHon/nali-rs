use std::path::PathBuf;

use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(
    name = "nali",
    about = "离线 IP 地理信息查询工具",
    version,
    long_about = "Nalide - 一个离线的 IP 地理信息和 CDN 服务提供商查询工具\n\n支持多种数据库格式：\n  - MaxMind mmdb 格式 (GeoLite2-City.mmdb 等)\n  - V2Ray dat 格式 (geoip.dat, geosite.dat)"
)]
pub struct Cli {
    /// 要查询的 IP 地址
    pub ips: Vec<String>,

    /// 显示所有数据库的查询结果
    #[arg(short = 'a', long = "all")]
    pub all: bool,

    /// 指定数据库文件
    #[arg(short, long = "db", value_name = "FILE")]
    pub databases: Vec<PathBuf>,

    /// 指定输出模式
    #[arg(short, long, value_enum, default_value = "both")]
    pub output: OutputMode,

    /// 指定配置目录
    #[arg(long, value_name = "DIR")]
    pub config_dir: Option<PathBuf>,

    /// 指定数据目录
    #[arg(long, value_name = "DIR")]
    pub data_dir: Option<PathBuf>,

    /// 指定缓存目录
    #[arg(long, value_name = "DIR")]
    pub cache_dir: Option<PathBuf>,

    /// 不使用配置文件
    #[arg(long)]
    pub no_config: bool,

    /// 输出详细日志
    #[arg(short, long)]
    pub verbose: bool,

    #[command(subcommand)]
    pub command: Option<Commands>,
}

#[derive(Subcommand)]
pub enum Commands {
    /// 更新数据库
    Update {
        /// 指定要更新的数据库
        #[arg(short, long = "db")]
        database: Option<String>,

        /// 强制更新（忽略更新周期）
        #[arg(short, long)]
        force: bool,
    },

    /// 显示数据库状态
    Status,

    /// 显示目录信息
    Dirs {
        /// 打开配置文件所在目录
        #[arg(long)]
        open_config: bool,

        /// 打开数据库目录
        #[arg(long)]
        open_data: bool,
    },
}

#[derive(Debug, Clone, Copy, clap::ValueEnum)]
pub enum OutputMode {
    /// 仅显示紧凑格式（类似原版 nali）
    Compact,
    /// 仅显示表格格式
    Table,
    /// 同时显示紧凑和表格格式
    Both,
}

impl From<OutputMode> for crate::config::OutputMode {
    fn from(mode: OutputMode) -> Self {
        match mode {
            OutputMode::Compact => crate::config::OutputMode::Compact,
            OutputMode::Table => crate::config::OutputMode::Table,
            OutputMode::Both => crate::config::OutputMode::Both,
        }
    }
}
