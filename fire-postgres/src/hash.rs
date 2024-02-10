use crate::Result;

pub fn hash<P: AsRef<[u8]>>(password: P) -> Result<String> {
	Ok(bcrypt::hash(password, 12)?)
}

pub fn verify<P: AsRef<[u8]>>(password: P, hash: &str) -> Result<bool> {
	Ok(bcrypt::verify(password, hash)?)
}
