//! Cutline Provenance Crate
//! C2PA Content Credentials manifest signing and AI disclosure stamping.

pub mod cert;
pub mod manifest;

pub use cert::{C2paCertificateChain, X509CertInfo};
pub use manifest::{
    C2paAction, C2paAssertion, C2paManifest, C2paSignature, ProvenanceError,
};
