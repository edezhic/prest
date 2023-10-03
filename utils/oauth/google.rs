use openidconnect::{core::*, *};

pub struct GoogleClient(GoogleOAuthClient);

impl GoogleClient {
    pub async fn init(origin: &str, client_id: String, client_secret: String) -> Self {
        let redirect_url = RedirectUrl::new(format!("{origin}/oauth/google/callback")).unwrap();
        let client_id = ClientId::new(client_id);
        let client_secret = ClientSecret::new(client_secret);
        let issuer_url = IssuerUrl::new("https://accounts.google.com".to_string()).unwrap();
        let provider_metadata =
            CoreProviderMetadata::discover_async(issuer_url, super::oauth_http_client)
                .await
                .unwrap();

        let client =
            CoreClient::from_provider_metadata(provider_metadata, client_id, Some(client_secret))
                .set_redirect_uri(redirect_url);

        Self(client)
    }
    pub fn authz_request(&self, scopes: &[&str]) -> (url::Url, CsrfToken, Nonce) {
        let mut authz_req = self.0.authorize_url(
            AuthenticationFlow::<CoreResponseType>::AuthorizationCode,
            CsrfToken::new_random,
            Nonce::new_random,
        );
        for scope in scopes {
            authz_req = authz_req.add_scope(Scope::new(scope.to_string()));
        }
        authz_req.url()
    }

    pub async fn get_token_and_claims(
        &self,
        code: String,
        nonce: Nonce,
    ) -> (
        GoogleTokenResponse,
        IdTokenClaims<EmptyAdditionalClaims, CoreGenderClaim>,
    ) {
        let token = self
            .0
            .exchange_code(AuthorizationCode::new(code))
            .request_async(super::oauth_http_client)
            .await
            .expect("valid token response");

        let id_claims = token
            .extra_fields()
            .id_token()
            .expect("Server did not return an ID token")
            .claims(&self.0.id_token_verifier(), &nonce)
            .unwrap()
            .clone();

        (token, id_claims)
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
