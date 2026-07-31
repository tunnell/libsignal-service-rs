use http::Method;
use libsignal_core::DeviceId;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{
    configuration::Endpoint, utils::serde_device_id,
    websocket::registration::DeviceActivationRequest,
};

use super::{
    response::HttpResponseExt, HttpAuth, HttpAuthOverride, PushService,
    ServiceError,
};

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LinkAccountAttributes {
    pub fetches_messages: bool,
    pub name: String,
    pub registration_id: u32,
    pub pni_registration_id: u32,
    pub capabilities: LinkCapabilities,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LinkCapabilities {
    /// `ATTACHMENT_BACKFILL` on Signal Server.
    pub attachment_backfill: bool,
    /// Sparse Post-Quantum Ratchet (`SPARSE_POST_QUANTUM_RATCHET` on Signal Server).
    ///
    /// Required for all devices; the server returns 409 if a linking device omits this capability
    /// while the account already has it on any existing device.
    pub spqr: bool,
    /// `USERNAME_CHANGE_SYNC_MESSAGE` on Signal Server, flagged preventDowngrade —
    /// omitting it while the account's primary already advertises it makes
    /// PUT /v1/devices/link fail with HTTP 409 (observed on hardware 2026-07-31).
    pub username_change_sync_message: bool,
}

// Mirrors upstream whisperfish/libsignal-service-rs fd48165 ("Update link
// capabilities", 2026-07-05), which tracks Signal-Desktop. The former
// delete_sync / versioned_expiration_timer / ssre2 capability names are
// retired server-side.
impl Default for LinkCapabilities {
    fn default() -> Self {
        Self {
            attachment_backfill: false,
            spqr: true,
            username_change_sync_message: true,
        }
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LinkResponse {
    #[serde(rename = "uuid")]
    pub aci: Uuid,
    pub pni: Uuid,
    #[serde(with = "serde_device_id")]
    pub device_id: DeviceId,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LinkRequest {
    pub verification_code: String,
    pub account_attributes: LinkAccountAttributes,
    #[serde(flatten)]
    pub device_activation_request: DeviceActivationRequest,
}

impl PushService {
    pub async fn link_device(
        &mut self,
        link_request: &LinkRequest,
        http_auth: HttpAuth,
    ) -> Result<LinkResponse, ServiceError> {
        self.request(
            Method::PUT,
            Endpoint::service("/v1/devices/link"),
            HttpAuthOverride::Identified(http_auth),
        )?
        .json(&link_request)
        .send()
        .await?
        .service_error_for_status()
        .await?
        .json()
        .await
        .map_err(Into::into)
    }
}
