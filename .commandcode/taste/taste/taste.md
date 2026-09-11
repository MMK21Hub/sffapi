# Taste
- Prefers managing Rust dependencies via the Cargo CLI (`cargo add <crate>`) rather than hand-editing `Cargo.toml`. Confidence: 0.6
- Prefers the agent not spend effort on testing/verification (e.g. scratch-DB smoke tests) during implementation unless explicitly asked — getting the code written and compiling is enough. Confidence: 0.5
- Prefers API endpoints to return informative, structured JSON error responses (distinct types for not-found vs. server errors) rather than generic errors. Confidence: 0.55
- Favors the simplest implementation that works and challenges added complexity (e.g. questioning `web::block`), and will direct its removal even after the trade-offs are explained — when the trade-off is marginal, prefers dropping the extra machinery (direct sync connection borrow) over correctness-under-load. Confidence: 0.65
- Wants non-obvious code decisions justified (why this signature/wrapper), with the trade-offs and a simpler alternative spelled out. Confidence: 0.5
- When asking how to implement something, prefers concrete illustrative example code (explicitly invites placeholder code) over abstract prose. Confidence: 0.55
- Prefers explicit, readable error-handling code over dense nested matching — e.g. flattens nested `Result` layers (`Ok(Ok(..))` / `Ok(Err(..))`) into separate `match`es so each handles one layer. Confidence: 0.6
