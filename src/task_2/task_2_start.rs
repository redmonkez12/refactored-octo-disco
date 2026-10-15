fn read_settings(v: Vec<String>) -> Result<(u32, u64), String> {
    let mut a = 3;
    let mut d = 1000;
    let mut seen_a = false;
    let mut seen_d = false;

    for i in 0..v.len() {
        let raw = v[i].clone();
        let parts: Vec<&str> = raw.split('=').collect();

        if parts.len() != 2 {
            return Err("invalid_format".to_string());
        }

        let key = parts[0].trim().to_string();
        let value = parts[1].trim().to_string();

        if key == "attempts" {
            if seen_a {
                return Err("duplicate_key".to_string());
            }
            seen_a = true;

            match value.parse::<u32>() {
                Ok(number) => {
                    if number < 1 || number > 10 {
                        return Err("invalid_attempts".to_string());
                    }
                    a = number;
                }
                Err(_) => return Err("invalid_attempts".to_string()),
            }
        } else if key == "delay_ms" {
            if seen_d {
                return Err("duplicate_key".to_string());
            }
            seen_d = true;

            match value.parse::<u64>() {
                Ok(number) => {
                    if number > 60_000 {
                        return Err("invalid_delay".to_string());
                    }
                    d = number;
                }
                Err(_) => return Err("invalid_delay".to_string()),
            }
        } else {
            return Err("unknown_key".to_string());
        }
    }

    Ok((a, d))
}

fn main() {
    let input = vec![" attempts = 4 ".to_string(), "delay_ms=250".to_string()];

    match read_settings(input) {
        Ok((attempts, delay)) => {
            println!("attempts={attempts};delay_ms={delay}");
        }
        Err(error) => println!("error={error}"),
    }
}
