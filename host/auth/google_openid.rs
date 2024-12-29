use crate::*;
use openidconnect::{core::*, reqwest::async_http_client, *};

state!(WITH_GOOGLE_AUTH: bool = {
    env_var("GOOGLE_CLIENT_ID").is_ok() && env_var("GOOGLE_CLIENT_SECRET").is_ok()
});

state!(GOOGLE_CLIENT: GoogleClient = async {
    if !*WITH_GOOGLE_AUTH {
        panic!("Attempted to use google client without credentials!")
    }
    let client_id = env_var("GOOGLE_CLIENT_ID")?;
    let client_secret = env_var("GOOGLE_CLIENT_SECRET")?;

    let domain = if let Some(domain) = &APP_CONFIG.domain {
        format!("https://{domain}")
    } else {
        format!("http://localhost")
    };
    let callback_url = format!("{domain}{GOOGLE_CALLBACK_ROUTE}");
    GoogleClient::init(callback_url, client_id, client_secret).await
});

pub struct GoogleClient(GoogleOAuthClient);

impl GoogleClient {
    pub async fn init(callback_url: String, client_id: String, client_secret: String) -> Self {
        let redirect_url = RedirectUrl::new(callback_url).unwrap();
        let client_id = ClientId::new(client_id);
        let client_secret = ClientSecret::new(client_secret);
        let issuer_url = IssuerUrl::new("https://accounts.google.com".to_string()).unwrap();
        let provider_metadata = CoreProviderMetadata::discover_async(issuer_url, async_http_client)
            .await
            .unwrap();

        let client =
            CoreClient::from_provider_metadata(provider_metadata, client_id, Some(client_secret))
                .set_redirect_uri(redirect_url);

        Self(client)
    }
    pub fn authz_request(&self) -> (url::Url, OAuthCSRF, OAuthNonce) {
        let mut authz_req = self.0.authorize_url(
            AuthenticationFlow::<CoreResponseType>::AuthorizationCode,
            OAuthCSRF::new_random,
            OAuthNonce::new_random,
        );
        let scopes = ["email"];
        for scope in scopes {
            authz_req = authz_req.add_scope(Scope::new(scope.to_string()));
        }
        authz_req.url()
    }

    pub async fn get_email(&self, code: OAuthCode, nonce: OAuthNonce) -> Result<String> {
        let token = self.get_token(code).await?;
        Ok(token
            .extra_fields()
            .id_token()
            .ok_or(e!("server did not return an ID token"))?
            .claims(&self.0.id_token_verifier(), &nonce)?
            .email()
            .ok_or(e!("email not found in openID claims"))?
            .to_string())
    }

    pub async fn get_token(&self, code: OAuthCode) -> Result<GoogleTokenResponse> {
        Ok(self
            .0
            .exchange_code(AuthorizationCode::new(code))
            .request_async(async_http_client)
            .await
            .somehow()?)
    }
}

pub type GoogleTokenResponse = StandardTokenResponse<
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

pub type GoogleOAuthClient = Client<
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
