use super::column::Column;
use crate::query::Param;

#[derive(Debug, Clone)]
pub struct Info {
	data: Vec<Column>,
}

impl Info {
	pub fn new(data: Vec<Column>) -> Self {
		Self { data }
	}

	pub fn with_capacity(cap: usize) -> Self {
		Self {
			data: Vec::with_capacity(cap),
		}
	}

	pub fn push(&mut self, col: Column) {
		self.data.push(col);
	}

	pub fn data(&self) -> &Vec<Column> {
		&self.data
	}

	pub fn names<'a>(
		&'a self,
	) -> impl ExactSizeIterator<Item = &'static str> + 'a {
		self.data.iter().map(|v| v.name)
	}

	// pub fn validate_params(
	// 	&self,
	// 	params: &[Param],
	// ) -> Result<(), ValidateParamsError> {
	// 	'param_loop: for param in params {
	// 		for col in &self.data {
	// 			if param.name != col.name {
	// 				continue;
	// 			}

	// 			if param.kind == col.kind {
	// 				continue 'param_loop;
	// 			} else {
	// 				return Err(ValidateParamsError(format!(
	// 					"{:?} != {:?}",
	// 					param, col
	// 				)));
	// 			}
	// 		}
	// 		return Err(ValidateParamsError(format!(
	// 			"param: {:?} not found",
	// 			param.name
	// 		)));
	// 	}
	// 	Ok(())
	// }
}

#[derive(Debug, Clone)]
pub struct ValidateParamsError(pub String);
