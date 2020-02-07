use structopt::StructOpt;
use structopt::clap::AppSettings;
use structopt::clap::ArgGroup;
use yansi::Paint;

#[derive(StructOpt)]
#[structopt(setting(AppSettings::DeriveDisplayOrder))]
#[structopt(setting(AppSettings::ColoredHelp))]
#[structopt(setting(AppSettings::UnifiedHelpMessage))]
#[structopt(group(ArgGroup::with_name("command").required(true)))]
struct Options {
	/// The host to connect to.
	#[structopt(long, short)]
	host: String,

	/// The user to authenticate as.
	#[structopt(long, short)]
	#[structopt(global = true)]
	#[structopt(default_value = "Default User")]
	user: String,

	/// The password for the user.
	#[structopt(long, short)]
	#[structopt(global = true)]
	#[structopt(default_value = "robotics")]
	password: String,

	/// List the contents of a directory.
	#[structopt(long)]
	#[structopt(group = "command")]
	list: Option<String>,

	/// Create a directory.
	#[structopt(long)]
	#[structopt(group = "command")]
	create_dir: Option<String>,

	/// Download a file.
	#[structopt(long)]
	#[structopt(value_names = &["SOURCE", "DEST"])]
	#[structopt(group = "command")]
	download: Option<Vec<String>>,

	/// Upload a file.
	#[structopt(long)]
	#[structopt(value_names = &["SOURCE", "DEST"])]
	#[structopt(group = "command")]
	#[structopt(requires = "content-type")]
	upload: Option<Vec<String>>,

	/// The content-type of the uploaded file.
	#[structopt(long)]
	#[structopt(value_name = "MIME")]
	#[structopt(requires = "upload")]
	content_type: Option<abbrws::Mime>,
}

#[tokio::main]
async fn main() {
	if let Err(e) = do_main(&Options::from_args()).await {
		eprintln!("{} {}", Paint::red("Error:").bold(), e);
		std::process::exit(1);
	}
}

async fn do_main(options: &Options) -> Result<(), String> {
	if !should_color() {
		Paint::disable();
	}

	let connect = || abbrws::Client::new(&options.host, &options.user, &options.password)
		.map_err(|e| format!("failed to connect to {:?}: {}", options.host, e));

	eprintln!("user: {}", options.user);

	if let Some(directory) = &options.list {
		let mut client = connect()?;
		let entries = client.list_files(&directory).await.map_err(|e| format!("failed to retrieve directory contents: {}", e))?;
		println!("{:#?}", entries);
	} else if let Some(directory) = &options.create_dir {
		let mut client = connect()?;
		client.create_directory(directory).await.map_err(|e| format!("failed to create directory: {}", e))?;
	} else if let Some(paths) = &options.download {
		let source = &paths[0];
		let destination = &paths[1];
		let mut client = connect()?;
		let (content_type, data) = client.download_file(source).await.map_err(|e| format!("failed to download file: {}", e))?;
		eprintln!("Content-Type: {}", content_type);
		write_file(destination, data)?;
	} else if let Some(paths) = &options.upload {
		let source = &paths[0];
		let destination = &paths[1];
		let data = read_file(source)?;
		let mut client = connect()?;
		client.upload_file(destination, options.content_type.clone().unwrap(), data).await.map_err(|e| format!("failed to upload file: {}", e))?;
	}

	Ok(())
}

fn read_file(path: impl AsRef<std::path::Path>) -> Result<Vec<u8>, String> {
	let path = path.as_ref();
	std::fs::read(path).map_err(|e| format!("failed to read from file {:?}: {}", path, e))
}

fn write_file(path: impl AsRef<std::path::Path>, data: impl AsRef<[u8]>) -> Result<(), String> {
	let path = path.as_ref();
	std::fs::write(path, data).map_err(|e| format!("failed to write to file {:?}: {}", path, e))
}

extern "C" {
	fn isatty(fd: std::os::raw::c_int) -> std::os::raw::c_int;
}

fn stdout_is_tty() -> bool {
	unsafe { isatty(1) != 0 }
}

fn should_color() -> bool {
	// CLICOLOR not set? Check if stdout is a TTY.
	let clicolor = match std::env::var_os("CLICOLOR") {
		Some(x) => x,
		None => return stdout_is_tty(),
	};

	// CLICOLOR not ascii? Disable colors.
	let clicolor = match clicolor.to_str() {
		Some(x) => x,
		None => return false,
	};

	if clicolor.eq_ignore_ascii_case("auto") {
		stdout_is_tty()
	} else {
		let force = false;
		let force = force || clicolor.eq_ignore_ascii_case("yes");
		let force = force || clicolor.eq_ignore_ascii_case("true");
		let force = force || clicolor.eq_ignore_ascii_case("always");
		let force = force || clicolor.eq_ignore_ascii_case("1");
		force
	}
}
