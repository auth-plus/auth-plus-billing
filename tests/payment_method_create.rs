mod payment_method_create_tests {
    use actix_web::{http::StatusCode, test, web, App};
    use auth_plus_billing::{
        config::database::get_connection,
        core::{
            dto::payment_method::{PaymentMethod, PaymentMethodInfo, PixInfo},
            repository::orm::{
                create_gateway, create_user, delete_gateway, delete_gateway_integration_by_pm,
                delete_payment_method, delete_user,
            },
        },
        presentation::http::routes::payment_method::{self, CreatePaymentMethodInputSchema},
    };
    use fake::{
        faker::{internet::en::FreeEmail, lorem::en::Word},
        uuid::UUIDv4,
        Fake,
    };
    use uuid::Uuid;

    #[actix_web::test]
    async fn should_create_payment_method() {
        let conn = get_connection().await;
        let external_id: Uuid = UUIDv4.fake();
        let user_id: Uuid = UUIDv4.fake();
        let gateway_id: Uuid = UUIDv4.fake();
        let gateway_name: String = Word().fake();
        let email: String = FreeEmail().fake();
        let pix_info = PixInfo {
            key: email,
            external_id: String::from("ABCDEFG"),
        };
        let info = PaymentMethodInfo::PixInfo(pix_info);

        create_user(&conn, user_id, external_id)
            .await
            .expect("should_create_payment_method: user setup went wrong");
        create_gateway(&conn, gateway_id, &gateway_name, 1)
            .await
            .expect("should_create_payment_method: gateway setup went wrong");

        let payload = CreatePaymentMethodInputSchema {
            external_user_id: external_id.to_string(),
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
        assert_eq!(resp.status(), StatusCode::OK);
        let body: PaymentMethod = test::read_body_json(resp).await;

        delete_gateway_integration_by_pm(&conn, body.id)
            .await
            .expect("should_create_payment_method: gateway_integration remove went wrong");
        delete_payment_method(&conn, body.id)
            .await
            .expect("should_create_payment_method: payment_method remove went wrong");
        delete_gateway(&conn, gateway_id)
            .await
            .expect("should_create_payment_method: gateway remove went wrong");
        delete_user(&conn, user_id)
            .await
            .expect("should_create_payment_method: user remove went wrong");
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
