use vi::engine::coze_refine::parse_sse_output;

fn main() {
    let mut passed = 0;
    let mut failed = 0;

    println!("Test 1: Single Message event");
    let content = r#"id: 0
event: Message
data: {"content":"{\"output\":\"你好\"}"}"#;
    match parse_sse_output(content) {
        Ok(s) => {
            if s == "你好" {
                println!("  PASS: got '{}'", s);
                passed += 1;
            } else {
                println!("  FAIL: expected '你好', got '{}'", s);
                failed += 1;
            }
        }
        Err(e) => {
            println!("  FAIL: {}", e);
            failed += 1;
        }
    }

    println!("\nTest 2: Message + Done (result.json format)");
    let content = r#"id: 0
event: Message
data: {"usage":{"output_count":7,"input_count":134,"token_count":141},"node_is_finish":true,"node_seq_id":"0","node_title":"End","content_type":"text","node_type":"End","node_id":"900001","content":"{\"output\":\"你在嗯啊什么呀？\"}","node_execute_uuid":""}

id: 1
event: Done
data: {"node_execute_uuid":"","debug_url":"https://www.coze.cn/work_flow"}"#;
    match parse_sse_output(content) {
        Ok(s) => {
            if s == "你在嗯啊什么呀？" {
                println!("  PASS: got '{}'", s);
                passed += 1;
            } else {
                println!("  FAIL: expected '你在嗯啊什么呀？', got '{}'", s);
                failed += 1;
            }
        }
        Err(e) => {
            println!("  FAIL: {}", e);
            failed += 1;
        }
    }

    println!("\nTest 3: Multiple Message events (should concat)");
    let content = r#"id: 0
event: Message
data: {"content":"{\"output\":\"第\"}"}

id: 1
event: Message
data: {"content":"{\"output\":\"二\"}"}

id: 2
event: Message
data: {"content":"{\"output\":\"三\"}"}

id: 3
event: Done"#;
    match parse_sse_output(content) {
        Ok(s) => {
            if s == "第二三" {
                println!("  PASS: got '{}'", s);
                passed += 1;
            } else {
                println!("  FAIL: expected '第二三', got '{}'", s);
                failed += 1;
            }
        }
        Err(e) => {
            println!("  FAIL: {}", e);
            failed += 1;
        }
    }

    println!("\nTest 4: Error event");
    let content = r#"id: 0
event: Error
data: {"error_code":4101,"error_message":"permission denied"}"#;
    match parse_sse_output(content) {
        Ok(s) => {
            println!("  FAIL: expected error, got '{}'", s);
            failed += 1;
        }
        Err(e) => {
            if e.to_string().contains("error") {
                println!("  PASS: got error '{}'", e);
                passed += 1;
            } else {
                println!("  FAIL: unexpected error '{}'", e);
                failed += 1;
            }
        }
    }

    println!("\nTest 5: Error event (real result.json - Workflow not found)");
    let content = r#"id: 0
event: Error
data: {"node_execute_uuid":"","error_message":"Workflow not found. Please verify the workflow exists.","error_code":4200}"#;
    match parse_sse_output(content) {
        Ok(s) => {
            println!("  FAIL: expected error, got '{}'", s);
            failed += 1;
        }
        Err(e) => {
            if e.to_string().contains("4200") && e.to_string().contains("Workflow not found") {
                println!("  PASS: got error '{}'", e);
                passed += 1;
            } else {
                println!("  FAIL: unexpected error '{}'", e);
                failed += 1;
            }
        }
    }

    println!();
    println!("Results: {} passed, {} failed", passed, failed);
    if failed > 0 {
        std::process::exit(1);
    }
}
