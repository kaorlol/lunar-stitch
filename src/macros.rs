/// Make a lua call:\
/// ```lua
/// -- path.lua
/// (function(...) ... end)();
/// ```
#[macro_export]
macro_rules! make_call {
	($type:expr, $path:expr, $ast:expr, $suffixes:expr, $needs_semi:expr) => {
		paste::paste! {
			$type::new(Prefix::Expression(Box::new(
				Expression::Parentheses {
					contained: span::ContainedSpan::new(
						TokenReference::new(
							vec![Token::new(
								TokenType::SingleLineComment {
									comment: format!(" {}\n", $path).into(),
								},
							)],
							Token::new(TokenType::Symbol {
								symbol: Symbol::LeftParen,
							}),
							Vec::new(),
						),
						TokenReference::symbol(")").unwrap(),
					),
					expression: Box::new(Expression::Function(Box::new((
						TokenReference::symbol("function").unwrap(),
						FunctionBody::new()
							.with_parameters({
								let mut punctuated = punctuated::Punctuated::new();
								punctuated.push(punctuated::Pair::End(
									Parameter::Ellipsis(
										TokenReference::symbol("...").unwrap(),
									),
								));
								punctuated
							})
							.with_block($ast.nodes().clone())
							.with_end_token(
								TokenReference::symbol("end").unwrap(),
							),
					)))),
				},
			)))
			.with_suffixes({
				let mut new_suffixes = Vec::new();
				new_suffixes.push(Suffix::Call(
					Call::AnonymousCall(FunctionArgs::Parentheses {
						parentheses: span::ContainedSpan::new(
							TokenReference::symbol("(").unwrap(),
							TokenReference::new(
								Vec::new(),
								Token::new(
									TokenType::Symbol {
										symbol: Symbol::RightParen,
									},
								),
								{
									if $suffixes.len() == 0 && $needs_semi {
										vec![
											Token::new(
												TokenType::Symbol {
													symbol: Symbol::Semicolon,
												},
											),
											Token::new(
												TokenType::Whitespace {
													characters: "\n".into(),
												},
											),
										]
									} else {
										Vec::new()
									}
								},
							),
						),
						arguments: punctuated::Punctuated::new(),
					}),
				));
				new_suffixes.extend($suffixes.into_iter());
				new_suffixes
			})
		}
	};
}

#[macro_export]
macro_rules! get_range {
	($call:expr) => {{
		std::ops::Range {
			start: $call.prefix().start_position().unwrap().character(),
			end: $call
				.suffixes()
				.last()
				.unwrap()
				.end_position()
				.unwrap()
				.character(),
		}
	}};
}

/// Get the suffixes of a call and process the tokens to add semicolons where needed.\
/// Returns the suffixes without the first one.
#[macro_export]
macro_rules! get_suffixes {
	($call:expr, $semi_colons:expr, $needs_semi:expr) => {{
		let mut suffixes: Vec<Suffix> = $call.suffixes().cloned().collect();
		if let Some(last_suffix) = suffixes.last_mut() {
			match last_suffix {
				Suffix::Call(call) => {
					let tokens: Vec<TokenReference> = call.tokens().cloned().collect();
					process_tokens(tokens, &mut $semi_colons, $needs_semi, |trivia| {
						if let Call::AnonymousCall(args) = call {
							match args {
								FunctionArgs::Parentheses { arguments, .. } => {
									*args = FunctionArgs::Parentheses {
										parentheses: span::ContainedSpan::new(
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

				Suffix::Index(index) => {
					let tokens = index.tokens().cloned().collect::<Vec<_>>();
					process_tokens(
						tokens,
						&mut $semi_colons,
						$needs_semi,
						|trivia| match index {
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
									brackets: span::ContainedSpan::new(
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
						},
					);
				}
				_ => (),
			}
		}
		suffixes.remove(0);
		suffixes
	}};
}

/// Gets the end position of a token,
/// if there is trailing whitespace it will return the end of position of the whitespace,
/// otherwise it will return the end position of the token.\
/// If the end position is not in the semicolons list, it will add a semicolon and a newline.
#[macro_export]
macro_rules! add_semicolon_if_needed {
	($token_ref:expr, $semicolons:expr, $needs_semi:expr) => {
		if !$needs_semi {
			None
		} else {
			$token_ref.end_position().and_then(|mut end_pos| {
				$token_ref.trailing_trivia().for_each(|token| {
					if matches!(token.token_type(), TokenType::Whitespace { .. }) {
						end_pos = token.end_position();
					}
				});

				if !$semicolons.contains(&end_pos.character()) {
					let trailing = vec![
						Token::new(TokenType::Symbol {
							symbol: Symbol::Semicolon,
						}),
						Token::new(TokenType::Whitespace {
							characters: "\n".into(),
						}),
					];

					return Some(trailing);
				}
				None
			})
		}
	};
}
