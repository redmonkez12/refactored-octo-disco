use std::fmt;

struct RequestResult {
    endpoint: String,
    status: u16,
    attempts: u32,
}

#[derive(Debug, PartialEq, Eq)]
enum PlanError {
    ZeroLimit,
    InvalidStatus { index: usize },
    ZeroAttempts { index: usize },
}

impl fmt::Display for PlanError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ZeroLimit => formatter.write_str("invalid_limit"),
            Self::InvalidStatus { index } => write!(formatter, "invalid_status:{index}"),
            Self::ZeroAttempts { index } => write!(formatter, "invalid_attempts:{index}"),
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
enum Action {
    Ignore,
    Accept,
    Retry,
    Reject,
    GiveUp,
}

impl fmt::Display for Action {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::Ignore => "ignore",
            Self::Accept => "accept",
            Self::Retry => "retry",
            Self::Reject => "reject",
            Self::GiveUp => "give_up",
        })
    }
}

// Status and attempt counts are validated by plan before action selection.
fn select_action(status: u16, attempts: u32, max_attempts: u32) -> Action {
    match status {
        200..=299 => Action::Accept,
        429 | 500..=599 if attempts < max_attempts => Action::Retry,
        429 | 500..=599 => Action::GiveUp,
        400..=499 => Action::Reject,
        _ => Action::Ignore,
    }
}

fn format_decision(endpoint: &str, action: Action) -> String {
    format!("{endpoint}:{action}")
}

fn plan(request_results: &[RequestResult], max_attempts: u32) -> Result<Vec<String>, PlanError> {
    if max_attempts == 0 {
        return Err(PlanError::ZeroLimit);
    }

    let mut result = Vec::new();

    for (index, request_result) in request_results.iter().enumerate() {
        if !(100..=599).contains(&request_result.status) {
            return Err(PlanError::InvalidStatus { index });
        }
        if request_result.attempts == 0 {
            return Err(PlanError::ZeroAttempts { index });
        }

        let action = select_action(request_result.status, request_result.attempts, max_attempts);
        result.push(format_decision(&request_result.endpoint, action));
    }

    Ok(result)
}

fn main() {
    let input = [
        RequestResult {
            endpoint: "/billing".into(),
            status: 503,
            attempts: 1,
        },
        RequestResult {
            endpoint: "/export".into(),
            status: 429,
            attempts: 3,
        },
        RequestResult {
            endpoint: "/users".into(),
            status: 200,
            attempts: 1,
        },
        RequestResult {
            endpoint: "/auth".into(),
            status: 401,
            attempts: 1,
        },
        RequestResult {
            endpoint: "/redirect".into(),
            status: 302,
            attempts: 1,
        },
    ];

    let first_plan = plan(&input, 3).unwrap();
    let second_plan = plan(&input, 3).unwrap();
    assert_eq!(first_plan, second_plan);

    for line in first_plan {
        println!("{line}");
    }
}

#[cfg(test)]
mod tests {
    use super::{Action, PlanError, RequestResult, format_decision, plan, select_action};

    fn request(endpoint: &str, status: u16, attempts: u32) -> RequestResult {
        RequestResult {
            endpoint: endpoint.into(),
            status,
            attempts,
        }
    }

    #[test]
    fn action_selection_preserves_status_boundaries() {
        for (status, attempts, expected_action) in [
            (100, 1, Action::Ignore),
            (199, 1, Action::Ignore),
            (200, 1, Action::Accept),
            (299, 1, Action::Accept),
            (300, 1, Action::Ignore),
            (399, 1, Action::Ignore),
            (400, 1, Action::Reject),
            (428, 1, Action::Reject),
            (429, 1, Action::Retry),
            (430, 1, Action::Reject),
            (499, 1, Action::Reject),
            (500, 1, Action::Retry),
            (599, 1, Action::Retry),
            (200, 4, Action::Accept),
            (302, 4, Action::Ignore),
            (401, 4, Action::Reject),
        ] {
            assert_eq!(
                select_action(status, attempts, 3),
                expected_action,
                "status={status}, attempts={attempts}"
            );
        }
    }

    #[test]
    fn retryable_statuses_give_up_at_or_above_the_attempt_limit() {
        for status in [429, 500, 503, 599] {
            assert_eq!(select_action(status, 2, 3), Action::Retry);
            assert_eq!(select_action(status, 3, 3), Action::GiveUp);
            assert_eq!(select_action(status, 4, 3), Action::GiveUp);
            assert_eq!(select_action(status, 1, 1), Action::GiveUp);
        }
    }

    #[test]
    fn formatting_preserves_endpoint_text() {
        assert_eq!(format_decision("", Action::Ignore), ":ignore");
        assert_eq!(
            format_decision(" /api:read ", Action::Retry),
            " /api:read :retry"
        );
    }

    #[test]
    fn validation_errors_report_the_zero_based_input_index() {
        for index in [0, 1, 12] {
            for (status, attempts, expected_error) in [
                (0, 1, PlanError::InvalidStatus { index }),
                (99, 1, PlanError::InvalidStatus { index }),
                (600, 1, PlanError::InvalidStatus { index }),
                (u16::MAX, 1, PlanError::InvalidStatus { index }),
                (503, 0, PlanError::ZeroAttempts { index }),
            ] {
                let mut input: Vec<RequestResult> =
                    (0..index).map(|_| request("/valid", 200, 1)).collect();
                input.push(request("/invalid", status, attempts));

                assert_eq!(plan(&input, 3), Err(expected_error));
            }
        }
    }

    #[test]
    fn validation_preserves_error_priority_and_input_order() {
        assert_eq!(plan(&[], 0), Err(PlanError::ZeroLimit));
        assert_eq!(
            plan(&[request("/invalid", 600, 0)], 0),
            Err(PlanError::ZeroLimit)
        );
        assert_eq!(
            plan(&[request("/invalid", 600, 0)], 3),
            Err(PlanError::InvalidStatus { index: 0 })
        );
        assert_eq!(
            plan(&[request("/first", 200, 0), request("/second", 600, 1)], 3),
            Err(PlanError::ZeroAttempts { index: 0 })
        );
    }

    #[test]
    fn example_actions_and_order_are_preserved() {
        let input: Vec<RequestResult> = [
            ("/billing", 503, 1),
            ("/export", 429, 3),
            ("/users", 200, 1),
            ("/auth", 401, 1),
            ("/redirect", 302, 1),
        ]
        .into_iter()
        .map(|(endpoint, status, attempts)| request(endpoint, status, attempts))
        .collect();

        assert_eq!(
            plan(&input, 3).unwrap(),
            [
                "/billing:retry",
                "/export:give_up",
                "/users:accept",
                "/auth:reject",
                "/redirect:ignore",
            ]
        );
    }

    #[test]
    fn plan_preserves_order_and_duplicates_at_status_boundaries() {
        let input = [
            request("/z", 100, 1),
            request("/same", 199, 1),
            request("/same", 200, 1),
            request("/a", 299, 1),
            request("/same", 300, 1),
            request("/same", 399, 1),
            request("/z", 400, 1),
            request("/same", 499, 1),
            request("/a", 500, 2),
            request("/same", 599, 3),
        ];

        assert_eq!(
            plan(&input, 3).unwrap(),
            [
                "/z:ignore",
                "/same:ignore",
                "/same:accept",
                "/a:accept",
                "/same:ignore",
                "/same:ignore",
                "/z:reject",
                "/same:reject",
                "/a:retry",
                "/same:give_up",
            ]
        );
    }

    #[test]
    fn plan_preserves_opaque_endpoint_text() {
        let input = [
            request("", 200, 1),
            request("/api", 200, 1),
            request("/API", 200, 1),
            request(" /api ", 200, 1),
            request("/api:read", 200, 1),
            request("/uživatelé/🚀", 200, 1),
        ];

        assert_eq!(
            plan(&input, 3).unwrap(),
            [
                ":accept",
                "/api:accept",
                "/API:accept",
                " /api :accept",
                "/api:read:accept",
                "/uživatelé/🚀:accept",
            ]
        );
    }

    #[test]
    fn plan_handles_maximum_attempt_counts_without_overflow() {
        for (status, attempts, max_attempts, expected_row) in [
            (429, u32::MAX - 1, u32::MAX, "/attempts:retry"),
            (429, u32::MAX, u32::MAX, "/attempts:give_up"),
            (503, u32::MAX - 1, u32::MAX, "/attempts:retry"),
            (503, u32::MAX, u32::MAX, "/attempts:give_up"),
            (429, u32::MAX, 1, "/attempts:give_up"),
            (503, u32::MAX, 1, "/attempts:give_up"),
            (200, u32::MAX, 1, "/attempts:accept"),
            (401, u32::MAX, 1, "/attempts:reject"),
            (302, u32::MAX, 1, "/attempts:ignore"),
        ] {
            assert_eq!(
                plan(&[request("/attempts", status, attempts)], max_attempts).unwrap(),
                [expected_row],
                "status={status}, attempts={attempts}, max_attempts={max_attempts}"
            );
        }
    }

    #[test]
    fn late_errors_preserve_priority_and_allow_input_reuse_after_repairs() {
        let mut input = [
            request("/prefix", 200, 1),
            request("/retry", 503, 0),
            request("/later", 99, 0),
        ];

        assert_eq!(plan(&input, 3), Err(PlanError::ZeroAttempts { index: 1 }));

        input[1].attempts = 1;
        assert_eq!(plan(&input, 3), Err(PlanError::InvalidStatus { index: 2 }));

        input[2].status = 200;
        assert_eq!(plan(&input, 3), Err(PlanError::ZeroAttempts { index: 2 }));

        input[2].attempts = 1;
        assert_eq!(
            plan(&input, 3).unwrap(),
            ["/prefix:accept", "/retry:retry", "/later:accept"]
        );
    }

    #[test]
    fn large_input_preserves_every_row_and_reports_a_late_error() {
        let fixtures = [
            ("/repeat", 503, 1, "/repeat:retry"),
            ("/other", 429, 3, "/other:give_up"),
            ("/repeat", 200, 1, "/repeat:accept"),
            ("/auth", 401, 1, "/auth:reject"),
            ("/redirect", 302, 1, "/redirect:ignore"),
        ];
        let mut input: Vec<RequestResult> = fixtures
            .iter()
            .cycle()
            .take(10_000)
            .map(|&(endpoint, status, attempts, _)| request(endpoint, status, attempts))
            .collect();

        let output = plan(&input, 3).unwrap();
        assert_eq!(output.len(), input.len());
        for (index, row) in output.iter().enumerate() {
            assert_eq!(row, fixtures[index % fixtures.len()].3, "row {index}");
        }

        let error_index = input.len();
        input.push(request("/invalid", 600, 0));
        let error = plan(&input, 3).unwrap_err();
        assert_eq!(error, PlanError::InvalidStatus { index: error_index });
        assert_eq!(error.to_string(), "invalid_status:10000");
    }

    #[test]
    fn empty_input_with_a_valid_limit_returns_an_empty_plan() {
        assert_eq!(plan(&[], 3), Ok(vec![]));
    }

    #[test]
    fn borrowed_input_can_be_reused_without_changes() {
        let input = [request("/billing", 503, 1)];

        let first_plan = plan(&input, 3).unwrap();
        let second_plan = plan(&input, 1).unwrap();

        assert_eq!(first_plan, ["/billing:retry"]);
        assert_eq!(second_plan, ["/billing:give_up"]);
        assert_eq!(input[0].endpoint, "/billing");
        assert_eq!(input[0].status, 503);
        assert_eq!(input[0].attempts, 1);
    }

    #[test]
    fn formatted_output_survives_input_mutation_and_drop() {
        let output = {
            let mut input = [request("/users", 200, 1)];
            let output = plan(&input, 3).unwrap();
            input[0].endpoint.clear();
            output
        };

        assert_eq!(output, ["/users:accept"]);
    }

    #[test]
    fn error_display_preserves_the_original_error_text() {
        for (error, expected_text) in [
            (PlanError::ZeroLimit, "invalid_limit"),
            (PlanError::InvalidStatus { index: 0 }, "invalid_status:0"),
            (PlanError::InvalidStatus { index: 12 }, "invalid_status:12"),
            (PlanError::ZeroAttempts { index: 0 }, "invalid_attempts:0"),
            (PlanError::ZeroAttempts { index: 12 }, "invalid_attempts:12"),
        ] {
            assert_eq!(error.to_string(), expected_text);
        }
    }
}
