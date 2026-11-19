//! Decisions over supplied password properties and verified factor observations.
//! This model never accepts a real password, certificate, key, or biometric.
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Policy {
    pub minimum_length: u8,
    pub reject_listed_passwords: bool,
    pub replace_after_compromise: bool,
    pub second_factor: SecondFactor,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SecondFactor {
    None,
    MemorizedCode,
    BoundAuthenticator,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PasswordProperties {
    pub characters: u8,
    pub listed: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Evidence {
    Missing,
    MemorizedCode,
    CertificateOnly,
    CertificateWithVerifiedKey,
    BiometricOnly,
    BiometricUnlockedVerifiedKey,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SignIn {
    pub current_password_verified: bool,
    pub compromise_reported: bool,
    pub second_proof: Evidence,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PasswordCheck {
    Accepted,
    TooShort,
    Listed,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SignInCheck {
    Admitted,
    PasswordFailed,
    ReplacementRequired,
    SecondProofMissing,
}

impl Policy {
    pub fn validate(self) -> Result<(), String> {
        if !(8..=64).contains(&self.minimum_length) {
            return Err("Set the training policy threshold between 8 and 64 characters.".into());
        }
        Ok(())
    }

    pub const fn proposed_password(self, properties: PasswordProperties) -> PasswordCheck {
        if properties.characters < self.minimum_length {
            PasswordCheck::TooShort
        } else if self.reject_listed_passwords && properties.listed {
            PasswordCheck::Listed
        } else {
            PasswordCheck::Accepted
        }
    }

    pub const fn sign_in(self, submitted: SignIn) -> SignInCheck {
        if !submitted.current_password_verified {
            return SignInCheck::PasswordFailed;
        }
        if self.replace_after_compromise && submitted.compromise_reported {
            return SignInCheck::ReplacementRequired;
        }
        let additional_proof_present = match self.second_factor {
            SecondFactor::None => true,
            SecondFactor::MemorizedCode => {
                matches!(submitted.second_proof, Evidence::MemorizedCode)
            }
            SecondFactor::BoundAuthenticator => matches!(
                submitted.second_proof,
                Evidence::CertificateWithVerifiedKey | Evidence::BiometricUnlockedVerifiedKey
            ),
        };
        if additional_proof_present {
            SignInCheck::Admitted
        } else {
            SignInCheck::SecondProofMissing
        }
    }
}

impl PasswordCheck {
    pub const fn describe(self) -> &'static str {
        match self {
            Self::Accepted => "The new password meets the current settings.",
            Self::TooShort => "The supplied length is below the current minimum.",
            Self::Listed => {
                "The proposed password is on the supplied list of weak or compromised passwords."
            }
        }
    }
}

impl SignInCheck {
    pub const fn describe(self, proof: Evidence) -> &'static str {
        match self {
            Self::Admitted => "Allowed by the current sign-in policy.",
            Self::PasswordFailed => "The supplied password check failed.",
            Self::ReplacementRequired => {
                "The compromise report requires an approved replacement first."
            }
            Self::SecondProofMissing => match proof {
                Evidence::MemorizedCode => {
                    "A password and a memorized code both use something the person knows."
                }
                Evidence::CertificateOnly => {
                    "The certificate is public information. This trial has no proof of the matching private key."
                }
                Evidence::BiometricOnly => {
                    "This trial has no bound authenticator unlocked by the biometric check."
                }
                _ => "The presented proof does not meet the current sign-in policy.",
            },
        }
    }
}
