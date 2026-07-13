# godot-codegen

Generate a reviewable Godot code proposal through the Hermes Game CLI.

## Parameters

- `project_path`: absolute path on the D: drive.
- `goal`: the requested game change.

## Execution contract

The skill writes a proposal artifact under `.operator/skill-runs` and never
modifies the project directly. Applying a proposal requires the operator patch
approval flow and a subsequent Godot validation step.
