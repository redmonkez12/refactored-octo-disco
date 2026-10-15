#[derive(Debug, PartialEq, Eq)]
struct RetryConfig {
    attempts: u32,
    delay_ms: u64,
}

#[derive(Debug, PartialEq, Eq)]
enum ConfigError {
    DuplicateKey,
    InvalidFormat,
    InvalidAttempts,
    InvalidDelay,
    UnknownKey,
}

impl std::fmt::Display for ConfigError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let code = match self {
            Self::DuplicateKey => "duplicate_key",
            Self::InvalidFormat => "invalid_format",
            Self::InvalidAttempts => "invalid_attempts",
            Self::InvalidDelay => "invalid_delay",
            Self::UnknownKey => "unknown_key",
        };
        formatter.write_str(code)
    }
}

fn read_settings(input: &[String]) -> Result<RetryConfig, ConfigError> {
    let mut attempts = 3;
    let mut delay_ms = 1_000;
    let mut seen_attempts = false;
    let mut seen_delay_ms = false;

    for line in input {
        let (key, value) = line.split_once('=').ok_or(ConfigError::InvalidFormat)?;

        if value.contains('=') {
            return Err(ConfigError::InvalidFormat);
        }

        let key = key.trim();
        let value = value.trim();

        match key {
            "attempts" => {
                if seen_attempts {
                    return Err(ConfigError::DuplicateKey);
                }
                seen_attempts = true;

                let parsed_attempts = value
                    .parse::<u32>()
                    .map_err(|_| ConfigError::InvalidAttempts)?;
                if !(1..=10).contains(&parsed_attempts) {
                    return Err(ConfigError::InvalidAttempts);
                }
                attempts = parsed_attempts;
            }
            "delay_ms" => {
                if seen_delay_ms {
                    return Err(ConfigError::DuplicateKey);
                }
                seen_delay_ms = true;

                let parsed_delay_ms = value
                    .parse::<u64>()
                    .map_err(|_| ConfigError::InvalidDelay)?;
                if parsed_delay_ms > 60_000 {
                    return Err(ConfigError::InvalidDelay);
                }
                delay_ms = parsed_delay_ms;
            }
            _ => return Err(ConfigError::UnknownKey),
        }
    }

    Ok(RetryConfig { attempts, delay_ms })
}

fn main() {
    let input = vec![" attempts = 4 ".to_string(), "delay_ms=250".to_string()];

    match read_settings(&input) {
        Ok(config) => {
            println!("attempts={};delay_ms={}", config.attempts, config.delay_ms);
        }
        Err(error) => println!("error={error}"),
    }

    assert_eq!(input, [" attempts = 4 ", "delay_ms=250"]);
}

#[cfg(test)]
mod tests {
    use super::{ConfigError, RetryConfig, read_settings};

    fn input_lines(lines: &[&str]) -> Vec<String> {
        lines.iter().map(|line| (*line).to_string()).collect()
    }

    #[test]
    fn empty_input_returns_both_defaults() {
        assert_eq!(
            read_settings(&[]),
            Ok(RetryConfig {
                attempts: 3,
                delay_ms: 1_000,
            })
        );
    }

    #[test]
    fn a_single_setting_preserves_the_other_default() {
        let scenarios = [
            (
                "attempts=4",
                RetryConfig {
                    attempts: 4,
                    delay_ms: 1_000,
                },
            ),
            (
                "delay_ms=250",
                RetryConfig {
                    attempts: 3,
                    delay_ms: 250,
                },
            ),
        ];

        for (line, expected) in scenarios {
            assert_eq!(read_settings(&input_lines(&[line])), Ok(expected), "{line}");
        }
    }

    #[test]
    fn range_boundaries_are_inclusive() {
        for attempts in [1, 10] {
            for delay_ms in [0, 60_000] {
                let input = vec![
                    format!("attempts={attempts}"),
                    format!("delay_ms={delay_ms}"),
                ];

                assert_eq!(
                    read_settings(&input),
                    Ok(RetryConfig { attempts, delay_ms }),
                    "input: {input:?}"
                );
            }
        }
    }

    #[test]
    fn values_outside_the_allowed_ranges_are_rejected() {
        let scenarios = [
            ("attempts=0", ConfigError::InvalidAttempts),
            ("attempts=11", ConfigError::InvalidAttempts),
            ("attempts=4294967295", ConfigError::InvalidAttempts),
            ("delay_ms=60001", ConfigError::InvalidDelay),
            ("delay_ms=18446744073709551615", ConfigError::InvalidDelay),
        ];

        for (line, expected) in scenarios {
            assert_eq!(
                read_settings(&input_lines(&[line])),
                Err(expected),
                "{line}"
            );
        }
    }

    #[test]
    fn whitespace_leading_plus_and_either_key_order_are_accepted() {
        let scenarios: &[&[&str]] = &[
            &[" attempts = 4 ", "delay_ms=250"],
            &["delay_ms=250", "attempts=4"],
            &["\t delay_ms \t=\t +250 \r\n", "\t attempts = +4\t"],
            &["\u{2003}attempts\u{a0}=\u{a0}+4\u{2003}", "delay_ms=+250"],
            &["attempts=0004", "delay_ms=000250"],
        ];

        for &scenario in scenarios {
            let input = input_lines(scenario);

            assert_eq!(
                read_settings(&input),
                Ok(RetryConfig {
                    attempts: 4,
                    delay_ms: 250,
                }),
                "input: {input:?}"
            );
        }
    }

    #[test]
    fn invalid_numbers_are_reported_for_their_setting() {
        for value in ["", " ", "nope", "-1", "-0", "1.0", "1_0", "+", "+ 1", "1 0"] {
            let attempts = vec![format!("attempts={value}")];
            let delay = vec![format!("delay_ms={value}")];

            assert_eq!(
                read_settings(&attempts),
                Err(ConfigError::InvalidAttempts),
                "input: {attempts:?}"
            );
            assert_eq!(
                read_settings(&delay),
                Err(ConfigError::InvalidDelay),
                "input: {delay:?}"
            );
        }
    }

    #[test]
    fn integer_overflow_is_rejected_without_panicking() {
        let scenarios = [
            ("attempts=4294967296", ConfigError::InvalidAttempts),
            ("delay_ms=18446744073709551616", ConfigError::InvalidDelay),
        ];

        for (line, expected) in scenarios {
            assert_eq!(
                read_settings(&input_lines(&[line])),
                Err(expected),
                "{line}"
            );
        }
    }

    #[test]
    fn unknown_empty_and_differently_cased_keys_are_rejected_before_values() {
        for line in [
            "other=1",
            "other=nope",
            "other=",
            "Attempts=3",
            "ATTEMPTS=3",
            "Delay_ms=250",
            "DELAY_MS=250",
            "=1",
            " \t =nope",
        ] {
            assert_eq!(
                read_settings(&input_lines(&[line])),
                Err(ConfigError::UnknownKey),
                "{line}"
            );
        }
    }

    #[test]
    fn duplicate_known_keys_are_rejected_before_parsing_their_values() {
        let scenarios: &[&[&str]] = &[
            &["attempts=3", "attempts=4"],
            &["attempts=3", "attempts=nope"],
            &["attempts=3", " attempts = 11"],
            &["attempts=3", "attempts="],
            &["delay_ms=250", "delay_ms=0"],
            &["delay_ms=250", " delay_ms = -1"],
            &["delay_ms=250", "delay_ms=60001"],
            &["attempts=3", "delay_ms=250", "attempts=nope"],
        ];

        for &scenario in scenarios {
            let input = input_lines(scenario);

            assert_eq!(
                read_settings(&input),
                Err(ConfigError::DuplicateKey),
                "input: {input:?}"
            );
        }
    }

    #[test]
    fn malformed_lines_take_priority_over_key_and_value_validation() {
        let scenarios: &[&[&str]] = &[
            &[""],
            &[" \t "],
            &["attempts"],
            &["delay_ms"],
            &["other"],
            &["=="],
            &["attempts=1=2"],
            &["delay_ms=0="],
            &["other=1=2"],
            &["attempts=3", "attempts=4=5"],
            &["delay_ms=250", " delay_ms = nope=1"],
        ];

        for &scenario in scenarios {
            let input = input_lines(scenario);

            assert_eq!(
                read_settings(&input),
                Err(ConfigError::InvalidFormat),
                "input: {input:?}"
            );
        }
    }

    #[test]
    fn the_first_error_in_input_order_is_returned() {
        let scenarios: &[(&[&str], ConfigError)] = &[
            (&["attempts=11", "other=1"], ConfigError::InvalidAttempts),
            (&["attempts=0", "other=1=2"], ConfigError::InvalidAttempts),
            (&["delay_ms=-1", "attempts=0"], ConfigError::InvalidDelay),
            (&["other=1", "attempts=11"], ConfigError::UnknownKey),
            (&["other=1", "attempts=1=2"], ConfigError::UnknownKey),
            (&["", "other=1"], ConfigError::InvalidFormat),
            (
                &["attempts=3", "attempts=nope", "other=1=2"],
                ConfigError::DuplicateKey,
            ),
            (
                &["delay_ms=250", "other=1", "attempts=0"],
                ConfigError::UnknownKey,
            ),
        ];

        for (scenario, expected) in scenarios {
            let input = input_lines(scenario);

            assert_eq!(
                read_settings(&input).as_ref().unwrap_err(),
                expected,
                "input: {input:?}"
            );
        }
    }

    #[test]
    fn every_error_displays_its_original_code() {
        let scenarios = [
            (ConfigError::DuplicateKey, "duplicate_key"),
            (ConfigError::InvalidFormat, "invalid_format"),
            (ConfigError::InvalidAttempts, "invalid_attempts"),
            (ConfigError::InvalidDelay, "invalid_delay"),
            (ConfigError::UnknownKey, "unknown_key"),
        ];

        for (error, expected) in scenarios {
            assert_eq!(error.to_string(), expected);
        }
    }

    #[test]
    fn input_remains_unchanged_and_reusable_after_success_or_failure() {
        let scenarios = [
            (
                [" attempts = 4 ", "delay_ms=250"],
                Ok(RetryConfig {
                    attempts: 4,
                    delay_ms: 250,
                }),
            ),
            (
                ["attempts=3", "attempts=nope"],
                Err(ConfigError::DuplicateKey),
            ),
        ];

        for (scenario, expected) in scenarios {
            let input = input_lines(&scenario);
            let original = input.clone();

            assert_eq!(read_settings(&input), expected);
            assert_eq!(input, original);
            assert_eq!(read_settings(&input), expected);
        }
    }
}
