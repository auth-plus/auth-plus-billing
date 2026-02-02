mod payment_method_create_tests {
    use actix_web::{App, http::StatusCode, test, web};
    use auth_plus_billing::{
        config::database::get_connection,
        core::{
            dto::payment_method::{PaymentMethod, PaymentMethodInfo, PixInfo},
            repository::orm::{
                create_gateway_integration, create_user, delete_gateway_integration_by_pm,
                delete_payment_method, delete_user, read_main_gateway,
            },
        },
        presentation::http::routes::payment_method::{self, CreatePaymentMethodInputSchema},
    };
    use fake::{Fake, faker::internet::en::FreeEmail, uuid::UUIDv4};
    use httpmock::prelude::{MockServer, POST};
    use serde_json::json;
    use uuid::Uuid;

    #[actix_web::test]
    async fn should_create_payment_method() {
        let conn = get_connection().await;
        let external_id: Uuid = UUIDv4.fake();
        let user_id: Uuid = UUIDv4.fake();
        let email: String = FreeEmail().fake();
        let pix_info = PixInfo {
            key: email.clone(),
            external_id: String::from("ABCDEFG"),
        };
        let info = PaymentMethodInfo::PixInfo(pix_info);

        let server = MockServer::start();
        let mock_gateway_host = server.mock(|when, then| {
            when.method(POST).path("/v1/payment_methods");
            then.status(201)
                .header("content-type", "text/json; charset=UTF-8")
                .json_body(json!({
                    "id": "cus_123",
                    "livemode": false,
                    "type":  "Boleto",
                }));
        });

        let gateway = read_main_gateway(&conn)
            .await
            .expect("read_main_gateway: read main gateway went wrong");
        create_user(&conn, user_id, external_id)
            .await
            .expect("should_create_payment_method: user setup went wrong");
        create_gateway_integration(&conn, UUIDv4.fake(), gateway.id, user_id, UUIDv4.fake())
            .await
            .expect("create_gateway_integration: user setup went wrong");
        let payload = CreatePaymentMethodInputSchema {
            external_user_id: external_id.to_string(),
            is_default: true,
            method: String::from("pix"),
            info,
        };
        let prev_stripe_base_url = std::env::var("STRIPE_BASE_URL").ok();
        unsafe {
            std::env::set_var("STRIPE_BASE_URL", &server.base_url());
        }
        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(server.base_url()))
                .service(payment_method::create_payment_method),
        )
        .await;
        let req = test::TestRequest::post()
            .uri("/payment_method")
            .set_json(web::Json(payload))
            .to_request();
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::OK);
        let body: PaymentMethod = test::read_body_json(resp).await;
        mock_gateway_host.assert();
        delete_gateway_integration_by_pm(&conn, body.id)
            .await
            .expect("should_create_payment_method: gateway_integration remove went wrong");
        delete_payment_method(&conn, body.id)
            .await
            .expect("should_create_payment_method: payment_method remove went wrong");
        delete_user(&conn, user_id)
            .await
            .expect("should_create_payment_method: user remove went wrong");
        unsafe {
            match prev_stripe_base_url {
                Some(value) => std::env::set_var("STRIPE_BASE_URL", value),
                None => std::env::remove_var("STRIPE_BASE_URL"),
            }
        }
    }

    #[actix_web::test]
    async fn should_fail_when_user_does_not_exist() {
        let pix_info = PixInfo {
            key: String::from("any@email.com"),
            external_id: String::from("ABCDEFG"),
        };
        let info = PaymentMethodInfo::PixInfo(pix_info);
        let payload = CreatePaymentMethodInputSchema {
            external_user_id: String::from("any-hash-that-is-not-uuid"),
            is_default: true,
            method: String::from("pix"),
            info,
        };

        let app =
            test::init_service(App::new().service(payment_method::create_payment_method)).await;
        let req = test::TestRequest::post()
            .uri("/payment_method")
            .set_json(web::Json(payload))
            .to_request();
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::INTERNAL_SERVER_ERROR);
    }
}
