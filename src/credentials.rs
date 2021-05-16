#[derive(Clone)]
pub struct Credentials {
    pub username: Vec<u8>,
    pub password: Vec<u8>,
}

impl Credentials {
    ///
    ///
    ///
    pub fn new<S: Into<Vec<u8>>>(
        username: S,
        password: S,
    ) -> Self {
        let username = username.into();
        let password = password.into();

        Credentials { username, password }
    }

    ///
    ///
    ///
    pub fn as_socks_bytes(&self) -> Vec<u8> {
        // Append username
        let mut bytes = vec![self.username.len() as u8];
        bytes.extend(self.username.clone());

        // Append password
        bytes.push(self.password.len() as u8);
        bytes.extend(self.password.clone());

        bytes
    }
}
