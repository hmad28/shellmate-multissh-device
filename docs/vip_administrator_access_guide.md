# VIP Administrator SSH Access Guide

This guide explains how `shellmate` enables direct, secure Administrator/Elevated terminal access to your desktop machine from your mobile device using the **VIP Passwordless Access** feature.

---

## 🛠️ How it Works under the Hood

When you connect to your machine via SSH, your privileges are determined by the account you log in with and the SSH server configuration. To get administrative terminal access, `shellmate` handles keys and permissions differently based on your operating system:

```mermaid
graph TD
    A[Mobile App] -->|Connects via SSH| B[Desktop OpenSSH Server]
    B -->|Check Privilege Level| C{Is Admin/Root Toggle Enabled?}
    
    C -->|No| D[Log in to User Account]
    D -->|Reads| E[~/.ssh/authorized_keys]
    
    C -->|Yes: Windows| F[Log in to Administrators Group]
    F -->|Reads| G[C:\ProgramData\ssh\administrators_authorized_keys]
    
    C -->|Yes: macOS/Linux| H[Log in as root]
    H -->|Reads| I[/root/.ssh/authorized_keys]
```

### 1. Windows (OpenSSH Server)
Windows OpenSSH handles administrator logins with strict security restrictions.
- By default, if an SSH user belongs to the `Administrators` group, OpenSSH ignores `C:\Users\<username>\.ssh\authorized_keys` and instead checks the system-wide file:  
  `C:\ProgramData\ssh\administrators_authorized_keys`
- To prevent unauthorized access, this file **MUST** have highly specific Access Control Lists (ACLs):
  - Owner: `SYSTEM` or `Administrators`
  - Inheritance: Disabled
  - Allowed Permissions: `SYSTEM` (Full Control) and `Administrators` (Full Control) only. Any permissions granted to normal users or group accounts will cause OpenSSH to reject the connection.
- **Elevation Process**: Toggling **Enable Administrator Privileges** in `shellmate` generates a temporary PowerShell script, executes it via `Start-Process -Verb RunAs` (triggering the native Windows UAC confirmation dialog), appends the public key to `administrators_authorized_keys`, and configures the exact required security ACLs.

### 2. macOS & Linux
- Logging in as administrator translates to connecting as the `root` user.
- **Elevation Process**: `shellmate` uses native GUI authentication agents:
  - **macOS**: Prompts for credentials via `osascript` (`do shell script ... with administrator privileges`) to authorize the public key inside `/var/root/.ssh/authorized_keys`.
  - **Linux**: Prompts for credentials via `pkexec` to authorize the public key inside `/root/.ssh/authorized_keys`.

---

## 🚀 Step-by-Step Desktop Setup

Before connecting from your mobile phone, make sure the SSH Server is running on your desktop machine:

### 1. Windows Setup
To start and configure the native Windows OpenSSH Server:
1. Open **PowerShell** as **Administrator** and run:
   ```powershell
   # Install OpenSSH Server (if not installed)
   Add-WindowsCapability -Online -Name OpenSSH.Server~~~~0.0.1.0

   # Start the SSH Daemon
   Start-Service sshd

   # Set startup type to Automatic
   Set-Service -Name sshd -StartupType 'Automatic'
   ```
2. Open `shellmate` on your desktop, navigate to **VIP Passwordless Access**, toggle **Enable Administrator Privileges**, enter your administrator username (defaults to `Administrator`), and click **Configure VIP Access**. Click **Yes** on the UAC prompt.

### 2. macOS Setup
1. Open **System Settings** -> **General** -> **Sharing**.
2. Toggle **Remote Login** to **ON**.
3. (Optional) If connecting as `root`, ensure root login is enabled in SSH settings.

### 3. Linux Setup
1. Install and start the OpenSSH server:
   ```bash
   sudo apt update && sudo apt install openssh-server -y
   sudo systemctl enable --now ssh
   ```
2. Ensure `PermitRootLogin` is allowed with public keys. Edit `/etc/ssh/sshd_config` to include:
   ```text
   PermitRootLogin prohibit-password
   ```
   Then reload the service: `sudo systemctl reload ssh`.

---

## 📱 Connecting from your Phone

1. Ensure **E2EE Cloud Sync** is active on both your desktop and phone so that your VIP credential key is synchronized securely.
2. In the `shellmate` mobile app, tap the **VIP Localhost (Admin)** entry in your hosts list.
3. You will instantly get a terminal session running with administrative privileges (Administrator command prompt on Windows, root shell on Unix) without having to type any passwords!

> [!TIP]
> Ensure both your mobile phone and desktop are on the same local network (Wi-Fi), or use a secure VPN/overlay network (like Tailscale) if connecting remotely.
