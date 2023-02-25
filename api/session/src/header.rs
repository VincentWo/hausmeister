use headers::{authorization::Bearer, Authorization, Header};
use uuid::Uuid;

#[derive(Clone, Debug)]
pub(crate) struct UncheckedSessionId(pub Uuid);

impl Header for UncheckedSessionId {
    fn name() -> &'static headers::HeaderName {
        Authorization::<Bearer>::name()
    }

    fn decode<'i, I>(values: &mut I) -> Result<Self, headers::Error>
    where
        Self: Sized,
        I: Iterator<Item = &'i headers::HeaderValue>,
    {
        let Authorization(bearer) = Authorization::<Bearer>::decode(values)?;

        Ok(UncheckedSessionId(
            bearer.token().parse().or(Err(headers::Error::invalid()))?,
        ))
    }

    fn encode<E: Extend<headers::HeaderValue>>(&self, values: &mut E) {
        let token = self.0.to_string();
        let bearer = Authorization::bearer(&token).expect("An uuid is always a valid header value");

        bearer.encode(values)
    }
}
