/// Possible operators
/// - = | != | < | <= | > | >= | LIKE | IN
/// - AND | OR
/// ## Example
/// ```
/// use fire_postgres::whr;
/// let a = &"val".to_string();
/// let b = &"val2".to_string();
/// let c: &Option<String> = &None;
/// let query = whr!(a AND "b" != b OR c);
///
/// assert_eq!(r#""a" = $1 AND "b" != $2 OR "c" IS NULL"#, query.sql().to_string().trim());
/// ```
#[macro_export]
macro_rules! whr {
	($($tt:tt)*) => ({
		#[allow(unused_mut)]
		let mut params = vec![];
		let mut sql = $crate::query::SqlBuilder::new();
		$crate::whr_comp!(sql, params, $($tt)*);
		$crate::query::Query::new(sql, params)
	})
}

#[macro_export]
macro_rules! whr_comp {
	($s:ident, $p:ident,) => ();
	($s:ident, $p:ident, LIMIT $value:tt $($tt:tt)*) => ({
		let v: usize = $value;
		$s.space(format!("LIMIT {}", v));
		$crate::whr_comp!($s, $p, $($tt)*);
	});
	// value should not be a user input
	($s:ident, $p:ident, ORDER $value:tt ASC $($tt:tt)*) => ({
		$s.space(format!("ORDER BY \"{}\" ASC", $value));
		$crate::whr_comp!($s, $p, $($tt)*);
	});
	// value should not be a user input
	($s:ident, $p:ident, ORDER $value:tt DESC $($tt:tt)*) => ({
		$s.space(format!("ORDER BY \"{}\" DESC", $value));
		$crate::whr_comp!($s, $p, $($tt)*);
	});
	($s:ident, $p:ident, $id:ident $($tt:tt)*) => (
		$crate::short_whr_comp!($s, $p, stringify!($id), "=", $id, $($tt)*);
	);
	($s:ident, $p:ident, $name:tt = $value:tt $($tt:tt)*) => (
		$crate::short_whr_comp!($s, $p, $name, "=", $value, $($tt)*);
	);
	($s:ident, $p:ident, $name:tt != $value:tt $($tt:tt)*) => (
		$crate::short_whr_comp!($s, $p, $name, "!=", $value, $($tt)*);
	);
	($s:ident, $p:ident, $name:tt < $value:tt $($tt:tt)*) => (
		$crate::short_whr_comp!($s, $p, $name, "<", $value, $($tt)*);
	);
	($s:ident, $p:ident, $name:tt <= $value:tt $($tt:tt)*) => (
		$crate::short_whr_comp!($s, $p, $name, "<=", $value, $($tt)*);
	);
	($s:ident, $p:ident, $name:tt > $value:tt $($tt:tt)*) => (
		$crate::short_whr_comp!($s, $p, $name, ">", $value, $($tt)*);
	);
	($s:ident, $p:ident, $name:tt >= $value:tt $($tt:tt)*) => (
		$crate::short_whr_comp!($s, $p, $name, ">=", $value, $($tt)*);
	);
	($s:ident, $p:ident, $name:tt LIKE $value:tt $($tt:tt)*) => (
		$crate::short_whr_comp!($s, $p, $name, "LIKE", $value, $($tt)*);
	);
	($s:ident, $p:ident, $name:tt ~ $value:tt $($tt:tt)*) => (
		$crate::short_whr_comp!($s, $p, $name, "LIKE", format!("%{}%", $value), $($tt)*);
	);
	($s:ident, $p:ident, $name:tt ~= $value:tt $($tt:tt)*) => (
		$crate::short_whr_comp!($s, $p, $name, "LIKE", format!("%{}", $value), $($tt)*);
	);
	($s:ident, $p:ident, $name:tt =~ $value:tt $($tt:tt)*) => (
		$crate::short_whr_comp!($s, $p, $name, "LIKE", format!("{}%", $value), $($tt)*);
	);
	($s:ident, $p:ident, $name:tt IN $value:tt $($tt:tt)*) => (
		$crate::short_whr_in_comp!($s, $p, $name, $value, $($tt)*);
	);
}

#[macro_export]
macro_rules! short_whr_comp {
	($s:ident, $p:ident, $name:expr, $symb:expr, $value:expr, $($tt:tt)*) => (
		let param = $crate::query::Param::new($name, $value);

		let mut cont = true;

		// todo: can this become a noop
		if param.maybe_null() && param.data().is_null() {
			match $symb {
				"=" => {
					$s.space_after(format!("\"{}\" IS NULL", $name));
					cont = false;
				},
				"!=" => {
					$s.space_after(format!("\"{}\" IS NOT NULL", $name));
					cont = false;
				},
				_ => {}
			}
		}

		if cont {
			$s.space_after(format!("\"{}\" {}", $name, $symb));
			$s.param();
			$p.push(param);
		}

		$crate::whr_log!($s, $p, $($tt)*);
	);
	// LIMIT
}

#[macro_export]
macro_rules! short_whr_in_comp {
	($s:ident, $p:ident, $name:expr, $value:expr, $($tt:tt)*) => (
		if $value.iter().len() > 0 {
			$s.no_space(format!("\"{}\" IN (", $name));
			let end = $value.iter().len() - 1;
			for (i, v) in $value.iter().enumerate() {
				$s.param();
				$p.push($crate::query::Param::new($name, v));
				if i != end {
					$s.space_after(",");
				}
			}
			$s.no_space(")");
		} else {
			$s.space_after(format!("\"{}\" IS NULL", $name));
		}
		$crate::whr_log!($s, $p, $($tt)*);
	);
}

#[macro_export]
macro_rules! whr_log {
	($s:ident, $p:ident, AND $($tt:tt)+) => (
		$s.space("AND");
		$crate::whr_comp!($s, $p, $($tt)+);
	);
	($s:ident, $p:ident, OR $($tt:tt)+) => (
		$s.space("OR");
		$crate::whr_comp!($s, $p, $($tt)+);
	);
	($s:ident, $p:ident, LIMIT $($tt:tt)+) => (
		$crate::whr_comp!($s, $p, LIMIT $($tt)+);
	);
	($s:ident, $p:ident, ORDER $($tt:tt)+) => (
		$crate::whr_comp!($s, $p, ORDER $($tt)+);
	);
	($s:ident, $p:ident,) => ();
}

#[cfg(test)]
mod tests {
	use crate::UniqueId;

	#[test]
	fn test_limit() {
		let id = &UniqueId::new();
		let limit = 10;
		let query = whr!(id LIMIT limit);
		assert_eq!(query.sql.to_string().trim(), "\"id\" = $1 LIMIT 10");
	}

	#[test]
	fn test_order() {
		let id = &UniqueId::new();
		let limit = 10;
		let query = whr!(id ORDER "id" ASC LIMIT limit);
		assert_eq!(
			query.sql.to_string().trim(),
			"\"id\" = $1 ORDER BY \"id\" ASC  LIMIT 10"
		);
		let query = whr!(ORDER "id" DESC);
		assert_eq!(query.sql.to_string().trim(), "ORDER BY \"id\" DESC");
	}
}
