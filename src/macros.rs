/// Make a lua function call:
/// -- path
/// (function(...) ... end)();
#[macro_export]
macro_rules! make_function_call {
	($path:expr, $ast:expr, $suffixes:expr) => {
		full_moon::ast::FunctionCall::new(full_moon::ast::Prefix::Expression(Box::new(
			full_moon::ast::Expression::Parentheses {
				contained: full_moon::ast::span::ContainedSpan::new(
					full_moon::tokenizer::TokenReference::new(
						vec![full_moon::tokenizer::Token::new(
							full_moon::tokenizer::TokenType::SingleLineComment {
								comment: format!(" {}\n", $path).into(),
							},
						)],
						full_moon::tokenizer::Token::new(full_moon::tokenizer::TokenType::Symbol {
							symbol: full_moon::tokenizer::Symbol::LeftParen,
						}),
						Vec::new(),
					),
					full_moon::tokenizer::TokenReference::symbol(")").unwrap(),
				),
				expression: Box::new(full_moon::ast::Expression::Function(Box::new((
					full_moon::tokenizer::TokenReference::symbol("function").unwrap(),
					full_moon::ast::FunctionBody::new()
						.with_parameters({
							let mut punctuated = full_moon::ast::punctuated::Punctuated::new();
							punctuated.push(full_moon::ast::punctuated::Pair::End(
								full_moon::ast::Parameter::Ellipsis(
									full_moon::tokenizer::TokenReference::symbol("...").unwrap(),
								),
							));
							punctuated
						})
						.with_block($ast.nodes().clone())
						.with_end_token(
							full_moon::tokenizer::TokenReference::symbol("end").unwrap(),
						),
				)))),
			},
		)))
		.with_suffixes({
			let new: Vec<full_moon::ast::Suffix> = vec![full_moon::ast::Suffix::Call(
				full_moon::ast::Call::AnonymousCall(full_moon::ast::FunctionArgs::Parentheses {
					parentheses: full_moon::ast::span::ContainedSpan::new(
						full_moon::tokenizer::TokenReference::symbol("(").unwrap(),
						full_moon::tokenizer::TokenReference::new(
							Vec::new(),
							full_moon::tokenizer::Token::new(
								full_moon::tokenizer::TokenType::Symbol {
									symbol: full_moon::tokenizer::Symbol::RightParen,
								},
							),
							{
								if $suffixes.len() == 0 {
									vec![
										full_moon::tokenizer::Token::new(
											full_moon::tokenizer::TokenType::Symbol {
												symbol: full_moon::tokenizer::Symbol::Semicolon,
											},
										),
										full_moon::tokenizer::Token::new(
											full_moon::tokenizer::TokenType::Whitespace {
												characters: "\n".into(),
											},
										),
									]
								} else {
									Vec::new()
								}
							},
						),
						// full_moon::tokenizer::TokenReference::symbol(")").unwrap(),
					),
					arguments: full_moon::ast::punctuated::Punctuated::new(),
				}),
			)]
			.into_iter()
			.chain($suffixes.into_iter())
			.collect();
			new
		})
	};
}

#[macro_export]
macro_rules! add_semicolon_if_needed {
	($token_ref:expr, $semicolons:expr) => {
		$token_ref.end_position().and_then(|mut end_pos| {
			$token_ref.trailing_trivia().for_each(|token| {
				if matches!(
					token.token_type(),
					full_moon::tokenizer::TokenType::Whitespace { .. }
				) {
					end_pos = token.end_position();
				}
			});

			if !$semicolons.contains(&end_pos.character()) {
				let trailing = vec![
					full_moon::tokenizer::Token::new(full_moon::tokenizer::TokenType::Symbol {
						symbol: tokenizer::Symbol::Semicolon,
					}),
					full_moon::tokenizer::Token::new(full_moon::tokenizer::TokenType::Whitespace {
						characters: "\n".into(),
					}),
				];

				Some(trailing)
			} else {
				None
			}
		})
	};
}
