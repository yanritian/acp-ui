---
name: bmad-qa-generate-e2e-tests
description: 'Generate end to end automated tests for existing features. Use when the user says "create qa automated tests for [feature]"'
---

## Goal

Generate comprehensive E2E automated tests for specified features.

## Instructions

1. **Identify Target Feature**
   - Ask user which feature needs E2E tests
   - Review existing implementation code
   - Identify critical user flows

2. **Analyze Test Requirements**
   - Map user journeys to test scenarios
   - Identify edge cases and error conditions
   - Determine test framework (Playwright/Cypress/etc)

3. **Generate Test Files**
   - Create test file structure
   - Write test scenarios covering:
     - Happy path flows
     - Error handling
     - Edge cases
     - Data validation

4. **Review and Refine**
   - Verify test coverage
   - Check for missing scenarios
   - Ensure tests are maintainable

## Output

- E2E test files in appropriate test directory
- Test coverage report
- Documentation of test scenarios