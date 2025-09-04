## Generation
Generate a file named AGENTS.md that serves as a contributor guide for this project.
Your goal is to produce a clear, concise, and well-structured document with descriptive headings and actionable explanations for each section.
Follow the outline below, but adapt as needed — add sections if relevant, and omit those that do not apply to this project.

Document Requirements

- Title the document "Project Guidelines".
- Use Markdown headings (#, ##, etc.) for structure.
- Keep the document VERY concise. 
- Keep explanations short, direct, and specific to this project.
- Provide examples where helpful (commands, directory paths, naming patterns).
- Maintain a professional, instructional tone.

Recommended Sections

Project Structure & Module Organization

- Outline the project structure, including where the source code, tests, and assets are located.

Build, Test, and Development Commands

- List key commands for building, testing, and running locally (e.g., npm test, make build).
- Briefly explain what each command does.

Coding Style & Naming Conventions

- Specify indentation rules, language-specific style preferences, and naming patterns.
- Include any formatting or linting tools used.

Testing Guidelines

- Identify testing frameworks and coverage requirements.
- State test naming conventions and how to run tests.

Commit & Pull Request Guidelines

- Summarize commit message conventions found in the project’s Git history.
- Outline pull request requirements (descriptions, linked issues, screenshots, etc.).

(Optional) Add other sections if relevant, such as Security & Configuration Tips, Architecture Overview, or Agent-Specific Instructions.

### FLOW
If ./flow directory existed in the current repository, then you should read them to understand more about the project. 
./flow might contain multiple directory and level and MUST have QUICK_REFERENCE.md in each directory and level. If you find some are missing then add them during your exploration. 

AGENTS.md now should mostly reference to the document in the QUICK_REFERENCE.md. MAKE SURE all reference are not skipping level, only the nearest level. 

## Updating
IF the file AGENTS.md existed, reread it and update it to match the expectation of the `Generation`. 