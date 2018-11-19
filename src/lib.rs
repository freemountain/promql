/*!
This crate allows one to parse PromQL query into some AST.

See [official documentation](https://prometheus.io/docs/prometheus/latest/querying/basics/) for query syntax description.

## Example

```
# extern crate nom;
# extern crate promql;
# fn main() {

use promql::*;

let ast = parse(b"
	sum(1 - something_used{env=\"production\"} / something_total) by (instance)
	and ignoring (instance)
	sum(rate(some_queries{instance=~\"localhost\\\\d+\"} [5m])) > 100
").unwrap(); // or show user that their query is invalid

// now we can look for all sorts of things

// AST can represent an operator
if let Node::Operator { x, op: Op::And(op_mod), y } = ast {
	// operators can have modifiers
	assert_eq!(op_mod, Some(OpMod {
		action: OpModAction::Ignore,
		labels: vec!["instance".to_string()],
		group: None,
	}));

	// aggregation functions like sum are represented as functions with optional modifiers (`by (label1, …)`/`without (…)`)
	if let Node::Function { ref name, ref aggregation, ref args } = *x {
		assert_eq!(*name, "sum".to_string());
		assert_eq!(*aggregation, Some(AggregationMod {
			action: AggregationAction::By,
			labels: vec!["instance".to_string()],
		}));

		// …
	}
} else {
	panic!("top operator is not an \"and\"");
}

# }
```
*/

#[macro_use]
extern crate nom;
#[macro_use]
extern crate quick_error;
#[macro_use]
extern crate serde_derive;

pub(crate) mod str;
pub(crate) mod vec;
pub(crate) mod expr;

pub use vec::*;
pub use expr::*;

use nom::{Err, ErrorKind};
use nom::types::CompleteByteSlice;

/**
Parse expression string into an AST.

This parser operates on byte sequence instead of `&str` because of the fact that PromQL, like Go, allows raw byte sequences to be included in the string literals (e.g. `{omg='∞'}` is equivalent to both `{omg='\u221e'}` and `{omg='\xe2\x88\x9e'}`).
*/
pub fn parse(e: &[u8]) -> Result<Node, nom::Err<CompleteByteSlice>> {
	match expression(CompleteByteSlice(e)) {
		Ok((CompleteByteSlice(b""), ast)) => Ok(ast),
		Ok((tail, _)) => Err(Err::Error(error_position!(tail, ErrorKind::Complete::<u32>))),
		Err(e) => Err(e),
	}
}

#[cfg(test)]
mod tests {
	use nom::{Err, ErrorKind, Context};
	use nom::types::CompleteByteSlice;

	#[test]
	fn completeness() {
		assert_eq!(
			super::parse(b"asdf hjkl"),
			Err(Err::Error(Context::Code(CompleteByteSlice(b"hjkl"), ErrorKind::Complete)))
		);
	}
}
