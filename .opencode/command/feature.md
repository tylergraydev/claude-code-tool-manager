---
description: Full feature workflow - plan, test, implement, document. Use for adding new game features end-to-end.
---

# Feature Development Workflow

You are orchestrating a complete feature implementation following TDD and SOLID principles.

## Workflow Steps

Execute these steps in order, using the appropriate subagent for each:

### 1. Plan the Feature
Use the **feature-planner** agent to:
- Read the gameplay bible for context
- Break down the feature into testable requirements
- Identify classes, interfaces, and dependencies
- Output a feature spec

Wait for the spec before proceeding.

### 2. Write Tests First
Use the **test-writer** agent to:
- Create failing tests based on the feature spec
- Follow GDUnit4 patterns
- Ensure tests compile but fail

Verify tests are in place before implementation.

### 3. Implement
Use the **implementer** agent to:
- Make the tests pass one by one
- Follow SOLID principles
- Use component-based architecture
- Run tests after each significant change

Continue until all tests pass.

### 4. Update Documentation
Use the **doc-updater** agent to:
- Update `docs/architecture.md` if new systems were added
- Create/update system-specific docs in `docs/systems/`
- Update component docs if reusable components were created

### 5. Summary
After all steps complete, provide:
- What was implemented
- Test coverage summary
- Files created/modified
- Any follow-up refactoring suggestions

## Example Usage

```
/feature Player can dash in the direction they're facing with a cooldown
```

This will:
1. Plan dash feature (DashComponent, IDashable interface, cooldown logic)
2. Write tests (dash moves player, cooldown prevents spam, etc.)
3. Implement until tests pass
4. Document the dash system
