use crate::client::builder::ConfigWantsServerVerifier;
use crate::error::Error;
use crate::kx::{SupportedKxGroup, ALL_KX_GROUPS};
use crate::server::builder::ConfigWantsClientVerifier;
use crate::suites::{SupportedCipherSuite, DEFAULT_CIPHERSUITES};
use crate::versions;

/// Building a [`ServerConfig`] or [`ClientConfig`] in a linker-friendly way.
///
/// Linker-friendly: meaning unused cipher suites, protocol
/// versions, key exchange mechanisms, etc. can be discarded
/// by the linker as they'll be unreferenced.
///
/// Example, to make a [`ServerConfig`]:
///
/// ```no_run
/// # use rustls::config_builder;
/// # let certs = vec![];
/// # let private_key = rustls::PrivateKey(vec![]);
/// config_builder()
///     .with_safe_default_cipher_suites()
///     .with_safe_default_kx_groups()
///     .with_safe_default_protocol_versions()
///     .for_server()
///     .unwrap()
///     .with_no_client_auth()
///     .with_single_cert(certs, private_key)
///     .expect("bad certificate/key");
/// ```
///
/// This may be shortened to:
///
/// ```no_run
/// # let certs = vec![];
/// # let private_key = rustls::PrivateKey(vec![]);
/// # use rustls::server_config_builder_with_safe_defaults;
/// server_config_builder_with_safe_defaults()
///     .with_no_client_auth()
///     .with_single_cert(certs, private_key)
///     .expect("bad certificate/key");
/// ```
///
/// To make a [`ClientConfig`]:
///
/// ```no_run
/// # use rustls::config_builder;
/// # let root_certs = rustls::RootCertStore::empty();
/// # let trusted_ct_logs = &[];
/// # let certs = vec![];
/// # let private_key = rustls::PrivateKey(vec![]);
/// config_builder()
///     .with_safe_default_cipher_suites()
///     .with_safe_default_kx_groups()
///     .with_safe_default_protocol_versions()
///     .for_client()
///     .unwrap()
///     .with_root_certificates(root_certs, trusted_ct_logs)
///     .with_single_cert(certs, private_key)
///     .expect("bad certificate/key");
/// ```
///
/// This may be shortened to:
///
/// ```
/// # use rustls::client_config_builder_with_safe_defaults;
/// # let root_certs = rustls::RootCertStore::empty();
/// # let trusted_ct_logs = &[];
/// client_config_builder_with_safe_defaults()
///     .with_root_certificates(root_certs, trusted_ct_logs)
///     .with_no_client_auth();
/// ```
///
///
/// The types used here fit together like this:
///
/// 1. You must make a decision on which cipher suites to use, typically
///    by calling [`ConfigWantsCipherSuites::with_safe_default_cipher_suites()`].
/// 2. Now you must make a decision
///    on key exchange groups: typically by calling [`ConfigWantsKxGroups::with_safe_default_kx_groups()`].
/// 3. Now you must make
///    a decision on which protocol versions to support, typically by calling
///    [`ConfigWantsVersions::with_safe_default_protocol_versions()`].
/// 4. You now need to indicate whether to make a [`ServerConfig`] or [`ClientConfig`],
///    by calling [`ConfigWantsPeerType::for_server()`]
///    or [`ConfigWantsPeerType::for_client()`] respectively.
/// 5. Now see [`ConfigWantsServerVerifier`] or [`ConfigWantsClientVerifier`] for further steps.
///
/// [`ServerConfig`]: crate::ServerConfig
/// [`ClientConfig`]: crate::ClientConfig
pub fn config_builder() -> ConfigWantsCipherSuites {
    ConfigWantsCipherSuites {}
}

fn config_builder_with_safe_defaults() -> ConfigWantsPeerType {
    config_builder()
        .with_safe_default_cipher_suites()
        .with_safe_default_kx_groups()
        .with_safe_default_protocol_versions()
}

/// Start building a [`ClientConfig`] and accept defaults for underlying
/// cryptography.
///
/// These are safe defaults, useful for 99% of applications.
///
/// [`ClientConfig`]: crate::ClientConfig
pub fn client_config_builder_with_safe_defaults() -> ConfigWantsServerVerifier {
    // this function exists to express that for_client is infallible when
    // using defaults.
    config_builder_with_safe_defaults()
        .for_client()
        .unwrap()
}

/// Start building a [`ServerConfig`] and accept defaults for underlying
/// cryptography.
///
/// These are safe defaults, useful for 99% of applications.
///
/// [`ServerConfig`]: crate::ServerConfig
pub fn server_config_builder_with_safe_defaults() -> ConfigWantsClientVerifier {
    // this function exists to express that for_server is infallible when
    // using defaults.
    config_builder_with_safe_defaults()
        .for_server()
        .unwrap()
}

/// A config builder where we want to know the cipher suites.
pub struct ConfigWantsCipherSuites;

impl ConfigWantsCipherSuites {
    /// Choose a specific set of cipher suites.
    pub fn with_cipher_suites(
        &self,
        cipher_suites: &[SupportedCipherSuite],
    ) -> ConfigWantsKxGroups {
        ConfigWantsKxGroups {
            cipher_suites: cipher_suites.to_vec(),
        }
    }

    /// Choose the default set of cipher suites.
    ///
    /// Note that this default provides only high-quality suites: there is no need
    /// to filter out low-, export- or NULL-strength cipher suites: rustls does not
    /// implement these.
    pub fn with_safe_default_cipher_suites(&self) -> ConfigWantsKxGroups {
        self.with_cipher_suites(DEFAULT_CIPHERSUITES)
    }
}

/// A config builder where we want to know which key exchange groups to use.
pub struct ConfigWantsKxGroups {
    cipher_suites: Vec<SupportedCipherSuite>,
}

impl ConfigWantsKxGroups {
    /// Choose a specific set of key exchange groups.
    pub fn with_kx_groups(self, kx_groups: &[&'static SupportedKxGroup]) -> ConfigWantsVersions {
        ConfigWantsVersions {
            cipher_suites: self.cipher_suites,
            kx_groups: kx_groups.to_vec(),
        }
    }

    /// Choose the default set of key exchange groups.
    ///
    /// This is a safe default: rustls doesn't implement any poor-quality groups.
    pub fn with_safe_default_kx_groups(self) -> ConfigWantsVersions {
        self.with_kx_groups(&ALL_KX_GROUPS)
    }
}

/// A config builder where we want to know the TLS versions.
pub struct ConfigWantsVersions {
    cipher_suites: Vec<SupportedCipherSuite>,
    kx_groups: Vec<&'static SupportedKxGroup>,
}

impl ConfigWantsVersions {
    /// Accept the default protocol versions: both TLS1.2 and TLS1.3 are enabled.
    pub fn with_safe_default_protocol_versions(self) -> ConfigWantsPeerType {
        self.with_protocol_versions(versions::DEFAULT_VERSIONS)
    }

    /// Use a specific set of protocol versions.
    pub fn with_protocol_versions(
        self,
        versions: &[&'static versions::SupportedProtocolVersion],
    ) -> ConfigWantsPeerType {
        ConfigWantsPeerType {
            cipher_suites: self.cipher_suites,
            kx_groups: self.kx_groups,
            versions: versions::EnabledVersions::new(versions),
        }
    }
}

/// A config builder where we want to know whether this will be a client or a server.
pub struct ConfigWantsPeerType {
    cipher_suites: Vec<SupportedCipherSuite>,
    kx_groups: Vec<&'static SupportedKxGroup>,
    versions: versions::EnabledVersions,
}

impl ConfigWantsPeerType {
    fn validate(&self) -> Result<(), Error> {
        let mut any_usable_suite = false;
        for suite in &self.cipher_suites {
            if self
                .versions
                .contains(suite.version().version)
            {
                any_usable_suite = true;
                break;
            }
        }

        if !any_usable_suite {
            return Err(Error::General("no usable cipher suites configured".into()));
        }

        if self.kx_groups.is_empty() {
            return Err(Error::General("no kx groups configured".into()));
        }

        Ok(())
    }

    /// This config is for a client. Continue by setting client-related options.
    ///
    /// This may fail, if the previous selections are contradictory or
    /// not useful (for example, if no protocol versions are enabled).
    pub fn for_client(self) -> Result<ConfigWantsServerVerifier, Error> {
        self.validate()?;
        Ok(ConfigWantsServerVerifier {
            cipher_suites: self.cipher_suites,
            kx_groups: self.kx_groups,
            versions: self.versions,
        })
    }

    /// This config is for a server. Continue by setting server-related options.
    ///
    /// This may fail, if the previous selections are contradictory or
    /// not useful (for example, if no protocol versions are enabled).
    pub fn for_server(self) -> Result<ConfigWantsClientVerifier, Error> {
        self.validate()?;
        Ok(ConfigWantsClientVerifier {
            cipher_suites: self.cipher_suites,
            kx_groups: self.kx_groups,
            versions: self.versions,
        })
    }
}
