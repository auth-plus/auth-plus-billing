mod tests {
    use actix_web::{test, App};
    use auth_plus_billing::{
        config::database::get_connection, presentation::http::routes::invoice,
    };
    use fake::{uuid::UUIDv4, Fake};
    use uuid::Uuid;

    #[actix_web::test]
    async fn test_index_post() {
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
        assert!(resp.status().is_success());
    }
}
