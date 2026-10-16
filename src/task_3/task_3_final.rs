use std::collections::HashSet;

#[derive(Debug, PartialEq, Eq)]
pub enum JobState {
    Running,
    Ready,
    Unknown,
}

impl From<&str> for JobState {
    fn from(state: &str) -> Self {
        match state {
            "running" => Self::Running,
            "ready" => Self::Ready,
            _ => Self::Unknown,
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct JobCandidate {
    pub id: String,
    pub queue: String,
    pub state: JobState,
}

pub fn dispatch(candidates: &[JobCandidate], limit: usize) -> Vec<String> {
    format_jobs(select_jobs(candidates, limit))
}

fn select_jobs(candidates: &[JobCandidate], limit: usize) -> Vec<&JobCandidate> {
    let mut selected_jobs: Vec<&JobCandidate> = Vec::new();
    let mut selected_ids: HashSet<&str> = HashSet::new();

    for candidate in candidates {
        if selected_jobs.len() >= limit {
            break;
        }

        if candidate.state != JobState::Ready {
            continue;
        }

        if !selected_ids.insert(candidate.id.as_str()) {
            continue;
        }

        selected_jobs.push(candidate);
    }

    selected_jobs
}

fn format_jobs(selected_jobs: Vec<&JobCandidate>) -> Vec<String> {
    let mut result = Vec::new();

    for job in selected_jobs {
        result.push(format!("{}@{}", job.id, job.queue));
    }

    result
}

fn main() {
    let input: Vec<JobCandidate> = vec![
        JobCandidate {
            id: "job-7".to_string(),
            queue: "emails".to_string(),
            state: "running".into(),
        },
        JobCandidate {
            id: "job-2".to_string(),
            queue: "reports".to_string(),
            state: "ready".into(),
        },
        JobCandidate {
            id: "job-2".to_string(),
            queue: "urgent".to_string(),
            state: "ready".into(),
        },
        JobCandidate {
            id: "job-7".to_string(),
            queue: "emails".to_string(),
            state: "ready".into(),
        },
        JobCandidate {
            id: "job-9".to_string(),
            queue: "exports".to_string(),
            state: "ready".into(),
        },
    ];

    let result = dispatch(&input, 2);
    let repeated_result = dispatch(&input, 2);
    println!("{result:?}");
    assert_eq!(result, ["job-2@reports", "job-7@emails"]);
    assert_eq!(repeated_result, result);
}

#[cfg(test)]
mod tests {
    use super::{JobCandidate, JobState, dispatch, format_jobs, select_jobs};

    fn candidates(records: &[(&str, &str, &str)]) -> Vec<JobCandidate> {
        records
            .iter()
            .map(|&(id, queue, state)| JobCandidate {
                id: id.into(),
                queue: queue.into(),
                state: state.into(),
            })
            .collect()
    }

    #[test]
    fn selection_preserves_first_eligible_occurrences_order_and_unique_job_limit() {
        let input = candidates(&[
            ("job-7", "slow", "running"),
            ("job-2", "reports", "ready"),
            ("job-2", "urgent", "ready"),
            ("job-7", "emails", "ready"),
            ("job-9", "exports", "ready"),
        ]);

        let selected = select_jobs(&input, 2);

        assert_eq!(selected, [&input[1], &input[3]]);
    }

    #[test]
    fn selection_accepts_only_the_exact_ready_state() {
        let input = candidates(&[
            ("a", "q", "running"),
            ("a", "q", "READY"),
            ("a", "q", " ready "),
            ("a", "q", "pending"),
            ("a", "q", ""),
            ("a", "eligible", "ready"),
        ]);

        let selected = select_jobs(&input, 3);

        assert_eq!(selected, [&input[5]]);
    }

    #[test]
    fn selection_is_empty_for_empty_input_or_zero_limit() {
        assert!(select_jobs(&[], 3).is_empty());
        assert!(select_jobs(&candidates(&[("a", "q", "ready")]), 0).is_empty());
    }

    #[test]
    fn selection_keeps_ids_and_queues_separate_even_when_they_contain_at_signs() {
        let input = candidates(&[("a@b", "c", "ready"), ("a", "b@c", "ready")]);

        let selected = select_jobs(&input, 3);

        assert_eq!(selected, [&input[0], &input[1]]);
    }

    #[test]
    fn formatting_preserves_selected_order_and_opaque_strings() {
        let input = candidates(&[(" a@b ", " q ", "running"), ("", "", "unknown")]);

        let rows = format_jobs(vec![&input[0], &input[1]]);

        assert_eq!(rows, [" a@b @ q ", "@"]);
    }

    #[test]
    fn dispatch_returns_the_original_formatted_example() {
        let rows = dispatch(
            &candidates(&[
                ("job-7", "emails", "running"),
                ("job-2", "reports", "ready"),
                ("job-2", "urgent", "ready"),
                ("job-7", "emails", "ready"),
                ("job-9", "exports", "ready"),
            ]),
            2,
        );

        assert_eq!(rows, ["job-2@reports", "job-7@emails"]);
    }

    #[test]
    fn selection_borrows_the_original_candidates() {
        let input = candidates(&[
            ("job-7", "slow", "running"),
            ("job-2", "reports", "ready"),
            ("job-2", "urgent", "ready"),
            ("job-7", "emails", "ready"),
        ]);

        let selected = select_jobs(&input, 2);

        assert_eq!(selected.len(), 2);
        assert!(std::ptr::eq(selected[0], &input[1]));
        assert!(std::ptr::eq(selected[1], &input[3]));
    }

    #[test]
    fn dispatch_reuses_unchanged_input_and_returns_output_that_outlives_it() {
        let rows = {
            let records = [
                ("job-7", "slow", "running"),
                ("job-2", "reports", "ready"),
                ("job-2", "urgent", "ready"),
                ("job-7", "emails", "ready"),
            ];
            let input = candidates(&records);

            let first_result = dispatch(&input, 2);
            assert_eq!(dispatch(&input, 2), first_result);
            assert_eq!(input.len(), records.len());
            for (candidate, (id, queue, state)) in input.iter().zip(records) {
                assert_eq!(candidate.id, id);
                assert_eq!(candidate.queue, queue);
                assert_eq!(candidate.state, JobState::from(state));
            }

            first_result
        };

        assert_eq!(rows, ["job-2@reports", "job-7@emails"]);
    }

    #[test]
    fn dispatch_preserves_all_regression_scenarios() {
        type Record<'a> = (&'a str, &'a str, &'a str);
        type Scenario<'a> = (&'a str, &'a [Record<'a>], usize, &'a [&'a str]);

        let scenarios: &[Scenario<'_>] = &[
            ("empty input", &[], 3, &[]),
            ("zero limit", &[("a", "emails", "ready")], 0, &[]),
            ("only running", &[("a", "emails", "running")], 3, &[]),
            ("uppercase state", &[("a", "emails", "READY")], 3, &[]),
            ("untrimmed state", &[("a", "emails", " ready ")], 3, &[]),
            ("unknown state", &[("a", "emails", "pending")], 3, &[]),
            ("empty state", &[("a", "emails", "")], 3, &[]),
            (
                "first eligible queue wins",
                &[("a", "slow", "ready"), ("a", "fast", "ready")],
                3,
                &["a@slow"],
            ),
            (
                "ineligible occurrence does not reserve the ID",
                &[("a", "slow", "running"), ("a", "fast", "ready")],
                3,
                &["a@fast"],
            ),
            (
                "input order is preserved",
                &[("b", "q", "ready"), ("a", "q", "ready")],
                3,
                &["b@q", "a@q"],
            ),
            (
                "limit counts unique selected jobs",
                &[
                    ("a", "q", "ready"),
                    ("a", "q", "ready"),
                    ("b", "q", "ready"),
                ],
                2,
                &["a@q", "b@q"],
            ),
            (
                "case-sensitive IDs",
                &[("a", "q", "ready"), ("A", "q", "ready")],
                3,
                &["a@q", "A@q"],
            ),
            (
                "empty IDs are deduplicated",
                &[("", "", "ready"), ("", "other", "ready")],
                3,
                &["@"],
            ),
            (
                "whitespace is preserved",
                &[(" a ", " q ", "ready")],
                3,
                &[" a @ q "],
            ),
            (
                "whitespace remains part of the ID",
                &[("a", "q", "ready"), (" a ", "q", "ready")],
                3,
                &["a@q", " a @q"],
            ),
            (
                "a later state does not cancel selection",
                &[("a", "q", "ready"), ("a", "q", "running")],
                3,
                &["a@q"],
            ),
            (
                "limit stops before later unique jobs",
                &[("b", "first", "ready"), ("a", "second", "ready")],
                1,
                &["b@first"],
            ),
        ];

        for &(name, records, limit, expected) in scenarios {
            assert_eq!(dispatch(&candidates(records), limit), expected, "{name}");
        }
    }

    #[test]
    fn selection_preserves_first_eligible_order_for_a_large_duplicate_batch() {
        const JOB_COUNT: usize = 5_000;
        let input: Vec<_> = (0..3)
            .flat_map(|round| {
                (0..JOB_COUNT).rev().map(move |index| JobCandidate {
                    id: format!("shared-prefix-job-{index:05}"),
                    queue: format!("queue-{round}"),
                    state: if round == 0 {
                        JobState::Running
                    } else {
                        JobState::Ready
                    },
                })
            })
            .collect();

        // A limit above the unique count forces inspection of the duplicate tail.
        let selected = select_jobs(&input, JOB_COUNT + 1);

        assert_eq!(selected.len(), JOB_COUNT);
        for (offset, job) in selected.iter().enumerate() {
            assert!(std::ptr::eq(*job, &input[JOB_COUNT + offset]));
        }

        let expected: Vec<_> = (0..JOB_COUNT)
            .rev()
            .map(|index| format!("shared-prefix-job-{index:05}@queue-1"))
            .collect();
        assert_eq!(format_jobs(selected), expected);
    }
}
