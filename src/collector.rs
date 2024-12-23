use full_moon::{
	ast::{
		span::ContainedSpan, Ast, Block, Call, FunctionArgs, FunctionCall, Index, Prefix, Suffix,
	},
	node::Node as _,
	tokenizer::{Symbol, Token, TokenReference, TokenType},
	visitors::VisitorMut,
};
use rustc_hash::{FxHashMap, FxHashSet};
use spdlog::info;

/// Used to parse the acquire calls in the Lua code
pub struct AcquireParser {
	pub root: String,
	pub input: String,
	pub output: String,
	pub processed_cache: FxHashMap<String, Ast>,
	semi_colons: FxHashSet<usize>,
	pub count: usize,
}

impl Default for AcquireParser {
	/// Uses the default values:
	/// - root: "."
	/// - input: "main.lua"
	/// - output: "bundled.lua"
	fn default() -> Self {
		Self::new(".".into(), "main.lua".into(), "bundled.lua".into())
	}
}

impl AcquireParser {
	pub fn new(root: String, input: String, output: String) -> Self {
		Self {
			root,
			input,
			output,
			processed_cache: FxHashMap::default(),
			semi_colons: FxHashSet::default(),
			count: 0,
		}
	}

	/// Parses a `ast::Prefix` to check if it contains an `acquire` identifier
	fn contains_acquire(&self, prefix: &Prefix) -> bool {
		prefix.tokens().any(|token| {
			matches!(
				token.token_type(),
				TokenType::Identifier { identifier } if identifier == &"acquire".into()
			)
		})
	}

	/// Parses a `ast::FunctionCall` to grab the path of the file to acquire
	fn grab_acquire_path(&self, call: &FunctionCall) -> Option<String> {
		call.suffixes().find_map(|suffix| {
			let Suffix::Call(call) = suffix else {
				return None;
			};

			call.tokens().find_map(|token| {
				if let TokenType::StringLiteral { literal, .. } = token.token_type() {
					return Some(format!("{}/{}", self.root, literal.to_string()));
				}
				None
			})
		})
	}
}

impl VisitorMut for AcquireParser {
	fn visit_block(&mut self, block: Block) -> Block {
		block.stmts_with_semicolon().for_each(|stmt| {
			stmt.tokens().for_each(|token| {
				if matches!(token.token_type(), TokenType::Symbol { symbol } if *symbol == Symbol::Semicolon)
				{
					if let Some(start_pos) = token.start_position() {
						self.semi_colons.insert(start_pos.character());
					}
				}
			});
		});
		block
	}

	fn visit_function_call(&mut self, mut call: FunctionCall) -> FunctionCall {
		if self.contains_acquire(call.prefix()) {
			let path = match self.grab_acquire_path(&call) {
				Some(p) if p != self.input && p != self.output => p,
				Some(_) => return call,
				None => panic!("Invalid acquire path"),
			};

			// TODO: Add implicit panic behavior
			if std::fs::exists(&path).unwrap() {
				let ast = self.processed_cache.entry(path.clone()).or_insert_with(|| {
					info!("Parsing {path}");
					full_moon::parse(&std::fs::read_to_string(&path).unwrap())
						.unwrap_or_else(|_| panic!("Failed to parse {path}"))
				});

				let mut suffixes: Vec<Suffix> = call.suffixes().cloned().collect();
				if let Some(last_suffix) = suffixes.last_mut() {
					match last_suffix {
						Suffix::Call(call) => {
							let tokens: Vec<TokenReference> = call.tokens().cloned().collect();
							process_tokens(tokens, &mut self.semi_colons, |trivia| {
								if let Call::AnonymousCall(args) = call {
									match args {
										FunctionArgs::Parentheses { arguments, .. } => {
											*args = FunctionArgs::Parentheses {
												parentheses: ContainedSpan::new(
													TokenReference::symbol("(").unwrap(),
													TokenReference::new(
														trivia.leading,
														Token::new(TokenType::Symbol {
															symbol: Symbol::RightParen,
														}),
														trivia.trailing,
													),
												),
												arguments: arguments.clone(),
											};
										}
										_ => todo!("New function args type"),
									}
								}
							});
						}

						// Doesn't work atm
						Suffix::Index(index) => {
							let tokens = index.tokens().cloned().collect::<Vec<_>>();
							process_tokens(tokens, &mut self.semi_colons, |trivia| match index {
								Index::Dot { name, .. } => {
									*index = Index::Dot {
										dot: TokenReference::symbol(".").unwrap(),
										name: TokenReference::new(
											trivia.leading,
											name.token().clone(),
											trivia.trailing,
										),
									};
								}
								Index::Brackets { expression, .. } => {
									*index = Index::Brackets {
										brackets: ContainedSpan::new(
											TokenReference::symbol("[").unwrap(),
											TokenReference::new(
												trivia.leading,
												Token::new(TokenType::Symbol {
													symbol: Symbol::RightBracket,
												}),
												trivia.trailing,
											),
										),
										expression: expression.clone(),
									};
								}
								_ => todo!("New index type"),
							});
						}
						_ => (),
					}
				}
				suffixes.remove(0);

				call = make_function_call!(path, ast, suffixes);
				self.count += 1;
			}
		}

		call
	}
}

/// Used to store the leading and trailing trivia of a token
struct Trivia {
	leading: Vec<Token>,
	trailing: Vec<Token>,
}

/// Processes the tokens to add semicolons where needed\
/// Returns the `Trivia` of the token in a closure
fn process_tokens<F>(tokens: Vec<TokenReference>, semicolons: &mut FxHashSet<usize>, mut handler: F)
where
	F: FnMut(Trivia),
{
	for token in tokens.iter() {
		if let TokenType::Symbol { symbol } = token.token_type() {
			if *symbol == Symbol::RightParen || *symbol == Symbol::RightBracket {
				if let Some(trailing) = add_semicolon_if_needed!(token, semicolons) {
					handler(Trivia {
						leading: token.leading_trivia().cloned().collect(),
						trailing,
					});
				}
			}
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use std::fs;

<<<<<<< HEAD
	macro_rules! acquire_parser {
		() => {
			AcquireParser::new("./test".into(), "main.lua".into(), "bundled.lua".into())
=======
	macro_rules! acquire_collector {
		() => {
			AcquireCollector::new("./test".into(), "main.lua".into(), "bundled.lua".into())
>>>>>>> c4b8fee7b94c9debbd29a776a6d007f879b26b82
		};
	}

	#[test]
	fn test_contains_acquire() {
<<<<<<< HEAD
		let parser = acquire_parser!();
=======
		let collector = acquire_collector!();
>>>>>>> c4b8fee7b94c9debbd29a776a6d007f879b26b82

		let lua_code = "acquire('test_file.lua')";
		let ast = full_moon::parse(lua_code).unwrap();
		match ast.clone().nodes().stmts().next().unwrap() {
			full_moon::ast::Stmt::FunctionCall(call) => {
<<<<<<< HEAD
				assert!(parser.contains_acquire(call.prefix()));
=======
				assert!(collector.contains_acquire(call.prefix()));
>>>>>>> c4b8fee7b94c9debbd29a776a6d007f879b26b82
			}
			_ => panic!("Expected function call"),
		}
	}

	#[test]
	fn test_grab_acquire_path() {
<<<<<<< HEAD
		let parser = acquire_parser!();
=======
		let collector = acquire_collector!();
>>>>>>> c4b8fee7b94c9debbd29a776a6d007f879b26b82

		let lua_code = "acquire('test_file.lua')";
		let ast = full_moon::parse(lua_code).unwrap();
		match ast.clone().nodes().stmts().next().unwrap() {
			full_moon::ast::Stmt::FunctionCall(call) => {
<<<<<<< HEAD
				let path = parser.grab_acquire_path(&call);
=======
				let path = collector.grab_acquire_path(&call);
>>>>>>> c4b8fee7b94c9debbd29a776a6d007f879b26b82
				assert_eq!(path, Some("./test/test_file.lua".into()));
			}
			_ => panic!("Expected function call"),
		}
	}

	#[test]
	fn test_acquire_with_invalid_path() {
<<<<<<< HEAD
		let mut parser = acquire_parser!();
=======
		let mut collector = acquire_collector!();
>>>>>>> c4b8fee7b94c9debbd29a776a6d007f879b26b82

		let lua_code = "acquire('invalid_file.lua')";
		let ast = full_moon::parse(lua_code).unwrap();
		match ast.clone().nodes().stmts().next().unwrap() {
			full_moon::ast::Stmt::FunctionCall(call) => {
<<<<<<< HEAD
				parser.visit_function_call(call.clone());
				assert_eq!(parser.count, 0);
=======
				collector.visit_function_call(call.clone());
				assert_eq!(collector.count, 0);
>>>>>>> c4b8fee7b94c9debbd29a776a6d007f879b26b82
			}
			_ => panic!("Expected function call"),
		}
	}

	#[test]
	fn test_semicolon_tracking() {
<<<<<<< HEAD
		let mut parser = acquire_parser!();
=======
		let mut collector = acquire_collector!();
>>>>>>> c4b8fee7b94c9debbd29a776a6d007f879b26b82

		let lua_code = "local x = 10;\nlocal ya = 20;\n";
		let ast = full_moon::parse(lua_code).unwrap();

<<<<<<< HEAD
		parser.visit_block(ast.nodes().clone());

		let expected_semicolons: FxHashSet<_> = [14, 13].into_iter().collect();
		assert_eq!(parser.semi_colons, expected_semicolons);
=======
		collector.visit_block(ast.nodes().clone());

		let expected_semicolons: FxHashSet<_> = [14, 13].into_iter().collect();
		assert_eq!(collector.semi_colons, expected_semicolons);
>>>>>>> c4b8fee7b94c9debbd29a776a6d007f879b26b82
	}

	#[test]
	fn test_process_cached_ast() {
<<<<<<< HEAD
		let mut parser = acquire_parser!();
=======
		let mut collector = acquire_collector!();
>>>>>>> c4b8fee7b94c9debbd29a776a6d007f879b26b82
		let test_path = "./test/test_file.lua";

		fs::write(test_path, "local x = 42").unwrap();

		let lua_code = "acquire('test_file.lua')";
		let ast = full_moon::parse(lua_code).unwrap();
		match ast.clone().nodes().stmts().next().unwrap() {
			full_moon::ast::Stmt::FunctionCall(call) => {
<<<<<<< HEAD
				parser.visit_function_call(call.clone());
				assert!(parser.processed_cache.contains_key(test_path));
=======
				collector.visit_function_call(call.clone());
				assert!(collector.processed_cache.contains_key(test_path));
>>>>>>> c4b8fee7b94c9debbd29a776a6d007f879b26b82
			}
			_ => panic!("Expected function call"),
		}

		fs::remove_file(test_path).unwrap();
	}

	#[test]
	fn test_function_call_suffix_modification() {
<<<<<<< HEAD
		let mut parser = acquire_parser!();
=======
		let mut collector = acquire_collector!();
>>>>>>> c4b8fee7b94c9debbd29a776a6d007f879b26b82

		let lua_code = "acquire('test_file.lua').do_something()";
		let ast = full_moon::parse(lua_code).unwrap();
		match ast.clone().nodes().stmts().next().unwrap() {
			full_moon::ast::Stmt::FunctionCall(call) => {
<<<<<<< HEAD
				parser.visit_function_call(call.clone());
				assert!(parser.count > 0);
=======
				collector.visit_function_call(call.clone());
				assert!(collector.count > 0);
>>>>>>> c4b8fee7b94c9debbd29a776a6d007f879b26b82
			}
			_ => panic!("Expected function call"),
		}
	}
}
