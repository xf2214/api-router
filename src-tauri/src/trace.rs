use std::time::Instant;

/// A single routing decision recorded in the trace.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct RouteDecision {
    pub step: usize,
    pub action: String, // "select_tier", "select_target", "fallback", "failure", "success"
    pub tier: Option<u32>,
    pub provider_id: Option<String>,
    pub model_name: Option<String>,
    pub error: Option<String>,
    pub elapsed_ms: u64,
}

/// Tracks the complete routing decision chain across multiple tiers/targets.
#[derive(Debug, Clone)]
pub struct RouteTrace {
    pub decisions: Vec<RouteDecision>,
    pub start: Instant,
    /// 检测请求是否为上下文压缩（compact）模式。
    /// 由客户端主动压缩，路由层仅做识别。
    pub is_compact: bool,
}

impl RouteTrace {
    /// Create a new trace with the current instant as the start time.
    pub fn new() -> Self {
        Self {
            decisions: Vec::new(),
            start: Instant::now(),
            is_compact: false,
        }
    }

    /// Set the compact flag for this trace.
    pub fn set_compact(&mut self, compact: bool) {
        self.is_compact = compact;
    }

    /// Record a routing decision, automatically computing the elapsed time
    /// since the trace was created.
    pub fn record(
        &mut self,
        action: &str,
        tier: Option<u32>,
        provider_id: Option<String>,
        model_name: Option<String>,
        error: Option<String>,
    ) {
        let step = self.decisions.len();
        let elapsed_ms = self.start.elapsed().as_millis() as u64;
        self.decisions.push(RouteDecision {
            step,
            action: action.to_string(),
            tier,
            provider_id,
            model_name,
            error,
            elapsed_ms,
        });
    }

    /// Format the trace as a compact JSON string suitable for an HTTP header value.
    /// Includes the `is_compact` flag and the full decision array.
    pub fn to_header_value(&self) -> String {
        let obj = serde_json::json!({
            "is_compact": self.is_compact,
            "decisions": self.decisions,
        });
        serde_json::to_string(&obj).unwrap_or_else(|_| "{}".to_string())
    }
}