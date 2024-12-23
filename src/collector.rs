// TODO: deabstraction pattern
// TODO: Better comments :)

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

pub struct AcquireCollector {
	pub root: String,
	pub input: String,
	pub output: String,
	pub processed_cache: FxHashMap<String, Ast>,
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

	pub fn contains_acquire(&self, prefix: &Prefix) -> bool {
		prefix.tokens().any(|token| {
			matches!(
				token.token_type(),
				TokenType::Identifier { identifier } if identifier == &"acquire".into()
			)
		})
	}

	pub fn grab_acquire_path(&self, call: &FunctionCall) -> Option<String> {
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

impl VisitorMut for AcquireCollector {
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
