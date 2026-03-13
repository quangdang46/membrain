# SECURITY GOVERNANCE

This document covers access control, retention, auditability, and compliance-aware memory behavior.

## 1. Namespace isolation

### Policy intent
Namespace isolation must be enforced consistently across write, read, and background-job paths.

### Required checks
- actor identity
- namespace scope
- retention class
- policy pinning
- audit requirement

### Failure mode
If namespace isolation is violated, the system must emit an auditable incident-grade event.

## 2. Workspace ACL

### Policy intent
Workspace ACL must be enforced consistently across write, read, and background-job paths.

### Required checks
- actor identity
- namespace scope
- retention class
- policy pinning
- audit requirement

### Failure mode
If workspace acl is violated, the system must emit an auditable incident-grade event.

## 3. Agent ACL

### Policy intent
Agent ACL must be enforced consistently across write, read, and background-job paths.

### Required checks
- actor identity
- namespace scope
- retention class
- policy pinning
- audit requirement

### Failure mode
If agent acl is violated, the system must emit an auditable incident-grade event.

## 4. Session visibility

### Policy intent
Session visibility must be enforced consistently across write, read, and background-job paths.

### Required checks
- actor identity
- namespace scope
- retention class
- policy pinning
- audit requirement

### Failure mode
If session visibility is violated, the system must emit an auditable incident-grade event.

## 5. Redaction

### Policy intent
Redaction must be enforced consistently across write, read, and background-job paths.

### Required checks
- actor identity
- namespace scope
- retention class
- policy pinning
- audit requirement

### Failure mode
If redaction is violated, the system must emit an auditable incident-grade event.

## 6. Retention compliance

### Policy intent
Retention compliance must be enforced consistently across write, read, and background-job paths.

### Required checks
- actor identity
- namespace scope
- retention class
- policy pinning
- audit requirement

### Failure mode
If retention compliance is violated, the system must emit an auditable incident-grade event.

## 7. Legal hold

### Policy intent
Legal hold must be enforced consistently across write, read, and background-job paths.

### Required checks
- actor identity
- namespace scope
- retention class
- policy pinning
- audit requirement

### Failure mode
If legal hold is violated, the system must emit an auditable incident-grade event.

## 8. Deletion guarantees

### Policy intent
Deletion guarantees must be enforced consistently across write, read, and background-job paths.

### Required checks
- actor identity
- namespace scope
- retention class
- policy pinning
- audit requirement

### Failure mode
If deletion guarantees is violated, the system must emit an auditable incident-grade event.

## 9. Audit logs

### Policy intent
Audit logs must be enforced consistently across write, read, and background-job paths.

### Required checks
- actor identity
- namespace scope
- retention class
- policy pinning
- audit requirement

### Failure mode
If audit logs is violated, the system must emit an auditable incident-grade event.

## 10. Secrets handling

### Policy intent
Secrets handling must be enforced consistently across write, read, and background-job paths.

### Required checks
- actor identity
- namespace scope
- retention class
- policy pinning
- audit requirement

### Failure mode
If secrets handling is violated, the system must emit an auditable incident-grade event.

## 11. Cross-tenant protection

### Policy intent
Cross-tenant protection must be enforced consistently across write, read, and background-job paths.

### Required checks
- actor identity
- namespace scope
- retention class
- policy pinning
- audit requirement

### Failure mode
If cross-tenant protection is violated, the system must emit an auditable incident-grade event.

## 12. Policy precedence

### Policy intent
Policy precedence must be enforced consistently across write, read, and background-job paths.

### Required checks
- actor identity
- namespace scope
- retention class
- policy pinning
- audit requirement

### Failure mode
If policy precedence is violated, the system must emit an auditable incident-grade event.

