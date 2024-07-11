use crate::{
    authentication::UserId,
    domain::SubscriberEmail,
    email_client::EmailClient,
    utils::{e500, see_other},
};
use actix_web::{web, HttpResponse};
use actix_web_flash_messages::FlashMessage;
use anyhow::Context;

use sqlx::PgPool;

#[derive(serde::Deserialize)]
pub struct FormData {
    title: String,
    text_content: String,
    html_content: String,
}

#[tracing::instrument(name = "Publish a newsletter issue", skip(body,pool,email_client, user_id), fields(user_id = %*user_id))]
pub async fn publish_newsletter(
    body: web::Form<FormData>,
    pool: web::Data<PgPool>,
    email_client: web::Data<EmailClient>,
    user_id: web::ReqData<UserId>,
) -> Result<HttpResponse, actix_web::Error> {
    let subscribers = get_confirmed_subscriber(&pool).await.map_err(e500)?;
    for subscriber in subscribers {
        match subscriber {
            Ok(subscriber) => {
                email_client
                    .send_email(
                        &subscriber.email,
                        &body.title,
                        &body.html_content,
                        &body.text_content,
                    )
                    .await
                    .with_context(|| {
                        format!("Failed to send newsletter issue to {}", subscriber.email)
                    })
                    .map_err(e500)?;
            }
            Err(error) => {
                tracing::warn!(
                    // Record the error chain as a structured field
                    // on the log record.
                    error.cause_chain = ?error,
                    // Using `\` to split a long string literal over
                    // two lines, without creating a `\n` character.
                    "Skipping a confirmed subscriber.\
                    Their stored contact details are invalid."
                );
            }
        }
    }
    FlashMessage::info("The newsletter issue has been published!").send();
    Ok(see_other("/admin/newsletters"))
}

struct ConfirmedSubscriber {
    email: SubscriberEmail,
}

#[tracing::instrument(name = "Get confirmed subscriber", skip(pool))]
async fn get_confirmed_subscriber(
    pool: &PgPool,
) -> Result<Vec<Result<ConfirmedSubscriber, anyhow::Error>>, anyhow::Error> {
    let rows = sqlx::query!(
        r#"
        SELECT email 
        FROM subscriptions 
        WHERE status = 'confirmed'
        "#,
    )
    .fetch_all(pool)
    .await?;

    // Map into the domain type
    let confirmed_subscribers = rows
        .into_iter()
        .map(|r| match SubscriberEmail::parse(r.email) {
            Ok(email) => Ok(ConfirmedSubscriber { email }),
            Err(error) => Err(anyhow::anyhow!(error)),
        })
        .collect();

    Ok(confirmed_subscribers)
}

// ! BELLOW IS AN IMPLEMENTATION WITH BASIC AUTHENTICATION
// ! IS ONLY FOR DEMONSTRATION PURPOSES
// ! IT IS NOT CURRENTLY BEING USED

// #[derive(serde::Deserialize)]
// pub struct BodyData {
//     title: String,
//     content: Content,
// }

// #[derive(serde::Deserialize)]
// pub struct Content {
//     text: String,
//     html: String,
// }

// #[tracing::instrument(name = "Publish a newsletter issue", skip(body,pool,email_client, request), fields(username=tracing::field::Empty, user_id=tracing::field::Empty))]
// pub async fn publish_newsletter(
//     body: web::Json<BodyData>,
//     pool: web::Data<PgPool>,
//     email_client: web::Data<EmailClient>,
//     request: HttpRequest,
// ) -> Result<HttpResponse, PublishError> {
//     let credentials = basic_authentication(request.headers()).map_err(PublishError::AuthError)?;
//     tracing::Span::current().record("username", &tracing::field::display(&credentials.username));

//     let user_id = validate_credentials(credentials, &pool)
//         .await
//         // We match on `AuthError`'s variants, but we pass the **whole** error
//         // into the constructors for `PublishError` variants. This ensures that
//         // the context of the top-level wrapper is preserved when the error is
//         // logged by our middleware.
//         .map_err(|e| match e {
//             AuthError::InvalidCredentials(_) => PublishError::AuthError(e.into()),
//             AuthError::UnexpectedError(_) => PublishError::UnexpectedError(e.into()),
//         })?;
//     tracing::Span::current().record("user_id", &tracing::field::display(&user_id));

//     let subscribers = get_confirmed_subscriber(&pool).await?;
//     for subscriber in subscribers {
//         match subscriber {
//             Ok(subscriber) => {
//                 email_client
//                     .send_email(
//                         &subscriber.email,
//                         &body.title,
//                         &body.content.html,
//                         &body.content.text,
//                     )
//                     .await
//                     .with_context(|| {
//                         format!("Failed to send newsletter issue to {}", subscriber.email)
//                     })?;
//             }
//             Err(error) => {
//                 tracing::warn!(
//                     // Record the error chain as a structured field
//                     // on the log record.
//                     error.cause_chain = ?error,
//                     // Using `\` to split a long string literal over
//                     // two lines, without creating a `\n` character.
//                     "Skipping a confirmed subscriber.\
//                     Their stored contact details are invalid."
//                 );
//             }
//         }
//     }
//     Ok(HttpResponse::Ok().finish())
// }

// struct ConfirmedSubscriber {
//     email: SubscriberEmail,
// }

// #[tracing::instrument(name = "Get confirmed subscriber", skip(pool))]
// async fn get_confirmed_subscriber(
//     pool: &PgPool,
// ) -> Result<Vec<Result<ConfirmedSubscriber, anyhow::Error>>, anyhow::Error> {
//     let rows = sqlx::query!(
//         r#"
//         SELECT email
//         FROM subscriptions
//         WHERE status = 'confirmed'
//         "#,
//     )
//     .fetch_all(pool)
//     .await?;

//     // Map into the domain type
//     let confirmed_subscribers = rows
//         .into_iter()
//         .map(|r| match SubscriberEmail::parse(r.email) {
//             Ok(email) => Ok(ConfirmedSubscriber { email }),
//             Err(error) => Err(anyhow::anyhow!(error)),
//         })
//         .collect();

//     Ok(confirmed_subscribers)
// }

// #[derive(thiserror::Error)]
// pub enum PublishError {
//     #[error("Authentication Failed")]
//     AuthError(#[source] anyhow::Error),
//     #[error(transparent)]
//     UnexpectedError(#[from] anyhow::Error),
// }

// impl std::fmt::Debug for PublishError {
//     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//         error_chain_fmt(self, f)
//     }
// }

// impl ResponseError for PublishError {
//     fn error_response(&self) -> HttpResponse {
//         match self {
//             PublishError::UnexpectedError(_) => {
//                 HttpResponse::new(StatusCode::INTERNAL_SERVER_ERROR)
//             }
//             PublishError::AuthError(_) => {
//                 let mut response = HttpResponse::new(StatusCode::UNAUTHORIZED);
//                 let header_value = HeaderValue::from_str(r#"Basic realm="publish""#).unwrap();
//                 response
//                     .headers_mut()
//                     // actix_web::http::header provides a collection of constants
//                     // for the names of several well-known/standard HTTP headers
//                     .insert(header::WWW_AUTHENTICATE, header_value);

//                 response
//             }
//         }
//     }

//     // `status_code` is invoked by the default `error_response`
//     // implementation. We are providing a bespoke `error_response` implementation
//     // therefore there is no need to maintain a `status_code` implementation.
// }

// fn basic_authentication(headers: &HeaderMap) -> Result<Credentials, anyhow::Error> {
//     // The header value, if present, must be a valid UTF8 string
//     let header_value = headers
//         .get("Authorization")
//         .context("The 'Authorization' header was missing.")?
//         .to_str()
//         .context("The 'Authorization' header was not a valid UTF8 string.")?;
//     let base4encoded_segment = header_value
//         .strip_prefix("Basic ")
//         .context("The authorization scheme was not 'Basic'.")?;
//     let decode_bytes = base64::engine::general_purpose::STANDARD
//         .decode(base4encoded_segment)
//         .context("Failed to base64-decode 'Basic' crededntials.")?;
//     let decoded_credentials = String::from_utf8(decode_bytes)
//         .context("The decoded credential string is not valid UTF8.")?;

//     // Split into 2 segments, using ":" as delimiter
//     let mut credentials = decoded_credentials.splitn(2, ':');
//     let username = credentials
//         .next()
//         .ok_or_else(|| anyhow::anyhow!("A username must be provided in the 'Basic' auth."))?
//         .to_string();
//     let password = credentials
//         .next()
//         .ok_or_else(|| anyhow::anyhow!("A password must be provided in the 'Basic' auth."))?
//         .to_string();

//     Ok(Credentials {
//         username,
//         password: Secret::new(password),
//     })
// }
