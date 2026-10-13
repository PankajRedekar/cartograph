# 🍳 Multi-Turn Autonomous Agent Action Recipes

Prescriptive, deterministic workflows for autonomous coding agents:

## Recipe 1: Surgical Refactoring

1. **Step 1 (Explore)**: `cartograph locate "<target_symbol>" --format xml`
2. **Step 2 (Risk Check)**: `cartograph impact --symbol "<target_symbol>" --format xml`
3. **Step 3 (Edit)**: Perform minimal surgical edit using exact coordinates.
4. **Step 4 (Verify Gate)**: `cartograph verify --staged`
5. **Step 5 (Commit)**: Proceed to git commit if `<verification_gate status="PASSED">`.

## Recipe 2: Test-Driven Bugfix

1. **Step 1 (Diagnose)**: `cartograph diagnose --file crash.log --scaffold-test`
2. **Step 2 (Reproduce)**: Execute generated test reproducer and observe failure.
3. **Step 3 (Patch)**: Implement minimal fix.
4. **Step 4 (Predictive Test)**: `cartograph test-selector --staged --run --bail`
5. **Step 5 (Verify)**: `cartograph verify --staged`
