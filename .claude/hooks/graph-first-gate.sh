#!/bin/bash
# ============================================================
# GRAPH-FIRST GATE — PreToolUse hook (Bash + Grep)
#
# Enforces graph-BEFORE-grep during diagnostic ground-truth.
# Instruction-level wiring (protocol/agent/command text) was
# proven insufficient: Claude takes the shortest path (grep) and
# never reaches for graphify. This gate makes the environment,
# not the instructions, the enforcer.
#
# RULE (v2): while graph enforcement is armed for this PROJECT:
#   - Sub-agent dispatch (Agent/Task tool) and the Grep TOOL are
#     blocked until at least one code-graph query (blast.py /
#     graphify explain|path) has run — one-shot: existence of
#     .graph_queried satisfies them permanently (Bash-less
#     sub-agents cannot run blast.py; a freshness demand would
#     deadlock them).
#   - Bash recursive code searches (`grep -r`/`rg`) require a graph
#     query within the last 15 MINUTES. A single front-loaded query
#     no longer buys unlimited grep for the whole session — the
#     graph must stay warm while structural work continues.
#
# WHY GATE FAN-OUT: many sub-agents (dimension-scorer, architect,
# codebase-expert) have NO Bash tool and so CANNOT run blast.py to
# satisfy this gate themselves. Gating their Grep would deadlock them.
# Instead, the orchestrator (which HAS Bash) front-loads ONE graph
# query before fanning out; every sub-agent inherits the satisfied
# .graph_queried flag (same session PID) and is never blocked.
#
# BOUNDED / FAIL-SAFE:
#   - No effect unless .diag_active or .graph_enforce is armed.
#   - No effect if the graph can't be queried (no graph.json or no
#     blast.py) — falls back to grep silently, never blocks.
#   - Armed flags older than 4h stop enforcing (staleness bound).
#   - PROJECT-scoped, not PID-scoped (v2). PID scoping exempted
#     continuation sessions (resume, second terminal, post-restart)
#     from Rule 28 forever — a live target-project session
#     grep-answered three structural questions unblocked while the
#     armed flag belonged to another PID. The tax on an unrelated
#     parallel session in the same project is one blast.py query,
#     and Rule 28 applies to every agent and command anyway.
#
# PATH ANCHORING (v3): flag files are resolved relative to THIS
# script's own directory (.claude/hooks/), NOT via CLAUDE_PROJECT_DIR.
# CLAUDE_PROJECT_DIR falls back to cwd `.` when unset, and cwd is not
# guaranteed identical across tool invocations — so the blast.py-time
# touch and the Agent-dispatch-time check could target DIFFERENT
# .graph_queried files. A genuinely-queried graph then read as "not
# queried" and the gate blocked the dispatch anyway. Anchoring to $0
# makes write and read always resolve the same path.
# ============================================================

INPUT=$(cat)

# --- Anchor to the hook's OWN directory (cwd-independent, see v3 note) ---
HOOK_DIR="$(cd "$(dirname "$0")" 2>/dev/null && pwd)"
[ -n "$HOOK_DIR" ] || HOOK_DIR="${CLAUDE_PROJECT_DIR:-.}/.claude/hooks"
PROJECT_DIR="$(cd "$HOOK_DIR/../.." 2>/dev/null && pwd)"
[ -n "$PROJECT_DIR" ] || PROJECT_DIR="${CLAUDE_PROJECT_DIR:-.}"
DIAG_FLAG="$HOOK_DIR/.diag_active"          # doctor/redesign
ENFORCE_FLAG="$HOOK_DIR/.graph_enforce"     # improve/new-feature/audit/...
GRAPH_QUERIED="$HOOK_DIR/.graph_queried"

# --- No graph-enforcing session active → no enforcement ---
{ [ -f "$DIAG_FLAG" ] || [ -f "$ENFORCE_FLAG" ]; } || exit 0

get_mtime() {
    if [ "$(uname)" = "Darwin" ]; then
        stat -f %m "$1" 2>/dev/null || echo "0"
    else
        stat -c %Y "$1" 2>/dev/null || echo "0"
    fi
}

# --- Staleness: an armed flag older than 4h no longer enforces ---
ACTIVE_FLAG="$ENFORCE_FLAG"
[ -f "$DIAG_FLAG" ] && ACTIVE_FLAG="$DIAG_FLAG"
FLAG_AGE=$(( $(date +%s) - $(get_mtime "$ACTIVE_FLAG") ))
[ "$FLAG_AGE" -gt 14400 ] && exit 0

# --- Parse the tool call ---
PARSED=$(echo "$INPUT" | python3 -c "
import sys, json
try:
    d = json.load(sys.stdin); ti = d.get('tool_input', {})
    print(d.get('tool_name', ''))
    print(ti.get('command', ''))
except Exception:
    print(''); print('')
" 2>/dev/null || printf '\n\n')
TOOL=$(echo "$PARSED" | sed -n '1p')
CMD=$(echo "$PARSED" | sed -n '2p')

# --- A code-graph query satisfies graph-first: record it UNCONDITIONALLY ---
# MUST run before the graph-resolution early-exit below. Running blast.py IS
# the graph being queried, whether or not THIS gate can independently
# re-resolve graph.json from its working directory. Gating the record behind
# that resolution let a real query go unrecorded, then blocked the next
# dispatch. Detect → touch → allow, no preconditions.
if printf '%s' "$CMD" | grep -qE 'blast\.py|graphify[[:space:]]+(explain|path)'; then
    touch "$GRAPH_QUERIED"
    exit 0
fi

# --- Resolve the graph; if unqueryable, do NOT enforce (grep fallback) ---
GRAPH=$(ls "$PROJECT_DIR"/graphify-out/graph.json "$PROJECT_DIR"/.omega/graph/graph.json 2>/dev/null | head -1)
BL="$PROJECT_DIR/.claude/scripts/blast.py"
{ [ -n "$GRAPH" ] && [ -f "$BL" ]; } || exit 0
GRAPHIFY=$(command -v graphify 2>/dev/null || ls "$HOME/.graphify-venv/bin/graphify" 2>/dev/null | head -1)

# Claude Code reads stderr for exit-2 block reasons.
exec 1>&2

# --- Satisfaction state (v2): existence vs freshness ---
# Existence one-shot-satisfies the Grep TOOL and sub-agent dispatch
# (Bash-less sub-agents cannot run blast.py — a freshness demand would
# deadlock them). Bash recursive greps require a FRESH query (15 min):
# a single front-loaded query must not buy unlimited grep for hours.
FRESH_WINDOW=900
GQ_EXISTS=0
GQ_FRESH=0
GQ_AGE=-1
if [ -f "$GRAPH_QUERIED" ]; then
    GQ_EXISTS=1
    GQ_AGE=$(( $(date +%s) - $(get_mtime "$GRAPH_QUERIED") ))
    [ "$GQ_AGE" -le "$FRESH_WINDOW" ] && GQ_FRESH=1
fi

# --- Fan-out gate: consult the graph BEFORE dispatching sub-agents ---
# The orchestrator has Bash and must front-load ONE graph query so
# Bash-less sub-agents inherit a satisfied flag instead of hitting an
# unsatisfiable demand on their first Grep. Tool is named "Agent" in
# current Claude Code, "Task" in older versions — accept both.
if [ "$TOOL" = "Task" ] || [ "$TOOL" = "Agent" ]; then
    [ "$GQ_EXISTS" = "1" ] && exit 0
    echo "🕸️  GRAPH-FIRST GATE — consult the graph before fanning out sub-agents"
    echo ""
    echo "A graph-enforced command is active and you are about to dispatch a sub-agent, but the"
    echo "code graph has not been queried yet this session. Many sub-agents (dimension-scorer,"
    echo "architect, codebase-expert) have NO Bash tool and cannot run blast.py themselves — so"
    echo "the orchestrator must front-load ONE graph query NOW. It sets a session flag every"
    echo "sub-agent inherits; you only do this once, then re-dispatch freely."
    echo ""
    echo "Run ONE of these on the feature/bug's primary symbol or file, then re-dispatch:"
    echo "  python3 $BL \"$GRAPH\" \"<SYMBOL_OR_FILE>\" --hops 2     # who depends on / is affected by it"
    if [ -n "$GRAPHIFY" ]; then
    echo "  \"$GRAPHIFY\" explain \"<SYMBOL>\" --graph \"$GRAPH\"     # directed callers (<--) / callees (-->)"
    fi
    echo ""
    echo "This makes complexity/risk/blast-radius reasoning graph-grounded from the first step."
    echo "See .claude/protocols/graph-briefing.md."
    exit 2
fi

# --- Grep TOOL: one-shot semantics (sub-agent safe) ---
if [ "$TOOL" = "Grep" ]; then
    [ "$GQ_EXISTS" = "1" ] && exit 0
    echo "🕸️  GRAPH-FIRST GATE (graph-enforced command active)"
    echo ""
    echo "Code search is blocked until the code graph has been queried at least once."
    echo "Reconstruct structure from the graph, not by scanning files — the graph"
    echo "returns typed, deduplicated callers/callees/dependents far cheaper than grep+read."
    echo ""
    echo "If you have Bash, run ONE of these first (then search unblocks):"
    echo "  python3 $BL \"$GRAPH\" \"<SYMBOL_OR_FILE>\" --hops 2     # who depends on / is affected by it"
    if [ -n "$GRAPHIFY" ]; then
    echo "  \"$GRAPHIFY\" explain \"<SYMBOL>\" --graph \"$GRAPH\"     # directed callers (<--) / callees (-->)"
    fi
    echo ""
    echo "See .claude/protocols/graph-briefing.md."
    exit 2
fi

# --- Bash recursive code search (structure reconstruction): freshness-gated ---
[ "$TOOL" = "Bash" ] || exit 0
IS_SEARCH=0
# recursive grep: a flag cluster containing r/R anywhere (-r, -rn, -nr, -Rl...) or --recursive
printf '%s' "$CMD" | grep -qE '(^|[;&|[:space:]])grep[[:space:]]+(-[A-Za-z]*[rR][A-Za-z]*|--recursive)' && IS_SEARCH=1
# ripgrep (recursive by default)
printf '%s' "$CMD" | grep -qE '(^|[;&|[:space:]])(rg|ripgrep)([[:space:]]|$)' && IS_SEARCH=1
[ "$IS_SEARCH" = "1" ] || exit 0

[ "$GQ_FRESH" = "1" ] && exit 0

# --- Block: graph-first (initial or stale window) ---
echo "🕸️  GRAPH-FIRST GATE (graph-enforced command active)"
echo ""
if [ "$GQ_EXISTS" = "1" ]; then
    GQ_MIN=$(( GQ_AGE / 60 ))
    echo "Your last code-graph query was ${GQ_MIN} minutes ago (window: 15 min). Structural"
    echo "work has moved on since then — re-ground in the graph before the next recursive"
    echo "code search. One query on the symbol you are about to grep for re-opens the window."
else
    echo "Recursive code search is blocked until you query the code graph. Reconstruct"
    echo "structure from the graph, not by scanning files — the graph returns typed,"
    echo "deduplicated callers/callees/dependents far cheaper than grep+read."
fi
echo ""
echo "Run ONE of these first (re-opens the 15-minute window for grep/rg):"
echo "  python3 $BL \"$GRAPH\" \"<SYMBOL_OR_FILE>\" --hops 2     # who depends on / is affected by it"
if [ -n "$GRAPHIFY" ]; then
echo "  \"$GRAPHIFY\" explain \"<SYMBOL>\" --graph \"$GRAPH\"     # directed callers (<--) / callees (-->)"
echo "  \"$GRAPHIFY\" path \"<A>\" \"<B>\" --graph \"$GRAPH\"      # confirm a causal-chain link A→B"
fi
echo ""
echo "Then grep/rg is allowed (to read the specific code the graph points to). See"
echo ".claude/protocols/graph-briefing.md. This gate exists because graph use wired as"
echo "instructions is skipped for grep — only the environment can enforce graph-before-grep."
exit 2
