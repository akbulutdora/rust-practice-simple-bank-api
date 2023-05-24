use std::num::ParseIntError;

const CARD_NUMBER_LENGTH: usize = 15;
const ACCOUNT_PREFIX_LENGTH: usize = 2;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CardError {
    InvalidLength,
    ParseError(ParseIntError),
}

/// Represents the account number associated with a virtual credit card.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct AccountNumber(String);

impl AccountNumber {
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// Represents a virtual credit card used for payments.
///
/// Card numbers have 15 digits, and the linked account number can be derived
/// from the card number.
///
/// Each time it is used a different card number is generated and provided
/// to merchants for payment.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Card {
    card_number: String,
    account_number: AccountNumber,
}

impl TryFrom<String> for Card {
    type Error = CardError;

    fn try_from(card_number: String) -> Result<Self, Self::Error> {
        if card_number.len() != CARD_NUMBER_LENGTH {
            Err(CardError::InvalidLength)
        } else {
            card_number.parse::<u64>().map_err(CardError::ParseError)?;
            let (account_number_str, _) = card_number.split_at(ACCOUNT_PREFIX_LENGTH);
            let account_number = AccountNumber(account_number_str.to_string());

            Ok(Self {
                card_number,
                account_number,
            })
        }
    }
}

impl From<Card> for String {
    fn from(card: Card) -> Self {
        card.card_number
    }
}

impl Card {
    /// Returns a reference to the account number associated with the given card.
    pub fn account_number(&self) -> &AccountNumber {
        &self.account_number
    }

    /// Returns a reference to the string representation of this card number.
    pub fn card_number(&self) -> &str {
        &self.card_number
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;

    impl Card {
        pub fn new_test() -> Self {
            use rand::Rng;

            let account_number = format!(
                "{:0>ACCOUNT_PREFIX_LENGTH$}",
                rand::thread_rng().gen_range(1..10u64.pow(ACCOUNT_PREFIX_LENGTH as u32))
            );

            assert_eq!(account_number.len(), ACCOUNT_PREFIX_LENGTH);

            Self::new_with_account_number(&account_number)
        }

        pub fn new_with_account_number(account_number: &str) -> Self {
            use rand::Rng;

            assert_eq!(account_number.len(), ACCOUNT_PREFIX_LENGTH);

            let suffix_len = CARD_NUMBER_LENGTH - ACCOUNT_PREFIX_LENGTH;

            let card_number = format!(
                "{account_number}{:0>suffix_len$}",
                rand::thread_rng().gen_range(0..10u64.pow(suffix_len as u32))
            );

            assert_eq!(card_number.len(), CARD_NUMBER_LENGTH);

            Self::try_from(card_number).expect("failed to parse card_number")
        }
    }
}
