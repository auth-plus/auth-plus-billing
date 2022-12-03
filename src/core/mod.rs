pub mod dto;
pub mod repository;
pub mod usecase;

use crate::config::database::get_connection;

pub struct ChargeUsecase {
    pub create: usecase::charge::charge_create_usecase::ChargeCreateUsecase,
}
pub struct InvoiceUsecase {
    pub create: usecase::invoice::invoice_create_usecase::InvoiceCreateUsecase,
    pub list: usecase::invoice::invoice_list_usecase::InvoiceListUsecase,
    pub update: usecase::invoice::invoice_update_usecase::InvoiceUpdateUsecase,
}
pub struct PaymentMethodUsecase {
    pub create: usecase::payment_method::payment_method_create_usecase::PaymentMethodCreateUsecase,
}
pub struct UserUsecase {
    pub create: usecase::user::user_create_usecase::UserCreateUsecase,
}
pub struct Core {
    pub charge: ChargeUsecase,
    pub invoice: InvoiceUsecase,
    pub payment_method: PaymentMethodUsecase,
    pub user: UserUsecase,
}

pub async fn get_core() -> Core {
    // Let start transaction
    let conn = get_connection().await;

    // repositories
    let charge_repository = repository::charge_repository::ChargeRepository::new(conn.clone());
    let gateway_repository = repository::gateway_repository::GatewayRepository::new(conn.clone());
    let invoice_repository = repository::invoice_repository::InvoiceRepository::new(conn.clone());
    let user_repository = repository::user_repository::UserRepository::new(conn.clone());
    let payment_method_repository =
        repository::payment_method_repository::PaymentMethodRepository::new(conn.clone());

    //usecases
    let charge_create_usecase = usecase::charge::charge_create_usecase::ChargeCreateUsecase {
        reading_invoice: Box::new(invoice_repository.clone()),
        reading_payment_method: Box::new(payment_method_repository.clone()),
        creating_charge: Box::new(charge_repository.clone()),
    };
    let invoice_create_usecase = usecase::invoice::invoice_create_usecase::InvoiceCreateUsecase {
        reading_user: Box::new(user_repository.clone()),
        creating_invoice: Box::new(invoice_repository.clone()),
    };
    let invoice_list_usecase = usecase::invoice::invoice_list_usecase::InvoiceListUsecase {
        reading_user: Box::new(user_repository.clone()),
        reading_invoice: Box::new(invoice_repository.clone()),
    };
    let invoice_update_usecase = usecase::invoice::invoice_update_usecase::InvoiceUpdateUsecase {
        reading_invoice: Box::new(invoice_repository.clone()),
        updating_invoice: Box::new(invoice_repository.clone()),
    };
    let payment_method_create_usecase =
        usecase::payment_method::payment_method_create_usecase::PaymentMethodCreateUsecase {
            reading_user: Box::new(user_repository.clone()),
            reading_gateway: Box::new(gateway_repository.clone()),
            creating_payment_method: Box::new(payment_method_repository.clone()),
        };
    let user_create_usecase = usecase::user::user_create_usecase::UserCreateUsecase {
        creating_user: Box::new(user_repository),
    };

    Core {
        charge: ChargeUsecase {
            create: charge_create_usecase,
        },
        invoice: InvoiceUsecase {
            create: invoice_create_usecase,
            list: invoice_list_usecase,
            update: invoice_update_usecase,
        },
        payment_method: PaymentMethodUsecase {
            create: payment_method_create_usecase,
        },
        user: UserUsecase {
            create: user_create_usecase,
        },
    }
}
