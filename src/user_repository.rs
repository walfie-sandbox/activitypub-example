use async_trait::async_trait;

use openssl::rsa::Rsa;

use std::collections::HashMap;
use std::error::Error;

#[derive(Clone, Debug)]
pub struct User {
    pub username: String,
    pub public_key: Vec<u8>,
    pub private_key: Vec<u8>,
}

#[async_trait]
pub trait UserRepository {
    async fn get_user(&mut self, username: &str) -> Result<User, Box<dyn Error>>;
}

pub struct InMemoryUserRepository {
    pub domain: String,
    cache: HashMap<String, User>,
}

impl InMemoryUserRepository {
    pub fn new(domain: String) -> Self {
        InMemoryUserRepository {
            domain,
            cache: HashMap::new(),
        }
    }
}

#[async_trait]
impl UserRepository for InMemoryUserRepository {
    async fn get_user(&mut self, username: &str) -> Result<User, Box<dyn Error>> {
        match self.cache.get(username) {
            Some(user) => return Ok(user.clone()),
            None => {
                let keypair = Rsa::generate(4096)?;

                let public_key = keypair.public_key_to_der()?.to_vec();
                let private_key = keypair.private_key_to_der()?.to_vec();

                let user = User {
                    username: username.to_string(),
                    public_key,
                    private_key,
                };

                self.cache.insert(username.to_string(), user.clone());
                Ok(user)
            }
        }
    }
}
