use std::convert::TryFrom;
use std::error::Error;
use std::fmt::{Debug, Display, Formatter};
use std::rc::Rc;
use gloo_console::error;
use gloo_storage::{SessionStorage, Storage};
use jwt_compact::{Claims, ParseError, UntrustedToken, ValidationError, alg::{RsaPublicKey, Rsa}, AlgorithmExt, jwk::{JsonWebKey}};
use oauth2::{AuthUrl, ClientId, CsrfToken, RedirectUrl, ResponseType, RevocationUrl, Scope, TokenUrl};
use oauth2::basic::{BasicClient};
use oauth2::url::Url;
use serde::{Serialize, Deserialize};
use wasm_bindgen_futures::{spawn_local};
use yew::{Component, Html};
use yew::prelude::*;
use web_sys::{Location, Window, window};
use yew_router::prelude::*;
use crate::planner::PlannerRoutes;
use crate::{UserInfo, UserInfoContext};

const NONCE_KEY: &str = "nonce";
const STATE_KEY: &str = "state";

pub struct Landing {
    google_login_image: String,
    google_oauth_client: Rc<BasicClient>,
    user: Option<UserInfo>,
}

#[derive(Clone)]
pub enum MouseEventType {
    Over,
    Out,
    Down,
    Up
}

pub enum LandingMsg {
    OnMouseEvent(MouseEventType),
    OnLoginClick,
    UpdateUser(UserInfo),
}

impl Component for Landing {
    type Message = LandingMsg;
    type Properties = ();

    fn create(ctx: &Context<Self>) -> Self {
        let client = Rc::new(create_auth_client());
        let link_ref = ctx.link().clone();
        spawn_local(async move {
            match handle_auth_code_redirect().await {
                Ok(user) => {
                    if let Some(user) = user { link_ref.send_message(LandingMsg::UpdateUser(user)) }                    
                },
                Err(e) => error!(e.to_string().as_str())
            }
        });
        
        Self {
            google_login_image: "btn_google_signin_light_normal_web.png".to_string(),
            google_oauth_client: client,
            user: None
        }
    }

    fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            LandingMsg::OnMouseEvent(event_type) => {
                self.google_login_image = match event_type {
                    MouseEventType::Over => "btn_google_signin_light_focus_web.png".to_string(),
                    MouseEventType::Out | MouseEventType::Up => "btn_google_signin_light_normal_web.png".to_string(),
                    MouseEventType::Down => "btn_google_signin_light_pressed_web.png".to_string()
                };
                true
            }
            LandingMsg::OnLoginClick => {
                let nonce = CsrfToken::new_random();
                // Generate the authorization URL to which we'll redirect the user.
                let (authorize_url, csrf_state) = self.google_oauth_client
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
                false
            }
            LandingMsg::UpdateUser(user) => {                
                self.user = Some(user);
                let (user_info_context, _) = _ctx.link().context::<UserInfoContext>(Callback::noop()).unwrap();
                user_info_context.dispatch(self.user.clone());
                let window: Window = window().expect("no global `window` object exists");
                let location: Location = window.location();
                location.set_hash("").expect("url hash/fragment could not be reset");
                true
            }
        }
    }

    fn changed(&mut self, _ctx: &Context<Self>) -> bool {
        false
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let link = ctx.link();
        let mouse_events = |event_type: MouseEventType| {
            link.callback(move |_event: MouseEvent| {
                LandingMsg::OnMouseEvent(event_type.clone())
            })
        };
        let on_mouse_over = mouse_events(MouseEventType::Over);
        let on_mouse_out = mouse_events(MouseEventType::Out);
        let on_mouse_down = mouse_events(MouseEventType::Down);
        let on_mouse_up = mouse_events(MouseEventType::Up);
        let on_login_click = link.callback(|_| {
            LandingMsg::OnLoginClick
        });
        
        match &self.user {
            Some(_) => {
                html! {
                    <div class="content">
                        <Link<PlannerRoutes> to={PlannerRoutes::Overview}>{ "New plan" }</Link<PlannerRoutes>>
                    </div>
                }
            }
            None => {
                html! {
                    <div id="login-content" class="flex-container flex-column">
                        <div id="login-card" class="mdc-card">
                            <div class="mdc-card-wrapper__text-section">
                                <div class="card-title">{ "Login" }</div>
                            </div>
                            <img src={format!("images/{}", self.google_login_image)} alt="Sign in with Google" width="191" height="46"
                                onmouseover={on_mouse_over}
                                onmouseout={on_mouse_out}
                                onmousedown={on_mouse_down}
                                onmouseup={on_mouse_up}
                                onclick={on_login_click} />
                        </div>
                    </div>
                }
            }
        }
    }

    fn rendered(&mut self, _ctx: &Context<Self>, _first_render: bool) {
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

#[derive(Debug, Serialize, Deserialize)]
struct GoogleOpenIdClaims {
    iss: String,
    azp: String,
    aud: String,
    sub: String,
    email: String,
    email_verified: bool,
    at_hash: String,
    nonce: String,
    name: String,
    picture: String,
    given_name: String,
    family_name: String,
    locale: String,
    jti: String
}

enum AuthError {
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
            AuthError::TokenParseError(e) => write!(f, "{}", e.to_string()),
            AuthError::TokenValidationError(e) => write!(f, "{}",  e.to_string()),
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

async fn handle_auth_code_redirect() -> Result<Option<UserInfo>, AuthError> {    
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
            let user = validate_id_token(&fragments).await?;
            
            Ok(Some(user))
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

async fn validate_id_token(fragments: &[(&str, &str)]) -> Result<UserInfo, AuthError> {
    let id_token = fragments.iter().find(|(key, _)| *key == "id_token");
    match id_token {
        Some((_, value)) => {
            let signing_keys = get_google_signing_keys().await;
            signing_keys
                .map_err(|_| AuthError::Other("error getting signing keys".into()))
                .and_then(|signing_keys| {
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
                            match token_message {
                                Ok(data) => {
                                    validate_nonce(data.claims())
                                }
                                Err(message) => {
                                    Err(AuthError::TokenValidationError(message))
                                }
                            }
                        })
                })
            
        }
        None => Err(AuthError::MissingIdTokenInResponse)
    }
}

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

fn validate_nonce(data: &Claims<GoogleOpenIdClaims>) -> Result<UserInfo, AuthError> {
    let nonce = SessionStorage::get(NONCE_KEY);
    nonce
        .map_err(|_| AuthError::MissingNonceInStorage)
        .and_then(|nonce: String| {
            SessionStorage::delete(NONCE_KEY);
            if data.custom.nonce != nonce {
                return Err(AuthError::MismatchedNonce);
            }
            Ok(UserInfo {
                email: data.custom.email.clone(),
                picture: data.custom.picture.clone(),
                name: data.custom.name.clone()
            })
        })
}