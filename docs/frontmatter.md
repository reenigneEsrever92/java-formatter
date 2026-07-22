---
type: Reference
title: Front matter schema
description: The YAML front matter every concept document in this bundle must carry.
tags: [docs]
status: active
---

# Front matter schema

Documents in this bundle are one of two kinds:

- **Reserved files** — `index.md` and `log.md`. They carry **no front matter**
  and are part of the bundle structure itself.
- **Concept documents** — everything else. Each must start with YAML front
  matter. The only required field is `type`; the common optional fields are
  `title`, `description`, `tags`, and `status`.

```yaml
---
type: Overview
title: java-formatter
description: A CLI that formats Java source according to IntelliJ IDEA code style schemes.
tags: [java, formatter, cli]
status: active
---
```

## Types

| `type` | Used for |
| --- | --- |
| `Overview` | What the project is (see [overview.md](overview.md)). |
| `Requirements` | Users, use cases, technology choices, and requirements (see [requirements.md](requirements.md)). |
| `Reference` | Reference material such as this schema. |
| `Changelog` | Shipped changes, newest first (see [dev/changelog.md](dev/changelog.md)). |
| `ChangeRequest` | Backlog items (see [dev/backlog/index.md](dev/backlog/index.md)). |

## ChangeRequest documents

A change request uses a single `state` field (never `status`) and carries
`kind`, `priority`, `tags`, and `owner`:

```yaml
---
type: ChangeRequest
kind: feature
title: <Title>
description: <one-line summary>
state: proposed
priority: medium
tags: [dev, formatter]
owner: maintainer
---
```

- `kind` — `feature`, `bug`, `improvement`, or `refactor`.
- `state` — the workflow lifecycle: `proposed` → `planned` → `in-progress` →
  `done`. Re-checking a request (`fawi-check`) may move it to `rejected` or
  `superseded`.
- `priority` — `low`, `medium`, or `high`.
- `owner` — the actor accountable for the request.

Each change request body carries the sections the workflow expects:
`# Problem`, `# Proposal`, `# Decisions`, and `# Acceptance criteria`.
