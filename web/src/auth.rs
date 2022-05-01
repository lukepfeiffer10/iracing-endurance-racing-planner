use std::convert::TryFrom;
use std::error::Error;
use std::fmt::{Display, Formatter};
use gloo_storage::{LocalStorage, SessionStorage, Storage};
use jwt_compact::{Claims, ValidationError, ParseError, Token, UntrustedToken, AlgorithmExt};
use jwt_compact::alg::{Rsa, RsaPublicKey};
use jwt_compact::jwk::JsonWebKey;
use oauth2::basic::BasicClient;
use oauth2::{AuthUrl, ClientId, CsrfToken, RedirectUrl, ResponseType, RevocationUrl, Scope, TokenUrl, url::Url};
use serde::{Deserialize};
use web_sys::{Location, Window, window};
use endurance_racing_planner_common::{GoogleOpenIdClaims, User};
use crate::UserInfo;

const NONCE_KEY: &str = "nonce";
const STATE_KEY: &str = "state";
pub const ID_TOKEN_KEY: &str = "id_token";

#[derive(Deserialize)]
struct GoogleDiscoveryResponse {
    jwks_uri: String
}

#[derive(Deserialize)]
struct GoogleSigningKey<'a> {
    kid: String,
    #[serde(flatten)]
    key: JsonWebKey<'a>
}

#[derive(Deserialize)]
struct GoogleSigningKeysResponse<'a> {
    keys: Vec<GoogleSigningKey<'a>>
}

pub enum AuthError {
    TokenParseError(ParseError),
    TokenValidationError(ValidationError),
    MismatchedNonce,
    MismatchedState,
    MissingIdTokenInResponse,
    MissingStateInResponse,
    MissingStateInStorage,
    MissingNonceInStorage,
    MissingTokenSigningKey,
    Other(String),
}

impl Display for AuthError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            AuthError::TokenParseError(e) => write!(f, "{}", e),
            AuthError::TokenValidationError(e) => write!(f, "{}",  e),
            AuthError::MismatchedNonce => write!(f, "mismatched nonce in the token response"),
            AuthError::MismatchedState => write!(f, "mismatched state in the token response"),
            AuthError::MissingIdTokenInResponse => write!(f, "missing id_token in the response"),
            AuthError::MissingStateInResponse => write!(f, "missing state in the response"),
            AuthError::MissingStateInStorage => write!(f, "missing state in session storage"),
            AuthError::MissingNonceInStorage => write!(f, "missing nonce in session storage"),
            AuthError::MissingTokenSigningKey => write!(f, "missing signing key from discovery response used to sign token"),
            AuthError::Other(message) => write!(f, "{}", message),
        }
    }
}

impl From<&str> for AuthError {
    fn from(value: &str) -> Self {
        Self::Other(value.into())
    }
}

fn create_auth_client() -> BasicClient {
    let google_client_id = ClientId::new(
        "709154627100-fbcvr0njtbah2jfgv5bghnt7t39r28k9.apps.googleusercontent.com".to_string()
    );
    let auth_url = AuthUrl::new("https://accounts.google.com/o/oauth2/v2/auth".to_string())
        .expect("Invalid authorization endpoint URL");
    let token_url = TokenUrl::new("https://www.googleapis.com/oauth2/v3/token".to_string())
        .expect("Invalid token endpoint URL");

    // Set up the config for the Google OAuth2 process.
    BasicClient::new(
        google_client_id,
        None,
        auth_url,
        Some(token_url),
    )
        .set_redirect_uri(
            RedirectUrl::new("http://localhost:9000/".to_string()).expect("Invalid redirect URL"),
        )
        // Google supports OAuth 2.0 Token Revocation (RFC-7009)
        .set_revocation_uri(
            RevocationUrl::new("https://oauth2.googleapis.com/revoke".to_string())
                .expect("Invalid revocation endpoint URL"),
        )
}

pub fn login() {
    let nonce = CsrfToken::new_random();
    // Generate the authorization URL to which we'll redirect the user.
    let (authorize_url, csrf_state) = create_auth_client()
        .authorize_url(CsrfToken::new_random)
        .add_scope(Scope::new(
            "https://www.googleapis.com/auth/userinfo.email".to_string(),
        ))
        .add_scope(Scope::new(
            "https://www.googleapis.com/auth/userinfo.profile".to_string(),
        ))
        .add_scope(Scope::new("openid".to_string()))
        .add_extra_param(NONCE_KEY, nonce.secret())
        .set_response_type(&ResponseType::new("token id_token".to_string()))
        .url();

    SessionStorage::set(NONCE_KEY, nonce.secret().to_owned()).expect("can't set session storage");
    SessionStorage::set(STATE_KEY, csrf_state.secret().to_owned()).expect("can't set session storage");

    let window: Window = window().expect("no global `window` object exists");
    let location: Location = window.location();
    location.set_href(authorize_url.as_str()).expect("location couldn't be changed");
}

pub async fn handle_auth_code_redirect() -> Result<Option<UserInfo>, AuthError> {
    let window: Window = window().expect("no global `window` object exists");
    let location: Location = window.location();
    let url = location.href().expect("no location `href` exists");
    let url = Url::parse(url.as_str()).unwrap();
    if let Some(fragment) = url.fragment() {
        return if !fragment.is_empty() {
            let fragments = fragment.split('&')
                .map(|section| {
                    let key_value_split = section.split('=').collect::<Vec<&str>>();
                    let key = key_value_split[0];
                    let value = key_value_split[1];
                    (key, value)
                })
                .collect::<Vec<_>>();

            validate_state_parameter(&fragments)?;
            let token = validate_id_token(&fragments).await?;
            validate_nonce(token.claims())?;
            let get_me_result = (get_me().await).map_err(|_| AuthError::Other("failed to get me".into()));
            let me = match get_me_result {
                Ok(user) => Ok(user),
                Err(_) => {
                    match create_user(&token.claims().custom).await {
                        Ok(created_user) => Ok(created_user),
                        Err(_) => Err(AuthError::Other("failed to create a user".into()))
                    }
                }
            }?;

            Ok(Some(UserInfo {
                name: me.name,
                email: me.email,
                picture: token.claims().custom.picture.clone()
            }))
        } else {
            Ok(None)
        }
    }
    Ok(None)
}

fn validate_state_parameter(fragments: &[(&str, &str)]) -> Result<(), AuthError> {
    let url_state = fragments.iter().find(|(key, _)| *key == "state");
    match url_state {
        Some((_, state)) => {
            let stored_state = SessionStorage::get(STATE_KEY);
            stored_state
                .map_err(|_| AuthError::MissingStateInStorage)
                .and_then(|stored_state: String| {
                    SessionStorage::delete(STATE_KEY);
                    if *state == stored_state.as_str() {
                        Ok(())
                    } else {
                        Err(AuthError::MismatchedState)
                    }
                })
        }
        None => {
            Err(AuthError::MissingStateInResponse)
        }
    }
}

async fn validate_id_token(fragments: &[(&str, &str)]) -> Result<Token<GoogleOpenIdClaims>, AuthError> {
    let id_token = fragments.iter().find(|(key, _)| *key == "id_token");
    match id_token {
        Some((_, value)) => {
            let signing_keys = get_google_signing_keys().await;
            signing_keys
                .map_err(|_| AuthError::Other("error getting signing keys".into()))
                .and_then(|signing_keys| {
                    LocalStorage::set(ID_TOKEN_KEY, value).expect("failed to set id token in local storage");
                    let token = UntrustedToken::new(value);
                    token
                        .map_err(AuthError::TokenParseError)
                        .and_then(|token| {
                            let mut signing_key = &signing_keys.keys[0].key;
                            let token_key_id = &token.header().key_id;
                            if let Some(key_id) = token_key_id {
                                signing_key = signing_keys.keys
                                    .iter()
                                    .find(|key| &key.kid == key_id)
                                    .map(|k| &k.key)
                                    .ok_or(AuthError::MissingTokenSigningKey)?;
                            }
                            let rsa_public_key = RsaPublicKey::try_from(signing_key).unwrap();
                            let token_message = Rsa::rs256().validate_integrity::<GoogleOpenIdClaims>(&token, &rsa_public_key);
                            token_message.map_err(AuthError::TokenValidationError)
                        })
                })

        }
        None => Err(AuthError::MissingIdTokenInResponse)
    }
}

async fn get_google_signing_keys() -> Result<GoogleSigningKeysResponse<'static>, Box<dyn Error>>{
    const GOOGLE_DISCOVERY_URL: &str = "https://accounts.google.com/.well-known/openid-configuration";

    let client = reqwest::Client::new();
    let discovery_info = client
        .get(GOOGLE_DISCOVERY_URL)
        .header("Accept", "application/json")
        .send()
        .await?
        .json::<GoogleDiscoveryResponse>()
        .await?;

    let signing_keys = client
        .get(&discovery_info.jwks_uri)
        .header("Accept", "application/json")
        .send()
        .await?
        .json::<GoogleSigningKeysResponse>()
        .await?;

    Ok(signing_keys)
}

fn validate_nonce(data: &Claims<GoogleOpenIdClaims>) -> Result<(), AuthError> {
    let nonce = SessionStorage::get(NONCE_KEY);
    nonce
        .map_err(|_| AuthError::MissingNonceInStorage)
        .and_then(|nonce: String| {
            SessionStorage::delete(NONCE_KEY);
            if data.custom.nonce != nonce {
                Err(AuthError::MismatchedNonce)
            } else {
                Ok(())
            }
        })
}

async fn create_user(claims: &GoogleOpenIdClaims) -> Result<User, Box<dyn Error>> {
    let user = User {
        id: 0,
        name: claims.name.clone(),
        email: claims.email.clone(),
        oauth_id: claims.sub.clone()
    };

    let client = reqwest::Client::new();
    let new_user = client
        .post("http://localhost:3000/users")
        .header("Accept", "application/json")
        .body(serde_json::to_string(&user).expect("failed to convert user to serde json value"))
        .send()
        .await?
        .json::<User>()
        .await?;

    Ok(new_user)
}

pub async fn get_me() -> Result<User, Box<dyn Error>> {
    let token: String = LocalStorage::get(ID_TOKEN_KEY)?;
    let client = reqwest::Client::new();
    let me = client
        .get("http://localhost:3000/users/me")
        .header("Accept", "application/json")
        .header("Authorization", format!("Bearer {}", token))
        .send()
        .await?
        .json::<User>()
        .await?;

    Ok(me)
}
