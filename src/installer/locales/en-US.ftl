# English (US) - Primary translation file for Collective Toolbox Installer
# This is the reference file for all other translations.

# ─── Application ──────────────────────────────────────────────────────────────
# app-name = Collective Toolbox
# ─── Intro Screen ─────────────────────────────────────────────────────────────
welcome-message = Welcome to the installation wizard
theme = Theme:
theme-auto = 🔄 Autodetect
theme-light = ☀ Light
theme-dark = 🌙 Dark
quick-install = Quick Install
customize = Customize...
quick-install-prompt = Use quick install with default settings?
read-license-button = Read full license text...
read-license-prompt = (or 'l' to read the full license text):
license-header = License
press-enter-to-return = Press Enter to return...
press-enter-to-continue = Press Enter to continue...
pager-continue = -- More -- (Enter to continue, q to close):
intro-invalid-input = Please enter 'y' or 'n', or 'l' for license.
prompt-invalid-yes-or-no = Please enter 'y' or 'n'.
prompt-enter-choice = Enter choice [1-{ $choice }, default={ $default }]:
# ─── Options Screen ───────────────────────────────────────────────────────────
options-title = Installation Options
install-dir = Installation directory:
storage-dir = Storage directory:
add-to-start-menu = Add to Start Menu:
add-to-dock = Add to Dock:
add-desktop-shortcut = Add shortcut to Desktop:
add-to-path = Add to PATH:
language = Language 🌏︎:
storage-dir-note = Note: The storage directory will be used for databases, logs, and user data. Ensure sufficient disk space is available.
browse = Browse...
options-configured = Options configured successfully.
parent-directory-not-exists = Warning: Parent directory '{ $path }' does not exist.
create-dir-during-installation = Create it during installation?
enter-number-range = Please enter a number between 1 and { $max }.
# ─── File Picker ──────────────────────────────────────────────────────────────
file-picker-title = Select Location
file-picker-select-folder = Select Folder
file-picker-select-file = Select File
file-picker-save-file = Save File
file-picker-back = Back (Alt+B)
file-picker-forward = Forward (Alt+F)
file-picker-up = Up (Alt+U)
file-picker-refresh = Refresh (Alt+R)
file-picker-new-folder = New Folder (Alt+N)
file-picker-create = Create (Alt+E)
file-picker-cancel-new-folder = Cancel (Alt+A)
file-picker-more-menu = More... (Alt+M)
file-picker-show-hidden = Show Hidden Files (Alt+H)
file-picker-path = Path:
file-picker-file-name = File name:
file-picker-places = Places
file-picker-home = Home
file-picker-desktop = Desktop
file-picker-documents = Documents
file-picker-downloads = Downloads
file-picker-this-pc = This PC
file-picker-empty = (empty)
file-picker-invalid-path = Invalid path: { $path }
file-picker-folder-exists = '{ $name }' already exists
file-picker-create-folder-failed = Failed to create folder: { $error }
file-picker-folder-name-empty = Folder name cannot be empty
file-picker-ok = OK (Alt+O)
# ─── Components Screen ────────────────────────────────────────────────────────
components-title = Select Components
components-instruction = Choose which components to install:
complete = Complete
complete-tooltip = Select all optional components for a full installation
minimal = Minimal
minimal-tooltip = Select only required components for a minimal installation
selected-size = Selected: { $selected } (of { $total } total)
storage-space-note = These file sizes reflect the storage needed by the application itself. Your own documents will occupy additional space; if you're unsure what you'll need, we recommend having at least 20 GB free.
required = (required)
toggle-prompt = Enter component number to toggle:
option-toggle = Toggle
option-continue = Continue
# ─── Progress Screen ──────────────────────────────────────────────────────────
progress-title = Installing...
overall-progress = Overall progress: { $completed }/{ $total } files
current-file = Current file: { $path }
chunk-progress = Chunk { $current }/{ $total }
installation-log = Installation log:
starting-installation = Starting installation...
downloading-file = Downloading: { $path } ({ $chunks } chunks)
downloading-chunk = Downloading chunk { $current }/{ $total }...
using-cached-chunk = Using cached chunk { $current }/{ $total }
file-installed = Installed: { $path } ({ $size } bytes)
retry-error = Retry { $attempt }/{ $max }: { $message }
error = Error: { $message }
retry = Retry
cancel = Cancel
installation-complete-count = Installation complete: { $count } files installed.
# ─── Complete Screen ──────────────────────────────────────────────────────────
complete-title = Installation Complete!
install-success = { $app } has been successfully installed.
quick-install-success = { $app } has now been successfully installed.
launch-after-install = Launch { $app } after closing
finish = Finish
summary = Summary:
start-menu-shortcut = Start Menu shortcut: { $value }
dock-shortcut = Dock shortcut: { $value }
desktop-shortcut = Desktop shortcut: { $value }
added-to-path = Added to PATH: { $value }
yes = Yes
no = No
launch-now-prompt = Launch { $app } now?
launching = Launching { $app }...
thank-you = Thank you for installing { $app }!
# ─── Repair Screen ────────────────────────────────────────────────────────────
repair-title = Repair Installation
repair-description = This will verify and repair your installation. Missing or corrupted files will be re-downloaded.
current-installation = Current installation:
location = Location: { $path }
start-repair = Start Repair
starting-repair = Starting repair...
continue-repair-prompt = Continue with repair?
repair-cancelled = Repair cancelled.
repair-complete = Repair complete!
# ─── Uninstall Screen ─────────────────────────────────────────────────────────
uninstall-title = Uninstall { $app }
uninstall-warning = Warning: This will remove the application from your system.
will-be-removed = The following will be removed:
application-files = Application files: { $path }
desktop-shortcuts = Shortcuts (Start Menu/Dock and Desktop)
path-modifications = PATH modifications
data-not-removed = Note: Your data files will NOT be removed.
data-location = Data location: { $path }
uninstall = Uninstall
starting-uninstall = Starting uninstall...
confirm-uninstall-prompt = Are you sure you want to uninstall?
uninstall-cancelled = Uninstall cancelled.
removing-files = Removing files...
uninstall-complete = { $app } has been uninstalled.
# ─── Navigation ───────────────────────────────────────────────────────────────
back = ← Back
next = Next →
install = Install →
# ─── TUI-specific ─────────────────────────────────────────────────────────────
tui-intro-guidance = This wizard will guide you through the installation process. You can customize the installation options or use the defaults.
unattended-mode = Running in unattended mode - using default values.
yes-no-help = Please enter 'y' or 'n'.
number-choice-help = Please enter a number between 1 and { $max }.
what-to-do = What would you like to do?
parent-dir-warning = Warning: Parent directory '{ $path }' does not exist.
create-dir-prompt = Create it during installation?
# ─── Window Titles ────────────────────────────────────────────────────────────
window-installer = { $app } Installer
window-repair = { $app } - Repair
window-uninstall = { $app } - Uninstall
