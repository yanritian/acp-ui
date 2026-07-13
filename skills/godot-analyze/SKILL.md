# godot-analyze

Analyze a Godot project without modifying files.

## Parameters

- `project_path`: absolute path on the D: drive.

## Execution contract

The backend parses `project.godot`, `.tscn` scenes, and supported scripts and
returns structured JSON. The skill is read-only and must report parse errors.
