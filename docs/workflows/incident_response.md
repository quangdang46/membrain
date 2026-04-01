---
id: incident_response
title: Incident Response
source_doc: docs/workflows/incident_response.md
section: ## Incident Response
summary: Contain degraded or unsafe serving, re-establish authoritative truth, and only then resume operator actions.
---
# Incident Response

## Steps
- Record the active degraded reasons or operator error kind and treat them as containment inputs, not just logging noise.
- Pause irreversible or action-critical follow-up until authoritative state, freshness, and policy scope are re-validated.
- After containment, rerun doctor or health and confirm the recovery evidence before resuming normal workflows.
