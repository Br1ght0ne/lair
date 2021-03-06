//! Types associated with Lair client actor.

use crate::*;
use derive_more::*;

ghost_actor::ghost_chan! {
    /// "Event" types emitted by Lair Client Actor Api.
    pub chan LairClientEvent<LairError> {
        /// The keystore is currently locked - the user
        /// must supply a passphrase in order to unlock.
        fn request_unlock_passphrase() -> String;
    }
}

/// Lair Client Event Sender Type.
pub type LairClientEventSenderType =
    futures::channel::mpsc::Sender<LairClientEvent>;

/// Lair Client Event Receiver Type.
pub type LairClientEventReceiver =
    futures::channel::mpsc::Receiver<LairClientEvent>;

/// Tls keypair algorithm to use.
#[non_exhaustive]
#[repr(u32)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum TlsCertAlg {
    /// Ed25519 Curve.
    PkcsEd25519 = 0x00000200,
    /// Ecdsa Curve 256.
    PkcsEcdsaP256Sha256 = 0x00000201,
    /// Ecdsa Curve 384.
    PkcsEcdsaP384Sha384 = 0x00000202,
}

impl Default for TlsCertAlg {
    fn default() -> Self {
        Self::PkcsEd25519
    }
}

impl TlsCertAlg {
    /// parse a u32 into a LairEntryType enum variant.
    pub fn parse(d: u32) -> LairResult<Self> {
        use TlsCertAlg::*;
        Ok(match d {
            x if x == PkcsEd25519 as u32 => PkcsEd25519,
            x if x == PkcsEcdsaP256Sha256 as u32 => PkcsEcdsaP256Sha256,
            x if x == PkcsEcdsaP384Sha384 as u32 => PkcsEcdsaP384Sha384,
            _ => return Err("invalide tls cert alg".into()),
        })
    }
}

/// Configuration for Tls Certificate Generation.
#[non_exhaustive]
pub struct TlsCertOptions {
    /// Tls keypair algorithm to use.
    pub alg: TlsCertAlg,
}

impl Default for TlsCertOptions {
    fn default() -> Self {
        Self {
            alg: TlsCertAlg::PkcsEd25519,
        }
    }
}

/// Keystore index type.
#[derive(
    Clone,
    Copy,
    Debug,
    Display,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Hash,
    Deref,
    From,
    Into,
)]
pub struct KeystoreIndex(pub u32);

/// Der encoded Tls Certificate bytes.
#[derive(
    Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Deref, From, Into,
)]
pub struct Cert(pub Arc<Vec<u8>>);

impl From<Vec<u8>> for Cert {
    fn from(d: Vec<u8>) -> Self {
        Self(Arc::new(d))
    }
}

/// Der encoded pkcs #8 Tls Certificate private key bytes.
#[derive(
    Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Deref, From, Into,
)]
pub struct CertPrivKey(pub Arc<Vec<u8>>);

impl From<Vec<u8>> for CertPrivKey {
    fn from(d: Vec<u8>) -> Self {
        Self(Arc::new(d))
    }
}

/// Sni encoded in given Tls Certificate.
#[derive(
    Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Deref, From, Into,
)]
pub struct CertSni(pub Arc<String>);

impl From<String> for CertSni {
    fn from(s: String) -> Self {
        Self(Arc::new(s))
    }
}

/// The 32 byte blake2b digest of given Tls Certificate.
#[derive(
    Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Deref, From, Into,
)]
pub struct CertDigest(pub Arc<Vec<u8>>);

impl From<Vec<u8>> for CertDigest {
    fn from(d: Vec<u8>) -> Self {
        Self(Arc::new(d))
    }
}

/// The 32 byte signature ed25519 public key.
#[derive(
    Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Deref, From, Into,
)]
pub struct SignEd25519PubKey(pub Arc<Vec<u8>>);

impl From<Vec<u8>> for SignEd25519PubKey {
    fn from(d: Vec<u8>) -> Self {
        Self(Arc::new(d))
    }
}

impl SignEd25519PubKey {
    /// Verify signature on given message with given public key.
    pub async fn verify(
        &self,
        message: Arc<Vec<u8>>,
        signature: SignEd25519Signature,
    ) -> LairResult<bool> {
        internal::sign_ed25519::sign_ed25519_verify(
            self.clone(),
            message,
            signature,
        )
        .await
    }
}

/// The 64 byte detached ed25519 signature data.
#[derive(
    Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Deref, From, Into,
)]
pub struct SignEd25519Signature(pub Arc<Vec<u8>>);

impl From<Vec<u8>> for SignEd25519Signature {
    fn from(d: Vec<u8>) -> Self {
        Self(Arc::new(d))
    }
}

/// The entry type for a given entry.
#[non_exhaustive]
#[repr(u32)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum LairEntryType {
    /// This entry index was deleted or corrupted.
    Invalid = 0x00000000,

    /// Tls Certificate & private key.
    TlsCert = 0x00000100,

    /// Ed25519 algorithm signature keypair.
    SignEd25519 = 0x00000200,
}

impl Default for LairEntryType {
    fn default() -> Self {
        Self::Invalid
    }
}

impl LairEntryType {
    /// parse a u32 into a LairEntryType enum variant.
    pub fn parse(d: u32) -> LairResult<Self> {
        use LairEntryType::*;
        Ok(match d {
            x if x == Invalid as u32 => Invalid,
            x if x == TlsCert as u32 => TlsCert,
            x if x == SignEd25519 as u32 => SignEd25519,
            _ => return Err("invalide lair entry type".into()),
        })
    }
}

/// Get information about the server we are connected to.
#[non_exhaustive]
#[derive(Debug, Default, Clone, PartialEq)]
pub struct LairServerInfo {
    /// Server name / identifier.
    pub name: String,

    /// Server version.
    pub version: String,
}

ghost_actor::ghost_chan! {
    /// Lair Client Actor Api.
    pub chan LairClientApi<LairError> {
        /// Get lair server info.
        fn lair_get_server_info() -> LairServerInfo;

        /// Get the highest entry index.
        /// Note, some entries my be stubs / erased values.
        fn lair_get_last_entry_index() -> KeystoreIndex;

        /// Get the entry type for a given index.
        fn lair_get_entry_type(
            keystore_index: KeystoreIndex,
        ) -> LairEntryType;

        /// Create a new self-signed tls certificate.
        fn tls_cert_new_self_signed_from_entropy(
            options: TlsCertOptions,
        ) -> (KeystoreIndex, CertSni, CertDigest);

        /// Get tls cert info by keystore index.
        fn tls_cert_get(
            keystore_index: KeystoreIndex,
        ) -> (CertSni, CertDigest);

        /// Fetch the certificate by entry index.
        fn tls_cert_get_cert_by_index(
            keystore_index: KeystoreIndex,
        ) -> Cert;

        /// Fetch the certificate by digest.
        fn tls_cert_get_cert_by_digest(
            cert_digest: CertDigest,
        ) -> Cert;

        /// Fetch the certificate by sni.
        fn tls_cert_get_cert_by_sni(
            cert_sni: CertSni,
        ) -> Cert;

        /// Fetch the certificate private key by entry index.
        fn tls_cert_get_priv_key_by_index(
            keystore_index: KeystoreIndex,
        ) -> CertPrivKey;

        /// Fetch the certificate private key by digest.
        fn tls_cert_get_priv_key_by_digest(
            cert_digest: CertDigest,
        ) -> CertPrivKey;

        /// Fetch the certificate private key by sni.
        fn tls_cert_get_priv_key_by_sni(
            cert_sni: CertSni,
        ) -> CertPrivKey;

        /// Create a new signature ed25519 keypair from entropy.
        fn sign_ed25519_new_from_entropy(
        ) -> (KeystoreIndex, SignEd25519PubKey);

        /// Get ed25519 keypair info by keystore index.
        fn sign_ed25519_get(
            keystore_index: KeystoreIndex,
        ) -> SignEd25519PubKey;

        /// Generate a signature for message by keystore index.
        fn sign_ed25519_sign_by_index(
            keystore_index: KeystoreIndex,
            message: Arc<Vec<u8>>,
        ) -> SignEd25519Signature;

        /// Generate a signature for message by signature pub key.
        fn sign_ed25519_sign_by_pub_key(
            pub_key: SignEd25519PubKey,
            message: Arc<Vec<u8>>,
        ) -> SignEd25519Signature;
    }
}

/// Lair Client Sender Type.
pub type LairClientSender = futures::channel::mpsc::Sender<LairClientApi>;
