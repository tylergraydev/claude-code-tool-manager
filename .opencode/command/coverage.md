---
description: Enforce 100% test coverage for specified files (project)
---

# Coverage Enforcement - Multi-Phase Workflow

Enforce 100% test coverage through an intelligent multi-phase approach using dedicated agents.

## Arguments

Files are provided in: $ARGUMENTS

Parse each file path from the arguments (space-separated).

## File Path Normalization

Before processing, normalize file paths:
1. Fix double extensions (`.html.html` → `.html`)
2. Resolve relative paths to full paths from `src/ComicRag.Web/ClientApp/src/`
3. Validate files exist using Glob

**Path resolution examples:**
- `app/components/users/users.html` → `src/ComicRag.Web/ClientApp/src/app/components/users/users.html`
- `ClientApp/app/...` → `src/ComicRag.Web/ClientApp/src/app/...`

## File Type Handling

| File Type | Test Strategy | Target Coverage |
|-----------|---------------|-----------------|
| `.ts` (not spec) | Unit tests for logic | 100% line |
| `.html` (Angular) | Template rendering tests | 80%+ (100% is often impractical) |
| `.cs` | Unit/integration tests | 100% line |
| `.spec.ts` | Skip - already tests | N/A |

## Agents Used

| Agent | Purpose | Phase |
|-------|---------|-------|
| `coverage-scanner` | Runs coverage ONCE, extracts per-file stats | Phase 1 |
| `coverage-enforcer` | Writes tests for a single file | Phase 2 |
| `coverage-verifier` | Verifies final coverage, decides on looping | Phase 4 |

---

## Phase 1: Initial Coverage Scan

Spawn a **coverage-scanner** agent to run coverage once for all files:

```
Task({
  subagent_type: "coverage-scanner",
  prompt: `
    Scan coverage for these files:
    ${files.map(f => `- ${f}`).join('\n')}

    Run coverage ONCE and return per-file statistics including:
    - Line coverage %
    - Branch coverage %
    - Uncovered line numbers
    - Recommendation (needs tests / skip / exclude)
  `,
  description: "Scan initial coverage",
  run_in_background: false  // Wait for results before Phase 2
})
```

**Output:** Per-file coverage data to pass to Phase 2 agents.

---

## Phase 2: Per-File Test Writing

For each file that needs work (not at 100%, not pure config), spawn a **coverage-enforcer** agent.

**Spawn ALL agents in parallel** using a single message with multiple Task calls:

### For TypeScript/C# Files:

```
Task({
  subagent_type: "coverage-enforcer",
  prompt: `
    Ensure 100% test coverage for: ${file1.path}

    ## Current Coverage (from Phase 1)
    - Line Coverage: ${file1.linePct}%
    - Branch Coverage: ${file1.branchPct}%
    - Uncovered Lines: ${file1.uncoveredLines}

    ## Instructions
    1. DO NOT re-run full coverage - use the data above
    2. Read the file and identify why those lines aren't covered
    3. If logic should be extracted to a testable helper, do that FIRST
    4. Write targeted tests for uncovered code paths
    5. Run ONLY the tests you create/modify to verify they pass
    6. Report: tests added, any exclusions applied, final status
  `,
  description: "Coverage: ${file1.name}",
  run_in_background: true
})
```

### For Angular HTML Template Files:

```
Task({
  subagent_type: "coverage-enforcer",
  prompt: `
    Ensure high test coverage for Angular template: ${file1.path}

    ## Current Coverage (from Phase 1)
    - Line Coverage: ${file1.linePct}%
    - Branch Coverage: ${file1.branchPct}%
    - Uncovered Template Sections: ${file1.uncoveredSections}

    ## This is an Angular HTML template file
    Tests must be written in the corresponding .spec.ts file.

    ## Template Testing Instructions
    1. Read the HTML template to understand the structure
    2. Read the corresponding .spec.ts file
    3. For each uncovered section, write template rendering tests:
       - @if blocks: Test both true AND false conditions
       - @for blocks: Test with items AND empty array
       - (click) handlers: Trigger click events and verify behavior
       - {{ expressions }}: Verify correct text content is displayed
       - [disabled]/[class] bindings: Test conditional attributes
    4. Use fixture.nativeElement.querySelector() to find elements
    5. Use data-testid attributes for reliable element selection
    6. Run the spec file tests to verify they pass
    7. Report: tests added, elements verified, final status

    ## Target: 80%+ coverage (100% is often impractical for templates)
  `,
  description: "Template: ${file1.name}",
  run_in_background: true
})

// File 2 (in same message for parallel execution)
Task({
  subagent_type: "coverage-enforcer",
  prompt: `...same format for file2...`,
  description: "Coverage: ${file2.name}",
  run_in_background: true
})

// ... more files
```

---

## Phase 3: Collect Results

Use **TaskOutput** to retrieve results from all Phase 2 agents:

```
TaskOutput({ task_id: "agent1_id", block: true })
TaskOutput({ task_id: "agent2_id", block: true })
// ... for each agent
```

Build an intermediate results table:

| File | Initial | Agent Claimed | Tests Added |
|------|---------|---------------|-------------|
| file1.cs | 45% | 100% | 5 |
| file2.cs | 78% | 100% | 3 |

---

## Phase 4: Verification & Loop

Spawn a **coverage-verifier** agent to verify actual coverage and decide on looping:

```
Task({
  subagent_type: "coverage-verifier",
  prompt: `
    Verify coverage for these files after test writing:

    ## Files Processed
    ${files.map(f => `- ${f.path}`).join('\n')}

    ## Initial Coverage (Phase 1)
    ${files.map(f => `- ${f.name}: ${f.linePct}% line, ${f.branchPct}% branch`).join('\n')}

    ## Agent Reports (Phase 3)
    ${results.map(r => `- ${r.file}: ${r.testsAdded} tests, claims ${r.claimedPct}%`).join('\n')}

    ## Iteration
    Current iteration: ${iteration} of 3
  `,
  description: "Verify final coverage",
  run_in_background: false  // Wait for decision
})
```

### Verifier Decisions

The coverage-verifier will return one of:

| Verdict | Action |
|---------|--------|
| **COMPLETE** | All files at 100% or validly excluded → Done |
| **RETRY** | Some files < 100% → Go to Phase 2 with incomplete files |
| **BLOCKED** | Max iterations reached → Report blockers, stop |

### Loop Logic

```
iteration = 1
while iteration <= 3:
    results = run_phase_2_and_3(incomplete_files)
    verdict = run_phase_4_verifier(results, iteration)

    if verdict == COMPLETE:
        break
    elif verdict == RETRY:
        incomplete_files = verdict.files_needing_retry
        iteration += 1
    elif verdict == BLOCKED:
        report_blockers(verdict.blocking_issues)
        break
```

---

## Final Output Format

```markdown
## Coverage Enforcement Results

### Summary
- Files processed: X
- Files at 100%: Y (TypeScript/C#)
- Templates at 80%+: Z (HTML)
- Files excluded: A
- Files blocked: B
- Total iterations: N

### Detailed Results

| File | Type | Initial | Final | Target | Status | Notes |
|------|------|---------|-------|--------|--------|-------|
| ServiceA.cs | C# | 67% | 100% | 100% | ✅ | +8 tests |
| ServiceB.ts | TS | 89% | 100% | 100% | ✅ | +3 tests |
| users.html | HTML | 25% | 82% | 80% | ✅ | +6 template tests |
| panel-editor.html | HTML | 35% | 78% | 80% | ⚠️ | Near target |
| Program.cs | C# | 12% | N/A | 100% | ⚪ | Excluded: config only |

### Tests Added

#### Logic Tests (TypeScript/C#)
- `ServiceATests.cs`: 8 new test methods
- `ServiceB.spec.ts`: 3 new test methods

#### Template Rendering Tests (HTML)
- `users.spec.ts`: 6 new template tests
  - Loading state verification
  - User list rendering
  - Empty state display
  - Delete button click handler
  - Role badge display
  - Search input binding
- `panel-editor.spec.ts`: 5 new template tests
  - Panel selection state
  - Mouse event handlers
  - Panel list rendering
  - Resize handle visibility
  - Toolbar button states

### Exclusions Applied
- `Program.cs`: Pure startup configuration, no business logic

### Blocked Items (if any)
- `ServiceC.cs` line 45: Static DateTime.Now - requires IDateTimeProvider injection
```

---

## Error Handling

- **File doesn't exist**: Report and skip
- **No files specified**: Show usage: `/coverage <file1> [file2] ...`
- **Coverage tool fails**: Report error, suggest fixes
- **Agent times out**: Mark as incomplete, suggest manual review

---

## Performance Characteristics

| Metric | Count |
|--------|-------|
| Coverage runs (Phase 1) | 1 |
| Coverage runs (Phase 4, per iteration) | 1 |
| Total coverage runs (best case) | 2 |
| Total coverage runs (worst case, 3 iterations) | 4 |
| Agent parallelism | All Phase 2 agents run in parallel |

---

## Example Execution

### Example 1: TypeScript/C# Files

```
User: /coverage src/Services/AuthService.cs src/Controllers/AuthController.cs

Phase 1: Spawn coverage-scanner
  → AuthService.cs: 65% line, 50% branch, uncovered: 45-48, 89
  → AuthController.cs: 80% line, 70% branch, uncovered: 23, 67-70

Phase 2: Spawn 2 coverage-enforcer agents in parallel
  → Agent 1 working on AuthService.cs
  → Agent 2 working on AuthController.cs

Phase 3: Collect results
  → Agent 1: Added 4 tests, claims 100%
  → Agent 2: Added 3 tests, claims 100%

Phase 4: Spawn coverage-verifier
  → Runs coverage, verifies both at 100%
  → VERDICT: COMPLETE

Output: Final summary table
```

### Example 2: Angular HTML Templates

```
User: /coverage app/components/users/users.html app/components/panel-editor/panel-editor.html

Phase 0: Normalize paths
  → users.html → src/ComicRag.Web/ClientApp/src/app/components/admin/users/users.html
  → panel-editor.html → src/ComicRag.Web/ClientApp/src/app/components/panel-editor/panel-editor.html

Phase 1: Spawn coverage-scanner
  → users.html: 25% line, 30% branch
    Uncovered: @if (isLoading), @for (user of users), (click)="deleteUser()"
  → panel-editor.html: 40% line, 35% branch
    Uncovered: @if (selectedPanel), (mousedown)="onMouseDown()", @for (panel of panels)

Phase 2: Spawn 2 coverage-enforcer agents in parallel (with template instructions)
  → Agent 1: Writing template tests for users.html in users.spec.ts
  → Agent 2: Writing template tests for panel-editor.html in panel-editor.spec.ts

Phase 3: Collect results
  → Agent 1: Added 8 template tests (loading state, user list, empty state, click handlers)
  → Agent 2: Added 6 template tests (panel selection, mouse events, panel list)

Phase 4: Spawn coverage-verifier
  → Runs coverage
  → users.html: 25% → 78%
  → panel-editor.html: 40% → 85%
  → VERDICT: COMPLETE (target 80% for templates)

Output: Final summary with template-specific results
```
