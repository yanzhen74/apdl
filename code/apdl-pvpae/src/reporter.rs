//! 报告生成器模块
//!
//! 实现协议验证与性能分析报告的生成

use crate::analyzer::PerformanceMetrics;
use std::collections::HashMap;

/// 报告类型
#[derive(Debug, Clone)]
pub enum ReportType {
    Validation,  // 验证报告
    Performance, // 性能报告
    Compliance,  // 符合性报告
    Summary,     // 汇总报告
}

/// 验证结果
#[derive(Debug, Clone)]
pub struct ValidationResult {
    pub passed: bool,
    pub message: String,
    pub details: Option<String>,
}

/// 报告生成器
pub struct ReportGenerator {
    report_title: String,
    report_author: String,
    results: Vec<ValidationResult>,
    metrics: HashMap<String, PerformanceMetrics>,
}

impl ReportGenerator {
    pub fn new(title: String, author: String) -> Self {
        Self {
            report_title: title,
            report_author: author,
            results: Vec::new(),
            metrics: HashMap::new(),
        }
    }

    /// 添加验证结果
    pub fn add_validation_result(&mut self, result: ValidationResult) {
        self.results.push(result);
    }

    /// 添加性能指标
    pub fn add_performance_metrics(&mut self, name: String, metrics: PerformanceMetrics) {
        self.metrics.insert(name, metrics);
    }

    /// 生成验证报告
    pub fn generate_validation_report(&self) -> String {
        let mut report = String::new();
        let report_title = &self.report_title;
        let report_author = &self.report_author;
        report.push_str(&format!("# {report_title}\n\n"));
        report.push_str(&format!("Author: {report_author}\n\n"));
        report.push_str("## Validation Results\n\n");

        let passed_count = self.results.iter().filter(|r| r.passed).count();
        let total_count = self.results.len();
        let pass_rate = if total_count > 0 {
            (passed_count as f64 / total_count as f64) * 100.0
        } else {
            100.0
        };

        report.push_str(&format!(
            "**Summary: {passed_count} out of {total_count} checks passed ({pass_rate:.1}%)\n\n"
        ));

        for (i, result) in self.results.iter().enumerate() {
            let status = if result.passed {
                "✅ PASS"
            } else {
                "❌ FAIL"
            };
            let index = i + 1;
            let message = &result.message;
            report.push_str(&format!("{index}. {status} - {message}\n"));
            if let Some(details) = &result.details {
                report.push_str(&format!("   - Details: {details}\n"));
            }
            report.push('\n');
        }

        report
    }

    /// 生成性能报告
    pub fn generate_performance_report(&self) -> String {
        let mut report = String::new();
        let report_title = &self.report_title;
        let report_author = &self.report_author;
        report.push_str(&format!("# {report_title}\n\n"));
        report.push_str(&format!("Author: {report_author}\n\n"));
        report.push_str("## Performance Metrics\n\n");

        for (name, metrics) in &self.metrics {
            report.push_str(&format!("### {name}\n\n"));
            let processing_time = metrics.processing_time;
            let throughput = metrics.throughput;
            let latency = metrics.latency;
            let utilization = metrics.utilization;
            let error_rate = metrics.error_rate;
            let packet_loss_rate = metrics.packet_loss_rate;
            report.push_str(&format!("Processing Time: {processing_time:?}\n"));
            report.push_str(&format!("Throughput: {throughput:.2} pps\n"));
            report.push_str(&format!("Latency: {latency:?}\n"));
            report.push_str(&format!("Utilization: {utilization:.2}%\n"));
            report.push_str(&format!("Error Rate: {error_rate:.2}%\n"));
            report.push_str(&format!("Packet Loss Rate: {packet_loss_rate:.2}%\n\n"));
        }

        report
    }

    /// 生成汇总报告
    pub fn generate_summary_report(&self) -> String {
        let mut report = String::new();
        let report_title = &self.report_title;
        let report_author = &self.report_author;
        report.push_str(&format!("# {report_title} - Summary Report\n\n"));
        report.push_str(&format!("Author: {report_author}\n\n"));

        // 添加验证摘要
        let passed_count = self.results.iter().filter(|r| r.passed).count();
        let total_count = self.results.len();
        let validation_success_rate = if total_count > 0 {
            (passed_count as f64 / total_count as f64) * 100.0
        } else {
            100.0
        };

        report.push_str("## Validation Summary\n\n");
        report.push_str(&format!(
            "Overall validation success rate: {validation_success_rate:.1}% ({passed_count} out of {total_count} checks passed)\n\n"
        ));

        // 添加性能摘要
        report.push_str("## Performance Summary\n\n");
        if !self.metrics.is_empty() {
            for (name, metrics) in &self.metrics {
                let throughput = metrics.throughput;
                let latency = metrics.latency;
                report.push_str(&format!(
                    "- {name}: {throughput:.2} pps throughput, {latency:?} latency\n"
                ));
            }
        } else {
            report.push_str("- No performance metrics recorded\n");
        }

        report.push_str("\n## Conclusion\n\n");
        if validation_success_rate >= 95.0 {
            report.push_str("✅ Protocol validation passed with high confidence.\n");
        } else if validation_success_rate >= 80.0 {
            report.push_str("⚠️ Protocol validation passed with some issues.\n");
        } else {
            report.push_str("❌ Protocol validation failed.\n");
        }

        report
    }

    /// 重置报告生成器
    pub fn reset(&mut self) {
        self.results.clear();
        self.metrics.clear();
    }
}
