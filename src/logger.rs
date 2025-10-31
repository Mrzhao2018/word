use bevy::prelude::*;
use std::collections::VecDeque;
use std::fs::{File, OpenOptions};
use std::io::Write;
use std::path::PathBuf;
use chrono::Local;

/// 日志级别
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LogLevel {
    Debug,
    Info,
    Warning,
    Error,
}

impl LogLevel {
    pub fn as_str(&self) -> &str {
        match self {
            LogLevel::Debug => "DEBUG",
            LogLevel::Info => "INFO",
            LogLevel::Warning => "WARN",
            LogLevel::Error => "ERROR",
        }
    }

    pub fn color(&self) -> Color {
        match self {
            LogLevel::Debug => Color::srgb(0.6, 0.6, 0.6),
            LogLevel::Info => Color::srgb(0.8, 0.8, 1.0),
            LogLevel::Warning => Color::srgb(1.0, 0.8, 0.3),
            LogLevel::Error => Color::srgb(1.0, 0.3, 0.3),
        }
    }
}

/// 日志消息
#[derive(Clone)]
pub struct LogMessage {
    pub level: LogLevel,
    pub message: String,
    pub timestamp: String,
}

/// 游戏日志系统资源
#[derive(Resource)]
pub struct GameLogger {
    pub messages: VecDeque<LogMessage>,
    pub max_messages: usize,
    pub log_file: Option<PathBuf>,
    pub debug_enabled: bool,
}

impl Default for GameLogger {
    fn default() -> Self {
        Self {
            messages: VecDeque::new(),
            max_messages: 100,
            log_file: Some(PathBuf::from("game.log")),
            debug_enabled: false,
        }
    }
}

impl GameLogger {
    /// 记录日志消息
    pub fn log(&mut self, level: LogLevel, message: String) {
        // 如果是Debug级别且调试未开启，不记录
        if level == LogLevel::Debug && !self.debug_enabled {
            return;
        }

        let timestamp = Local::now().format("%H:%M:%S").to_string();
        
        let log_msg = LogMessage {
            level,
            message: message.clone(),
            timestamp: timestamp.clone(),
        };

        // 添加到消息队列
        self.messages.push_back(log_msg.clone());
        
        // 保持队列大小
        if self.messages.len() > self.max_messages {
            self.messages.pop_front();
        }

        // 写入日志文件
        if let Some(log_path) = &self.log_file {
            if let Ok(mut file) = OpenOptions::new()
                .create(true)
                .append(true)
                .open(log_path)
            {
                let _ = writeln!(
                    file,
                    "[{}] [{}] {}",
                    timestamp,
                    level.as_str(),
                    message
                );
            }
        }
    }

    /// 调试消息
    pub fn debug(&mut self, message: String) {
        self.log(LogLevel::Debug, message);
    }

    /// 信息消息
    pub fn info(&mut self, message: String) {
        self.log(LogLevel::Info, message);
    }

    /// 警告消息
    pub fn warning(&mut self, message: String) {
        self.log(LogLevel::Warning, message);
    }

    /// 错误消息
    pub fn error(&mut self, message: String) {
        self.log(LogLevel::Error, message);
    }

    /// 切换调试模式
    pub fn toggle_debug(&mut self) {
        self.debug_enabled = !self.debug_enabled;
        let status = if self.debug_enabled { "开启" } else { "关闭" };
        self.info(format!("调试模式已{}", status));
    }

    /// 清空日志文件
    pub fn clear_log_file(&mut self) {
        if let Some(log_path) = &self.log_file {
            if let Ok(mut file) = File::create(log_path) {
                let _ = writeln!(file, "=== 游戏日志 ===");
                let _ = writeln!(file, "开始时间: {}", Local::now().format("%Y-%m-%d %H:%M:%S"));
                let _ = writeln!(file, "");
            }
        }
    }
}
