//! Authenticated AW -> dotlink remote-Start bridge primitives.
//!
//! A RemoteStartAuthorization is source-fencing evidence, not a dotlink grant.
//! It is accepted only from an OAuth-authenticated HTTP client, is bound to the
//! exact live runtime generation and immutable host/profile identity, and is
//! rechecked against the actual semantic operation/arguments before the existing
//! JobManager owner may claim it.

use std::{
    sync::Arc,
    time::{SystemTime, UNIX_EPOCH},
};

use anyhow::{Result, anyhow, bail};
use aw_tools::{
    OperationId,
    execution_ref::{ExecutionRef, ExecutionSegment},
    remote_start::{
        MAX_REMOTE_START_LEASE_MILLIS, RemoteRuntimeLease, RemoteSessionAction,
        RemoteStartAuthorization,
    },
};
use rmcp::{
    model::RequestMetaObject,
    service::{RequestContext, RoleServer},
};
use serde_json::Value;
use sha2::{Digest, Sha256};

pub(crate) use aw_tools::remote_start::arguments_sha256;

pub(crate) const REMOTE_START_META_KEY: &str = "io.abird.aw/remoteStart";
pub(crate) const REMOTE_START_LEASE_MILLIS: u64 = 30_000;

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct AuthenticatedMcpClient {
    client_id: Arc<str>,
}

impl AuthenticatedMcpClient {
    pub(crate) fn new(client_id: impl Into<Arc<str>>) -> Result<Self> {
        let client_id = client_id.into();
        if client_id.is_empty() || client_id.len() > 512 || client_id.chars().any(char::is_control)
        {
            bail!("invalid authenticated MCP client identity");
        }
        Ok(Self { client_id })
    }

    pub(crate) fn id(&self) -> &str {
        self.client_id.as_ref()
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct BridgeTarget {
    host: Arc<str>,
    profile: Arc<str>,
}

impl BridgeTarget {
    pub(crate) fn new(host: impl Into<Arc<str>>, profile: impl Into<Arc<str>>) -> Result<Self> {
        let host = host.into();
        let profile = profile.into();
        for (label, value) in [("host", host.as_ref()), ("profile", profile.as_ref())] {
            if value.is_empty() || value.len() > 128 || value.chars().any(char::is_control) {
                bail!("invalid remote bridge {label}");
            }
        }
        Ok(Self { host, profile })
    }

    pub(crate) fn host(&self) -> &str {
        self.host.as_ref()
    }

    pub(crate) fn profile(&self) -> &str {
        self.profile.as_ref()
    }

    pub(crate) fn lease(&self, generation: [u8; 32]) -> Result<RemoteRuntimeLease> {
        if generation == [0; 32] {
            bail!("remote bridge runtime generation is unavailable");
        }
        let issued_at_unix_ms = now_unix_ms()?;
        let expires_at_unix_ms = issued_at_unix_ms
            .checked_add(REMOTE_START_LEASE_MILLIS.min(MAX_REMOTE_START_LEASE_MILLIS))
            .ok_or_else(|| anyhow!("remote Start lease timestamp overflow"))?;
        let lease = RemoteRuntimeLease {
            target_host: self.host().to_owned(),
            target_profile: self.profile().to_owned(),
            generation,
            issued_at_unix_ms,
            expires_at_unix_ms,
        };
        lease
            .validate()
            .map_err(|_| anyhow!("invalid remote Start runtime lease"))?;
        Ok(lease)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum StartClass {
    Submit,
    Control,
}

impl StartClass {
    fn action(self) -> RemoteSessionAction {
        match self {
            Self::Submit => RemoteSessionAction::Submit,
            Self::Control => RemoteSessionAction::Control,
        }
    }
}

#[derive(Clone, Debug)]
pub(crate) struct ValidatedRemoteStart {
    authorization: RemoteStartAuthorization,
    semantic_hash: [u8; 32],
}

impl ValidatedRemoteStart {
    pub(crate) fn semantic_hash(&self) -> [u8; 32] {
        self.semantic_hash
    }

    pub(crate) fn expires_at_unix_ms(&self) -> u64 {
        self.authorization.intent.target.expires_at_unix_ms
    }

    pub(crate) fn claim_key(&self) -> [u8; 32] {
        let mut hash = Sha256::new();
        hash.update(b"dotlink-aw-remote-start-claim-v1\0");
        // One AW authorization ID owns exactly one remote effect-attempt slot.
        // Authenticated client identity belongs in the semantic fingerprint,
        // not the lookup key: the same proof from another OAuth client must
        // collide and fail closed rather than allocate a second effect slot.
        hash.update(self.authorization.authorization_id);
        hash.finalize().into()
    }

    pub(crate) fn replay_key(&self) -> String {
        format!("aw-start:{:x}", Sha256::digest(self.claim_key()))
    }
}

pub(crate) fn now_unix_ms() -> Result<u64> {
    let millis = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|_| anyhow!("system clock is before the Unix epoch"))?
        .as_millis();
    u64::try_from(millis).map_err(|_| anyhow!("system clock exceeds remote Start timestamp range"))
}

fn authenticated_client(context: &RequestContext<RoleServer>) -> Result<AuthenticatedMcpClient> {
    let parts = context
        .extensions
        .get::<http::request::Parts>()
        .ok_or_else(|| anyhow!("remote Start requires authenticated HTTP transport"))?;
    parts
        .extensions
        .get::<AuthenticatedMcpClient>()
        .cloned()
        .ok_or_else(|| anyhow!("remote Start requires authenticated MCP client identity"))
}

pub(crate) fn has_metadata(meta: &RequestMetaObject) -> bool {
    meta.get(REMOTE_START_META_KEY).is_some()
}

fn authorization_from_meta(meta: &RequestMetaObject) -> Result<Option<RemoteStartAuthorization>> {
    let Some(value) = meta.get(REMOTE_START_META_KEY) else {
        return Ok(None);
    };
    let authorization = serde_json::from_value::<RemoteStartAuthorization>(value.clone())
        .map_err(|_| anyhow!("invalid remote Start metadata"))?;
    authorization
        .validate()
        .map_err(|_| anyhow!("invalid remote Start authorization"))?;
    Ok(Some(authorization))
}

fn semantic_hash(
    authorization: &RemoteStartAuthorization,
    client_id: &str,
    operation: OperationId,
    arguments: &Value,
) -> [u8; 32] {
    let mut hash = Sha256::new();
    hash.update(b"dotlink-aw-remote-start-semantic-v1\0");
    hash.update((client_id.len() as u64).to_be_bytes());
    hash.update(client_id.as_bytes());
    hash.update(authorization.authorization_id);
    hash.update(operation.semantic_name().as_bytes());
    hash.update(arguments_sha256(arguments));
    hash.finalize().into()
}

fn finite_suffix(reference: &str) -> Result<(String, Option<usize>)> {
    let parsed = ExecutionRef::parse(reference).map_err(|_| anyhow!("invalid remote Start ref"))?;
    let resolved = parsed
        .resolve(None, None)
        .map_err(|_| anyhow!("remote Start ref must already be canonical"))?;
    let suffix = resolved.suffix();
    let job = suffix
        .iter()
        .find_map(|segment| match segment {
            ExecutionSegment::Job(job) => Some(job.as_str().to_owned()),
            _ => None,
        })
        .ok_or_else(|| anyhow!("remote control Start must bind an exact finite Job"))?;
    let node = suffix.iter().find_map(|segment| match segment {
        ExecutionSegment::Node(node) => Some(node.get() as usize),
        _ => None,
    });
    Ok((job, node))
}

pub(crate) struct ValidationContext<'a> {
    pub(crate) client_id: &'a str,
    pub(crate) target: &'a BridgeTarget,
    pub(crate) generation: [u8; 32],
    pub(crate) now_unix_ms: u64,
    pub(crate) operation: OperationId,
    pub(crate) class: StartClass,
    pub(crate) arguments: &'a Value,
    pub(crate) control_ref: Option<&'a str>,
}

pub(crate) fn validate_authorization(
    authorization: RemoteStartAuthorization,
    request: ValidationContext<'_>,
) -> Result<ValidatedRemoteStart> {
    let client = AuthenticatedMcpClient::new(request.client_id)?;
    authorization
        .validate_for_target(
            request.target.host(),
            request.target.profile(),
            &request.generation,
            request.now_unix_ms,
        )
        .map_err(|_| anyhow!("remote Start is expired or targets another runtime"))?;

    if authorization.intent.operation != request.operation.semantic_name()
        || authorization.intent.session_action != request.class.action()
        || authorization.intent.arguments_sha256 != arguments_sha256(request.arguments)
    {
        bail!("remote Start differs from the exact semantic request");
    }

    if let Some(reference) = request.control_ref {
        let (requested_job, requested_node) = finite_suffix(reference)?;
        let (bound_job, bound_node) = finite_suffix(&authorization.intent.canonical_ref)?;
        if bound_job != requested_job || bound_node != requested_node {
            bail!("remote Start finite target differs from requested Job ref");
        }
    }

    let semantic_hash = semantic_hash(
        &authorization,
        client.id(),
        request.operation,
        request.arguments,
    );
    Ok(ValidatedRemoteStart {
        authorization,
        semantic_hash,
    })
}

pub(crate) fn validate_request(
    context: &RequestContext<RoleServer>,
    target: &BridgeTarget,
    generation: [u8; 32],
    operation: OperationId,
    class: StartClass,
    arguments: &Value,
    control_ref: Option<&str>,
) -> Result<Option<ValidatedRemoteStart>> {
    let Some(authorization) = authorization_from_meta(&context.meta)? else {
        return Ok(None);
    };
    let client = authenticated_client(context)?;
    validate_authorization(
        authorization,
        ValidationContext {
            client_id: client.id(),
            target,
            generation,
            now_unix_ms: now_unix_ms()?,
            operation,
            class,
            arguments,
            control_ref,
        },
    )
    .map(Some)
}

#[cfg(test)]
mod tests {
    use super::*;
    use aw_tools::remote_start::{RemoteStartIntent, RemoteStartProvenance, RemoteStartSource};

    fn auth(arguments: &Value, generation: [u8; 32]) -> RemoteStartAuthorization {
        RemoteStartAuthorization::new(
            RemoteStartIntent {
                version: aw_tools::remote_start::REMOTE_START_VERSION,
                original_invocation_id: "invocation".into(),
                turn_id: 7,
                call_id: "call".into(),
                effect_attempt_id: "attempt".into(),
                operation: OperationId::Bash.semantic_name().into(),
                canonical_ref: "/d:d/i:i/p:p/a:0/t:7".into(),
                arguments_sha256: arguments_sha256(arguments),
                original_request_sha256: [4; 32],
                session_action: RemoteSessionAction::Submit,
                target: RemoteRuntimeLease {
                    target_host: "l5".into(),
                    target_profile: "default".into(),
                    generation,
                    issued_at_unix_ms: 100,
                    expires_at_unix_ms: 200,
                },
            },
            RemoteStartProvenance {
                actor: "actor".into(),
                source: RemoteStartSource {
                    source_kind: "host".into(),
                    source_id: "shell".into(),
                    source_epoch: 1,
                    site: "l5".into(),
                    resource_scope: "root".into(),
                    grant_sha256: [5; 32],
                },
                policy_revision_sha256: [6; 32],
                policy_admission_id: "policy".into(),
                session_id: "session".into(),
                branch_id: "branch".into(),
                writer_epoch: 1,
                membership_epoch: 1,
                audience_epoch: 1,
            },
        )
        .unwrap()
    }

    #[test]
    fn argument_digest_is_object_order_independent() {
        let left = serde_json::json!({"b":[2,1],"a":{"z":true,"x":"v"}});
        let right = serde_json::json!({"a":{"x":"v","z":true},"b":[2,1]});
        assert_eq!(arguments_sha256(&left), arguments_sha256(&right));
    }

    #[test]
    fn replay_slot_is_authorization_owned_and_client_identity_is_semantic_bound() {
        let arguments = serde_json::json!({"command":"true"});
        let authorization = auth(&arguments, [9; 32]);
        let now = now_unix_ms().unwrap();
        let mut intent = authorization.intent.clone();
        intent.target.issued_at_unix_ms = now;
        intent.target.expires_at_unix_ms = now + 1_000;
        let authorization =
            RemoteStartAuthorization::new(intent, authorization.provenance.clone()).unwrap();
        let first = ValidatedRemoteStart {
            semantic_hash: semantic_hash(&authorization, "client-a", OperationId::Bash, &arguments),
            authorization: authorization.clone(),
        };
        let second = ValidatedRemoteStart {
            semantic_hash: semantic_hash(&authorization, "client-b", OperationId::Bash, &arguments),
            authorization,
        };
        assert_eq!(first.claim_key(), second.claim_key());
        assert_eq!(first.replay_key(), second.replay_key());
        assert_ne!(first.semantic_hash(), second.semantic_hash());

        let manager = crate::jobs::JobManager::with_spool_limit(
            std::time::Duration::from_secs(60),
            crate::limits::DEFAULT_JOB_SPOOL_BYTES,
        )
        .unwrap();
        let guard = match manager.claim_remote_start(&first).unwrap() {
            crate::jobs::RemoteStartDecision::New(guard) => guard,
            _ => panic!("first authenticated client must own the Start claim"),
        };
        assert!(manager.claim_remote_start(&second).is_err());
        guard.release_before_effect();
    }

    #[test]
    fn validation_binds_target_generation_operation_arguments_and_expiry() {
        let arguments = serde_json::json!({"command":"true","interactive":false});
        let generation = [9; 32];
        let target = BridgeTarget::new("l5", "default").unwrap();
        let authorization = auth(&arguments, generation);
        let validated = validate_authorization(
            authorization.clone(),
            ValidationContext {
                client_id: "client-a",
                target: &target,
                generation,
                now_unix_ms: 150,
                operation: OperationId::Bash,
                class: StartClass::Submit,
                arguments: &arguments,
                control_ref: None,
            },
        )
        .unwrap();
        assert_ne!(validated.claim_key(), [0; 32]);
        assert!(
            validate_authorization(
                authorization.clone(),
                ValidationContext {
                    client_id: "client-a",
                    target: &target,
                    generation: [8; 32],
                    now_unix_ms: 150,
                    operation: OperationId::Bash,
                    class: StartClass::Submit,
                    arguments: &arguments,
                    control_ref: None,
                },
            )
            .is_err()
        );
        assert!(
            validate_authorization(
                authorization.clone(),
                ValidationContext {
                    client_id: "client-a",
                    target: &target,
                    generation,
                    now_unix_ms: 201,
                    operation: OperationId::Bash,
                    class: StartClass::Submit,
                    arguments: &arguments,
                    control_ref: None,
                },
            )
            .is_err()
        );
        assert!(
            validate_authorization(
                authorization.clone(),
                ValidationContext {
                    client_id: "client-a",
                    target: &target,
                    generation,
                    now_unix_ms: 150,
                    operation: OperationId::Batch,
                    class: StartClass::Submit,
                    arguments: &arguments,
                    control_ref: None,
                },
            )
            .is_err()
        );
        let changed = serde_json::json!({"command":"false","interactive":false});
        assert!(
            validate_authorization(
                authorization,
                ValidationContext {
                    client_id: "client-a",
                    target: &target,
                    generation,
                    now_unix_ms: 150,
                    operation: OperationId::Bash,
                    class: StartClass::Submit,
                    arguments: &changed,
                    control_ref: None,
                },
            )
            .is_err()
        );
    }

    #[test]
    fn job_manager_remote_start_claim_is_single_use_and_semantic_bound() {
        let arguments = serde_json::json!({"command":"true"});
        let generation = [9; 32];
        let now = now_unix_ms().unwrap();
        let mut authorization = auth(&arguments, generation);
        authorization.intent.target.issued_at_unix_ms = now;
        authorization.intent.target.expires_at_unix_ms = now + 1_000;
        authorization = RemoteStartAuthorization::new(
            authorization.intent.clone(),
            authorization.provenance.clone(),
        )
        .unwrap();
        let target = BridgeTarget::new("l5", "default").unwrap();
        let validated = validate_authorization(
            authorization,
            ValidationContext {
                client_id: "client-a",
                target: &target,
                generation,
                now_unix_ms: now,
                operation: OperationId::Bash,
                class: StartClass::Submit,
                arguments: &arguments,
                control_ref: None,
            },
        )
        .unwrap();
        let manager = crate::jobs::JobManager::with_spool_limit(
            std::time::Duration::from_secs(60),
            crate::limits::DEFAULT_JOB_SPOOL_BYTES,
        )
        .unwrap();
        let guard = match manager.claim_remote_start(&validated).unwrap() {
            crate::jobs::RemoteStartDecision::New(guard) => guard,
            _ => panic!("first claim must be new"),
        };
        assert!(matches!(
            manager.claim_remote_start(&validated).unwrap(),
            crate::jobs::RemoteStartDecision::Unknown
        ));
        guard
            .mark_ready(crate::jobs::RemoteStartReceipt::Mcp(
                rmcp::model::CallToolResult::success(vec![rmcp::model::ContentBlock::text(
                    "j:abc process running rev=1",
                )]),
            ))
            .unwrap();
        assert!(matches!(
            manager.claim_remote_start(&validated).unwrap(),
            crate::jobs::RemoteStartDecision::Existing(crate::jobs::RemoteStartReceipt::Mcp(
                ref result
            )) if result.content[0].as_text().is_some_and(|text| text.text == "j:abc process running rev=1")
        ));

        let mut changed = validated.clone();
        changed.semantic_hash[0] ^= 1;
        assert!(manager.claim_remote_start(&changed).is_err());
    }

    #[test]
    fn old_generation_authorization_is_rejected_after_runtime_restart() {
        let arguments = serde_json::json!({"command":"true"});
        let first = crate::jobs::JobManager::with_spool_limit(
            std::time::Duration::from_secs(60),
            crate::limits::DEFAULT_JOB_SPOOL_BYTES,
        )
        .unwrap();
        let second = crate::jobs::JobManager::with_spool_limit(
            std::time::Duration::from_secs(60),
            crate::limits::DEFAULT_JOB_SPOOL_BYTES,
        )
        .unwrap();
        let first_generation = first.runtime_generation_fingerprint();
        let second_generation = second.runtime_generation_fingerprint();
        assert_ne!(first_generation, second_generation);

        let now = now_unix_ms().unwrap();
        let mut authorization = auth(&arguments, first_generation);
        authorization.intent.target.issued_at_unix_ms = now;
        authorization.intent.target.expires_at_unix_ms = now + 1_000;
        authorization = RemoteStartAuthorization::new(
            authorization.intent.clone(),
            authorization.provenance.clone(),
        )
        .unwrap();
        let target = BridgeTarget::new("l5", "default").unwrap();
        assert!(
            validate_authorization(
                authorization,
                ValidationContext {
                    client_id: "client-a",
                    target: &target,
                    generation: second_generation,
                    now_unix_ms: now,
                    operation: OperationId::Bash,
                    class: StartClass::Submit,
                    arguments: &arguments,
                    control_ref: None,
                },
            )
            .is_err(),
            "a proof for the previous live runtime must fail before replay claim/effect"
        );
    }

    #[test]
    fn finite_control_target_extracts_only_exact_job_and_node() {
        assert_eq!(
            finite_suffix("/d:d/i:i/p:p/a:0/j:abc/n:3").unwrap(),
            ("abc".into(), Some(3))
        );
        assert_eq!(
            finite_suffix("/d:d/i:i/p:p/a:0/j:abc").unwrap(),
            ("abc".into(), None)
        );
        assert!(finite_suffix("/d:d/i:i/p:p/a:0/t:7").is_err());
    }
}
