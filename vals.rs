use crate::*;

/// Utility that deserializes from either [`Query`] or [`Form`] based on request method
pub struct Vals<T>(pub T);
#[async_trait]
impl<T, S> FromRequest<S> for Vals<T>
where
    T: serde::de::DeserializeOwned + Send,
    S: Send + Sync,
{
    type Rejection = Error;

    async fn from_request(req: Request<Body>, state: &S) -> Result<Self> {
        if req.method() == Method::GET {
            let (mut parts, _) = req.into_parts();
            match Query::<T>::from_request_parts(&mut parts, state).await {
                Ok(Query(params)) => Ok(Vals(params)),
                Err(e) => Err(e.into()),
            }
        } else {
            match Json::<T>::from_request(req, state).await {
                Ok(Json(params)) => Ok(Vals(params)),
                Err(e) => Err(e.into()),
            }
        }
    }
}

impl<T> std::ops::Deref for Vals<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T> std::ops::DerefMut for Vals<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<T, S> extract::FromRef<S> for Vals<T>
where
    S: Send + Sync,
{
    fn from_ref(_state: &S) -> Self {
        unreachable!("Param is not created from state")
    }
}
