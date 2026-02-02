mod user_create_tests {
    use actix_web::{App, http::StatusCode, test, web};
    use auth_plus_billing::{
        config::database::get_connection,
        core::{
            dto::user::User,
            repository::orm::{delete_gateway_integration, delete_user},
        },
        presentation::http::routes::user::{self, CreateUserInputSchema},
    };
    use fake::{
        Fake,
        faker::{internet::en::FreeEmail, name::en::Name},
        uuid::UUIDv4,
    };
    use httpmock::prelude::{MockServer, POST};
    use serde_json::json;
    use uuid::Uuid;

    #[actix_web::test]
    async fn should_create_user() {
        let conn = get_connection().await;
        let external_id: Uuid = UUIDv4.fake();
        let name: String = Name().fake();
        let email: String = FreeEmail().fake();
        let server = MockServer::start();
        let mock_gateway_host = server.mock(|when, then| {
            when.method(POST).path("/v1/customers");
            then.status(201)
                .header("content-type", "text/json; charset=UTF-8")
                .json_body(json!({
                    "id": "cus_123",
                    "name":  name.clone(),
                    "email": email.clone(),
                    "balance": 0,
                    "created": 123456789,
                    "livemode": false
                }));
        });
        let payload = CreateUserInputSchema {
            external_id: external_id.to_string(),
            name: name.clone(),
            email: email.clone(),
        };
        let prev_stripe_base_url = std::env::var("STRIPE_BASE_URL").ok();
        unsafe {
            std::env::set_var("STRIPE_BASE_URL", &server.base_url());
        }
        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(server.base_url()))
                .service(user::create_user),
        )
        .await;
        let req = test::TestRequest::post()
            .uri("/user")
            .set_json(web::Json(payload))
            .to_request();
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::OK);
        mock_gateway_host.assert();
        let body: User = test::read_body_json(resp).await;
        delete_user(&conn, body.id)
            .await
            .expect("should_list_invoices: user remove went wrong");
        unsafe {
            match prev_stripe_base_url {
                Some(value) => std::env::set_var("STRIPE_BASE_URL", value),
                None => std::env::remove_var("STRIPE_BASE_URL"),
            }
        }
    }

    #[actix_web::test]
    async fn should_fail_when_user_does_not_exist() {
        let name: String = Name().fake();
        let email: String = FreeEmail().fake();
        let payload = CreateUserInputSchema {
            external_id: String::from("any-hash-that-is-not-uuid"),
            name: name.clone(),
            email: email.clone(),
        };

        let app = test::init_service(App::new().service(user::create_user)).await;
        let req = test::TestRequest::post()
            .uri("/user")
            .set_json(web::Json(payload))
            .to_request();
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::INTERNAL_SERVER_ERROR);
    }
}
