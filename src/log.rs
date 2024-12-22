use spdlog::formatter::{pattern, PatternFormatter};

pub fn use_formatter() {
	let pattern = pattern!(
		"[{^{level}}] {payload} - \u{001B}[1m\u{001B}[33m{source}\u{001B}[39m\u{001B}[22m{eol}"
	);
	let formatter = Box::new(PatternFormatter::new(pattern));

	let logger = spdlog::default_logger();
	for sink in logger.sinks() {
		sink.set_formatter(formatter.clone());
	}

	logger.set_level_filter(spdlog::LevelFilter::All);

	assert_eq!(spdlog::init_env_level().unwrap(), false);
}
