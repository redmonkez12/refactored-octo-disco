fn plan(
    a: Vec<(String, u16, u32)>,
    max: u32,
) -> Result<Vec<String>, String> {
    if max == 0 {
        return Err("invalid_limit".to_string());
    }

    let b = a.clone();
    let mut result = Vec::new();

    for i in 0..b.len() {
        let endpoint = b[i].0.clone();
        let status = b[i].1;
        let attempts = b[i].2;

        if status < 100 || status > 599 {
            return Err(format!("invalid_status:{i}"));
        }
        if attempts == 0 {
            return Err(format!("invalid_attempts:{i}"));
        }

        let mut action = "ignore".to_string();

        if status >= 200 && status <= 299 {
            action = "accept".to_string();
        } else if status == 429 || (status >= 500 && status <= 599) {
            if attempts < max {
                action = "retry".to_string();
            } else {
                action = "give_up".to_string();
            }
        } else if status >= 400 && status <= 499 {
            action = "reject".to_string();
        }

        result.push(format!("{endpoint}:{action}"));
    }

    Ok(result)
}

fn main() {
    let input: Vec<(String, u16, u32)> = [
        ("/billing", 503, 1),
        ("/export", 429, 3),
        ("/users", 200, 1),
        ("/auth", 401, 1),
        ("/redirect", 302, 1),
    ]
        .into_iter()
        .map(|(endpoint, status, attempts)| (endpoint.into(), status, attempts))
        .collect();

    for line in plan(input, 3).unwrap() {
        println!("{line}");
    }
}