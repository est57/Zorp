use crate::models::runner::{RunItemResult, RunSummary};

fn escape(text: &str) -> String {
    text.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

/// One line per request for the terminal.
pub fn text_line(item: &RunItemResult, last_iteration: &mut u32, iterations: u32) -> String {
    let mut out = String::new();
    if iterations > 1 && item.iteration != *last_iteration {
        *last_iteration = item.iteration;
        out.push_str(&format!(
            "Iterasi {}  {}\n",
            item.iteration,
            item.iteration_label.clone().unwrap_or_default()
        ));
    }
    let mark = if item.passed { "LULUS" } else { "GAGAL" };
    let status = item.status.map_or("-".into(), |s| s.to_string());
    let time = item.duration_ms.map_or("-".into(), |d| format!("{d} ms"));
    out.push_str(&format!(
        "  {mark}  {:<7}{:<36}{status:>5}  {time:>8}",
        format!("{:?}", item.method).to_uppercase(),
        item.name
    ));
    if let Some(error) = &item.error {
        out.push_str(&format!("\n          {error}"));
    }
    for test in item.tests.iter().filter(|t| !t.passed) {
        out.push_str(&format!(
            "\n          {}: {}",
            test.name,
            test.error.clone().unwrap_or_default()
        ));
    }
    out
}

pub fn summary_line(s: &RunSummary) -> String {
    let iterations = if s.iterations > 1 {
        format!(" · {} iterasi", s.iterations)
    } else {
        String::new()
    };
    format!(
        "\n{} lulus, {} gagal dari {} request{iterations} · {:.1} dtk",
        s.passed,
        s.failed,
        s.total,
        f64::from(s.duration_ms) / 1000.0
    )
}

pub fn json(name: &str, summary: &RunSummary, items: &[RunItemResult]) -> String {
    serde_json::to_string_pretty(
        &serde_json::json!({ "collection": name, "summary": summary, "items": items }),
    )
    .unwrap_or_default()
}

/// JUnit XML, read by most CI systems to show test results.
pub fn junit(name: &str, summary: &RunSummary, items: &[RunItemResult]) -> String {
    let mut xml = format!(
        "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n<testsuite name=\"{}\" tests=\"{}\" failures=\"{}\" time=\"{:.3}\">\n",
        escape(name),
        summary.total,
        summary.failed,
        f64::from(summary.duration_ms) / 1000.0
    );
    for item in items {
        let case = if summary.iterations > 1 {
            format!("{} (iterasi {})", item.name, item.iteration)
        } else {
            item.name.clone()
        };
        let time = f64::from(item.duration_ms.unwrap_or(0)) / 1000.0;
        xml.push_str(&format!(
            "  <testcase name=\"{}\" time=\"{time:.3}\">",
            escape(&case)
        ));
        if !item.passed {
            let detail: Vec<String> =
                item.error
                    .iter()
                    .cloned()
                    .chain(
                        item.tests.iter().filter(|t| !t.passed).map(|t| {
                            format!("{}: {}", t.name, t.error.clone().unwrap_or_default())
                        }),
                    )
                    .collect();
            xml.push_str(&format!(
                "<failure message=\"{}\"/>",
                escape(&detail.join("; "))
            ));
        }
        xml.push_str("</testcase>\n");
    }
    xml.push_str("</testsuite>\n");
    xml
}
