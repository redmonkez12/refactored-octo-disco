fn dispatch(a: Vec<(String, String, String)>, n: usize) -> Vec<String> {
    let b = a.clone();
    let mut selected: Vec<(String, String)> = Vec::new();

    for i in 0..b.len() {
        if selected.len() >= n {
            break;
        }

        if b[i].2 == "ready" {
            let mut found = false;

            for j in 0..selected.len() {
                if selected[j].0 == b[i].0 {
                    found = true;
                }
            }

            if found == false {
                selected.push((b[i].0.clone(), b[i].1.clone()));
            }
        }
    }

    let mut result = Vec::new();

    for i in 0..selected.len() {
        result.push(format!("{}@{}", selected[i].0, selected[i].1));
    }

    result
}

fn main() {
    let input: Vec<(String, String, String)> = [
        ("job-7", "emails", "running"),
        ("job-2", "reports", "ready"),
        ("job-2", "urgent", "ready"),
        ("job-7", "emails", "ready"),
        ("job-9", "exports", "ready"),
    ]
    .into_iter()
    .map(|(id, queue, state)| (id.into(), queue.into(), state.into()))
    .collect();

    let result = dispatch(input, 2);
    println!("{result:?}");
    assert_eq!(result, ["job-2@reports", "job-7@emails"]);
}
