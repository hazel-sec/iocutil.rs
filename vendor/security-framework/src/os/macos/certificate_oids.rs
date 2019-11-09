//! OIDs associated with certificate properties.
use core_foundation::base::TCFType;
use core_foundation::string::CFString;
use core_foundation_sys::string::CFStringRef;
use security_framework_sys::certificate_oids::*;

/// An identifier of a property of a certificate.
pub struct CertificateOid(CFStringRef);

#[allow(missing_docs)]
impl CertificateOid {
    pub fn x509_v1_signature_algorithm() -> CertificateOid {
        unsafe { CertificateOid(kSecOIDX509V1SignatureAlgorithm) }
    }

    /// Returns the underlying raw pointer corresponding to this OID.
    pub fn as_ptr(&self) -> CFStringRef {
        self.0
    }

    /// Returns the string representation of the OID.
    pub fn to_str(&self) -> CFString {
        unsafe { CFString::wrap_under_get_rule(self.0) }
    }
}
