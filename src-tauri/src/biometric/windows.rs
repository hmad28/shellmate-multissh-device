use super::BiometricProvider;
use log::{info, warn};
use windows_future::AsyncStatus;

const KEY_NAME: &str = "ShellMate.Biometric.Unlock";

/// Windows Hello biometric provider using `KeyCredentialManager`.
pub struct WindowsHelloProvider;

impl BiometricProvider for WindowsHelloProvider {
    fn is_available(&self) -> bool {
        use windows::Security::Credentials::KeyCredentialManager;

        let op = match KeyCredentialManager::IsSupportedAsync() {
            Ok(op) => op,
            Err(e) => {
                warn!("IsSupportedAsync failed: {e}");
                return false;
            }
        };

        loop {
            match op.Status() {
                Ok(status) if status == AsyncStatus::Completed => {
                    return op.GetResults().unwrap_or(false);
                }
                Ok(status) if status == AsyncStatus::Error => {
                    warn!("IsSupportedAsync completed with error");
                    return false;
                }
                Err(e) => {
                    warn!("IsSupportedAsync status check failed: {e}");
                    return false;
                }
                _ => {
                    std::thread::sleep(std::time::Duration::from_millis(20));
                }
            }
        }
    }

    fn verify_user(&self, reason: &str) -> bool {
        match verify_with_hello(reason) {
            Ok(verified) => verified,
            Err(e) => {
                warn!("Windows Hello verification failed: {e}");
                false
            }
        }
    }
}

/// Verify the user via Windows Hello by opening/creating a key.
fn verify_with_hello(reason: &str) -> Result<bool, String> {
    use windows::core::HSTRING;
    use windows::Security::Credentials::{
        KeyCredentialCreationOption, KeyCredentialManager, KeyCredentialStatus,
    };

    info!("Windows Hello verification requested: {reason}");

    let name = HSTRING::from(KEY_NAME);

    // Try to open an existing key — this triggers Windows Hello verification.
    let open_op =
        KeyCredentialManager::OpenAsync(&name).map_err(|e| format!("OpenAsync failed: {e}"))?;

    let mut need_create = false;
    loop {
        match open_op.Status() {
            Ok(status) if status == AsyncStatus::Completed => {
                break;
            }
            Ok(status) if status == AsyncStatus::Error => {
                need_create = true;
                break;
            }
            Err(e) => {
                return Err(format!("OpenAsync status: {e}"));
            }
            _ => {
                std::thread::sleep(std::time::Duration::from_millis(20));
            }
        }
    }

    if !need_create {
        let _retrieval = open_op
            .GetResults()
            .map_err(|e| format!("OpenAsync GetResults: {e}"))?;
        info!("Windows Hello verification succeeded (key opened)");
        return Ok(true);
    }

    // Key doesn't exist — create one (triggers enrollment prompt).
    let create_op = KeyCredentialManager::RequestCreateAsync(
        &name,
        KeyCredentialCreationOption::ReplaceExisting,
    )
    .map_err(|e| format!("RequestCreateAsync failed: {e}"))?;

    loop {
        match create_op.Status() {
            Ok(status) if status == AsyncStatus::Completed => {
                let result = create_op
                    .GetResults()
                    .map_err(|e| format!("Create GetResults: {e}"))?;
                let success = result
                    .Status()
                    .map(|s| s == KeyCredentialStatus::Success)
                    .unwrap_or(false);
                if success {
                    info!("Windows Hello key created and user verified");
                }
                return Ok(success);
            }
            Ok(status) if status == AsyncStatus::Error => {
                return Err("create operation failed".into());
            }
            Err(e) => {
                return Err(format!("create status: {e}"));
            }
            _ => {
                std::thread::sleep(std::time::Duration::from_millis(20));
            }
        }
    }
}
