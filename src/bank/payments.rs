use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use time::PrimitiveDateTime;
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq, Eq, Copy, Serialize, Deserialize, sqlx::Type)]
#[serde(rename_all = "snake_case")]
pub enum Status {
    /// The payment is being processed, and its state is unknown.
    Processing,
    /// The payment was approved by the bank.
    Approved,
    /// The payment was declined by the bank (e.g. insufficient funds).
    Declined,
    /// The payment was unable to complete (e.g. banking system crashed).
    Failed,
}

// Struct representing a payment.
//
// Once a payment has been persisted with an "approved" state, the merchant is guaranteed to
// receive money from the bank: they can therefore release the purchased goods to the customer.
#[derive(Debug, Clone, PartialEq, Eq, sqlx::FromRow)]
pub struct Payment {
    pub id: Uuid,
    pub amount: i32,
    pub card_number: String,
    pub status: Status,
    pub inserted_at: PrimitiveDateTime,
    pub updated_at: PrimitiveDateTime,
}

pub async fn insert(
    pool: &PgPool,
    amount: i32,
    card_number: String,
    status: Status,
) -> Result<Uuid, sqlx::Error> {
    sqlx::query!(
        r#"
            INSERT INTO payments ( id, amount, card_number, status, inserted_at, updated_at )
            VALUES ( $1, $2, $3, $4, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP )
            RETURNING id
        "#,
        Uuid::new_v4(),
        amount,
        card_number,
        status as Status
    )
    .fetch_one(pool)
    .await
    .map(|record| record.id)
}

pub async fn get(pool: &PgPool, id: Uuid) -> Result<Payment, sqlx::Error> {
    sqlx::query_as!(
            Payment,
            r#"
                SELECT id, amount, card_number, inserted_at, updated_at, status as "status: _"  FROM payments
                WHERE id = $1
            "#,
            id
        )
        .fetch_one(pool)
        .await
}

pub async fn get_by_card_number(
    pool: &PgPool,
    card_number: &str,
) -> Result<Option<Payment>, sqlx::Error> {
    sqlx::query_as!(
            Payment,
            r#"
                SELECT id, amount, card_number, inserted_at, updated_at, status as "status: _"  FROM payments
                WHERE card_number = $1
            "#,
            card_number
        )
        .fetch_optional(pool)
        .await
}

#[cfg(test)]
pub mod tests {

    use super::*;
    use crate::bank::payment_instruments::Card;

    pub const PAYMENT_AMOUNT: i32 = 123;
    pub const PAYMENT_STATUS: Status = Status::Approved;

    impl Payment {
        pub async fn new_test(pool: &PgPool) -> Result<Payment, sqlx::Error> {
            let card = Card::new_test();

            let id = insert(pool, PAYMENT_AMOUNT, card.into(), PAYMENT_STATUS).await?;

            get(pool, id).await
        }
    }

    #[tokio::test]
    async fn test_payment() {
        let pool = crate::pg_pool()
            .await
            .expect("failed to connect to postgres");

        let payment = Payment::new_test(&pool)
            .await
            .expect("failed to create payment");

        assert_eq!(payment.amount, PAYMENT_AMOUNT);
        assert_eq!(payment.status, PAYMENT_STATUS);
    }
}
