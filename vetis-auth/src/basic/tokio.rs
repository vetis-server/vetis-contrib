use crate::basic::{Algorithm, BasicAuthConfig};
use argon2::{PasswordHash, PasswordVerifier};
use base64::Engine;
use http::{HeaderValue, StatusCode, header};
use std::{collections::HashMap, path::PathBuf, sync::Arc};
use tokio::sync::RwLock;
use vetis::{
    Request, Response, VetisFutureResult, VetisResult,
    auth::Auth,
    errors::{HostError, VetisError},
};

/// BasicAuth holds actual data for basic auth purposes
///
/// # Notes
///
/// If htpasswd file is provided, that will populate
/// users map only if users map wasn't previously set.
pub struct BasicAuth {
    users: Arc<RwLock<HashMap<String, String>>>,
    algo: Arc<Algorithm>,
    htpasswd: Arc<Option<PathBuf>>,
    realm: Arc<Option<String>>,
}

impl BasicAuth {
    /// Build a new BasicAuth out of its configuration
    pub fn new(config: BasicAuthConfig) -> Self {
        let users = Arc::new(RwLock::new(config.users));
        let algo = config
            .algorithm
            .clone();
        Self {
            users,
            algo: Arc::new(algo),
            htpasswd: Arc::new(config.htpasswd),
            realm: Arc::new(config.realm),
        }
    }
}

impl Auth for BasicAuth {
    fn authenticate<'a>(&self, request: &'a Request) -> VetisFutureResult<'a, Response> {
        let users = self.users.clone();
        let algo = self.algo.clone();
        let realm = self.realm.clone();
        let file = self
            .htpasswd
            .clone();

        let future = async move {
            let users = if users
                .read()
                .await
                .is_empty()
                && let Some(file) = &*file
                && file.exists()
            {
                let Ok(users_from_file) = load_users_from_file(file).await else {
                    return Err(VetisError::Host(HostError::Auth(
                        "Could not load users from file".into(),
                    )));
                };

                users
                    .write()
                    .await
                    .clone_from(&users_from_file);

                users
            } else {
                users
            };

            let Some(auth_header) = request
                .headers()
                .get(http::header::AUTHORIZATION)
            else {
                return unauthorized_respose(realm).await;
            };

            let Ok(auth_header) = auth_header.to_str() else {
                return unauthorized_respose(realm).await;
            };

            let Some(pair) = auth_header.strip_prefix("Basic ") else {
                return unauthorized_respose(realm).await;
            };

            let auth_header = base64::engine::general_purpose::STANDARD.decode(auth_header);
            let auth_header = auth_header.map_err(|e| {
                VetisError::Host(HostError::Auth(format!("Could not decode header: {}", e)))
            })?;

            let auth_header = String::from_utf8(auth_header).map_err(|_| {
                VetisError::Host(HostError::Auth("Invalid Authorization header".to_string()))
            })?;

            let (username, password) = auth_header
                .split_once(':')
                .ok_or(VetisError::Host(HostError::Auth("Invalid credentials".to_string())))?;

            let guard = users.read().await;
            let saved_password = guard
                .get(username)
                .cloned();
            drop(guard);

            if let Some(hashed_password) = saved_password {
                let password = password.to_string();
                let verify_task = tokio::task::spawn_blocking(move || {
                    verify_password(password, hashed_password, *algo)
                })
                .await;

                let auth_success = match verify_task {
                    Ok(result) => result,
                    Err(e) => {
                        return Err(VetisError::Host(HostError::Auth(format!(
                            "Could not verify password: {}",
                            e
                        ))));
                    }
                };

                if auth_success {
                } else {
                    return unauthorized_respose(realm).await;
                }
            }

            unauthorized_respose(realm).await
        };

        Box::pin(future)
    }
}

async fn unauthorized_respose(realm: Arc<Option<String>>) -> VetisResult<Response> {
    let header = if let Some(realm) = realm.as_ref() {
        HeaderValue::from_str(&format!("Basic realm=\"{}\"", realm))
            .map_err(|e| VetisError::Host(HostError::Auth(e.to_string())))?
    } else {
        HeaderValue::from_str("Basic")
            .map_err(|e| VetisError::Host(HostError::Auth(e.to_string())))?
    };

    Ok(Response::builder()
        .status(StatusCode::UNAUTHORIZED)
        .header(header::WWW_AUTHENTICATE, header)
        .empty())
}

async fn load_users_from_file(file: &PathBuf) -> VetisResult<HashMap<String, String>> {
    let htpasswd = if file.exists() {
        file
    } else {
        return Err(VetisError::Host(HostError::Auth(".htpasswd file does not exists".into())));
    };

    let htpasswd = tokio::fs::read_to_string(htpasswd).await;
    let mut users = HashMap::new();
    match htpasswd {
        Ok(file) => {
            file.lines()
                .for_each(|line| {
                    let credentials = line.split_once(':');

                    if let Some(credentials) = credentials {
                        users.insert(
                            credentials
                                .0
                                .to_string(),
                            credentials
                                .1
                                .to_string(),
                        );
                    }
                });
        }
        Err(e) => {
            log::error!("Failed to read htpasswd file: {}", e);
        }
    }
    Ok(users)
}

fn verify_password(password: String, hashed_password: String, algorithm: Algorithm) -> bool {
    match algorithm {
        Algorithm::BCrypt => {
            bcrypt::verify(password.as_str(), hashed_password.as_str()).unwrap_or(false)
        }
        Algorithm::Argon2 => {
            let argon2 = argon2::Argon2::default();
            let parsed_hash = PasswordHash::new(hashed_password.as_str());
            match parsed_hash {
                Ok(parsed_hash) => {
                    let result = argon2.verify_password(password.as_bytes(), &parsed_hash);
                    result.is_ok()
                }
                Err(_) => false,
            }
        }
    }
}
