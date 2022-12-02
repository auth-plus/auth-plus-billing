mod invoice_list_tests {
    use actix_web::{http::StatusCode, test, App};
    use auth_plus_billing::{
        config::database::get_connection, presentation::http::routes::invoice,
    };
    use fake::{uuid::UUIDv4, Fake};
    use uuid::Uuid;

    #[actix_web::test]
    async fn should_list_invoices() {
        let conn = get_connection().await;
        let user_id: Uuid = UUIDv4.fake();
        let external_id: Uuid = UUIDv4.fake();
        let q_user = format!(
            "INSERT INTO \"user\" (id, external_id) VALUES ('{}', '{}');",
            user_id.to_string(),
            external_id.to_string()
        );
        sqlx::query(&q_user)
            .execute(&conn)
            .await
            .expect("should_list_invoices: user setup went wrong");
        let q_invoice = format!(
            "INSERT INTO invoice (user_id, status) VALUES ('{}', 'pending');",
            user_id.to_string(),
        );
        sqlx::query(&q_invoice)
            .execute(&conn)
            .await
            .expect("should_list_invoices: invoice setup went wrong");
        let app = test::init_service(App::new().service(invoice::get_invoice)).await;
        let req = test::TestRequest::get()
            .uri(&format!("/invoice/{}", external_id.to_string()))
            .to_request();
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::OK);
    }

    #[actix_web::test]
    async fn should_return_empty_list() {
        let conn = get_connection().await;
        let user_id: Uuid = UUIDv4.fake();
        let external_id: Uuid = UUIDv4.fake();
        let q_user = format!(
            "INSERT INTO \"user\" (id, external_id) VALUES ('{}', '{}');",
            user_id.to_string(),
            external_id.to_string()
        );
        sqlx::query(&q_user)
            .execute(&conn)
            .await
            .expect("should_return_empty_list: user setup went wrong");
        let app = test::init_service(App::new().service(invoice::get_invoice)).await;
        let req = test::TestRequest::get()
            .uri(&format!("/invoice/{}", external_id.to_string()))
            .to_request();
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::OK);
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