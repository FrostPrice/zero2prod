use crate::{
    authentication::UserId,
    idempotency::{save_response, try_processing, IdempotencyKey, NextAction},
    utils::{e400, e500, see_other},
};
use actix_web::{web, HttpResponse};
use actix_web_flash_messages::FlashMessage;
use anyhow::Context;
use sqlx::Executor;

use sqlx::{PgPool, Postgres, Transaction};
use uuid::Uuid;

#[derive(serde::Deserialize)]
pub struct FormData {
    title: String,
    text_content: String,
    html_content: String,
    idempotency_key: String,
}

#[tracing::instrument(name = "Publish a newsletter issue", skip_all, fields(user_id = %&*user_id))]
pub async fn publish_newsletter(
    form: web::Form<FormData>,
    pool: web::Data<PgPool>,
    user_id: web::ReqData<UserId>,
) -> Result<HttpResponse, actix_web::Error> {
    let user_id = user_id.into_inner();
    // Destructure the form to please the borrow checker
    let FormData {
        title,
        text_content,
        html_content,
        idempotency_key,
    } = form.0;
    let idempotency_key: IdempotencyKey = idempotency_key.try_into().map_err(e400)?;

    // Return early if we have a saved response in the Database
    let mut transaction = match try_processing(&pool, &idempotency_key, *user_id)
        .await
        .map_err(e500)?
    {
        NextAction::StartProcessing(t) => t,
        NextAction::ReturnSavedResponse(saved_response) => {
            success_message().send();
            return Ok(saved_response);
        }
    };

    let issue_id = insert_newsletter_issue(&mut transaction, &title, &text_content, &html_content)
        .await
        .context("Failed to store newsletter issue details.")
        .map_err(e500)?;
    enqueue_delivery_tasks(&mut transaction, issue_id)
        .await
        .context("Failed to enqueue delivery tasks.")
        .map_err(e500)?;

    success_message().send();
    let response = see_other("/admin/newsletters");
    let response = save_response(transaction, &idempotency_key, *user_id, response)
        .await
        .map_err(e500)?;

    Ok(response)
}

fn success_message() -> FlashMessage {
    FlashMessage::success("The newsletter issue has been accepted - emails will go out shortly.")
}

#[tracing::instrument(skip_all)]
async fn insert_newsletter_issue(
    transaction: &mut Transaction<'_, Postgres>,
    title: &str,
    text_content: &str,
    html_content: &str,
) -> Result<Uuid, sqlx::Error> {
    let newsletter_issue_id = Uuid::new_v4();
    let query = sqlx::query!(
        r#"
        INSERT INTO newsletter_issues (
            newsletter_issue_id,
            title,
            text_content,
            html_content,
            published_at
        )
        VALUES ($1, $2, $3, $4, NOW())
        "#,
        newsletter_issue_id,
        title,
        text_content,
        html_content,
    );
    transaction.execute(query).await?;
    Ok(newsletter_issue_id)
}

#[tracing::instrument(skip_all)]
async fn enqueue_delivery_tasks(
    transaction: &mut Transaction<'_, Postgres>,
    newsletter_issue_id: Uuid,
) -> Result<(), sqlx::Error> {
    let query = sqlx::query!(
        r#"
        INSERT INTO issue_delivery_queue (
            newsletter_issue_id,
            subscriber_email
        )
        SELECT $1, email
        FROM subscriptions
        WHERE status = 'confirmed'
        "#,
        newsletter_issue_id
    );
    transaction.execute(query).await?;

    Ok(())
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
