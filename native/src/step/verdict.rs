//! The verdict of a step and the evidence for its spec. A verdict is
//! written for accepted, refused and not judged alike.

use super::context::{Context, Hashes};
use super::functions::Tally;
use crate::adapters::driver::cli::emit_json;
use serde::Serialize;
use std::collections::BTreeMap;

pub const ACCEPTED: &str = "accepted";
pub const REFUSED: &str = "refused";
pub const NOT_JUDGED: &str = "not_judged";

#[derive(Debug, Serialize)]
pub struct Check {
    pub name: &'static str,
    /// `pass`, `refused` or `not_judged`.
    pub result: &'static str,
    pub detail: String,
}

/// One function whose faults were run, before and after the step.
#[derive(Debug, Serialize)]
pub struct FunctionRow {
    pub file: String,
    pub function: String,
    pub before: Option<Tally>,
    pub after: Option<Tally>,
    /// The reason the declaration gives for leaving it unprotected.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub out_of_reach: Option<String>,
    /// The faults the tests do not notice after the step.
    pub unnoticed_after: Vec<String>,
}

/// Tokens and nodes of the scope, before and after.
#[derive(Debug, Default, Serialize)]
pub struct Size {
    pub tokens: [usize; 2],
    pub nodes: [usize; 2],
}

#[derive(Debug, Serialize)]
pub struct Verdict {
    pub receipt: &'static str,
    pub step: String,
    pub spec: String,
    pub base: String,
    pub revision: String,
    pub hashes: Hashes,
    pub checks: Vec<Check>,
    /// Per file, the functions faults were run in.
    pub functions_run: BTreeMap<String, Vec<String>>,
    pub functions: Vec<FunctionRow>,
    /// Functions that exist before or after the step only.
    pub unmatched_functions: Vec<String>,
    pub size: Size,
    pub result: &'static str,
    pub reasons: Vec<String>,
}

impl Verdict {
    pub fn new(context: &Context) -> Verdict {
        Verdict {
            receipt: "step-verdict-v1",
            step: context.declaration.step.clone(),
            spec: context.declaration.spec.clone(),
            base: context.revisions.base.clone(),
            revision: context.revisions.step.clone(),
            hashes: context.hashes_of(&context.step_tree),
            checks: Vec::new(),
            functions_run: BTreeMap::new(),
            functions: Vec::new(),
            unmatched_functions: Vec::new(),
            size: Size::default(),
            result: ACCEPTED,
            reasons: Vec::new(),
        }
    }

    pub fn pass(&mut self, name: &'static str, detail: impl Into<String>) {
        let detail = detail.into();
        self.checks.push(Check {
            name,
            result: "pass",
            detail,
        });
    }

    /// Record the reasons of a refusal; with none, the check passes.
    pub fn settle(&mut self, name: &'static str, reasons: Vec<String>, passed: &str) {
        if reasons.is_empty() {
            return self.pass(name, passed);
        }
        self.checks.push(Check {
            name,
            result: REFUSED,
            detail: reasons.join("; "),
        });
        self.reasons.extend(reasons);
        if self.result == ACCEPTED {
            self.result = REFUSED;
        }
    }

    pub fn not_judged(&mut self, name: &'static str, why: impl Into<String>) {
        let detail = why.into();
        self.reasons.push(detail.clone());
        self.checks.push(Check {
            name,
            result: NOT_JUDGED,
            detail,
        });
        self.result = NOT_JUDGED;
    }

    pub fn judged(&self) -> bool {
        self.result != NOT_JUDGED
    }

    /// 0 accepted, 1 refused, 2 not judged.
    pub fn exit_code(&self) -> u8 {
        match self.result {
            ACCEPTED => 0,
            REFUSED => 1,
            _ => 2,
        }
    }

    pub fn write(&self, context: &Context) -> Result<(), String> {
        let path = context.repo.join(&context.declaration.outputs.verdict);
        emit_json(self, Some(&path))
    }
}

/// What ran for one acceptance criterion of the step's spec.
#[derive(Debug, Serialize)]
pub struct CriterionEvidence {
    pub id: String,
    pub command: Vec<String>,
    pub exit_code: Option<i32>,
    pub passed: bool,
}

/// Executed evidence per criterion. The format is not agreed with the
/// specification tools yet and says so.
#[derive(Debug, Serialize)]
pub struct Evidence<'a> {
    pub receipt: &'static str,
    pub provisional: bool,
    pub step: &'a str,
    pub spec: &'a str,
    pub base: &'a str,
    pub revision: &'a str,
    pub hashes: &'a Hashes,
    pub criteria: Vec<CriterionEvidence>,
}

impl Verdict {
    pub fn write_evidence(
        &self,
        context: &Context,
        criteria: Vec<CriterionEvidence>,
    ) -> Result<(), String> {
        let evidence = Evidence {
            receipt: "step-evidence-provisional-v0",
            provisional: true,
            step: &self.step,
            spec: &self.spec,
            base: &self.base,
            revision: &self.revision,
            hashes: &self.hashes,
            criteria,
        };
        let path = context.repo.join(&context.declaration.outputs.evidence);
        emit_json(&evidence, Some(&path))
    }
}
