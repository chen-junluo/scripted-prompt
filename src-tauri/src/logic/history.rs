// 使用历史统计
use crate::data::{Script, Template};
use chrono::Local;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};

/// 使用记录
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UsageRecord {
    pub id: String,
    pub item_type: String,        // "script" 或 "template"
    pub timestamp: String,        // 使用字符串存储时间戳
    pub duration_ms: Option<u64>, // 执行持续时间（毫秒）
}

/// 历史管理器
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoryManager {
    records: VecDeque<UsageRecord>,
    max_records: usize,
}

impl Default for HistoryManager {
    fn default() -> Self {
        Self::new()
    }
}

impl HistoryManager {
    pub fn new() -> Self {
        Self {
            records: VecDeque::with_capacity(1000),
            max_records: 1000,
        }
    }

    /// 记录脚本使用
    pub fn record_script_usage(&mut self, script: &Script, duration_ms: Option<u64>) {
        let record = UsageRecord {
            id: script.id.clone(),
            item_type: "script".to_string(),
            timestamp: Local::now().to_rfc3339(),
            duration_ms,
        };

        self.records.push_front(record);

        // 限制记录数量
        if self.records.len() > self.max_records {
            self.records.pop_back();
        }
    }

    /// 记录模板使用
    pub fn record_template_usage(&mut self, template: &Template, duration_ms: Option<u64>) {
        let record = UsageRecord {
            id: template.id.clone(),
            item_type: "template".to_string(),
            timestamp: Local::now().to_rfc3339(),
            duration_ms,
        };

        self.records.push_front(record);

        // 限制记录数量
        if self.records.len() > self.max_records {
            self.records.pop_back();
        }
    }

    /// 获取最近使用的脚本
    pub fn get_recent_scripts(&self, count: usize) -> Vec<String> {
        let mut script_ids = Vec::new();
        let mut seen = HashMap::new();

        for record in self.records.iter() {
            if record.item_type == "script" && !seen.contains_key(&record.id) {
                script_ids.push(record.id.clone());
                seen.insert(&record.id, true);

                if script_ids.len() >= count {
                    break;
                }
            }
        }

        script_ids
    }

    /// 获取最近使用的模板
    pub fn get_recent_templates(&self, count: usize) -> Vec<String> {
        let mut template_ids = Vec::new();
        let mut seen = HashMap::new();

        for record in self.records.iter() {
            if record.item_type == "template" && !seen.contains_key(&record.id) {
                template_ids.push(record.id.clone());
                seen.insert(&record.id, true);

                if template_ids.len() >= count {
                    break;
                }
            }
        }

        template_ids
    }

    /// 获取使用频率统计（过去指定天数）
    pub fn get_usage_stats(&self, days: u32) -> HashMap<String, usize> {
        let cutoff = Local::now()
            .checked_sub_days(chrono::Days::new(days.into()))
            .unwrap();
        let mut stats = HashMap::new();

        for record in self.records.iter() {
            // 解析时间戳字符串为DateTime对象进行比较
            if let Ok(timestamp) = chrono::DateTime::parse_from_rfc3339(&record.timestamp) {
                if timestamp.with_timezone(&Local) >= cutoff {
                    *stats
                        .entry(format!("{}_{}", record.item_type, record.id))
                        .or_insert(0) += 1;
                }
            }
        }

        stats
    }

    /// 清空历史记录
    pub fn clear_history(&mut self) {
        self.records.clear();
    }

    /// 导出历史记录
    pub fn export_history(&self) -> Vec<UsageRecord> {
        self.records.clone().into_iter().collect()
    }

    /// 导入历史记录
    pub fn import_history(&mut self, new_records: Vec<UsageRecord>) {
        // 添加新记录到队列前端
        for record in new_records.iter().rev() {
            self.records.push_front(record.clone());
        }

        // 限制记录数量
        while self.records.len() > self.max_records {
            self.records.pop_back();
        }
    }
}

/// 计算脚本使用趋势（按天）
pub fn calculate_script_trend(
    history_manager: &HistoryManager,
    script_id: &str,
    days: u32,
) -> Vec<(String, usize)> {
    let cutoff = Local::now()
        .checked_sub_days(chrono::Days::new(days.into()))
        .unwrap();
    let mut daily_counts: HashMap<String, usize> = HashMap::new();

    // 初始化每一天的计数
    for i in 0..days {
        let date = Local::now()
            .checked_sub_days(chrono::Days::new(i.into()))
            .unwrap();
        let date_str = date.format("%Y-%m-%d").to_string();
        daily_counts.insert(date_str, 0);
    }

    // 统计每一天的使用次数
    for record in history_manager.records.iter() {
        if record.item_type == "script" && record.id == script_id {
            // 解析时间戳字符串为DateTime对象进行比较
            if let Ok(timestamp) = chrono::DateTime::parse_from_rfc3339(&record.timestamp) {
                if timestamp.with_timezone(&Local) >= cutoff {
                    // 格式化日期字符串用于统计
                    let date_str = timestamp
                        .with_timezone(&Local)
                        .format("%Y-%m-%d")
                        .to_string();
                    *daily_counts.entry(date_str).or_insert(0) += 1;
                }
            }
        }
    }

    // 转换为排序的向量
    let mut trend: Vec<(String, usize)> = daily_counts.into_iter().collect();
    trend.sort_by_key(|(date, _)| date.clone());

    trend
}
