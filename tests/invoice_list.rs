mod invoice_list_tests {
    use actix_web::{http::StatusCode, test, App};
    use auth_plus_billing::{
        config::database::get_connection,
        core::{
            dto::invoice::InvoiceStatus,
            repository::orm::{create_invoice, create_user, delete_invoice, delete_user},
        },
        presentation::http::routes::invoice::{self},
    };
    use fake::{uuid::UUIDv4, Fake};
    use uuid::Uuid;

    #[actix_web::test]
    async fn should_list_invoices() {
        let conn = get_connection().await;
        let user_id: Uuid = UUIDv4.fake();
        let invoice_id: Uuid = UUIDv4.fake();
        let external_id: Uuid = UUIDv4.fake();
        create_user(&conn, user_id, external_id)
            .await
            .expect("should_list_invoices: user setup went wrong");
        create_invoice(&conn, invoice_id, user_id, InvoiceStatus::Draft)
            .await
            .expect("should_list_invoices: invoice setup went wrong");
        let app = test::init_service(App::new().service(invoice::get_invoice)).await;
        let req = test::TestRequest::get()
            .uri(&format!("/invoice/{}", external_id.to_string()))
            .to_request();
        let resp = test::call_service(&app, req).await;
        /*
        TODO: fix test to assert response payload as below, but still be able to assert status
        let result = test::read_body(resp).await;
        println!("{:?}", result[0]);
        */
        assert_eq!(resp.status(), StatusCode::OK);
        delete_invoice(&conn, invoice_id)
            .await
            .expect("should_list_invoices: invoice remove went wrong");
        delete_user(&conn, user_id)
            .await
            .expect("should_list_invoices: user remove went wrong");
    }

    #[actix_web::test]
    async fn should_return_empty_list() {
        let conn = get_connection().await;
        let user_id: Uuid = UUIDv4.fake();
        let external_id: Uuid = UUIDv4.fake();
        create_user(&conn, user_id, external_id)
            .await
            .expect("should_return_empty_list: user setup went wrong");
        let app = test::init_service(App::new().service(invoice::get_invoice)).await;
        let req = test::TestRequest::get()
            .uri(&format!("/invoice/{}", external_id.to_string()))
            .to_request();
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::OK);
        delete_user(&conn, user_id)
            .await
            .expect("should_return_empty_list: user remove went wrong");
    }

    #[actix_web::test]
    async fn should_fail_when_user_does_not_exist() {
        let external_id: Uuid = UUIDv4.fake();
        let app = test::init_service(App::new().service(invoice::get_invoice)).await;
        let req = test::TestRequest::get()
            .uri(&format!("/invoice/{}", external_id.to_string()))
            .to_request();
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::INTERNAL_SERVER_ERROR);
    }
}
