use serde_json::Value;

use crate::config::ModelTarget;

/// 检测请求是否为上下文压缩（compact）模式。
///
/// 路由层不做主动压缩，而是识别客户端已经做过的压缩操作。
/// 检测信号（参考 cc-switch 的 copilot_optimizer.rs）：
/// 1. system prompt 以压缩/摘要类专用前缀开头
/// 2. 最后一条用户消息包含特定的压缩标记
///
/// 检测结果可用于计费优化、路由降级等场景。
pub fn is_compact_request(body: &Value) -> bool {
    // 信号 1: system prompt 以压缩专用前缀开头
    if let Some(sys) = body.get("system").and_then(|v| v.as_str()) {
        if sys.starts_with("You are a helpful AI assistant tasked with summarizing conversations")
            || sys.starts_with("You are an AI assistant that reviews and summarizes")
            || sys.contains("以下是对对话历史的压缩摘要")
        {
            return true;
        }
    }

    // 信号 2: 检查最后一条用户消息中的压缩标记
    let messages = match body.get("messages").and_then(|m| m.as_array()) {
        Some(msgs) if !msgs.is_empty() => msgs,
        _ => return false,
    };

    let last_msg = match messages.last() {
        Some(msg) => msg,
        None => return false,
    };

    if last_msg.get("role").and_then(|r| r.as_str()) != Some("user") {
        return false;
    }

    let text = extract_text_from_message(last_msg);
    // 机器指令标记
    if text.contains("CRITICAL: Respond with TEXT ONLY. Do NOT call any tools.") {
        return true;
    }
    // 结构标记（同时出现才算）
    if text.contains("Pending Tasks:") && text.contains("Current Work:") {
        return true;
    }

    false
}

/// 从消息中提取文本内容，兼容 string 和 array 两种 content 格式。
fn extract_text_from_message(msg: &Value) -> String {
    match msg.get("content") {
        Some(Value::String(s)) => s.clone(),
        Some(Value::Array(blocks)) => {
            let mut text = String::new();
            for block in blocks {
                if let Some(content) = block.get("content").and_then(|c| c.as_str()) {
                    if !text.is_empty() {
                        text.push('\n');
                    }
                    text.push_str(content);
                } else if let Some(Value::String(s)) = block.get("text") {
                    if !text.is_empty() {
                        text.push('\n');
                    }
                    text.push_str(s);
                }
            }
            text
        }
        _ => String::new(),
    }
}

/// Transform an incoming request body for the upstream provider.
/// For OpenAI-compatible providers this rewrites the `model` field and
/// strips empty/null fields that some providers (e.g. SenseNova) reject.
/// 若 target 配置了 override_params，会合并/覆盖到请求体中。
pub fn transform_request(body: &mut Value, target: &ModelTarget) {
    let upstream_model = &target.model_name;
    if let Some(obj) = body.as_object_mut() {
        obj.insert("model".to_string(), Value::String(upstream_model.to_string()));

        // 应用 target 级别的参数覆盖
        if let Some(overrides) = &target.override_params {
            for (k, v) in overrides {
                obj.insert(k.clone(), v.clone());
            }
        }

        obj.retain(|k, v| {
            if v.is_null() {
                return false;
            }
            // 去掉空数组、空字符串
            if is_empty_value(v) {
                return false;
            }
            // max_tokens 为 0 或无意义小数时移除，避免 OUT_OF_RANGE
            if k == "max_tokens" {
                return v.as_u64().unwrap_or(0) > 0;
            }
            true
        });
    }
}

fn is_empty_value(v: &Value) -> bool {
    match v {
        Value::String(s) => s.is_empty(),
        Value::Array(a) => a.is_empty(),
        _ => false,
    }
}

/// Rewrite the `model` field in an upstream response back to the local model alias.
pub fn transform_response(body: &mut Value, local_model: &str) {
    if let Some(obj) = body.as_object_mut() {
        obj.insert("model".to_string(), Value::String(local_model.to_string()));
    }
}

/// Rewrite model identifiers in a streaming SSE chunk if present.
/// OpenAI streaming chunks are JSON objects prefixed with `data: `.
pub fn transform_stream_chunk(chunk: &str, local_model: &str) -> String {
    if !chunk.starts_with("data: ") {
        return chunk.to_string();
    }

    let payload = &chunk[6..];
    if payload.trim() == "[DONE]" {
        return chunk.to_string();
    }

    match serde_json::from_str::<Value>(payload) {
        Ok(mut value) => {
            transform_response(&mut value, local_model);
            format!("data: {}", serde_json::to_string(&value).unwrap_or_else(|_| payload.to_string()))
        }
        Err(_) => chunk.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn target(model_name: &str) -> ModelTarget {
        ModelTarget {
            provider_id: "test".to_string(),
            model_name: model_name.to_string(),
            weight: 1,
            override_params: None,
            tier: 1,
        }
    }

    #[test]
    fn rewrites_model_and_removes_nulls() {
        let mut body = json!({
            "model": "local-glm",
            "messages": [{"role": "user", "content": "hi"}],
            "temperature": null,
            "top_p": 0.9
        });
        transform_request(&mut body, &target("glm-5.2"));
        assert_eq!(body["model"], "glm-5.2");
        assert!(!body.as_object().unwrap().contains_key("temperature"));
        assert_eq!(body["top_p"], 0.9);
    }

    #[test]
    fn removes_empty_arrays_and_strings() {
        let mut body = json!({
            "model": "local-glm",
            "messages": [{"role": "user", "content": "hi"}],
            "tools": [],
            "tool_choice": "",
            "stream_options": {}
        });
        transform_request(&mut body, &target("glm-5.2"));
        assert!(!body.as_object().unwrap().contains_key("tools"));
        assert!(!body.as_object().unwrap().contains_key("tool_choice"));
        assert!(body.as_object().unwrap().contains_key("stream_options"));
    }

    #[test]
    fn removes_zero_max_tokens() {
        let mut body = json!({
            "model": "local-glm",
            "messages": [{"role": "user", "content": "hi"}],
            "max_tokens": 0
        });
        transform_request(&mut body, &target("glm-5.2"));
        assert!(!body.as_object().unwrap().contains_key("max_tokens"));
    }

    #[test]
    fn keeps_positive_max_tokens() {
        let mut body = json!({
            "model": "local-glm",
            "messages": [{"role": "user", "content": "hi"}],
            "max_tokens": 10
        });
        transform_request(&mut body, &target("glm-5.2"));
        assert_eq!(body["max_tokens"], 10);
    }

    #[test]
    fn applies_override_params() {
        let mut body = json!({
            "model": "local-glm",
            "messages": [{"role": "user", "content": "hi"}],
            "temperature": 0.5
        });
        let target = ModelTarget {
            provider_id: "test".to_string(),
            model_name: "glm-5.2".to_string(),
            weight: 1,
            override_params: Some({
                let mut m = serde_json::Map::new();
                m.insert("temperature".to_string(), json!(0.2));
                m.insert("max_tokens".to_string(), json!(100));
                m
            }),
            tier: 1,
        };
        transform_request(&mut body, &target);
        assert_eq!(body["model"], "glm-5.2");
        assert_eq!(body["temperature"], 0.2);
        assert_eq!(body["max_tokens"], 100);
    }

    // ── compact 检测测试 ──────────────────────────────────────────────────

    #[test]
    fn compact_detected_by_system_prompt() {
        let body = json!({
            "model": "claude-sonnet",
            "system": "You are a helpful AI assistant tasked with summarizing conversations. Keep it concise.",
            "messages": [{"role": "user", "content": "hello"}]
        });
        assert!(is_compact_request(&body));
    }

    #[test]
    fn compact_detected_by_critical_instruction() {
        let body = json!({
            "model": "claude-sonnet",
            "messages": [
                {"role": "user", "content": "CRITICAL: Respond with TEXT ONLY. Do NOT call any tools.\nSummarize the above."}
            ]
        });
        assert!(is_compact_request(&body));
    }

    #[test]
    fn compact_detected_by_pending_current_work() {
        let body = json!({
            "model": "claude-sonnet",
            "messages": [
                {"role": "user", "content": "Pending Tasks:\n- Fix bug\n- Add feature\n\nCurrent Work:\nWorking on routing"}
            ]
        });
        assert!(is_compact_request(&body));
    }

    #[test]
    fn compact_detected_by_chinese_system_prompt() {
        let body = json!({
            "model": "glm-5",
            "system": "以下是对对话历史的压缩摘要",
            "messages": [{"role": "user", "content": "继续"}]
        });
        assert!(is_compact_request(&body));
    }

    #[test]
    fn compact_not_detected_for_normal_request() {
        let body = json!({
            "model": "gpt-4",
            "messages": [{"role": "user", "content": "What is Rust?"}]
        });
        assert!(!is_compact_request(&body));
    }

    #[test]
    fn compact_not_detected_for_empty_messages() {
        let body = json!({
            "model": "gpt-4",
            "messages": []
        });
        assert!(!is_compact_request(&body));
    }

    #[test]
    fn compact_detected_with_array_content_blocks() {
        let body = json!({
            "model": "claude-sonnet",
            "messages": [
                {"role": "user", "content": [
                    {"type": "text", "text": "Pending Tasks:\n- Item 1"},
                    {"type": "text", "text": "Current Work:\n- Working on X"}
                ]}
            ]
        });
        assert!(is_compact_request(&body));
    }

    #[test]
    fn compact_not_detected_when_last_message_is_assistant() {
        let body = json!({
            "model": "claude-sonnet",
            "messages": [
                {"role": "user", "content": "hello"},
                {"role": "assistant", "content": "Pending Tasks:\n- test"}
            ]
        });
        assert!(!is_compact_request(&body));
    }
}
