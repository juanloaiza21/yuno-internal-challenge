use serde::{Deserialize, Serialize};

/// Supported currencies in the FashionForward marketplace.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum Currency {
    BRL,
    MXN,
    COP,
}

impl std::fmt::Display for Currency {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Currency::BRL => write!(f, "BRL"),
            Currency::MXN => write!(f, "MXN"),
            Currency::COP => write!(f, "COP"),
        }
    }
}

/// Countries where FashionForward operates.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum Country {
    Brazil,
    Mexico,
    Colombia,
}

impl std::fmt::Display for Country {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Country::Brazil => write!(f, "Brazil"),
            Country::Mexico => write!(f, "Mexico"),
            Country::Colombia => write!(f, "Colombia"),
        }
    }
}

/// A payment transaction from a FashionForward customer.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Transaction {
    /// Unique transaction identifier.
    pub id: String,
    /// Transaction amount in the local currency.
    pub amount: f64,
    /// Currency of the transaction.
    pub currency: Currency,
    /// Country where the transaction originates.
    pub country: Country,
    /// First 6 digits of the card (Bank Identification Number).
    pub card_bin: String,
    /// Last 4 digits of the card.
    pub card_last4: String,
    /// Unique customer identifier.
    pub customer_id: String,
    /// ISO 8601 timestamp of the transaction.
    pub timestamp: String,
}
