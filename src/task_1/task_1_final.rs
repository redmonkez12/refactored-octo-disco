use std::collections::HashMap;

#[derive(PartialEq)]
enum Level {
    Error,
    Info,
    Other,
}

impl From<&str> for Level {
    fn from(value: &str) -> Self {
        match value {
            "error" => Self::Error,
            "info" => Self::Info,
            _ => Self::Other,
        }
    }
}

struct LogRecord {
    service: String,
    level: Level,
}

#[derive(Debug, PartialEq, Eq)]
struct ServiceErrorCount<'a> {
    service: &'a str,
    count: usize,
}

fn count_errors(logs: &[LogRecord]) -> Vec<ServiceErrorCount<'_>> {
    let mut counts_by_service: HashMap<&str, usize> = HashMap::new();

    for log in logs {
        if log.level == Level::Error {
            *counts_by_service.entry(log.service.as_str()).or_default() += 1;
        }
    }

    let mut counts: Vec<_> = counts_by_service
        .into_iter()
        .map(|(service, count)| ServiceErrorCount { service, count })
        .collect();
    counts.sort_unstable_by(|left, right| left.service.cmp(right.service));

    counts
}

fn format_report(counts: Vec<ServiceErrorCount<'_>>) -> Vec<String> {
    counts
        .into_iter()
        .map(|entry| format!("{}: {}", entry.service, entry.count))
        .collect()
}

fn report(logs: &[LogRecord]) -> Vec<String> {
    format_report(count_errors(logs))
}

fn main() {
    let logs = vec![
        LogRecord {
            service: "payments".into(),
            level: "error".into(),
        },
        LogRecord {
            service: "api".into(),
            level: "info".into(),
        },
        LogRecord {
            service: "api".into(),
            level: "error".into(),
        },
        LogRecord {
            service: "payments".into(),
            level: "error".into(),
        },
    ];

    println!("{:?}", report(&logs));
    assert_eq!(logs.len(), 4);
}

#[cfg(test)]
mod tests {
    use super::*;

    fn log_records(input: &[(&str, &str)]) -> Vec<LogRecord> {
        input
            .iter()
            .map(|&(service, level)| LogRecord {
                service: service.into(),
                level: level.into(),
            })
            .collect()
    }

    #[test]
    fn count_errors_returns_structured_counts_for_exact_error_levels() {
        let logs = log_records(&[
            ("payments", "error"),
            ("api", "info"),
            ("api", "error"),
            ("api", "ERROR"),
            ("api", "Error"),
            ("api", "error "),
            ("api", "unknown"),
            ("api", "debug"),
            ("api", "error"),
            ("worker", "warn"),
        ]);

        assert_eq!(
            count_errors(&logs),
            vec![
                ServiceErrorCount {
                    service: "api",
                    count: 2,
                },
                ServiceErrorCount {
                    service: "payments",
                    count: 1,
                },
            ]
        );
    }

    #[test]
    fn count_errors_preserves_and_sorts_exact_service_names() {
        let logs = log_records(&[
            ("worker", "error"),
            ("api", "error"),
            ("API", "error"),
            (" api ", "error"),
            ("", "error"),
            (" api ", "error"),
        ]);

        assert_eq!(
            count_errors(&logs),
            vec![
                ServiceErrorCount {
                    service: "",
                    count: 1,
                },
                ServiceErrorCount {
                    service: " api ",
                    count: 2,
                },
                ServiceErrorCount {
                    service: "API",
                    count: 1,
                },
                ServiceErrorCount {
                    service: "api",
                    count: 1,
                },
                ServiceErrorCount {
                    service: "worker",
                    count: 1,
                },
            ]
        );
    }

    #[test]
    fn count_errors_returns_no_counts_without_errors() {
        assert!(count_errors(&[]).is_empty());
        assert!(count_errors(&log_records(&[("api", "info"), ("worker", "warn")])).is_empty());
    }

    #[test]
    fn count_errors_borrows_service_names_from_input() {
        let logs = log_records(&[
            ("worker", "error"),
            ("api", "info"),
            ("api", "error"),
            ("worker", "error"),
        ]);

        let counts = count_errors(&logs);

        assert_eq!(counts.len(), 2);
        for entry in counts {
            assert!(
                logs.iter()
                    .any(|log| std::ptr::eq(entry.service, log.service.as_str()))
            );
        }
    }

    #[test]
    fn format_report_formats_structured_counts_independently() {
        let counts = vec![
            ServiceErrorCount {
                service: "",
                count: 1,
            },
            ServiceErrorCount {
                service: " api ",
                count: 12,
            },
            ServiceErrorCount {
                service: "api:worker",
                count: 3,
            },
        ];

        assert_eq!(
            format_report(counts),
            vec![": 1", " api : 12", "api:worker: 3"]
        );
    }

    #[test]
    fn format_report_returns_no_entries_without_counts() {
        assert!(format_report(Vec::new()).is_empty());
    }

    #[test]
    fn report_leaves_input_reusable_and_returns_owned_output() {
        let input = [("worker", "error"), ("api", "info"), ("api", "error")];

        let error_report = {
            let logs = log_records(&input);
            let first_report = report(&logs);

            assert_eq!(logs.len(), input.len());
            for (log, &(service, level)) in logs.iter().zip(&input) {
                assert_eq!(log.service, service);
                assert!(log.level == Level::from(level));
            }
            assert_eq!(report(&logs), first_report);

            first_report
        };

        assert_eq!(error_report, vec!["api: 1", "worker: 1"]);
    }

    #[test]
    fn report_matches_reference_for_generated_inputs() {
        let record_options = [
            ("", "error"),
            (" api ", "error"),
            ("API", "error"),
            ("api", "error"),
            ("api", "info"),
            ("api", "ERROR"),
            ("api", "unknown"),
            ("a", "error"),
            ("a:", "error"),
            ("žluťoučký", "error"),
        ];

        for length in 0..=3 {
            for mut encoding in 0..record_options.len().pow(length) {
                let mut input = Vec::new();
                for _ in 0..length {
                    input.push(record_options[encoding % record_options.len()]);
                    encoding /= record_options.len();
                }

                // A deliberately simple reference keeps counting independent of the map.
                let mut services: Vec<_> = input
                    .iter()
                    .filter(|&&(_, level)| level == "error")
                    .map(|&(service, _)| service)
                    .collect();
                services.sort_unstable();
                services.dedup();

                let expected: Vec<_> = services
                    .into_iter()
                    .map(|service| {
                        let count = input
                            .iter()
                            .filter(|&&(name, level)| name == service && level == "error")
                            .count();
                        format!("{service}: {count}")
                    })
                    .collect();

                assert_eq!(report(&log_records(&input)), expected, "input: {input:?}");
            }
        }
    }

    #[test]
    fn report_counts_many_services_in_sorted_order() {
        const SERVICE_COUNT: usize = 10_000;
        let mut logs = Vec::new();

        for occurrence in 0..5 {
            for index in (0..SERVICE_COUNT).rev() {
                logs.push(LogRecord {
                    service: format!("service-{index:05}"),
                    level: if occurrence <= index % 5 {
                        Level::Error
                    } else {
                        Level::Info
                    },
                });
            }
        }
        logs.push(LogRecord {
            service: "non-error-only".into(),
            level: Level::Other,
        });

        let expected: Vec<_> = (0..SERVICE_COUNT)
            .map(|index| format!("service-{index:05}: {}", index % 5 + 1))
            .collect();

        assert_eq!(report(&logs), expected);
    }

    #[test]
    fn report_preserves_the_behavior_contract() {
        type ReportScenario<'a> = (&'a str, &'a [(&'a str, &'a str)], &'a [&'a str]);

        let scenarios: &[ReportScenario<'_>] = &[
            ("empty input", &[], &[]),
            ("no errors", &[("api", "info"), ("worker", "warn")], &[]),
            (
                "repeated errors",
                &[("api", "error"), ("api", "error")],
                &["api: 2"],
            ),
            (
                "reverse input order",
                &[("worker", "error"), ("api", "error")],
                &["api: 1", "worker: 1"],
            ),
            (
                "mixed levels",
                &[("api", "info"), ("api", "error"), ("api", "debug")],
                &["api: 1"],
            ),
            (
                "case-sensitive levels",
                &[("api", "ERROR"), ("worker", "error")],
                &["worker: 1"],
            ),
            (
                "unknown level",
                &[("api", "unknown"), ("api", "error")],
                &["api: 1"],
            ),
            (
                "exact service names",
                &[("api", "error"), ("API", "error")],
                &["API: 1", "api: 1"],
            ),
            (
                "service names with whitespace",
                &[("api", "error"), (" api ", "error")],
                &[" api : 1", "api: 1"],
            ),
            (
                "empty service names",
                &[("", "error"), ("api", "error"), ("", "error")],
                &[": 2", "api: 1"],
            ),
            (
                "Unicode service names",
                &[
                    ("β", "error"),
                    ("ž", "error"),
                    ("é", "error"),
                    ("api", "error"),
                ],
                &["api: 1", "é: 1", "ž: 1", "β: 1"],
            ),
            (
                "sort by service name before formatting",
                &[("a:", "error"), ("a", "error")],
                &["a: 1", "a:: 1"],
            ),
        ];

        for &(name, input, expected) in scenarios {
            assert_eq!(report(&log_records(input)), expected, "{name}");
        }
    }
}
