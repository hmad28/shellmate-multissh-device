use async_trait::async_trait;
use russh::client;
use russh::keys::key::PublicKey;

/// Minimal russh client handler. For MVP we accept all server keys (TOFU is
/// deferred to Phase 4 known-hosts management). See PRD §8.
pub struct ClientHandler;

#[async_trait]
impl client::Handler for ClientHandler {
    type Error = russh::Error;

    async fn check_server_key(
        &mut self,
        _server_public_key: &PublicKey,
    ) -> Result<bool, Self::Error> {
        // TODO(Phase 4): verify against known_hosts table, prompt user on mismatch.
        Ok(true)
    }
}
