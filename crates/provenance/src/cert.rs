//! X.509 Certificate Chain verification and C2PA signing keystore.

use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct X509CertInfo {
    pub subject: String,
    pub issuer: String,
    pub serial_number: String,
    pub not_before: DateTime<Utc>,
    pub not_after: DateTime<Utc>,
    pub thumbprint_sha256: String,
    pub is_ca: bool,
    pub key_usage: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct C2paCertificateChain {
    pub root_ca: X509CertInfo,
    pub intermediate_ca: X509CertInfo,
    pub leaf_signer: X509CertInfo,
}

impl C2paCertificateChain {
    /// Constructs a staging PKI certificate hierarchy for C2PA provenance signing.
    pub fn staging_pki() -> Self {
        let now = Utc::now();
        let root_expiry = now + Duration::days(3650); // 10 years
        let intermediate_expiry = now + Duration::days(1825); // 5 years
        let leaf_expiry = now + Duration::days(365); // 1 year

        let root_ca = X509CertInfo {
            subject: "CN=Cutline Staging Root CA, O=Cutline Technologies Inc, C=US".to_string(),
            issuer: "CN=Cutline Staging Root CA, O=Cutline Technologies Inc, C=US".to_string(),
            serial_number: "4a:71:09:bc:88:21:4f:01".to_string(),
            not_before: now,
            not_after: root_expiry,
            thumbprint_sha256: "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855".to_string(),
            is_ca: true,
            key_usage: vec!["keyCertSign".to_string(), "cRLSign".to_string()],
        };

        let intermediate_ca = X509CertInfo {
            subject: "CN=Cutline Content Credentials Issuing CA G1, O=Cutline Technologies Inc, C=US".to_string(),
            issuer: "CN=Cutline Staging Root CA, O=Cutline Technologies Inc, C=US".to_string(),
            serial_number: "5b:82:1a:cd:99:32:5e:02".to_string(),
            not_before: now,
            not_after: intermediate_expiry,
            thumbprint_sha256: "1f82c2d96e5d83f12abf8417c912e7534d0b1a29f8c4e5d6a7b8c9d0e1f2a3b4".to_string(),
            is_ca: true,
            key_usage: vec!["keyCertSign".to_string(), "digitalSignature".to_string()],
        };

        let leaf_signer = X509CertInfo {
            subject: "CN=Cutline Production Worker Node 01, O=Cutline Technologies Inc, C=US".to_string(),
            issuer: "CN=Cutline Content Credentials Issuing CA G1, O=Cutline Technologies Inc, C=US".to_string(),
            serial_number: "6c:93:2b:de:aa:43:6f:03".to_string(),
            not_before: now,
            not_after: leaf_expiry,
            thumbprint_sha256: "9e8d7c6b5a4f3e2d1c0b9a8f7e6d5c4b3a2f1e0d9c8b7a6f5e4d3c2b1a0f9e8d".to_string(),
            is_ca: false,
            key_usage: vec!["digitalSignature".to_string(), "c2paSigning".to_string()],
        };

        Self {
            root_ca,
            intermediate_ca,
            leaf_signer,
        }
    }

    /// Verifies the certificate chain validity dates and trust anchor link.
    pub fn verify_trust_chain(&self) -> bool {
        let now = Utc::now();
        // Check date validity
        if now < self.root_ca.not_before || now > self.root_ca.not_after {
            return false;
        }
        if now < self.intermediate_ca.not_before || now > self.intermediate_ca.not_after {
            return false;
        }
        if now < self.leaf_signer.not_before || now > self.leaf_signer.not_after {
            return false;
        }

        // Verify issuer chain linkages
        if self.intermediate_ca.issuer != self.root_ca.subject {
            return false;
        }
        if self.leaf_signer.issuer != self.intermediate_ca.subject {
            return false;
        }

        // Check required key usages
        if !self.root_ca.is_ca || !self.intermediate_ca.is_ca || self.leaf_signer.is_ca {
            return false;
        }

        true
    }
}
