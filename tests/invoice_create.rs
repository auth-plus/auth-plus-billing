mod invoice_create_tests {
    use actix_web::{http::StatusCode, test, web, App};
    use auth_plus_billing::{
        config::database::get_connection,
        core::{
            dto::{invoice::Invoice, invoice_item::InvoiceItem},
            repository::helpers::{create_user, delete_invoice, delete_user},
        },
        presentation::http::routes::invoice::{self, CreateInvoiceInputSchema},
    };
    use fake::{faker::lorem::en::Sentence, uuid::UUIDv4, Fake, Faker};
    use rust_decimal::Decimal;
    use uuid::Uuid;

    #[actix_web::test]
    async fn should_create_invoice() {
        let conn = get_connection().await;
        let user_id: Uuid = UUIDv4.fake();
        let external_id: Uuid = UUIDv4.fake();
        let quantity = Faker.fake::<u16>();
        let amount = Faker.fake::<f32>();
        let description: String = Sentence(3..5).fake();
        let currency = "BRL";
        let item = InvoiceItem {
            id: None,
            amount: Decimal::from_f32_retain(amount).unwrap(),
            quantity,
            description: description.clone(),
            currency: String::from(currency),
        };
        let itens = Vec::from([item]);
        let payload = CreateInvoiceInputSchema {
            external_user_id: external_id.to_string(),
            itens,
        };
        create_user(&conn, user_id, external_id)
            .await
            .expect("should_create_invoice: user setup went wrong");

        let app = test::init_service(App::new().service(invoice::create_invoice)).await;
        let req = test::TestRequest::post()
            .uri("/invoice")
            .set_json(web::Json(payload))
            .to_request();
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::OK);
        let body: Invoice = test::read_body_json(resp).await;
        delete_invoice(&conn, body.id)
            .await
            .expect("should_create_invoice: invoice remove went wrong");
        delete_user(&conn, user_id)
            .await
            .expect("should_create_invoice: user remove went wrong");
    }

    #[actix_web::test]
    async fn should_fail_when_user_does_not_exist() {
        let external_id: Uuid = UUIDv4.fake();
        let quantity = Faker.fake::<u16>();
        let amount = Faker.fake::<f32>();
        let description: String = Sentence(3..5).fake();
        let currency = "BRL";
        let item = InvoiceItem {
            id: None,
            amount: Decimal::from_f32_retain(amount).unwrap(),
            quantity,
            description: description.clone(),
            currency: String::from(currency),
        };
        let itens = Vec::from([item]);
        let payload = CreateInvoiceInputSchema {
            external_user_id: external_id.to_string(),
            itens,
        };
        let app = test::init_service(App::new().service(invoice::create_invoice)).await;
        let req = test::TestRequest::post()
            .uri("/invoice")
            .set_json(web::Json(payload))
            .to_request();
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::INTERNAL_SERVER_ERROR);
    }
}
