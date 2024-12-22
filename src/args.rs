use clap::Parser;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct Args {
	/// The root directory to use
	#[arg(short, long, default_value = ".")]
	pub root: String,

	/// The input file to read from
	#[arg(short, long, default_value = "main.lua")]
	pub input: String,

	/// The output file to write to
	#[arg(short, long, default_value = "bundled.lua")]
	pub output: String,

	/// Whether to minify the output
	#[arg(short, long)]
	pub minify: bool,

	/// Whether to beautify the output
	#[arg(short, long)]
	pub beautify: bool,
}

impl Args {
	pub fn parse() -> Self {
		let mut args = <Self as clap::Parser>::parse();
		args.input = format!("{}/{}", args.root, args.input);
		args.output = format!("{}/{}", args.root, args.output);
		args
	}
}
