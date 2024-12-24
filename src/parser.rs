use full_moon::{ast::*, node::Node as _, tokenizer::*, visitors::VisitorMut};
use rustc_hash::{FxHashMap, FxHashSet};
use spdlog::info;

/// Used to parse the acquire calls in the Lua code
pub struct AcquireParser {
	pub root: String,
	pub input: String,
	pub output: String,
	pub processed_cache: FxHashMap<String, Ast>,
	latest_call_range: std::ops::Range<usize>,
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
			latest_call_range: 0..0,
			semi_colons: FxHashSet::default(),
			count: 0,
		}
	}

	/// Parses a [`Prefix`] to check if it contains an "acquire" [`identifier`](TokenType::Identifier)
	fn contains_acquire(&self, prefix: &Prefix) -> bool {
		prefix.tokens().any(|token| {
			matches!(
				token.token_type(),
				TokenType::Identifier { identifier } if identifier == &"acquire".into()
			)
		})
	}

	/// Parses a [`FunctionCall`] to grab the path of the file to acquire
	fn call_acquire_path(&self, call: &FunctionCall) -> Option<String> {
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

	/// Parses a [`VarExpression`] to grab the path of the file to acquire
	fn var_acquire_path(&self, var: &VarExpression) -> Option<String> {
		var.suffixes().find_map(|suffix| {
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
		let range = get_range!(call);

		if self.contains_acquire(call.prefix()) {
			let path = match self.call_acquire_path(&call) {
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

				let needs_semi = !self.latest_call_range.contains(&range.start);
				let suffixes = get_suffixes!(call, self.semi_colons, needs_semi);
				call = make_call!(FunctionCall, path, ast, suffixes, needs_semi);
				self.count += 1;
			}
		} else {
			self.latest_call_range = range;
		}

		call
	}

	fn visit_var_expression(&mut self, mut var_expr: VarExpression) -> VarExpression {
		let range = get_range!(var_expr);
		if self.contains_acquire(var_expr.prefix()) {
			let path = match self.var_acquire_path(&var_expr) {
				Some(p) if p != self.input && p != self.output => p,
				Some(_) => return var_expr,
				None => panic!("Invalid acquire path"),
			};

			// TODO: Add implicit panic behavior
			if std::fs::exists(&path).unwrap() {
				let ast = self.processed_cache.entry(path.clone()).or_insert_with(|| {
					info!("Parsing {path}");
					full_moon::parse(&std::fs::read_to_string(&path).unwrap())
						.unwrap_or_else(|_| panic!("Failed to parse {path}"))
				});

				let needs_semi = !self.latest_call_range.contains(&range.start);
				let suffixes = get_suffixes!(var_expr, self.semi_colons, needs_semi);
				var_expr = make_call!(VarExpression, path, ast, suffixes, needs_semi);
				self.count += 1;
			}
		} else {
			self.latest_call_range = range;
		}

		var_expr
	}
}

/// Used to store the leading and trailing trivia of a [`Token`]
struct Trivia {
	leading: Vec<Token>,
	trailing: Vec<Token>,
}

/// Processes the [`tokens`](TokenReference) to add semicolons where needed\
/// Returns the [`Trivia`] of the [`TokenReference`](TokenReference) in a closure
fn process_tokens<F>(
	tokens: Vec<TokenReference>, semicolons: &mut FxHashSet<usize>, needs_semi: bool,
	mut handler: F,
) where
	F: FnMut(Trivia),
{
	for token in tokens.iter() {
		if let TokenType::Symbol { symbol } = token.token_type() {
			if *symbol == Symbol::RightParen || *symbol == Symbol::RightBracket {
				if let Some(trailing) = add_semicolon_if_needed!(token, semicolons, needs_semi) {
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

	macro_rules! acquire_parser {
		() => {
			AcquireParser::new("./test".into(), "main.lua".into(), "bundled.lua".into())
		};
	}

	#[test]
	fn test_contains_acquire() {
		let parser = acquire_parser!();

		let lua_code = "acquire('test_file.lua')";
		let ast = full_moon::parse(lua_code).unwrap();
		match ast.clone().nodes().stmts().next().unwrap() {
			full_moon::ast::Stmt::FunctionCall(call) => {
				assert!(parser.contains_acquire(call.prefix()));
			}
			_ => panic!("Expected function call"),
		}
	}

	#[test]
	fn test_grab_acquire_path() {
		let parser = acquire_parser!();

		let lua_code = "acquire('test_file.lua')";
		let ast = full_moon::parse(lua_code).unwrap();
		match ast.clone().nodes().stmts().next().unwrap() {
			full_moon::ast::Stmt::FunctionCall(call) => {
				let path = parser.call_acquire_path(&call);
				assert_eq!(path, Some("./test/test_file.lua".into()));
			}
			_ => panic!("Expected function call"),
		}
	}

	#[test]
	fn test_acquire_with_invalid_path() {
		let mut parser = acquire_parser!();

		let lua_code = "acquire('invalid_file.lua')";
		let ast = full_moon::parse(lua_code).unwrap();
		match ast.clone().nodes().stmts().next().unwrap() {
			full_moon::ast::Stmt::FunctionCall(call) => {
				parser.visit_function_call(call.clone());
				assert_eq!(parser.count, 0);
			}
			_ => panic!("Expected function call"),
		}
	}

	#[test]
	fn test_semicolon_tracking() {
		let mut parser = acquire_parser!();

		let lua_code = "local x = 10;\nlocal ya = 20;\n";
		let ast = full_moon::parse(lua_code).unwrap();

		parser.visit_block(ast.nodes().clone());

		let expected_semicolons: FxHashSet<_> = [14, 13].into_iter().collect();
		assert_eq!(parser.semi_colons, expected_semicolons);
	}

	#[test]
	fn test_process_cached_ast() {
		let mut parser = acquire_parser!();
		let test_path = "./test/test_file.lua";

		fs::write(test_path, "local x = 42").unwrap();

		let lua_code = "acquire('test_file.lua')";
		let ast = full_moon::parse(lua_code).unwrap();
		match ast.clone().nodes().stmts().next().unwrap() {
			full_moon::ast::Stmt::FunctionCall(call) => {
				parser.visit_function_call(call.clone());
				assert!(parser.processed_cache.contains_key(test_path));
			}
			_ => panic!("Expected function call"),
		}

		fs::remove_file(test_path).unwrap();
	}
}
