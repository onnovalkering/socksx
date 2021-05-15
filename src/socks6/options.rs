use anyhow::Result;
use num_traits::FromPrimitive;

#[repr(u8)]
#[derive(Clone, Debug, FromPrimitive)]
pub enum AuthMethod {
    NoAuthentication = 0x00,
    Gssapi = 0x01,
    UsernamePassword = 0x02,
    NoAcceptableMethods = 0xFF,
}

#[derive(Clone, Debug)]
pub enum SocksOption {
    AuthMethodAdvertisement(AuthMethodAdvertisementOption),
    AuthMethodSelection(AuthMethodSelectionOption),
    Metadata(MetadataOption),
    Unrecognized(UnrecognizedOption),
}

impl SocksOption {
    pub fn as_socks_bytes(&self) -> Vec<u8> {
        use SocksOption::*;

        match self {
            AuthMethodAdvertisement(option) => option.clone().into_socks_bytes(),
            AuthMethodSelection(option) => option.clone().into_socks_bytes(),
            Metadata(option) => option.clone().into_socks_bytes(),
            Unrecognized(option) => option.clone().into_socks_bytes(),
        }
    }
}

#[derive(Clone, Debug)]
pub struct AuthMethodAdvertisementOption {
    pub initial_data_length: u16,
    pub methods: Vec<AuthMethod>,
}

impl AuthMethodAdvertisementOption {
    pub fn new(
        initial_data_length: u16,
        methods: Vec<AuthMethod>,
    ) -> SocksOption {
        SocksOption::AuthMethodAdvertisement(Self {
            initial_data_length,
            methods,
        })
    }

    ///
    ///
    ///
    pub fn from_socks_bytes(bytes: Vec<u8>) -> Result<SocksOption> {
        ensure!(bytes.len() >= 2, "Expected at least two bytes, got: {}", bytes.len());
        let initial_data_length = ((bytes[0] as u16) << 8) | bytes[1] as u16;

        let methods = bytes
            .iter()
            .skip(2)
            .filter(|m| {
                let m = **m;
                // Ingore "No Authentication Required" (implied) and padding bytes.
                m > 0 && m < 3
            })
            .map(|m| AuthMethod::from_u8(*m).unwrap())
            .collect();

        Ok(Self::new(initial_data_length, methods))
    }

    ///
    ///
    ///
    pub fn into_socks_bytes(self) -> Vec<u8> {
        let mut data = self.initial_data_length.to_be_bytes().to_vec();
        data.extend(self.methods.iter().cloned().map(|m| m as u8));

        combine_and_pad(0x02, data)
    }
}

#[derive(Clone, Debug)]
pub struct AuthMethodSelectionOption {
    pub method: AuthMethod,
}

impl AuthMethodSelectionOption {
    pub fn new(method: AuthMethod) -> SocksOption {
        SocksOption::AuthMethodSelection(Self { method })
    }

    pub fn from_socks_bytes(bytes: Vec<u8>) -> Result<SocksOption> {
        ensure!(bytes.len() == 4, "Expected exactly four bytes, got: {}", bytes.len());

        let method = bytes[0];
        if let Some(method) = AuthMethod::from_u8(method) {
            Ok(Self::new(method))
        } else {
            bail!("Not a valid authentication method selection: {}", method)
        }
    }

    pub fn into_socks_bytes(self) -> Vec<u8> {
        let data = vec![self.method as u8];

        combine_and_pad(0x03, data)
    }
}

#[derive(Clone, Debug)]
pub struct MetadataOption {
    pub key: u16,
    pub value: String,
}

impl MetadataOption {
    pub fn new(
        key: u16,
        value: String,
    ) -> SocksOption {
        SocksOption::Metadata(Self { key, value })
    }

    pub fn from_socks_bytes(bytes: Vec<u8>) -> Result<SocksOption> {
        ensure!(bytes.len() >= 4, "Expected at least four bytes, got: {}", bytes.len());
        let key = ((bytes[0] as u16) << 8) | bytes[1] as u16;
        let length = ((bytes[2] as u16) << 8) | bytes[3] as u16;

        let value = bytes[4..(length as usize) + 4].to_vec();
        if let Ok(value) = String::from_utf8(value) {
            Ok(Self::new(key, value))
        } else {
            bail!("Not a valid metadata UTF-8 string: {:?}", bytes[2..].to_vec())
        }
    }

    pub fn into_socks_bytes(self) -> Vec<u8> {
        let mut data = self.key.to_be_bytes().to_vec();
        data.extend((self.value.len() as u16).to_be_bytes().iter());
        data.extend(self.value.as_bytes().iter());

        // kind: 65000
        combine_and_pad(0xFDE8, data)
    }
}

#[derive(Clone, Debug)]
pub struct UnrecognizedOption {
    kind: u16,
    data: Vec<u8>,
}

impl UnrecognizedOption {
    pub fn new(
        kind: u16,
        data: Vec<u8>,
    ) -> SocksOption {
        SocksOption::Unrecognized(Self { kind, data })
    }

    pub fn into_socks_bytes(self) -> Vec<u8> {
        combine_and_pad(self.kind, self.data)
    }
}

///
///
///
fn combine_and_pad(
    kind: u16,
    data: Vec<u8>,
) -> Vec<u8> {
    // The total length of the option is the combined number of bytes of
    // the kind, length, and data fields, plus the number of padding bytes.
    let option_length = data.len() + 2 + 2;
    let padding_bytes = vec![0; 4 - (option_length % 4)];
    let total_length: u16 = (option_length + padding_bytes.len()) as u16;

    let mut bytes = vec![];
    bytes.extend(kind.to_be_bytes().iter());
    bytes.extend(total_length.to_be_bytes().iter());
    bytes.extend(data);
    bytes.extend(padding_bytes);

    bytes
}
