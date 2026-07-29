package compliance.logging

# Swarm OS Phase 4 - Logging Compliance Policy
# Denies logs containing PII or sensitive patterns to ensure GDPR/SOC2 compliance.

default allow = true

# Deny if log contains PII patterns (emails, SSNs, etc.)
deny[msg] {
    input.method == "write_log"
    contains(lower(input.data), "email")
    msg = sprintf("Compliance Violation: Log contains email pattern: %v", [input.data])
}

deny[msg] {
    input.method == "write_log"
    contains(lower(input.data), "ssn")
    msg = sprintf("Compliance Violation: Log contains SSN pattern: %v", [input.data])
}

deny[msg] {
    input.method == "write_log"
    contains(lower(input.data), "password")
    msg = sprintf("Compliance Violation: Log contains password keyword: %v", [input.data])
}

# Enforce encryption for sensitive destinations
deny[msg] {
    input.destination == "external_untrusted"
    input.encrypted == false
    msg = "Compliance Violation: Unencrypted data transfer to untrusted destination"
}
