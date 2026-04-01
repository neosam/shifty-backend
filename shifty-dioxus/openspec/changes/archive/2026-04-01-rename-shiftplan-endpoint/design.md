## Context

The `get_all_shiftplans()` function in `src/api.rs` currently calls `GET /shiftplan`. The backend has renamed this to `GET /shiftplan-catalog`.

## Goals / Non-Goals

**Goals:**
- Update the API URL to match the backend rename

**Non-Goals:**
- No functional changes, only URL rename

## Decisions

### 1. Direct string replacement

Change the URL format string from `"{}/shiftplan"` to `"{}/shiftplan-catalog"` in `src/api.rs`.

**Rationale:** Trivial one-line change to match backend rename.
