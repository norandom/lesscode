//! Normative adapter traits and carrier contracts for multi-language deconvolution.
//!
//! Governed by:
//! - `.kiro/specs/native-deconvolution-closure/adapter-contract.md`
//! - REQ-NAT-1.1, REQ-NAT-3.1, REQ-NAT-5.1, REQ-NAT-6.3

use serde::{Deserialize, Serialize};

/// High-level 4-outcome verification result.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "status", content = "payload", rename_all = "snake_case")]
pub enum VerificationResult {
    Verified { evidence: String, domain: String },
    Divergent { witness: String },
    Unverified { reason: String },
    ExecutionFailed { failure: String },
}

/// Discrete tagged value carrier for finite-domain and stream transport.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", content = "value", rename_all = "snake_case")]
pub enum TaggedValue {
    Null,
    Bool(bool),
    Int64(i64),
    UInt64(u64),
    FloatBits(u64),
    String(String),
    Bytes(Vec<u8>),
    Array(Vec<TaggedValue>),
    Object(Vec<(String, TaggedValue)>),
}

impl TaggedValue {
    pub fn from_f64(f: f64) -> Self {
        TaggedValue::FloatBits(f.to_bits())
    }

    pub fn as_f64(&self) -> Option<f64> {
        match self {
            TaggedValue::FloatBits(b) => Some(f64::from_bits(*b)),
            _ => None,
        }
    }

    pub fn is_nan(&self) -> bool {
        self.as_f64().is_some_and(|f| f.is_nan())
    }
}

/// Unresolved semantic gap or missing coverage reason.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CoverageGap {
    pub reason: String,
    pub location: Option<String>,
}

/// Harness or execution failure details.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExecutionFailure {
    pub step: String,
    pub error: String,
}

/// Declared state frame specification.
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct FrameSpec {
    pub allowed_write_sites: Vec<String>,
    pub permitted_shell_locations: Vec<String>,
    pub failure_invariants: Vec<String>,
}

/// Request for static and dynamic state frame analysis.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AnalysisRequest {
    pub scope_name: String,
    pub source_paths: Vec<String>,
    pub frame_spec: FrameSpec,
    pub effect_profile: String,
}

/// Result of static/dynamic state frame analysis.
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct AnalysisResult {
    pub write_sites: Vec<String>,
    pub permitted_writes: Vec<String>,
    pub violations: Vec<String>,
    pub unverified_gaps: Vec<CoverageGap>,
    pub is_valid: bool,
}

/// Finite domain specification for bounded pure core comparison.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct DomainSpec {
    pub param_names: Vec<String>,
    pub finite_values: Vec<(String, Vec<TaggedValue>)>,
    pub sample_limit: Option<usize>,
}

/// Request for bounded pure-core comparison.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ComparisonRequest {
    pub entrypoint_baseline: String,
    pub entrypoint_candidate: String,
    pub domain: DomainSpec,
    pub timeout_ms: u64,
}

/// AST mutation kind for adverse control generation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum MutationKind {
    MissingFailureState,
    TruncatedStream,
    SilentNone,
    InvertedErrorOrder,
    InvertedCondition,
    BoundaryOffByOne,
}

/// Specification of an adverse mutation control.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MutationSpec {
    pub id: String,
    pub location: String,
    pub kind: MutationKind,
    pub description: String,
}

/// Resulting status of an adverse mutation trial.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum MutantOutcome {
    Killed {
        witness: String,
    },
    Survived,
    Invalid {
        reason: String,
    },
    Timeout,
    InfrastructureFailure {
        reason: String,
    },
    /// Mutants exist for the control, and no test was run against them.
    NotRun {
        mutants: usize,
    },
}

/// Individual mutation trial record.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MutationResult {
    pub spec: MutationSpec,
    pub outcome: MutantOutcome,
}

/// Aggregated mutation sensitivity report.
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct MutationBatch {
    pub results: Vec<MutationResult>,
    pub all_killed: bool,
    pub denominator: usize,
    pub killed_count: usize,
}

/// Process supervisor run request.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RunRequest {
    pub command: String,
    pub args: Vec<String>,
    pub env: Vec<(String, String)>,
    pub cwd: Option<String>,
    pub timeout_ms: u64,
    pub max_output_bytes: usize,
}

/// Ordered runtime event emitted over reserved protocol stream.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RunEvent {
    pub seq: u64,
    pub step: String,
    pub kind: String,
    pub payload: TaggedValue,
}

/// Complete supervisor run outcome.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct RunResult {
    pub exit_code: i32,
    pub events: Vec<RunEvent>,
    pub stdout_bytes: Vec<u8>,
    pub stderr_bytes: Vec<u8>,
    pub success: bool,
    pub failure_reason: Option<String>,
}

/// Core trait: analyze state frame and effect profiles.
pub trait StateFrameAnalyzer {
    fn analyze(&self, request: &AnalysisRequest) -> AnalysisResult;
}

/// Core trait: verify pure-core equivalence over finite domains.
pub trait PureCoreVerifier {
    fn compare(&self, request: &ComparisonRequest) -> VerificationResult;
}

/// Core trait: generate and evaluate adverse mutation sensitivity.
pub trait AdverseMutantGenerator {
    fn generate(&self, request: &MutationRequest) -> MutationBatch;
}

/// Request carrier for adverse mutation batch execution.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MutationRequest {
    pub target_path: String,
    pub controls: Vec<MutationSpec>,
}

/// Core trait: run specimen under supervised lifecycle and capture events.
pub trait SpecimenTraceRunner {
    fn run(&self, request: &RunRequest) -> RunResult;
}
