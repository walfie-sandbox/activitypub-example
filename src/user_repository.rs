use openssl::rsa::Rsa;
use std::collections::HashMap;
use std::error::Error;
use std::sync::RwLock;

#[derive(Clone, Debug)]
pub struct User {
    pub username: String,
    pub public_key: Vec<u8>,
    pub private_key: Vec<u8>,
}

pub trait UserRepository {
    fn get_user(&self, username: &str) -> Result<User, Box<dyn Error + Send + Sync>>;
}

pub struct InMemoryUserRepository {
    pub domain: String,
    cache: RwLock<HashMap<String, User>>,
}

impl InMemoryUserRepository {
    pub fn new(domain: String) -> Self {
        InMemoryUserRepository {
            domain,
            cache: RwLock::new(HashMap::new()),
        }
    }
}

impl UserRepository for InMemoryUserRepository {
    fn get_user(&self, username: &str) -> Result<User, Box<dyn Error + Send + Sync>> {
        {
            if let Some(user) = self
                .cache
                .read()
                .expect("failed to read cache")
                .get(username)
            {
                return Ok(user.clone());
            }
        }

        let keypair = Rsa::generate(4096)?;

        let public_key = keypair.public_key_to_der()?.to_vec();
        let private_key = keypair.private_key_to_der()?.to_vec();

        let user = User {
            username: username.to_string(),
            public_key,
            private_key,
        };

        self.cache
            .write()
            .expect("failed to obtain write lock on cache")
            .insert(username.to_string(), user.clone());
        Ok(user)
    }
}
