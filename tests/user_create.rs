mod user_create_tests {
    use actix_web::{http::StatusCode, test, web, App};
    use auth_plus_billing::{
        config::database::get_connection,
        core::{dto::user::User, repository::helpers::delete_user},
        presentation::http::routes::user::{self, CreateUserInputSchema},
    };
    use fake::{uuid::UUIDv4, Fake};
    use uuid::Uuid;

    #[actix_web::test]
    async fn should_create_user() {
        let conn = get_connection().await;
        let external_id: Uuid = UUIDv4.fake();
        let payload = CreateUserInputSchema {
            external_id: external_id.to_string(),
        };

        let app = test::init_service(App::new().service(user::create_user)).await;
        let req = test::TestRequest::post()
            .uri("/user")
            .set_json(web::Json(payload))
            .to_request();
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::OK);
        let body: User = test::read_body_json(resp).await;
        delete_user(&conn, body.id)
            .await
            .expect("should_list_invoices: user remove went wrong");
    }

    #[actix_web::test]
    async fn should_fail_when_user_does_not_exist() {
        let payload = CreateUserInputSchema {
            external_id: String::from("any-hash-that-is-not-uuid"),
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
