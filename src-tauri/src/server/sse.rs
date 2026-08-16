//! 服务端发送事件（SSE）行缓冲解析器。
//!
//! 背景：原先的流式转发使用 `bytes_stream().map(|chunk| { ... lines().map(...) })`
//! 直接对每个 chunk 按行处理。但 TCP 分包可能将一行 SSE 事件切成多个 chunk，
//! 也会把多行粘在同一个 chunk 中。这会导致：
//! - `data: {...}` 被切到两个 chunk 时 JSON 解析失败，整行被原样透传；
//! - `transform_stream_chunk` 拿到不完整 JSON，无法重写 model 字段；
//! - 极端情况下响应被破坏。
//!
//! 本模块维护一个内部行缓冲区，仅在收到完整行（以 `\n` 结尾）时才输出处理后的内容；
//! 收到流末尾时把残留缓冲也输出。保证交给下游的字节流仍是合法的 SSE 文本。

use crate::metrics::TokenUsage;
use crate::transform::transform_stream_chunk;

/// SSE 行缓冲处理器。
#[derive(Debug, Default)]
pub struct SseLineBuffer {
    /// 上一次输出后剩余的不完整行。
    pending: String,
    /// 本地模型名，用于重写响应中的 `model` 字段。
    local_model: String,
    /// 累积的 token usage（OpenAI 流式通常在最后一个 chunk 中包含 usage）。
    accumulated_usage: Option<TokenUsage>,
}

impl SseLineBuffer {
    /// 用指定本地模型名构造缓冲。
    pub fn with_local_model(local_model: String) -> Self {
        Self {
            pending: String::new(),
            local_model,
            accumulated_usage: None,
        }
    }

    /// 处理一个输入 chunk，返回可直接转发给客户端的字节序列。
    ///
    /// 算法：
    /// 1. 把 chunk 以 UTF-8 lossy 追加到 `pending`；
    /// 2. 按 `\n` 切分，最后一段可能不完整，保留在 `pending`；
    /// 3. 对每个完整行调用 `transform_stream_chunk`，再以 `\n` 重新拼接输出。
    pub fn push(&mut self, chunk: &[u8]) -> Vec<u8> {
        let text = String::from_utf8_lossy(chunk);
        self.pending.push_str(&text);

        // 如果缓冲里完全没有换行，等下次再处理
        if !self.pending.contains('\n') {
            return Vec::new();
        }

        // 取出 pending 的所有权，避免借用冲突
        let full = std::mem::take(&mut self.pending);
        // split 总会返回至少 1 个元素；最后一个元素之后没有 `\n`，是潜在的不完整行
        let mut lines: Vec<&str> = full.split('\n').collect();
        // 把最后一段（不完整行）放回 pending
        let trailing = lines.pop().unwrap_or_default();
        self.pending.push_str(trailing);

        let mut out = String::with_capacity(self.pending.len() + 64);
        for line in lines {
            // 检测并累积 usage（OpenAI 流式通常在最后一个 chunk 中包含 usage）
            if let Some(usage) = extract_usage_from_sse_line(line) {
                self.accumulated_usage = Some(usage);
            }
            let transformed = transform_stream_chunk(line, &self.local_model);
            out.push_str(&transformed);
            out.push('\n');
        }
        out.into_bytes()
    }

    /// 流结束时，把残留缓冲以最后一行形式输出（如果非空）。
    pub fn flush(&mut self) -> Vec<u8> {
        if self.pending.is_empty() {
            return Vec::new();
        }
        let line = std::mem::take(&mut self.pending);
        // 残留行也可能包含 usage
        if let Some(usage) = extract_usage_from_sse_line(&line) {
            self.accumulated_usage = Some(usage);
        }
        let transformed = transform_stream_chunk(&line, &self.local_model);
        let mut out = transformed;
        out.push('\n');
        out.into_bytes()
    }

    /// 取出累积的 token usage（用于流结束后记录 metrics）。
    pub fn take_usage(&mut self) -> Option<TokenUsage> {
        self.accumulated_usage.take()
    }
}

/// 从一行 SSE 文本中提取 usage 字段。
fn extract_usage_from_sse_line(line: &str) -> Option<TokenUsage> {
    let payload = line.strip_prefix("data:")?;
    let payload = payload.strip_prefix(' ').unwrap_or(payload);
    if payload.trim() == "[DONE]" {
        return None;
    }
    let value: serde_json::Value = serde_json::from_str(payload).ok()?;
    crate::metrics::extract_usage(&value)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn splits_complete_lines_only() {
        let mut buf = SseLineBuffer::with_local_model("local-model".to_string());

        // 第一段切断了一行
        let out1 = buf.push(b"data: {\"model\":\"upstr");
        // 没有换行，不应该输出
        assert!(out1.is_empty(), "expected no output for partial line");

        let out2 = buf.push(b"eam\",\"content\":\"hi\"}\n\n");
        let s = String::from_utf8(out2).unwrap();
        // 应当包含 model 被重写后的版本
        assert!(s.contains("\"model\":\"local-model\""), "got: {s}");
        // 末尾的空行也要保留（SSE 协议要求事件之间空行）
        assert!(s.ends_with("\n\n"), "should preserve SSE blank line");
    }

    #[test]
    fn handles_data_done() {
        let mut buf = SseLineBuffer::with_local_model("local-model".to_string());

        let out = buf.push(b"data: [DONE]\n");
        let s = String::from_utf8(out).unwrap();
        // [DONE] 行应当原样输出
        assert!(s.contains("data: [DONE]"));
    }

    #[test]
    fn non_data_lines_pass_through() {
        let mut buf = SseLineBuffer::with_local_model("local-model".to_string());

        let out = buf.push(b": heartbeat\n");
        let s = String::from_utf8(out).unwrap();
        assert!(s.contains(": heartbeat"));
    }

    #[test]
    fn flush_emits_remaining() {
        let mut buf = SseLineBuffer::with_local_model("local-model".to_string());

        let _ = buf.push(b"data: {\"model\":\"upstream\"}");
        // 没换行，不输出
        let flushed = buf.flush();
        let s = String::from_utf8(flushed).unwrap();
        assert!(s.contains("\"model\":\"local-model\""));
    }

    #[test]
    fn extracts_usage_from_final_sse_chunk() {
        let mut buf = SseLineBuffer::with_local_model("local-model".to_string());

        let chunk = br#"data: {"model":"upstream","choices":[],"usage":{"prompt_tokens":5,"completion_tokens":3,"total_tokens":8}}
"#;
        let out = buf.push(chunk);
        let s = String::from_utf8(out).unwrap();
        assert!(s.contains("\"model\":\"local-model\""));

        let flushed = buf.flush();
        assert!(flushed.is_empty(), "no pending data after newline");

        let usage = buf.take_usage().expect("usage should be extracted");
        assert_eq!(usage.prompt_tokens, 5);
        assert_eq!(usage.completion_tokens, 3);
        assert_eq!(usage.total_tokens, 8);
    }

    #[test]
    fn extracts_usage_without_space_prefix() {
        let mut buf = SseLineBuffer::with_local_model("m".to_string());

        let chunk = br#"data:{"usage":{"prompt_tokens":2,"completion_tokens":1,"total_tokens":3}}
"#;
        let _ = buf.push(chunk);

        let usage = buf.take_usage().expect("usage should be extracted without space");
        assert_eq!(usage.total_tokens, 3);
    }
}
