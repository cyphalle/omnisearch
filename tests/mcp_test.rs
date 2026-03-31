use std::io::Write;
use std::process::{Command, Stdio};

#[test]
fn test_mcp_initialize_and_list_tools() {
    let mut child = Command::new("cargo")
        .args(["run", "--", "mcp"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .expect("Failed to start omnisearch mcp");

    let stdin = child.stdin.as_mut().unwrap();

    // Send initialize
    writeln!(
        stdin,
        r#"{{"jsonrpc":"2.0","id":1,"method":"initialize","params":{{}}}}"#
    )
    .unwrap();
    // Send tools/list
    writeln!(
        stdin,
        r#"{{"jsonrpc":"2.0","id":2,"method":"tools/list","params":{{}}}}"#
    )
    .unwrap();
    // Close stdin to let server exit
    drop(child.stdin.take());

    let output = child.wait_with_output().unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);
    let lines: Vec<&str> = stdout.lines().collect();

    assert!(
        lines.len() >= 2,
        "Expected at least 2 response lines, got: {stdout}"
    );

    // Check initialize response
    let init_resp: serde_json::Value = serde_json::from_str(lines[0]).unwrap();
    assert_eq!(init_resp["result"]["serverInfo"]["name"], "omnisearch");

    // Check tools/list response
    let tools_resp: serde_json::Value = serde_json::from_str(lines[1]).unwrap();
    let tools = tools_resp["result"]["tools"].as_array().unwrap();
    let tool_names: Vec<&str> = tools.iter().map(|t| t["name"].as_str().unwrap()).collect();
    assert!(tool_names.contains(&"search"));
    assert!(tool_names.contains(&"list_sources"));
}
