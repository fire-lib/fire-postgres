
use super::{Query, SqlBuilder, Param};

#[derive(Debug, Clone)]
pub struct UpdateParams<'a> {
	params: Vec<Param<'a>>
}

impl<'a> UpdateParams<'a> {
	pub fn new(params: Vec<Param<'a>>) -> Self {
		Self {params}
	}

	pub fn into_query(self) -> Query<'a> {

		let mut sql = SqlBuilder::new();

		let last = self.params.len() - 1;
		for (i, param) in self.params.iter().enumerate() {

			sql.space_after(format!("\"{}\" =", param.name));
			sql.param();

			if i != last {
				sql.space_after(",");
			}

		}

		Query::new(sql, self.params)
	}
}

/// ## Example
/// ```
/// use fire_postgres::updt;
/// let a = &"val".to_string();
/// let b = &"val2".to_string();
/// let query = updt!{
/// 	a,
/// 	"b": b
/// }.into_query();
/// 
/// assert_eq!(r#""a" = $1, "b" = $2"#, query.sql().to_string().trim());
/// ```
#[macro_export]
macro_rules! updt {
	($($tt:tt)*) => ({
		let mut params = vec![];
		$crate::updt_item!(params, $($tt)*);
		$crate::query::UpdateParams::new(params)
	})
}

#[macro_export]
macro_rules! updt_item {
	($p:ident, $id:ident, $($tt:tt)+) => (
		$crate::updt_item!($p, $id);
		$crate::updt_item!($p, $($tt)+);
	);
	($p:ident, &$id:ident, $($tt:tt)+) => (
		$crate::updt_item!($p, &$id);
		$crate::updt_item!($p, $($tt)+);
	);
	($p:ident, $name:tt: $value:expr, $($tt:tt)+) => (
		$crate::updt_item!($p, $name: $value);
		$crate::updt_item!($p, $($tt)+)
	);
	($p:ident, $id:ident) => (
		$p.push($crate::query::Param::new(stringify!($id), $id));
	);
	($p:ident, &$id:ident) => (
		$p.push($crate::query::Param::new(stringify!($id), &$id));
	);
	($p:ident, $name:tt: $value:expr) => (
		$p.push($crate::query::Param::new($name, $value));
	);
}

/*
abc, "abc": abc
*/