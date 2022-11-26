pub mod dto;
pub mod repository;
pub mod usecase;

use crate::config::database::get_connection;

pub struct InvoiceUsecase {
    pub create: usecase::invoice::invoice_create_usecase::InvoiceCreateUsecase,
    pub list: usecase::invoice::invoice_list_usecase::InvoiceListUsecase,
}

pub struct UserUsecase {
    pub create: usecase::user::user_create_usecase::UserCreateUsecase,
}

pub struct Core {
    pub invoice: InvoiceUsecase,
    pub user: UserUsecase,
}

pub async fn get_core() -> Core {
    // Let start transaction
    let conn = get_connection().await;

    // repositories
    let invoice_repository = repository::invoice_repository::InvoiceRepository::new(conn.clone());
    let user_repository = repository::user_repository::UserRepository::new(conn);

    //usecases
    let invoice_create_usecase = usecase::invoice::invoice_create_usecase::InvoiceCreateUsecase {
        reading_user: Box::new(user_repository.clone()),
        creating_invoice: Box::new(invoice_repository.clone()),
    };

    let invoice_list_usecase = usecase::invoice::invoice_list_usecase::InvoiceListUsecase {
        reading_user: Box::new(user_repository.clone()),
        reading_invoice: Box::new(invoice_repository),
    };

    let user_create_usecase = usecase::user::user_create_usecase::UserCreateUsecase {
        creating_user: Box::new(user_repository),
    };

    Core {
        invoice: InvoiceUsecase {
            create: invoice_create_usecase,
            list: invoice_list_usecase,
        },
        user: UserUsecase {
            create: user_create_usecase,
        },
    }
}
