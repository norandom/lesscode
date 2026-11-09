import type { DegradationNotice, DegradationSeverity, WorkflowPhase } from "./types.js"

export type WorkflowRuntimeState = "active" | "degraded" | "blocked" | "disabled"

interface NoticeContext {
  readonly reason: string
  readonly occurredAt?: string
  readonly phase?: WorkflowPhase
}

export interface UnsupportedCapabilityNoticeInput extends NoticeContext {
  readonly capability: string
}
export type MultiAgentUnavailableNoticeInput = NoticeContext
export interface FailedPhaseNoticeInput extends NoticeContext {
  readonly phase: WorkflowPhase
}
export interface PermissionDeniedNoticeInput extends NoticeContext {
  readonly operation: string
}
export type HighEffortLimitationNoticeInput = NoticeContext
export interface DisabledWorkflowNoticeInput {
  readonly reason: string
  readonly occurredAt?: string
}
export interface ToolErrorNoticeInput extends NoticeContext {
  readonly tool: string
  readonly phase: WorkflowPhase
}
export interface WorkflowDegradationSummaryInput {
  readonly enabled: boolean
  readonly notices: readonly DegradationNotice[]
}
export interface WorkflowDegradationSummary {
  readonly state: WorkflowRuntimeState
  readonly message: string
  readonly safeNextAction: string
  readonly notices: readonly DegradationNotice[]
}

const slugify = (v: string): string =>
  v.trim().toLowerCase().replace(/[^a-z0-9]+/g, "-").replace(/^-|-$/g, "") || "unknown"

const mk = (
  c: string, id: string, sev: DegradationSeverity, r: string, msg: string, act: string, occ?: string, ph?: WorkflowPhase
): DegradationNotice => ({
  id, capability: c, severity: sev, reason: r, userVisibleMessage: msg, safeNextAction: act,
  occurredAt: occ ?? new Date().toISOString(), ...(ph === undefined ? {} : { phase: ph }),
})

export const createUnsupportedCapabilityNotice = (i: UnsupportedCapabilityNoticeInput): DegradationNotice =>
  mk(i.capability, `unsupported-${slugify(i.capability)}`, "warning", i.reason,
    `OpenUltraCode cannot use ${i.capability}: ${i.reason}.`,
    "Continue without this capability and use the visible workflow phases to track progress.", i.occurredAt, i.phase)

export const createMultiAgentUnavailableNotice = (i: MultiAgentUnavailableNoticeInput): DegradationNotice =>
  mk("role-based multi-agent execution", "multi-agent-unavailable", "warning", i.reason,
    "Role-based multi-agent execution is unavailable; OpenUltraCode will use a single-session fallback with the same phase structure.",
    "Continue in a single-session workflow using intake, planning, execution, review, reconciliation, and verification phases.",
    i.occurredAt, i.phase)

export const createFailedPhaseNotice = (i: FailedPhaseNoticeInput): DegradationNotice =>
  mk(`workflow phase ${i.phase}`, `phase-${i.phase}-failed`, "blocked", i.reason,
    `The ${i.phase} phase failed: ${i.reason}.`,
    `Stop the workflow at ${i.phase}, fix or explicitly accept the issue, then rerun the phase before reporting completion.`,
    i.occurredAt, i.phase)

export const createToolErrorNotice = (i: ToolErrorNoticeInput): DegradationNotice =>
  mk(`tool ${i.tool}`, `tool-error-${slugify(i.tool)}`, "warning", i.reason,
    `${i.tool} failed during ${i.phase}: ${i.reason}.`,
    `Check whether the ${i.tool} failure affects the result; record a blocked check if required evidence could not be produced.`,
    i.occurredAt, i.phase)

export const createPermissionDeniedNotice = (i: PermissionDeniedNoticeInput): DegradationNotice =>
  mk(`permission for ${i.operation}`, `permission-denied-${slugify(i.operation)}`, "blocked", i.reason,
    `OpenUltraCode cannot perform ${i.operation}: ${i.reason}.`,
    `Respect the permission denial; ask the user to approve ${i.operation} or choose a permitted alternative before retrying.`,
    i.occurredAt, i.phase)

export const createHighEffortLimitationNotice = (i: HighEffortLimitationNoticeInput): DegradationNotice =>
  mk("high-effort provider controls", "high-effort-limitation", "warning", i.reason,
    `High-effort provider controls are unavailable: ${i.reason}.`,
    "Continue with normal provider settings and rely on visible planning, review, and verification discipline.",
    i.occurredAt, i.phase)

export const createDisabledWorkflowNotice = (i: DisabledWorkflowNoticeInput): DegradationNotice =>
  mk("OpenUltraCode workflow mode", "workflow-disabled", "notice", i.reason,
    `OpenUltraCode workflow mode is disabled: ${i.reason}.`,
    "Use normal opencode behavior or enable OpenUltraCode before starting a governed workflow.", i.occurredAt)

const fmt = (n: readonly DegradationNotice[], p = ""): string =>
  `${n.length} ${n.length === 1 ? p + "capability" : p + "capabilities"}: ${n.map((x) => x.capability).join(", ")}`

export function summarizeWorkflowDegradation(i: WorkflowDegradationSummaryInput): WorkflowDegradationSummary {
  if (!i.enabled) {
    return {
      state: "disabled",
      message: "OpenUltraCode is disabled. Normal opencode behavior is unchanged.",
      safeNextAction: "Enable OpenUltraCode before starting a governed workflow.",
      notices: i.notices,
    }
  }
  const blk = i.notices.filter((n) => n.severity === "blocked")
  if (blk.length > 0) {
    return {
      state: "blocked",
      message: `OpenUltraCode is blocked by ${fmt(blk)}.`,
      safeNextAction: blk[0]?.safeNextAction ?? "Resolve the blocked capability before continuing.",
      notices: i.notices,
    }
  }
  const deg = i.notices.filter((n) => n.severity === "warning")
  if (deg.length > 0) {
    return {
      state: "degraded",
      message: `OpenUltraCode is running with ${fmt(deg, "degraded ")}.`,
      safeNextAction: deg[0]?.safeNextAction ?? "Continue with visible workflow checks.",
      notices: i.notices,
    }
  }
  return {
    state: "active",
    message: "OpenUltraCode workflow support is active with no reported degradation.",
    safeNextAction: "Continue the current workflow and collect verification evidence before completion.",
    notices: i.notices,
  }
}
