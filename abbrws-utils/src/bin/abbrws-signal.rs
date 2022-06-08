use structopt::StructOpt;
use yansi::Paint;

#[derive(StructOpt)]
#[structopt(setting(structopt::clap::AppSettings::DeriveDisplayOrder))]
#[structopt(setting(structopt::clap::AppSettings::ColoredHelp))]
#[structopt(setting(structopt::clap::AppSettings::UnifiedHelpMessage))]
#[structopt(group(structopt::clap::ArgGroup::with_name("selection").required(true)))]
struct Options {
	/// The host to connect to.
	#[structopt(long, short)]
	host: String,

	/// The user to authenticate as.
	#[structopt(long, short)]
	#[structopt(default_value = "Default User")]
	user: String,

	/// The password for the user.
	#[structopt(long, short)]
	#[structopt(default_value = "robotics")]
	password: String,

	/// List all available signals.
	#[structopt(long)]
	#[structopt(group = "selection")]
	list: bool,

	/// Show or set one signal.
	#[structopt(long)]
	#[structopt(group = "selection")]
	signal: Option<String>,

	/// Set the value of a signal.
	#[structopt(long)]
	#[structopt(requires = "signal")]
	set: Option<abbrws::SignalValue>
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

	let mut client = abbrws::Client::new(&options.host, &options.user, &options.password)
		.map_err(|e| format!("failed to connect to {:?}: {}", options.host, e))?;

	if options.list {
		list_signals(&mut client).await
	} else if let Some(signal) = &options.signal {
		if let Some(value) = options.set {
			set_signal(&mut client, signal, value).await?;
			show_signal(&mut client, signal).await
		} else {
			show_signal(&mut client, signal).await
		}
	} else {
		Err(String::from("no --list and no --signal specified."))
	}
}

async fn list_signals(client: &mut abbrws::Client) -> Result<(), String> {
	let signals = client.get_signals().await
		.map_err(|e| format!("failed to retrieve signals: {}", e))?;

	let title_width = signals.iter().map(|x| x.title.len()).max().unwrap_or(0);

	for signal in signals {
		println!("{title:<title_width$} = {value:<10} ({kind})",
			title       = Paint::blue(signal.title),
			title_width = title_width,
			kind        = Paint::magenta(signal.kind),
			value       = Paint::yellow(format!("{}", signal.lvalue)),
		);
	}
	Ok(())
}

async fn show_signal(client: &mut abbrws::Client, signal: &str) -> Result<(),  String> {
	let signal = client.get_signal(signal).await
		.map_err(|e| format!("failed to retrieve signal {:?}: {}", signal, e))?;
		println!("{title} = {value} ({kind})",
			title = Paint::blue(signal.title),
			kind  = Paint::magenta(signal.kind),
			value = Paint::yellow(signal.lvalue),
		);
	Ok(())

}

async fn set_signal(client: &mut abbrws::Client, signal: &str, value: abbrws::SignalValue) -> Result<(), String> {
	client.set_signal(signal, value).await
		.map_err(|e| format!("failed to set signal {:?} to {}: {}", signal, value, e))?;
	Ok(())
}

extern "C" {
	fn isatty(fd: std::os::raw::c_int) -> std::os::raw::c_int;
}

fn stdout_is_tty() -> bool {
	unsafe { isatty(1) != 0 }
}

#[allow(clippy::let_and_return)]
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
