//! Pump.fun instruction decoder

use anyhow::Result;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Action {
    Create,
    Buy,
    Sell,
    Swap,
    Unknown,
}

impl Action {
    pub fn as_str(&self) -> &'static str {
        match self {
            Action::Create => "CREATE",
            Action::Buy => "BUY",
            Action::Sell => "SELL",
            Action::Swap => "SWAP",
            Action::Unknown => "UNKNOWN",
        }
    }
}

#[derive(Debug, Clone)]
pub struct DecodedInstruction {
    pub action: Action,
    pub mint: Option<String>,
    pub token_amount: Option<u64>,
    pub max_sol_cost: Option<u64>, // For BUY: max SOL to spend; For SELL: min SOL to receive
}

/// Pump.fun instruction discriminators (first 8 bytes of instruction data)
/// These are derived from the method name hashes in the Pump.fun program
const DISCRIMINATOR_CREATE: [u8; 8] = [0x18, 0x1e, 0xc8, 0x28, 0x05, 0x1c, 0x07, 0x77];
const DISCRIMINATOR_BUY: [u8; 8] = [0x66, 0x06, 0x3d, 0x12, 0x01, 0xda, 0xeb, 0xea];
const DISCRIMINATOR_SELL: [u8; 8] = [0x33, 0xe6, 0x85, 0xa4, 0x01, 0x7f, 0x83, 0xad];

/// Decode a Pump.fun instruction by discriminator
pub fn decode_instruction(data: &[u8], accounts: &[String]) -> Result<DecodedInstruction> {
    if data.len() < 8 {
        return Ok(DecodedInstruction {
            action: Action::Unknown,
            mint: None,
            token_amount: None,
            max_sol_cost: None,
        });
    }

    let discriminator = &data[0..8];
    
    let action = if discriminator == DISCRIMINATOR_CREATE {
        Action::Create
    } else if discriminator == DISCRIMINATOR_BUY {
        Action::Buy
    } else if discriminator == DISCRIMINATOR_SELL {
        Action::Sell
    } else {
        Action::Unknown
    };

    // Extract mint from accounts based on instruction type
    // Based on official Pump.fun IDL:
    // - CREATE: mint is at index 0
    // - BUY: mint is at index 2
    // - SELL: mint is at index 2
    let mint = match action {
        Action::Create => accounts.get(0).cloned(),
        Action::Buy | Action::Sell => accounts.get(2).cloned(),
        _ => None,
    };

    // Parse instruction data:
    // For BUY/SELL: [discriminator (8)] + [token_amount (8)] + [max_sol_cost (8)]
    let token_amount = if data.len() >= 16 {
        Some(u64::from_le_bytes(
            data[8..16].try_into().unwrap_or([0u8; 8])
        ))
    } else {
        None
    };

    let max_sol_cost = if data.len() >= 24 {
        Some(u64::from_le_bytes(
            data[16..24].try_into().unwrap_or([0u8; 8])
        ))
    } else {
        None
    };

    Ok(DecodedInstruction {
        action,
        mint,
        token_amount,
        max_sol_cost,
    })
}
