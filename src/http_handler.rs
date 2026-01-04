use aws_config::BehaviorVersion;
use aws_sdk_sts::Client as StsClient;
use lambda_http::{Body, Error, Request, Response};
use serde::Serialize;

#[derive(Serialize)]
struct S3Credentials {
    access_key_id: String,
    secret_access_key: String,
    session_token: String,
    expiration: String,
}

#[derive(Serialize)]
struct ErrorResponse {
    error: String,
}

pub(crate) async fn function_handler(event: Request) -> Result<Response<Body>, Error> {
    let method = event.method();
    let path = event.uri().path();

    println!("Received request: {:?}", event);

    match (method.as_str(), path) {
        ("POST", "/Prod/api-zhongli") => get_s3_credentials().await,
        _ => {
            let error = ErrorResponse {
                error: format!("Received request: {:?}", event),
            };
            let resp = Response::builder()
                .status(404)
                .header("content-type", "application/json")
                .body(serde_json::to_string(&error)?.into())
                .map_err(Box::new)?;
            Ok(resp)
        }
    }
}

async fn get_s3_credentials() -> Result<Response<Body>, Error> {
    let role_arn = "arn:aws:iam::658140043938:role/api-zhongli";

    let bucket_name = "storage-zhongli-dev";

    let config = aws_config::load_defaults(BehaviorVersion::latest()).await;
    let sts_client = StsClient::new(&config);

    let policy = format!(
        r#"{{
        "Version": "2012-10-17",
        "Statement": [
            {{
                "Effect": "Allow",
                "Action": [
                    "s3:GetObject",
                    "s3:PutObject"
                ],
                "Resource": "arn:aws:s3:::{}/*"
            }},
            {{
                "Effect": "Allow",
                "Action": [
                    "s3:ListBucket"
                ],
                "Resource": "arn:aws:s3:::{}"
            }}
        ]
    }}"#,
        bucket_name, bucket_name
    );

    let assume_role_result = match sts_client
        .assume_role()
        .role_arn(role_arn)
        .role_session_name("blog-frontend-session")
        .policy(&policy)
        .duration_seconds(3600) // 1 hour
        .send()
        .await
    {
        Ok(resp) => {
            // Print the whole SDK response for debugging
            println!("AssumeRole raw response: {:#?}", resp);
            resp
        }
        Err(sdk_err) => {
            // Print the raw SDK error (full debug)
            eprintln!("Failed to assume role (raw SDK error): {:#?}", sdk_err);
            // Return the SDK error wrapped as the function's Error
            return Err(Box::new(sdk_err));
        }
    };

    let credentials = assume_role_result
        .credentials()
        .ok_or("No credentials returned")?;

    let s3_creds = S3Credentials {
        access_key_id: credentials.access_key_id().to_string(),
        secret_access_key: credentials.secret_access_key().to_string(),
        session_token: credentials.session_token().to_string(),
        expiration: credentials.expiration().to_string(),
    };

    let resp = Response::builder()
        .status(200)
        .header("content-type", "application/json")
        .header("access-control-allow-origin", "*")
        .header("access-control-allow-methods", "POST, OPTIONS")
        .header("access-control-allow-headers", "Content-Type")
        .body(serde_json::to_string(&s3_creds)?.into())
        .map_err(Box::new)?;

    Ok(resp)
}

#[cfg(test)]
mod tests {
    use super::*;
    use lambda_http::Request;

    #[tokio::test]
    async fn test_not_found() {
        let request = Request::default();
        let response = function_handler(request).await.unwrap();
        assert_eq!(response.status(), 404);
    }
}
