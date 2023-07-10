use crate::{
    auth::{AuthContext, Email, User},
    ENV_GOOGLE_CLIENT_ID, ENV_GOOGLE_CLIENT_SECRET, ENV_ORIGIN,
};

use axum::{
    extract::Query,
    response::{IntoResponse, Redirect},
};
use axum_login::axum_sessions::extractors::{ReadableSession, WritableSession};
use openidconnect::{core::*, reqwest::async_http_client, *};

lazy_static! {
    static ref GCLIENT: GoogleOAuthClient = tokio::task::block_in_place(|| {
        tokio::runtime::Handle::current().block_on(init_client())
    });
    static ref GVERIFIER: CoreIdTokenVerifier<'static> = GCLIENT.id_token_verifier();
}

pub async fn authorize(mut session: WritableSession) -> impl IntoResponse {
    let (authz_url, csrf_token, nonce) = init_authz_request();

    session.insert("nonce", nonce).unwrap();
    session.insert("csrf", csrf_token).unwrap();

    Redirect::to(authz_url.as_ref())
}

pub async fn callback(
    session: ReadableSession,
    Query(query): Query<OAuthQuery>,
    mut auth: AuthContext,
) -> impl IntoResponse {
    let Some(initial_csrf) = session.get::<CsrfToken>("csrf") else { panic!("missing csrf!") };
    let Some(nonce) = session.get::<Nonce>("nonce") else { panic!("missing nonce!") };

    // remove anonymous session
    drop(session);

    // validate CSRF
    if initial_csrf.secret() != query.state.secret() {
        tracing::warn!("mismatched csrf! Might be something shady going on");
        return Redirect::to("/");
    }

    let token = get_oauth_token(query.code).await;
    let email = extract_openid_data(token, nonce);
    let email = Email::new_unchecked(email);

    let user = match crate::Storage::get_user_by_email(&email).await {
        Some(user) => user,
        None => User::signup(email, None).await.unwrap(),
    };

    auth.login(&user).await.unwrap();

    Redirect::to("/")
}

fn init_authz_request() -> (url::Url, CsrfToken, Nonce) {
    GCLIENT
        .authorize_url(
            AuthenticationFlow::<CoreResponseType>::AuthorizationCode,
            CsrfToken::new_random,
            Nonce::new_random,
        )
        .add_scope(Scope::new("email".to_string()))
        .url()
}

async fn get_oauth_token(code: String) -> GoogleTokenResponse {
    GCLIENT
        .exchange_code(AuthorizationCode::new(code))
        .request_async(async_http_client)
        .await
        .expect("valid token response")
}

fn extract_openid_data(token: GoogleTokenResponse, nonce: Nonce) -> String {
    let id_token_claims = token
        .extra_fields()
        .id_token()
        .expect("Server did not return an ID token")
        .claims(&GVERIFIER, &nonce)
        .unwrap();

    id_token_claims.email().unwrap().as_str().to_owned()
}

async fn init_client() -> GoogleOAuthClient {
    use std::env::var;
    let origin = var(ENV_ORIGIN).unwrap();
    let redirect_url = RedirectUrl::new(format!("{origin}/oauth/google/callback")).unwrap();
    let google_client_id = ClientId::new(var(ENV_GOOGLE_CLIENT_ID).unwrap());
    let google_client_secret = ClientSecret::new(var(ENV_GOOGLE_CLIENT_SECRET).unwrap());
    let issuer_url = IssuerUrl::new("https://accounts.google.com".to_string()).unwrap();
    let provider_metadata = CoreProviderMetadata::discover_async(issuer_url, async_http_client)
        .await
        .unwrap();

    let client = CoreClient::from_provider_metadata(
        provider_metadata,
        google_client_id,
        Some(google_client_secret),
    )
    .set_redirect_uri(redirect_url);

    client
}

#[derive(Debug, serde::Deserialize)]
pub struct OAuthQuery {
    pub code: String,
    pub state: CsrfToken,
}

type GoogleTokenResponse = StandardTokenResponse<
    IdTokenFields<
        EmptyAdditionalClaims,
        EmptyExtraTokenFields,
        CoreGenderClaim,
        CoreJweContentEncryptionAlgorithm,
        CoreJwsSigningAlgorithm,
        CoreJsonWebKeyType,
    >,
    CoreTokenType,
>;

type GoogleOAuthClient = Client<
    EmptyAdditionalClaims,
    CoreAuthDisplay,
    CoreGenderClaim,
    CoreJweContentEncryptionAlgorithm,
    CoreJwsSigningAlgorithm,
    CoreJsonWebKeyType,
    CoreJsonWebKeyUse,
    CoreJsonWebKey,
    CoreAuthPrompt,
    StandardErrorResponse<CoreErrorResponseType>,
    GoogleTokenResponse,
    CoreTokenType,
    StandardTokenIntrospectionResponse<EmptyExtraTokenFields, CoreTokenType>,
    CoreRevocableToken,
    StandardErrorResponse<RevocationErrorResponseType>,
>;
