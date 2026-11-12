# Issue Tracker: Local Markdown

Issues and specs in this repository are stored as Markdown files inside `.scratch/`.
All content (titles, specs, tickets, comments) is written in **English (en-US)**.

## Conventions

- One directory per feature: `.scratch/<feature-slug>/`
- The spec lives at `.scratch/<feature-slug>/spec.md`
- Implementation issues are one file per ticket at `.scratch/<feature-slug>/issues/<NN>-<slug>.md`, numbered starting at `01` — never put all tickets in a single file
- Triage state lives on a `Status:` line near the top of each issue (see `triage-labels.md` for values)
- Comments and conversation history are appended at the bottom of the file under `## Comments`

## When a skill says "post to the issue tracker"

Create a new file in `.scratch/<feature-slug>/` (creating the directory if needed).

## When a skill says "fetch the relevant ticket"

Read the file at the indicated path. Usually the user provides the path or issue number.

## Wayfinding operations

Used by `/wayfinder`. The **map** is a file with one **child** file per ticket.

- **Map**: `.scratch/<effort>/map.md` (body with Notes / Decisions so far / Fog).
- **Child ticket**: `.scratch/<effort>/issues/NN-<slug>.md`, numbered from `01`, with the question in the body. A `Type:` line records the ticket type (`research`/`prototype`/`interview`/`task`); a `Status:` line records `claimed`/`resolved`.
- **Blocking**: a `Blocked by: NN, NN` line near the top. A ticket is unblocked when all listed files are `resolved`.
- **Frontier**: look in `.scratch/<effort>/issues/` for open, unblocked, unclaimed files; lowest number wins.
- **Claim**: set `Status: claimed` and save before doing any work.
- **Resolve**: append the answer under `## Answer`, set `Status: resolved`, and append a context pointer (summary + link) under "Decisions so far" in `map.md`.
