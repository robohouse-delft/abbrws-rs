use structopt::StructOpt;

type Client = abbrws::Client;

#[derive(StructOpt)]
#[structopt(setting(structopt::clap::AppSettings::DeriveDisplayOrder))]
#[structopt(setting(structopt::clap::AppSettings::ColoredHelp))]
#[structopt(setting(structopt::clap::AppSettings::UnifiedHelpMessage))]
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
	set: Option<bool>
}

#[tokio::main]
async fn main() {
	if let Err(e) = do_main(&Options::from_args()).await {
		eprintln!("{}", e);
		std::process::exit(1);
	}
}

async fn do_main(options: &Options) -> Result<(), String> {
	let mut client = abbrws::Client::new_default(&options.host, &options.user, &options.password)
		.map_err(|e| format!("Failed to connect to {:?}: {}", options.host, e))?;

	if options.list {
		list_signals(&mut client).await
	} else if let Some(signal) = &options.signal {
		if let Some(value) = options.set {
			set_signal(&mut client, signal, abbrws::SignalValue::Binary(value)).await
		} else {
			show_signal(&mut client, signal).await
		}
	} else {
		Err(String::from("No --list and no --signal specified."))
	}
}

async fn list_signals(client: &mut Client) -> Result<(), String> {
	let signals = client.get_signals().await
		.map_err(|e| format!("Failed to retrieve signals: {}", e))?;

	let title_width = signals.iter().map(|x| x.title.len()).max().unwrap_or(0);

	for signal in signals {
		println!("{title:<title_width$} : {kind:<14} = {value}",
			title       = signal.title,
			title_width = title_width,
			kind        = signal.kind,
			value       = signal.lvalue
		);
	}
	Ok(())
}

async fn show_signal(client: &mut Client, signal: &str) -> Result<(),  String> {
	let signal = client.get_signal(signal).await
		.map_err(|e| format!("Failed to retrieve signal {:?}: {}", signal, e))?;
		println!("{title} : {kind} = {value}",
			title = signal.title,
			kind  = signal.kind,
			value = signal.lvalue
		);
	Ok(())

}

async fn set_signal(client: &mut Client, signal: &str, value: abbrws::SignalValue) -> Result<(), String> {
	client.set_signal(signal, value).await
		.map_err(|e| format!("Failed to set signal {:?} to {}: {}", signal, value, e))?;
	Ok(())
}
