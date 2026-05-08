# Project-Wide Instructions for Agents

## Overview

We are building _markdown-formatter-ja_, a tool to format markdown files, with a focus on Japanese text.

## Development Approach: Test-Driven Development (TDD)

Adopt Test-Driven Development as advocated by Kent Beck and t_wada.

- **Write Small Tests First**
- **Pass Tests in Simple Ways**
- **Refactor for Clarity and Performance**

## Rules

- Redundant fallbacks and error handling are prohibited.
- Be mindful of **YAGNI**.

## Documentations

- [SPEC.md](./docs/SPEC.md) Current project purpose, direction, and externally visible requirements. Keep it simple; implementation details and historical notes belong elsewhere.
- [TODO.md](./docs/TODO.md) TDD working notes for small next steps and deferred ideas. Do not treat it as an authoritative roadmap or exact source-code state.
- [ADRs](./docs/adr/) Architectural Decision Records documenting key decisions, rejected alternatives, and rationale.

See [ADR 0006](./docs/adr/0006-document-roles-for-spec-todo-and-adr.md) for the roles of `SPEC.md`, `TODO.md`, and ADRs.
