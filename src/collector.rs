use full_moon::{
	ast,
	node::Node as _,
	tokenizer::{self, Token, TokenReference, TokenType},
	visitors::VisitorMut,
};
use rustc_hash::{FxHashMap, FxHashSet};
use spdlog::info;

pub struct AcquireCollector {
	pub root: String,
	pub input: String,
	pub output: String,
	pub processed_cache: FxHashMap<String, ast::Ast>,
	semi_colons: FxHashSet<usize>,
	pub count: usize,
}

impl Default for AcquireCollector {
	fn default() -> Self {
		Self::new(".".into(), "main.lua".into(), "bundled.lua".into())
	}
}

impl AcquireCollector {
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

	pub fn contains_acquire(&self, prefix: &ast::Prefix) -> bool {
		prefix.tokens().any(|token| {
			matches!(
				token.token_type(),
				TokenType::Identifier { identifier } if identifier == &"acquire".into()
			)
		})
	}

	pub fn grab_acquire_path(&self, call: &ast::FunctionCall) -> Option<String> {
		call.suffixes().find_map(|suffix| {
			let ast::Suffix::Call(call) = suffix else {
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

impl VisitorMut for AcquireCollector {
	fn visit_block(&mut self, block: ast::Block) -> ast::Block {
		block.stmts_with_semicolon().for_each(|stmt| {
			stmt.tokens().for_each(|token| {
				if matches!(token.token_type(), TokenType::Symbol { symbol } if *symbol == tokenizer::Symbol::Semicolon) {
					if let Some(start_pos) = token.start_position() {
						self.semi_colons.insert(start_pos.character());
					}
				}
			});
		});
		block
	}

	fn visit_function_call(&mut self, mut call: ast::FunctionCall) -> ast::FunctionCall {
		if self.contains_acquire(call.prefix()) {
			let path = match self.grab_acquire_path(&call) {
				Some(p) if p != self.input && p != self.output => p,
				_ => panic!("Invalid acquire path"),
			};

			if std::fs::exists(&path).unwrap() {
				let ast = self.processed_cache.entry(path.clone()).or_insert_with(|| {
					info!("Parsing {path}");
					full_moon::parse(&std::fs::read_to_string(&path).unwrap())
						.unwrap_or_else(|_| panic!("Failed to parse {path}"))
				});

				let mut suffixes = call.suffixes().cloned().collect::<Vec<_>>();
				if let Some(last_suffix) = suffixes.last_mut() {
					match last_suffix {
						ast::Suffix::Call(call) => {
							let tokens = call.tokens().cloned().collect::<Vec<_>>();
							process_tokens(tokens, &mut self.semi_colons, |trivia| {
								if let ast::Call::AnonymousCall(args) = call {
									match args {
										ast::FunctionArgs::Parentheses { arguments, .. } => {
											*args = ast::FunctionArgs::Parentheses {
												parentheses: ast::span::ContainedSpan::new(
													TokenReference::symbol("(").unwrap(),
													TokenReference::new(
														trivia.leading,
														Token::new(TokenType::Symbol {
															symbol: tokenizer::Symbol::RightParen,
														}),
														trivia.trailing,
													),
												),
												arguments: arguments.clone(),
											};
										}
										_ => (),
									}
								}
							});
						}

						// Doesn't work atm
						ast::Suffix::Index(index) => {
							let tokens = index.tokens().cloned().collect::<Vec<_>>();
							process_tokens(tokens, &mut self.semi_colons, |trivia| match index {
								ast::Index::Dot { name, .. } => {
									*index = ast::Index::Dot {
										dot: TokenReference::symbol(".").unwrap(),
										name: TokenReference::new(
											trivia.leading,
											name.token().clone(),
											trivia.trailing,
										),
									};
								}
								ast::Index::Brackets { expression, .. } => {
									*index = ast::Index::Brackets {
										brackets: ast::span::ContainedSpan::new(
											TokenReference::symbol("[").unwrap(),
											TokenReference::new(
												trivia.leading,
												Token::new(TokenType::Symbol {
													symbol: tokenizer::Symbol::RightBracket,
												}),
												trivia.trailing,
											),
										),
										expression: expression.clone(),
									};
								}
								_ => (),
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

struct Trivia {
	leading: Vec<Token>,
	trailing: Vec<Token>,
}

fn process_tokens<F>(tokens: Vec<TokenReference>, semicolons: &mut FxHashSet<usize>, mut handler: F)
where
	F: FnMut(Trivia),
{
	for token in tokens.iter() {
		if let TokenType::Symbol { symbol } = token.token_type() {
			if *symbol == tokenizer::Symbol::RightParen
				|| *symbol == tokenizer::Symbol::RightBracket
			{
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
