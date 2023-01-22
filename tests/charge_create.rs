mod charge_create_tests {
    use actix_web::{http::StatusCode, test, web, App};
    use auth_plus_billing::{
        config::database::get_connection,
        core::{
            dto::{
                charge::Charge,
                invoice::InvoiceStatus,
                payment_method::{Method, PaymentMethodInfo, PixInfo},
            },
            repository::orm::{
                create_gateway, create_invoice, create_payment_method, create_user, delete_charge,
                delete_gateway, delete_invoice, delete_payment_method, delete_user,
            },
        },
        presentation::http::routes::charge::{self, CreateChargeInputSchema},
    };
    use fake::{faker::lorem::en::Word, uuid::UUIDv4, Fake};
    use uuid::Uuid;

    #[actix_web::test]
    async fn should_create_charge() {
        let conn = get_connection().await;
        let external_id: Uuid = UUIDv4.fake();
        let user_id: Uuid = UUIDv4.fake();
        let invoice_id: Uuid = UUIDv4.fake();
        let gateway_id: Uuid = UUIDv4.fake();
        let gateway_name: String = Word().fake();
        let payment_method_id: Uuid = UUIDv4.fake();
        let pix_info = PixInfo {
            key: String::from("any@email.com"),
            external_id: String::from("ABCDEFG"),
        };
        let info = PaymentMethodInfo::PixInfo(pix_info);

        create_user(&conn, user_id, external_id)
            .await
            .expect("should_create_charge: user setup went wrong");
        create_invoice(&conn, invoice_id, user_id, InvoiceStatus::Draft)
            .await
            .expect("should_create_charge: invoice setup went wrong");
        create_gateway(&conn, gateway_id, &gateway_name, 1)
            .await
            .expect("should_create_charge: gateway setup went wrong");
        create_payment_method(&conn, payment_method_id, user_id, true, Method::Pix, info)
            .await
            .expect("should_create_charge: payment_method setup went wrong");

        let payload = CreateChargeInputSchema {
            invoice_id: invoice_id.to_string(),
        };
        let app = test::init_service(App::new().service(charge::create_charge)).await;
        let req = test::TestRequest::post()
            .uri("/charge")
            .set_json(web::Json(payload))
            .to_request();
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::OK);
        let body: Charge = test::read_body_json(resp).await;

        delete_charge(&conn, body.id)
            .await
            .expect("should_create_charge: gateway remove went wrong");
        delete_payment_method(&conn, payment_method_id)
            .await
            .expect("should_create_charge: gateway remove went wrong");
        delete_gateway(&conn, gateway_id)
            .await
            .expect("should_create_charge: gateway remove went wrong");
        delete_invoice(&conn, invoice_id)
            .await
            .expect("should_create_charge: invoice remove went wrong");
        delete_user(&conn, user_id)
            .await
            .expect("should_create_charge: user remove went wrong");
    }

    #[actix_web::test]
    async fn should_fail_when_user_does_not_exist() {
        let payload = CreateChargeInputSchema {
            invoice_id: String::from("any-hash-that-is-not-uuid"),
        };

        let app = test::init_service(App::new().service(charge::create_charge)).await;
        let req = test::TestRequest::post()
            .uri("/charge")
            .set_json(web::Json(payload))
            .to_request();
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::INTERNAL_SERVER_ERROR);
    }
}
