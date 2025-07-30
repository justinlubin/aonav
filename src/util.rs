use indexmap::IndexMap;

pub fn cartesian_product<K: Clone + Eq + std::hash::Hash, V: Clone>(
    choices: IndexMap<K, Vec<V>>,
) -> Vec<IndexMap<K, V>> {
    let mut results = vec![IndexMap::new()];
    for (k, vs) in choices.iter() {
        let mut new_results = vec![];
        for map in results {
            for v in vs {
                let mut new_map = map.clone();
                new_map.insert(k.clone(), v.clone());
                new_results.push(new_map)
            }
        }
        results = new_results;
    }
    results
}

pub fn read_lines(path: &str) -> Option<Vec<String>> {
    match std::fs::read_to_string(&path) {
        Ok(s) => Some(s.lines().map(String::from).collect()),
        Err(_) => None,
    }
}
