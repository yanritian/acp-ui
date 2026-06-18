# Vertical Domain Examples

This directory contains domain-specific examples and guides for applying ACP-Swarm in various scenarios.

## Available Verticals

### 1. Code Generation (`code-generation/`)
- Automated code writing and refactoring
- TypeScript/Python/Rust code generation
- Unit test generation

### 2. Documentation (`documentation/`)
- README generation
- API documentation
- Code comments and explanations

### 3. Testing (`testing/`)
- Test execution and validation
- Coverage analysis
- Regression testing

### 4. DevOps (`devops/`)
- CI/CD pipeline configuration
- Deployment automation
- Infrastructure provisioning

## Goal Templates

Each vertical includes `.goal.yaml` templates for common workflows:

```
verticals/
├── code-generation/
│   └── fix-typescript.goal.yaml
│   └── refactor-module.goal.yaml
├── documentation/
│   └── update-readme.goal.yaml
│   └── generate-api-docs.goal.yaml
├── testing/
│   └── run-unit-tests.goal.yaml
│   └── integration-test.goal.yaml
└── devops/
    └── deploy-to-staging.goal.yaml
    └── configure-ci.goal.yaml
```

## Usage

1. Copy the appropriate `.goal.yaml` template to your workspace
2. Modify the `description` and `completion_condition` fields
3. Submit to GoalGraph via UI or API

```bash
# Example: Run code generation goal
cargo run --bin acp-cli goal submit verticals/code-generation/fix-typescript.goal.yaml
```