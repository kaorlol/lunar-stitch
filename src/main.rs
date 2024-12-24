#[macro_use]
pub mod macros;

use std::time::Instant;

use darklua_core::{Configuration, GeneratorParameters, Options, Resources};

mod collector;
use collector::AcquireParser;

mod log;
use full_moon::visitors::VisitorMut as _;
use spdlog::{debug, info};

mod args;
use args::Args;

fn main() -> Result<(), Box<dyn std::error::Error>> {
	log::use_formatter();

	let args = Args::parse();
	if args.input == args.output {
		return Err("Input and output cannot be the same".into());
	};

	debug!("Root: {}", args.root);
	debug!("Input: {}", args.input);
	debug!("Output: {}", args.output);

	if args.beautify && args.minify {
		return Err("Cannot beautify and minify simultaneously".into());
	}

	let Ok(input) = std::fs::read_to_string(&args.input) else {
		return Err("Failed to read input file".into());
	};

	info!("Parsing main.lua");
	let time = Instant::now();
	let ast = full_moon::parse(&input).unwrap();
	let mut parser = AcquireParser::new(args.root, args.input, args.output);
	let bundled_ast = parser.visit_ast(ast);

	info!(
		"Took {} seconds to bundle {} unique files and {} acquire calls",
		time.elapsed().as_secs_f64(),
		parser.processed_cache.len(),
		parser.count
	);

	info!("Writing to {}", parser.output);
	std::fs::write(&parser.output, bundled_ast.to_string()).unwrap();

	let resources = Resources::from_file_system();
	let generator_parameters = match (args.minify, args.beautify) {
		(true, _) => GeneratorParameters::default_dense(),
		(_, true) => GeneratorParameters::default_readable(),
		_ => GeneratorParameters::default(),
	};

	let configuration = Configuration::empty().with_generator(generator_parameters);
	let process_options = Options::new(&parser.output)
		.with_output(parser.output)
		.with_configuration(configuration);

	info!("Processing output");
	darklua_core::process(&resources, process_options);

	Ok(())
}
