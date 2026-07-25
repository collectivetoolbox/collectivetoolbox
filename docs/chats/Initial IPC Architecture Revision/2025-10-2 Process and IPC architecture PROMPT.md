Hi Copilot, I'm hoping to get some architectural advice on how to accomplish what I want for subprocess handling and IPC. I'll try to summarize the constraints, and then provide my current logic.

# Overview
- The application typically runs from a single binary. That process I call the "workspace".
- The workspace then allows the user to open documents, as "renderer" processes (document runtime).
- Each "renderer" process, and the root workspace process, should be able to run certain code: file format conversions, i/o, network access, etc. depending on the permissions the document loaded into the renderer has been granted.
  - My thinking is that those features should be handled by the renderer creating additional subprocesses, roughly one subprocess for each Rust module that it needs to run code from (there aren't strict, clear boundaries between the modules, though - for instance, the UI might call some APIs from storage to load icons from disk - possibly this represents a design flaw).
- Renderers can start other renderers, representing documents embedded within other documents.
- Renderers, and the workspace, should be able to make function calls to the subprocesses they spawn, as well as to the workspace (and embedded documents should be able to pass limited messages back to their embedder).
- If a process panics or is killed, it should take responsibility for killing its own subprocesses (perhaps recursively).
- If a renderer process panics, it should only bring down that document or subdocument, not the whole workspace or other documents.
- Basically I'm trying to imitate how web browsers isolate documents from each other so they can't crash the browser or other documents.
- Right now, I'm handling IPC as JSON messages passed via web servers and processes polling the server. I'm pretty sure that's terrible and wrong - consider a renderer process passing rendered frames as bitmaps crammed into JSON - but I'm not sure what the *right* way to do this is.
- Some library resources can *only be safely called from a single process* - so far, that's only come up with the database. So, I think I need to ensure that a subprocess cannot call the underlying database APIs directly. Properly speaking, they belong in the storage module, but I stuffed them into the IPC module because of this, and then added wrapper functions in the storage module that make the IPC calls. Again, this feels like it's awful, but I'm not sure what the right approach is.

Could you review my architecture and provide me an action plan for refactoring my application to accomplish my underlying aims? Please identify significant flaws in the plan I summarized above.


# bin.rs
```
#[tokio::main]
pub async fn main() -> Result<()> {
 ctoolbox::workspace::entry().await
}
```

# workspace.rs
```
static PROCESS_MANAGER: OnceCell<Arc<Mutex<ProcessManager>>> = OnceCell::new();

/// Initializes the global process manager singleton.
pub fn set_global_process_manager(
 pm: Arc<Mutex<ProcessManager>>,
) -> anyhow::Result<()> {
 PROCESS_MANAGER
 .set(pm)
 .map_err(|_| anyhow::anyhow!("ProcessManager already set"))
}

/// Gets a reference to the global process manager.
pub fn get_global_process_manager() -> anyhow::Result<Arc<Mutex<ProcessManager>>>
{
 PROCESS_MANAGER
 .get()
 .cloned()
 .ok_or_else(|| anyhow::anyhow!("ProcessManager not initialized"))
}

pub struct WorkspaceState {
 pub processes: Arc<Mutex<ProcessManager>>,
}

/// Initializes the global process manager singleton and returns the `WorkspaceState`.
fn initialize_main_process_state(
 invocation: &cli::Invocation,
) -> anyhow::Result<WorkspaceState> {
 let mut pm = ProcessManager {
 port: 0,
 ipc_control_channel: Channel {
 name: String::new(),
 port: 0,
 authentication_key: String::new(),
 },
 processes: HashMap::new(),
 };
 pm.processes.insert(
 0,
 Process {
 pid: 0,
 system_pid: 0,
 channel: Channel {
 name: String::new(),
 port: 0,
 authentication_key: String::new(),
 },
 parent_pid: 0,
 },
 );

 setup_panic_hooks(&pm);

 let pm_arc = Arc::new(Mutex::new(pm));
 set_global_process_manager(pm_arc.clone())?;

 Ok(WorkspaceState {
 processes: get_global_process_manager()?,
 })
}

pub async fn entry() -> anyhow::Result<()> {
 // Parse either a subprocess invocation or a user CLI command set.
 let invocation = cli::parse_invocation()?;

 // If it's a subprocess, initialize and exit early.
 if let cli::Invocation::Subprocess(subproc) = &invocation {
 subprocess_init(subproc)?;
 return Ok(());
 }
 let cli = invocation.expect_cli();

 // Proceed with full application boot.
 let state = initialize_main_process_state(&invocation)?;
 boot(state, cli).await?;
 Ok(())
}

async fn boot(state: WorkspaceState, args: &Cli) -> Result<()> {
 let port: u16 = if args.ctoolbox_ipc_port.is_none() {
 pick_unused_port().expect("No ports free")
 } else {
 args.ctoolbox_ipc_port.unwrap()
 };
 let authentication_key: String = generate_authentication_key();
 let ipc_control_channel: Channel = Channel {
 name: "ipc_control".to_string(),
 port,
 authentication_key: authentication_key.clone(),
 };

 let pm_arc = get_global_process_manager()?;
 let process_manager = pm_arc.lock();
 if process_manager.is_err() {
 return Err(anyhow::anyhow!("Failed to lock process manager"));
 }
 let mut process_manager = process_manager.unwrap();
 process_manager.port = port;
 process_manager.ipc_control_channel = ipc_control_channel.clone();
 drop(process_manager);

 let authentication_key_for_server = authentication_key.clone();
 log!("Waiting for IPC server to come up...");
 let pm_for_server = pm_arc.clone();
 std::thread::spawn(move || {
 start_ipc_server(pm_for_server, port, authentication_key_for_server);
 });
 loop {
 let response = maybe_send_message(
 &ipc_control_channel,
 json!({ "type": "ipc_ping" }).as_str(),
 );
 if response.is_ok() {
 break;
 }
 sleep(1);
 }
 log!("IPC server OK.");

 loop {
 sleep(10);
 }
}

```

# utilities.rs

```
pub fn this_pid() -> Vec<u8> {
 std::process::id().to_string().into_bytes()
}

pub fn is_subprocess(invocation: &Invocation) -> bool {
 invocation.is_subprocess()
}

pub fn get_service_name(invocation: &Invocation) -> String {
 if is_subprocess(invocation) {
 invocation
 .subprocess()
 .map(|s| s.service_name.clone())
 .unwrap_or_default()
 } else {
 String::new()
 }
}

pub fn setup_panic_hooks(manager: &ProcessManager) {
 // Custom panic hook to clean up subprocesses, only set for the main process.
 let _ = std::panic::take_hook();
 let hook_manager = manager.clone();
 cfg_if! {
 if #[cfg(target_family = "wasm")] {
 console_error_panic_hook::set_once();
 } else {
 }
 }
 let default_panic_hook = std::panic::take_hook();
 let shutdown_manager = manager.clone();
 std::panic::set_hook(Box::new(move |info| {
 cleanup_processes(&hook_manager);
 default_panic_hook(info);
 }));
 // FIXME: can't replace handler, see https://github.com/Detegr/rust-ctrlc/issues/106 and https://github.com/Detegr/rust-ctrlc/issues/118
 let _ignore_error = ctrlc::set_handler(move || {
 shutdown(&shutdown_manager);
 });
}

pub fn send_message(channel: &Channel, message: &str) -> String {
 crate::workspace::ipc::send_message(channel, message)
}

pub fn maybe_send_message(
 channel: &Channel,
 message: &str,
) -> Result<String, Box<dyn Error>> {
 crate::workspace::ipc::maybe_send_message(channel, message)
}

pub fn wait_for_message(channel: &Channel, msgid: u64) -> String {
 crate::workspace::ipc::wait_for_message(channel, msgid)
}

pub fn listen_for_message(channel: &Channel, msgid: u64) -> String {
 crate::workspace::ipc::listen_for_message(channel, msgid)
}

pub fn call_ipc_sync(
 channel: &Channel,
 method: &str,
 args: Value,
) -> Result<String> {
 crate::workspace::ipc::call_ipc_sync(channel, method, args)
}

pub fn call_ipc(
 channel: &Channel,
 method: &str,
 args: Value,
) -> Result<String> {
 crate::workspace::ipc::call_ipc(channel, method, args)
}

pub fn shutdown(process_manager: &ProcessManager) {
 println!("Shutting down");
 cleanup_processes(process_manager);
 std::process::exit(1);
}

fn cleanup_processes(process_manager: &ProcessManager) {
 // let json = json!(process_manager);
 // println!("Cleaning up processes: {}", json);
 process_manager
 .processes
 .iter()
 .for_each(|(_id, process_record)| {
 let mut s = System::new_all();
 if let Some(process) = get_ctoolbox_process(
 &mut s,
 process_record
 .system_pid
 .try_into()
 .expect("Invalid PID for this platform"),
 ) {
 // println!("Killing process {}", process.pid());
 process.kill();
 }
 });
}

```

# renderer.rs

```
struct Document {
 data: Vec<u8>,
}

pub fn start(document: Vec<u8>) {
 Document { data: document };
}

```

# cli.rs

```

#[derive(Debug)]
pub enum Invocation {
 Subprocess(SubprocessArgs),
 User(Cli),
}

impl Invocation {
 pub fn is_subprocess(&self) -> bool {
 matches!(self, Invocation::Subprocess(_))
 }

 pub fn subprocess(&self) -> Option<&SubprocessArgs> {
 if let Invocation::Subprocess(s) = self {
 Some(s)
 } else {
 None
 }
 }

 pub fn expect_cli(&self) -> &Cli {
 match self {
 Invocation::User(cli) => cli,
 Invocation::Subprocess(_) => {
 panic!("Called expect_cli() on a subprocess invocation")
 }
 }
 }
}

// -----------------------------------------------------------------------------
// Subprocess Argument Structures
// -----------------------------------------------------------------------------

#[derive(Debug, Clone)]
pub struct SubprocessArgs {
 pub ipc: IpcEndpoint,
 pub subprocess_index: u32,
 pub service_name: String,
 pub extra: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct IpcEndpoint {
 pub port: u16,
 pub identity: String,
}

impl FromStr for IpcEndpoint {
 type Err = anyhow::Error;

 fn from_str(s: &str) -> Result<Self> {
 // Expect "<port>:<identity>"
 let mut parts = s.splitn(2, ':');
 let port_part = parts
 .next()
 .ok_or_else(|| anyhow!("Missing port in IPC specification"))?;
 let identity_part = parts
 .next()
 .ok_or_else(|| anyhow!("Missing identity in IPC specification"))?;
 let port: u16 = port_part
 .parse()
 .map_err(|e| anyhow!("Invalid port '{port_part}': {e}"))?;
 Ok(IpcEndpoint {
 port,
 identity: identity_part.to_string(),
 })
 }
}

fn parse_subprocess(raw: &[String]) -> Result<SubprocessArgs> {
 // raw[0] is program name
 // raw[1] == SUBPROCESS_MAGIC
 if raw.len() < 5 {
 bail!(
 "Subprocess invocation requires at least 4 arguments after program name: \
 <magic> <port:identity> <index> <type> [extra..]"
 );
 }
 let ipc = IpcEndpoint::from_str(&raw[2])?;
 let subprocess_index: u32 = raw[3]
 .parse()
 .map_err(|e| anyhow!("Invalid subprocess index '{}': {}", raw[3], e))?;
 let service_name = raw[4].clone();
 let extra = if raw.len() > 5 {
 raw[5..].to_vec()
 } else {
 Vec::new()
 };
 Ok(SubprocessArgs {
 ipc,
 subprocess_index,
 service_name,
 extra,
 })
}

// Public parsing entry point used by lib::entry().
pub fn parse_invocation() -> Result<Invocation> {
 let raw: Vec<String> = env::args().collect();
 if raw.get(1).is_some_and(|s| s == IPC_ARG) {
 let sub = parse_subprocess(&raw)?;
 return Ok(Invocation::Subprocess(sub));
 }
 // Fallback: user CLI
 let cli = Cli::parse(); // Clap handles errors & help display
 Ok(Invocation::User(cli))
}

#[derive(Parser, Debug)]
#[command(
 name = "ctoolbox",
 version,
 about = "Collective Toolbox",
 disable_help_subcommand = true
)]
pub struct Cli {
 #[arg(long)]
 pub ctoolbox_ipc_port: Option<u16>,

 #[command(subcommand)]
 pub command: Option<Command>,
}

```


#[derive(Debug)]
pub enum Invocation {
 Subprocess(SubprocessArgs),
 User(Cli),
}

impl Invocation {
 pub fn is_subprocess(&self) -> bool {
 matches!(self, Invocation::Subprocess(_))
 }

 pub fn subprocess(&self) -> Option<&SubprocessArgs> {
 if let Invocation::Subprocess(s) = self {
 Some(s)
 } else {
 None
 }
 }

 pub fn expect_cli(&self) -> &Cli {
 match self {
 Invocation::User(cli) => cli,
 Invocation::Subprocess(_) => {
 panic!("Called expect_cli() on a subprocess invocation")
 }
 }
 }
}

// -----------------------------------------------------------------------------
// Subprocess Argument Structures
// -----------------------------------------------------------------------------

#[derive(Debug, Clone)]
pub struct SubprocessArgs {
 pub ipc: IpcEndpoint,
 pub subprocess_index: u32,
 pub service_name: String,
 pub extra: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct IpcEndpoint {
 pub port: u16,
 pub identity: String,
}

impl FromStr for IpcEndpoint {
 type Err = anyhow::Error;

 fn from_str(s: &str) -> Result<Self> {
 // Expect "<port>:<identity>"
 let mut parts = s.splitn(2, ':');
 let port_part = parts
 .next()
 .ok_or_else(|| anyhow!("Missing port in IPC specification"))?;
 let identity_part = parts
 .next()
 .ok_or_else(|| anyhow!("Missing identity in IPC specification"))?;
 let port: u16 = port_part
 .parse()
 .map_err(|e| anyhow!("Invalid port '{port_part}': {e}"))?;
 Ok(IpcEndpoint {
 port,
 identity: identity_part.to_string(),
 })
 }
}

// Manual parsing of subprocess arguments (since the magic token won't play
// nicely as a Clap subcommand).
fn parse_subprocess(raw: &[String]) -> Result<SubprocessArgs> {
 // raw[0] is program name
 // raw[1] == SUBPROCESS_MAGIC
 if raw.len() < 5 {
 bail!(
 "Subprocess invocation requires at least 4 arguments after program name: \
 <magic> <port:identity> <index> <type> [extra..]"
 );
 }
 let ipc = IpcEndpoint::from_str(&raw[2])?;
 let subprocess_index: u32 = raw[3]
 .parse()
 .map_err(|e| anyhow!("Invalid subprocess index '{}': {}", raw[3], e))?;
 let service_name = raw[4].clone();
 let extra = if raw.len() > 5 {
 raw[5..].to_vec()
 } else {
 Vec::new()
 };
 Ok(SubprocessArgs {
 ipc,
 subprocess_index,
 service_name,
 extra,
 })
}

// Public parsing entry point used by lib::entry().
pub fn parse_invocation() -> Result<Invocation> {
 let raw: Vec<String> = env::args().collect();
 if raw.get(1).is_some_and(|s| s == IPC_ARG) {
 let sub = parse_subprocess(&raw)?;
 return Ok(Invocation::Subprocess(sub));
 }
 // Fallback: user CLI
 let cli = Cli::parse(); // Clap handles errors & help display
 Ok(Invocation::User(cli))
}

#[derive(Parser, Debug)]
#[command(
 name = "ctoolbox",
 version,
 about = "Collective Toolbox",
 disable_help_subcommand = true
)]
pub struct Cli {
 #[arg(long)]
 pub ctoolbox_ipc_port: Option<u16>,

 #[command(subcommand)]
 pub command: Option<Command>,
}
```

# utilities/ipc.rs

```
use crate::cli::IpcEndpoint;

pub static IPC_ARG: &str = "--76c89de8-96b3-4372-ab16-cd832871fce3";

#[derive(Clone, Serialize, Deserialize)]
pub struct Channel {
 pub name: String,
 pub port: u16,
 pub authentication_key: String,
}

impl Channel {
 pub fn to_arg_string(&self) -> String {
 format!("{}:{}", self.port, self.name)
 }

 // Doesn't work
 pub fn workspace() -> Self {
 Channel {
 name: "workspace".to_string(),
 port: 0,
 authentication_key: String::new(),
 }
 }
}

pub fn channel_from_args_and_key(
 name_and_port: &IpcEndpoint,
 authentication_key: String,
) -> Channel {
 Channel {
 name: name_and_port.identity.clone(),
 port: name_and_port.port,
 authentication_key,
 }
}

pub fn channel_from_json_string(json_string: &str) -> Result<Channel> {
 serde_json::from_str(json_string)
 .map_err(|e| anyhow::anyhow!("{e}: {json_string}"))
}

```

# utilities/process.rs

```

#[derive(Clone, Serialize, Deserialize)]
pub struct Process {
 pub pid: u64,
 pub system_pid: u64,
 pub channel: Channel,
 /// PID of the workspace or renderer process that started this process.
 pub parent_pid: u64,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct ProcessManager {
 pub port: u16,
 pub ipc_control_channel: Channel,
 pub processes: HashMap<u64, Process>,
}

impl ProcessManager {
 pub fn last_index(&self) -> u64 {
 self.next_index() - 1
 }
 pub fn next_index(&self) -> u64 {
 u64::try_from(self.processes.len()).expect("usize did not fit in u64")
 }
 pub fn last_process(&self) -> Process {
 // let json = json!(self);
 // println!("Getting last process: {} {}", json, &self.last_index());
 self.processes.get(&self.last_index()).unwrap().clone()
 }
 pub fn workspace_channel(&self) -> Channel {
 self.processes.get(&0).unwrap().channel.clone()
 }
}

/// Only the workspace should call this.
pub async fn start_process(
 manager: &Arc<Mutex<ProcessManager>>,
 parent_pid: u64,
 args: Vec<String>,
) -> Process {
 let manager = manager.lock();
 assert!(manager.is_ok(), "Failed to lock process manager");
 let mut manager = manager.unwrap();
 let next_id = manager.next_index();
 let channel = Channel {
 name: format!("com.ctoolbox.workspace{next_id}"),
 port: manager.port,
 authentication_key: generate_authentication_key(),
 };

 let path =
 env::current_exe().expect("failed to get current executable path");
 let mut new_args = vec![
 IPC_ARG.to_string(),
 channel.to_arg_string(),
 next_id.to_string(),
 ];
 new_args.extend(args);
 #[allow(clippy::zombie_processes)]
 let mut _process = Command::new(path.clone())
 .args(new_args)
 .stdin(Stdio::piped())
 .stdout(Stdio::inherit())
 .stderr(Stdio::inherit())
 .spawn()
 .expect("failed to execute server process");
 let subprocess_stdin = _process.stdin.as_mut().unwrap();
 subprocess_stdin
 .write_all(channel.authentication_key.as_bytes())
 .expect("Failed to write authentication key to subprocess");
 let system_pid = u64::from(_process.id());
 send_message(
 &manager.ipc_control_channel,
 json!({"type": "add_channel", "channel": channel})
 .to_string()
 .as_str(),
 );

 let new_process = Process {
 pid: next_id,
 system_pid,
 channel,
 parent_pid,
 };

 let len = manager.processes.len();
 manager.processes.insert(
 u64::try_from(len).expect("usize did not fit in u64"),
 new_process,
 );

 manager.last_process()
}

```

# workspace/ipc.rs

```


pub static IPC_API: LazyLock<HashMap<String, Vec<&str>>> =
 LazyLock::new(|| {
 HashMap::from([
 ("formats".to_string(), vec!["convert_if_needed"]),
 (
 "io".to_string(),
 vec!["print", "poll_hid", "start_local_web_ui"],
 ),
 ("network".to_string(), vec!["get_asset", "get_url"]),
 (
 "renderer".to_string(),
 vec!["test_echo", "start", "get_frame"],
 ),
 ("storage".to_string(), vec!["set", "get"]),
 (
 "workspace".to_string(),
 vec![
 "pc_restart",
 "pc_shutdown",
 "workspace_restart",
 "workspace_shutdown",
 ],
 ),
 ])
 });

pub fn subprocess_init(args: &SubprocessArgs) -> Result<()> {
 // This is run in subprocesses only
 let service = &args.service_name;
 let authentication_key: String = read!("{}\n");
 let channel: Channel =
 channel_from_args_and_key(&args.ipc, authentication_key);

 assert!(IPC_API.contains_key(service), "Unknown service");

 log!(format!("Service started: {}", service));
 loop {
 let message = json!({ "type": "poll_for_task" });
 let response = send_message(&channel, message.to_string().as_str());

 let response_type = jq(".type", response.as_str()).unwrap();

 if response_type == "new_task" {
 let new_task_method = jq(".method", response.as_str()).unwrap();
 let new_task_args = jq(".args", response.as_str()).unwrap();
 let msgid = jq(".msgid", response.as_str())
 .unwrap()
 .parse::<u64>()
 .unwrap();
 let channel_copy =
 channel_from_json_string(json!(channel).as_str()).unwrap();
 std::thread::spawn({
 let service = service.clone();
 move || {
 dispatch_call(
 service.as_str(),
 &channel_copy,
 msgid,
 new_task_method.as_str(),
 new_task_args.as_str(),
 );
 }
 });
 } else if response_type == "no_new_tasks" {
 usleep(100000);
 } else {
 log!(
 format!("Received unexpected IPC response: {response}")
 .as_str()
 );
 usleep(100000);
 }
 }
}

pub fn send_message(channel: &Channel, message: &str) -> String {
 maybe_send_message(channel, message).expect("Failed to send IPC message")
}

pub fn maybe_send_message(
 channel: &Channel,
 message: &str,
) -> Result<String, Box<dyn Error>> {
 let message_type = jqq(".type", message).unwrap_or_default();
 if message_type != "add_channel"
 && message_type != "poll_for_task"
 && message_type != "poll_for_result"
 {
 log!(
 format!("Sending message to channel {}: {}", channel.name, message)
 .as_str()
 );
 }

 let response = ureq::post(
 format!("http://127.0.0.1:{}/ctb_ipc", channel.port).as_str(),
 )
 .header("X-CollectiveToolbox-IPCAuth", json!(channel))
 .send(message.as_bytes())?
 .body_mut()
 .read_to_string()?;
 Ok(response)
}

pub fn wait_for_message(channel: &Channel, msgid: u64) -> String {
 loop {
 let message = json!({ "type": "poll_for_result", "msgid": msgid });
 let response = send_message(channel, message.to_string().as_str());
 let response_type = jq(".type", response.as_str()).unwrap();

 if response_type == "result" {
 return jq(".content", response.as_str()).unwrap();
 } else if response_type == "pending" {
 usleep(100000);
 } else {
 log!(
 format!("Received unexpected IPC response: {response}")
 .as_str()
 );
 usleep(100000);
 }
 }
}

pub fn listen_for_message(channel: &Channel, msgid: u64) -> String {
 wait_for_message(channel, msgid)
}

pub fn call_ipc_sync(
 channel: &Channel,
 method: &str,
 args: Value,
) -> Result<String> {
 let message = json!({ "type": "call", "method": method, "args": args });
 let response = send_message(channel, &message.to_string());
 let response_type = jqq(".type", &response).unwrap_or_default();

 if response_type != "call_pending" {
 return Err(anyhow!("Unexpected IPC response: {response}"));
 }

 let msgid = jq(".msgid", &response)
 .map_err(|e| {
 anyhow!("Failed to get msgid from response: {response}, error: {e}")
 })?
 .parse::<u64>()?;

 Ok(wait_for_message(channel, msgid))
}

pub fn call_ipc(
 channel: &Channel,
 method: &str,
 args: Value,
) -> Result<String> {
 call_ipc_sync(channel, method, args)
}

```

# workspace/ipc/server.rs

```

#[derive(PartialEq, Clone, Serialize, Deserialize)]
enum CallState {
 Created,
 Claimed,
 Success,
 Error,
}

#[derive(Clone, Serialize, Deserialize)]
struct Call {
 method: String,
 args: String,
 response: Option<String>,
 state: CallState,
}

#[derive(Clone, Serialize, Deserialize)]
struct CallWithId {
 id: u64,
 call: Call,
}

struct Calls {
 calls: HashMap<u64, Call>,
 last_id: u64,
}

impl Calls {
 fn new() -> Calls {
 Calls {
 calls: HashMap::new(),
 last_id: 0,
 }
 }
 fn get(&mut self, id: &u64) -> &mut Call {
 self.calls.get_mut(id).unwrap()
 }
}

pub fn start_ipc_server(
 process_manager: Arc<Mutex<ProcessManager>>,
 port: u16,
 authentication_key: String,
) {
 // log!("Starting IPC server");

 let mut queued_calls: HashMap<String, Calls> = HashMap::new();

 let mut known_channels: Vec<Channel> = vec![Channel {
 name: "ipc_control".to_string(),
 port,
 authentication_key: authentication_key.clone(),
 }];

 let server = Server::http(format!("127.0.0.1:{port}")).unwrap();

 for request in server.incoming_requests() {
 handle_request(
 process_manager.clone(),
 request,
 &mut known_channels,
 &mut queued_calls,
 );
 }
}

fn process_request(
 process_manager: Arc<Mutex<ProcessManager>>,
 target_channel: Channel,
 content: String,
 known_channels: &mut Vec<Channel>,
 queued_calls: &mut HashMap<String, Calls>,
) -> Result<String> {
 let provided_key = target_channel.authentication_key.clone();

 let expected_key: String = known_channels
 .iter()
 .find(|c| c.name == target_channel.name)
 .map_or(String::new(), |c| c.authentication_key.clone());

 if expected_key != provided_key {
 return Err(anyhow::anyhow!("Invalid authentication key"));
 }

 let channel_calls: &mut Calls = queued_calls
 .entry(target_channel.name.clone())
 .or_insert(Calls::new());

 let message_type = jq(".type", content.as_str()).unwrap_or_default();

 let mut response = None;

 // log!(format!("Received message: {}", content).as_str());

 if message_type == "workspace_call" {
 let new_task_method = jq(".method", content.as_str()).unwrap();
 // get Serde value from jq .args
 let new_task_args = jq(".args", content.as_str()).unwrap();
 let task_args_val =
 serde_json::from_str::<serde_json::Value>(new_task_args.as_str())
 .with_context(|| "Could not parse args as JSON")?;
 ipc_call_method(
 &new_task_method,
 &task_args_val,
 Some(process_manager),
 );
 response = Some(json!({"type": "success"}).to_string());
 } else if message_type == "add_channel" {
 let channel: Channel = serde_json::from_str::<Channel>(
 jq(".channel", content.as_str())
 .with_context(|| "Could not get channel from body json")?
 .as_str(),
 )
 .with_context(|| "Could not parse channel from body json")?;
 known_channels.push(channel);
 response = Some(json!({"type": "success"}).to_string());
 } else if message_type == "remove_channel" {
 let name = jq(".channel.name", content.as_str())
 .with_context(|| "Could not get channel from json")?;
 let name = name.as_str();
 known_channels.retain(|c| c.name != name);
 response = Some(json!({"type": "success"}).to_string());
 } else if message_type == "ipc_ping" {
 response = Some(json!({"type": "ipc_ready"}).to_string());
 } else if message_type == "poll_for_result" {
 let msgid = jq(".msgid", content.as_str())
 .with_context(|| "Could not jq msgid")?;
 if msgid == "null" {
 return Err(anyhow::anyhow!("No msgid provided"));
 }
 let msgid = msgid
 .parse::<u64>()
 .with_context(|| "Could not parse msgid")?;
 let message = &channel_calls.get(&msgid);
 if message.state == CallState::Success {
 response = Some(
 json!({"type": "result", "content": message.response})
 .to_string(),
 );
 channel_calls.calls.remove(&msgid);
 } else if message.state == CallState::Error {
 response = Some(json!({"type": "error"}).to_string());
 } else {
 response = Some(json!({"type": "pending"}).to_string());
 }
 } else if message_type == "poll_for_task" {
 response = Some(json!({"type": "no_new_tasks"}));
 for (msgid, call) in &mut channel_calls.calls {
 if call.state == CallState::Created {
 call.state = CallState::Claimed;
 response = Some(
 json!({
 "type": "new_task",
 "method": call.method,
 "args": call.args,
 "msgid": msgid,
 })
 .to_string(),
 );
 break;
 }
 }
 } else if message_type == "call" {
 let method = jq(".method", content.as_str())
 .with_context(|| "Could not query method")?;
 let args = jq(".args", content.as_str())
 .with_context(|| "Could not query args")?;
 let msgid = channel_calls.last_id + 1;
 channel_calls.calls.insert(
 msgid,
 Call {
 method: method.clone(),
 args: args.clone(),
 response: None,
 state: CallState::Created,
 },
 );
 channel_calls.last_id = msgid;
 response =
 Some(json!({"type": "call_pending", "msgid": msgid}).to_string());
 } else if message_type == "response" {
 let msgid: u64 = jq(".msgid", content.as_str())
 .with_context(|| "Could not jq msgid")?
 .parse::<u64>()
 .with_context(|| "Could not parse msgid")?;

 let content = jq(".content", content.as_str())
 .with_context(|| "Could not jq content")?;
 channel_calls.get(&msgid).response = Some(content.clone());
 channel_calls.get(&msgid).state = CallState::Success;
 response = Some(json!({"type": "success"}).to_string());
 }

 response.with_context(|| format!("Invalid message type {message_type}"))
}

fn error_response(request: tiny_http::Request, message: &str) {
 match request.respond(Response::from_string(
 json!({"type": "error", "message": message}).to_string(),
 )) {
 Ok(()) => {}
 Err(e) => {
 eprintln!("Failed to send error response: {e}");
 }
 }
}

fn handle_request(
 process_manager: Arc<Mutex<ProcessManager>>,
 mut request: tiny_http::Request,
 known_channels: &mut Vec<Channel>,
 queued_calls: &mut HashMap<String, Calls>,
) {
 let target_channel = channel_from_json_string(
 request
 .headers()
 .iter()
 .find(|h| h.field.equiv("X-CollectiveToolbox-IPCAuth"))
 .map_or_else(
 || "No channel provided".to_string(),
 |h| h.value.clone().to_string(),
 )
 .as_str(),
 )
 .context("Invalid channel header");

 if target_channel.is_err() {
 error_response(request, "Invalid channel");
 return;
 }

 let mut content = String::new();
 if request
 .as_reader()
 .read_to_string(&mut content)
 .context("Could not read request body")
 .is_err()
 {
 error_response(request, "Could not read request body");
 return;
 }

 let response = process_request(
 process_manager,
 target_channel.unwrap(),
 content,
 known_channels,
 queued_calls,
 );

 let response = match response {
 Ok(r) => r,
 Err(e) => {
 json!({"type": "error", "message": e.to_string()}).to_string()
 }
 };

 request.respond(Response::from_string(response)).unwrap();
}

```

# workspace/ipc/dispatch.rs

```

pub fn dispatch_call(
 service_name: &str,
 to_channel: &Channel,
 msgid: u64,
 method: &str,
 args: &str,
) {
 if IPC_API.contains_key(service_name) {
 let allowed_methods = IPC_API[service_name].clone();
 if allowed_methods.contains(&method) {
 // Parse args as serde_json::Value
 let args_value: Value = match serde_json::from_str(args) {
 Ok(val) => val,
 Err(e) => {
 let response = json!({
 "type": "response",
 "msgid": msgid,
 "error": format!("Invalid args: {}", e),
 });
 send_message(to_channel, &response.to_string());
 return;
 }
 };

 let result = ipc_call_method(method, &args_value, None);

 let response = json!({
 "type": "response",
 "msgid": msgid,
 "content": result,
 });
 send_message(to_channel, &response.to_string());

 return;
 }
 }
 // Use anyhow for error handling, avoid panics
 let response = json!({
 "type": "response",
 "msgid": msgid,
 "error": format!("Method {method} not allowed or not implemented for {service_name}"),
 });
 send_message(to_channel, &response.to_string());
}

/// Dispatches an IPC method call with structured arguments.
pub fn ipc_call_method(
 method: &str,
 args: &Value,
 process_manager: Option<Arc<Mutex<ProcessManager>>>,
) -> Value {
 match method {
 // IO
 "start_local_web_ui" => {
 json_value!({ "value" => crate::io::webui::start_webui_server() })
 }

 // Database operations
 "get_str_u64" => {
 let db = args.get("db").and_then(Value::as_str).unwrap_or_default();
 let key =
 args.get("key").and_then(Value::as_str).unwrap_or_default();
 if let Some(val) = db::get_str_u64(db, key) {
 json_value!({ "value" => val })
 } else {
 json_value!({ "none" => true })
 }
 }
 "put_str_u64" => {
 let db = args.get("db").and_then(Value::as_str).unwrap_or_default();
 let key =
 args.get("key").and_then(Value::as_str).unwrap_or_default();
 let value = args.get("value").and_then(Value::as_u64).unwrap_or(0);
 match db::put_str_u64(db, key, value) {
 Ok(()) => json_value!({ "value" => "ok" }),
 Err(e) => json_value!({ "error" => e.to_string() }),
 }
 }

 // (snip many variants of database operations for different types)

 // Renderer
 "test_echo" => {
 json_value!({ "value" => renderer::test_echo(args.to_string().as_str()) })
 }
 // Workspace calls are handled by the server, not the usual IPC path
 "workspace_restart" => {
 if let Some(pm) = process_manager {
 crate::workspace::workspace_restart(&pm);
 json_value!({ "value" => "" })
 } else {
 json_value!({ "error" => "process manager required" })
 }
 }
 "workspace_shutdown" => {
 if let Some(pm) = process_manager {
 crate::workspace::workspace_shutdown(&pm);
 json_value!({ "value" => "" })
 } else {
 json_value!({ "error" => "process manager required" })
 }
 }
 &_ => {
 json_value!({ "error" => format!("Method {method} not allowed or not implemented") })
 }
 }
}

```

# workspace/ipc/dispatch/db.rs

```
// (snip helper code)

/// Get a u64 value by string key.
pub fn get_str_u64(db: &str, key: &str) -> Option<u64> {
 let conn: TableConnection<&str, u64> = TableConnection::open(db).ok()?;
 conn.get_u64(key)
}

/// Put a u64 value by string key.
pub fn put_str_u64(db: &str, key: &str, value: u64) -> Result<()> {
 let conn: TableConnection<&str, u64> = TableConnection::open(db)?;
 conn.put(key, value)
}

/// Delete a u64 value by string key.
pub fn delete_str_u64(db: &str, key: &str) -> Result<()> {
 let conn: TableConnection<&str, u64> = TableConnection::open(db)?;
 conn.delete(key)
}

// (snip many more lines of this)
```


