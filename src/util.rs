use argon2::Argon2;
use hex::encode;

pub fn hash_password(password: &str) -> [u8; 32] {
  let salt = format!("{password}_salt_lol");
  let mut output_password = [0u8; 32];
  Argon2::default().hash_password_into(
    password.as_bytes(),
    salt.as_bytes(),
    &mut output_password
  ).expect("unable to hash password");
  output_password
}

pub fn hash_password_to_string(password: &str) -> String {
  let salt = format!("{password}_salt_lol");
  let mut output_password = [0u8; 32];
  Argon2::default().hash_password_into(
    password.as_bytes(),
    salt.as_bytes(),
    &mut output_password
  ).expect("unable to hash password");
  encode(output_password)
}