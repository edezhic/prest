

Includes [Google OAuth](https://developers.google.com/identity/protocols/oauth2) + [OpenID](https://developers.google.com/identity/openid-connect/openid-connect) flows and in-memory session + user management. Expects `GOOGLE_CLIENT_ID`, `GOOGLE_CLIENT_SECRET` and `ADMIN_EMAIL` environment variables.

OAuth and OpenID flows are powered by [openidconnect-rs](https://github.com/ramosbugs/openidconnect-rs) which provides extensible strongly-typed interfaces. 

User + session management is powered by [axum-login](https://github.com/maxcountryman/axum-login) which is quite fast and flexible. Even basic example is still pretty verbose, but this library is evolving and APIs are already much easier to use than just a few months ago. 