fn put(
    c: &mut Vec<(String, String)>,
    n: usize,
    k: String,
    v: String,
) -> Result<(), String> {
    if k.is_empty() {
        return Err("empty_key".to_string());
    }
    if n == 0 {
        return Err("disabled_cache".to_string());
    }

    let mut found = false;
    for i in 0..c.len() {
        if c[i].0 == k {
            found = true;
        }
    }

    if found == true {
        for i in 0..c.len() {
            if c[i].0 == k {
                c[i].1 = v.clone();
            }
        }
    } else {
        if c.len() >= n {
            c.remove(0);
        }
        c.push((k.clone(), v.clone()));
    }

    Ok(())
}

fn get(c: &Vec<(String, String)>, k: String) -> Option<String> {
    for i in 0..c.len() {
        if c[i].0 == k {
            return Some(c[i].1.clone());
        }
    }

    None
}

fn main() {
    let mut cache = Vec::new();
    let capacity = 2;

    put(&mut cache, capacity, "a".into(), "one".into()).unwrap();
    put(&mut cache, capacity, "b".into(), "two".into()).unwrap();
    put(&mut cache, capacity, "a".into(), "updated".into()).unwrap();
    put(&mut cache, capacity, "c".into(), "three".into()).unwrap();

    println!("a={:?}", get(&cache, "a".into()));
    println!("b={:?}", get(&cache, "b".into()));
    println!("c={:?}", get(&cache, "c".into()));
}