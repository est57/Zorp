use super::*;

fn vars(pairs: &[(&str, &str)]) -> Variables {
    Variables::from_layers(vec![
        pairs
            .iter()
            .map(|(k, v)| (k.to_string(), v.to_string()))
            .collect::<Vec<_>>(),
    ])
}

#[test]
fn resolves_nested_and_reports_missing() {
    let v = vars(&[("host", "api.dev"), ("base", "https://{{host}}/v1")]);
    let mut missing = BTreeSet::new();
    assert_eq!(
        v.resolve("{{ base }}/users/{{id}}", &mut missing),
        "https://api.dev/v1/users/{{id}}"
    );
    assert_eq!(missing.into_iter().collect::<Vec<_>>(), vec!["id"]);
}

#[test]
fn later_layers_override_earlier_ones() {
    let v = Variables::from_layers(vec![
        vec![("a".into(), "1".into())],
        vec![("a".into(), "2".into())],
    ]);
    assert_eq!(v.resolve("{{a}}", &mut BTreeSet::new()), "2");
}

#[test]
fn self_reference_terminates() {
    let v = vars(&[("loop", "{{loop}}")]);
    assert_eq!(v.resolve("{{loop}}", &mut BTreeSet::new()), "{{loop}}");
}

#[test]
fn unclosed_braces_are_kept() {
    assert_eq!(vars(&[]).resolve("a {{b", &mut BTreeSet::new()), "a {{b");
}
