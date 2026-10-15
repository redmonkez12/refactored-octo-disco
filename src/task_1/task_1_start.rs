fn report(a: Vec<(String, String)>) -> Vec<String> {
    let mut b = Vec::new();

    for i in 0..a.len() {
        if a[i].1 == "error" {
            let mut found = false;

            for j in 0..b.len() {
                if b[j] == a[i].0 {
                    found = true;
                }
            }

            if found == false {
                b.push(a[i].0.clone());
            }
        }
    }

    b.sort();

    let mut c = Vec::new();

    for i in 0..b.len() {
        let mut n = 0usize;

        for j in 0..a.len() {
            if a[j].0 == b[i] {
                if a[j].1 == "error" {
                    n += 1;
                }
            }
        }

        c.push(format!("{}: {}", b[i], n));
    }

    c
}

fn main() {
    let logs = vec![
        ("payments".into(), "error".into()),
        ("api".into(), "info".into()),
        ("api".into(), "error".into()),
        ("payments".into(), "error".into()),
    ];

    println!("{:?}", report(logs));
}
